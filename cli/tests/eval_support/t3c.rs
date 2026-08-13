//! t3-candidates shared surface (M5-3c): the frozen constants, the
//! corpus→candidate-pass runner and the pool digest — ONE binding for
//! the candidate instrument (eval_t3_candidates.rs) and the sample
//! instrument (eval_t3_sample.rs), so the frozen pair universe and
//! the sample drawn from it can never disagree about what a pair is.

use codeeraser::dedup::candidates::{self, Candidates};
use serde_json::{Value, json};

/// Frozen constants of the t3-candidates family — the numeric knobs
/// echo the ONE product binding (dedup::candidates) so doc and code
/// cannot disagree. `min_reported_pairs` is the per-corpus T-G14
/// output floor (anti-silence: "report almost nothing" must red),
/// set as HALF the scout run's identical-struct pair count, rounded
/// up: pairs whose two units carry byte-equal structural facts
/// (nodes, shingle set, kind histogram) are near-certain 1.0-TSED
/// clones, and the ×0.5 margin covers the residual gap between
/// equal pre-order facts and equal tree shape — a derived bound
/// with a stated hedge, not a guess (scout 2026-08-13: identical
/// pairs 105/126/35/1234/6506).
pub fn t3c_constants(corpus: &str) -> Value {
    use codeeraser::dedup::candidates as c;
    let floor: u64 = match corpus {
        "self" => 53,
        "cobra" => 63,
        "requests" => 18,
        "ripgrep" => 617,
        "zod" => 3253,
        other => panic!("no t3-candidates floor for {other}"),
    };
    json!({
        "t3_min_nodes": c::T3_MIN_NODES,
        "tsed_num": c::TSED_NUM,
        "tsed_den": c::TSED_DEN,
        "minhash_perms": c::LSH_SHAPE.0,
        "lsh_bands": c::LSH_SHAPE.1,
        "lsh_rows": c::LSH_SHAPE.2,
        "hot_group_cap": c::HOT_GROUP_CAP,
        "min_reported_pairs": floor,
    })
}

/// Materialized pinned tree → product index → candidate pass, with
/// the walk anchored to the frozen t3-universe sibling first (same
/// tip, same path→sha256 inventory).
pub fn corpus_candidates(repo: Option<&str>, name: &Option<String>, tip: &str) -> Candidates {
    let universe = super::load(&super::eval_doc(&super::doc_stem("t3-universe", name)));
    assert_eq!(
        universe["corpus"]["tip"].as_str().expect("tip"),
        tip,
        "candidates must freeze the universe's pinned tip"
    );
    let (walked, _) = super::walk_tree_in(repo, tip);
    let frozen = super::str_pairs(&universe, "files", "path", "sha256");
    assert_eq!(frozen.len(), walked.len(), "inventory size drifted");
    for (p, _, t) in &walked {
        assert_eq!(
            frozen.get(p.as_str()).copied(),
            Some(super::content_sha(t).as_str()),
            "{p}: walked tree differs from the frozen universe inventory"
        );
    }
    let corpus = name.as_deref().unwrap_or("self");
    let root = super::materialize_tree("cand", corpus, &walked);
    let (mut idx, _) = codeeraser::dedup::refreshed_index(&root, None).expect("index");
    let c = candidates::collect(&root, &mut idx).expect("candidate pass");
    // the open connection holds .ce/index.db inside the tree — close
    // it first or Windows refuses the removal
    drop(idx);
    std::fs::remove_dir_all(&root).expect("drop temp tree");
    c
}

/// sha256 over the sorted identity lines of the surviving pairs —
/// the pool anchor: one digest binds the sample to the frozen
/// candidate universe instead of ten thousand doc rows.
pub fn pair_digest(c: &Candidates) -> String {
    let mut lines: Vec<String> = c.pairs.iter().map(|p| pair_line(c, p)).collect();
    lines.sort();
    super::content_sha(&lines.join("\n"))
}

/// One sample-pool row — typed once, so the frozen doc's row shape
/// is a declaration instead of a scatter of json! keys.
#[derive(serde::Serialize)]
pub struct SampleRow<'a> {
    pub corpus: &'a str,
    pub tip: &'a str,
    pub lang: &'a str,
    pub band: &'static str,
    pub source: &'static str,
    pub sources: u8,
    pub a_path: &'a str,
    pub a_key: &'a str,
    pub a_nth: i64,
    pub a_nodes: i64,
    pub b_path: &'a str,
    pub b_key: &'a str,
    pub b_nth: i64,
    pub b_nodes: i64,
}

/// One surviving pair as a sample-pool row Value.
pub fn sample_row(c: &Candidates, p: &candidates::PairRow, corpus: &str, tip: &str) -> Value {
    let (a, b) = (&c.units[p.a], &c.units[p.b]);
    serde_json::to_value(SampleRow {
        corpus,
        tip,
        lang: &a.lang,
        band: super::band_of(a.nodes.max(b.nodes)),
        source: candidates::SOURCES[p.sources.trailing_zeros() as usize],
        sources: p.sources,
        a_path: &a.path,
        a_key: &a.key,
        a_nth: a.nth,
        a_nodes: a.nodes,
        b_path: &b.path,
        b_key: &b.key,
        b_nth: b.nth,
        b_nodes: b.nodes,
    })
    .expect("row")
}

/// The canonical identity line of one surviving pair.
pub fn pair_line(c: &Candidates, p: &candidates::PairRow) -> String {
    let (a, b) = (&c.units[p.a], &c.units[p.b]);
    format!(
        "{}|{}|{}|{}|{}|{}|{}",
        a.path, a.key, a.nth, b.path, b.key, b.nth, p.sources
    )
}
