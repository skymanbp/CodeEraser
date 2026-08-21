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
//!    committer-time), churn = 1 - survival.
//! 3. co-change pairs: files repeatedly changing in the same commits.
//!
//! M5-3h: attribution is a per-unit LEDGER keyed (path, unit key,
//! nth), key "" for top-level lines, and the report's window totals
//! are SUMS over that ledger — conservation by construction, there
//! is no second bookkeeping to drift. The ledger reuses the commit's
//! existing `show` + `units::segments` surfaces and adds ZERO git
//! calls (PERF-BUDGET M5-3h: blame alone already costs 155 s on the
//! self window). Known degradation: nth is taken in each commit's
//! own after-snapshot, so deleting an earlier same-key sibling later
//! in the window shifts nth for its survivors (the member-id caveat,
//! 2026-08-13-m5-3-dedup-algorithms.md §7.2) — recorded, not masked.
//!
//! Recorded basis, beside that caveat: the window has exactly ONE
//! clock on BOTH sides — COMMITTER time. window_commits selects with
//! `git log --since`, which is committer-dated, so survival dates
//! blame lines the same way. The author-time filter it replaced
//! disagreed with its own numerator: a rebased or cherry-picked
//! commit carries an old author date and a fresh committer date, so
//! `--since` counted its lines as window additions while the
//! survivor pass threw every one of them out — churn biased upward
//! by construction, worst on exactly the histories that rebase most.
//!
//! Report-only in M4 (no thresholds, no gating); numbers feed the
//! M5 three-signal join. Report shapes live in report.rs (the 300
//! dogfood gate split).

mod report;
mod survival;

pub use report::{Report, UnitRow, print_console, report_json};

use crate::fourclass::session;
use crate::fourclass::units::{self, Unit};
use crate::scan::lang::Lang;
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::Path;

/// Commits with more changed files than this are skipped for pair
/// counting (quadratic) and reported, never silently dropped.
pub(crate) const COCHANGE_FILE_CAP: usize = 20;

/// (path, unit key, nth) → (appended, rewrote) line counts.
type Ledger = HashMap<(String, String, i64), (usize, usize)>;

pub fn run(root: &Path, days: u32) -> Result<Report> {
    let shas = window_commits(root, days)?;
    let mut ledger = Ledger::new();
    let mut pair_counts: HashMap<(String, String), usize> = HashMap::new();
    let mut skipped_large = 0usize;
    let mut touched: Vec<String> = Vec::new();
    for sha in &shas {
        let pairs = session::commit_pairs(root, sha).unwrap_or_default();
        classify_commit(root, sha, &pairs, &mut ledger);
        count_cochange(&pairs, &mut pair_counts, &mut skipped_large);
        for (_, after) in &pairs {
            if let Some(f) = after
                && !touched.contains(f)
            {
                touched.push(f.clone());
            }
        }
    }
    // the wire floor follows the CONFIGURED knob when one is set
    // (batch-7 slice 12: the hardcoded >= 2 silently withheld
    // count-1 pairs from a core configured to judge them); default 2
    // = the core's cochangeFloor default, so the default table is
    // byte-identical
    let floor = crate::config::Config::load(root)
        .ok()
        .and_then(|c| c.score.cochange_floor)
        .map(|f| f as usize)
        .unwrap_or(2);
    Ok(Report {
        commits: shas.len(),
        units: sorted_rows(ledger),
        surviving: survival::survival(root, days, &touched)?,
        cochange: top_pairs(pair_counts, floor),
        skipped_large,
    })
}

/// One commit's ledger rows through the SAME classify_commit throat
/// run() folds over — the eval instrument's product surface (the
/// 40-commit hand-audited ledger replays exactly this).
pub fn commit_ledger(root: &Path, sha: &str) -> Vec<UnitRow> {
    let pairs = session::commit_pairs(root, sha).unwrap_or_default();
    let mut ledger = Ledger::new();
    classify_commit(root, sha, &pairs, &mut ledger);
    sorted_rows(ledger)
}

