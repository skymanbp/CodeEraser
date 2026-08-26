//! Schema lifecycle of the single `.ce/index.db` (ADR-005 + M5-2
//! design brief §3; current version = SCHEMA_VERSION below, whose
//! doc lists the ladder). The dedup tables (files/fingerprints/meta) are
//! defined here; the graph tables (symbols/sites/edges) are created
//! from graph/store's CREATE-only DDL — but their DROPs belong to
//! this wipe, because schema versioning is one lifecycle, not two.

use super::{Params, tokens};
use crate::graph::store;
use anyhow::Result;
use rusqlite::{Connection, Transaction, TransactionBehavior};

/// Pre-release schema versioning: a mismatch drops and recreates the
/// tables (the index is a cache — rebuilding is always safe).
/// v7 (M7-P4): the trend table — score points ride the SAME wipe as
/// the fingerprints because a measurement-rev bump breaks their
/// comparability (trend/mod.rs header). v6 (M5 close): idx_edge_site —
/// edges was the only cascade child with no FK-key index, so every
/// per-file sites delete full-scanned the edge table (review MED).
/// v5 (M5-3b): `symbols.nth` + UNIQUE identity (F2), the unitsig
/// structural cache and the docsegs table, struct_rev + docdup_rev
/// in the cache key. v4: `has_tokens` on files, the graph tables,
/// graph_rev in the key. v8: edges.via_reexport (the §4 R5
/// amendment's mark). ALTERs fold into CREATE — the wipe model
/// has no migration path to alter along.
/// v10: density scoring (proto 2.17.0) re-prices every cached trend
/// score — a mixed-regime trajectory would fabricate a slope.
/// v11 (plan v2.14): the `bindings` table — the names each import
/// site brings into scope, the symbol-edge input that has no home in
/// any existing table.
/// v12 (K step 10, PERF ledger): `result_cache` — ONE memoized
/// analyze result keyed by files-table digest + effective filter
/// (rescache.rs); params and every algorithm rev already key the
/// whole database above, so a rev bump wipes the slot with the rest.
const SCHEMA_VERSION: i64 = 12; // 9: trend rows carry their measuring toolchain

const SCHEMA: &str = "
DROP TABLE IF EXISTS result_cache;
DROP TABLE IF EXISTS trend;
DROP TABLE IF EXISTS docsegs;
DROP TABLE IF EXISTS unitsig;
DROP TABLE IF EXISTS bindings;
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
CREATE TABLE result_cache (
  k INTEGER PRIMARY KEY CHECK (k = 1),
  digest INTEGER NOT NULL,
  min_tokens INTEGER NOT NULL,
  min_distinct INTEGER NOT NULL,
  blocks TEXT NOT NULL
);
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
        rebuild(&tx, p)?;
    }
    tx.commit()?;
    Ok(())
}

/// The wipe-and-recreate body, one DDL domain per line (split out
/// when the trend batch pushed the caller over the complexity gate).
fn rebuild(tx: &Transaction, p: Params) -> Result<()> {
    tx.execute_batch(SCHEMA)?;
    tx.execute_batch(store::GRAPH_SCHEMA)?;
    tx.execute_batch(super::unitcache::UNITSIG_SCHEMA)?;
    tx.execute_batch(crate::docdup::DOCSEGS_SCHEMA)?;
    tx.execute_batch(crate::trend::TREND_SCHEMA)?;
    tx.pragma_update(None, "user_version", SCHEMA_VERSION)?;
    let mut stmt = tx.prepare("INSERT INTO meta (k, v) VALUES (?1, ?2)")?;
    for (k, v) in meta_entries(p) {
        stmt.execute((k, v))?;
    }
    stmt.execute(("epoch", new_epoch()))?;
    Ok(())
}

/// A generation stamp for one wipe. The cache key protects against a
/// STALE database; the epoch protects against a database that was
/// wiped UNDER a run in flight — a mixed-revision pair (a SHA-pinned
/// hook binary and a `cargo install`ed terminal one) wipes each other
/// routinely, and the writer noticed nothing: the files it had already
/// indexed were gone, everything it wrote afterwards landed under the
/// wiper's key, and it then stamped full_build over the result. Not
/// cryptographic — it only has to differ from the previous wipe's.
fn new_epoch() -> i64 {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as i64)
        .unwrap_or(0);
    nanos ^ ((std::process::id() as i64) << 40)
}

/// The database's current generation, or None on a pre-epoch file.
pub(crate) fn epoch(conn: &Connection) -> Result<Option<i64>> {
    conn.query_row("SELECT v FROM meta WHERE k = 'epoch'", [], |r| r.get(0))
        .map(Some)
        .or_else(ignore_no_rows)
        .map_err(Into::into)
}

/// Whether the stored schema version AND cache key already match this
/// binary's. `pub(crate)` for index::peek — the diagnostics need the
/// verdict WITHOUT ensure_cache_key's wipe, and a second copy of
/// SCHEMA_VERSION over in index.rs would drift the day this one bumps.
pub(crate) fn schema_current(conn: &Connection, p: Params) -> Result<bool> {
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

/// Stamp "a full analyze completed on this database" — the coldstart
/// completeness fact (idempotent upsert; additive meta key, absent =
/// never completed; a bare row count was a per-process premise the
/// clearance review retired).
///
/// Refuses when the database was wiped out from under this run: the
/// stamp asserts "a full analyze completed on THIS database", and
/// after a concurrent wipe that sentence is false — coldstart reads
/// the stamp, answers READY, and probes then report matches=0 against
/// a truncated index (the silent-clean class its own header forbids).
/// A named error costs the caller a rebuild; the stamp cost a wrong
/// answer that survived until the file's bytes changed.
pub(crate) fn mark_full_build(
    conn: &rusqlite::Connection,
    opened_at: Option<i64>,
) -> anyhow::Result<()> {
    if epoch(conn)? != opened_at {
        anyhow::bail!("index was rebuilt by another process mid-run — not stamping full_build");
    }
    conn.execute(
        "INSERT INTO meta (k, v) VALUES ('full_build', 1)
         ON CONFLICT(k) DO UPDATE SET v = 1",
        [],
    )?;
    Ok(())
}

/// Whether any full analyze ever completed here (see mark_full_build).
pub(crate) fn full_build_done(conn: &rusqlite::Connection) -> anyhow::Result<bool> {
    let v: Option<i64> = conn
        .query_row("SELECT v FROM meta WHERE k = 'full_build'", [], |r| {
            r.get(0)
        })
        .map(Some)
        .or_else(ignore_no_rows)?;
    Ok(v == Some(1))
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
