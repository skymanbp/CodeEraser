//! Parser-derived tables have a separate revision gate. A parser
//! upgrade must refresh unchanged files, without deleting trend history.
//! Trend's toolchain stamp includes this revision, so retained points
//! are remeasured before they can join the new parser's trajectory.

use crate::dedup::tokens::TOKENIZER_REV;
use anyhow::Result;
use rusqlite::{Connection, OptionalExtension, Transaction};

/// The meta row this gate reads; schema.rs keys its cache entries
/// by the same name and leaves it out of the storage match.
pub(super) const KEY: &str = "tokenizer_rev";

pub(super) fn current(conn: &Connection) -> Result<bool> {
    let rev = conn
        .query_row("SELECT v FROM meta WHERE k = ?1", [KEY], |r| {
            r.get::<_, i64>(0)
        })
        .optional()?;
    Ok(rev == Some(TOKENIZER_REV))
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
         DELETE FROM docsegs;
         DELETE FROM unitsig;
         DELETE FROM edges;
         DELETE FROM sites;
         DELETE FROM symbols;
         DELETE FROM fingerprints;
         DELETE FROM files;
         DELETE FROM meta WHERE k IN ('full_build', 'resolve_key', 'mention_rev');",
    )?;
    for (key, value) in [(KEY, TOKENIZER_REV), ("epoch", epoch)] {
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
