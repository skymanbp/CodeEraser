# CodeEraser

> 对抗 LLM 代码/文档熵增的橡皮擦 —— An eraser against LLM-induced code & document entropy.

LLM 在长期代码/文档工作中极度偏向"堆叠"与"打补丁"：重复实现同一函数、同一事实写多处、
更新变成追加、文件越改越长。CodeEraser 是一个 CLI + GUI 工具，同时可作为 LLM Agent
（Claude Code / Codex / Kimi Code …）的插件，用**主动审计**与**被动拦截**两种模式对抗这种熵增。

## Status

🔒 **Planning locked — implementation not started.**

完整开发计划见 [docs/DEVELOPMENT_PLAN.md](docs/DEVELOPMENT_PLAN.md)（由 cc-memory 锁定，
任何推进以计划为准）。

## Architecture (planned)

| 层 | 语言 | 职责 |
|---|---|---|
| Core (`core/`) | Haskell | 解析、度量、克隆检测、规则引擎 —— 所有分析逻辑 |
| Frontend (`cli/`, `gui/`) | Rust（备选 Go） | CLI、GUI、进程编排、agent 集成 |
| Plugin (`plugin/`) | manifest + hooks | Claude Code 插件市场分发、PreToolUse/Stop 拦截 |

## License

TBD（见开发计划开放问题）。