fn sorted_rows(ledger: Ledger) -> Vec<UnitRow> {
    let mut rows: Vec<UnitRow> = ledger
        .into_iter()
        .map(|((path, key, nth), (appended, rewrote))| UnitRow {
            path,
            key,
            nth,
            appended,
            rewrote,
        })
        .collect();
    rows.sort_by(|a, b| (&a.path, &a.key, a.nth).cmp(&(&b.path, &b.key, b.nth)));
    rows
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

/// The one before/after fetch for a commit's pair: after path, both
/// texts, the language. classify_commit and the conservation
/// instrument read the same bytes through this, so the two can never
/// diverge on what a commit changed. None = deletion, non-language
/// file, or an unreadable after side.
pub fn pair_texts<'p>(
    root: &Path,
    sha: &str,
    pair: &'p session::PathPair,
) -> Option<(&'p str, String, String, Lang)> {
    let after_path = pair.1.as_deref()?;
    // judged_path: the scan-only arm (plan v2.5) is never churned
    let lang = Lang::judged_path(Path::new(after_path))?;
    let before = pair
        .0
        .as_deref()
        .and_then(|p| show(root, sha, p, true))
        .unwrap_or_default();
    let after = show(root, sha, after_path, false)?;
    Some((after_path, before, after, lang))
}

/// Append vs rewrite per added line, attributed into the ledger by
/// the owning unit of the commit's after-snapshot. nth comes from
/// the same `with_nth` throat the unitsig/symbols caches persist,
/// so a HEAD-side join on (path, key, nth) names the same unit.
fn classify_commit(root: &Path, sha: &str, pairs: &[session::PathPair], ledger: &mut Ledger) {
    for pair in pairs {
        let Some((after_path, before, after, lang)) = pair_texts(root, sha, pair) else {
            continue;
        };
        let c = crate::fourclass::classify(&before, &after, lang);
        let before_units = units::segments(&before, lang);
        let after_units = units::segments(&after, lang);
        let nths = units::with_nth(&after_units);
        for &l in &c.changed.added {
            let owner = units::owner(&after_units, l);
            let rewrite = owner.is_some_and(|u| before_units.iter().any(|b| b.key == u.key));
            let row = ledger.entry(unit_id(after_path, owner, &nths)).or_default();
            if rewrite {
                row.1 += 1;
            } else {
                row.0 += 1;
            }
        }
    }
}

/// Ledger identity of `owner`: pointer-match into the `with_nth`
/// view of the SAME unit slice (both borrow after_units, so a miss
/// is a bug worth a loud stop, not a silent nth 0).
fn unit_id(path: &str, owner: Option<&Unit>, nths: &[(&Unit, i64)]) -> (String, String, i64) {
    let Some(u) = owner else {
        return (path.to_string(), String::new(), 0);
    };
    let nth = nths
        .iter()
        .find(|(w, _)| std::ptr::eq(*w, u))
        .expect("owner and with_nth walk the same slice")
        .1;
    (path.to_string(), u.key.clone(), nth)
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

/// Every pair at or over the floor, count-descending — NO rank cut
/// (batch-7 slice 12: `truncate(20)` ran before the judge and before
/// the relevance filter, so 75% of the judge's evidence budget went
/// to rows the score path discarded anyway and a real hotspot could
/// rank 21st; the honest guard is the core's verdictRowCap, which
/// degrades whole rather than dropping rows). The CONSOLE keeps its
/// own display cut in report.rs — that one is rendering.
fn top_pairs(
    counts: HashMap<(String, String), usize>,
    floor: usize,
) -> Vec<(String, String, usize)> {
    let mut pairs: Vec<(String, String, usize)> = counts
        .into_iter()
        .filter(|(_, n)| *n >= floor)
        .map(|((a, b), n)| (a, b, n))
        .collect();
    pairs.sort_by(|x, y| y.2.cmp(&x.2).then_with(|| (&x.0, &x.1).cmp(&(&y.0, &y.1))));
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

/// This family's face over proc::git_output (pub(crate) since M6
/// S3c: the structure staleness join reuses it). Its stance: the
/// error carries git's own stderr — a bare "failed" cost the trend
/// battery a debugging cycle and would reach users via failed[].
pub(crate) fn git(root: &Path, args: &[&str]) -> Result<String> {
    let out = crate::proc::git_output(root, args).context("git")?;
    if !out.status.success() {
        let err = String::from_utf8_lossy(&out.stderr);
        anyhow::bail!("git {args:?} failed: {}", err.trim());
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}
