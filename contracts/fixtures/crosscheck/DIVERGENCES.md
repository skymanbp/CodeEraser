# M1 对拍结果与分歧清单（2026-08-07，fixtures 见 SOURCES.md）

> 验收口径（计划 §6 M1）：分歧全部清单化归因，无未解释分歧。
> "归因保留"= 规范差异，ce 语义立场注明出处；"已修复"= 对拍暴露的 ce 缺口。

## 总成绩单（全函数集合 set-diff，按 file:line join）

| 轴 | 对照物 | 函数数 | 一致 | 结果 |
|---|---|---|---|---|
| Go CC | gocyclo 0.6.0 | 52 | **52/52 (100%)** | ✅ 零分歧 |
| Python CC | lizard 1.23.0 | 104 | 102/104 | 2 条归因保留（finally） |
| TS CC | lizard 1.23.0 | 22 | 13/22 | 9 条全归因为 lizard reader 缺陷（详下） |
| Rust CC | rust-code-analysis 0.0.25（文本模式抽查） | 9（ban.rs 全量） | **9/9 (100%)** | ✅ 修复 `?` 后零分歧 |
| Go CoC | gocognit | 32 非零 | 29/32 | 3 条归因保留（gocognit 的 else 块不提升嵌套，实验实锤，详下） |

## 对拍暴露并已修复的 ce 缺口（真收益）

1. **Python comprehension 分支**：`[x for x in xs if x > 0]` 的 for/if 子句是真实
   分支路径，lizard 与 radon 均计入 CC——ce 补入 `for_in_clause`/`if_clause`
   （spec.rs Python 表）。修复后 Python 分歧 11 → 2。
2. **Rust `?` 运算符**：`expr?` 是隐式 early-return（等价 match Ok/Err），RCA 计入
   ——ce 补入 `try_expression`。实锤：ban.rs `check` 差值 4 = 函数体内恰好 4 个 `?`；
   修复后 ban.rs 9/9 与 RCA 全对（含全部闭包边界与值）。

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

## Rust 全量对拍中间状态（2026-08-07 第二轮）

RCA 文本模式跑通全部 5 文件：**319/319 函数单位与 ce 完全对齐（含全部闭包，
onlyRca=0 onlyCe=0）**，值一致 298/319。剩 21 条"分歧"经抽查证实主要是
**临时扁平解析器在嵌套 space 处的错位假象**（skip_entry@1149 批量解析读作 2，
单文件人工核对 RCA 真值=10=ce）。终版一致率待缩进感知 harness 固化后复跑出具；
ban.rs 子集 9/9 为已核实基线。

## 工具注记

- rust-code-analysis 0.0.25 在 Windows 下 JSON 输出不可靠：`--pr` 无文件产出、
  `-o <dir>` 行为不稳定（一次落在输入文件旁、批量循环零产出）。文本树模式稳定
  可用（ANSI 剥离后解析，须缩进感知）。lizard 的 RustReader 因 match_arm/`?`/
  闭包三重定义差异不适合作 Rust 对照物（57/226 分歧，弃用）。
- PowerShell 管道会注入 UTF-8 BOM，两次破坏对拍通道——对拍一律走 bash/文件。
