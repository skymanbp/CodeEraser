//! The frozen-universe gate skeleton — ONE binding for the slice
//! (eval_graph.rs) and t3 (eval_t3_universe.rs) families: the doc
//! stem, the gate opening, the nine-key envelope checks and the
//! working-tree drift walk. The generator half (tree walker, doc
//! assembly) retired with the one-shot instruments (git history;
//! resurrection takes cli/tests/eval_support at the same commit).

use serde_json::{Value, json};
use std::collections::BTreeMap;

/// The doc stem of one corpus in a family: "graph-slice-zod" /
/// "graph-slice" (self). Shared by generators and cross-family
/// lookups so file naming can never fork.
pub fn doc_stem(family: &str, name: &Option<String>) -> String {
    match name {
        Some(n) => format!("{family}-{n}"),
        None => family.into(),
    }
}

/// The shared opening of every universe gate: iterate the frozen docs
/// of one family (set anchored to FROZEN_CORPORA) through a per-doc
/// check.
pub fn each_frozen_doc(family: &str, mut check: impl FnMut(&str, &Value)) {
    for path in super::assert_frozen_corpus_set(family) {
        let doc = super::load(&path);
        check(&path, &doc);
    }
}

/// The frozen-doc envelope every per-corpus universe doc carries:
/// summary re-derived by the family's own scorer (G1), frozen
/// constants, frozen scope, well-formed exclusion ledger, pinned
/// full-OID tip, embedded corpus name matching the file name, rows
/// sorted and duplicate-free. Returns the corpus name.
pub fn assert_doc_envelope(
    path: &str,
    doc: &Value,
    family: &str,
    scorer: fn(&[Value]) -> Value,
    constants: fn() -> Value,
) -> Option<String> {
    let files = doc["files"].as_array().expect("files");
    assert_eq!(doc["summary"], scorer(files), "{path}: summary drifted");
    assert_eq!(doc["constants"], constants(), "{path}: constants drifted");
    assert_eq!(
        doc["scope"]["extensions"],
        json!(super::SCOPE_EXTS),
        "{path}"
    );
    assert_eq!(
        doc["scope"]["excludes"],
        json!(super::SCOPE_EXCLUDES),
        "{path}"
    );
    for (category, n) in doc["excluded"].as_object().expect("excluded") {
        assert!(
            ["excluded_prefix", "variant_extension", "other_extension"]
                .contains(&category.as_str())
                && n.as_u64().is_some_and(|v| v > 0),
            "{path}: malformed excluded row {category}"
        );
    }
    let tip = doc["corpus"]["tip"].as_str().expect("tip");
    assert!(
        tip.len() == 40 && tip.chars().all(|c| c.is_ascii_hexdigit()),
        "{path}: tip is not a pinned full OID"
    );
    let name = doc["corpus"]["name"].as_str().map(str::to_string);
    assert_eq!(
        name,
        super::doc_suffix(path, family),
        "{path}: embedded corpus name does not match the file name"
    );
    for pair in files.windows(2) {
        assert!(
            pair[0]["path"].as_str() < pair[1]["path"].as_str(),
            "{path}: rows unsorted or duplicated"
        );
    }
    name
}

/// Sum of a JSON object's u64 values — the map-total idiom every
/// per-key summary gate re-derives (site totals, survivor strata).
pub fn sum_obj(v: &Value) -> u64 {
    v.as_object()
        .expect("object")
        .values()
        .map(|x| x.as_u64().expect("u64"))
        .sum()
}

/// Accumulate a JSON object's u64 values into a cross-corpus map and
/// return the object's own total — the one accumulation walk every
/// universe gate and summarizer shares.
pub fn sum_obj_into(obj: &Value, into: &mut BTreeMap<String, u64>) -> u64 {
    let mut total = 0;
    for (k, v) in obj.as_object().expect("object") {
        let n = v.as_u64().expect("u64");
        *into.entry(k.clone()).or_insert(0) += n;
        total += n;
    }
    total
}

/// The working-tree drift walk both self gates share: every frozen
/// row whose working-tree file still carries the frozen sha256 is
/// handed to `check` as (row, path, lang code, text); returns how
/// many verified. Content-changed files are skipped — that is
/// editing, not drift; a semantics change re-pins the tip and
/// re-freezes.
pub fn each_frozen_match(doc: &Value, mut check: impl FnMut(&Value, &str, &str, &str)) -> usize {
    let mut verified = 0;
    for row in doc["files"].as_array().expect("files") {
        let path = row["path"].as_str().expect("path");
        let Ok(bytes) = std::fs::read(format!("../{path}")) else {
            continue;
        };
        // the frozen identity is git-blob text (LF); the Windows CI
        // runner checks out with autocrlf=true, so normalize before
        // comparing — first CI run of the slice gate was 0-row
        // vacuous on windows-latest for exactly this reason
        let text = String::from_utf8_lossy(&bytes).replace("\r\n", "\n");
        if super::content_sha(&text) != row["sha256"].as_str().expect("sha256") {
            continue;
        }
        check(row, path, row["lang"].as_str().expect("lang"), &text);
        verified += 1;
    }
    verified
}
