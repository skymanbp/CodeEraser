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
    assert_frozen_corpus_set, assert_identity_echo, eval_doc, generated_from, graph_corpus, load,
    review_doc, str_pairs, write_doc,
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
/// external_ok) — the frozen ledger of G8: growth needs blessing.
fn attention(rows: &[Value]) -> BTreeSet<String> {
    rows.iter()
        .filter(|r| !matches!(r["verdict"].as_str(), Some("correct") | Some("external_ok")))
        .map(|r| r["rank"].as_str().expect("rank").to_string())
        .collect()
}

/// G8: an existing doc's wrong/missed/unresolved ledger is frozen —
/// new attention rows need explicit blessing.
fn assert_ledger_frozen(path: &str, rows: &[Value]) {
    let Ok(old) = std::fs::read_to_string(path) else {
        return;
    };
    let old: Value = serde_json::from_str(&old).expect("old doc");
    let frozen = attention(old["rows"].as_array().expect("rows"));
    let grown: Vec<_> = attention(rows).difference(&frozen).cloned().collect();
    assert!(
        grown.is_empty() || std::env::var("CE_ACCEPT_GRAPH").as_deref() == Ok("1"),
        "wrong/missed/unresolved ledger grew ({grown:?}) — regressions need CE_ACCEPT_GRAPH=1 (G8)"
    );
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
    assert_ledger_frozen(&path, &rows);
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
    let truth = truths
        .get(rank)
        .unwrap_or_else(|| panic!("{corpus_key}: {rank} not in the audit"));
    assert_eq!(
        row["truth"].as_str(),
        Some(*truth),
        "{corpus_key}/{rank}: truth echo drifted"
    );
    let recomputed = parts::verdict_of(
        truth,
        row["answered"].as_str(),
        row["external"].as_bool().unwrap_or(false),
    );
    assert_eq!(
        row["verdict"].as_str(),
        Some(recomputed),
        "{corpus_key}/{rank}: stored verdict contradicts its own row (G6)"
    );
}

/// One corpus's precision doc checked end to end (G1/G3/G4/G6/G7).
fn check_precision_doc(corpus_key: &str, doc: &Value, sample: &Value) {
    let rows = doc["rows"].as_array().expect("rows");
    assert_eq!(
        doc["summary"],
        parts::rescore(rows),
        "{corpus_key}: summary drifted from rows (G1)"
    );
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
    let verdicts = doc["summary"]["verdicts"].as_object().expect("verdicts");
    let total: u64 = verdicts.values().map(|v| v.as_u64().expect("n")).sum();
    assert_eq!(total, rows.len() as u64, "{corpus_key}: conservation (G3)");
    for key in verdicts.keys() {
        assert!(
            parts::VERDICTS.contains(&key.as_str()),
            "{corpus_key}: unknown verdict class {key}"
        );
    }
}

/// Aggregates one doc into the cross-corpus tallies and applies the
/// per-corpus gate where the in-corpus GT denominator reaches 5.
fn corpus_gate(path: &str, sample: &Value, agg: &mut (u64, u64, u64, BTreeMap<String, u64>)) {
    let doc = load(path);
    let key = eval_support::doc_suffix(path, "graph-precision").unwrap_or_else(|| "self".into());
    check_precision_doc(&key, &doc, sample);
    let v = &doc["summary"]["verdicts"];
    let n = |k: &str| v[k].as_u64().unwrap_or(0);
    let (c, w) = (n("correct"), n("wrong"));
    agg.0 += c;
    agg.1 += w;
    agg.2 += doc["rows"].as_array().expect("rows").len() as u64;
    for (lang, verdicts) in doc["summary"]["by_lang"].as_object().expect("by_lang") {
        let n: u64 = verdicts
            .as_object()
            .expect("v")
            .values()
            .map(|x| x.as_u64().unwrap())
            .sum();
        *agg.3.entry(lang.clone()).or_insert(0) += n;
    }
    let in_truth = doc["summary"]["in_corpus_truths"].as_u64().expect("in");
    if in_truth >= 5 {
        let p = c as f64 / (c + w) as f64;
        assert!(
            p >= 0.90,
            "{key}: per-corpus precision {p:.3} < 0.90 on denominator {in_truth} (G2)"
        );
    }
}

/// The full gate: every frozen corpus doc present (G10), each
/// internally consistent, 100 judged rows in total (G3), per-language
/// floors (G5), and the precision contract (G2): overall >= 0.90,
/// per-corpus >= 0.90 where the in-corpus GT denominator reaches 5
/// (2026-08-12 decision — derived from the docs, not hardcoded).
#[test]
fn precision_meets_the_contract() {
    let docs = assert_frozen_corpus_set("graph-precision");
    let sample = load(&eval_doc("graph-sample"));
    let mut agg = (0u64, 0u64, 0u64, BTreeMap::new());
    for path in &docs {
        corpus_gate(path, &sample, &mut agg);
    }
    let (c_all, w_all, total, by_lang) = agg;
    assert_eq!(
        total, 100,
        "judged rows across corpora (G3, the contract 100)"
    );
    for (lang, n) in &by_lang {
        assert!(*n >= 15, "{lang}: {n} judged rows < per-lang floor 15 (G5)");
    }
    let overall = c_all as f64 / (c_all + w_all) as f64;
    assert!(
        overall >= 0.90,
        "overall precision {overall:.3} < 0.90 over the frozen 100 (G2 contract)"
    );
}

/// G9: every advertised refusal is exercised — a tampered doc must
/// redden the checker, not pass politely.
#[test]
fn precision_refuses_tampering() {
    let sample = load(&eval_doc("graph-sample"));
    let pristine = load(&eval_doc(&doc_stem(Some("zod"))));
    let refused =
        |doc: &Value| eval_support::doc_refused(doc, &|d| check_precision_doc("zod", d, &sample));
    assert!(!refused(&pristine), "pristine doc must pass");
    let mut flipped = pristine.clone();
    flipped["rows"][0]["verdict"] = Value::from("correct");
    flipped["rows"][0]["answered"] = Value::from("not/the/truth.ts");
    let mut phantom = pristine.clone();
    phantom["rows"][0]["rank"] = Value::from("not-a-sampled-rank");
    let mut dropped = pristine.clone();
    dropped["rows"].as_array_mut().expect("rows").remove(0);
    let mut cooked = pristine.clone();
    cooked["summary"]["verdicts"]["wrong"] = json!(0);
    for (label, doc) in [
        ("flipped verdict", &flipped),
        ("phantom rank", &phantom),
        ("dropped row", &dropped),
        ("cooked summary", &cooked),
    ] {
        assert!(refused(doc), "{label} must refuse (G9)");
    }
}
