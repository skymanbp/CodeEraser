# 热路径延迟分解表（M0 验收项，DEVELOPMENT_PLAN.md §6 M0 / §7.6）

> **封册（M7.5 深度瘦身，2026-08-18）**：重放仪器 `perf_budget.rs` 已随
> 休眠仪器整体退役（EVAL-SET.md 修正案），本册数字为最终实测账本；
> 后续性能数字随各批 As-built 记录，回归复核走 git 历史复活仪器。
> 末尾七节（v0.2.0 符号绑定批后、M5-3j、M5-3i、M5-3g、M3 探针端到端、ADR-008 P3、
> M6 S4b，实测 2026-08-14–08-19）已于 2026-10-05 逐字节迁入
> [PERF-BUDGET-ARCHIVE.md](PERF-BUDGET-ARCHIVE.md)（正册离 `ce scan` 的 750 行硬线
> 只剩 7 行，算法轨 v2.33 W2-text 阶段 C 的一节放不下）；阶段 E 的一节又放不下时，
> 末尾两节（v2.33 W3 与 W2a，实测 2026-10-03–10-04）同样逐字节迁去。W1 的一节（2026-10-06）又放不下时，v2.32 步 2 一节（实测 2026-10-01）同样逐字节迁去。

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
| 合计 | PreToolUse 端到端 | **p95 < 1 s**（含 Defender/冷 daemon 余量） | ce 侧（行 2+3+4）：deny p95 69 / clean p95 81 ms；冷首呼(懒起 daemon) 213 ms。行 1 用实测 p95 303.7 加总 ≈ 0.39 s < 1 s。**全链实录（2026-08-29，见下注）**：修前 20 次 Write 中 13 次触发宿主 `Slow PreToolUse hooks` 告警 2016–2344 ms；`ce.sh` 改会话绑定戳后 min 322 / median 385 / **p95 502** / max 568 ms，告警 0 | ✅ 2026-08-29（全链 p95 0.50 s < 1 s） |

> **行 1 注（2026-08-08 实测）**：PowerShell 形态 p95 303.7 ms **压线越过自身
> 300 ms 预算**（60 次中 5 次 >300），但它是 `ce` 启动之前的**宿主开销**，CE 侧
> 无法优化；总预算 p95 < 1 s 仍有 0.6 s 余量，故不列为阻塞项。cmd.exe 形态快
> 一个数量级（p95 23.4 ms），说明该成本几乎全部是 PowerShell 自身启动。
> **全链注（2026-08-29 实测，L 轮步 #15 O52）**：真 headless 会话（`claude -p`，
> `--settings` 只启 codeeraser 插件，`--debug-file` + `-d hooks`），每条 assistant
> 消息恰一次 Write，共 20 次；单次 = 宿主 `[Stall] tool_dispatch_start` 时戳 −
> 该 assistant 消息落盘时戳 − 同行 `permissionDecisionMs`，两端都是宿主时钟。
> 修前的根因不在 ce：`plugin/bin/ce.sh` 每次 hook 都重走全链——两次 SHA256 +
> ~15 次 fork，Windows 每 fork 70–100 ms，单 wrapper 1.7 s，全链 2.0–2.3 s，
> 宿主 ≥ 2000 ms 才打 `Slow PreToolUse hooks`（无逐 hook 起点标记，故取上式）。
> 修法 = 校验按会话一次：已验证路径写入 `CLAUDE_PLUGIN_DATA/bound-<清单版本>.env`，
> 后续 hook 经戳直接 exec（清单或二进制比戳新即重验，`health` 恒全链），单 wrapper
> 0.21 s；信任边界不变（ADR-007）。`bootstrap_e2e.sh` 状态 11–14 钉住戳的四条律。
> 口径：`<shell> -c exit` 的进程创建到退出，用 Stopwatch 计时，**不含** Claude
> Code 在 fork 之前的内部开销 —— 那部分已由上面的全链注覆盖（2026-08-29 真实
> 会话实录，两端都是宿主时钟；步 #16 O52 收口）。

## M2 克隆索引预算（计划 §6 M2，实测 2026-08-07，release，合成 101,200 LOC 语料）

| 项 | 预算 | 实测 | 状态 |
|---|---|---|---|
| 10 万 LOC 全量索引（含扩展验证与配对，919 块） | < 30 s | 1.92 s（合成 101,200 LOC） | ✅ |
| 真实仓库列：ripgrep 3fce3b5 全仓 56,386 LOC(.rs) 冷启动 | 同上口径 | 1.29 s（10,920 块） | ✅ |
| 单文件增量刷新（内容哈希门控 + 重插指纹） | < 200 ms | 2.50 ms | ✅ |
| 参考：warm 全量 analyze（索引快路径 + 全配对） | —（无预算） | 701 ms | 记录 |
| 提及语料宇宙 pass（`ce graph --mentions`：自有第二 walk + 三发射器分词 + 两表写入，自仓 595 文件 / 199,941 行；plan v2.17 L 轮片 (3)，不在 `dedup::analyze` 热路径内；口径 = pass 本体 = wall − 前置判决索引刷新 ≈0.37 s〔trace 实测〕） | 冷 < 2 s / 暖 < 600 ms（暖地板 = walk 0.26 s + 全量读哈希 0.20 s：spec §5.1 自有内容哈希门必读字节，mtime 门不采） | 冷 ≈1.95 s（wall 2.32 s；32 文件一批提交 + 末尾单次 checkpoint + 每 run 一次 COUNT 快照——每批 COUNT 曾实测 24 ms×19 批 = +0.5 s 且随表线性增长，故不采）/ 暖 ≈0.54 s（wall 0.91 s；`capped` GROUP BY 27 ms 计入；实测 2026-08-27，release，对抗审查修后） | ✅ |
| ↳ v2.31 步 4 提交 B 收尾：`MENTION_REV` 3 → 4（产品签名规则，walk.rs `signed`）A/B（release，同树同库 = 主根 `.ce/index.db`，顺序 A A B B A A B B、每对首跑是切 rev 后的 `rescanned` 全量重扫、次跑暖；wall 含前置判决索引刷新；量前 `Win32_Process` 无别的 cargo / ce、CPU 14 %；2026-09-30） | —（无预算，记录用） | A（rev 3）：冷 7.42 / 6.38 s、暖 1.51 / 1.54 s，universe 1,394、rows 488,545；B（rev 4）：冷 7.33 / 6.68 s、暖 1.55 / 1.68 s，universe 1,254、`skipped.signed` 140、rows 415,053。结论：140 份签名档不再分词入表，rows −73,492（−15.0 %）；冷跑的差在同坐噪声内，暖跑 +0.04–0.13 s——签名档从不入表，故每跑都读一次并解析 JSON 判签名（与二进制规则同一代价形） | 记录 |
| `ce deadcode` 端到端相位分解（自仓，暖，release，`.ce/index.db`；三连跑稳态取后两跑；临时探针实测后即回退，2026-08-30） | —（无预算，记录用） | **总 ≈1.70 s** = 判决索引刷新 `dedup::refreshed_index` **0.59–0.68 s** + 图装配（`graph_rows`/config/`nodes_of`/`node_row`/`Declared`）≈0.13 s + `edge_wire+contain` **0 ms** + `export_surface` **3 ms** + **`advisory::tables` 0.66–0.73 s**〔`mention::refresh` 0.32–0.39 s ∥ `candidates::unmentioned` 0.33–0.35 s ∥ `mounts::facts` 0.02 s〕+ 核往返 `judge+consume` **0.11 s** + 进程起停/打印 ≈0.20 s | 记录 |
| ↳ 读法：v1.2.0→v1.3.0 的 deadcode 暖跑回归（bench 413→923 ms）**整体对应 `advisory::tables` 这一新增相位**，且该相位近似对半分——提及刷新一半、候选/否决一半。最大的单一相位是判决索引刷新（≈0.6 s），但它**非本轮新增**，每条 ce 命令都付。故「把提及宇宙那一遍优化掉」最多触及全命令的 ~9%（读+哈希 ≈0.15 s，见上一行 mtime 条），**不是大头，也拿不回 1.2.0 的基线**。**上限的正确算法是差值**：预筛不是把读省成零，是把读换成 stat——自仓宇宙 832 文件 / 10.29 MiB 实测读 155 ms、stat 32 ms，故净上限 **123 ms**，比它前面那两问 `git ls-files`（151 ms）还小 | —（无预算，记录用） | 结论：无单一大头可攻；上一行的 `mtime 门不采` 维持 | 记录 |

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
摘要 + 生效 filter；params 与各算法 rev 沿既有 meta 键整库 wipe（tokenizer rev 例外：只清解析派生表，trend 行按工具链戳保留待重量，v2.29 DEP-TS 起），免费失效。
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

## v2.26 墓碑度量 Stop 腿 A/B（实测 2026-09-04，release，同一台机、静默窗〔计时前 `Get-CimInstance` 零外来负载〕，n=5 交错 ABAB，各腿先预热一次）

口径：A = HEAD 7e06f45 的二进制（本批之前），B = 本批二进制；两者先在同一棵干净 HEAD worktree
（0 个改动文件）上量，再在同一棵本批工作树（主根 27 个改动文件 / 578 KB，gated 子仓根 12 / 59 KB）上量——
tombstone 腿在后者真配对真读真量，A 在同一棵树上付的是它本来就付的 numstat + fourclass diff + 判决索引刷新。
首测 B 在干净树上慢 190 ms：tombstone 腿在零改动时也付 `diff` + `rev-parse --show-prefix` + `cat-file`
三个 git spawn——改为 numstat 已知零改动即不配对不 spawn、blob 规格改 `HEAD:./path` 借 `git -C root`
自解析（去掉 rev-parse 那一个 spawn）后重量如下。

| 项 | 预算 | 实测 A（HEAD） | 实测 B（本批） | 状态 |
|---|---|---|---|---|
| `ce audit --hook` e2e（Stop 信封，干净 HEAD 树，0 改动） | median <1.5 s | 0.592 s（0.575–0.594） | 0.580 s（0.561–0.587） | ✅ 不因本批变慢 |
| 同上，本批工作树（27 改动文件 + 子仓 12） | —（记录） | 3.511 s（3.395–3.897） | 4.166 s（4.060–4.495） | 记录：+0.65 s |
| ↳ 分解（临时探针三跑取中，实测后即回退；主根 / 子仓根） | —（记录） | — | 配对 `diff -M -C` 180 / 89 ms、`cat-file --batch` 124 / 86 ms、measure 138 / 44 ms | 记录 |

读法：+0.65 s 里约 72 % 是两个 git spawn（每个 gated 根各一对），28 % 是度量本体（每侧三次 tree-sitter
解析：docdup 段落、单元、字面量）。可攻的两处都在判决侧模块之外不可得：fourclass 与 tombstone 对 HEAD
问的是同一个 `-M -C` diff 的两种拼法（unscoped / `--relative`），合一省 ≈180 ms 但要先统一 fourclass 的路径
词汇；三次解析共用一棵树省 ≤ 120 ms 但要给 docdup/units 入口加带树的变体。第一段（observe-only）两者都不动，
随第二段裁。

## v2.27 墓碑判决进核后 Stop 腿 A/B（实测 2026-09-05，release，同一台机，静默窗 = 计时前后 `Get-CimInstance` 零外来负载、系统基线占用 17–31 %，n=5 交错 ABAB，各腿先预热一次）

口径与 v2.26 那节同：A 仍是 7e06f45 的二进制（墓碑两段之前），B = 7a0698c 的二进制（判决进核 + `ce commitmsg`
+ `///` 合段之后）；两者用**同一份树内容的两棵 worktree**（一棵一个二进制——两个二进制的索引修订互异，同一个
`.ce/` 会被来回重建），先量干净的 7a0698c 树（0 改动），再把 16 个文件（`cli/src/tombstone`、`audit`、`guard`、
`config` 与册 14，共 103 KB）换回 70dfebb 的版本作脏树；脏树上 B 真配对真读真量（29 个候选面、81 个被抹名字、
1 处散文站点 `surfaces.rs:111`），核只在 `[tombstone] budget` 声明时被问（缺席 = 条件不评估），A 对同一棵树付的
仍是 numstat + fourclass diff + 判决索引刷新。绝对值与 v2.26 节不可比（不同 worktree、不同机器状态），只读 A/B
之差与配对控制组之差。

