//! M5-3g: the docdup audit — assembly leg. Per corpus, the frozen
//! census rows are joined to the pinned tree and both segments'
//! VERBATIM texts are assembled for the independent auditors — with
//! NO judgment fields in the assembly (RM18) and no report/margin
//! band either: the auditor sees two pieces of documentation text and
//! their identities, nothing that hints at what any filter or floor
//! thinks of them. Assemblies land under .ce-eval/analysis/
//! (machine-local, never committed); the frozen audit tables live in
//! cli/tests/eval_docdup_review/ and their gates run below on the
//! shared AuditFamily machinery.
//!
//! Run (corpora repos under .ce-eval/corpora/ as in eval_t3_sample):
//!   cargo test --test eval_docdup_audit -- --ignored --nocapture

mod eval_support;

use eval_support::{AuditFamily, eval_doc, load, of_corpus, out_dir, t3f, walk_tree_in};
use serde_json::{Value, json};

const CORPORA: [&str; 5] = ["cobra", "requests", "ripgrep", "self", "zod"];

const FAMILY: AuditFamily = AuditFamily {
    identity: &eval_support::DOCDUP_IDENTITY,
    truths: &eval_support::DOCDUP_TRUTHS,
    gaps_key: "doc_gaps",
};

/// Every retired identity of the 2026-08-14 amendment, frozen: rows
/// audited under the v1 census whose pairs the three masks removed
/// from the universe — the exemption's evidence, never silently
/// dropped and never allowed back into the active bijection.
const RETIRED_TOTAL: usize = 15;

/// One corpus's assembly: identity echo + verbatim segment texts.
fn assemble_corpus(corpus: &str, sample: &Value) -> usize {
    let rows = of_corpus(sample["main"].as_array().expect("main"), corpus);
    assert!(!rows.is_empty(), "{corpus}: empty census slice");
    let tip = rows[0]["tip"].as_str().expect("tip").to_string();
    let repo = (corpus != "self").then(|| format!("{}/corpora/{corpus}", out_dir().display()));
    let (walked, _) = walk_tree_in(repo.as_deref(), &tip);
    let texts = t3f::texts_by_path(&walked);
    let assembled: Vec<Value> = rows.iter().map(|r| assembly_row(r, &texts)).collect();
    let n = assembled.len();
    eval_support::write_assembly(
        "docdup",
        corpus,
        &tip,
        "verbatim assembly for the independent docdup audit; no judgment fields \
         by design (RM18) — the auditor sees documentation text, not verdicts, \
         floors or bands",
        assembled,
    );
    n
}

/// Frozen identity + both sides' verbatim line slices (the segment
/// geometry IS the identity, so the slice needs no re-derivation —
/// the spans were frozen by the 3d extraction).
fn assembly_row(r: &Value, texts: &std::collections::BTreeMap<&str, (&str, &str)>) -> Value {
    let side = |p: &str, s: &str, e: &str| {
        let path = r[p].as_str().expect(p);
        let (_, text) = texts
            .get(path)
            .unwrap_or_else(|| panic!("{path}: not in the pinned tree"));
        t3f::slice_lines(
            text,
            r[s].as_i64().expect(s) as usize,
            r[e].as_i64().expect(e) as usize,
        )
    };
    let mut row = json!({
        "rank": r["rank"],
        "a_text": side("a_path", "a_start", "a_end"),
        "b_text": side("b_path", "b_start", "b_end"),
    });
    for key in eval_support::DOCDUP_IDENTITY {
        row[key] = r[key].clone();
    }
    row
}

#[test]
#[ignore] // needs the four external corpus repositories
fn generate_docdup_audit_assembly() {
    let sample = load(&eval_doc("docdup-sample"));
    let total: usize = CORPORA
        .iter()
        .map(|corpus| assemble_corpus(corpus, &sample))
        .sum();
    assert_eq!(
        total,
        sample["main"].as_array().expect("main").len(),
        "assemblies must cover the census exactly"
    );
}

/// One corpus's frozen table: the shared frame over ACTIVE rows (this
/// family has no bespoke row axis), then the retired evidence checked
/// against the same vocabulary and refused back into the census.
fn check_corpus(corpus: &str, doc: &Value, sample: &Value) {
    let mains = sample["main"].as_array().expect("main");
    FAMILY.check_frame(corpus, doc, mains, |_, _, _| {});
    let census: std::collections::BTreeSet<&str> = mains
        .iter()
        .map(|r| r["rank"].as_str().expect("rank"))
        .collect();
    for row in doc["retired"].as_array().expect("retired") {
        let rank = row["rank"].as_str().expect("rank");
        assert!(
            !census.contains(rank),
            "{corpus}/{rank}: retired row still in the census"
        );
        FAMILY.check_verdict(corpus, rank, row);
        assert!(
            row["note"].as_str().is_some_and(|n| n.len() >= 10),
            "{corpus}/{rank}: retired row without its amendment note"
        );
    }
}

/// The audit exit: every census row audited per corpus with identity
/// echo intact, the retired evidence complete, and the vocabulary
/// seats real — redundant plus every audited FP family (skeleton,
/// tabular, quoted) hold nonzero seats across active+retired.
/// paraphrase's seat is EMPTY by measurement (docseg.rs note): the
/// miss class lives below the oracle floor, recorded not manufactured.
#[test]
fn audit_covers_census_bijectively() {
    let sample = load(&eval_doc("docdup-sample"));
    eval_support::assert_coverage_and_seats(
        &sample,
        &CORPORA,
        |corpus| {
            let doc = eval_support::docdup_review_doc(corpus);
            check_corpus(corpus, &doc, &sample);
            doc
        },
        &["rows", "retired"],
        &["redundant", "skeleton", "tabular", "quoted"],
    );
    let retired: usize = CORPORA
        .iter()
        .map(|c| {
            eval_support::docdup_review_doc(c)["retired"]
                .as_array()
                .expect("retired")
                .len()
        })
        .sum();
    assert_eq!(retired, RETIRED_TOTAL, "retired evidence shrank");
}

/// This family's tamper mutations for the shared battery.
const MUTATIONS: [(&str, &str, &str); 4] = [
    ("rank", "not-a-census-rank", "phantom rank"),
    ("why", "yes", "stub why"),
    ("truth", "mismatch", "non-vocabulary truth"),
    ("a_path", "phantom/path.md", "identity echo drift"),
];

/// Counterfactual (G9 discipline): the shared table-driven battery
/// plus the retired-row cases only this family owns.
#[test]
fn audit_refuses_tampering() {
    let sample = load(&eval_doc("docdup-sample"));
    let check = |doc: &Value| check_corpus("zod", doc, &sample);
    let pristine = eval_support::tamper_prologue("docdup", "zod", &MUTATIONS, &check);
    // a retired row whose rank is still in the census must refuse —
    // "retired" claims the pair LEFT the universe
    let mut smuggled = pristine.clone();
    let active_rank = smuggled["rows"][0]["rank"].clone();
    smuggled["retired"][0]["rank"] = active_rank;
    assert!(
        eval_support::doc_refused(&smuggled, &check),
        "retired row colliding with the census must refuse"
    );
    let mut noteless = pristine.clone();
    noteless["retired"][0]
        .as_object_mut()
        .expect("row")
        .remove("note");
    assert!(
        eval_support::doc_refused(&noteless, &check),
        "retired row without its amendment note must refuse"
    );
}
