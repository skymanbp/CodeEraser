//! The graph screen's document (plan v2.30 step 5b item 31; laid out
//! by the core since plan v2.32 step 4, CE.Graph.Screen): the canvas
//! the GUI draws and the deadcode document off ONE judgment. One
//! refreshed index, the deadcode face's own wire (the advisory and the
//! dead-file gate, plan v2.35 5C), one graph reply with the full
//! file-tier position request; this side sends the deadcode face's
//! request — its tables, the grouping and the gate, so the screen's
//! deadcode half is the document `ce deadcode --format json` prints for
//! the same tree — and, beside it, each node's
//! kind and the canvas row its path names, the wire's edges, the
//! reply's positions and its cycle members — the core collapses the
//! edges onto files, places the verdicts and counts the cycles that
//! hold a file (RG9: cycles are reported, never judged).

use super::deadcode::{self, GraphWire, Judged, file_nodes};
use anyhow::{Context, Result};
use serde_json::Value;
use std::collections::HashMap;

pub fn screen(root: &std::path::Path, core: &str) -> Result<Value> {
    let w = deadcode::face_wire(root, None)?;
    let pos_req: Vec<i64> = file_nodes(&w).iter().map(|x| x.0).collect();
    let (judged, reply, held) = deadcode::judged(root, core, &w, &pos_req)?;
    document((core, held), &w, &judged, &reply)
}

/// The screen's document over one judgment: the deadcode tables and,
/// beside them, the canvas's — laid out by the core.
fn document(
    (core, held): (&str, crate::document::Held),
    w: &GraphWire,
    judged: &Judged,
    reply: &Value,
) -> Result<Value> {
    let req = deadcode::face_request(("graphscreen", false), w, judged)?;
    let pos: Vec<[i64; 6]> = serde_json::from_value(reply["pos"].clone()).context("pos rows")?;
    let req = req
        .rows("graph", graph_rows(w))
        .rows(
            "edges",
            w.edges.iter().map(|e| [e[0], e[1]]).collect::<Vec<_>>(),
        )
        .rows("pos", pos)
        .rows("cycles", cycle_rows(reply)?);
    crate::document::assemble_over(core, held, req).map(|a| a.document)
}

/// [node, kind, canvas row]: the canvas rows are the file nodes in
/// node order, and a node's row is the one its path names (a section
/// stands for the file that holds it; −1 = none).
fn graph_rows(w: &GraphWire) -> Vec<[i64; 3]> {
    let row: HashMap<&str, i64> = file_nodes(w)
        .iter()
        .enumerate()
        .map(|(r, &(_, p))| (p, r as i64))
        .collect();
    w.nodes
        .iter()
        .enumerate()
        .map(|(i, n)| {
            [
                i as i64,
                n.kind,
                row.get(n.path.as_str()).copied().unwrap_or(-1),
            ]
        })
        .collect()
}

/// `cycles` = [[sccId, [nodeIdx..]]] flattened to [sccId, member].
fn cycle_rows(reply: &Value) -> Result<Vec<[i64; 2]>> {
    Ok(deadcode::cycles::cycle_table(reply)?
        .into_iter()
        .flat_map(|(c, members)| members.into_iter().map(move |m| [c, m]))
        .collect())
}

#[cfg(test)]
#[path = "../../tests/unit/graph/canvas.rs"]
mod tests;
