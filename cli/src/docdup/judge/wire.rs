//! docdup/1 wire codec (contracts/fixtures/docdup/golden.ndjson is
//! the byte-level contract; corelink stamps proto/type/id). The caps
//! and the Jaccard ratio are CE.Docdup.Cost's, read off the core's
//! package (plan v2.33 W3, `limits`); the reply's knob echo still pins
//! the judging core to the package this run read — the ratio, the
//! shingle width (D13/F29: the estimator's alphabet geometry and the
//! judge's thresholds each have exactly one owner) and the verbatim
//! floor.

use crate::docdup::spec::table;
use anyhow::Result;
use serde_json::{Value, json};

/// Capability name the core's hello must offer (Protocol.hs).
pub const CAP: &str = "docdup/1";

/// The docdup judgment's ratio and ceilings (CE.Docdup.Cost), off the
/// package. The ratio is byte-equal to the instrument side's
/// pre-registered JACCARD_REPORT_FLOOR (frozen with the sample census
/// before the judge existed).
pub fn limits() -> &'static crate::tables::DocdupLimits {
    &crate::tables::get().limits.docdup
}

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
/// the package's doc_shingle (D13 — two sides shingling at different widths would
/// compare incommensurable alphabets and no downstream gate could
/// tell), and since ADR-008 P1 verbatimFloor == verbatim_floor (the
/// floor's verdict home moved to Docdup/Cost.hs; since plan v2.32
/// step 2 the measuring side reads its copy from the package). The rows zip with the core's
/// per-row verdict bits — the reported set is the core's decision.
pub fn parse_result(reply: &Value) -> Result<crate::lockstep::Scored<(u64, u64, bool)>> {
    crate::lockstep::parse_scores(
        reply,
        &[
            ("jaccardNum", json!(limits().jaccard_num)),
            ("jaccardDen", json!(limits().jaccard_den)),
            ("shingleK", json!(table().doc_shingle)),
            ("verbatimFloor", json!(table().verbatim_floor)),
            ("minDocTokens", json!(table().min_doc_tokens)),
            ("docLineCap", json!(table().doc_line_cap)),
            ("licHeadLines", json!(table().license_head_lines)),
        ],
        "the package's docdup numbers vs the judging core's Docdup/Cost.hs (shingleK: D13 alphabet geometry)",
        &["judged", "jaccardDups"],
        |[i, j, inter, union]: [u64; 4], v| (i as usize, j as usize, (inter, union, v)),
    )
}

/// This family's corelink bindings for the shared lockstep machine.
pub fn family(core: &str) -> crate::lockstep::Family<'_> {
    crate::lockstep::Family {
        core,
        cap: CAP,
        kind: "docdup",
        chunk: limits().doc_pair_cap,
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/docdup/judge/wire.rs"]
mod tests;
