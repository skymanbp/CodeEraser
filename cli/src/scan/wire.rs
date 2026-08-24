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

use crate::config::{RulesCfg, Thresholds};
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

/// The rulepack's grade overrides [classId, code, warn, fail] (P3,
/// 3.2.0): one row per (class, code) the class declares a line for —
/// code 0 file-lines, 1 fn-lines, 4 cognitive — carrying the class's
/// EFFECTIVE pair (a declared warn beside an inherited fail sends
/// both, so the wire never has to know which half was written).
/// (class, code)-ascending by construction; empty = no class
/// overrides a scan line.
pub fn class_grade_rows(rules: &RulesCfg, global: &Thresholds) -> Vec<[u64; 4]> {
    let mut out = Vec::new();
    for (i, c) in rules.class.iter().enumerate() {
        let (k, t) = (&c.knobs, c.effective(global));
        let declared = [
            (
                0,
                k.file_lines_warn.or(k.file_lines_fail).is_some(),
                t.file_lines_warn,
                t.file_lines_fail,
            ),
            (
                1,
                k.fn_lines_warn.or(k.fn_lines_fail).is_some(),
                t.fn_lines_warn,
                t.fn_lines_fail,
            ),
            (4, k.cognitive_warn.is_some(), t.cognitive_warn, 0),
        ];
        for (code, rides, warn, fail) in declared {
            if rides {
                out.push([i as u64 + 1, code, warn as u64, fail as u64]);
            }
        }
    }
    out
}

/// One scan judgment's tables: the measurement rows, the global grade
/// table, the naming facts aligned to the code-6 rows (2.30.0), and
/// the rulepack channel (3.2.0) — a class per row, riding only on a
/// classed run, beside the per-class overrides. A record rather than
/// six parameters: the fn-params line is this repo's own.
pub struct ScanRequest<'a> {
    pub rows: &'a [[u64; 2]],
    pub grades: &'a [[u64; 3]],
    pub naming: &'a [[i64; 5]],
    pub row_classes: Option<&'a [u64]>,
    pub overrides: &'a [[u64; 4]],
}

/// Chunked scan judging over ONE link (review C5: the single-request
/// form errored out entirely past the row cap — rows grade
/// independently, so chunking is trivially sound): levels come back
/// positionally per chunk and concatenate, the fail bit ORs, the
/// echoed grade table (and override table, when it rode) must be the
/// one this side sent every time, and a degraded reply to a
/// chunk-sized request is a cap-mirror drift error, never a judgment.
/// The naming facts and the row classes ride aligned: each chunk
/// carries the facts of ITS code-6 rows and the classes of ITS rows.
pub fn judge(core: &str, r: &ScanRequest) -> Result<(Vec<u8>, bool)> {
    let mut link = crate::lockstep::open_family(core, CAP)?;
    let (mut levels, mut fail) = (Vec::new(), false);
    let reserved = r.grades.len() + r.overrides.len();
    for c in chunk_plan(r.rows, r.naming, SCAN_ROW_CAP - reserved) {
        let mut body = json!({"rows": c.rows, "grades": r.grades, "naming": c.naming});
        if let Some(classes) = r.row_classes {
            body["rowClasses"] = json!(&classes[c.span.clone()]);
        }
        if !r.overrides.is_empty() {
            body["gradeOverrides"] = json!(r.overrides);
        }
        let reply = link.request("scan", body).map_err(anyhow::Error::msg)?;
        ensure!(
            reply["degraded"] == json!(false),
            "core degraded a chunk-sized request ({}) — cap mirror drift (scan/wire.rs vs Scan/Cost.hs)",
            reply["reason"]
        );
        assert_echo(&reply, r)?;
        let chunk_levels: Vec<u8> =
            serde_json::from_value(reply["levels"].clone()).context("levels")?;
        ensure!(
            chunk_levels.len() == c.rows.len(),
            "core sent {} levels for {} rows",
            chunk_levels.len(),
            c.rows.len()
        );
        levels.extend(chunk_levels);
        fail |= reply["fail"].as_bool().context("fail")?;
    }
    Ok((levels, fail))
}

/// Both tables the core judged with must be the ones this side sent
/// — one table, two owners; the override echo is absent exactly when
/// none rode.
fn assert_echo(reply: &serde_json::Value, r: &ScanRequest) -> Result<()> {
    let echoed: Vec<[u64; 3]> =
        serde_json::from_value(reply["grades"].clone()).context("grades")?;
    ensure!(
        echoed == r.grades,
        "core judged with grade table {echoed:?}, ce sent {:?} — one table, two owners",
        r.grades
    );
    let overrides: Vec<[u64; 4]> = match reply.get("gradeOverrides") {
        Some(v) => serde_json::from_value(v.clone()).context("gradeOverrides")?,
        None => Vec::new(),
    };
    ensure!(
        overrides == r.overrides,
        "core judged with gradeOverrides {overrides:?}, ce sent {:?}",
        r.overrides
    );
    Ok(())
}

/// Greedy chunk split whose budget counts EVERY request dimension
/// the core's cap counts (the C15 lesson made structural — the old
/// rows-only `chunks(SCAN_ROW_CAP)` left no room for the grade
/// table, so the first chunk of a cap-sized tree degraded): a row
/// pays 1 (2 with a class column riding — the caller prices that by
/// reserving the override rows and halving nothing: the row class
/// travels with its row), a code-6 row pays 2 (its aligned naming
/// fact travels with it), and the caller reserves the grade and
/// override tables' rows. The walk that prices code-6 rows is the
/// walk that slices the facts — alignment by construction; the
/// chunk's row SPAN is what the caller slices the class column by.
struct Chunk<'a> {
    rows: &'a [[u64; 2]],
    naming: &'a [[i64; 5]],
    span: std::ops::Range<usize>,
}

fn chunk_plan<'a>(rows: &'a [[u64; 2]], naming: &'a [[i64; 5]], budget: usize) -> Vec<Chunk<'a>> {
    let mut out = Vec::new();
    let (mut row0, mut fact0, mut facts, mut weight) = (0usize, 0usize, 0usize, 0usize);
    for (i, row) in rows.iter().enumerate() {
        let w = 1 + usize::from(row[0] == 6);
        if weight + w > budget && weight > 0 {
            out.push(Chunk {
                rows: &rows[row0..i],
                naming: &naming[fact0..fact0 + facts],
                span: row0..i,
            });
            (row0, fact0, facts, weight) = (i, fact0 + facts, 0, 0);
        }
        weight += w;
        facts += usize::from(row[0] == 6);
    }
    out.push(Chunk {
        rows: &rows[row0..],
        naming: &naming[fact0..],
        span: row0..rows.len(),
    });
    out
}

#[cfg(test)]
#[path = "wire_tests.rs"]
mod tests;
