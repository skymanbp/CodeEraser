//! M5-3b t3-universe instrument: the frozen UNIT universe per corpus
//! (design vol.3 §9.2) — file inventory with content identity,
//! per-language symbol counts (the identity universe; md sections
//! included) and per-language code-unit counts stratified by the
//! pre-registered sizebands. Frozen BEFORE any candidate generator
//! or TED judge exists (3c/3e), so neither can ever choose its own
//! denominator (RM2). Anchored byte-for-byte to the graph-slice
//! sibling: one pinned tip, one tree, two frozen views of it. The
//! instrument skeleton is eval_support/universe.rs, shared with the
//! slice family.
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

fn build_doc(
    name: &Option<String>,
    tip: &str,
    walked: &[WalkedFile],
    excluded: &BTreeMap<&'static str, u64>,
) -> Value {
    let parts = universe_parts(
        walked,
        excluded.clone(),
        t3_row,
        t3_constants(),
        t3_summarize,
    );
    universe_doc("ce.eval-t3-universe/1.0.0", METHOD, name, tip, parts)
}

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
    let (name, tip) = graph_corpus();
    let slice = load(&eval_doc(&doc_stem("graph-slice", &name)));
    assert_eq!(
        slice["corpus"]["tip"].as_str().expect("tip"),
        tip,
        "the t3 universe must freeze the slice's pinned tip"
    );
    let (walked, excluded) = walk_tree(&tip);
    assert_slice_identity(&slice, &walked);
    let doc = build_doc(&name, &tip, &walked, &excluded);
    let again = {
        let (w2, e2) = walk_tree(&tip);
        build_doc(&name, &tip, &w2, &e2)
    };
    assert_eq!(
        serde_json::to_string(&doc).expect("ser"),
        serde_json::to_string(&again).expect("ser"),
        "double run diverged — the doc is not a function of the tree"
    );
    index_materialized(name.as_deref().unwrap_or("self"), &walked, &doc);
    write_universe("t3-universe", &name, &doc);
}

/// CI gate, no git, every frozen universe: the shared envelope
/// (summary re-derived via the generator's own scorer with per-file
/// band conservation, frozen constants and scope, pinned tip, sorted
/// rows) plus the family's own facts — each doc anchored
/// byte-for-byte to its graph-slice sibling (same tip, same
/// path→sha256 inventory, same exclusion tally: the CI-persistent
/// form of the generation-time RG3 proof), five languages covered
/// with symbols, four code languages with units, markdown at zero
/// units (the T3/docdup domain split).
#[test]
fn t3_universe_consistent() {
    let mut symbols_by: BTreeMap<String, u64> = BTreeMap::new();
    let mut units_by: BTreeMap<String, u64> = BTreeMap::new();
    each_frozen_doc("t3-universe", |path, doc| {
        let name = assert_doc_envelope(path, doc, "t3-universe", t3_summarize, t3_constants);
        let slice = load(&eval_doc(&doc_stem("graph-slice", &name)));
        assert_eq!(
            doc["corpus"]["tip"], slice["corpus"]["tip"],
            "{path}: tip differs from the slice sibling"
        );
        assert_eq!(
            str_pairs(doc, "files", "path", "sha256"),
            str_pairs(&slice, "files", "path", "sha256"),
            "{path}: inventory differs from the slice sibling (one tip, two trees?)"
        );
        assert_eq!(
            doc["excluded"], slice["excluded"],
            "{path}: exclusion tally differs from the slice sibling"
        );
        for (by, key) in [(&mut symbols_by, "symbols_by"), (&mut units_by, "units_by")] {
            for (lang, n) in doc["summary"][key].as_object().expect(key) {
                *by.entry(lang.clone()).or_insert(0) += n.as_u64().expect("n");
            }
        }
    });
    for lang in SCOPE_EXTS {
        assert!(
            symbols_by.get(lang).copied().unwrap_or(0) > 0,
            "no {lang} symbols across frozen universes"
        );
        let units = units_by.get(lang).copied().unwrap_or(0);
        if lang == "md" {
            assert_eq!(units, 0, "markdown grew units — the domain split drifted");
        } else {
            assert!(units > 0, "no {lang} units across frozen universes");
        }
    }
}

/// The unit/symbol throats and the frozen self universe must not
/// drift apart silently (the self_universe_tracks_detector posture,
/// unit edition): every sha-matched row is re-derived through the
/// shared t3_row throat — whole-row byte equality, not just counts.
#[test]
fn self_universe_tracks_units() {
    let doc = load(&eval_doc("t3-universe"));
    let verified = each_frozen_match(&doc, |row, path, lang, text| {
        assert_eq!(
            row,
            &t3_row(path, lang, text),
            "{path}: unit throats drifted from the frozen universe"
        );
    });
    // ordinary churn shrinks the verifiable set between freezes;
    // this floor only guards against the gate going vacuous
    assert!(verified >= 25, "drift gate near-vacuous: {verified} rows");
}
