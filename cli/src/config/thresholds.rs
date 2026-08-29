//! The `[thresholds]` table (plan §4.1): the size / complexity
//! ladder every reader judges against, split from config.rs at the
//! 300-line dogfood wall when the canonical digest arrived (O39).

use serde::{Deserialize, Serialize};

/// Thresholds; defaults from DEVELOPMENT_PLAN.md §4.1 (provenance:
/// ESLint max-lines=300, Sonar S104=750/S138=75, ESLint fn=50,
/// Pylint max-args=5, Sonar S3776 CoC=15, lizard CC=15).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Thresholds {
    pub file_lines_warn: usize,
    pub file_lines_fail: usize,
    pub fn_lines_warn: usize,
    pub fn_lines_fail: usize,
    pub params_warn: usize,
    pub cyclomatic_warn: usize,
    pub cognitive_warn: usize,
    pub nesting_warn: usize,
}

impl Default for Thresholds {
    fn default() -> Self {
        Self {
            file_lines_warn: 300,
            file_lines_fail: 750,
            fn_lines_warn: 50,
            fn_lines_fail: 75,
            params_warn: 5,
            cyclomatic_warn: 15,
            cognitive_warn: 15,
            nesting_warn: 4,
        }
    }
}

impl Thresholds {
    /// The ladder must climb, or the warn arm is unreachable. ONE
    /// predicate for the two readers of these keys: scan/wire.rs
    /// refused `fail < warn` and the report.rs mirror judged on
    /// silently, so `ce scan` exited 2 on a ce.toml the MCP scan tool
    /// served a full report from. `fail == 0` is the published "no
    /// hard line" (CE.Scan.Cost.gradeTable), never a low line.
    pub fn ladder_fault(&self) -> Option<String> {
        [
            (
                self.file_lines_warn,
                self.file_lines_fail,
                "file_lines_warn/file_lines_fail",
            ),
            (
                self.fn_lines_warn,
                self.fn_lines_fail,
                "fn_lines_warn/fn_lines_fail",
            ),
        ]
        .into_iter()
        .find(|&(warn, fail, _)| fail != 0 && fail < warn)
        .map(|(warn, fail, keys)| {
            format!(
                "ce.toml [thresholds] {keys}: the fail line {fail} sits below the warn line {warn}"
            )
        })
    }
}
