# 算法轨设计册：算法进核（计划 v2.33，ADR-008 分工句修正）

> 本册是 v2.33 的落码权威（计划书横幅 v2.33 条、ADR-008 与 §6 T 轨 v2.33 行指向这里）。落码时以本册为准，改设计先改本册再改计划书再动代码。体例照 [authority-track.md](authority-track.md)（v2.32）与 [analysis-track.md](analysis-track.md)（v2.31）。

## 0. 一句话定位

凡是在测量侧已读到的事实——整数或文本——上做的计算都归核：Rust 碰世界（文件、文法、索引、git、进程）并把读到的交出来，Haskell 在这些事实上跑算法、下判决、出文档（文本可经本机管道过线，§3，2026-10-04 修正案）。本轨把今天仍在 Rust 里的这类算法按波次搬进核，每波落地后实读语言条，Haskell 字节数超过 Rust 即停；1.9.0 等这一刻再发。

## 1. 裁定与目标

1. **用户令**（2026-10-03，原话）：「要求改成：haskell占比比rust高。不固定比例阈值。」——取代 2026-09-25 的「Haskell ≥ 40 %」与 v2.32 步 9 的「口径类裁定」。
2. **两裁**（同日 AskUserQuestion）：① 路线 = 把算法搬进 Haskell，**不拆仓**（拆 GUI 子仓、把 Rust 挪出主仓一类的口径办法不做）；② **1.9.0 等目标达成再发**（v2.32 步 10 的发版排在本轨达标之后）。
3. **口径** = GitHub 语言条（主仓）：`.gitattributes` 里 `cli/tests/**`、`contracts/**` 为 linguist-vendored，`site/**` 为 linguist-documentation，`core/test` 计入。本地近似读法见 §10；发版声明以 `gh api repos/skymanbp/CodeEraser/languages` 的读数为准。
4. **达标式**：设 R / H 为某一提交上 Rust / Haskell 的计入字节，目标 = H > R。搬走的 Rust 净减 x、核净增 y（含电池与参考实现，`core/test` 计入），v2.32 步 6 的删除量 s（落地后实读）：H₀ + y > R₀ − x − s。以本册立项基线（§10 第二行）R₀ − H₀ = 936,192 B 计，y ≈ x 时要 x > (936,192 B − s) / 2。
5. **每波落地后实读一次，达标即停**：不预设要做完五波；未做的波在 §11 记去向。

## 2. 分工句

- **旧**（ADR-008 原文与硬约束 1）：判决在核；解析 / 索引 / 前端在 Rust。测量语义留 Rust；判据 = 需源文本或行级内容过 wire 即测量侧（§5.9.2）。ADR-008 验收段：「占比提升是副产品，禁止为占比搬迁或改写已有代码」。
- **新**（立项 2026-10-03，文本过线修正案 2026-10-04 定稿）：凡是在测量侧已读到的事实——整数或文本——上做的计算（算法、判定、定义、文档、文字）都在核；Rust 只做碰世界的事（文件系统、tree-sitter、SQLite、git、进程、管道、clap / MCP / daemon / 钩子接线），并把读到的东西交给核。只有 tree-sitter 才能做的词法（CST）留 Rust；只是一个字符串的函数的词法（拆说明符、词干、Markdown 掩码）是算法、可以搬。
- **两判例按新句改写**：细则第五期（墓碑）里对文本的切分与标记匹配是字符串上的算法、可以搬（W6，钩子路径上的成员受 §3 性能门约束）；细则第六期（同角色顾问）里 BM25 分子分母与 PPMI 计数已随 W3 进核，分词、词干与词袋随 W6 搬。
- §5.9.2 的一票否决（「需源文本或行级内容过 wire 即测量侧」）由 §3 的线规则取代：文本可以过本机管道，判据改为「碰不碰世界」；§5.9 的隐私保证一条不动（计划书 §5.9-6）。

## 3. 不变量

- **文本过本机管道，且只过这一处**（2026-10-04 修正案，取代「整数过线」；§11 第 39–42 条）：
  1. *可以过线的*（请求侧，ce → ce-core）：测量侧从树上读到的任何文本——仓库相对路径、文件名与目录名、import / include / require 说明符的文本、配置文件的值（模块路径、包名、声明的根、编译数据库的旗标）、单元键与符号名、源文本片段（行、注释、散文段）——前提是读它的算法住在核里。文本是 UTF-8 的 JSON 字符串；不是合法 UTF-8 的字节序列按测量侧今天已有的读法送（有损，与 Rust 算法原来的读法相同），搬那个算法的车道写明这一点。
  2. *留在 Rust 的*（碰世界）：读文件、tree-sitter 解析、SQLite、git、进程、管道、clap / MCP / daemon / 钩子接线。Rust 不必再把文本降成整数再问，读到什么送什么。只有 tree-sitter 能做的词法留下（CST）；只是一个字符串的函数的词法（拆说明符、词干、Markdown 掩码）是算法，可以搬。
  3. *核可以答的*：也可以是文本——解析出的路径、渲染好的报告行、名字——所以 v2.32 留在 Rust 的绑定 / 符号表胶水（`symbols` 计数、码 → 字符串表）在核能自己拼出字符串的地方可以退役。
  4. *不动的隐私保证*（计划书 §5.9-6，在此重申，免得把放宽读得比它宽）：线是同一台机器上 `ce` 与它的子进程 `ce-core` 之间的 stdio 管道，`ce-core` 不开文件、不开套接字、不联网（§5.9-1 不变）；核不持久化、不记日志，请求里的文本只活一个请求；§5.9-2 的索引隐私不变（`.ce/index.db` 仍只存哈希、span 与符号名，不存源文本；提及表仍只存 fnv1a64）；observe feed、trend 存储、基线、SARIF 与每份报告的字段与今天完全相同（报告本就印路径与单元键——今天由 Rust 渲染，明天可能由核渲染；这条规则不让任何**新**文本到达任何持久化或用户可见的面）；`contracts/` 下的 golden 与夹具只带合成文本（既有夹具树），不抄第三方仓库的文本；secrets 排除（`.env*`、`*.pem`……）在读之前由走查施行，被排除文件的文本同样到不了管道。
  5. *性能门*：文本比 id 大。每条搬文本的车道量请求字节与它碰到的面的 ABAB；PreToolUse 钩子路径的预算（PERF-BUDGET）不变，钩子路径上的搬文本改动只有守住预算才落地。
  6. *wire 版本*：每个开始带文本的族在自己的 proto 步里带（旧的整数键能在旁边多留一个 minor 时为加性，否则为断代），版本号由落码车道定，不在此定。
- **三面等价**（CLI / MCP / GUI）。
- **判决字节不动**：每族切换以新旧二进制十语料逐字节同为门（§9）。
- **查重预算只降不升**；**E01**（函数 ≤ 50 / 75 行、CoC ≤ 15、文件 ≤ 300 / 750 行）；**核 `.hs` ≤ 290 行**（子仓 `core_size_gate`）。
- **无核行为**：这些面在 v2.32 之后本就需要核；新搬的族沿用「无核 → 具名拒绝退 2 / 钩子具名降级」，不留 Rust 兜底副本。

## 4. 盘点

只读盘点（Explore 代理 2026-10-03，在 e35fde53 上；`cli/src` 382 个 `.rs` 文件 2,323.7 KiB，与本册基线 40ab1a4f 的 `.rs` / `.hs` 无差——`git diff --stat e35fde53 40ab1a4f -- '*.rs' '*.hs'` 为空）：

| 类 | 含义 | 文件 | KiB |
|---|---|---|---|
| A | 纯算法（内存数据、无 I/O） | 87 | 484.6（整数输入 ≈ 107，文本 / 路径输入 ≈ 378） |
| B | 算法与 tree-sitter / SQL 交织，先降成行流才能搬 | 85 | 581.9 |
| C | 必须留 Rust | 122 | 785.1 |
| D | 既有家族的线缆胶水 | 88 | 472.2 |

同日量的本仓 `.ce/index.db` 行数（设计草案所记）：files 1,080、fingerprints 56,139、symbols 15,008、unitsig 14,202、sites 7,233、edges 5,341、docsegs 2,927、bag postings 272,466、df 6,535——这是 W2–W4 新请求在自仓上的量级。

## 5. 镜像清单（核已有、Rust 仍留一份）

核的结果一被采用，Rust 那份即删（W1 主体）：

| Rust 一侧 | 核已有的判定 |
|---|---|
| `cli/src/scan/report.rs::evaluate` | scan 分级 |
| `cli/src/scan/metrics/naming.rs::conforms` | 命名合规 |
| `cli/src/dedup/t3/mod.rs::is_clone` | T3 克隆判定 |
| `cli/src/dedup/candidates.rs::verdict` + `cli/src/dedup/struct_fp.rs::label_intersection` | 候选判决 |
| `cli/src/docdup/judge/mod.rs::is_dup` 与 `cli/src/docdup/judge/wire.rs` 的 Jaccard | docdup 判定 |
| `cli/src/dedup/pairs.rs` 的 `minDistinct` | 记号种类地板 |
| `cli/src/similar/bm25.rs::role` | 同角色合取 |
| `cli/src/merge/groups_trim.rs::isomorphic` 与 `cli/src/merge/groups.rs` 的族码 | 合并组判定 |
| `cli/src/guard/zone.rs` 的档位 | 分级区档位（热路径，需缓存值；不在 W1） |
| `cli/src/score/knobs.rs::core_defaults` | 旋钮默认值 |
| `cli/src/query/legend.rs`、`cli/src/query/columns.rs` | 查询的图例与列 |
| `*/wire.rs` 的 cap 常量（草案盘点记七个文件；立项时 `grep -l -E "const [A-Z_]*CAP"` 在 `cli/src` 的 `wire.rs` 上数到十一个：`arch` / `flow` / `merge` / `query` / `scan` / `score` / `similar` / `structure` / `tombstone` / `dedup/t3` / `docdup/judge`，哪些是核 cap 的镜像落码时逐个核对） | 各族契约的 cap |

