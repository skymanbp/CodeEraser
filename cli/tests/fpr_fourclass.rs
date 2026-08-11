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
//! Run: CE_CORE_BIN=$(cd core && cabal list-bin ce-core) \
//!      cargo test --test fpr_fourclass -- --ignored --nocapture

mod eval_support;

use codeeraser::corelink::Link;
use codeeraser::fourclass::batch::classify_batch;
use eval_support::{eval_doc, load, sample_pair, write_doc};
use serde_json::{Value, json};

#[test]
#[ignore] // needs the local .ce-eval payloads + a built ce-core
fn generate_fpr_fourclass() {
    let bin = std::env::var("CE_CORE_BIN").expect(
        "CE_CORE_BIN is unset — build the core and export it:\n  \
         cd core && cabal build all && export CE_CORE_BIN=$(cabal list-bin ce-core)",
    );
    let mut link = Link::open(&bin).expect("ce-core link").0;
    let manifest = load("../contracts/eval/manifest-v1.json");
    let rows = manifest["samples"].as_array().expect("samples");
    let mut flagged: Vec<Value> = Vec::new();
    eval_support::each_sample(rows, |id, sample| {
        let inputs = sample_pair(&sample);
        let b = classify_batch(&inputs, Some(&mut link));
        assert!(b.degraded.is_none(), "{id}: degraded {:?}", b.degraded);
        for (_, kind) in &b.suspicions {
            let c = &b.pairs[0].counts;
            flagged.push(json!({
                "id": id, "kind": kind,
                "added_novel": c.added_novel,
                "removed_deleted": c.removed_deleted,
            }));
        }
    });
    let samples = rows.len();
    assert!(samples >= 500, "denominator floor: {samples}");
    let doc = json!({
        "schema": "ce.eval-fpr-fourclass/1.0.0",
        "source_manifest": manifest["frozen_at"],
        "generated_from": eval_support::generated_from(),
        "method": "every manifest sample replayed through \
                   fourclass::batch::classify_batch over a live ce-core \
                   link; a sample counts as a false positive when any M4 \
                   judgment rule fires (the corpus is all-normal by \
                   review). Constants under test live in \
                   CE.FourClass.Verdict. Recall is undefined on this \
                   corpus (zero abnormal samples) and reported as such, \
                   per plan (no gamed 100% recall gate).",
        "samples": samples,
        "flagged": flagged,
        "gate_max_percent": 1,
        "recall": "undefined: corpus contains no abnormal edits",
    });
    write_doc(&eval_doc("fpr-fourclass"), &doc, "FPR doc written");
}

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
