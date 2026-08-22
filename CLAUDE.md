# CodeEraser — 项目级指令（Claude Code 每次会话加载）

## 当前状态：**v1.0.0 已发布（2026-08-22）**——M0–M9 全交付，项目收口。M9 批 0 发 v0.7.3；批 1–9 与终扫全落 main：113 条两车道审阅收口（81 修 + 29 具名书面处置 + 3 驳回成立，收口册 memory/audit-reconciliation.md）、方法学 12 册（citations/nav/consts 三 CI 门）、`ce erase` 确定性两段式、GUI 八屏（含 Graph）、bench dashboard（contracts/bench/bench.json 单源，README·官网·GUI 字节门控，per-tag 回放回填，v1.0.0 自身 7 行已入列）、两真仓实战+密度评分根修（2.17.0）、判决回迁 16 片（proto→2.25.0，audit 第十族）、57-agent 架构全审（2 修 + 36 拒驳处置）、批 9 十九项终打磨（P0–P18，环轴修正案 v2.12 / proto 2.27.0）、非门控文档 716 断言终核（24 修）、发版夜 macOS daemon 尸体根修（tag 腿 observe golden 抓出；bind 侧 connect 分流+`try_overwrite` 收尸，daemon_singleton 电池，f62cff2——为此重建资产重 pin 重 tag）。渠道四开：GitHub Release 十资产（macOS 腿绿亲证）、crates.io、npm（1.0.0 补位，0.7.3 从未上）、codeeraser.dev（stack/bench 新页+v1.0.0 仪表盘）。协议 ce↔core **2.27.0** / daemon **1.1.0**；dedup 预算 **188**（台账在 ce.toml 注释块）；CI 地板 **950**。分数迁移已随 release notes 声明：1.0.0 的分数与 0.7.3 **不可比**。

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
     `ce check --fail-under 950`）与 **dedup 预算**——两者都是硬门。预算「只降不升」
     指**无声不升**：每次上调必须在 ce.toml 注释块入账（那里是预算台账的正册，
     历史 143→188 的每一步都在册各有其偿），无账上调即违规。
   - **判决**语言集 = `py/ts/tsx/rs/go/md/hs`；v0.5.0 起 scan 另有**纯尺寸臂**（js/mjs/cjs/jsx、
     css/scss/less、html/htm、vue、svelte、sh/bash、yml/yaml）——只进 scan 尺寸门 + guard 硬预算 +
     棘轮，永不进判决族（边界权威 = `Lang::scan_only`/`judged_path`，cli/src/scan/lang.rs）。GUI 的
     JS 语义另由 `gui/tests/lens_invariant.js` 一条 CI 腿覆盖。
4. **禁止堆叠式编辑**：更新文档/代码时就地修改，不做"追加新段落覆盖旧段落"式打补丁 ——
   这正是本项目要消灭的行为。