## 6. 波次

每波 = 一条或多条并行车道；每族落地后重读语言条（§10），达标即停。路径缩写（本册定义）：**H** = PreToolUse 钩子热路径；**S** = Stop / precommit 审计与命令批扫；**K** = 索引刷新（daemon 与命令共用）；**cold** = 只在命令里跑的冷路径。

| 波 | 内容（Rust 侧现量 KiB） | wire 形 | 路径 | 风险 |
|---|---|---|---|---|
| W1 镜像退役 + 冷路径小件 | §5 镜像（除 zone）；`query/` 的 lexer / program / columns / legend（27）；`structure/` 的 tree / rows / edges + `arch/tables.rs`（25）；`config/` 的故障检查与 `canonical.rs`（16，钩子留本地 fail-open 读法）；`update/version.rs` / `update/manifest.rs` 的比较 | 既有请求不动或加性；cap 与默认值改读 `tables/1` / hello | cold / K | 低 |
| W2a 引用阶梯（四语言，已落地） | Python / Lua / Go / C·C++ 的查找（下方「W2 状态」） | `resolve/1`（8.1.0）：路径 = 段 id 序列、站点 `[kind, from, 段…]`、文件集、逐语言配置事实；应答每站点 `[rung, target \| refusal]` | K（增量刷新） | 已落地 |
| W2a′ 简化 `resolve/1`（W2-text 阶段 A，车道 `lane/v233-w2-text`） | 路径、说明符与 Python / Go / C·C++ 的配置原文改以文本过线；删段驻留表、`vocab`、词缀行与 `cli/src/graph/resolve/` 的降；go.mod、`pyproject.toml` 的键、编译数据库的命令行拆分 / 旗标链 / 响应文件展开搬进核（下方「W2 状态」） | `resolve/1` 9.0.0（major：退役 `segs` / `vocab` / `affixes` / `dirs`） | K | 中：差分门与冻结预言机（§11 第 26 条）原样保留、全绿；配置读法另立冻结预言机与差分（§11 第 47 条） |
| W2b 其余七个阶梯 | ts / hs / java / r / rs / md / html 的阶梯（`graph/ladder/` 余下文件）及其配置面逻辑（`graph/cabal_parse.rs` / `jsonc.rs` / `roots_ts.rs` 一类；`cmdline.rs` / `compdb_flags.rs` / `gomod.rs` 已随 W2a′ 进核，r / java / hs 与 `cabal.rs` / `cabal_parse.rs` 已随阶段 B / C / D 进核；量由车道实量） | `resolve/1`：路径、说明符、配置值以文本过线 | K | 中：十三语言精度册重生成（判决字节须同） |
| W3 候选与排序 | T3 / docdup 候选生成（`dedup/sources.rs` / `dedup/minhash.rs` / `dedup/candidates.rs` / `docdup/judge/candidates/runs.rs`，32）；`similar/bm25.rs` / `similar/ppmi.rs`（17；`stem.rs` / `terms.rs` 不在本波，W6 搬）；fourclass L1（`diff.rs` / `model.rs` / `decls.rs` / `stacking.rs` / `anchor.rs`，21）；score / join / merge 的装配 join（30） | 既有族加性表：单元签名行、shingle 集、查询词的 postings 切片、行哈希对 | K / S | 中：超线性段要 ABAB，核端用数组 / IntMap |
| W4 CST 行导出 + 其上的算法 | Rust 一个通用导出器（仿 `scan/metrics/events.rs`）：每单元前序行 `[kind, field, parent, named, line, identHash]`；其上搬 `scan/calls.rs` / `scan/callees.rs`（24）、fourclass 的 `declared.rs` 与 `visibility/` 各语言（≈ 54）、mention 的 `conv/` / `selfref.rs` / `candidates.rs`（≈ 45） | 新族 `cst/1`（或各族加性 `rows`） | K（刷新） | 中 |
| W5 flow 降表 | `flow/` 的 17 个降表文件（≈ 114）改在核里从 CST 行算 CFG 事实（表本就在核） | `flow/1` 请求改送 CST 行 | **H**（guard / flow 两侧降表）与 S | 高：钩子路径，只送改动单元；探针预算见 §8 |
| W6 文本规则 | 墓碑文本管线（`cli/src/tombstone/`）、提及分词（`mention/token.rs`）、Markdown 掩码 / slug（`graph/md_mask.rs`、`ladder/md_slug.rs`）、similar 的词干 / 词 / 词袋（`similar/stem.rs` / `terms.rs` / `bag.rs`）、docdup 的 shingle（`docdup/shingle.rs`）；量由车道实量 | 各族带文本的 proto 步：源文本片段、名字以文本过线 | **H**（墓碑、guard 用到的分词）与 K / S | 高：钩子路径上的成员受 §3 第 5 点性能门约束，守不住预算不落地 |
| W7 报告渲染胶水 | v2.32 因字符串不能过线而留在 Rust 的绑定与符号表（`symbols` 计数、码 → 字符串表）；量由车道实量 | 应答带文本（路径、名字、渲染好的行），报告字段不变 | cold / S | 中：报告字节须同 |

- **W2 状态**（2026-10-03）：W2a 落码在车道 `lane/v233-w2`（未落地）——新族 `resolve/1`（proto 8.1.0，步 6 的 8.0.0 之上的加性 minor）接走 Python / Lua / Go / C·C++ 四个阶梯的查找：核 `core/app/CE/Resolve.hs` + `Resolve/` 十一个模块，Rust `cli/src/graph/resolve/`（驻留表、逐语言降、请求与应答）+ `graph/owed.rs`（无核记账）；`ladder/py.rs` / `lua.rs` / `go.rs` / `c.rs` / `c_search.rs` / `c_index.rs` 删除。留在 Rust 的降：站点检测、`c_head.rs` 读 include 列表、`lua_path.rs` 抽 `package.path` 模板、`gomod.rs` 读 go.mod、`compdb*.rs` / `cmdline.rs` 读编译数据库并拆旗标、`[graph.search_roots]` 的配置读法；`ladder/paths.rs` 留给仍在 Rust 的 R / Haskell 阶梯（§11 第 16–23 条）。其余语言（ts / hs / java / r / rs / md / html）仍走 Rust 阶梯，W2b 接；2026-10-04 修正案之后先做 W2a′（把 W2a 的段 id 换成字符串、删掉那层降），W2b 直接按文本过线写（§11 第 42 条）。W2a′ 落码在车道 `lane/v233-w2-text`（2026-10-04，未落地）：`resolve/1` 升 9.0.0、全部以文本过线；Rust 删 `graph/resolve/{intern,tokens,lower,facts}.rs` 与 `graph/{cmdline,compdb,compdb_flags,gomod}.rs`、`roots.rs` 的 pyproject 读法，新 `graph/resolve/request.rs` 只把读到的东西装进请求；核新加 `CE.Resolve.{Str,Chars,Cmdline,Flags,CompDb,Inspect,Tables}`，`Go` / `Py` / `CIndex` 改读原文，`CE.Resolve.Vocab` 退役。留在 Rust 的：TOML / JSON 解码、读文件与探库、`lua_path.rs`、`c_head.rs`（§11 第 44 条）。 W2b 阶段 B（R，同车道，2026-10-04，未落地）：`ladder/r/` 两个文件与 `deadcode/targets.rs` 的包代码展开删除，R 阶梯、`DESCRIPTION` 读法与包代码展开进核（`CE.Resolve.R` / `CE.Resolve.Description`）；请求加性 `r.descriptions`、应答加性 `packages`，9.0.0 内加性；`ladder/paths.rs` 只剩 Haskell 阶梯用的 `one_of`（§11 第 50–53 条）。 W2b 阶段 C（Java，同车道，2026-10-05，未落地）：`ladder/java.rs` / `java_inherit.rs` / `java_pick.rs` / `java_sets.rs` 与 `java_header.rs` 的注解读法删除，Java 的四级阶梯进核（`CE.Resolve.Java` / `JavaAt` / `JavaInherit` / `JavaPick` / `JavaName` / `JavaHeader`）；请求加性 `java.headers`、Java 站点带行号，9.0.0 内加性；头部与类型的词法（`java_header.rs` 的 `read`、`java_types.rs`）留在 Rust（§11 第 54–57 条）。 W2b 阶段 D（Haskell，同车道，2026-10-05，未落地）：`ladder/hs.rs`、`graph/cabal.rs` 的读法、整份 `graph/cabal_parse.rs`、`ladder/paths.rs`（只剩 Haskell 用的 `one_of`）、`roots.rs` 的 `beside` 与 `deadcode/targets.rs` 的 `main_targets` 删除，Haskell 阶梯、`.cabal` 读法、可执行与测试入口、包内私有的判定进核（`CE.Resolve.Hs` / `Cabal` / `CabalWalk`）；请求加性 `hs.cabals` / `hs.owners`、应答加性 `mains` / `private`，9.0.0 内加性；找最近的 `.cabal`（`graph/cabal_find.rs`）留在 Rust（§11 第 58–62 条）。
- **proto**：每个新族 / 加性表随自己的 minor（或在 v2.32 步 6 的 8.0.0 之后的 minor），版本号落地时按 `contracts/VERSIONING.md` 的顺序定；golden 由子仓 `fixture_contract::regen` 生成。
- **粗算**（设计草案：净减按现量 70 %、核增量按搬走量 1.0× 含电池，单位 KB）：W1 ≈ −50 / +45，W2 ≈ −145 / +190，W3 ≈ −70 / +95，W4 ≈ −85 / +115，W5 ≈ −80 / +105 → Rust ≈ 2,406 − 430 − s，Haskell ≈ 1,468 + 550 = 2,018；s ≥ 0 时 Rust ≤ 1,976。五波全做才过线、余量不大——每波落地实读，偏差当波修正。
- **车道**：W1 / W2 / W3 三条车道并行（目录不相交：query · structure · config / graph / dedup · docdup · similar · fourclass），W4 的导出器同时起，W5 等 W4。落地按波序，一族的落地提交才带它的 proto / golden / 事实。修正案之后的次序（§11 第 42 条）：W2a′ 先于 W2b（同在 `graph/`）；W6、W7 与 W1 / W4 目录不相交、可并行；粗算一行不为 W2a′ / W2b / W6 / W7 估字节，量由各车道实读（§10）。

