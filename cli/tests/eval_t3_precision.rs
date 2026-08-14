//! M5-3f precision instrument: the frozen 100-pair sample, judged by
//! the SHIPPED T3 judge against the materialized pinned trees, scored
//! against the frozen independent audit (design vol.3 §9.2). The
//! candidate universe froze in 3c before the judge existed and the
//! audit froze before any scoring ran (dedup_provenance.rs holds the
//! git legs), so neither the denominator nor the truth could bend to
//! the judge. A degraded core reply refuses at the wire layer
//! (wire::parse_result — T-G12); all row/summary containers are
//! BTreeMaps, so file order cannot leak into bytes (T-G11).
//!
//! Generate (all five corpora, one invocation; corpora repos under
//! .ce-eval/corpora/; needs CE_CORE_BIN; re-blessing a grown wrong
//! ledger needs CE_ACCEPT_T3=1):
//!   cargo test --test eval_t3_precision -- --ignored --nocapture

mod eval_support;
mod eval_t3_precision_parts;

use eval_support::{
    FROZEN_CORPORA, assert_frozen_corpus_set, core_bin, core_link, doc_suffix, eval_doc,
    generated_from, load, materialize_tree, str_pairs, t3c_constants, t3f, walk_tree_in, write_doc,
};
use eval_t3_precision_parts as parts;
use serde_json::{Value, json};
use std::collections::BTreeMap;

const METHOD: &str = "the frozen t3-sample mains of this corpus, joined back to the \
    digest-anchored live candidate pass, judged by the shipped judge (product tree \
    build, product wire codec, product 85/100 verdict binding) over the pinned tree, \
    scored against the frozen independent audit. correct = judged-clone row whose \
    audited truth has copy lineage {clone, variant, t1t2} (user-delegated mapping, \
    2026-08-13; t1t2 credited per decision 3's whole-product stance); boilerplate / \
    unrelated / generated are the no-lineage FP families and score wrong. Precision \
    is over answered (judged-clone) rows only; the candidate pass's recall is a \
    CEILING — pairs the four sources cannot see are in no denominator. The universe \
    ledger carries the corpus-wide product run (reported pairs vs the pre-registered \
    min_reported_pairs floor, T-G14, plus the over-cap/forest drop ledgers). The θ \
    table publishes every grid threshold; the release knob is the loosest θ \
    clearing 0.85 — published, never tuned silently.";

fn precision_stem(name: Option<&str>) -> String {
    match name {
        Some(n) => format!("t3-precision-{n}"),
        None => "t3-precision".into(),
    }
}

/// One judged sample row joined to its audited truth.
fn row_json(s: &Value, j: t3f::Judgment, truth: &str) -> Value {
    let judged_clone = j.is_clone();
    let mut row = json!({
        "rank": s["rank"],
        "truth": truth,
        "judgment": j.label(),
        "judged_clone": judged_clone,
        "verdict": parts::verdict_of(j.label(), judged_clone, truth),
    });
    if let t3f::Judgment::Scored { ted, n1, n2 } = j {
        row["ted"] = json!(ted);
        row["n1"] = json!(n1);
        row["n2"] = json!(n2);
    }
    for key in t3f::T3_IDENTITY {
        row[key] = s[key].clone();
    }
    row
}

/// The corpus-wide product run: the T-G14 output floor's evidence
/// and the drop ledgers, from the SAME materialized tree.
fn universe_ledger(corpus: &str, walked: &[eval_support::WalkedFile]) -> Value {
    let root = materialize_tree("prec", corpus, walked);
    let report = codeeraser::dedup::t3::run(&root, None, &core_bin()).expect("product run");
    std::fs::remove_dir_all(&root).expect("drop temp tree");
    let c = &report.counts;
    json!({
        "reported_pairs": c.clones,
        "survivors": c.survivors,
        "sent": c.sent,
        "judged": c.judged,
        "over_cap_units": c.over_cap_units,
        "forest_units": c.forest_units,
        "pairs_dropped_over_cap": c.pairs_dropped_over_cap,
        "pairs_dropped_forest": c.pairs_dropped_forest,
    })
}

