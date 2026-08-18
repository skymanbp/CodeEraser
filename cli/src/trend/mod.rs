//! `ce trend` (M7-P4): the check score's trajectory over mainline
//! git history. Charter ruling ②: HISTORY IS THE SOURCE OF TRUTH —
//! the SQLite rows are a cache, rebuildable from commits at will
//! (trend_rebuild.rs pins exactly that). The rows live inside
//! `.ce/index.db`'s one wipe lifecycle, so a measurement-rev bump
//! wipes them WITH the fingerprints: points measured under different
//! toolchain revs are not comparable, and the shared wipe IS the
//! comparability contract, not a cost.
//!
//! Each point = `score::run` at that commit in a detached temp
//! worktree — NULL baseline (absolute score, no ratchet noise), no
//! churn window (blame costs minutes), and the tree's OWN ce.toml
//! knobs (a historical tree is judged as it declared itself then;
//! only the measuring toolchain is today's).

pub mod judge;

use crate::score;
use crate::{churn, dedup};
use anyhow::{Context, Result};
use serde::Serialize;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// JSON output schema id; bump on shape change (plan §7.1).
/// 0.2.0 (M7.5b): the report carries the core's trend/1 judgment.
pub const SCHEMA_ID: &str = "ce.trend-report/0.2.0";

/// DDL executed inside the index's ONE schema batch (dedup::schema)
/// — created and wiped with the cache, never on a second lifecycle.
pub const TREND_SCHEMA: &str = "
CREATE TABLE trend (
  commit_hash TEXT PRIMARY KEY,
  ts INTEGER NOT NULL,
  score INTEGER NOT NULL,
  scale INTEGER NOT NULL,
  axes TEXT NOT NULL
);
";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Row {
    pub commit: String,
    pub ts: i64,
    pub score: i64,
    pub scale: i64,
    pub axes: Vec<[i64; 2]>,
}

#[derive(Debug)]
pub struct Report {
    /// Mainline commits found inside the requested window.
    pub window: usize,
    /// Measured rows, oldest first (chart order).
    pub rows: Vec<Row>,
    /// Window commits still unmeasured after this batch (includes
    /// this run's failures — they retry next run).
    pub pending: usize,
    /// (short sha, reason) for commits that refused to measure this
    /// run — reported, never silently absent.
    pub failed: Vec<(String, String)>,
    /// The core's trend/1 slope verdict over the window (M7.5b).
    pub judgment: judge::Judgment,
}

/// Measure up to `batch` (None = all) uncached mainline commits of
/// the newest `commits`, persist them, hand the window to the core's
/// trend/1 judgment, and return the report.
pub fn run(
    root: &Path,
    db: Option<PathBuf>,
    core: &str,
    commits: usize,
    batch: Option<usize>,
) -> Result<Report> {
    let shas = mainline(root, commits)?;
    let idx = dedup::index::Index::open(&dedup::index_db_path(root, db), dedup::Params::default())?;
    let mut have = cached(idx.raw())?;
    let missing: Vec<&(String, i64)> = shas.iter().filter(|(s, _)| !have.contains_key(s)).collect();
    let mut failed = Vec::new();
    let mut measured = 0usize;
    for (sha, ts) in missing.iter().take(batch.unwrap_or(usize::MAX)) {
        match measure(root, core, sha, *ts) {
            Ok(row) => {
                put(idx.raw(), &row)?;
                have.insert(sha.clone(), row);
                measured += 1;
            }
            Err(e) => failed.push((sha[..12].to_string(), format!("{e:#}"))),
        }
    }
    let mut rows: Vec<Row> = shas.iter().filter_map(|(s, _)| have.remove(s)).collect();
    rows.reverse(); // git log is newest-first; charts read oldest-first
    let judgment = judge::judge(root, core, &rows)?;
    Ok(Report {
        window: shas.len(),
        pending: missing.len() - measured,
        rows,
        failed,
        judgment,
    })
}

/// Newest `n` first-parent commits of HEAD: (full sha, author time).
fn mainline(root: &Path, n: usize) -> Result<Vec<(String, i64)>> {
    let out = churn::git(
        root,
        &[
            "log",
            "--first-parent",
            "-n",
            &n.to_string(),
            "--format=%H %ct",
        ],
    )?;
    out.lines()
        .map(|l| {
            let (sha, ts) = l.split_once(' ').context("git log line")?;
            Ok((sha.to_string(), ts.trim().parse::<i64>()?))
        })
        .collect()
}

