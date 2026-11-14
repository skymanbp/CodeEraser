-- | The `ce erase` console sentences (plan v2.32 step 5), transcribed
-- byte for byte from cli/src/erase/render.rs (`print`,
-- `out_of_class_line`, `reason_detail`), cli/src/main_erase.rs (the
-- `--check` line, English only, and the `--apply` line) and the trail's
-- reader cli/src/erase/log.rs (`print`, whose record row is English
-- only). The diff is one reference per file the measuring side renders
-- (`{}`). An out-of-class kind without a family command cannot occur:
-- every kind the plan counts has its command (CE.Erase.Document), so
-- render.rs's "see the family command" sentence has no template here.
-- An advisory row's provenance clause is written here from its facts
-- (release 1.10.0: `prov_*`, the English byte for byte the sentence the
-- measuring side keeps in the document); a dead file's reason is the
-- deadcode catalogue's own two rows, borrowed, never copied.
module CE.Text.Erase (catalogue) where

import CE.Text (Catalogue, table)
import qualified CE.Text.Deadcode as Deadcode

catalogue :: Catalogue
catalogue = own <> [row | row@(k, _) <- Deadcode.catalogue, k `elem` ["why_unref", "why_unreach"]]

own :: Catalogue
own =
  table
    "diff\t{}\n\
    \advisory\tadvisory {} {}{}: {}{} ({})\t仅建议 {} {}{}：{}{}（{}）\n\
    \detail\t — {} unresolved reference sites in this language\t——该语言尚有 {} 个未解析引用点位\n\
    \kind\tadvisory {}: {} finding(s) — no deterministic-safe erase; see `{}`\t仅建议 {}：{} 条——无确定性安全擦除；见 `{}`\n\
    \summary_dry\terase plan: {} eraseable, {} advisory (dry-run; --apply to act)\t擦除计划：可擦 {}，仅建议 {}（演练；--apply 才动手）\n\
    \summary_applied\terase plan: {} eraseable, {} advisory (applying the eraseable rows)\t擦除计划：可擦 {}，仅建议 {}（正在执行可擦行）\n\
    \check\terase check: {} eraseable row(s) planned\n\
    \applied\terase applied: {} row(s); audit trail in .ce/erase-log.ndjson\t擦除已执行：{} 行；审计轨迹在 .ce/erase-log.ndjson\n\
    \prov_dead\tdeadcode: {} — {}\t死码：{}——{}\n\
    \prov_doc\tdocdup: twin of {}:{}-{} (J {}/{}, verbatim {} words){}\t文档重复：{}:{}-{} 的孪生（J {}/{}，逐字 {} 词）{}\n\
    \prov_linked\t; section linked from {} places\t；所在小节被 {} 处链接\n\
    \prov_twin\tdedup: T1 block twin at {}:{}-{} ({} tokens), covers unit {}\t克隆：{}:{}-{} 处的 T1 孪生块（{} 个 token），覆盖单元 {}\n\
    \record\t{} {} {}{} ({}) plan {}\n\
    \unreadable\tunreadable line {}: {}\t第 {} 行不可读：{}\n\
    \log\terase log: {} record(s) in {} ({} unreadable)\t擦除日志：{} 条记录在 {}（{} 行不可读）\n\
    \no_log\terase log: no {} yet — nothing has been applied here\t擦除日志：尚无 {}——这里还没执行过擦除"
