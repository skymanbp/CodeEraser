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
//! Generate (per corpus; corpora under .ce-eval/corpora; CE_CORE_BIN):
//!   cargo test --release --test eval_docdup_precision -- --ignored --nocapture

mod eval_support;
#[path = "eval_docdup_precision_parts/mod.rs"]
mod parts;

use eval_support::*;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

type PairKey = (String, i64, String, i64);

/// "path:start-end kind" (the product Hit name format) → (path,
/// start). A format drift breaks this loudly, never silently.
fn parse_hit(name: &str) -> (String, i64) {
    let (span, _kind) = name.rsplit_once(' ').expect("hit name");
    let (path, lines) = span.rsplit_once(':').expect("hit span");
    let (start, _) = lines.split_once('-').expect("hit lines");
    (path.to_string(), start.parse().expect("start line"))
}

fn hit_key(h: &Value) -> PairKey {
    let (ap, asr) = parse_hit(h["a"].as_str().expect("a"));
    let (bp, bs) = parse_hit(h["b"].as_str().expect("b"));
    (ap, asr, bp, bs)
}

/// The whole product pass over one materialized pinned tree: the
/// full judgment report plus the candidate-pair key set (D1's
/// numerator source), both through product throats only.
fn product_pass(root: &std::path::Path) -> (Value, Value, BTreeSet<PairKey>) {
    let report = codeeraser::docdup::judge::run(root, None, &core_bin()).expect("product run");
    let (idx, _) = codeeraser::dedup::refreshed_index(root, None).expect("index");
    let segs = codeeraser::docdup::judge::candidates::live_rows(&idx).expect("live rows");
    let cand = codeeraser::docdup::judge::candidates::collect(root, &segs).expect("candidates");
    let cand_keys: BTreeSet<PairKey> = cand
        .pairs
        .iter()
        .map(|&(a, b, _)| {
            (
                segs[a].path.clone(),
                segs[a].start_line,
                segs[b].path.clone(),
                segs[b].start_line,
            )
        })
        .collect();
    (
        serde_json::to_value(&report.hits).expect("hits"),
        serde_json::to_value(&report.counts).expect("counts"),
        cand_keys,
    )
}

/// One corpus generated end to end: anchored walk, materialized tree,
/// full product run, candidate pass, census join, frozen doc.
fn generate_corpus(name: &Option<String>, sample: &Value) {
    let corpus = name.as_deref().unwrap_or("self").to_string();
    let (seg_doc, tip) = frozen_tip("docdup-segments", name);
    let (walked, _) = walk_tree_in(corpus_repo(name).as_deref(), &tip);
    let frozen = str_pairs(&seg_doc, "files", "path", "sha256");
    assert_eq!(frozen.len(), walked.len(), "{corpus}: inventory drifted");
    let root = materialize_tree("docdup", &corpus, &walked);
    let (hits, universe, cand_keys) = product_pass(&root);
    let dups: BTreeMap<PairKey, &Value> = hits
        .as_array()
        .expect("hits")
        .iter()
        .map(|h| (hit_key(h), h))
        .collect();
    let oracle = parts::oracle_join(&corpus);
    let d1 = d1_recall(&corpus, &oracle, &cand_keys);
    let rows: Vec<Value> = of_corpus(sample["main"].as_array().expect("main"), &corpus)
        .iter()
        .map(|r| census_row(&corpus, r, &dups, &oracle))
        .collect();
    // D8: the wrong ledger is frozen — regressions need the family's
    // explicit blessing variable, never a silent regeneration.
    let path = eval_doc(&doc_stem("docdup-precision", name));
    assert_ledger_frozen(&path, &rows, &eval_support::wrong_ranks, "CE_ACCEPT_DOCDUP");
    let doc = json!({
        "schema": "ce.eval-docdup-precision/1.0.0",
        "corpus": {"name": name, "tip": tip},
        "constants": docdup_constants(),
        "generated_from": generated_from(),
        "method": "every census row joined to the FULL product judgment on the \
                   materialized pinned tree; correct = {redundant, paraphrase} \
                   (delegated mapping), wrong = the six no-duplication families; \
                   the D3 gate covers md_para+comment_block, docstring publishes \
                   ungated (decision ④); D1 = product coarse-filter recall over \
                   the oracle's report-floor pairs; D2 = the core's exact \
                   inter/union must equal the offline oracle's; the J-floor grid \
                   re-derives from the frozen oracle numbers. The census is the \
                   full 32-pair REV-3 universe (population < 100, zero \
                   selection); per-kind floors are population-bounded.",
        "universe": universe,
        "d1": d1,
        "summary": parts::rescore(&corpus, &rows),
        "rows": rows,
    });
    write_doc(&path, &doc, &format!("{path} written"));
}

/// D1: the coarse filter's recall over the oracle's report-floor
/// pairs — the hard 0.99 gate, asserted at generation AND re-checked
/// from the frozen numbers in CI.
fn d1_recall(
    corpus: &str,
    oracle: &BTreeMap<PairKey, (u64, u64, u64)>,
    cand: &BTreeSet<PairKey>,
) -> Value {
    let floor: Vec<&PairKey> = oracle
        .iter()
        .filter(|(_, (i, u, v))| parts::reported_at(80, *i, *u, *v))
        .map(|(k, _)| k)
        .collect();
    let missing: Vec<String> = floor
        .iter()
        .filter(|k| !cand.contains(**k))
        .map(|k| format!("{k:?}"))
        .collect();
    let (found, total) = (floor.len() - missing.len(), floor.len());
    assert!(
        total == 0 || found as f64 / total as f64 >= 0.99,
        "{corpus}: D1 oracle recall {found}/{total} < 0.99 — missing {missing:?}"
    );
    json!({"found": found, "total": total, "missing": missing})
}

/// One census row joined: identity + truth echo slots come from the
/// frozen census; judged facts from the product run; oracle echo from
/// the frozen oracle; verdict through the ONE derivation.
fn census_row(
    corpus: &str,
    r: &Value,
    dups: &BTreeMap<PairKey, &Value>,
    oracle: &BTreeMap<PairKey, (u64, u64, u64)>,
) -> Value {
    let key = parts::census_key(r);
    let truth = {
        let review = docdup_review_doc(corpus);
        let rows = review["rows"].as_array().expect("rows").clone();
        rows.iter()
            .find(|row| row["rank"] == r["rank"])
            .unwrap_or_else(|| panic!("{corpus}: census row missing from audit"))["truth"]
            .as_str()
            .expect("truth")
            .to_string()
    };
    let (oi, ou, ov) = oracle[&key];
    let hit = dups.get(&key);
    let mut row = json!({
        "rank": r["rank"],
        "truth": truth,
        "oracle": {"inter": oi, "union": ou, "verbatim": ov},
        "judged": match hit {
            Some(h) => json!({"reported": true, "inter": h["inter"], "union": h["union"],
                              "verbatim": h["verbatim"]}),
            None => json!({"reported": false}),
        },
    });
    for k in DOCDUP_IDENTITY {
        row[k] = r[k].clone();
    }
    row["verdict"] = json!(parts::verdict_of(
        &truth,
        hit.is_some(),
        parts::scope_of(&row)
    ));
    row
}

#[test]
#[ignore] // needs the corpus repositories + CE_CORE_BIN
fn generate_docdup_precision() {
    let sample = load(&eval_doc("docdup-sample"));
    for name in FROZEN_CORPORA {
        generate_corpus(&name.map(str::to_string), &sample);
    }
}

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
