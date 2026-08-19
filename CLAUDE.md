# CodeEraser — 项目级指令（Claude Code 每次会话加载）

## 当前状态：M0–M8 全交付，v0.3.0 已发布（2026-08-19 收口）；下一周期等用户试用反馈开新计划，勿自行开工

- 唯一权威计划：[docs/DEVELOPMENT_PLAN.md](docs/DEVELOPMENT_PLAN.md)，
  已通过 cc-memory (ccm) 锁定为项目 PLAN。推进任何里程碑前先读它。
- 会话交接：先读 `memory/PROGRESS.md`（cc-memory 生成的 handoff 契约）。它是
  **机器本地状态，不入版本库**（.gitignore 排除，用户 2026-08-07 决策）——
  新 clone 上没有该文件属正常，由 cc-memory 首次会话重建。

## 硬性约束

1. **语言分工不可漂移**（用户已拍板）：判决层 = Haskell；解析/索引/前端 = Rust。
   不得混入第三种实现语言；任何"用 Python 快速原型一下"的冲动都违反本约束。
2. **计划即契约**：偏离 DEVELOPMENT_PLAN.md 的架构决策（ADR 章节）必须先改计划、
   重新走 ccm 锁定，再动代码。
3. **吃自己的狗粮**：本仓库自身必须持续通过 CodeEraser 的规则（文件 LOC、函数长度、
   复杂度、去冗预算），从 M1 起在 CI 强制。
4. **禁止堆叠式编辑**：更新文档/代码时就地修改，不做"追加新段落覆盖旧段落"式打补丁 ——
   这正是本项目要消灭的行为。
