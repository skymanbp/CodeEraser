//! Scoring machinery for the L2 bar (eval_l2.rs): batch runs, the
//! L2-over-L1 delta, per-file cross ground truth via the labels
//! machinery, gate rows, and the extras ledger.
//!
//! Compiled only by eval_l2; split out to keep both files inside the
//! repo's own E01 file budget.
#![allow(dead_code)]

use crate::eval_commit_review as review;
use crate::eval_support::{pair_contents, pair_lang, u64s};
use codeeraser::corelink::Link;
use codeeraser::fourclass::batch::{BatchClassification, PairInput, classify_batch};
use serde_json::{Value, json};
use std::collections::BTreeMap;

pub type Texts = Vec<(String, String, Value)>;
pub type FileKey = (String, String); // (side, file)

/// (before text, after text, gt pair row) per pair, in slice order.
pub fn pair_texts(sha: &str, gt_rows: &[Value]) -> Texts {
    gt_rows
        .iter()
        .map(|gp| {
            let (before, after) = pair_contents(sha, gp);
            (before, after, gp.clone())
        })
        .collect()
}

pub fn run_batch(texts: &Texts, link: Option<&mut Link>) -> BatchClassification {
    let inputs: Vec<PairInput> = texts
        .iter()
        .map(|(before, after, gp)| PairInput {
            before,
            after,
            lang: pair_lang(gp),
        })
        .collect();
    let out = classify_batch(&inputs, link);
    for (c, (_, _, gp)) in out.pairs.iter().zip(texts) {
        assert!(!c.degraded, "diff cap tripped on {gp}");
    }
    out
}

pub fn counts_of(b: &BatchClassification) -> Vec<[u64; 4]> {
    b.pairs
        .iter()
        .map(|c| {
            [
                c.counts.added_novel as u64,
                c.counts.added_moved as u64,
                c.counts.removed_deleted as u64,
                c.counts.removed_moved as u64,
            ]
        })
        .collect()
}

/// The side's text for a (side, file) key within one batch.
pub fn side_text<'a>(texts: &'a Texts, side: &str, file: &str) -> &'a str {
    texts
        .iter()
        .find_map(|(before, after, gp)| {
            if side == "out" && gp["before"].as_str() == Some(file) {
                Some(before.as_str())
            } else if side == "in" && gp["after"].as_str() == Some(file) {
                Some(after.as_str())
            } else {
                None
            }
        })
        .unwrap_or_else(|| panic!("{side} {file} not in batch"))
}

/// The L2-over-L1 delta as (side, file) -> sorted line list. Side
/// "out" keys the before path, "in" the after path.
pub fn delta_lines(
    texts: &Texts,
    l1: &BatchClassification,
    l2: &BatchClassification,
) -> BTreeMap<FileKey, Vec<usize>> {
    let mut out: BTreeMap<FileKey, Vec<usize>> = BTreeMap::new();
    for ((a, b), (_, _, gp)) in l1.pairs.iter().zip(&l2.pairs).zip(texts) {
        let base: Vec<(usize, bool)> = a.moved.iter().map(|m| (m.line, m.removed)).collect();
        for m in &b.moved {
            if base.contains(&(m.line, m.removed)) {
                continue;
            }
            let (side, path) = if m.removed {
                ("out", gp["before"].as_str().expect("before"))
            } else {
                ("in", gp["after"].as_str().expect("after"))
            };
            out.entry((side.into(), path.into()))
                .or_default()
                .push(m.line);
        }
    }
    out.values_mut().for_each(|v| v.sort_unstable());
    out
}

/// Reviewed per-file cross GT via the labels machinery (mechanical
/// partition + the reviewed corrections), same code that generated
/// commit-labels-v1.json.
pub fn cross_gt(sha: &str) -> BTreeMap<FileKey, u64> {
    let mut p = review::partition(sha);
    let _ = review::apply_corrections(sha, &mut p);
    let mut out = BTreeMap::new();
    let mut insert = |side: &str, per_file: &std::collections::HashMap<String, u64>| {
        for (f, n) in per_file {
            out.insert((side.to_string(), f.clone()), *n);
        }
    };
    insert("out", &p.out.cross);
    insert("in", &p.into.cross);
    out
}

