-- | The `ce deadcode` console sentences (plan v2.32 step 5),
-- transcribed byte for byte from cli/src/graph/deadcode/report.rs,
-- the reason and trust words (cli/src/graph/deadcode/why.rs,
-- cli/src/graph/deadcode.rs `conf_word`) and the `--check` lines
-- (cli/src/main_cmds.rs `deadcode_cmd`). The verdict and advisory code
-- names and the advisory readings print in English under both
-- languages (they are the document's words).
module CE.Text.Deadcode (catalogue) where

import CE.Text (Catalogue, table)

catalogue :: Catalogue
catalogue =
  table
    "dead\tunreferenced{}: {}  {}  ({}){}{}\t无引用{}：{}  {}（{}）{}{}\n\
    \state_mark\t ({})\t（{}）\n\
    \st_standing\tstanding\t存量\n\
    \st_orphaned\torphaned\t失去引用\n\
    \st_born\tborn\t新出现\n\
    \falls\t — {} file(s) fall with it\t——另有 {} 个文件随它无引用\n\
    \under\t — under {}\t——随根 {}\n\
    \under_more\t — under {} and {} more\t——随根 {} 及另 {} 个\n\
    \why_unref\tno kept in-edge and no entry flag\t无保留入边且无入口标记\n\
    \why_unreach\treferenced only from unreferenced files; no entry flag\t仅被无引用的文件引用且无入口标记\n\
    \unvouched\t [unvouched: unresolved sites in this language]\t〔未担保：该语言尚有未解析点位〕\n\
    \vacuous\t [vacuous]\t〔空担保〕\n\
    \vouched\t [vouched]\t〔已担保〕\n\
    \aggregate\taggregate: {}  {}  (reported, never a file verdict — decision 4)\t聚合件：{}  {}（仅报告，不作文件判决 — 决议 4）\n\
    \advisory_dropped\tadvisory: the core dropped the unmentioned table — more than {} candidate rows, none judged at symbol level\t顾问：核已丢弃未提及表——候选行超过 {}，符号层一行未判\n\
    \advisory\tadvisory: {}:{}  {}  {}  ({})\t顾问：{}:{}  {}  {}（{}）\n\
    \advisory_census\tadvisory: {} unmentioned declaration(s) in {} file(s) — {} public, {} private, {} restricted, {} reexported; no other file spells them (an advisory, never a verdict)\t顾问：{} 个未提及声明分布于 {} 个文件——公开 {}、私有 {}、受限 {}、再导出 {}；无他文件拼写其名（仅建议，永不判决）\n\
    \advisory_cut\tadvisory: the candidate table was cut at the producer's {}-row cap — the rows above are a prefix, the same prefix every run\t顾问：候选表已在生产者侧 {} 行上限截断——以上各行是前缀，每次运行同一前缀\n\
    \summary\tdeadcode: {} nodes, {} kept edges, {} unreferenced, {} aggregate reports, {} unresolved sites (verdicts assume none lands in-corpus)\t死码：{} 节点，{} 保留边，{} 无引用，{} 聚合报告，{} 未解析调用点（判决假设它们皆不落语料内）\n\
    \summary_states\tdeadcode: {} nodes, {} kept edges, {} unreferenced ({} root(s): {} standing, {} orphaned, {} born), {} aggregate reports, {} unresolved sites (verdicts assume none lands in-corpus)\t死码：{} 节点，{} 保留边，{} 无引用（{} 个根：存量 {}、失去引用 {}、新出现 {}），{} 聚合报告，{} 未解析调用点（判决假设它们皆不落语料内）\n\
    \unanswered\tnote: [deadcode] block ({}) is declared but cannot be answered — no committed baseline records which files were unreferenced (from the project root: CE_ACCEPT_BASELINE=1 ce baseline .)\t注：已声明 [deadcode] block（{}），但无从回答——已提交的基线没有记下当时哪些文件无引用（在项目根目录：CE_ACCEPT_BASELINE=1 ce baseline .）\n\
    \check_unanswered\tdeadcode check: [deadcode] block ({}) is declared and no committed baseline records which files were unreferenced — nothing to compare, refusing to pass; write one from the project root: CE_ACCEPT_BASELINE=1 ce baseline .\tdeadcode check：已声明 [deadcode] block（{}），但已提交的基线没有记下当时哪些文件无引用——无可比较，拒绝通过；在项目根目录写一份：CE_ACCEPT_BASELINE=1 ce baseline .\n\
    \check_blocked\tdeadcode check: {} root(s) blocked ({}): {} — declare an entry or delete them\tdeadcode check：{} 个根被拦（{}）：{}——请声明入口或删除\n\
    \check_blocked_more\tdeadcode check: {} root(s) blocked ({}): {} and {} more — declare an entry or delete them\tdeadcode check：{} 个根被拦（{}）：{} 及另 {} 个——请声明入口或删除\n\
    \check_listed\tdeadcode check: {} unreferenced file(s) listed, not blocked — a project blocks new ones with ce.toml [deadcode] block = [\"orphaned\", \"born\"] and a committed baseline\tdeadcode check：{} 个无引用文件只列出、不拦——项目可在 ce.toml 写 [deadcode] block = [\"orphaned\", \"born\"] 并提交基线，拦新出现的\n\
    \check_listed_gated\tdeadcode check: {} unreferenced file(s) listed; none in a blocked state ({})\tdeadcode check：{} 个无引用文件只列出；无一处于被拦状态（{}）\n\
    \summary_roots\tdeadcode: {} nodes, {} kept edges, {} unreferenced ({} root(s)), {} aggregate reports, {} unresolved sites (verdicts assume none lands in-corpus)\t死码：{} 节点，{} 保留边，{} 无引用（{} 个根），{} 聚合报告，{} 未解析调用点（判决假设它们皆不落语料内）\n\
    \grouping_dropped\tdeadcode: the grouping dropped its root pairs — more than {} units of work; the roots are listed, the dependents without their roots\t死码：分组已丢弃根对——工作量超过 {}；根照列，依附不注其根\n\
    \degraded\tdegraded: {} (nothing was analyzed)\t降级：{}（未分析任何内容）\n\
    \entry_hint\tnote: {} of {} files are unreferenced for want of an entry flag — a convention-loaded repo (a plugin, a script collection) declares its roots in ce.toml [graph] entry_globs\t注：{} / {} 个文件皆因缺入口标志而无引用——按约定加载的仓库（插件、脚本集）应在 ce.toml [graph] entry_globs 声明其根\n\
    \check_degraded\tdeadcode check: degraded ({}) — nothing was judged, refusing to pass\tdeadcode check：已降级（{}）— 未判决任何内容，拒绝通过"
