//! Report JSON contracts: goldens pin the exact serialized shape of
//! `ce scan --json` and `ce dedup --format json` (plan §7.1 / M2
//! review R10 — any shape change must bump the schema id and
//! deliberately regenerate with
//! `CE_BLESS=1 cargo test --test report_schema`).

use codeeraser::config::Thresholds;
use codeeraser::scan::metrics::{FileMetrics, FnMetrics};
use codeeraser::scan::report::{self, Report};

mod common;
use common::{assert_matches_golden, golden_path};

/// One file, one function that trips fn-naming — exercises every
/// Report field including a Finding, with fixed input values.
fn sample() -> Vec<FileMetrics> {
    vec![FileMetrics {
        path: "src/sample.py".into(),
        lang: "python",
        total_lines: 12,
        comment_lines: 2,
        functions: vec![FnMetrics {
            name: "loadConfig".into(),
            start_line: 3,
            end_line: 9,
            lines: 7,
            params: 2,
            cyclomatic: 2,
            cognitive: 1,
            max_nesting: 1,
            name_ok: false,
        }],
    }]
}

#[test]
fn scan_report_json_matches_golden() {
    let files = sample();
    let findings: Vec<_> = files
        .iter()
        .flat_map(|f| report::evaluate(f, &Thresholds::default()))
        .collect();
    let summary = report::summarize(&files, &findings);
    let rep = Report {
        schema: report::SCHEMA,
        files: &files,
        findings: &findings,
        summary,
    };
    let json = serde_json::to_string_pretty(&rep).expect("serialize");
    assert_matches_golden(&json, &golden_path("scan-report/report.golden.json"));
}

/// Real pipeline on a deterministic seed pair: pins block spans,
/// token/distinct counts, groups, and every summary field of the
/// 0.5.x shape.
#[test]
fn dedup_report_json_matches_golden() {
    let dir = common::tmp("dedup-golden");
    common::seed_clone_pair(&dir);
    let (found, summary) = codeeraser::dedup::analyze(&dir, None, None, None).expect("analyze");
    let value = codeeraser::dedup::report_json(&found, &summary).expect("report json");
    let json = serde_json::to_string_pretty(&value).expect("serialize");
    assert_matches_golden(&json, &golden_path("dedup-report/report.golden.json"));
}