/// Generate one corpus's precision doc.
fn generate_corpus(name: &Option<String>, sample: &Value) {
    let a = eval_support::anchored_candidates(name);
    let (walked, _) = walk_tree_in(a.repo.as_deref(), &a.tip);
    let texts = t3f::texts_by_path(&walked);
    let sampled = t3f::main_rows(sample, &a.corpus, &a.tip);
    let mut link = core_link();
    let judgments = t3f::judge_sample(&a.candidates, &texts, &sampled, &mut link);
    let review = t3f::t3_review_doc(&a.corpus);
    let truths = str_pairs(&review, "rows", "rank", "truth");
    let rows: Vec<Value> = sampled
        .iter()
        .zip(&judgments)
        .map(|(s, &j)| {
            let rank = s["rank"].as_str().expect("rank");
            let truth = truths
                .get(rank)
                .copied()
                .unwrap_or_else(|| panic!("{rank}: unaudited"));
            row_json(s, j, truth)
        })
        .collect();
    let path = eval_doc(&precision_stem(name.as_deref()));
    parts::assert_wrong_frozen(&path, &rows);
    let universe = universe_ledger(&a.corpus, &walked);
    let floor = t3c_constants(&a.corpus)["min_reported_pairs"]
        .as_u64()
        .expect("floor");
    assert!(
        universe["reported_pairs"].as_u64().expect("n") >= floor,
        "{}: reported pairs below the pre-registered output floor (T-G14)",
        a.corpus
    );
    let doc = json!({
        "schema": "ce.eval-t3-precision/1.0.0",
        "corpus": {"name": name, "tip": a.tip},
        "generated_from": generated_from(),
        "method": METHOD,
        "constants": t3c_constants(&a.corpus),
        "summary": parts::rescore(&rows),
        "universe": universe,
        "rows": rows,
    });
    write_doc(&path, &doc, &format!("{path} written"));
}

#[test]
#[ignore] // needs the five corpus repositories + CE_CORE_BIN
fn generate_t3_precision() {
    let sample = load(&eval_doc("t3-sample"));
    for name in FROZEN_CORPORA {
        generate_corpus(&name.map(str::to_string), &sample);
    }
}

/// One row's stored facts re-derived (T-G4/G6 + the θ pin): identity
/// echo, truth echo against the frozen audit, judged_clone recomputed
/// from raw ted at the product threshold binding, verdict recomputed
/// from its own row.
fn check_row(corpus: &str, rank: &str, row: &Value, s: &Value, truths: &BTreeMap<&str, &str>) {
    for key in t3f::T3_IDENTITY {
        assert_eq!(row[key], s[key], "{corpus}/{rank}: {key} echo drifted");
    }
    let truth = truths
        .get(rank)
        .unwrap_or_else(|| panic!("{corpus}/{rank}: not in the audit"));
    assert_eq!(
        row["truth"].as_str(),
        Some(*truth),
        "{corpus}/{rank}: truth echo"
    );
    let judged_clone = row["judged_clone"].as_bool().expect("judged_clone");
    if let (Some(ted), Some(n1), Some(n2)) =
        (row["ted"].as_i64(), row["n1"].as_i64(), row["n2"].as_i64())
    {
        assert_eq!(
            judged_clone,
            codeeraser::dedup::t3::is_clone(ted, n1, n2),
            "{corpus}/{rank}: judged_clone contradicts its raw ted (θ pin)"
        );
    } else {
        assert!(
            !judged_clone,
            "{corpus}/{rank}: clone verdict without a score"
        );
    }
    let recomputed = parts::verdict_of(
        row["judgment"].as_str().expect("judgment"),
        judged_clone,
        truth,
    );
    assert_eq!(
        row["verdict"].as_str(),
        Some(recomputed),
        "{corpus}/{rank}: stored verdict contradicts its own row (T-G6)"
    );
}

/// One corpus's precision doc checked end to end. The doc's corpus
/// header is the {name, tip} envelope (not the audit tables' bare
/// string), so the frame's pieces run directly: sample filter,
/// bijection walk, per-row re-derivation with the tip pinned.
fn check_precision_doc(corpus: &str, doc: &Value, sample: &Value) {
    let rows = doc["rows"].as_array().expect("rows");
    assert_eq!(
        doc["summary"],
        parts::rescore(rows),
        "{corpus}: summary drifted from rows (T-G1)"
    );
    let name = doc["corpus"]["name"].as_str();
    assert_eq!(name, (corpus != "self").then_some(corpus), "{corpus}: name");
    let review = t3f::t3_review_doc(corpus);
    let truths = str_pairs(&review, "rows", "rank", "truth");
    let sampled = t3f::corpus_mains(sample, corpus);
    eval_support::each_audited_row(corpus, rows, &sampled, |rank, row, s| {
        assert_eq!(doc["corpus"]["tip"], s["tip"], "{corpus}: tip pin");
        check_row(corpus, rank, row, s, &truths);
    });
    let verdicts = doc["summary"]["verdicts"].as_object().expect("verdicts");
    let total: u64 = verdicts.values().map(|v| v.as_u64().expect("n")).sum();
    assert_eq!(total, rows.len() as u64, "{corpus}: conservation (T-G3)");
    for key in verdicts.keys() {
        assert!(
            parts::VERDICTS.contains(&key.as_str()),
            "{corpus}: unknown verdict class {key}"
        );
    }
}

