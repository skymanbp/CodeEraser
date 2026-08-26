//! SQLite inverted fingerprint index (ADR-005): `files` +
//! `fingerprints` + the schema-v4 graph cache, WAL + busy_timeout per
//! ADR-003 as amended (plan v1.7): the index is a CONVERGENT
//! multi-writer cache. Incremental invalidation is content-hash gated
//! per file: unchanged bytes touch nothing; a change deletes and
//! reinserts only that file's rows — fingerprints and its graph
//! phase-1 rows in ONE transaction. Convergence rests on TWO legs,
//! named precisely (clearance review): refresh_file's gate check runs
//! before its DEFERRED transaction, so two writers may both refresh —
//! and converge because delete-then-reinsert of IDENTICAL bytes is
//! idempotent; remove_missing acts on a read it takes INSIDE an
//! IMMEDIATE lock (a DEFERRED read-then-upgrade dies BUSY_SNAPSHOT
//! under WAL — busy_timeout does not cover snapshot invalidation).
//! Acceptance: the concurrent_writers battery. The daemon exists for
//! performance, not correctness. Schema lifecycle lives in
//! dedup/schema.rs; graph DDL + phase 2 in graph/store.rs.

use super::{Params, schema, tokens, winnow};
use crate::graph::store;
use crate::scan::lang::Lang;
use anyhow::{Context, Result};
use rusqlite::{Connection, TransactionBehavior};
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
    /// The wipe generation this handle opened under (None on a
    /// pre-epoch database). Every durable claim about "this index"
    /// is only true while the generation still holds — see
    /// schema::mark_full_build.
    opened_at: Option<i64>,
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
        let opened_at = schema::epoch(&conn)?;
        Ok(Self { conn, opened_at })
    }

    /// Refresh one file; returns false when the stored content hash
    /// already matches (nothing touched — the incremental fast path).
    /// Concurrent-writer safe by idempotence (v1.7): the delete +
    /// reinsert rides one transaction keyed by content, so two
    /// writers of the same bytes commit the same rows in either
    /// order — the convergence leg the battery pins.
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
        let text = String::from_utf8_lossy(src);
        store::refresh_graph(&tx, id, &text, lang)?;
        // the honest FOURTH per-file parse (design F11) — unitsig
        // rows ride the same content-hash-gated transaction
        super::unitcache::refresh_units(&tx, id, &text, lang)?;
        // and the FIFTH: docdup's own additive tree (RM7 — sharing
        // the tokenizer's would couple TOKENIZER_REV to comments)
        crate::docdup::refresh_segments(&tx, id, &text, lang)?;
        tx.commit()?;
        Ok(true)
    }

    /// Drop rows for files no longer on disk; returns removed count.
    /// One IMMEDIATE transaction (clearance review HIGH): the SELECT
    /// this acts on sits inside the write lock — the DEFERRED
    /// read-then-upgrade form hit SQLITE_BUSY_SNAPSHOT under WAL
    /// whenever a concurrent writer landed between them, and the busy
    /// handler is not consulted for that case. Also one transaction,
    /// not per-row autocommit (one fsync per file).
    /// `seen` bounds the deletion to what THIS run's walk could have
    /// seen: `live` is a snapshot taken by a filesystem walk that
    /// finished before this lock was taken, so a file created after
    /// the walk — indexed and committed by a concurrent writer — is
    /// absent from `live` through no fault of its own. Deleting it
    /// cascaded that writer's fingerprints, sites and edges, made its
    /// report silently miss the file, and the next `ce check` then
    /// reddened on a "removed" member. Only rows whose path was
    /// walkable when this run started are this run's to reap.
    pub fn remove_missing(
        &mut self,
        live: &BTreeSet<String>,
        seen: &BTreeSet<String>,
    ) -> Result<usize> {
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let paths: Vec<(i64, String)> = tx
            .prepare("SELECT id, path FROM files")?
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?
            .collect::<rusqlite::Result<_>>()?;
        let mut removed = 0;
        for (id, path) in paths {
            if !live.contains(&path) && seen.contains(&path) {
                tx.execute("DELETE FROM files WHERE id = ?1", (id,))?;
                removed += 1;
            }
        }
        tx.commit()?;
        Ok(removed)
    }

    /// Every indexed path, read BEFORE a walk starts — the "what this
    /// run may reap" snapshot remove_missing is bounded by.
    pub fn indexed_paths(&self) -> Result<BTreeSet<String>> {
        let mut stmt = self.conn.prepare("SELECT path FROM files")?;
        let rows = stmt.query_map([], |r| r.get::<_, String>(0))?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    /// Indexed file count; the health line does NOT read it (it goes
    /// through index::peek without opening) — dedup_index's contract does.
    pub fn file_count(&self) -> Result<i64> {
        Ok(self
            .conn
            .query_row("SELECT COUNT(*) FROM files", [], |r| r.get(0))?)
    }

    /// Coldstart completeness stamp + probe — the meta lifecycle
    /// lives in schema.rs; these are the Index-typed doors.
    pub fn mark_full_build(&mut self) -> Result<()> {
        schema::mark_full_build(&self.conn, self.opened_at)
    }

    pub fn full_build_done(&self) -> Result<bool> {
        schema::full_build_done(&self.conn)
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

    /// The raw connection. Read surface for the graph domain
    /// (graph/load.rs); the writer whitelist is this module,
    /// graph/store.rs, trend/mod.rs (M7-P4 — its rows are keyed by
    /// immutable commit hash, idempotent per ADR-003 v1.7), and
    /// rescache.rs (K step 10 — a one-slot REPLACE of derived data,
    /// last-writer-wins by design).
    pub(crate) fn raw(&self) -> &rusqlite::Connection {
        &self.conn
    }
}

/// READ-ONLY index facts for diagnostics: `(indexed files, cache key
/// still current)`. Never rebuilds, never stamps, never wipes.
///
/// `Index::open` is the wrong door for a status line: its
/// `schema::ensure_cache_key` WIPES the database on any revision
/// mismatch, so `ce doctor` and the SessionStart line destroyed the
/// index they were reporting on — and a mixed-revision pair of `ce`
/// binaries on one tree (a SHA-pinned hook one beside a `cargo
/// install`ed terminal one) wiped each other's work every session
/// while both printed a healthy-looking count. A diagnostic must not
/// mutate the state it reports (health.rs's own stance); a stale key
/// is REPORTED here and repaired by the next dedup, which owns it.
pub fn peek(db_path: &Path, p: Params) -> Result<(i64, bool)> {
    let conn =
        Connection::open(db_path).with_context(|| format!("peek index {}", db_path.display()))?;
    // WAL readers do not block on writers, but a rollback-journal or
    // pre-WAL file would: a status line must not cry "unreadable"
    // merely because a dedup run holds the lock.
    conn.pragma_update(None, "busy_timeout", 2000)?;
    let current = schema::schema_current(&conn, p)?;
    let files = conn.query_row("SELECT COUNT(*) FROM files", [], |r| r.get(0))?;
    Ok((files, current))
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

#[cfg(test)]
mod tests {
    use super::*;

    /// A revision-skewed index is REPORTED stale and left intact —
    /// twice, because the defect this pins was a diagnostic that
    /// repaired what it measured: the same read through `Index::open`
    /// wipes the row and then answers "0 files, current".
    #[test]
    fn peek_reports_a_stale_key_and_wipes_nothing() {
        let dir = crate::testutil::scratch("peek");
        let db = dir.join(".ce/index.db");
        let p = Params::default();
        drop(Index::open(&db, p).expect("open"));
        let raw = Connection::open(&db).expect("raw");
        raw.execute(
            "INSERT INTO files (path, content_hash, token_count, has_tokens)
             VALUES ('a.rs', 1, 0, 1)",
            [],
        )
        .expect("row");
        // exactly what a sibling binary of another revision presents
        raw.execute("UPDATE meta SET v = v + 1 WHERE k = 'tokenizer_rev'", [])
            .expect("skew");
        drop(raw);
        assert_eq!(peek(&db, p).expect("peek"), (1, false), "stale, intact");
        assert_eq!(peek(&db, p).expect("peek"), (1, false), "still intact");
        std::fs::remove_dir_all(&dir).ok();
    }
}