## 7. 不搬的与理由

2026-10-04 修正案后逐行重读（§11 第 42 条）：理由只是「要读文本」的行不再成立，理由是钩子热路径的行带着那条理由留下。

- **墓碑文本管线**（`cli/src/tombstone/`）：「对源文本切分与匹配是降」这条理由已不成立，列入 W6；留下的理由只剩它在 H 路径上——守住 §3 第 5 点的预算才搬，守不住就留在这里并记账。其上的整数判定早已在核（`tombstone/1`）。
- **dedup 的 winnow / pairs**（`cli/src/dedup/winnow.rs`、`cli/src/dedup/pairs.rs` 的配对）：daemon 探针在 H 路径上仍要 Rust 一份，搬了是复制不是减少（`minDistinct` 的判定按 §5 退役，配对本身留下）。理由是热路径，不变。
- **tree-sitter 的词法**（CST）：只有 tree-sitter 能做，留 Rust（W4 把它的输出作行导出）。纯字符串的分词 / 词干 / 掩码原记为「文本侧的降」，此理由不再成立，列入 W6。
- **C 类全部**（§4）：碰世界的事。
- **GUI**：前端接线。

W5 之后仍未达标，再议余下各项与 GUI（§11 第 9 条）。

## 8. 性能与钩子

- 每族切换提交带 release 同坐 ABAB ×7（旧 / 新二进制、自仓副本），读数进 `docs/PERF-BUDGET.md`；`ce check` 暖跑不得慢过 +15 %，慢了先用中继核录真请求单测核（核端的二次方行走是已见过的成因）。
- 搬文本的车道（W2a′ / W2b / W6 / W7）另量请求字节（§3 第 5 点），与 ABAB 一同进 PERF-BUDGET。
- 钩子探针（PreToolUse）：W1–W4 与 W7 不碰探针路径；W6 的钩子路径成员（墓碑、guard 用到的分词）与 W5 同一预算。W5 的 guard / flow 经 daemon 持有的核链，预算 = 探针 p50 增量 ≤ 10 ms（bench `hook_probe` 同坐对比）；过不了就让 guard 留 Rust 降表一份、其余面走核，并如实记账。

## 9. 门（每族）

- 十语料 + 自仓，新旧二进制逐字节同（判决、文档、控制台三形）。
- 核电池 = 性质腿 + 200 例参考对拍（参考实现换一种写法，查重门不把它当克隆）。
- golden 由子仓 `fixture_contract::regen` 生成；精度册受影响即两提交重生成（退役 → 在干净树上重生成）。
- 两仓 dogfood 七腿 × 两种点核方式（`CE_CORE_BIN` 与 `--core` 旗标）。
- ADR-006 两仓具名重立；引文重签；语言条实读一行进 CHANGELOG 与 §10。

## 10. 读数

量法（本地近似 GitHub 口径）：`git ls-tree -r -l <rev>` 的 blob 字节，只计 `.rs` / `.hs`，去掉 `cli/tests/`、`contracts/`、`site/` 前缀下的路径；blob 字节即仓库里提交的字节（不再折 CRLF）。

| 提交 | Rust 计入（B） | Haskell 计入（B） | H / (R + H) | 差 R − H（B） | 来源 |
|---|---|---|---|---|---|
| e35fde53 | ≈ 2,406 KB（`cli/src` 2,379,503 + `gui/src-tauri` 18 KB + `scripts/tsprobe` 10 KB） | 1,467,930（`core/app` 957,403 + `core/test` 510,527） | — | ≈ 938 KB | 设计草案（主会话 2026-10-03） |
| 40ab1a4f（立项基线） | 2,405,719（`cli/src` 2,379,503 + `gui/src-tauri` 18,088 + `scripts/tsprobe` 8,128） | 1,469,527（`core/app` 957,403 + `core/test` 510,527 + `scripts/tsprobe` 1,597） | 37.92 % | 936,192 | 本册立项提交，上述量法 |
| v2.32 步 6 并入立项提交后（车道 `lane/v232-step6` 的终点） | 2,400,357（`cli/src` 2,374,748 + `gui/src-tauri` 17,481 + `scripts/tsprobe` 8,128） | 1,468,877（`core/app` 956,942 + `core/test` 510,338 + `scripts/tsprobe` 1,597） | 37.96 % | 931,480 | 权威轨步 6（Rust 无读者项退役、8.0.0 退役 `judgedMask` 与 `patterns`），上述量法 |
| W2a（车道 `lane/v233-w2`，变基到 093aede4 之后，四个阶梯的查找进核） | 2,395,609（`cli/src` 2,370,000 + `gui/src-tauri` 17,481 + `scripts/tsprobe` 8,128） | 1,554,411（`core/app` 1,017,058 + `core/test` 535,756 + `scripts/tsprobe` 1,597） | 39.35 % | 841,198 | 本车道终树，上述量法；对 093aede4（上一行）Rust −4,748、Haskell +85,534（`core/app` +60,116、`core/test` +25,418） |
| W3（车道 `lane/v233-w3`，变基到 a0198c2f 之后，四组：候选 / 排序 / 候选生成 / fourclass L1） | 2,393,396 | 1,706,269 | 41.62 % | 687,127 | 本车道终树，上述量法；对 a0198c2f（上一行）Rust 删 66,548 B、胶水加 64,335 B（净 −2,213），核 app 净 +90,095、test 净 +63,340（§11 第 37 条） |
| 68eda600（main，W3 落地之后；文本过线修正案的基点） | 2,393,396（`cli/src` 2,367,787 + `gui/src-tauri` 17,481 + `scripts/tsprobe` 8,128） | 1,706,269（`core/app` 1,103,850 + `core/test` 600,822 + `scripts/tsprobe` 1,597） | 41.62 % | 687,127 | `v233_s0_scratch/langbar.py HEAD` 在本提交上的读数（2026-10-04）；与上一行同值（W3 车道终树即落地树） |
| W2a′（车道 `lane/v233-w2-text`，W2-text 阶段 A：`resolve/1` 以文本过线，Python / Go / C·C++ 的配置读法进核） | 2,355,295（`cli/src` 2,329,686 + `gui/src-tauri` 17,481 + `scripts/tsprobe` 8,128） | 1,738,863（`core/app` 1,138,585 + `core/test` 598,681 + `scripts/tsprobe` 1,597） | 42.47 % | 616,432 | 本车道终树，上述量法；对 92e728b1（上一行之后的修正案提交，`.rs` / `.hs` 与上一行同）Rust 删 46,245 B、加 8,144 B（净 −38,101），核 `core/app` 净 +34,735、`core/test` 净 −2,141 |
| W2b 阶段 B（同车道：R 阶梯、`DESCRIPTION` 读法与包代码展开进核） | 2,349,478（`cli/src` 2,323,869 + `gui/src-tauri` 17,481 + `scripts/tsprobe` 8,128） | 1,750,948（`core/app` 1,146,908 + `core/test` 602,443 + `scripts/tsprobe` 1,597） | 42.70 % | 598,530 | 本车道阶段 B 终树，上述量法；对 c96ab3f6（阶段 A 落地）Rust 删 7,470 B、加 1,653 B（净 −5,817），核 `core/app` 净 +8,323、`core/test` 净 +3,762 |
| W2b 阶段 C（同车道：Java 阶梯与类型注解的读法进核） | 2,327,680（`cli/src` 2,302,071 + `gui/src-tauri` 17,481 + `scripts/tsprobe` 8,128） | 1,796,836（`core/app` 1,173,679 + `core/test` 621,560 + `scripts/tsprobe` 1,597） | 43.56 % | 530,844 | 本车道阶段 C 终树，上述量法；对 27d0d56d（阶段 B 落地）Rust 删 23,528 B、加 1,730 B（净 −21,798），核 `core/app` 净 +26,712、`core/test` 净 +19,143 |
| W2b 阶段 D（同车道：Haskell 阶梯与 `.cabal` 读法进核） | 2,302,184（`cli/src` 2,276,575 + `gui/src-tauri` 17,481 + `scripts/tsprobe` 8,128） | 1,831,721（`core/app` 1,194,956 + `core/test` 635,168 + `scripts/tsprobe` 1,597） | 44.31 % | 470,463 | 本车道阶段 D 终树，上述量法；对 fa83a48d（阶段 C 落地）Rust 删 29,475 B、加 3,979 B（净 −25,496），核 `core/app` 净 +21,277、`core/test` 净 +13,608 |

