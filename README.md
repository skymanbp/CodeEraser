# CodeEraser

> 对抗 LLM 代码/文档熵增的橡皮擦 —— An eraser against LLM-induced code & document entropy.

LLM 在长期代码/文档工作中极度偏向"堆叠"与"打补丁"：重复实现同一函数、同一事实写多处、
更新变成追加、文件越改越长。CodeEraser 是一个 CLI + GUI 工具，作为 **Claude Code 插件**
提供 hooks 强制拦截，其他 agent（Codex / Kimi Code …）经 MCP / pre-commit / CI 集成，
用**主动审计**与**被动拦截**两种模式对抗这种熵增。

## Status

🔒 **Planning locked — implementation not started.**

完整开发计划见 [docs/DEVELOPMENT_PLAN.md](docs/DEVELOPMENT_PLAN.md)（由 cc-memory 锁定，
任何推进以计划为准）。

## Architecture (planned)

| 层 | 语言 | 职责 |
|---|---|---|
| Core (`core/`) | Haskell | 判决层：规则引擎、编辑四分类、评分棘轮、依赖图、TSED |
| Frontend (`cli/`) | Rust | 解析、索引、CLI、GUI(Tauri)、daemon、agent 集成 |
| Plugin (`plugin/`) | manifest + hooks | Claude Code 插件市场分发、PreToolUse/Stop 拦截 |

## License

Apache-2.0 — see [LICENSE](LICENSE).
