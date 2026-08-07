//! Scan-report JSON contract: the golden pins the exact serialized
//! shape of `ce scan --json` (plan §7.1 — any shape change must bump
//! report::SCHEMA and deliberately regenerate the golden with
//! `CE_BLESS=1 cargo test --test report_schema`).

use codeeraser::config::Thresholds;
use codeeraser::scan::metrics::{FileMetrics, FnMetrics};
use codeeraser::scan::report::{self, Report};
use std::path::PathBuf;

fn golden_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("cli/ has a parent")
        .join("contracts/fixtures/scan-report/report.golden.json")
}

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
fn report_json_matches_golden() {
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
    let path = golden_path();
    if std::env::var("CE_BLESS").is_ok() {
        std::fs::write(&path, format!("{json}\n")).expect("bless golden");
        return;
    }
    let golden = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| {
            panic!(
                "missing golden {} ({e}); CE_BLESS=1 to create",
                path.display()
            )
        })
        .replace("\r\n", "\n");
    assert_eq!(
        json.trim_end(),
        golden.trim_end(),
        "report shape drifted — bump report::SCHEMA and re-bless deliberately"
    );
}
