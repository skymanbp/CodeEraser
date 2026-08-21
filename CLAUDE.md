# CodeEraser — 项目级指令（Claude Code 每次会话加载）

## 当前状态：M0–M8 全交付；计划 v2.10 = M9 收口令执行中（批 0 已发 v0.7.3；批 1–5 已落 main——审阅 113 条落地/方法学册 11 篇/`ce erase` 两段式〔proto 2.16.0 第九族〕/GUI 七屏完整化/bench dashboard〔contracts/bench/bench.json 单源，README·官网·GUI 三面字节门控，per-tag 回放回填 v0.1.0–v0.7.3〕；批 6 实战检验已落〔两真仓案例册 docs/FIELD-TEST.md + 密度评分根修 2.17.0，CI 地板 950〕；批 7 判决回迁已收口〔16 片=12 落线+4 书面处置，proto 2.18.0→2.25.0：RG9/多样性地板/全证据/钉地板/缺陷扫/原始陈旧表/audit 第十族/豁免权威；清单+处置横幅在 memory/batch7-inventory.md〕；批 8 架构全审已落。57-agent 八维+逐条双段对抗：40 发现→2 实（锚门收喉 hookio+工具链钉上根）+36 拒驳处置，台账 memory/batch8-review.md〃；待推：批 9 全域优雅性终打磨〔v2.10 用户令 1.0 前置〕、终扫清欠账、终版 **v1.0.0** 一次收口不再中间发版）；v0.7.x 线已收：尺寸顾问收环（wire 2.15.0，ROI v1.1 四腿价目+`[guard] zone_tiers` opt-in——FPR 纪律不破）、三补丁版（perMachine 提权/CLI 上机器 PATH/插件复用钉扎命中副本/GUI 无窗化/裸 `ce` 印 help）

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
     `ce check --fail-under 800`）与 **dedup 预算**——两者都是硬门。预算「只降不升」
     指**无声不升**：每次上调必须在 ce.toml 注释块入账（那里是预算台账的正册，
     历史 143→174 的每一步都在册各有其偿），无账上调即违规。
   - **判决**语言集 = `py/ts/tsx/rs/go/md/hs`；v0.5.0 起 scan 另有**纯尺寸臂**（js/mjs/cjs/jsx、
     css/scss/less、html/htm、vue、svelte、sh/bash、yml/yaml）——只进 scan 尺寸门 + guard 硬预算 +
     棘轮，永不进判决族（边界权威 = `Lang::scan_only`/`judged_path`，cli/src/scan/lang.rs）。GUI 的
     JS 语义另由 `gui/tests/lens_invariant.js` 一条 CI 腿覆盖。
4. **禁止堆叠式编辑**：更新文档/代码时就地修改，不做"追加新段落覆盖旧段落"式打补丁 ——
   这正是本项目要消灭的行为。
