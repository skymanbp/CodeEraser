//! M5-3b t3-universe instrument: the frozen UNIT universe per corpus
//! (design vol.3 §9.2) — file inventory with content identity,
//! per-language symbol counts (the identity universe; md sections
//! included) and per-language code-unit counts stratified by the
//! pre-registered sizebands. Frozen BEFORE any candidate generator
//! or TED judge exists (3c/3e), so neither can ever choose its own
//! denominator (RM2). Anchored byte-for-byte to the graph-slice
//! sibling: one pinned tip, one tree, two frozen views of it. The
//! family skeleton is eval_support/family.rs.
//!
//! Generate (per corpus; external corpora via CE_SLICE_REPO +
//! CE_GRAPH_NAME + CE_GRAPH_TIP; release for an admissible PERF line):
//!   cargo test --release --test eval_t3_universe -- --ignored --nocapture

mod eval_support;

use eval_support::*;
use std::collections::BTreeMap;

const METHOD: &str = "unit universe of the pinned tree: every in-scope file \
    inventoried with its symbols identity count (the detect_with_units + \
    with_nth throat the graph store writes) and its code-unit count by \
    named-node sizeband (the unitcache::unit_facts throat the unitsig \
    cache writes; markdown carries zero units by design — text \
    duplication is docdup's domain). below_floor units are structurally \
    invisible to the T3 candidate pass, so candidate recall is an upper \
    bound. Frozen before any candidate generator or TED judge exists; \
    sizeband constants pre-registered before any measurement.";

const FAMILY: UniverseFamily = UniverseFamily {
    family: "t3-universe",
    schema: "ce.eval-t3-universe/1.0.0",
    method: METHOD,
    row: t3_row,
    constants: t3_constants,
    summarize: t3_summarize,
};

/// CI gate, no git: the family skeleton asserts the envelope and the
/// graph-slice sibling anchor per doc; this family then requires five
/// languages covered with symbols, four code languages with units,
/// and markdown at zero units (the T3/docdup domain split).
#[test]
fn t3_universe_consistent() {
    let mut symbols_by: BTreeMap<String, u64> = BTreeMap::new();
    let mut units_by: BTreeMap<String, u64> = BTreeMap::new();
    FAMILY.each_consistent(|_path, doc| {
        for (by, key) in [(&mut symbols_by, "symbols_by"), (&mut units_by, "units_by")] {
            sum_obj_into(&doc["summary"][key], by);
        }
    });
    assert_covered(&symbols_by, SCOPE_EXTS, "symbols");
    assert_eq!(
        units_by.get("md").copied().unwrap_or(0),
        0,
        "markdown grew units — the domain split drifted"
    );
    assert_covered(
        &units_by,
        SCOPE_EXTS.iter().copied().filter(|l| *l != "md"),
        "units",
    );
}

/// The unit/symbol throats and the frozen self universe must not
/// drift apart silently (whole-row byte equality; ordinary churn
/// shrinks the verifiable set between freezes — the floor only
/// guards against a vacuous gate).
#[test]
fn self_universe_tracks_units() {
    assert_self_tracks("t3-universe", t3_row, 25);
}
