//! scan/1 wire codec (contracts/fixtures/scan/golden.ndjson is the
//! byte-level contract; corelink stamps proto/type/id): measurement
//! rows [code, value] out, positional levels and the fail bit back,
//! the effective grade table echoed WHOLE and pinned against the
//! rows this side sent (ADR-008 P3: ce.toml is the source, the
//! core's gradeTable holds the DEFAULTS, and the level judgment
//! never happens here — report.rs::evaluate is a pinned mirror for
//! the auxiliary surfaces, proven equal by scan::run's whole-report
//! ensure on every gate run). Only codes, values and name-shape
//! facts cross the wire; subjects, names and paths never do
//! (§5.9.2 index privacy).

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
/// An incoherent ladder is refused HERE with the ce.toml keys named
/// (review C6: the core refuses it too, but a config mistake must
/// not surface as a wire refusal).
pub fn grade_rows(t: &Thresholds) -> Result<Vec<[u64; 3]>> {
    for (warn, fail, keys) in [
        (
            t.file_lines_warn,
            t.file_lines_fail,
            "file_lines_warn/file_lines_fail",
        ),
        (
            t.fn_lines_warn,
            t.fn_lines_fail,
            "fn_lines_warn/fn_lines_fail",
        ),
    ] {
        ensure!(
            fail == 0 || fail >= warn,
            "ce.toml [thresholds] {keys}: the fail line {fail} sits below the warn line {warn}"
        );
    }
    Ok(vec![
        [0, t.file_lines_warn as u64, t.file_lines_fail as u64],
        [1, t.fn_lines_warn as u64, t.fn_lines_fail as u64],
        [2, t.params_warn as u64, 0],
        [3, t.cyclomatic_warn as u64, 0],
        [4, t.cognitive_warn as u64, 0],
        [5, t.nesting_warn as u64, 0],
        [6, 0, 0],
    ])
}

/// Chunked scan judging over ONE link (review C5: the single-request
/// form errored out entirely past the row cap — rows grade
/// independently, so chunking is trivially sound): levels come back
/// positionally per chunk and concatenate, the fail bit ORs, the
/// echoed grade table must be the one this side sent every time, and
/// a degraded reply to a chunk-sized request is a cap-mirror drift
/// error, never a judgment. The naming facts table (2.30.0) rides
/// aligned: each chunk carries the facts of ITS code-6 rows.
pub fn judge(
    core: &str,
    rows: &[[u64; 2]],
    grades: &[[u64; 3]],
    naming: &[[i64; 5]],
) -> Result<(Vec<u8>, bool)> {
    let mut link = crate::lockstep::open_family(core, CAP)?;
    let (mut levels, mut fail) = (Vec::new(), false);
    for (chunk, facts) in chunk_plan(rows, naming, SCAN_ROW_CAP - grades.len()) {
        let reply = link
            .request(
                "scan",
                json!({"rows": chunk, "grades": grades, "naming": facts}),
            )
            .map_err(anyhow::Error::msg)?;
        ensure!(
            reply["degraded"] == json!(false),
            "core degraded a chunk-sized request ({}) — cap mirror drift (scan/wire.rs vs Scan/Cost.hs)",
            reply["reason"]
        );
        let echoed: Vec<[u64; 3]> =
            serde_json::from_value(reply["grades"].clone()).context("grades")?;
        ensure!(
            echoed == grades,
            "core judged with grade table {echoed:?}, ce sent {grades:?} — one table, two owners"
        );
        let chunk_levels: Vec<u8> =
            serde_json::from_value(reply["levels"].clone()).context("levels")?;
        ensure!(
            chunk_levels.len() == chunk.len(),
            "core sent {} levels for {} rows",
            chunk_levels.len(),
            chunk.len()
        );
        levels.extend(chunk_levels);
        fail |= reply["fail"].as_bool().context("fail")?;
    }
    Ok((levels, fail))
}

/// Greedy chunk split whose budget counts EVERY request dimension
/// the core's cap counts (the C15 lesson made structural — the old
/// rows-only `chunks(SCAN_ROW_CAP)` left no room for the grade
/// table, so the first chunk of a cap-sized tree degraded): a row
/// pays 1, a code-6 row pays 2 (its aligned naming fact travels
/// with it), and the caller reserves the grade table's rows. The
/// walk that prices code-6 rows is the walk that slices the facts
/// — alignment by construction.
fn chunk_plan<'a>(
    rows: &'a [[u64; 2]],
    naming: &'a [[i64; 5]],
    budget: usize,
) -> Vec<(&'a [[u64; 2]], &'a [[i64; 5]])> {
    let mut out = Vec::new();
    let (mut row0, mut fact0, mut facts, mut weight) = (0usize, 0usize, 0usize, 0usize);
    for (i, row) in rows.iter().enumerate() {
        let w = 1 + usize::from(row[0] == 6);
        if weight + w > budget && weight > 0 {
            out.push((&rows[row0..i], &naming[fact0..fact0 + facts]));
            (row0, fact0, facts, weight) = (i, fact0 + facts, 0, 0);
        }
        weight += w;
        facts += usize::from(row[0] == 6);
    }
    out.push((&rows[row0..], &naming[fact0..]));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The default grade table mirrors Thresholds::default — the
    /// same numbers CE.Scan.Cost.gradeTable declares; the fixture
    /// book pins the core half, this pins the assembly half.
    #[test]
    fn default_grades_carry_every_code_in_order() {
        let rows = grade_rows(&Thresholds::default()).expect("coherent defaults");
        assert_eq!(rows.len(), 7);
        assert!(rows.iter().enumerate().all(|(i, r)| r[0] == i as u64));
        assert_eq!(rows[0], [0, 300, 750]);
        assert_eq!(rows[1], [1, 50, 75]);
        assert_eq!(rows[6], [6, 0, 0]);
        // the C6 ladder guard names the ce.toml keys, and fail 0
        // stays a legal "no hard line"
        let raised = Thresholds {
            file_lines_warn: 800,
            ..Thresholds::default()
        };
        let err = grade_rows(&raised).expect_err("fail below warn refuses");
        assert!(err.to_string().contains("file_lines_warn"), "{err}");
        let unlined = Thresholds {
            file_lines_warn: 800,
            file_lines_fail: 0,
            ..Thresholds::default()
        };
        grade_rows(&unlined).expect("fail 0 = no hard line is coherent");
    }

    /// The chunk budget pays 1 per row + 1 per riding naming fact,
    /// so chunk + grades always fits the core's cap: with budget 3,
    /// [plain, code-6, plain, code-6] splits after the first code-6
    /// row (1+2), and each facts slice follows its own rows.
    #[test]
    fn chunk_plan_counts_every_request_dimension() {
        let rows = [[0u64, 1], [6, 0], [0, 2], [6, 0]];
        let naming = [[4i64, 2, 0, 1, 1], [1, 2, 0, 1, 1]];
        let plan = chunk_plan(&rows, &naming, 3);
        assert_eq!(
            plan,
            vec![(&rows[..2], &naming[..1]), (&rows[2..], &naming[1..])]
        );
        // an empty scan still sends ONE (empty, legal) request
        let empty: (&[[u64; 2]], &[[i64; 5]]) = (&[], &[]);
        assert_eq!(chunk_plan(&[], &[], 3), vec![empty]);
    }
}
