//! Threshold evaluation + console/JSON emission. Score polarity and
//! any future scoring live in the Haskell judgment layer (M4+); here
//! it is plain data-driven comparisons only.

use super::metrics::{FileMetrics, FnMetrics};
use crate::config::Thresholds;
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Level {
    Warn,
    Fail,
}

#[derive(Debug, Serialize, PartialEq)]
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
            let m = |code, value| file_row(code, value, func.start_line, &func.name);
            out.push(m(1, func.lines));
            out.push(m(2, func.params));
            out.push(m(3, func.cyclomatic as usize));
            out.push(m(4, func.cognitive as usize));
            out.push(m(5, func.max_nesting as usize));
            out.push(m(6, usize::from(!func.name_ok)));
        }
    }
    out
}

/// Findings from the CORE's positional levels (ADR-008 P3): level 0
/// rows vanish, 1 = warn, 2 = fail; the displayed limit comes from
/// the echoed grade table by (code, level) — the same effective set
/// the core judged with.
pub fn findings_from(rows: &[Row], levels: &[u8], grades: &[[u64; 3]]) -> Vec<Finding> {
    rows.iter()
        .zip(levels.iter().copied())
        .filter(|&(_, l)| l > 0)
        .map(|(r, l)| Finding {
            file: r.file.clone(),
            line: r.line,
            rule: RULES[r.code as usize],
            level: if l == 2 { Level::Fail } else { Level::Warn },
            value: r.value,
            threshold: grades[r.code as usize][if l == 2 { 2 } else { 1 }] as usize,
            subject: r.subject.clone(),
        })
        .collect()
}

/// Threshold evaluation — since ADR-008 P3 a MIRROR of the core's
/// graded verdict table (CE.Scan.Cost), not an authority: `ce scan`
/// builds its findings from the wire's levels and proves this
/// binding equal per run (the whole-report ensure); the auxiliary
/// surfaces (mcp scan tool, the score family's measurement reuse,
/// report_schema) keep reading it locally.
pub fn evaluate(file: &FileMetrics, t: &Thresholds) -> Vec<Finding> {
    let mut out = Vec::new();
    check_file(file, t, &mut out);
    for f in &file.functions {
        check_fn(&file.path, f, t, &mut out);
    }
    out
}

fn check_file(file: &FileMetrics, t: &Thresholds, out: &mut Vec<Finding>) {
    let mk = |level, value, threshold| Finding {
        file: file.path.clone(),
        line: 1,
        rule: "file-lines",
        level,
        value,
        threshold,
        subject: file.path.clone(),
    };
    // fail 0 = no hard line exists — the published P3 contract
    // (CE.Scan.Cost.gradeTable); the review panel caught this mirror
    // reading 0 as "everything fails" while the core read "no line"
    if t.file_lines_fail > 0 && file.total_lines > t.file_lines_fail {
        out.push(mk(Level::Fail, file.total_lines, t.file_lines_fail));
    } else if file.total_lines > t.file_lines_warn {
        out.push(mk(Level::Warn, file.total_lines, t.file_lines_warn));
    }
}

fn check_fn(path: &str, f: &FnMetrics, t: &Thresholds, out: &mut Vec<Finding>) {
    let mk = |rule, level, value: usize, threshold| Finding {
        file: path.to_string(),
        line: f.start_line,
        rule,
        level,
        value,
        threshold,
        subject: f.name.clone(),
    };
    // fail 0 = no hard line (the check_file contract, same owner)
    if t.fn_lines_fail > 0 && f.lines > t.fn_lines_fail {
        out.push(mk("fn-lines", Level::Fail, f.lines, t.fn_lines_fail));
    } else if f.lines > t.fn_lines_warn {
        out.push(mk("fn-lines", Level::Warn, f.lines, t.fn_lines_warn));
    }
    if f.params > t.params_warn {
        out.push(mk("fn-params", Level::Warn, f.params, t.params_warn));
    }
    if f.cyclomatic as usize > t.cyclomatic_warn {
        out.push(mk(
            "cyclomatic",
            Level::Warn,
            f.cyclomatic as usize,
            t.cyclomatic_warn,
        ));
    }
    if f.cognitive as usize > t.cognitive_warn {
        out.push(mk(
            "cognitive",
            Level::Warn,
            f.cognitive as usize,
            t.cognitive_warn,
        ));
    }
    if f.max_nesting as usize > t.nesting_warn {
        out.push(mk(
            "nesting",
            Level::Warn,
            f.max_nesting as usize,
            t.nesting_warn,
        ));
    }
    // Naming is boolean: value 1 = one non-conforming name, limit 0.
    if !f.name_ok {
        out.push(mk("fn-naming", Level::Warn, 1, 0));
    }
}

/// JSON output schema id; bump on any shape change (plan §7.1: schema
/// changes must bump the version — mechanism live since M0).
pub const SCHEMA: &str = "ce.scan-report/0.1.0";

#[derive(Serialize)]
pub struct Report<'a> {
    pub schema: &'static str,
    pub files: &'a [FileMetrics],
    pub findings: &'a [Finding],
    pub summary: Summary,
}

#[derive(Serialize)]
pub struct Summary {
    pub files: usize,
    pub functions: usize,
    pub warns: usize,
    pub fails: usize,
}

pub fn summarize(files: &[FileMetrics], findings: &[Finding]) -> Summary {
    Summary {
        files: files.len(),
        functions: files.iter().map(|f| f.functions.len()).sum(),
        warns: findings.iter().filter(|f| f.level == Level::Warn).count(),
        fails: findings.iter().filter(|f| f.level == Level::Fail).count(),
    }
}

pub fn print_console(findings: &[Finding], summary: &Summary) {
    for f in findings {
        let tag = match f.level {
            Level::Fail => "FAIL",
            Level::Warn => "warn",
        };
        println!(
            "{tag} {}:{} {} = {} (limit {}) [{}]",
            f.file, f.line, f.rule, f.value, f.threshold, f.subject
        );
    }
    println!(
        "scanned {} files / {} functions — {} warn, {} fail",
        summary.files, summary.functions, summary.warns, summary.fails
    );
}
