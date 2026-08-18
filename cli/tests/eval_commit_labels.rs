//! M4-5 commit-slice ground truth (review layer over the prelabels in
//! commit-slice-v1.json). Line-level moved GT at commit scope uses
//! **provenance semantics**, not content-set semantics: a freshly
//! written line whose content coincides with a removal elsewhere did
//! not *relocate* — it is exactly the duplication signal this product
//! exists to catch, and counting it "moved" would hide duplication
//! inside the health signal. Content-set matching cannot make that
//! distinction; unit-level matching (L2) approximates it.
//!
//! Three layers, all auditable (machinery + review tables in
//! eval_commit_review/):
//! 1. mechanical significance filter — moved marks on lines with no
//!    alphanumeric content carry no line identity and reclassify to
//!    novel/deleted (labels-v1 convention, `fourclass::significant`);
//! 2. mechanical cross/within partition — trimmed-content pairing
//!    across the commit's sides, preferring the within-file reading
//!    when both exist;
//! 3. reviewed corrections — content-coincidence pairs verified
//!    against the raw diffs, each with its mechanism.
//!
//! Commits with zero moved marks need no review: their labels equal
//! the slice prelabels verbatim.
//!
//! Run: cargo test --test eval_commit_labels -- --ignored --nocapture

mod eval_commit_review;
mod eval_support;

use eval_commit_review as review;
use eval_support::by_sha;
use serde_json::{Value, json};

fn pair_sum(row: &Value, key: &str) -> u64 {
    row["pairs"]
        .as_array()
        .expect("pairs")
        .iter()
        .map(|p| p[key].as_u64().unwrap())
        .sum()
}

/// Σ over rows of a nested `row[key][side]` counter.
fn sum_side(rows: &[Value], key: &str, side: &str) -> u64 {
    rows.iter().map(|r| r[key][side].as_u64().unwrap()).sum()
}

/// Re-derivable from the rows alone (the CI gate re-runs this).
fn summarize(rows: &[Value]) -> Value {
    let side = |key, s| sum_side(rows, key, s);
    let corrections: u64 = rows
        .iter()
        .flat_map(|r| r["corrections"].as_array().expect("corrections"))
        .map(|c| c["lines"].as_u64().unwrap())
        .sum();
    let unit_pairs = |key: &str| -> u64 {
        rows.iter()
            .flat_map(|r| r[key].as_array().unwrap())
            .map(|u| u["units"].as_array().unwrap().len() as u64)
            .sum()
    };
    let mut s = json!({
        "commits_reviewed": rows.len(),
        "moved_lines": {"cross_out": side("cross_file", "out"),
                        "cross_in": side("cross_file", "in"),
                        "within_out": side("within_file", "out"),
                        "within_in": side("within_file", "in")},
        "nonsignificant": {"out": side("nonsignificant", "out"),
                           "in": side("nonsignificant", "in")},
        "correction_lines": corrections,
        "relocated_units": unit_pairs("relocated_units"),
        // edge-unit pairs: one unit riding N edges counts N times.
        "relocation_edges": unit_pairs("relocation_edges"),
    });
    let below_floor: u64 = rows
        .iter()
        .map(|r| r["below_floor"].as_array().map_or(0, |a| a.len() as u64))
        .sum();
    if below_floor > 0 {
        s["below_floor_lines"] = json!(below_floor);
    }
    s
}

/// CI gate, no git needed, every corpus: labels rows cover exactly
/// the slice's moved-bearing commits; per-pair sums conserve the
/// slice totals; the moved partition and reclassification ledger
/// balance per row; the summary re-derives.
#[test]
fn commit_labels_consistent() {
    for (slice_path, labels_path) in eval_support::corpus_doc_pairs("labels") {
        check_corpus(&slice_path, &labels_path);
    }
}

fn check_corpus(slice_path: &str, labels_path: &str) {
    let (corpus, slice, labels) = eval_support::gate_docs("labels", slice_path, labels_path);
    let by = by_sha(&labels);
    let mut seen = 0;
    for s in slice["commits"].as_array().expect("commits") {
        let moved = pair_sum(s, "added_moved") + pair_sum(s, "removed_moved");
        let sha = s["sha"].as_str().expect("sha");
        match by.get(sha) {
            None => assert_eq!(moved, 0, "{sha}: moved-bearing but unreviewed"),
            Some(l) => {
                seen += 1;
                check_row(corpus.as_deref(), sha, s, l, moved);
            }
        }
    }
    assert_eq!(seen, by.len(), "labels rows outside the slice");
    let rows: Vec<Value> = labels["commits"].as_array().unwrap().to_vec();
    assert_eq!(labels["summary"], summarize(&rows), "summary drifted");
}

fn check_row(corpus: Option<&str>, sha: &str, s: &Value, l: &Value, slice_moved: u64) {
    for (sp, lp) in s["pairs"]
        .as_array()
        .unwrap()
        .iter()
        .zip(l["pairs"].as_array().expect("pairs"))
    {
        let g = |v: &Value, k: &str| v[k].as_u64().unwrap();
        assert_eq!(lp["before"], sp["before"], "{sha}: pair order");
        assert_eq!(lp["after"], sp["after"], "{sha}: pair order");
        assert_eq!(lp["copied"], sp["copied"], "{sha}: copy marker drifted");
        let add = g(lp, "added_novel") + g(lp, "added_moved");
        assert_eq!(g(lp, "added"), add, "{sha}");
        let rem = g(lp, "removed_deleted") + g(lp, "removed_moved");
        assert_eq!(g(lp, "deleted"), rem, "{sha}");
        assert_eq!(g(lp, "added"), g(sp, "added"), "{sha}: numstat drifted");
        assert_eq!(g(lp, "deleted"), g(sp, "deleted"), "{sha}: numstat drifted");
    }
    // The register is DATA (this corpus's review record, resolved BY
    // NAME — the gate iterates every frozen corpus); pin the frozen
    // row to it at line identity so editing either side alone reddens
    // CI (Codex review C3 — totals-only anchoring left register edits
    // invisible until manual regeneration).
    let reg: Vec<Value> = review::below_floor_in(corpus, sha)
        .into_iter()
        .map(|(side, file, line)| json!({"side": side, "file": file, "line": line}))
        .collect();
    match l.get("below_floor") {
        None => assert!(reg.is_empty(), "{sha}: register rows missing from labels"),
        Some(bf) => assert_eq!(bf, &json!(reg), "{sha}: register/labels drift"),
    }
    let g2 = |k: &str, side: &str| l[k][side].as_u64().unwrap();
    let final_moved = pair_sum(l, "added_moved") + pair_sum(l, "removed_moved");
    let partitioned = g2("cross_file", "out")
        + g2("cross_file", "in")
        + g2("within_file", "out")
        + g2("within_file", "in");
    assert_eq!(final_moved, partitioned, "{sha}: partition mismatch");
    let corr: u64 = l["corrections"]
        .as_array()
        .expect("corrections")
        .iter()
        .map(|c| c["lines"].as_u64().unwrap())
        .sum();
    let reclassified = g2("nonsignificant", "out") + g2("nonsignificant", "in") + corr;
    assert_eq!(slice_moved, final_moved + reclassified, "{sha}: ledger");
}
