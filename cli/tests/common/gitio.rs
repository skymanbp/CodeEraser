//! Test-side git spawners, split from common/mod.rs at the repo's
//! own 300-line dogfood gate when git_out became the shared
//! spawn-and-assert throat (the ratchet had caught its third copy).

use std::path::Path;
use std::process::Command;

/// Run git in `dir` with a throwaway identity; panic on failure.
pub fn git(dir: &Path, args: &[&str]) {
    let out = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(["-c", "user.email=t@t", "-c", "user.name=t"])
        .args(args)
        .output()
        .expect("git");
    assert!(out.status.success(), "git {args:?}: {out:?}");
}

/// git in `dir`, stdout bytes back, success asserted — the ONE
/// spawn-and-assert reader (fpr_replay grew the second copy, the
/// churn-ledger gate started a third; the ratchet called it).
pub fn git_out(dir: &Path, args: &[&str]) -> Vec<u8> {
    let out = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .output()
        .expect("git");
    assert!(out.status.success(), "git {args:?} failed: {out:?}");
    out.stdout
}
