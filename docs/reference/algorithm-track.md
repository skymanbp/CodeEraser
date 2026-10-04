# 算法轨设计册：算法进核（计划 v2.33，ADR-008 分工句修正）

> 本册是 v2.33 的落码权威（计划书横幅 v2.33 条、ADR-008 与 §6 T 轨 v2.33 行指向这里）。落码时以本册为准，改设计先改本册再改计划书再动代码。体例照 [authority-track.md](authority-track.md)（v2.32）与 [analysis-track.md](analysis-track.md)（v2.31）。

## 0. 一句话定位

凡是只在「已降成整数的事实」上做的计算都归核：Rust 把世界（文件、文法、索引、git、进程）降成整数行，Haskell 在这些行上跑算法、下判决、出文档。本轨把今天仍在 Rust 里的这类算法按波次搬进核，每波落地后实读语言条，Haskell 字节数超过 Rust 即停；1.9.0 等这一刻再发。

## 1. 裁定与目标

1. **用户令**（2026-10-03，原话）：「要求改成：haskell占比比rust高。不固定比例阈值。」——取代 2026-09-25 的「Haskell ≥ 40 %」与 v2.32 步 9 的「口径类裁定」。
2. **两裁**（同日 AskUserQuestion）：① 路线 = 把算法搬进 Haskell，**不拆仓**（拆 GUI 子仓、把 Rust 挪出主仓一类的口径办法不做）；② **1.9.0 等目标达成再发**（v2.32 步 10 的发版排在本轨达标之后）。
3. **口径** = GitHub 语言条（主仓）：`.gitattributes` 里 `cli/tests/**`、`contracts/**` 为 linguist-vendored，`site/**` 为 linguist-documentation，`core/test` 计入。本地近似读法见 §10；发版声明以 `gh api repos/skymanbp/CodeEraser/languages` 的读数为准。
4. **达标式**：设 R / H 为某一提交上 Rust / Haskell 的计入字节，目标 = H > R。搬走的 Rust 净减 x、核净增 y（含电池与参考实现，`core/test` 计入），v2.32 步 6 的删除量 s（落地后实读）：H₀ + y > R₀ − x − s。以本册立项基线（§10 第二行）R₀ − H₀ = 936,192 B 计，y ≈ x 时要 x > (936,192 B − s) / 2。
5. **每波落地后实读一次，达标即停**：不预设要做完五波；未做的波在 §11 记去向。

## 2. 分工句

- **旧**（ADR-008 原文与硬约束 1）：判决在核；解析 / 索引 / 前端在 Rust。测量语义留 Rust；判据 = 需源文本或行级内容过 wire 即测量侧（§5.9.2）。ADR-008 验收段：「占比提升是副产品，禁止为占比搬迁或改写已有代码」。
- **新**：凡是只在已降成整数的事实上做的计算——算法、判定、定义、文档、文字——都在核；Rust 只做碰世界的事（文件系统、tree-sitter、SQLite、git、进程、管道、clap / MCP / daemon / 钩子接线）和把世界降成整数行。
- **两判例按新句改写**：细则第五期（墓碑）里对文本的切分与标记匹配是「降」、留 Rust，建立在降出的计数之上的判定是算法、归核（墓碑文本管线在钩子热路径上，本轨具名不搬，§7）；细则第六期（同角色顾问）里分词与词干是词法、留 Rust，BM25 分子分母与 PPMI 计数是词哈希 / tf / df 上的算法、归核（W3）。
- §5.9.2 一票否决原样有效：需源文本或行级内容过 wire 的仍是测量、留 Rust；新句扩的是它的反面——不需要源文本的计算不再因「历来在 Rust」而留下。

## 3. 不变量（一条不动）

- **整数过线**：名字、路径、源文本不过线；文本侧的切分 / 词法 / 分词仍是「降」，留 Rust，过线的是哈希与驻留 id。
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

