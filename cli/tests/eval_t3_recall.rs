//! M5-close instrument B: T3 recall vs mizchi/similarity (plan §6
//! M5-3A as amended v1.6 — the literal ≥0.90 gate is instrument-
//! demonstrated unreachable under the repo's own registered TSED
//! definition against this comparator, so the gate is a REGRESSION
//! FLOOR: recall may only improve across re-freezes). The frozen
//! docs carry the comparator's full detectable set (the denominator
//! NEVER shrinks — decision ③), ce full-stack credit per pair, and a
//! CLOSED miss-attribution vocabulary in which every miss is
//! mechanical and definitional: size_bound_not_clone (best-case
//! similarity min/max < 0.85 under ce's TSED — provably impossible),
//! below_floor (the registered T3_MIN_NODES=24 short-unit domain
//! bound), judged_not_clone (sent through TED, refused at θ),
//! no_unit (no ce unit overlaps a side). Coverage: zod/requests/
//! cobra; ripgrep is excluded (similarity-rs is "(future)" per the
//! tool's own --supported output) and self is excluded (its covered-
//! language slice is crosscheck fixture data the product walk
//! excludes) — both reasons ride every doc's method text.
//!
//! Generation is scripted outside the repo test tree (the tools are
//! cargo-installed binaries; provenance — name, version, channel,
//! threshold source file:line, invocation — rides each doc's tool
//! block); this gate replays the arithmetic from the frozen rows.

mod eval_support;

use eval_support::load;

/// (corpus, frozen detected, frozen denominator) — the regression
/// floor epoch (2026-08-14, S5 exhaustive source active). A future
/// re-freeze may only improve the ratio.
const FLOORS: [(&str, u64, u64); 3] = [("zod", 3, 6), ("requests", 67, 425), ("cobra", 1417, 9205)];

const VOCAB: [&str; 4] = [
    "no_unit",
    "below_floor",
    "size_bound_not_clone",
    "judged_not_clone",
];

fn doc_path(corpus: &str) -> String {
    format!("../contracts/eval/t3-recall-{corpus}-v1.json")
}

type Buckets = std::collections::BTreeMap<&'static str, u64>;

/// Walk the frozen rows once: (detected, t1t2, t3-of-rest, miss
/// buckets), refusing anything outside the closed vocabulary and any
/// detected row carrying an attribution.
fn tally_rows(corpus: &str, rows: &[serde_json::Value]) -> (u64, u64, u64, Buckets) {
    let (mut det, mut t12, mut t3_rest) = (0u64, 0u64, 0u64);
    let mut buckets: Buckets = Default::default();
    for r in rows {
        let layers: Vec<&str> = r["layers"]
            .as_array()
            .expect("layers")
            .iter()
            .map(|l| l.as_str().expect("layer"))
            .collect();
        let attribution = r["attribution"].as_str();
        if layers.is_empty() {
            let a = attribution.expect("every miss carries an attribution");
            let seat = VOCAB
                .iter()
                .find(|v| **v == a)
                .unwrap_or_else(|| panic!("{corpus}: {a} outside the closed vocabulary"));
            *buckets.entry(seat).or_insert(0) += 1;
        } else {
            assert!(
                attribution.is_none(),
                "{corpus}: detected rows carry no attribution"
            );
            det += 1;
            if layers.contains(&"t1t2") {
                t12 += 1;
            } else if layers.contains(&"t3") {
                t3_rest += 1;
            }
        }
    }
    (det, t12, t3_rest, buckets)
}

/// Provenance + description pins (split from the test body at the
/// E01 warn line): tool identity fields, coverage exclusions, and
/// the method text SPEAKING the vocabulary the rows use — the first
/// freeze described a different one (clearance review: the prose
/// denied the very separation the rows perform).
fn check_provenance(corpus: &str, doc: &serde_json::Value, buckets: &Buckets) {
    assert_eq!(doc["schema"], "ce.eval-t3-recall/1.0.0", "{corpus}");
    assert_eq!(doc["corpus"]["name"], corpus, "{corpus}: name");
    for field in ["name", "version", "threshold_source", "invocation"] {
        assert!(
            doc["tool"][field].as_str().is_some_and(|s| !s.is_empty()),
            "{corpus}: tool.{field} missing — provenance is the contract"
        );
    }
    let method = doc["method"].as_str().expect("method");
    assert!(
        method.contains("ripgrep EXCLUDED") && method.contains("self EXCLUDED"),
        "{corpus}: the coverage exclusions must ride the doc"
    );
    for seat in buckets.keys() {
        assert!(
            method.contains(seat),
            "{corpus}: bucket {seat} carries misses but the method text never names it"
        );
    }
}

