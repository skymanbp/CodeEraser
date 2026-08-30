# CodeEraser

**[English](README.md)** | 中文（徽章行只住英文 README——两份文件里逐字节相同的块正是本工具判死的冗余）

> 对抗 LLM 引致的代码与文档熵增的橡皮擦。

![架构图：仓库由 Rust 度量侧解析并取指纹（tree-sitter、由逐项目 daemon 保温的 SQLite 指纹索引、引用图、git 窗口），经一条十个家族的 NDJSON wire 进入 Haskell 判决核（策略作为数据随之发布），同一批报告由五张面孔渲染——终端、GUI、MCP 服务器、Claude Code hooks、CI](docs/assets/architecture.zh.svg)

## 简介

长期由 LLM 协作的代码库以同一种方式漂移：同一个函数实现两遍、同一段话贴进三个文件、更新以追加到来、文件只增不减。CodeEraser 在写入当下拦住这种漂移，并在 CI 里把住大门。全链路没有任何模型参与。

两种拒绝发生在写入时、文件落盘之前。一次会**引入** T1/T2 精确克隆（被替换内容原本不携带的重复）的写入在 PreToolUse 当场被拒，指名它复制的区域，并教出能通过的次序；一次让文件超过 750 行——或超过其 `[[rules.class]]` 声明的那条线——的写入同样当场被拒。其余一切都是报告或门：Stop 审计拒绝结束本轮，CI 退出码拒绝提交。

**范围。** 判决语言：Python、TypeScript/TSX、Rust、Go、Haskell、Markdown（<!--ce:count:grammars#word-->六<!--/ce-->套 tree-sitter 语法上的<!--ce:count:langs#word-->七<!--/ce-->个语言码）。纯尺寸臂：js/mjs/cjs/jsx、css/scss/less、html/htm、vue、svelte、sh/bash、yml/yaml——进尺寸门、硬预算与棘轮，永不进语义判决。面：CLI · GUI（<!--ce:count:screens#word-->十<!--/ce-->屏）· Claude Code 插件（<!--ce:count:hooks#word-->三<!--/ce-->钩、<!--ce:count:skills#word-->一<!--/ce--> skill、<!--ce:count:commands#word-->一<!--/ce-->命令、<!--ce:count:mcp_tools#word-->十四<!--/ce-->个只读 MCP 工具）· pre-commit · CI。

## 具体实现——以及它的不同之处

![一个判决如何产生：Rust 度量语法单元、token 指纹、文档 shingle、git 窗口与引用图；Haskell 判决结构与分数、克隆、文档重复、轨迹与审计、存活性与擦除——每行一个 wire 家族；门与逐家族报告交付判决](docs/assets/judgment.zh.svg)

