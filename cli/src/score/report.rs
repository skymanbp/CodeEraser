//! Check report faces (split from mod.rs at the 300-line dogfood
//! gate when the bilingual console landed, M8-G3b): the document the
//! core lays out (score/document.rs, plan v2.32 step 4) printed as
//! is, and the console lines read off it. Templates are data through
//! i18n::line — English bytes stay identical under the default,
//! CE_LANG=zh picks whole Chinese lines; FAIL/pass stay English in
//! both (exit-code vocabulary, not prose). JSON is never translated.

use crate::i18n::{line, t};
use crate::report::colon_pairs;
use crate::score::document::Report;

/// The `--roast` easter egg: one verdict-flavored line per score
/// band, i18n-tabled like every console string. Bands are computed
/// against the EFFECTIVE scale, never a /1000 literal (the C17
/// discipline holds for jokes too). Console-only by contract —
/// JSON is the machine face and machines don't laugh.
pub fn roast_line(r: &Report) {
    let scale = r.scale.unwrap_or(1000).max(1);
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

crate::report::bound!(Report, print_console);

fn print_console(r: &Report) {
    // the effective scale, never the retired /1000 literal (C17)
    let scale = r.scale.unwrap_or(1000);
    println!(
        "{}",
        line(
            "check score {}/{} | axes {} | {} candidates",
            "检查分数 {}/{} | 判轴 {} | 候选 {}",
            &[&r.score, &scale, &colon_pairs(&r.axes), &r.candidates.len()],
        )
    );
    print_ratchet_tail(r);
}

/// The ratchet / note / degraded lines (split at the 50-line fn gate).
/// A FAIL names what held (report::fail_suffix, O36); pass bytes are
/// the ones this line always printed.
fn print_ratchet_tail(r: &Report) {
    let k = &r.ratchet;
    let verdict = if k.fail {
        format!("FAIL{}", crate::report::fail_suffix(&k.failed))
    } else {
        "pass".to_string()
    };
    println!(
        "{}",
        line(
            "ratchet: {} added, {} removed, {} over, {} tolerance drawn -> {}",
            "棘轮：新增 {}，移除 {}，超限 {}，动用容差 {} -> {}",
            &[
                &k.added.len(),
                &k.removed.len(),
                &k.over.len(),
                &k.tolerance_drawn.len(),
                &verdict,
            ],
        )
    );
    let c = &r.counts;
    if c.collapsed > 0 || c.skipped_self > 0 {
        println!(
            "{}",
            line(
                "note: {} blocks collapsed into existing members, {} intra-file pairs off the sim table",
                "注：{} 块并入既有成员，{} 个文件内对不入相似表",
                &[&c.collapsed, &c.skipped_self],
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
