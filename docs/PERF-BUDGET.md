# 热路径延迟分解表（M0 验收项，DEVELOPMENT_PLAN.md §6 M0 / §7.6）

> 口径：被动 guard 的 PreToolUse 端到端 = hook 触发 → 判定返回。
> 预算为硬上界（进 CI 基准后回归即 fail）；"实测"列只写真实测过的数字，
> 未测项标注实测里程碑，不预填。

| # | 环节 | 预算 | 实测（本机 Win11, 2026-08-07） | 实测里程碑 |
|---|---|---|---|---|
| 1 | Claude Code fork hook 进程（Windows shell form 经 PowerShell） | ≤ 300 ms | 未测 | M3（echo-hook 计时） |
| 2 | `ce` 冷启动（进程 + clap 解析 + 退出） | ≤ 100 ms | `ce --version` ×10：min 28.3 / median 30.3 / max 53.1 ms | ✅ M0 |
| 3 | named pipe 连接 + 指纹探针往返（daemon 热态） | p95 ≤ 150 ms | 未测（daemon 未建） | M2 |
| 4 | 判定组装 + stdout JSON 回传 | ≤ 50 ms | 未测 | M3 |
| 合计 | PreToolUse 端到端 | **p95 < 1 s**（含 Defender/冷 daemon 余量） | 未测 | M3 验收门 |

补充口径：

- 环节 2 的 Defender **首扫**（新编译 exe 第一次运行）不计入常规预算，单列记录（M0 验收原文）。
- 会话累计口径：hook 延迟中位数 < 15 s / 百次编辑（M3 验收）。
- daemon 冷启动（首次索引构建）不占热路径——未就绪期显式降级为廉价检查档（ADR-003）。
- 复测命令：`cli/` 下 `cargo build --release` 后
  `1..10 | %{ (Measure-Command { .\target\release\ce.exe --version }).TotalMilliseconds }`。