| 项 | 预算 | 实测 A（7e06f45） | 实测 B（7a0698c） | 状态 |
|---|---|---|---|---|
| `ce audit --hook` e2e（Stop 信封，干净树，0 改动） | median <1.5 s 的 v2.26 口径；本节只读差 | 1.648 s（1.353–1.817） | 1.594 s（1.343–1.895） | ✅ 打平 |
| 同上，脏树 16 文件，未声明 budget（核不问） | —（记录） | 1.965 s（1.915–1.994） | 2.426 s（2.370–2.511） | 记录：+0.46 s = 度量腿本体 |
| 脏树 + `budget = 1000`（核问，答 `over = false`）vs 同分钟的无 budget 控制组 | —（记录） | — | 2.644 s（2.577–2.655）vs 2.525 s（2.479–2.622） | 记录：+0.12 s |
| 脏树 + `budget = 0`（核问，答 `over = true`）vs 同分钟的无 budget 控制组 | —（记录） | — | 2.553 s（2.515–2.583）vs 2.561 s（2.498–2.609） | 记录：−0.01 s |
| 干净树 + `budget = 0`（零候选面，核不问） | —（记录） | — | 1.368 s（1.339–1.589） | 记录：声明本身不付钱 |

读法：判决进核这一步（`tombstone/1` 一问一答，29 行）的代价 ≤ 0.12 s，两次配对（+0.12 / −0.01 s）落在噪声内——
`verdict::open` 打开一次链路给两个判决共用，`audit/tombstone.rs` 在 `over` 为真时也只是把站点格式化进 feed，
observe 档不做第二件事。度量腿本体的 +0.46 s 与 v2.26 节的 +0.65 s（27 文件）同量级，仍是每个 gated 根一对
git spawn 加三次 tree-sitter 解析。**一次被丢掉的读数**：游戏进程刚退出的那一分钟里，`budget = 0` 脏树读到
3.680 s（3.263–3.892，五次单调上升），同一配置十分钟后重量为 2.553 s——退出后的系统整理是外来负载，按
「量前量中查负载」的规矩作废，记在这里是为了下次先等一分钟。

## v2.29 步 3 词袋倒排表入索引后冷 / 暖索引 A/B（实测 2026-09-05，release，同一台机，自仓 687 文件，`ce dedup --db <scratch>` 各自成库，A/B 交错 n=3，量前 `Get-CimInstance` 系统占用 ≈10 %）

口径：A = af8dbf8 的二进制（索引 schema 15），B = 本批（schema 16：`bag` + `df` 两表随 `refresh_file` 同事务写入）；
冷 = 先删库再 `ce dedup`，暖 = 原地立即再跑一次（零改动，内容哈希门全部短路）。判决面零变化，本节只记索引代价。

| 项 | 预算 | 实测 A（schema 15） | 实测 B（schema 16） | 状态 |
|---|---|---|---|---|
| `ce dedup`（冷，687 文件） | 沿用 M2 全量 < 30 s | 5.1–5.3 s | 8.0–8.7 s | 记录：+3 s |
| `ce dedup`（暖，零改动） | —（不变慢） | 0.51–0.56 s | 0.50–0.55 s | ✅ 打平 |
| `.ce/index.db` 体积 | — | 10.7 MB | 18.0 MB | 记录：+7.3 MB（bag 177,536 行、df 5,697 行；自有单元 5,458——687 文件里 387 自有，子仓 300 只当读者不写行） |
| ↳ +3 s 分解（临时探针三跑取中，实测后即回退） | — | — | 解析 0.65 s（第六次 tree-sitter + docdup 段再抽取）；SQL ≈ 2.3 s，其中倒排索引 ≈ 1.5 s、`df` 差分 ≈ 0.1 s | 记录 |

读法：SQL 那 2.3 s 里大头是**随机键**——`term_hash` 是 fnv1a64，177,536 行按逐文件事务写入时每次提交都把 b-tree 的随机页刷回，
这是倒排表的本性而不是布局之过：把同一批行按同一事务形回放到五种布局（Python `sqlite3`，同一 SQLite 库、无解析，三跑取最小）——
rowid + 双索引 2.45 s / 14.8 MB、WITHOUT ROWID (term, unit) + unit 索引 2.50 s / 12.3 MB（**采用**）、WITHOUT ROWID (unit, term) + term 索引 2.40 s / 12.3 MB、
去外键 2.54 s、去掉倒排索引 0.91 s / 11.4 MB、页缓存 64 MiB 2.52 s（无益，pragma 不采）——时间彼此打平，唯一能省的是倒排索引本身，而那正是查询要走的路。
**被否决的形**：spec 初稿的 `cooc(a, b, n)` 对表——自仓 688k 行、库 58 MB、冷索引 35–50 s（7–10×）；联想视图是 opt-in，于是对表不存、
边际 n_a 存进 `df.marg`，共现对在查询时由携带该词的单元的 bag 行推导（回放五语料逐位与内存表相同）。暖路不付钱：零改动时 `retire` /
`refresh_bags` 都在内容哈希门之后，一行 SQL 都不发。

## v2.29 步 7 `ce similar` 查询代价（实测 2026-09-05，release 05abf06，同一台机，HEAD 的独立 worktree 自带 `.ce/`，量前 `Get-CimInstance` 系统占用 13 %，各臂预热一次后 n=5 三臂交错）

口径：索引已按步 3 节建好（本次冷 `ce dedup` 9.19 s / 暖 0.52 s，库 17.4 MB，与步 3 节同量级）；`ce similar` 端到端 = 进程起 + 同一内容哈希门的
索引刷新（零改动即短路）+ `Reader` 打开（座位 / 长度两问）+ 查询 + 一次 `similar/1` 核问答 + 渲染。查询点 `--at cli/src/similar/bm25.rs:84`（`top_k`），
`--text "fetch the user row by id"`；一文件 Δ = 给 `stem.rs` 追加一行注释再撤回，各三次。顾问不在 bench 面，本节只记代价不设门。

| 项 | 预算 | 实测（n=5，秒） | 状态 |
|---|---|---|---|
| `ce similar --at`（裸臂，零改动） | 与 `ce dedup` 暖跑同量级 | 0.71 / 0.72 / 0.74 / 0.76 / 0.76 | ✅ 0.7 s 级 |
| `ce similar --at --widen`（联想视图） | —（opt-in，不设） | 1.78 / 1.83 / 1.86 / 1.87 / 1.88 | 记录：+1.1 s |
| `ce similar --text`（自由文本） | 同裸臂 | 0.70 / 0.71 / 0.73 / 0.74 / 0.74 | ✅ 与裸臂打平 |
| 一文件改动后的 `ce similar --at`（n=3） | 步 3 差分：只付那个文件 | 0.99 / 1.05 / 1.09（撤回后 1.00 / 1.05 / 1.06） | 记录：+0.3 s = 一文件解析 + unitsig + 词袋差分 |

读法：裸臂 0.7 s 里索引刷新短路后的大头是 `Reader::open` 把自有单元的座位与长度整表读出（`SEATS` / `LENS` 两问，5.4k 单元）与一次核握手；
`--text` 不比 `--at` 便宜，说明查询本身（119 词 × 倒排范围扫描）不是大头。**联想臂多付的 1.1 s 是步 3 裁定的直接代价**：不存 cooc 对表，
每个拼出的词通道查询词都要从携带它的单元的 bag 行现算共现计数（`reader.rs`，`4·n_a > N` 的词按界直接跳过）——联想视图是 opt-in，所以这
1.1 s 只由要它的人付，而库不为它多长 40 MB、冷索引不慢 7–10×（步 3 节的 A/B）。一文件 Δ ≈ 0.3 s 与步 3 节「暖不变、差分只付净变化的词」一致。
复跑：`perf_similar.ps1` 形——`git worktree add --detach <tmp> HEAD` → `ce dedup .` 两次 → 三臂各预热一次 → 5 轮交错 → 追加 / 撤回一行各三次 → 删 worktree。

## v2.30 步 5b-7 PostToolUse 记录腿 `ce settle --hook` 代价（实测 2026-09-27，release 本树，同一台机，夹具 = 本仓当日 `.ce/observe.ndjson` 副本 2,996 行 / 880 KB 再追加三行 `ask` 探针行，二进制预热后 n=30，`python` `perf_counter` 夹 `subprocess.run`、含进程起）

口径：`ce settle --hook` 端到端 = 进程起 + 读信封 + `judging_root` + 读一次本会话的 feed 窗口（`hookio::session_lines`，逐行找同 `tool_use_id` 的 `probe` 行与 `settled` 行）+ 至多追加一行；不问 daemon、不问核、零输出。三条路各量：本会话答过 `ask` 且未记（追加一行）、已记过（只读）、从未答过 `ask`（只读）。

| 项 | 预算 | 实测（毫秒） | 状态 |
|---|---|---|---|
| 追加路（首次为该 `tool_use_id` 落 `settled`，n=2） | 与 PreToolUse 探针同量级（M3 节 median 64 ms） | 41.2 / 39.7 | ✅ |
| 已记路（同 id 再触发，n=30） | 同上 | median 40.5 / p95 51.2 / max 232.2（30 次中 1 次离群，原因未查；紧随其后的 30 次 max 48.3） | ✅ |
| 从未 `ask` 路（n=30） | 同上 | median 39.4 / p95 47.0 / max 48.3 | ✅ |
| 新编译二进制冷首呼 | —（M3 节同一口径单列） | 857 | 记录 |

读法：三条路打平，说明大头是进程起 + 把 880 KB 的 feed 读成 JSON 行——追加那一行与找那一行都在噪声内；feed 按会话读，长会话的窗口就是它的代价上限，与墓碑腿的会话并集（同一读法）一样。**PostToolUse 只在人放行之后触发**，这 40 ms 不在任何写入的前面。
复跑：空目录 `git init` + 出厂 `ce.toml`，把一份真 feed 拷成 `.ce/observe.ndjson` 并追加三行 `{"event":"probe","session_id":"timing","tool_use_id":"toolu_time_N","decision":"ask",…}`；信封 = PostToolUse JSON（`hook_event_name` / `session_id` timing / `tool_name` Write / `tool_input.file_path` / `tool_use_id` toolu_time_N / `cwd`）；先用一个从未 `ask` 的 id 预热五次，再各路计时，每次看 feed 行数只在追加路 +1。

## v2.30 步 5b-9 T3 进门 + 判决缓存 A/B（实测 2026-09-27，release，同一台机、同一棵树、同一窗口：树 = 2263995 的干净检出〔子模块就位〕，A = 2263995 的二进制〔索引 schema 16，T3 不进门〕，B = 本批〔schema 17，T3 进门 + `t3ted` 判决缓存〕；块序 A B A B、每块 4 跑，换二进制后的首跑是整库重建、单列不计；量前 `Get-CimInstance` CPU 8 %、无 cargo / 对拍进程；`python` `perf_counter` 夹 `subprocess.run`、含进程起）

口径：两臂读同一棵树的同一份 `.ce/index.db`（schema 16 / 17 交替时各自整库重建一次——正是交替使用两个版本时索引会做的事，首跑因此单列）；B 的缓存暖 = 上一跑已把这棵树的每一对判决记进 `t3ted`，缓存空 = 用 `DELETE FROM t3ted; DELETE FROM meta WHERE k = 't3_cache'` 清掉后的一跑。判决面：两臂 `ce clone` 命中同为 444 对；`ce check` A 944 / B 917、sim 对 36 → 255——那是 kind 1 进了 sim 表，是判决变化不是代价，见 CHANGELOG。

| 项 | A（5b-8） | B（5b-9，缓存暖） | B（缓存空，索引暖） | 状态 |
|---|---|---|---|---|
| `ce check .`（暖索引，n=6 / 6 / 1） | 3.00–3.04 s | 4.04–4.40 s | 6.99 s | 记录：进门 +1.2 s；缓存省下核判决 ≈ 2.7 s |
| `ce clone .`（暖索引，n=6 / 6 / 1） | 4.34–4.55 s（每跑判 2,155 对） | 1.59–1.64 s（2,155 对全回放） | 4.40 s（判 2,155 对） | ✅ −2.8 s；缓存空的一跑与不缓存同价 |
| 换二进制后的整库重建首跑（n=2 / 2） | 16.0–17.8 s | 17.2–20.3 s | — | 记录：一次性 T3 全判 +2–3 s |
| 缓存表 | — | 1,737 槽 / 2,155 可发对（`tree_key` 相同的单元对同一槽） | — | 记录 |
| 分相（临时探针，同日早些时候、有并发负载，实测后回退） | — | 候选 0.6 s / S5 扩展 0.08 s / 建树 0.35 s / 核 TED 2,155 对 3.1 s | — | 记录：册 02 引的就是这一行 |

