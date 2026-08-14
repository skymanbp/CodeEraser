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

const ORACLE_METHOD: &str = "exact oracle over live admitted segments: every pair \
    with integer-exact Jaccard >= 30/100 on the deduped shingle sets, plus every \
    pair whose longest common contiguous shingle run spans >= 50 words. Candidate \
    enumeration walks the shingle inverted index — complete for both rules since \
    each requires a shared shingle. Segment counts above oracle_segcap withhold \
    the corpus with a written reason (F31), never a silent downsample.";

fn oracle_constants() -> Value {
    json!({
        "jaccard_universe_floor": [JACCARD_UNIVERSE_FLOOR.0, JACCARD_UNIVERSE_FLOOR.1],
        "verbatim_floor": codeeraser::docdup::spec::VERBATIM_FLOOR,
        "doc_shingle": codeeraser::docdup::spec::DOC_SHINGLE,
        "oracle_segcap": DOCDUP_ORACLE_SEGCAP,
    })
}

fn seg_id(s: &OracleSeg) -> Value {
    json!({
        "path": s.path,
        "kind": codeeraser::docdup::spec::KIND_NAMES[s.kind as usize],
        "start_line": s.start_line,
        "end_line": s.end_line,
    })
}

/// All emitted pair rows, in identity order.
fn oracle_pairs(segs: &[OracleSeg]) -> Vec<Value> {
    let mut out = Vec::new();
    for (i, j) in candidate_pairs(segs) {
        let (a, b) = (&segs[i], &segs[j]);
        let inter = set_inter(&a.set, &b.set);
        let union = a.set.len() as u64 + b.set.len() as u64 - inter;
        let verbatim = verbatim_words(&a.seq, &b.seq) as u64;
        let jaccard_hit = inter * JACCARD_UNIVERSE_FLOOR.1 >= JACCARD_UNIVERSE_FLOOR.0 * union;
        let verbatim_hit = verbatim >= codeeraser::docdup::spec::VERBATIM_FLOOR as u64;
        if jaccard_hit || verbatim_hit {
            out.push(json!({
                "a": seg_id(a), "b": seg_id(b),
                "inter": inter, "union": union, "verbatim": verbatim,
            }));
        }
    }
    out
}

fn summarize(segs_live: usize, candidates: usize, pairs: &[Value]) -> Value {
    let floor = codeeraser::docdup::spec::VERBATIM_FLOOR as u64;
    let by_jaccard = pairs
        .iter()
        .filter(|p| {
            p["inter"].as_u64().expect("inter") * JACCARD_UNIVERSE_FLOOR.1
                >= JACCARD_UNIVERSE_FLOOR.0 * p["union"].as_u64().expect("union")
        })
        .count();
    let by_verbatim = pairs
        .iter()
        .filter(|p| p["verbatim"].as_u64().expect("verbatim") >= floor)
        .count();
    json!({
        "segments_live": segs_live,
        "candidate_pairs": candidates,
        "pairs": pairs.len(),
        "by_rule": {"jaccard": by_jaccard, "verbatim": by_verbatim},
    })
}

fn build_doc(name: &Option<String>, tip: &str, segs: &[OracleSeg]) -> Value {
    assert!(
        segs.len() <= DOCDUP_ORACLE_SEGCAP,
        "corpus exceeds oracle_segcap ({} > {}): WITHHELD per F31 — record the \
         reason in the plan, do not downsample",
        segs.len(),
        DOCDUP_ORACLE_SEGCAP
    );
    let candidates = candidate_pairs(segs).len();
    let pairs = oracle_pairs(segs);
    json!({
        "schema": "ce.eval-docdup-oracle/1.0.0",
        "corpus": {"name": name, "tip": tip},
        "constants": oracle_constants(),
        "generated_from": generated_from(),
        "method": ORACLE_METHOD,
        "summary": summarize(segs.len(), candidates, &pairs),
        "pairs": pairs,
    })
}

#[test]
#[ignore] // needs the corpus repository (git show at the pinned tip)
fn generate_docdup_oracle() {
    let (name, doc, _) = universe_gen(
        "docdup-segments",
        |_, _| {},
        |name, tip, walked, _| build_doc(name, tip, &live_segments(walked)),
    );
    write_universe("docdup-oracle", &name, &doc);
}

/// The DOC_SHINGLE k-window (spec.rs provenance): re-run the oracle
/// pair count on the self corpus at k−1/k/k+1 through the ONE
/// production shingle throat; the numbers land in the frozen doc
/// method line.
#[test]
#[ignore]
fn scout_doc_shingle_window() {
    let (name, tip) = graph_corpus();
    assert!(name.is_none(), "the k-window scout runs on the self corpus");
    let (walked, _) = walk_tree(&tip);
    let segs = live_segments(&walked);
    for k in 4..=6 {
        let sets: Vec<Vec<u64>> = segs
            .iter()
            .map(|s| codeeraser::docdup::shingle::shingle_set_k(&s.words, k))
            .collect();
        let mut resized: Vec<OracleSeg> = Vec::new();
        for (s, set) in segs.iter().zip(sets) {
            resized.push(OracleSeg {
                path: s.path.clone(),
                kind: s.kind,
                start_line: s.start_line,
                end_line: s.end_line,
                words: s.words.clone(),
                seq: Vec::new(),
                set,
            });
        }
        let hits = candidate_pairs(&resized)
            .into_iter()
            .filter(|(i, j)| {
                let inter = set_inter(&resized[*i].set, &resized[*j].set);
                let union = resized[*i].set.len() as u64 + resized[*j].set.len() as u64 - inter;
                inter * JACCARD_UNIVERSE_FLOOR.1 >= JACCARD_UNIVERSE_FLOOR.0 * union
            })
            .count();
        println!("k={k}: segments {} jaccard_pairs {hits}", resized.len());
    }
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
