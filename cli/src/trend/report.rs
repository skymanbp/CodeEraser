//! Trend report faces (split from mod.rs at the 300-line dogfood
//! gate when the bilingual console landed, M8-G3b): the JSON
//! document and the console lines. Templates are data through
//! i18n::line — English bytes stay identical under the default,
//! CE_LANG=zh picks whole Chinese lines. JSON is never translated.

use super::{Report, SCHEMA_ID, judge};
use crate::i18n::line;
use serde_json::{Value, json};

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
    println!(
        "{}",
        line(
            "trend window: {} commits, {} measured, {} pending",
            "趋势窗口：{} 个提交，已测 {}，待测 {}",
            &[&r.window, &r.rows.len(), &r.pending],
        )
    );
}
