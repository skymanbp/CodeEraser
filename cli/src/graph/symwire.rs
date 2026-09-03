//! The export surface on the wire (plan v2.14, proto 4.1.0): which
//! files declare something, and how visibly. One projection of the
//! symbols table, mapped onto the graph's dense node identity.
//!
//! Why this and not the symbol edges once measured beside it (retired
//! at index schema v14, plan v2.17 L round, user ruling: delete): a
//! symbol edge was high-precision evidence that a declaration IS
//! referenced (the K10 site audit read 683 of them and found no known
//! error class), but every "unreferenced" claim spends the OTHER
//! quantity — recall — and import bindings held ~23% of it on this
//! repo, because `crate::graph::md::is_md_path(dst)` and
//! `idx.resolve_refreshed(..)` are not import sites. The export
//! surface asks nothing of recall: whether a file declares an
//! exported symbol is answerable exactly, and the unreferenced half
//! stays where the ladder's full edge coverage already lives — the
//! file graph (the per-file mention veto that replaces the edges is
//! the L round's `unmentioned` table, not this one).
//!
//! What the core does with it (CE.Graph.effectiveFlags): a node named
//! here as exporting something carries flag bit 0, the public/private
//! axis Dead.deadTable has always split on and that no producer could
//! set — "bit 0 is not a file fact — public-ness is a symbol fact"
//! (deadcode/flags.rs:9 `symbol fact`). So verdict codes 2 and 4,
//! unref_public and unreach_public, become reachable for the first
//! time since 2.28.0. The bit is deliberately outside entryMask: an
//! export surface is a verdict axis, never an entry claim (RG10), so
//! it can change which code a dead node reports and can never change
//! which nodes are dead.

use crate::dedup::index::Index;
use crate::fourclass::visibility::VIS_EXPORTED;
use anyhow::{Context, Result};
use std::collections::{BTreeMap, BTreeSet, HashMap};

/// The `symbols` wire table: deduped `[nodeIdx, visibility]` pairs,
/// ascending, the visibility MASKED to bit 0.
///
/// The stored word is wider than the wire's copy (plan v2.17 L
/// round): `symbols.flags` also holds scope-exported and restricted
/// bits — stored facts for the `unmentioned` table, no verdict axis
/// of this family — so the projection happens here, in the SELECT,
/// and the core reads exactly the bit it read at 4.1.0 (K16/K34).
/// DISTINCT at the query, a set at the collect: two declarations of
/// equal visibility in one file are not two facts about that file's
/// export surface, the same reason Build.hs dedups arcs. The mask is
/// what makes that "two rows per file" rather than eight: `symCap` is
/// sized for a file to appear at most twice (5149 symbols become 750
/// rows here at most), and an unmasked word would let it appear once
/// per distinct stored value.
///
/// A symbol whose path is not a node is index skew, named the way
/// `edge_wire` names a skewed endpoint: symbols reference files by
/// row id and every walked file is a node, so a miss is a cache to
/// report, never a row to drop quietly.
pub fn export_surface(
    idx: &Index,
    ids: &BTreeMap<(&str, &str), usize>,
) -> Result<BTreeSet<[i64; 2]>> {
    let masked = format!(
        "SELECT DISTINCT f.path, s.flags & {VIS_EXPORTED} FROM symbols s
         JOIN files f ON f.id = s.file_id ORDER BY 1, 2"
    );
    super::load::rows(idx.raw(), &masked, |r| {
        Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?))
    })?
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
/// visibility crosses as `export_surface` projected it (bit 0 alone)
/// because which bit means "exported" is judgment
/// (`CE.Graph.Cost.exportVisBit`); a pre-decided `Vec<u>` would move
/// that call to the measurement side, and the projection withholds
/// only bits the core has no verdict axis for. Ascent comes free
/// (node ids ascend and file nodes keep
/// their order); a FOREIGN owner is outside the verdict universe by
/// design (deadcode::measured_nodes) and its row simply does not
/// cross, while an own owner that is not a file node is index skew,
/// refused the way `export_surface` refuses its own miss.
pub fn rekeyed(w: &super::deadcode::GraphWire, idx: &HashMap<&str, i64>) -> Result<Vec<[i64; 2]>> {
    w.symbols
        .iter()
        .filter_map(|&[node, vis]| {
            let owner = w
                .nodes
                .get(usize::try_from(node).unwrap_or(usize::MAX))
                .context("symbol row names a node outside the graph");
            match owner {
                Ok(n) if n.foreign => None,
                Ok(n) => Some(
                    idx.get(n.path.as_str())
                        .map(|&u| [u, vis])
                        .with_context(|| {
                            format!("export surface owner {} is not a file node", n.path)
                        }),
                ),
                Err(e) => Some(Err(e)),
            }
        })
        .collect()
}
