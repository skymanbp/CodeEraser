# codeeraser plugin

被动 guard <!--ce:count:hooks#word-->三<!--/ce-->钩（全部 fail-open，内部失败一律放行）+ <!--ce:count:skills#word-->一<!--/ce-->个主动 skill + <!--ce:count:commands#word-->一<!--/ce-->条命令：

| hook | 命令 | 行为 |
|---|---|---|
| SessionStart | `ce health --hook` | 健康行（版本/guard 档/索引/daemon）+ daemon 预热；有新版本时另起一行更新通知（检查结果缓存一天，`CE_UPDATE_CHECK=0` 关闭，无网络即无此行） |
| PreToolUse (Write\|Edit) | `ce probe --hook` | 对将写入的内容做 T1/T2 重复探针 + 硬预算（写后行数超本文件那条硬线）两类，按 `ce.toml [guard] mode` 决策；未超硬线而落在软线与硬线之间的写入由分级区观察器记账，`[guard] zone_tiers` 开启后才按落点出声；墓碑类按自己的 `[tombstone] tier` 与 budget 在核里判，只在核答 over 时出声 |
| Stop | `ce audit --hook` | 净 LOC + 涉改重复块，仅 deny 档拦停；墓碑腿同样在核里按 `[tombstone] tier` 与 budget 判、也只有 deny 档拦停。四分类汇总（跨文件搬迁 / 堆叠嫌疑）只记不判，账本见 [docs/FPR-L2.md](../docs/FPR-L2.md)；同角色顾问行（本会话新增的单元其 top-1 带角色位时）只落进 observe 账本，永不拦停 |

skill：[`skills/erase/`](skills/erase/SKILL.md)——把 dedup/deadcode/join
的发现引导成安全删除（先读全文、查引用、小批删、重跑门证收敛），
用户说"清理重复/死码"时由 Claude 自动调用。

命令：[`/codeeraser:update`](commands/update.md)——跑 `ce update` 并转述判定
（退出码 0 最新 / 1 有更新 / 2 未知）。插件绑定的副本由清单 pin 决定，
所以它的更新动作永远是 `/plugin update codeeraser`（新清单带新 pin，下一会话
启动器重验重下载）；`ce update --yes` 只替换手工放置或安装包随附的副本。

MCP：[`.mcp.json`](.mcp.json) 注册只读报告面（`ce mcp`），<!--ce:count:mcp_tools#word-->十六<!--/ce-->个工具随插件
一起到位——装插件 = 钩子与报告一起装，不需要另外 `claude mcp add`。工具名
由 Claude Code 自动命名空间化为 `mcp__plugin_codeeraser_reports__<tool>`。
`erase` 工具只到**计划**为止、`erase_log` 只读它的审计轨迹：`apply` 没有 face、也不会有——一个能凭自己
的权限删文件的机器面，是橡皮擦唯一不能出的东西；`update_check` 同理只到检查；
`similar_units` 是同角色顾问（`ce similar` 的同一份文档），只当顾问不判决。

## 安装

公开 marketplace 一键装（清单在仓根 `.claude-plugin/marketplace.json`；注册的是
`release` 分支——verify-publish 每次 publish 后把它快进到 tag 提交，装机跟发布不跟 main）：

1. `/plugin marketplace add skymanbp/CodeEraser@release`
2. `/plugin install codeeraser@codeeraser`

v1.7.0 起 **`ce setup` 就是这两步**，任何平台同一具身体（`cli/src/setup/`）：找到 Claude Code
→ 已注册则保留、否则注册 → 安装并刷新 → 只在本次注册时写 `claude-plugin-wired` 标记
→ 报告本二进制所在目录是否在 PATH 上；运行账户 ≠ 登录用户时按名拒绝、退 13 什么都不接；
`--unwire` 以标记为凭只拆它自己接的。Windows 安装包装机 / 卸载时各调一次
（`gui/src-tauri/windows/hooks.nsh` 只印退出码图例，失败一律降级为提示行），AppImage / dmg /
纯 CLI 底座自己跑一次。本地开发 clone 则注册**仓根目录**为 marketplace
（`claude plugin marketplace add <repo根>`）；注册 `plugin/` 子目录是 9f86d58 之前的旧位，
清单已不在那里，会以 cache-miss 静默掉钩。

