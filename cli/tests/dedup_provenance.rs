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
//!
//! The auditor-blindness half that ordering cannot carry is
//! procedural and enforced at assembly time instead: the assembly
//! carries verbatim code and no judgment fields (RM18, asserted by
//! the eval_t3_audit gates).
//!
//! The docdup family (D12, M5-3g) honors the FULL original chain the
//! T3 geometry could not: its census derives from the 3d exact
//! oracle, so it froze — and its audits froze — before a single line
//! of CE/Docdup or cli/src/docdup/judge existed. The census's
//! judgment-independence is carried by the full-re-derivation CI
//! gate (eval_docdup_sample), not by git ordering: the 2026-08-14
//! amendment refroze its CONTENT (a strict subset of the audited
//! v1 draw plus zero additions), while the git legs below pin the
//! audits ≺ judge ≺ scoring order.

mod eval_support;

use eval_support::{assert_subtree_postdates, intro_commit, require_full_history};

const CORPORA: [&str; 5] = ["cobra", "requests", "ripgrep", "self", "zod"];

/// The judge subtrees whose every file must postdate the T3 sample
/// (design §9.2: the full union — the docdup extraction landed in 3d
/// and the docdup judge in 3g, both after the 3c draw).
const JUDGE_SUBTREES: [&str; 5] = [
    "cli/src/dedup/t3",
    "cli/src/docdup",
    "core/app/CE/Clone",
    "core/app/CE/Clone.hs",
    "core/app/CE/Docdup",
];

const SAMPLE_PATH: &str = "contracts/eval/t3-sample-v1.json";
const DOCDUP_SAMPLE: &str = "contracts/eval/docdup-sample-v1.json";

/// The docdup JUDGE subtrees — narrower than the family's whole
/// directory on purpose: the 3d extraction defines the universe and
/// legitimately precedes the census; only the judgment could game it.
const DOCDUP_JUDGE_SUBTREES: [&str; 3] = [
    "cli/src/docdup/judge",
    "core/app/CE/Docdup",
    "core/app/CE/Docdup.hs",
];

/// Leg 1 (RM2): the sample was frozen before a single line of the
/// judge existed — full-subtree scan, not a convention.
#[test]
fn sample_precedes_the_judge_subtrees() {
    require_full_history();
    let sample = intro_commit(SAMPLE_PATH);
    for subtree in JUDGE_SUBTREES {
        assert_subtree_postdates(&sample, subtree);
    }
}

/// Legs 2 and 3: sample ≺ every audit table ≺ every precision doc's
/// generated_from.commit — the shared ordering walk.
#[test]
fn sample_audit_scoring_ordered() {
    eval_support::assert_audit_scoring_legs(
        SAMPLE_PATH,
        &|c| format!("cli/tests/eval_t3_review/{c}.json"),
        &CORPORA,
        "t3-precision",
        "T-G13",
    );
}

/// D12 leg 1: BOTH the docdup census and its audit tables strictly
/// precede every file of the docdup judge subtrees — the audit could
/// not have seen the judge, by git fact, not procedure.
#[test]
fn docdup_census_and_audits_precede_the_judge_subtrees() {
    require_full_history();
    let census = intro_commit(DOCDUP_SAMPLE);
    let audit = intro_commit("cli/tests/eval_docdup_review/zod.json");
    for subtree in DOCDUP_JUDGE_SUBTREES {
        assert_subtree_postdates(&census, subtree);
        assert_subtree_postdates(&audit, subtree);
    }
}

/// D12 legs 2 and 3: census ≺ every audit table ≺ every
/// docdup-precision doc's generated_from.commit.
#[test]
fn docdup_sample_audit_scoring_ordered() {
    eval_support::assert_audit_scoring_legs(
        DOCDUP_SAMPLE,
        &|c| format!("cli/tests/eval_docdup_review/{c}.json"),
        &CORPORA,
        "docdup-precision",
        "D12",
    );
}
