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

use eval_support::{CLASSES, Corpus, LineClasses, corpus, git_run, load, write_doc};
use serde_json::{Value, json};
use std::collections::HashMap;

/// Git's canonical empty tree: diff base for the root commit.
const EMPTY_TREE: &str = "4b825dc642cb6eb9a060e54bf8d69288fbee4904";

/// (before, after) paths per `--name-status -z` entry (NUL tokens,
/// trailing empty one dropped); `None` marks the created/deleted
/// side. Copies would break the "before side is consumed" reading of
/// a pair, so they fail loudly — a deliberate stop forcing the
/// copy-semantics decision (none in any corpus yet).
fn name_status(base: &str, sha: &str) -> Vec<(Option<String>, Option<String>)> {
    let raw = git_run(
        &["diff", "--name-status", "-z", "-M", "-C", base, sha],
        true,
    );
    let toks: Vec<&str> = raw.split('\0').take_while(|t| !t.is_empty()).collect();
    let own = |t: Option<&&str>| Some((*t.expect("path")).to_string());
    let (mut pairs, mut i) = (Vec::new(), 0);
    while i < toks.len() {
        let (one, two) = (toks.get(i + 1), toks.get(i + 2));
        let (pair, arity) = match toks[i].chars().next().expect("status") {
            'A' => ((None, own(one)), 2),
            'D' => ((own(one), None), 2),
            'M' | 'T' => ((own(one), own(one)), 2),
            'R' => ((own(one), own(two)), 3),
            s => panic!("{sha}: unsupported status {s}"),
        };
        pairs.push(pair);
        i += arity;
    }
    pairs
}

