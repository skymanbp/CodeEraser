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

/// The sample was drawn blind, then audited: every review table must
/// descend from the commit that froze the sample.
#[test]
fn sample_precedes_audit() {
    require_full_history();
    let sample = intro_commit("contracts/eval/graph-sample-v1.json");
    for corpus in CORPORA {
        let audit = intro_commit(&format!("cli/tests/eval_graph_review/{corpus}.json"));
        assert!(
            is_ancestor(&sample, &audit),
            "{corpus}: audit table predates the sample freeze (G13)"
        );
    }
}

/// Armed tripwire: today no resolver exists (cli/src/graph/ladder/
/// has no history — that emptiness is itself asserted structure);
/// the moment a ladder file lands, its FIRST commit must descend
/// from every audit table, or "audit before scoring" was violated.
#[test]
fn audit_precedes_any_resolver() {
    require_full_history();
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
    let Some(first_ladder) = ladder.lines().next() else {
        return; // pre-resolver era: nothing to order yet, gate stays armed
    };
    for corpus in CORPORA {
        let audit = intro_commit(&format!("cli/tests/eval_graph_review/{corpus}.json"));
        assert!(
            is_ancestor(&audit, first_ladder),
            "{corpus}: a resolver landed before the audit froze (G13)"
        );
    }
}
