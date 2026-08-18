//! M4-3 L0 baseline (plan §4.3 B3c): score the zero-custom-code
//! ladder rung against the finalized ground truth.
//!
//! Two variants, both derivable from the committed prelabels (so the
//! CI test needs no local payloads):
//! - `l0_numstat` — the plan-named `git diff --numstat -M -C
//!   --find-copies-harder`. For a single-file before/after pair the
//!   rename/copy flags are inert, so its four-class prediction is
//!   novel=added, deleted=removed, moved=0/0. The ignored test runs
//!   the literal command per sample to prove that equivalence.
//! - `reference_color_moved` — git's own move detection (the prelabel
//!   engine) scored against the corrected ground truth.
//!
//! The headline metric is the moved class, not overall line accuracy:
//! both variants sit near 99.5% on lines while one has 0% moved
//! recall and the other 100% recall at ~50% precision. L1 must beat
//! both — detect the genuine moves without the blank-line artifacts.

mod eval_support;

use eval_support::{CLASSES, by_id, load};
use serde_json::{Value, json};
use std::collections::HashMap;

/// Score one variant's per-sample predictions against the labels:
/// sample-exact count, per-class absolute line error, and moved-class
/// recall/precision at line-count granularity (min(pred, gt) counts
/// as detected — count-level, since neither variant names lines).
fn score(preds: &HashMap<&str, [u64; 4]>, labels: &HashMap<&str, &Value>) -> Value {
    let mut exact = 0u64;
    let mut abs_err: HashMap<&str, u64> = HashMap::new();
    let (mut moved_gt, mut moved_pred, mut moved_hit) = (0u64, 0u64, 0u64);
    let mut mismatched: Vec<Value> = Vec::new();
    let mut ids: Vec<&&str> = labels.keys().collect();
    ids.sort();
    for id in ids {
        let l = labels[*id];
        let p = preds[*id];
        let gt: Vec<u64> = CLASSES.iter().map(|c| l[c].as_u64().unwrap()).collect();
        if p.to_vec() == gt {
            exact += 1;
        } else {
            mismatched.push(json!({"id": id, "gt": gt, "pred": p.to_vec()}));
        }
        for (i, class) in CLASSES.iter().enumerate() {
            *abs_err.entry(class).or_insert(0) += p[i].abs_diff(gt[i]);
        }
        for i in [1usize, 3] {
            moved_gt += gt[i];
            moved_pred += p[i];
            moved_hit += p[i].min(gt[i]);
        }
    }
    json!({
        "sample_exact": exact,
        "sample_total": labels.len(),
        "abs_line_error": CLASSES.iter().map(|c| (*c, abs_err[c])).collect::<HashMap<_, _>>(),
        "moved_lines": {"ground_truth": moved_gt, "predicted": moved_pred,
                        "detected": moved_hit},
        "mismatched_samples": mismatched,
    })
}

fn variant_preds(pre: &Value, numstat_only: bool) -> HashMap<&str, [u64; 4]> {
    pre["prelabels"]
        .as_array()
        .expect("prelabels")
        .iter()
        .map(|r| {
            let g = |k: &str| r[k].as_u64().unwrap();
            let p = if numstat_only {
                [g("numstat_added"), 0, g("numstat_deleted"), 0]
            } else {
                [
                    g("added_novel"),
                    g("added_moved"),
                    g("removed_deleted"),
                    g("removed_moved"),
                ]
            };
            (r["id"].as_str().expect("id"), p)
        })
        .collect()
}

fn build_doc(pre: &Value, lab: &Value) -> Value {
    let labels = by_id(lab, "labels");
    json!({
        "schema": "ce.eval-baseline/1.0.0",
        "source_manifest": lab["source_manifest"],
        "method": "l0_numstat = git diff --no-index --numstat -M -C \
                   --find-copies-harder (plan §4.3 B3c; rename/copy flags are \
                   inert on single-file pairs — verified per sample by the \
                   ignored test); reference_color_moved = the prelabel engine \
                   scored against the corrected ground truth",
        "l0_numstat": score(&variant_preds(pre, true), &labels),
        "reference_color_moved": score(&variant_preds(pre, false), &labels),
    })
}

/// CI gate: the committed baseline must equal what the committed
/// prelabels + labels derive. No local payloads needed.
#[test]
fn baseline_l0_matches_derivation() {
    let pre = load("../contracts/eval/prelabels-v1.json");
    let lab = load("../contracts/eval/labels-v1.json");
    let committed = load("../contracts/eval/baseline-l0-v1.json");
    assert_eq!(committed, build_doc(&pre, &lab), "baseline drifted");
}
