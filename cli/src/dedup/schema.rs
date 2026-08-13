//! Schema v4 lifecycle of the single `.ce/index.db` (ADR-005 + M5-2
//! design brief §3). The dedup tables (files/fingerprints/meta) are
//! defined here; the graph tables (symbols/sites/edges) are created
//! from graph/store's CREATE-only DDL — but their DROPs belong to
//! this wipe, because schema versioning is one lifecycle, not two.

use super::{Params, tokens};
use crate::graph::store;
use anyhow::Result;
use rusqlite::Connection;

/// Pre-release schema versioning: a mismatch drops and recreates the
/// tables (the index is a cache — rebuilding is always safe).
/// v4: `has_tokens` on files (Markdown enters with zero fingerprint
/// rows), the graph tables, and graph_rev in the cache key. The
/// design's `ALTER TABLE files ADD has_tokens` folds into CREATE —
/// the wipe model has no migration path to alter along.
const SCHEMA_VERSION: i64 = 4;

const SCHEMA: &str = "
DROP TABLE IF EXISTS edges;
DROP TABLE IF EXISTS sites;
DROP TABLE IF EXISTS symbols;
DROP TABLE IF EXISTS fingerprints;
DROP TABLE IF EXISTS files;
DROP TABLE IF EXISTS meta;
CREATE TABLE files (
  id INTEGER PRIMARY KEY,
  path TEXT UNIQUE NOT NULL,
  content_hash INTEGER NOT NULL,
  token_count INTEGER NOT NULL,
  has_tokens INTEGER NOT NULL
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

/// Wipe-and-recreate unless both the schema version and the meta
/// cache key (params + tokenizer rev + graph rev) match
/// (attack-review D2: params/tokenizer changes silently reused stale
/// fingerprints for unchanged files).
pub(crate) fn ensure_cache_key(conn: &Connection, p: Params) -> Result<()> {
    let version: i64 = conn.query_row("PRAGMA user_version", [], |r| r.get(0))?;
    if version == SCHEMA_VERSION && meta_matches(conn, p)? {
        return Ok(());
    }
    conn.execute_batch(SCHEMA)?;
    conn.execute_batch(store::GRAPH_SCHEMA)?;
    conn.pragma_update(None, "user_version", SCHEMA_VERSION)?;
    let mut stmt = conn.prepare("INSERT INTO meta (k, v) VALUES (?1, ?2)")?;
    for (k, v) in meta_entries(p) {
        stmt.execute((k, v))?;
    }
    Ok(())
}

fn meta_entries(p: Params) -> [(&'static str, i64); 4] {
    [
        ("kgram", p.kgram as i64),
        ("window", p.window as i64),
        ("tokenizer_rev", tokens::TOKENIZER_REV),
        ("graph_rev", store::GRAPH_REV),
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

pub(crate) fn ignore_no_rows(e: rusqlite::Error) -> rusqlite::Result<Option<i64>> {
    if e == rusqlite::Error::QueryReturnedNoRows {
        Ok(None)
    } else {
        Err(e)
    }
}