读法：T3 进门让本仓每次 `ce check` 多付 ≈ 1.2 s（2,155 可发对、1,737 棵不同的树）——留下的是候选生成、S5 扩展、建树与回放，核那 2.7–3.1 s 被缓存拿掉；`ce trend` 每个点与 `ce join` 同付这 1.2 s。缓存空的一跑与 A 臂同价（4.40 vs 4.34–4.55 s），即记住判决本身不额外收费；第二跑起 `ce clone` 由 4.4 s 降到 1.6 s，且报告逐字节同（`cached` 2,155、`judged` 0）。**一个提醒**：同一天早些时候在有并发负载的窗口读到 `ce check` 暖 5.33 / 5.20 / 4.95 s、`ce clone` 暖 2.18 / 1.86 s——与本节差 20–30 %，按「量前量中查负载」的规矩作废，只留在这里说明为什么要重量。
复跑：`git worktree add <lane>/perf-5b9 2263995 && git -C <lane>/perf-5b9 submodule update --init`；A = 在该 worktree 里 `CARGO_TARGET_DIR=<lane>/perf-5b8-target cargo build --release`，B = 本树 `cargo build --release`；对同一 worktree 按块序 A B A B 各跑四次 `ce check --format json .` / `ce clone --format json .`，每跑前读 `pragma user_version` 与 `select count(*) from t3ted`，首跑（user_version 与上一跑不同）单列；缓存空 = 两条 DELETE 后再跑一次。

## v2.30 步 6 提交 E 新文法 release 体积（实测 2026-09-28，同一台机、同一工具链 rustc 1.94.1 / cargo 1.94.1，`cargo build --release --locked`，两棵树各自独立的 target 目录）

> 设计册 §11「性能」行要求新文法的 release 体积增量记进本册。A = v1.7.4 tag 的树（2377b3c，本机 worktree），B = 本树（提交 C 7020f41；`cli/src` 与 `core/app` 无未提交改动）。两者只差 v2.30 步 1–6：六套文法（tree-sitter-c 0.24.2 / cpp 0.23.4 / lua 0.5.0 / java 0.23.5 / r 1.3.0 / html 0.23.2）与驱动它们的 Rust 代码；Haskell 核不带文法，`ce-core` 不在本节。

| 制品 | A（v1.7.4） | B（本树） | 增量 |
|---|---|---|---|
| `ce.exe`（release） | 19,914,752 B | 25,662,976 B | +5,748,224 B（+28.9 %） |

六套新文法 crate 的 rlib（`target/release/deps`，B 独有）：cpp 3,775,158 · c 812,510 · r 660,996 · java 616,872 · lua 138,172 · html 49,982，合计 6,053,690 B——C++ 一套占六套的 62 %。两树共有的七个 crate（tree-sitter 运行时 1,281,050 与 go / haskell / python / rust / typescript / language）版本与 rlib 尺寸相同。新 Rust 代码（阶梯、LangSpec 表、可见性、提及）与文法表在二进制里的份额未分开量——分开得再编一版只带 crate 不带代码的树，本节不做；rlib 合计与增量同量级，文法表是大头，与 2026-09-24 探针二进制（运行时 + 含 Ruby 的七套文法 7.6 MB）的读法一致。

参照：Release v1.7.4 的 Windows 资产 `ce-1.7.4-x86_64-windows.exe` 18,548,736 B（Actions 矩阵的工具链），比本机同树的 A 小 1,366,016 B——跨工具链的读数只作界，A/B 才是同口径；1.8.0 发出后按资产复核一次（步 8）。

复现（本机；worktree 与两个 target 目录都在 `.worktrees` 车道下，`<lane>` 即该目录）：`git worktree add <lane>/wt-174 v1.7.4`，
`CARGO_TARGET_DIR=<lane>/rel-174 cargo build --release --locked --manifest-path <lane>/wt-174/cli/Cargo.toml`，
`CARGO_TARGET_DIR=<lane>/rel-head cargo build --release --locked --manifest-path cli/Cargo.toml`，
再读 `<lane>/rel-*/release/ce.exe` 与 `<lane>/rel-head/release/deps/libtree_sitter_*.rlib` 的字节数。

## v2.30 步 7b ③ 复杂度规则进核 A/B（实测 2026-09-29，release，同一台机、同一棵树、同一窗口：树 = c60e6a97 的干净 worktree〔子模块就位〕，A = c60e6a97 的 ce + 核〔三数在 Rust 走查器里算〕，B = 本批的 ce + 核〔Rust 只送事件流，核折三数〕；各臂预热一跑后 ABAB ×5，`bash` `EPOCHREALTIME` 夹整个进程、含进程起；量前无 cargo / 对拍进程）

口径：`ce scan --format json .` 与 `ce check --format json .` 暖跑（索引已建、核复用系统缓存），中位数（最小–最大）。两坐：首坐是核的事件契约首稿，第二坐在契约索引之后（见下）。

| 面 | A（旧，首坐） | B（首坐，契约首稿） | A（第二坐） | B（第二坐，契约索引后） | 状态 |
|---|---|---|---|---|---|
| `ce scan .` | 1.40 s（1.05–1.52） | 3.86 s（3.60–4.10） | 1.17 s（1.05–1.23） | 1.12 s（1.03–1.14） | ✅ 打平 |
| `ce check .` | 6.23 s（5.30–6.77） | 8.57 s（8.02–8.71） | 5.42 s（5.18–5.72） | 5.96 s（5.38–6.17） | 记录：+0.5 s 中位，在首坐 A 自己的离散之内 |

- 首坐的 +2.4 s 全在核：把自仓的真请求录下来（一个代核的记流壳，`CE_CORE_BIN` 指向它、它把每行转给真核；35,067 行 / 8,276 事件 / 2,139 弧 / 473 KB）单喂核 ×3——带事件表 2.60–2.65 s、去掉事件表 0.25–0.30 s。`CE.Scan.Events` 首稿的契约检查对每一行读邻位、对每个事件读所在行，都从列表头 `drop` 过去，按行数平方付钱。改为一次建 `IntMap` 索引（码按行下标）后同一请求 0.32–0.35 s，应答逐字节同，`cabal test` PASS。
- 事件路留下的固定成本 = 事件表的编码、核的解码与折叠：自仓请求核侧 0.32 − 0.26 ≈ 0.06 s；`ce scan` 两臂第二坐打平（差 0.05 s 在噪声内）。
- 复跑：两臂各自 `cargo build --release --locked` 到独立 target 目录（`--manifest-path <树>/cli/Cargo.toml --target-dir <lane>/…`），`CE_CORE_BIN` 各指自己的核；同一棵干净 worktree 里 `rm -rf .ce` 后各臂预热一跑，再 ABAB ×5 取中位。

## v2.31 步 5 PreToolUse flow 腿 A/B（实测 2026-09-30，release，同一台机、同一棵树内容、同一窗口：两份相同的树 = 本批 `cli/src` 的副本 364 个 `.rs`〔`git init`、各先 `ce dedup .` 建满索引，两臂各自一份免得两代 daemon 互相重建〕，A = 21d3aa28 的 ce〔无 flow 腿〕，B = 本批的 ce〔flow 腿在场，`[flow] tier` 不写 = observe〕，核同一个 ce-core 1.8.0；各臂预热两跑后 ABAB ×10，`python` `perf_counter` 夹 `subprocess.run`、含进程起；第一坐量时另两个车道在编译〔`Get-CimInstance` 80 → 65 %〕，第二坐 21 → 25 %）

口径：一次 `Write` 信封写 `src/mention/walk.rs`（299 行）并在末尾追加一段——`clean` = 一行注释（flow 腿两侧都降表、经 daemon 判，都无发现，不落 flow 行）、`finding` = 一个带不可达语句的新单元（flow 腿落一行，`novel` 1）；整个钩子进程的墙钟，中位数（最小–最大），毫秒。

| 面 | A（无 flow 腿） | B（flow 腿在场） | 状态 |
|---|---|---|---|
| `clean`，第一坐 | 86.1（81.7–113.0） | 97.2（90.6–113.3） | 记录：中位 +11.1 ms |
| `clean`，第二坐 | 84.8（82.6–132.7） | 95.3（91.4–157.1） | 记录：中位 +10.5 ms |
| `finding`，第二坐 | 89.7（84.6–145.2） | 100.3（96.5–110.3） | 记录：中位 +10.6 ms；B 的 flow 行 `{units: 25, before: 0, after: 1, novel: 1, judged: false}`（Rust 不在判决掩码里，只记不说） |

- 读法与任务书不同的一处：feed 探针行的 `elapsed_ms` 只计重复探针自己（`guard.rs::probed` 夹的是 `probe::novel_matches`），flow 腿不在里面——两臂二十跑的中位都是 18 ms、全部 `degraded: false`；flow 腿的代价只在整个进程的墙钟里看得见，故表读墙钟。
- flow 腿的固定成本 = 两侧各降一次表 + 两次 daemon 往返（核判 `flow/1`）：299 行的文件每次写入中位 +10.5–11.1 ms，三组读数一致；找到发现时多落一行 feed，不另加可见成本。
- 复跑：`cargo build --release` 各出一个二进制拷到车道目录；两份树各 `git init` + `ce dedup .`（daemon 的冷启动在第一次探针时还没盖完「全量已建」戳，探针会答 `degraded: true`——先用 CLI 建一次满索引）；各臂预热两跑后 ABAB ×10，`clean` 与 `finding` 各一坐。

## v2.32 步 5 Rust 半第二部分 十二族改印核的 `lines` A/B（实测 2026-10-03，release，同一台机、同一窗口：A = 7d69cf27〔R0〕的 ce + 它的核〔十一个面自己装控制台的行〕，B = 本车道的 ce + 它的核〔控制台面问同一次 `document/1`，应答多带 `lines`〕；车道 HEAD 归档〔测试子仓就位〕的两份拷贝，两臂各一份；各臂先冷建一次索引〔`dedup` / `scan` / `check`〕、每个面暖跑一次，再 ABAB ×7，`python` `perf_counter` 夹 `subprocess.run`、含进程起；控制台面，stdout 丢弃；坐前 `Get-CimInstance` 处理器负载 74 %、坐后 54 %〔别的会话〕）

口径：整个进程的墙钟，中位数（最小–最大），毫秒。

| 面 | A（Rust 打印） | B（核的 `lines`） | 状态 |
|---|---|---|---|
| `ce check .` | 12588.7（10662.2–15620.8） | 11812.8（10391.1–16271.7） | 噪声内 |
| `ce structure --deep .` | 4120.5（2821.3–4707.2） | 3948.2（3769.3–5160.9） | 噪声内 |
| `ce join --days 14 .` | 385085.5（233264.0–624391.5） | 390856.7（224614.7–407778.6） | 噪声内（墙钟是逐文件 blame） |
| `ce deadcode .` | 2814.4（2532.2–3413.5） | 2719.5（2438.1–3390.2） | 噪声内 |
| `ce graph --mentions .` | 2358.6（2280.2–2766.4） | 2318.6（2158.8–2656.0） | 噪声内 |
| `ce graph --sites .` | 1679.2（1594.4–1856.6） | 1660.8（1588.7–1774.8） | 噪声内 |
| `ce flow .` | 1082.6（1024.9–1203.0） | 1087.9（977.1–1253.8） | 噪声内 |
| `ce merge .` | 6248.6（5854.8–7469.5） | 6144.9（5785.8–10416.0） | 噪声内 |
| `ce arch .` | 2017.9（1404.1–2794.3） | 1507.5（1378.3–2737.2） | 噪声内（两臂区间重叠） |
| `ce query 'dead(F)' .` | 2330.8（2175.5–2953.7） | 2317.6（2186.2–2636.3） | 噪声内 |
| `ce rules .` | 2322.1（2229.5–2516.7） | 2324.3（2201.1–2515.7） | 噪声内 |

- B 不多问核：文档本来就由核装配（步 3B / 4B），控制台面只是同一次应答多读 `lines`；十一个面的差都落在两臂的抖动之内。
- 核的文档问答单独计时未做（任务书 §4 的「最大请求的核单问」）：十一个面的整进程差已在噪声内，单问只会更小。
- 复跑：A 臂 = 7d69cf27 的源码（`git archive` 导出到车道目录），`cabal build exe:ce-core` 与 `cargo build --release --locked`；B 臂终树；脚本 `v232_s5b_scratch/perf.py`（车道目录，不入库）。