注：草案把 `scripts/tsprobe` 的两种扩展名合记在 Rust 名下（8,128 + 1,597 ≈ 10 KB）；按扩展名分开后 Haskell 多 1,597 B。e35fde53 与 40ab1a4f 之间 `.rs` / `.hs` 无改动，两行是同一份字节的两种记法。

注（W2a）：Rust 只净减 4,748 B——删掉的四个阶梯的六个文件 37,908 B（`ladder/py.rs` / `lua.rs` / `go.rs` / `c.rs` / `c_search.rs` / `c_index.rs`），换来的降与接线 33,160 B（`graph/resolve/` 五个文件 24,404、`graph/owed.rs` 3,428、其余十三个文件的改动 5,328——`ladder/outcome.rs` 是从 `ladder/mod.rs` 搬出的原码、与它合计——其中差分门的 `#[cfg(test)]` 挂载与说明 279 B）。查找本身在 Rust 里很紧凑，而把路径、站点与配置降成段 id 的那一层与它同量级；W2 行的「≈ −145」按这个比例看要大幅下调（§11 第 24 条）。

注（W2a′）：Rust 净减 38,101 B——删掉 W2a 留下的整层降与三份配置读法：`graph/resolve/lower.rs` 10,058、`compdb_flags.rs` 9,611、`compdb.rs` 7,595、`cmdline.rs` 6,555、`resolve/facts.rs` 4,411、`resolve/tokens.rs` 3,073、`gomod.rs` 2,057、`resolve/intern.rs` 1,321（八个文件 44,681 B），`roots.rs` 的 pyproject 读法 −1,547、`tables/pack.rs` −17；加回的是送文本的那一层：新 `graph/resolve/request.rs` 5,222（读出原文并装进请求）、`resolve/mod.rs` +1,461（`wanted` 回路与 `responses`）、`compdb_find.rs` +853（响应文件名改问核、`root_text`）、`graph/mod.rs` +498（冻结读法的 `#[cfg(test)]` 挂载与说明）、`corelink.rs` +110（proto 9.0.0）。与 W2a（删 37,908、补 33,160）对照：线上能带文本之后，降胶水从「与被搬走的查找同量级」缩到被删字节的约六分之一（§11 第 39 条说的是只许整数过线时的情形）。核这边 `core/app` 新增 `Str` / `Chars` / `Cmdline` / `Flags` / `CompDb` / `Inspect` / `Tables` 七个模块（44,960 B），`Vocab` 退役、`Contract` / `World` / `Request` / `Cost` 随整数降一起变小；`core/test` 的 `ReferenceResolveGen` 不再自己降（−3,208）。

注（阶段 B）：Rust 净减 5,817 B——删 `ladder/r/description.rs` 3,233、`ladder/r/mod.rs` 2,547、`ladder/paths.rs` 的 `beside_or_root` / `declared` 1,148、`deadcode/targets.rs` 的 `package_code` 538、`ladder/mod.rs` 的分派臂 4（共 7,470 B）；加回的是 `resolve/mod.rs` 的 `packages`（+827，声明目标那一侧单发的请求）、`resolve/request.rs` 按文件名分送 go.mod 与 `DESCRIPTION`（+760）、`deadcode.rs` 把可失败的 `gather` 接上（+38）与两处注释（+28）。核 `core/app` 新增 `R` 2,546 B、`Description` 3,474 B，`World` 收进 `besideOrRoot` / `declaredIn` / `ofLangs`（+1,269，Lua 的 `loaded` 与 C 的 `declared` 改读它们，`Lua` −131、`CIndex` −503）；`core/test` 的参考电池加 R（+3,762）。

注（阶段 C）：Rust 净减 21,798 B——删 `ladder/java.rs` 12,190、`java_inherit.rs` 5,308、`java_pick.rs` 3,460、`java_sets.rs` 2,352、`java_header.rs` 的 `past_annotation` 与两处说明 218（共 23,528 B）；加回的是 `resolve/request.rs` 把每个 Java 文件头装进请求、Java 站点带行号（+1,463），`ladder/mod.rs` / `resolve/mod.rs` / `graph/mod.rs` / `outcome.rs` / `owed.rs` 的分派与说明（+267）。核 `core/app` 新增 `JavaPick` 5,720、`Java` 5,536、`JavaInherit` 5,048、`JavaName` 3,565、`JavaAt` 3,042、`JavaHeader` 1,596 B，`Request` / `Contract` / `Tables` / `Resolve` / `Cost` 带 Java 的行（+2,205）；`core/test` 新增 Java 的参考电池 `ReferenceJava` 11,350、`ReferenceJavaGen` 7,653 B，`Spec` / `ReferenceResolve` / `ResolveProps` +140。头部与类型的词法留在 Rust：走查每跑都读每个 Java 文件（类型扫描读全文），搬过去就要每跑把每个 Java 文件的全文送核。

注（阶段 D）：Rust 净减 25,496 B——删 `graph/cabal_parse.rs` 10,583、`ladder/hs.rs` 9,311、`graph/cabal.rs` 7,515、`ladder/paths.rs` 902、`deadcode/targets.rs` 的 `main_targets` 727、`roots.rs` 的 `beside` 403、`ladder/mod.rs` 的分派 34（共 29,475 B）；加回的是 `graph/cabal_find.rs` 1,500（`nearest` / `cabal_in` 自 `cabal.rs` 原样搬出）、`resolve/mod.rs` 的 `declared` / `private` +1,498、`resolve/request.rs` 送 `.cabal` 原文 +534、`mounts.rs` 把 Haskell 文件的最近 `.cabal` 组成 `owners` +375、`graph/mod.rs` / `owed.rs` 的挂载与说明 +72。核 `core/app` 新增 `CabalWalk` 8,792、`Hs` 6,008、`Cabal` 3,779 B，`Resolve` / `Contract` / `Inspect` / `Tables` / `Request` / `Cost` 带 Haskell 的行（+2,698）；`core/test` 新增 Haskell 的参考电池 `ReferenceHs` 7,184、`ReferenceHsGen` 6,356 B，`Spec` / `ResolveProps` +68。

## 11. 拍板记录

