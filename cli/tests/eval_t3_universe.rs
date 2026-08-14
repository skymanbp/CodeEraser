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
use serde_json::{Value, json};
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

/// The RG3 proof at generation time: the frozen slice doc and this
/// walk saw byte-identical trees (same path set, same per-file
/// sha256), and the CURRENT detector still reproduces the frozen
/// per-file site counts and the frozen sites_by summary on those
/// bytes — the 3b exit criterion "five-corpus site counts identical
/// to the existing slice docs", asserted rather than assumed.
fn assert_slice_identity(slice: &Value, walked: &[WalkedFile]) {
    let frozen = str_pairs(slice, "files", "path", "sha256");
    assert_eq!(
        frozen.keys().copied().collect::<Vec<_>>(),
        walked
            .iter()
            .map(|(p, _, _)| p.as_str())
            .collect::<Vec<_>>(),
        "walked path set != frozen slice path set"
    );
    let rows = by_field(slice, "files", "path");
    let mut sites_by: BTreeMap<String, u64> = BTreeMap::new();
    for (path, code, text) in walked {
        assert_eq!(
            frozen[path.as_str()],
            content_sha(text),
            "{path}: content drifted from the frozen slice"
        );
        let kinds = kind_counts(&codeeraser::graph::sites::detect(text, lang_of(code)));
        assert_eq!(
            rows[path.as_str()]["sites"],
            json!(kinds),
            "{path}: recomputed sites != frozen slice row (RG3)"
        );
        for (kind, n) in kinds {
            *sites_by.entry(format!("{code}/{kind}")).or_insert(0) += n;
        }
    }
    assert_eq!(
        slice["summary"]["sites_by"],
        json!(sites_by),
        "recomputed sites_by != frozen slice summary (RG3)"
    );
}

#[test]
#[ignore] // needs the corpus repository (git show at the pinned tip)
fn generate_t3_universe() {
    let (name, doc, walked) = FAMILY.generate("graph-slice", assert_slice_identity);
    index_materialized(name.as_deref().unwrap_or("self"), &walked, &doc);
    write_universe(FAMILY.family, &name, &doc);
}

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
