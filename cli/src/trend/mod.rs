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
mod report;

pub use report::{print, report_json};

use crate::score;
use crate::{churn, dedup};
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// The mainline window every face defaults to. It was a `30` literal
/// in clap and a `10` literal in the MCP adapter, so one project
/// answered two window sizes depending on which door asked — and a
/// window size IS the judgment's domain (minPoints, slope). Named
/// where the window is measured; the faces read it.
pub const DEFAULT_COMMITS: usize = 30;

/// DDL executed inside the index's ONE schema batch (dedup::schema)
/// — created and wiped with the cache, never on a second lifecycle.
pub const TREND_SCHEMA: &str = "
CREATE TABLE trend (
  commit_hash TEXT PRIMARY KEY,
  ts INTEGER NOT NULL,
  score INTEGER NOT NULL,
  scale INTEGER NOT NULL,
  axes TEXT NOT NULL,
  -- the ce + ce-core pair that produced this score. The header's
  -- comparability contract leaned on the index cache key, whose six
  -- values (params, tokenizer, graph, struct, docdup revs) contain
  -- NOTHING about the scoring formula — so a ce-core upgrade left old
  -- points scored by the old formula sitting next to new ones, and the
  -- slope verdict was computed across two measurement regimes. A row
  -- whose stamp is not today's is re-measured, not reused.
  stamp TEXT NOT NULL
);
";

// Row lives with the wire leg and Report with the faces since the
// headroom sprint (children importing them THROUGH this hub made
// the family a module cycle the graph axis itself billed); the
// re-exports keep every outside path where it always was.
pub use judge::Row;
pub use report::Report;

/// Measure up to `batch` (None = all) uncached mainline commits of
/// the newest `commits`, persist them, hand the window to the core's
/// trend/2 judgment, and return the report.
pub fn run(
    root: &Path,
    db: Option<PathBuf>,
    core: &str,
    commits: usize,
    batch: Option<usize>,
) -> Result<Report> {
    let shas = mainline(root, commits)?;
    let idx = dedup::index::Index::open(&dedup::index_db_path(root, db), dedup::Params::default())?;
    let stamp = toolchain_stamp(core)?;
    // ONE fence for the whole trajectory: the committed soft line, or
    // the config's warn line, or the core's own default. Deriving it
    // per point let a bloat commit move its own goalpost.
    let soft = crate::structure::judge::committed_soft(root);
    let mut have = cached(idx.raw(), &stamp)?;
    let missing: Vec<&(String, i64)> = shas.iter().filter(|(s, _)| !have.contains_key(s)).collect();
    let mut failed = Vec::new();
    let mut measured = 0usize;
    // each uncached point is a full check in a detached temp worktree,
    // so a cold cache is the slowest thing this binary does — and the
    // one whose length the caller CHOOSES, with --batch. The counter
    // is what makes that choice informed (plan v2.16).
    let _span = crate::progress::span();
    let todo = missing.len().min(batch.unwrap_or(usize::MAX));
    for (n, (sha, ts)) in missing.iter().take(todo).enumerate() {
        crate::progress::at(crate::progress::Phase::Measure, n, todo);
        match measure(root, core, sha, *ts, soft) {
            Ok(row) => {
                put(idx.raw(), &row, &stamp)?;
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

/// Newest `n` first-parent commits of HEAD: (full sha, committer
/// time — %ct, the clock the cache keys by).
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

/// One commit's absolute score, judged in a detached temp worktree
/// against a soft line the CALLER pins — see score::Opts::pinned_soft
/// for why deriving it here made the trajectory self-referential.
fn measure(root: &Path, core: &str, sha: &str, ts: i64, soft: u64) -> Result<Row> {
    let wt = Worktree::add(root, sha)?;
    let out = score::run(
        &wt.path,
        score::Opts {
            db: None, // the worktree's own .ce — removed with it
            core: core.to_string(),
            days: None,
            floor: None,
            establish: true,         // no ratchet: an absolute score
            pinned_soft: Some(soft), // ...but ONE fence for every point
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
        // the effective scale, never a /1000 literal (C17) — and a
        // NAMED refusal when the echo lacks it, matching the
        // score-wire stance (batch-7 defect sweep: the silent 1000
        // fallback could mis-scale a whole trajectory)
        scale: *r
            .knobs
            .get("scoreScale")
            .ok_or_else(|| anyhow::anyhow!("trend: reply echo missing scoreScale"))?,
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
        // Drop cannot propagate, but a swallowed failure left BOTH
        // residues silently: the tree (with its .ce) in the temp dir
        // and the .git/worktrees metadata entry accumulating in the
        // real repo. Fall back to a plain directory remove, prune the
        // metadata, and say one line — a leak someone can see is
        // prune-able; a silent one just grows.
        let named = self.path.to_string_lossy().into_owned();
        if churn::git(&self.root, &["worktree", "remove", "--force", &named]).is_err() {
            let _ = std::fs::remove_dir_all(&self.path);
            let _ = churn::git(&self.root, &["worktree", "prune"]);
            eprintln!("ce trend: worktree {named} needed a filesystem teardown");
        }
    }
}

/// Rows measured by the CURRENT ce + ce-core pair. A row from another
/// pair is not wrong, it is incomparable — dropping it here re-measures
/// that commit instead of mixing two scoring regimes into one slope.
fn cached(conn: &rusqlite::Connection, stamp: &str) -> Result<HashMap<String, Row>> {
    let mut stmt =
        conn.prepare("SELECT commit_hash, ts, score, scale, axes FROM trend WHERE stamp = ?1")?;
    let rows = stmt.query_map((stamp,), |r| {
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
/// rev changes wipe the table — so re-landing a row converges.
fn put(conn: &rusqlite::Connection, r: &Row, stamp: &str) -> Result<()> {
    // REPLACE, not IGNORE: the primary key is the commit, and a row
    // left by another toolchain is exactly the one this run just
    // re-measured — keeping it would preserve the incomparable score
    // the WHERE clause in `cached` refused to read.
    conn.execute(
        "INSERT OR REPLACE INTO trend (commit_hash, ts, score, scale, axes, stamp)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        (
            &r.commit,
            r.ts,
            r.score,
            r.scale,
            serde_json::to_string(&r.axes)?,
            stamp,
        ),
    )?;
    Ok(())
}

/// The measuring pair, as one opaque string: this binary's version and
/// the core's, asked of the core itself through the handshake it
/// already performs. The index cache key cannot carry this — its six
/// values describe MEASUREMENT revisions (params, tokenizer, graph,
/// struct, docdup) and say nothing about the scoring formula, so a
/// ce-core upgrade left old points scored by the old formula.
fn toolchain_stamp(core: &str) -> Result<String> {
    let hello = crate::corelink::run(core).map_err(anyhow::Error::msg)?;
    Ok(format!(
        "ce{} core{}",
        env!("CARGO_PKG_VERSION"),
        hello.version
    ))
}
