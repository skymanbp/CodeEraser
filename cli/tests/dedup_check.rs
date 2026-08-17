//! `ce dedup --check` ratchet e2e (M2 review R12): clone blocks over
//! the ce.toml [dedup] budget fail the run; at/under budget passes;
//! --check without a configured budget is a hard error. Since
//! ADR-008 P2 the comparison is the CORE's verdict, so the gate
//! rides the real core binary like every judgment e2e.

use std::path::Path;
use std::process::Output;

mod common;
use common::{rust_fn, tmp};

fn run_check(dir: &Path) -> Output {
    let core = std::env::var("CE_CORE_BIN").expect(
        "CE_CORE_BIN is unset — build the core and export it:\n  \
         cd core && cabal build all && export CE_CORE_BIN=$(cabal list-bin ce-core)",
    );
    common::run_ce(dir, &["dedup", ".", "--check", "--core", &core])
}

#[test]
fn check_fails_over_budget_and_passes_at_budget() {
    let dir = tmp("dedup-check");
    common::seed_clone_pair(&dir);
    std::fs::write(dir.join("ce.toml"), "[dedup]\nbudget = 0\n").expect("ce.toml");
    let over = run_check(&dir);
    assert!(!over.status.success(), "1 block > budget 0 must fail");
    let err = String::from_utf8_lossy(&over.stderr);
    assert!(err.contains("ratchet"), "names the gate: {err}");
    std::fs::write(dir.join("ce.toml"), "[dedup]\nbudget = 1\n").expect("ce.toml");
    let at = run_check(&dir);
    assert!(at.status.success(), "at budget passes: {at:?}");
}

#[test]
fn check_without_budget_is_an_error() {
    let dir = tmp("dedup-check-nocfg");
    std::fs::write(dir.join("a.rs"), rust_fn(1)).expect("a.rs");
    let out = run_check(&dir);
    assert!(!out.status.success(), "--check without budget must error");
}
