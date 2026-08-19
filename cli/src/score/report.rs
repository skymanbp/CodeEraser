//! Check report faces (split from mod.rs at the 300-line dogfood
//! gate when the bilingual console landed, M8-G3b): the JSON
//! document and the console lines. Templates are data through
//! i18n::line — English bytes stay identical under the default,
//! CE_LANG=zh picks whole Chinese lines; FAIL/pass stay English in
//! both (exit-code vocabulary, not prose). JSON is never translated.

use super::{Outcome, SCHEMA_ID};
use crate::i18n::{line, t};
use serde_json::json;

/// The `--roast` easter egg: one verdict-flavored line per score
/// band, i18n-tabled like every console string. Bands are computed
/// against the EFFECTIVE scale, never a /1000 literal (the C17
/// discipline holds for jokes too). Console-only by contract —
/// JSON is the machine face and machines don't laugh.
pub fn roast_line(o: &Outcome) {
    let r = &o.reply;
    let scale = r.knobs.get("scoreScale").copied().unwrap_or(1000).max(1);
    let (en, zh) = match r.score * 1000 / scale {
        900.. => (
            "roast: suspiciously clean. who did you pay?",
            "毒舌：干净得可疑。你贿赂谁了？",
        ),
        750..=899 => (
            "roast: entropy is losing, slowly. keep the eraser warm.",
            "毒舌：熵在缓慢败退。橡皮别放凉。",
        ),
        600..=749 => (
            "roast: half garden, half landfill. bring the shovel.",
            "毒舌：半是花园半是垃圾场。铲子带上。",
        ),
        _ => (
            "roast: this repo needs an exorcist, not an eraser.",
            "毒舌：这仓库需要的不是橡皮，是驱魔师。",
        ),
    };
    println!("{}", t(en, zh));
}

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
