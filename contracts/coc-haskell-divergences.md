# Haskell CoC/CC 分歧登记册（M5-3k，2026-08-14）

> S3776（Sonar Cognitive Complexity v1.7）从未命名任何 Haskell 构造，也不存在
> 实现了 CoC 的 Haskell 外部工具——**"无外部 oracle"本身是本册第一条分歧（D0）**，
> 诚实的归因就是缺席（plan `:282-283` 逐条归因条款）。下表每条都是**立场**：
> 引白皮书最近似条款 + 记录我们的裁定 + 指向执行它的电池行
> （[cli/tests/coc_haskell.rs](../cli/tests/coc_haskell.rs)，表即登记册的机检半身；
> 表格 kind 全部实探自 tree-sitter-haskell 0.23.1，探针记录 2026-08-14 两轮）。
> 表体=[cli/src/scan/spec_hs.rs](../cli/src/scan/spec_hs.rs)。

| # | 构造 | 裁定 | S3776 依据 / 先例 | 执行 |
|---|---|---|---|---|
| D0 | 全体 | 无外部 oracle：一切映射为仓内立场，电池是唯一执行者 | —（缺席如实登记） | 全表 |
| D1 | 守卫 `\| cond =` | 每个守卫备选 flat +1（elif 类比，无嵌套罚）；每个条件（`boolean`/`pattern_guard`）计 1 个 CC 决策 | p.7 hybrid increments；Python elif 先例 | classify 行：coc 5 / cc 6 |
| D2 | `case` 备选 | CoC 记 case 一次（备选免费）；CC 逐备选计数**含通配 `_`**（case 是全函数，`_` 是真路径；不采纳 gocyclo 的 default 跳过） | p.10 switch 一次结构增量；Rust match_arm 先例 | getWords 行：coc 1 / cc 5 |
| D3 | `if-then-else` | 表达式=三元类比：+1 结构带嵌套罚；`else` 零成本；else-if 链按嵌套三元付嵌套罚（S3776 的 else-if 豁免限语句 if，Haskell 没有语句 if） | p.6 三元；TS ternary 先例 | depth 行：coc 3；五语言等价对拍恒 3 |
| D4 | 循环 / 带标签跳转 | Haskell 无此形：sumOfPrimes/toRegexp 两道白皮书例题**不可移植**（登记非静默略过）；惯用递归承担循环，而递归 +1（S3776）依赖调用图=M1 已录缺口，在 Haskell 面更宽 | p.5/p.10/p.19 例题；M1 recursion 缺口记录 | 本册（无电池行=不可移植的记录本身） |
| D5 | 异常处理 | `catch`/`handle` 是普通函数应用，语法层不可见：无增量——与"try 透明"（p.9）对我们可见的语法一致；handler lambda 走 D6 | p.9 try transparent | —（无语法可测） |
| D6 | `lambda` / `\case` | 吸收进宿主单元、只抬嵌套不增量 | p.9/p.13；Go func_literal 先例 | myMethod2 行：coc 2 |
| D7 | 多方程函数 | 每方程一单元（`function` 结点本就逐方程），同键由 with_nth 区分；方程间的模式分派本身不计（它是定义层的 case 类比）——相对"一个函数"观是有意低计，如实登记 | —（无条款） | s 行：fns 2 / params 2,2 |
| D8 | `bind` 三义性 | 具名 bind（顶层/where/let）是独立单元（Rust closure 先例）；do 语句 `x <- act` 与模式绑定**不是**（name 门控）；箭头类型 `A -> B` 的 kind 也是 `function`（电池抓获：三参签名曾铸出三个伪单元）同受 name 门控 | —（无条款） | main 行 fns 1；p/where 行 fns 2 |
| D9 | 列表推导 | `generator` 与过滤 `boolean` 计 CC（真分支路径）、不计 CoC（声明式无嵌套代价） | Python for_in_clause/if_clause 先例（M1 立场） | spec_hs cc_kinds（电池未逐钉，M1 Python 行同立场） |
| D10 | `multi_way_if` | 按 case 嵌套（+1 带罚），臂内守卫走 D1 | p.10 switch 类比 | spec_hs coc_nesting（结构探针实证 match/guards 形） |
| D11 | 关键字 token 撞名 | 该文法 anon 关键字 token 与结构结点同 kind 名（`case`）：度量步进器只对 **named** 结点做表匹配——对既有五语言零语义变化（其表全用长名，由既有电池同批复验绿证明） | —（文法事实） | cyclo.rs/cognitive.rs 守卫 + 三电池同批绿 |
| D12 | 命名规范 | camelCase → MixedCaps 检查（撇号 `'` 无下划线不受罚） | —（社区惯例） | spec_hs name_style |

## 范围声明

- 本册随 3k 冻结；3l（graph 阶梯）已于 2026-08-14 落地
  （[cli/src/graph/ladder/hs.rs](../cli/src/graph/ladder/hs.rs)，本册无行受其触动——
  import 边不改任何 CoC/CC 裁定）；递归检测（M5 调用图）落地时**就地改写**对应行
  （D4 的调用图缺口），不追加平行条目（CLAUDE.md 硬约束 4）。
- 电池行的 why 字符串与本册互为索引：改任一侧必须同批改另一侧。
