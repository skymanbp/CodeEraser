//! §5.1 of the sealed criterion: the two mention tables in the shared
//! `.ce/index.db`, owned END TO END by the mention pass — created
//! additively here (a v14 database built before this pass gains them
//! on first use), dropped by the schema wipe (dedup/schema.rs: one
//! lifecycle), and emptied by a `MENTION_REV` mismatch WITHOUT
//! touching `files` or `trend` (the pass has its own meta row).
//!
//! `mention_files.hash` is the pass's OWN content gate, not a share of
//! `files.content_hash`: any path that drops mention rows without
//! changing bytes — a `.gitignore` edit, crossing the 4 MiB cap, an
//! early NUL — would otherwise leave `refresh_file`'s gate answering
//! "unchanged" forever and the rows never rebuilt. Creation and
//! deletion under one authority is what makes a debt ledger
//! unnecessary here.
//!
//! Privacy (§5.1, restated honestly): no plaintext token of a
//! non-judged file enters the database — `ident_hash`/`folded_hash`
//! are 64-bit unkeyed FNV-1a. That hash is a CONFIRMATION oracle (a
//! guessed low-entropy secret can be membership-tested), accepted as
//! the residue. The existing plaintext faces are `symbols.key` and
//! `sites.spec` (a `url` site with `?token=` lands verbatim) — named,
//! not repaired, in this batch.

use crate::dedup::schema::{ignore_no_rows, replace_file_rows};
use anyhow::Result;
use rusqlite::{Connection, Transaction, TransactionBehavior};
use std::collections::BTreeSet;

/// CREATE-only, `IF NOT EXISTS` on purpose (module doc). The
/// `file_id` index is the cascade child's FK index — the schema-v6
/// lesson (a cascade without one full-scans the child per delete).
pub const MENTION_SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS mention_files (id INTEGER PRIMARY KEY,
  path TEXT UNIQUE NOT NULL, hash INTEGER NOT NULL);
CREATE TABLE IF NOT EXISTS mentions (
  file_id INTEGER NOT NULL REFERENCES mention_files(id) ON DELETE CASCADE,
  ident_hash INTEGER NOT NULL, folded_hash INTEGER,
  UNIQUE(ident_hash, file_id));
CREATE INDEX IF NOT EXISTS idx_mention_file ON mentions(file_id);
CREATE INDEX IF NOT EXISTS idx_mention_folded ON mentions(folded_hash);
";

/// One de-duplicated token of one file: the identity hash and, for a
/// token long enough for the second chance, its fold hash.
pub(super) struct Row {
    pub ident: i64,
    pub folded: Option<i64>,
}

/// The pass's connection tuning, alive for exactly the pass: three
/// measured levers go on in `prepare` and come off in `Drop`, so an
/// early `?` anywhere in the pass cannot leave the shared connection
/// in the pass's mode. The levers: the page cache (4.90 s → 0.98 s
/// inserts on the self corpus); `synchronous=NORMAL`, which drops the
/// per-commit WAL fsync — in WAL mode the database stays consistent
/// through any crash and only the last batches are at risk on a power
/// cut, which for a cache is a re-run; and auto-checkpointing off, so
/// the 200k-row burst is checkpointed once (`finish`) instead of every
/// 4 MiB of WAL (files loop 1.96 s → 1.26 s).
pub(super) struct Tuned<'c> {
    conn: &'c Connection,
    cache: i64,
}

impl Drop for Tuned<'_> {
    fn drop(&mut self) {
        // Drop has no caller to report to and the three restores are
        // independent, so each is attempted regardless of the others —
        // the connection's owner drops it right after the pass anyway
        let _ = self.conn.pragma_update(None, "wal_autocheckpoint", 1000);
        let _ = self.conn.pragma_update(None, "synchronous", "FULL");
        let _ = self.conn.pragma_update(None, "cache_size", self.cache);
    }
}

/// Tables present, the connection tuned for the insert burst, and the
/// revision gate: returns the tuning guard and whether a `MENTION_REV`
/// mismatch emptied both tables.
pub(super) fn prepare(conn: &Connection, rev: i64) -> Result<(Tuned<'_>, bool)> {
    conn.execute_batch(MENTION_SCHEMA)?;
    let cache: i64 = conn.pragma_query_value(None, "cache_size", |r| r.get(0))?;
    let tuned = Tuned { conn, cache };
    conn.pragma_update(None, "cache_size", -64000)?;
    conn.pragma_update(None, "synchronous", "NORMAL")?;
    conn.pragma_update(None, "wal_autocheckpoint", 0)?;
    let stored: Option<i64> = conn
        .query_row("SELECT v FROM meta WHERE k = 'mention_rev'", [], |r| {
            r.get(0)
        })
        .map(Some)
        .or_else(ignore_no_rows)?;
    if stored == Some(rev) {
        return Ok((tuned, false));
    }
    let tx = Transaction::new_unchecked(conn, TransactionBehavior::Immediate)?;
    // children first: an unqualified DELETE on a table with no
    // dependants takes SQLite's truncate path, while letting the
    // parent's cascade do it walks 200k rows through three indexes
    // (measured: ~5 s on the self corpus)
    tx.execute("DELETE FROM mentions", [])?;
    tx.execute("DELETE FROM mention_files", [])?;
    tx.execute(
        "INSERT INTO meta (k, v) VALUES ('mention_rev', ?1)
         ON CONFLICT(k) DO UPDATE SET v = ?1",
        (rev,),
    )?;
    tx.commit()?;
    Ok((tuned, true))
}

