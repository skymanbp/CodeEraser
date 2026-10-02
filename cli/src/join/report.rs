//! The join document read back (plan v2.32 step 4): the core lays it
//! out (join/document.rs, CE.Join.Document) and this face prints it —
//! the machine faces the bound document itself, the console its lines.
//! Rendering only: every code and rank here is the core's or the
//! measurement's, and the words are this face's (ADR-008).

use super::Pos;
use super::churn_unit::{self, Lines};
use crate::i18n::line;
use serde::Deserialize;
use serde_json::Value;

/// The document as the console and the readers take it; `doc` is the
/// document itself.
#[derive(Debug, Deserialize)]
pub struct Report {
    pub days: u32,
    pub commits: usize,
    pub files: Vec<FileRow>,
    pub units: Vec<churn_unit::UnitRow>,
    pub degraded: Option<String>,
    #[serde(skip)]
    pub doc: Value,
}

/// One Tier F row: a similar file pair with all three legs and the
/// core's verdict on it.
#[derive(Debug, Deserialize)]
pub struct FileRow {
    pub a: String,
    pub b: String,
    pub blocks: usize,
    pub tokens: usize,
    /// T3 near-miss unit pairs between the two files (5b-9).
    pub near_miss: usize,
    pub graph_a: Option<Pos>,
    pub graph_b: Option<Pos>,
    pub churn_a: Lines,
    pub churn_b: Lines,
    /// None = the pair is outside the churn report's co-change table.
    pub cochange: Option<usize>,
    /// The core's verdict for this pair (2.33.0, H4); None for a
    /// self-pair (the wire's u < v contract cannot carry it) and when
    /// the judgment degraded.
    pub verdict: Option<String>,
    pub severity: Option<i64>,
    pub confidence: Option<i64>,
}

crate::report::bound!(Report, print_console);

fn print_console(r: &Report) {
    for f in &r.files {
        println!(
            "{}",
            line(
                "join {} <-> {}: {} blocks / {} tokens / {} near-miss | graph {} | {} | churn +{}/~{} | +{}/~{} | cochange {} | {}",
                "联判 {} <-> {}：{} 块 / {} tokens / {} 近似对 | 图 {} | {} | 改动 +{}/~{} | +{}/~{} | 共变 {} | {}",
                &[
                    &f.a,
                    &f.b,
                    &f.blocks,
                    &f.tokens,
                    &f.near_miss,
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
    match (&f.verdict, f.severity, f.confidence) {
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
                "unit {}#{}~{} <-> {}#{}~{}: {} | churn +{}/~{} | +{}/~{} | graph null (R6 locked)",
                "单元 {}#{}~{} <-> {}#{}~{}：{} | 改动 +{}/~{} | +{}/~{} | 图 null（R6 锁定）",
                &[
                    &u.a.path,
                    &u.a.key,
                    &u.a.nth,
                    &u.b.path,
                    &u.b.key,
                    &u.b.nth,
                    &sim_str(u.sim),
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

/// The unit row's similarity in its family's own words (5b-9).
fn sim_str(sim: churn_unit::UnitSim) -> String {
    match sim {
        churn_unit::UnitSim::T1t2 { tokens } => line("{} tokens", "{} tokens", &[&tokens]),
        churn_unit::UnitSim::T3 { ted, n1, n2 } => line(
            "t3 ted {} (nodes {}/{})",
            "t3 ted {}（节点 {}/{}）",
            &[&ted, &n1, &n2],
        ),
    }
}

fn pos_str(p: Option<Pos>) -> String {
    match p {
        Some([indeg, outdeg, scc, size, reach]) => {
            format!("in{indeg} out{outdeg} scc{scc}x{size} reach{reach}")
        }
        None => "null".into(),
    }
}
