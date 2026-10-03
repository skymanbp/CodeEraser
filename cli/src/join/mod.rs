//! `ce join` (M5-3h): the three-signal assembly — similarity (clone
//! blocks and, since plan v2.30 step 5b-9, T3 near-miss pairs), graph position (the SAME wire deadcode judges, answered
//! through graph.result's pos rows), and per-unit window churn —
//! joined into file-tier and unit-tier rows, each file pair judged
//! by the SAME verdict/1 lattice `ce check` gates with (2.33.0,
//! H4: one judgment, two faces — before that this face printed raw
//! legs only). Still report-only at the EXIT: candidates inform,
//! the fail bit never reads them, and nothing here thresholds.
//!
//! Tier F (files) carries all three legs and is what the lattice
//! will gate. Tier U (units) carries similarity + churn only; its
//! graph leg is null by design, with the reason riding as a code
//! the core names (CE.Join.Document, plan v2.15). The core lays the
//! document out (document/1, plan v2.32 step 4; document.rs sends
//! the tables) and its console lines (step 5, CE.Join.Lines); `Report`
//! is that document read back for the library's callers.

pub mod churn_unit;
mod document;
mod report;
pub mod verdicts;

pub use report::{FileRow, Report};

use crate::document::Answer;

use crate::churn;
use crate::dedup;
use crate::graph::deadcode;
use anyhow::{Context, Result};
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Graph position of one file: [indeg, outdeg, sccId, sccSize,
/// reachIn] (the Position.hs row minus its echoed index). None =
/// the graph could not answer (degraded reply, or the file fell off
/// the graph between passes) — absence, never a fabricated zero.
pub type Pos = [i64; 5];

pub fn run(root: &Path, db: Option<PathBuf>, core: &str, days: u32) -> Result<Answer> {
    // 265.0 s end to end on a cold db (PERF-BUDGET M5-3h), most of it
    // the churn leg's own — which brings its own span. These three
    // name the legs around it, so the quiet stretches before and
    // after churn are not read as a hang either (plan v2.16).
    let _span = crate::progress::span();
    // one snapshot (batch 9 P10): blocks and wire from ONE walk
    crate::progress::step(crate::progress::Phase::Index);
    let (found, idx, db_path) = dedup::snapshot(root, db)?;
    crate::progress::step(crate::progress::Phase::Graph);
    // the join reads positions and the degraded bit alone: the
    // symbol advisory has no seat in its lattice (W4-F17)
    let w = deadcode::wire_of(root, &idx, &db_path, deadcode::Advisory::No)?;
    // the T3 family off the same snapshot (plan v2.30 step 5b-9): its
    // pairs are sim rows of kind 1, file rows and unit rows here
    let t3 = dedup::t3::judge_index(root, &idx, core)?;
    drop(idx);
    let sim = crate::score::Similar {
        blocks: &found.blocks,
        t3: &t3,
    };
    let pos_req: Vec<i64> = deadcode::measured_nodes(&w)
        .iter()
        .map(|&(i, _)| i)
        .collect();
    let reply = deadcode::judge(core, &w, &pos_req)?;
    // Degrade reads the wire's boolean, not reason presence (the C9
    // discipline; same throat shape as deadcode::consume).
    let graph_degraded = (reply["degraded"].as_bool() == Some(true))
        .then(|| reply["reason"].as_str().unwrap_or("degraded").to_string());
    let posmap = pos_map(&reply, &w)?;
    // the self-loop projection (6.4.0): the verdict road needs it at
    // floor 1, and this face judges over the same graph reply
    let loops = deadcode::self_loop_rows(&w, &deadcode::self_loop_nodes(&reply)?);
    let ch = churn::run(root, days)?;
    // the judgment leg (2.33.0, H4): the same verdict/1 road the
    // check gate uses, over this run's own measurement
    crate::progress::step(crate::progress::Phase::Assemble);
    let mut judged = verdicts::judge_pairs(root, core, &w, &sim, (&posmap, loops), &ch)?;
    let held = std::mem::replace(&mut judged.held, Err(String::new()));
    let parts = document::Parts {
        days,
        churn: &ch,
        pairs: &pair_sums(&sim),
        units: &churn_unit::rows(root, &sim, &ch),
        posmap: &posmap,
        graph_degraded,
        judged: &judged,
    };
    document::assemble(core, held, &parts)
}

/// path → position from the reply's pos rows; each row's echoed
/// index names a node in OUR dense assignment (F19: one id space),
/// so an out-of-range echo is a refusal, not a skip. pub(crate):
/// the score assembly reads the same map (one decoding, 3i).
pub(crate) fn pos_map(reply: &Value, w: &deadcode::GraphWire) -> Result<HashMap<String, Pos>> {
    let rows: Vec<[i64; 6]> = serde_json::from_value(reply["pos"].clone()).context("pos rows")?;
    let mut map = HashMap::new();
    for [p, indeg, outdeg, scc_id, scc_size, reach_in] in rows {
        let node = usize::try_from(p)
            .ok()
            .and_then(|i| w.nodes.get(i))
            .context("pos row echoes an index outside the node list")?;
        map.insert(
            node.path.clone(),
            [indeg, outdeg, scc_id, scc_size, reach_in],
        );
    }
    Ok(map)
}

/// Tier F's sums: blocks and T3 pairs per unordered file pair (the
/// smaller path first).
fn pair_sums(sim: &crate::score::Similar<'_>) -> document::Sums {
    let ordered = |a: &str, b: &str| {
        if a <= b {
            (a.to_string(), b.to_string())
        } else {
            (b.to_string(), a.to_string())
        }
    };
    let mut by_pair = document::Sums::new();
    for b in sim.blocks {
        let e = by_pair.entry(ordered(&b.a_file, &b.b_file)).or_default();
        e.0 += 1;
        e.1 += b.tokens;
    }
    for (a, b) in sim.t3.file_pairs() {
        by_pair.entry(ordered(a, b)).or_default().2 += 1;
    }
    by_pair
}
