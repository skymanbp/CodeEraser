//! M5-3d docdup-oracle instrument (instruments §9.5 C1): brute-force
//! EXACT Jaccard + verbatim-run pairs over the LIVE admitted segments
//! of each pinned tree. MinHash/LSH (3g) is the estimator; this doc
//! is its uncontestable denominator — no human in the loop, D1
//! recall >= 0.99 measures the estimator against it. Enumeration via
//! the shingle inverted index is complete, not approximate: both
//! emission rules require at least one shared shingle.
//!
//! Generate (per corpus; external corpora via CE_SLICE_REPO +
//! CE_GRAPH_NAME + CE_GRAPH_TIP; release — the run DP is quadratic):
//!   cargo test --release --test eval_docdup_oracle -- --ignored --nocapture

mod eval_support;

use eval_support::*;
use serde_json::{Value, json};

fn oracle_constants() -> Value {
    json!({
        "jaccard_universe_floor": [JACCARD_UNIVERSE_FLOOR.0, JACCARD_UNIVERSE_FLOOR.1],
        "verbatim_floor": codeeraser::docdup::spec::VERBATIM_FLOOR,
        "doc_shingle": codeeraser::docdup::spec::DOC_SHINGLE,
        "oracle_segcap": DOCDUP_ORACLE_SEGCAP,
    })
}

/// CI gate, no git: tip anchored to the segments sibling, constants
/// single-bound, rows sorted by identity, every row satisfies an
/// emission rule, endpoints exist in the sibling inventory, summary
/// re-derived.
#[test]
fn docdup_oracle_consistent() {
    each_frozen_doc("docdup-oracle", |path, doc| {
        let name = doc_suffix(path, "docdup-oracle");
        let sibling = load(&eval_doc(&doc_stem("docdup-segments", &name)));
        assert_eq!(
            doc["corpus"]["tip"], sibling["corpus"]["tip"],
            "{path}: tip differs from the segments sibling"
        );
        assert_eq!(doc["constants"], oracle_constants(), "{path}: constants");
        let inventory = str_pairs(&sibling, "files", "path", "sha256");
        let floor = codeeraser::docdup::spec::VERBATIM_FLOOR as u64;
        let pairs = doc["pairs"].as_array().expect("pairs");
        let mut last_id: Option<(String, i64, String, i64)> = None;
        for p in pairs {
            let id = (
                p["a"]["path"].as_str().expect("a").to_string(),
                p["a"]["start_line"].as_i64().expect("a line"),
                p["b"]["path"].as_str().expect("b").to_string(),
                p["b"]["start_line"].as_i64().expect("b line"),
            );
            assert!(
                last_id.as_ref() < Some(&id),
                "{path}: rows unsorted or duplicated at {id:?}"
            );
            last_id = Some(id);
            for side in ["a", "b"] {
                let sp = p[side]["path"].as_str().expect("path");
                assert!(inventory.contains_key(sp), "{path}: {sp} not in sibling");
            }
            let (i, u) = (
                p["inter"].as_u64().expect("inter"),
                p["union"].as_u64().expect("union"),
            );
            assert!(
                i * JACCARD_UNIVERSE_FLOOR.1 >= JACCARD_UNIVERSE_FLOOR.0 * u
                    || p["verbatim"].as_u64().expect("verbatim") >= floor,
                "{path}: pair below both emission rules"
            );
        }
        let s = &doc["summary"];
        assert_eq!(s["pairs"].as_u64(), Some(pairs.len() as u64), "{path}");
        assert!(
            s["segments_live"].as_u64().expect("live") as usize <= DOCDUP_ORACLE_SEGCAP,
            "{path}: over segcap yet not withheld"
        );
    });
}
