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
use super::wire::{GRAN_FILE, GRAN_PACKAGE};
use crate::join::Pos;
use anyhow::{Context, Result};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet, HashMap};

/// JSON output schema id; bump on shape change (plan §7.1). 0.3.0:
/// each file row carries `cycle`, the core's cycle membership.
pub const SCHEMA_ID: &str = "ce.graph-canvas/0.3.0";

/// The one-judgment assembly (P10 half-doors end to end): refreshed
/// index → wire → judge with the full file-tier pos request →
/// verdicts + positions + cycles → document. The face layer only
/// delegates here, so the canvas road has one owner.
pub fn run(root: &std::path::Path, core: &str) -> Result<Value> {
    let (idx, db_path) = crate::dedup::refreshed_index(root, None)?;
    // the canvas draws liveness and position; the symbol advisory is
    // the deadcode face's (W4-F2: a road that cannot render it pays
    // nothing for it)
    let w = super::deadcode::wire_of(root, &idx, &db_path, super::deadcode::Advisory::No)?;
    drop(idx);
    let pos_req: Vec<i64> = file_nodes(&w).iter().map(|x| x.0).collect();
    let (report, reply) = super::deadcode::judged(root, core, &w, &pos_req)?;
    let pos = crate::join::pos_map(&reply, &w)?;
    let cycles = file_cycles(&reply, &w)?;
    Ok(document(&w, &report, &pos, &cycles))
}

/// The core's cycle report restricted to the file tier (RG9: cycles
/// are reported, never judged): which file paths sit in a reported
/// SCC, and how many reported SCCs hold at least one file. The floor
/// is the core's alone (`sccFloor`, Graph/Cost.hs) — the 2026-08-26
/// residue audit found this side re-deriving it as `sccSize > 1`,
/// with a third copy in the GUI: the dead-knob blind spot Cost.hs
/// exists to prevent (a raised floor moved the core report and the
/// score axis while the canvas kept counting two-node SCCs).
struct FileCycles {
    pub files: BTreeSet<String>,
    pub count: usize,
}

/// Decode `cycles` = [[sccId, [nodeIdx..]]]; a member index outside
/// our node list is a refusal, not a skip (F19: one id space).
fn file_cycles(reply: &Value, w: &GraphWire) -> Result<FileCycles> {
    let rows: Vec<(i64, Vec<i64>)> =
        serde_json::from_value(reply["cycles"].clone()).context("cycle rows")?;
    let (mut files, mut count) = (BTreeSet::new(), 0);
    for (_, members) in rows {
        let mut holds_file = false;
        for m in members {
            let node = usize::try_from(m)
                .ok()
                .and_then(|i| w.nodes.get(i))
                .context("cycle member echoes an index outside the node list")?;
            if node.kind == GRAN_FILE {
                files.insert(node.path.clone());
                holds_file = true;
            }
        }
        count += usize::from(holds_file);
    }
    Ok(FileCycles { files, count })
}

