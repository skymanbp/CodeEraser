# M3 FPR 重放验收记录（2026-08-07）

> 验收门（计划 §6 M3）：500 次真实正常编辑重放误拦 ≤ 1。
> 方法：`cli/tests/fpr_replay.rs`（`#[ignore]`，release 跑）——把 git
> 线性历史当真实编辑流：每提交每改动代码文件 = 一次"把子版本内容写入
> 父状态"的编辑事件；影子目录增量物化父状态，**先探针后应用**；guard
> 默认档（t=50 / min_distinct=7）。拦截逐条列出供仲裁——真实引入了
> 跨文件重复的提交是真阳，不计入误拦。

## 结果

| 语料 | 事件数 | 拦截 | 仲裁后误拦 | 折算 /500 |
|---|---|---|---|---|
| requests 完整历史尾段 400 提交（钉定 1f6589ec 止） | 487 | 0 | **0** | 0.00 |
| CodeEraser 自仓全史（真实 agent 编辑流） | 143 | 35 | **0**（全为真重复，见下） | 0.00 |
| 合计 | 630 | 35 | **0** | **0.00 ≤ 1 ✅** |

## 自仓 35 条拦截仲裁（全真阳）

- 2 条：zod locale fixtures 入库提交——这些文件**按设计**互为 T2 克隆，
  "新增文件与既有文件重复"判定正确。
- 33 条：`cli/tests/*.rs` 之间共享的 `rust_fn(seed)`/`tmp()`/git 助手
  ——**作者本人复制粘贴的真实重复**（8 个测试文件各持一份）。工具当场
  抓住了它要消灭的行为。治理项：抽 `tests/common/mod.rs` 共享助手
  （已排期，M3 内完成）。

## 重放捎带抓获的产品缺陷（已修）

探针候选文件在"索引后、探针前"消失（删除/改名竞态；requests 史上的
目录改名提交实锤触发）曾使探针整体报错——现按陈旧锚点同一哲学降级
跳过（probe.rs `load_candidate_streams`），daemon 探针请求不再因竞态
失败。

## 复现

- 自仓：`cargo test --release --test fpr_replay -- --ignored --nocapture`
- 外部仓：`CE_FPR_REPO=<path> cargo test --release --test fpr_replay -- --ignored --nocapture`
  （本记录的 requests 语料：钉定 commit 浅克隆后 `git fetch --deepen 400`）