/// The pass is over on the happy path: one checkpoint for the whole
/// burst; the levers come off when `tuned` drops.
pub(super) fn finish(tuned: &Tuned<'_>) -> Result<()> {
    Ok(tuned.conn.execute_batch("PRAGMA wal_checkpoint(PASSIVE)")?)
}

/// The stored content hash of one path — the pass's own gate.
pub(super) fn stored_hash(conn: &Connection, rel: &str) -> Result<Option<i64>> {
    Ok(conn
        .query_row(
            "SELECT hash FROM mention_files WHERE path = ?1",
            (rel,),
            |r| r.get(0),
        )
        .map(Some)
        .or_else(ignore_no_rows)?)
}

/// The rows one path holds today (0 for a path the pass never saw).
pub(super) fn rows_of(conn: &Connection, rel: &str) -> Result<usize> {
    Ok(conn.query_row(
        "SELECT COUNT(*) FROM mentions m JOIN mention_files f ON f.id = m.file_id
         WHERE f.path = ?1",
        (rel,),
        |r| r.get(0),
    )?)
}

/// Replace one file's rows inside the caller's batch transaction —
/// per-file atomicity is all the pass promises (run-wide consistency
/// is unreachable: a file edited between two reads is wrong this run
/// and converges next run), and a batch commits whole.
pub(super) fn replace_file(tx: &Transaction<'_>, rel: &str, hash: i64, rows: &[Row]) -> Result<()> {
    tx.execute(
        "INSERT INTO mention_files (path, hash) VALUES (?1, ?2)
         ON CONFLICT(path) DO UPDATE SET hash = ?2",
        (rel, hash),
    )?;
    let id: i64 = tx.query_row(
        "SELECT id FROM mention_files WHERE path = ?1",
        (rel,),
        |r| r.get(0),
    )?;
    replace_file_rows(
        tx,
        "mentions",
        id,
        "INSERT INTO mentions (file_id, ident_hash, folded_hash) VALUES (?1, ?2, ?3)",
        |ins| {
            for row in rows {
                ins.execute((id, row.ident, row.folded))?;
            }
            Ok(())
        },
    )
}

/// Every path the pass has rows for, read BEFORE the walk — the bound
/// on what this run may reap (a file a concurrent run indexed after
/// our walk began is not ours to delete).
pub(super) fn indexed_paths(conn: &Connection) -> Result<BTreeSet<String>> {
    Ok(conn
        .prepare("SELECT path FROM mention_files")?
        .query_map([], |r| r.get::<_, String>(0))?
        .collect::<rusqlite::Result<_>>()?)
}

/// Reap the rows of `seen` files not in `live` — one IMMEDIATE
/// transaction, the cascade takes the mentions. Returns the files
/// actually deleted, so a second call over an overlapping set counts
/// nothing twice.
pub(super) fn prune(
    conn: &Connection,
    live: &BTreeSet<String>,
    seen: &BTreeSet<String>,
) -> Result<usize> {
    let gone: Vec<&String> = seen.difference(live).collect();
    if gone.is_empty() {
        return Ok(0);
    }
    let tx = Transaction::new_unchecked(conn, TransactionBehavior::Immediate)?;
    let mut del = tx.prepare("DELETE FROM mention_files WHERE path = ?1")?;
    let mut deleted = 0;
    for path in &gone {
        deleted += del.execute((path,))?;
    }
    drop(del);
    tx.commit()?;
    Ok(deleted)
}

/// `(rows, files holding at least one row)` — the table's size and
/// the "mention source files" counter of the pass header.
pub(super) fn totals(conn: &Connection) -> Result<(usize, usize)> {
    Ok(conn.query_row(
        "SELECT COUNT(*), COUNT(DISTINCT file_id) FROM mentions",
        [],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )?)
}

/// Files holding the per-file cap of rows — the store's standing clip,
/// which the header states every run beside this run's delta.
pub(super) fn capped(conn: &Connection, cap: usize) -> Result<usize> {
    Ok(conn.query_row(
        "SELECT COUNT(*) FROM (SELECT file_id FROM mentions GROUP BY file_id HAVING COUNT(*) >= ?1)",
        (cap as i64,),
        |r| r.get(0),
    )?)
}

/// Whether any walked file mentions this identity hash — the read the
/// veto will stand on; here it serves the pass's own legs.
pub fn mentioned_by_other(conn: &Connection, ident: i64, own_path: &str) -> Result<bool> {
    Ok(conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM mentions m JOIN mention_files f ON f.id = m.file_id
         WHERE m.ident_hash = ?1 AND f.path <> ?2)",
        (ident, own_path),
        |r| r.get(0),
    )?)
}