| 波 | 内容（Rust 侧现量 KiB） | wire 形（整数） | 路径 | 风险 |
|---|---|---|---|---|
| W1 镜像退役 + 冷路径小件 | §5 镜像（除 zone）；`query/` 的 lexer / program / columns / legend（27）；`structure/` 的 tree / rows / edges + `arch/tables.rs`（25）；`config/` 的故障检查与 `canonical.rs`（16，钩子留本地 fail-open 读法）；`update/version.rs` / `update/manifest.rs` 的比较 | 既有请求不动或加性；cap 与默认值改读 `tables/1` / hello | cold / K | 低 |
| W2 引用阶梯 | `graph/ladder/` 纯路径代数 22 个文件 + `graph/cabal_parse.rs` / `cmdline.rs` / `compdb_flags.rs` / `gomod.rs` / `jsonc.rs` / `roots_ts.rs` / `deadcode/targets.rs` / `nodes.rs` / `stored.rs`（≈ 205）；Rust 侧 `rs_*` / md / html 阶梯先出逐文件 surface 行 | 新族 `resolve/1`：路径 = 段 id 序列（Rust 驻留表，段文本不过线）、站点 `[kind, from, 段…]`、文件集、逐语言配置事实；应答每站点 `[rung, target \| refusal]` | K（增量刷新） | 中：体量最大；十三语言精度册全部重生成（判决字节须同） |
| W3 候选与排序 | T3 / docdup 候选生成（`dedup/sources.rs` / `dedup/minhash.rs` / `dedup/candidates.rs` / `docdup/judge/candidates/runs.rs`，32）；`similar/bm25.rs` / `similar/ppmi.rs`（17；`stem.rs` / `terms.rs` 是词法，留 Rust）；fourclass L1（`diff.rs` / `model.rs` / `decls.rs` / `stacking.rs` / `anchor.rs`，21）；score / join / merge 的装配 join（30） | 既有族加性表：单元签名行、shingle 集、查询词的 postings 切片、行哈希对 | K / S | 中：超线性段要 ABAB，核端用数组 / IntMap |
| W4 CST 行导出 + 其上的算法 | Rust 一个通用导出器（仿 `scan/metrics/events.rs`）：每单元前序行 `[kind, field, parent, named, line, identHash]`；其上搬 `scan/calls.rs` / `scan/callees.rs`（24）、fourclass 的 `declared.rs` 与 `visibility/` 各语言（≈ 54）、mention 的 `conv/` / `selfref.rs` / `candidates.rs`（≈ 45） | 新族 `cst/1`（或各族加性 `rows`） | K（刷新） | 中 |
| W5 flow 降表 | `flow/` 的 17 个降表文件（≈ 114）改在核里从 CST 行算 CFG 事实（表本就在核） | `flow/1` 请求改送 CST 行 | **H**（guard / flow 两侧降表）与 S | 高：钩子路径，只送改动单元；探针预算见 §8 |

- **W2 状态**（2026-10-03）：W2a 落码在车道 `lane/v233-w2`（未落地）——新族 `resolve/1`（proto 8.1.0，步 6 的 8.0.0 之上的加性 minor）接走 Python / Lua / Go / C·C++ 四个阶梯的查找：核 `core/app/CE/Resolve.hs` + `Resolve/` 十一个模块，Rust `cli/src/graph/resolve/`（驻留表、逐语言降、请求与应答）+ `graph/owed.rs`（无核记账）；`ladder/py.rs` / `lua.rs` / `go.rs` / `c.rs` / `c_search.rs` / `c_index.rs` 删除。留在 Rust 的降：站点检测、`c_head.rs` 读 include 列表、`lua_path.rs` 抽 `package.path` 模板、`gomod.rs` 读 go.mod、`compdb*.rs` / `cmdline.rs` 读编译数据库并拆旗标、`[graph.search_roots]` 的配置读法；`ladder/paths.rs` 留给仍在 Rust 的 R / Haskell 阶梯（§11 第 16–23 条）。其余语言（ts / hs / java / r / rs / md / html）仍走 Rust 阶梯，W2b 接。
- **proto**：每个新族 / 加性表随自己的 minor（或在 v2.32 步 6 的 8.0.0 之后的 minor），版本号落地时按 `contracts/VERSIONING.md` 的顺序定；golden 由子仓 `fixture_contract::regen` 生成。
- **粗算**（设计草案：净减按现量 70 %、核增量按搬走量 1.0× 含电池，单位 KB）：W1 ≈ −50 / +45，W2 ≈ −145 / +190，W3 ≈ −70 / +95，W4 ≈ −85 / +115，W5 ≈ −80 / +105 → Rust ≈ 2,406 − 430 − s，Haskell ≈ 1,468 + 550 = 2,018；s ≥ 0 时 Rust ≤ 1,976。五波全做才过线、余量不大——每波落地实读，偏差当波修正。
- **车道**：W1 / W2 / W3 三条车道并行（目录不相交：query · structure · config / graph / dedup · docdup · similar · fourclass），W4 的导出器同时起，W5 等 W4。落地按波序，一族的落地提交才带它的 proto / golden / 事实。

