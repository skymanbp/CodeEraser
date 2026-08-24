//! `ce join` (M5-3h): the three-signal assembly — similarity (clone
//! blocks), graph position (the SAME wire deadcode judges, answered
//! through graph.result's pos rows), and per-unit window churn —
//! joined into file-tier and unit-tier rows, each file pair judged
//! by the SAME verdict/1 lattice `ce check` gates with (2.33.0,
//! H4: one judgment, two faces — before that this face printed raw
//! legs only). Still report-only at the EXIT: candidates inform,
//! the fail bit never reads them, and nothing here thresholds.
//!
//! Tier F (files) carries all three legs and is what the lattice
//! will gate. Tier U (units) carries similarity + churn only; its
//! graph leg is null by design (churn_unit::GRAPH_CAVEAT).

pub mod churn_unit;
pub mod verdicts;

use crate::churn;
use crate::dedup::{self, pairs::Block};
use crate::graph::deadcode;
use crate::i18n::line;
use anyhow::{Context, Result};
use serde::Serialize;
use serde_json::{Value, json};
use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};

/// JSON output schema id; bump on shape change (plan §7.1).
/// 0.2.0 (2.33.0, H4): file rows carry the core's join verdict —
/// name, severity, leg-agreement confidence, legsMask, reasons —
/// from the SAME verdict/1 judgment `ce check` gates with.
pub const SCHEMA_ID: &str = "ce.join-report/0.2.0";

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
    /// (below the configured cochange floor) — unknown-small, not zero.
    pub cochange: Option<usize>,
    /// The core's verdict for this pair (2.33.0, H4) — rendering
    /// vocabulary for verdict/1's candidate row. None for self-pairs
    /// (the wire's u < v contract cannot carry them — the same
    /// "intra-file pairs off the sim table" class the check report
    /// counts) and when the judgment degraded.
    pub verdict: Option<&'static str>,
    /// The verdict's severity rank from the core's table face.
    pub severity: Option<i64>,
    /// Leg-agreement confidence: how many present legs corroborate.
    pub confidence: Option<i64>,
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
    // one snapshot (batch 9 P10): blocks and wire from ONE walk
    let (found, idx, db_path) = dedup::snapshot(root, db)?;
    let w = deadcode::wire_of(root, &idx, &db_path)?;
    drop(idx);
    let pos_req: Vec<i64> = deadcode::file_nodes(&w).iter().map(|&(i, _)| i).collect();
    let reply = deadcode::judge(core, &w, &pos_req)?;
    // Degrade reads the wire's boolean, not reason presence (the C9
    // discipline; same throat shape as deadcode::consume).
    let degraded = (reply["degraded"].as_bool() == Some(true))
        .then(|| reply["reason"].as_str().unwrap_or("degraded").to_string());
    let posmap = pos_map(&reply, &w)?;
    let ch = churn::run(root, days)?;
    // the judgment leg (2.33.0, H4): the same verdict/1 road the
    // check gate uses, over this run's own measurement
    let judged = verdicts::judge_pairs(root, core, &w, &found.blocks, &posmap, &ch)?;
    let degraded = degraded.or(judged.degraded);
    let mut files = file_rows(&found.blocks, &posmap, &ch);
    for f in &mut files {
        if let Some(v) = judged.pairs.get(&(f.a.clone(), f.b.clone())) {
            f.verdict = Some(v.verdict);
            f.severity = Some(v.severity);
            f.confidence = Some(v.confidence);
        }
    }
    Ok(Report {
        days,
        commits: ch.commits,
        files,
        units: churn_unit::rows(root, &found.blocks, &ch),
        degraded,
    })
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
            verdict: None,
            severity: None,
            confidence: None,
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
    crate::report::print_doc(as_json, || report_json(r), || print_console(r));
}

fn print_console(r: &Report) {
    for f in &r.files {
        println!(
            "{}",
            line(
                "join {} <-> {}: {} blocks / {} tokens | graph {} | {} | churn +{}/~{} | +{}/~{} | cochange {} | {}",
                "联判 {} <-> {}：{} 块 / {} tokens | 图 {} | {} | 改动 +{}/~{} | +{}/~{} | 共变 {} | {}",
                &[
                    &f.a,
                    &f.b,
                    &f.blocks,
                    &f.tokens,
                    &pos_str(f.graph_a),
                    &pos_str(f.graph_b),
                    &f.churn_a.appended,
                    &f.churn_a.rewrote,
                    &f.churn_b.appended,
                    &f.churn_b.rewrote,
                    &f.cochange.map_or_else(|| "-".into(), |n| n.to_string()),
                    &verdict_str(f),
                ],
            )
        );
    }
    print_unit_tail(r);
}

/// Console tail for a row's core verdict — rendering only, codes
/// and ranks are the core's (2.33.0, H4). A self-pair is named in
/// the check report's own vocabulary: the wire's u < v contract
/// cannot carry it, so no candidate row exists to relay.
fn verdict_str(f: &FileRow) -> String {
    match (f.verdict, f.severity, f.confidence) {
        (Some(v), Some(s), Some(c)) => format!("{v} (sev {s}, conf {c})"),
        _ if f.a == f.b => "self-pair (off the sim table)".into(),
        _ => "unjudged".into(),
    }
}

/// The unit rows + degraded note + window summary (split at the
/// 50-line fn gate when the bilingual console landed, M8-G3b).
fn print_unit_tail(r: &Report) {
    for u in &r.units {
        println!(
            "{}",
            line(
                "unit {}#{}~{} <-> {}#{}~{}: {} tokens | churn +{}/~{} | +{}/~{} | graph null (R6 locked)",
                "单元 {}#{}~{} <-> {}#{}~{}：{} tokens | 改动 +{}/~{} | +{}/~{} | 图 null（R6 锁定）",
                &[
                    &u.a.path,
                    &u.a.key,
                    &u.a.nth,
                    &u.b.path,
                    &u.b.key,
                    &u.b.nth,
                    &u.tokens,
                    &u.churn_a.appended,
                    &u.churn_a.rewrote,
                    &u.churn_b.appended,
                    &u.churn_b.rewrote,
                ],
            )
        );
    }
    if let Some(reason) = &r.degraded {
        println!(
            "{}",
            line(
                "join graph leg degraded: {}",
                "联判图信号腿已降级：{}",
                &[reason],
            )
        );
    }
    println!(
        "{}",
        line(
            "join {}d window: {} file pairs, {} unit rows, {} commits (verdicts by the check lattice; exit stays report-only)",
            "联判 {} 天窗口：{} 文件对，{} 单元行，{} 提交（判决出自 check 判决格；退出码仍仅报告）",
            &[&r.days, &r.files.len(), &r.units.len(), &r.commits],
        )
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
