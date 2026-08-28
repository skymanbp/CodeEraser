# 热路径延迟分解表（M0 验收项，DEVELOPMENT_PLAN.md §6 M0 / §7.6）

> **封册（M7.5 深度瘦身，2026-08-18）**：重放仪器 `perf_budget.rs` 已随
> 休眠仪器整体退役（EVAL-SET.md 修正案），本册数字为最终实测账本；
> 后续性能数字随各批 As-built 记录，回归复核走 git 历史复活仪器。

> 口径：被动 guard 的 PreToolUse 端到端 = hook 触发 → 判定返回。
> 预算为硬上界；"实测"列只写真实测过的数字，未测项标注实测里程碑，
> 不预填。
>
> **本预算不进 CI，这是口径而非欠账（K 步 10 改定，2026-08-25）**：
> 同一台静默机上同形窗口的墙钟已实测漂 1.8×（M5-3h：churn 先决 156.7 s
> → 落地复测 278.4 s，成因=子进程创建开销随机器状态漂移），共享 CI
> runner 的代际与邻租噪声只会更宽——门要么天天误报、要么容差宽到形同
> 虚设。延迟回归的复核路是：本机静默复测入本册 + BENCH.md 单主机逐版本
> 系列（bench_append/bench_backfill，`#[ignore]` 手跑）；`perf_budget.rs`
> 需要时按 EVAL-SET.md「再生成」三步复活（checkout 父提交连同同代支撑，
> 跑毕重退役）。CI 守住的是行为与判决，不是毫秒。

| # | 环节 | 预算 | 实测（本机 Win11, 2026-08-07） | 实测里程碑 |
|---|---|---|---|---|
| 1 | Claude Code fork hook 进程（Windows shell form 经 PowerShell） | ≤ 300 ms | 空转 hook ×60（Win11，Stopwatch 计进程创建到退出）：**PowerShell** min 247.4 / median 271.4 / **p95 303.7** / max 338.0 ms（60 次中 5 次 >300）；对照 **cmd.exe** min 20.2 / median 21.6 / p95 23.4 / max 24.4 ms | ⚠️ M3（见下注） |
| 2 | `ce` 冷启动（进程 + clap 解析 + 退出） | ≤ 100 ms | `ce --version` ×10：min 28.3 / median 30.3 / max 53.1 ms | ✅ M0 |
| 3 | named pipe 连接 + 指纹探针往返（daemon 热态） | p95 ≤ 150 ms | ping ×100（101k LOC 仓）：median 0.27 / p95 0.50 / max 5.78 ms | ✅ M2 |
| 4 | 判定组装 + stdout JSON 回传 | ≤ 50 ms | deny(含组装回传) 中位 64 vs clean(无输出) 中位 70 ms——边际成本埋没于运行噪声(≲10 ms) | ✅ M3 |
| 合计 | PreToolUse 端到端 | **p95 < 1 s**（含 Defender/冷 daemon 余量） | ce 侧（行 2+3+4）：deny p95 69 / clean p95 81 ms；冷首呼(懒起 daemon) 213 ms。行 1 用实测 p95 303.7 加总 ≈ 0.39 s < 1 s | ce 侧 ✅ M3；全链待 0.x 预览实录 |

> **行 1 注（2026-08-08 实测）**：PowerShell 形态 p95 303.7 ms **压线越过自身
> 300 ms 预算**（60 次中 5 次 >300），但它是 `ce` 启动之前的**宿主开销**，CE 侧
> 无法优化；总预算 p95 < 1 s 仍有 0.6 s 余量，故不列为阻塞项。cmd.exe 形态快
> 一个数量级（p95 23.4 ms），说明该成本几乎全部是 PowerShell 自身启动。
> 口径：`<shell> -c exit` 的进程创建到退出，用 Stopwatch 计时，**不含** Claude
> Code 在 fork 之前的内部开销 —— 那部分只能由 0.x 预览的真实会话实录覆盖，
> 与 dogfood 验收同批完成。

## M2 克隆索引预算（计划 §6 M2，实测 2026-08-07，release，合成 101,200 LOC 语料）

