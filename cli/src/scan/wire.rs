//! scan/1 wire codec (contracts/fixtures/scan/golden.ndjson is the
//! byte-level contract; corelink stamps proto/type/id): measurement
//! rows [code, value] out, positional levels and the fail bit back,
//! the effective grade table echoed WHOLE and pinned against the
//! rows this side sent (ADR-008 P3: ce.toml is the source, the
//! core's gradeTable holds the DEFAULTS, and the level judgment
//! never happens here — report.rs::evaluate is a pinned mirror for
//! the auxiliary surfaces, proven equal by scan::run's whole-report
//! ensure on every gate run). Only codes and values cross the wire;
//! subjects, names and paths never do (§5.9.2 index privacy).

use crate::config::Thresholds;
use anyhow::{Context, Result, ensure};
use serde_json::json;

/// Capability name the core's hello must offer (Protocol.hs).
pub const CAP: &str = "scan/1";

/// Row ceiling — mirror of CE.Scan.Cost.scanRowCap.
pub const SCAN_ROW_CAP: usize = 524288;

/// The grade rows ce.toml speaks: all seven codes every time, warn
/// and fail per row (fail 0 = no hard line), straight from
/// Thresholds — the source (the P4 knob-table pattern, third table
/// form). Code 6 (fn-naming) is the boolean row: warn 0, value 0/1.
pub fn grade_rows(t: &Thresholds) -> Vec<[u64; 3]> {
    vec![
        [0, t.file_lines_warn as u64, t.file_lines_fail as u64],
        [1, t.fn_lines_warn as u64, t.fn_lines_fail as u64],
        [2, t.params_warn as u64, 0],
        [3, t.cyclomatic_warn as u64, 0],
        [4, t.cognitive_warn as u64, 0],
        [5, t.nesting_warn as u64, 0],
        [6, 0, 0],
    ]
}

/// One scan.request over one link: levels come back positionally
/// (length-locked to the rows), the echoed grade table must be the
/// one this side sent, and a degraded reply to a client-sized
/// request is a cap-mirror drift error, never a judgment.
pub fn judge(core: &str, rows: &[[u64; 2]], grades: &[[u64; 3]]) -> Result<(Vec<u8>, bool)> {
    ensure!(
        rows.len() <= SCAN_ROW_CAP,
        "{} measurement rows exceed the scan/1 cap {SCAN_ROW_CAP}",
        rows.len()
    );
    let mut link = crate::lockstep::open_family(core, CAP)?;
    let reply = link
        .request("scan", json!({"rows": rows, "grades": grades}))
        .map_err(anyhow::Error::msg)?;
    ensure!(
        reply["degraded"] == json!(false),
        "core degraded a client-sized request ({}) — cap mirror drift (scan/wire.rs vs Scan/Cost.hs)",
        reply["reason"]
    );
    let echoed: Vec<[u64; 3]> =
        serde_json::from_value(reply["grades"].clone()).context("grades")?;
    ensure!(
        echoed == grades,
        "core judged with grade table {echoed:?}, ce sent {grades:?} — one table, two owners"
    );
    let levels: Vec<u8> = serde_json::from_value(reply["levels"].clone()).context("levels")?;
    ensure!(
        levels.len() == rows.len(),
        "core sent {} levels for {} rows",
        levels.len(),
        rows.len()
    );
    let fail = reply["fail"].as_bool().context("fail")?;
    Ok((levels, fail))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The default grade table mirrors Thresholds::default — the
    /// same numbers CE.Scan.Cost.gradeTable declares; the fixture
    /// book pins the core half, this pins the assembly half.
    #[test]
    fn default_grades_carry_every_code_in_order() {
        let rows = grade_rows(&Thresholds::default());
        assert_eq!(rows.len(), 7);
        assert!(rows.iter().enumerate().all(|(i, r)| r[0] == i as u64));
        assert_eq!(rows[0], [0, 300, 750]);
        assert_eq!(rows[1], [1, 50, 75]);
        assert_eq!(rows[6], [6, 0, 0]);
    }
}
