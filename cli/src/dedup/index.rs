//! SQLite inverted fingerprint index (ADR-005): `files` +
//! `fingerprints` + the schema-v4 graph cache, WAL + busy_timeout per
//! ADR-003. Incremental invalidation is content-hash gated per file:
//! unchanged bytes touch nothing; a change deletes and reinserts only
//! that file's rows — fingerprints and its graph phase-1 rows in the
//! same transaction. The M2 daemon is the sole writer; the batch CLI
//! uses the same code single-threaded. Schema lifecycle lives in
//! dedup/schema.rs; graph DDL + phase 2 in graph/store.rs.

use super::{Params, schema, tokens, winnow};
use crate::graph::store;
use crate::scan::lang::Lang;
use anyhow::{Context, Result};
use rusqlite::Connection;
use std::collections::BTreeSet;
use std::path::Path;

/// One fingerprint occurrence, joined back to its file. `start_tok`
/// is the k-gram's token index — the anchor for extension verify.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Instance {
    pub hash: u64,
    pub file: String,
    pub start_tok: usize,
    pub start_line: usize,
    pub end_line: usize,
}

impl Instance {
    /// The one row mapper both instance queries share.
    fn from_row(r: &rusqlite::Row<'_>) -> rusqlite::Result<Self> {
        Ok(Instance {
            hash: r.get::<_, i64>(0)? as u64,
            file: r.get(1)?,
            start_tok: r.get::<_, i64>(2)? as usize,
            start_line: r.get::<_, i64>(3)? as usize,
            end_line: r.get::<_, i64>(4)? as usize,
        })
    }
}

pub struct Index {
    conn: Connection,
}

