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
    FROZEN_CORPORA, PrecisionAgg, core_bin, core_link, doc_stem, eval_doc, generated_from, load,
    materialize_tree, str_pairs, t3c_constants, t3f, walk_tree_in, write_doc,
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
/// and the drop ledgers, from the SAME materialized tree — the
/// product's own Counts serialization, no re-keyed copy.
fn universe_ledger(corpus: &str, walked: &[eval_support::WalkedFile]) -> Value {
    let root = materialize_tree("prec", corpus, walked);
    let report = codeeraser::dedup::t3::run(&root, None, &core_bin()).expect("product run");
    std::fs::remove_dir_all(&root).expect("drop temp tree");
    serde_json::to_value(&report.counts).expect("counts")
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
    let path = eval_doc(&doc_stem("t3-precision", name));
    eval_support::assert_ledger_frozen(&path, &rows, &parts::wrong_ranks, "CE_ACCEPT_T3");
    let universe = universe_ledger(&a.corpus, &walked);
    let floor = t3c_constants(&a.corpus)["min_reported_pairs"]
        .as_u64()
        .expect("floor");
    assert!(
        universe["clones"].as_u64().expect("n") >= floor,
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
    eval_support::assert_row_verdict(corpus, rank, row, truths, |truth| {
        parts::verdict_of(
            row["judgment"].as_str().expect("judgment"),
            judged_clone,
            truth,
        )
    });
}

/// One corpus's precision doc checked end to end. The doc's corpus
/// header is the {name, tip} envelope (not the audit tables' bare
/// string), so the frame's pieces run directly: sample filter,
/// bijection walk, per-row re-derivation with the tip pinned.
fn check_precision_doc(corpus: &str, doc: &Value, sample: &Value) {
    let name = doc["corpus"]["name"].as_str();
    assert_eq!(name, (corpus != "self").then_some(corpus), "{corpus}: name");
    eval_support::assert_summary_rederived(corpus, doc, parts::rescore);
    let rows = doc["rows"].as_array().expect("rows");
    let review = t3f::t3_review_doc(corpus);
    let truths = str_pairs(&review, "rows", "rank", "truth");
    let sampled = t3f::corpus_mains(sample, corpus);
    eval_support::each_audited_row(corpus, rows, &sampled, |rank, row, s| {
        assert_eq!(doc["corpus"]["tip"], s["tip"], "{corpus}: tip pin");
        check_row(corpus, rank, row, s, &truths);
    });
    eval_support::assert_verdict_conservation(corpus, doc, &parts::VERDICTS);
}

/// Aggregate one doc and apply the per-corpus gate on the answered
/// denominator (the family's T-G2 semantics: answered = judged
/// clone), plus the frozen output floor (T-G14).
fn corpus_gate(path: &str, sample: &Value, agg: &mut PrecisionAgg) {
    let (key, doc, c, w) =
        eval_support::open_scored_doc(path, "t3-precision", sample, agg, check_precision_doc);
    let rows = doc["rows"].as_array().expect("rows");
    agg.rows += rows.len() as u64;
    eval_support::tally_field(rows, "lang", &mut agg.by_lang);
    eval_support::assert_corpus_precision(&key, (c, w), c + w, 0.85, "T-G2");
    let floor = doc["constants"]["min_reported_pairs"]
        .as_u64()
        .expect("floor");
    assert!(
        doc["universe"]["clones"].as_u64().expect("n") >= floor,
        "{key}: frozen doc below its output floor (T-G14)"
    );
}

/// The 3f contract and its refusals in one battery: every frozen
/// corpus doc present (T-G10), 100 judged rows (T-G3), per-language
/// floors (T-G5), precision >= 0.85 overall and per corpus (T-G2),
/// output floors (T-G14), then T-G9 — the shared mutation table
/// (row-0-robust: a value equal to the stored one would make "must
/// refuse" vacuous) plus the θ-pin and cooked-summary cases below.
const SPEC: eval_support::FamilySpec = eval_support::FamilySpec {
    family: "t3-precision",
    sample: "t3-sample",
    gate: 0.85,
    tag: "T-G2 contract",
    total: 100,
    slice_floor: 15,
    mutations: &[
        ("rank", "not-a-sampled-rank", "phantom rank"),
        ("truth", "not-a-truth", "cooked truth echo"),
        ("a_key", "phantom-key", "identity echo drift"),
    ],
};

#[test]
fn precision_contract_and_refusals() {
    eval_support::assert_precision_family(
        &SPEC,
        corpus_gate,
        &check_precision_doc,
        |pristine, refused| {
            let scored = pristine["rows"]
                .as_array()
                .expect("rows")
                .iter()
                .position(|r| r["ted"].is_i64())
                .expect("a scored row");
            let mut stubbed = pristine.clone();
            let flip = !stubbed["rows"][scored]["judged_clone"]
                .as_bool()
                .expect("b");
            stubbed["rows"][scored]["judged_clone"] = json!(flip);
            assert!(
                refused(&stubbed),
                "stubbed threshold (flipped judged_clone) must refuse"
            );
            let mut cooked = pristine.clone();
            cooked["summary"]["verdicts"]["wrong"] = json!(0);
            assert!(refused(&cooked), "cooked summary must refuse");
        },
    );
}
