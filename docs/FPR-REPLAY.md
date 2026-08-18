# M3 FPR 重放验收记录（2026-08-07）

> **封册（M7.5 深度瘦身，2026-08-18）**：重放仪器 `fpr_replay.rs` 已随
> 休眠仪器整体退役（EVAL-SET.md 修正案），本册即 FPR 记录的最终账本；
> 复核走 git 历史复活仪器（退役修正案提交的父 commit）。

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
  抓住了它要消灭的行为。治理已分四批完成（251→211→209→205→202 块，
  预算同步降档）：`tests/common/mod.rs` 统一数据助手 + hook 运行器 +
  daemon 启动/回收 + observe 尾行读取 + dedup 断言 + tree-sitter 解析
  harness（第 5 份拷贝在 RCA 对拍 harness，切换后 322 单元实跑复验绿）；
  `src/hookio.rs` 统一三 hook 的 stdin 进气与 observe 写入。保留不抽
  =命名边界（ce.toml 注释同步）：用例平行结构（deny/warn/observe 三态
  断言、dedup_index 刷新序列）、三 hook 的信封契约声明（4 块）、
  metrics/divergence/sonar 的文档性测试体（~150 块——逐例白皮书引注
  即内容本体，T2 把 src 字面量折叠后相邻用例连成一个家族）。

## 重放捎带抓获的产品缺陷（已修）

探针候选文件在"索引后、探针前"消失（删除/改名竞态；requests 史上的
目录改名提交实锤触发）曾使探针整体报错——现按陈旧锚点同一哲学降级
跳过（probe.rs `load_candidate_streams`），daemon 探针请求不再因竞态
失败。

## 复现

- 自仓：`cargo test --release --test fpr_replay -- --ignored --nocapture`
- 外部仓：`CE_FPR_REPO=<path> cargo test --release --test fpr_replay -- --ignored --nocapture`
  （本记录的 requests 语料：钉定 commit 浅克隆后 `git fetch --deepen 400`）
