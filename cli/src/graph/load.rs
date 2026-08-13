//! The cached graph's read surface (M5-2h): every live file path,
//! every edge joined to its source file, and the count of sites the
//! ladder refused — deadcode's input, deterministically ordered so
//! the wire bytes downstream are a function of the graph (G11).
//! Split from dedup/index.rs at the 300-line dogfood gate: the graph
//! domain reads THROUGH the index, it does not live in it.

use crate::dedup::index::Index;
use anyhow::Result;

/// One cached edge joined to its source path.
pub struct GraphEdge {
    pub src: String,
    pub dst_path: String,
    pub dst_unit: String,
    pub kind: i64,
    pub rung: i64,
}

impl GraphEdge {
    // tuple hop on purpose: a field-by-field literal would align
    // token-for-token with store.rs's CachedSite mapper (ratchet)
    fn from_row(r: &rusqlite::Row<'_>) -> rusqlite::Result<Self> {
        let (src, dst_path, dst_unit) = (r.get(0)?, r.get(1)?, r.get(2)?);
        let (kind, rung) = (r.get(3)?, r.get(4)?);
        Ok(GraphEdge {
            src,
            dst_path,
            dst_unit,
            kind,
            rung,
        })
    }
}

pub fn graph_rows(idx: &Index) -> Result<(Vec<String>, Vec<GraphEdge>, i64)> {
    let conn = idx.raw();
    let files: Vec<String> = conn
        .prepare("SELECT path FROM files ORDER BY path")?
        .query_map([], |r| r.get(0))?
        .collect::<rusqlite::Result<_>>()?;
    let edges: Vec<GraphEdge> = conn
        .prepare(
            "SELECT f.path, e.dst_path, e.dst_unit, e.kind, e.rung
             FROM edges e JOIN sites s ON s.id = e.site_id
             JOIN files f ON f.id = s.file_id
             ORDER BY f.path, e.dst_path, e.dst_unit, e.kind, e.rung",
        )?
        .query_map([], GraphEdge::from_row)?
        .collect::<rusqlite::Result<_>>()?;
    let unresolved: i64 = conn.query_row(
        "SELECT COUNT(*) FROM sites s
         WHERE NOT EXISTS (SELECT 1 FROM edges e WHERE e.site_id = s.id)",
        [],
        |r| r.get(0),
    )?;
    Ok((files, edges, unresolved))
}
