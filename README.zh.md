# CodeEraser

**[English](README.md)** | 中文 （徽章行只住英文 README——逐字节相同的块在两份文件里正是本工具判死的冗余）

> 对抗 LLM 引致的代码与文档熵增的橡皮擦。

<img src="docs/assets/gui-structure.png" alt="GUI 结构树图与分数——判决本仓库自身" width="740">

LLM 在长期项目上会漂移出堆叠与打补丁的习性：同一个函数被实现两遍、同一个事实写在三处、更新以追加的方式到来、文件只增不减。
CodeEraser 在写入当下对抗这种漂移——Rust CLI + Tauri GUI 前端、Haskell 判决核、Claude Code hooks、只读 MCP、pre-commit 与 CI。

## CodeEraser 做什么

1. **在重复出现之前就拒绝它。** 一次会**引入** T1/T2 精确克隆——被替换内容原本不携带的重复——的写入，在 PreToolUse 当场被拒；拦截发生在写入当下而不是事后报告里，且被拒的搬移会被告知能通过的次序。
2. **拒绝越过硬线的文件。** 一次让文件超过 750 行——或超过它所属 `[[rules.class]]` 声明的那条线——的写入，同样当场被拒。
3. **揪出写了两遍的文档。** 全树范围的重复段落、注释与 docstring，`--check` 把这个发现变成 CI 退出码。
4. **揪出被改写掩饰过的克隆。** 近似块以语法树编辑距离比对，改了名、换了顺序的副本照样认领自己。
5. **揪出没有人到达的东西。** 文件级引用图承载四态存活性判决与环成员：死代码与死文档是被点名的，不是猜出来的。
6. **只擦除可证明安全的部分。** 死文件、逐字重复的文档孪生、整单元精确克隆——先出计划，只在干净工作区上应用，且永不通过让模型重写你的代码来完成。
7. **在你动手拆分之前先给缝定价。** 每个越过软线的文件都会得到最佳拆分缝的价格，或者一份写明为何该整体保留的内聚性辩词。
8. **给整棵树打分，并守住一条只会收紧的地板。** 七条结构轴得出一个分数，`ce check --fail-under` 为它设门；来自已接受基线的逐文件上限可被清理收紧，却不会被无声放松。
9. **看的是轨迹，不只是今天。** 主线提交上的分数历史、git 窗口内的改动热度，以及把改动、重复与存活性合成一条判决的三信号联判。
10. **从你已经在用的每个面上报。** 终端、GUI、只读 MCP 服务器、Claude Code hooks、pre-commit 与 CI 退出码——每一面渲染的都是同一个核给出的同一批判决。

## 状态

