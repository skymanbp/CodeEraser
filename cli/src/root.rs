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

/// The root that judges a WRITE at `target` from a session rooted at
/// `session` (plan v2.18 step #12). An own path is the session's. A
/// path a nested project owns — a declared submodule (a reader of
/// this tree, measured by nobody here) or an undeclared nested
/// repository (nobody's here) — is that project's, and only a project
/// that opted in with a `ce.toml` of its own has a gate to judge
/// with: the nearest one above the target answers, its own config,
/// index and baseline; None = no gate there, the hook stays inert. A
/// target the session root does not contain keeps the session's root
/// (the walk's Scope then says it is not ours, as before).
pub fn judging_root(session: &Path, target: &Path) -> Option<PathBuf> {
    let full = std::path::absolute(target).unwrap_or_else(|_| target.to_path_buf());
    let Ok(rel) = full.strip_prefix(session) else {
        return Some(session.to_path_buf());
    };
    let rel = rel.to_string_lossy().replace('\\', "/");
    match crate::gitmodules::owner(session, &crate::gitmodules::declared(session), &rel) {
        crate::gitmodules::Owner::Own => Some(session.to_path_buf()),
        _ => {
            let own = project_root(full.parent()?);
            (own != session && own.starts_with(session) && own.join("ce.toml").is_file())
                .then_some(own)
        }
    }
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
pub(crate) fn is_git_anchor(p: &Path) -> bool {
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
#[path = "../tests/unit/root.rs"]
mod tests;
