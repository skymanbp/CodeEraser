//! The one document of `ce arch`, the MCP tool `architecture` and the
//! GUI's reports hub (design booklet §7.3): the directory layers, the
//! cut arcs with the file references behind each, the clusters, the
//! files outside their cluster's directory, the impact of the focus
//! and the per-directory metrics — every id the core answered
//! labelled back to the path this side kept. Labelling only: which
//! arcs are cut, the levels, the clusters, the misplaced files and the
//! depths are the core's, and a document with a `degraded` reason
//! carries no answer at all.

use super::tables::{self, Tables};
use super::wire::{self, Reply};
use crate::graph::deadcode::{Advisory, wire_of};
use crate::structure::tree;
use anyhow::Result;
use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub const SCHEMA_ID: &str = "ce.arch-report/0.1.0";

#[derive(Serialize)]
pub struct Layer {
    pub dir: String,
    pub level: i64,
}

#[derive(Serialize)]
pub struct Reference {
    pub from: String,
    pub to: String,
    pub refs: i64,
}

#[derive(Serialize)]
pub struct Cut {
    pub from: String,
    pub to: String,
    pub refs: i64,
    pub exact: bool,
    /// The file references folded into this arc; a package target is
    /// its directory with a trailing slash.
    pub files: Vec<Reference>,
}

#[derive(Serialize)]
pub struct Cluster {
    pub cluster: i64,
    pub majority: String,
    pub files: Vec<String>,
}

#[derive(Serialize)]
pub struct Misplaced {
    pub path: String,
    pub dir: String,
    pub majority: String,
}

