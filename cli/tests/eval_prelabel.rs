//! M4-2 pre-labels for the 200-sample labeling subset: per-sample
//! line-class counts derived from `git diff --no-index` twice — plain
//! (numstat) and `--color-moved=plain`, whose SGR colors are the only
//! interface git exposes for line-level move detection (31/32 =
//! deleted/added, 35/36 = moved-out/moved-in). Numbers only — the
//! output goes in-repo, the sample contents must not (D2-7).
//!
//! Pre-labels are the HEURISTIC half of the ground truth; the reviewed
//! half (per-item agent review, user delegation 2026-08-10) lands
//! separately in labels-v1.json so anchoring bias stays auditable.
//!
//! Run: CE_EVAL_OUT=<dir> cargo test --test eval_prelabel -- --ignored --nocapture

mod eval_support;

use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Default)]
struct LineClasses {
    added_novel: usize,
    added_moved: usize,
    removed_deleted: usize,
    removed_moved: usize,
}

fn out_dir() -> PathBuf {
    PathBuf::from(std::env::var("CE_EVAL_OUT").unwrap_or_else(|_| "../.ce-eval".into()))
}

fn read_sample(dir: &Path, id: &str) -> Value {
    let path = dir.join("samples").join(format!("{id}.json"));
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("{}: {e} — regenerate via eval_extract", path.display()));
    serde_json::from_str(&text).expect("sample json")
}

/// Split a line into its leading SGR parameter runs and the first
/// payload character. `\x1b[1;36m+` and `\x1b[36m+` must both read as
/// cyan (empirically git's moved defaults are the BOLD variants, and
/// the `---`/`+++` headers are bold with no color at all).
fn sgr_prefix(line: &str) -> (Vec<&str>, Option<char>) {
    let mut rest = line;
    let mut codes = Vec::new();
    while let Some(tail) = rest.strip_prefix("\x1b[") {
        let Some(end) = tail.find('m') else { break };
        codes.extend(tail[..end].split(';'));
        rest = &tail[end + 1..];
    }
    (codes, rest.chars().next())
}

/// Classify one diff body line under --color-moved=plain: 31/32 =
/// plain removed/added, 35/36 = moved out/in (bold or not); anything
/// else (hunk/file headers) is not a body line.
fn classify_line(line: &str, c: &mut LineClasses) {
    let (codes, first) = sgr_prefix(line);
    let has = |code: &str| codes.contains(&code);
    match first {
        Some('-') if has("35") => c.removed_moved += 1,
        Some('-') if has("31") => c.removed_deleted += 1,
        Some('+') if has("36") => c.added_moved += 1,
        Some('+') if has("32") => c.added_novel += 1,
        _ => {}
    }
}

fn color_moved_counts(a: &Path, b: &Path) -> LineClasses {
    // -c color.diff=always because stdout is a pipe; U0 keeps the body
    // to exactly the changed lines so every body line is classifiable
    let out = Command::new("git")
        .args(["-c", "color.diff=always", "diff", "--no-index", "-U0"])
        .args([
            "--color-moved=plain",
            "--color-moved-ws=allow-indentation-change",
        ])
        .arg(a)
        .arg(b)
        .output()
        .expect("git diff --color-moved");
    let text = String::from_utf8_lossy(&out.stdout);
    let mut classes = LineClasses::default();
    for line in text.lines() {
        classify_line(line, &mut classes);
    }
    classes
}

fn prelabel_one(tmp: &Path, id: &str, sample: &Value) -> Value {
    let before = sample["before"].as_str().expect("before");
    let after = sample["after"].as_str().expect("after");
    let a = tmp.join("a");
    let b = tmp.join("b");
    std::fs::write(&a, before).expect("write a");
    std::fs::write(&b, after).expect("write b");
    let (add, del) = eval_support::numstat(&a, &b, &[]);
    let (add, del) = (add as usize, del as usize);
    let c = color_moved_counts(&a, &b);
    // moved lines are still counted by numstat as +/-; the split is
    // the whole point of the color pass
    assert_eq!(add, c.added_novel + c.added_moved, "{id}: added split");
    assert_eq!(
        del,
        c.removed_deleted + c.removed_moved,
        "{id}: removed split"
    );
    json!({
        "id": id,
        "before_lines": before.lines().count(),
        "after_lines": after.lines().count(),
        "numstat_added": add,
        "numstat_deleted": del,
        "added_novel": c.added_novel,
        "added_moved": c.added_moved,
        "removed_deleted": c.removed_deleted,
        "removed_moved": c.removed_moved,
    })
}

#[test]
#[ignore] // needs the local .ce-eval payloads (eval_extract output)
fn prelabel_labeling_subset() {
    let manifest: Value = serde_json::from_str(
        &std::fs::read_to_string("../contracts/eval/manifest-v1.json").expect("manifest"),
    )
    .expect("manifest json");
    let samples_dir = out_dir();
    let tmp = samples_dir.join("tmp-prelabel");
    std::fs::create_dir_all(&tmp).expect("tmp dir");

    let mut rows = Vec::new();
    for row in manifest["samples"].as_array().expect("samples") {
        if row["labeling"].as_bool() != Some(true) {
            continue;
        }
        let id = row["id"].as_str().expect("id");
        let sample = read_sample(&samples_dir, id);
        rows.push(prelabel_one(&tmp, id, &sample));
    }
    std::fs::remove_dir_all(&tmp).expect("cleanup tmp");
    assert_eq!(rows.len(), 200, "labeling subset size");
    rows.sort_by(|a, b| a["id"].as_str().cmp(&b["id"].as_str()));

    let git_version = Command::new("git").arg("--version").output().expect("git");
    let doc = json!({
        "schema": "ce.eval-prelabels/1.0.0",
        "source_manifest": manifest["frozen_at"],
        "git_version": String::from_utf8_lossy(&git_version.stdout).trim(),
        "method": "git diff --no-index -U0 --color-moved=plain \
                   --color-moved-ws=allow-indentation-change; SGR 31/32 = \
                   deleted/novel, 35/36 = moved; numstat cross-checked",
        "prelabels": rows,
    });
    std::fs::write(
        "../contracts/eval/prelabels-v1.json",
        serde_json::to_string_pretty(&doc).expect("ser"),
    )
    .expect("write prelabels");
    println!("prelabeled 200 samples; numstat splits cross-checked");
}