| 项 | 预算 | 实测 | 状态 |
|---|---|---|---|
| 10 万 LOC 全量索引（含扩展验证与配对，919 块） | < 30 s | 1.92 s（合成 101,200 LOC） | ✅ |
| 真实仓库列：ripgrep 3fce3b5 全仓 56,386 LOC(.rs) 冷启动 | 同上口径 | 1.29 s（10,920 块） | ✅ |
| 单文件增量刷新（内容哈希门控 + 重插指纹） | < 200 ms | 2.50 ms | ✅ |
| 参考：warm 全量 analyze（索引快路径 + 全配对） | —（无预算） | 701 ms | 记录 |
| 提及语料宇宙 pass（`ce graph --mentions`：自有第二 walk + 三发射器分词 + 两表写入，自仓 595 文件 / 199,941 行；plan v2.17 L 轮片 (3)，不在 `dedup::analyze` 热路径内；口径 = pass 本体 = wall − 前置判决索引刷新 ≈0.37 s〔trace 实测〕） | 冷 < 2 s / 暖 < 600 ms（暖地板 = walk 0.26 s + 全量读哈希 0.20 s：spec §5.1 自有内容哈希门必读字节，mtime 门不采） | 冷 ≈1.95 s（wall 2.32 s；32 文件一批提交 + 末尾单次 checkpoint + 每 run 一次 COUNT 快照——每批 COUNT 曾实测 24 ms×19 批 = +0.5 s 且随表线性增长，故不采）/ 暖 ≈0.54 s（wall 0.91 s；`capped` GROUP BY 27 ms 计入；实测 2026-08-27，release，对抗审查修后） | ✅ |

## M5-2e 图缓存预算（设计档 RG4，实测 2026-08-12，release，合成语料）

| 项 | 预算 | 实测 | 状态 |
|---|---|---|---|
| resolve_key 变更 ⇒ 全仓重解析扫掠（13,800 缓存站点 / 115,000 LOC；回调空转——机制成本，阶梯真实成本 2f 实测） | < 2 s | 6.62 ms | ✅ |
| 复测：10 万 LOC 全量索引（v4 起同事务含相 1 符号/站点抽取） | < 30 s | 2.19 s（v3 时 1.92 s） | ✅ |
| 复测：单文件增量刷新（v4 起同事务写图行） | < 200 ms | 1.90 ms | ✅ |

## M5-3b/3d 第 4+5 次 per-file 解析预算（设计 F11/RM7，重测 2026-08-13 3d 批，release，钉定树材料化五语料）

schema v5 起 refresh_file 同事务多一次 unitsig 解析（token 流丢树，T3 事实
另花一次解析）；3d 起再多一次 docdup 段抽取解析（RM7：独立加法抽取器，与
tokenize 各走各的树——诚实记账，不写"零重解析"）。口径：`eval_t3_universe`
生成器的产品索引腿——钉定 tip 的 in-scope 树材料化后 `dedup::refreshed_index`
冷/暖各一次；文件数 = 仪器 scope（含 md；tokenized 仅 grammar 语言）。

| 语料（文件数） | 预算 | 冷索引（含第 4+5 次解析） | 暖（内容哈希门控） | 状态 |
|---|---|---|---|---|
| self（141） | 沿用 M2 全量 < 30 s | 1.07 s（3b 两解析时 1.19 s） | 144 ms | ✅ |
| ripgrep（133 = 110 rs + 23 md） | 同上 | 2.06 s（3b 2.14 s） | 356 ms | ✅ |
| zod（393，最大语料） | 同上 | 3.12 s（3b 3.58 s） | 626 ms | ✅ |
| cobra（53）/ requests（50） | 同上 | 512 / 390 ms | 92 / 61 ms | ✅ |

复跑：该 `#[ignore]` 生成器腿已随 M7.5 封册退役（见文首横幅）——复现本表
需按 EVAL-SET.md 再生成节从 git 历史复活仪器后以 `--release` 跑（外部语料
另设 CE_SLICE_REPO/CE_GRAPH_NAME/CE_GRAPH_TIP；debug 构建的数字带
"NOT admissible" 标注拒收——2026-08-13 曾测得 debug 冷 7.56 s 即为此类，已弃）。

## M5-3e T3 判决冷路径（设计卷二 §4.4，实测 2026-08-13，release）

口径：`ce clone <root> --db <fresh>` 端到端 = 冷索引（含五次解析）+ 四源候选
+ 单元树重建 + 分块 clone.request（pairCap 4096）+ TED 判决 + 回映；暖 = 同
库重跑（索引哈希门控，判决照跑——判决无缓存，3f 前不设）。达标线 = self 与
ripgrep 冷 < 60 s；zod 为容量压力例（存活对最多），超线即收 pairCap 并公布
触发器——本轮未触发。

