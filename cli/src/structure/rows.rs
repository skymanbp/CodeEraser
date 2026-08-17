//! Request-row assembly for structure/1 (split from judge.rs when
//! the S3b rollup pushed it past the repo's own file gate): every
//! fact table the wire carries is produced here — dense tree rows,
//! pattern/convention distributions, the reference splits, the
//! declared layout, the S6 rollup. Names resolve to dense ids on
//! this side and never cross (§5.9.2); judging stays in the core.

use super::{edges, tree};
use anyhow::{Context, Result};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

pub fn node_rows(t: &tree::Tree) -> Vec<[u64; 5]> {
    t.dirs
        .iter()
        .enumerate()
        .map(|(i, d)| {
            [
                i as u64,
                d.parent as u64,
                d.depth as u64,
                d.subdirs as u64,
                d.files as u64,
            ]
        })
        .collect()
}

pub fn pattern_rows(t: &tree::Tree) -> Vec<[u64; 3]> {
    let mut rows = Vec::new();
    for (i, d) in t.dirs.iter().enumerate() {
        for (code, &n) in d.patterns.iter().enumerate() {
            if n > 0 {
                rows.push([i as u64, code as u64, n as u64]);
            }
        }
    }
    rows
}

pub fn convention_rows(t: &tree::Tree) -> Vec<[u64; 2]> {
    t.dirs
        .iter()
        .enumerate()
        .filter(|(_, d)| d.conventions > 0)
        .map(|(i, d)| [i as u64, d.conventions as u64])
        .collect()
}

/// The cached reference graph adapted into aggregated
/// [dirId, inside, outside, count] rows — graph nodes that never
/// entered the walked tree (a universe mismatch) are an error by
/// name, never a guess.
pub fn file_ref_rows(root: &Path, db: Option<PathBuf>, t: &tree::Tree) -> Result<Vec<[u64; 4]>> {
    let w = crate::graph::deadcode::build_wire(root, db)?;
    let fnodes = crate::graph::deadcode::file_nodes(&w);
    let mut file_dirs = Vec::with_capacity(fnodes.len());
    let mut index_of: BTreeMap<i64, usize> = BTreeMap::new();
    for (slot, &(i, p)) in fnodes.iter().enumerate() {
        let dir = tree::dir_of(t, p)
            .ok_or_else(|| anyhow::anyhow!("graph node {p} outside the walked tree"))?;
        file_dirs.push(dir);
        index_of.insert(i, slot);
    }
    // graph edges between FILE nodes only (unit-tier endpoints have
    // no directory of their own)
    let pairs: Vec<(usize, usize)> = w
        .edges
        .iter()
        .filter_map(|e| Some((*index_of.get(&e[0])?, *index_of.get(&e[1])?)))
        .collect();
    let g = edges::aggregate(&pairs, &file_dirs, t.dirs.len());
    let mut counted: BTreeMap<[u64; 3], u64> = BTreeMap::new();
    for (slot, io) in g.files.iter().enumerate() {
        if io[0] + io[1] > 0 {
            *counted
                .entry([file_dirs[slot] as u64, io[0] as u64, io[1] as u64])
                .or_insert(0) += 1;
        }
    }
    Ok(counted
        .into_iter()
        .map(|([d, i, o], n)| [d, i, o, n])
        .collect())
}

/// The S6 rollup: clone blocks and dead units convolved per
/// directory — the per-file families' own outputs, never re-derived
/// here. A block counts once per involved directory; a degraded
/// liveness judgment REFUSES the whole rollup (zeros it would send
/// instead are fakes, and the axis's honest absence road already
/// exists — drop --deep).
pub fn redundancy_rows(
    root: &Path,
    db: Option<PathBuf>,
    core: &str,
    t: &tree::Tree,
) -> Result<Vec<[u64; 3]>> {
    let mut dup: BTreeMap<usize, u64> = BTreeMap::new();
    let (found, _stats) = crate::dedup::analyze(root, db.clone(), None, None)?;
    for b in &found.blocks {
        let mut dirs = BTreeSet::new();
        for f in [b.a_file.as_str(), b.b_file.as_str()] {
            dirs.insert(
                tree::dir_of(t, f).ok_or_else(|| {
                    anyhow::anyhow!("clone block file {f} outside the walked tree")
                })?,
            );
        }
        for d in dirs {
            *dup.entry(d).or_insert(0) += 1;
        }
    }
    let dead_report = crate::graph::deadcode::run(root, db, core)?;
    if let Some(reason) = &dead_report.degraded {
        anyhow::bail!("liveness degraded ({reason}) — refusing a fake-zero S6 rollup");
    }
    let mut dead: BTreeMap<usize, u64> = BTreeMap::new();
    for (name, _verdict, _why) in &dead_report.dead {
        // dead rows are file nodes by Report contract; a name the
        // walked tree cannot place is a universe mismatch, an error
        // by name (the file_ref_rows posture), never a guess
        let d = tree::dir_of(t, name)
            .ok_or_else(|| anyhow::anyhow!("dead node {name} outside the walked tree"))?;
        *dead.entry(d).or_insert(0) += 1;
    }
    let dirs: BTreeSet<usize> = dup.keys().chain(dead.keys()).copied().collect();
    Ok(dirs
        .into_iter()
        .map(|d| {
            [
                d as u64,
                dup.get(&d).copied().unwrap_or(0),
                dead.get(&d).copied().unwrap_or(0),
            ]
        })
        .collect())
}

