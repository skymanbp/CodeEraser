# T1 拦截实录（M3 验收项，DEVELOPMENT_PLAN.md §6 M3）

> 口径：真实 Claude Code 会话（headless `claude -p`，hooks 全量生效）在
> deny 档测试仓库中写入 T1 重复内容，被 PreToolUse 拦截，transcript 为证。
> 同批完成 0.x 预览干净环境安装验证。实测日期 2026-08-08，装机 ce 0.0.1。

## 1. 端到端拦截（2026-08-08 22:46）

测试仓库（临时目录，不入 dogfood 语料）：`util.py`（20 行，
`rolling_checksum` 函数，归一化 128 tokens）+ `ce.toml`（`[guard]
mode = "deny"`）+ git init。插件为本地 marketplace 正式安装件。

headless 会话（session_id `c6cfc772-9981-4ef2-8d70-05c89265ca7c`，仅放行
Write 工具以堵死 Bash 绕过）被指示将 util.py 内容逐字写入新文件 `dup.py`。
三方证据：

1. **transcript**（`~/.claude/projects/<test-repo-slug>/c6cfc772….jsonl`，
   本地留存，按 D2-7 不入库）——Write 的 tool_result 为 `is_error: true`，
   携带 guard 的拒绝理由逐字：

   > ce: content for <test-repo>/dup.py duplicates 1 indexed region(s):
   > util.py:1-20 (128 tokens). Reuse the existing implementation instead
   > of re-writing it.

2. **observe feed**（该仓 `.ce/observe.ndjson`，schema ce.observe/0.2.0）：
   `{"event":"probe","mode":"deny","matches":1,"degraded":false,
   "session_id":"c6cfc772-…"}`，随后同会话 `stop_audit` 记
   `changed_files:0`。
3. **磁盘**：`dup.py` 不存在（拦截发生在落盘前）。

会话最终答复原文引用了上述拒绝理由并确认未创建文件。

## 2. 干净环境安装验证（同日）

| 面 | 做法 | 结果 |
|---|---|---|
| 二进制 | 全新 `CARGO_HOME` + `cargo install --path cli --locked`（全依赖重新解析下载） | 成功，1m17s，`ce --version` = 0.0.1 |
| 插件 | 全新 `CLAUDE_CONFIG_DIR` + `claude plugin marketplace add plugin/` + `claude plugin install codeeraser@codeeraser` | 成功；缓存件 `hooks/hooks.json`、`plugin.json` 与源逐字节一致（diff 空）；登记钉 `gitCommitSha 2f28b46` |
| 接线 | 用干净二进制照 hooks.json 原样跑三条命令 | health 输出健康行（`guard: deny \| index: 2 files`）；probe 输出上述 deny 判定；audit rc=0 并落 feed |

## 3. 捎带修复：daemon 冷启动盲窗口（ADR-003 未实现项）

实录首次预验证时发现：daemon 懒启动**不建索引**、probe 只读永不刷新、
Stop 审计无变更时也跳过刷新——全新仓库首会话 probe 一律
`matches=0, degraded=false`，盲探针在 feed 里与真实 clean 不可分
（M4 语料假阴性，违反 ADR-003"首次索引后台异步构建 + 未就绪显式降级"
与 §5.9"绝不静默失效"）。

修复（本提交）：daemon 绑定 socket 后即后台构建首个索引（索引已有内容
则零行为变化）；构建期/构建失败时 probe 回显式 Error，客户端按既有
通道记 `degraded=true` 并 fail-open。协议与 guard.rs 零改动。验证：
新增 e2e `cold_start_probe_degrades_then_serves_matches` 对旧码红
（"silent empty report"）新码绿；新鲜仓库实测首探即 deny（小仓首建在
懒启动窗口内完成），全程未手动跑过 `ce dedup`。

## 4. M3 dogfood 收口 census（2026-08-10）

D2-2 判据全部达标（口径：`D:\Projects\*\.ce\observe.ndjson` 中
schema `ce.observe/0.2.0` 的 distinct `session_id`；计数起点 = 2026-08-08
22:03 重装，此前 1289 条 0.1.0 事件无会话身份、仅作旁证）：

| 判据 | 目标 | 实测 |
|---|---|---|
| dogfood 会话数 | ≥ 10 | **10**（7 个项目：lore_disaster / CodeEraser / docsbot×2 / cc-memory×2 / Autoshop / interview-helper / cc-tree / anti-laziness） |
| 其中观察档 | ≥ 5 | **10**（全部纯 observe，零拦截塑形——M4 语料满足 D2-1 纯净度） |
| 会话累计 hook 延迟中位数 | < 15 s/百次编辑 | **0.982 s**（按会话求 probe 均值×100 取中位；min 0.196 / max 2.111） |

跨度 08-08 22:04 → 08-10 04:59；共 2,671 probe + 225 stop_audit，
degraded 仅 1 次（ce 重装瞬间探针连不上 daemon，fail-open 且如实标注）。
用户拍板（2026-08-08）：计数只取自然积累，绝不模拟对话凑数；
测试仓库的 headless 会话（§1）不在 dogfood feed 内，未混入计数。

## 复跑

```
# 拦截：临时仓 + util.py + ce.toml(deny) + git init 后
claude -p --allowedTools "Write" < prompt.txt   # prompt 要求逐字写入 dup.py
# 冷启动回归
cargo test --test daemon_e2e cold_start
# census：扫 D:\Projects\*\.ce\observe.ndjson，按 0.2.0 行 distinct session_id 计数
```