/// One commit's absolute score, judged in a detached temp worktree.
fn measure(root: &Path, core: &str, sha: &str, ts: i64) -> Result<Row> {
    let wt = Worktree::add(root, sha)?;
    let out = score::run(
        &wt.path,
        score::Opts {
            db: None, // the worktree's own .ce — removed with it
            core: core.to_string(),
            days: None,
            floor: None,
            establish: true, // NULL baseline: absolute score
        },
    )?;
    let r = &out.reply;
    if let Some(reason) = &r.degraded {
        anyhow::bail!("degraded: {reason}");
    }
    Ok(Row {
        commit: sha.to_string(),
        ts,
        score: r.score,
        // the effective scale, never a /1000 literal (C17)
        scale: r.knobs.get("scoreScale").copied().unwrap_or(1000),
        axes: r.axes.clone(),
    })
}

/// A detached worktree that tears itself down. The name (which also
/// names git's worktree metadata dir) is sha+pid+SEQ-unique: two
/// threads of one process measuring the same sha — or two test repos
/// whose seeded commits hash identically — must not race one path,
/// and a crash-leaked dir stays `git worktree prune`-able.
struct Worktree {
    root: PathBuf,
    path: PathBuf,
}

static WT_SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

impl Worktree {
    fn add(root: &Path, sha: &str) -> Result<Self> {
        let seq = WT_SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "ce-trend-{}-{}-{seq}",
            &sha[..12],
            std::process::id()
        ));
        let p = path.to_str().context("worktree path not utf8")?;
        churn::git(root, &["worktree", "add", "--detach", p, sha])?;
        Ok(Self {
            root: root.to_path_buf(),
            path,
        })
    }
}

impl Drop for Worktree {
    fn drop(&mut self) {
        // best-effort teardown (Drop cannot propagate): a leaker is
        // pid-unique and prune-able, so swallowing here loses nothing
        let _ = churn::git(
            &self.root,
            &[
                "worktree",
                "remove",
                "--force",
                &self.path.to_string_lossy(),
            ],
        );
    }
}

fn cached(conn: &rusqlite::Connection) -> Result<HashMap<String, Row>> {
    let mut stmt = conn.prepare("SELECT commit_hash, ts, score, scale, axes FROM trend")?;
    let rows = stmt.query_map([], |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, i64>(1)?,
            r.get::<_, i64>(2)?,
            r.get::<_, i64>(3)?,
            r.get::<_, String>(4)?,
        ))
    })?;
    let mut map = HashMap::new();
    for row in rows {
        let (commit, ts, score, scale, axes) = row?;
        let axes: Vec<[i64; 2]> = serde_json::from_str(&axes).context("trend axes json")?;
        map.insert(
            commit.clone(),
            Row {
                commit,
                ts,
                score,
                scale,
                axes,
            },
        );
    }
    Ok(map)
}

/// Idempotent insert (ADR-003 v1.7): the key is an immutable commit
/// hash and the value is deterministic under the db's cache key —
/// rev changes wipe the table — so IGNORE converges, never clobbers.
fn put(conn: &rusqlite::Connection, r: &Row) -> Result<()> {
    conn.execute(
        "INSERT OR IGNORE INTO trend (commit_hash, ts, score, scale, axes)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        (
            &r.commit,
            r.ts,
            r.score,
            r.scale,
            serde_json::to_string(&r.axes)?,
        ),
    )?;
    Ok(())
}

pub fn report_json(r: &Report) -> Value {
    json!({
        "schema": SCHEMA_ID,
        "window": r.window,
        "pending": r.pending,
        "rows": r.rows,
        "failed": r.failed.iter().map(|(s, w)| json!([s, w])).collect::<Vec<_>>(),
        "judgment": judge::judgment_json(&r.judgment),
    })
}

pub fn print(r: &Report, as_json: bool) {
    crate::report::print_doc(
        as_json,
        || report_json(r),
        || {
            for row in &r.rows {
                let axes: Vec<String> = row.axes.iter().map(|[c, p]| format!("{c}:{p}")).collect();
                println!(
                    "trend {} {} score {}/{} | axes {}",
                    &row.commit[..12],
                    row.ts,
                    row.score,
                    row.scale,
                    axes.join(" ")
                );
            }
            for (sha, why) in &r.failed {
                println!("trend {sha} FAILED: {why}");
            }
            let j = &r.judgment;
            println!(
                "trend verdict: {}{}{}",
                judge::verdict_str(j),
                j.slope_micro_per_day
                    .map(|s| format!(" (slope {s} micro-permille/day)"))
                    .unwrap_or_default(),
                if j.fail { " -> FAIL" } else { "" }
            );
            println!(
                "trend window: {} commits, {} measured, {} pending",
                r.window,
                r.rows.len(),
                r.pending
            );
        },
    );
}
