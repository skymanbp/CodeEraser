//! FPR replay harness (M3 acceptance gate: ≤1 false intercept per
//! 500 real normal edits). Replays a git repo's linear history as
//! edit events: each changed code file in each commit becomes a probe
//! of the CHILD content against the PARENT-state index (probe before
//! apply), then the shadow tree + index advance. Real history = real
//! edit distribution; intercepts are listed for arbitration — a real
//! commit that genuinely introduced cross-file duplication is a TRUE
//! positive and goes into the arbitrated allowlist, not the FPR.
//!
//! #[ignore]: needs git history. Run:
//!   cargo test --release --test fpr_replay -- --ignored --nocapture
//! Optionally CE_FPR_REPO=<path> to replay another repository.

mod common;

use codeeraser::dedup::{Params, index::Index, pairs, probe};
use codeeraser::scan::lang::Lang;
use common::git_out;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

fn replay_root() -> PathBuf {
    std::env::var("CE_FPR_REPO")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .expect("cli/ has a parent")
                .to_path_buf()
        })
}

fn git_lines(repo: &Path, args: &[&str]) -> Vec<String> {
    String::from_utf8_lossy(&git_out(repo, args))
        .lines()
        .map(str::to_string)
        .collect()
}

fn is_code(path: &str) -> bool {
    Lang::from_path(Path::new(path))
        .and_then(|l| l.grammar())
        .is_some()
}

/// Materialize one blob into the shadow tree and the index.
fn apply(shadow: &Path, idx: &mut Index, repo: &Path, commit: &str, rel: &str, p: Params) {
    let bytes = git_out(repo, &["show", &format!("{commit}:{rel}")]);
    let dest = shadow.join(rel);
    if let Some(dir) = dest.parent() {
        std::fs::create_dir_all(dir).expect("mkdir");
    }
    std::fs::write(&dest, &bytes).expect("write shadow");
    idx.refresh_file(
        rel,
        &bytes,
        Lang::from_path(Path::new(rel)).expect("lang"),
        p,
    )
    .expect("refresh");
}

/// Returns the live set — remove_missing later must SEE the seeds,
/// or the first reap would wipe the whole baseline index.
fn seed_first_commit(
    shadow: &Path,
    idx: &mut Index,
    repo: &Path,
    first: &str,
    p: Params,
) -> BTreeSet<String> {
    let mut live = BTreeSet::new();
    for rel in git_lines(repo, &["ls-tree", "-r", "--name-only", first]) {
        if is_code(&rel) {
            apply(shadow, idx, repo, first, &rel, p);
            live.insert(rel);
        }
    }
    live
}

/// (status, path) pairs for code files changed by `commit`.
fn changed_code(repo: &Path, parent: &str, commit: &str) -> Vec<(char, String)> {
    git_lines(
        repo,
        &["diff", "--name-status", "--no-renames", parent, commit],
    )
    .into_iter()
    .filter_map(|l| {
        let mut cols = l.split('\t');
        let status = cols.next()?.chars().next()?;
        let path = cols.next()?.to_string();
        (is_code(&path) && matches!(status, 'A' | 'M' | 'D')).then_some((status, path))
    })
    .collect()
}

#[test]
#[ignore = "FPR acceptance harness: replays git history, run manually"]
fn replay_history_counts_intercepts() {
    let repo = replay_root();
    let shadow = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("fpr-shadow");
    let _ = std::fs::remove_dir_all(&shadow);
    std::fs::create_dir_all(&shadow).expect("mkdir");
    let p = Params::default();
    let f = pairs::Filter {
        min_tokens: p.guarantee(),
        min_distinct: pairs::DEFAULT_MIN_DISTINCT,
    };
    let mut idx = Index::open(&shadow.join(".ce/index.db"), p).expect("open");
    let commits = git_lines(&repo, &["rev-list", "--reverse", "--first-parent", "HEAD"]);
    assert!(commits.len() >= 2, "need history to replay");
    let mut live = seed_first_commit(&shadow, &mut idx, &repo, &commits[0], p);
    let (mut events, mut intercepts) = (0usize, Vec::new());
    for pair in commits.windows(2) {
        let (parent, commit) = (&pair[0], &pair[1]);
        for (status, rel) in changed_code(&repo, parent, commit) {
            if status == 'D' {
                live.remove(&rel);
                let _ = std::fs::remove_file(shadow.join(&rel));
                continue;
            }
            let bytes = git_out(&repo, &["show", &format!("{commit}:{rel}")]);
            events += 1;
            let target = probe::Target {
                rel: &rel,
                content: &bytes,
                lang: Lang::from_path(Path::new(&rel)).expect("lang"),
            };
            let m = probe::probe(&idx, &shadow, target, p, f).expect("probe");
            if !m.is_empty() {
                intercepts.push(format!(
                    "{} {} -> {} ({} tok)",
                    &commit[..8],
                    rel,
                    m[0].file,
                    m[0].tokens
                ));
            }
            apply(&shadow, &mut idx, &repo, commit, &rel, p);
            live.insert(rel);
        }
        idx.remove_missing(&live).ok();
    }
    println!(
        "replayed {events} edit events, {} intercepts:",
        intercepts.len()
    );
    for i in &intercepts {
        println!("  INTERCEPT {i}");
    }
    let per_500 = intercepts.len() as f64 * 500.0 / events.max(1) as f64;
    println!("rate: {per_500:.2} intercepts per 500 events (gate: <=1 after arbitration)");
}
