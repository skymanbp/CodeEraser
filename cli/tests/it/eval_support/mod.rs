//! Shared helpers for the eval_* test binaries (labels gate, prelabel
//! generator, baselines, commit slice). Extracted when the repo's own
//! dedup ratchet caught these copied verbatim across the test files —
//! the exact defect class this project exists to stop.
//!
//! Each integration-test binary compiles this module independently and
//! uses a different subset, so the unused remainder is expected.
#![allow(dead_code)]
#![allow(unused_imports)]

pub mod auditgen;
pub mod corpus;
pub mod dedup;
pub mod docseg;
pub mod family;
pub mod graph;
pub mod precision;
pub mod provenance;
pub mod t3c;
pub mod t3f;
pub mod universe;
pub use auditgen::*;
pub use corpus::*;
pub use dedup::*;
pub use docseg::*;
pub use family::*;
pub use graph::*;
pub use precision::*;
pub use provenance::*;
pub use t3c::*;
pub use t3f::*;
pub use universe::*;

use codeeraser::scan::lang::Lang;
use serde_json::Value;
use std::collections::HashMap;

/// Manifest lang codes and file extensions share one vocabulary.
pub fn lang_of(code: &str) -> Lang {
    match code {
        "py" => Lang::Python,
        "ts" => Lang::TypeScript,
        "rs" => Lang::Rust,
        "go" => Lang::Go,
        "md" => Lang::Markdown,
        other => panic!("unexpected lang {other}"),
    }
}

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

/// sha256 hex of the text the detector saw — the graph docs' content
/// identity (moved here from eval_graph.rs when the sample instrument
/// became its second consumer).
pub fn content_sha(text: &str) -> String {
    use sha2::Digest;
    let mut h = sha2::Sha256::new();
    h.update(text.as_bytes());
    h.finalize().iter().map(|b| format!("{b:02x}")).collect()
}

/// Per-kind site counts, the shape frozen in every graph-slice file
/// row (same relocation as content_sha).
pub fn kind_counts(
    sites: &[codeeraser::graph::sites::RawSite],
) -> std::collections::BTreeMap<&'static str, u64> {
    let mut kinds = std::collections::BTreeMap::new();
    for s in sites {
        *kinds.entry(s.kind).or_insert(0) += 1;
    }
    kinds
}

/// Index a doc's row array by one of its string fields.
pub fn by_field<'a>(doc: &'a Value, key: &str, field: &str) -> HashMap<&'a str, &'a Value> {
    doc[key]
        .as_array()
        .expect(key)
        .iter()
        .map(|r| (r[field].as_str().expect(field), r))
        .collect()
}

pub fn by_id<'a>(doc: &'a Value, key: &str) -> HashMap<&'a str, &'a Value> {
    by_field(doc, key, "id")
}

/// Commit-slice/labels/baseline docs all key their rows by sha.
pub fn by_sha(doc: &Value) -> HashMap<&str, &Value> {
    by_field(doc, "commits", "sha")
}

/// The canonical path of a contracts/eval document, derived from its
/// name so a generator and its CI gate can never drift apart.
pub fn eval_doc(name: &str) -> String {
    format!("../contracts/eval/{name}-v1.json")
}

/// A JSON array of integers as `Vec<u64>`.
pub fn u64s(v: &Value) -> Vec<u64> {
    v.as_array()
        .expect("u64 array")
        .iter()
        .map(|x| x.as_u64().expect("u64"))
        .collect()
}

/// Anchor a doc summary against its labels doc: the cross GT totals
/// must match the frozen labels, and the doc's below-floor total
/// (stored under `bf_key`, absent = empty register) must equal the
/// labels-side register total. Returns (cross GT total, below-floor
/// total) for the caller's conservation check — the one anchor both
/// the L2 and ablation gates share.
pub fn anchor_to_labels(s: &Value, labels: &Value, bf_key: &str) -> (u64, u64) {
    let ml = &labels["summary"]["moved_lines"];
    assert_eq!(s["cross_gt_out"], ml["cross_out"], "cross GT anchor");
    assert_eq!(s["cross_gt_in"], ml["cross_in"], "cross GT anchor");
    let bf = s[bf_key].as_u64().unwrap_or(0);
    let lbf = labels["summary"]["below_floor_lines"].as_u64().unwrap_or(0);
    assert_eq!(bf, lbf, "below-floor register anchor");
    let gt = s["cross_gt_out"].as_u64().unwrap() + s["cross_gt_in"].as_u64().unwrap();
    (gt, bf)
}
