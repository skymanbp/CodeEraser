//! Fixture seeding — the T2 seed source, the tmp dir anchor, and
//! the git-repo shapes the gate batteries share. Split from mod.rs
//! in the headroom sprint: audit.rs importing these THROUGH the hub
//! made this support tree a module cycle the graph axis itself
//! billed on the self-scan.

use super::gitio::git;
use std::path::{Path, PathBuf};

/// Fresh per-test dir under the cargo target tmpdir (wiped if
/// present). Carries an empty `.git` anchor: hookio::project_root
/// ascends to the nearest ce.toml/.git, and an anchorless fixture
/// under target/tmp would ascend into the REAL repo (three guard
/// batteries did exactly that when the anchoring landed). Real hook
/// cwds are never anchorless voids; the walker skips hidden dirs,
/// so scans see nothing.
pub fn tmp(name: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join(".git")).expect("mkdir");
    dir
}

/// ~60 normalized tokens; `seed` renames identifiers and changes
/// literals so pairs of outputs are T2 (not T1) clones clearing t=50.
pub fn rust_fn(seed: u32) -> String {
    format!(
        "fn work_{seed}(input_{seed}: &[i64], limit_{seed}: i64) -> i64 {{
    let mut total_{seed} = {seed};
    for value_{seed} in input_{seed} {{
        if *value_{seed} > limit_{seed} {{
            total_{seed} += value_{seed} * {seed} + 7;
        }} else {{
            total_{seed} -= value_{seed} / 3;
        }}
    }}
    total_{seed}
}}
"
    )
}

/// Write `a.rs` (the T2 seed) plus a ce.toml pinning the guard mode.
pub fn seed_sources(dir: &Path, mode: &str) {
    std::fs::write(dir.join("a.rs"), rust_fn(1)).expect("a.rs");
    std::fs::write(dir.join("ce.toml"), format!("[guard]\nmode = \"{mode}\"\n")).expect("ce.toml");
}

/// a.rs + b.rs forming a T2 clone pair that clears t=50.
pub fn seed_clone_pair(dir: &Path) {
    std::fs::write(dir.join("a.rs"), rust_fn(1)).expect("a.rs");
    std::fs::write(dir.join("b.rs"), rust_fn(2)).expect("b.rs (T2 clone)");
}

/// Stage everything present and commit it — the one commit stanza
/// (the P4 ratchet caught trend_rebuild's seed re-growing this trio
/// of git calls token for token).
pub fn commit_all(dir: &Path, msg: &str) {
    git(dir, &["add", "."]);
    git(dir, &["commit", "-qm", msg]);
}

/// The three-commit history the churn batteries measure, and the one
/// every long-measurement face reuses: a root commit adding a.rs; an
/// edit INSIDE work_1 plus a new b.rs (one rewrite, one append, one
/// co-change); a third touching both — a.rs gaining a top-level tail
/// and b.rs losing its function body, so some window additions stop
/// surviving. Shared because the progress face needs the same three
/// phases to have work to do, and its own copy of this was a
/// 121-token twin of churn.rs's that the dedup ratchet refused.
pub fn seed_churn_history(dir: &Path) {
    git(dir, &["init", "-q"]);
    std::fs::write(dir.join("a.rs"), rust_fn(1)).expect("a.rs");
    commit_all(dir, "seed");
    let edited = rust_fn(1).replace("+ 7;", "+ 8;\n            total_1 += 1;");
    std::fs::write(dir.join("a.rs"), &edited).expect("a.rs edit");
    std::fs::write(dir.join("b.rs"), rust_fn(2)).expect("b.rs");
    commit_all(dir, "edit + new");
    std::fs::write(dir.join("a.rs"), format!("{edited}\n// tail\n")).expect("a.rs tail");
    std::fs::write(dir.join("b.rs"), "// emptied\n").expect("b.rs emptied");
    commit_all(dir, "entangle + churn");
}

/// Git repo with the T2 seed committed and b.rs (the clone) staged
/// but uncommitted — the audit/precommit fixture shape.
pub fn seed_git_clone_repo(dir: &Path, mode: &str) {
    seed_sources(dir, mode);
    git(dir, &["init", "-q"]);
    commit_all(dir, "seed");
    std::fs::write(dir.join("b.rs"), rust_fn(2)).expect("b.rs (uncommitted clone)");
    git(dir, &["add", "b.rs"]); // numstat vs HEAD sees staged new files
}