- **在写入的瞬间拦截。** 每个文件的规范化 token（标识符→`ID`、字面量→`LIT`、注释丢弃）以 k = 25、w = 26 做 winnowing，任何 50+ token 的共享片段必有共享指纹。指纹存在由逐项目懒启动 daemon 维护的 SQLite WAL 索引里；PreToolUse 探针 p50 34 ms / p95 37 ms，插件全链 p95 0.50 s。守卫只计**新引入**的重复：被替换内容本已携带的匹配被减掉，故按活流口径 719 条生产探针零误拦（0.00/500）；2,761 事件重放按全文写口径把 32 条拆文件中间态计作误拦（7.03/500）——两种口径都记在 [FPR-REPLAY](docs/FPR-REPLAY.md)。
- **两层克隆，一个判决主体。** T1/T2 是上面的热路径。T3 是冷路径：结构指纹 + MinHash/LSH（128 置换、32 带 × 4 行）生成候选而不丢掉任何一对能过线的，再由 Haskell 核计算 Zhang–Shasha 树编辑距离，以 TSED ≥ 0.85 判定，全程精确整数运算。
- **改过措辞也逃不掉的文档重复。** NFC 规范化的词、5 词 shingle、MinHash/LSH 候选，然后在核内以精确有理数判定 Jaccard ≥ 0.80 或 50 词逐字连续段。
- **被点名而非猜出来的存活性。** 逐语言的解析阶梯（import、再导出、文档链接、资源、包根）喂出按 rung 过滤的图；SCC、自入口根的可达性与四态判决（未引用/不可达 × 私有/公开）带着由未解析站点台账推出的置信码返回。旁边的提及宇宙——每个文本文件里的每个标识符，只以 fnv1a64 哈希存储——产出**未被提及的声明**顾问，它永不把门翻红。
- **被度量的结构。** <!--ce:count:axes#word-->七<!--/ce-->轴（几何、命名多样性、混杂、错位、约定、过期文档、冗余）、逐目录 Tsallis-2 熵、与声明布局的卡方散度、四条成本腿（穿越引用、克隆切口、变动穿越、新文件 φ）的拆分 ROI 定价或内聚性辩词。
- **挪几行骗不过的分数。** 每轴计 floor(1000·v/(v+n))——违规质量除以机会数——加权折叠落在 0–1000。ADR-006 棘轮自动收紧每个上限；增长需要容差 max(+2 %, +10 行) 或具名重立（`CE_ACCEPT_BASELINE=1`），改一个旋钮会让 `ce check` 具名停下而非挪动所有线。
- **时间是一等信号。** 最近 512 个分数点上的 Theil–Sen 斜率（一个野点拽不动中位数）；变动 = 新增 − 按 blame 存活的行；联判格把相似度、图位置与变动合成 merge / delete / churn-hotspot，带理由位与置信。
- **有安全谓词的擦除，不是启发式。** <!--ce:count:erase_classes#word-->三<!--/ce-->类（逐字文档孪生、副本已死的整单元 T1 孪生、置信的非公开死文件）、<!--ce:count:erase_reasons#word-->七<!--/ce-->个冻结理由码、<!--ce:gate:erase.row_cap#digits-->4,096<!--/ce--> 行上限，以及任一已应用判决幸存即失败的收敛重规划。
- **由构造保证的确定性。** 任何判决里没有随机数与时钟；golden 夹具逐字节比对；过线的是码，句子留在各自的面；配置以事实过线，从不以名字。

## 实际效果——同一任务，跑两遍

同一个编码任务——*加折扣、紧凑报表、CSV 与 JSON 输出、API 里的金额格式化*——由脚本化的 agent 在 [`demo/seed`](demo/seed/README.md)（一个 Python + TypeScript 的小型开票服务）的两份相同副本上重放，唯一变量是 PreToolUse 守卫与 Stop 审计是否在环内。种子树先被量过一遍，所以下表每一处发现都是这次任务写出来的。此后两条环各自跑到**自己**的终点——环内没有东西时也就没有东西会拒绝什么，于是那一条终止在最后一次写入。每条判决都是 `ce` 的逐字输出，两棵树由同样六条命令度量。

<!-- demo:begin -->
| | 不带 CodeEraser | 带 CodeEraser |
|---|---|---|
| 种子树，同样六道门实测：克隆块 · 文档孪生 · 死文件 | 0 · 0 · 0 | 0 · 0 · 0 |
| 落地的写入 | 7 / 7 | 5 / 7 |
| PreToolUse 当场拒绝 | 0 | 2 |
| Stop 审计 | 不在环内 | **拦停** — `本会话的编辑留下 2 个触及改动文件的重复块（净 +105 行）` |
| 审计点名的那处修复 | — | 写下之后，审计转为沉默 |
| `ce erase --apply` | — | 移除 1 行：逐字文档孪生 |
| `ce check` 分数（棘轮） | 952/1000 — **FAIL**: ratchet_over, discrete_added | 979/1000 — **FAIL**: ratchet_over |
| T1/T2 克隆块（`ce dedup --check`，预算 0） | 4 (**FAIL**) | 0 (**pass**) |
| 近似克隆对（`ce clone`） | 4 | 0 |
| 重复文档段（`ce docdup --check`） | 1 (**FAIL**) | 0 (**pass**) |
| 死文件（`ce deadcode --check`） | 3 (**FAIL**) | 2 (**FAIL**) |
| 仍待执行的可证安全删除（`ce erase --check`） | 1 (**FAIL**) | 0 (**pass**) |
<!-- demo:end -->

![带 CodeEraser：两次写入在 PreToolUse 被拒并指名所复制的区域，Stop 审计随后拒绝结束本轮、点名溜过去的两个块，它要求的修复落地，擦除计划移除逐字文档孪生](demo/out/with-codeeraser.svg)

