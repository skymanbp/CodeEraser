//! Schema v4 lifecycle of the single `.ce/index.db` (ADR-005 + M5-2
//! design brief §3). The dedup tables (files/fingerprints/meta) are
//! defined here; the graph tables (symbols/sites/edges) are created
//! from graph/store's CREATE-only DDL — but their DROPs belong to
//! this wipe, because schema versioning is one lifecycle, not two.

use super::{Params, tokens};
use crate::graph::store;
use anyhow::Result;
use rusqlite::{Connection, Transaction, TransactionBehavior};

/// Pre-release schema versioning: a mismatch drops and recreates the
/// tables (the index is a cache — rebuilding is always safe).
/// v5 (M5-3b): `symbols.nth` + UNIQUE identity (F2), the unitsig
/// structural cache and the docsegs table, struct_rev + docdup_rev
/// in the cache key. v4: `has_tokens` on files, the graph tables,
/// graph_rev in the key. ALTERs fold into CREATE — the wipe model
/// has no migration path to alter along.
const SCHEMA_VERSION: i64 = 5;

const SCHEMA: &str = "
DROP TABLE IF EXISTS docsegs;
DROP TABLE IF EXISTS unitsig;
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
/// fingerprints for unchanged files). Concurrent openers race this
/// check — CI caught two autocommit rebuilds interleaving statement
/// by statement ("table sites already exists") — so the rebuild
/// takes the write lock FIRST (IMMEDIATE; busy_timeout waits) and
/// re-checks under it: the race loser finds the winner's fresh
/// schema and touches nothing.
pub(crate) fn ensure_cache_key(conn: &Connection, p: Params) -> Result<()> {
    if schema_current(conn, p)? {
        return Ok(());
    }
    let tx = Transaction::new_unchecked(conn, TransactionBehavior::Immediate)?;
    if !schema_current(&tx, p)? {
        tx.execute_batch(SCHEMA)?;
        tx.execute_batch(store::GRAPH_SCHEMA)?;
        tx.execute_batch(super::unitcache::UNITSIG_SCHEMA)?;
        tx.execute_batch(crate::docdup::DOCSEGS_SCHEMA)?;
        tx.pragma_update(None, "user_version", SCHEMA_VERSION)?;
        let mut stmt = tx.prepare("INSERT INTO meta (k, v) VALUES (?1, ?2)")?;
        for (k, v) in meta_entries(p) {
            stmt.execute((k, v))?;
        }
        drop(stmt);
    }
    tx.commit()?;
    Ok(())
}

fn schema_current(conn: &Connection, p: Params) -> Result<bool> {
    let version: i64 = conn.query_row("PRAGMA user_version", [], |r| r.get(0))?;
    Ok(version == SCHEMA_VERSION && meta_matches(conn, p)?)
}

fn meta_entries(p: Params) -> [(&'static str, i64); 6] {
    [
        ("kgram", p.kgram as i64),
        ("window", p.window as i64),
        ("tokenizer_rev", tokens::TOKENIZER_REV),
        ("graph_rev", store::GRAPH_REV),
        ("struct_rev", super::struct_fp::STRUCT_REV),
        ("docdup_rev", crate::docdup::DOCDUP_REV),
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

/// The one per-file cache-refresh shape: delete the file's rows,
/// prepare the insert, let the caller drive it (unitsig and docsegs
/// both write this way — the ratchet caught the pair re-instantiating
/// the walk).
pub(crate) fn replace_file_rows(
    tx: &Transaction<'_>,
    table: &str,
    file_id: i64,
    insert: &str,
    fill: impl FnOnce(&mut rusqlite::Statement<'_>) -> Result<()>,
) -> Result<()> {
    tx.execute(
        &format!("DELETE FROM {table} WHERE file_id = ?1"),
        (file_id,),
    )?;
    let mut ins = tx.prepare(insert)?;
    fill(&mut ins)
}
