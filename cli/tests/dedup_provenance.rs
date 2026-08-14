//! T-G13 (design vol.3 §9.2): the T3 ancestry legs as CHECKED git
//! facts. The design prose chains "sample ≺ audit ≺ judge subtree ≺
//! scoring" — copied from the graph family, where the audit (2d)
//! really did precede the resolver (2f). The T3 sequence the SAME
//! design registers in §10 puts the judge (3e) before the audit
//! (3f←3e: the judgment must exist to compute min_answered), so the
//! audit-precedes-judge leg is geometrically impossible here and the
//! honest checkable claims are exactly three:
//!   1. the sample strictly precedes EVERY file under the judge
//!      subtrees (the judge never chose its denominator — RM2),
//!   2. the sample strictly precedes every audit table (the audit
//!      audited the frozen draw), and
//!   3. every audit table strictly precedes every precision doc's
//!      generated_from.commit (GT froze before any score existed).
//! The auditor-blindness half that ordering cannot carry is
//! procedural and enforced at assembly time instead: the assembly
//! carries verbatim code and no judgment fields (RM18, asserted by
//! the eval_t3_audit gates). The docdup family adds its own legs in
//! 3g (D12 shares this implementation).

mod eval_support;

use eval_support::{
    assert_subtree_postdates, eval_doc, intro_commit, is_strict_ancestor, load,
    require_full_history,
};

const CORPORA: [&str; 5] = ["cobra", "requests", "ripgrep", "self", "zod"];

/// The judge subtrees whose every file must postdate the sample.
const JUDGE_SUBTREES: [&str; 3] = [
    "cli/src/dedup/t3",
    "core/app/CE/Clone",
    "core/app/CE/Clone.hs",
];

fn sample_intro() -> String {
    intro_commit("contracts/eval/t3-sample-v1.json")
}

fn audit_intros() -> Vec<String> {
    CORPORA
        .iter()
        .map(|c| intro_commit(&format!("cli/tests/eval_t3_review/{c}.json")))
        .collect()
}

/// Leg 1 (RM2): the sample was frozen before a single line of the
/// judge existed — full-subtree scan, not a convention.
#[test]
fn sample_precedes_the_judge_subtrees() {
    require_full_history();
    let sample = sample_intro();
    for subtree in JUDGE_SUBTREES {
        assert_subtree_postdates(&sample, subtree);
    }
}

/// Leg 2: the sample was drawn blind, then audited — every review
/// table strictly descends from the sample freeze.
#[test]
fn sample_precedes_audit() {
    require_full_history();
    let sample = sample_intro();
    for audit in audit_intros() {
        assert!(
            is_strict_ancestor(&sample, &audit),
            "audit table does not strictly descend from the sample freeze (T-G13)"
        );
    }
}

/// Leg 3: the audit froze before any scoring ran — every precision
/// doc's generated_from.commit strictly descends from every audit
/// table.
#[test]
fn audit_precedes_scoring() {
    require_full_history();
    let audits = audit_intros();
    for name in [
        None,
        Some("cobra"),
        Some("requests"),
        Some("ripgrep"),
        Some("zod"),
    ] {
        let stem = match name {
            Some(n) => format!("t3-precision-{n}"),
            None => "t3-precision".into(),
        };
        let doc = load(&eval_doc(&stem));
        let commit = doc["generated_from"]["commit"].as_str().expect("commit");
        for audit in &audits {
            assert!(
                is_strict_ancestor(audit, commit),
                "{stem}: scored before the audit froze (T-G13)"
            );
        }
    }
}
