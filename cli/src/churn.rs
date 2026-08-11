//! `ce churn` (plan §4.1, M4): time-dimension metrics over the git
//! history window — the behavioural layer the snapshot scanners
//! cannot see, and the third join-signal leg for M5.
//!
//! 1. append vs rewrite, function-level: an added line owned by a
//!    unit that exists on BOTH sides of its commit is REWRITE (an
//!    existing function was edited); a line in a new unit or at top
//!    level is APPEND (new material). A stacking session shows up as
//!    append-heavy growth next to untouched twins.
//! 2. windowed churn (GitClear's signal): of the lines added inside
//!    the window, how many no longer survive at HEAD (blame
//!    author-time), churn = 1 - survival.
//! 3. co-change pairs: files repeatedly changing in the same commits.
//!
//! Report-only in M4 (no thresholds, no gating); numbers feed the
//! M5 three-signal join.

use crate::fourclass::session;
use crate::scan::lang::Lang;
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

/// Commits with more changed files than this are skipped for pair
/// counting (quadratic) and reported, never silently dropped.
const COCHANGE_FILE_CAP: usize = 20;

pub struct Report {
    pub commits: usize,
    pub append_lines: usize,
    pub rewrite_lines: usize,
    pub added_in_window: usize,
    pub surviving: usize,
    pub cochange: Vec<(String, String, usize)>,
    pub skipped_large: usize,
}

pub fn run(root: &Path, days: u32) -> Result<Report> {
    let shas = window_commits(root, days)?;
    let mut report = Report {
        commits: shas.len(),
        append_lines: 0,
        rewrite_lines: 0,
        added_in_window: 0,
        surviving: 0,
        cochange: Vec::new(),
        skipped_large: 0,
    };
    let mut pair_counts: HashMap<(String, String), usize> = HashMap::new();
    let mut touched: Vec<String> = Vec::new();
    for sha in &shas {
        let pairs = session::commit_pairs(root, sha).unwrap_or_default();
        classify_commit(root, sha, &pairs, &mut report);
        count_cochange(&pairs, &mut pair_counts, &mut report.skipped_large);
        for (_, after) in &pairs {
            if let Some(f) = after
                && !touched.contains(f)
            {
                touched.push(f.clone());
            }
        }
    }
    report.surviving = survival(root, days, &touched)?;
    report.cochange = top_pairs(pair_counts);
    Ok(report)
}

fn window_commits(root: &Path, days: u32) -> Result<Vec<String>> {
    let since = format!("{days} days ago");
    let out = git(
        root,
        &[
            "log",
            "--since",
            &since,
            "--first-parent",
            "--no-merges",
            "--format=%H",
        ],
    )?;
    Ok(out.split_whitespace().map(str::to_string).collect())
}

/// Append vs rewrite per pair, plus the window's added-line total.
fn classify_commit(root: &Path, sha: &str, pairs: &[session::PathPair], r: &mut Report) {
    for (before_path, after_path) in pairs {
        let Some(after_path) = after_path.as_deref() else {
            continue; // deletion adds nothing
        };
        let Some(lang) = Lang::from_path(Path::new(after_path)) else {
            continue;
        };
        let before = before_path
            .as_deref()
            .and_then(|p| show(root, sha, p, true))
            .unwrap_or_default();
        let Some(after) = show(root, sha, after_path, false) else {
            continue;
        };
        let c = crate::fourclass::classify(&before, &after, lang);
        let before_units = crate::fourclass::units::segments(&before, lang);
        let after_units = crate::fourclass::units::segments(&after, lang);
        for &l in &c.changed.added {
            r.added_in_window += 1;
            let rewrite = crate::fourclass::units::owner(&after_units, l)
                .is_some_and(|u| before_units.iter().any(|b| b.key == u.key));
            if rewrite {
                r.rewrite_lines += 1;
            } else {
                r.append_lines += 1;
            }
        }
    }
}

fn count_cochange(
    pairs: &[session::PathPair],
    counts: &mut HashMap<(String, String), usize>,
    skipped: &mut usize,
) {
    let files: Vec<&String> = pairs.iter().filter_map(|(_, a)| a.as_ref()).collect();
    if files.len() > COCHANGE_FILE_CAP {
        *skipped += 1; // reported, not silent (no-silent-caps discipline)
        return;
    }
    for i in 0..files.len() {
        for j in (i + 1)..files.len() {
            let (a, b) = (files[i].clone(), files[j].clone());
            let key = if a <= b { (a, b) } else { (b, a) };
            *counts.entry(key).or_default() += 1;
        }
    }
}

/// Lines at HEAD blame-dated inside the window, over the touched set.
fn survival(root: &Path, days: u32, touched: &[String]) -> Result<usize> {
    let cutoff = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .context("clock")?
        .as_secs()
        .saturating_sub(u64::from(days) * 86_400);
    let mut surviving = 0usize;
    for file in touched {
        let Ok(out) = git(root, &["blame", "--line-porcelain", "HEAD", "--", file]) else {
            continue; // deleted since: zero survivors by definition
        };
        surviving += out
            .lines()
            .filter_map(|l| l.strip_prefix("author-time "))
            .filter_map(|t| t.trim().parse::<u64>().ok())
            .filter(|&t| t >= cutoff)
            .count();
    }
    Ok(surviving)
}

fn top_pairs(counts: HashMap<(String, String), usize>) -> Vec<(String, String, usize)> {
    let mut pairs: Vec<(String, String, usize)> = counts
        .into_iter()
        .filter(|(_, n)| *n >= 2)
        .map(|((a, b), n)| (a, b, n))
        .collect();
    pairs.sort_by(|x, y| y.2.cmp(&x.2).then_with(|| (&x.0, &x.1).cmp(&(&y.0, &y.1))));
    pairs.truncate(20);
    pairs
}

fn show(root: &Path, sha: &str, path: &str, parent: bool) -> Option<String> {
    let rev = if parent {
        format!("{sha}^:{path}")
    } else {
        format!("{sha}:{path}")
    };
    git(root, &["show", &rev]).ok()
}

fn git(root: &Path, args: &[&str]) -> Result<String> {
    let out = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .context("git")?;
    if !out.status.success() {
        anyhow::bail!("git {args:?} failed");
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

pub fn report_json(r: &Report) -> serde_json::Value {
    let churned = r.added_in_window.saturating_sub(r.surviving);
    serde_json::json!({
        "schema": "ce.churn-report/0.1.0",
        "commits": r.commits,
        "append_lines": r.append_lines,
        "rewrite_lines": r.rewrite_lines,
        "added_in_window": r.added_in_window,
        "surviving": r.surviving,
        "churned": churned,
        "cochange": r.cochange.iter()
            .map(|(a, b, n)| serde_json::json!({"a": a, "b": b, "count": n}))
            .collect::<Vec<_>>(),
        "skipped_large_commits": r.skipped_large,
    })
}

pub fn print_console(r: &Report, days: u32) {
    let churned = r.added_in_window.saturating_sub(r.surviving);
    println!(
        "churn window {days}d: {} commits, appended {} / rewrote {} lines",
        r.commits, r.append_lines, r.rewrite_lines
    );
    println!(
        "window survival: {} of {} added lines survive at HEAD ({churned} churned)",
        r.surviving, r.added_in_window
    );
    for (a, b, n) in &r.cochange {
        println!("co-change x{n}: {a} <-> {b}");
    }
    if r.skipped_large > 0 {
        println!(
            "note: {} commit(s) above {COCHANGE_FILE_CAP} files skipped for pairing",
            r.skipped_large
        );
    }
}
