//! M5-2h precision instrument: the frozen 100-site sample, resolved
//! by the REAL ladder against the materialized pinned trees, judged
//! against the frozen audit ground truth (design §5). The universe
//! and the audit were frozen before any resolver existed (G13
//! ancestry in graph_provenance.rs), so the resolver never chooses
//! its own denominator.
//!
//! Generate (per corpus; external corpora via CE_SLICE_REPO +
//! CE_GRAPH_NAME + CE_GRAPH_TIP; re-blessing a grown wrong/missed
//! ledger needs CE_ACCEPT_GRAPH=1):
//!   cargo test --test eval_graph_precision -- --ignored --nocapture

mod eval_graph_precision_parts;
mod eval_support;

use eval_graph_precision_parts as parts;
use eval_support::{
    assert_identity_echo, eval_doc, generated_from, graph_corpus, load, review_doc, str_pairs,
    write_doc,
};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

fn doc_stem(name: Option<&str>) -> String {
    match name {
        Some(n) => format!("graph-precision-{n}"),
        None => "graph-precision".into(),
    }
}

fn slice_stem(name: Option<&str>) -> String {
    match name {
        Some(n) => format!("graph-slice-{n}"),
        None => "graph-slice".into(),
    }
}

/// Sample rows of one corpus, in frozen sample order (the shared
/// of_corpus filter under the graph sample's key).
fn corpus_rows<'a>(sample: &'a Value, corpus_key: &str) -> Vec<&'a Value> {
    eval_support::of_corpus(sample["rows"].as_array().expect("rows"), corpus_key)
}

/// Ranks whose verdict demands attention (everything but correct /
/// external_ok) — the frozen ledger of G8: growth needs blessing
/// (the shared eval_support ledger gate consumes this).
fn attention(rows: &[Value]) -> BTreeSet<String> {
    rows.iter()
        .filter(|r| !matches!(r["verdict"].as_str(), Some("correct") | Some("external_ok")))
        .map(|r| r["rank"].as_str().expect("rank").to_string())
        .collect()
}

#[test]
#[ignore] // needs the corpus repository (git show at the pinned tip)
fn generate_graph_precision() {
    let (name, tip) = graph_corpus();
    let corpus_key = name.clone().unwrap_or_else(|| "self".into());
    let slice = load(&eval_doc(&slice_stem(name.as_deref())));
    assert_eq!(
        slice["corpus"]["tip"].as_str(),
        Some(tip.as_str()),
        "pinned tip disagrees with the frozen slice — wrong tree never scores"
    );
    let sample = load(&eval_doc("graph-sample"));
    let review = review_doc(&corpus_key);
    let truths = str_pairs(&review, "rows", "rank", "truth");
    let mat = parts::materialize(&corpus_key, &tip, &slice);
    let scope = mat.scope();
    let rows: Vec<Value> = corpus_rows(&sample, &corpus_key)
        .into_iter()
        .map(|r| {
            let rank = r["rank"].as_str().expect("rank");
            let truth = truths
                .get(rank)
                .unwrap_or_else(|| panic!("{rank}: sampled but unaudited"));
            parts::judge_row(r, truth, &scope)
        })
        .collect();
    let path = eval_doc(&doc_stem(name.as_deref()));
    eval_support::assert_ledger_frozen(&path, &rows, &attention, "CE_ACCEPT_GRAPH");
    let doc = json!({
        "schema": "ce.eval-graph-precision/1.0.0",
        "corpus": {"name": name, "tip": tip},
        "generated_from": generated_from(),
        "method": "the frozen graph-sample rows of this corpus, resolved by the \
                   shipped ladder against the pinned tree materialized under a \
                   scratch root (every in-scope file sha-checked against the \
                   frozen slice — a polluted materialization never scores, G12), \
                   judged against the frozen audit truth. An in-corpus answer \
                   matches exactly, or at file level when the resolver made no \
                   unit claim (GT definition units are extra precision; symbol \
                   binding is the future R5 stance). External is an answer — \
                   external on an in-corpus truth is wrong, never missed. The \
                   universe ledger runs the ladder over EVERY site of the frozen \
                   universe: its resolution rate is a recall CEILING — constructs \
                   the detector cannot see are in no denominator. Cut rule \
                   (pre-registered): publish the full per-rung table; the loosest \
                   minRung clearing 0.90 is the release knob.",
        "summary": parts::rescore(&rows),
        "universe": parts::universe_ledger(&mat),
        "rows": rows,
    });
    write_doc(&path, &doc, &format!("{path} written"));
}

