# CodeEraser 开发计划

> **版本** v1.3 · 2026-08-07 · 状态：🔒 已由 cc-memory 锁定
> 本文件是本仓库唯一权威计划。修改流程：改本文件 → 重新 ccm 锁定 → 才能动代码。
> v1.2 拍板 12 项决策（§9）并经第二轮 Opus delta 攻击评审收口为 v1.3；
> v1.0 曾经首轮攻击评审重写。两轮评审记录见 [docs/reviews/](reviews/)。
> 本文件行数以锁定时为棘轮上界：只准变短，不准变长；更新必须就地改写。
> 调研依据：2026-08-06 七路并行实证调研（GitHub API / 官方文档 / 论文原文），关键事实附 URL。

---

## 0. 一句话定位

**在 LLM 写入代码/文档的当轮拦截熵增** —— diff 级、实时、可强制。
"可强制"指能力而非默认值：deny 能力从 M3 起存在，默认档位按 §4.2 的演进路线随证据升级。

## 1. 问题与证据（立项依据）

- GitClear《The Maintainability Gap》（6.23 亿次变更，2023–2026）：每百万变更行中的
  **重复代码块 40.3 → 73.0（+81%，历史最高）**；moved/重构占比 21%(2022) → **3.8%(2026)**；
  copy/paste 升至 **15.7%**。指标是按变更行归一化的，不受仓库规模效应影响。
  <https://www.gitclear.com/the_ai_code_quality_maintainability_gap>
- 「Volume-Quality Inverse Law」（arXiv 2605.02741）：代码总量与架构级 smell 计数相关
  **ρ=0.94 (p<0.001)**（注意：相关非因果，且 smell 计数含规模效应；据此只主张
  "体积与腐化强相关"，不主张因果）。
- 反证诚实纳入：arXiv 2603.27130 发现 AI 与人类代码在"代码层"差异极小。
  → 卖点锚定在**编辑/提交行为层**（重复、堆叠、净增长、churn），不是"AI 代码天生更烂"。

## 2. 竞争格局与差异化

