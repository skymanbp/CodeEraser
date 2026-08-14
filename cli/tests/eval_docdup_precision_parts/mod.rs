//! Attainment-line-B scorer parts (M5-3g): the verdict vocabulary,
//! the delegated correct↔truth mapping, the oracle join and the
//! J-floor grid — pure functions over frozen materials, shared by the
//! generator and the CI gates so the two can never diverge.

use serde_json::{Value, json};
use std::collections::BTreeMap;

/// Verdict classes (D4 conservation): scoped correct/wrong carry the
/// gate; docstring rows publish outside the D3 scope (decision ④);
/// not_reported closes the census.
pub const VERDICTS: [&str; 4] = ["correct", "wrong", "docstring", "not_reported"];

/// The delegated correct↔truth mapping (the 3f "most elegant"
/// delegation, docdup edition, recorded in every doc method):
/// correct = genuine documentation duplication {redundant,
/// paraphrase}; wrong = the no-duplication families {license,
/// skeleton, tabular, quoted, deliberate_xref, unrelated}.
pub const CORRECT_TRUTHS: [&str; 2] = ["redundant", "paraphrase"];

/// The published J-floor grid (percent numerators over 100); 80 is
/// the shipped jaccardNum.
pub const FLOOR_GRID: [u64; 9] = [50, 60, 70, 75, 80, 85, 90, 95, 100];

/// A census row's scope: docstring pairs publish ungated.
pub fn scope_of(row: &Value) -> &'static str {
    let kind = |k: &str| row[k].as_str().expect(k);
    if kind("a_kind") == "docstring" || kind("b_kind") == "docstring" {
        "docstring"
    } else {
        "scoped"
    }
}

/// The ONE verdict derivation.
pub fn verdict_of(truth: &str, reported: bool, scope: &str) -> &'static str {
    if !reported {
        "not_reported"
    } else if scope == "docstring" {
        "docstring"
    } else if CORRECT_TRUTHS.contains(&truth) {
        "correct"
    } else {
        "wrong"
    }
}

/// A pair's identity key from flat census fields.
pub fn census_key(row: &Value) -> (String, i64, String, i64) {
    let s = |k: &str| row[k].as_str().expect(k).to_string();
    let n = |k: &str| row[k].as_i64().expect(k);
    (s("a_path"), n("a_start"), s("b_path"), n("b_start"))
}

/// The same key from an oracle pair's nested sides.
pub fn oracle_key(p: &Value) -> (String, i64, String, i64) {
    let side = |s: &str, k: &str| p[s][k].clone();
    (
        side("a", "path").as_str().expect("path").to_string(),
        side("a", "start_line").as_i64().expect("line"),
        side("b", "path").as_str().expect("path").to_string(),
        side("b", "start_line").as_i64().expect("line"),
    )
}

/// identity → (inter, union, verbatim) of one corpus's frozen oracle
/// — the exact numbers every re-derivation joins on.
pub fn oracle_join(corpus: &str) -> BTreeMap<(String, i64, String, i64), (u64, u64, u64)> {
    let name = (corpus != "self").then(|| corpus.to_string());
    let doc = super::eval_support::load(&super::eval_support::eval_doc(
        &super::eval_support::doc_stem("docdup-oracle", &name),
    ));
    doc["pairs"]
        .as_array()
        .expect("pairs")
        .iter()
        .map(|p| {
            let n = |k: &str| p[k].as_u64().expect(k);
            (oracle_key(p), (n("inter"), n("union"), n("verbatim")))
        })
        .collect()
}

/// Does the report rule fire at `floor`/100 on these exact numbers?
pub fn reported_at(floor: u64, inter: u64, union: u64, verbatim: u64) -> bool {
    inter * 100 >= floor * union || verbatim >= codeeraser::docdup::spec::VERBATIM_FLOOR as u64
}

/// One frozen row's full check (D2/D5/D6 row half): truth echoed
/// against the audit, verdict re-derived through the ONE derivation,
/// oracle echo intact, and — for reported rows — the core's exact
/// inter/union/verbatim EQUAL to the offline oracle's (D2: the
/// re-check demonstrably fired; two independent exact computations
/// agreeing is the strongest cheap proof there is) with the report
/// rule satisfied.
pub fn check_row(
    corpus: &str,
    rank: &str,
    row: &Value,
    truths: &BTreeMap<&str, &str>,
    oracle: &BTreeMap<(String, i64, String, i64), (u64, u64, u64)>,
) {
    super::eval_support::assert_row_verdict(corpus, rank, row, truths, |truth| {
        verdict_of(
            truth,
            row["judged"]["reported"] == json!(true),
            scope_of(row),
        )
    });
    let (oi, ou, ov) = oracle[&census_key(row)];
    assert_eq!(
        row["oracle"],
        json!({"inter": oi, "union": ou, "verbatim": ov}),
        "{corpus}/{rank}: oracle echo drifted"
    );
    if row["judged"]["reported"] == json!(true) {
        for (key, want) in [("inter", oi), ("union", ou), ("verbatim", ov)] {
            assert_eq!(
                row["judged"][key],
                json!(want),
                "{corpus}/{rank}: core {key} disagrees with the exact oracle (D2)"
            );
        }
        assert!(
            reported_at(80, oi, ou, ov),
            "{corpus}/{rank}: reported below both report rules (D2)"
        );
    }
}

