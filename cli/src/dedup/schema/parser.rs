//! Parser-derived tables have a separate revision gate. A parser
//! upgrade must refresh unchanged files, without deleting trend history.
//! Trend's toolchain stamp includes this revision, so retained points
//! are remeasured before they can join the new parser's trajectory.
//!
//! The gate has two keys since v2.35 G2 (R2-4): the Rust tokenizer's
//! revision and the definition package's digest. Every row this gate
//! clears was cut with the package's tables — comment kinds, literal
//! delimiters, docdup markers and floors, the term road — so a core
//! whose package moved refreshes them like a parser upgrade does,
//! with no hand bump of a Rust revision (booklet 01's promise that a
//! parameter change cannot silently reuse stale fingerprints). Trend
//! keeps its rows: its stamp names the core, which is what moved.

use crate::dedup::tokens::TOKENIZER_REV;
use anyhow::Result;
use rusqlite::{Connection, OptionalExtension, Transaction};

/// The meta row this gate reads; schema.rs keys its cache entries
/// by the same name and leaves it out of the storage match.
pub(super) const KEY: &str = "tokenizer_rev";

/// The meta row holding the definition package's digest.
const PACKAGE_KEY: &str = "tables_digest";

/// The package digest this process measures with; Err = the named
/// reason there is no package (the index cannot be keyed without it).
fn package_digest() -> Result<i64> {
    crate::tables::load()
        .map(|pack| pack.digest as i64)
        .map_err(|why| anyhow::anyhow!("the index is keyed by the definition package: {why}"))
}

/// Both keys match. A process with no package reads every index as
/// not current (a status line says stale; a writer refuses by name).
pub(super) fn current(conn: &Connection) -> Result<bool> {
    let read = |k: &str| {
        conn.query_row("SELECT v FROM meta WHERE k = ?1", [k], |r| {
            r.get::<_, i64>(0)
        })
        .optional()
    };
    Ok(read(KEY)? == Some(TOKENIZER_REV) && keyed_by(read(PACKAGE_KEY)?, package_digest().ok()))
}

/// The package half of the gate: only a package this process holds can
/// match. An index from before the key (no row) under no package is
/// NOT current — `None == None` would read it so.
fn keyed_by(stored: Option<i64>, package: Option<i64>) -> bool {
    package.is_some_and(|digest| stored == Some(digest))
}

/// Called under ensure_cache_key's IMMEDIATE transaction and recheck.
/// Children first avoids per-row cascade work. All content gates and
/// completion stamps leave with their rows; epoch fences in-flight runs.
pub(super) fn invalidate(tx: &Transaction<'_>, epoch: i64) -> Result<()> {
    tx.execute_batch(
        "DELETE FROM df;
         DELETE FROM bag;
         DELETE FROM mentions;
         DELETE FROM mention_files;
         DELETE FROM resolve_pending;
         DELETE FROM result_cache;
         DELETE FROM t3ted;
         DELETE FROM docsegs;
         DELETE FROM unitsig;
         DELETE FROM edges;
         DELETE FROM sites;
         DELETE FROM symbols;
         DELETE FROM fingerprints;
         DELETE FROM files;
         DELETE FROM meta WHERE k IN ('full_build', 'resolve_key', 'mention_rev', 't3_cache');",
    )?;
    stamp(tx, epoch)
}

/// Write both keys and the generation: the parser-level invalidation
/// and a full rebuild end on the same stamp.
pub(super) fn stamp(tx: &Transaction<'_>, epoch: i64) -> Result<()> {
    let package = package_digest()?;
    for (key, value) in [
        (KEY, TOKENIZER_REV),
        (PACKAGE_KEY, package),
        ("epoch", epoch),
    ] {
        tx.execute(
            "INSERT INTO meta (k, v) VALUES (?1, ?2)
             ON CONFLICT(k) DO UPDATE SET v = ?2",
            (key, value),
        )?;
    }
    Ok(())
}

#[cfg(test)]
#[path = "../../../tests/unit/dedup/parser.rs"]
mod tests;