/// One judged row against its sampled identity and frozen truth:
/// G4 bijection + echo, G6 verdict self-falsification, G7 dups.
fn check_row<'a>(
    corpus_key: &str,
    row: &'a Value,
    sampled: &BTreeMap<&'a str, &'a Value>,
    truths: &BTreeMap<&str, &str>,
    seen: &mut BTreeSet<&'a str>,
) {
    let (rank, s) = eval_support::bijective_row(corpus_key, row, sampled, seen);
    assert_identity_echo(corpus_key, rank, row, s);
    eval_support::assert_row_verdict(corpus_key, rank, row, truths, |truth| {
        parts::verdict_of(
            truth,
            row["answered"].as_str(),
            row["external"].as_bool().unwrap_or(false),
        )
    });
}

/// One corpus's precision doc checked end to end (G1/G3/G4/G6/G7).
fn check_precision_doc(corpus_key: &str, doc: &Value, sample: &Value) {
    eval_support::assert_summary_rederived(corpus_key, doc, parts::rescore);
    let rows = doc["rows"].as_array().expect("rows");
    let sampled: BTreeMap<&str, &Value> = corpus_rows(sample, corpus_key)
        .into_iter()
        .map(|r| (r["rank"].as_str().expect("rank"), r))
        .collect();
    assert_eq!(rows.len(), sampled.len(), "{corpus_key}: row count (G3)");
    let review = review_doc(corpus_key);
    let truths = str_pairs(&review, "rows", "rank", "truth");
    let mut seen = BTreeSet::new();
    for row in rows {
        check_row(corpus_key, row, &sampled, &truths, &mut seen);
    }
    eval_support::assert_verdict_conservation(corpus_key, doc, &parts::VERDICTS);
}

/// Aggregates one doc into the cross-corpus tallies and applies the
/// per-corpus gate where the in-corpus GT denominator reaches 5
/// (this family's G2 semantics; the T3 family gates on answered).
fn corpus_gate(path: &str, sample: &Value, agg: &mut eval_support::PrecisionAgg) {
    let (key, doc, c, w) =
        eval_support::open_scored_doc(path, "graph-precision", sample, agg, check_precision_doc);
    agg.rows += doc["rows"].as_array().expect("rows").len() as u64;
    for (lang, verdicts) in doc["summary"]["by_lang"].as_object().expect("by_lang") {
        let n: u64 = verdicts
            .as_object()
            .expect("v")
            .values()
            .map(|x| x.as_u64().unwrap())
            .sum();
        eval_support::tally_add(&mut agg.by_lang, lang, n);
    }
    let in_truth = doc["summary"]["in_corpus_truths"].as_u64().expect("in");
    eval_support::assert_corpus_precision(&key, (c, w), in_truth, 0.90, "G2");
}

const SPEC: eval_support::FamilySpec = eval_support::FamilySpec {
    family: "graph-precision",
    sample: "graph-sample",
    gate: 0.90,
    tag: "G2 contract",
    mutations: &[("rank", "not-a-sampled-rank", "phantom rank")],
};

/// The full gate in one battery: every frozen corpus doc present
/// (G10), each internally consistent, 100 judged rows (G3),
/// per-language floors (G5), the precision contract (G2: overall and
/// per-corpus >= 0.90 where the in-corpus GT denominator reaches 5 —
/// the 2026-08-12 decision), then G9's refusals — pristine, phantom
/// and dropped in the shared battery; the two-field flip and cooked
/// summary as this family's bespoke cases.
#[test]
fn precision_contract_and_refusals() {
    eval_support::assert_precision_family(
        &SPEC,
        corpus_gate,
        &check_precision_doc,
        |pristine, refused| {
            let mut flipped = pristine.clone();
            flipped["rows"][0]["verdict"] = Value::from("correct");
            flipped["rows"][0]["answered"] = Value::from("not/the/truth.ts");
            let mut cooked = pristine.clone();
            cooked["summary"]["verdicts"]["wrong"] = json!(0);
            for (label, doc) in [("flipped verdict", &flipped), ("cooked summary", &cooked)] {
                assert!(refused(doc), "{label} must refuse (G9)");
            }
        },
    );
}
