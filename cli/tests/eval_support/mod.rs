//! Shared helpers for the eval_* test binaries (labels gate, prelabel
//! generator, baseline). Extracted when the repo's own dedup ratchet
//! caught these copied verbatim across the three files — the exact
//! defect class this project exists to stop.
//!
//! Each integration-test binary compiles this module independently and
//! uses a different subset, so the unused remainder is expected.
#![allow(dead_code)]

use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

/// The four ground-truth line classes, in canonical order.
pub const CLASSES: [&str; 4] = [
    "added_novel",
    "added_moved",
    "removed_deleted",
    "removed_moved",
];

pub fn load(path: &str) -> Value {
    serde_json::from_str(&std::fs::read_to_string(path).expect(path)).expect(path)
}

pub fn by_id<'a>(doc: &'a Value, key: &str) -> HashMap<&'a str, &'a Value> {
    doc[key]
        .as_array()
        .expect(key)
        .iter()
        .map(|r| (r["id"].as_str().expect("id"), r))
        .collect()
}

/// The local eval-payload directory (eval_extract output).
pub fn out_dir() -> std::path::PathBuf {
    std::path::PathBuf::from(std::env::var("CE_EVAL_OUT").unwrap_or_else(|_| "../.ce-eval".into()))
}

pub fn read_sample(dir: &Path, id: &str) -> Value {
    let path = dir.join("samples").join(format!("{id}.json"));
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("{}: {e} — regenerate via eval_extract", path.display()));
    serde_json::from_str(&text).expect("sample json")
}

/// The manifest rows flagged for the 200-sample labeling subset.
pub fn labeling_rows(manifest: &Value) -> Vec<&Value> {
    manifest["samples"]
        .as_array()
        .expect("samples")
        .iter()
        .filter(|r| r["labeling"].as_bool() == Some(true))
        .collect()
}

/// Every generated document carries exactly the 200 labeling rows,
/// sorted by id for stable diffs.
pub fn finish_rows(rows: &mut [Value]) {
    assert_eq!(rows.len(), 200, "labeling subset size");
    rows.sort_by(|a, b| a["id"].as_str().cmp(&b["id"].as_str()));
}

/// Write a generated contracts/eval document (pretty + trailing
/// newline) and announce it.
pub fn write_doc(path: &str, doc: &Value, done: &str) {
    std::fs::write(path, serde_json::to_string_pretty(doc).expect("ser") + "\n").expect(path);
    println!("{done}");
}

/// `git diff --no-index --numstat [extra…] a b` → (added, deleted).
/// `extra` lets the baseline pass the plan-literal `-M -C
/// --find-copies-harder` while the prelabel pass runs plain.
pub fn numstat(a: &Path, b: &Path, extra: &[&str]) -> (u64, u64) {
    let out = Command::new("git")
        .args(["diff", "--no-index", "--numstat"])
        .args(extra)
        .arg(a)
        .arg(b)
        .output()
        .expect("git diff numstat");
    let text = String::from_utf8_lossy(&out.stdout);
    let mut fields = text.split_whitespace();
    let add = fields.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    let del = fields.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    (add, del)
}
