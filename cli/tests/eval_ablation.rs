//! M5-1c-ii shadow ablation — the decision instrument behind the L2
//! upgrade (user decisions 2026-08-11, ccm #470): requests L2 broke
//! the invention gate (2 stations / 4 lines on a black/isort commit),
//! and hand measurement showed the menu variants failing to separate
//! there while a content-quality floor did. Before any core change,
//! every candidate is measured OFFLINE on both frozen corpora:
//!
//! - baseline — an exact Rust mirror of the core's judgment, proven
//!   per commit against the live ce-core delta (fidelity assert);
//! - quality / freq / chain / flow — site filters (formalizations in
//!   eval_ablation_parts::variants and EVAL-SET.md);
//! - phase3_edge — the F4 width probe (deletion-side attribution
//!   with an anchored-edge requirement).
//!
//! The self bars (recall 547/547, zero invention) are hard: a variant
//! row that breaks them is disqualified BY the matrix, and the
//! baseline row must keep them by construction. FPR replay is
//! untouched — this instrument never enters the production path.
//!
//! Run: CE_CORE_BIN=$(cd core && cabal list-bin ce-core) \
//!      cargo test --test eval_ablation -- --ignored --nocapture

mod eval_ablation_parts;
mod eval_commit_review;
mod eval_l2_parts;
mod eval_support;

use eval_ablation_parts::ledgers;
use eval_ablation_parts::variants::{self};
use eval_support::{corpus_doc_pairs, eval_doc, load, u64s};
use serde_json::Value;
use std::collections::HashMap;

/// CI gate, no git needed, every corpus (hardened per Codex review
/// F2 — a resummed doc is not integrity): row shas must be unique
/// members of the frozen slice; every variant must CONSERVE the
/// labels-anchored GT total (hits + misses == cross GT, so a zeroed
/// or deleted row breaks the sum against an EXTERNAL anchor); the
/// baseline column keeps the L2 bars and the quality column keeps
/// the frozen verdict (zero misses, zero invention, every corpus).
#[test]
fn commit_ablation_consistent() {
    let labels_docs: HashMap<_, _> = corpus_doc_pairs("labels").into_iter().collect();
    for (slice_path, doc_path) in corpus_doc_pairs("ablation") {
        check_corpus(&slice_path, &labels_docs[&slice_path], &doc_path);
    }
}

fn check_corpus(slice_path: &str, labels_path: &str, doc_path: &str) {
    let (doc, slice, labels) = (load(doc_path), load(slice_path), load(labels_path));
    let all = doc["commits"].as_array().expect("commits");
    let (tail, rows) = all.split_last().expect("rows");
    let s = &doc["summary"];
    let n = slice["commits"].as_array().expect("slice").len() as u64;
    assert_eq!(s["commits"].as_u64(), Some(n), "commit coverage");
    assert_eq!(
        s["equivalence_commits"], s["commits"],
        "fidelity must cover every commit"
    );
    let (_, bf) = eval_support::anchor_to_labels(s, &labels, "below_floor");
    check_rows_membership(&slice, rows);
    check_sums(s, rows, bf);
    check_ledgers(s, tail);
    let base = u64s(&s["variants"]["baseline"]);
    assert_eq!(base[1], 0, "baseline cross misses");
    assert_eq!(base[2], 0, "baseline identity misses");
    if doc_path == eval_doc("commit-ablation") {
        assert_eq!(base[3], 0, "self corpus: baseline invention");
    }
    let quality = u64s(&s["variants"]["quality"]);
    assert_eq!(&quality[1..4], [0, 0, 0], "quality verdict pins");
}

fn check_rows_membership(slice: &Value, rows: &[Value]) {
    let shas: std::collections::HashSet<&str> = slice["commits"]
        .as_array()
        .expect("slice")
        .iter()
        .map(|c| c["sha"].as_str().expect("sha"))
        .collect();
    let mut seen = std::collections::HashSet::new();
    for r in rows {
        let sha = r["sha"].as_str().expect("row sha");
        assert!(shas.contains(sha), "{sha}: row outside the slice");
        assert!(seen.insert(sha), "{sha}: duplicate row");
    }
}

fn check_sums(s: &Value, rows: &[Value], below_floor: u64) {
    let gt = s["cross_gt_out"].as_u64().unwrap() + s["cross_gt_in"].as_u64().unwrap();
    for (name, _) in variants::ALL {
        let mut sum = vec![0u64; 8];
        for r in rows {
            for (i, v) in u64s(&r["variants"][name]).iter().enumerate() {
                sum[i] += v;
            }
        }
        assert_eq!(sum, u64s(&s["variants"][name]), "{name}: summary drifted");
        // reviewed below-floor lines leave every variant's ledger
        assert_eq!(
            sum[0] + sum[1],
            gt - below_floor,
            "{name}: GT total not conserved"
        );
    }
}

fn check_ledgers(s: &Value, tail: &Value) {
    let kills = tail["kill_ledger"].as_array().expect("kill ledger");
    assert_eq!(
        s["quality_kills"].as_u64(),
        Some(kills.len() as u64),
        "kill count"
    );
    let width = tail["width_ledger"].as_array().expect("width ledger");
    assert_eq!(
        s["phase3_width"],
        ledgers::width_summary(width),
        "width summary"
    );
}
