# hook-payloads — PreToolUse(Edit|Write) 实测采集

官方 hooks 文档只给 Bash 的 `tool_input` 逐字示例，Edit/Write 没有（ADR-007 ⚠️）。
此目录用真实会话采集 payload，固化为 M3 guard 的输入契约。

- `dump.py`：由 `.claude/settings.json` 的 PreToolUse hook 调用，把 stdin 事件
  追加到 `payloads.ndjson`（只记录、不拦截、exit 0）。
- `payloads.ndjson`：原始采集（含 Edit / Write / replace_all 变体后按需精选）。
- 采集到足量样本后，人工抽取代表性样本改名 `edit.golden.json` /
  `write.golden.json` 入库，`payloads.ndjson` 清空重采。

注意：hooks 配置在**会话启动时**加载；本文件所在配置若中途写入，需重启
Claude Code 会话后才开始采集。
