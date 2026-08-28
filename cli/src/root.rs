//! Where a project ROOT is, for every surface that takes one from a
//! human: the CLI's positional `[ROOT]`, the MCP `path` argument, the
//! GUI's root field, and the three hook envelopes.
//!
//! The rule is one line: ascend from the given path to the NEAREST
//! ancestor holding a `ce.toml` or a real `.git` (the path itself
//! first); a tree with neither keeps what was given. That resolved
//! root owns the per-project state — `ce.toml`, `ce-baseline.json`,
//! `.ce/` — while the path the human typed stays the analysis SCOPE.
//!
//! The hooks got this in 2026-08-19 (a raw cwd made the same edit
//! judge differently depending on where the shell had cd'd, and
//! fragmented the index and daemon per directory); the CLI, MCP and
//! GUI did not, so `ce check cli` judged with no baseline at all and
//! planted a second `.ce/` wherever it was pointed. Same class, same
//! throat, now one implementation for all of them.
//!
//! LIBRARY CALLERS ARE NOT ROUTED THROUGH HERE ON PURPOSE. `analyze`,
//! `Config::load` and `baseline::read` take the root their caller
//! already resolved — the daemon is handed one by its client, and a
//! test's scratch directory is its own project by construction. An
//! ascent inside those throats would walk a temp tree under this very
//! repository up to this repository.

use std::path::{Path, PathBuf};

/// The project root for a path a human named. Absolutized first: a
/// relative `cli` has `parent() == Some("")`, which ends the walk
/// before it tests a single ancestor — the documented contract said
/// "nearest ancestor" while the code checked exactly one level.
pub fn project_root(given: &Path) -> PathBuf {
    let start = std::path::absolute(given).unwrap_or_else(|_| given.to_path_buf());
    let mut probe = start.as_path();
    loop {
        if probe.join("ce.toml").is_file() {
            return probe.to_path_buf();
        }
        if is_git_anchor(&probe.join(".git")) {
            return match superproject_of(probe) {
                Some(sup) => project_root(&sup),
                None => probe.to_path_buf(),
            };
        }
        match probe.parent() {
            Some(p) if !p.as_os_str().is_empty() => probe = p,
            _ => return given.to_path_buf(),
        }
    }
}

/// The enclosing anchor whose `.gitmodules` DECLARES `sub`, when one
/// does. A declared submodule is part of its superproject by
/// declaration — the same tracked answer the mention walk keeps
/// (gitmodules.rs) — so its state throats resolve there, seated or
/// not; an UNDECLARED nested repository (a vendored checkout, a
/// `.git`-anchored fixture under target/tmp) keeps the 2026-08-21
/// escape and roots at itself.
fn superproject_of(sub: &Path) -> Option<PathBuf> {
    let sup = sub
        .ancestors()
        .skip(1)
        .find(|p| !p.as_os_str().is_empty() && is_anchored(p))?;
    let rel = crate::scan::walk::rel_str(sup, sub);
    crate::gitmodules::declared(sup)
        .contains(&rel)
        .then(|| sup.to_path_buf())
}

/// The resolved root plus the ascent, for surfaces that must SAY when
/// they moved. A silent re-root is the other half of the same defect:
/// the operator has to be able to see which project answered.
pub fn resolve(given: &Path) -> (PathBuf, bool) {
    let root = project_root(given);
    let same = std::path::absolute(given)
        .map(|g| g == root)
        .unwrap_or(true);
    (root, !same)
}

/// Whether `p` itself carries an anchor — the discriminator between
/// "this directory is a project" and "project_root fell back to what
/// it was given". The guard uses it to stay INERT in anchorless
/// territory instead of planting state at an arbitrary cwd.
pub fn is_anchored(p: &Path) -> bool {
    p.join("ce.toml").is_file() || is_git_anchor(&p.join(".git"))
}

