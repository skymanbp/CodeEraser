//! M5-3c t3-candidates instrument: the frozen CANDIDATE-PAIR universe
//! per corpus (design vol.3 §9.2) — the four-source union with its
//! per-source, per-drop and per-prune tallies, the published S4
//! band-group distribution (F22), the pre-registered per-corpus
//! output floor (T-G14), and the sha256 digest of the surviving pair
//! set (the pool anchor the t3 sample binds to). Frozen before any
//! TED judge exists; the judge inherits this denominator, it never
//! chooses it (RM2).
//!
//! Regenerate — the `--ignored` generator half retired in 0c7c936
//! (M7.5a); revive it with its coeval support (EVAL-SET.md「再生成」):
//!   git checkout 0c7c936^ -- cli/tests/eval_t3_candidates.rs cli/tests/eval_support
//!   cargo test --release --test eval_t3_candidates -- --ignored --nocapture   # per corpus: CE_SLICE_REPO + CE_GRAPH_NAME + CE_GRAPH_TIP
//!   git checkout HEAD -- cli/tests/eval_t3_candidates.rs cli/tests/eval_support

mod eval_support;

use eval_support::*;
use serde_json::Value;

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