| 语料 | 存活对 | 送判（请求数） | 台账丢弃(over-cap+forest) | 冷 | 暖 | 状态 |
|---|---|---|---|---|---|---|
| self | 655 | 524（1） | 5+126 | 2.44 s | 1.15 s | ✅ |
| ripgrep | 6,201 | 6,156（2） | 25+20 | 24.9 s | 22.4 s | ✅ |
| zod（压力例） | 21,742 | 19,193（5） | 2,318+231 | 47.5 s | 43.8 s | ✅ |

> 判决段主导（暖 ≈ 冷 − 索引 1~4 s）。首版 Ted.hs 用 IntMap 逐格建表，
> self 524 对即 ≈ 20 s、ripgrep 600 s 未跑完（中止）；改 ST 无箱数组
> （森林表是稠密矩形）后 self 判决段 ≈ 1 s、ripgrep 22 s——重写全程由
> CloneProps 穷举等价电池守护，clone golden 重放字节同值。

## M5-3h churn 腿先决测量（设计卷二 §6.1 红线：blame 代价先入册，实测 2026-08-14，release）

口径：`ce churn .. --days N` 现状端到端 = 窗口提交枚举 + 逐提交逐文件 `git show`
双侧 + fourclass 解析 + 逐 touched 文件 `git blame --line-porcelain HEAD`（无缓存）。
自仓 138 提交（14d 与 30d 同数=仓史不足 30 天）：**156.7 s / 154.9 s**——blame+show
子进程风暴主导。3h 约束（产品化前先决）：逐单元 churn 归属必须在 classify_commit
**既有**的 show+units::segments 解析面内完成，**零新增 git 调用**（结构性断言，
非事后测量）；blame/survival 半场不动。

### 3h 落地复测（同日晚，release，静默机）

| 项 | 实测 | 记录 |
|---|---|---|
| `ce churn .. --days 14`（逐单元台账重构后，139 提交） | 278.4 s | 同形窗口墙钟对先决测量摆动 156.7→278.4 s（~1.8×）——子进程风暴的创建开销随机器状态漂移（行 1 的 PowerShell 进程创建教训同源），非台账重构成本 |
| `ce join ..` 端到端（冷 fresh db：全量索引 + 四源图 + pos 行 + churn + 双层装配） | 265.0 s | join 的非 churn 腿 ≈ 噪声内（同日 join 全程 < churn 单测）；首测 284.3 s 弃用=五盲审代理并发争用污染 |
| 台账跨期守恒 | 44,879 = 44,870(473adfc 双计数器) + 9(473adfc 自身) | 派生总数逐数复现旧计数器口径，rewrote 4,255 不变 |

## M5-3k Haskell 入语料后全链（实测 2026-08-14，release，静默机，fresh `.ce/` 已核实删除）

口径：37 个 `.hs`（3,646 行）首次进入 scan/dedup 语料（graph 仍五语言=3l 前显式门）。

| 项 | 实测 | 记录 |
|---|---|---|
| `ce scan .`（252→251 文件、2,984 函数含 ~790 Haskell 单元） | 0.9 s | tree-sitter-haskell 解析增量在噪声内 |
| `ce dedup .`（冷，225 文件、150 块） | 2.3 s | 语料 generation 后冷索引不劣化 |
| `ce check .`（暖索引） | 1.15 s | ✅ pre-commit 级维持 |

## M5-3l graph 阶梯后全链（实测 2026-08-14，release，静默机，fresh `.ce/` 已核实删除）

口径：176 个 Haskell import 站点入图（cabal 解析 + 两 rung 阶梯 + 1,371 模块
boot 表逐串匹配），GRAPH_REV 3 全量重解析。

| 项 | 实测 | 记录 |
|---|---|---|
| `ce scan .`（256 文件、2,765 函数） | 0.52 s | 噪声内 |
| `ce dedup .`（冷，228 文件、149 块） | 2.68 s | 站点检测 + phase-2 阶梯全量重放在冷成本内 |
| `ce deadcode .`（暖索引 + 判决往返） | 0.47 s | 619 kept 边、0 dead |
| `ce check .`（暖索引） | 1.36 s | ✅ pre-commit 级维持；boot 表线性扫描（~80 站点 × 43 包）无感 |

## M8 缺口清算 `#[path]` 阶梯后（实测 2026-08-18，release，GRAPH_REV 5 全量重建）

