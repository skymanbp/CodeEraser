//! M5-3g: the docdup audit — assembly leg. Per corpus, the frozen
//! census rows are joined to the pinned tree and both segments'
//! VERBATIM texts are assembled for the independent auditors — with
//! NO judgment fields in the assembly (RM18) and no report/margin
//! band either: the auditor sees two pieces of documentation text and
//! their identities, nothing that hints at what any filter or floor
//! thinks of them. Assemblies land under .ce-eval/analysis/
//! (machine-local, never committed); the frozen audit tables land in
//! cli/tests/eval_docdup_review/ and their gates join this file once
//! the tables exist.
//!
//! Run (corpora repos under .ce-eval/corpora/ as in eval_t3_sample):
//!   cargo test --test eval_docdup_audit -- --ignored --nocapture

mod eval_support;

use eval_support::{eval_doc, load, of_corpus, out_dir, t3f, walk_tree_in};
use serde_json::{Value, json};

const CORPORA: [&str; 5] = ["cobra", "requests", "ripgrep", "self", "zod"];

/// One corpus's assembly: identity echo + verbatim segment texts.
fn assemble_corpus(corpus: &str, sample: &Value) -> usize {
    let rows = of_corpus(sample["main"].as_array().expect("main"), corpus);
    assert!(!rows.is_empty(), "{corpus}: empty census slice");
    let tip = rows[0]["tip"].as_str().expect("tip").to_string();
    let repo = (corpus != "self").then(|| format!("{}/corpora/{corpus}", out_dir().display()));
    let (walked, _) = walk_tree_in(repo.as_deref(), &tip);
    let texts = t3f::texts_by_path(&walked);
    let assembled: Vec<Value> = rows.iter().map(|r| assembly_row(r, &texts)).collect();
    let path = format!(
        "{}/analysis/docdup-audit-assembly-{corpus}.json",
        out_dir().display()
    );
    let doc = json!({
        "corpus": corpus,
        "tip": tip,
        "note": "verbatim assembly for the independent docdup audit; no judgment \
                 fields by design (RM18) — the auditor sees documentation text, \
                 not verdicts, floors or bands",
        "rows": assembled,
    });
    std::fs::write(
        &path,
        serde_json::to_string_pretty(&doc).expect("ser") + "\n",
    )
    .expect(&path);
    println!("{corpus}: {} rows assembled -> {path}", rows.len());
    rows.len()
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
