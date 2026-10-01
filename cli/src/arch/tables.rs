//! The five arch/1 request tables off one graph wire and one tree
//! (design booklet docs/reference/analysis-track.md §7.1; the step-9
//! measuring-side rulings 1–6). The node universe is the structure
//! family's: the measured FILE nodes of the deadcode wire, in path
//! order, dense from 0. The directories are that family's tree over
//! those paths. The references are the graph's file → file arcs (any
//! kind, any rung, through the structure family's own join), a section
//! target folded onto its file, and the file → package arcs onto the
//! package's directory. Counting and numbering only — the directory
//! graph, its cuts, layers, clusters and the impact walk are the
//! core's (CE.Arch.*).

use crate::graph::deadcode::GraphWire;
use crate::graph::wire::{GRAN_FILE, GRAN_PACKAGE, GRAN_SECTION};
use crate::structure::tree::Tree;
use anyhow::{Result, bail, ensure};
use std::collections::{BTreeMap, BTreeSet};

/// One request table: rows of `N` integers.
pub type Rows<const N: usize> = Vec<[i64; N]>;

/// The request tables and the two name ledgers the reply is labelled
/// through. Every table is ascending as the wire demands.
pub struct Tables {
    /// `[F, D, lines]`, F the path-order slot.
    pub files: Rows<3>,
    /// F → path.
    pub paths: Vec<String>,
    /// `[D, parent]`: the root row 0 with parent −1, every other
    /// parent an earlier row.
    pub dirs: Rows<2>,
    /// D → directory path, the root `""`.
    pub dir_paths: Vec<String>,
    /// `[F, G, w]`, F ≠ G, w the arcs between them.
    pub edges: Rows<3>,
    /// `[F, D, w]`: F's references to the package at directory D.
    pub pkg_edges: Rows<3>,
    /// The `--impact` files, ascending, distinct.
    pub focus: Vec<i64>,
}

/// The measured file paths, in the wire's node order — path order,
/// since the node set is a BTreeSet (graph/nodes.rs).
pub fn measured_paths(w: &GraphWire) -> Vec<String> {
    crate::graph::deadcode::measured_nodes(w)
        .into_iter()
        .map(|(_, p)| p.to_string())
        .collect()
}

/// The five tables. `lines[F]` is the F-th measured file's line count
/// (the caller reads the files); `focus` are root-relative paths, and
/// one that names no measured file is a named error.
pub fn assemble(w: &GraphWire, t: &Tree, focus: &[String], lines: &[i64]) -> Result<Tables> {
    let (pairs, file_dirs) = crate::structure::rows::file_join(w, t)?;
    let paths = measured_paths(w);
    ensure!(
        lines.len() == paths.len(),
        "arch: {} line counts for {} files",
        lines.len(),
        paths.len()
    );
    let slot: BTreeMap<&str, usize> = paths
        .iter()
        .enumerate()
        .map(|(f, p)| (p.as_str(), f))
        .collect();
    let (sections, packages) = folded(w, t, &slot);
    let files = file_dirs
        .iter()
        .zip(lines)
        .enumerate()
        .map(|(f, (&d, &n))| [f as i64, d as i64, n])
        .collect();
    Ok(Tables {
        files,
        dirs: dir_rows(t)?,
        edges: counted(pairs.into_iter().chain(sections).filter(|(f, g)| f != g)),
        pkg_edges: counted(packages),
        focus: focus_rows(focus, &slot)?,
        dir_paths: dir_paths(t),
        paths,
    })
}

/// The section references folded onto files, and the package
/// references onto directories: `(F, G)` and `(F, D)` pairs.
type Folded = (Vec<(usize, usize)>, Vec<(usize, usize)>);

/// The two arcs the file join leaves out: a file's reference to a
/// section (a Markdown anchor) is a reference to that section's file,
/// and its reference to a package is one to the package's directory
/// in the tree. A target outside the measured files or the tree — a
/// foreign or asset file, a package no measured file sits in — has no
/// row to be.
fn folded(w: &GraphWire, t: &Tree, slot: &BTreeMap<&str, usize>) -> Folded {
    let (mut sections, mut packages) = (Vec::new(), Vec::new());
    for e in &w.edges {
        let (src, dst) = (&w.nodes[e[0] as usize], &w.nodes[e[1] as usize]);
        let Some(&f) = slot
            .get(src.path.as_str())
            .filter(|_| src.kind == GRAN_FILE)
        else {
            continue;
        };
        match dst.kind {
            GRAN_SECTION => sections.extend(slot.get(dst.path.as_str()).map(|&g| (f, g))),
            GRAN_PACKAGE => packages.extend(t.ids.get(dst.path.as_str()).map(|&d| (f, d))),
            _ => {}
        }
    }
    (sections, packages)
}

/// Pairs to `[a, b, w]` rows, one per distinct pair, ascending.
fn counted(pairs: impl IntoIterator<Item = (usize, usize)>) -> Rows<3> {
    let mut by: BTreeMap<(usize, usize), i64> = BTreeMap::new();
    for p in pairs {
        *by.entry(p).or_insert(0) += 1;
    }
    by.into_iter()
        .map(|((a, b), w)| [a as i64, b as i64, w])
        .collect()
}

/// The tree as `[D, parent]` rows. `tree::build` enters every
/// ancestor before its child, so a parent is always an earlier row;
/// the check says so by name rather than let the core refuse it.
fn dir_rows(t: &Tree) -> Result<Rows<2>> {
    let mut rows = vec![[0, -1]];
    for (d, dir) in t.dirs.iter().enumerate().skip(1) {
        ensure!(
            dir.parent < d,
            "arch: directory {d}'s parent {} is not an earlier row",
            dir.parent
        );
        rows.push([d as i64, dir.parent as i64]);
    }
    Ok(rows)
}

fn dir_paths(t: &Tree) -> Vec<String> {
    let mut names = vec![String::new(); t.dirs.len()];
    for (path, &d) in &t.ids {
        names[d] = path.clone();
    }
    names
}

/// Ruling 6: every `--impact` path names a measured file, spelled
/// root-relative with forward slashes.
fn focus_rows(focus: &[String], slot: &BTreeMap<&str, usize>) -> Result<Vec<i64>> {
    let mut rows = BTreeSet::new();
    for p in focus {
        match slot.get(p.as_str()) {
            Some(&f) => rows.insert(f as i64),
            None => bail!("--impact: {p} is not a measured file"),
        };
    }
    Ok(rows.into_iter().collect())
}

#[cfg(test)]
#[path = "../../tests/unit/arch/tables.rs"]
mod tests;