口径：mod_decl 站点先探 `#[path]` 属性（per-sweep 缓存树上的兄弟回溯），自仓
+4 kept 边（三个 `#[path]` 测试挂载 + eval_docdup_precision 的挂载）、−4 未解析。

| 项 | 实测 | 记录 |
|---|---|---|
| `ce dedup .`（冷，REV 5 全量重建，`.ce/` 删除后计时同链） | 2.72 s | 与 REV 3 先例 2.68 s 同量级，属性探针无感 |
| `ce deadcode .`（暖索引 + 判决往返） | 1.56 s | 733 kept 边、0 dead、615 未解析 |

## Stop 审计预算（立行 2026-08-19，release，自仓 278 文件，非静默机，n=5 手测）

口径：e2e = 信封 + 两 git 腿 + 进程内 dedup（大头）+ four-class 降级字段 + observe。成本立场（拍板统一）：**执法腿为判决付费、信息腿绝不付 spawn**。
Request::Dedup 路由实测打平（721 vs 723 ms，daemon 逐请求重算），撤案。此处原挂
「真提速=结果缓存+失效」，**K 步 10 兑现（2026-08-25，用户拍板「能实现并利用起来
就做」）——但层位从 daemon 挪到 `dedup::analyze` 咽喉**：daemon 层缓存只服务
`Request::Dedup`（生产面零发送者，grep 全仓：唯 dispatch.rs 接收），是死码；而
analyze 是全产品暖路的单一咽喉（audit Stop 腿、precommit、`ce check` 的 score
腿、erase/structure/join/docdup/deadcode、GUI 与 MCP faces、daemon 两臂、CLI），
一处缓存全员受益、零 wire 变更。实现 = `dedup/rescache.rs` 单槽结果缓存入
index.db（schema v12），失效键按原设计钉 = files 表逐文件 content-hash 链式聚合
摘要 + 生效 filter；params 与全部算法 rev 沿既有 meta 键整库 wipe，免费失效。
命中路仍跑 refresh（变更检测本身）与边扫（resolve_key 可无内容漂移而变），summary
的 refreshed/removed 按本次实况重建、绝不回放存储跑的值。

| 项 | 预算 | 实测 | 状态 |
|---|---|---|---|
| `ce audit --hook` e2e 暖 / 冷 | median <1.5 s / <5 s | 721 ms（707–958）/ 3.06 s（评审 PoC） | ✅ |
| 分解：dedup 单独（暖）/ git 腿 | —（记录） | 469 ms / 80 ms | 记录 |

## K 步 10 结果缓存落地后（实测 2026-08-25，release，自仓 ~393 文件，用户会话活跃窗口，n=5）

口径：analyze 相位打点（临时 eprintln，测后还原）实测暖态 ~555 ms 分摊 = refresh
walk+hash ~255 / instances ~7 / **stream 重载 ~300** / 边扫 ~1 / **clone_blocks ~5**——
可缓存段（后三相）占 56%。缓存落地后同窗同法复测；audit e2e 绝对值不与 2026-08-19
的 721 ms 行直比（本窗工作树带 15 文件未提交差异，four_class 腿真跑 daemon+core），
留档如下、判决内容经 rescache_face 双电池与毒饵反事实证与重算逐字节同判。

| 项 | 预算 | 实测 | 状态 |
|---|---|---|---|
| `ce dedup .`（暖，命中） | —（记录） | 294 ms median（276–310），改前同窗 547–613 ms，−47% | 记录 |
| `ce audit --hook` e2e（暖，本窗口径） | median <1.5 s | 1.26 s median（1.25–1.50） | ✅ |
| schema v11→v12 wipe 首跑（冷重建） | —（一次性） | 3.76 s | 记录 |

## L 轮片 (7)+(8) K45 传否路 A/B（实测 2026-08-28，release，自仓 HEAD da68275 两棵 worktree 各自 `.ce/`，用户会话静默窗，n=9 交错 ABAB）

口径：mention pass 是自有入口、不在 `dedup::analyze` 内，五条传否路不得为顾问付费（spec S-A15/W3-F7/W4-F2）。
A = 旧客户端（1f493df，L 轮前，graph/1 6.1.0）、B = 本批客户端，两者对同一新核（6.2.0，合法 minor 偏斜），
各自一棵相同内容的 HEAD 树、各自索引（schema v13 vs v14）；每腿先各预热一次，再九轮 A/B 交错取中位数。
同日早一窗（并发 20 代理跑 rg）测得 erase 散布 1.63–8.07 s，作废不列——噪声窗与静默窗不混列。

