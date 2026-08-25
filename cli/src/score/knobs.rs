//! ADR-008 P4 knob plumbing: the DECLARED mapping tables between
//! ce.toml keys, verdict.request wire codes and the reply's echo
//! keys — code IS the array index (the wire numbers every table
//! 0..n densely), so each registry is a plain name list and its two
//! consumers (row assembly here, the round-trip assertion in
//! wire::judge) share it with zero scattered literals. The ceilings axis mapping used
//! to live as one inline expression in run() — the "size ceiling IS
//! file_lines_warn" semantics now has a named home.

use crate::config::{RulesCfg, ScoreCfg, Thresholds};
use anyhow::{Context, Result};

/// Score axes by NAME, index = wire axis code (decision ⑦ order):
/// the ce.toml [score.weights] key vocabulary.
pub const AXES: [&str; 7] = [
    "size",
    "complexity",
    "clone",
    "docdup",
    "deadcode",
    "churn",
    "cycle",
];

/// Reply echo keys, index = wire code, one list per knob table.
/// Codes 2..4 are the v0.6 zone knobs (plan v2.6): the hard line H,
/// P_max, and the soft-line exponent k.
pub const CEILING_KEYS: [&str; 5] = ["sizeCeil", "cocCeil", "sizeHard", "sizePMax", "softLineK"];
pub const THRESHOLD_KEYS: [&str; 7] = [
    "deadIndegCeil",
    "rewriteNum",
    "rewriteDen",
    "cochangeFloor",
    "violCost",
    "defaultWeight",
    "scoreScale",
];
pub const TOLERANCE_KEYS: [&str; 3] = ["tolNum", "tolDen", "tolAbs"];

/// The ceilings rows ce.toml speaks: codes 0/1 always (both have
/// config defaults — the 27b9bc2 road), code 2 = the hard line H
/// whenever one is declared (`file_lines_fail == 0` means "no hard
/// line" per the scan grade contract, and the core refuses a
/// ceiling below 1 — so the curve then scales on the core's default
/// 750), codes 3/4 only when ce.toml declares them (the trend
/// pattern: absent = the core default judges).
pub fn ceiling_rows(t: &Thresholds, s: &ScoreCfg) -> Vec<[i64; 2]> {
    let mut out = vec![[0, t.file_lines_warn as i64], [1, t.cognitive_warn as i64]];
    if t.file_lines_fail >= 1 {
        out.push([2, t.file_lines_fail as i64]);
    }
    if let Some(p) = s.size_penalty_max {
        out.push([3, p as i64]);
    }
    if let Some(k) = s.soft_line_k {
        out.push([4, k as i64]);
    }
    out
}

/// The rulepack's knob rows [classId, code, value] (plan v2.13 ①,
/// 3.1.0): each declared class's overrides on the ceilings codes
/// 0 / 1 / 2 — the sizeCeil / cocCeil / sizeHard shadows, no new
/// code minted — under its 1-based declaration index, leaving
/// (classId, code)-ascending as the wire demands. A class hard line
/// of 0 sends no code-2 row (the global reading: 0 = no hard line,
/// and the core refuses a ceiling below 1); the two warn lines send
/// whatever was declared, so a nonsense 0 refuses loudly instead of
/// vanishing.
pub fn class_knob_rows(rules: &RulesCfg) -> Vec<[i64; 3]> {
    let mut out = Vec::new();
    for (i, c) in rules.class.iter().enumerate() {
        let id = i as i64 + 1;
        let k = &c.knobs;
        // codes 0/1/2 shadow the ceilings table; code 3 is the class's
        // own ratchet allowance (5.1.0) and is the one knob whose ZERO
        // is meaningful, so it is not filtered like a line would be
        let declared = [
            (0, k.file_lines_warn),
            (1, k.cognitive_warn),
            (2, k.file_lines_fail.filter(|h| *h >= 1)),
            (3, k.ratchet_tolerance),
        ];
        out.extend(
            declared
                .iter()
                .filter_map(|&(code, v)| v.map(|v| [id, code, v as i64])),
        );
    }
    out
}

