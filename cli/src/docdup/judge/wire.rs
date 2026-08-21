//! docdup/1 wire codec (contracts/fixtures/docdup/golden.ndjson is
//! the byte-level contract; corelink stamps proto/type/id). The caps
//! and the Jaccard ratio are MIRRORS of CE.Docdup.Cost — the Haskell
//! module owns the numbers, and both drift directions are named at
//! runtime: a mirror larger than the core's makes the core degrade
//! (asserted below), a ratio, shingle-width or verbatim-floor drift
//! breaks the knobs echo (D13/F29: the estimator's alphabet geometry
//! and the judge's thresholds each have exactly one owner and one
//! pinned mirror — verbatimFloor joined the pinned set at ADR-008 P1
//! when its verdict home moved to the core).

use crate::docdup::spec::{DOC_SHINGLE, VERBATIM_FLOOR};
use anyhow::Result;
use serde_json::{Value, json};

/// Capability name the core's hello must offer (Protocol.hs).
pub const CAP: &str = "docdup/1";

/// Per-set element ceiling — mirror of CE.Docdup.Cost.docSetCap.
pub const DOC_SET_CAP: usize = 8192;

/// Per-request pair ceiling — mirror of CE.Docdup.Cost.docPairCap.
pub const DOC_PAIR_CAP: usize = 4096;

/// The Jaccard report ratio — mirror of CE.Docdup.Cost.jaccardNum/
/// jaccardDen, and byte-equal to the instrument side's pre-registered
/// JACCARD_REPORT_FLOOR (frozen with the sample census before the
/// judge existed).
pub const JACCARD_NUM: u64 = 80;
pub const JACCARD_DEN: u64 = 100;

/// One chunk's request-local layout: global segment ids by the shared
/// sorted-rank throat (corelink) and the encoded body carrying each
/// pair's Rust-computed verbatim run (F26: one wire transcript holds
/// the complete verdict inputs). ONE throat — the product driver and
/// the precision instrument both lay out chunks here.
pub fn chunk_request<'s>(
    pairs: &[(usize, usize, u64)],
    set_of: impl Fn(usize) -> &'s [u64],
) -> (Vec<usize>, Value) {
    let (order, rank) = crate::lockstep::sorted_rank(pairs.iter().map(|&(a, b, _)| (a, b)));
    let sets: Vec<&[u64]> = order.iter().map(|&g| set_of(g)).collect();
    let local: Vec<[Value; 3]> = pairs
        .iter()
        .map(|&(a, b, run)| [json!(rank[&a]), json!(rank[&b]), json!(run)])
        .collect();
    (order, json!({"sets": sets, "pairs": local}))
}

/// Decode one docdup.result into request-local rows `(i, j, (inter,
/// union, verdict))` plus `[judged, jaccardDups]` — the shared
/// parse_scores throat with this family's knob list, which pins
/// every single-owner number: the 80/100 ratio, shingleK ==
/// DOC_SHINGLE (D13 — two sides shingling at different widths would
/// compare incommensurable alphabets and no downstream gate could
/// tell), and since ADR-008 P1 verbatimFloor == VERBATIM_FLOOR (the
/// floor's verdict home moved to Docdup/Cost.hs; the Rust constant
/// is the instruments' mirror). The rows zip with the core's
/// per-row verdict bits — the reported set is the core's decision.
pub fn parse_result(reply: &Value) -> Result<crate::lockstep::Scored<(u64, u64, bool)>> {
    let (rows, c): (Vec<[u64; 4]>, _) = crate::lockstep::parse_scores(
        reply,
        &[
            ("jaccardNum", json!(JACCARD_NUM)),
            ("jaccardDen", json!(JACCARD_DEN)),
            ("shingleK", json!(DOC_SHINGLE)),
            ("verbatimFloor", json!(VERBATIM_FLOOR)),
            ("minDocTokens", json!(crate::docdup::spec::MIN_DOC_TOKENS)),
        ],
        "judge/wire.rs vs Docdup/Cost.hs (shingleK: D13 alphabet geometry)",
        &["judged", "jaccardDups"],
    )?;
    let bits = crate::lockstep::verdict_bits(reply, rows.len())?;
    Ok((
        rows.into_iter()
            .zip(bits)
            .map(|([i, j, inter, union], v)| (i as usize, j as usize, (inter, union, v)))
            .collect(),
        c,
    ))
}

/// This family's corelink bindings for the shared lockstep machine.
pub fn family(core: &str) -> crate::lockstep::Family<'_> {
    crate::lockstep::Family {
        core,
        cap: CAP,
        kind: "docdup",
        chunk: DOC_PAIR_CAP,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Sorted-rank locals: sets are deduplicated across pairs and
    /// rows reference them by rank; the run rides each row verbatim.
    /// The generic pin battery lives with corelink; this family owns
    /// the D13 shingle-width pin, exercised through its parse_result.
    #[test]
    fn chunk_request_dedups_sets_and_shingle_width_pins() {
        let sets: Vec<Vec<u64>> = vec![vec![1, 2], vec![3, 4], vec![5, 6]];
        let (order, body) = chunk_request(&[(2, 0, 7), (2, 1, 0)], |g| &sets[g]);
        assert_eq!(order, vec![0, 1, 2]);
        assert_eq!(body["sets"], json!([[1, 2], [3, 4], [5, 6]]));
        assert_eq!(body["pairs"], json!([[2, 0, 7], [2, 1, 0]]));
        let ok = json!({"scores": [[0, 1, 2, 4]], "verdicts": [false],
            "counts": {"judged": 1, "jaccardDups": 0},
            "knobs": {"jaccardNum": 80, "jaccardDen": 100, "shingleK": 5,
                "verbatimFloor": 50, "minDocTokens": 50},
            "degraded": false});
        assert_eq!(
            parse_result(&ok).expect("well-formed").0,
            vec![(0, 1, (2, 4, false))]
        );
        let mut drifted = ok;
        drifted["knobs"]["shingleK"] = json!(4);
        assert!(
            parse_result(&drifted).is_err(),
            "alphabet-geometry drift refuses (D13)"
        );
    }
}
