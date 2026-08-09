# 热路径延迟分解表（M0 验收项，DEVELOPMENT_PLAN.md §6 M0 / §7.6）

> 口径：被动 guard 的 PreToolUse 端到端 = hook 触发 → 判定返回。
> 预算为硬上界（进 CI 基准后回归即 fail）；"实测"列只写真实测过的数字，
> 未测项标注实测里程碑，不预填。

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

## M3 hook 端到端预算（实测 2026-08-07，release，warm daemon，deny 档）

| 项 | 预算 | 实测（30 次） | 状态 |
|---|---|---|---|
| `ce probe --hook` e2e：解析信封 + 探针往返 + 判定组装 + 回传 | p95 < 1 s（合计行 ce 侧） | median 64 / p95 69 / max 73 ms | ✅ |
| 同上，clean 路径（无判定输出，静默） | —（无预算） | median 70 / p95 81 ms | 记录 |
| 冷首呼（懒起 daemon + 首连 + 判定） | —（降级档兜底，ADR-003） | 213 ms | 记录 |

复跑：`cargo test --release --test perf_budget -- --ignored --nocapture`
（合成语料确定性生成；M2 收口大仓复测时用真实仓库再录一列；
hook e2e = `hook_e2e_p95_under_1s`）。

补充口径：

- 环节 2 的 Defender **首扫**（新编译 exe 第一次运行）不计入常规预算，单列记录（M0 验收原文）。
- 会话累计口径：hook 延迟中位数 < 15 s / 百次编辑（M3 验收）。
- daemon 冷启动（首次索引构建）不占热路径——未就绪期显式降级为廉价检查档（ADR-003）。
- 复测命令：`cli/` 下 `cargo build --release` 后
  `1..10 | %{ (Measure-Command { .\target\release\ce.exe --version }).TotalMilliseconds }`。
