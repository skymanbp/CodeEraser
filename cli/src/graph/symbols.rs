//! The symbols table's read surface (design vol.1 §1: none existed —
//! load.rs reads files/edges/sites only). Consumers: `ce clone
//! --units` cross-checks the unitsig cache against these rows today;
//! the M5-3 join reads per-unit spans through here later. Reads only
//! — every writer stays in graph/store.rs.

use crate::dedup::index::Index;
use anyhow::Result;

/// One symbol row joined to its file path.
pub struct SymbolRow {
    pub path: String,
    pub key: String,
    pub nth: i64,
    pub start_line: i64,
    pub end_line: i64,
    /// The declaration's own visibility bits (fourclass::visibility);
    /// bit 0 is the public/private axis the graph's verdict codes
    /// have always meant.
    pub vis: i64,
    /// The AST half of its convention-category word (mention::conv).
    pub conv: i64,
}

/// Every cached symbol, deterministically ordered by identity.
pub fn symbol_rows(idx: &Index) -> Result<Vec<SymbolRow>> {
    Ok(super::load::rows(
        idx.raw(),
        "SELECT f.path, s.key, s.nth, s.start_line, s.end_line, s.flags, s.conv
         FROM symbols s JOIN files f ON f.id = s.file_id
         ORDER BY f.path, s.key, s.nth",
        super::load::t7,
    )?
    .into_iter()
    .map(
        |(path, key, nth, start_line, end_line, vis, conv)| SymbolRow {
            path,
            key,
            nth,
            start_line,
            end_line,
            vis,
            conv,
        },
    )
    .collect())
}
