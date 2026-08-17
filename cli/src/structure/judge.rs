//! `ce structure` (M6 S2): the tree-scale judgment — walk the tree
//! once through the scan measurement (ONE walk for every surface),
//! assemble the fact tables through structure::rows, send ONE
//! structure.request, and re-label the core's dense verdicts with
//! the names this side kept (§5.9.2). Report-only until a score
//! floor lands (S3+): the CLI gates nothing here.

use super::{rows, tree, wire};
use anyhow::{Context, Result};
use serde::Serialize;
use std::path::{Path, PathBuf};

/// JSON output schema id; bump on shape change (plan §7.1). This is
/// the ONE schema the CLI report and the S4 GUI tree share.
/// 0.2.0 (M6 S3a): + divergence (per-mille χ² or null), deviations
/// [{dir, kind}], declaredDirs. 0.3.0 (S3b): + deep (whether the S6
/// redundancy rollup rode the wire — axis-6 rows exist only then).
/// 0.4.0 (S3c): + days (the staleness window; null = axis 5 off).
pub const SCHEMA_ID: &str = "ce.structure-report/0.4.0";

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
    /// Whether the S6 rollup rode the wire (--deep): axis-6 rows in
    /// axes/findings exist exactly when true.
    pub deep: bool,
    /// The staleness window in days (--days): axis-5 rows exist
    /// exactly when set.
    pub days: Option<u32>,
}

pub fn run(
    root: &Path,
    db: Option<PathBuf>,
    core: &str,
    deep: bool,
    days: Option<u32>,
) -> Result<Report> {
    let (files, _findings, _summary) = crate::scan::analyze(root)?;
    let paths: Vec<String> = files.iter().map(|f| f.path.clone()).collect();
    let t = tree::build(&paths);
    let req = assemble(root, db, core, &t, (deep, days))?;
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
        deep,
        days,
    })
}

/// The whole request from one walked tree: the always-on tables,
/// the ce.toml layout, and the two honestly-optional axes (split
/// from run() when the third optional table pushed it past the
/// repo's own function gate). The two axis switches travel as ONE
/// pair — they are one decision ("which optional axes ride"), and
/// the param gate agrees.
fn assemble(
    root: &Path,
    db: Option<PathBuf>,
    core: &str,
    t: &tree::Tree,
    (deep, days): (bool, Option<u32>),
) -> Result<wire::Request> {
    let cfg = crate::config::Config::load(root).map_err(anyhow::Error::msg)?;
    let stale_docs = match days {
        Some(d) => Some(rows::stale_doc_rows(root, db.clone(), t, d)?),
        None => None,
    };
    let redundancy = if deep {
        Some(rows::redundancy_rows(root, db.clone(), core, t)?)
    } else {
        None
    };
    Ok(wire::Request {
        nodes: rows::node_rows(t),
        patterns: rows::pattern_rows(t),
        conventions: rows::convention_rows(t),
        file_refs: rows::file_ref_rows(root, db, t)?,
        declared: rows::declared_rows(&cfg.structure.layout, t)?,
        stale_docs,
        redundancy,
    })
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
    deep: bool,
    days: Option<u32>,
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
            deep: r.deep,
            days: r.days,
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
