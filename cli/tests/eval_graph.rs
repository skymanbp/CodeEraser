//! M5-2 graph-slice instrument: the frozen SITE universe per corpus
//! (design brief docs/reviews/2026-08-12-m5-2-graph-design.md §5).
//! The universe is sites, not edges — frozen BEFORE any resolver
//! exists, so the resolver can never choose its own precision
//! denominator; the falsification constants (min_per_lang,
//! r0_share_trigger) are written into the doc before any measurement
//! exists. The instrument skeleton (walker, envelope, gate opening,
//! drift walk) lives in eval_support/universe.rs, shared with the
//! M5-3 t3-universe family.
//!
//! Generate (per corpus; external corpora via CE_SLICE_REPO +
//! CE_GRAPH_NAME + CE_GRAPH_TIP):
//!   cargo test --test eval_graph -- --ignored --nocapture

mod eval_support;

use codeeraser::graph::sites::detect;
use eval_support::*;
use serde_json::{Value, json};
use std::collections::BTreeMap;

/// Pre-registered falsification constants (design §5), one binding
/// for generator AND gate — duplicated literals were the exact
/// throat-drift shape this file polices elsewhere (Opus review).
fn constants() -> Value {
    json!({"min_per_lang": 15, "r0_share_trigger": 0.80})
}

/// Re-derivable from the rows alone — the CI gate re-runs this exact
/// function (the G1 discipline: generator and gate share one scorer).
fn summarize(files: &[Value]) -> Value {
    let mut by: BTreeMap<String, u64> = BTreeMap::new();
    let mut total = 0;
    for f in files {
        for (kind, n) in f["sites"].as_object().expect("sites") {
            let n = n.as_u64().expect("count");
            let key = format!("{}/{kind}", f["lang"].as_str().expect("lang"));
            *by.entry(key).or_insert(0) += n;
            total += n;
        }
    }
    json!({"files": files.len(), "total_sites": total, "sites_by": by})
}

/// CI gate, no git, every frozen slice: the shared envelope (summary
/// re-derived by the generator's own scorer, frozen constants and
/// scope, pinned tip, sorted rows) plus the family's own coverage
/// fact — the frozen docs jointly cover all five languages with
/// sites (D2-4 — the 2b exit criterion).
#[test]
fn graph_slice_consistent() {
    let mut lang_sites: BTreeMap<String, u64> = BTreeMap::new();
    each_frozen_doc("graph-slice", |path, doc| {
        assert_doc_envelope(path, doc, "graph-slice", summarize, constants);
        for (langkind, n) in doc["summary"]["sites_by"].as_object().expect("sites_by") {
            let lang = langkind.split('/').next().expect("lang/kind");
            *lang_sites.entry(lang.to_string()).or_insert(0) += n.as_u64().expect("n");
        }
    });
    eval_support::assert_nonzero_seats(
        &lang_sites,
        &SCOPE_EXTS,
        "no sites in any frozen slice (D2-4)",
    );
}

/// The detector and the frozen self universe must not drift apart
/// silently (design RG3 made a CI fact — Opus review: a spec.rs or
/// md.rs change used to invalidate all five docs with zero red
/// signal). Every sha-matched row is re-detected: per-kind counts
/// must match, and every detected spec must be a substring of its
/// statement window (the 2b exit criterion, previously asserted only
/// on toy fixtures).
#[test]
fn self_universe_tracks_detector() {
    let doc = load(&eval_doc("graph-slice"));
    let verified = each_frozen_match(&doc, |row, path, lang, text| {
        let sites = detect(text, lang_of(lang));
        let lines: Vec<&str> = text.lines().collect();
        for s in &sites {
            // the site line is the STATEMENT HEAD: a multi-line TS
            // import carries its full specifier on a later line of
            // the same statement (2c/2d review F1 — 14 frozen zod
            // sites), so the anti-invention property is "spec within
            // the statement window", not "spec on the head line"
            let end = (s.line + 15).min(lines.len());
            assert!(
                lines[s.line - 1..end].iter().any(|l| l.contains(&s.spec)),
                "{path}:{}: spec {:?} not within its statement window",
                s.line,
                s.spec
            );
        }
        assert_eq!(
            row["sites"],
            json!(kind_counts(&sites)),
            "{path}: detector drifted from the frozen universe"
        );
    });
    // ordinary churn shrinks the verifiable set between freezes;
    // this floor only guards against the gate going vacuous
    assert!(verified >= 25, "drift gate near-vacuous: {verified} rows");
}
