//! M5-3g attainment line B: the docdup judgment scored against the
//! frozen independent audit. Generation runs the FULL product
//! judgment (`docdup::judge::run`) plus the product candidate pass
//! over each materialized pinned tree; the frozen docs carry every
//! census row's verdict, the D1 coarse-filter recall against the
//! exact oracle, the D2 reliability echo (the core's exact Jaccard
//! must EQUAL the offline oracle's — proof the re-check fired), the
//! Counts universe ledger and the published J-floor grid. Method
//! registration: correct = {redundant, paraphrase} (delegated
//! mapping, parts::CORRECT_TRUTHS); the D3 gate covers md_para +
//! comment_block (decision ④), docstring rows publish ungated.
//!
//! Regenerate — the `--ignored` generator half retired in 0c7c936
//! (M7.5a); revive it with its coeval support (EVAL-SET.md「再生成」):
//!   git checkout 0c7c936^ -- cli/tests/eval_docdup_precision.rs cli/tests/eval_support
//!   cargo test --release --test eval_docdup_precision -- --ignored --nocapture   # per corpus; corpora under .ce-eval/corpora; CE_CORE_BIN
//!   git checkout HEAD -- cli/tests/eval_docdup_precision.rs cli/tests/eval_support

mod eval_support;
#[path = "eval_docdup_precision_parts/mod.rs"]
mod parts;

use eval_support::*;
use serde_json::json;

/// The attainment-line-B contract: 32 census rows across corpora,
/// D3 at least 0.85 overall and per corpus where the answered
/// denominator reaches 5, D1/D2/D4/D5 per doc, then D9 — the shared
/// mutation table plus the re-check-stub and exemption-stub bespoke
/// variants.
const SPEC: eval_support::FamilySpec = eval_support::FamilySpec {
    mutations: &[
        ("rank", "not-a-census-rank", "phantom rank"),
        ("truth", "mismatch", "cooked truth echo"),
        ("a_path", "phantom/path.md", "identity echo drift"),
    ],
    total: 32,
    slice_floor: 0, // per-kind floors are population-bounded (method)
    gate: 0.85,
    tag: "D3 attainment line B",
    family: "docdup-precision",
    sample: "docdup-sample",
};

#[test]
fn precision_contract_and_refusals() {
    eval_support::assert_precision_family(
        &SPEC,
        parts::corpus_gate,
        &parts::check_precision_doc,
        |pristine, refused| {
            let rows = pristine["rows"].as_array().expect("rows");
            let reported = rows
                .iter()
                .position(|r| r["judged"]["reported"] == json!(true))
                .expect("zod holds a reported row");
            // D9 re-check stub: a core number drifting off the exact
            // oracle must refuse (D2 is a live gate, not prose)
            let mut stubbed = pristine.clone();
            let inter = stubbed["rows"][reported]["judged"]["inter"]
                .as_u64()
                .expect("inter");
            stubbed["rows"][reported]["judged"]["inter"] = json!(inter + 1);
            assert!(refused(&stubbed), "re-check stub must refuse");
            // D9 exemption stub: a verdict flipped off its own truth
            // must refuse (the verdict re-derives; the wrong→correct
            // direction has no seat here because the frozen zod doc
            // holds ZERO wrong rows — the flip direction exercises
            // the same ONE derivation)
            let scoped = rows
                .iter()
                .position(|r| r["verdict"] == "correct")
                .expect("zod holds a correct scoped row");
            let mut flipped = pristine.clone();
            flipped["rows"][scoped]["verdict"] = json!("wrong");
            assert!(refused(&flipped), "exemption stub must refuse");
            // cooked floor grid: the published curve re-derives
            // (value-robust: +1 off the stored number, never a
            // constant that might equal it)
            let mut cooked = pristine.clone();
            let stored = cooked["summary"]["floor_grid"]["80"]["correct"]
                .as_u64()
                .expect("grid");
            cooked["summary"]["floor_grid"]["80"]["correct"] = json!(stored + 1);
            assert!(refused(&cooked), "cooked floor grid must refuse");
        },
    );
}

/// F12 (DEDUP-CALIBRATION.md:107): the requests corpus's six pure
/// docstring misses are now split by mechanism — the models.py
/// __bool__/__nonzero__ family is REPORTED by the docdup judge, and
/// api.py's post/put/patch trio sits in the below_floor ledger by
/// name (the skeleton strip leaves them under 50 words).
#[test]
fn f12_calibration_misses_resolved() {
    let doc = load(&eval_doc("docdup-precision-requests"));
    let models_reported = doc["rows"]
        .as_array()
        .expect("rows")
        .iter()
        .filter(|r| {
            r["a_path"] == "src/requests/models.py" && r["judged"]["reported"] == json!(true)
        })
        .count();
    assert!(
        models_reported >= 3,
        "models.py docstring family must be reported (F12), got {models_reported}"
    );
    let segs = load(&eval_doc("docdup-segments-requests"));
    let api = segs["files"]
        .as_array()
        .expect("files")
        .iter()
        .find(|f| f["path"] == "src/requests/api.py")
        .expect("api.py row");
    assert!(
        api["ledger"]["below_floor"].as_u64().unwrap_or(0) >= 1,
        "api.py trio must appear as named below_floor ledger rows (F12)"
    );
}
