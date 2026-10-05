# 热路径延迟分解表——归档（2026-08-14–08-19 实测的七节）

> 自 [PERF-BUDGET.md](PERF-BUDGET.md) 末尾于 2026-10-05 逐字节迁入，原文一字未改；正册离 `ce scan` 的 750 行硬线只剩 7 行。

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
