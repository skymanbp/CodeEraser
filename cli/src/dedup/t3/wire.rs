//! clone/1 wire codec (contracts/fixtures/clone/golden.ndjson is the
//! byte-level contract; corelink stamps proto/type/id). The caps are
//! MIRRORS of CE.Clone.Cost — the Haskell module owns the numbers,
//! and both drift directions are named at runtime: a mirror larger
//! than the core's makes the core degrade (asserted below), a
//! threshold drift breaks the knobs echo.

use super::tree::UnitTree;
use crate::dedup::candidates::{TSED_DEN, TSED_NUM};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::BTreeMap;

/// Capability name the core's hello must offer (Protocol.hs).
pub const CAP: &str = "clone/1";

/// Per-tree node ceiling — mirror of CE.Clone.Cost.unitNodeCap.
pub const UNIT_NODE_CAP: i64 = 256;

/// Per-request pair ceiling — mirror of CE.Clone.Cost.pairCap.
pub const PAIR_CAP: usize = 4096;

/// The request body for one chunk: trees with request-local DENSE
/// labels (first-seen order across the chunk's trees — the judge
/// only ever compares codes for equality), pairs as given (the
/// caller's sorted-rank locals keep the wire's strictly-ascending
/// row order).
pub fn request_body(trees: &[&UnitTree], pairs: &[[usize; 2]]) -> Value {
    let mut dense: BTreeMap<u64, i64> = BTreeMap::new();
    let rows: Vec<Value> = trees
        .iter()
        .map(|t| {
            let lab: Vec<i64> = t
                .lab
                .iter()
                .map(|k| {
                    let next = dense.len() as i64;
                    *dense.entry(*k).or_insert(next)
                })
                .collect();
            json!({"lab": lab, "lld": t.lld})
        })
        .collect();
    json!({"trees": rows, "pairs": pairs})
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
        chunk: PAIR_CAP,
    }
}

/// Decode one clone.result into request-local rows `(i, j, (ted, n1,
/// n2, verdict))` plus `[judged, prefiltered]` — the shared
/// parse_scores throat with this family's knob list (the prunes'
/// admissibility argument collapses if the judge's ratio drifts from
/// candidates.rs; a degraded reply to a client-sized request means
/// the cap mirrors above disagree with Cost.hs), zipped with the
/// core's per-row verdict bits (ADR-008 P1: the reported set is the
/// core's decision — raw ted stays for the instruments' cut tables).
pub fn parse_result(reply: &Value) -> Result<crate::lockstep::Scored<(i64, i64, i64, bool)>> {
    let (rows, c): (Vec<[i64; 5]>, _) = crate::lockstep::parse_scores(
        reply,
        &[("tsedNum", json!(TSED_NUM)), ("tsedDen", json!(TSED_DEN))],
        "t3/wire.rs+candidates.rs vs Clone/Cost.hs",
        &["judged", "prefiltered"],
    )?;
    let bits = crate::lockstep::verdict_bits(reply, rows.len())?;
    let local = rows
        .into_iter()
        .zip(bits)
        .map(|([i, j, ted, n1, n2], v)| (i as usize, j as usize, (ted, n1, n2, v)))
        .collect();
    Ok((local, c))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The dense mapping is request-scoped and first-seen ordered:
    /// two trees sharing kinds share codes, and codes never exceed
    /// the distinct-kind count (the judge compares for equality only,
    /// but small labels keep the wire inspectable). The generic pin
    /// battery lives with corelink; this family owns only its knob
    /// list, exercised through its own parse_result.
    #[test]
    fn dense_labels_are_request_scoped_and_knobs_pin() {
        let a = UnitTree {
            lab: vec![900, 700, 900],
            lld: vec![0, 1, 0],
        };
        let b = UnitTree {
            lab: vec![700, 800],
            lld: vec![0, 0],
        };
        let body = request_body(&[&a, &b], &[[0, 1]]);
        assert_eq!(body["trees"][0]["lab"], json!([0, 1, 0]));
        assert_eq!(body["trees"][1]["lab"], json!([1, 2]));
        assert_eq!(body["pairs"], json!([[0, 1]]));
        let ok = json!({"scores": [[0, 1, 2, 3, 3]], "verdicts": [false],
            "counts": {"judged": 1, "prefiltered": 0},
            "knobs": {"tsedNum": 85, "tsedDen": 100}, "degraded": false});
        assert_eq!(
            parse_result(&ok).expect("well-formed").0,
            vec![(0, 1, (2, 3, 3, false))]
        );
        let mut drifted = ok;
        drifted["knobs"]["tsedNum"] = json!(80);
        assert!(parse_result(&drifted).is_err(), "threshold drift refuses");
    }
}
