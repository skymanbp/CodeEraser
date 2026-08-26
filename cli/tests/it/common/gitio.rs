//! Test-side git spawner (throwaway identity, success asserted),
//! split from common/mod.rs at the repo's own 300-line dogfood gate.

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
