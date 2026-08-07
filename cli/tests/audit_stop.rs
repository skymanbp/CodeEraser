//! `ce audit --hook` end-to-end: real git repo, Stop envelope on
//! stdin. Deny mode blocks when the working tree adds duplication
//! touching changed files; observe stays silent but logs; the
//! stop_hook_active loop guard short-circuits.

use std::path::Path;

mod common;
use common::{git, seed_git_clone_repo as seed_repo, tmp};

use common::stop_envelope as envelope;

fn run_audit(dir: &Path, envelope: &str) -> String {
    common::run_hook(dir, &["audit", "--hook"], envelope)
}

fn silent_audit_observe(dir: &Path) -> serde_json::Value {
    common::silent_hook_observe(
        dir,
        &["audit", "--hook"],
        &envelope(dir, false),
        "stop_audit",
    )
}

#[test]
fn deny_mode_blocks_on_touched_duplication() {
    let dir = tmp("audit-deny");
    seed_repo(&dir, "deny");
    let out = run_audit(&dir, &envelope(&dir, false));
    let v: serde_json::Value = serde_json::from_str(out.trim()).expect("block json");
    assert_eq!(v["decision"], "block");
    let reason = v["reason"].as_str().expect("reason");
    assert!(reason.contains("b.rs"), "reason names the clone: {reason}");
}

#[test]
fn observe_mode_is_silent_but_logs() {
    let dir = tmp("audit-observe");
    seed_repo(&dir, "observe");
    let line = silent_audit_observe(&dir);
    assert_eq!(line["degraded"], false);
    assert!(line["dup_blocks"].as_u64().expect("n") >= 1);
    assert!(line["net_loc"].as_i64().expect("net") > 0);
}

/// A9f: a broken index must not brick the stop (fail-open) and must
/// not be conflated with "no duplicates" — the observe entry stamps
/// degraded, and deny mode does NOT block on unverifiable state.
#[test]
fn broken_index_degrades_visibly_not_silently() {
    let dir = tmp("audit-degraded");
    seed_repo(&dir, "deny");
    common::corrupt_index(&dir);
    let line = silent_audit_observe(&dir);
    assert_eq!(line["degraded"], true, "degradation stamped, not silent");
    assert_eq!(line["dup_blocks"], 0);
}

#[test]
fn loop_guard_and_clean_tree_stay_silent() {
    let dir = tmp("audit-loop");
    seed_repo(&dir, "deny");
    // stop_hook_active: a prior Stop block already fired — pass through
    let out = run_audit(&dir, &envelope(&dir, true));
    assert!(out.trim().is_empty(), "loop guard passes: {out}");
    // clean tree: commit everything, audit stays silent even in deny
    git(&dir, &["commit", "-qm", "b"]);
    let out2 = run_audit(&dir, &envelope(&dir, false));
    assert!(out2.trim().is_empty(), "clean tree passes: {out2}");
}
