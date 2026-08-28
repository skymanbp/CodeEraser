//! The deadcode console rendering, moved beside its measurement
//! (K step 9) — the pattern churn/, trend/, join/ and health::doctor
//! each took at their own 300-line walls; deadcode was the last
//! judgment family still rendering inside main_cmds.rs, and the
//! entry_globs hint was the growth step that made the debt visible.
//! Rendering only: every verdict, count and confidence word below is
//! the core's or the measurement's.

use super::Report;
use super::advisory::{ADVISORY_NAMES, UnmentionedFace};
use crate::i18n::line;
use crate::mention::UNMENTIONED_SOFT_CAP;

/// Section/package aggregates are reported, never called dead
/// (decision 4); the unresolved-site count rides every summary — the
/// verdicts assume none of those sites lands in-corpus, and the
/// reader sees it.
pub fn print(r: &Report, json: bool) {
    if json {
        println!("{}", crate::report::deadcode_json(r));
        return;
    }
    for d in &r.dead {
        println!(
            "{}",
            line(
                "dead: {}  {}  ({}){}",
                "死件：{}  {}（{}）{}",
                &[&d.path, &d.verdict, &d.why, &super::conf_word(d.conf)],
            )
        );
    }
    for (name, verdict) in &r.reported {
        println!(
            "{}",
            line(
                "aggregate: {}  {}  (reported, never dead — decision 4)",
                "聚合件：{}  {}（仅报告，永不判死 — 决议 4）",
                &[name, verdict],
            )
        );
    }
    advisory(r);
    tail(r);
}

/// The symbol-level advisory (6.2.0), rendered only when the road was
/// asked: one line per unmentioned declaration with the core's code,
/// a census by code, and the two states the rows alone cannot show —
/// the producer's cut (the rows are a prefix) and the core's drop (no
/// rows were judged). Advisories, never verdicts: nothing here moves
/// the exit.
fn advisory(r: &Report) {
    let Some(face) = &r.unmentioned else {
        return;
    };
    let UnmentionedFace::Rows { rows, cut } = face else {
        println!(
            "{}",
            line(
                "advisory: the core dropped the unmentioned table — more than {} candidate rows, none judged at symbol level",
                "顾问：核已丢弃未提及表——候选行超过 {}，符号层一行未判",
                &[&UNMENTIONED_SOFT_CAP],
            )
        );
        return;
    };
    for a in rows {
        println!(
            "{}",
            line(
                "advisory: {}:{}  {}  {}  ({})",
                "顾问：{}:{}  {}  {}（{}）",
                &[&a.name, &a.line, &a.symbol, &a.code, &a.why],
            )
        );
    }
    let by_code = ADVISORY_NAMES.map(|c| rows.iter().filter(|a| a.code == c).count());
    let files = rows
        .iter()
        .map(|a| a.name.as_str())
        .collect::<std::collections::BTreeSet<_>>()
        .len();
    println!(
        "{}",
        line(
            "advisory: {} unmentioned declaration(s) in {} file(s) — {} public, {} private, {} restricted, {} reexported; no other file spells them (an advisory, never a verdict)",
            "顾问：{} 个未提及声明分布于 {} 个文件——公开 {}、私有 {}、受限 {}、再导出 {}；无他文件拼写其名（仅建议，永不判决）",
            &[
                &rows.len(),
                &files,
                &by_code[0],
                &by_code[1],
                &by_code[2],
                &by_code[3]
            ],
        )
    );
    if *cut {
        println!(
            "{}",
            line(
                "advisory: the candidate table was cut at the producer's {}-row cap — the rows above are a prefix, the same prefix every run",
                "顾问：候选表已在生产者侧 {} 行上限截断——以上各行是前缀，每次运行同一前缀",
                &[&UNMENTIONED_SOFT_CAP],
            )
        );
    }
}

/// The summary + degraded lines (split at the 50-line fn gate).
fn tail(r: &Report) {
    println!(
        "{}",
        line(
            "deadcode: {} nodes, {} kept edges, {} dead, {} aggregate reports, \
             {} unresolved sites (verdicts assume none lands in-corpus)",
            "死码：{} 节点，{} 保留边，{} 死件，{} 聚合报告，{} 未解析调用点（判决假设它们皆不落语料内）",
            &[
                &r.nodes,
                &r.kept,
                &r.dead.len(),
                &r.reported.len(),
                &r.unresolved_sites,
            ],
        )
    );
    if let Some(reason) = &r.degraded {
        println!(
            "{}",
            line(
                "degraded: {} (nothing was analyzed)",
                "降级：{}（未分析任何内容）",
                &[reason],
            )
        );
    }
    // A display heuristic, not a verdict (the report.rs display-cut
    // stance): when HALF or more of the file tier is dead — and it is
    // a pattern (>= 2), not one stray file — the repo is most likely
    // convention-loaded (a plugin, a script collection) and the
    // remedy is a declared root, not mass deletion. One dead file
    // stays hint-free on purpose: hinting an exemption knob at a
    // genuinely dead file would teach masking over deleting.
    if r.dead.len() >= 2 && r.dead.len() * 2 >= r.files {
        println!(
            "{}",
            line(
                "note: {} of {} files are dead for want of an entry flag — a convention-loaded repo (a plugin, a script collection) declares its roots in ce.toml [graph] entry_globs",
                "注：{} / {} 个文件皆因缺入口标志而判死——按约定加载的仓库（插件、脚本集）应在 ce.toml [graph] entry_globs 声明其根",
                &[&r.dead.len(), &r.files],
            )
        );
    }
}
