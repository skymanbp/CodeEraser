//! G13 (design §5): "sampled before audited before scored" as a
//! CHECKED git fact, not a convention. This is deliberately the one
//! graph gate that runs git — CI checks out with fetch-depth: 0 for
//! exactly this test, and a shallow clone refuses loudly instead of
//! passing vacuously.

mod eval_support;

use eval_support::{git_in, intro_commit, is_ancestor, is_strict_ancestor, require_full_history};

const CORPORA: [&str; 5] = ["cobra", "requests", "ripgrep", "self", "zod"];

fn audit_intros() -> Vec<String> {
    eval_support::corpus_intros(
        &|c| format!("cli/tests/eval_graph_review/{c}.json"),
        &CORPORA,
    )
}

/// Legs 1 and 3: sample ≺ every audit table ≺ every precision doc's
/// generated_from.commit — the shared ordering walk.
#[test]
fn sample_audit_scoring_ordered() {
    eval_support::assert_audit_scoring_legs(
        "contracts/eval/graph-sample-v1.json",
        &|c| format!("cli/tests/eval_graph_review/{c}.json"),
        &CORPORA,
        "graph-precision",
        "G13",
    );
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
