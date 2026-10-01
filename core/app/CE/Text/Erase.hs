-- | The `ce erase` console sentences (plan v2.32 step 5), transcribed
-- byte for byte from cli/src/erase/render.rs (`print`,
-- `out_of_class_line`, `reason_detail`), cli/src/main_erase.rs (the
-- `--check` line, English only, and the `--apply` line) and the trail's
-- reader cli/src/erase/log.rs (`print`, whose record row is English
-- only). The diff is one reference per file the measuring side renders
-- (`{}`). An out-of-class kind without a family command cannot occur:
-- every kind the plan counts has its command (CE.Erase.Document), so
-- render.rs's "see the family command" sentence has no template here.
module CE.Text.Erase (catalogue) where

import CE.Text (Catalogue, table)

catalogue :: Catalogue
catalogue =
  table
    "diff\t{}\n\
    \advisory\tadvisory {} {}{}: {}{} ({})\t仅建议 {} {}{}：{}{}（{}）\n\
    \detail\t — {} unresolved reference sites in this language\t——该语言尚有 {} 个未解析引用点位\n\
    \kind\tadvisory {}: {} finding(s) — no deterministic-safe erase; see `{}`\t仅建议 {}：{} 条——无确定性安全擦除；见 `{}`\n\
    \summary\terase plan: {} eraseable, {} advisory (dry-run; --apply to act)\t擦除计划：可擦 {}，仅建议 {}（演练；--apply 才动手）\n\
    \check\terase check: {} eraseable row(s) planned\n\
    \applied\terase applied: {} row(s); audit trail in .ce/erase-log.ndjson\t擦除已执行：{} 行；审计轨迹在 .ce/erase-log.ndjson\n\
    \record\t{} {} {}{} ({}) plan {}\n\
    \unreadable\tunreadable line {}: {}\t第 {} 行不可读：{}\n\
    \log\terase log: {} record(s) in {} ({} unreadable)\t擦除日志：{} 条记录在 {}（{} 行不可读）\n\
    \no_log\terase log: no {} yet — nothing has been applied here\t擦除日志：尚无 {}——这里还没执行过擦除"
