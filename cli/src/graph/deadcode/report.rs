//! The deadcode console rendering, moved beside its measurement
//! (K step 9) — the pattern churn/, trend/, join/ and health::doctor
//! each took at their own 300-line walls; deadcode was the last
//! judgment family still rendering inside main_cmds.rs, and the
//! entry_globs hint was the growth step that made the debt visible.
//! Rendering only: every verdict, count and confidence word below is
//! the core's or the measurement's.

use super::Report;
use crate::i18n::line;

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
    tail(r);
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
