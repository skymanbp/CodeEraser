-- | The Stop audit's and the git-hook faces' sentences (plan v2.32
-- step 5), transcribed byte for byte from cli/src/audit.rs (`reason`,
-- the gated mount's prefix), cli/src/audit/verdict.rs (a shown block),
-- cli/src/audit/tombstone.rs (`reason`, `subject`, `summary`,
-- `incomplete_note`, `sites_line`, the incomplete count),
-- cli/src/audit/precommit.rs (`staged_summary`, not a git repo) and
-- cli/src/audit/commitmsg.rs (the unreadable message). A `\` line
-- continuation there drops the next line's leading space.
module CE.Text.Audit (catalogue) where

import CE.Text (Catalogue, table)

catalogue :: Catalogue
catalogue =
  table
    "dup_reason\tce audit: this session's edits leave {} duplicate block(s) touching changed files (net {} LOC): {} — deduplicate before stopping.\tce audit：本会话的编辑留下 {} 个触及改动文件的重复块（净 {} 行）：{} — 停止前请先去重。\n\
    \block\t{}:{}-{} <-> {}:{}-{} ({} tokens)\n\
    \mount\t{}: {}\n\
    \tomb_reason\t{} leave {} tombstone site(s), past the `[tombstone] budget` of {}: {} — a removed name must not survive as an absence label or an argument from absence; drop the label, or say what replaced it.\t{}留下 {} 处墓碑残留，越过 `[tombstone] budget` 的 {}：{} — 被删的名字不该以「无 X」标签或缺席论证留下；去掉标签，或写清替代物。\n\
    \subject_stop\tce audit: this session's edits\tce audit：本会话的编辑\n\
    \subject_precommit\tce precommit: the staged changes\tce precommit：暂存的改动\n\
    \subject_commitmsg\tce commitmsg: the staged changes and the message\tce commitmsg：暂存的改动与提交说明\n\
    \place\t{}:{} {}\n\
    \tomb_degraded\tce {}: tombstone verdict unavailable (DEGRADED: {})\tce {}：墓碑残留判决不可用（已降级：{}）\n\
    \sites\tce {}: {} tombstone site(s) — {} label / {} prose over {} erased name(s): {} (tier {}; see .ce/observe.ndjson)\tce {}：{} 处墓碑残留 — 标签 {} / 散文 {}，涉及 {} 个被删名字：{}（档位 {}；详见 .ce/observe.ndjson）\n\
    \incomplete\t (measurement incomplete: {}; not enforced)\t（度量不完整：{}；不作强制）\n\
    \unread\t{} pair(s) unread, {} with a bounded diff\t{} 对未读、{} 对 diff 有界\n\
    \not_git\tce {}: not a git repo (skipped)\tce {}：不是 git 仓库（跳过）\n\
    \staged_degraded\tce {}: {} staged file(s), net {} LOC — duplicate verdict unavailable (DEGRADED: duplicate check skipped)\tce {}：{} 个暂存文件，净 {} 行 — 重复判决不可用（已降级：重复检查已跳过）\n\
    \staged\tce {}: {} staged file(s), net {} LOC, no touched duplicates\tce {}：{} 个暂存文件，净 {} 行，未触及重复块\n\
    \unreadable\tce commitmsg: cannot read {} (absent, binary or past READ_CAP)\tce commitmsg：读不了 {}（不存在、二进制或超过 READ_CAP）"