🏁 **v1.2.0——已发布。** K 轮把 1.1.0 之后的欠账清单全数清偿。wire 越过三次断代（4.0.0 退役 erase 的
class 0——冻结位保留、按名拒绝，5.0.0 退役 graph 遗留 flags 列、6.0.0 拓宽基线的配置指纹）并落地
`symbols` 表——去重的 `[节点, 可见性]` 对，说出每个文件声明了什么、有多公开（可见性是声明自身的语法
事实），正是 flags 位 0 从来没有过的那个生产者；导出面自此有了会照着它动手的下游：`unref_public` 上报、
erase 拒绝把公开 API 计划掉的新理由码、join 格同读一位。基线现在给**整份** `ce.toml` 打指纹（`knobs_digest`）——任何配置改动都让
`ce check` 具名停下而非无声挪线；类还可用 `ratchet_tolerance = 0` 冻结自身增长。守卫的重复写入规则经复活
FPR 重放、以 2,761 个真实编辑事件重测，现在只拒**引入**重复的写入（[台账](docs/FPR-REPLAY.md)）。
`ce scan` 与 `ce dedup` 学会 `--format sarif`（本仓自己的 CI 上传 GitHub code scanning），暖态分析走
内容摘要键控的结果缓存（本仓实测 555→294 ms），daemon 客户端有了整场对话期限，长测量在 stderr 上报进度，
GUI 添了第九屏（doctor）。此前：v1.1.0 = 路径规则包（`[[rules.class]]`）、v1.0.1 = 安装器接线、
v1.0.0 = 锁定计划全部里程碑。安装包、[crates.io](https://crates.io/crates/codeeraser)、npm 指针包与
[codeeraser.dev](https://codeeraser.dev) 均已上线 1.2.0。未声明任何类的仓库（含本仓）分数与 1.1.0 完全
一致；一旦声明类，文件被量的线就变了，跨这道开关的分数**不可比较**，需要具名 `CE_ACCEPT_BASELINE=1` 重立。

锁定计划即契约：[docs/DEVELOPMENT_PLAN.md](docs/DEVELOPMENT_PLAN.md)。本仓库对 `main` 的每次 push（外加 pull request 与每周定时跑）
都用自己的扫描器、克隆棘轮、基线与死码/文档重复检查门禁自身。

## 安装

安装面分层：**安装包**是超集（GUI + CLI 上 PATH），**插件**是叠在任一底座上的守卫钩子层，其余路线只装 CLI。

**安装包（推荐）。** 每个 [release](https://github.com/skymanbp/CodeEraser/releases) 发三个 GUI 安装包
（NSIS `setup.exe` / AppImage / dmg，均内含 GUI、`ce` 与 `ce-core` sidecar）。Windows（v0.7.2 起）安装器提权并写入机器 PATH（AppImage/dmg 用户自行加）；
v1.0.1 起还会探测 Claude Code 并自动接入下述插件——装一次即整个产品，卸载只拆它自己添加的注册。

**Claude Code 插件（守卫层）。** `/plugin marketplace add skymanbp/CodeEraser`，再 `/plugin install codeeraser`
（Windows 安装包检测到 Claude Code 时替你跑这两步；AppImage/dmg/纯 CLI 路线手动跑一次）。
引导脚本按 pin 解析：先取本地或 PATH 上的 SHA256 命中副本，再钉扎下载，最后才是会自报未校验的 PATH 兜底
（v0.7.3 起；未命中的 PATH 二进制仍走下载校验）。

**只要 CLI。** 从同一 release 下载 `ce-<版本>-<平台>` 与 `ce-core-<版本>-<平台>`
（x86_64-windows / x86_64-linux / aarch64-macos），改名后并排放 PATH；判决命令经兄弟腿找到核。
也可 `cargo install codeeraser` 构建 `ce`，再把 `ce-core` 放旁边；`SHA256SUMS` 覆盖全部资产。

**从源码。** 前置：钉版 Rust 工具链（仓根 `rust-toolchain.toml`）与 GHC 9.14.1 + cabal（判决核）。

```sh
# 判决核（ce-core）
cd core && cabal build all && export CE_CORE_BIN=$(cabal list-bin ce-core)
cd .. && cargo install --path cli   # CLI（二进制名：ce）
```

核解析全线一条链：`CE_CORE_BIN` → 运行中二进制旁的 `ce-core` 兄弟 →
PATH；显式 `--core <路径>` 永远最优先。

### 二进制 —— 未签名，请校验哈希

发布工件由 [release 工作流](.github/workflows/release.yml)构建并以
`SHA256SUMS` 钉住。**不做代码签名/公证**（2026-08-19 裁定——免费工具
成本收益不成立）：Windows SmartScreen 与 macOS Gatekeeper 会警告，需你
显式允许。永久信任锚是校验链——下载后：

```sh
sha256sum -c --ignore-missing SHA256SUMS
```

Claude Code 插件的引导脚本（`plugin/bin/ce.sh`）自动执行同一套 pin
校验，对不匹配的下载响亮拒绝。

## 命令

| 命令 | 报告 / 判决内容 |
|---|---|
| `ce scan` | 尺寸 / 复杂度 / 可读性度量，核内按文件自己的线（全局或 `[[rules.class]]`）分级；纯尺寸臂另门禁 js/css/html/vue/svelte/sh/yml；`--format sarif` 把发现再编码给 code scanning |
| `ce dedup` | T1/T2 克隆块（winnowing 索引）；`--check` 门控预算；暖态走内容摘要键控的结果缓存；`--format sarif` 把块作为 note 发出 |
| `ce clone` | T3 近似克隆（树编辑距离） |
| `ce docdup` | 文档重复（段落、注释、docstring） |
| `ce graph --sites` / `ce deadcode` | 引用站点；存活性判决 |
| `ce churn` / `ce join` | git 窗口变动；三信号联结。以分钟计而非秒——逐提交一个 git 子进程、逐触及文件一次 blame（[实测](docs/PERF-BUDGET.md)）；两者运行中在 stderr 上报进度 |
| `ce structure` | 树尺度结构判决（七轴）；`--split-candidates` 为越线文件计最优缝价——或写下它的内聚豁免 |
| `ce trend` | 主线历史分数轨迹（缓存可从 git 重建） |
| `ce erase` | 确定性两段式擦除：只计划可证安全的消除（死文件、逐字文档孪生、整单元 T1 孪生），默认演练，`--apply` 有干净工作区前置 |
| `ce check` / `ce baseline` | ADR-006 棘轮 + 分数地板（对 `ce-baseline.json`） |
| `ce mcp` | 只读 MCP 服务器：13 个报告工具，由插件自己注册。`erase` 只到**计划**为止——执行是 CLI 或 GUI 上的人类动作 |
| `ce doctor` / `ce eject` | 健康行；按项目完整卸载（默认 dry-run） |

控制台报告与 `--help` 默认英文，`--lang zh`（或 `CE_LANG=zh`，旗标
优先）切换整行中文查表。JSON 输出与 FAIL/pass 词汇永不翻译——那是
机器面。GUI 自带语言切换钮。长测量（`churn`、`join`、`trend`）用同一
语言在 **stderr** 上报进度，stdout 因而始终是干净管道；仅当 stderr 是
终端才绘制，`CE_PROGRESS=1` / `=0` 可双向强制。

## Guard（Claude Code 插件）

插件在 PreToolUse 拦截（廉价探针）、在 Stop 审计。自 1.0 档位切换起，
两类有 FPR 记录背书的规则——T1/T2 重复写入与硬预算突破（写入使
文件超过其硬线：默认 750 行，或其 `[[rules.class]]` 声明的那条）——**默认 deny**；其余规则在拿到各自的误报记录前保持
observe（台账见 [CHANGELOG.md](CHANGELOG.md)）。1.2.0 起重复写入规则只计**新引入**的重复：被替换内容
本就携带的匹配（携预算内债块文件的全文重写、债块内部的编辑）保持沉默；拆到新文件的写入仍拒——写入瞬间
复制与搬移无从分辨——且拒绝词会教出能通过的次序（先削源）。这次改动背后的 2,761 事件重放在
[FPR-REPLAY.md](docs/FPR-REPLAY.md) 入账。`ce.toml` 的
`[guard] mode` 显式声明可覆盖所有类别；软线与硬预算之间的渐进区
默认只记台账，`[guard] zone_tiers` 显式声明才启用位置→档位映射
（<25% observe / 25–75% warn / >75% ask）。诚实边界：PreToolUse 塑造行为，
不是安全墙——shell 写入可绕过它。兜底按各腿所测内容分工：Stop 重新判决
净 LOC 与涉改重复，CI 门负责硬尺寸墙与棘轮。

## 路径类（`[[rules.class]]`）

生成代码、vendored 树与测试 fixture 很少配得上手写代码那一套尺寸与复杂度线。`ce.toml` 里的路径类
让一组 glob 拥有自己的线——声明序中第一个 glob 命中的类认领该文件，谁都不命中的文件沿用全局表：

```toml
[[rules.class]]
name  = "vendored"
globs = ["third_party/**", "**/*.pb.rs"]   # 与 exclude 列表同一套 glob 方言
[rules.class.knobs]
file_lines_warn = 600
file_lines_fail = 1200                       # 该类自己的硬线
cognitive_warn  = 25
ratchet_tolerance = 0                        # 该类一行都不许长
```

三面同读这一条线、彼此不可能打架：分数的尺寸与复杂度轴（wire proto 3.1.0——连续行携类下标、
`classKnobs` 表伴行过线，而基线永远三列，所以类是本次收费参数、绝非棘轮事实）、`ce scan` 阶梯
（proto 3.2.0——`rowClasses` 与 `gradeOverrides` 伴行过线、回复原样回显）、PreToolUse 硬预算
（零 wire——钩子在本地解析文件自己的表）。类名与 glob 永不过线，过线的只有类下标与 knobs
（ADR-008）。至多 64 类；fail 线低于 warn 线的类在加载时被拒，与全局阶梯同律。

自 proto 5.1.0 起，类还可声明 `ratchet_tolerance`——它自己的 ADR-006 容差，单位是行数。一旦声明就
**取代两条全局腿**，故 `0` 意味着该类一行都不许长、全局 `max(+2%, +10)` 救不了它——这正是 vendored 树
与冻结夹具想要的设置，也正是让这类树别再花掉手写代码需要的增长额度的那一条。而既然改一处配置本可以一次挪动
所有的线，基线记录它**在哪份 `ce.toml` 下立的指纹**：改 glob、改阈值、改分数旋钮、改 exclude——凡是解析结果
里有的，`ce check` 都会**具名停下**（`knobs_digest`），而不是悄悄放松。同意一份新配置与同意一条新地板是
同一个动作——`CE_ACCEPT_BASELINE=1 ce baseline`，在 git 里看得见。它给**整份配置**而不是某一张表打指纹，
因为「挑表」正是缺口的成因：对第一版（只围类表）的对抗审查实测发现，`[score] viol_cost = 0` 能把一个仓
从 939/1000 不及格变成 1000/1000 及格，而轴上仍老实报着违规费。

配置等于出厂默认的仓库判决逐字节不变、不发指纹、基线文件也不多一个键；一旦声明类，文件被量的线就变了，
跨这道开关的分数**不可比较**。
键位见 [ce.toml 参考](docs/reference/ce-toml.md)，收费定律见
[方法学 05](docs/reference/methodology/05-scoring-and-the-adr-006-ratchet.md)。

## 内部构造 / 技术栈

![详细技术栈：Rust 度量、版本化 wire、Haskell 判决、产品面与发布 pin 链](docs/assets/stack.svg)

Rust 负责面对源码的工作：tree-sitter 解析、SQLite WAL 指纹索引、解析阶梯、
git 窗口、逐项目懒启动 daemon 与事实组装。事实只经一条 SemVer 协商的
NDJSON wire 过界。Haskell 负责产品判决：分数与棘轮、图存活性与 cycle
成员、克隆/文档重复、拆分缝价与 erase 授权。终端、Tauri GUI、只读 MCP、
Claude Code hooks 与 CI 渲染或执行的是同一批报告形状。

- push 工作流运行六条自门禁产品腿，含显式分数地板；本仓就是常设 dogfood fixture。
- ADR-006 上限与违规集存于 `ce-baseline.json`；清理会收紧，增长必须显式重立。
- `ce.toml` 的 `[[rules.class]]` 为一组 glob 声明自己的尺寸与复杂度线：分数、`ce scan` 阶梯与 PreToolUse 硬预算读的都是文件自己那条；类名与 glob 永不过线。
- CLI/配置参考由生成器产出；十二册方法学的引文、导航与中英常数均由机器检查。
- 守卫规则只有在自己的误报记录写入 [CHANGELOG.md](CHANGELOG.md) 后才能晋级 deny；无记录者保持 observe。
- `ce erase` 组装确定性事实，再由 Haskell 安全谓词授权删除；从不让模型重写代码。
- 发布分两段：hash 来自 draft 工件，pin 提交进树，tag 校验同一批字节且不重建。

[网站技术栈](https://codeeraser.dev/zh/stack/) · [判决方法学](docs/reference/methodology.md) · [wire 契约](contracts/VERSIONING.md)

## 评估仪表盘

<!-- bench:begin -->
### 最新版本延迟 · v1.2.0

| percentile | `check_warm` | `deadcode_warm` | `dedup_cold` | `dedup_warm` | `docdup_warm` | `hook_probe` | `scan` |
|---|---:|---:|---:|---:|---:|---:|---:|
| p50 ms | 1111 | 450 | 2958 | 267 | 621 | 34 | 586 |
| p95 ms | 1131 | 462 | 2979 | 275 | 659 | 37 | 609 |

### 冻结评估点

| 指标 | 值 | 来源 |
|---|---|---|
| `docdup_d3_precision` | 17/17 scoped (100%) | `docs/EVAL-SET-M5-3.md:81-87 + contracts/eval/docdup-precision-*-v1.json` |
| `docdup_d1_recall` | 100% | `docs/EVAL-SET-M5-3.md:81-87 + contracts/eval/docdup-precision-*-v1.json` |
| `t3_precision` | 61 answered / 0 wrong (1.000) | `docs/EVAL-SET-M5-3.md:41-47 + contracts/eval/t3-precision-*-v1.json` |
| `graph_precision` | overall gate >= 0.90 held | `docs/EVAL-SET.md:280-292 + contracts/eval/graph-precision-*-v1.json` |
| `fourclass_fpr` | 0/600 flagged (gate <= 1%) | `contracts/eval/fpr-fourclass-v1.json + docs/EVAL-SET.md:131-140` |
| `guard_fpr_per500` | 0.00 per 500 edits | `docs/FPR-REPLAY.md:16-36 + :47-94` |
| `l2_moved_recall` | 547/547 cross-file moved lines | `docs/EVAL-SET.md:97-129 + contracts/eval/commit-l2*-v1.json` |
| `dedup_recall_vs_jscpd` | cobra 106/109 raw -> 106/106 attributed | `contracts/fixtures/crosscheck/DEDUP-CALIBRATION.md:96-137` |
| `t3_recall_vs_similarity` | zod 0.50 / requests 0.158 / cobra 0.154 (raw) | `docs/EVAL-SET-M5-CLOSE.md:38-63` |

所有值均由 `contracts/bench/bench.json` 生成；本块手改会被测试拒绝。 [完整回放说明与逐版本系列](docs/BENCH.md) · [网站完整仪表盘](https://codeeraser.dev/zh/bench/)
<!-- bench:end -->

## 文档

- [技术栈](https://codeeraser.dev/zh/stack/) · [评估仪表盘](https://codeeraser.dev/zh/bench/) — 网站组件地图与完整生成记录
- [CLI 参考](docs/reference/cli.md) · [ce.toml 参考](docs/reference/ce-toml.md) — 由二进制与配置 schema 生成；漂移即 CI 门变红
- [DEVELOPMENT_PLAN](docs/DEVELOPMENT_PLAN.md) · [EVAL-SET](docs/EVAL-SET.md) · [FIELD-TEST](docs/FIELD-TEST.md) — 锁定计划、冻结评估设计与真实仓发现
- [PERF-BUDGET](docs/PERF-BUDGET.md) · [FPR-REPLAY](docs/FPR-REPLAY.md) · [T1-INTERCEPT](docs/T1-INTERCEPT.md) — 实测预算与重放台账
- [BENCH](docs/BENCH.md) — 逐版本延迟与冻结评估点，由 `contracts/bench/bench.json` 生成
- [contracts/VERSIONING.md](contracts/VERSIONING.md) · [docs/RELEASE.md](docs/RELEASE.md) — wire SemVer 与两段式发布 runbook
- [docs/reference/methodology.md](docs/reference/methodology.md) — 每个判决的数学实现，一族一册，公式与常数逐条引到实现行
- [structure axes](docs/reference/structure-axes.md) · [size advisory](docs/reference/size-advisory.md) · [erase contract](docs/reference/erase.md) · [GUI 参考](docs/reference/gui.md) — 聚焦行为契约

## 许可证

Apache-2.0 —— 见 [LICENSE](LICENSE)。第三方清单：[NOTICE](NOTICE)
（由 `cli/tests/notice_gate.rs` 在 CI 中再生成并逐字节门控）。

"CodeEraser"™ 为 skymanbp 商标。按 Apache-2.0 §6，
许可证覆盖代码，不授予名称使用权。
