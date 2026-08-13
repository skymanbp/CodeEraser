//! M5-3c t3-candidates instrument: the frozen CANDIDATE-PAIR universe
//! per corpus (design vol.3 §9.2) — the four-source union with its
//! per-source, per-drop and per-prune tallies, the published S4
//! band-group distribution (F22), the pre-registered per-corpus
//! output floor (T-G14), and the sha256 digest of the surviving pair
//! set (the pool anchor the t3 sample binds to). Frozen before any
//! TED judge exists; the judge inherits this denominator, it never
//! chooses it (RM2).
//!
//! Generate (per corpus; external corpora via CE_SLICE_REPO +
//! CE_GRAPH_NAME + CE_GRAPH_TIP):
//!   cargo test --release --test eval_t3_candidates -- --ignored --nocapture

mod eval_support;

use codeeraser::dedup::candidates::Candidates;
use eval_support::*;
use serde_json::{Value, json};
use std::collections::BTreeMap;

const METHOD: &str = "candidate-pair universe of the pinned tree: the four-source \
    union (S1 verified near-miss runs 25<=len<50 reclaimed from the \
    T1/T2 extension pass; S2 same unit key across files; S3 raw \
    fingerprint co-occurrence, no extension — wider than any \
    reasonable candidate pass on purpose; S4 structural MinHash/LSH, \
    supplementary, its band-group distribution published) over the \
    admitted units (>= 24 named nodes) of the t3-universe sibling, \
    reduced by the two provably admissible prunes (size band, label \
    intersection — both decide 'provably below the 0.85 threshold', \
    so no true clone pair is ever dropped). The candidate pass never \
    consults TSED or TED: this universe freezes before the judge \
    exists, and candidate recall is an upper bound on judged recall. \
    pairs_sha256 fixes the exact surviving pair set; the frozen \
    sample proves membership against it.";

/// This corpus's candidate pass via the shared runner (env-selected
/// repo, the single-corpus generators' convention).
fn candidates_for(name: &Option<String>, tip: &str) -> Candidates {
    let repo = std::env::var("CE_SLICE_REPO").ok();
    corpus_candidates(repo.as_deref(), name, tip)
}

/// Surviving pairs with byte-equal structural facts (nodes, shingle
/// set, histogram) — TED on equal-label equal-shape trees is 0, so
/// these are the machine floor under any honest 0.85 judge; the
/// per-corpus min_reported_pairs is derived from this count.
fn identical_struct_pairs(c: &Candidates) -> u64 {
    c.pairs
        .iter()
        .filter(|p| {
            let (a, b) = (&c.units[p.a], &c.units[p.b]);
            a.nodes == b.nodes && a.sig == b.sig && a.hist == b.hist
        })
        .count() as u64
}

/// Re-derivable summary (G1: the gate re-checks conservation on it).
fn summarize_candidates(c: &Candidates) -> Value {
    let mut admitted_by: BTreeMap<&str, u64> = BTreeMap::new();
    for u in &c.units {
        *admitted_by.entry(&u.lang).or_insert(0) += 1;
    }
    let mut survivors_by: BTreeMap<String, u64> = BTreeMap::new();
    for p in &c.pairs {
        let (a, b) = (&c.units[p.a], &c.units[p.b]);
        let band = band_of(a.nodes.max(b.nodes));
        *survivors_by
            .entry(format!("{}/{band}", a.lang))
            .or_insert(0) += 1;
    }
    let t = &c.tally;
    json!({
        "units_admitted": c.units.len(), "admitted_by_lang": admitted_by,
        "raw_by": t.raw_by, "union_pairs": t.union_pairs,
        "cross_lang_dropped": t.cross_lang_dropped,
        "unowned_dropped": t.unowned_dropped,
        "self_pair_dropped": t.self_pair_dropped,
        "pruned_size": t.pruned_size, "pruned_label": t.pruned_label,
        "survivors": t.survivors, "survivors_by": survivors_by,
        "s3_hot_chained": t.s3_hot_chained, "s4_hot_chained": t.s4_hot_chained,
        "s4_band_groups": t.s4_band_groups,
        "identical_struct_pairs": identical_struct_pairs(c),
    })
}

