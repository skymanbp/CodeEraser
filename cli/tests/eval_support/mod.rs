//! Shared helpers for the eval_* test binaries (labels gate, prelabel
//! generator, baselines, commit slice). Extracted when the repo's own
//! dedup ratchet caught these copied verbatim across the test files —
//! the exact defect class this project exists to stop.
//!
//! Each integration-test binary compiles this module independently and
//! uses a different subset, so the unused remainder is expected —
//! including the colordiff re-export in binaries that never touch
//! colored diffs.
#![allow(dead_code)]
#![allow(unused_imports)]

pub mod colordiff;
pub mod corpus;
pub mod graph;
pub use colordiff::*;
pub use corpus::*;
pub use graph::*;

use codeeraser::fourclass::batch::PairInput;
use codeeraser::scan::lang::Lang;
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;

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

/// Ground-truth pairs for a commit: the reviewed labels row when one
/// exists, else the slice prelabels (zero moved marks — nothing to
/// review, labels equal prelabels verbatim).
pub fn gt_pairs<'a>(slice_row: &'a Value, labels: &HashMap<&str, &'a Value>) -> &'a Vec<Value> {
    let sha = slice_row["sha"].as_str().expect("sha");
    let row = labels.get(sha).copied().unwrap_or(slice_row);
    row["pairs"].as_array().expect("pairs")
}

/// File content at `rev`, empty for the created/deleted side.
pub fn file_at(rev: &str, path: Option<&str>) -> String {
    match path {
        None => String::new(),
        Some(p) => git_run(&["show", &format!("{rev}:{p}")], false),
    }
}

/// The classification language of a slice pair, from its extension.
pub fn pair_lang(pair: &Value) -> Lang {
    let path = pair["after"]
        .as_str()
        .or(pair["before"].as_str())
        .expect("pair has a side");
    lang_of(path.rsplit('.').next().expect("extension"))
}

/// Both sides of a slice pair at `sha` (before side from `sha^`).
pub fn pair_contents(sha: &str, pair: &Value) -> (String, String) {
    let base = format!("{sha}^");
    (
        file_at(&base, pair["before"].as_str()),
        file_at(sha, pair["after"].as_str()),
    )
}

/// Iterate doc rows that carry sample ids, yielding each id with its
/// loaded local payload (eval_extract output).
pub fn each_sample(rows: &[Value], mut f: impl FnMut(&str, Value)) {
    let dir = out_dir();
    for row in rows {
        let id = row["id"].as_str().expect("id");
        f(id, read_sample(&dir, id));
    }
}

/// A one-pair classify_batch input built from an eval sample payload
/// — the manifest-replay shape shared by the L1-identity and FPR runs.
pub fn sample_pair(sample: &Value) -> [PairInput<'_>; 1] {
    [PairInput {
        before: sample["before"].as_str().expect("before"),
        after: sample["after"].as_str().expect("after"),
        lang: lang_of(sample["lang"].as_str().expect("lang")),
    }]
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

/// A labels/slice pair row's four ground-truth class counts.
pub fn gt_counts(pair: &Value) -> Vec<u64> {
    CLASSES.iter().map(|c| pair[*c].as_u64().unwrap()).collect()
}

/// Run the fourclass classifier and return the four counts in
/// CLASSES order, refusing degraded (diff-capped) results.
pub fn classify_counts(before: &str, after: &str, lang: Lang, what: &str) -> [u64; 4] {
    let c = codeeraser::fourclass::classify(before, after, lang);
    assert!(!c.degraded, "{what}: diff cap tripped on an eval sample");
    [
        c.counts.added_novel as u64,
        c.counts.added_moved as u64,
        c.counts.removed_deleted as u64,
        c.counts.removed_moved as u64,
    ]
}

/// The live ce-core link every L2-family generator drives.
pub fn core_link() -> codeeraser::corelink::Link {
    let bin = std::env::var("CE_CORE_BIN").expect(
        "CE_CORE_BIN is unset — build the core and export it:\n  \
         cd core && cabal build all && export CE_CORE_BIN=$(cabal list-bin ce-core)",
    );
    codeeraser::corelink::Link::open(&bin)
        .expect("ce-core link")
        .0
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

/// The source revision an eval doc was generated from — a committed
/// replay result is evidence only when bound to the code that
/// produced it (attack review F1). By convention the doc lands in
/// the CHILD of this commit (EVAL-SET.md), so `dirty` normally
/// covers only the doc files themselves.
pub fn generated_from() -> Value {
    let git = |args: &[&str]| -> String {
        let out = std::process::Command::new("git")
            .args(args)
            .output()
            .expect("git");
        String::from_utf8_lossy(&out.stdout).trim().to_string()
    };
    serde_json::json!({
        "commit": git(&["rev-parse", "HEAD"]),
        "dirty": !git(&["status", "--porcelain"]).is_empty(),
        "ce": env!("CARGO_PKG_VERSION"),
    })
}

/// Load the corpus's frozen commit slice, build the `family` doc from
/// it, write to the corpus's path for that family. Every derived
/// document shares this envelope and source (schema, source_slice,
/// method, summary, commits).
pub fn generate_commit_doc(
    family: &str,
    schema: &str,
    method: &str,
    build: impl FnOnce(&Value) -> (Value, Vec<Value>),
) {
    let c = corpus();
    let slice = load(&c.doc("slice"));
    let (summary, commits) = build(&slice);
    let doc = serde_json::json!({
        "schema": schema,
        "source_slice": slice["universe_tip"],
        "generated_from": generated_from(),
        "method": method,
        "summary": summary,
        "commits": commits,
    });
    let path = c.doc(family);
    write_doc(&path, &doc, &format!("{path} written"));
}