| 项 | 预算 | 实测 A（旧） | 实测 B（本批） | 状态 |
|---|---|---|---|---|
| `ce audit --hook` e2e（Stop 信封，暖） | median <1.5 s | 1.186 s（1.145–1.256） | 0.954 s（0.936–1.091） | ✅ 不因本批变慢 |
| `ce erase .`（plan，dry-run） | —（不变慢） | 1.526 s（1.451–1.676） | 1.493 s（1.427–1.673） | ✅ |
| `ce check .`（暖索引） | —（不变慢） | 1.786 s（1.720–2.558） | 1.802 s（1.729–1.928） | ✅ 差 +0.9%，在散布内 |

## v0.2.0 符号绑定批后（实测 2026-08-19，release，GRAPH_REV 7 + SCHEMA v8 全量重建，非静默机）

口径：`pub use` 绑定面入阶梯（rs_reexport 单遍历 surface+hash）+ pubuse_hash 入 resolve_key + edges.via_reexport；REV 6→7 与 v7→v8 双 wipe 同批；用户会话活跃窗口（3j 先例：环境负载可致数倍摆动，绝对值按本窗口读）。

| 项 | 实测 | 记录 |
|---|---|---|
| `ce dedup .`（冷，272 文件、169 块/86 组） | 5.15 s | REV 5 先例 2.72 s；文件 228→272 + 绑定面首建 + 负载窗口合成，未静默机复测 |
| `ce check .`（token 暖 + REV 7 图首建） | 4.84 s | 边相阶梯全量重放一次性成本 |
| `ce check .`（真暖） | 2.54 s | ✅ pre-commit 级维持 |

## M5-3j 门迁移后 `ce check`（+.hs size-only 走文件，实测 2026-08-14，release，静默机）

口径：3i 口径 + `hs_size_rows` 的第二次全树 walk（37 个 `.hs` 读文件计行）。

| 项 | 实测 | 记录 |
|---|---|---|
| 冷（fresh `.ce/`，删除已 Test-Path 核实） | 2.2–2.8 s（4 跑） | `.hs` 走文件增量在噪声内 |
| 暖（哈希门控索引） | 0.90–0.98 s | ✅ pre-commit 级 |
| 3i 行的冷 31.6 s | 今日静默机不可复现 | 按实测保留原记录不改写；量级差=环境主导（3i 测量窗口与盲审/CI 并发同期），非判决路径成本——本节 4 跑散布为现行口径 |

## M5-3i `ce check` 判决路径（ADR-006 门，实测 2026-08-14，release，静默机）

口径：`ce check ..` 端到端 = 索引刷新 + T1/T2 块 + 图 pos + scan 全量度量指纹化
+ 成员集哈希 + 单条 verdict.request + 回判。churn 表默认空（`--days` 显式开——
blame 代价见 3h 节，空表=诚实缺席非零主张）。

| 项 | 实测 | 状态 |
|---|---|---|
| 冷（fresh db，全量索引+图+scan+判决） | 31.6 s | CI 门可承受（与 cargo test 同级） |
| 暖（哈希门控索引） | 1.21 s | ✅ pre-commit 级 |
| 容差活体 | 2 条 toleranceDrawn（本批文档编辑被 max(+2%,+10) 吸收）| 机制实证 |

## M5-3g docdup 判决冷路径（设计卷二 §5.3，实测 2026-08-14，release）

口径：`ce docdup <root> --db <fresh>` 端到端 = 冷索引（六次解析含 docsegs）
+ LSH∪种子候选 + 逐字 run（seed-extend，候选文件重读走 walked_text 单喉）
+ 分块 docdup.request（docPairCap 4096）+ Haskell 精确 Jaccard + 回映。
自仓（工作树，ce.toml 排除 crosscheck）：89 live 段、38 候选（全种子源）、
1 请求、38 判、**0 上报**（RM13 报告态：自仓文档现无可报重复）——冷 2.59 s ✅。
五语料 docdup-precision 生成（钉定树材料化 + 全量产品跑 ×5 + 候选双跑）
一次 67.5 s ✅；单语料判决段均 < 3 s（段宇宙远小于单元宇宙，pairCap 未触发）。