/// One look at the real numbers BEFORE the per-corpus output floors
/// are written down — the floors are derived, not guessed.
#[test]
#[ignore] // needs the corpus repository
fn scout_t3_candidates() {
    let (name, tip) = graph_corpus();
    let c = candidates_for(&name, &tip);
    println!(
        "SCOUT {}: {}",
        name.as_deref().unwrap_or("self"),
        summarize_candidates(&c)
    );
}

#[test]
#[ignore] // needs the corpus repository (git show at the pinned tip)
fn generate_t3_candidates() {
    let (name, tip) = graph_corpus();
    let corpus = name.as_deref().unwrap_or("self");
    let c = candidates_for(&name, &tip);
    let again = candidates_for(&name, &tip);
    assert_eq!(
        pair_digest(&c),
        pair_digest(&again),
        "double run diverged — the pair set is not a function of the tree"
    );
    let floor = t3c_constants(corpus)["min_reported_pairs"]
        .as_u64()
        .expect("floor");
    assert!(
        floor >= 1 && floor <= identical_struct_pairs(&c),
        "output floor must be a true lower bound of honest yield"
    );
    let doc = json!({
        "schema": "ce.eval-t3-candidates/1.0.0",
        "corpus": {"name": name, "tip": tip},
        "constants": t3c_constants(corpus),
        "generated_from": generated_from(),
        "method": METHOD,
        "summary": summarize_candidates(&c),
        "pairs_sha256": pair_digest(&c),
    });
    let path = eval_doc(&doc_stem("t3-candidates", &name));
    write_doc(&path, &doc, &format!("{path} written"));
}

/// CI gate, no git, every frozen candidate doc: constants single-bound
/// (the per-corpus floor included), internal conservation (union ==
/// pruned + survivors; survivors_by sums back), the floor honest
/// (1 <= floor <= identical_struct_pairs <= survivors), and the doc
/// anchored to its t3-universe sibling — same tip, and per language
/// the admitted units equal the universe's at-or-above-floor bands
/// (s+m+l), so the two frozen denominators can never drift apart.
#[test]
fn t3_candidates_consistent() {
    each_frozen_doc("t3-candidates", |path, doc| {
        let name = doc["corpus"]["name"].as_str().map(str::to_string);
        assert_eq!(name, doc_suffix(path, "t3-candidates"), "{path}: name");
        let corpus = name.as_deref().unwrap_or("self");
        assert_eq!(doc["constants"], t3c_constants(corpus), "{path}: constants");
        let s = &doc["summary"];
        let n = |k: &str| s[k].as_u64().unwrap_or_else(|| panic!("{path}: {k}"));
        assert_eq!(
            n("union_pairs"),
            n("pruned_size") + n("pruned_label") + n("survivors"),
            "{path}: prune stages leak pairs"
        );
        assert_eq!(
            sum_obj(&s["survivors_by"]),
            n("survivors"),
            "{path}: survivors_by leaks"
        );
        let floor = doc["constants"]["min_reported_pairs"]
            .as_u64()
            .expect("floor");
        assert!(
            floor >= 1
                && floor <= n("identical_struct_pairs")
                && n("identical_struct_pairs") <= n("survivors"),
            "{path}: floor is not a true lower bound"
        );
        let universe = load(&eval_doc(&doc_stem("t3-universe", &name)));
        assert_eq!(
            doc["corpus"]["tip"], universe["corpus"]["tip"],
            "{path}: tip differs from the universe sibling"
        );
        let bands = universe["summary"]["bands_by"].as_object().expect("bands");
        for (lang, admitted) in s["admitted_by_lang"].as_object().expect("langs") {
            let above: u64 = ["s", "m", "l"]
                .iter()
                .map(|b| bands.get(&format!("{lang}/{b}")).and_then(Value::as_u64))
                .map(|v| v.unwrap_or(0))
                .sum();
            assert_eq!(
                admitted.as_u64().expect("n"),
                above,
                "{path}: {lang} admitted units drifted from the universe bands"
            );
        }
    });
}
