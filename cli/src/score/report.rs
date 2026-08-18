//! Check report faces (split from mod.rs at the 300-line dogfood
//! gate when the bilingual console landed, M8-G3b): the JSON
//! document and the console lines. Templates are data through
//! i18n::line — English bytes stay identical under the default,
//! CE_LANG=zh picks whole Chinese lines; FAIL/pass stay English in
//! both (exit-code vocabulary, not prose). JSON is never translated.

use super::{Outcome, SCHEMA_ID};
use crate::i18n::line;
use serde_json::json;

pub fn report_json(o: &Outcome) -> serde_json::Value {
    let r = &o.reply;
    json!({
        "schema": SCHEMA_ID,
        "score": r.score,
        // the denominator is a knob since P4 — a bare score was
        // unrecoverable for consumers (review C17)
        "scoreScale": r.knobs.get("scoreScale"),
        "axes": r.axes,
        "candidates": r.candidates,
        "ratchet": {
            "added": r.added, "removed": r.removed, "over": r.over,
            "toleranceDrawn": r.tolerance_drawn, "fail": r.fail,
            "failed": r.failed,
        },
        "counts": {
            "files": o.files, "simPairs": o.sim_pairs, "members": o.members,
            "collapsed": o.collapsed, "skippedSelf": o.skipped_self,
        },
        "degraded": r.degraded,
    })
}

pub fn print(o: &Outcome, as_json: bool) {
    if as_json {
        println!("{}", report_json(o));
        return;
    }
    let r = &o.reply;
    let axes: Vec<String> = r.axes.iter().map(|[c, p]| format!("{c}:{p}")).collect();
    // the effective scale, never the retired /1000 literal (C17)
    let scale = r.knobs.get("scoreScale").copied().unwrap_or(1000);
    println!(
        "{}",
        line(
            "check score {}/{} | axes {} | {} candidates",
            "检查分数 {}/{} | 判轴 {} | 候选 {}",
            &[&r.score, &scale, &axes.join(" "), &r.candidates.len()],
        )
    );
    print_ratchet_tail(o);
}

/// The ratchet / note / degraded lines (split at the 50-line fn gate).
fn print_ratchet_tail(o: &Outcome) {
    let r = &o.reply;
    let verdict = if r.fail { "FAIL" } else { "pass" };
    println!(
        "{}",
        line(
            "ratchet: {} added, {} removed, {} over, {} tolerance drawn -> {}",
            "棘轮：新增 {}，移除 {}，超限 {}，动用容差 {} -> {}",
            &[
                &r.added.len(),
                &r.removed.len(),
                &r.over.len(),
                &r.tolerance_drawn.len(),
                &verdict,
            ],
        )
    );
    if o.collapsed > 0 || o.skipped_self > 0 {
        println!(
            "{}",
            line(
                "note: {} blocks collapsed into existing members, {} intra-file pairs off the sim table",
                "注：{} 块并入既有成员，{} 个文件内对不入相似表",
                &[&o.collapsed, &o.skipped_self],
            )
        );
    }
    if let Some(reason) = &r.degraded {
        println!(
            "{}",
            line(
                "check degraded: {} -> FAIL (a gate that cannot judge must not pass)",
                "检查降级：{} -> FAIL（不能判决的门绝不放行）",
                &[reason],
            )
        );
    }
}
