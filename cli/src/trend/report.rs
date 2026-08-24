//! Trend report faces (split from mod.rs at the 300-line dogfood
//! gate when the bilingual console landed, M8-G3b): the JSON
//! document and the console lines. Templates are data through
//! i18n::line — English bytes stay identical under the default,
//! CE_LANG=zh picks whole Chinese lines. JSON is never translated.

use super::judge;
use super::judge::Row;
use crate::i18n::line;
use serde_json::{Value, json};

/// JSON output schema id; bump on shape change (plan §7.1) — the
/// report owns its own schema stamp.
/// 0.2.0 (M7.5b): the report carries the core's trend/1 judgment.
/// 0.3.0 (2.31.0): the judgment carries trend/2's cliff and
/// declineRun shape facts beside the robust slope.
pub const SCHEMA_ID: &str = "ce.trend-report/0.3.0";

#[derive(Debug)]
pub struct Report {
    /// Mainline commits found inside the requested window.
    pub window: usize,
    /// Measured rows, oldest first (chart order).
    pub rows: Vec<Row>,
    /// Window commits still unmeasured after this batch (includes
    /// this run's failures — they retry next run).
    pub pending: usize,
    /// (short sha, reason) for commits that refused to measure this
    /// run — reported, never silently absent.
    pub failed: Vec<(String, String)>,
    /// The core's trend/2 judgment over the window (M7.5b; robust
    /// since 2.31.0).
    pub judgment: judge::Judgment,
}

pub fn report_json(r: &Report) -> Value {
    json!({
        "schema": SCHEMA_ID,
        "window": r.window,
        "pending": r.pending,
        "rows": r.rows,
        "failed": r.failed.iter().map(|(s, w)| json!([s, w])).collect::<Vec<_>>(),
        "judgment": judge::judgment_json(&r.judgment),
    })
}

pub fn print(r: &Report, as_json: bool) {
    crate::report::print_doc(as_json, || report_json(r), || print_console(r));
}

fn print_console(r: &Report) {
    for row in &r.rows {
        let axes: Vec<String> = row.axes.iter().map(|[c, p]| format!("{c}:{p}")).collect();
        println!(
            "{}",
            line(
                "trend {} {} score {}/{} | axes {}",
                "趋势 {} {} 分数 {}/{} | 判轴 {}",
                &[
                    &&row.commit[..12],
                    &row.ts,
                    &row.score,
                    &row.scale,
                    &axes.join(" ")
                ],
            )
        );
    }
    for (sha, why) in &r.failed {
        println!(
            "{}",
            line("trend {} FAILED: {}", "趋势 {} 失败：{}", &[sha, why])
        );
    }
    print_verdict_tail(r);
}

/// The judgment + window summary lines (split at the 50-line fn gate).
fn print_verdict_tail(r: &Report) {
    let j = &r.judgment;
    // per-mille of the score scale per day: the rows above print
    // score/scale, so the slope reads in the same currency (batch 9
    // P15) — the JSON keeps slopeMicroPerDay verbatim
    let slope = j
        .slope_micro_per_day
        .map(|s| {
            let pm = format!("{:.1}", s as f64 / 1000.0);
            line(" (slope {}‰/day)", "（斜率 {}‰/日）", &[&pm])
        })
        .unwrap_or_default();
    let fail_tail = if j.fail { " -> FAIL" } else { "" };
    println!(
        "{}",
        line(
            "trend verdict: {}{}{}",
            "趋势判决：{}{}{}",
            &[&judge::verdict_str(j), &slope, &fail_tail],
        )
    );
    print_shape_facts(r);
    println!(
        "{}",
        line(
            "trend window: {} commits, {} measured, {} pending",
            "趋势窗口：{} 个提交，已测 {}，待测 {}",
            &[&r.window, &r.rows.len(), &r.pending],
        )
    );
}

/// The trend/2 shape facts, rendered with the commit each request
/// index points at — judge::judge fenced the indices against the
/// rows it sent, so the lookup is total here (the #{} fallback only
/// guards hand-built reports in tests).
fn print_shape_facts(r: &Report) {
    let name = |idx: i64| {
        r.rows
            .get(idx as usize)
            .map(|row| row.commit[..12].to_string())
            .unwrap_or_else(|| format!("#{idx}"))
    };
    if let Some([idx, drop]) = r.judgment.cliff {
        let pm = format!("{:.1}", drop as f64 / 1000.0);
        println!(
            "{}",
            line(
                "trend cliff: -{}‰ into {}",
                "趋势断崖：-{}‰ 落在 {}",
                &[&pm, &name(idx)],
            )
        );
    }
    if let Some([start, count]) = r.judgment.decline_run {
        println!(
            "{}",
            line(
                "trend decline run: {} commits from {}",
                "趋势持续下行：{} 个提交，起于 {}",
                &[&count, &name(start)],
            )
        );
    }
}