## v2.32 步 5 车道 A 第二批四族改印核的 `lines` A/B（实测 2026-10-03，release，同一台机、同一窗口：A = 7d69cf27 的 ce + 核〔四族由 Rust 装配打印〕，B = 本批的 ce + 核〔erase / trend / similar 在判决那条核链上多一次 `document/1` 问答；`erase --log` 起一个核〕）

口径同第一批一节：整个进程的墙钟，毫秒，ABAB ×7，自仓带历史的拷贝（两臂各一份，各先暖跑两次，各带一份一千行的轨迹：第一批切换门那四行重复 250 次，250 行读不出）；similar 问 `cli/src/erase/model.rs:62`。

| 面 | A | B | 状态 |
|---|---|---|---|
| `ce erase .` | 中位 3649（3415–3796） | 中位 3575（3467–3691） | 噪声内 |
| `ce erase . --log` | 中位 44（41–51） | 中位 178（174–209） | +134：此前不起核，现在起一个核答文档（握手、目录加载与一千条记录的文档） |
| `ce trend . --commits 5`（五点全缓存） | 中位 372（352–412） | 中位 385（372–418） | 噪声内 |
| `ce similar . --at …` | 中位 1534（1429–1749） | 中位 1542（1463–1865） | 噪声内（另一窗口、负载更高时量） |

- 第一轮 erase 与 trend 为文档另起一个核：erase 中位 4343 → 4544、trend 411 → 563（同一负载下）；改为在判决那条核链上问后如上表。
- 复跑：`s5_scratch/g_abab.py erase erase-log trend similar`（车道外的一次性脚本，不入库）。

## v2.32 步 5 车道 A 第一批五族改印核的 `lines` A/B（实测 2026-10-03，release，同一台机、同一窗口：A = 7d69cf27 的 ce + 核〔五族由 Rust 装配打印〕，B = 本批的 ce + 核〔判决后在同一条核链上多一次 `document/1` 问答；dedup 不判时另起一个核〕）

口径：整个进程的墙钟，毫秒，`python` `perf_counter` 夹 `subprocess.run`、含进程起；语料 = 自仓带历史的拷贝（两臂各一份，各先暖跑两次），ABAB ×7；机器同时有别的会话在编译（负载约 80 %），读数是界不是常数。

| 面 | A | B | 状态 |
|---|---|---|---|
| `ce scan .` | 中位 1725（1564–2215） | 中位 1985（1766–2615） | +260：文档请求 278 KB、应答（文档与 `lines`）1.3 MB，核单测三次 329–360 ms、减去进程起（只答握手 128 ms）≈ 200 ms |
| `ce dedup .` | 中位 988（919–1280） | 中位 1099（1042–1259） | +111：不判时没有核链可复用，多起一个核 |
| `ce clone .` | 中位 4202（3983–7477） | 中位 4342（3866–8463） | 噪声内 |
| `ce docdup .` | 中位 3807（3441–4304） | 中位 3927（3608–4629） | +120：请求带全部活段（290 KB），核单测三次 252–320 ms、减去进程起 ≈ 130–190 ms |

- 第一轮把文档请求另起一个核时 scan 中位 +835 ms（同样负载下）；改为在判决那条核链上问（`document::assemble_over`），省掉一次进程起与目录加载。
- 复跑：`s5_scratch/g_abab.py scan dedup clone docdup`（车道外的一次性脚本，不入库）；核单测 = 用记录中继（`rec_relay.py`）录下真请求，对核单独计时。

## v2.32 步 5 Rust 半第一部分 precommit / commitmsg 空改动集 A/B（实测 2026-10-02，release，同一台机、同一坐：两份相同的树 = 车道 a12b4c75 的 `cli/src` 副本 395 个文件〔`git init`、提交一次、各用本臂的 ce 先 `ce dedup .` 建满索引〕，A = 分叉点 eec9dae4 的 ce + ce-core，B = 本车道的 ce + ce-core；各臂预热两跑后 ABAB ×7，`python` `perf_counter` 夹 `subprocess.run`、含进程起；`Get-CimInstance` 处理器负载坐前 37 %、坐后 14 %）

口径：暂存区为空（改动集空）时 `ce precommit` 与 `ce commitmsg <msg>`（消息一行 `perf: empty changeset`）整个进程的墙钟，中位数（最小–最大），毫秒；两臂每跑都退 0、首行逐字节同（`ce precommit: 0 staged file(s), net +0 LOC, no touched duplicates`，commitmsg 同句换面名）。

| 面 | A（分叉点） | B（本车道） | 状态 |
|---|---|---|---|
| `ce precommit` | 123.7（111.8–129.8） | 230.1（214.9–248.3） | 记录：中位 +106.4 ms |
| `ce commitmsg` | 200.6（191.2–213.9） | 311.6（299.7–338.3） | 记录：中位 +111.0 ms |

- 多出的这一笔是一次核的起动加一问：改动集空时审计不开核链（`audit.rs` 的 `gather` 只在 `changed` 非空时 `verdict::open`），而这一面要印的那一行改由核写出，`audit/speech.rs` 的 `spoken` 便自己开一条新链问 `audit`；分叉点在这里直接印本地句子、不起核。两面的增量一致（+106.4 / +111.0 ms）。
- 主会话裁定（2026-10-02，设计册 §13 第 62 条）：代价按此记账、不改；无核时 precommit 印兜底句而不印本地的暂存摘要，同样不改。
- 复跑：`cargo build --release` 出本车道的 ce，拷到车道目录外；分叉点的 ce 与 ce-core 各一份；两份树各 `git init` + 提交 + `ce dedup .`；各臂预热两跑后 ABAB ×7，precommit 与 commitmsg 各一组，跑前跑后各读一次处理器负载。

## v2.32 步 5 R0 churn 改印核的 `lines` A/B（实测 2026-10-02，release，同一台机、同一窗口：A = R0 父提交的 ce + 核〔Rust 与 cacc2741 只差 PROTO 一行，churn 由 Rust 装配打印〕，B = R0 的 ce + 核〔每次多一次 `document/1` 问答〕）

口径：整个进程的墙钟，毫秒，bash `EPOCHREALTIME` 夹进程、含进程起；语料 = crosscheck `rust` 提交成单提交仓（每臂一份拷贝），面 `ce churn --days 14 .`，ABAB ×7。

| 面 | A | B | 状态 |
|---|---|---|---|
| `ce churn --days 14`（rust 语料） | 中位 2158（2010–2548） | 中位 2143（2100–2491） | 噪声内 |
| `ce churn --days 14`（自仓带历史的拷贝，各一次，切换门顺带记下） | 474 s / 568 s / 489 s（en / json / zh） | 483 s / 464 s / 486 s | 噪声内；墙钟是 git 历史与逐文件 blame，两臂同一份测量代码 |

- B 多做的事 = 一次核问答（整数事实 + 配对表进、文档与 `lines` 出）与绑定；落在两臂的抖动之内。
- 复跑：`s5_scratch/r0_abab.sh`（车道外的一次性脚本，不入库）：两份语料拷贝、`CE_CORE_BIN` 每臂各指自己的核。

## v2.32 步 4B 七族文档改由核装配 A/B（实测 2026-10-02，release，同一台机、同一窗口：b3443723 自仓干净克隆〔测试子仓就位〕的两份拷贝，两臂各一份，A = b3443723 的 release ce〔`git archive` 构建，七族文档在 Rust 里装配〕，B = 终树的 ce〔判决后问 `document/1` 并绑定，文档沿用判决的核链〕，核同一个车道 ce-core 1.8.0〔proto 7.9.0〕；各臂先冷建一次索引〔`dedup` / `scan` / `check`〕、三个面各暖跑一次，再 ABAB ×7，`python` `perf_counter` 夹 `subprocess.run`、含进程起；`--format json`，stdout 丢弃）

口径：整个进程的墙钟，中位数（最小–最大），毫秒。

| 面 | A（Rust 装配） | B（核装配 + 绑定） | 差 | 状态 |
|---|---|---|---|---|
| `ce check .` | 7404.5（7249.9–16403.0） | 7603.2（7284.0–12661.9） | +198.7（+2.7 %） | 噪声内（第二坐 +22.9 ms，两坐区间互相覆盖） |
| `ce deadcode .` | 1984.5（1908.6–2668.3） | 1993.2（1944.6–2753.8） | +8.7 | 噪声内（两坐正负相反） |
| `ce structure --deep .` | 2030.8（1963.8–2144.5） | 2038.7（2019.4–2187.3） | +7.9 | 噪声内（第二坐负载升到 60 %，B 区间 2467.4–3747.5 覆盖 A 的 2323.4–2821.1） |

- 量时 `Get-CimInstance` 处理器负载：第一坐前 11 %、坐后 12 %；第二坐坐后 60 %（别的会话）。**第二坐**中位数 A / B：check 7223.1 / 7246.0、deadcode 2407.3 / 2381.0、structure --deep 2506.7 / 2663.0。
- **一个面一个核进程**：本步首稿的文档另起一条核链，那时连坐两次（处理器负载 97–100 %），deadcode 两坐都是 B 慢 （+777 / +426 ms），读作多起一个核进程；改为沿用判决的链（`document::Held`）后两坐差都在噪声内。那几坐的读数不入表。
- join / `graph --mentions` / `graph --sites` / 图屏未计时（切换门只比字节）。
- 复跑：A 臂 `git archive`（b3443723）解到车道目录、`cargo build --release --locked`（独立 `CARGO_TARGET_DIR`）；B 臂终树 `cargo build --release`；脚本 `s9_flow/v232s4_gen/perf.py`（车道目录，不入库）。

## v2.32 步 3B 文档改由核装配 A/B（实测 2026-10-01，release，同一台机、同一窗口：e877f389 干净树〔`git archive` + 测试子仓 74620180，去掉 `.gitmodules`〕的两份拷贝，两臂各一份，A = e877f389 的 release ce〔`git archive` 构建，步 1–3 之前的主线，五族文档在 Rust 里装配〕，B = 终树的 ce〔步 3A–3D 变基到 0a128885 后，判决后问 `document/1` 并绑定〕，核同一个 ce-core 1.8.0〔proto 7.8.0，终树构建〕；各臂先冷建一次索引〔`dedup` / `scan` / `check`〕、五个面各暖跑一次，再 ABAB ×7，`python` `perf_counter` 夹 `subprocess.run`、含进程起；`--format json`，stdout 丢弃；量时 `Get-CimInstance` 处理器负载 59–77 %〔别的会话的进程〕，故紧接着连坐两次）

口径：整个进程的墙钟，中位数（最小–最大），毫秒；表是第一坐，末列是第二坐的差。

| 面 | A（Rust 装配） | B（核装配 + 绑定） | 差 | 第二坐的差 | 状态 |
|---|---|---|---|---|---|
| `ce arch .` | 1798.2（1531.0–1996.8） | 1725.7（1582.1–1957.2） | −72.5 | +34.9 | 噪声内（两坐正负相反） |
| `ce flow .` | 2167.7（1952.2–2372.1） | 2088.5（1940.0–2191.2） | −79.2 | +77.6 | 噪声内（两坐正负相反） |
| `ce merge .` | 8690.2（8328.4–9616.4） | 9107.3（8819.9–9838.5） | +417.1（+4.8 %） | +408.3（+4.1 %） | 文档本身的代价，见下 |
| `ce query 'dead(F)' .` | 2390.8（2278.3–2422.9） | 2401.7（2293.1–2704.8） | +10.9 | +171.1 | 噪声内（第二坐两臂区间互相覆盖：A 2243.9–2978.8、B 2297.1–2775.1） |
| `ce rules .` | 2572.5（2235.5–2623.2） | 2391.4（2246.3–2418.1） | −181.1 | −178.4 | 两坐都是 B 快、区间互相覆盖，未定因（两臂都退 1：这棵树的 `ce.rules` 有违规） |