/// The S5 staleness join: every markdown file's outgoing reference
/// targets (the md ladder's own resolutions, file-granular via each
/// node's path) against ONE windowed `git log` pass. A doc is stale
/// when any target changed after the doc's own last change inside
/// the window — a same-commit update of both is NOT stale (the doc
/// kept up). Docs without references carry no total; absence of the
/// whole table (no --days) leaves axis 5 unjudged.
pub fn stale_doc_rows(
    root: &Path,
    db: Option<PathBuf>,
    t: &tree::Tree,
    days: u32,
) -> Result<Vec<[u64; 3]>> {
    let w = crate::graph::deadcode::build_wire(root, db)?;
    let mut targets: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
    for e in &w.edges {
        let s = &w.nodes[e[0] as usize];
        let d = &w.nodes[e[1] as usize];
        if s.path.ends_with(".md") && d.path != s.path {
            targets.entry(s.path.as_str()).or_default().insert(&d.path);
        }
    }
    let newest = newest_in_window(root, days)?;
    let mut per_dir: BTreeMap<usize, [u64; 2]> = BTreeMap::new();
    for (md, tgts) in &targets {
        let dir = tree::dir_of(t, md)
            .ok_or_else(|| anyhow::anyhow!("md node {md} outside the walked tree"))?;
        let doc_last = newest.get(*md).copied();
        let is_stale = tgts.iter().any(|tg| match (newest.get(*tg), doc_last) {
            (Some(&tt), Some(dl)) => tt > dl,
            (Some(_), None) => true,
            (None, _) => false,
        });
        let e = per_dir.entry(dir).or_insert([0, 0]);
        e[1] += 1;
        if is_stale {
            e[0] += 1;
        }
    }
    Ok(per_dir
        .into_iter()
        .map(|(d, [s, n])| [d as u64, s, n])
        .collect())
}

/// Newest commit time per touched file inside the window, one git
/// pass (churn's own runner — no second git throat). The \x01
/// sentinel keeps an all-digit FILENAME from parsing as a commit
/// time.
fn newest_in_window(root: &Path, days: u32) -> Result<BTreeMap<String, i64>> {
    let since = format!("--since={days} days ago");
    let log = crate::churn::git(root, &["log", &since, "--format=%x01%ct", "--name-only"])?;
    let mut newest: BTreeMap<String, i64> = BTreeMap::new();
    let mut cur = 0i64;
    for line in log.lines() {
        if let Some(ts) = line.strip_prefix('\u{1}') {
            cur = ts.trim().parse().context("commit time")?;
        } else if !line.is_empty() {
            let e = newest.entry(line.to_string()).or_insert(cur);
            *e = (*e).max(cur);
        }
    }
    Ok(newest)
}

/// ce.toml's [structure] layout compiled to [dirId, weight] rows —
/// a declared path that names no walked directory is a LOUD config
/// error (a template that names nothing judges nothing), never a
/// silently dropped row.
pub fn declared_rows(layout: &BTreeMap<String, u32>, t: &tree::Tree) -> Result<Vec<[u64; 2]>> {
    let mut rows = Vec::with_capacity(layout.len());
    for (path, &w) in layout {
        let key = path.trim_end_matches('/');
        let key = if key == "." { "" } else { key };
        let id = t.ids.get(key).with_context(|| {
            format!("[structure] layout declares {path:?}, which is not a walked directory")
        })?;
        rows.push([*id as u64, u64::from(w)]);
    }
    rows.sort_unstable();
    Ok(rows)
}