/// Per-commit cross rows {file, side, gt, pred}; misses must be zero.
/// A miss dumps the predicted lines with content so the divergence is
/// diagnosable from the failure alone.
pub fn cross_rows(
    sha: &str,
    texts: &Texts,
    gt: &BTreeMap<FileKey, u64>,
    delta: &BTreeMap<FileKey, Vec<usize>>,
) -> Vec<Value> {
    let mut keys: Vec<&FileKey> = gt.keys().chain(delta.keys()).collect();
    keys.sort();
    keys.dedup();
    keys.iter()
        .map(|key| {
            let g = gt.get(*key).copied().unwrap_or(0);
            let lines = delta.get(*key).cloned().unwrap_or_default();
            let p = lines.len() as u64;
            assert!(
                p >= g,
                "{sha}: cross miss on {key:?}: gt {g} pred {p}\npred lines:\n{}",
                dump(texts, key, &lines)
            );
            json!({"side": key.0, "file": key.1, "gt": g, "pred": p})
        })
        .collect()
}

fn dump(texts: &Texts, (side, file): &FileKey, lines: &[usize]) -> String {
    let text: Vec<&str> = side_text(texts, side, file).lines().collect();
    lines
        .iter()
        .map(|&l| format!("  {l}: {}\n", text[l - 1].trim()))
        .collect()
}

/// Extras (pred above GT) itemized with content for review.
pub fn extras_ledger(
    sha: &str,
    texts: &Texts,
    gt: &BTreeMap<FileKey, u64>,
    delta: &BTreeMap<FileKey, Vec<usize>>,
) -> Vec<Value> {
    delta
        .iter()
        .filter(|(key, lines)| (lines.len() as u64) > gt.get(*key).copied().unwrap_or(0))
        .map(|((side, file), lines)| {
            let text: Vec<&str> = side_text(texts, side, file).lines().collect();
            let dump: Vec<String> = lines
                .iter()
                .map(|&l| format!("{l}: {}", text[l - 1].trim()))
                .collect();
            json!({"sha": &sha[..9], "side": side, "file": file,
                   "gt": gt.get(&(side.clone(), file.clone())).copied().unwrap_or(0),
                   "pred": lines.len(), "lines": dump})
        })
        .collect()
}

/// A cross row's (gt, pred) counts.
pub fn gt_pred(c: &Value) -> (u64, u64) {
    (c["gt"].as_u64().unwrap(), c["pred"].as_u64().unwrap())
}

/// Everything re-derivable from the committed rows (CI gate re-runs).
pub fn summarize(rows: &[Value], ledger: &[Value]) -> Value {
    let mut sums: BTreeMap<&str, u64> = BTreeMap::new();
    let mut add = |k: &'static str, v: u64| *sums.entry(k).or_default() += v;
    for r in rows {
        let mut exact = true;
        for p in r["pairs"].as_array().expect("pairs") {
            add("pairs", 1);
            exact &= p["l2"] == p["gt"];
            add("pairs_exact", (p["l2"] == p["gt"]) as u64);
            let (gt, l2) = (u64s(&p["gt"]), u64s(&p["l2"]));
            for i in [1usize, 3] {
                add("moved_gt", gt[i]);
                add("moved_pred", l2[i]);
                add("moved_detected", gt[i].min(l2[i]));
            }
        }
        add("commits", 1);
        add("commits_exact", exact as u64);
        for c in r["cross"].as_array().expect("cross") {
            let (g, p) = gt_pred(c);
            let side = c["side"].as_str().unwrap();
            add(
                if side == "out" {
                    "cross_gt_out"
                } else {
                    "cross_gt_in"
                },
                g,
            );
            add("cross_hits", g.min(p));
            add("cross_misses", g.saturating_sub(p));
        }
    }
    add("extras_files", ledger.len() as u64);
    add(
        "extras_lines",
        ledger
            .iter()
            .map(|e| e["pred"].as_u64().unwrap() - e["gt"].as_u64().unwrap())
            .sum(),
    );
    json!(sums)
}