- **第二坐**中位数 A / B：arch 1472.5 / 1507.4、flow 1946.6 / 2024.2、merge 9893.0 / 10301.3、query 2315.1 / 2486.2、rules 2594.2 / 2415.8。
- **一个面一个核进程**：文档沿用判决用过的那条核链（`document::Held`：判决请求失败的链作废，文档时刻另起一条），绑定与读者不做整份克隆（`Value::take`、`Report::deserialize(&doc)`）；这两处是变基前的坐次里定下的，那几坐的读数不入册。
- **merge 多出的约 410 ms 是文档本身**：终树 merge 文档 3056 组、6147 个成员、34,529 个洞，`document.request` 857,019 B（43,732 行）、应答 2,815,738 B。经中继核把这条请求录下、单问核五次：hello 44–58 ms，文档 326–387 ms（核读请求、校验、装配、写应答）；其余是 Rust 写请求、读应答、绑定。旧路在 Rust 里直接拼同一份 JSON，没有这一来一回。arch / flow 的请求小（arch 8,877 行 / 98,721 B、flow 1,923 行 / 17,896 B）。
- 复跑：A 臂的树由 `git archive`（e877f389）解到车道目录、`cargo build --release --locked`（独立 `CARGO_TARGET_DIR`）；B 臂终树 `cargo build --release --locked`；树 = `git archive`（e877f389）加子仓、删 `.gitmodules`；脚本 `s9_flow/v232s3_gen/perf.py`、录请求的中继 `dump_core.py` 与单问核的 `core_alone.py`（车道目录，不入库）。

## v2.33 W2-text 阶段 A `resolve/1` 以文本过线 A/B（实测 2026-10-04，release，同一台机、同一坐：A = 92e728b1 的 ce + 它的核〔`git archive` 构建〕，B = 车道终树 1af49a44 的 ce + 它的核〔路径、说明符与 Python / Go / C·C++ 的配置原文过线，配置读法在核〕；每臂每棵树各一份拷贝〔`.ce` 删掉，koreader 拷贝去掉未就位的 `.gitmodules`〕；冷 = 跑前删 `.ce`，暖 = 有 `.ce` 之后的一跑；每个（树、面、冷暖）ABAB ×7，`python` `perf_counter` 夹 `subprocess.run`、含进程起，stdout 丢弃；坐时每分钟 `Get-CimInstance` 处理器负载 6–62 %，含本坐自己的 ce）

口径：整个进程的墙钟，中位数（最小–最大），毫秒。树：本车道 HEAD 的归档（测试子仓就位）与每个阶梯语言最大的真树（requests = Python、koreader = Lua、cobra = Go、lua = C、fmt = C++）。`check` 只量暖（任务书：暖 `ce check` 的预算）。

| 树 | 面 | A（92e728b1） | B（车道） | B / A − 1 |
|---|---|---|---|---|
| self | `graph --sites` 冷 | 1733.5（1637.8–1827.4） | 1763.1（1679.2–1867.5） | +1.7 % |
| self | `graph --sites` 暖 | 1646.4（1605.7–1848.6） | 1651.3（1612.4–1947.2） | +0.3 % |
| self | `deadcode` 冷 | 32301.0（29389.2–33837.5） | 31818.9（31008.6–34623.8） | -1.5 % |
| self | `deadcode` 暖 | 2327.7（2164.0–2563.0） | 2212.2（2158.7–2370.3） | -5.0 % |
| self | `check` 暖 | 9930.1（9875.9–15671.3） | 9971.7（9365.7–15840.0） | +0.4 % |
| requests | `graph --sites` 冷 | 370.1（366.7–489.6） | 374.0（366.8–474.8） | +1.1 % |
| requests | `graph --sites` 暖 | 260.3（251.2–293.0） | 280.5（258.3–315.0） | +7.8 % |
| requests | `deadcode` 冷 | 1395.8（1365.0–1535.0） | 1395.8（1365.0–1460.5） | +0.0 % |
| requests | `deadcode` 暖 | 265.0（254.3–283.5） | 266.2（254.7–288.6） | +0.5 % |
| requests | `check` 暖 | 1010.6（943.9–1858.7） | 1028.7（983.5–1835.9） | +1.8 % |
| koreader | `graph --sites` 冷 | 2725.1（2675.7–3032.3） | 2798.8（2621.3–3005.8） | +2.7 % |
| koreader | `graph --sites` 暖 | 2634.8（2561.7–2718.6） | 2656.9（2551.6–2916.3） | +0.8 % |
| koreader | `deadcode` 冷 | 26950.0（25988.8–29421.6） | 27686.8（25929.7–30270.3） | +2.7 % |
| koreader | `deadcode` 暖 | 1515.8（1500.9–1647.3） | 1543.9（1508.1–1709.0） | +1.9 % |
| koreader | `check` 暖 | 11035.0（9750.4–91247.4） | 10613.2（10248.6–99978.4） | -3.8 % |
| cobra | `graph --sites` 冷 | 355.8（352.9–383.8） | 363.3（359.5–428.5） | +2.1 % |
| cobra | `graph --sites` 暖 | 254.1（249.3–283.3） | 258.1（254.6–283.7） | +1.6 % |
| cobra | `deadcode` 冷 | 1383.6（1340.8–1543.9） | 1418.0（1307.3–1573.7） | +2.5 % |
| cobra | `deadcode` 暖 | 271.9（251.4–360.1） | 276.9（252.3–295.7） | +1.8 % |
| cobra | `check` 暖 | 1057.3（1020.1–3854.7） | 1118.3（1095.7–3749.7） | +5.8 % |
| lua | `graph --sites` 冷 | 688.7（649.0–752.3） | 672.8（651.9–709.4） | -2.3 % |
| lua | `graph --sites` 暖 | 581.5（563.4–600.5） | 584.0（567.0–629.5） | +0.4 % |
| lua | `deadcode` 冷 | 3802.7（3638.7–4258.8） | 3671.1（3619.4–4002.3） | -3.5 % |
| lua | `deadcode` 暖 | 451.8（437.9–477.7） | 445.3（443.9–503.2） | -1.4 % |
| lua | `check` 暖 | 1996.4（1966.5–3779.4） | 2035.8（1983.1–3832.2） | +2.0 % |
| fmt | `graph --sites` 冷 | 1024.3（1013.7–1175.7） | 1068.8（1026.8–1168.7） | +4.3 % |
| fmt | `graph --sites` 暖 | 971.7（908.2–1022.2） | 937.9（911.9–953.0） | -3.5 % |
| fmt | `deadcode` 冷 | 7681.5（6510.0–8880.8） | 7574.6（7211.6–8649.0） | -1.4 % |
| fmt | `deadcode` 暖 | 692.6（659.8–814.4） | 718.8（684.9–878.3） | +3.8 % |
| fmt | `check` 暖 | 4640.5（4282.3–8493.3） | 4980.6（4359.4–8217.8） | +7.3 % |

- 预算（任务书）：暖 `ce check` ≤ +15 %——六棵树 −3.8 % 到 +7.3 %（fmt 最高），都在线内；其余各面 −5.0 % 到 +7.8 %（requests 暖 `graph --sites`，260 → 281 ms，两臂区间 251–293 / 258–315 ms 互相覆盖）。
- 每棵树 `check` 的头一对（一冷 `deadcode` 之后的第一次 `check`，要填 T3 判决缓存）两臂同样慢：self 15,671 / 15,840、koreader 91,247 / 99,978 ms，进了最大值、不进中位数。
- 自仓 `check` 两臂都退 1：归档树里基线尚未按本车道重立，与本表无关；其余各跑全退 0。
- **请求字节**（§3 第 5 点；同一棵树一次冷 `deadcode`，经中继核 `relay_core.py` 记下每个 `resolve.request` 与应答的字节，两臂 `deadcode` 输出逐字节同）：

| 树 | 请求 A → B（B） | 应答 A → B（B） |
|---|---|---|
| self | 22,585 → 43,578 | 285 → 498 |
| requests | 8,016 → 10,922 | 3,969 → 7,205 |
| koreader | 151,000 → 178,851 | 63,804 → 188,081 |
| cobra | 6,236 → 5,384 | 2,407 → 2,805 |
| lua | 20,914 → 22,768 | 7,731 → 11,609 |
| fmt | 26,537 → 33,041 | 9,365 → 13,485 |

  请求变大是路径与说明符以文本过线（段 id 驻留表与 `vocab` 删掉之后，同一个目录名在每条路径里重复出现）；cobra 变小是 go.mod 的原文比 W2a 的逐模块事实行短。应答变大是目标改为路径（koreader 的站点多、路径长）。
- **钩子**：PreToolUse 探针不问 `resolve/1`——一个写入 Python 文件（带两条 import）的 `Write` 信封经 `ce probe --hook` 在 requests 拷贝上冷跑、暖跑各一次，返回时中继核记下的 `resolve.request` 都是 0；之后 daemon 后台的冷启分析送过一次（10,981 B），与 W2a 的「钩子路径不变」（设计册 §11 第 19 条）同。
- 复跑：车道目录 `v233_w2t_scratch/perf.py 7`（树拷自 `.ce-eval/corpora` 与车道归档），读数原文 `perf.log`、逐跑 `perf.ndjson`、负载 `perf_load.txt`；请求字节 `reqbytes.sh`，钩子 `hookprobe.sh`。

## v2.33 W2-text 阶段 B R 阶梯与 `DESCRIPTION` 读法进核 A/B（实测 2026-10-04，release，同一台机、同一坐：A = c96ab3f6 的 ce + 它的核〔`git archive` 构建〕，B = 车道树 74768b93 的 ce + 它的核〔R 的两级阶梯、`DESCRIPTION` 读法与包代码展开在核，`DESCRIPTION` 原文过线〕；每臂每棵树各一份拷贝〔`.ce` 删掉〕；冷 = 跑前删 `.ce`，暖 = 有 `.ce` 之后的一跑；每个（树、面、冷暖）ABAB ×7，`python` `perf_counter` 夹 `subprocess.run`、含进程起，stdout 丢弃；本坐没有记处理器负载）

口径：整个进程的墙钟，中位数（最小–最大），毫秒。树：本车道 HEAD 的归档（测试子仓就位）与两份 R 语料（covid19model 是最大的 R 树、`DESCRIPTION` 在子目录 `covid19AgeModel/`；stringr 是仓根即包）。

| 树 | 面 | A（c96ab3f6） | B（车道） | B / A − 1 |
|---|---|---|---|---|
| self | `graph --sites` 冷 | 1795.8（1672.0–1897.8） | 1745.3（1692.9–2075.4） | -2.8 % |
| self | `graph --sites` 暖 | 1592.1（1549.3–1625.8） | 1589.2（1545.4–1646.1） | -0.2 % |
| self | `deadcode` 冷 | 34496.5（31500.4–36642.6） | 30969.5（30778.5–37980.7） | -10.2 % |
| self | `deadcode` 暖 | 2439.0（2383.5–2566.0） | 2513.4（2363.9–2641.5） | +3.1 % |
| self | `check` 暖 | 10285.9（9817.3–16506.4） | 10594.8（10061.7–15350.9） | +3.0 % |
| covid19model | `graph --sites` 冷 | 1021.2（986.1–1103.1） | 1033.2（1000.4–1094.7） | +1.2 % |
| covid19model | `graph --sites` 暖 | 959.0（948.5–1011.3） | 970.7（909.1–1074.7） | +1.2 % |
| covid19model | `deadcode` 冷 | 8348.7（7753.5–8431.8） | 7941.9（7732.0–8583.9） | -4.9 % |
| covid19model | `deadcode` 暖 | 791.0（751.2–869.1） | 901.8（851.2–929.4） | +14.0 % |
| covid19model | `check` 暖 | 2658.0（2563.7–4619.0） | 2835.0（2714.6–4842.9） | +6.7 % |
| stringr | `graph --sites` 冷 | 339.8（307.7–421.4） | 345.2（331.2–382.6） | +1.6 % |
| stringr | `graph --sites` 暖 | 233.4（215.5–259.5） | 220.4（211.1–235.3） | -5.6 % |
| stringr | `deadcode` 冷 | 900.0（861.6–963.9） | 1021.1（956.3–1052.9） | +13.5 % |
| stringr | `deadcode` 暖 | 254.4（228.2–279.0） | 351.4（326.6–361.7） | +38.1 % |
| stringr | `check` 暖 | 841.2（778.6–1392.1） | 946.4（864.2–1536.4） | +12.5 % |