/// [code, value] rows from a value array whose index IS the code;
/// absent values send nothing and the core default judges.
fn rows(values: &[Option<u32>]) -> Vec<[i64; 2]> {
    values
        .iter()
        .enumerate()
        .filter_map(|(code, v)| v.map(|v| [code as i64, v as i64]))
        .collect()
}

/// Configured thresholds rows, THRESHOLD_KEYS order.
pub fn threshold_rows(s: &ScoreCfg) -> Vec<[i64; 2]> {
    rows(&[
        s.dead_indeg_ceil,
        s.rewrite_num,
        s.rewrite_den,
        s.cochange_floor,
        s.viol_cost,
        s.default_weight,
        s.score_scale,
    ])
}

pub fn tolerance_rows(s: &ScoreCfg) -> Vec<[i64; 2]> {
    rows(&[s.tol_num, s.tol_den, s.tol_abs])
}

/// weights rows from named ce.toml keys — an unknown axis name is a
/// loud config error (the deny_unknown_fields stance at the value
/// level), and the rows leave code-ascending as the wire demands.
pub fn weight_rows(s: &ScoreCfg) -> Result<Vec<[i64; 2]>> {
    let mut out = Vec::new();
    for (name, w) in &s.weights {
        let code = AXES
            .iter()
            .position(|n| n == name)
            .with_context(|| format!("ce.toml [score.weights]: unknown axis '{name}'"))?;
        out.push([code as i64, *w as i64]);
    }
    out.sort_unstable();
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The name→code weld is load-bearing wire vocabulary (review
    /// C3): the axis order IS the core's Score.penalties order, so
    /// this literal pin turns a reorder into a named red instead of
    /// a silently re-aimed weight — with the 2.8.0 echo assert as
    /// the runtime half of the same weld.
    #[test]
    fn axes_order_is_frozen_and_weight_rows_resolve_names() {
        assert_eq!(
            AXES,
            [
                "size",
                "complexity",
                "clone",
                "docdup",
                "deadcode",
                "churn",
                "cycle"
            ]
        );
        let mut cfg = ScoreCfg::default();
        cfg.weights.insert("clone".into(), 5);
        assert_eq!(weight_rows(&cfg).expect("known name"), vec![[2, 5]]);
        cfg.weights.insert("bogus".into(), 1);
        assert!(weight_rows(&cfg).is_err(), "unknown axis name refuses");
    }

    /// The class rows are the ceilings codes under a class index,
    /// (class, code)-ascending: an absent knob sends nothing, a hard
    /// line of 0 sends nothing (no hard line), a warn line of 0 still
    /// rides — the core's refusal is the loud reading.
    #[test]
    fn class_knob_rows_shadow_the_ceiling_codes_in_order() {
        use crate::config::{ClassCfg, ClassKnobs};
        let class = |w: Option<usize>, h: Option<usize>, c: Option<usize>| ClassCfg {
            name: "x".into(),
            globs: vec!["**".into()],
            knobs: ClassKnobs {
                file_lines_warn: w,
                file_lines_fail: h,
                cognitive_warn: c,
                // the scan-only fn lines (P3) never reach the score
                // rows — declared here to prove exactly that
                fn_lines_warn: Some(80),
                fn_lines_fail: Some(90),
                ratchet_tolerance: None,
            },
        };
        let rules = RulesCfg {
            class: vec![
                class(Some(400), Some(0), None),
                class(None, Some(900), Some(20)),
                class(Some(0), None, None),
            ],
        };
        assert_eq!(
            class_knob_rows(&rules),
            vec![[1, 0, 400], [2, 1, 20], [2, 2, 900], [3, 0, 0]]
        );
        assert!(class_knob_rows(&RulesCfg::default()).is_empty());
    }
}
