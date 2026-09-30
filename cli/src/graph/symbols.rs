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
    /// Its fourclass kind code (kinds.rs: fn 1, named 2, impl 3,
    /// section 4), stored since GRAPH_REV 23.
    pub kind: i64,
}

/// Every cached symbol, deterministically ordered by identity.
pub fn symbol_rows(idx: &Index) -> Result<Vec<SymbolRow>> {
    super::load::rows(
        idx.raw(),
        "SELECT f.path, s.key, s.nth, s.start_line, s.end_line, s.flags, s.conv, s.kind
         FROM symbols s JOIN files f ON f.id = s.file_id
         ORDER BY f.path, s.key, s.nth",
        |r| {
            let int = |i: usize| r.get::<_, i64>(i);
            Ok(SymbolRow {
                path: r.get(0)?,
                key: r.get(1)?,
                nth: int(2)?,
                start_line: int(3)?,
                end_line: int(4)?,
                vis: int(5)?,
                conv: int(6)?,
                kind: int(7)?,
            })
        },
    )
}