- 预算（任务书）：暖 `ce check` ≤ +15 %——三棵树 +3.0 %、+6.7 %、+12.5 %（stringr，841 → 946 ms），都在线内。
- **超过 +15 % 的面只有一个**：stringr 暖 `deadcode` +38.1 %（254 → 351 ms，+97 ms；B 的最小值 326.6 高于 A 的最大值 279.0，不是噪声）。同一机制让 covid19model 暖 `deadcode` +14.0 %（+111 ms）、stringr 冷 `deadcode` +13.5 %（+121 ms），也是两棵 R 树暖 `check` 多出的那部分（+105 / +177 ms）。原因已按请求核实（`warmreq_b.sh`：每臂在建好索引的拷贝上经中继核再跑一次暖 `deadcode`，数 `resolve.request`）：A 臂暖跑一个 `resolve/1` 请求也不发——包代码在 Rust 里算，图边从索引读；B 臂在有 `DESCRIPTION` 的树上每跑都要问核一次包代码（`Declared::gather` → `resolve::packages`，stringr 一个请求 3,552 B / 应答 642 B，covid19model 11,520 / 667 B），于是暖跑多起一个 `ce-core` 进程、多一次握手和一次问答，约 100 ms；自仓没有 `DESCRIPTION`，B 臂暖跑同样 0 个请求，`deadcode` 暖 +3.1 %。绝对量是每跑约 0.1 s 的定额，不随树变大；没做缓存（把包代码存进索引要动索引 schema），记为未解决项。
- 自仓 `check` 两臂都退 1：归档树里基线尚未按本车道重立，与本表无关；其余各跑全退 0。
- **请求字节**（§3 第 5 点；同一棵树一次冷 `deadcode`，经中继核记下每个 `resolve.request` 与应答的字节；两臂 `deadcode` 输出逐字节同）：

| 树 | 请求个数 A → B | 请求字节 A → B | 应答字节 A → B |
|---|---|---|---|
| self | 1 → 1 | 43,881 → 43,905 | 498 → 512 |
| covid19model | 1 → 2 | 10,198 → 61,985 | 281 → 27,683 |
| stringr | 0 → 2 | 0 → 8,340 | 0 → 2,053 |

  A 臂在 R 树上只为别的语言问核（covid19model 9 个非 R 站点，stringr 一个也没有）；B 臂把 1,697 / 55 个 R 站点连同 `DESCRIPTION` 原文送进图的请求（50,465 / 4,788 B），声明目标那一侧另送一次（11,520 / 3,552 B，只带文件表与 `DESCRIPTION`）。应答变大是 R 站点的目标改为路径。自仓多 24 B：请求多一个空的 `r.descriptions` 键，应答多一个空的 `packages` 表。
- **钩子**：PreToolUse 探针不问 `resolve/1`——一个写入 R 文件（带一条 `library` 与一条 `source`）的 `Write` 信封经 `ce probe --hook` 在 covid19model 拷贝上冷跑、暖跑各一次，中继核记下的 `resolve.request` 都是 0（`hookprobe_b.sh`），与阶段 A 同。
- 复跑：车道目录 `v233_w2t_scratch/perf_b.py 7`（树拷自 `.ce-eval/corpora` 与车道归档），读数原文 `perf_b.log`、逐跑 `perf_b.ndjson`；请求字节 `reqbytes_b.sh`，暖跑请求 `warmreq_b.sh`，钩子 `hookprobe_b.sh`。

## v2.33 W2-text 阶段 C Java 阶梯进核 A/B（实测 2026-10-05，release，同一台机、同一坐：A = 27d0d56d 的 ce + 它的核〔`git archive` 构建〕，B = 车道树 5a7fbe7f 的 ce + 它的核〔Java 的四级阶梯与类型注解读法在核，每个走查到的 Java 文件头随请求过线〕；每臂每棵树各一份拷贝〔`.ce` 删掉〕；冷 = 跑前删 `.ce`，暖 = 有 `.ce` 之后的一跑；每个（树、面、冷暖）ABAB ×7，`python` `perf_counter` 夹 `subprocess.run`、含进程起，stdout 丢弃；坐时每分钟 `Get-CimInstance` 处理器负载，03:53–04:16 共 23 次 18–85 %，含本坐自己的 ce）

口径：整个进程的墙钟，中位数（最小–最大），毫秒。树：本车道 HEAD 的归档（测试子仓就位）与两份 Java 语料 gson、jsoup。

| 树 | 面 | A（27d0d56d） | B（车道） | B / A − 1 |
|---|---|---|---|---|
| self | `graph --sites` 冷 | 1985.7（1782.9–2122.9） | 1847.0（1764.4–2166.5） | -7.0 % |
| self | `graph --sites` 暖 | 1801.5（1620.9–2017.9） | 1831.6（1662.2–2085.2） | +1.7 % |
| self | `deadcode` 冷 | 35563.0（33808.1–39092.1） | 35352.3（31986.6–38531.5） | -0.6 % |
| self | `deadcode` 暖 | 2524.7（2241.1–2939.6） | 2441.6（2284.5–2820.1） | -3.3 % |
| self | `check` 暖 | 11976.2（10954.6–18259.7） | 11876.2（11408.6–19046.0） | -0.8 % |
| gson | `graph --sites` 冷 | 1230.5（1186.9–1654.2） | 1263.5（1182.1–1410.0） | +2.7 % |
| gson | `graph --sites` 暖 | 1198.5（1040.6–1304.7） | 1228.4（1133.9–1282.5） | +2.5 % |
| gson | `deadcode` 冷 | 7054.9（6911.7–7641.6） | 7119.9（6859.9–8128.3） | +0.9 % |
| gson | `deadcode` 暖 | 887.3（825.2–991.2） | 878.5（825.3–961.5） | -1.0 % |
| gson | `check` 暖 | 3306.8（3224.4–21664.3） | 3330.2（3229.3–20828.6） | +0.7 % |
| jsoup | `graph --sites` 冷 | 1388.3（1289.6–1502.4） | 1369.1（1263.4–1412.7） | -1.4 % |
| jsoup | `graph --sites` 暖 | 1231.1（1187.7–1339.5） | 1225.8（1154.9–1309.0） | -0.4 % |
| jsoup | `deadcode` 冷 | 7461.4（7279.9–7871.8） | 7396.1（7259.0–7667.4） | -0.9 % |
| jsoup | `deadcode` 暖 | 756.1（700.4–863.1） | 746.5（721.1–902.6） | -1.3 % |
| jsoup | `check` 暖 | 4967.9（4543.5–101720.3） | 4834.1（4581.1–98971.7） | -2.7 % |

- 预算（任务书）：暖 `ce check` ≤ +15 %——三棵树 -0.8 %、+0.7 %、-2.7 %，都在线内；没有一个面超过 +15 %。`check` 暖的最大值（gson 约 21 s、jsoup 约 100 s）是每臂第一次 `check`：近似克隆缓存在那一跑填满，两臂同形。
- **第一版的核超线，已修**：同一坐法量本车道 52de63a1 的核（用同一个 ce），gson 冷 `deadcode` +20.4 %（7,229 → 8,703 ms）、jsoup +15.0 %（8,248 → 9,489 ms）。把那一跑经中继核录下的会话单独喂核（`relay_dump.py` + `core_time_c.py`），gson 2.0 s、jsoup 1.38 s；把 Java 文件头清空再喂 0.19 s，把站点清空 0.08 s——时间花在阶梯，不在解码。原因是每个站点每问一次包里的文件，都把每个候选文件的路径重新切段、找一遍标准源集（`sourceRoot` 还是按下标取列表元素），再逐个查头部看它声明了哪些类。改为每次请求在 `javaEnv` 里只建一次：每个文件「在不在 `main` 以外的源集」那一位、包索引、（包、类）索引；`sourceRoot` 改为一次线性扫描。之后同一份会话单独喂核 gson 0.34 s、jsoup 0.33 s（无负载时；本表那一坐里重量为 0.53 / 0.57 s），上表即修后的读数。修前修后两版核在四条阶梯腿 × 三颗种子各 12,000 个站点与 47,257 个真 Java 站点上与冻结的 Rust 阶梯不一致 0，`cabal test` 962 ok。
- **多出核请求的面**（`warmreq_c.sh`：每臂在一份拷贝上冷跑一次 `graph --sites` 建索引，再经中继核把 `graph --sites` / `deadcode` / `structure` / `check` / `join --days 14` / `arch` / `erase` / `rules` 各跑一次，数每个核会话与每种请求）：只有在 Java 树上第一次建图边的那一跑多一个会话、一个 `resolve.request`——gson、jsoup 上就是那次 `deadcode`（A 臂 0 个，Java 在 Rust 里答）；之后的暖跑图边读自索引，两臂请求逐种相同。自仓同一跑两臂都是 1 个请求、11 个站点，多 22 B（空的 `java.headers`）。
- 自仓 `check` 两臂都退 1：归档树里基线尚未按本车道重立，与本表无关；其余各跑全退 0。
- **请求字节**（§3 第 5 点；同一棵树一次冷 `deadcode`，经中继核记下每个 `resolve.request` 与应答的字节；两臂 `deadcode` 输出逐字节同）：

| 树 | 请求个数 A → B | 请求字节 A → B | 应答字节 A → B |
|---|---|---|---|
| self | 1 → 1 | 44,398 → 44,420 | 512 → 512 |
| gson | 0 → 1 | 0 → 885,394 | 0 → 748,188 |
| jsoup | 0 → 1 | 0 → 820,387 | 0 → 849,885 |

  B 臂把 22,698 / 24,210 个 Java 站点（每个带行号）连同每个走查到的 Java 文件头（`java.headers` 183,303 / 120,612 B：包、import、类型与成员、首末行）送进一次请求；A 臂在这两棵树上一个请求也不发。
- **钩子**：PreToolUse 探针不问 `resolve/1`——一个在 gson 的 main 源集里写入新 Java 文件（两条 import、一个类型引用）的 `Write` 信封经 `ce probe --hook` 在 gson 拷贝上冷跑、暖跑各一次，中继核记下的 `resolve.request` 都是 0（`hookprobe_c.sh`），与阶段 A、B 同。
- 复跑：车道目录 `v233_w2t_scratch/perf_c.py 7`（树拷自 `.ce-eval/corpora` 与车道归档），读数原文 `perf_c.log`、逐跑 `perf_c.ndjson`、负载 `perf_c_load.txt`；请求字节 `reqbytes_c.sh`，各面请求 `warmreq_c.sh`，钩子 `hookprobe_c.sh`，核单独计时 `coretime_c.sh` / `core_split_c.py`。

## v2.33 W2-text 阶段 D Haskell 阶梯与 `.cabal` 读法进核 A/B（实测 2026-10-05，release，同一台机、同一坐：A = fa83a48d 的 ce + 它的核〔`git archive` 构建〕，B = 车道树 9c1f7668 的 ce〔sha256 a6836d5c…，构建进车道的 scratch target〕+ 它的核〔Haskell 的三级阶梯、`.cabal` 读法、入口与包内私有在核，每个 `.cabal` 的原文随请求过线〕；每臂每棵树各一份拷贝〔`.ce` 删掉〕；冷 = 跑前删 `.ce`，暖 = 有 `.ce` 之后的一跑；每个（树、面、冷暖）ABAB ×7，`python` `perf_counter` 夹 `subprocess.run`、含进程起，stdout 丢弃；坐时每分钟 `Get-CimInstance` 处理器负载，07:59–08:17 共 18 次 2–55 %，含本坐自己的 ce）

口径：整个进程的墙钟，中位数（最小–最大），毫秒。树：本车道 HEAD 的归档（测试子仓就位）与两棵 Haskell 树 dataframe@96ec374、keel@71ed44a（各提交一次）。

| 树 | 面 | A（fa83a48d） | B（车道） | B / A − 1 |
|---|---|---|---|---|
| self | `graph --sites` 冷 | 1699.0（1636.5–1718.2） | 1709.3（1680.5–1739.7） | +0.6 % |
| self | `graph --sites` 暖 | 1614.6（1566.4–2072.2） | 1674.8（1611.3–1705.6） | +3.7 % |
| self | `deadcode` 冷 | 29366.2（29133.6–29977.0） | 29389.6（29105.3–30388.9） | +0.1 % |
| self | `deadcode` 暖 | 2165.6（2145.2–2402.6） | 2300.7（2271.4–2384.7） | +6.2 % |
| self | `check` 暖 | 9814.0（9634.0–15661.9） | 9793.4（9538.6–15865.2） | -0.2 % |
| dataframe | `graph --sites` 冷 | 1472.4（1432.9–1496.8） | 1496.0（1455.2–1531.7） | +1.6 % |
| dataframe | `graph --sites` 暖 | 1382.9（1349.2–1427.0） | 1405.0（1369.6–1460.2） | +1.6 % |
| dataframe | `deadcode` 冷 | 11753.1（11625.7–11923.6） | 11299.7（11238.4–12248.0） | -3.9 % |
| dataframe | `deadcode` 暖 | 1226.0（1179.1–1279.3） | 1353.1（1307.3–1414.7） | +10.4 % |
| dataframe | `check` 暖 | 5447.5（5367.3–40465.2） | 5685.5（5563.8–40402.6） | +4.4 % |
| keel | `graph --sites` 冷 | 359.9（353.5–398.1） | 365.9（353.9–371.5） | +1.7 % |
| keel | `graph --sites` 暖 | 254.9（250.1–285.9） | 263.2（254.5–289.6） | +3.3 % |
| keel | `deadcode` 冷 | 1134.8（1108.0–1172.3） | 1173.3（1155.3–1202.8） | +3.4 % |
| keel | `deadcode` 暖 | 276.5（260.4–296.7） | 399.8（360.2–417.2） | +44.6 % |
| keel | `check` 暖 | 1181.7（1143.3–1463.2） | 1266.3（1225.1–1514.9） | +7.2 % |

