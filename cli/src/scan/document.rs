//! The scan report as the core lays it out (plan v2.32 step 5;
//! CE.Scan.Document, CE.Scan.Lines): this side sends the measured
//! files and functions as integers, the core's non-zero levels with
//! each row's class, the grade tables it judged with and the held
//! conditions by code, and the paths and function names the core
//! spells in.
//! The findings, the summary, the console lines and the exit bit are
//! the core's; the SARIF face re-spells the bound document's findings.

use super::Settled;
use super::lang::Lang;
use crate::document::{self, Answer, Request};
use crate::sarif::{num, text};
use anyhow::{Context, Result};
use serde_json::Value;

/// The held conditions in the core's canonical order (CE.Scan
/// `conditionNames`, the scan/1 vocabulary wire.rs checks replies
/// against); a condition is sent by its place here.
pub(super) const CONDITIONS: [&str; 3] = ["hard_line", "knobs_digest", "degraded"];

/// The scan document, its lines and its veto for a settled tree, over
/// the link it was judged over when `held` is whole.
pub fn answer(core: &str, s: &Settled, held: document::Held) -> Result<Answer> {
    let mut files = Vec::with_capacity(s.files.len());
    let mut fns = Vec::new();
    for (f, m) in s.files.iter().enumerate() {
        let lang = Lang::ALL
            .iter()
            .position(|l| l.name() == m.lang)
            .with_context(|| format!("scan document: no language code for {:?}", m.lang))?;
        files.push([f, m.total_lines, m.comment_lines, lang]);
        for func in &m.functions {
            fns.push([
                fns.len(),
                f,
                func.start_line,
                func.end_line,
                func.lines,
                func.params,
                usize::from(func.name_ok),
                func.cyclomatic as usize,
                func.cognitive as usize,
                func.max_nesting as usize,
            ]);
        }
    }
    let levels: Vec<[u64; 3]> = s
        .levels
        .iter()
        .enumerate()
        .filter(|&(_, &l)| l > 0)
        .map(|(i, &l)| {
            [
                i as u64,
                u64::from(l),
                s.row_classes.get(i).copied().unwrap_or(0),
            ]
        })
        .collect();
    let failed: Vec<[usize; 1]> = s
        .failed
        .iter()
        .filter_map(|n| CONDITIONS.iter().position(|c| c == n).map(|i| [i]))
        .collect();
    let req = Request::new("scan")
        .range("files", files.len())
        .range("fns", fns.len())
        .range("rows", s.rows.len())
        .rows("files", files)
        .rows("fns", fns)
        .rows("levels", levels)
        .rows("grades", &s.grades)
        .rows("overrides", &s.overrides)
        .rows("failed", failed);
    // each file's path and each function's name, in the order the
    // request counted them
    let paths: Vec<&String> = s.files.iter().map(|f| &f.path).collect();
    let names: Vec<&String> = (s.files.iter())
        .flat_map(|f| &f.functions)
        .map(|f| &f.name)
        .collect();
    document::assemble_over(core, held, req.text("path", paths).text("fn", names))
}

/// One finding of the bound document as a SARIF result (the `--format
/// sarif` face, sarif::projected): ruleIds under `ce.scan/`, grades
/// respelled in SARIF vocabulary (fail carries the gate's exit-code
/// meaning, hence "error"). The message is the console line's English
/// body — SARIF is a machine face, never translated (i18n.rs charter).
pub(super) fn finding(f: &Value) -> Value {
    let (rule, line) = (text(f, "rule"), num(f, "line"));
    let level = if text(f, "level") == "fail" {
        "error"
    } else {
        "warning"
    };
    let message = format!(
        "{rule} = {} (limit {}) [{}]",
        num(f, "value"),
        num(f, "threshold"),
        text(f, "subject")
    );
    let at = crate::sarif::location(text(f, "file"), line, line);
    crate::sarif::result(&format!("ce.scan/{rule}"), level, &message, at, Vec::new())
}
