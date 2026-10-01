//! The console face of the merge document (booklet §6.4): one line of
//! counts and of the groups not sent, then per group its head line —
//! family, fragment, members, parameters, savings, feasible or why
//! not — its members and one line per parameter with every member's
//! text at it. Every sentence has its Chinese twin (i18n::line); the
//! code's own words (paths, units, texts) stay as written.

use super::report::{GroupFace, MemberFace, Report};
use crate::i18n::line;

/// A parameter's text on one line: 40 characters at most.
const TEXT_CAP: usize = 40;

/// The whole face, or one group's (`only`, a group number the caller
/// has checked).
pub fn console(r: &Report, only: Option<usize>) -> Vec<String> {
    if let Some(why) = &r.degraded {
        return vec![line(
            "merge: degraded — {} (no judgment)",
            "合并：已降级——{}（未判决）",
            &[why],
        )];
    }
    let u = &r.unsendable;
    let mut out = vec![line(
        "{} group(s) judged, {} feasible, {} hole(s), {} duplicate(s) merged; not sent: {} not isomorphic, {} without a slot table, {} unbuilt, {} over the cap",
        "判决 {} 组，可行 {}，洞 {} 个，同集合并 {}；未送：不同构 {}、无位置类表 {}、树未建 {}、超上限 {}",
        &[
            &r.counts["groups"],
            &r.counts["feasible"],
            &r.counts["holes"],
            &r.counts["merged_duplicates"],
            &u.not_isomorphic,
            &u.no_slot_table,
            &u.unbuilt,
            &u.over_cap,
        ],
    )];
    for g in r
        .groups
        .iter()
        .filter(|g| only.is_none_or(|k| k == g.group))
    {
        out.extend(group(g));
    }
    out
}

fn group(g: &GroupFace) -> Vec<String> {
    let family = if g.fragment {
        format!("{} (fragment)", g.family)
    } else {
        g.family.to_string()
    };
    let verdict = if g.feasible {
        line("feasible", "可行", &[])
    } else {
        line("infeasible ({})", "不可行（{}）", &[&g.reason])
    };
    let mut out = vec![line(
        "group {}: {} · {} members · {} params · savings {} lines · {}",
        "组 {}：{} · {} 个成员 · {} 个参数 · 省 {} 行 · {}",
        &[
            &g.group,
            &family,
            &g.members.len(),
            &g.params,
            &g.savings,
            &verdict,
        ],
    )];
    out.extend(g.members.iter().map(member));
    for p in &g.holes {
        let cells: Vec<String> = p
            .values
            .iter()
            .map(|v| format!("m{} \"{}\"", v.member, clip(&v.text)))
            .collect();
        out.push(line(
            "  param {}: {}",
            "  参数 {}：{}",
            &[&(p.param + 1), &cells.join(" | ")],
        ));
    }
    out
}

/// A member's line: its unit or path and its clone-family span, and the
/// run it sent when a trim made that shorter (the lines the core priced).
fn member(m: &MemberFace) -> String {
    let name = m.unit.clone().unwrap_or_else(|| m.path.clone());
    let [a, b] = m.lines;
    if m.run == m.lines {
        return line("  {}  lines {}-{}", "  {}  行 {}-{}", &[&name, &a, &b]);
    }
    let [ra, rb] = m.run;
    line(
        "  {}  lines {}-{} (run {}-{})",
        "  {}  行 {}-{}（合并段 {}-{}）",
        &[&name, &a, &b, &ra, &rb],
    )
}

/// One line, at most TEXT_CAP characters, `…` when cut.
fn clip(text: &str) -> String {
    let flat: String = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if flat.chars().count() <= TEXT_CAP {
        return flat;
    }
    let cut: String = flat.chars().take(TEXT_CAP).collect();
    format!("{cut}…")
}
