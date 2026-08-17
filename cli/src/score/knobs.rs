//! ADR-008 P4 knob plumbing: the DECLARED mapping tables between
//! ce.toml keys, verdict.request wire codes and the reply's echo
//! keys — code IS the array index (the wire numbers every table
//! 0..n densely), so each registry is a plain name list and the
//! three consumers (row assembly here, the round-trip assertion in
//! wire::judge, the default drift gate in tests/core_wire.rs) share
//! it with zero scattered literals. The ceilings axis mapping used
//! to live as one inline expression in run() — the "size ceiling IS
//! file_lines_warn" semantics now has a named home.

use crate::config::{ScoreCfg, Thresholds};
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
pub const CEILING_KEYS: [&str; 2] = ["sizeCeil", "cocCeil"];
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

/// The ceilings rows ce.toml always speaks (both axes have config
/// defaults, so both rows are always sent — the 27b9bc2 road).
pub fn ceiling_rows(t: &Thresholds) -> Vec<[i64; 2]> {
    vec![[0, t.file_lines_warn as i64], [1, t.cognitive_warn as i64]]
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
