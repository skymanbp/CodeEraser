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

#[derive(Debug, Serialize)]
pub struct Finding {
    pub file: String,
    pub line: usize,
    pub rule: &'static str,
    pub level: Level,
    pub value: usize,
    pub threshold: usize,
    pub subject: String,
}

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
    if file.total_lines > t.file_lines_fail {
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
    if f.lines > t.fn_lines_fail {
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