## 7. 不搬的与理由

- **墓碑文本管线**（`cli/src/tombstone/`）：在 H 路径上对源文本做切分与匹配，是「降」；其上的整数判定早已在核（`tombstone/1`）。
- **dedup 的 winnow / pairs**（`cli/src/dedup/winnow.rs`、`cli/src/dedup/pairs.rs` 的配对）：daemon 探针在 H 路径上仍要 Rust 一份，搬了是复制不是减少（`minDistinct` 的判定按 §5 退役，配对本身留下）。
- **分词 / 词法 / stemmer**：文本侧的「降」。
- **C 类全部**（§4）：碰世界的事。
- **GUI**：前端接线。

W5 之后仍未达标，再议这三项与 GUI（§11 第 9 条）。

## 8. 性能与钩子

- 每族切换提交带 release 同坐 ABAB ×7（旧 / 新二进制、自仓副本），读数进 `docs/PERF-BUDGET.md`；`ce check` 暖跑不得慢过 +15 %，慢了先用中继核录真请求单测核（核端的二次方行走是已见过的成因）。
- 钩子探针（PreToolUse）：W1–W4 不碰探针路径。W5 的 guard / flow 经 daemon 持有的核链，预算 = 探针 p50 增量 ≤ 10 ms（bench `hook_probe` 同坐对比）；过不了就让 guard 留 Rust 降表一份、其余面走核，并如实记账。

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
| W2a（车道 `lane/v233-w2`，变基到 093aede4 之后，四个阶梯的查找进核） | 2,396,214（`cli/src` 2,370,605 + `gui/src-tauri` 17,481 + `scripts/tsprobe` 8,128） | 1,553,185（`core/app` 1,015,832 + `core/test` 535,756 + `scripts/tsprobe` 1,597） | 39.33 % | 843,029 | 本车道终树，上述量法；对 093aede4（上一行）Rust −4,143、Haskell +84,308（`core/app` +58,890、`core/test` +25,418） |

注：草案把 `scripts/tsprobe` 的两种扩展名合记在 Rust 名下（8,128 + 1,597 ≈ 10 KB）；按扩展名分开后 Haskell 多 1,597 B。e35fde53 与 40ab1a4f 之间 `.rs` / `.hs` 无改动，两行是同一份字节的两种记法。

注（W2a）：Rust 只净减 4,143 B——删掉的四个阶梯的六个文件 37,908 B（`ladder/py.rs` / `lua.rs` / `go.rs` / `c.rs` / `c_search.rs` / `c_index.rs`），换来的降与接线 33,765 B（`graph/resolve/` 五个文件 24,610、`graph/owed.rs` 3,428、其余十三个文件的改动 5,727，其中差分门的 `#[cfg(test)]` 挂载与说明约 1.1 KB）。查找本身在 Rust 里很紧凑，而把路径、站点与配置降成段 id 的那一层与它同量级；W2 行的「≈ −145」按这个比例看要大幅下调（§11 第 24 条）。

## 11. 拍板记录