/// The document: files in file_nodes order, edges as index pairs into
/// that order, counts the header can print. Verdict/pos absence is
/// null, never a fabricated zero (the join's own rule).
///
/// `conf` rides beside the verdict because the graph family's trust
/// column (2.32.0) is part of the judgment, not decoration: the same
/// number decides whether `ce erase` may act on the row at all, and a
/// face that shows the verdict while hiding it shows the reader less
/// than the console does. `cycle` is the core's membership bit so
/// the GUI draws what the core reported instead of re-deriving it.
fn document(
    w: &GraphWire,
    report: &Report,
    pos: &HashMap<String, Pos>,
    cycles: &FileCycles,
) -> Value {
    let files = file_nodes(w);
    let dead: BTreeMap<&str, (&str, &str, Option<i64>)> = report
        .dead
        .iter()
        .map(|d| (d.path.as_str(), (d.verdict, d.why.as_str(), d.conf)))
        .collect();
    let rows: Vec<Value> = files
        .iter()
        .map(|&(_, p)| {
            let d = dead.get(p);
            json!({
                "path": p,
                "verdict": d.map(|&(v, _, _)| v),
                "why": d.map(|&(_, w, _)| w),
                "conf": d.and_then(|&(_, _, c)| c),
                "pos": pos.get(p),
                "cycle": cycles.files.contains(p),
            })
        })
        .collect();
    let edges = file_edges(w, &files);
    let n_edges = edges.len();
    json!({
        "schema": SCHEMA_ID,
        "files": rows,
        "edges": edges,
        "counts": {
            "files": files.len(),
            "edges": n_edges,
            "dead": report.dead.len(),
            "cycles": cycles.count,
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
    use super::super::wire::{GRAN_FILE, GRAN_PACKAGE, GRAN_SECTION};
    use super::*;
    use crate::testutil::node;

    /// Two files, one section of a.rs, one package; b.rs judged dead.
    fn fixture() -> (GraphWire, Report) {
        let nodes = vec![
            node("a.rs", "", GRAN_FILE),
            node("b.rs", "", GRAN_FILE),
            node("a.rs", "Intro", GRAN_SECTION),
            node("pkg", "", GRAN_PACKAGE),
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
            // the canvas draws the graph, it never reads the export
            // surface or the advisory — an empty table and the
            // road-not-asked None are this fixture's whole claim
            symbols: Default::default(),
            unmentioned: None,
            mounts: None,
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
            files: 4,
            kept: 3,
            unresolved_sites: 7,
            degraded: None,
            fail: true,
            unmentioned: None,
        };
        (w, report)
    }

    #[test]
    fn sections_collapse_packages_drop_and_cycles_count() {
        let (w, report) = fixture();
        let pos = HashMap::from([
            ("a.rs".into(), [1, 2, 0, 1, 0]),
            ("b.rs".into(), [2, 0, 4, 2, 1]),
        ]);
        // the core reports SCC 4 = {b.rs, pkg}; the canvas counts it
        // because it holds a file, and the row carries the membership
        let cycles = file_cycles(&json!({ "cycles": [[4, [1, 3]]] }), &w).expect("cycles");
        let doc = document(&w, &report, &pos, &cycles);
        assert_eq!(doc["edges"], json!([[0, 1]]));
        assert_eq!(doc["counts"]["files"], 2);
        assert_eq!(doc["counts"]["edges"], 1);
        assert_eq!(doc["counts"]["dead"], 1);
        assert_eq!(doc["counts"]["cycles"], 1);
        assert_eq!(doc["files"][0]["cycle"], false);
        assert_eq!(doc["files"][1]["cycle"], true);
        assert_eq!(doc["files"][0]["verdict"], Value::Null);
        assert_eq!(doc["files"][0]["pos"], json!([1, 2, 0, 1, 0]));
        // a live file carries no trust column either: absence is null,
        // never a fabricated 0, which on this scale means UNVOUCHED
        assert_eq!(doc["files"][0]["conf"], Value::Null);
        assert_eq!(doc["files"][1]["verdict"], "unref_private");
        assert_eq!(doc["files"][1]["why"], "no kept in-edge and no entry flag");
        assert_eq!(doc["files"][1]["conf"], 2);
        assert_eq!(doc["files"][1]["pos"], json!([2, 0, 4, 2, 1]));
        assert_eq!(doc["unresolvedSites"], 7);
        assert_eq!(doc["degraded"], Value::Null);
        assert_eq!(doc["schema"], SCHEMA_ID);
    }

    /// The floor is never re-derived here: a reported SCC counts iff
    /// it holds a FILE (a section-only SCC is the core's to report and
    /// this tier's to ignore), the degraded reply's empty list counts
    /// zero, and a member outside our node list is refused.
    #[test]
    fn file_cycles_take_the_core_report_and_restrict_it_to_files() {
        let (w, _) = fixture();
        let c = file_cycles(&json!({ "cycles": [[4, [1, 3]], [5, [2]]] }), &w).expect("cycles");
        assert_eq!(c.count, 1, "the section-only SCC is not a file-tier cycle");
        assert_eq!(c.files, BTreeSet::from(["b.rs".to_string()]));
        let none = file_cycles(&json!({ "cycles": [] }), &w).expect("degraded reply");
        assert_eq!((none.count, none.files.len()), (0, 0));
        assert!(file_cycles(&json!({ "cycles": [[0, [9]]] }), &w).is_err());
        assert!(
            file_cycles(&json!({}), &w).is_err(),
            "a reply without the key is malformed"
        );
    }
}
