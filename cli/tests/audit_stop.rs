//! `ce audit --hook` e2e: real git repo, Stop envelope on stdin. Deny
//! blocks on touched duplication; observe logs silently; the
//! stop_hook_active loop guard short-circuits.

mod common;
// The seed-and-assert throat lives in common/audit.rs — the bypass
// battery wants the same one, and two copies of it were a clone.
use common::{
    assert_stop_blocks as assert_blocks, committed_then, git, seed_git_clone_repo as seed_repo,
    stop_envelope as envelope, stop_observe as silent_audit_observe, tmp,
};

#[test]
fn deny_mode_blocks_on_touched_duplication() {
    let dir = tmp("audit-deny");
    seed_repo(&dir, "deny");
    assert_blocks(&dir, "b.rs");
}

/// The probe-bypass shape: a duplicate file created WITHOUT git add
/// (what `bash > new.rs` leaves behind) is invisible to `git diff
/// HEAD` — this pins the ls-files --others merge in gather().
#[test]
fn untracked_duplicate_from_bash_still_blocks() {
    // commit the helper's staged clone away; then a THIRD, unstaged copy
    committed_then(
        "audit-untracked",
        |d| std::fs::write(d.join("c.rs"), common::rust_fn(3)).expect("c.rs"),
        "c.rs",
    );
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

/// The loop guard passes silently on stdout — but SILENT is not
/// ABSENT: the skip leaves a feed line naming itself, because a
/// session whose only Stop lands here used to be indistinguishable
/// from one the hook never ran in, inside the ledger D2-2 counts
/// session coverage from. The zeros are placeholders; `skipped` says
/// so by name.
#[test]
fn loop_guard_and_clean_tree_stay_silent() {
    let dir = tmp("audit-loop");
    seed_repo(&dir, "deny");
    // a prior Stop already blocked (loop guard), then a clean tree
    let out = common::run_hook(&dir, &["audit", "--hook"], &envelope(&dir, true));
    assert!(out.trim().is_empty(), "loop guard passes: {out}");
    let line = common::last_observe(&dir);
    assert_eq!(line["event"], "stop_audit");
    assert_eq!(line["skipped"], "loop_guard");
    assert_eq!(line["degraded"], false, "nothing degraded; nothing ran");
    git(&dir, &["commit", "-qm", "b"]);
    let out2 = common::run_hook(&dir, &["audit", "--hook"], &envelope(&dir, false));
    assert!(out2.trim().is_empty(), "clean tree passes: {out2}");
}