/// A REAL git anchor: the `.git` DIRECTORY, or a worktree/submodule
/// gitfile whose target actually resolves.
///
/// PRECEDENCE, decided (audit 2026-08-21, refined 2026-08-28): the
/// NEAREST anchor wins, even when a nested `.git` sits under an
/// enclosing ce.toml — git's own stop-at-first-gitdir rule — with ONE
/// refinement: a `.git`-only anchor the enclosing project's
/// `.gitmodules` DECLARES is that project's (`superproject_of`; the
/// test suite rides at `cli/tests` that way, and a guard rooted there
/// once minted a tests-only index under default knobs), while a
/// nested ce.toml still opts a submodule out. A vendored checkout or
/// UNDECLARED nested repository is its own project and escapes the
/// outer guard policy BY THIS RULE; the outer project's Stop audit
/// still sees its own tree, and eject leaves an own-project alone.
/// Continuing the ascent past EVERY `.git` was rejected because each
/// `.git`-anchored test fixture under target/tmp would then re-root
/// to this repository itself.
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

    /// A scratch tree whose `repo/` carries the anchor, returning
    /// (scratch, anchored repo, a deep descendant). Both ascent tests
    /// need exactly this and the dedup gate refused the second copy.
    fn anchored(tag: &str, depth: &str) -> (PathBuf, PathBuf, PathBuf) {
        let dir = scratch(tag);
        let repo = dir.join("repo");
        let deep = repo.join(depth);
        std::fs::create_dir_all(&deep).expect("mkdir");
        std::fs::write(repo.join("ce.toml"), "\n").expect("anchor");
        (dir, repo, deep)
    }

    /// The root ascends to the nearest anchor (ce.toml or .git), the
    /// path itself first; an anchorless tree keeps what it was given —
    /// the field-report counterexample was `cd background/` flipping
    /// the same write's verdict.
    #[test]
    fn project_root_ascends_to_the_nearest_anchor() {
        let (dir, repo, deep) = anchored("root", "sub/deep");
        assert_eq!(project_root(&deep), repo, "ascends to ce.toml");
        assert_eq!(project_root(&repo), repo, "the path itself first");
        let (root, ascended) = resolve(&deep);
        assert_eq!(root, repo, "resolve agrees with project_root");
        assert!(ascended, "an ascent reports itself");
        assert!(!resolve(&repo).1, "no ascent, nothing to say");

        let loose = dir.join("loose");
        std::fs::create_dir_all(&loose).expect("mkdir");
        // the walk above `loose` may cross REAL anchors on the host
        // (temp dirs live under a user profile) — assert the honest
        // property instead: the answer is `loose` itself or one of
        // its ancestors carrying a real anchor, never a sibling
        let got = project_root(&loose);
        assert!(loose.starts_with(&got), "never leaves the ancestry line");
        std::fs::remove_dir_all(&dir).ok();
    }

    /// A relative path walks its ancestry too. `Path::new("cli")
    /// .parent()` is `Some("")`, which used to end the walk after one
    /// level — latent while only absolute hook cwds arrived here, live
    /// the moment the CLI's `.`-shaped positionals came through.
    #[test]
    fn a_relative_path_still_ascends() {
        let (dir, repo, deep) = anchored("relroot", "sub");
        let keep = std::env::current_dir().expect("cwd");
        std::env::set_current_dir(&deep).expect("cd");
        let got = project_root(Path::new("."));
        std::env::set_current_dir(keep).expect("cd back");
        // canonicalize both sides: the temp dir may itself be reached
        // through a symlink (macOS /var -> /private/var)
        let want = std::fs::canonicalize(&repo).expect("canon");
        assert_eq!(std::fs::canonicalize(&got).expect("canon"), want);
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

    /// A declared submodule roots at its superproject, seated or not
    /// (the same answer before and after `git submodule update
    /// --init`); an undeclared nested repository and a submodule
    /// carrying its own ce.toml still root at themselves.
    #[test]
    fn a_declared_submodule_roots_at_its_superproject() {
        let (dir, repo, deep) = anchored("submod", "sub/deep");
        std::fs::create_dir_all(repo.join("realgit")).expect("mkdir");
        let declare = "[submodule \"s\"]\n\tpath = sub\n[submodule \"t\"]\n\tpath = sub2\n";
        std::fs::write(repo.join(".gitmodules"), declare).expect(".gitmodules");
        for sub in ["sub", "foreign", "sub2"] {
            std::fs::create_dir_all(repo.join(sub)).expect("mkdir");
            std::fs::write(repo.join(sub).join(".git"), "gitdir: ../realgit\n").expect("gitfile");
        }
        std::fs::write(repo.join("sub2/ce.toml"), "\n").expect("opt-out");
        assert_eq!(
            project_root(&repo.join("sub")),
            repo,
            "declared: the parent's"
        );
        assert_eq!(project_root(&deep), repo, "…all the way down");
        assert_eq!(
            project_root(&repo.join("foreign")),
            repo.join("foreign"),
            "undeclared escapes"
        );
        assert_eq!(
            project_root(&repo.join("sub2")),
            repo.join("sub2"),
            "own ce.toml opts out"
        );
        std::fs::remove_file(repo.join("sub/.git")).expect("deinit");
        assert_eq!(project_root(&deep), repo, "checkout-invariant");
        std::fs::remove_dir_all(&dir).ok();
    }
}
