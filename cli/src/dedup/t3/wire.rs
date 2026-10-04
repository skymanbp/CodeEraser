//! clone/1 wire codec (contracts/fixtures/clone/golden.ndjson is the
//! byte-level contract; corelink stamps proto/type/id). The caps and
//! the threshold are CE.Clone.Cost's, read off the core's package
//! (plan v2.33 W3, `candidates::limits`); the reply's knob echo still
//! pins the judging core to the package this run read, and a degraded
//! reply to a request laid out by the package's own caps is refused.

use super::tree::UnitTree;
use crate::dedup::candidates::limits;
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::BTreeMap;

/// Capability name the core's hello must offer (Protocol.hs).
pub const CAP: &str = "clone/1";

/// The request body for one chunk: trees with request-local DENSE
/// labels (first-seen order across the chunk's trees — the judge
/// only ever compares codes for equality), pairs as given (the
/// caller's sorted-rank locals keep the wire's strictly-ascending
/// row order).
pub fn request_body(trees: &[&UnitTree], pairs: &[[usize; 2]]) -> Value {
    let rows: Vec<Value> = dense(trees)
        .into_iter()
        .zip(trees)
        .map(|(lab, t)| json!({"lab": lab, "lld": t.lld}))
        .collect();
    json!({"trees": rows, "pairs": pairs})
}

/// Each tree's labels mapped request-locally to dense codes in
/// first-seen order across the trees — one mapping for every family
/// whose trees ride clone/1's encoding (merge/1 compares them too).
pub fn dense(trees: &[&UnitTree]) -> Vec<Vec<i64>> {
    let mut dense: BTreeMap<u64, i64> = BTreeMap::new();
    trees
        .iter()
        .map(|t| {
            t.lab
                .iter()
                .map(|k| {
                    let next = dense.len() as i64;
                    *dense.entry(*k).or_insert(next)
                })
                .collect()
        })
        .collect()
}

/// One chunk's request-local layout: global unit ids by the shared
/// sorted-rank throat (corelink) and the encoded body. Reply score
/// rows map back through the returned order. ONE throat — the product
/// driver and the 3f precision instrument both lay out chunks here,
/// so the rank discipline can never fork.
pub fn chunk_request<'t>(
    pairs: &[(usize, usize)],
    tree_of: impl Fn(usize) -> &'t UnitTree,
) -> (Vec<usize>, Value) {
    let (order, rank) = crate::lockstep::sorted_rank(pairs.iter().copied());
    let trees: Vec<&UnitTree> = order.iter().map(|&g| tree_of(g)).collect();
    let local: Vec<[usize; 2]> = pairs.iter().map(|&(a, b)| [rank[&a], rank[&b]]).collect();
    (order, request_body(&trees, &local))
}

/// This family's corelink bindings — capability, request kind, chunk
/// ceiling — for the shared lockstep machine.
pub fn family(core: &str) -> crate::lockstep::Family<'_> {
    crate::lockstep::Family {
        core,
        cap: CAP,
        kind: "clone",
        chunk: limits().pair_cap,
    }
}

/// Decode one clone.result into request-local rows `(i, j, (ted, n1,
/// n2, verdict))` plus `[judged, prefiltered]` — the shared
/// parse_scores throat with this family's knob list (the prunes'
/// admissibility argument collapses if the judge's ratio drifts from
/// the one the candidate pass bounded by; a degraded reply to a
/// package-sized request means two cores answered one run), zipped with the
/// core's per-row verdict bits (ADR-008 P1: the reported set is the
/// core's decision — raw ted stays for the instruments' cut tables).
pub fn parse_result(reply: &Value) -> Result<crate::lockstep::Scored<(i64, i64, i64, bool)>> {
    crate::lockstep::parse_scores(
        reply,
        &[
            ("tsedNum", json!(limits().tsed_num)),
            ("tsedDen", json!(limits().tsed_den)),
            ("minUnitNodes", json!(limits().min_unit_nodes)),
        ],
        "the package's clone limits vs the judging core's Clone/Cost.hs",
        &["judged", "prefiltered"],
        |[i, j, ted, n1, n2]: [i64; 5], v| (i as usize, j as usize, (ted, n1, n2, v)),
    )
}

#[cfg(test)]
#[path = "../../../tests/unit/dedup/t3/wire.rs"]
mod tests;
