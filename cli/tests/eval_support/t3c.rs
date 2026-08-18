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

/// One corpus resolved end to end: frozen candidates doc → pinned
/// tip → live candidate pass, digest-anchored before a single pair
/// is consumed. The start line of every 3c/3f consumer (the sample
/// generator and the audit assembly both walked this verbatim until
/// the ratchet paired them).
pub struct Anchored {
    pub corpus: String,
    pub tip: String,
    pub repo: Option<String>,
    pub candidates: Candidates,
}
