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

use codeeraser::fourclass;
use codeeraser::scan::lang::Lang;
use eval_support::{CLASSES, by_id, load};
use serde_json::{Value, json};

fn lang_of(name: &str) -> Lang {
    match name {
        "py" => Lang::Python,
        "ts" => Lang::TypeScript,
        "rs" => Lang::Rust,
        "go" => Lang::Go,
        "md" => Lang::Markdown,
        other => panic!("unexpected manifest lang {other}"),
    }
}

fn classify_sample(out_dir: &std::path::Path, id: &str, lang: &str) -> [u64; 4] {
    let sample = eval_support::read_sample(out_dir, id);
    let c = fourclass::classify(
        sample["before"].as_str().expect("before"),
        sample["after"].as_str().expect("after"),
        lang_of(lang),
    );
    assert!(!c.degraded, "{id}: diff cap tripped on an eval sample");
    [
        c.counts.added_novel as u64,
        c.counts.added_moved as u64,
        c.counts.removed_deleted as u64,
        c.counts.removed_moved as u64,
    ]
}

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

#[test]
#[ignore] // needs the local .ce-eval payloads (eval_extract output)
fn generate_l1_scores() {
    let manifest = load("../contracts/eval/manifest-v1.json");
    let lab = load("../contracts/eval/labels-v1.json");
    let labels = by_id(&lab, "labels");
    let out_dir = eval_support::out_dir();
    let mut rows = Vec::new();
    for s in eval_support::labeling_rows(&manifest) {
        let id = s["id"].as_str().expect("id");
        let pred = classify_sample(&out_dir, id, s["lang"].as_str().expect("lang"));
        let gt: Vec<u64> = CLASSES
            .iter()
            .map(|c| labels[id][c].as_u64().unwrap())
            .collect();
        rows.push(json!({"id": id, "gt": gt, "pred": pred.to_vec()}));
    }
    eval_support::finish_rows(&mut rows);
    let doc = json!({
        "schema": "ce.eval-l1/1.0.0",
        "source_manifest": lab["source_manifest"],
        "method": "cli/src/fourclass classify(): self-contained Myers line diff \
                   + significant-line move marking (indentation-insensitive, \
                   alphanumeric-bearing lines only, sides independent) + \
                   tree-sitter unit attribution. Saturation caveat: this eval \
                   set's moved ground truth cannot discriminate the full \
                   function-boundary machinery from blank-line filtering alone \
                   (user-acknowledged 2026-08-10); the alignment's added value \
                   is the per-line unit attribution and intact-relocation \
                   summary, exercised by lib unit tests.",
        "alignment_note": "every mismatched sample is a pure diff-alignment \
                   delta: the ground truth's totals inherit git's alignment \
                   via the numstat invariant, and git's diff is not minimal \
                   there — L1's Myers matches k identical blank-line pairs git \
                   split as del+add (symmetric -k/-k on novel/deleted, no \
                   significant line classified differently, moved exact). \
                   Independently confirmed with python difflib, which \
                   reproduces L1's counts on 4/5 samples exactly; on the \
                   fifth L1 is strictly more minimal than both.",
        "summary": summarize(&rows),
        "rows": rows,
    });
    eval_support::write_doc(
        "../contracts/eval/l1-v1.json",
        &doc,
        "scored 200 samples; summary written to contracts/eval/l1-v1.json",
    );
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