被拒的两次写入是对既有 helper 的复制。溜过去的紧凑渲染器是有意留下的诚实边界——整文件重写复制的是自己的块，写入瞬间没有任何**新**重复——而它恰恰是 Stop 审计据以拒绝结束本轮的东西，两个块都被点名；随之而来的那次修复，是全程唯一一次因为门要求、而非因为任务要求而写下的东西。仍然红着的，是只有人能定夺的部分：`invoicer/invoice.py` 93 行，对着容差后 61 行的天花板，棘轮把它留给一次具名重立，而不是悄悄吸收；另有两个文件无人引用——没人链接的新页面，以及 CLI 转投 JSON 之后不再导入的那个渲染器。两份转录、两张 SVG 与表格背后的 JSON 由 [`demo/run.js`](demo/README.md) 生成，并在 CI 里逐字节复核（`demo_replay`）。第一次真实拦截的当日记录：[T1-INTERCEPT](docs/T1-INTERCEPT.md)。

两个近景——第一个就是上表第 1 步；第二个在同一棵种子树上多加了一条 `ce.toml` 声明。

<!-- vignettes:begin -->
**抄来的辅助函数，在文件存在之前就被拒。** 上表第 1 步单独拿出来。理由点名这段内容重复了哪块区域、以及怎样排序才能通过——所以这是一条可执行的拒绝，不是一张否决票。

```console
$ Write invoicer/discount.py
✗ ce：<work>/invoicer/discount.py 的内容与 1 处已索引区域重复：invoicer/money.py:1-18 (89 tokens)。请复用既有实现，而不是另写一份。若是在搬移？先删去源区域：探针以当前树为准校验，同一次写入随即通过。
```

**一条线，两张嘴。** `ce.toml` 给 `invoicer/**` 定下 `file_lines_fail = 40`。写入时守卫拒绝会越线的那次写入，`ce scan` 用同一个数给同一棵树评级——一处声明，钩子与 CI 同读。

```console
$ Write invoicer/invoice.py
✗ ce：这次写入会让 <work>/invoicer/invoice.py 达到 93 行，越过 40 行的硬预算（计划 §4.1）。请拆分文件，而不是继续让它长大。
$ ce scan .
FAIL invoicer/invoice.py:1 file-lines = 51（上限 40）[invoicer/invoice.py]
warn invoicer/report.py:1 file-lines = 35（上限 30）[invoicer/report.py]
已扫描 9 文件 / 19 函数 — 1 warn，1 fail -> FAIL（失败条件：hard_line）
```
<!-- vignettes:end -->

<!-- bench:begin -->
### 最新版本延迟 · v1.3.0

| percentile | `check_warm` | `deadcode_warm` | `dedup_cold` | `dedup_warm` | `docdup_warm` | `hook_probe` | `scan` |
|---|---:|---:|---:|---:|---:|---:|---:|
| p50 ms | 1078 | 923 | 4738 | 381 | 801 | 41 | 518 |
| p95 ms | 1082 | 2093 | 4743 | 384 | 809 | 43 | 2428 |