| 先例 | 占据的位置 | 没做的（= 我们的空位） |
|---|---|---|
| [jscpd](https://github.com/kucherenko/jscpd) v5（Rust 引擎，自带 MCP/skill/SARIF） | Token 级 T1 查重 + 事后报告 | T3 near-miss；写入当轮强制拦截；结构度量 |
| [desloppify](https://github.com/peteromallet/desloppify)（3k★） | 全库健康扫描 | "True incremental or diff-only scanning is not the supported model **yet**"（README 原文；注意 yet——它可能补上，见 R5 触发器） |
| [CodeScene](https://codescene.com/engineering-blog/codescene-ci-cd-quality-gates/) | PR 级 Code Health 门禁 | 商业闭源；不做写入时拦截 |
| [colbymchenry/codegraph](https://github.com/colbymchenry/codegraph)（65k★） | 代码图谱喂 LLM 上下文 | 图谱项目均不做克隆/重复判定 |
| [mizchi/similarity](https://github.com/mizchi/similarity)（TSED） | T3 跨文件相似检测（最近先例） | 无 gating、无插件面、无文档查重 |
| [fuck-u-code](https://github.com/Done-0/fuck-u-code)（7.3k★） | 快照式质量评分 + 幽默输出 | 重复检测仅文件内且正则实现；无时间维度 |
| [betterer](https://github.com/phenomnomnominal/betterer) | ratchet 棘轮范式（JS 生态） | 无跨语言主导者 |

**空位（调研 agent 逐 README 核对）：「每次编辑判定真修改 vs 堆叠新增」无人在做。**
三个差异化判决：

1. **写入当轮拦截**：编辑落盘当轮完成判定与强制（PreToolUse 廉价门 + PostToolUse/Stop
   深判与阻断，见 ADR-004），超限时给出量化依据并指回既有实现；
2. **三信号 join**：克隆相似度 × 依赖图位置 × git 历史（co-change/churn）联合给出
   "删除/合并候选"判决——调研确认没有任何项目同时具备三者；
3. **编辑四分类**：把一次 diff 分解为 matched / novel / moved / deleted，直接度量
   "是更新还是堆叠"。

## 3. 产品形态

| 形态 | 载体 | 里程碑 |
|---|---|---|
| **主动**：`ce` CLI | 单二进制（Rust），`codeeraser` 为等价 alias；`ce scan / check / dedup / report / baseline / doctor / eject` | M1 起 |
| **被动**：Claude Code 插件 | hooks（PreToolUse/PostToolUse/Stop/SessionStart）+ skills + `bin/` | M3 |
| **被动**：通用 agent 集成 | pre-commit、CI（退出码 + `--fail-under`）、**最小 MCP server（M3）**、完整 MCP（M7） | M3/M7 |
| GUI | Tauri（复用 Rust 前端） | M6 |
| 分发 | 私有开发；M3 后发 **0.x 预览**（本地/私有 marketplace 自用 dogfood；预览期二进制走 ADR-007 air-gapped 手动放置——私有仓 Releases 无匿名下载，D2-3）；M7 公开上 marketplace + GitHub Releases | M3/M7 |

## 4. 功能规格

### 4.1 主动模块（用户按需启用，`ce.toml` 配置，纯声明式）

| 模块 | 检查项 | 默认阈值（出处经核实） | 里程碑 |
|---|---|---|---|
| `size` | 文件 LOC；函数长度；参数个数 | 文件 300 警告 / 750 阻断（ESLint max-lines=300；Sonar S104=750）；函数 50/75（ESLint=50；Sonar S138=75）；参数 5（Pylint） | M1 |
| `complexity` | Cognitive Complexity 主判罚；Cyclomatic 辅助 | CoC 15（Sonar S3776）；CC 10–15（Sonar S1541=10 / lizard=15）。证据边界如实声明：ESEM 2020 元分析中 CoC 仅在理解耗时（r=0.54）与主观评分轴有支持，正确率轴无支持（r=−0.13 CI 跨零）；arXiv 2303.07722 中 CC 略优于 CoC。选 CoC 主判罚的理由是其对嵌套的惩罚正对准"堆叠"形态，而非"已证明的可维护性代理" | M1 |
| `readability` | 命名规范、嵌套深度、注释密度 | 不用 Maintainability Index 作主分（van Deursen 批判：1994 系数从未重标定、与 LOC 共线）。主判罚永远 = LOC + CoC + 重复率 | M1 |
| `clone` | 跨文件 T1/T2（热路径）；T3 near-miss（冷路径）；**不承诺 T4**（arXiv 2606.25272：SOTA 在 T4 全线退化） | T1/T2 min-tokens 50（jscpd 默认）；T3 TSED 0.85（mizchi/similarity 默认） | M2/M5 |
| `docdup` | Markdown/纯文本段落 + **代码注释/docstring** 查重（与 `clone` 联动）：shingle + MinHash/LSH 粗筛 → Jaccard 复核 | 段落粒度；逐字下界 50 tokens（Lee et al. 2107.06499） | M5 |
| `churn` | 函数级追加 vs 重写比例、两周 churn、co-change 纠缠对 | 先例：GitClear 指标 + ops-codegraph-tool co-change | M4 |
| `graph` | import/调用边抽取、跨文件符号解析、入度/环 | 工程量锚点：ops-codegraph-tool 用 6 级 import 解析达 precision 94.9%/recall 66.7%——这不是一行验收能带过的子系统 | M5 |
| `deadcode` | 无引用符号/文档段落（图入度 = 0 ∧ 非入口） | 依赖 `graph` | M5 |
| `score` | 综合评分 + 棘轮基线（语义见 ADR-006） | 权重表配**敏感性测试**：扰动任一权重断言总分变化（fuck-u-code 的真实 bug 是权重字段从未被评分路径读取——"权重和=1"断言测不到死字段） | M5 |

评分极性全程统一"越高越好"。幽默评语表（i18n 静态查表）为可选彩蛋，默认关闭，`--roast` 开启。

**排除模型（M1 起内置，A2d）**：默认排除 lockfile、minified/生成物（`*.min.js`、
protobuf/OpenAPI 产物）、vendored、快照测试、migration、二进制/数据文件；叠加
`.gitignore`、`.ceignore` 与 `ce.toml` 的 glob。**类别级豁免（D2-5）**：license 文件头、
结构化 docstring 骨架（`Args:`/`Returns:` 等模板行）不入 docdup 语料。豁免三条路：行内
`ce:allow(<rule>) -- <why>`（无 why 即违规）、`.ceignore`、基线豁免存量（JSON 等无注释语法文件用后两条）。

### 4.2 被动模块（guard，Claude Code 插件）

拦截点依据官方 hooks 文档（<https://code.claude.com/docs/en/hooks.md>，2026-08-06 核实；
PostToolUse 不能阻断工具执行，但可反馈；强制阻断点 = PreToolUse 与 Stop）：

| Hook | 职责（与 ADR-004 混合强制点一致） |
|---|---|
| `PreToolUse`（`Edit\|Write`） | 只做**无需 AST 的廉价检查**：路径排除、目标文件当前 LOC 预算、单次写入体积、`new_string` 片段对指纹索引的 T1/T2 探针。超限 → `permissionDecision:"deny"/"ask"` + 指回既有 `file:line`。不做 AST diff（避免重放 Edit 落盘语义这一隐藏子系统，评审 A2a） |
| `PostToolUse`（`Edit\|Write`）/ `FileChanged` | 拿**已落盘全文**做深判：AST 级度量增量、四分类（M4 起）、跨文件查重。结果写入 `.ce/session-findings`，不阻断 |
| `Stop` | 本轮净效果审计（基于 **git diff**，因此对 Bash/`>>` 写入同样生效）：净 LOC、新增重复块、（M4 起）四分类汇总。引入净冗余而声称完成 → `decision:"block"` 要求返工 |
| `SessionStart` | 引导二进制（见 §5.9）；注入 guard 健康状态一行（daemon 是否存活、索引 freshness、上会话降级计数） |
| `UserPromptSubmit`（可选） | 廉价启发式标记本轮意图（更新 vs 新增），仅作 §4.3 的可选辅助信号，非判定前提 |

**诚实边界（A2b）**：PreToolUse 是**行为塑形层，不是安全边界**——agent 可用
`Bash: echo >>`/`sed -i` 绕过。兜底 = Stop 审计走 git diff（与写入工具无关）+ CI 门禁。
文档必须如实写明这一点，不得宣传为"不可绕过"。

**默认档位演进路线（A1，写死在此，不许默认永远 warn）**：

1. 0.x（M3–M4）：默认 `warn`；`deny` 能力存在，用户可按规则开启；
2. M4 的 FPR 门（§6）通过后：**T1/T2 精确重复写入**与**硬预算超限（文件 >750 行）**两类
   规则默认升为 `ask`；
3. 1.0（M7）：上述两类默认 `deny`，其余规则默认 `ask`/`warn` 按各自 FPR 记录决定。
   每次默认档位变更在 CHANGELOG 记录依据（FPR 数据）。

### 4.3 F4「更新监督」判定模型（核心创新）

四分类：**matched / novel / moved / deleted**。算法借鉴 difftastic 的代价模型思想
（novel atom 高成本、节点匹配低成本，Dijkstra 最短路，
<https://difftastic.wilfred.me.uk/diffing.html>）但自研实现——difftastic 的 JSON 标记
unstable、有图规模/文件体积硬上限、且**不识别 moved**，而 moved 恰是 GitClear 体系的
关键健康信号。

**Fallback 阶梯（B3c，先易后难，各级都是上一级的对照组）**：
L0 = `git diff --numstat -M -C --find-copies-harder`（行级 added/deleted/moved，零自研）；
L1 = L0 + 函数边界对齐（tree-sitter 符号表）；
L2 = AST 级四分类（自研代价模型）。M4 从 L0/L1 建立 baseline 准确率，L2 必须证明
相对 L1 的增量收益，L2 不达标时产品退回 L1 而非退回无。

判定规则（**意图无关**，A2c 修复；意图信号仅可选增强）：

- novel 内容与仓库既有代码/段落结构指纹相似度超阈值 → **重复实现嫌疑**（主规则，
  不依赖任务意图；写新测试/新 endpoint 等正常新增不含相似既有实现，不触发）；
- `novel ≫ deleted` **且** novel 与被修改函数同名/同签名/高相似 → **堆叠嫌疑**
  （旧实现没删、新实现又写一份的典型形态）;
- 文档编辑中新增段落与既有段落 MinHash 相似 → **重复陈述嫌疑**；
- 每个判定输出量化依据（novel/moved/deleted 行数、相似度、指回位置），
  绝不输出裸"感觉太长"。不承诺语义级矛盾检测。

### 4.4 CLI UX 与输出

- 退出码：`0` 通过 / `1` 违规 / `2` 内部错误；`--fail-under <score>`（与棘轮合成语义见 ADR-006）。
- 格式：console、JSON（agent/skill 消费）、SARIF、Markdown。
- `analyze → JSON → skill 解读`分工：CLI 只出结构化事实，skill 负责向 LLM 解释怎么改。
- **hook 输出 token 预算（B4，anti-bloat 工具不得自己成为上下文熵源）**：
  warn 注入 ≤ 200 tokens/事件；同一 `(rule, file)` 每会话只报一次，后续静默累积；
  深度报告落盘 `.ce/`，由 skill 按需读取；Stop 汇总 ≤ 400 tokens。预算进 M3 验收。

## 5. 架构

```
┌ Claude Code / 其他 agent ──────────────────────────────────┐
│ hooks(Pre/PostToolUse/Stop/SessionStart) · skills · bin/ce │
└──────────────┬（hook 每次触发 = 短命 ce 进程）─────────────┘
               ▼
┌ 前端 ce (Rust 单二进制) ───────────────────────────────────┐
│ CLI/配置/排除 · tree-sitter 解析(官方 Rust 绑定) · 热路径  │
│ 廉价检查 · hook I/O · GUI(Tauri) ──┐                       │
└────────────────────────────────────┼───────────────────────┘
              named pipe(Win)/UDS ─► ▼
┌ ce daemon (同一 Rust 二进制, per-project, 懒启动) ─────────┐
│ 指纹索引(SQLite WAL, 唯一写者) · git 历史抽取 · 文件监听   │
│ 子进程: ce-core (Haskell) ↕ NDJSON over stdio(均长驻)      │
│   判决层: 规则引擎(hlint 式双层) · 四分类(L2) · TSED       │
│   依赖图/三信号 join · 评分与棘轮                          │
└────────────────────────────────────────────────────────────┘
```

**职责边界（B1 采纳后）**：延迟敏感的热路径（PreToolUse 廉价检查、索引探针）完全在
Rust 进程内完成，不跨语言；**一切"判决"**（四分类 L2、规则引擎、评分、棘轮、图分析、
TSED）在 Haskell——这些全部位于放宽预算的路径上（PostToolUse 异步 / Stop 秒级 / 批扫），
Haskell 承重且不背 1s 预算。

### 架构决策记录（ADR，偏离须先改本文件）

**ADR-001 前端语言 = Rust（Go 落选）。**
tree-sitter Rust 绑定是官方一等公民且语法 crate 跟随 0.26.x；最近先例（difftastic、
mizchi/similarity、jscpd v5 引擎、ast-grep）全是 Rust，可参考复用（ast-grep-core，MIT）；
GUI 由 Tauri 覆盖。Go 无以上任何优势。

**ADR-002 Haskell 不拥有解析层；职责 = 判决层。**
实证：Hackage `tree-sitter` 停在 0.9.0.3（2022-04-12），包描述自劝退
（<https://hackage.haskell.org/package/tree-sitter>）；github/semantic 已 archived；
唯一活跃替代 hs-tree-sitter 是 AGPL-3.0-only + 单人维护。→ Rust 解析并输出归一化 IR
（符号、span、结构指纹、import 边；**token 流只入本地索引，不跨进程**——A6），
Haskell 消费 IR 做判决。wire format 借鉴 ast-grep `--json=stream`。

**ADR-003 进程模型（A3 拆分后）。**
- hook 触发 = 短命 `ce` 进程；重活委托给 **per-project daemon**（同一二进制 `ce daemon`，
  首次使用懒启动，空闲 30 min 自动退出）。
- 通道：Windows named pipe / Unix domain socket，管道名 = 项目路径哈希，凭据即本用户。
- daemon 是 SQLite 索引的**唯一写者**（WAL + busy_timeout），多 session/subagent 并发
  由 daemon 串行化。
- 版本 skew：连接握手带协议版本，不匹配 → daemon 自杀重启（新二进制路径由客户端传入）。
- 冷启动：首次索引后台异步构建；未就绪期间 guard **显式降级**为廉价检查档（降级状态
  进 SessionStart 健康行与 Stop 汇总，不静默——A9f）。
- Haskell core 是 daemon 的长驻子进程，NDJSON over stdio，全程 `ByteString` +
  `hSetBinaryMode`（规避 GHC #10762/#15021 的 Windows code page 坑）；
  **禁止 Haskell DLL**（GHC #16429/#23644 未解决；`foreign export`+DllMain 官方警告冻结）。

**ADR-004 强制点 = 混合（B3a 采纳，替代 v1.0 的 PreToolUse 独担）。**
PreToolUse 只做无需 AST 的廉价检查（见 §4.2）；AST 深判在 PostToolUse/FileChanged
（已落盘全文，无需重放 Edit 语义）；强制力由 PreToolUse（廉价规则）+ Stop（深判结果）
提供。否决"PreToolUse 独担"的理由：需自建与 Claude Code 逐字节等价的 Edit 落盘语义
重放器（unique 匹配、replace_all、空白/CRLF 归一化），任何偏差 = 判定一个与实际落盘
不同的文件；且 `new_string` 片段常含 ERROR 节点无法可靠建树。代价（文件短暂脏后被要求
返工）在 agent 工作流中可接受。

**ADR-005 克隆检测两层；自研 winnowing 而非复用 jscpd（否决理由记录，B3b）。**
热路径：归一化 token 流 winnowing/Rabin-Karp 指纹倒排索引（Schleimer et al. SIGMOD'03
的无漏检下界保证），SQLite 分片、增量失效。冷路径：候选集 → AST 结构指纹 → TSED（T3）。
不复用 jscpd 引擎的理由：① 需要进程内毫秒级探针（jscpd 是批扫 CLI，Node 生态）；
② 需要增量失效的常驻索引（jscpd 无）；③ 避免 Node 运行时依赖。代价：自研索引正确性
风险，用"增量 ≡ 全量重建"的 property 测试覆盖（§7）。检出能力对齐 jscpd 可检出集
（验收含配对精度，§6 M2）。embedding 只进离线报告。排除：后缀数组（构建不增量）、
全仓 pairwise 树编辑、全仓 embedding。

**ADR-006 棘轮语义（B5 修复）。**
- **连续型指标**（文件 LOC、函数 CoC）：per-file/per-function ceiling = 基线值；
  超 ceiling 即 fail，低于 ceiling 自动收紧到新值。修 bug 需要加行时：ceiling 有
  单次编辑 +2% 或 +10 行（取大）的容差，容差消耗计入 Stop 汇总。
- **离散型违规**（clone 实例、deadcode 符号）：基线是**违规集合**（指纹标识）；
  新增成员即 fail，移除成员自动收基线。
- 与 `--fail-under` 合成：有基线的仓库以棘轮为主门，`--fail-under` 为下限保险；
  两者任一 fail 即 fail。`ce-baseline.json` 提交进仓库（betterer 范式）。

**ADR-007 插件工程约束（官方文档核实，2026-08-06）。**
- 布局：`.claude-plugin/plugin.json`（省略 `version` 则 commit SHA 即版本；**自 0.x 预览起
  改带显式 version**——D2-2）；仓库根 `.claude-plugin/marketplace.json` 即 marketplace。
- 安装拷贝进 `~/.claude/plugins/cache` → 禁止越界相对引用。
- 二进制分发路径**唯一化（A9a）**：仓库 `bin/` 只放轻量启动脚本；真身二进制由
  SessionStart 从 GitHub Releases 下载到 `CLAUDE_PLUGIN_DATA`（跨版本保留），
  **HTTPS + SHA256 pinned 在插件清单内**，校验失败拒绝执行并明示。三平台二进制
  预期 8–19 MB/个（shellcheck 7.69 MB ~ hlint 18.99 MB 区间），不塞仓库。
  air-gapped 模式：允许用户手动放置二进制 + 本地校验。代码签名/公证列入 M7 发布
  验收（Windows SmartScreen / macOS Gatekeeper），完成前 README 明示未签名状态。
- DENY 协议：exit 2 + stderr，或 exit 0 + `{"hookSpecificOutput":{"permissionDecision":"deny",...}}`；
  自设 `timeout` 并按 R3 fail-open + 显式记录。
- ⚠️ 官方文档无 Edit/Write hook payload 逐字示例 → M0 用 echo-hook 实测 dump 固化 fixture。

### 5.9 安全与隐私（A9，上市场的准入条件）

1. **网络承诺**：`ce` 与 `ce-core` 在分析路径上**绝不联网**；唯一网络行为是 SessionStart
   二进制下载（可关）。embedding 特性仅限本地模型；任何云 API 需按仓库显式 opt-in。
2. **索引隐私**：SQLite 索引只存 token 哈希指纹、span、符号名，**不存源代码文本**；
   位置在 `CLAUDE_PLUGIN_DATA`（或 CLI 模式下项目 `.ce/`，入 `.gitignore` 模板）。
   默认排除 secrets（`.env`、`*.pem`、`id_*`、`*.key` 等内置 glob + `.gitignore` 项）。
3. **配置信任模型**：`ce.toml` 纯声明式（阈值/开关/glob），**不可指定可执行命令**——
   clone 恶意仓库不产生代码执行。
4. **卸载**：`ce eject` 清除基线、`.ce/`、`CLAUDE_PLUGIN_DATA` 索引；插件卸载文档含
   eject 指引。
5. **可见性**：`ce doctor`（daemon 健康、索引 freshness、降级计数）；SessionStart 健康行；
   降级事件计入 Stop 汇总——fail-open 但绝不静默失效。

### 5.10 仓库布局（M0 建立）

```
CodeEraser/
├── .claude-plugin/marketplace.json   # 本仓库即 marketplace
├── plugin/                           # hooks.json, skills/, bin/(启动脚本)
├── cli/                              # Rust workspace：ce（CLI+daemon+GUI 后端）
├── core/                             # Haskell cabal：ce-core（判决层）
├── contracts/                        # 契约版本化机制 + 双语言共享 golden fixtures
├── docs/                             # 本计划、协议文档、评审记录
└── memory/                           # cc-memory（已存在）
```

## 6. 里程碑（工期为单人 + agent 协作的粗估，标 ± 者不确定度高）

| # | 内容 | 工期 | 验收标准（量化、可复跑、防作弊） |
|---|---|---|---|
| **M0** 契约机制与骨架 | License 已拍板 **Apache-2.0**（LICENSE 已入库）；`ce` 命名撞名核查（crates.io/npm/brew）；契约**版本化机制**（信封格式 + SemVer 协商，内容不冻结——B1）；echo-hook 实测 Edit/Write payload 固化 fixture；双工程骨架 + 三平台 CI（私有仓计费额度评估：macOS 10× 倍率，限 tag/夜间触发——D2-8）；工具链锁定并**实测依赖可解**（C3：`cabal build` 全依赖集在 GHC 9.14 LTS 通过，Stackage 快照记录在案）；热路径延迟分解表（fork→ce 冷启动→探针→回传各项预算） | 1–2 周 | CI 三平台绿；`ce --version`↔`ce-core --version` 握手；payload fixture 入库；ce 冷启动实测 < 100ms（Windows 含 Defender 首扫除外，单列记录） |
| **M1** 度量 MVP | `size`+`complexity`+`readability`+排除模型；首发语言 **TypeScript / Python / Rust / Go / Markdown**（Markdown 仅 size；Haskell 支持后移 M5——无外部对照物，B2）；`ce scan` console/JSON | 3 周 ± | fixtures 从**钉死 commit 的真实仓库随机抽样**（清单入 contracts/）；CC 与 lizard(TS/Py)、rust-code-analysis(Rust)、gocyclo(Go) 一致率 100%；CoC 与 gocognit(Go) 对拍且分歧全部清单化归因（规范差异注明出处，无未解释分歧——D2-6）；CoC 过 Sonar 白皮书共通例题 + 自建 golden；分歧 case（短路、装饰器、可选链）显式收录不回避 |
| **M2** 克隆热路径 + 进程模型 | winnowing 指纹索引（token 归一化覆盖全部五门首发语言——D2-4）、`ce dedup`、daemon（ADR-003 全项：懒启动/握手/WAL/冷启动降级） | 3 周 ± | 10 万 LOC 全量索引 < 30s；单文件增量 < 200ms；探针往返（含管道）p95 < 150ms；对 jscpd 可检出集召回 ≥ 95%（属 docdup 域〔docstring/注释重复〕或阈值测度差异的条目可逐条证据归因排除，排除项入册——用户拍板 2026-08-07）**且**在同一真实仓库上精度 ≥ 90%（召回必配精度，B2）；property：增量 ≡ 全量重建 |
| **M3** 被动 guard v1 | 插件成型：PreToolUse 廉价门（预算+T1/T2 探针）、Stop 审计 v1（git diff 净 LOC + 新增重复块，**不含四分类**——A4）、SessionStart 引导+健康行、hook 输出 token 预算、pre-commit 模式、**最小 MCP server**（`check_duplication`/`scan`，对标 jscpd 已在位的位置——A8）；**收尾发 0.x 预览**（本地/私有 marketplace，自有真实项目 dogfood；部分会话跑**静默观察档**——只记录判定不注入不拦截，为 M4 积累未被 guard 塑形的 transcript；plugin.json 自此带显式 version） | 2–3 周 ± | 本地 marketplace 安装 → 测试仓库端到端拦截 T1 重复写入（transcript 为证）+ **500 次真实正常编辑重放误拦 ≤ 1 次**（N=1 演示不算数，B2）；hook 端到端 p95 < 1s 且分解表各项达标；会话累计 hook 延迟中位数 < 15s/百次编辑；0.x 预览在干净环境安装成功，dogfood 会话 ≥ 10（其中观察档 ≥ 5——D2-2） |
| **M4** 更新监督 + Haskell 判决层引入 | 四分类 fallback 阶梯 L0→L1→L2（L2 = Haskell 承重首战）；`churn`；契约内容随真实需求定稿为 1.0 | 3–4 周 ± | **预注册**评估集（实现前冻结、≥200 编辑样本、≥50% 来自真实 agent transcript，**样本纯净度（D2-1）**：只采观察档会话与 M3 前无 guard 历史会话，被 guard 干预过的编辑排除并报告排除比例——否则 FPR 被 guard 塑形向下偏，deny 准入门自证）；主门 = **FPR：500 次真实正常编辑误报 ≤ 1%**；recall 报告但不设作弊性 100% 门；moved 以 `git -M -C` 交叉 + 人工标注为 ground truth（difftastic 不识别 moved，不能当对照——A5）；L2 需证明对 L1 的增量收益，否则产品走 L1 |
| **M5** 深度去冗 + 图 | `graph`（独立子系统，验收对齐 ops-codegraph-tool 锚点）、`deadcode`、T3 冷路径、`docdup`（含代码注释/docstring 域）、三信号 join、`score`+棘轮、Haskell 语言支持（CoC 适配规范自定义并文档化） | 3–4 周 ± | T3 对 mizchi/similarity 可检出集召回 ≥ 90% 且精度 ≥ 85%；import 边 precision ≥ 90%（抽样人工核对 100 条，覆盖五门首发语言——D2-4）；本仓库自身跑通棘轮入 CI |
| **M6** GUI | Tauri：报告可视化、趋势、删除候选浏览 | 2–3 周 | 对 10 万 LOC 仓库**从冷启动 scan 到首屏** < 60s、已扫描报告打开 < 3s；三平台打包 |
| **M7** 发布 | marketplace 上架、签名/公证、Releases 自动化、完整 MCP、许可证合规（NOTICE/第三方 MIT 署名清单——D1-7）、文档 | 1–2 周 | 陌生机器一条命令可用；二进制 SHA256 校验链路端到端验证；**仓库转公开前全历史审计**（memory/、transcript、密钥、路径泄漏——D2-7）；文档过 `docdup` 自检；默认档位切换依据（各规则 FPR 数据）发布在 CHANGELOG |

**依赖**：M2←M1；M3←M2；M4←M3；M5←M4（churn 是三信号一腿，**串行**，v1.0 "并行"之说
作废——A4）；M6 可与 M5 并行；M7 收尾。总计粗估 4–6 个月。

## 7. 质量与测试策略

1. **跨语言契约测试**：contracts/ 同一批 golden fixtures，Rust 生成 IR、Haskell 判决，
   round-trip 断言；schema 变更必须 bump 版本（机制 M0 起，内容 M4 定稿）。
2. **交叉核对**：度量与 lizard / radon / rust-code-analysis / Sonar 例题对拍，
   fixtures 来自钉死 commit 的真实仓库随机抽样，分歧 case 显式收录。
3. **property-based**：Haskell 侧 Hedgehog（判决单调性、棘轮不可逆）；Rust 侧 proptest
   （索引增量 ≡ 全量重建）。
4. **评分敏感性测试**（B2）：扰动任一权重断言总分变化——直接针对 fuck-u-code 的
   死字段 bug 形态。
5. **Dogfooding**：M1 起 CI 对 cli/ 与 docs/ 强制 `ce scan --fail-under`；M5 起
   core/（Haskell 支持就位）+ 棘轮。本文件受行数棘轮（首部声明）与 `docdup` 约束。
6. **性能预算进 CI 基准**，回归即 fail。锚点如实标注：jscpd 3.44s/17K 文件是**批扫**
   数据，只锚定 M2 的全量索引预算；热路径预算不由它推出，由 M0 分解表实测建立（A6）。

## 8. 风险登记册

| # | 风险 | 缓解 |
|---|---|---|
| R1 | Haskell/Windows 工具链 | GHC 9.14 LTS 锁版 + M0 实测依赖可解；CI 必含 windows-latest；stdio 全程 binary mode；禁 DLL；未签名二进制的 Defender/EDR 误报风险 → M7 签名，之前 README 明示 |
| R2 | tree-sitter 语法 crate 漂移 | 锁 0.26.x；语法版本入 lockfile；升级走独立 PR + golden 全绿 |
| R3 | hook 延迟劣化 → 用户关插件 | daemon + 增量索引；分解表预算进 CI；超时 fail-open 降级为 warn，降级**必须可见**（doctor/健康行/Stop 汇总——A9f） |
| R4 | 误报 → 信任崩塌 | 分级 warn/ask/deny + 演进路线（§4.2）；deny 准入 = M4 FPR 门（≤1%）；豁免带 why；每判决附量化依据 |
| R5 | 竞品挤压（**触发器式**，A8） | 监测触发器：jscpd/desloppify 发布 diff 级 gating，或 Claude Code 内置类似能力 → 差异化收缩至三信号 join + 四分类，届时 M5 join 提前、热路径查重改评估复用竞品引擎 |
| R6 | 双语言成本先于价值支付（B1） | Haskell 承重首战后移到 M4（判决层），M0 只付骨架+握手的最小成本；契约内容不提前冻结；core 不依赖跟随 GHC 版本发布的库（stan 教训） |
| R7 | "处处 deny"招致反感 | 默认档位演进路线写死在 §4.2，不是永久 warn 也不是上来就 deny；排除模型 M1 内置 |
| R8 | 四分类（L2）不达标 | fallback 阶梯：退 L1（git+函数边界）仍可交付；deny 降级 ask 如实标注 |

## 9. 已拍板决策（2026-08-06 grill 问答，12 项，用户逐项确认）

License=**Apache-2.0**（LICENSE 已入库）· 前端=**Rust** 定案 · 仓库**私有开发、M7 公开** ·
guard 默认档位=**渐进路线**（§4.2）· M1 语言集=**TS/Python/Rust/Go/Markdown** ·
集成优先级=**Claude Code 优先** · GUI=**按 M6** · 发布节奏=**M3 后 0.x 预览** ·
Haskell 边界=**判决层**（ADR-002 确认）· docdup 域=**Markdown/纯文本 + 代码注释/docstring** ·
CLI=**`ce`**（codeeraser 作 alias，M0 撞名核查）· 幽默评语=**默认关闭彩蛋**（`--roast` 开启）。

---

*等待用户指令后从 M0 开始推进；任何"顺手先写点代码"的行为违反本计划。*
