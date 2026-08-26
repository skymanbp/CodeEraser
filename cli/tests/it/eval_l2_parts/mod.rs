//! Scoring machinery for the L2 bar (eval_l2.rs): the committed-row
//! summary, its cross fold and the labels-row predicate (the batch
//! delta and per-file GT builders retired with the ablation shadow,
//! git history).

use crate::eval_support::u64s;
use serde_json::{Value, json};
use std::collections::BTreeMap;

/// Whether a labels row records any cross-file moved lines.
pub fn has_cross(row: Option<&&Value>) -> bool {
    let n = |l: &&Value, k: &str| l["cross_file"][k].as_u64().unwrap();
    row.map(|l| n(l, "out") + n(l, "in") > 0).unwrap_or(false)
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
            add_cross(c, &mut add);
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

/// The per-row cross fold (split from summarize at the E01 fn gate):
/// side bucketing, the below-floor subtraction, hits and misses.
fn add_cross(c: &Value, add: &mut impl FnMut(&'static str, u64)) {
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

/// A cross row's (gt, pred) counts.
pub fn gt_pred(c: &Value) -> (u64, u64) {
    (c["gt"].as_u64().unwrap(), c["pred"].as_u64().unwrap())
}
