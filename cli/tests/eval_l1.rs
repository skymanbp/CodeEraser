//! M4-4 L1 evaluation: run the fourclass classifier (Myers diff +
//! tree-sitter function-boundary alignment) over the 200-sample
//! labeling subset and score it against the finalized ground truth.
//!
//! Needs the local .ce-eval payloads, so the scorer is an ignored
//! generator (same pattern as eval_prelabel / eval_baseline); the
//! committed l1-v1.json is the record. A non-ignored CI gate checks
//! the file's internal consistency (summary counts re-derivable from
//! its own rows).
//!
//! Run: CE_EVAL_OUT=<dir> cargo test --test eval_l1 -- --ignored --nocapture

mod eval_support;

use eval_support::{CLASSES, by_id, load};
use serde_json::{Value, json};

/// Score rows and assemble the document. Kept free of file I/O so the
/// CI gate can re-run it on the committed rows.
fn summarize(rows: &[Value]) -> Value {
    let mut exact = 0u64;
    let (mut moved_gt, mut moved_pred, mut moved_hit) = (0u64, 0u64, 0u64);
    let mut mismatched = Vec::new();
    for r in rows {
        let (gt, pred) = (&r["gt"], &r["pred"]);
        if gt == pred {
            exact += 1;
        } else {
            mismatched.push(r["id"].clone());
        }
        for i in [1usize, 3] {
            let (g, p) = (gt[i].as_u64().unwrap(), pred[i].as_u64().unwrap());
            moved_gt += g;
            moved_pred += p;
            moved_hit += g.min(p);
        }
    }
    json!({
        "sample_exact": exact,
        "sample_total": rows.len(),
        "moved_lines": {"ground_truth": moved_gt, "predicted": moved_pred,
                        "detected": moved_hit},
        "mismatched_samples": mismatched,
    })
}

/// CI gate: the committed summary must re-derive from the committed
/// per-sample rows, and every gt row must match labels-v1.json.
#[test]
fn l1_summary_consistent_with_rows() {
    let doc = load("../contracts/eval/l1-v1.json");
    let lab = load("../contracts/eval/labels-v1.json");
    let labels = by_id(&lab, "labels");
    let rows = doc["rows"].as_array().expect("rows");
    assert_eq!(rows.len(), 200);
    for r in rows {
        let id = r["id"].as_str().expect("id");
        for (i, class) in CLASSES.iter().enumerate() {
            assert_eq!(r["gt"][i], labels[id][class], "{id}: gt drifted");
        }
    }
    let rows_owned: Vec<Value> = rows.to_vec();
    assert_eq!(doc["summary"], summarize(&rows_owned), "summary drifted");
}
