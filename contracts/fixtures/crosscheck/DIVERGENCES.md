# M1 对拍结果与分歧清单（2026-08-07，fixtures 见 SOURCES.md）

> 验收口径（计划 §6 M1）：分歧全部清单化归因，无未解释分歧。
> "归因保留"= 规范差异，ce 语义立场注明出处；"已修复"= 对拍暴露的 ce 缺口。

## 总成绩单（全函数集合 set-diff，按 file:line join）

| 轴 | 对照物 | 函数数 | 一致 | 结果 |
|---|---|---|---|---|
| Go CC | gocyclo 0.6.0 | 52 | **52/52 (100%)** | ✅ 零分歧（default_case 轴 fixtures 沉默，规范侧修正见缺口 #3） |
| Python CC | lizard 1.23.0 | 104 | 102/104 | 2 条归因保留（finally） |
| TS CC | lizard 1.23.0 | 22 | 13/22 | 9 条全归因为 lizard reader 缺陷（详下） |
| Rust CC | rust-code-analysis 0.0.25（JSON 通路，harness 固化） | 322 | **322/322 (100%)** | ✅ 零分歧（`cli/tests/crosscheck_rca.rs` 可复跑；同 span 闭包多重集合比较） |
| Go CoC | gocognit | 32 非零 | 29/32 | 3 条归因保留（gocognit 的 else 块不提升嵌套，实验实锤，详下） |
| CoC 白皮书例题 | Sonar v1.7 原文页边判分 | 6 例题 | **6/6** | ✅ `cli/tests/sonar_whitepaper.rs`（页码内注，含 p.8 括号断链） |

## 对拍暴露并已修复的 ce 缺口（真收益）

1. **Python comprehension 分支**：`[x for x in xs if x > 0]` 的 for/if 子句是真实
   分支路径，lizard 与 radon 均计入 CC——ce 补入 `for_in_clause`/`if_clause`
   （spec.rs Python 表）。修复后 Python 分歧 11 → 2。
2. **Rust `?` 运算符**：`expr?` 是隐式 early-return（等价 match Ok/Err），RCA 计入
   ——ce 补入 `try_expression`。实锤：ban.rs `check` 差值 4 = 函数体内恰好 4 个 `?`；
   修复后 ban.rs 9/9 与 RCA 全对（含全部闭包边界与值）。
3. **Go `default_case` 误计**（攻击审阅发现）：gocyclo v0.6.0 complexity.go 明示
   "ignore default case"，白皮书 p.5 margin（getWords CC=4）同侧——ce 移除。
   fixtures 中零 `default:`，故 52/52 对此轴沉默、结论不受影响。
4. **Rust let-chain `&&` 双指标漏计**（攻击审阅发现）：`let_chain` 无 operator
   字段，`&&` 全匿名——CC/CoC 都数不到，且 ce 自身代码正用此惯用法。新增
   `chain_kinds` 机制：CC 计 N-1 个接合点，CoC 计一个算子序列。

## 归因保留的规范差异（ce 立场正确，不跟随对照物）

- **Python `finally`（2 条）**：lizard 对 finally 关键词计 +1（最小样本试探证实：
  纯 try/finally 无 except → lizard CCN=2）。finally 无条件执行、不产生独立路径，
  McCabe 语义与 radon 均不计。ce 不计。
- **Python `@overload` 存根（only-ce 14 条）**：`def f(...) -> T: ...` 是真实函数
  定义（CC=1），lizard 不单独报告。函数单位划分差异，非 CC 计算分歧。
- **TS lizard reader 三类缺陷（9 条 mismatch + 3 only-ce，三个同构 locale 文件
  各重复一次）**：① 嵌套箭头+模板字符串处函数边界切错（L58 内层箭头 lizard=7，
  手数源码 case10+if13+三元4+??6+1=**34**=ce，逐项吻合）；② 边界泄漏把内层分支
  记到外层箭头（L5：lizard=5，实际外层仅声明，ce=1）；③ `??` 被拆成两个 `?` 计 2
  （ce 计 1 个短路算子）；④ `export default function` 匿名默认导出未被识别。
  lizard 无 AST 的状态机 reader 在 TS 上不可靠——正是本项目立项要解决的那类问题。

## Go CoC 3 条：已归因保留（gocognit 的 else 块不提升嵌套层）

