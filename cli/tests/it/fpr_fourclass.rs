//! M4 MAIN GATE (plan §6 M4): the update-supervision judgment layer
//! replayed over the full 600-sample real-edit corpus — every sample
//! a genuine agent edit, every sample reviewed/known normal
//! (labels-v1: 200/200 is_normal; the corpus contains no abnormal
//! edits at all) — must false-fire at most 1%.
//!
//! Denominator: all 600 manifest samples (plan floor: 500). The rule
//! under test is the M4 stacking suspicion (CE.FourClass.Verdict):
//! novel >= 20 significant lines AND deletions under novel/10 AND a
//! unit key newly duplicated. Recall is UNDEFINED on this corpus by
//! construction (zero abnormal samples) and therefore reported as
//! such, exactly as the plan anticipated ("recall 报告但不设
//! 作弊性 100% 门").
//!
//! Run (the CI gate — no core, no local data): cargo test --test fpr_fourclass
//! Regenerate — the `--ignored` replay half retired in 0c7c936
//! (M7.5a); revive it with its coeval support (EVAL-SET.md「再生成」):
//!   git checkout 0c7c936^ -- cli/tests/fpr_fourclass.rs cli/tests/eval_support
//!   CE_CORE_BIN=$(cd core && cabal list-bin ce-core) cargo test --test fpr_fourclass -- --ignored --nocapture
//!   git checkout HEAD -- cli/tests/fpr_fourclass.rs cli/tests/eval_support

use crate::eval_support::{eval_doc, load};

/// CI gate, no local data needed: the committed replay must clear
/// the plan's main gate — false positives <= 1% of >= 500 samples.
#[test]
fn fpr_gate_holds() {
    let doc = load(&eval_doc("fpr-fourclass"));
    let samples = doc["samples"].as_u64().expect("samples");
    let flagged = doc["flagged"].as_array().expect("flagged").len() as u64;
    assert!(samples >= 500, "denominator floor: {samples}");
    assert!(
        flagged * 100 <= samples * doc["gate_max_percent"].as_u64().expect("gate"),
        "FPR gate: {flagged} of {samples} flagged"
    );
}
