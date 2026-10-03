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

## v2.32 步 2 定义包改读 A/B（实测 2026-10-01，release，同一台机、同一窗口：两棵 baa4f8af 的 git 工作树〔测试子仓就位，两臂各一棵，免得两代索引互相重建〕，A = baa4f8af 的 ce〔`git archive` 构建，表在二进制里〕，B = 本批的 ce〔表读自 `.ce/tables-1.8.0-7.7.0.json`，缓存命中〕，核同一个 ce-core 1.8.0〔proto 7.7.0〕；各臂先冷建一次索引〔`dedup` / `scan` / `check`〕，再 ABAB ×7，`python` `perf_counter` 夹 `subprocess.run`、含进程起；量前 `Get-CimInstance` 处理器负载 8 %）

口径：整个进程的墙钟，中位数（最小–最大），毫秒；`hook_probe` = bench 的同名形（`ce probe --hook` 对两文件夹具〔`a.rs` / `c.rs` 一对 T2 孪生〕写第三个文件的 PreToolUse 信封，stdout 落文件），ABAB ×30。

| 面 | A（表在二进制里） | B（表读自缓存） | 状态 |
|---|---|---|---|
| `ce scan .` 暖 | 1300.4（1253.4–1333.3） | 1262.0（1248.5–1296.9） | 噪声内 |
| `ce check .` 暖 | 6419.5（6181.6–6608.1） | 6459.8（6123.7–6905.7） | 噪声内（+40 ms，区间互相覆盖） |
| `ce dedup .` 暖 | 801.2（760.7–866.1） | 807.2（762.8–826.7） | 噪声内 |
| `hook_probe`（n = 30） | 53.8（48.3–276.8） | 54.5（50.1–260.9） | +0.7 ms；p50 预算 50 ms 两臂都不达，A 也不达——是本机此窗口的底，不是包的代价 |

缓存未命中的第一跑（B 自己 ABAB ×7：每对先删 `.ce/tables-*.json` 再跑、随即缓存命中再跑，`ce scan .`）：未命中 1381.7（1305.3–1513.9）、命中 1250.8（1199.3–1341.8），**未命中多付 ≈ 131 ms**——起一个核进程问一次 `tables/1`、读 135 KB 应答、写缓存文件；每份核二进制（`{path, len, mtime_ns}`）每个项目付一次。

- 缓存命中时每个 `ce` 进程多做的事 = 读 135,472 B 的缓存文件、对包字节算一次 fnv1a64、serde 读成 `'static` 表（泄漏进进程，活到进程结束）；暖跑三面与钩子探针都量不出来，故缓存保持 JSON、不换二进制格式。
- release 体积：A 27,388,416 B → B 27,470,336 B（+81,920 B）：删掉的表文本（Rust 源码净 −149,033 B，见设计册 §4.5）在二进制里只是字符串常量，新加的是 serde 读包的派生代码（每个表结构一份所有权孪生）。
- 复跑：A 臂的树由 `git archive`（baa4f8af，子仓 123b1bce 同法）解到车道目录，`cargo build --release --locked`（独立 `CARGO_TARGET_DIR`）出 A；本车道 `cargo build --release --locked` 出 B；`git worktree add --detach <dir> baa4f8af` 两次、各 `submodule update --init`（`protocol.file.allow=always`）；脚本 `s9_flow/v232s2_gen/perf.py`（车道目录，不入库）。

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

## M3 PreToolUse 探针端到端（口径 = 首表合计行的 ce 侧 = 行 2+3+4，release，n=30；本节自身未记测量日期）

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
