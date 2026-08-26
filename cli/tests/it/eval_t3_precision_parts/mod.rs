//! Scoring half of the T3 precision instrument (M5-3f): the verdict
//! semantics, the row→summary rescorer the generator AND the gate
//! share (T-G1: one scorer, so a cooked summary cannot exist), and
//! the θ cut table over the scored rows.

use serde_json::{Value, json};
use std::collections::BTreeMap;

/// Verdict classes of one scored sample row.
pub const VERDICTS: [&str; 4] = ["correct", "wrong", "not_clone", "dropped"];

/// Truths that make a judged-clone row CORRECT — the copy-lineage
/// partition (user-delegated mapping decision, 2026-08-13): `clone`
/// (the substitutability rule), `variant` (shared lineage, adapted
/// past substitution) and `t1t2` (verbatim/renamed duplication the
/// hot path also reports — decision ③'s whole-product credit).
/// `boilerplate` / `unrelated` / `generated` are the no-copy-lineage
/// classes — the calibration's FP families — and stay wrong.
pub const CORRECT_TRUTHS: [&str; 3] = ["clone", "variant", "t1t2"];

/// The published threshold grid (per-mille numerators over 100); the
/// release knob is the loosest θ clearing 0.85 — published, never
/// silently tuned.
pub const THETA_GRID: [i64; 7] = [70, 75, 80, 85, 90, 95, 100];

/// Clone verdict at an alternative threshold num/100 — same integer
/// cross-multiplication shape as the product's is_clone; the gate
/// pins the θ=85 column equal to the product-judged counts, so this
/// derivation cannot drift from the one binding unnoticed.
pub fn clone_at(ted: i64, n1: i64, n2: i64, num: i64) -> bool {
    let mx = n1.max(n2);
    (mx - ted) * 100 >= num * mx
}

/// The stored verdict recomputed from its own row (T-G6).
pub fn verdict_of(judgment: &str, judged_clone: bool, truth: &str) -> &'static str {
    match (judgment, judged_clone) {
        ("over_cap", _) | ("forest", _) => "dropped",
        (_, false) => "not_clone",
        (_, true) if CORRECT_TRUTHS.contains(&truth) => "correct",
        (_, true) => "wrong",
    }
}

/// The whole summary from the rows alone — verdict tallies, truth
/// conservation, per-language verdicts, the answered denominator and
/// the θ cut table. The CI gate re-runs this exact function; flat
/// tallies run through the shared tally_field throat.
pub fn rescore(rows: &[Value]) -> Value {
    let mut verdicts: BTreeMap<String, u64> = BTreeMap::new();
    let mut by_truth: BTreeMap<String, u64> = BTreeMap::new();
    super::eval_support::tally_field(rows, "verdict", &mut verdicts);
    super::eval_support::tally_field(rows, "truth", &mut by_truth);
    let mut by_lang: BTreeMap<String, BTreeMap<String, u64>> = BTreeMap::new();
    for r in rows {
        super::eval_support::tally_add(
            by_lang
                .entry(r["lang"].as_str().expect("lang").into())
                .or_default(),
            r["verdict"].as_str().expect("verdict"),
            1,
        );
    }
    let n = |k: &str| verdicts.get(k).copied().unwrap_or(0);
    json!({
        "verdicts": verdicts,
        "by_truth": by_truth,
        "by_lang": by_lang,
        "answered": n("correct") + n("wrong"),
        "theta": theta_table(rows),
    })
}

/// Precision at every grid threshold, over the SCORED rows only
/// (dropped rows have no ted to re-threshold): answered / correct /
/// wrong per θ, same truth mapping.
fn theta_table(rows: &[Value]) -> Value {
    let mut table = BTreeMap::new();
    for num in THETA_GRID {
        let (mut correct, mut wrong) = (0u64, 0u64);
        for r in rows {
            let (Some(ted), Some(n1), Some(n2)) =
                (r["ted"].as_i64(), r["n1"].as_i64(), r["n2"].as_i64())
            else {
                continue;
            };
            if clone_at(ted, n1, n2, num) {
                if CORRECT_TRUTHS.contains(&r["truth"].as_str().expect("truth")) {
                    correct += 1;
                } else {
                    wrong += 1;
                }
            }
        }
        table.insert(
            format!("{num}"),
            json!({"answered": correct + wrong, "correct": correct, "wrong": wrong}),
        );
    }
    json!(table)
}
