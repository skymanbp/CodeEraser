//! `ce join` (M5-3h): the three-signal assembly — similarity (clone
//! blocks), graph position (the SAME wire deadcode judges, answered
//! through graph.result's pos rows), and per-unit window churn —
//! joined into file-tier and unit-tier rows. REPORT-ONLY: the
//! verdict lattice (CE/Verdict/Join.hs, design §6.3) exists and is
//! battery-tested, but its wire hookup is 3i; nothing here
//! thresholds or gates.
//!
//! Tier F (files) carries all three legs and is what the lattice
//! will gate. Tier U (units) carries similarity + churn only; its
//! graph leg is null by design (churn_unit::GRAPH_CAVEAT).

pub mod churn_unit;

use crate::churn;
use crate::dedup::{self, pairs::Block};
use crate::graph::deadcode;
use crate::graph::wire::GRAN_FILE;
use anyhow::{Context, Result};
use serde::Serialize;
use serde_json::{Value, json};
use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};

/// JSON output schema id; bump on shape change (plan §7.1).
pub const SCHEMA_ID: &str = "ce.join-report/0.1.0";

/// Graph position of one file: [indeg, outdeg, sccId, sccSize,
/// reachIn] (the Position.hs row minus its echoed index). None =
/// the graph could not answer (degraded reply, or the file fell off
/// the graph between passes) — absence, never a fabricated zero.
pub type Pos = [i64; 5];

/// One Tier F row: a similar file pair with all three legs.
#[derive(Debug, Serialize)]
pub struct FileRow {
    pub a: String,
    pub b: String,
    pub blocks: usize,
    pub tokens: usize,
    pub graph_a: Option<Pos>,
    pub graph_b: Option<Pos>,
    pub churn_a: churn_unit::Lines,
    pub churn_b: churn_unit::Lines,
    /// None = the pair is outside the churn report's co-change table
    /// (top 20, count >= 2) — unknown-small, not zero.
    pub cochange: Option<usize>,
}

#[derive(Debug)]
pub struct Report {
    pub days: u32,
    pub commits: usize,
    pub files: Vec<FileRow>,
    pub units: Vec<churn_unit::UnitRow>,
    pub degraded: Option<String>,
}

pub fn run(root: &Path, db: Option<PathBuf>, core: &str, days: u32) -> Result<Report> {
    let (found, _summary) = dedup::analyze(root, db.clone(), None, None)?;
    let w = deadcode::build_wire(root, db)?;
    let pos_req: Vec<i64> = w
        .nodes
        .iter()
        .enumerate()
        .filter(|(_, n)| n.kind == GRAN_FILE)
        .map(|(i, _)| i as i64)
        .collect();
    let reply = deadcode::judge(core, &w, &pos_req)?;
    let degraded = reply["reason"].as_str().map(str::to_string);
    let posmap = pos_map(&reply, &w)?;
    let ch = churn::run(root, days)?;
    Ok(Report {
        days,
        commits: ch.commits,
        files: file_rows(&found.blocks, &posmap, &ch),
        units: churn_unit::rows(root, &found.blocks, &ch),
        degraded,
    })
}

/// path → position from the reply's pos rows; each row's echoed
/// index names a node in OUR dense assignment (F19: one id space),
/// so an out-of-range echo is a refusal, not a skip.
fn pos_map(reply: &Value, w: &deadcode::GraphWire) -> Result<HashMap<String, Pos>> {
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

/// Tier F: aggregate blocks per unordered file pair, attach both
/// sides' graph position and window churn, and the co-change count
/// where the pair made the churn report's table.
fn file_rows(blocks: &[Block], posmap: &HashMap<String, Pos>, ch: &churn::Report) -> Vec<FileRow> {
    let mut by_pair: BTreeMap<(String, String), (usize, usize)> = BTreeMap::new();
    for b in blocks {
        let key = if b.a_file <= b.b_file {
            (b.a_file.clone(), b.b_file.clone())
        } else {
            (b.b_file.clone(), b.a_file.clone())
        };
        let e = by_pair.entry(key).or_default();
        e.0 += 1;
        e.1 += b.tokens;
    }
    let by_file = file_churn(ch);
    by_pair
        .into_iter()
        .map(|((a, b), (blocks, tokens))| FileRow {
            graph_a: posmap.get(&a).copied(),
            graph_b: posmap.get(&b).copied(),
            churn_a: by_file.get(&a).copied().unwrap_or_default(),
            churn_b: by_file.get(&b).copied().unwrap_or_default(),
            cochange: ch
                .cochange
                .iter()
                .find(|(x, y, _)| *x == a && *y == b)
                .map(|(_, _, n)| *n),
            a,
            b,
            blocks,
            tokens,
        })
        .collect()
}

/// Per-file churn sums over the per-unit ledger — derived the same
/// way the report totals are (one bookkeeping, summed two ways).
fn file_churn(ch: &churn::Report) -> HashMap<String, churn_unit::Lines> {
    let mut map: HashMap<String, churn_unit::Lines> = HashMap::new();
    for u in &ch.units {
        let e = map.entry(u.path.clone()).or_default();
        e.appended += u.appended;
        e.rewrote += u.rewrote;
    }
    map
}

pub fn report_json(r: &Report) -> Value {
    json!({
        "schema": SCHEMA_ID,
        "days": r.days,
        "commits": r.commits,
        "degraded": r.degraded,
        "files": r.files,
        "units": r.units.iter().map(|u| json!({
            "a": u.a, "b": u.b, "tokens": u.tokens,
            "churn_a": u.churn_a, "churn_b": u.churn_b,
            "graph": Value::Null,
            "caveat": churn_unit::GRAPH_CAVEAT,
        })).collect::<Vec<_>>(),
    })
}

pub fn print(r: &Report, as_json: bool) {
    if as_json {
        println!("{}", report_json(r));
        return;
    }
    for f in &r.files {
        println!(
            "join {} <-> {}: {} blocks / {} tokens | graph {} | {} | churn +{}/~{} | +{}/~{} | cochange {}",
            f.a,
            f.b,
            f.blocks,
            f.tokens,
            pos_str(f.graph_a),
            pos_str(f.graph_b),
            f.churn_a.appended,
            f.churn_a.rewrote,
            f.churn_b.appended,
            f.churn_b.rewrote,
            f.cochange.map_or_else(|| "-".into(), |n| n.to_string()),
        );
    }
    for u in &r.units {
        println!(
            "unit {}#{}~{} <-> {}#{}~{}: {} tokens | churn +{}/~{} | +{}/~{} | graph null (R6 locked)",
            u.a.path,
            u.a.key,
            u.a.nth,
            u.b.path,
            u.b.key,
            u.b.nth,
            u.tokens,
            u.churn_a.appended,
            u.churn_a.rewrote,
            u.churn_b.appended,
            u.churn_b.rewrote,
        );
    }
    if let Some(reason) = &r.degraded {
        println!("join graph leg degraded: {reason}");
    }
    println!(
        "join {}d window: {} file pairs, {} unit rows, {} commits (report-only; verdict lattice lands in 3i)",
        r.days,
        r.files.len(),
        r.units.len(),
        r.commits
    );
}

fn pos_str(p: Option<Pos>) -> String {
    match p {
        Some([indeg, outdeg, scc, size, reach]) => {
            format!("in{indeg} out{outdeg} scc{scc}x{size} reach{reach}")
        }
        None => "null".into(),
    }
}
