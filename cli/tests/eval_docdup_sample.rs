//! M5-3g docdup audit sample (instruments §9.5 C2): the pair rows the
//! independent auditors will judge, frozen BEFORE any docdup judgment
//! code exists (the D12 ancestry leg asserts sample ≺ audits ≺ judge
//! subtree ≺ scoring by git log — the full T-G13 chain, which this
//! family CAN honor because the pool is the 3d exact oracle, not a
//! judge output). The C2 design draws 100 hash-ranked rows assuming a
//! population >= 100; the frozen universe holds 30 report-floor pairs
//! (47 with the sub-floor margin band), so the sample is the COMPLETE
//! CENSUS — zero selection, per-kind floors bounded by population.
//! The margin band is audited too: the floor-sensitivity table gets
//! audited truth on both sides of the 80/100 boundary, and the
//! paraphrase seat can materialize where paraphrase actually lives.
//!
//! Generate (deterministic from the frozen oracle docs — ignored only
//! because it WRITES a contracts file):
//!   cargo test --test eval_docdup_sample -- --ignored --nocapture

mod eval_support;

use eval_support::*;
use serde_json::{Value, json};
use std::collections::BTreeMap;

const RANK_DOMAIN: &str = "ce-docdup-pair-v1";
const AUDIT_DOMAIN: &str = "ce-docdup-audit-v1";

/// The domain-separated hash of one row — the shared identity_hash
/// throat over this family's frozen identity order; the ONE
/// derivation the gate repeats from the frozen fields (D5 posture).
fn row_hash(domain: &str, row: &Value) -> String {
    eval_support::identity_hash(domain, row, &eval_support::DOCDUP_IDENTITY)
}

fn side_fields(row: &mut Value, side: &str, seg: &Value) {
    let get = |k: &str| seg[k].clone();
    row[format!("{side}_path")] = get("path");
    row[format!("{side}_kind")] = get("kind");
    row[format!("{side}_start")] = get("start_line");
    row[format!("{side}_end")] = get("end_line");
}

/// The whole census, re-derived from the frozen oracle docs: rows in
/// audit-domain order plus the population/kind/corpus tallies. One
/// function, two callers (generator and gate) — the doc can only ever
/// be this derivation's output.
fn derive() -> (Vec<Value>, Value) {
    let mut rows = Vec::new();
    let (mut by_corpus, mut by_kind) = (BTreeMap::new(), BTreeMap::new());
    let (mut report, mut margin) = (0u64, 0u64);
    let vfloor = codeeraser::docdup::spec::VERBATIM_FLOOR as u64;
    each_frozen_doc("docdup-oracle", |path, doc| {
        let corpus = doc_suffix(path, "docdup-oracle").unwrap_or_else(|| "self".into());
        let tip = doc["corpus"]["tip"].as_str().expect("tip").to_string();
        for p in doc["pairs"].as_array().expect("pairs") {
            let mut row = json!({"corpus": corpus, "tip": tip});
            side_fields(&mut row, "a", &p["a"]);
            side_fields(&mut row, "b", &p["b"]);
            row["rank"] = json!(row_hash(RANK_DOMAIN, &row));
            let (i, u) = (
                p["inter"].as_u64().expect("inter"),
                p["union"].as_u64().expect("union"),
            );
            if i * JACCARD_REPORT_FLOOR.1 >= JACCARD_REPORT_FLOOR.0 * u
                || p["verbatim"].as_u64().expect("verbatim") >= vfloor
            {
                report += 1;
            } else {
                margin += 1;
            }
            *by_corpus.entry(corpus.clone()).or_insert(0u64) += 1;
            let kinds = format!(
                "{}|{}",
                p["a"]["kind"].as_str().expect("kind"),
                p["b"]["kind"].as_str().expect("kind")
            );
            *by_kind.entry(kinds).or_insert(0u64) += 1;
            rows.push(row);
        }
    });
    rows.sort_by_key(|r| row_hash(AUDIT_DOMAIN, r));
    let tallies = json!({
        "population": {"oracle_pairs": report + margin, "report_floor": report, "margin": margin},
        "by_corpus": by_corpus,
        "by_kind": by_kind,
    });
    (rows, tallies)
}

/// CI gate: the frozen doc IS the derivation's output — rows
/// (including every rank), order and tallies re-derive from the
/// frozen oracle docs; nothing else can sit in main.
#[test]
fn docdup_sample_is_the_oracle_census() {
    let doc = load(&eval_doc("docdup-sample"));
    let (rows, tallies) = derive();
    assert_eq!(
        doc["domains"],
        json!({"rank": RANK_DOMAIN, "audit": AUDIT_DOMAIN}),
        "domains drifted"
    );
    assert_eq!(doc["tallies"], tallies, "population tallies drifted");
    assert_eq!(
        doc["main"],
        Value::Array(rows),
        "census drifted from the oracle derivation"
    );
}
