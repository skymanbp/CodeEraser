# hook-payloads — PreToolUse(Edit|Write) 实测采集（已完成并冻结）

官方 hooks 文档只给 Bash 的 `tool_input` 逐字示例，Edit/Write 没有（ADR-007 ⚠️）。
此目录 2026-08-07 用真实会话采集 payload，固化为 M3 guard 的输入契约。

- **已固化 golden（2026-08-07 实采）**：`write.golden.json`、`edit.golden.json`、
  `edit-replace-all.golden.json`。脱敏规则：`transcript_path`/`cwd`/`file_path` 的
  机器路径替换为 `<HOME>`/`<PROJECT_DIR>` 占位符——**字段结构与类型是契约，路径值不是**。
- 实采结论：信封字段 = session_id / transcript_path / cwd / prompt_id /
  permission_mode / effort{level} / hook_event_name / tool_name / tool_input /
  tool_use_id；Write 的 tool_input = {file_path, content}；Edit 的 tool_input =
  {file_path, old_string, new_string, replace_all}。
- 采集器（`dump.py` + `.claude/settings.json` 的 PreToolUse 挂载）已随任务完成
  退役（2026-08-20 清理批移除——采毕即冻，脚本方法在 git 历史 `30c8d90`）。
