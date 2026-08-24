//! M5-3d docdup-segments instrument: the frozen document-segment
//! universe per corpus (design vol.2 §5.1-5.2, instruments §9.5) —
//! file inventory with content identity, admitted segments by kind,
//! the four-route exemption ledger and a per-file digest over the
//! full shingle sets. Frozen BEFORE any docdup judge exists (3g), so
//! the coarse filter can never choose its own denominator. The
//! family skeleton is eval_support/family.rs.
//!
//! Generate (per corpus; external corpora via CE_SLICE_REPO +
//! CE_GRAPH_NAME + CE_GRAPH_TIP; release):
//!   cargo test --release --test eval_docdup_universe -- --ignored --nocapture

mod eval_support;

use eval_support::*;
use std::collections::BTreeMap;

const FAMILY: UniverseFamily = UniverseFamily {
    family: "docdup-segments",
    constants: docdup_constants,
    summarize: docdup_summarize,
};

/// CI gate, no git: the family skeleton asserts the envelope (summary
/// re-derived with row-level conservation, frozen constants and
/// scope, pinned tip, sorted rows) and the graph-slice sibling
/// anchor; this family then requires every segment kind alive
/// somewhere across the five corpora and every exemption route
/// either counted or explained in route_notes.
#[test]
fn docdup_segments_consistent() {
    // kinds, exemption classes and ledger keys share one namespace-
    // disjoint accumulator across the five corpora
    let mut totals: BTreeMap<String, u64> = BTreeMap::new();
    FAMILY.each_consistent(|_path, doc| {
        for key in ["segs_by", "exempt_by", "ledger"] {
            sum_obj_into(&doc["summary"][key], &mut totals);
        }
    });
    assert_covered(&totals, codeeraser::docdup::spec::KIND_NAMES, "segments");
    let notes = docdup_constants()["route_notes"].clone();
    for route in ["license_header", "inline_allow", "skeleton_line"] {
        assert!(
            totals.get(route).copied().unwrap_or(0) > 0 || notes[route].as_str().is_some(),
            "exemption route {route} is zero everywhere and unexplained"
        );
    }
}

/// The extraction throat and the frozen self universe must not drift
/// apart silently (ordinary churn shrinks the verifiable set between
/// freezes; the floor only guards against a vacuous gate).
#[test]
fn self_docdup_tracks_segments() {
    assert_self_tracks("docdup-segments", docdup_row, 25);
}
