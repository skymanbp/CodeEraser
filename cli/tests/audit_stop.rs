//! `ce audit --hook` end-to-end: real git repo, Stop envelope on
//! stdin. Deny mode blocks when the working tree adds duplication
//! touching changed files; observe stays silent but logs; the
//! stop_hook_active loop guard short-circuits.

use std::path::Path;

mod common;
use common::{git, rust_fn, tmp};

/// Repo with a.rs committed; working tree adds b.rs = T2 clone of a.rs.
fn seed_repo(dir: &Path, mode: &str) {
    common::seed_sources(dir, mode);
    git(dir, &["init", "-q"]);
    git(dir, &["add", "."]);
    git(dir, &["commit", "-qm", "seed"]);
    std::fs::write(dir.join("b.rs"), rust_fn(2)).expect("b.rs (uncommitted clone)");
    git(dir, &["add", "b.rs"]); // numstat vs HEAD sees staged new files
}

fn run_audit(dir: &Path, envelope: &str) -> String {
    common::run_hook(dir, &["audit", "--hook"], envelope)
}

fn envelope(dir: &Path, stop_hook_active: bool) -> String {
    serde_json::json!({
        "session_id": "t", "transcript_path": "t",
        "cwd": dir.display().to_string().replace('\\', "/"),
        "hook_event_name": "Stop", "stop_hook_active": stop_hook_active
    })
    .to_string()
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
    let out = run_audit(&dir, &envelope(&dir, false));
    assert!(out.trim().is_empty(), "observe emits nothing: {out}");
    let line = common::last_observe(&dir);
    assert_eq!(line["event"], "stop_audit");
    assert!(line["dup_blocks"].as_u64().expect("n") >= 1);
    assert!(line["net_loc"].as_i64().expect("net") > 0);
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