最小实验（goprobe，2026-08-07）实锤：`else { if ... }` 内的 if，gocognit 按 if
链所在层计罚（样本 a=3 / b=10），而 Sonar 白皮书"嵌套于断流结构内"的直读是
else 分支与 then 同层受罚（ce：a=4 / b=11）。闭包语义两边一致（probe2：
gocognit=5=ce，排除 func_literal 假说）。ce 站白皮书侧，保留分歧。逐条：
cobra.go:192 差 2 = ld() 两个 else 内 if 各差 1（Sonar 语义手算 18 逐项吻合）；
completions.go:932 差 1 = findFlag() 一处 else 内 if；completions.go:316
（span 316–585，270 行）差 17 = 3 个 else 块内嵌套结构的级联累积（机制已证，
未逐项分解）。另：gocognit 省略 CoC=0 函数（20 条 only-ce 已验证全为 0，非分歧）。

## Rust 全量对拍终版（2026-08-07 第三轮，harness 固化）

`cli/tests/crosscheck_rca.rs`（`#[ignore]` 集成测试，
`cargo test --test crosscheck_rca -- --ignored --nocapture`）走 RCA JSON
通路复跑全部 5 文件：**322/322 函数单位双向对齐且值全部一致，零分歧**。
第二轮的 21 条"分歧"与"319 个单位"均为临时扁平文本解析器的错位假象。
真值单位数 **322**：初版 harness 按 (start,end) 单键 join 时，walk.rs
1529/2718 两处同行嵌套闭包同 span 对称覆盖、双侧各静默丢 1 个单位
（攻击审阅发现）——现改同 span 多重集合比较并钉死总数断言。harness
另含 RCA 版本断言（钉 0.0.25）、陈旧产物硬防与归因白名单（当前为空）。

## Sonar 白皮书例题与立场钉死（2026-08-07 第三轮）

白皮书 v1.7（2023-08-29）例题全过：sumOfPrimes=7 / getWords=1（p.10）、
myMethod try-catch=9 / lambda 提嵌套=2（p.9）、toRegexp=20（p.19，
验证 else-if 链子结构留在链层语义）。例题驱动补齐 2 个 ce 缺口：

1. **labeled jump +1**（p.8 "Jumps to labels"）：`goto` / `break L` /
   `continue L` 计基本 +1，普通 break/continue 不计。三门语言 label
   子节点 kind 经 AST 探针核实（Go `label_name`、Rust `label`、TS
   `statement_identifier`）；Go fixtures 中无 labeled jump/goto（grep
   核实），已归档的 gocognit 对拍数字不受影响。
2. **CoC 算子分表**：新增 `coc_operators`，TS `??` 从 CoC 移除（p.6
   "Ignore shorthand" 明示忽略 null-coalescing）；CC 保留计 `??`
   （真实分支路径）。

立场钉死（`cli/tests/divergence_stances.rs`，计划 §6 M1 "分歧 case
显式收录不回避"）：装饰器（p.15）——ce 单位拆分模型（lizard 同型）下
Sonar 的装饰器特例天然不适用，not_a_decorator Sonar 聚合=2 vs ce 单位
和=1，分歧如实记录；`?.` CC/CoC 均不计（CoC 依 p.6，CC 为 M1 立场）；
Rust `break value` 不误计为 labeled jump。仍未实现：递归 +1（规范 §1，
需调用图，M5；cognitive.rs 头部注记）。

## 工具注记

- rust-code-analysis 0.0.25 JSON 通路可用条件（harness 已固化）：outdir 须
  **预先存在**；`-o <outdir>` 与输入的**相对路径**拼接产出
  `<outdir>/<rel>.json`（盘符绝对输入静默零产出）；每个 space 的
  `cyclomatic.sum` 聚合**全部后代 space**，函数自身 CC =
  `sum − Σ(直接子 space 的 sum)`（ban.rs check：29−8=21=ce 手工核实）。
  `--pr` 无文件产出；文本树模式可用但须缩进感知（第二轮教训）。lizard 的
  RustReader 因 match_arm/`?`/闭包三重定义差异不适合作 Rust 对照物
  （57/226 分歧，弃用）。
- PowerShell 管道会注入 UTF-8 BOM，两次破坏对拍通道——对拍一律走 bash/文件。
