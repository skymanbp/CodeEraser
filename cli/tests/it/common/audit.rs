//! Stop-audit fixtures: the two repo shapes and the two assertions
//! every audit battery needs. Split out of common/mod.rs (which sits
//! at its own 300-line gate) when the bypass battery joined
//! audit_stop.rs and both wanted the same seed-and-assert throat.
#![allow(dead_code)]

use super::fixtures::{commit_all, rust_fn, seed_git_clone_repo, seed_sources, tmp};
use super::gitio::git;
use std::path::{Path, PathBuf};

/// Audit once, expect a Stop block whose reason names `needle`.
///
/// An empty stdout is diagnosed, not merely reported. The audit fails
/// OPEN by charter, so an unusable ce-core produces silence — and the
/// bare `expect("block json")` then said `EOF while parsing a value`,
/// which reads as a broken assertion rather than a missing core. That
/// misreport cost two sessions chasing a phantom flake: the runs that
/// were green had CE_CORE_BIN exported, the runs that were red found
/// a stale ce-core on PATH whose proto no longer matched. The feed
/// the audit just wrote carries the answer, so it is read here.
pub fn assert_stop_blocks(dir: &Path, needle: &str) {
    let out = super::run_hook(dir, &["audit", "--hook"], &super::stop_envelope(dir, false));
    let v: serde_json::Value = serde_json::from_str(out.trim()).unwrap_or_else(|e| {
        panic!(
            "no Stop block to parse ({e}). The audit {}. \
             Raw stdout: {out:?}",
            match degraded_bit(dir) {
                Some(true) =>
                    "DEGRADED — its ce-core was unreachable or \
                               spoke a different proto; export CE_CORE_BIN \
                               at the core you just built",
                Some(false) => "measured and chose not to block",
                None => "left no observe entry at all",
            }
        )
    });
    assert_eq!(v["decision"], "block");
    let reason = v["reason"].as_str().expect("reason");
    assert!(reason.contains(needle), "reason names {needle}: {reason}");
}

/// The `degraded` bit of the last stop_audit line in the project's
/// observe feed — the audit's own record of whether it could judge.
fn degraded_bit(dir: &Path) -> Option<bool> {
    let log = std::fs::read_to_string(dir.join(".ce/observe.ndjson")).ok()?;
    log.lines()
        .filter_map(|l| serde_json::from_str::<serde_json::Value>(l).ok())
        .rfind(|j| j["event"] == "stop_audit")?["degraded"]
        .as_bool()
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

/// A subdirectory carrying a PLAIN FILE named `.git` next to a clone
/// of the repo's seed — the anchor-hijack shape, which a broken
/// submodule checkout leaves behind without anyone meaning to.
pub fn fake_anchor_subdir(dir: &Path, name: &str) -> PathBuf {
    let sub = dir.join(name);
    std::fs::create_dir_all(&sub).expect("mkdir");
    for (file, body) in [
        (".git", String::from("not a gitdir pointer")),
        ("c.rs", rust_fn(3)),
    ] {
        std::fs::write(sub.join(file), body).expect("fixture file");
    }
    sub
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
