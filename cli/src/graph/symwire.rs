//! The export surface on the wire (plan v2.14, proto 4.1.0): which
//! files declare something, and how visibly. One projection of the
//! symbols table, mapped onto the graph's dense node identity.
//!
//! Why this and not the symbol edges measured beside it: a symbol
//! edge is high-precision evidence that a declaration IS referenced
//! (the K10 site audit read 683 of them and found no known error
//! class), but every "unreferenced" claim spends the OTHER quantity
//! — recall — and import bindings hold ~23% of it on this repo,
//! because `crate::graph::md::is_md_path(dst)` and
//! `idx.resolve_refreshed(..)` are not import sites. The export
//! surface asks nothing of recall: whether a file declares an
//! exported symbol is answerable exactly, and the unreferenced half
//! stays where the ladder's full edge coverage already lives — the
//! file graph.
//!
//! What the core does with it (CE.Graph.effectiveFlags): a node named
//! here as exporting something carries flag bit 0, the public/private
//! axis Dead.deadTable has always split on and that no producer could
//! set — "bit 0 stays unset at file granularity, public-ness is a
//! symbol fact" (deadcode/flags.rs:9). So verdict codes 2 and 4,
//! unref_public and unreach_public, become reachable for the first
//! time since 2.28.0. The bit is deliberately outside entryMask: an
//! export surface is a verdict axis, never an entry claim (RG10), so
//! it can change which code a dead node reports and can never change
//! which nodes are dead.

use crate::dedup::index::Index;
use anyhow::{Context, Result};
use std::collections::{BTreeMap, BTreeSet, HashMap};

/// The `symbols` wire table: deduped `[nodeIdx, visibility]` pairs,
/// ascending.
///
/// DISTINCT at the query, a set at the collect: two declarations of
/// equal visibility in one file are not two facts about that file's
/// export surface, which is the same reason Build.hs dedups arcs —
/// multiplicity is not extra evidence. That is also what keeps the
/// table bounded at two rows per file rather than one per
/// declaration (5149 symbols become 750 rows here at most).
///
/// A symbol whose path is not a node is index skew, named the way
/// `edge_wire` names a skewed endpoint: symbols reference files by
/// row id and every walked file is a node, so a miss is a cache to
/// report, never a row to drop quietly.
pub fn export_surface(
    idx: &Index,
    ids: &BTreeMap<(&str, &str), usize>,
) -> Result<BTreeSet<[i64; 2]>> {
    super::load::rows(
        idx.raw(),
        "SELECT DISTINCT f.path, s.flags FROM symbols s JOIN files f ON f.id = s.file_id
         ORDER BY f.path, s.flags",
        |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?)),
    )?
    .into_iter()
    .map(|(path, vis)| {
        let id = ids
            .get(&(path.as_str(), ""))
            .with_context(|| format!("symbol owner {path} not a node — index skew"))?;
        Ok([*id as i64, vis])
    })
    .collect()
}

/// The same surface under the verdict family's universe (6.1.0):
/// `verdict/1` keys files by position in the score road's `files`
/// list, so the table is re-keyed — and re-keyed is all it is. The
/// visibility word crosses untouched because which bit means
/// "exported" is judgment (`CE.Graph.Cost.exportVisBit`); a
/// pre-decided `Vec<u>` would move that call to the measurement
/// side. Ascent comes free (node ids ascend and file nodes keep
/// their order), and an owner that is not a file node is index
/// skew, refused the way `export_surface` refuses its own miss.
pub fn rekeyed(w: &super::deadcode::GraphWire, idx: &HashMap<&str, i64>) -> Result<Vec<[i64; 2]>> {
    w.symbols
        .iter()
        .map(|&[node, vis]| {
            let path = w
                .nodes
                .get(usize::try_from(node).unwrap_or(usize::MAX))
                .map(|n| n.path.as_str())
                .context("symbol row names a node outside the graph")?;
            let u = idx
                .get(path)
                .with_context(|| format!("export surface owner {path} is not a file node"))?;
            Ok([*u, vis])
        })
        .collect()
}
