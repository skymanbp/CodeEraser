//! M5-3f: the T3 audit — assembly leg. Per corpus, the frozen sample
//! rows are joined back to the live candidate pass (digest-anchored),
//! judged by the SHIPPED judge (the answered count decides the
//! min_answered top-up before any auditor starts), and the two units'
//! VERBATIM texts are assembled for the independent auditors — with
//! NO judgment fields in the assembly (RM18: an auditor who sees the
//! verdict drifts toward it; the assembler writes texts, verbatim,
//! and nothing else). Assemblies land under .ce-eval/analysis/
//! (machine-local, never committed); the frozen audit tables land in
//! cli/tests/eval_t3_review/ and their gates join this file once the
//! tables exist.
//!
//! Run (all five corpora, one invocation; corpora repos under
//! .ce-eval/corpora/ as in eval_t3_sample.rs; needs CE_CORE_BIN):
//!   cargo test --test eval_t3_audit -- --ignored --nocapture

mod eval_support;

use eval_support::{
    FROZEN_CORPORA, WalkedFile, core_link, eval_doc, load, out_dir, t3f, walk_tree_in,
};
use serde_json::{Value, json};

/// One corpus's assembly: judge the sampled mains, then write the
/// verbatim audit package. Returns (clone-judged, judged, total).
fn assemble_corpus(name: &Option<String>, sample: &Value) -> (u64, u64, u64) {
    let a = eval_support::anchored_candidates(name);
    let (corpus, tip, c) = (a.corpus, a.tip, a.candidates);
    let (walked, _) = walk_tree_in(a.repo.as_deref(), &tip);
    let texts = t3f::texts_by_path(&walked);
    let rows = t3f::main_rows(sample, &corpus, &tip);
    let mut link = core_link();
    let judgments = t3f::judge_sample(&c, &texts, &rows, &mut link);
    let assembled: Vec<Value> = rows.iter().map(|r| assembly_row(r, &walked)).collect();
    let path = format!(
        "{}/analysis/t3-audit-assembly-{corpus}.json",
        out_dir().display()
    );
    let doc = json!({
        "corpus": corpus,
        "tip": tip,
        "note": "verbatim assembly for the independent T3 audit; no judgment fields \
                 by design (RM18) — the auditor sees code, not verdicts",
        "rows": assembled,
    });
    std::fs::write(
        &path,
        serde_json::to_string_pretty(&doc).expect("ser") + "\n",
    )
    .expect(&path);
    let clones = judgments.iter().filter(|j| j.is_clone()).count() as u64;
    let scored = judgments.iter().filter(|j| j.label() == "scored").count() as u64;
    println!(
        "{corpus}: {} rows assembled -> {path}; judged {scored}, clone {clones}, drops {:?}",
        rows.len(),
        judgments
            .iter()
            .filter(|j| j.label() != "scored")
            .map(|j| j.label())
            .collect::<Vec<_>>()
    );
    (clones, scored, rows.len() as u64)
}

/// One assembly row: frozen identity + spans + verbatim texts, both
/// sides sliced through the SAME segmentation throat the cache used.
fn assembly_row(r: &Value, walked: &[WalkedFile]) -> Value {
    let texts = t3f::texts_by_path(walked);
    let side = |p: &str, k: &str, n: &str| {
        let path = r[p].as_str().expect(p);
        let (code, text) = texts[path];
        let spans = t3f::file_spans(text, eval_support::lang_of(code));
        let key = (r[k].as_str().expect(k).to_string(), r[n].as_i64().expect(n));
        let (start, end) = *spans
            .get(&key)
            .unwrap_or_else(|| panic!("{path}: sampled unit {key:?} has no span"));
        (start, end, t3f::slice_lines(text, start, end))
    };
    let (a_start, a_end, a_text) = side("a_path", "a_key", "a_nth");
    let (b_start, b_end, b_text) = side("b_path", "b_key", "b_nth");
    let mut row = json!({
        "rank": r["rank"],
        "a_start": a_start, "a_end": a_end, "a_text": a_text,
        "b_start": b_start, "b_end": b_end, "b_text": b_text,
    });
    for key in t3f::T3_IDENTITY {
        row[key] = r[key].clone();
    }
    row
}

#[test]
#[ignore] // needs the five corpus repositories + CE_CORE_BIN
fn generate_t3_audit_assembly() {
    let sample = load(&eval_doc("t3-sample"));
    let (mut clones, mut scored, mut total) = (0, 0, 0);
    for name in FROZEN_CORPORA {
        let name = name.map(str::to_string);
        let (c, s, t) = assemble_corpus(&name, &sample);
        clones += c;
        scored += s;
        total += t;
    }
    println!(
        "mains: {total} rows, {scored} judged, {clones} judged clone \
         (min_answered {} — backups needed only below it)",
        t3f::MIN_ANSWERED
    );
}