/// Switch to WAL, tolerating the racing-creators deadlock: two fresh
/// connections both upgrading SHARED → EXCLUSIVE for the mode switch
/// make SQLite hand one an IMMEDIATE busy — deadlock detection
/// structurally bypasses the busy handler, so the timeout never
/// applies. A bounded re-check loop is the prescribed handling (the
/// pragma may also report the OLD mode instead of erroring): the
/// winner's persistent switch satisfies the loser's re-read.
fn ensure_wal(conn: &Connection) -> Result<()> {
    for _ in 0..100 {
        match conn.query_row("PRAGMA journal_mode = WAL", [], |r| r.get::<_, String>(0)) {
            Ok(mode) if mode.eq_ignore_ascii_case("wal") => return Ok(()),
            Ok(_) => {}
            Err(e) if is_busy(&e) => {}
            Err(e) => return Err(e.into()),
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    anyhow::bail!("journal_mode=WAL still contended after 100 retries")
}

fn is_busy(e: &rusqlite::Error) -> bool {
    matches!(e, rusqlite::Error::SqliteFailure(f, _)
        if f.code == rusqlite::ErrorCode::DatabaseBusy)
}

impl Index {
    /// Open (or wipe-and-recreate) the index. The cache key is
    /// schema version + winnowing params + tokenizer revision +
    /// graph revision (attack-review D2: params/tokenizer changes
    /// silently reused stale fingerprints for unchanged files).
    pub fn open(db_path: &Path, p: Params) -> Result<Self> {
        if let Some(dir) = db_path.parent() {
            std::fs::create_dir_all(dir).with_context(|| format!("create {}", dir.display()))?;
        }
        let conn = Connection::open(db_path)
            .with_context(|| format!("open index {}", db_path.display()))?;
        // busy_timeout before any lock-taking statement — the default
        // of 0 turns every contention into an instant error
        conn.pragma_update(None, "busy_timeout", 5000)?;
        ensure_wal(&conn)?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        schema::ensure_cache_key(&conn, p)?;
        Ok(Self { conn })
    }

    /// Refresh one file; returns false when the stored content hash
    /// already matches (nothing touched — the incremental fast path).
    /// (Concurrent-writer hardening is out of contract: the module
    /// header's "daemon is the sole writer" stands; only OPEN races.)
    pub fn refresh_file(&mut self, rel: &str, src: &[u8], lang: Lang, p: Params) -> Result<bool> {
        let chash = tokens::fnv1a(src) as i64;
        let stored: Option<i64> = self
            .conn
            .query_row(
                "SELECT content_hash FROM files WHERE path = ?1",
                (rel,),
                |r| r.get(0),
            )
            .map(Some)
            .or_else(schema::ignore_no_rows)?;
        if stored == Some(chash) {
            return Ok(false);
        }
        // Markdown (no grammar) enters `files` for the graph cache
        // with zero fingerprint rows: all_instances joins from the
        // fingerprints side, so the dedup ratchet is structurally
        // untouched (design §3)
        let toks = match lang.grammar() {
            Some(_) => tokens::stream(src, lang)?,
            None => Vec::new(),
        };
        let hashes: Vec<u64> = toks.iter().map(|t| t.hash).collect();
        let fps = winnow::fingerprints(&hashes, p);
        let tx = self.conn.transaction()?;
        tx.execute(
            "INSERT INTO files (path, content_hash, token_count, has_tokens)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(path) DO UPDATE SET content_hash = ?2, token_count = ?3, has_tokens = ?4",
            (
                rel,
                chash,
                toks.len() as i64,
                i64::from(lang.grammar().is_some()),
            ),
        )?;
        let id: i64 = tx.query_row("SELECT id FROM files WHERE path = ?1", (rel,), |r| r.get(0))?;
        tx.execute("DELETE FROM fingerprints WHERE file_id = ?1", (id,))?;
        insert_fps(&tx, id, &fps, &toks, p)?;
        store::refresh_graph(&tx, id, &String::from_utf8_lossy(src), lang)?;
        tx.commit()?;
        Ok(true)
    }

    /// Drop rows for files no longer on disk; returns removed count.
    /// One transaction — per-row autocommit meant one fsync per file.
    pub fn remove_missing(&mut self, live: &BTreeSet<String>) -> Result<usize> {
        let tx = self.conn.transaction()?;
        let paths: Vec<(i64, String)> = tx
            .prepare("SELECT id, path FROM files")?
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?
            .collect::<rusqlite::Result<_>>()?;
        let mut removed = 0;
        for (id, path) in paths {
            if !live.contains(&path) {
                tx.execute("DELETE FROM files WHERE id = ?1", (id,))?;
                removed += 1;
            }
        }
        tx.commit()?;
        Ok(removed)
    }

    /// Indexed file count (SessionStart health line). Since v4 this
    /// includes zero-fingerprint Markdown rows — they ARE indexed.
    pub fn file_count(&self) -> Result<i64> {
        Ok(self
            .conn
            .query_row("SELECT COUNT(*) FROM files", [], |r| r.get(0))?)
    }

    /// Phase-2 gate (design §3): edges depend on the whole file set
    /// plus config bytes, never on one file. The resolver callback
    /// is the wire.rs ladder bridge since 2f.
    pub fn ensure_edges_resolved(
        &mut self,
        key: i64,
        resolve: impl FnMut(&store::CachedSite) -> Vec<store::EdgeRow>,
    ) -> Result<bool> {
        store::ensure_resolved(&mut self.conn, key, resolve)
    }

    /// Phase-1.5 per-file edge refresh (store.rs contract): only for
    /// refreshed files a phase-2 sweep did NOT already cover.
    pub fn resolve_refreshed(
        &mut self,
        dirty: &BTreeSet<String>,
        resolve: impl FnMut(&store::CachedSite) -> Vec<store::EdgeRow>,
    ) -> Result<()> {
        store::resolve_refreshed(&mut self.conn, dirty, resolve)
    }

    /// Occurrences of the given hashes only (probe hot path) —
    /// chunked to stay under SQLite's bind-parameter limit.
    pub fn instances_for_hashes(&self, hashes: &[u64]) -> Result<Vec<Instance>> {
        let mut rows: Vec<Instance> = Vec::new();
        for chunk in hashes.chunks(500) {
            let marks = vec!["?"; chunk.len()].join(",");
            let sql = format!(
                "SELECT f.hash, fl.path, f.start_tok, f.start_line, f.end_line
                 FROM fingerprints f JOIN files fl ON fl.id = f.file_id
                 WHERE f.hash IN ({marks})"
            );
            let mut stmt = self.conn.prepare(&sql)?;
            let params = rusqlite::params_from_iter(chunk.iter().map(|h| *h as i64));
            let found = stmt
                .query_map(params, Instance::from_row)?
                .collect::<rusqlite::Result<Vec<_>>>()?;
            rows.extend(found);
        }
        rows.sort();
        Ok(rows)
    }

    /// Every fingerprint occurrence, deterministically ordered.
    pub fn all_instances(&self) -> Result<Vec<Instance>> {
        let mut rows: Vec<Instance> = self
            .conn
            .prepare(
                "SELECT f.hash, fl.path, f.start_tok, f.start_line, f.end_line
                 FROM fingerprints f JOIN files fl ON fl.id = f.file_id",
            )?
            .query_map([], Instance::from_row)?
            .collect::<rusqlite::Result<_>>()?;
        rows.sort();
        Ok(rows)
    }

    /// The raw connection, for the graph domain's read surface
    /// (graph/load.rs) — reads only; every writer stays in this
    /// module or graph/store.rs.
    pub(crate) fn raw(&self) -> &rusqlite::Connection {
        &self.conn
    }
}

fn insert_fps(
    tx: &rusqlite::Transaction<'_>,
    file_id: i64,
    fps: &[winnow::Fingerprint],
    toks: &[tokens::Token],
    p: Params,
) -> Result<()> {
    let mut stmt = tx.prepare(
        "INSERT INTO fingerprints (hash, file_id, start_tok, start_line, end_line)
         VALUES (?1, ?2, ?3, ?4, ?5)",
    )?;
    for f in fps {
        let start = toks[f.start].start_line;
        let end = toks[f.start + p.kgram - 1].end_line;
        stmt.execute((
            f.hash as i64,
            file_id,
            f.start as i64,
            start as i64,
            end as i64,
        ))?;
    }
    Ok(())
}
