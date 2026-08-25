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
    /// Stored wire granularity — node minting reads it instead of
    /// inferring "package" from mere absence (M5-close review LOW:
    /// image assets and dangling doc refs were minted as packages).
    pub granularity: i64,
}

/// The one prepare→query_map→collect throat every cached-table read
/// surface calls (load / symbols / unitcache): the boilerplate lived
/// three times for one batch before the ratchet bit it.
pub(crate) fn rows<T>(
    conn: &rusqlite::Connection,
    sql: &str,
    map: impl FnMut(&rusqlite::Row<'_>) -> rusqlite::Result<T>,
) -> Result<Vec<T>> {
    Ok(conn
        .prepare(sql)?
        .query_map([], map)?
        .collect::<rusqlite::Result<_>>()?)
}

/// Column-generic get-chains: under T2 normalization EVERY
/// field-by-field row mapper is the same token stream, so the chain
/// exists once per arity and each read surface keeps only its
/// destructure→construct semantics (below the clone floor).
/// `Col` is both shorthand and the token-shape breaker: four
/// spelled-out FromSql bounds were themselves a 50-token run.
pub(crate) trait Col: rusqlite::types::FromSql {}
impl<T: rusqlite::types::FromSql> Col for T {}

pub(crate) fn t5<A: Col, B: Col, C: Col, D: Col, E: Col>(
    r: &rusqlite::Row<'_>,
) -> rusqlite::Result<(A, B, C, D, E)> {
    let head = t4(r)?;
    Ok((head.0, head.1, head.2, head.3, r.get(4)?))
}

pub(crate) fn t6<A: Col, B: Col, C: Col, D: Col, E: Col, F: Col>(
    r: &rusqlite::Row<'_>,
) -> rusqlite::Result<(A, B, C, D, E, F)> {
    let head = t5(r)?;
    Ok((head.0, head.1, head.2, head.3, head.4, r.get(5)?))
}

pub(crate) fn t4<A: Col, B: Col, C: Col, D: Col>(
    r: &rusqlite::Row<'_>,
) -> rusqlite::Result<(A, B, C, D)> {
    Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?))
}

/// Every path owning at least one unresolved site — the erase
/// planner's trust boundary (erase.md: a dead-file row is refused
/// while its LANGUAGE still carries unresolved sites; the scalar
/// count travels with the deadcode report, this names the owners so
/// the language fold is a fact, never a guess).
pub fn unresolved_paths(idx: &Index) -> Result<Vec<String>> {
    rows(
        idx.raw(),
        "SELECT DISTINCT f.path FROM sites s JOIN files f ON f.id = s.file_id
         WHERE NOT EXISTS (SELECT 1 FROM edges e WHERE e.site_id = s.id)
         ORDER BY f.path",
        |r| r.get(0),
    )
}

/// One import binding joined to the edge it rides: the citing file,
/// the file the ladder resolved to, and both halves of the name.
/// This is the symbol-edge join's input — the target name is what the
/// resolved file's symbols table is asked for, and a miss means the
/// name was a module or a re-export, not a declaration (plan v2.14).
pub struct BindingEdge {
    pub src: String,
    pub dst_path: String,
    pub local: String,
    pub target: String,
}

/// Every candidate binding that rides a FILE-granularity edge,
/// deterministically ordered. Package and section edges carry no
/// declaration to name, so they are not joined.
pub fn binding_edges(idx: &Index) -> Result<Vec<BindingEdge>> {
    Ok(rows(
        idx.raw(),
        "SELECT f.path, e.dst_path, b.local, b.target
         FROM bindings b
         JOIN sites s ON s.id = b.site_id
         JOIN edges e ON e.site_id = s.id
         JOIN files f ON f.id = s.file_id
         WHERE e.granularity = 0
         ORDER BY f.path, e.dst_path, b.target, b.local",
        t4,
    )?
    .into_iter()
    .map(|(src, dst_path, local, target)| BindingEdge {
        src,
        dst_path,
        local,
        target,
    })
    .collect())
}

/// One row per site-owning file: (path, total sites, unresolved
/// sites) — the trust ledger raw material (2.32.0, H3).
pub type PathSites = Vec<(String, i64, i64)>;

pub fn graph_rows(idx: &Index) -> Result<(Vec<String>, Vec<GraphEdge>, i64, PathSites)> {
    let conn = idx.raw();
    // ONE read snapshot for one graph: as three autocommit statements
    // each read took its own WAL snapshot, so a convergent writer
    // (ADR-003) landing between them could hand the edge query a
    // source file the files query never saw — and the wire build
    // indexes nodes by source (review 2026-08-19, codex lane).
    let txn = conn.unchecked_transaction()?;
    let conn = &*txn;
    let files = rows(conn, "SELECT path FROM files ORDER BY path", |r| r.get(0))?;
    let edges = rows(
        conn,
        "SELECT f.path, e.dst_path, e.dst_unit, e.kind, e.rung, e.granularity
         FROM edges e JOIN sites s ON s.id = e.site_id
         JOIN files f ON f.id = s.file_id
         ORDER BY f.path, e.dst_path, e.dst_unit, e.kind, e.rung",
        t6,
    )?
    .into_iter()
    .map(
        |(src, dst_path, dst_unit, kind, rung, granularity)| GraphEdge {
            src,
            dst_path,
            dst_unit,
            kind,
            rung,
            granularity,
        },
    )
    .collect();
    let unresolved: i64 = conn.query_row(
        "SELECT COUNT(*) FROM sites s
         WHERE NOT EXISTS (SELECT 1 FROM edges e WHERE e.site_id = s.id)",
        [],
        |r| r.get(0),
    )?;
    // per-path (total, unresolved) site counts — the graph family's
    // trust ledger (2.32.0, H3): folded to per-language rows at the
    // wire, judged into each dead row's confidence by the core. Read
    // inside the SAME snapshot as the edges it vouches about.
    let sites = rows(
        conn,
        "SELECT f.path, COUNT(*),
                SUM(CASE WHEN NOT EXISTS (SELECT 1 FROM edges e WHERE e.site_id = s.id)
                    THEN 1 ELSE 0 END)
         FROM sites s JOIN files f ON f.id = s.file_id
         GROUP BY f.path ORDER BY f.path",
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
    )?;
    txn.finish()?; // read-only: closing the snapshot, nothing to write
    Ok((files, edges, unresolved, sites))
}
