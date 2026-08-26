//! The join report's two emitters — the JSON document and the
//! bilingual console rendering — split from mod.rs at the repo's own
//! 300-line dogfood gate when the progress spans landed (plan v2.16).
//! churn/ and trend/ have had this leaf since their own 300-line
//! walls; join was the family that never got one, so the assembly
//! and the faces that render it had shared a file since M5-3h.
//!
//! Rendering only: every code and rank here is the core's or the
//! measurement's, and the words are this face's (ADR-008).

use super::{FileRow, Pos, Report, SCHEMA_ID, churn_unit};
use crate::i18n::line;
use serde_json::{Value, json};

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
            // the CODE, not the sentence (plan v2.15): the reader
            // that renders this row owns the words for it
            "caveatCode": churn_unit::GRAPH_NULL_IMPORT_GRANULARITY,
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
