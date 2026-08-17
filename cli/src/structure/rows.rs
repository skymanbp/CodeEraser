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
