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
/// each file row carries `cycle`, the core's cycle membership. 0.4.0
/// (plan v2.25, O23): a dead row carries `whyCode` beside the English
/// `why`, so a face can render the reason in its own language.
pub const SCHEMA_ID: &str = "ce.graph-canvas/0.4.0";

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
    let dead: BTreeMap<&str, (&str, usize, Option<i64>)> = report
        .dead
        .iter()
        .map(|d| (d.path.as_str(), (d.verdict, d.why_code, d.conf)))
        .collect();
    let rows: Vec<Value> = files
        .iter()
        .map(|&(_, p)| {
            let d = dead.get(p);
            json!({
                "path": p,
                "verdict": d.map(|&(v, _, _)| v),
                "why": d.map(|&(_, w, _)| super::deadcode::WHY_CODES[w].0),
                "whyCode": d.map(|&(_, w, _)| w),
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
#[path = "../../tests/unit/graph/canvas.rs"]
mod tests;
