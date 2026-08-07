# hook-payloads — PreToolUse(Edit|Write) 实测采集

官方 hooks 文档只给 Bash 的 `tool_input` 逐字示例，Edit/Write 没有（ADR-007 ⚠️）。
此目录用真实会话采集 payload，固化为 M3 guard 的输入契约。

- `dump.py`：由 `.claude/settings.json` 的 PreToolUse hook 调用，把 stdin 事件
  追加到 `payloads.ndjson`（只记录、不拦截、exit 0；该原始文件含机器路径，已 gitignore）。
- **已固化 golden（2026-08-07 实采）**：`write.golden.json`、`edit.golden.json`、
  `edit-replace-all.golden.json`。脱敏规则：`transcript_path`/`cwd`/`file_path` 的
  机器路径替换为 `<HOME>`/`<PROJECT_DIR>` 占位符——**字段结构与类型是契约，路径值不是**。
- 实采结论：信封字段 = session_id / transcript_path / cwd / prompt_id /
  permission_mode / effort{level} / hook_event_name / tool_name / tool_input /
  tool_use_id；Write 的 tool_input = {file_path, content}；Edit 的 tool_input =
  {file_path, old_string, new_string, replace_all}。

注意：hooks 配置在**会话启动时**加载；改动 `.claude/settings.json` 后需重启会话生效。