/// Aggregate one doc and apply the per-corpus gate where the
/// answered denominator reaches 5 (the EVAL-SET small-denominator
/// stance).
fn corpus_gate(path: &str, sample: &Value, agg: &mut (u64, u64, u64, BTreeMap<String, u64>)) {
    let doc = load(path);
    let key = doc_suffix(path, "t3-precision").unwrap_or_else(|| "self".into());
    check_precision_doc(&key, &doc, sample);
    let v = &doc["summary"]["verdicts"];
    let n = |k: &str| v[k].as_u64().unwrap_or(0);
    let (c, w) = (n("correct"), n("wrong"));
    agg.0 += c;
    agg.1 += w;
    agg.2 += doc["rows"].as_array().expect("rows").len() as u64;
    for r in doc["rows"].as_array().expect("rows") {
        *agg.3
            .entry(r["lang"].as_str().expect("lang").into())
            .or_insert(0) += 1;
    }
    if c + w >= 5 {
        let p = c as f64 / (c + w) as f64;
        assert!(
            p >= 0.85,
            "{key}: per-corpus precision {p:.3} < 0.85 on denominator {} (T-G2)",
            c + w
        );
    }
    let floor = doc["constants"]["min_reported_pairs"]
        .as_u64()
        .expect("floor");
    assert!(
        doc["universe"]["reported_pairs"].as_u64().expect("n") >= floor,
        "{key}: frozen doc below its output floor (T-G14)"
    );
}

/// The 3f contract: every frozen corpus doc present (T-G10), 100
/// judged rows (T-G3), per-language floors (T-G5), overall and
/// per-corpus precision >= 0.85 (T-G2), output floors (T-G14).
#[test]
fn precision_meets_the_contract() {
    let docs = assert_frozen_corpus_set("t3-precision");
    let sample = load(&eval_doc("t3-sample"));
    let mut agg = (0u64, 0u64, 0u64, BTreeMap::new());
    for path in &docs {
        corpus_gate(path, &sample, &mut agg);
    }
    let (c_all, w_all, total, by_lang) = agg;
    assert_eq!(total, 100, "judged rows across corpora (T-G3)");
    for (lang, n) in &by_lang {
        assert!(*n >= 15, "{lang}: {n} rows < per-lang floor 15 (T-G5)");
    }
    let overall = c_all as f64 / (c_all + w_all) as f64;
    assert!(
        overall >= 0.85,
        "overall precision {overall:.3} < 0.85 over the frozen sample (T-G2 contract)"
    );
}

/// T-G9: every advertised refusal exercised — the shared battery
/// plus the family's θ-pin and cooked-summary cases.
#[test]
fn precision_refuses_tampering() {
    let sample = load(&eval_doc("t3-precision-zod"));
    let sample_doc = load(&eval_doc("t3-sample"));
    let check = |doc: &Value| check_precision_doc("zod", doc, &sample_doc);
    // row-0-robust mutations only: a value already equal to the
    // stored one would make the "must refuse" claim vacuous
    eval_support::assert_tampering_refused(
        &sample,
        &[
            ("rank", "not-a-sampled-rank", "phantom rank"),
            ("truth", "not-a-truth", "cooked truth echo"),
            ("a_key", "phantom-key", "identity echo drift"),
        ],
        &check,
    );
    let mut cooked = sample.clone();
    cooked["summary"]["verdicts"]["wrong"] = json!(0);
    assert!(
        eval_support::doc_refused(&cooked, &check),
        "cooked summary must refuse"
    );
    let scored = sample["rows"]
        .as_array()
        .expect("rows")
        .iter()
        .position(|r| r["ted"].is_i64())
        .expect("a scored row");
    let mut stubbed = sample.clone();
    let flip = !stubbed["rows"][scored]["judged_clone"]
        .as_bool()
        .expect("b");
    stubbed["rows"][scored]["judged_clone"] = json!(flip);
    assert!(
        eval_support::doc_refused(&stubbed, &check),
        "stubbed threshold (flipped judged_clone) must refuse"
    );
}
