//! M4-7 L2 evaluation — THE BAR. Runs the real pipeline
//! (fourclass::batch over a live ce-core link) on every commit of the
//! frozen slice and freezes the gates in commit-l2-v1.json:
//!
//! 1. cross recall — per-file hits cover the reviewed cross GT
//!    (366 out / 181 in), misses = 0 beyond the reviewed below-floor
//!    register (M5-1d: true relocated lines with no >=2-distinct
//!    contiguous companion, itemized per line — the miss-side mirror
//!    of the extras ledger; empty on self and requests);
//! 2. coincidence gate — on each reviewed correction file the
//!    predicted cross count EQUALS the GT count: with per-file misses
//!    at zero, an exact count leaves no room for the excluded line;
//! 3. invention gate — zero cross predictions on commits whose GT has
//!    no cross moves;
//! 4. monotonicity + L1 identity — L2 >= L1 on moved with conserved
//!    sums per pair (asserted at generation), and single-pair batches
//!    reproduce l1-v1.json on all 200 samples (a generation-time leg);
//! 5. extras ledger — every predicted-above-GT file itemized with
//!    line content (GT's blocks-mode floor under-marks sub-block
//!    moves; an allowance in bulk would be a hole, a ledger is not);
//! 6. determinism — reversed pair order yields identical per-file
//!    deltas (asserted during generation);
//! 7. cost-model sensitivity — pinned Haskell-side (Spec.hs
//!    costModel: the floor tracks the site cost).
//!
//! M5-1c: corpus-aware throughout — the gate covers every FROZEN
//! corpus (requests pending: EVAL-SET), the coincidence table derives
//! from each labels doc's corrections (verified set-equal to the old
//! hardcoded self table).
//!
//! Run (the CI gate — no core, no git): cargo test --test eval_l2
//! Regenerate — the `--ignored` generator half retired in 0c7c936
//! (M7.5a); revive it with its coeval support (EVAL-SET.md「再生成」):
//!   git checkout 0c7c936^ -- cli/tests/eval_l2.rs cli/tests/eval_support
//!   CE_CORE_BIN=$(cd core && cabal list-bin ce-core) cargo test --test eval_l2 -- --ignored --nocapture   # CE_SLICE_* retargets the corpus
//!   git checkout HEAD -- cli/tests/eval_l2.rs cli/tests/eval_support

use crate::eval_l2_parts as parts;
use crate::eval_l2_register;
use crate::eval_support;
use crate::eval_support::{by_sha, corpus_doc_pairs, u64s};
use serde_json::Value;
use std::collections::HashMap;

/// Both class splits must sum to the same numstat on each side.
fn assert_conserved(sha: &str, a: &[u64], b: &[u64]) {
    assert_eq!(a[0] + a[1], b[0] + b[1], "{sha}: added conserved");
    assert_eq!(a[2] + a[3], b[2] + b[3], "{sha}: removed conserved");
}

/// CI gate, no git needed, every corpus: summary re-derives; cross
/// GT anchors to the frozen labels totals; misses zero; coincidence
/// files exact; invention zero; per-pair L2 conserves the L1 totals
/// (the judgment invariant — monotone reclassification moves lines
/// between classes, never creates or drops them).
#[test]
fn commit_l2_consistent() {
    let labels_docs: HashMap<_, _> = corpus_doc_pairs("labels").into_iter().collect();
    for (slice_path, doc_path) in eval_support::corpus_doc_pairs_frozen("l2") {
        check_corpus(&labels_docs[&slice_path], &doc_path);
    }
}

fn check_corpus(labels_path: &str, doc_path: &str) {
    // The register checks resolve the review record BY corpus name —
    // through the active corpus they silently checked external docs
    // against the (empty-for-those-shas) self register.
    let (name, labels, doc) = eval_support::gate_docs("l2", labels_path, doc_path);
    let all = doc["commits"].as_array().expect("commits");
    let (tail, rows) = all.split_last().expect("rows");
    let ledger = tail["extras_ledger"].as_array().expect("ledger");
    let s = &doc["summary"];
    assert_eq!(*s, parts::summarize(rows, ledger), "summary drifted");
    assert_eq!(s["cross_misses"], 0, "cross recall gate");
    // Anchors + the below-floor ledger: hits + waived == cross GT.
    let (gt, bf) = eval_support::anchor_to_labels(s, &labels, "below_floor_lines");
    assert_eq!(
        s["cross_hits"].as_u64().unwrap() + bf,
        gt,
        "cross ledger conservation"
    );
    let misses = eval_l2_register::register_misses(name.as_deref(), rows);
    assert!(misses.is_empty(), "relocation register misses: {misses:?}");
    let bad = eval_l2_register::edge_violations(name.as_deref(), rows);
    assert!(bad.is_empty(), "invented relocation edges: {bad:?}");
    check_rows(rows, &labels);
}

/// The labels row's reviewed-correction (file, cross-side) set —
/// corrections record "added"/"removed", cross rows say "in"/"out".
/// Replaces the old hardcoded self-corpus table: same data, one
/// source, every corpus.
fn correction_files(labels_row: &Value) -> Vec<(String, &'static str)> {
    labels_row["corrections"]
        .as_array()
        .expect("corrections")
        .iter()
        .map(|c| {
            let file = c["file"].as_str().expect("file").to_string();
            let side = match c["side"].as_str().expect("side") {
                "added" => "in",
                _ => "out",
            };
            (file, side)
        })
        .collect()
}

fn check_rows(rows: &[Value], labels: &Value) {
    let by = by_sha(labels);
    for r in rows {
        let sha = r["sha"].as_str().expect("sha");
        let lrow = by.get(sha);
        let has_cross = parts::has_cross(lrow);
        let coincidence = lrow.map(|l| correction_files(l)).unwrap_or_default();
        for c in r["cross"].as_array().expect("cross") {
            let (g, p) = parts::gt_pred(c);
            if !has_cross {
                assert_eq!(p, 0, "{sha}: invention on a non-cross commit");
            }
            let hit = coincidence
                .iter()
                .any(|(f, sd)| c["file"] == *f && c["side"] == *sd);
            if hit {
                assert_eq!(p, g, "{sha}: coincidence file must be exact: {c}");
            }
        }
        // Conservation holds between L1 and L2 — the judgment's own
        // invariant. GT-vs-pipeline totals may legally diverge: the
        // GT carrier is git's hunk arithmetic, the pipeline is our
        // minimal Myers, and the two edit scripts differ on prose
        // rewrites (requests 26466b54 README.md: 46/56 vs 41/51,
        // both sides off by the same 5 — the 28d537dd class). The
        // gt column stays as data; pairs_exact tracks its alignment.
        for p in r["pairs"].as_array().expect("pairs") {
            assert_conserved(sha, &u64s(&p["l1"]), &u64s(&p["l2"]));
        }
    }
}