- 预算（任务书）：暖 `ce check` ≤ +15 %——三棵树 -0.2 %、+4.4 %、+7.2 %，都在线内。
- **超线的面：暖 `deadcode`**——keel +44.6 %（276.5 → 399.8 ms，+123 ms）、dataframe +10.4 %、self +6.2 %。原因按请求点名（`warm2_d.sh`：每臂冷跑一次 `graph --sites` 与一次 `deadcode`，再经中继核跑第二次 `deadcode` 与 `structure`）：B 臂的暖 `deadcode` 多一个核会话（2 → 3）、两个 `resolve.request`——一个是 mounts 表位 1 的包内私有（keel 15,778 B、self 60,050 B），一个是声明目标的入口（keel 16,929 B、self 44,987 B，`DESCRIPTION` 与 `.cabal` 同一次请求）；A 臂两者都在 Rust 里答、一个请求也不发。小树上多起的那一个 ce-core 进程（连同它的 hello 与定义包的读入）就是增量的大头（keel 上那两个请求共 32,707 B；本节没有把它们与进程起动分开计时）。收法是让一个命令只开一个核会话——`resolve::ask` 的链接与图族的链接共用一个——这是本轨收尾阶段（车道日志 Stage Z 第 Z1 条，阶段 B 审阅时立）的事，本阶段不做。
- **多出核请求的面**（`warmreq_d.sh`：每臂冷跑一次 `graph --sites` 建索引，再经中继核把八个面各跑一次）：`structure` / `check` / `join --days 14` / `arch` / `erase` / `rules` 在三棵树上都多一个会话、一个 `resolve.request`（mounts 表的包内私有：self 60,050 B、dataframe 108,048 B、keel 15,778 B，其中 `hs` 15,739 / 90,236 / 13,537 B）；`deadcode` 在这一跑是第一次建图边，B 臂 3 个请求（全部 Haskell 站点 + 入口 + 私有）、A 臂 1 个；`graph --sites` 与冷建两臂请求逐种相同。
- 自仓 `check` 两臂都退 1：归档树里基线尚未按本车道重立，与本表无关；其余各跑全退 0。
- **请求字节**（§3 第 5 点；同一棵树一次冷 `deadcode`，经中继核记下每个 `resolve.request` 与应答的字节；两臂 `deadcode` 输出逐字节同）：

| 树 | 请求个数 A → B | 请求字节 A → B | 应答字节 A → B | 站点 A → B |
|---|---|---|---|---|
| self | 1 → 3 | 44,749 → 218,542 | 512 → 63,866 | 11 → 1,957 |
| dataframe | 1 → 3 | 19,947 → 466,163 | 1,170 → 131,293 | 71 → 3,731 |
| keel | 1 → 3 | 2,734 → 54,948 | 231 → 5,901 | 4 → 230 |

- **核单独计时**（`coretime_d.sh`：把 B 臂那次冷 `deadcode` 的 `resolve/1` 会话经中继录下，单独喂核 5 次）：self 一个会话三个请求共 218,607 B → 中位 105.0 ms（103.8–111.8），dataframe 共 466,228 B（最大的一个请求 224,533 B）→ 162.5 ms（159.9–193.0）；请求字节翻 2.1 倍、时间 1.5 倍，没有超线性的迹象。
- **钩子**：PreToolUse 探针不问 `resolve/1`——一个在 dataframe 的库源根下写入新 Haskell 模块（包内模块、boot 包模块、带包名的 import 各一条）的 `Write` 信封经 `ce probe --hook` 在 dataframe 拷贝上冷跑、暖跑各一次，中继核记下的 `resolve.request` 都是 0（`hookprobe_d.sh`），与阶段 A、B、C 同。
- 复跑：车道目录 `v233_w2t_scratch/perf_d.py 7`（树拷自车道归档与 `gate1d/seed-hs-*`），读数原文 `perf_d.log`、逐跑 `perf_d.ndjson`、负载 `perf_d_load.txt`；请求字节 `reqbytes_d.sh`，各面请求 `warmreq_d.sh` / `warm2_d.sh`，钩子 `hookprobe_d.sh`，核单独计时 `coretime_d.sh`。

## v2.33 W2-text 阶段 E TS 阶梯、tsconfig 链、package.json 与 JSONC 读法进核 A/B（实测 2026-10-05，release，同一台机、同一坐：A = dd0eec61 的 ce + 它的核〔`git archive` 构建〕，B = 车道树 a51fbe7f 的 ce〔sha256 ea83acfb…，构建进车道的 scratch target；a51fbe7f 与落地的第一个提交 097e5ba3 判决代码相同，只差两个不判决的文件：`cli/src/graph/roots.rs`（测试挂载处的一行导入与注释）与测试子仓 `unit/graph/oracle_cfg/ts_package.rs`（冻结读法的挂载路径）〕+ 它的核〔sha256 9e2adb5b…；TS 的五级阶梯、extends 链、package.json 与 JSONC 读法在核，测量侧只读文件、问盘〕；每臂每棵树各一份拷贝〔`.ce` 删掉〕；冷 = 跑前删 `.ce`，暖 = 有 `.ce` 之后的一跑；每个（树、面、冷暖）ABAB ×7，`python` `perf_counter` 夹 `subprocess.run`、含进程起，stdout 丢弃；坐时每分钟 `Get-CimInstance` 处理器负载，15:43–16:48 共 64 次 31–100 %——别的会话的一个 `python` 进程一直占着，本坐不是静默窗）

口径：整个进程的墙钟，中位数（最小–最大），毫秒。树：本车道 HEAD 的归档（测试子仓就位）与两棵 TS 树 zod@912f0f5、careeros（`gate1e/seed-ts-*`，各提交一次）。

| 树 | 面 | A（dd0eec61） | B（车道） | B / A − 1 |
|---|---|---|---|---|
| self | `graph --sites` 冷 | 2219.4（1982.1–2751.3） | 2312.1（2038.7–2399.9） | +4.2 % |
| self | `graph --sites` 暖 | 2175.3（2113.6–2545.2） | 2304.9（1934.9–2563.4） | +6.0 % |
| self | `deadcode` 冷 | 40120.5（33241.2–43467.5） | 40509.2（35978.1–41606.2） | +1.0 % |
| self | `deadcode` 暖 | 3016.1（2694.5–6722.1） | 3079.8（2652.1–3714.2） | +2.1 % |
| self | `check` 暖 | 14738.9（11642.0–21014.2） | 14512.4（12791.6–20216.8） | -1.5 % |
| zod | `graph --sites` 冷 | 1373.3（1317.9–1697.2） | 1515.1（1302.2–1648.8） | +10.3 % |
| zod | `graph --sites` 暖 | 1414.6（1280.8–1453.7） | 1359.7（1255.9–1540.9） | -3.9 % |
| zod | `deadcode` 冷 | 10737.1（10085.6–12174.8） | 11162.9（9467.8–14013.1） | +4.0 % |
| zod | `deadcode` 暖 | 819.7（748.1–882.2） | 899.7（834.1–1028.1） | +9.8 % |
| zod | `check` 暖 | 7384.1（7007.7–82406.1） | 8076.7（6644.9–71861.2） | +9.4 % |
| careeros | `graph --sites` 冷 | 10511.7（8347.9–12189.7） | 10882.8（9828.1–11260.0） | +3.5 % |
| careeros | `graph --sites` 暖 | 9490.0（8759.6–10077.3） | 9188.0（8589.4–10917.7） | -3.2 % |
| careeros | `deadcode` 冷 | 131839.6（114932.9–170243.0） | 131762.5（117279.8–134931.2） | -0.1 % |
| careeros | `deadcode` 暖 | 7716.9（6903.0–8858.4） | 7617.1（7305.3–8629.6） | -1.3 % |
| careeros | `check` 暖 | 25471.4（20100.6–26521.8） | 25730.3（21252.8–26424.8） | +1.0 % |

- 预算（任务书）：暖 `ce check` ≤ +15 %——三棵树 -1.5 %、+9.4 %、+1.0 %，都在线内（zod 两臂各有一跑 72–82 s 的离群，中位数不受影响）。
- 两行量的是退出非零的跑：自仓 `check` 两臂都退 1（归档树里基线尚未按本车道重立）；careeros 的 `check` 两臂都退 2（`desync: expected candidates.result id 1`，dd0eec61 上同样如此、早于本阶段，见 `docs/reference/algorithm-track.md` §11 第 66 条），这一行是退出之前的那段时间。其余各跑全退 0。
- **多出核请求的面**（`warmreq_e.sh`：每臂冷跑一次 `graph --sites` 建索引，再经中继核把八个面各跑一次）：在 TS 树上，`structure` / `check` / `join --days 14` / `arch` / `erase` / `rules` 各多一个核会话、两个 `resolve.request`——`keys.rs` 算索引键时问 tsconfig 的 extends 链（`ts.chains`；第一个请求带走查到的配置原文，第二个补上核点名的事实），zod 12,540 B（其中 `ts` 12,428 B）、careeros 1,615 B（`ts` 1,503 B）；A 臂在 Rust 里读链、一个请求也不发。`deadcode` 在这一跑第一次建图边：B 臂在 zod 上 4 个请求（A 臂 0：TS 站点都在 Rust 里答）、careeros 4 个（A 臂 1）。自仓只有一个 TS 站点、没有 tsconfig，冷 `deadcode` 的请求多这一个站点（2,017 → 2,018），其余七个面两臂的请求逐种相同。`graph --sites` 与冷建两臂请求逐种相同。
- **请求字节**（§3 第 5 点；同一棵树一次冷 `deadcode`，经中继核记下每个 `resolve.request` 与应答的字节；两臂 `deadcode` 输出逐字节同）：

| 树 | 请求个数 A → B | 请求字节 A → B | 应答字节 A → B | 站点 A → B |
|---|---|---|---|---|
| self | 3 → 3 | 222,327 → 223,181 | 65,577 → 65,699 | 2,017 → 2,018 |
| zod | 0 → 4 | 0 → 154,378 | 0 → 51,865 | 0 → 2,186 |
| careeros | 1 → 4 | 63,536 → 438,585 | 9,798 → 186,139 | 503 → 10,824 |

- **核单独计时**（`coretime_e.sh`：把 B 臂那次冷 `deadcode` 的 `resolve/1` 会话经中继录下，单独喂核 5 次）：self 223,246 B → 中位 212.9 ms（193.4–274.4）、zod 154,443 B → 273.2 ms（262.7–307.0）、careeros 438,650 B → 187.2 ms（179.0–209.5）。逐请求拆开（`core_split_e.py`，会话截到每个请求为止、5 跑取中位、减去前一截；含约 80 ms 的进程起与握手）：两棵 TS 树都是四个请求——两轮问链、两轮答站点（第二轮带上核点名的事实：zod 447 条、careeros 126 条）；careeros 最后一个请求的站点截到 1,353 / 2,706 / 5,412 个，核用 14.6 / 34.9 / 63.2 ms，随站点数约成线性，没有平方项的迹象。
- **钩子**：PreToolUse 探针不问 `resolve/1`——在 zod 拷贝上写入一个带四条导入（同目录的 `./index.js`、工作区成员 `zod/v4/core`、内建 `node:fs`、依赖 `vitest`）的新 TS 文件的 `Write` 信封经 `ce probe --hook` 冷跑、暖跑各一次，中继核记下的 `resolve.request` 都是 0（`hookprobe_e.sh`），与阶段 A–D 同。
- 复跑：车道目录 `v233_w2t_scratch/perf_e.py 7`（树拷自车道归档与 `gate1e/seed-ts-*`），读数原文 `perf_e_all.log`、逐跑 `perf_e.ndjson`、负载 `perf_e_load.txt`；请求字节 `reqbytes_e.sh`，各面请求 `warmreq_e.sh`，钩子 `hookprobe_e.sh`，核单独计时 `coretime_e.sh` 与 `core_split_e.py`（`core_split_e.log`）。

