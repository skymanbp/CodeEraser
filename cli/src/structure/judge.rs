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
/// 0.2.0 (M6 S3a): + divergence (per-mille χ² or null), deviations
/// [{dir, kind}], declaredDirs.
pub const SCHEMA_ID: &str = "ce.structure-report/0.2.0";

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
    /// A-layer overlay (S3a): the χ² per-mille against ce.toml's
    /// declared layout, the named deviations, and how many dirs the
    /// layout declared (0 = row-C floor alone; divergence None with
    /// declared > 0 means the deviations rows say where the mass
    /// escaped — null is never a silent shrug).
    pub divergence: Option<i64>,
    pub deviations: Vec<(String, i64)>,
    pub declared: usize,
}

pub fn run(root: &Path, db: Option<PathBuf>, core: &str) -> Result<Report> {
    let (files, _findings, _summary) = crate::scan::analyze(root)?;
    let paths: Vec<String> = files.iter().map(|f| f.path.clone()).collect();
    let t = tree::build(&paths);
    let cfg = crate::config::Config::load(root).map_err(anyhow::Error::msg)?;
    let declared = declared_rows(&cfg.structure.layout, &t)?;
    let req = wire::Request {
        nodes: node_rows(&t),
        patterns: pattern_rows(&t),
        conventions: convention_rows(&t),
        file_refs: file_ref_rows(root, db, &t)?,
        declared,
    };
    let reply = wire::judge(core, &req)?;
    let names = names_by_id(&t);
    let findings = relabel(&names, &reply.findings);
    let deviations = relabel(&names, &reply.deviations);
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
        divergence: reply.divergence,
        deviations,
        declared: req.declared.len(),
    })
}

/// ce.toml's [structure] layout compiled to [dirId, weight] rows —
/// a declared path that names no walked directory is a LOUD config
/// error (a template that names nothing judges nothing), never a
/// silently dropped row.
fn declared_rows(
    layout: &std::collections::BTreeMap<String, u32>,
    t: &tree::Tree,
) -> Result<Vec<[u64; 2]>> {
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

/// Dense [dirId, code] rows re-labelled with the names this side
/// kept — findings and deviations ride the same throat (§5.9.2).
fn relabel(names: &[String], rows: &[[i64; 2]]) -> Vec<(String, i64)> {
    rows.iter()
        .map(|&[d, code]| (names[d as usize].clone(), code))
        .collect()
}

/// (name, code) pairs as {key1, key2} JSON objects — the report's
/// two labelled tables share one shape.
fn labeled(rows: &[(String, i64)], k1: &str, k2: &str) -> Vec<serde_json::Value> {
    rows.iter()
        .map(|(d, c)| serde_json::json!({k1: d, k2: c}))
        .collect()
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
    divergence: Option<i64>,
    deviations: Vec<serde_json::Value>,
    #[serde(rename = "declaredDirs")]
    declared_dirs: usize,
}

pub fn print(r: &Report, as_json: bool) {
    if as_json {
        let doc = JsonReport {
            schema: SCHEMA_ID,
            score: r.score,
            score_scale: r.scale,
            entropy: &r.entropy,
            axes: &r.axes,
            findings: labeled(&r.findings, "dir", "axis"),
            dirs: r.dirs,
            divergence: r.divergence,
            deviations: labeled(&r.deviations, "dir", "kind"),
            declared_dirs: r.declared,
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
    if r.declared > 0 {
        match r.divergence {
            Some(d) => println!("layout divergence {d}‰ over {} declared dirs", r.declared),
            None => println!(
                "layout divergence undefined: mass outside the {} declared dirs",
                r.declared
            ),
        }
    }
    for (dir, kind) in &r.deviations {
        let label = if *kind == 0 {
            "undeclared territory"
        } else {
            "declared but empty"
        };
        println!("deviation {dir}  {label}");
    }
    for (dir, axis) in &r.findings {
        println!("finding {dir}  axis {axis}");
    }
}
