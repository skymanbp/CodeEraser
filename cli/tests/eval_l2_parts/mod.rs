//! Scoring machinery shared by the L2 bar (eval_l2.rs) and the
//! ablation shadow (eval_ablation.rs): batch runs, the L2-over-L1
//! delta, per-file cross GT via the labels machinery (cross.rs),
//! extras ledger.
#![allow(dead_code)]

pub mod cross;
// Re-exported so consumers keep the flat parts:: paths; each test
// binary compiles this module separately and uses its own subset.
#[allow(unused_imports)]
pub use cross::{CrossGt, cross_gt, cross_rows, extras_ledger, gt_pred};

use crate::eval_support::{by_sha, gt_pairs, pair_contents, pair_lang, u64s};
use codeeraser::corelink::Link;
use codeeraser::fourclass::MovedLine;
use codeeraser::fourclass::batch::{BatchClassification, PairInput, classify_batch};
use serde_json::{Value, json};
use std::collections::BTreeMap;

pub type Texts = Vec<(String, String, Value)>;
pub type FileKey = (String, String); // (side, file)

/// Whether a labels row records any cross-file moved lines.
pub fn has_cross(row: Option<&&Value>) -> bool {
    let n = |l: &&Value, k: &str| l["cross_file"][k].as_u64().unwrap();
    row.map(|l| n(l, "out") + n(l, "in") > 0).unwrap_or(false)
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

/// (pair index, moved line) present in l2 but not in l1 — the
/// shared substrate of the per-file delta and the unit naming.
pub fn delta_moved<'a>(
    l1: &'a BatchClassification,
    l2: &'a BatchClassification,
) -> Vec<(usize, &'a MovedLine)> {
    let mut out = Vec::new();
    for (i, (a, b)) in l1.pairs.iter().zip(&l2.pairs).enumerate() {
        let base: Vec<(usize, bool)> = a.moved.iter().map(|m| (m.line, m.removed)).collect();
        for m in &b.moved {
            if !base.contains(&(m.line, m.removed)) {
                out.push((i, m));
            }
        }
    }
    out
}

/// The L2-over-L1 delta as (side, file) -> sorted line list. Side
/// "out" keys the before path, "in" the after path.
pub fn delta_lines(
    texts: &Texts,
    l1: &BatchClassification,
    l2: &BatchClassification,
) -> BTreeMap<FileKey, Vec<usize>> {
    let mut out: BTreeMap<FileKey, Vec<usize>> = BTreeMap::new();
    for (i, m) in delta_moved(l1, l2) {
        let gp = &texts[i].2;
        let (side, path) = if m.removed {
            ("out", gp["before"].as_str().expect("before"))
        } else {
            ("in", gp["after"].as_str().expect("after"))
        };
        out.entry((side.into(), path.into()))
            .or_default()
            .push(m.line);
    }
    out.values_mut().for_each(|v| v.sort_unstable());
    out
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
            // Reviewed below-floor lines leave the hit/miss ledger
            // (hits + misses + below_floor == cross GT), so the miss
            // gate reads "zero UNREVIEWED misses" on every corpus.
            let w = c["below_floor"].as_array().map_or(0, |a| a.len() as u64);
            add("cross_hits", (g - w).min(p));
            add("cross_misses", (g - w).saturating_sub(p));
            if w > 0 {
                add("below_floor_lines", w);
            }
        }
    }
    add("extras_files", ledger.len() as u64);
    add(
        "extras_lines",
        ledger
            .iter()
            // charged against the recoverable bar gt − below_floor
            // (pred + bf > gt guaranteed by the ledger filter), the
            // same charge as the ablation scorer (Codex review C1)
            .map(|e| {
                e["pred"].as_u64().unwrap() + e["below_floor"].as_u64().unwrap_or(0)
                    - e["gt"].as_u64().unwrap()
            })
            .sum(),
    );
    json!(sums)
}