## v2.33 W2-text 阶段 F Rust 阶梯、Cargo.toml 读法、crate 根与包内私有进核 A/B（实测 2026-10-06，release，同一台机、同一坐：A = 1324c927 的 ce + 它的核〔`git archive` 构建〕，B = 车道树的 ce〔sha256 51cf5cbb…，构建进车道的 scratch target；与落地的第一个提交 f6c2788a 的 `cli/src` 与 `core/app` 相同〕+ 它的核〔sha256 275fe509…；Rust 的五级阶梯、Cargo.toml 读法、crate 根与包内私有在核，测量侧只答核点名的盘上与语法事实〕；每臂每棵树各一份拷贝〔`.ce` 删掉〕；冷 = 跑前删 `.ce`，暖 = 有 `.ce` 之后的一跑；每个（树、面、冷暖）ABAB ×7，`python` `perf_counter` 夹 `subprocess.run`、含进程起，stdout 丢弃；坐时每分钟 `Get-CimInstance` 处理器负载，01:23–02:08 共 44 次 45–99 %——不是静默窗；每次记下的前五个进程按累计 CPU 秒排序，44 次里没有一次出现 cargo / rustc / ghc / cabal，但累计排序看不见一次短的编译；本坐前后这台机器上另有三条车道〔W1、W7、W4〕开工，它们何时开始编译本坐没有记下，故不能排除坐时有别的车道在编译）

口径：整个进程的墙钟，中位数（最小–最大），毫秒。树：本车道 HEAD 的归档（测试子仓就位）与两棵 Rust 树 ripgrep@3fce3b5、autoshade（`gate1f/seed-rs-*`，各提交一次）。

| 树 | 面 | A（1324c927） | B（车道） | B / A − 1 |
|---|---|---|---|---|
| self | `graph --sites` 冷 | 1930.7（1796.9–2251.9） | 1928.4（1813.4–2040.7） | -0.1 % |
| self | `graph --sites` 暖 | 1760.5（1674.0–2099.0） | 1820.2（1748.0–2011.3） | +3.4 % |
| self | `deadcode` 冷 | 44341.6（31812.1–49418.2） | 48473.0（43245.4–57963.6） | +9.3 % |
| self | `deadcode` 暖 | 3592.9（2874.3–4856.9） | 3658.2（3288.3–3803.4） | +1.8 % |
| self | `check` 暖 | 16187.7（15078.8–24785.5） | 16372.6（15384.0–24213.3） | +1.1 % |
| ripgrep | `graph --sites` 冷 | 1456.4（1191.0–1727.6） | 1285.0（1143.0–1766.7） | -11.8 % |
| ripgrep | `graph --sites` 暖 | 1365.1（1117.8–1471.8） | 1265.6（1030.3–1375.5） | -7.3 % |
| ripgrep | `deadcode` 冷 | 8833.7（7399.4–9810.8） | 9441.9（8107.0–10422.7） | +6.9 % |
| ripgrep | `deadcode` 暖 | 955.7（797.4–1154.4） | 1134.5（1067.1–1279.2） | +18.7 % |
| ripgrep | `check` 暖 | 5866.9（3875.4–35706.9） | 5891.4（4424.6–34982.3） | +0.4 % |
| autoshade | `graph --sites` 冷 | 6416.2（5870.2–7215.8） | 6701.1（6286.7–7353.5） | +4.4 % |
| autoshade | `graph --sites` 暖 | 6810.6（6400.9–7473.9） | 6514.8（6315.3–6873.2） | -4.3 % |
| autoshade | `deadcode` 冷 | 58798.0（47365.3–62627.9） | 59667.0（47470.7–68339.3） | +1.5 % |
| autoshade | `deadcode` 暖 | 4188.9（3629.4–5621.5） | 4410.5（3884.0–5572.7） | +5.3 % |
| autoshade | `check` 暖 | 23716.7（19588.7–44701.6） | 22202.7（20373.8–44044.0） | -6.4 % |

- 预算（任务书）：暖 `ce check` ≤ +15 %——三棵树 +1.1 %、+0.4 %、-6.4 %，都在线内（自仓 `check` 两臂都退 1：归档树里基线尚未按本车道重立）。暖 `deadcode` 在 ripgrep 上 +18.7 %（955.7 → 1134.5 ms），自仓冷 `deadcode` +9.3 %；预算不管这两个面，如实记下。
- **请求字节**（§3 第 5 点；同一棵树一次冷 `deadcode`，经中继核记下每个 `resolve.request` 与应答的字节；站点按请求累加，同一个站点每轮各算一次；两臂 `deadcode` 输出逐字节同）：

| 树 | 请求个数 A → B | 请求字节 A → B | 应答字节 A → B | 站点 A → B |
|---|---|---|---|---|
| self | 3 → 15 | 227,731 → 35,218,074 | 67,728 → 215,650 | 2,085 → 83,954 |
| ripgrep | 0 → 6 | 0 → 646,492 | 0 → 20,318 | 0 → 2,124 |
| autoshade | 1 → 5 | 16,871 → 1,640,350 | 6,101 → 43,305 | 433 → 5,364 |

- **多出来的字节从哪来**：自仓 B 臂的 15 个请求里 13 个是同一批 6,458 个站点的逐轮重问（2,569,749 → 2,719,869 B 一个），另两个不带站点（63,926 与 131,925 B）；ripgrep 4 轮 × 531 个站点、autoshade 3 轮 × 1,788 个。每一轮把全部站点与已答的事实再送一遍，核在 `tsWanted` 里点名还缺的事实；自仓轮数多，是因为 `CE.Resolve.Rs` 的 `globbed` 顺着通配再导出一次只问一个文件的顶层面（op 5），第 4 轮起每轮只多约两个面。可做的改法（本阶段不做）：一级之内把通配的各跳一次问齐、op 4 的行事实按需才送——改了要重跑切换门与本节。
- **核单独计时**（`coretime_f.sh`：把 B 臂那次冷 `deadcode` 的 `resolve/1` 会话经中继录下，单独喂核 5 次）：self 35,218,139 B → 中位 3586.7 ms（3494.6–4052.8）、ripgrep 646,557 B → 172.0 ms（136.9–185.1）、autoshade 1,640,415 B → 295.0 ms（262.9–318.2）。
- **其余七个面**（`warmreq_f.sh`：每臂冷跑一次 `graph --sites` 建索引，再经中继核把各面跑一次）：`structure` / `check` / `join --days 14` / `arch` / `erase` / `rules` 各一个 `resolve.request`、不带站点（自仓 61,653 → 63,925 B；ripgrep 0 → 17,042 B、autoshade 0 → 10,504 B，这两棵树各多一个核会话）；`graph --sites` 与冷建两臂请求逐种相同。
- **钩子**：PreToolUse 探针不问 `resolve/1`——在 ripgrep 拷贝上写入一个带四条 `use`（`crate::`、工作区成员 `grep_matcher`、`std`、`super::`）的新 Rust 文件的 `Write` 信封经 `ce probe --hook` 冷跑、暖跑各一次，中继核记下的 `resolve.request` 都是 0（`hookprobe_f.sh`），与阶段 A–E 同。
- 复跑：车道目录 `v233_w2t_scratch/perf_f.py 7`（树拷自车道归档与 `gate1f/seed-rs-*`），读数原文 `perf_f_all.log`、逐跑 `perf_f.ndjson`、负载 `perf_f_load.txt`；请求字节 `reqbytes_f.sh`（`reqbytes_f_sum.py`），各面请求 `warmreq_f.sh`（`warmreq_f_sum.py`），钩子 `hookprobe_f.sh`，核单独计时 `coretime_f.sh`。

## v2.33 W1 镜像退役与 `query/1` 词法进核 A/B（实测 2026-10-06，release，同一台机、同一坐 12:59–13:09Z：A = 3b7234eb 的 ce〔sha256 9c942054…〕+ 它的核〔sha256 0e13014b…〕，在 3b7234eb 的工作树里构建；B = 车道树的 ce〔sha256 d45df209…，构建进车道的 scratch target，与本项提交的 `cli/src` 相同〕+ 它的核〔sha256 c8bc8015…；对本项提交的 `core/app` 重跑 `cabal build` 零模块重编、sha 不变〕——scan 的等级复判与命名符合性退出 Rust，七个请求上限读自定义包，`ce query` / `ce rules` 的程序原文过线、由核分词；每臂每棵树各一份拷贝〔`.ce` 删掉〕，每个（树、面）先跑一次，再 ABAB ×7 暖跑，`python` `perf_counter` 夹 `subprocess.run`、含进程起，stdout 丢弃；坐时每分钟 `Get-CimInstance` 处理器负载，12:59–13:08Z 共 10 次 4–76 %——不是静默窗，前五个进程〔按累计 CPU 秒〕是别的会话的 Code 与一个 python，10 次里没有 cargo / rustc / ghc / cabal，但累计排序看不见一次短的编译，W2-text / W6 / W7 三条车道同时开着，它们何时编译本坐没有记下）

口径：整个进程的墙钟，中位数（最小–最大），毫秒。树：本车道 HEAD 的归档（测试子仓就位）与两棵真树 requests、cobra（拷自 W2-text 车道的 `.ce-eval/corpora`）。只量暖跑：本车道动的是判决后的读者与 `query/1`，不碰索引刷新与冷建。

| 树 | 面 | A（3b7234eb） | B（车道） | B / A − 1 |
|---|---|---|---|---|
| self | `check` 暖 | 11929.2（10256.5–13465.4） | 12759.5（10323.3–13675.7） | +7.0 % |
| self | `scan` 暖 | 1837.5（1738.1–2231.5） | 1810.9（1767.9–1866.9） | -1.4 % |
| self | `query` 暖 | 2709.5（2548.4–3014.4） | 2712.2（2603.1–2832.3） | +0.1 % |
| self | `rules` 暖 | 2243.8（2225.4–2478.7） | 2265.5（2203.9–2611.1） | +1.0 % |
| requests | `check` 暖 | 1033.9（1008.9–1091.7） | 1023.4（981.1–1095.5） | -1.0 % |
| requests | `scan` 暖 | 290.6（272.3–326.7） | 254.1（243.4–342.6） | -12.6 % |
| requests | `query` 暖 | 203.7（197.4–276.1） | 188.4（184.5–198.1） | -7.5 % |
| requests | `rules` 暖 | 198.9（179.3–232.8） | 182.7（173.5–231.8） | -8.1 % |
| cobra | `check` 暖 | 1173.0（1122.5–1197.1） | 1135.0（1076.9–1198.0） | -3.2 % |
| cobra | `scan` 暖 | 299.2（282.9–328.7） | 277.6（273.5–378.6） | -7.2 % |
| cobra | `query` 暖 | 205.4（191.0–256.9） | 214.9（185.3–223.6） | +4.6 % |
| cobra | `rules` 暖 | 204.2（186.3–254.9） | 187.1（179.8–206.3） | -8.4 % |

- 预算（§8）：暖 `ce check` ≤ +15 %——三棵树 +7.0 %、-1.0 %、-3.2 %，都在线内；自仓两臂的极差都过 3 秒（10256.5–13465.4 与 10323.3–13675.7），+7.0 % 落在这段噪声里。自仓 `check` 两臂都退 1（归档树里基线尚未按本车道重立），其余面两臂都退 0。`query` 用的问题是门里的 `file(F), lang(F, rust), lines(F, N), N > 200`。
- **请求字节**（§3 第 5 点；自仓头树，暖，经中继核记下每个 `query.request` 与应答的字节；两臂输出逐字节同）：`ce query` 一个请求 304,417 B、应答 3,485 B → 两个请求（一个 `lex`）306,762 B、应答 12,287 B；`ce rules` 一个请求 289,265 B、应答 414 B → 两个请求 291,697 B、应答 8,859 B。多出来的是程序原文（前奏八条 + 用户程序）与核答回的分词结果。
- **钩子**：本车道动的读者都不在 PreToolUse 探针上——`ce query` / `ce rules` 只由命令行、MCP 与 GUI 调用；留在钩子路上的镜像（`dedup/pairs.rs` 的 `minDistinct` 过滤、`score/knobs.rs` 的出厂值、`guard/zone.rs`）一个没动，故未单独量探针。
- 复跑：车道目录 `v233_w1_scratch/perf_w1.py 7`，读数原文 `perf_w1.log`、逐跑 `perf_w1.ndjson`、负载 `perf_w1_load.txt`；请求字节 `reqbytes_q.sh`（读数 `reqbytes_q.log`）。