| 项 | 预算 | 实测（30 次） | 状态 |
|---|---|---|---|
| `ce probe --hook` e2e：解析信封 + 探针往返 + 判定组装 + 回传 | p95 < 1 s（合计行 ce 侧） | median 64 / p95 69 / max 73 ms | ✅ |
| 同上，clean 路径（无判定输出，静默） | —（无预算） | median 70 / p95 81 ms | 记录 |
| 冷首呼（懒起 daemon + 首连 + 判定） | —（降级档兜底，ADR-003） | 213 ms | 记录 |

复跑：`perf_budget.rs` 已随 M7.5 封册退役——按 EVAL-SET.md「再生成」节的复活律**连同同代支撑**
复活、跑毕重退役：`git show 0c7c936^:cli/tests/perf_budget.rs > cli/tests/perf_budget.rs && git archive 0c7c936^ cli/tests/common | tar -x`，再
`cargo test --release --test perf_budget -- --ignored --nocapture`（合成语料确定性生成；hook e2e =
`hook_e2e_p95_under_1s`），跑毕 `rm -rf cli/tests/perf_budget.rs cli/tests/common`
（`cli/tests` 自 9bedcc4 起是 submodule：复活件在两个仓都未跟踪，对着 submodule 路径 `git checkout <sha> -- <路径>`
会静默把 gitlink 换成历史 blob，退役必须是纯 `rm`；退役仪器留在树里会进下一次门：dedup 预算与棘轮都计其块与行）。

补充口径：

- 环节 2 的 Defender **首扫**（新编译 exe 第一次运行）不计入常规预算，单列记录（M0 验收原文）。
- 会话累计口径：hook 延迟中位数 < 15 s / 百次编辑（M3 验收）——**实测 0.982 s**
  （2026-08-10 定稿：0.2.0 feed 全量 10 会话、2,671 次 probe，按会话求
  均值×100 后取中位；min 0.196 / max 2.111 s，census 见 T1-INTERCEPT.md §4）。✅
- daemon 冷启动（首次索引构建）不占热路径——未就绪期显式降级为廉价检查档（ADR-003）。
- 复测命令：`cli/` 下 `cargo build --release` 后
  `1..10 | %{ (Measure-Command { .\target\release\ce.exe --version }).TotalMilliseconds }`。

## ADR-008 P3 `ce scan` 接核后冷路径（实测 2026-08-17，release，自仓 273 文件 / 3,015 函数 ≈ 18.6k 测量行）

> 口径：`ce scan . --core <exe>` 端到端 = walk+parse+度量 + 一次 scan.request
> （分块阈 524288 行，自仓单请求）+ 整报告镜像 ensure + 渲染。P3 验收条款
> "冷延迟入 PERF-BUDGET 实测、超标单片回滚"的落账（反审 C18 补记）。

| 环节 | 账面基线（3l，无核） | 实测（×3 连测） | 判 |
|---|---|---|---|
| `ce scan .` 冷（首跑，核首启+握手） | 0.52 s | 0.98 s | ✅ 无回滚触发 |
| `ce scan .` 暖（核复用系统缓存） | 0.52 s | 0.42–0.43 s | ✅ 反快于无核基线 |

复跑：`cargo build --release` 后
`1..3 | %{ (Measure-Command { .\target\release\ce.exe scan . --core $env:CE_CORE_BIN }).TotalSeconds }`。

## M6 S4b `ce structure` 验收实测（2026-08-17，release，外部真语料，--db 指 scratch=真冷）

> 计划 M6 验收行：「10 万 LOC 冷启动到首屏 <60s、报告打开 <3s」。GUI 首屏
> = 同一 judge::run+report_json 管线 + webview 渲染（毫秒级），故 CLI 端到端
> 即首屏时延的账面上界。冷=索引/图/判决全新建；暖=同 db 重跑（≈「报告打开」）。

| 语料 | 规模（rs/ts/py/go/md 行数） | 冷 | 暖 | 门 |
|---|---|---|---|---|
| zod | 71,645 | 8.36 s | 2.66 s | ✅ 冷 ≪60s；暖 <3s |
| ripgrep | 55,076 | 5.29 s | 1.64 s | ✅ 同上 |

- 10 万 LOC 无现成单仓语料——按 zod 线性外推 ≈11.7s，距 60s 门 5 倍余量；
  外推注记如实入册（非实测数字）。
- 面效度旁证：ripgrep 树 982/1000、zod 平铺 src 794/1000（axes 3:63 错位
  文件）——与两仓结构口碑同向。
- 复跑：`ce structure .ce-eval\corpora\<c> --db <fresh> --core $env:CE_CORE_BIN`
  各二连（首=冷、次=暖）。
