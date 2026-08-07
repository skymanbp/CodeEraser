//! Report JSON contracts: goldens pin the exact serialized shape of
//! `ce scan --json` and `ce dedup --format json` (plan §7.1 / M2
//! review R10 — any shape change must bump the schema id and
//! deliberately regenerate with
//! `CE_BLESS=1 cargo test --test report_schema`).

use codeeraser::config::Thresholds;
use codeeraser::scan::metrics::{FileMetrics, FnMetrics};
use codeeraser::scan::report::{self, Report};
use std::path::{Path, PathBuf};

mod common;

fn golden_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("cli/ has a parent")
        .join("contracts/fixtures")
        .join(name)
}

/// CE_BLESS=1 regenerates the golden; otherwise byte-compare
/// (CRLF-normalized). Exactly "1": any-value is_ok() would let
/// CE_BLESS=0 or an empty var silently bless-and-pass
/// (attack-review finding).
fn assert_matches_golden(json: &str, path: &Path) {
    if std::env::var("CE_BLESS").as_deref() == Ok("1") {
        std::fs::create_dir_all(path.parent().expect("golden dir")).expect("mkdir");
        std::fs::write(path, format!("{json}\n")).expect("bless golden");
        return;
    }
    let golden = std::fs::read_to_string(path)
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
        "report shape drifted — bump the schema id and re-bless deliberately"
    );
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
/// token/distinct counts, and every summary field of the 0.4.x shape.
#[test]
fn dedup_report_json_matches_golden() {
    let dir = common::tmp("dedup-golden");
    common::seed_clone_pair(&dir);
    let (found, summary) = codeeraser::dedup::analyze(&dir, None, None, None).expect("analyze");
    let value = codeeraser::dedup::report_json(&found, &summary).expect("report json");
    let json = serde_json::to_string_pretty(&value).expect("serialize");
    assert_matches_golden(&json, &golden_path("dedup-report/report.golden.json"));
}
