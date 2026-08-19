//! Stop-audit fixtures: the two repo shapes and the two assertions
//! every audit battery needs. Split out of common/mod.rs (which sits
//! at its own 300-line gate) when the bypass battery joined
//! audit_stop.rs and both wanted the same seed-and-assert throat.
#![allow(dead_code)]

use super::{commit_all, git, rust_fn, seed_git_clone_repo, seed_sources, tmp};
use std::path::{Path, PathBuf};

/// Audit once, expect a Stop block whose reason names `needle`.
pub fn assert_stop_blocks(dir: &Path, needle: &str) {
    let out = super::run_hook(dir, &["audit", "--hook"], &super::stop_envelope(dir, false));
    let v: serde_json::Value = serde_json::from_str(out.trim()).expect("block json");
    assert_eq!(v["decision"], "block");
    let reason = v["reason"].as_str().expect("reason");
    assert!(reason.contains(needle), "reason names {needle}: {reason}");
}

/// The observe entry a SILENT Stop audit leaves behind.
pub fn stop_observe(dir: &Path) -> serde_json::Value {
    super::silent_hook_observe(
        dir,
        &["audit", "--hook"],
        &super::stop_envelope(dir, false),
        "stop_audit",
    )
}

/// Seed a deny-mode repo with the clone pair COMMITTED, apply one
/// mutation, and assert the Stop blocks naming it. Every changed-file
/// spelling case is this shape with a different mutation, so the
/// skeleton lives here and each test is its one distinguishing line.
pub fn committed_then(name: &str, mutate: impl FnOnce(&Path), needle: &str) {
    let dir = tmp(name);
    seed_git_clone_repo(&dir, "deny");
    git(&dir, &["commit", "-qm", "clean"]);
    mutate(&dir);
    assert_stop_blocks(&dir, needle);
}

/// A deny-mode repo whose CE anchor (ce.toml) sits `pkg_rel` below
/// the git anchor, with the clone STAGED rather than committed;
/// `commit_seed = false` leaves HEAD unborn. Returns the CE root —
/// the two axes the diff leg has to survive.
pub fn staged_repo(name: &str, pkg_rel: &str, commit_seed: bool) -> PathBuf {
    let dir = tmp(name);
    let pkg = dir.join(pkg_rel);
    std::fs::create_dir_all(&pkg).expect("mkdir");
    git(&dir, &["init", "-q"]);
    seed_sources(&pkg, "deny");
    if commit_seed {
        commit_all(&dir, "seed");
    }
    std::fs::write(pkg.join("b.rs"), rust_fn(2)).expect("b.rs");
    git(&dir, &["add", "."]);
    pkg
}