#[test]
fn recall_docs_replay_and_never_regress() {
    for (corpus, f_det, f_den) in FLOORS {
        let doc = load(&doc_path(corpus));
        let rows = doc["rows"].as_array().expect("rows");
        let (det, t12, t3_rest, buckets) = tally_rows(corpus, rows);
        check_provenance(corpus, &doc, &buckets);
        // decision ③: the denominator NEVER shrinks — the ratio floor
        // alone would stay green through a row-deleting re-freeze
        // (clearance review HIGH)
        assert!(
            rows.len() as u64 >= f_den,
            "{corpus}: denominator shrank below the frozen epoch {f_den}"
        );
        replay_summary(
            corpus,
            &doc["summary"],
            rows.len() as u64,
            (det, t12, t3_rest),
        );
        let frozen: Buckets = doc["miss_attribution"]
            .as_object()
            .expect("miss_attribution")
            .iter()
            .map(|(k, v)| {
                let seat = VOCAB
                    .iter()
                    .find(|c| **c == k.as_str())
                    .unwrap_or_else(|| panic!("{corpus}: bucket {k} outside vocabulary"));
                (*seat, v.as_u64().expect("count"))
            })
            .collect();
        assert_eq!(buckets, frozen, "{corpus}: attribution drifted from rows");
        // the regression floor: detected/denominator may only improve
        assert!(
            det * f_den >= f_det * rows.len() as u64,
            "{corpus}: recall regressed below the frozen floor {f_det}/{f_den}"
        );
    }
}

/// The summary block replays FROM the tallied rows — including the
/// two PUBLISHED ratios, the only fields the first cut skipped
/// (clearance review: a doc could carry recall_raw 0.90 over
/// 1417/9205 integers and stay green). Split from the test body at
/// the E01 75-line hard cap — the tool biting this very file a
/// second time.
fn replay_summary(corpus: &str, s: &serde_json::Value, den: u64, tally: (u64, u64, u64)) {
    let (det, t12, t3) = tally;
    let n = |k: &str| s[k].as_u64().unwrap_or_else(|| panic!("{corpus}: {k}"));
    assert_eq!(n("denominator"), den, "{corpus}: denominator");
    assert_eq!(n("detected"), det, "{corpus}: detected drifted from rows");
    assert_eq!(n("t1t2_covered"), t12, "{corpus}: t1t2 drifted");
    assert_eq!(n("t3_of_rest"), t3, "{corpus}: t3 drifted");
    assert_eq!(
        n("incremental_denominator"),
        den - t12,
        "{corpus}: incremental denominator"
    );
    // tolerance = half an ulp of the generator's 4-decimal rounding
    // (requests stores 0.1576 for 67/425 = 0.157647… — convention)
    let f = |k: &str| s[k].as_f64().unwrap_or_else(|| panic!("{corpus}: {k}"));
    assert!(
        (f("recall_raw") - det as f64 / den as f64).abs() < 5e-5,
        "{corpus}: recall_raw drifted from detected/denominator"
    );
    assert!(
        (f("recall_incremental") - t3 as f64 / (den - t12) as f64).abs() < 5e-5,
        "{corpus}: recall_incremental drifted from t3_of_rest/incremental_denominator"
    );
}

/// The three covered corpora are the whole family — a fourth doc (or
/// a vanished one) is a coverage-list change that must be decided,
/// not slipped in.
#[test]
fn coverage_list_is_closed() {
    for (corpus, _, _) in FLOORS {
        assert!(
            std::path::Path::new(&doc_path(corpus)).exists(),
            "{corpus}: frozen doc missing"
        );
    }
    for absent in ["ripgrep", "self"] {
        assert!(
            !std::path::Path::new(&doc_path(absent)).exists(),
            "{absent}: excluded from coverage by the documented reasons — a doc here is a decision"
        );
    }
    // summed from the FROZEN DOCS, not the in-file constant — the
    // first cut summed FLOORS, a compile-time truth that could never
    // fire on data (clearance review)
    let total: u64 = FLOORS
        .iter()
        .map(|(corpus, _, _)| {
            load(&doc_path(corpus))["rows"]
                .as_array()
                .expect("rows")
                .len() as u64
        })
        .sum();
    assert!(total >= 9_000, "family denominator went near-vacuous");
}