`ce` 与判决核 `ce-core` 两个真身均由 `bin/ce.sh` 三级解析（ADR-007）：
已验证本地副本 → 按 `bin/manifest.env` 的 SHA256 pin（每目标两枚：五目标 ×
双二进制，v1.7.0 前的清单只钉前三个目标）从 GitHub Releases 下载并校验 → PATH 兜底。校验按会话一次：验证过的
路径记在 `CLAUDE_PLUGIN_DATA/bound-<清单版本>.env`，同会话后续 hook 直接 exec；
清单或二进制更新即重验，SessionStart 的 `health` 每会话必验。手放到
`CLAUDE_PLUGIN_DATA/ce-<清单版本>-<平台>` 的副本走同一道 pin：相符即按已验证执行并落戳，
不相符具名拒绝、不执行（`CE_AIRGAPPED=1` 永不下载，只在手放副本与 PATH 之间选）。
源码安装（`cargo install codeeraser` 或 `--path cli`）依然可用。

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
分级区档位映射（软线→硬线之间按位置 <25 % observe / 25–75 % warn / >75 % ask）走 `[guard] zone_tiers`
这一个开关；首测自仓 28/5196 = 0.5388 %、requests 11/448 = 2.4553 %，后者超 1 %，采用变体 B，默认仍为 `false`。完整台账在 [docs/FPR-REPLAY.md](../docs/FPR-REPLAY.md)「分级区档位映射」节。
观察档数据在 `<project>/.ce/observe.ndjson`（已被 `.ce/` gitignore 规则覆盖），
每行带 `schema`（单一来源 `cli/src/hookio.rs::OBSERVE_SCHEMA`——版本号以那一处为准，此处不再抄写；M5 收口审计抓获抄本 0.3.0 陈旧于实际 0.4.0）、`session_id` 与 `ts_ms`；
`tombstone` 事件行（PreToolUse 一次写入删掉了名字、或它的标题 / 标识符 / 散文把本会话删掉的名字
写回成「无 X」时才落一行，只记名字的哈希）与 Stop / precommit / commitmsg 行上的 `tombstone` 对象
（本次改动集的度量：删掉的名字数、候选面数 `rows`、changelog 定位豁免——整文件按路径、台账形或 `[tombstone] ledger` 声明，或只豁免一段而段条目带起始 `line`——与核的判决 `judged`：前几处站点 `file:line kind`、标签 / 散文分账、`over`）
按类自己的 `[tombstone] tier`（默认 observe）出声，且只在核答 `over`（站点数超过 `[tombstone] budget`）时——observe 只记不拦，FPR 账本见 [docs/FPR-TOMBSTONE.md](../docs/FPR-TOMBSTONE.md)；
`fourclass` 对象（跨文件搬迁与堆叠嫌疑）永不出声——它不属于任何档位，其改动集级
FPR 账本见 [docs/FPR-L2.md](../docs/FPR-L2.md)：821 事件零跳过，严格 0/125、宽读法 0/820；唯一 copy 正例 `1035f6b1` 漏过（召回 0/1），本批无默认档位变更；
`session_id` 为 `null` 表示该条不属于任何会话——`ce precommit` / `ce commitmsg`（后者把提交说明也当一个面，站点记 `COMMIT_EDITMSG:行 prose`）跑在终端里、
不是 hook，是仅有的会出现 null 的来源。按会话切分是 M4 评估集的前置
（计划 D2-1 样本纯净度 / D2-2 观察档会话计数）。

> hooks 配置在会话启动时加载——安装/改档后重启会话生效
> （2026-08-07 hook payload 实采会话的实测结论）。