1. **用户改令**（2026-10-03）：「要求改成：haskell占比比rust高。不固定比例阈值。」——目标从 Haskell ≥ 40 % 改为 Haskell 字节数 > Rust 字节数，不设比例线（§1）。
2. **路线**（同日 AskUserQuestion）：把算法搬进 Haskell，不拆仓（§1 第 2 条）。
3. **发版**（同日 AskUserQuestion）：1.9.0 等目标达成再发；v2.32 步 10 排在本轨达标之后。
4. **口径**：GitHub 语言条（主仓），`.gitattributes` 三条照旧；本地按 §10 近似读，发版以 `gh api` 读数为准。
5. **分工句修正**（ADR-008，立项提交）：§2 的新句取代旧句；细则第五 / 六期两判例按新句改写；验收段「禁止为占比搬迁或改写已有代码」改写为「按新分工搬迁是分工的兑现，每次搬迁仍过本族的逐字节门」。
6. **文本侧的降留 Rust**：切分 / 词法 / 分词 / stemmer 是降，过线的是哈希与驻留 id（§3）。
7. **不搬三项与 C 类**（§7）：墓碑文本管线、dedup 的 winnow / pairs、分词 / 词法 / stemmer；C 类与 GUI。
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
24. **W2a 的读数**：语言条 37.96 % → 39.33 %，差 931,480 → 843,029 B（对 093aede4）；Rust 只净减 4,143 B（§10 注）——整数过线要求 Rust 侧留一层降（驻留表、逐语言降、配置事实），它与被搬走的查找几乎一样大，所以一个阶梯的搬迁对 Rust 的净减远小于 §6 粗算的「净减按现量 70 %」。
25. **W2a 家族计数**：`count:families#word`（README / 官网的「十六个判决家族」）不计 `resolve/1`，与 `hello` / `tables/1` / `document/1` 同理——它答一个站点通向哪里，判决仍是图族的事（子仓 `it/facts/count.rs`）。
26. **W2a 差分门**（主会话令 2026-10-03「仔细确保算法逻辑、行为完全一致」）：a8db74a9 的 `ladder/py.rs` / `lua.rs` / `go.rs` / `c.rs` / `c_search.rs` / `c_index.rs` 原样拷进测试子仓 `unit/graph/ladder/oracle/`、只在测试里编译（`ladder/mod.rs` 以 `#[cfg(test)] #[path]` 挂上测试子仓的 `unit/graph/ladder/frozen.rs` 作 `ladder::frozen`，它以 glob 取回旧父模块的名字、逐个 `#[path]` 挂六份拷贝、带 a8db74a9 的分发器；`paths.rs` 自 a8db74a9 起代码未变、用现行的），差分仪器 `unit/dedup/ladder_diff/` 把同一批站点一边经真 `resolve/1` 问核、一边交冻结的 Rust 阶梯，逐站点比 `Outcome`，C 的强制包含弧也比。腿：真树（十个对拍语料、自仓归档、requests / cobra / koreader / luarocks / lua / fmt 六棵真树，共 7,934 个站点）与四个语言各 400 棵种子随机树 × 30 站点（两颗种子，各语言各 24,000 站点；含歧义、`..`、空说明符、空段、根外路径、模板头尾、MSVC 包含栈、框架头、强制包含）：不一致 0。核的容量上限（`Cost` 的 cap）随机树碰不到，由 `ResolveProps` 的上限腿管。
27. **W2a 正品按 Rust 控制流照抄，偏离逐条列出**：变基后为对齐控制流改了四处（Go `moduleRung` 改成按模块顺序的 best_len 折叠、`replaceRung` 改成按余部长度取首个最小、Lua 标准库改为整串比对、Python `moduleAt` 空点路径读根的原文）。仍与 Rust 写法不同、按等价论证保留的：路径是段 id 列表而非字符串（`join_rel` 的拼接改成逐段折叠，「`prefix ++ affix`」由词缀表答、树里不存在的拼接是不匹配任何文件的 `U` 段）；命中集合是文件 id 的 `IntSet` 而非路径字符串的 `BTreeSet`（文件 id 按路径序分配，二者一一对应）；Go 模块与余部长度按段数而非字节数（拥有同一说明符的模块按两种长度排序相同）；Lua 搜索目录是插入序列表而非按文本排序的 `BTreeMap`（命中取集合，次序不影响答案）；C 的旗标链在收集时按出现序编号而非在覆盖时编号（链号只用作集合的键，闭包逐链独立）；Python 的源根不做相邻去重（命中取集合）。词法判断（Python 前导点数、Lua `dofile` 是否有根、Go 头段是否带点、C 的 `<…>` 形、绝对路径的强制包含落到树内哪个文件）与空说明符拒答留在 Rust。逐函数对照表与语义单在车道日志。
28. **W2a 精度考题册分出归档册**：`docs/EVAL-SET-LANGS.md` 记第五次退役并重生成时到 756 行、过 750 行硬线；此前四次的记录（步 7、v2.31 步 4 提交 B、v2.32 步 2、v2.32 步 6）逐字节搬进新册 `docs/EVAL-SET-LANGS-REGEN.md`，主册留一行指针；新册入冻结集（子仓 `frozen_set.rs`）与引文门的退出表（`contracts/docs-citations-optout.json`），与 `EVAL-SET-FLOW-GEN1.md` 同形。
