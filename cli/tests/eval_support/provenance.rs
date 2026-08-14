//! Ancestry-gate primitives (G13/T-G13): "sampled before audited
//! before scored" as CHECKED git facts. ONE binding for the graph
//! and dedup provenance gates — these are deliberately the only
//! gates that run git, and a shallow clone refuses loudly instead of
//! passing vacuously (CI checks out fetch-depth: 0 for exactly this).

use super::git_in;

pub fn require_full_history() {
    let shallow = git_in(Some(".."), &["rev-parse", "--is-shallow-repository"]);
    assert_eq!(
        shallow.trim(),
        "false",
        "shallow clone: ancestry is uncheckable — fetch-depth: 0 (see ci.yml)"
    );
}

/// First commit that introduced `path` (repo-relative).
pub fn intro_commit(path: &str) -> String {
    let log = git_in(Some(".."), &["log", "--reverse", "--format=%H", "--", path]);
    log.lines()
        .next()
        .unwrap_or_else(|| panic!("{path}: never committed"))
        .to_string()
}

/// merge-base --is-ancestor: the exit code IS the answer, so this one
/// call bypasses git_in's success assertion on purpose.
pub fn is_ancestor(ancestor: &str, descendant: &str) -> bool {
    std::process::Command::new("git")
        .args([
            "-C",
            "..",
            "merge-base",
            "--is-ancestor",
            ancestor,
            descendant,
        ])
        .status()
        .expect("git")
        .success()
}

/// --is-ancestor is REFLEXIVE (review F6): two artifacts frozen in
/// one commit would pass the plain form and defeat the very ordering
/// the gate certifies — strictness is the claim, so assert it.
pub fn is_strict_ancestor(ancestor: &str, descendant: &str) -> bool {
    ancestor != descendant && is_ancestor(ancestor, descendant)
}

/// Every file currently under `subtree` must have been introduced
/// STRICTLY after `anchor` — the full-scan blind-window leg: nothing
/// judge-shaped may predate the frozen draw, wherever it landed.
pub fn assert_subtree_postdates(anchor: &str, subtree: &str) {
    let tree = git_in(Some(".."), &["ls-files", "--", subtree]);
    let mut any = false;
    for path in tree.lines().filter(|p| !p.is_empty()) {
        any = true;
        assert!(
            is_strict_ancestor(anchor, &intro_commit(path)),
            "{path}: landed at or before the anchor freeze (G13)"
        );
    }
    assert!(any, "{subtree}: empty subtree makes the gate vacuous");
}
