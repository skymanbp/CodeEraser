//! M4-5 pre-registered whole-commit slice: the L2 increment instrument.
//! The 200-sample slice is saturated at line-level moved GT (user
//! ruling 2026-08-10), so L2 must show its increment on the dimensions
//! per-file-pair L1 is structurally blind to: a function leaving one
//! file and landing in another is deleted+novel to L1, moved to L2.
//!
//! Sample = one commit of this repository's own history (real edits,
//! nothing fabricated). File pairing by `git -M -C` — a pure rename
//! is explained by the pairing, not counted as moved lines. Line
//! classes prelabeled by `--color-moved=blocks`: git's >=20-alnum
//! block floor kills the trivial-line cross-file artifact `plain`
//! mode invents at commit granularity (153 fake moved-in on 2f40f22),
//! at the cost of missing sub-block moves — a GT recall bound the
//! per-item review must weigh. Scope = the five supported languages,
//! excluding `memory/` paths (machine-local state that already left
//! version control, and the M7 filter-repo surface — D2-7). The M7
//! history rewrite renames every sha, so the slice is regenerated
//! then: deterministic, diffable modulo sha renames.
//!
//! M5-1: CE_SLICE_REPO (+ NAME/TIP/BASE) retargets the instrument at
//! an external corpus window (eval_support::corpus). The walk follows
//! the first-parent chain including merges (a merge diffs against its
//! first parent = the mainline increment; this repo's history has none).
//!
//! Run: cargo test --test eval_commits -- --ignored --nocapture

mod eval_support;

use eval_support::{CLASSES, load};
use serde_json::{Value, json};
use std::collections::HashMap;

/// Re-derivable from the rows alone, so the CI gate needs no git.
fn summarize(rows: &[Value], excluded: usize) -> Value {
    let mut totals = [0u64; 4];
    let mut moved_bearing = 0u64;
    for r in rows {
        let mut moved = 0;
        for p in r["pairs"].as_array().expect("pairs") {
            for (i, class) in CLASSES.iter().enumerate() {
                totals[i] += p[*class].as_u64().unwrap();
            }
            moved += p["added_moved"].as_u64().unwrap() + p["removed_moved"].as_u64().unwrap();
        }
        if moved > 0 {
            moved_bearing += 1;
        }
    }
    json!({
        "commits": rows.len(),
        "commits_excluded": excluded,
        "moved_bearing": moved_bearing,
        "totals": CLASSES.iter().zip(totals).collect::<HashMap<_, _>>(),
    })
}

fn check_slice(path: &str) {
    let doc = load(path);
    let rows = doc["commits"].as_array().expect("commits");
    for r in rows {
        for p in r["pairs"].as_array().expect("pairs") {
            let g = |k: &str| p[k].as_u64().unwrap();
            let sha = r["sha"].as_str().expect("sha");
            assert_eq!(
                g("added"),
                g("added_novel") + g("added_moved"),
                "{sha}: added"
            );
            let removed = g("removed_deleted") + g("removed_moved");
            assert_eq!(g("deleted"), removed, "{sha}: deleted");
        }
    }
    let excluded = doc["excluded"].as_array().expect("excluded").len();
    let derived = summarize(rows, excluded);
    assert_eq!(doc["summary"], derived, "{path}: summary drifted");
}

/// CI gate: per-pair numstat conservation and the summary must
/// re-derive from the committed rows — for every committed slice doc
/// (self and external corpora). No git or local data needed.
#[test]
fn commit_slice_consistent() {
    let docs = eval_support::frozen_docs("commit-slice");
    assert!(!docs.is_empty(), "no committed slice docs");
    for path in &docs {
        check_slice(path);
    }
}
