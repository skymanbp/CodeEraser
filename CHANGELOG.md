# Changelog

> 记录义务来自 DEVELOPMENT_PLAN.md §4.2：“每次默认档位变更在
> CHANGELOG 记录依据（FPR 数据）。”本册只记守卫档位变更；
> 功能发布史见 GitHub Releases。

## [Unreleased]

（空——当前无未发布的档位变更。）

## [v0.1.0] — 2026-08-18 首发（下列两条均随其发布；v0.2.0 无档位变更）

### 2026-08-17 — 默认档位晋升：§4.2 路线第 3 级（1.0，M7-P2）

**变更**：`[guard]` 未显式设 `mode` 时，两类已晋级 PreToolUse 规则
默认从 ask 升为 **deny**（单一权威常量 `config::PROMOTED_DEFAULT`，
guard 执行面与 health/doctor 报告面同源）：

1. **T1/T2 精确/改名重复写入**；
2. **硬预算超限**（写后文件 > 750 行）。

Stop 审计 / precommit **不晋级**，默认维持 observe：两者无独立
FPR 记录在案（§4.2 第 3 级原文"其余规则默认 ask/warn 按各自 FPR
记录决定"——无记录即无晋级资格，如实缺席）。显式 `mode` 依旧
统一覆盖全部规则类。

**依据（FPR 数据，与第 2 级同一账本，无新增拦截记录在案）**：

- T1/T2 探针：630 次真实编辑重放 0 误拦（docs/FPR-REPLAY.md
  2026-08-07：requests 487 + 自仓 143）；
- 判定层主门：600 全语料真实正常编辑重放 0 标记 = 0% ≤ 1%
  （contracts/eval/fpr-fourclass-v1.json）;
- 硬预算：自仓全史 first-parent 380 个 A/M 事件按排除模型 0 触发
  （2026-08-11 条目的复现命令不变）。

### 2026-08-11 — 默认档位晋升：§4.2 路线第 2 级

**变更**：`ce.toml` 的 `[guard]` 未显式设 `mode` 时，两类 PreToolUse
规则默认从 observe 升为 **ask**：

1. **T1/T2 精确/改名重复写入**（M3 探针，daemon 索引比对）；
2. **硬预算超限**——写后文件 > 750 行（本级新增规则：按 Edit/Write
   自身语义在内存精确套用后计行，走 scan 同源排除模型
   `scan::walk::in_scope`，免 daemon）。

Stop 审计 / precommit 不在晋升类，默认仍 observe；显式 `mode` 统一
覆盖全部规则类。观察档 schema 随之 0.3.0 → 0.4.0（新增 `budget`
事件行，为 §4.2 第 3 级的按规则 FPR 记录留账）。

**触发条件**：计划 §4.2 第 2 级 “M4 的 FPR 门（§6）通过后”——已于
2026-08-11 通过。

**依据（FPR 数据）**：

- T1/T2 探针：630 次真实编辑事件重放（requests 完整历史尾段 487 +
  CodeEraser 自仓全史 143），仲裁后误拦 **0**，折算 0.00/500 ≤ 1
  （docs/FPR-REPLAY.md，2026-08-07）。
- M4 判定层主门：600 全语料真实正常编辑重放误报 **0/600** = 0% ≤ 1%
  （contracts/eval/fpr-fourclass-v1.json：samples 600、flagged 0、
  gate_max_percent 1）。
- 硬预算规则：判定为精确行数算术，无统计误报模态；对自仓全史
  first-parent 重放（五门首发语言扩展名，380 个 A/M 事件）超限事件
  5 个，全部落在 `contracts/fixtures/crosscheck/**`（钉定第三方对照
  物，ce.toml 排除项覆盖）——按已装排除模型计发火 **0/380**。复现：
  `git rev-list --reverse --first-parent HEAD` 逐提交
  `git diff-tree -r --root --no-renames --diff-filter=AM` 取
  `rs|py|ts|tsx|go|md` 文件，`git cat-file -p <sha>:<file> | wc -l`
  与 750 比较。