1. **用户改令**（2026-10-03）：「要求改成：haskell占比比rust高。不固定比例阈值。」——目标从 Haskell ≥ 40 % 改为 Haskell 字节数 > Rust 字节数，不设比例线（§1）。
2. **路线**（同日 AskUserQuestion）：把算法搬进 Haskell，不拆仓（§1 第 2 条）。
3. **发版**（同日 AskUserQuestion）：1.9.0 等目标达成再发；v2.32 步 10 排在本轨达标之后。
4. **口径**：GitHub 语言条（主仓），`.gitattributes` 三条照旧；本地按 §10 近似读，发版以 `gh api` 读数为准。
5. **分工句修正**（ADR-008，立项提交）：§2 的新句取代旧句；细则第五 / 六期两判例按新句改写；验收段「禁止为占比搬迁或改写已有代码」改写为「按新分工搬迁是分工的兑现，每次搬迁仍过本族的逐字节门」。
6. **文本侧的降留 Rust**：切分 / 词法 / 分词 / stemmer 是降，过线的是哈希与驻留 id——2026-10-04 由第 41 条取代（§3）。
7. **不搬三项与 C 类**（§7）：墓碑文本管线、dedup 的 winnow / pairs、分词 / 词法 / stemmer；C 类与 GUI——2026-10-04 第 42 条把墓碑文本管线与纯字符串的分词 / 词干移入 W6，余下照旧。
8. **每波实读、达标即停**（§1 第 5 条）；偏差当波修正。
9. **W5 之后仍未达标**：再议 §7 三项与 GUI，经 AskUserQuestion 上呈。
10. **性能门**：`ce check` 暖跑 +15 % 为线；钩子探针 p50 增量 ≤ 10 ms，过不了 guard 留 Rust 降表一份并记账（§8）。
11. **无核行为**：沿用 v2.32 的具名拒绝 / 钩子具名降级，不留 Rust 兜底副本（§3）。
12. **车道与次序**：W1 / W2 / W3 并行（目录不相交），W4 导出器同时起，W5 等 W4（§6）。
13. **v2.32 步 9 取代**：权威轨步 9「口径类裁定」由本轨取代；权威轨步 10（发版 1.9.0）等本轨达标。
14. **立项提交零代码改动**：本册 + 计划书 v2.33（横幅、ADR-008、§6 T 轨）+ 权威轨册两处指向 + CHANGELOG 一块；项目 CLAUDE.md（被 gitignore）的硬约束 1 与状态行由主会话在主根改。
15. **权威轨步 6 的删减量只有约 5 KB**（2026-10-03）：步 6 删掉的 Rust 是 5,362 B（`cli/src` −4,755、`gui/src-tauri` −607），不是权威轨册 §5.4 估的约 200 KB——旧面、控制台与 `print_*` 已在步 3–5 各族切换时删掉，步 6 只剩盘点出的余项（权威轨册 §13 第 69–75 条）。差 R − H 由 936,192 降到 931,480（§10），缺口仍要靠本轨的波次补。
16. **W2a 词表一维对齐**：`vocab` 是一列段 id，与定义包 `resolve.words ++ resolve.affixes` 逐项对齐（任务书草案写成 `[[vocabCode, segId]]` 对；一列已带同样的映射，次序由包定）。
17. **W2a 文件行带 `walked` 位**：站点所在的文件若不在走查集（考题仪器的例子），照样追加一行、`walked` = 0——它的目录可读，但它永远不是目标、不进 Go 的目录集、不进 C 的覆盖集。
18. **W2a 无核记账**（任务书 §3）：核答不了时（无核 / 无 `resolve/1` / 降级）四个语言的站点存为未解析、不出边行，其余语言照常解析；`graph/owed.rs` 把这些文件记进 `resolve_pending`（下一次有核的刷新只重解这些）、meta `resolve_degraded` 记文件数；读边的面（`graph::load::graph_rows` / `unresolved_paths`，即 graph / deadcode / structure / check / join / arch / merge / query / erase）在该行存在时按名拒 `resolve_unavailable: … — <原因>`；干净的一遍删掉两者。初稿让整次扫描拒答，会让索引刷新（daemon 冷启分析与 Stop 审计的 similar 腿都要它）一起停，改为记账。
19. **W2a 钩子路径不变**：PreToolUse 探针（daemon `probe::probe`）从不调阶梯；冷启分析在后台——没有钩子路径因此新需要核（任务书的 STOP 条件未触发）。
20. **W2a 编译数据库链在收集时驻留并按结构去重**（旗标链在内），一个请求里同形的链只过线一次。
21. **W2a 主仓 `graph/` 之外的改动**：`core/app/CE/Protocol.hs`（族行）、`CE/Tables.hs`（包键）、`CE/Protocol/Version.hs` 与 `cli/src/corelink.rs`（proto 8.1.0 的两处拼写）、`core/ce-core.cabal`（模块）、`cli/src/tables/pack.rs`（读 `resolve` 键）、`cli/src/dedup/{mod,index}.rs`（索引的解析回调改成整批签名）；`core/test/DocumentProps.hs` 的目录腿多删一个包键并断言它等于 `CE.Resolve.Vocab.table`。
22. **W2a 精度册出处**：子仓 `exams.rs` 的 C 与新加的 Lua 阶梯清单列入 `cli/src/graph/resolve/` 与 `core/app/CE/Resolve(.hs|/)`——查找住在那里，精度册的出处门要跟着它；`ladder/mod.rs` 本就在出处清单里且已改，全部已评分的精度册按两提交退役再重生成。
23. **W2a 参考实现换一种写法**：`ReferenceResolve` 按路径字符串做（切分与拼接文本、集合成员、字符串比较 External 表），`ReferenceResolveGen` 用自己的降（按排序后的字符串编号）；正品读段 id、词缀表与目录树。两者在 200 例 1,620 站点上逐站点同答，另一腿要求这些例子走到四个阶梯的每一级与每种拒答（探针：改坏参考的一处，等价腿即红）。
24. **W2a 的读数**：语言条 37.96 % → 39.35 %，差 931,480 → 841,198 B（对 093aede4）；Rust 只净减 4,748 B（§10 注）——整数过线要求 Rust 侧留一层降（驻留表、逐语言降、配置事实），它与被搬走的查找几乎一样大，所以一个阶梯的搬迁对 Rust 的净减远小于 §6 粗算的「净减按现量 70 %」。
25. **W2a 家族计数**：`count:families#word`（README / 官网的「十六个判决家族」）不计 `resolve/1`，与 `hello` / `tables/1` / `document/1` 同理——它答一个站点通向哪里，判决仍是图族的事（子仓 `it/facts/count.rs`）。
26. **W2a 差分门**（主会话令 2026-10-03「仔细确保算法逻辑、行为完全一致」）：a8db74a9 的 `ladder/py.rs` / `lua.rs` / `go.rs` / `c.rs` / `c_search.rs` / `c_index.rs` 原样拷进测试子仓 `unit/graph/ladder/oracle/`、只在测试里编译（`ladder/mod.rs` 以 `#[cfg(test)] #[path]` 挂上测试子仓的 `unit/graph/ladder/frozen.rs` 作 `ladder::frozen`，它以 glob 取回旧父模块的名字、逐个 `#[path]` 挂六份拷贝、带 a8db74a9 的分发器；`paths.rs` 自 a8db74a9 起代码未变、用现行的），差分仪器 `unit/dedup/ladder_diff/` 把同一批站点一边经真 `resolve/1` 问核、一边交冻结的 Rust 阶梯，逐站点比 `Outcome`，C 的强制包含弧也比。腿：真树（十个对拍语料、自仓归档、requests / cobra / koreader / luarocks / lua / fmt 六棵真树，共 7,934 个站点）与四个语言各 400 棵种子随机树 × 30 站点（两颗种子，各语言各 24,000 站点；含歧义、`..`、空说明符、空段、根外路径、模板头尾、MSVC 包含栈、框架头、强制包含）：不一致 0。核的容量上限（`Cost` 的 cap）随机树碰不到，由 `ResolveProps` 的上限腿管。
27. **W2a 正品按 Rust 控制流照抄，偏离逐条列出**：变基后为对齐控制流改了四处（Go `moduleRung` 改成按模块顺序的 best_len 折叠、`replaceRung` 改成按余部长度取首个最小、Lua 标准库改为整串比对、Python `moduleAt` 空点路径读根的原文）。仍与 Rust 写法不同、按等价论证保留的：路径是段 id 列表而非字符串（`join_rel` 的拼接改成逐段折叠，「`prefix ++ affix`」由词缀表答、树里不存在的拼接是不匹配任何文件的 `U` 段）；命中集合是文件 id 的 `IntSet` 而非路径字符串的 `BTreeSet`（文件 id 按路径序分配，二者一一对应）；Go 模块与余部长度按段数而非字节数（拥有同一说明符的模块按两种长度排序相同）；Lua 搜索目录是插入序列表而非按文本排序的 `BTreeMap`（命中取集合，次序不影响答案）；C 的旗标链在收集时按出现序编号而非在覆盖时编号（链号只用作集合的键，闭包逐链独立）；Python 的源根不做相邻去重（命中取集合）。词法判断（Python 前导点数、Lua `dofile` 是否有根、Go 头段是否带点、C 的 `<…>` 形、绝对路径的强制包含落到树内哪个文件）与空说明符拒答留在 Rust。逐函数对照表与语义单在车道日志。
28. **W2a 精度考题册分出归档册**：`docs/EVAL-SET-LANGS.md` 记第五次退役并重生成时到 756 行、过 750 行硬线；此前四次的记录（步 7、v2.31 步 4 提交 B、v2.32 步 2、v2.32 步 6）逐字节搬进新册 `docs/EVAL-SET-LANGS-REGEN.md`，主册留一行指针；新册入冻结集（子仓 `frozen_set.rs`）与引文门的退出表（`contracts/docs-citations-optout.json`），与 `EVAL-SET-FLOW-GEN1.md` 同形。
29. **W3 候选判决只做一次**（车道 W3 组 1，2026-10-03）：本仓自仓旧臂实测（`ce clone --format json`）准入单元 6,387、四源并集存活 5,434、S5 窗口对 1,475,659、其中标签界剪掉 1,467,709、S5 新对 2,516、送判 5,966——若 Rust 把每个生成出来的候选都送 `clone/1` 由核预滤，要为每个单元送树、为约 148 万个 S5 窗口对付线费，且报告的 `sent` / `prefiltered` 会变（字节不再相同）。取另一案：新族 `candidates/1`，Rust 送准入单元行与三个索引源的原料，核做尺寸界 + 标签界（`CE.Clone.Prefilter.boundOf`，一处陈述）与 S5，答保留对与账；`clone/1` 自己的预滤留作判决侧的守卫。
30. **S2 同键源在核里生成**（同组，性能门）：先行版把 S2 的约 36.6 万对随请求送过线，请求 5.7 MB、`ce clone` 慢约 2 s；改为单元行带键码、核按键分组（`CE.Candidates.T3.sameKey`），请求降到 0.88 MB。`raw_by` 的 `s2/<lang>` 与 `cross_lang_dropped` 由应答的 `raw`（每源每语言的去重对数）/ `counts.crossLanguage` 给回，字节同。
31. **产品常量走定义包**：`tables/1` 加顶层键 `limits`（`CE.Limits`），clone / candidates / docdup / dedup / similar / moves 各族的上限与比例（热组上限 64、`minDistinct` 7、Jaccard 80/100、BM25 的 k1 / b、PPMI 的地板与比例等）由 Rust 经 `tables::get().limits` 读，不再在 `wire.rs` 里留镜像；各族应答的旋钮回显保留，钉住本次判决的核就是读包的那个核。
32. **排序进核**（组 2）：新族 `rank/1`（`CE.Similar.Rank{,.Contract,.Cost,.Math,.Score}`）——请求是查询袋 `[term, channel, tf]`、语料标量、词的 df / postings / 长度与 PPMI 的边际与共现行，核算 idf、BM25 贡献（与 Rust 的闭式 `22·tf·avg / (10·tf·avg + 3·avg + 9·len)` 同一写法）、top-k、PPMI 扩展与加权袋；`similar/1` 不动，角色位仍由它判。Rust 取行的过滤读包里的两个比例（`scoredDfRatio` 2、`neighbourDfRatio` 4），核按完整性复核。无核时 similar 文档具名降级、不出行（此前按度量序出行、role 为 null）。
33. **候选生成进核**（组 3）：`candidates/1` 改送单元行 `[lang, file, key, nodes, start, end, kind, count…]`、每单元的 shingle 集、指纹实例 `[hash, file, line, tok]` 与近似段锚 `[fileA, lineA, fileB, lineB]`；S1 / S3 / S4（MinHash 128 = 32 × 4 / LSH，`CE.Candidates.Lsh`）、行锚归属、并集、两界与 S5 都在核。近似段本身（T1/T2 扩展）、`each_hash_pair` 的 T1/T2 配对留 Rust（§7）。docdup：新族 `docpairs/1` 做粗筛（LSH ∪ 共享 shingle 种子对，热组链化），`docdup/1` 加性 `seqs` 形——送未排序的 shingle 序列，核取集合、量最长逐字段并判。T3 判决缓存（`dedup/t3/cache.rs`）留 Rust、契约不变：回放的行经 `clone/1` 加性 `decide` 行仍过核的判定。
34. **fourclass L1 进核**（组 4）：新族 `moves/1`（`CE.FourClass.Moves{,.Classify,.Contract,.Cost}`）——每对文件送两侧的行 `[内容码, fnv1a(trim), 字母数字宽]`、Rust 行 diff 给出的改动行、单元行 `[键码, kind, 起, 止, 可堆叠]`；核判移动行、归属、整体搬迁的单元、单侧声明、新出现的同名顶层跨度与剩余段。行 diff（`diff.rs`，Myers over 行哈希）留 Rust：墓碑（PreToolUse 路径）与 churn 只读改动行（`fourclass::changed`），搬了是复制不是减少。上限（行 4,194,304、单元 262,144）是新的：Rust 按上限把相邻文件对装进请求，只有单个文件对自己过线时才得具名降级 `moves_too_large`；无核 / 无能力 / L1 出错 = 无文件对 + 具名原因（此前退回 Rust 的 L1）。
35. **等价门**（用户令 2026-10-03「仔细确保算法逻辑、行为完全一致」）：每组在测试子仓 `unit/w3_oracle/` 冻结一份 093aede4 的 Rust 原文（逐句照抄、永不改），差分腿把至少一万个定种子输入同时送冻结原文与真核：T3 候选全程（`unit/dedup/candidate_wire.rs`，另 1 万例克隆判定与 2 千例标签界）、docdup 粗筛与逐字段（`unit/docdup/judge/candidates.rs` 各 1 万）、排序（`unit/similar/rank_differential.rs`，1 万查询含扩展与角色）、L1（`unit/fourclass/moves.rs`，1 万文件对），边界含 ≥ 2^63 的哈希、并列与并列次序、上限与上限 +1、空输入；每条腿另断言各条边都真的走到过。
36. **生产核与 Rust 控制流的三处不同**（性能，留待主会话裁）：① S2 在核里生成（第 30 条）；② 标签交集一旦证明够不到地板就提前停（`CE.Candidates.Units.interUpTo`：返回值低于 `reachFloor` 当且仅当完整的交集低于它）；③ S5 先过标签界、再按「已在并集里」分账（并集里的对都过了两界，所以从不被标签界剪掉，两种次序的账相同）。等价论证逐条写在车道日志，三条都由差分腿覆盖。
37. **W3 的字节账**（§10 W3 行）：Rust 删 66,548 B、补回的线缆胶水 64,335 B，净只降 2,213 B——每个搬走的算法都换来一份请求装配与应答校验；差 R − H 的收窄几乎全由核增量给出（app +90,095、test +63,340）。
38. **W3 家族计数**：与第 25 条同理，`count:families#word` 不计 `candidates/1` / `rank/1` / `docpairs/1` / `moves/1`——它们答的是判决的前段（候选、排序、粗筛、L1），判决仍由 `clone/1` / `similar/1` / `docdup/1` / `fourclass/2` 下，README 与官网的「十六个判决家族」不动（子仓 `it/facts/count.rs` 的 `NOT_FAMILIES` 八项，与 `resolve/1` 同表）。
39. **量出来的结论**（W2a、W3 落地后，2026-10-04）：两波对 Rust 的净减只有约 7 KB——W2a 净 −4,748 B（§10 注）、W3 净 −2,213 B（第 37 条）。每个搬走的算法都要一层同样大小的降胶水（字符串 → 驻留 id → 请求 → 应答 → 字符串）；只要线上只许整数，这层胶水就删不掉，文本形的算法（引用阶梯、配置面读法的逻辑、墓碑 / 提及 / Markdown 的文本规则、报告渲染）也搬不动。
40. **用户裁**（2026-10-04，AskUserQuestion 三选一，选「放开「只传整数」的规定」）：放开整数过线的规定，让 Rust 胶水可以删、文本形算法可以真搬。本条只立规则（本修正案提交零代码、零 wire 改动、不升 proto）；落码车道在主会话重锁 cc-memory 计划之后起。
41. **线规则**（第 40 条的落文，§3 第一条六点）：文本过本机管道，且只过这一处——可以过线的文本、留在 Rust 的（碰世界）、核可以答文本、不动的隐私保证（计划书 §5.9-6）、性能门、wire 版本由各族自己的 proto 步定。分工句随之改为「在测量侧已读到的事实——整数或文本——上的计算在核；Rust 碰世界并交出读到的东西」，「Rust 把世界降成整数行」一句被取代，「tree-sitter 留 Rust」不变（§2）。
42. **波次重排**（§6、§7）：W2a′ 简化 `resolve/1`（路径与说明符以字符串过线，删段驻留表、`vocab`、词缀行与 `graph/resolve/lower.rs` / `tokens.rs` 的大部分；差分门与冻结预言机原样、须保持全绿）→ W2b 其余七个阶梯连同配置面逻辑；W1 / W4 / W5 不变；新增 W6 文本规则（墓碑文本管线、提及分词、Markdown 掩码 / slug、similar 的词干 / 词 / 词袋、docdup shingle；钩子路径成员受性能门约束）与 W7 报告渲染胶水（v2.32 因字符串不能过线而留在 Rust 的绑定与符号表）；§7 里理由只是「要读文本」的行重读后移出，理由是钩子热路径的行留下。新波的字节量不预估，由车道实读。
43. **W2a′ 的范围**（W2-text 阶段 A，主会话任务书 2026-10-04）：`resolve/1` 过线的是文本——路径、说明符原文、`[graph.search_roots]` 原表、各语言配置文件的原文或解码后的文档；Rust 的驻留表、逐语言降与配置事实（`graph/resolve/{intern,tokens,lower,facts}.rs`）删掉；Python / Go / C·C++ 的配置读法（`gomod.rs`、`roots.rs` 的 pyproject 读法、`cmdline.rs`、`compdb_flags.rs`、`compdb.rs` 的编译数据库读法）与阶梯一起进核。删之前每个读法都按 92e728b1 原样拷进测试子仓 `unit/graph/oracle_cfg/`（只改每份末尾单元测试的挂载路径一行；`pyproject.rs` = 旧 `roots.rs` 的那两段），只在测试里编译，挂回旧路径供冻结阶梯与差分门读。
44. **留在 Rust 的四件**（偏离任务书的「全部搬」，理由逐条）：① TOML 与 JSON 的解码——核没有 TOML 库（依赖只有 aeson / containers / array / bytestring），JSON 库解码后作 JSON 值过线、读法在核；② `lua_path.rs`——它从 tree-sitter CST 里取 `package.path` 赋值的字面量，CST 是 Rust 的（§7），且它的输出是索引键的输入、冻结的 Lua 阶梯与差分门都读它的类型；③ `c_head.rs`——include 列表在走查时读、是走查键的输入，搬进核就要把每个 C 族文件的原文送上线，列表本身已以文本过线；④ 读文件与 clangd 三探名找库（`compdb_find.rs`）——碰世界。
45. **响应文件按需读**：编译数据库的 `@file` 展开在核，读文件在 Rust——核把展开读到而请求没带的响应文件列在 `wanted`、本次不出 `results` / `forced`，Rust 读来补进 `c.responses` 再问（多轮至 `wanted` 为空）；应答的 `responses`（每个 JSON 库展开读过的响应文件）是解析键的输入，此前由 Rust 自己展开得出。
46. **字符串语义照抄 Rust**：`str::lines`（Rust 1.94：末尾单独的 `\r` 留在最后一行）、`trim` / `split_whitespace`（Unicode White_Space 25 个字符，不是 GHC 的 `isSpace`）、`char::is_alphanumeric`（由 rustc 1.94.1 逐码点生成的 844 段区间表）、长度按 UTF-8 字节（Go 的 `best_len`、`min_by_key` / `max_by_key` 取首 / 取末）、`strip_prefix` 按字节、`is_absolute` 看第 2 字节是否 `:`、`Path::extension` 的规则给 `isC`。逐函数对照表与语义单在车道日志。
47. **配置读法的差分门**（测试子仓 `unit/graph/resolve/`，挂在 `graph/resolve/mod.rs` 下）：九条腿经请求键 `inspect`（`CE.Resolve.Inspect`，产品从不送）直接问核内各读法，与冻结的 92e728b1 读法逐题比：字符类（全部码点）、`lines` / `trim` / `split_whitespace`、三种分词与 MSVC 判定、`relativize`、旗标链、go.mod、pyproject、编译数据库（含 `wanted` 回路）、`compile_flags.txt`；每腿 ≥ 10,000 题、两颗种子，go.mod 与 pyproject 每四题一题取自十二棵本机真树里的真文件再变异（BOM、CRLF、注释、续行、非 ASCII、截断、重复与删行）。本机没有任何 `compile_commands.json` / `compile_flags.txt`，这两腿只有合成题。不一致 0。阶梯的差分门（§11 第 26 条）驱动与冻结阶梯一字不改、重跑全绿，计数与 W2a 同。
48. **参考电池改按文本**：`ReferenceResolveGen` 的请求改成产品的文本形（go.mod 写成原文、pyproject 写成文档，核的读法因此在路径上），参考 `ReferenceResolve` 仍按路径片段另写一遍；`ResolveProps` 十一腿（站点倒序 ⇒ 答案倒序、未走查的文件不是目标、计数、无库无强制弧、`wanted` 先于答案、五条按名拒绝、cap 恰在 2^28）。
49. **两条测试门顺带改**：子仓 `it/unit_mounts.rs` 认一种新形——`#[cfg(test)]` 下 `pub(crate) use x::名;` 且 `x` 是本文件上方挂载的模块（`roots.rs` 把冻结的 pyproject 读法放回 `roots::pyproject` 给冻结的 Python 阶梯），它不带测试体；`it/graph_ladder_c_db.rs` 的 E01 腿改量仍在 Rust 的读法（`compdb_find.rs`、`c_head.rs` 与四份单元测试）加核里的 `Cmdline` / `Flags` / `CompDb`（核文件 ≤ 290 行）。
50. **W2b 阶段 B 的范围（R）**：R 阶梯（`ladder/r/mod.rs`）、`DESCRIPTION` 读法（`ladder/r/description.rs`）与声明目标的包代码展开（`deadcode/targets.rs` 的 `package_code` / `is_r`）进核——`CE.Resolve.R`、`CE.Resolve.Description`；`paths.rs` 的 `beside_or_root` / `declared` 在核成 `CE.Resolve.World` 的 `besideOrRoot` / `declaredIn`，核里原有的两份同义写法（Lua 的 `loaded`、C 的 `declared`）改读它们，`Path::extension` 从 `CIndex.isC` 移到 `Str.extension`、语言判定成 `World.ofLangs`。删之前按 c96ab3f6 原样拷进测试子仓：`oracle/r.rs`（只改 `description` 的挂载一行，指向 `../../oracle_cfg/description.rs`）、`oracle/paths.rs`（整份，遮住旧阶梯从父模块读到的同名模块、代码自 a8db74a9 未变）、`oracle_cfg/description.rs`（只改末尾单元测试的挂载一行）、`oracle_cfg/r_package.rs`（`package_code` / `is_r` 原文 + `gather` 的 R 分支，非整份拷贝、文件头写明）。
51. **线形与版本**：请求的 `r.descriptions` `[[路径, 原文]]` 是 `go.mods` 的同胞——走查到的配置里文件名为定义包 `resolve.configs` 所列的，`go.mod` 按 UTF-8 读、`DESCRIPTION` 按字节有损读（与旧 `parse` 同一个 `from_utf8_lossy`），读不到不带；应答 `packages` `[[包目录, [代码文件…]]]` 每次都答（降级为空表）。声明目标那一侧（`Declared::gather`）用 `nearest_up` 在盘上找到的 `DESCRIPTION`（可能不在走查里）另发一次请求，与 `forced_wire` 同形：没有 `DESCRIPTION` 就不问核；`gather` 因此可失败、核答不了即具名拒绝。9.0.0 尚未发布，按 7.2.0 的同版加性先例不另升版号（`contracts/VERSIONING.md` 9.0.0 条的「同版加性」段）。
52. **阶段 B 的差分门**：阶梯腿新增 `r-source` / `r-library` 两条（`unit/dedup/ladder_diff/r.rs`，各 400 棵树 × 30 站点），每棵树另比包代码（`beside.rs`：冻结的 `gather` R 分支对核的 `packages`）；`source` 的说明符四成取走查到的文件、从根 / 引用文件目录 / 声明根拼出。三颗种子（默认、1594323、20261004）各 12,000 + 12,000 个 R 站点，不一致 0；真树 19 棵（十个对拍语料、阶段 A 的六棵真树、自仓归档、stringr、covid19model）R 站点 1,762、包代码两份，不一致 0；其余四种语言的阶梯腿与配置读法九条腿随核里的共用函数一起重跑，不一致 0。配置读法新增 `description` 腿（`unit/graph/resolve/description.rs`，经 `inspect.description`，三颗种子各 10,000 题，每四题一题取自 stringr / covid19model 的 `DESCRIPTION` 再变异），不一致 0。核的参考电池加 R（`ReferenceResolveGen` 每例两个 R 站点、`DESCRIPTION` 写成带 CRLF / 续行 / 引号的原文；参考实现另写一遍包代码）。
53. **阶段 B 的切换门**：c96ab3f6 的 release + 核 对 本车道的 release + 核，十个对拍语料、自仓归档与八棵真树（加 stringr、covid19model），面加 `arch` / `erase` / `rules`（阶段 A 只比五个面）：identical 303、differing 1——那一个是自仓的 `ce rules`，只差回显的规则文件绝对路径（两臂的拷贝在不同名的目录里），长度同为 1,256 B、其余字段逐字同。
54. **W2b 阶段 C 的范围（Java）**：Java 阶梯（`ladder/java.rs`、`java_inherit.rs`、`java_pick.rs`、`java_sets.rs`）与类型注解的读法（`java_header.rs` 的 `past_annotation` 与它调到的词法）进核——`CE.Resolve.Java`（分派、`header_name`、`type_ref`）、`JavaAt`（`class_of` / `package_dir` / `declared`）、`JavaInherit`（继承级）、`JavaPick`（定案规则、JDK 的读法、源集规则、`class_files`）、`JavaName`（`unannotated` 与注解词法）、`JavaHeader`（请求行），每个函数以它取代的 Rust 函数命名；包索引与（包、类）索引连同每个文件「在不在 `main` 以外的源集」那一位在每次请求里只建一次（`javaEnv`），`visible` 与 `class_files` 的逐站点过滤读它们——逐站点重算源集让 gson 的一次请求在核里要 2.0 s，建一次之后 0.34 s。头部与类型的读法（`java_header.rs` 的 `read`、`java_types.rs`）留在 Rust：它们读源文件全文、走查每跑都读（`dedup/walkidx.rs`），送核的是读出的结果。删之前按 27d0d56d 原样拷进测试子仓：`oracle/java.rs` / `java_inherit.rs` / `java_pick.rs` / `java_sets.rs`（`java_pick.rs` 一处改：`past_annotation` 的路径），`oracle_cfg/java_annotation.rs`（`java_header.rs` 的注解那一半——`past_annotation` 与它调到的词法方法、`java_types.rs` 的 `ident_char`——非整份拷贝、文件头写明）。
55. **线形与版本**：Java 站点的行号成为站点行的第五列（只 Java 带；核按行找头部读到的 import 与包着站点的类型，缺行号按名拒绝）；`java.headers` 是 `r.descriptions` 的同胞，但送的是走查读出的结构而不是原文（原文是每个 Java 文件的全文）：包、import `[名字, star, static, 行]`、类型 `[名字, [父类型], [成员], 首行, 末行]`——比只送 `[路径, 包]` 多，因为阶梯也读 import（补全被折行截断的 import、单类型 import、星号 import 的包、只问 JDK 的星号判定）与类型（声明判定、继承级）。JDK 名表本就在定义包里（`tables/1` 的 `ladder.java {lang, packages}`），不另带。9.0.0 未发布，同版加性（`contracts/VERSIONING.md` 9.0.0 条的第二个「同版加性」段）。
56. **阶段 C 的差分门**：阶梯腿新增四条（`unit/dedup/ladder_diff/java.rs` + `java_gen.rs`，一个测试按表跑四条，各自计数）：`java-import`、`java-import-star`、`java-type-ref` 各只抽一种站点，`java-annotated` 三种都抽、八成说明符带类型注解（二十一种写法：字符串、文本块、字符字面量、注释、未闭合的括号与字符串）——注解读法的那条腿；说明符另带各种 White_Space 与不是空白的 U+200B、`static` 的七种拼法、空段；各 400 棵树 × 30 站点，头部由生成器直接给（包取自目录或表、import 有时同行、类型带成员与父类型、偶有走查没收的文件的头）。三颗种子（默认、11、12）各 4 × 12,000，不一致 0；真树 gson 22,698 + jsoup 24,210 + Java 对拍语料 349 个 Java 站点（自仓归档没有 Java 站点），不一致 0；其余语言的阶梯腿、21 棵真树与配置读法十腿随共用的请求与分派重跑，不一致 0。核的参考电池另写一份 Java（`ReferenceJava` / `ReferenceJavaGen`，200 例 × 12 站点，十种结果形都到；把参考里继承级的级号改错一处即红）。
57. **阶段 C 的切换门**：27d0d56d 的 release + 核 对 本车道的 release + 核，十个对拍语料、自仓归档与十棵真树（阶段 B 的八棵加 gson、jsoup），八个面（graph --sites / deadcode / structure / check / join / arch / erase / rules）：identical 335、differing 1——那一个是自仓的 `ce rules`，只差回显的规则文件绝对路径（两臂的拷贝在不同名的目录里），长度同为 1,257 B，把路径换成同一个之后逐字节同。这道门的第一次跑与随后的代价量法用的是阶段 B 遗留的 ce（建头树的脚本没带车道的 `CARGO_TARGET_DIR`，release 落进了车道自己的 `cli/target`），被请求字节那一步看出（gson / jsoup 上新臂一个 `resolve.request` 也没发），按正确的 ce 全部重跑，上面是重跑的读数。
58. **W2b 阶段 D 的范围（Haskell）**：Haskell 阶梯（`ladder/hs.rs`）、`.cabal` 读法（`graph/cabal.rs` 的 `parse` / `exposes` / `library_roots` / `keeps_private` / `module_under` 与整份 `graph/cabal_parse.rs`）、可执行与测试入口（`deadcode/targets.rs` 的 `main_targets`）与包内私有（`mounts.rs` 的 Haskell 臂）进核——`CE.Resolve.Hs`（分派与三级）、`CE.Resolve.Cabal`（读出的结构上的问答）、`CE.Resolve.CabalWalk`（逐行读法：节头、续行、字段、`common` 节与 `import:`、`main-is`、`build-depends` 的包名），每个函数以它取代的 Rust 函数命名（第 62 条）。`ladder/paths.rs` 只剩 Haskell 用的 `one_of`，核里本就有同义的 `Answer.oneOf`，随之删除；`roots.rs` 的 `beside` 只剩 cabal 读法用，删除。GHC 的 boot 包表本就在定义包（`ladder.hs.boot`），核经新的 `Tables.packAt` 读出（`Hs.bootPackages`）。留在 Rust 的是找最近的 `.cabal`（`graph/cabal_find.rs` 的 `nearest` / `cabal_in`，自 `cabal.rs` 原样搬出）——问盘上每一级目录有没有 `.cabal`，碰世界。删之前按 fa83a48d 原样拷进测试子仓：`oracle/hs.rs`（整份）、`oracle_cfg/cabal.rs`（只改末尾单元测试的挂载一行）、`oracle_cfg/cabal_parse.rs`（整份）、`oracle_cfg/cabal_mains.rs`（`main_targets` 原文 + `Declared::gather` 里 cabal 那一支，非整份拷贝、文件头写明）；`frozen_cfg.rs` 带一个 `roots` 垫片，把冻结的 `beside` 放回给冻结读法。
59. **线形与版本**：请求加性 `hs.cabals` `[[路径, 原文]]`——走查到的 `.cabal`（定义包 `resolve.configs` 加一项 `*.cabal`，开头的 `*` 读作文件名后缀；包的摘要随之变，hello 与 `tables` 的 golden 重生成），按 UTF-8 读、读不到不带；`hs.owners` `[[文件, cabal 路径]]`——只在问包内私有时带，文件须在走查里、cabal 须在 `hs.cabals` 里，否则按名拒绝。应答加性 `mains`（所带每个 cabal 的入口里在走查中的文件，升序并集）与 `private`（`hs.owners` 里被各自 cabal 留作包内私有的文件）；降级时两者为空表。声明目标那一侧把 R 的 `DESCRIPTION` 与 Haskell 的 `.cabal` 放进同一次请求（`resolve::declared`，两者都没有就不问核）；mounts 表的位 1 由每个 Haskell 文件的最近 `.cabal`（`cabal_find::nearest`，按目录缓存）组成 `owners`、一次问核（`resolve::private`，读不到的 cabal 不带、它的文件不算私有，与旧 `parse` 返回 `None` 同）。9.0.0 未发布，同版加性（`contracts/VERSIONING.md` 9.0.0 条的第三个「同版加性」段）。
60. **阶段 D 的差分门**：阶梯腿新增三条（`unit/dedup/ladder_diff/hs.rs`）：`hs-roots`（源根下的模块，R1）、`hs-packages`（依赖包导出的模块，PackageImports 点名或经 `build-depends`，R2；树里种一个 `pkg/pkg.cabal` 依赖 `other/other.cabal` 的布局，偶尔再种一个导出同一批模块的 `x/x.cabal`，名字取 `other` 或 `twin`）、`hs-external`（boot 包表的模块，受 `build-depends` 约束或被点名，R3）；每条腿另抽残缺说明符（小写段、空段、未闭合的引号）；cabal 由生成器按行表拼出（`name:` 在节头前后、四种节头大小写与有无名字、`common` 与 `import:`、续行里夹注释、顶格字段、制表符与 CRLF）；各 400 棵树 × 30 站点。每棵树另比入口与包内私有（`beside.rs` 的 `compare_declared` / `compare_private`：冻结的 `main_targets` 与 `cabal::nearest` / `parse` / `keeps_private` 对核的 `mains` / `private`）。三颗种子（1、2、3）各 3 × 12,000，不一致 0；真树 25 棵（十个对拍语料、阶段 A 的六棵真树、本车道的头树、stringr、covid19model、gson、jsoup，加三棵 Haskell 树：dataframe@96ec374、keel@71ed44a、nccl@fc487d34 的 `companion/haskell`）Haskell 站点 6,438（R1 1,662、R2 937、R3 2,516、ambiguous_root 2、out_of_scope 1,321）、入口 55、包内私有 470 / 771，不一致 0；其余语言的阶梯腿与配置读法十腿随共用的请求重跑，不一致 0；查重门点名的新块消掉之后（比对的拼写收成 `Tally::agree`、三条 Haskell 腿收成一张表、文本读法的抽题收成 `bom_texts`），种子 1 与真树重跑，各腿计数与改写前逐行同。配置读法新增 `cabal` 腿（`unit/graph/resolve/cabal.rs`，经 `inspect.cabal` 比目录、名字、各节的源根 / 入口 / 是否库节、依赖、有无库节、隐藏模块与导出集——导出集逐词问，词表取文本拼出的每个词并上核答出的名字），三颗种子各 10,000 题、每四题一题取自真树的 28 份 `.cabal` 再变异（行表里的包名带 `_` / `.` / `:` / `+` / 非 ASCII），不一致 0。两个探针：去掉 `sourceRoots` 的「没有节认领就取全部节」回退 → 阶梯腿红；让 `depName` 收 `_` → `cabal` 腿红；按副本还原、sha 相同。核的参考电池另写一份 Haskell（`ReferenceHs` / `ReferenceHsGen`，200 例 × 12 站点，比阶梯、入口与包内私有；另一腿要求每种结果都走到）。
61. **阶段 D 的切换门**：fa83a48d 的 release + 核 对 本车道的 release + 核，十个对拍语料、自仓归档、阶段 C 的十棵真树与三棵 Haskell 树（dataframe、keel、nccl 的 `companion/haskell`），八个面（graph --sites / deadcode / structure / check / join / arch / erase / rules）：identical 383、differing 1——那一个是自仓的 `ce rules`，只差回显的规则文件绝对路径（两臂的拷贝在不同名的目录里），长度同为 1,257 B。三棵 Haskell 树与自仓共 6,438 个 Haskell 站点（`graph --sites`）。自仓的 `ce check` 两臂都退 1（本车道的基线最后才具名重立，两处棘轮超线），两臂逐字节同。
62. **阶段 D 的函数对照表**（fa83a48d 的 Rust → 核；语义按 Rust 照抄，字符串语义同第 46 条）：