const CORPORA: [&str; 5] = ["cobra", "requests", "ripgrep", "self", "zod"];

/// One corpus's frozen audit table against the frozen sample:
/// embedded name and tip, a rank bijection (phantom and missing rows
/// equally loud), verbatim identity echo, and a verdict per row.
fn check_corpus(corpus: &str, doc: &Value, sample: &Value) {
    let mains = sample["main"].as_array().expect("main");
    eval_support::check_audit_frame(corpus, doc, mains, |rank, row, s| {
        for key in t3f::T3_IDENTITY {
            assert_eq!(row[key], s[key], "{corpus}/{rank}: {key} echo drifted");
        }
        assert_eq!(doc["tip"], s["tip"], "{corpus}: tip vs sampled tip");
        check_verdict(corpus, rank, row);
    });
    eval_support::assert_gaps_accounted(corpus, doc, "unit_gaps");
}

/// truth in the closed vocabulary; the edit axis rides clone rows
/// ONLY (both directions asserted); why holds the mechanism floor.
fn check_verdict(corpus: &str, rank: &str, row: &Value) {
    let truth = row["truth"].as_str().expect("truth");
    assert!(
        t3f::T3_TRUTHS.contains(&truth),
        "{corpus}/{rank}: unknown truth {truth:?}"
    );
    match (truth == "clone", row["edit"].as_str()) {
        (true, Some(e)) => assert!(
            t3f::T3_EDITS.contains(&e),
            "{corpus}/{rank}: unknown edit {e:?}"
        ),
        (true, None) => panic!("{corpus}/{rank}: clone row without its edit axis"),
        (false, Some(_)) => panic!("{corpus}/{rank}: edit on a non-clone row"),
        (false, None) => {}
    }
    let why = row["why"].as_str().expect("why").trim();
    assert!(
        why.len() >= 15,
        "{corpus}/{rank}: why is not a mechanism: {why:?}"
    );
}

/// The 3f audit exit: every sampled main row audited, per corpus,
/// identity echo intact — 100/100 with mechanism-naming whys — and
/// the promised vocabulary seats are real: boilerplate and t1t2 each
/// hold at least one (folding them away is the hiding move the
/// closed vocabulary exists to prevent).
#[test]
fn audit_covers_sample_bijectively() {
    let sample = load(&eval_doc("t3-sample"));
    let mut total = 0;
    let mut by_truth: std::collections::BTreeMap<String, u64> = Default::default();
    for corpus in CORPORA {
        let doc = t3f::t3_review_doc(corpus);
        check_corpus(corpus, &doc, &sample);
        for row in doc["rows"].as_array().expect("rows") {
            *by_truth
                .entry(row["truth"].as_str().expect("truth").into())
                .or_insert(0) += 1;
        }
        total += doc["rows"].as_array().expect("rows").len();
    }
    assert_eq!(total, sample["main"].as_array().expect("main").len());
    eval_support::assert_nonzero_seats(
        &by_truth,
        &["boilerplate", "t1t2"],
        "promised vocabulary seat is empty",
    );
}

/// Counterfactual (G9 discipline): every advertised refusal is
/// exercised, not assumed — the shared table-driven battery plus the
/// edit-axis cases only this family owns.
#[test]
fn audit_refuses_tampering() {
    let sample = load(&eval_doc("t3-sample"));
    let check = |doc: &Value| check_corpus("cobra", doc, &sample);
    let pristine = t3f::t3_review_doc("cobra");
    eval_support::assert_tampering_refused(
        &pristine,
        &[
            ("rank", "not-a-sampled-rank", "phantom rank"),
            ("why", "yes", "stub why"),
            ("truth", "mismatch", "non-vocabulary truth"),
            ("a_key", "phantom-key", "identity echo drift"),
        ],
        &check,
    );
    let clone_row = pristine["rows"]
        .as_array()
        .expect("rows")
        .iter()
        .position(|r| r["truth"] == "clone")
        .expect("cobra audit holds a clone row");
    let mut editless = pristine.clone();
    editless["rows"][clone_row]
        .as_object_mut()
        .expect("row")
        .remove("edit");
    assert!(
        eval_support::doc_refused(&editless, &check),
        "clone row without edit must refuse"
    );
    let mut edited = pristine.clone();
    edited["rows"][0]["edit"] = Value::from("rename");
    let non_clone = pristine["rows"][0]["truth"] != "clone";
    assert!(
        !non_clone || eval_support::doc_refused(&edited, &check),
        "edit on a non-clone row must refuse"
    );
}
