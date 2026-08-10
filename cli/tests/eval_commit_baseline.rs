//! M4-5 L1-on-slice baseline: run the L1 classifier per file pair
//! over every commit of the slice and score it against the reviewed
//! ground truth (commit-labels-v1). This is the bar L2 must beat.
//!
//! Structural fact this records: L1 operates per file pair, so a line
//! that relocated to a *different* file has no within-pair partner
//! and classifies as novel/deleted — cross-file moved recall is zero
//! by construction, up to content coincidences inside a single pair
//! (bounded here by `cross_credit_upper_bound`).
//!
//! Honest-comparison caveat the other way: the ground truth's
//! within-file moved marks come from git's blocks mode (>=20-alnum
//! block floor), while L1 marks any significant matching line — L1
//! may legitimately claim sub-block moves the GT engine cannot see;
//! those surface as predicted-above-detected, not as errors.
//!
//! Run: cargo test --test eval_commit_baseline -- --ignored --nocapture

mod eval_support;

use eval_support::{CLASSES, by_sha, classify_counts, git_run, lang_of, load};
use serde_json::{Value, json};
use std::collections::HashMap;

/// File content at `rev`, empty for the created/deleted side.
fn file_at(rev: &str, path: Option<&str>) -> String {
    match path {
        None => String::new(),
        Some(p) => git_run(&["show", &format!("{rev}:{p}")], false),
    }
}

fn classify_pair(sha: &str, pair: &Value) -> [u64; 4] {
    let (before_path, after_path) = (pair["before"].as_str(), pair["after"].as_str());
    let path = after_path.or(before_path).expect("pair has a side");
    let ext = path.rsplit('.').next().expect("extension");
    let base = format!("{sha}^");
    let before = file_at(&base, before_path);
    let after = file_at(sha, after_path);
    classify_counts(&before, &after, lang_of(ext), &format!("{sha}:{path}"))
}

/// Ground-truth pairs for a commit: the reviewed labels row when one
/// exists, else the slice prelabels (zero moved marks — nothing to
/// review, labels equal prelabels verbatim).
fn gt_pairs<'a>(slice_row: &'a Value, labels: &HashMap<&str, &'a Value>) -> &'a Vec<Value> {
    let sha = slice_row["sha"].as_str().expect("sha");
    let row = labels.get(sha).copied().unwrap_or(slice_row);
    row["pairs"].as_array().expect("pairs")
}

/// Score one commit: per-pair gt/pred rows (the summary re-derives
/// every total from these).
fn score_commit(sha: &str, gt_rows: &[Value]) -> Value {
    let pairs: Vec<Value> = gt_rows
        .iter()
        .map(|gp| {
            let pred = classify_pair(sha, gp);
            let gt: Vec<u64> = CLASSES.iter().map(|c| gp[c].as_u64().unwrap()).collect();
            json!({"before": gp["before"], "after": gp["after"],
                   "gt": gt, "pred": pred.to_vec()})
        })
        .collect();
    json!({"sha": sha, "pairs": pairs})
}

#[derive(Default)]
struct Totals {
    pairs: u64,
    pairs_exact: u64,
    moved_gt: u64,
    moved_pred: u64,
    moved_detected: u64,
}

impl Totals {
    fn add(&mut self, gt: &[u64], pred: &[u64; 4]) {
        self.pairs += 1;
        if gt == pred.as_slice() {
            self.pairs_exact += 1;
        }
        for i in [1usize, 3] {
            self.moved_gt += gt[i];
            self.moved_pred += pred[i];
            self.moved_detected += gt[i].min(pred[i]);
        }
    }
}

/// Everything below re-derives from the committed rows (CI gate).
fn summarize(commits: &[Value], labels_doc: &Value) -> Value {
    let mut t = Totals::default();
    let mut commits_exact = 0u64;
    for c in commits {
        let mut exact = true;
        for p in c["pairs"].as_array().expect("pairs") {
            let gt: Vec<u64> = p["gt"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_u64().unwrap())
                .collect();
            let pred: Vec<u64> = p["pred"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_u64().unwrap())
                .collect();
            t.add(&gt, &[pred[0], pred[1], pred[2], pred[3]]);
            exact &= gt == pred;
        }
        commits_exact += exact as u64;
    }
    let ml = &labels_doc["summary"]["moved_lines"];
    let cross_gt = ml["cross_out"].as_u64().unwrap() + ml["cross_in"].as_u64().unwrap();
    let within_gt = ml["within_out"].as_u64().unwrap() + ml["within_in"].as_u64().unwrap();
    json!({
        "commits": commits.len(),
        "commits_exact": commits_exact,
        "pairs": t.pairs,
        "pairs_exact": t.pairs_exact,
        "moved_lines": {"ground_truth": t.moved_gt, "predicted": t.moved_pred,
                        "detected": t.moved_detected},
        "cross_file_gt": cross_gt,
        "within_file_gt": within_gt,
        "cross_credit_upper_bound": t.moved_detected.saturating_sub(within_gt),
    })
}

#[test]
#[ignore] // needs full (non-shallow) git history up to the slice tip
fn generate_commit_baseline() {
    eval_support::generate_commit_doc(
        "../contracts/eval/commit-baseline-l1-v1.json",
        "ce.eval-commit-baseline/1.0.0",
        "cli/src/fourclass classify() per file pair (empty side for \
         created/deleted files), summed per commit; gt = \
         commit-labels-v1 reviewed rows, slice prelabels for \
         zero-move commits. Cross-file moved recall is zero by \
         construction for a per-pair classifier; \
         cross_credit_upper_bound bounds within-pair coincidence \
         credit. GT within-file marks carry git's blocks-mode \
         >=20-alnum block floor; L1's line-level matching may \
         legitimately exceed them (predicted > detected).",
        |slice| {
            let labels_doc = load("../contracts/eval/commit-labels-v1.json");
            let labels = by_sha(&labels_doc);
            let commits: Vec<Value> = slice["commits"]
                .as_array()
                .expect("commits")
                .iter()
                .map(|s| score_commit(s["sha"].as_str().expect("sha"), gt_pairs(s, &labels)))
                .collect();
            (summarize(&commits, &labels_doc), commits)
        },
    );
}

/// CI gate: gt rows must match their source docs and the summary must
/// re-derive from the committed rows. No git needed.
#[test]
fn commit_baseline_consistent() {
    let slice = load("../contracts/eval/commit-slice-v1.json");
    let labels_doc = load("../contracts/eval/commit-labels-v1.json");
    let labels = by_sha(&labels_doc);
    let doc = load("../contracts/eval/commit-baseline-l1-v1.json");
    let commits = doc["commits"].as_array().expect("commits");
    let slice_rows = slice["commits"].as_array().expect("commits");
    assert_eq!(commits.len(), slice_rows.len(), "commit coverage");
    for (c, s) in commits.iter().zip(slice_rows) {
        assert_eq!(c["sha"], s["sha"], "commit order");
        let gt_rows = gt_pairs(s, &labels);
        for (p, gp) in c["pairs"].as_array().expect("pairs").iter().zip(gt_rows) {
            for (i, class) in CLASSES.iter().enumerate() {
                assert_eq!(p["gt"][i], gp[*class], "{}: gt drifted", c["sha"]);
            }
        }
    }
    let rows: Vec<Value> = commits.to_vec();
    assert_eq!(
        doc["summary"],
        summarize(&rows, &labels_doc),
        "summary drifted"
    );
}
