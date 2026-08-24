//! The graph canvas document (batch 9 P18, user-ruled): the file-tier
//! projection the GUI's canvas screen draws — nodes with their liveness
//! verdict and graph position, edges collapsed to file→file pairs.
//! Everything here is a re-arrangement of ONE deadcode-family judgment
//! (P10 half-doors); no measurement, no policy, zero core change.
//! Sections collapse into the file that holds them (their path IS the
//! file); the synthetic package containment arcs drop — an aggregate is
//! not a code entity (RG9) and its arcs exist for liveness, not for the
//! reader's map. Self-loops after the collapse drop; duplicates fold.

use super::deadcode::{GraphWire, Report, file_nodes};
use super::wire::GRAN_PACKAGE;
use crate::join::Pos;
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet, HashMap};

/// JSON output schema id; bump on shape change (plan §7.1).
pub const SCHEMA_ID: &str = "ce.graph-canvas/0.1.0";

/// The one-judgment assembly (P10 half-doors end to end): refreshed
/// index → wire → judge with the full file-tier pos request →
/// verdicts + positions → document. The face layer only delegates
/// here, so the canvas road has one owner.
pub fn run(root: &std::path::Path, core: &str) -> Result<Value> {
    let (idx, db_path) = crate::dedup::refreshed_index(root, None)?;
    let w = super::deadcode::wire_of(root, &idx, &db_path)?;
    drop(idx);
    let pos_req: Vec<i64> = file_nodes(&w).iter().map(|x| x.0).collect();
    let (report, reply) = super::deadcode::judged(root, core, &w, &pos_req)?;
    let pos = crate::join::pos_map(&reply, &w)?;
    Ok(document(&w, &report, &pos))
}

/// The document: files in file_nodes order, edges as index pairs into
/// that order, counts the header can print. Verdict/pos absence is
/// null, never a fabricated zero (the join's own rule).
pub fn document(w: &GraphWire, report: &Report, pos: &HashMap<String, Pos>) -> Value {
    let files = file_nodes(w);
    let dead: BTreeMap<&str, (&str, &str)> = report
        .dead
        .iter()
        .map(|d| (d.path.as_str(), (d.verdict, d.why.as_str())))
        .collect();
    let rows: Vec<Value> = files
        .iter()
        .map(|&(_, p)| {
            let d = dead.get(p);
            json!({
                "path": p,
                "verdict": d.map(|&(v, _)| v),
                "why": d.map(|&(_, w)| w),
                "pos": pos.get(p),
            })
        })
        .collect();
    let edges = file_edges(w, &files);
    let n_edges = edges.len();
    let cycles = pos
        .values()
        .filter(|p| p[3] > 1)
        .map(|p| p[2])
        .collect::<BTreeSet<_>>()
        .len();
    json!({
        "schema": SCHEMA_ID,
        "files": rows,
        "edges": edges,
        "counts": {
            "files": files.len(),
            "edges": n_edges,
            "dead": report.dead.len(),
            "cycles": cycles,
        },
        "unresolvedSites": w.unresolved_sites,
        "degraded": report.degraded,
    })
}

/// Wire edges → deduped file-index pairs. An endpoint that is a
/// package drops the edge (synthetic containment); a section endpoint
/// resolves to its file's path; a self-loop after the collapse drops.
fn file_edges(w: &GraphWire, files: &[(i64, &str)]) -> Vec<[usize; 2]> {
    let at: HashMap<&str, usize> = files
        .iter()
        .enumerate()
        .map(|(i, &(_, p))| (p, i))
        .collect();
    let mut out = BTreeSet::new();
    for &[s, d, _, _] in &w.edges {
        let Some(a) = w.nodes.get(s as usize) else {
            continue;
        };
        let Some(b) = w.nodes.get(d as usize) else {
            continue;
        };
        if a.kind == GRAN_PACKAGE || b.kind == GRAN_PACKAGE {
            continue;
        }
        let Some(&ai) = at.get(a.path.as_str()) else {
            continue;
        };
        let Some(&bi) = at.get(b.path.as_str()) else {
            continue;
        };
        if ai != bi {
            out.insert([ai, bi]);
        }
    }
    out.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::super::deadcode::{GraphWire, Report};
    use super::super::nodes::Node;
    use super::super::wire::{GRAN_FILE, GRAN_PACKAGE, GRAN_SECTION};
    use super::*;

    #[test]
    fn sections_collapse_packages_drop_and_cycles_count() {
        let nodes = vec![
            Node {
                path: "a.rs".into(),
                unit: "".into(),
                kind: GRAN_FILE,
            },
            Node {
                path: "b.rs".into(),
                unit: "".into(),
                kind: GRAN_FILE,
            },
            Node {
                path: "a.rs".into(),
                unit: "Intro".into(),
                kind: GRAN_SECTION,
            },
            Node {
                path: "pkg".into(),
                unit: "".into(),
                kind: GRAN_PACKAGE,
            },
        ];
        let edges = [[2, 1, 0, 0], [0, 1, 5, 0], [3, 0, 0, 0], [2, 0, 0, 0]]
            .into_iter()
            .collect();
        let w = GraphWire {
            nodes,
            rows: vec![],
            edges,
            unresolved_sites: 7,
            unres: vec![],
        };
        let report = Report {
            dead: vec![crate::graph::deadcode::DeadRow {
                path: "b.rs".into(),
                verdict: "unref_private",
                why: "no kept in-edge and no entry flag".into(),
                conf: Some(2),
            }],
            reported: vec![],
            nodes: 4,
            kept: 3,
            unresolved_sites: 7,
            degraded: None,
            fail: true,
        };
        let pos = HashMap::from([
            ("a.rs".into(), [1, 2, 0, 1, 0]),
            ("b.rs".into(), [2, 0, 4, 2, 1]),
        ]);
        let doc = document(&w, &report, &pos);
        assert_eq!(doc["edges"], json!([[0, 1]]));
        assert_eq!(doc["counts"]["files"], 2);
        assert_eq!(doc["counts"]["edges"], 1);
        assert_eq!(doc["counts"]["dead"], 1);
        assert_eq!(doc["counts"]["cycles"], 1);
        assert_eq!(doc["files"][0]["verdict"], Value::Null);
        assert_eq!(doc["files"][0]["pos"], json!([1, 2, 0, 1, 0]));
        assert_eq!(doc["files"][1]["verdict"], "unref_private");
        assert_eq!(doc["files"][1]["why"], "no kept in-edge and no entry flag");
        assert_eq!(doc["files"][1]["pos"], json!([2, 0, 4, 2, 1]));
        assert_eq!(doc["unresolvedSites"], 7);
        assert_eq!(doc["degraded"], Value::Null);
        assert_eq!(doc["schema"], SCHEMA_ID);
    }
}