#[derive(Serialize)]
pub struct Impact {
    pub path: String,
    pub depth: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Metric {
    pub dir: String,
    pub fan_in: i64,
    pub fan_out: i64,
    /// Per mille; None when no arc touches the directory.
    pub instability: Option<i64>,
}

#[derive(Serialize)]
pub struct Report {
    pub schema: &'static str,
    pub counts: BTreeMap<&'static str, i64>,
    pub layers: Vec<Layer>,
    pub cuts: Vec<Cut>,
    pub clusters: Vec<Cluster>,
    pub misplaced: Vec<Misplaced>,
    pub impact: Vec<Impact>,
    pub metrics: Vec<Metric>,
    pub degraded: Option<String>,
}

/// The whole leg: the graph wire off a refreshed index, the tree over
/// its measured files, the tables, the core, the labels. A focus path
/// that names no measured file is an error and no document.
pub fn run(root: &Path, db: Option<PathBuf>, core: &str, focus: &[String]) -> Result<Report> {
    let (idx, db_path) = crate::dedup::refreshed_index(root, db)?;
    let w = wire_of(root, &idx, &db_path, Advisory::No)?;
    drop(idx);
    let paths = tables::measured_paths(&w);
    let t = tree::build(&paths);
    let lines = paths
        .iter()
        .map(|p| line_count(root, p))
        .collect::<Result<Vec<i64>>>()?;
    let tables = tables::assemble(&w, &t, focus, &lines)?;
    Ok(match wire::judge(core, &tables)? {
        Ok(reply) => label(&tables, &reply),
        Err(why) => degraded(why),
    })
}

/// A file's total lines (scan's own count; a file deleted mid-run has
/// none).
fn line_count(root: &Path, path: &str) -> Result<i64> {
    let bytes = crate::scan::walk::read_surviving(&root.join(path))?;
    Ok(bytes.map_or(0, |b| crate::scan::metrics::size::total_lines(&b) as i64))
}

/// The six tables empty, the counts zero, the reason named.
fn degraded(why: String) -> Report {
    Report {
        schema: SCHEMA_ID,
        counts: wire::count_keys().map(|k| (k, 0)).collect(),
        layers: Vec::new(),
        cuts: Vec::new(),
        clusters: Vec::new(),
        misplaced: Vec::new(),
        impact: Vec::new(),
        metrics: Vec::new(),
        degraded: Some(why),
    }
}

/// The core's ids labelled through the two ledgers (consume checked
/// every id is in range and every row's width).
fn label(t: &Tables, r: &Reply) -> Report {
    let dir = |d: i64| t.dir_paths[d as usize].clone();
    let path = |f: i64| t.paths[f as usize].clone();
    // the layers and the metrics carry one row per directory, in order
    let by_dir = |key| t.dir_paths.iter().cloned().zip(r.rows(key));
    let layers = by_dir("layers").map(|(dir, x)| Layer { dir, level: x[1] });
    let misplaced = r.rows("misplaced").iter().map(|x| Misplaced {
        path: path(x[0]),
        dir: dir(t.files[x[0] as usize][1]),
        majority: dir(x[1]),
    });
    let impact = r.rows("impact").iter().map(|x| Impact {
        path: path(x[0]),
        depth: x[1],
    });
    let metrics = by_dir("metrics").map(|(dir, x)| Metric {
        dir,
        fan_in: x[1],
        fan_out: x[2],
        instability: (x[3] >= 0).then_some(x[3]),
    });
    Report {
        schema: SCHEMA_ID,
        counts: r.counts.clone(),
        layers: layers.collect(),
        cuts: r.rows("cuts").iter().map(|c| cut(t, c)).collect(),
        clusters: clusters(t, r.rows("clusters")),
        misplaced: misplaced.collect(),
        impact: impact.collect(),
        metrics: metrics.collect(),
        degraded: None,
    }
}

/// A directory as an arc end: its path with a trailing slash, the
/// root as `./`.
pub fn slashed(dir: &str) -> String {
    if dir.is_empty() {
        "./".into()
    } else {
        format!("{dir}/")
    }
}

/// One cut arc with the file references it folds: the file edges and
/// the package references whose two directories are the arc's.
fn cut(t: &Tables, row: &[i64]) -> Cut {
    let (a, b) = (row[0], row[1]);
    let dir_of = |f: i64| t.files[f as usize][1];
    let file_refs = t
        .edges
        .iter()
        .filter(|e| (dir_of(e[0]), dir_of(e[1])) == (a, b))
        .map(|e| {
            let to = t.paths[e[1] as usize].clone();
            Reference {
                from: t.paths[e[0] as usize].clone(),
                to,
                refs: e[2],
            }
        });
    let pkg_refs = t
        .pkg_edges
        .iter()
        .filter(|e| (dir_of(e[0]), e[1]) == (a, b))
        .map(|e| {
            let to = slashed(&t.dir_paths[e[1] as usize]);
            Reference {
                from: t.paths[e[0] as usize].clone(),
                to,
                refs: e[2],
            }
        });
    let mut files: Vec<Reference> = file_refs.chain(pkg_refs).collect();
    files.sort_by(|x, y| (&x.from, &x.to).cmp(&(&y.from, &y.to)));
    Cut {
        from: t.dir_paths[a as usize].clone(),
        to: t.dir_paths[b as usize].clone(),
        refs: row[2],
        exact: row[3] == 1,
        files,
    }
}

/// The core's cluster ids with their files, and the directory holding
/// most of each cluster's files — the least directory id on a tie, the
/// same reading the core names a misplaced file's majority by, so a
/// cluster and its misplaced rows never name two majorities.
fn clusters(t: &Tables, rows: &[Vec<i64>]) -> Vec<Cluster> {
    let mut by: BTreeMap<i64, Vec<i64>> = BTreeMap::new();
    for x in rows {
        by.entry(x[1]).or_default().push(x[0]);
    }
    by.into_iter()
        .map(|(cluster, members)| {
            let mut held: BTreeMap<i64, i64> = BTreeMap::new();
            for f in &members {
                *held.entry(t.files[*f as usize][1]).or_insert(0) += 1;
            }
            let most = held.values().copied().max().unwrap_or(0);
            let majority = held
                .iter()
                .find(|(_, n)| **n == most)
                .map_or(0, |(d, _)| *d);
            Cluster {
                cluster,
                majority: t.dir_paths[majority as usize].clone(),
                files: members
                    .iter()
                    .map(|f| t.paths[*f as usize].clone())
                    .collect(),
            }
        })
        .collect()
}

pub fn report_json(r: &Report) -> Value {
    serde_json::to_value(r).expect("an arch report serializes")
}

/// The document a machine face relays for a root: the report on the
/// root's own index, serialized — what `ce arch --format json` prints.
pub fn document(root: &Path, core: &str, focus: &[String]) -> Result<Value> {
    Ok(report_json(&run(root, None, core, focus)?))
}
