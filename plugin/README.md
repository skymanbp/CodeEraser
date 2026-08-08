# codeeraser plugin — 0.x preview（air-gapped）

被动 guard 三件套（全部 fail-open，内部失败一律放行）：

| hook | 命令 | 行为 |
|---|---|---|
| SessionStart | `ce health --hook` | 健康行（版本/guard 档/索引/daemon）+ daemon 预热 |
| PreToolUse (Write\|Edit) | `ce probe --hook` | 对将写入内容做 T1/T2 探针；按 `ce.toml [guard] mode` 决策 |
| Stop | `ce audit --hook` | 净 LOC + 涉改重复块；仅 deny 档拦停 |

## 预览期安装（无网络分发，ADR-007 的 SHA256 pinned 下载在 M7）

1. `cargo install --path cli`（或把 `cli/target/release` 加入 PATH），
   确认 `ce --version` 可用；
2. 本地 marketplace 安装本目录（`/plugin marketplace add <repo>/plugin`，
   再 `/plugin install codeeraser@codeeraser`）；
3. 项目里可选 `ce.toml`：

```toml
[guard]
mode = "observe"   # observe（默认）| warn | ask | deny
```

档位晋升按计划 D-4：FPR 数据达标才从 warn 升 ask/deny。
观察档数据在 `<project>/.ce/observe.ndjson`（已被 `.ce/` gitignore 规则覆盖），
每行带 `schema`（当前 `ce.observe/0.2.0`）、`session_id` 与 `ts_ms`；
`session_id` 为 `null` 表示该条不属于任何会话——`ce precommit` 跑在终端里、
不是 hook，是唯一会出现 null 的来源。按会话切分是 M4 评估集的前置
（计划 D2-1 样本纯净度 / D2-2 观察档会话计数）。

> hooks 配置在会话启动时加载——安装/改档后重启会话生效
> （contracts/fixtures/hook-payloads/README.md 的实测结论）。
