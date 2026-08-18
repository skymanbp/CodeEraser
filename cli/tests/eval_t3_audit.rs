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

use eval_support::{eval_doc, load, t3f};
use serde_json::Value;

const CORPORA: [&str; 5] = ["cobra", "requests", "ripgrep", "self", "zod"];

const FAMILY: eval_support::AuditFamily = eval_support::AuditFamily {
    identity: &t3f::T3_IDENTITY,
    truths: &t3f::T3_TRUTHS,
    gaps_key: "unit_gaps",
};

/// The shared frame plus this family's bespoke axis: the edit label
/// rides clone rows ONLY (both directions asserted).
fn check_corpus(corpus: &str, doc: &Value, sample: &Value) {
    let mains = sample["main"].as_array().expect("main");
    FAMILY.check_frame(corpus, doc, mains, |rank, row, _| {
        match (row["truth"] == "clone", row["edit"].as_str()) {
            (true, Some(e)) => assert!(
                t3f::T3_EDITS.contains(&e),
                "{corpus}/{rank}: unknown edit {e:?}"
            ),
            (true, None) => panic!("{corpus}/{rank}: clone row without its edit axis"),
            (false, Some(_)) => panic!("{corpus}/{rank}: edit on a non-clone row"),
            (false, None) => {}
        }
    });
}

/// The 3f audit exit: every sampled main row audited, per corpus,
/// identity echo intact — 100/100 with mechanism-naming whys — and
/// the promised vocabulary seats are real: boilerplate and t1t2 each
/// hold at least one (folding them away is the hiding move the
/// closed vocabulary exists to prevent).
#[test]
fn audit_covers_sample_bijectively() {
    let sample = load(&eval_doc("t3-sample"));
    eval_support::assert_coverage_and_seats(
        &sample,
        &CORPORA,
        |corpus| {
            let doc = t3f::t3_review_doc(corpus);
            check_corpus(corpus, &doc, &sample);
            doc
        },
        &["rows"],
        &["boilerplate", "t1t2"],
    );
}

/// This family's tamper mutations for the shared battery.
const MUTATIONS: [(&str, &str, &str); 4] = [
    ("rank", "not-a-sampled-rank", "phantom rank"),
    ("why", "yes", "stub why"),
    ("truth", "mismatch", "non-vocabulary truth"),
    ("a_key", "phantom-key", "identity echo drift"),
];

/// Counterfactual (G9 discipline): every advertised refusal is
/// exercised, not assumed — the shared table-driven battery plus the
/// edit-axis cases only this family owns.
#[test]
fn audit_refuses_tampering() {
    let sample = load(&eval_doc("t3-sample"));
    let check = |doc: &Value| check_corpus("cobra", doc, &sample);
    let pristine = eval_support::tamper_prologue("t3", "cobra", &MUTATIONS, &check);
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
