-- | Every sentence the PreToolUse guard speaks (plan v2.32 step 5),
-- transcribed byte for byte from cli/src/guard/say.rs (a `\` line
-- continuation there drops the next line's leading space), the
-- duplicate probe's match list (cli/src/guard/probe.rs `reason`) and
-- the tombstone place (cli/src/tombstone/mod.rs `Row::place`).
module CE.Text.Guard (catalogue) where

import CE.Text (Catalogue, table)

catalogue :: Catalogue
catalogue =
  table
    "duplicate\tce: content for {} duplicates {} indexed region(s): {}. Reuse the existing implementation instead of re-writing it. Moving it? Trim the source region first: the probe verifies against the current tree, and the same write then passes.\tce：{} 的内容与 {} 处已索引区域重复：{}。请复用既有实现，而不是另写一份。若是在搬移？先删去源区域：探针以当前树为准校验，同一次写入随即通过。\n\
    \match\t{}:{}-{} ({} tokens)\n\
    \over_budget\tce: this write leaves {} at {} lines, past the hard budget of {} (plan §4.1). Split the file instead of growing it.{}\tce：这次写入会让 {} 达到 {} 行，越过 {} 行的硬预算（计划 §4.1）。请拆分文件，而不是继续让它长大。{}\n\
    \graded_zone\tce: this write leaves {} at {} lines, {}‰ into the graded zone ({}..{}); `ce structure --split-candidates` prices its best seam.\tce：这次写入会让 {} 达到 {} 行，进入分级区 {}‰（{}..{}）；`ce structure --split-candidates` 会为它最好的那条切缝定价。\n\
    \tombstone_over\tce: this write leaves {} tombstone site(s), past the `[tombstone] budget` of {}: {}. A removed name must not survive as an absence label or an argument from absence — drop the label, or say what replaced it.\tce：这次写入留下 {} 处墓碑残留，越过 `[tombstone] budget` 的 {}：{}。被删的名字不该以「无 X」标签或缺席论证的形式留下——去掉标签，或写清替代物。\n\
    \place\t{}:{} {}\n\
    \flow_novel\tce: this write brings {} new dead-code finding(s) into {} (first: {} {} at line {}); `ce flow` lists them. Delete the dead code, or use what it computes.\tce：这次写入带来 {} 条新的函数内死代码发现，落在 {}（第一条：{} {}，第 {} 行）；`ce flow` 会列出全部。请删去死代码，或用上它算出的值。\n\
    \config_unreadable\t(ce.toml unreadable, guard degraded to observe: {})\t（ce.toml 不可读，守卫已降级为 observe：{}）\n\
    \drifted\t(ce.toml drifted from the fenced baseline: judged with the shipped budgets)\t（ce.toml 已偏离围栏基线：改按出厂预算判决）\n\
    \baseline_unreadable\t(the committed baseline is unreadable: judged with the shipped budgets)\t（提交在库的基线不可读：改按出厂预算判决）\n\
    \note\t {}"
