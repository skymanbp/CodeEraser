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
use eval_support::{by_sha, load};
use review::{PerFile, total};
use serde_json::{Value, json};

fn pair_sum(row: &Value, key: &str) -> u64 {
    row["pairs"]
        .as_array()
        .expect("pairs")
        .iter()
        .map(|p| p[key].as_u64().unwrap())
        .sum()
}

fn out_in(out: u64, into: u64) -> Value {
    json!({"out": out, "in": into})
}

/// Move this pair-side's reclassified lines from `moved` into the
/// plain class, consuming the side's pending delta.
fn take_delta(left: &mut PerFile, path: &Value, moved: &mut u64, plain: &mut u64) {
    if let Some(d) = path.as_str().and_then(|f| left.remove(f)) {
        assert!(*moved >= d, "delta exceeds moved");
        *moved -= d;
        *plain += d;
    }
}

/// Final per-pair labels: slice prelabels minus every reclassified
/// line (nonsignificant + corrections) on its owning side.
fn adjust_pairs(row: &Value, out_delta: &PerFile, in_delta: &PerFile) -> Vec<Value> {
    let (mut out_left, mut in_left) = (out_delta.clone(), in_delta.clone());
    let pairs = row["pairs"].as_array().expect("pairs");
    let adjusted = pairs
        .iter()
        .map(|p| {
            let g = |k: &str| p[k].as_u64().unwrap();
            let (mut nov, mut mvi) = (g("added_novel"), g("added_moved"));
            let (mut del, mut mvo) = (g("removed_deleted"), g("removed_moved"));
            take_delta(&mut in_left, &p["after"], &mut mvi, &mut nov);
            take_delta(&mut out_left, &p["before"], &mut mvo, &mut del);
            json!({"before": p["before"], "after": p["after"],
                   "added": g("added"), "deleted": g("deleted"),
                   "added_novel": nov, "added_moved": mvi,
                   "removed_deleted": del, "removed_moved": mvo})
        })
        .collect();
    assert!(
        out_left.is_empty() && in_left.is_empty(),
        "unconsumed deltas"
    );
    adjusted
}

fn merge(a: &PerFile, b: &PerFile) -> PerFile {
    let mut out = a.clone();
    for (k, v) in b {
        *out.entry(k.clone()).or_default() += v;
    }
    out
}

fn label_row(row: &Value) -> Value {
    let sha = row["sha"].as_str().expect("sha");
    let mut p = review::partition(sha);
    let corrections = review::apply_corrections(sha, &mut p);
    let corr_out = review::per_file_corrections(sha, false);
    let corr_in = review::per_file_corrections(sha, true);
    let pairs = adjust_pairs(
        row,
        &merge(&p.out.nonsig, &corr_out),
        &merge(&p.into.nonsig, &corr_in),
    );
    let mut out = json!({
        "sha": sha,
        "pairs": pairs,
        "nonsignificant": out_in(total(&p.out.nonsig), total(&p.into.nonsig)),
        "cross_file": out_in(total(&p.out.cross), total(&p.into.cross)),
        "within_file": out_in(p.out.within, p.into.within),
        "corrections": corrections,
        "relocated_units": review::units_for(sha),
        "relocation_edges": review::edges_for(sha),
    });
    // Reviewed below-floor lines (still cross-file moved TRUTH above;
    // this is the waiver annotation). Absent when empty — corpora
    // without a register regenerate byte-identical.
    let bf: Vec<Value> = review::below_floor_for(sha)
        .into_iter()
        .map(|(side, file, line)| json!({"side": side, "file": file, "line": line}))
        .collect();
    if !bf.is_empty() {
        out["below_floor"] = json!(bf);
    }
    out
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

#[test]
#[ignore] // needs full (non-shallow) git history up to the slice tip
fn generate_commit_labels() {
    eval_support::generate_commit_doc(
        "labels",
        "ce.eval-commit-labels/1.0.0",
        "provenance semantics: moved = line that relocated as part \
                   of a block/unit relocation. Layer 1 (mechanical): moved \
                   marks without alphanumeric content reclassify to \
                   novel/deleted (labels-v1 convention, fourclass::significant). \
                   Layer 2 (mechanical): trimmed-content pairing across the \
                   commit partitions moved lines cross-file vs within-file, \
                   preferring within when both readings exist. Layer 3 \
                   (reviewed): content-coincidence pairs verified against raw \
                   diffs reclassify to novel/deleted — a fresh line duplicating \
                   removed content is the duplication signal, not a move. \
                   Commits with zero moved marks are unreviewed: their labels \
                   equal the slice prelabels verbatim.",
        |slice| {
            let rows: Vec<Value> = slice["commits"]
                .as_array()
                .expect("commits")
                .iter()
                .filter(|r| pair_sum(r, "added_moved") + pair_sum(r, "removed_moved") > 0)
                .map(label_row)
                .collect();
            (summarize(&rows), rows)
        },
    );
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
    let slice = load(slice_path);
    let labels = load(labels_path);
    let by = by_sha(&labels);
    let mut seen = 0;
    for s in slice["commits"].as_array().expect("commits") {
        let moved = pair_sum(s, "added_moved") + pair_sum(s, "removed_moved");
        let sha = s["sha"].as_str().expect("sha");
        match by.get(sha) {
            None => assert_eq!(moved, 0, "{sha}: moved-bearing but unreviewed"),
            Some(l) => {
                seen += 1;
                check_row(sha, s, l, moved);
            }
        }
    }
    assert_eq!(seen, by.len(), "labels rows outside the slice");
    let rows: Vec<Value> = labels["commits"].as_array().unwrap().to_vec();
    assert_eq!(labels["summary"], summarize(&rows), "summary drifted");
}

fn check_row(sha: &str, s: &Value, l: &Value, slice_moved: u64) {
    for (sp, lp) in s["pairs"]
        .as_array()
        .unwrap()
        .iter()
        .zip(l["pairs"].as_array().expect("pairs"))
    {
        let g = |v: &Value, k: &str| v[k].as_u64().unwrap();
        assert_eq!(lp["before"], sp["before"], "{sha}: pair order");
        assert_eq!(lp["after"], sp["after"], "{sha}: pair order");
        let add = g(lp, "added_novel") + g(lp, "added_moved");
        assert_eq!(g(lp, "added"), add, "{sha}");
        let rem = g(lp, "removed_deleted") + g(lp, "removed_moved");
        assert_eq!(g(lp, "deleted"), rem, "{sha}");
        assert_eq!(g(lp, "added"), g(sp, "added"), "{sha}: numstat drifted");
        assert_eq!(g(lp, "deleted"), g(sp, "deleted"), "{sha}: numstat drifted");
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
