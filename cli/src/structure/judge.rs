//! `ce structure` (M6 S2): the tree-scale judgment — walk the tree
//! once through the scan measurement (ONE walk for every surface),
//! aggregate through structure::tree and structure::edges, adapt
//! the cached reference graph into file→dir reference splits, send
//! ONE structure.request, and re-label the core's dense verdicts
//! with the names this side kept (§5.9.2). Report-only in S2: the
//! CLI gates nothing until a score floor lands with S3+.

use super::{edges, tree, wire};
use anyhow::{Context, Result};
use serde::Serialize;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// JSON output schema id; bump on shape change (plan §7.1). This is
/// the ONE schema the CLI report and the S4 GUI tree share.
pub const SCHEMA_ID: &str = "ce.structure-report/0.1.0";

pub struct Report {
    pub score: i64,
    /// The effective scale from the knob echo (row 8) — the review
    /// C17 lesson applied from day one: no /1000 literal anywhere.
    pub scale: i64,
    pub entropy: Vec<[i64; 2]>,
    pub axes: Vec<[i64; 2]>,
    /// (dir path, axis code), re-labelled from the core's dense ids.
    pub findings: Vec<(String, i64)>,
    pub dirs: usize,
}

pub fn run(root: &Path, db: Option<PathBuf>, core: &str) -> Result<Report> {
    let (files, _findings, _summary) = crate::scan::analyze(root)?;
    let paths: Vec<String> = files.iter().map(|f| f.path.clone()).collect();
    let t = tree::build(&paths);
    let req = wire::Request {
        nodes: node_rows(&t),
        patterns: pattern_rows(&t),
        conventions: convention_rows(&t),
        file_refs: file_ref_rows(root, db, &t)?,
    };
    let reply = wire::judge(core, &req)?;
    let names = names_by_id(&t);
    let findings = reply
        .findings
        .iter()
        .map(|&[d, axis]| (names[d as usize].clone(), axis))
        .collect();
    let scale = reply
        .knobs
        .iter()
        .find(|[c, _]| *c == 8)
        .map(|[_, v]| *v)
        .context("knob echo missing the scale row")?;
    Ok(Report {
        score: reply.score,
        scale,
        entropy: reply.entropy,
        axes: reply.axes,
        findings,
        dirs: t.dirs.len(),
    })
}

fn node_rows(t: &tree::Tree) -> Vec<[u64; 5]> {
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

fn pattern_rows(t: &tree::Tree) -> Vec<[u64; 3]> {
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

fn convention_rows(t: &tree::Tree) -> Vec<[u64; 2]> {
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
fn file_ref_rows(root: &Path, db: Option<PathBuf>, t: &tree::Tree) -> Result<Vec<[u64; 4]>> {
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

fn names_by_id(t: &tree::Tree) -> Vec<String> {
    let mut names = vec![String::from("."); t.dirs.len()];
    for (path, &id) in &t.ids {
        if !path.is_empty() {
            names[id] = path.clone();
        }
    }
    names
}

#[derive(Serialize)]
struct JsonReport<'a> {
    schema: &'static str,
    score: i64,
    #[serde(rename = "scoreScale")]
    score_scale: i64,
    entropy: &'a [[i64; 2]],
    axes: &'a [[i64; 2]],
    findings: Vec<serde_json::Value>,
    dirs: usize,
}

pub fn print(r: &Report, as_json: bool) {
    if as_json {
        let doc = JsonReport {
            schema: SCHEMA_ID,
            score: r.score,
            score_scale: r.scale,
            entropy: &r.entropy,
            axes: &r.axes,
            findings: r
                .findings
                .iter()
                .map(|(d, a)| serde_json::json!({"dir": d, "axis": a}))
                .collect(),
            dirs: r.dirs,
        };
        println!("{}", serde_json::to_string(&doc).expect("report json"));
        return;
    }
    let axes: Vec<String> = r.axes.iter().map(|[c, p]| format!("{c}:{p}")).collect();
    let ent: Vec<String> = r.entropy.iter().map(|[k, v]| format!("{k}:{v}")).collect();
    println!(
        "structure score {}/{} | entropy {} | axes {} | {} dirs",
        r.score,
        r.scale,
        ent.join(" "),
        axes.join(" "),
        r.dirs
    );
    for (dir, axis) in &r.findings {
        println!("finding {dir}  axis {axis}");
    }
}
