//! The measurement rows the scan wire carries and the findings read
//! back from the core's levels, beside the pinned local mirror
//! (`evaluate`) the whole-report ensure proves equal. The report, its
//! console lines and its SARIF face are laid out from the core's
//! document since plan v2.32 step 5 (scan/document.rs).

use super::metrics::{FileMetrics, FnMetrics};
use crate::config::Thresholds;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level {
    Warn,
    Fail,
}

#[derive(Debug, PartialEq)]
pub struct Finding {
    pub file: String,
    pub line: usize,
    pub rule: &'static str,
    pub level: Level,
    pub value: usize,
    pub threshold: usize,
    pub subject: String,
}

/// Metric codes are frozen positions; index = code (the knobs.rs
/// registry pattern shared with wire::grade_rows and the core's
/// CE.Scan.Cost.gradeTable).
pub const RULES: [&str; 7] = [
    "file-lines",
    "fn-lines",
    "fn-params",
    "cyclomatic",
    "cognitive",
    "nesting",
    "fn-naming",
];

/// One measurement row bound for the wire plus everything the
/// report needs to name it locally — paths and subjects never cross
/// (§5.9.2 index privacy; ADR-008 P3).
pub struct Row {
    pub code: u64,
    pub value: usize,
    pub file: String,
    pub line: usize,
    pub subject: String,
}

/// Metric codes a function answers for — 1..=6, the length of the
/// list below. The row arithmetic every other reader does (which row
/// is this function's cognitive one) is this number's, so it is
/// declared once, here, beside the walk that lays the rows out.
pub const FN_CODES: usize = 6;

/// Each file's row count: its own row plus its functions' — the
/// block a chunk boundary must respect, because a call arc never
/// leaves the file that minted it. Owned by rows_of's neighbour,
/// since rows_of is the row order's author.
pub fn blocks_of(files: &[FileMetrics]) -> Vec<usize> {
    files
        .iter()
        .map(|f| 1 + FN_CODES * f.functions.len())
        .collect()
}

/// Every (subject, metric) measurement in report order — the file
/// row first, then each function's six metric rows: the SAME order
/// and vocabulary evaluate() walks, so the whole-report mirror
/// ensure in scan::run can compare finding lists directly.
pub fn rows_of(files: &[FileMetrics]) -> Vec<Row> {
    let mut out = Vec::new();
    for f in files {
        let file_row = |code, value, line: usize, subject: &str| Row {
            code,
            value,
            file: f.path.clone(),
            line,
            subject: subject.to_string(),
        };
        out.push(file_row(0, f.total_lines, 1, &f.path));
        for func in &f.functions {
            for (i, value) in fn_values(func).into_iter().enumerate() {
                out.push(file_row(i as u64 + 1, value, func.start_line, &func.name));
            }
        }
    }
    out
}

/// A function's six metric values in code order 1..=6 — the ONE list
/// rows_of and evaluate both walk (naming is boolean: 1 = one
/// non-conforming name). The registry was spelled three ways in this
/// file until the v2.18 survey folded it.
fn fn_values(f: &FnMetrics) -> [usize; FN_CODES] {
    [
        f.lines,
        f.params,
        f.cyclomatic as usize,
        f.cognitive as usize,
        f.max_nesting as usize,
        usize::from(!f.name_ok),
    ]
}

/// The seven (warn, fail) ladders by metric code. fail 0 = no hard
/// line exists — the published P3 contract (CE.Scan.Cost.gradeTable);
/// the review panel caught this mirror reading 0 as "everything
/// fails" while the core read "no line". An INCOHERENT ladder (fail <
/// warn) never reaches here: Thresholds::ladder_fault refuses it at
/// Config::load, the throat this mirror and wire.rs::grade_rows both
/// read through — one config must not be refused by one reader and
/// judged by the other. Naming's limit is 0. Cognitive is the one
/// complexity metric with a fail SLOT (plan v2.24, `cognitive_fail`,
/// 0 by default so nothing changes until a repo declares one);
/// params / cyclomatic / nesting stay wall-less by the same evidence
/// argument that keeps cognitive's own default at 0.
fn ladders(t: &Thresholds) -> [(usize, usize); 7] {
    [
        (t.file_lines_warn, t.file_lines_fail),
        (t.fn_lines_warn, t.fn_lines_fail),
        (t.params_warn, 0),
        (t.cyclomatic_warn, 0),
        (t.cognitive_warn, t.cognitive_fail),
        (t.nesting_warn, 0),
        (0, 0),
    ]
}

/// Findings from the CORE's positional levels (ADR-008 P3): level 0
/// rows vanish, 1 = warn, 2 = fail; the displayed limit comes from
/// the echoed grade table by (code, level) — or, for a classed row
/// whose class overrides that code (3.2.0), from the override row —
/// the same effective set the core judged with.
pub fn findings_from(
    rows: &[Row],
    levels: &[u8],
    grades: &[[u64; 3]],
    (row_classes, overrides): (&[u64], &[[u64; 4]]),
) -> Vec<Finding> {
    let line_of = |i: usize, code: u64, l: u8| -> u64 {
        let pick = |warn: u64, fail: u64| if l == 2 { fail } else { warn };
        let class = row_classes.get(i).copied().unwrap_or(0);
        overrides
            .iter()
            .find(|o| o[0] == class && o[1] == code)
            .map_or(
                pick(grades[code as usize][1], grades[code as usize][2]),
                |o| pick(o[2], o[3]),
            )
    };
    rows.iter()
        .zip(levels.iter().copied())
        .enumerate()
        .filter(|&(_, (_, l))| l > 0)
        .map(|(i, (r, l))| Finding {
            file: r.file.clone(),
            line: r.line,
            rule: RULES[r.code as usize],
            level: if l == 2 { Level::Fail } else { Level::Warn },
            value: r.value,
            threshold: line_of(i, r.code, l) as usize,
            subject: r.subject.clone(),
        })
        .collect()
}

/// Threshold evaluation — since ADR-008 P3 a MIRROR of the core's
/// graded verdict table (CE.Scan.Cost), not an authority: `ce scan`
/// builds its findings from the wire's levels and proves this
/// binding equal per run (the whole-report ensure) on every judged
/// surface — CLI, MCP and GUI alike; the score family's measurement
/// reuse and report_schema keep reading it locally.
pub fn evaluate(file: &FileMetrics, t: &Thresholds) -> Vec<Finding> {
    let ladders = ladders(t);
    let mut out = Vec::new();
    let mut judge = |code: usize, value: usize, line: usize, subject: &str| {
        let (warn, fail) = ladders[code];
        let (level, threshold) = if fail > 0 && value > fail {
            (Level::Fail, fail)
        } else if value > warn {
            (Level::Warn, warn)
        } else {
            return;
        };
        out.push(Finding {
            file: file.path.clone(),
            line,
            rule: RULES[code],
            level,
            value,
            threshold,
            subject: subject.to_string(),
        });
    };
    judge(0, file.total_lines, 1, &file.path);
    for f in &file.functions {
        for (i, value) in fn_values(f).into_iter().enumerate() {
            judge(i + 1, value, f.start_line, &f.name);
        }
    }
    out
}