| Rust（fa83a48d） | 核 |
|---|---|
| `ladder/hs.rs` `resolve` | `CE.Resolve.Hs.resolveHs`（分派在 `CE.Resolve.judged` 的 `site`） |
| `ladder/hs.rs` `module_shaped` | `Hs.moduleShaped` |
| `ladder/hs.rs` `split_package` | `Hs.splitPackage` |
| `ladder/hs.rs` `cabals` | `CE.Resolve.judged` 的 `cabals`（请求 `hs.cabals` 每份交 `Cabal.parse`） |
| `ladder/hs.rs` `owner` | `Hs.owner` |
| `ladder/hs.rs` `source_roots` | `Hs.sourceRoots` |
| `ladder/hs.rs` `depended_rung` | `Hs.dependedRung` |
| `ladder/hs.rs` `external_rung` | `Hs.externalRung`；boot 包表 `Hs.bootPackages`（`Tables.hsBoot` 经 `Tables.packAt` 读定义包） |
| `ladder/paths.rs` `one_of` | `CE.Resolve.Answer.oneOf`（核里已有） |
| `graph/cabal.rs` `parse` | `CE.Resolve.Cabal.parse`（逐行交 `CabalWalk.walk`；读文件留 Rust：`request::texts`） |
| `graph/cabal.rs` `nearest` / `cabal_in` | 留 Rust：`graph/cabal_find.rs`（原样搬出） |
| `graph/cabal.rs` `Cabal::exposes` | `Cabal.exposes` |
| `graph/cabal.rs` `Cabal::library_roots` | `Cabal.libraryRoots` |
| `graph/cabal.rs` `Cabal::keeps_private` | `Cabal.keepsPrivate`（mounts 表经应答 `private`，`CE.Resolve.judged`） |
| `graph/cabal.rs` `module_under` | `Cabal.moduleUnder` |
| `deadcode/targets.rs` `main_targets` | `Cabal.mainTargets`（经应答 `mains`） |
| `graph/cabal_parse.rs` `step`（含 `HEADS` 与缩进判断） | `CabalWalk.steps`（`heads`、`indented`） |
| `graph/cabal_parse.rs` `open` | `CabalWalk.open` |
| `graph/cabal_parse.rs` `continuation` | `CabalWalk.continuation` |
| `graph/cabal_parse.rs` `finish` | `CabalWalk.finish` |
| `graph/cabal_parse.rs` `consume` | `CabalWalk.consume` |
| `graph/cabal_parse.rs` `merge` | `CabalWalk.merge` |
| `graph/cabal_parse.rs` `import_commons` | `CabalWalk.importCommons` |
| `graph/cabal_parse.rs` `words` | `CabalWalk.wordsOf`（`words` 是 Prelude 的名字） |
| `graph/cabal_parse.rs` `set_main` | `CabalWalk.setMain` |
| `graph/cabal_parse.rs` `head_word` | `CabalWalk.headWord`（`lowerAscii` / `alnumAscii` = `to_ascii_lowercase` / `is_ascii_alphanumeric`） |
| `graph/cabal_parse.rs` `split_field` | `CabalWalk.splitField` |
| `graph/cabal_parse.rs` `dep_name` | `CabalWalk.depName` |
| `graph/cabal_parse.rs` `Walk::default` + `parse` 的循环 | `CabalWalk.walk` |
| `graph/roots.rs` `beside` | 删除：读文本在 `request::texts`，目录在核（`Str.parentDir`） |
| `graph/mounts.rs` `Manifests::haskell` | `Manifests::cabal_of`（只找路径，读法进核） |
