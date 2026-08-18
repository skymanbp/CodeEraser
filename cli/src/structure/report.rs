//! Structure report faces (split from judge.rs at the 300-line
//! dogfood gate when the bilingual console landed, M8-G3b): the JSON
//! document and the console lines. Templates are data through
//! i18n::line — English bytes stay identical under the default,
//! CE_LANG=zh picks whole Chinese lines. JSON is never translated.

use super::judge::{Report, SCHEMA_ID};
use crate::i18n::{line, t};
use serde::Serialize;

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
    tree: Vec<serde_json::Value>,
}

/// The ONE report document (§5: CLI JSON and the GUI consume the
/// SAME shape — a second report form is exactly what the booklet
/// forbids). Public since S4a: the Tauri command calls this.
pub fn report_json(r: &Report) -> serde_json::Value {
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
        tree: r
            .tree
            .iter()
            .enumerate()
            .map(|(i, n)| {
                serde_json::json!({
                    "id": i, "parent": n.parent, "name": n.name,
                    "depth": n.depth, "subdirs": n.subdirs,
                    "files": n.files, "axes": n.axes,
                })
            })
            .collect(),
    };
    serde_json::to_value(&doc).expect("report json")
}

pub fn print(r: &Report, as_json: bool) {
    if as_json {
        println!("{}", report_json(r));
        return;
    }
    let axes: Vec<String> = r.axes.iter().map(|[c, p]| format!("{c}:{p}")).collect();
    let ent: Vec<String> = r.entropy.iter().map(|[k, v]| format!("{k}:{v}")).collect();
    println!(
        "{}",
        line(
            "structure score {}/{} | entropy {} | axes {} | {} dirs",
            "结构分数 {}/{} | 熵 {} | 判轴 {} | {} 目录",
            &[&r.score, &r.scale, &ent.join(" "), &axes.join(" "), &r.dirs],
        )
    );
    if r.declared > 0 {
        match r.divergence {
            Some(d) => println!(
                "{}",
                line(
                    "layout divergence {}‰ over {} declared dirs",
                    "布局偏离 {}‰（{} 个已声明目录）",
                    &[&d, &r.declared],
                )
            ),
            None => println!(
                "{}",
                line(
                    "layout divergence undefined: mass outside the {} declared dirs",
                    "布局偏离未定义：质量落在 {} 个已声明目录之外",
                    &[&r.declared],
                )
            ),
        }
    }
    print_findings_tail(r);
}

/// The deviation + finding lines (split at the 50-line fn gate).
fn print_findings_tail(r: &Report) {
    for (dir, kind) in &r.deviations {
        let label = if *kind == 0 {
            t("undeclared territory", "未声明领地")
        } else {
            t("declared but empty", "已声明但为空")
        };
        println!(
            "{}",
            line("deviation {}  {}", "偏离 {}  {}", &[dir, &label])
        );
    }
    for (dir, axis) in &r.findings {
        println!(
            "{}",
            line("finding {}  axis {}", "发现 {}  判轴 {}", &[dir, axis])
        );
    }
}
