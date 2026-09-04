//! Every sentence the PreToolUse guard speaks, in both languages.
//!
//! The Stop audit has answered `CE_LANG=zh` in Chinese since M8-G3b
//! (audit::reason, audit::staged_summary) while this face — the one
//! a person reads most, and the only one that ever refuses a write —
//! answered in English on all six of its lines. The templates sat
//! inline at six call sites across two files, so nothing ever asked
//! them as a set and nothing could gate them as a set. They are a
//! set here: a seventh sentence that forgets its Chinese half has to
//! be written next to six that did not.
//!
//! The English half is the byte-identical literal each call site
//! carried (i18n.rs's hard constraint: the default path must not
//! move a byte). Only the holes changed shape — from inline captures
//! to the positional `{}` that `i18n::line` fills left to right — so
//! the two templates of a row must consume their arguments in the
//! same order, which is why each row takes typed parameters rather
//! than a caller-ordered argument list.

use crate::i18n::{line, t};

/// The T1/T2 rule: this content duplicates indexed regions.
pub(super) fn duplicate(file: &str, regions: usize, top: &str) -> String {
    line(
        "ce: content for {} duplicates {} indexed region(s): {}. \
         Reuse the existing implementation instead of re-writing it. \
         Moving it? Trim the source region first: the probe verifies \
         against the current tree, and the same write then passes.",
        "ce：{} 的内容与 {} 处已索引区域重复：{}。请复用既有实现，\
         而不是另写一份。若是在搬移？先删去源区域：探针以当前树为准\
         校验，同一次写入随即通过。",
        &[&file, &regions, &top],
    )
}

/// The hard-budget rule: the write would leave the file past its
/// class line, or the global one. `note` is the fence's own clause.
pub(super) fn over_budget(file: &str, lines: usize, cap: usize, fence: Option<&str>) -> String {
    let note = fence.map_or(String::new(), |f| format!(" {f}"));
    line(
        "ce: this write leaves {} at {} lines, past the hard budget \
         of {} (plan §4.1). Split the file instead of growing it.{}",
        "ce：这次写入会让 {} 达到 {} 行，越过 {} 行的硬预算（计划 §4.1）。\
         请拆分文件，而不是继续让它长大。{}",
        &[&file, &lines, &cap, &note],
    )
}

/// The graded-zone rule (v2.7 ①, opt-in): under the hard line, but
/// far enough into the zone that the tier map has something to say.
pub(super) fn graded_zone(
    file: &str,
    lines: usize,
    permille: usize,
    soft: usize,
    cap: usize,
) -> String {
    line(
        "ce: this write leaves {} at {} lines, {}‰ into the \
         graded zone ({}..{}); `ce structure --split-candidates` prices \
         its best seam.",
        "ce：这次写入会让 {} 达到 {} 行，进入分级区 {}‰（{}..{}）；\
         `ce structure --split-candidates` 会为它最好的那条切缝定价。",
        &[&file, &lines, &permille, &soft, &cap],
    )
}

/// The tombstone class (plan v2.27): the core judged more sites than
/// `[tombstone] budget` allows in this one write.
pub(super) fn tombstone_over(sites: usize, budget: u32, shown: &str) -> String {
    line(
        "ce: this write leaves {} tombstone site(s), past the `[tombstone] budget` \
         of {}: {}. A removed name must not survive as an absence label or an \
         argument from absence — drop the label, or say what replaced it.",
        "ce：这次写入留下 {} 处墓碑残留，越过 `[tombstone] budget` 的 {}：{}。\
         被删的名字不该以「无 X」标签或缺席论证的形式留下——去掉标签，或写清替代物。",
        &[&sites, &budget, &shown],
    )
}

/// Fail-open, but never silent: a ce.toml that will not parse.
pub(super) fn config_unreadable(err: &str) -> String {
    line(
        "(ce.toml unreadable, guard degraded to observe: {})",
        "（ce.toml 不可读，守卫已降级为 observe：{}）",
        &[&err],
    )
}

/// The two fence clauses (O33) — why this write was judged against
/// the shipped budgets rather than the declared ones.
pub(super) fn drifted() -> &'static str {
    t(
        "(ce.toml drifted from the fenced baseline: judged with the shipped budgets)",
        "（ce.toml 已偏离围栏基线：改按出厂预算判决）",
    )
}

pub(super) fn baseline_unreadable() -> &'static str {
    t(
        "(the committed baseline is unreadable: judged with the shipped budgets)",
        "（提交在库的基线不可读：改按出厂预算判决）",
    )
}
