# CodeEraser — 项目级指令（Claude Code 每次会话加载）

## 当前状态：M0–M8 全交付，v0.7.1 已发布（2026-08-20）；计划 v2.7 收尾批落地（wire 2.15.0）：拆分 ROI v1.1 四腿价目（跨缝引用 250/切穿克隆块 500/跨缝共变对 150/φ500，外语料标定实录在册）、guard 区内档位映射 opt-in 接线（`[guard] zone_tiers`，默认恒 observe——FPR 纪律不破，feed 0.6.0 记映射档）、MCP/GUI split 面补齐；相对软线随具名重立重算（现值恒以 ce-baseline.json 为准）；设计册 docs/reference/size-advisory.md 含 P2/P4/v1.1 as-built 实录；v0.7.1=NSIS 提权补丁（installMode perMachine，v0.7.0 notes 载 workaround）

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
3. **吃自己的狗粮**：本仓库自身必须持续通过 CodeEraser 的门，从 M1 起 CI 强制。
   **门的真实档位（2026-08-19 核实，勿凭印象引用）**：
   - `ce scan` 只在 **FAIL** 档退 1 —— 文件 >750 行、函数 >75 行。**300/50 是 WARN**，
     不由 scan 退出码强制；复杂度四项（params/cyclomatic/cognitive/nesting）**无 fail 档**。
   - 真正把文件摁在 300 附近的是 **ADR-006 逐文件棘轮**（单次增长 ≤ max(+2%, +10)，
     `ce check --fail-under 800`）与 **dedup 预算**（只降不升）—— 两者都是硬门。
   - **判决**语言集 = `py/ts/tsx/rs/go/md/hs`；v0.5.0 起 scan 另有**纯尺寸臂**（js/mjs/cjs/jsx、
     css/scss/less、html/htm、vue、svelte、sh/bash、yml/yaml）——只进 scan 尺寸门 + guard 硬预算 +
     棘轮，永不进判决族（边界权威 = `Lang::scan_only`/`judged_path`，cli/src/scan/lang.rs）。GUI 的
     JS 语义另由 `gui/tests/lens_invariant.js` 一条 CI 腿覆盖。
4. **禁止堆叠式编辑**：更新文档/代码时就地修改，不做"追加新段落覆盖旧段落"式打补丁 ——
   这正是本项目要消灭的行为。
