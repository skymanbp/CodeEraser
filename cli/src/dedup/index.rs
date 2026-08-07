//! SQLite inverted fingerprint index (ADR-005): `files` +
//! `fingerprints`, WAL + busy_timeout per ADR-003. Incremental
//! invalidation is content-hash gated per file: unchanged bytes touch
//! nothing; a change deletes and reinserts only that file's rows. The
//! M2 daemon becomes the sole writer; the batch CLI uses the same
//! code single-threaded.

use super::{Params, tokens, winnow};
use crate::scan::lang::Lang;
use anyhow::{Context, Result};
use rusqlite::Connection;
use std::collections::BTreeSet;
use std::path::Path;

/// Pre-release schema versioning: a mismatch drops and recreates the
/// tables (the index is a cache — rebuilding is always safe).
const SCHEMA_VERSION: i64 = 3;

const SCHEMA: &str = "
DROP TABLE IF EXISTS fingerprints;
DROP TABLE IF EXISTS files;
DROP TABLE IF EXISTS meta;
CREATE TABLE files (
  id INTEGER PRIMARY KEY,
  path TEXT UNIQUE NOT NULL,
  content_hash INTEGER NOT NULL,
  token_count INTEGER NOT NULL
);
CREATE TABLE fingerprints (
  hash INTEGER NOT NULL,
  file_id INTEGER NOT NULL REFERENCES files(id) ON DELETE CASCADE,
  start_tok INTEGER NOT NULL,
  start_line INTEGER NOT NULL,
  end_line INTEGER NOT NULL
);
CREATE INDEX idx_fp_hash ON fingerprints(hash);
CREATE INDEX idx_fp_file ON fingerprints(file_id);
CREATE TABLE meta (k TEXT PRIMARY KEY, v INTEGER NOT NULL);
";

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

pub struct Index {
    conn: Connection,
}

impl Index {
    /// Open (or wipe-and-recreate) the index. The cache key is
    /// schema version + winnowing params + tokenizer revision
    /// (attack-review D2: params/tokenizer changes silently reused
    /// stale fingerprints for unchanged files).
    pub fn open(db_path: &Path, p: Params) -> Result<Self> {
        if let Some(dir) = db_path.parent() {
            std::fs::create_dir_all(dir).with_context(|| format!("create {}", dir.display()))?;
        }
        let conn = Connection::open(db_path)
            .with_context(|| format!("open index {}", db_path.display()))?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "busy_timeout", 5000)?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        ensure_cache_key(&conn, p)?;
        Ok(Self { conn })
    }

    /// Refresh one file; returns false when the stored content hash
    /// already matches (nothing touched — the incremental fast path).
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
            .or_else(ignore_no_rows)?;
        if stored == Some(chash) {
            return Ok(false);
        }
        let toks = tokens::stream(src, lang)?;
        let hashes: Vec<u64> = toks.iter().map(|t| t.hash).collect();
        let fps = winnow::fingerprints(&hashes, p);
        let tx = self.conn.transaction()?;
        tx.execute(
            "INSERT INTO files (path, content_hash, token_count) VALUES (?1, ?2, ?3)
             ON CONFLICT(path) DO UPDATE SET content_hash = ?2, token_count = ?3",
            (rel, chash, toks.len() as i64),
        )?;
        let id: i64 = tx.query_row("SELECT id FROM files WHERE path = ?1", (rel,), |r| r.get(0))?;
        tx.execute("DELETE FROM fingerprints WHERE file_id = ?1", (id,))?;
        insert_fps(&tx, id, &fps, &toks, p)?;
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

    /// Every fingerprint occurrence, deterministically ordered.
    pub fn all_instances(&self) -> Result<Vec<Instance>> {
        let mut rows: Vec<Instance> = self
            .conn
            .prepare(
                "SELECT f.hash, fl.path, f.start_tok, f.start_line, f.end_line
                 FROM fingerprints f JOIN files fl ON fl.id = f.file_id",
            )?
            .query_map([], |r| {
                Ok(Instance {
                    hash: r.get::<_, i64>(0)? as u64,
                    file: r.get(1)?,
                    start_tok: r.get::<_, i64>(2)? as usize,
                    start_line: r.get::<_, i64>(3)? as usize,
                    end_line: r.get::<_, i64>(4)? as usize,
                })
            })?
            .collect::<rusqlite::Result<_>>()?;
        rows.sort();
        Ok(rows)
    }
}

/// Wipe-and-recreate unless both the schema version and the meta
/// cache key (params + tokenizer rev) match.
fn ensure_cache_key(conn: &Connection, p: Params) -> Result<()> {
    let version: i64 = conn.query_row("PRAGMA user_version", [], |r| r.get(0))?;
    if version == SCHEMA_VERSION && meta_matches(conn, p)? {
        return Ok(());
    }
    conn.execute_batch(SCHEMA)?;
    conn.pragma_update(None, "user_version", SCHEMA_VERSION)?;
    let mut stmt = conn.prepare("INSERT INTO meta (k, v) VALUES (?1, ?2)")?;
    for (k, v) in meta_entries(p) {
        stmt.execute((k, v))?;
    }
    Ok(())
}

fn meta_entries(p: Params) -> [(&'static str, i64); 3] {
    [
        ("kgram", p.kgram as i64),
        ("window", p.window as i64),
        ("tokenizer_rev", tokens::TOKENIZER_REV),
    ]
}

fn meta_matches(conn: &Connection, p: Params) -> Result<bool> {
    // a pre-meta database (or a foreign file) is a mismatch, not an
    // error — but real SQL failures must still propagate
    let has_meta: i64 = conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'meta'",
        [],
        |r| r.get(0),
    )?;
    if has_meta == 0 {
        return Ok(false);
    }
    for (k, want) in meta_entries(p) {
        let got: Option<i64> = conn
            .query_row("SELECT v FROM meta WHERE k = ?1", (k,), |r| r.get(0))
            .map(Some)
            .or_else(ignore_no_rows)?;
        if got != Some(want) {
            return Ok(false);
        }
    }
    Ok(true)
}

fn ignore_no_rows(e: rusqlite::Error) -> rusqlite::Result<Option<i64>> {
    if e == rusqlite::Error::QueryReturnedNoRows {
        Ok(None)
    } else {
        Err(e)
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