/// One frozen doc's whole frame (D4/D5 + T-G1 posture): constants
/// single-bound, census bijection with verbatim identity echo and tip
/// pin, every row checked, summary re-derived, verdict conservation.
pub fn check_precision_doc(corpus: &str, doc: &Value, sample: &Value) {
    assert_eq!(
        doc["constants"],
        super::eval_support::docdup_constants(),
        "{corpus}: constants"
    );
    let review = super::eval_support::docdup_review_doc(corpus);
    let review_rows = review["rows"].as_array().expect("rows");
    let truths: BTreeMap<&str, &str> = review_rows
        .iter()
        .map(|r| {
            (
                r["rank"].as_str().expect("rank"),
                r["truth"].as_str().expect("truth"),
            )
        })
        .collect();
    let oracle = oracle_join(corpus);
    let sampled = super::eval_support::of_corpus(sample["main"].as_array().expect("main"), corpus);
    let rows = doc["rows"].as_array().expect("rows");
    super::eval_support::each_audited_row(corpus, rows, &sampled, |rank, row, s| {
        for key in super::eval_support::DOCDUP_IDENTITY {
            assert_eq!(row[key], s[key], "{corpus}/{rank}: {key} echo drifted");
        }
        assert_eq!(
            doc["corpus"]["tip"], s["tip"],
            "{corpus}: tip vs sampled tip"
        );
        check_row(corpus, rank, row, &truths, &oracle);
    });
    super::eval_support::assert_summary_rederived(corpus, doc, |rows| rescore(corpus, rows));
    super::eval_support::assert_verdict_conservation(corpus, doc, &VERDICTS);
}

/// The per-corpus gate half: D3 on the scoped answered pair, D1 from
/// the frozen recall numbers, the aggregate fed by kind slices.
pub fn corpus_gate(path: &str, sample: &Value, agg: &mut super::eval_support::PrecisionAgg) {
    let (key, doc, c, w) = super::eval_support::open_scored_doc(
        path,
        "docdup-precision",
        sample,
        agg,
        check_precision_doc,
    );
    let rows = doc["rows"].as_array().expect("rows");
    agg.rows += rows.len() as u64;
    for row in rows {
        super::eval_support::tally_add(&mut agg.by_lang, row["a_kind"].as_str().expect("kind"), 1);
    }
    super::eval_support::assert_corpus_precision(&key, (c, w), c + w, 0.85, "D3 attainment line B");
    let d1 = &doc["d1"];
    let (found, total) = (
        d1["found"].as_u64().expect("found"),
        d1["total"].as_u64().expect("total"),
    );
    assert!(
        total == 0 || found as f64 / total as f64 >= 0.99,
        "{key}: D1 oracle recall {found}/{total} < 0.99"
    );
    assert_eq!(
        d1["missing"].as_array().expect("missing").len() as u64,
        total - found,
        "{key}: D1 missing list vs counters"
    );
}

/// The summary re-derived from rows + the frozen oracle numbers: the
/// verdict tallies, the scoped/docstring splits and the J-floor grid
/// (T-G1 posture — the stored summary can only ever be this).
pub fn rescore(corpus: &str, rows: &[Value]) -> Value {
    let oracle = oracle_join(corpus);
    let mut verdicts: BTreeMap<String, u64> = BTreeMap::new();
    let mut doc_split: BTreeMap<String, u64> = BTreeMap::new();
    let mut grid: BTreeMap<String, Value> = BTreeMap::new();
    for row in rows {
        super::eval_support::tally_add(&mut verdicts, row["verdict"].as_str().expect("v"), 1);
        if row["verdict"] == "docstring" {
            let key = if CORRECT_TRUTHS.contains(&row["truth"].as_str().expect("t")) {
                "correct"
            } else {
                "wrong"
            };
            super::eval_support::tally_add(&mut doc_split, key, 1);
        }
    }
    for floor in FLOOR_GRID {
        let (mut c, mut w) = (0u64, 0u64);
        for row in rows {
            let (inter, union, verbatim) = oracle[&census_key(row)];
            if scope_of(row) == "scoped" && reported_at(floor, inter, union, verbatim) {
                if CORRECT_TRUTHS.contains(&row["truth"].as_str().expect("t")) {
                    c += 1;
                } else {
                    w += 1;
                }
            }
        }
        grid.insert(floor.to_string(), json!({"correct": c, "wrong": w}));
    }
    json!({"verdicts": verdicts, "docstring": doc_split, "floor_grid": grid})
}
