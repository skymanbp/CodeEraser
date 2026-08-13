//! G13 (design §5): "sampled before audited before scored" as a
//! CHECKED git fact, not a convention. This is deliberately the one
//! graph gate that runs git — CI checks out with fetch-depth: 0 for
//! exactly this test, and a shallow clone refuses loudly instead of
//! passing vacuously.

mod eval_support;

use eval_support::git_in;

const CORPORA: [&str; 5] = ["cobra", "requests", "ripgrep", "self", "zod"];

fn require_full_history() {
    let shallow = git_in(Some(".."), &["rev-parse", "--is-shallow-repository"]);
    assert_eq!(
        shallow.trim(),
        "false",
        "shallow clone: ancestry is uncheckable — fetch-depth: 0 (see ci.yml)"
    );
}

/// First commit that introduced `path` (repo-relative).
fn intro_commit(path: &str) -> String {
    let log = git_in(Some(".."), &["log", "--reverse", "--format=%H", "--", path]);
    log.lines()
        .next()
        .unwrap_or_else(|| panic!("{path}: never committed"))
        .to_string()
}

/// merge-base --is-ancestor: the exit code IS the answer, so this one
/// call bypasses git_in's success assertion on purpose.
fn is_ancestor(ancestor: &str, descendant: &str) -> bool {
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

/// --is-ancestor is REFLEXIVE (review F6): sample and audit frozen in
/// one commit would pass the plain form and defeat the very ordering
/// the gate certifies — strictness is the claim, so assert it.
fn is_strict_ancestor(ancestor: &str, descendant: &str) -> bool {
    ancestor != descendant && is_ancestor(ancestor, descendant)
}

fn audit_intros() -> Vec<String> {
    CORPORA
        .iter()
        .map(|c| intro_commit(&format!("cli/tests/eval_graph_review/{c}.json")))
        .collect()
}

/// The sample was drawn blind, then audited: every review table must
/// STRICTLY descend from the commit that froze the sample.
#[test]
fn sample_precedes_audit() {
    require_full_history();
    let sample = intro_commit("contracts/eval/graph-sample-v1.json");
    for audit in audit_intros() {
        assert!(
            is_strict_ancestor(&sample, &audit),
            "audit table does not strictly descend from the sample freeze (G13)"
        );
    }
}

/// Armed tripwire, two layers (review F7: a resolver landing outside
/// the pre-registered ladder/ path must not slip the ordering check).
/// Layer 1: the design-registered resolver home cli/src/graph/ladder
/// must postdate every audit table. Layer 2: EVERY cli/src/graph file
/// in the current tree either predates the sample freeze (detector
/// era) or descends from the audit freeze — nothing graph-shaped may
/// land inside the blind window between sampling and audit.
#[test]
fn audit_precedes_any_resolver() {
    require_full_history();
    let sample = intro_commit("contracts/eval/graph-sample-v1.json");
    let audits = audit_intros();
    let ladder = git_in(
        Some(".."),
        &[
            "log",
            "--reverse",
            "--format=%H",
            "--",
            "cli/src/graph/ladder",
        ],
    );
    if let Some(first_ladder) = ladder.lines().next() {
        for audit in &audits {
            assert!(
                is_strict_ancestor(audit, first_ladder),
                "a ladder file landed before the audit froze (G13)"
            );
        }
    }
    let tree = git_in(Some(".."), &["ls-files", "--", "cli/src/graph"]);
    for path in tree.lines().filter(|p| !p.is_empty()) {
        let intro = intro_commit(path);
        if is_ancestor(&intro, &sample) {
            continue; // detector-era file, in place before the draw
        }
        for audit in &audits {
            assert!(
                is_ancestor(audit, &intro),
                "{path}: graph code landed inside the sample→audit blind window (G13)"
            );
        }
    }
}
