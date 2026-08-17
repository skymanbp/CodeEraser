//! Gate e2e batteries through the real binaries — ONE home for the
//! --check/--core gates (review C16 + the eleventh ratchet bite:
//! dedup_check.rs and scan_check.rs re-grew the harness skeleton as
//! sibling files the moment the second gate landed, so the gates
//! live together and walk common::gate_red_green).
//! dedup budget: M2 review R12, comparison in the core since P2;
//! scan grading: ADR-008 P3, levels and exit semantics in the core.

use std::path::Path;
use std::process::Output;

mod common;
use common::{core_bin, gate_red_green, rust_fn, tmp};

fn run_dedup_check(dir: &Path) -> Output {
    common::run_ce(dir, &["dedup", ".", "--check", "--core", &core_bin()])
}

fn run_scan(dir: &Path) -> Output {
    common::run_ce(dir, &["scan", ".", "--core", &core_bin()])
}

#[test]
fn dedup_check_fails_over_budget_and_passes_at_budget() {
    let dir = tmp("gate-dedup");
    common::seed_clone_pair(&dir);
    std::fs::write(dir.join("ce.toml"), "[dedup]\nbudget = 0\n").expect("ce.toml");
    gate_red_green(&dir, &run_dedup_check, "ratchet", true, &|| {
        std::fs::write(dir.join("ce.toml"), "[dedup]\nbudget = 1\n").expect("ce.toml");
    });
}

#[test]
fn dedup_check_without_budget_is_an_error() {
    let dir = tmp("gate-dedup-nocfg");
    std::fs::write(dir.join("a.rs"), rust_fn(1)).expect("a.rs");
    let out = run_dedup_check(&dir);
    assert!(!out.status.success(), "--check without budget must error");
}

#[test]
fn scan_fails_past_the_hard_line_and_passes_clean() {
    let dir = tmp("gate-scan");
    std::fs::write(dir.join("a.rs"), "// filler\n".repeat(800)).expect("a.rs");
    gate_red_green(&dir, &run_scan, "file-lines", false, &|| {
        std::fs::write(dir.join("a.rs"), "fn tiny() -> i64 { 1 }\n").expect("shrink");
    });
}
