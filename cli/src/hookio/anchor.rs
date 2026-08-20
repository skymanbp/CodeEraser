//! Where a hook decides the project ROOT is. Split from the envelope
//! and feed plumbing when the anchor test stopped being one line:
//! there are two kinds of git anchor and the second one has a target
//! that has to resolve, which is exactly what the first version of
//! this guard skipped (review 2026-08-19, codex lane).

use std::path::{Path, PathBuf};

/// The project root for a hook envelope's cwd: the NEAREST ancestor
/// (cwd itself first) holding a `ce.toml` or a `.git` — cross-session
/// field report 2026-08-18: the raw cwd made the same edit judge
/// differently depending on where the shell had cd'd, and fragmented
/// the index/daemon per directory. A tree with neither anchor keeps
/// the cwd verbatim (the old behavior as the honest fallback). One
/// throat for all three hooks — the drift was a class, not a site.
pub fn project_root(cwd: &str) -> PathBuf {
    let start = PathBuf::from(cwd);
    let mut probe = start.as_path();
    loop {
        if probe.join("ce.toml").is_file() || is_git_anchor(&probe.join(".git")) {
            return probe.to_path_buf();
        }
        match probe.parent() {
            Some(p) if !p.as_os_str().is_empty() => probe = p,
            _ => return start,
        }
    }
}

/// A REAL git anchor: the `.git` DIRECTORY, or a worktree/submodule
/// gitfile whose target actually resolves.
///
/// Two rounds of this guard, two lessons. `.exists()` took any file of
/// that name, so one Write re-rooted a whole subtree's hooks to a place
/// with no ce.toml, where the guard fell to its unset default and went
/// quiet — and `.git` has no language, so the probe cannot object.
/// Then the shape test (`starts_with("gitdir:")`) was still weaker
/// than git itself: `gitdir: /nonexistent/garbage` passed here while
/// `git rev-parse` answers "fatal: not a git repository". A pointer to
/// nothing is not an anchor.
fn is_git_anchor(p: &Path) -> bool {
    if p.is_dir() {
        return true;
    }
    let Ok(text) = std::fs::read_to_string(p) else {
        return false;
    };
    let Some(target) = text.trim_end().strip_prefix("gitdir:") else {
        return false;
    };
    let target = Path::new(target.trim());
    // the gitfile's path is relative to the FILE's directory when it
    // is not absolute (git's own rule for linked worktrees)
    let resolved = match (target.is_absolute(), p.parent()) {
        (true, _) => target.to_path_buf(),
        (false, Some(dir)) => dir.join(target),
        (false, None) => return false,
    };
    resolved.is_dir()
}

#[cfg(test)]
mod tests {
    use super::*;

    // the crate's ONE tmp-dir scaffold — writing a second copy here
    // is exactly what testutil exists to prevent, and the ratchet
    // caught it before this comment did
    use crate::testutil::scratch;

    /// The hook root ascends to the nearest anchor (ce.toml or
    /// .git), cwd itself first; an anchorless tree keeps the cwd —
    /// the field-report counterexample was `cd background/` flipping
    /// the same write's verdict.
    #[test]
    fn project_root_ascends_to_the_nearest_anchor() {
        let dir = scratch("root");
        let deep = dir.join("repo/sub/deep");
        std::fs::create_dir_all(&deep).expect("mkdir");
        std::fs::write(dir.join("repo/ce.toml"), "\n").expect("anchor");
        let cwd = deep.to_string_lossy().to_string();
        assert_eq!(project_root(&cwd), dir.join("repo"), "ascends to ce.toml");
        assert_eq!(
            project_root(&dir.join("repo").to_string_lossy()),
            dir.join("repo"),
            "cwd itself first"
        );
        let loose = dir.join("loose");
        std::fs::create_dir_all(&loose).expect("mkdir");
        let lc = loose.to_string_lossy().to_string();
        // the walk above `loose` may cross REAL anchors on the host
        // (temp dirs live under a user profile) — assert the honest
        // property instead: the answer is `loose` itself or one of
        // its ancestors carrying a real anchor, never a sibling
        let got = project_root(&lc);
        assert!(loose.starts_with(&got), "never leaves the ancestry line");
        std::fs::remove_dir_all(&dir).ok();
    }

    /// A `.git` FILE anchors only when its pointer resolves — git's
    /// own bar. A plain file and a dangling pointer are both writable
    /// in one Write, and neither may re-root a hook.
    #[test]
    fn a_gitfile_anchors_only_when_its_target_resolves() {
        let dir = scratch("anchor");
        let sub = dir.join("sub");
        std::fs::create_dir_all(&sub).expect("mkdir");
        let gitfile = sub.join(".git");

        std::fs::write(&gitfile, "not a gitdir pointer").expect("plain");
        assert!(!is_git_anchor(&gitfile), "a plain file is not an anchor");

        std::fs::write(&gitfile, "gitdir: ../nowhere\n").expect("dangling");
        assert!(!is_git_anchor(&gitfile), "a pointer to nothing is not one");

        let real = dir.join("realgit");
        std::fs::create_dir_all(&real).expect("mkdir");
        std::fs::write(&gitfile, "gitdir: ../realgit\n").expect("relative");
        assert!(is_git_anchor(&gitfile), "a resolving pointer IS an anchor");

        std::fs::write(&gitfile, format!("gitdir: {}\n", real.display())).expect("absolute");
        assert!(is_git_anchor(&gitfile), "absolute targets resolve too");

        std::fs::remove_dir_all(&dir).ok();
    }
}