/// (added, deleted) per plain `-U0` diff section, keyed like the
/// color walk — hunk-header arithmetic over the SAME edit script the
/// bodies come from. numstat is out: myers overcounts against its own
/// patch (requests 28d537dd, 15/6 vs 14/5; why in hunk_totals).
fn hunk_stats(base: &str, sha: &str) -> HashMap<SectionKey, (u64, u64)> {
    let raw = git_run(&["diff", "-U0", "-M", "-C", base, sha], true);
    let mut map = HashMap::new();
    for (a, b, del, add) in eval_support::hunk_totals(&raw) {
        let dup = map.insert((a, b), (add, del));
        assert!(dup.is_none(), "{sha}: duplicate diff section");
    }
    map
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

/// One commit → row with per-pair added/deleted + four-class counts,
/// or `None` when nothing in scope changed. Every pair's class split
/// must conserve its hunk totals; every section must land on a pair.
fn commit_row(base: &str, sha: &str) -> Option<Value> {
    let names = name_status(base, sha);
    if names.is_empty() {
        return None;
    }
    let mut stats = hunk_stats(base, sha);
    let mut classes = color_classes(base, sha);
    let pairs: Vec<Value> = names
        .into_iter()
        .map(|(before, after)| {
            let key = (before, after);
            let (added, deleted) = stats.remove(&key).unwrap_or((0, 0));
            let c = classes.remove(&key).unwrap_or_default();
            assert_eq!(
                added as usize,
                c.added_novel + c.added_moved,
                "{sha}: {key:?} added"
            );
            let removed = c.removed_deleted + c.removed_moved;
            assert_eq!(deleted as usize, removed, "{sha}: {key:?} deleted");
            let (before, after) = key;
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
    assert!(
        stats.is_empty(),
        "{sha}: unmatched hunk sections {:?}",
        stats.keys()
    );
    let subject = git_run(&["log", "-1", "--format=%s", sha], false);
    let mut row = json!({"sha": sha, "subject": subject.trim(), "pairs": pairs});
    // Merge rows are marked so the GT review can stratify them — an
    // aggregate merge diff can pair lines across constituent commits.
    // Absent on non-merges (frozen self slice: zero merges, unchanged).
    let parents = git_run(&["rev-list", "--parents", "-n", "1", sha], false);
    let n = parents.split_whitespace().count().saturating_sub(1);
    if n > 1 {
        row["parents"] = json!(n);
    }
    Some(row)
}

/// Walk the corpus window oldest-first along the first-parent chain;
/// a from-root walk diffs the root commit against the empty tree.
fn commit_rows(c: &Corpus) -> (Vec<Value>, Vec<Value>) {
    let range = match &c.base {
        Some(base) => format!("{base}..{}", c.tip),
        None => c.tip.clone(),
    };
    let list = git_run(&["rev-list", "--first-parent", "--reverse", &range], false);
    let shas: Vec<&str> = list.split_whitespace().collect();
    assert!(!shas.is_empty(), "empty window {range}");
    match &c.base {
        None => {
            // A shallow boundary masquerades as a root and truncated
            // history would silently regenerate the frozen self slice.
            let shallow = git_run(&["rev-parse", "--is-shallow-repository"], false);
            assert_eq!(shallow.trim(), "false", "self slice needs full history");
            let root = git_run(&["rev-list", "--max-parents=0", &c.tip], false);
            assert_eq!(root.trim(), shas[0], "universe root");
        }
        Some(base) => {
            // The base must be the oldest window commit's FIRST parent
            // — a merge's second parent is a valid rev that silently
            // shifts the walk (reproduced: requests 1b40fdd's ^2).
            let parent = git_run(&["rev-parse", &format!("{}^", shas[0])], false);
            assert_eq!(parent.trim(), base, "base is not the first-parent boundary");
        }
    }
    let (mut rows, mut excluded) = (Vec::new(), Vec::new());
    for (i, sha) in shas.iter().enumerate() {
        let parent = format!("{sha}^");
        let base = if i == 0 && c.base.is_none() {
            EMPTY_TREE
        } else {
            parent.as_str()
        };
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

/// The frozen shared method tail; the head names the corpus walk.
/// The self-corpus wording is part of the frozen M4-5 doc.
fn method(c: &Corpus) -> String {
    let walk = match &c.name {
        None => "this repository's own linear history".into(),
        Some(name) => format!(
            "the {name} corpus first-parent window {}..{} (a merge diffs \
             against its first parent = the mainline increment)",
            c.base.as_deref().expect("windowed corpus"),
            c.tip
        ),
    };
    // "numstat" stays in the frozen self wording (equal to the hunk
    // totals there — proven by byte-identical regen); external docs
    // name the actual source, since numstat can diverge (28d537dd).
    let conserved = if c.name.is_some() {
        "hunk-header"
    } else {
        "numstat"
    };
    format!(
        "whole-commit git diff -U0 -M -C --color-moved=blocks \
         --color-moved-ws=allow-indentation-change over {walk}; scope = \
         five supported languages minus memory/ paths; per-pair splits \
         {conserved}-conserved. blocks mode trades sub-block move recall \
         for immunity to plain mode's trivial-line cross-file artifact."
    )
}

#[test]
#[ignore] // needs git history: full for the self slice, the pinned window otherwise
fn generate_commit_slice() {
    let c = corpus();
    let (rows, excluded) = commit_rows(&c);
    let mut doc = json!({
        "schema": "ce.eval-commit-slice/1.0.0",
        "universe_tip": c.tip,
        "method": method(&c),
        "summary": summarize(&rows, excluded.len()),
        "excluded": excluded,
        "commits": rows,
    });
    if let Some(name) = &c.name {
        doc["corpus"] = json!(name);
        doc["window_base"] = json!(c.base);
    }
    let path = c.doc_path();
    write_doc(&path, &doc, &format!("commit slice written to {path}"));
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
    let mut checked = 0;
    for entry in std::fs::read_dir("../contracts/eval").expect("eval dir") {
        let file = entry.expect("entry").file_name();
        let file = file.to_string_lossy();
        if file.starts_with("commit-slice") && file.ends_with("-v1.json") {
            check_slice(&format!("../contracts/eval/{file}"));
            checked += 1;
        }
    }
    assert!(checked >= 1, "no committed slice docs");
}
