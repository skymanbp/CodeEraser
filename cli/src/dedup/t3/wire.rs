//! clone/1 wire codec (contracts/fixtures/clone/golden.ndjson is the
//! byte-level contract; corelink stamps proto/type/id). The caps are
//! MIRRORS of CE.Clone.Cost — the Haskell module owns the numbers,
//! and both drift directions are named at runtime: a mirror larger
//! than the core's makes the core degrade (asserted below), a
//! threshold drift breaks the knobs echo.

use super::tree::UnitTree;
use crate::dedup::candidates::{TSED_DEN, TSED_NUM};
use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

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

/// One chunk's request-local layout: global unit ids by sorted rank
/// (the monotone map keeps the wire's strictly-ascending pair rows
/// for free) and the encoded body. Reply score rows map back through
/// the returned order. ONE throat — the product driver and the 3f
/// precision instrument both lay out chunks here, so the rank
/// discipline can never fork.
pub fn chunk_request<'t>(
    pairs: &[(usize, usize)],
    tree_of: impl Fn(usize) -> &'t UnitTree,
) -> (Vec<usize>, Value) {
    let order: Vec<usize> = pairs
        .iter()
        .flat_map(|&(a, b)| [a, b])
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    let rank: BTreeMap<usize, usize> = order.iter().enumerate().map(|(r, &g)| (g, r)).collect();
    let trees: Vec<&UnitTree> = order.iter().map(|&g| tree_of(g)).collect();
    let local: Vec<[usize; 2]> = pairs.iter().map(|&(a, b)| [rank[&a], rank[&b]]).collect();
    (order, request_body(&trees, &local))
}

/// One chunk's judged scores, in request-local tree indices.
pub struct Scores {
    pub rows: Vec<(usize, usize, i64, i64, i64)>,
    pub judged: u64,
    pub prefiltered: u64,
}

/// Decode one clone.result. The knobs echo pins the ONE threshold
/// across the language boundary (the prunes' admissibility argument
/// collapses if the judge's ratio drifts from candidates.rs); a
/// degraded reply to a client-sized request means the cap mirrors
/// above disagree with Cost.hs.
pub fn parse_result(reply: &Value) -> Result<Scores> {
    let knobs = &reply["knobs"];
    ensure!(
        knobs["tsedNum"] == json!(TSED_NUM) && knobs["tsedDen"] == json!(TSED_DEN),
        "core knobs {knobs} disagree with the Rust prunes' {TSED_NUM}/{TSED_DEN} — one threshold, two owners"
    );
    ensure!(
        reply["degraded"] == json!(false),
        "core degraded a client-sized request ({}) — cap mirror drift (t3/wire.rs vs Cost.hs)",
        reply["reason"]
    );
    let rows: Vec<[i64; 5]> = serde_json::from_value(reply["scores"].clone()).context("scores")?;
    Ok(Scores {
        rows: rows
            .into_iter()
            .map(|[i, j, ted, n1, n2]| (i as usize, j as usize, ted, n1, n2))
            .collect(),
        judged: reply["counts"]["judged"]
            .as_u64()
            .context("counts.judged")?,
        prefiltered: reply["counts"]["prefiltered"]
            .as_u64()
            .context("counts.prefiltered")?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The dense mapping is request-scoped and first-seen ordered:
    /// two trees sharing kinds share codes, and codes never exceed
    /// the distinct-kind count (the judge compares for equality only,
    /// but small labels keep the wire inspectable).
    #[test]
    fn dense_labels_are_request_scoped() {
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
    }

    /// Both runtime pins fire: a knob drift and a degraded reply are
    /// errors that NAME the owning module pair, never silent scores.
    #[test]
    fn result_pins_knobs_and_refuses_degraded() {
        let ok = json!({"scores": [[0, 1, 2, 3, 3]],
            "counts": {"judged": 1, "prefiltered": 0},
            "knobs": {"tsedNum": 85, "tsedDen": 100}, "degraded": false});
        let s = parse_result(&ok).expect("well-formed");
        assert_eq!(s.rows, vec![(0, 1, 2, 3, 3)]);
        let mut drifted = ok.clone();
        drifted["knobs"]["tsedNum"] = json!(80);
        assert!(parse_result(&drifted).is_err());
        let mut degraded = ok;
        degraded["degraded"] = json!(true);
        degraded["reason"] = json!("clone_too_large");
        assert!(parse_result(&degraded).is_err());
    }
}