所有值均由 `contracts/bench/bench.json` 生成；本块手改会被测试拒绝。 [完整回放说明与逐版本系列](docs/BENCH.md) · [网站完整仪表盘](https://codeeraser.dev/zh/bench/)
<!-- bench:end -->

延迟行是 release 构建在同一台固定主机上的回放，只做版本间比较。精度与召回点随各自的评估台账冻结（[EVAL-SET](docs/EVAL-SET.md)），渲染在 [BENCH](docs/BENCH.md)；对照工具（jscpd、similarity-*）标明所测的确切版本。

## 安装、运行与更新

**安装包。** 每个 [release](https://github.com/skymanbp/CodeEraser/releases) 发<!--ce:count:installers#word-->三<!--/ce-->个 GUI 安装包（NSIS `setup.exe` / AppImage / dmg），内含 GUI、`ce` 与判决核 `ce-core`；Windows 安装包把安装目录写入 PATH，检测到 Claude Code 时自动接入下述插件。<!--ce:count:binaries#word-->九<!--/ce-->个二进制与 `SHA256SUMS` 按裁定不签名——用 `sha256sum -c --ignore-missing SHA256SUMS` 校验。

**Claude Code 插件。** `/plugin marketplace add skymanbp/CodeEraser`，再 `/plugin install codeeraser@codeeraser`。启动器按 pin 解析 `ce` 与 `ce-core`：先取命中的本地或 PATH 副本，再钉扎下载，最后才是会自报未校验的 PATH 二进制。

**只要 CLI，或从源码。** 下载 `ce-<版本>-<平台>` 与 `ce-core-<版本>-<平台>`（x86_64-windows / x86_64-linux / aarch64-macos），改名 `ce` / `ce-core` 并排放上 PATH；或 `cargo install codeeraser` 再把 `ce-core` 放旁边；或用钉版 Rust 工具链（`rust-toolchain.toml`）与 GHC <!--ce:tool:ghc#v-->9.14.1<!--/ce--> + cabal 自己构建——`cd core && cabal build all && export CE_CORE_BIN=$(cabal list-bin ce-core)`，再 `cargo install --path cli`。核解析全线一条链：`CE_CORE_BIN` → 旁边的 `ce-core` → PATH；`--core <路径>` 最优先。

| 命令 | 报告 / 判决内容 |
|---|---|
| `ce scan` / `ce dedup` | 尺寸 / 复杂度 / 可读性度量，按文件自己的线分级；T1/T2 克隆块，`--check` 对照预算，摘要键控的暖态缓存；两者都有 `--format sarif` |
| `ce clone` / `ce docdup` | T3 近似克隆；文档重复 |
| `ce graph` / `ce deadcode` | 引用站点与提及宇宙；存活性判决 + 符号顾问 |
| `ce churn` / `ce join` / `ce trend` | git 窗口变动；三信号联判；分数轨迹（stderr 报进度） |
| `ce structure` | <!--ce:count:axes#word-->七<!--/ce-->轴；`--split-candidates` 为每个越过软线的文件计最优缝价 |
| `ce check` / `ce baseline` | ADR-006 棘轮与分数地板，<!--ce:count:fail_conditions#word-->六<!--/ce-->个 fail 条件逐名报在控制台；`baseline` 只在根、且只在具名动作下持久化 |
| `ce erase` | 确定性两段式擦除；默认演练，`--apply` 有干净工作区前置 |
| `ce update` | 最新发布对比本构建，退出码 0 / 1 / 2；`--yes` 两枚 pin 都通过后替换 `ce` + `ce-core`，`--installer` 另存已校验的 GUI 安装包 |
| `ce doctor` / `ce eject` / `ce mcp` | 本机状态；按项目卸载；只读 MCP 服务器 |

控制台输出、`--help` 与钩子自己的拒绝语默认英文，`--lang zh`（或 `CE_LANG=zh`）切中文；JSON schema 与 FAIL/pass 词汇永不翻译。`ce.toml` 里的 `[[rules.class]]` 给一组 glob 自己的尺寸与复杂度线和棘轮容差（`0` = 一行不许长），分数、`ce scan` 阶梯与 PreToolUse 预算读的是同一条线（[ce.toml 参考](docs/reference/ce-toml.md)）。

**更新。** 发布分两段——draft 工件被哈希，pin 提交进 `plugin/bin/manifest.env`，之后 tag 才校验同一批字节（[RELEASE](docs/RELEASE.md)）；插件启动器与 `ce update` 对照的是这些 pin，安装包则由 tag 腿在 Release 发布前对照同一批 pin 校验。`ce update` 读最新 tag 与该 tag 上已提交的 `manifest.env`；判定即退出码，`--yes` 只在没有别的账本管着这份二进制时动手。插件绑定的副本由 `/plugin update codeeraser` 重钉；cargo 安装由 `cargo install codeeraser`；GUI 应用本体由 `--installer` 保存的安装包更新。插件的 SessionStart 行每天通报一次新版本（`CE_UPDATE_CHECK=0` 关闭）；GUI 有更新屏；`/codeeraser:update` 在 Claude Code 里跑检查。

### 三面一体

下表把每项能力恰好认领一次，各集合从代码派生（clap 枚举、Tauri 命令表、MCP 目录、`hooks.json`、`plugin/commands`、`plugin/skills`），CI 门 `face_parity` 拒绝任何没写下来的面与任何没交付的认领。有意的省略是表里的一行，不是沉默。

<!-- parity:begin -->
| 能力 | CLI | GUI（屏 · 命令） | 插件（hooks · MCP · 命令 · skill） |
|---|---|---|---|
| 尺寸 / 复杂度 / 可读性度量 | `ce scan` | `reports`, `scan_report` | MCP `scan` |
| T1/T2 克隆块 | `ce dedup` | `reports`, `dedup_report` | MCP `check_duplication` |
| T3 近似克隆 | `ce clone` | `reports`, `clone_report` | MCP `clone` |
| 文档重复 | `ce docdup` | `reports`, `docdup_report` | MCP `docdup` |
| 引用站点与提及宇宙 | `ce graph` | `reports`, `sites_report` | MCP `graph_sites` |
| 存活性判决 + 符号顾问 | `ce deadcode` | `graph`, `graphcanvas_report`, `deadcode_report` | MCP `deadcode` |
| git 窗口变动 | `ce churn` | `candidates`, `churn_report` | MCP `churn` |
| 三信号联判 | `ce join` | `candidates`, `join_report` | MCP `join` |
| 树尺度结构（七轴、拆分定价） | `ce structure` | `structure`, `structure_report` | MCP `structure` |
| 分数轨迹 | `ce trend` | `trend`, `trend_report` | MCP `trend` |
| 分数、棘轮与地板 | `ce check` | `score`, `check_report` | MCP `check` |
| 基线写入 | `ce baseline` | — 只在 CLI：机器面永不写基线 | — |
| 擦除计划 | `ce erase` | `erase`, `erase_preview` | MCP `erase`, skill `erase` |
| 擦除执行 | `ce erase --apply` | `erase`, `erase_apply` | — 无 MCP 面：执行是人类动作 |
| 本机状态 | `ce doctor` | `doctor`, `doctor_report` | MCP `doctor` |
| 更新检查 | `ce update` | `update`, `update_check` | MCP `update_check`, `/codeeraser:update`, hook `SessionStart` |
| 更新执行 | `ce update --yes` | `update`, `update_apply` | — 插件副本由 `/plugin update codeeraser` 重钉 |
| 写入时守卫 | `ce probe --hook` | — 钩子即插件之面 | hook `PreToolUse` |
| Stop 审计 / pre-commit | `ce audit --hook`, `ce precommit` | — 钩子即插件之面 | hook `Stop` |
| 会话健康行 | `ce health --hook` | — 钩子即插件之面 | hook `SessionStart` |
| 项目 daemon | `ce daemon`, `ce ping` | — 每一面惰性启动 | — |
| 只读报告服务器 | `ce mcp` | — 插件自行注册 | `.mcp.json` |
| 卸载 | `ce eject` | — 只在 CLI | — |
| 实测仪表盘 | — 编译内置序列；README 与官网带同一块 | `bench`, `bench_doc` | — |
| 根锚定 | — 每条命令与钩子都经 `root` 锚定 | `default_root`, `resolve_root` | — |
<!-- parity:end -->

## 技术栈、设计与哲学

- **Rust <!--ce:tool:rust#v-->1.94.1<!--/ce-->**（edition <!--ce:tool:edition#digits-->2,024<!--/ce-->）：`codeeraser` crate——tree-sitter <!--ce:tool:tree_sitter#vminor-->0.26<!--/ce--> 与<!--ce:count:grammars#word-->六<!--/ce-->套语法、rusqlite <!--ce:tool:rusqlite#vminor-->0.37<!--/ce-->（内置 SQLite、WAL，索引 schema <!--ce:ver:schema.index#digits-->15<!--/ce--> / GRAPH_REV <!--ce:ver:graph_rev#digits-->15<!--/ce--> / MENTION_REV <!--ce:ver:mention_rev#digits-->2<!--/ce-->）、`ignore` 遍历器、`interprocess` 命名管道 / Unix socket、clap、serde、更新器 pin 用的 sha2。
- **Haskell（GHC <!--ce:tool:ghc#v-->9.14.1<!--/ce-->，GHC2021，`-Wall -Werror`）**：`ce-core`——每个判决家族、冻结的依赖图。
- **Tauri <!--ce:tool:tauri#digits-->2<!--/ce-->** GUI 直接链接同一 crate，webview 内是无构建步骤的原生 JavaScript；**NSIS / AppImage / dmg** 包内以 sidecar 携带 `ce` 与 `ce-core`。
- **一条 wire。** ce ↔ core 是 stdio 上的 NDJSON，SemVer 协商（proto <!--ce:ver:proto#v-->6.4.0<!--/ce-->，<!--ce:count:families#word-->十<!--/ce-->个家族）；逐项目 daemon 在 `interprocess` 上讲自己的协议（<!--ce:ver:daemon#v-->2.0.0<!--/ce-->）；协议 major 偏斜是具名拒绝，从不猜。
- **设计规则。** ADR-001 Rust 前端 · ADR-002 Haskell 只判决不解析 · ADR-003 懒启动 daemon、30 分钟空闲退出、钩子失败开放 · ADR-004 廉价 PreToolUse、深度 Stop、CI 兜底 · ADR-005 两层克隆 · ADR-006 只收紧的棘轮 · ADR-007 钉扎分发 · ADR-008 策略即 Haskell 数据。计划即契约：[DEVELOPMENT_PLAN](docs/DEVELOPMENT_PLAN.md)。
- **哲学。** 在 Rust 里度量，在 Haskell 里裁决，在其余一切面上渲染。码过线，句子归各面。任何面都不问模型任何事。钩子失败开放并明说。守卫类只有在 [CHANGELOG](CHANGELOG.md) 里有了自己的误报记录才能到 `deny`。文档要么生成要么门控：CLI 与配置参考、<!--ce:count:booklets#word-->十三<!--/ce-->册带机器核验引文的[方法学](docs/reference/methodology.md)、bench 块、demo、等价表、NOTICE。本仓是自己的第一个用户——每次 push 都在这棵树上跑<!--ce:count:gates#word-->六<!--/ce-->道产品门。

## 路线图与已知限制

**限制。** PreToolUse 塑造行为，不是安全墙（shell 写入绕过它——Stop 审计与 CI 是兜底）。钩子遇内部错误失败开放并记录降级。语义判决覆盖上述<!--ce:count:grammars#word-->六<!--/ce-->套语法；JSDoc 与 Rust `///` 按注释而非 docstring 处理；不承诺 T4 克隆。`churn`、`join`、`trend` 以分钟计。二进制未签名。判决本仓需要 `cli/tests` submodule 就位（它是树的读者，永不是被度量的部分）。跨 `[[rules.class]]` 开关、跨 v1.2.0 → v1.3.0 测试子仓搬迁的分数不可比。**路线图。** 后置束在计划书 K–L 行具名：M（评分与评测项、产品小项）、N（分发）与四道决定守卫类能否晋级的证据门。没有自己的误报记录，什么都不晋级。

## 文档

- [CLI 参考](docs/reference/cli.md) · [ce.toml 参考](docs/reference/ce-toml.md)——由二进制与配置 schema 生成，漂移即 CI 变红 · [方法学](docs/reference/methodology.md)（<!--ce:count:booklets#word-->十三<!--/ce-->册，引到实现行）· [结构轴](docs/reference/structure-axes.md) · [尺寸顾问](docs/reference/size-advisory.md) · [擦除契约](docs/reference/erase.md) · [GUI 参考](docs/reference/gui.md) · [插件](plugin/README.md) · [demo](demo/README.md)
- [DEVELOPMENT_PLAN](docs/DEVELOPMENT_PLAN.md) · [EVAL-SET](docs/EVAL-SET.md) · [FIELD-TEST](docs/FIELD-TEST.md) · [BENCH](docs/BENCH.md) · [PERF-BUDGET](docs/PERF-BUDGET.md) · [FPR-REPLAY](docs/FPR-REPLAY.md) · [T1-INTERCEPT](docs/T1-INTERCEPT.md) · [contracts/VERSIONING.md](contracts/VERSIONING.md) · [docs/RELEASE.md](docs/RELEASE.md)——wire SemVer 与两段式发布 runbook
- 官网：[codeeraser.dev/zh](https://codeeraser.dev/zh/) · [工作原理](https://codeeraser.dev/zh/how/) · [技术栈](https://codeeraser.dev/zh/stack/) · [实测](https://codeeraser.dev/zh/bench/) <!-- ce:allow(docdup) -- 文档链接是同一集合，两种语言各列一遍 -->

## 许可证

Apache-2.0——见 [LICENSE](LICENSE)；第三方清单在 [NOTICE](NOTICE)（CI 再生成并门控）。测试套件在 [skymanbp/CodeEraser-tests](https://github.com/skymanbp/CodeEraser-tests)——clone 时带 `--recurse-submodules`。"CodeEraser"™ 为 skymanbp 商标；按 Apache-2.0 §6，许可证覆盖代码，不授予名称。
