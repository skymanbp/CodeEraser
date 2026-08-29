# codeeraser plugin

被动 guard 三钩（全部 fail-open，内部失败一律放行）+ 一个主动 skill：

| hook | 命令 | 行为 |
|---|---|---|
| SessionStart | `ce health --hook` | 健康行（版本/guard 档/索引/daemon）+ daemon 预热 |
| PreToolUse (Write\|Edit) | `ce probe --hook` | 对将写入内容做 T1/T2 探针；按 `ce.toml [guard] mode` 决策 |
| Stop | `ce audit --hook` | 净 LOC + 涉改重复块；仅 deny 档拦停 |

skill：[`skills/erase/`](skills/erase/SKILL.md)——把 dedup/deadcode/join
的发现引导成安全删除（先读全文、查引用、小批删、重跑门证收敛），
用户说"清理重复/死码"时由 Claude 自动调用。

MCP：[`.mcp.json`](.mcp.json) 注册只读报告面（`ce mcp`），13 个工具随插件
一起到位——装插件 = 钩子与报告一起装，不需要另外 `claude mcp add`。工具名
由 Claude Code 自动命名空间化为 `mcp__plugin_codeeraser_reports__<tool>`。
`erase` 工具只到**计划**为止：`apply` 没有 face、也不会有——一个能凭自己
的权限删文件的机器面，是橡皮擦唯一不能出的东西。

## 安装

公开 marketplace 一键装（清单在仓根 `.claude-plugin/marketplace.json`）：

1. `/plugin marketplace add skymanbp/CodeEraser`
2. `/plugin install codeeraser@codeeraser`

v1.0.1 起 **Windows 安装包装机时自动跑这两步**（检测到 Claude Code 才动手，
失败一律降级为提示行；`gui/src-tauri/windows/hooks.nsh` 是权威实现）——
手动路径留给 AppImage/dmg/纯 CLI 底座。本地开发 clone 则注册**仓根目录**
为 marketplace（`claude plugin marketplace add <repo根>`）；注册 `plugin/`
子目录是 9f86d58 之前的旧位，清单已不在那里，会以 cache-miss 静默掉钩。

`ce` 与判决核 `ce-core` 两个真身均由 `bin/ce.sh` 三级解析（ADR-007）：
已验证本地副本 → 按 `bin/manifest.env` 的 SHA256 pin（六枚：三平台 ×
双二进制）从 GitHub Releases 下载并校验 → PATH 兜底。校验按会话一次：验证过的
路径记在 `CLAUDE_PLUGIN_DATA/bound-<清单版本>.env`，同会话后续 hook 直接 exec；
清单或二进制更新即重验，SessionStart 的 `health` 每会话必验。源码安装
（`cargo install codeeraser` 或 `--path cli`）依然可用。

3. 项目里可选 `ce.toml`：

```toml
[guard]
mode = "observe"   # 全局覆盖（可选）：observe | warn | ask | deny
                   # 不写 = 计划 §4.2 路线默认（见下）

[[rules.class]]    # 可选：路径类分参（计划 v2.13）——声明序首中，无命中 = 全局表
name  = "generated"
globs = ["gen/**"]
[rules.class.knobs]
file_lines_fail = 900   # 该类自己的硬预算——PreToolUse 按这条拒；至多 64 类，类名与 glob 永不过线
```

默认档位 = §4.2 路线第 3 级（2026-08-17 1.0 档位切换生效，依据见根目录
[CHANGELOG.md](../CHANGELOG.md)）：**T1/T2 精确重复写入**与**硬预算超限
（写后文件超过其硬线：默认 750，或其 `[[rules.class]]` 声明的那条——hook 经 `guard::budget::lines_for`、`ce scan` 直呼，两面同读 `scan::classes::Classes::thresholds_for` 那张类尺）**两类 PreToolUse 规则默认 `deny`；Stop 审计 /
precommit 不在晋升类，默认仍 observe。显式 `mode` 统一覆盖全部规则类。
观察档数据在 `<project>/.ce/observe.ndjson`（已被 `.ce/` gitignore 规则覆盖），
每行带 `schema`（单一来源 `cli/src/hookio.rs::OBSERVE_SCHEMA`——版本号以那一处为准，此处不再抄写；M5 收口审计抓获抄本 0.3.0 陈旧于实际 0.4.0）、`session_id` 与 `ts_ms`；
`session_id` 为 `null` 表示该条不属于任何会话——`ce precommit` 跑在终端里、
不是 hook，是唯一会出现 null 的来源。按会话切分是 M4 评估集的前置
（计划 D2-1 样本纯净度 / D2-2 观察档会话计数）。

> hooks 配置在会话启动时加载——安装/改档后重启会话生效
> （contracts/fixtures/hook-payloads/README.md 的实测结论）。
