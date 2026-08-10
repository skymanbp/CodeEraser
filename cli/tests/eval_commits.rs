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
//! Run: cargo test --test eval_commits -- --ignored --nocapture

mod eval_support;

use eval_support::{CLASSES, LineClasses, git_run, load, write_doc};
use serde_json::{Value, json};
use std::collections::HashMap;

/// The L1 commit — the last commit before any L2 work, so the
/// instrument stays outside its own universe.
const UNIVERSE_TIP: &str = "2f40f22b85dcf3fd0979395286223ddf972550ff";
/// Git's canonical empty tree: diff base for the root commit.
const EMPTY_TREE: &str = "4b825dc642cb6eb9a060e54bf8d69288fbee4904";

/// One scoped `git diff <flag> -z -M -C` run → its NUL-separated
/// tokens, trailing empty token dropped (no real token is empty).
fn diff_z(flag: &str, base: &str, sha: &str) -> Vec<String> {
    let raw = git_run(&["diff", flag, "-z", "-M", "-C", base, sha], true);
    raw.split('\0')
        .take_while(|t| !t.is_empty())
        .map(str::to_string)
        .collect()
}

/// (before, after) paths per `--name-status -z` entry; `None` marks
/// the created/deleted side. Copies would break the "before side is
/// consumed" reading of a pair, so they fail loudly (none in scope).
fn name_status(base: &str, sha: &str) -> Vec<(Option<String>, Option<String>)> {
    let mut toks = diff_z("--name-status", base, sha).into_iter();
    let mut pairs = Vec::new();
    while let Some(status) = toks.next() {
        let mut path = || toks.next().expect("path");
        pairs.push(match status.chars().next().expect("status") {
            'A' => (None, Some(path())),
            'D' => (Some(path()), None),
            'M' | 'T' => {
                let p = path();
                (Some(p.clone()), Some(p))
            }
            'R' => (Some(path()), Some(path())),
            s => panic!("{sha}: unsupported status {s}"),
        });
    }
    pairs
}

/// (added, deleted) per `--numstat -z` entry, same order as
/// name_status (same diff, same flags). Rename entries carry their
/// two paths as extra NUL tokens; binary `-` counts fail the parse
/// loudly (in-scope files are text by construction).
fn numstats(base: &str, sha: &str) -> Vec<(u64, u64)> {
    let mut toks = diff_z("--numstat", base, sha).into_iter();
    let mut rows = Vec::new();
    while let Some(entry) = toks.next() {
        let mut f = entry.split('\t');
        let add = f.next().expect("added").parse().expect("numeric added");
        let del = f.next().expect("deleted").parse().expect("numeric deleted");
        if f.next().expect("path field").is_empty() {
            toks.next().expect("old path");
            toks.next().expect("new path");
        }
        rows.push((add, del));
    }
    rows
}

type SectionKey = (Option<String>, Option<String>);

/// Four-class counts of one force-colored whole-commit diff, per
/// (before, after) file section.
fn color_classes(base: &str, sha: &str) -> HashMap<SectionKey, LineClasses> {
    let raw = eval_support::commit_color_diff(base, sha);
    let mut map: HashMap<SectionKey, LineClasses> = HashMap::new();
    for l in eval_support::walk_color_diff(&raw) {
        map.entry((l.a_path, l.b_path))
            .or_default()
            .count(l.added, l.moved);
    }
    map
}

/// One commit → row with per-pair numstat + four-class counts, or
/// `None` when nothing in scope changed. Every pair's class split must
/// conserve its numstat, and every diff section must land on a pair.
fn commit_row(base: &str, sha: &str) -> Option<Value> {
    let names = name_status(base, sha);
    if names.is_empty() {
        return None;
    }
    let stats = numstats(base, sha);
    assert_eq!(names.len(), stats.len(), "{sha}: name-status vs numstat");
    let mut classes = color_classes(base, sha);
    let pairs: Vec<Value> = names
        .into_iter()
        .zip(stats)
        .map(|((before, after), (added, deleted))| {
            let key = (before.clone(), after.clone());
            let c = classes.remove(&key).unwrap_or_default();
            assert_eq!(
                added as usize,
                c.added_novel + c.added_moved,
                "{sha}: {key:?} added"
            );
            let removed = c.removed_deleted + c.removed_moved;
            assert_eq!(deleted as usize, removed, "{sha}: {key:?} deleted");
            json!({"before": before, "after": after,
                   "added": added, "deleted": deleted,
                   "added_novel": c.added_novel, "added_moved": c.added_moved,
                   "removed_deleted": c.removed_deleted,
                   "removed_moved": c.removed_moved})
        })
        .collect();
    assert!(
        classes.is_empty(),
        "{sha}: unmatched sections {:?}",
        classes.keys()
    );
    let subject = git_run(&["log", "-1", "--format=%s", sha], false);
    Some(json!({"sha": sha, "subject": subject.trim(), "pairs": pairs}))
}

/// Walk the linear universe oldest-first; the root commit diffs
/// against the empty tree.
fn commit_rows() -> (Vec<Value>, Vec<Value>) {
    let list = git_run(
        &[
            "rev-list",
            "--first-parent",
            "--no-merges",
            "--reverse",
            UNIVERSE_TIP,
        ],
        false,
    );
    let shas: Vec<&str> = list.split_whitespace().collect();
    let root = git_run(&["rev-list", "--max-parents=0", UNIVERSE_TIP], false);
    assert_eq!(root.trim(), shas[0], "universe root");
    let (mut rows, mut excluded) = (Vec::new(), Vec::new());
    for (i, sha) in shas.iter().enumerate() {
        let parent = format!("{sha}^");
        let base = if i == 0 { EMPTY_TREE } else { parent.as_str() };
        match commit_row(base, sha) {
            Some(row) => rows.push(row),
            None => excluded.push(json!({"sha": sha, "reason": "no in-scope files"})),
        }
    }
    (rows, excluded)
}

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

#[test]
#[ignore] // needs full (non-shallow) git history up to UNIVERSE_TIP
fn generate_commit_slice() {
    let (rows, excluded) = commit_rows();
    let doc = json!({
        "schema": "ce.eval-commit-slice/1.0.0",
        "universe_tip": UNIVERSE_TIP,
        "method": "whole-commit git diff -U0 -M -C --color-moved=blocks \
                   --color-moved-ws=allow-indentation-change over this \
                   repository's own linear history; scope = five supported \
                   languages minus memory/ paths; per-pair splits \
                   numstat-conserved. blocks mode trades sub-block move \
                   recall for immunity to plain mode's trivial-line \
                   cross-file artifact.",
        "summary": summarize(&rows, excluded.len()),
        "excluded": excluded,
        "commits": rows,
    });
    write_doc(
        "../contracts/eval/commit-slice-v1.json",
        &doc,
        "commit slice written to contracts/eval/commit-slice-v1.json",
    );
}

/// CI gate: per-pair numstat conservation and the summary must
/// re-derive from the committed rows. No git or local data needed.
#[test]
fn commit_slice_consistent() {
    let doc = load("../contracts/eval/commit-slice-v1.json");
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
    assert_eq!(doc["summary"], summarize(rows, excluded), "summary drifted");
}
