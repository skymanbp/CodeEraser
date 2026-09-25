# M1 对拍结果与分歧清单（2026-08-07，fixtures 见 SOURCES.md）

> 验收口径（计划 §6 M1）：分歧全部清单化归因，无未解释分歧。
> "归因保留"= 规范差异，ce 语义立场注明出处；"已修复"= 对拍暴露的 ce 缺口。

## 总成绩单（全函数集合 set-diff，按 file:line join）

| 轴 | 对照物 | 函数数 | 一致 | 结果 |
|---|---|---|---|---|
| Go CC | gocyclo 0.6.0 | 52 | **52/52 (100%)** | ✅ 零分歧（default_case 轴 fixtures 沉默，规范侧修正见缺口 #3） |
| Python CC | lizard 1.23.0 | 104 | 102/104 | 2 条归因保留（finally） |
| TS CC | lizard 1.23.0 | 22 | 13/22 | 9 条全归因为 lizard reader 缺陷（详下） |
| Rust CC | rust-code-analysis 0.0.25（JSON 通路，harness 固化） | 322 | **322/322 (100%)** | ✅ 零分歧（harness 已随 M7.5 封册退役，复跑从 git 历史复活；同 span 闭包多重集合比较） |
| Go CoC | gocognit | 32 非零 | 29/32 | 3 条归因保留（gocognit 的 else 块不提升嵌套，实验实锤，详下） |
| CoC 白皮书例题 | Sonar v1.7 原文页边判分 | 6 例题 | **6/6** | ✅ `cli/tests/it/sonar_whitepaper.rs`（页码内注，含 p.8 括号断链） |
| CoC 递归增量 | 四语料重跑（新旧二进制同树） | 514 单位 | **0 条移动** | ✅ 既有对拍全部不受影响（2026-08-31，详见「递归增量」节） |
| C CC | lizard 1.23.0 | 118 | **116/118** | 2 条归因保留（`default:`，D2；2026-09-24 计划 v2.30 步 2，详见 C / C++ 节） |
| C++ CC | lizard 1.23.0 | 420 join（lizard 430 起始行 / ce 428 单位） | 394/420 | 26 条 + 两侧独有 18 条全归因：D1 20、D2 3、局部类 1 + 4、解析器恢复 1 + 14、lizard 三类缺陷 1 + 1 + 10 重复行（详见 C / C++ 节） |
| Java CC | lizard 1.23.0 | 33 join（按结束行；lizard 34 行 / ce 38 单位） | **33/33** | 零数值分歧；ce 独有 5 条 = 带类型实参的匿名类方法，lizard 并进宿主或整段不报（D29，详见 Java 节） |
| Java CoC | PMD 7.27.0 | 38 join（PMD 46 方法，8 个无体） | **37/38** | 匿名类方法并进宿主（D29，归因保留）1；`if` 条件里的三元那条随「条件不抬嵌套」（D31）落码两侧一致 |
| CoC 条件不抬嵌套（D31） | 七个对拍语料重跑（新旧二进制同树） | 1,098 单元 | 2 条移动 | Java `checkAccessible` 3 → 2（与 PMD 相符）、C++ `do_write_float` 16 → 15，都是 `if` 条件里的三元（详见「条件不抬嵌套」节） |

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

harness `crosscheck_rca.rs`（`#[ignore]` 集成测试；已随 M7.5 深度瘦身
退役，复跑=按 EVAL-SET.md 再生成节从 git 历史复活）当时走 RCA JSON
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

立场钉死（`cli/tests/it/divergence_stances.rs`，计划 §6 M1 "分歧 case
显式收录不回避"）：装饰器（p.15）——ce 单位拆分模型（lizard 同型）下
Sonar 的装饰器特例天然不适用，not_a_decorator Sonar 聚合=2 vs ce 单位
和=1，分歧如实记录；`?.` CC/CoC 均不计（CoC 依 p.6，CC 为 M1 立场）；
Rust `break value` 不误计为 labeled jump。递归 +1 当时未实现，**已于
2026-08-31（计划 v2.23 步 4）补齐**，见下节。

## 递归增量与它带来的系统性分歧（2026-08-31，计划 v2.23 步 4）

白皮书 p.8 与 Appendix B1 写的是 `+1 for each method in a recursion cycle,
whether direct or indirect`。**SonarSource 自家三个分析器一条都不实现**——
sonar-java / sonar-python / sonar-javascript 三仓源码 2026-08-31 第一方核对，
`recurs` 零命中。我们站规范侧做全，于是与真实 SonarQube 分数之间存在一条
**系统性正偏**：凡在递归环里的函数，我们比 SonarQube 高 1 分（每个环成员各 1）。
这条差不是缺陷，也不会在任何语料上被"修掉"——它是两侧对同一份规范的取舍差，
在此具名登记。

**对照物逐个：**

- **gocognit**（Go 社区的 S3776 实现）：只做直接递归，用符号身份判定。
  同一份探针实测（2026-08-31）：

  ```
  $ gocognit -top 20 .
  2 p fact probe.go:3:1
  1 p plain probe.go:14:1
  ```

  `fact`（一个 if + 自递归）两侧都是 **2**，`plain`（一个 if）两侧都是 **1**
  ——**直接递归部分逐值一致**。互递归的 `a`/`b` gocognit 静默（它省略 CoC=0
  的函数，已在上文登记），我们各计 1。这一对就是两个实现分手的地方。
- **rust-code-analysis 0.0.25 / lizard 1.23.0**：均不实现任何递归增量，
  故本轴对它们全部是系统性正偏，与上同源。

**重跑结果：四份语料零移动。** 用新旧两个二进制在同一棵树上逐函数对拍
（go 52 / python 118 / rust 319 / typescript 25 个单位，键 = 路径 + 起始行 +
名字）：**没有任何一个单位的 CoC 变化**。也就是说上表里 gocyclo 52/52、
lizard 102/104、RCA 322/322、gocognit 29/32 这几行**全部原封不动**——四份语料
里没有一个我们能证明的文件内环。

首轮重跑曾有**唯一一条**移动，且是**误记**，当场根修：`ignore` crate 的
`walk.rs:2215`

```rust
#[cfg(unix)]
fn symlink<P: AsRef<Path>, Q: AsRef<Path>>(src: P, dst: Q) {
    use std::os::unix::fs::symlink;
    symlink(src, dst).unwrap();
}
```

体内那句 `use` 是**最内层绑定**，裸 `symlink(..)` 是被导入的那个函数，不是它
自己。修法 = `LangSpec::call_import_kinds`：一个单元自己体内的导入所绑定的
名字，裸调用永不认领（`cli/src/scan/calls.rs`）。配一正一反两条腿；摘掉规则即
重现这条误记。修后四语料全部零移动。

**锚的来源（白皮书无递归计分例题）。** v1.7 六道例题无一含递归调用，所以锚是
**推导的**，且推导成两面可对：底数取 p.10 的 `sumOfPrimes`（页边判分 7，已由
`sonar_whitepaper.rs` 对着同一页边钉住），给它加一句丢弃返回值的自调用——调用
本身不是结构增量，所以**同一份源码**的环前读数必须仍是 7，而结清后必须是 8。
差值就是被测的那条规则，两面读数把它钉死在这条规则上而不是别处。
电池：`cli/tests/it/coc_recursion.rs`，另含互递归与直接递归同价、只有环内成员
付钱（环外调用者与无环链各 0）、跨文件环不建边三条腿。

**具名不做：跨文件环。** 调用弧是单个解析单元内的词法事实。跨文件要么按名铸边
（R6 实测精度 0.576），要么走 symEdges（召回约 23 %）——错的 +1 会流进分数与
尺寸门，漏的只是一分没收。立场以断言的形式留在电池里，不会哪天默默变成真的。

## C / C++ 对拍（2026-09-24，计划 v2.30 步 2；fixtures 见 SOURCES.md 的 c / cpp 行）

对照物只有 lizard 1.23.0 的 CCN：C/C++ 没有认知复杂度对照物，CoC 的执行者是
白皮书电池 `cli/tests/it/coc_c.rs`（每条 D 表立场各一行）。join 键 = 文件 +
起始行；lizard 在同一起始行重复报出的两行算一个单位（见下）。复现：

```
python -m lizard -l c   contracts/fixtures/crosscheck/c/*.c
python -m lizard -l cpp contracts/fixtures/crosscheck/cpp/*.h
ce scan contracts/fixtures/crosscheck --format json     # 按 start_line join
```

**C（lua 五文件）**：118 / 118 单位逐一 join，116 一致；2 条差 = D2——
lbaselib.c:201 `luaB_collectgarbage` ce 8 / lizard 7、lfunc.c:145
`prepcallclosemth` ce 4 / lizard 3，各恰含一个 `default:`（ce 计真路径，lizard
只计 `case` 关键字）。零两侧独有。

**C++（fmt 五头文件）**：lizard 440 行 = 430 个起始行（10 行重复），ce 428 单位，
join 420，一致 394。26 条差值全部机械归因（lizard − ce == 区间内 `#if` 系行数
− `default:` 行数 + Σ 局部类成员 (cc − 1)，残差 2 条逐条读过）：

- **D1 预处理条件 20 条**：lizard 对每条 `#if` / `#ifdef` / `#ifndef` / `#elif` 行
  +1，ce 不计——且条件表达式里的 `&&` / `||` 也不计。后半句是本轮对拍修出来的
  ce 缺口：首轮 `is_big_endian`（format.h:272）读 2，多出的 1 是 `#elif
  defined(__BYTE_ORDER__) && defined(__ORDER_BIG_ENDIAN__)` 里的 `&&`；`operator+=`
  （:368）读 7 与 lizard 的 7 相等纯属两种错误相抵（lizard 计三条 `#if` 行，ce
  计三条行里的三个 `&&`）。修法 = LangSpec 新增 `opaque_fields`
  （`(preproc_if | preproc_elif, condition)`），度量层唯一的子结点入口
  `metrics::walk::measured` 跳过；电池行 `q`。
- **D2 `default:` 3 条**：`write_escaped_cp`（:2003）、`write_int`（:2173、:2296）
  各含一个 `default:`。
- **局部类成员 1 + 4 条**：`write`（:2395）体内的 `struct bounded_output_iterator`
  四个运算符（:2458–:2465）ce 各自成单位（电池 `local() { struct L { void m() {} }; }`
  行），lizard 折进宿主——宿主 lizard 24 = ce 23 + `operator=` 的一个 `if`。
- **`if FMT_CONSTEXPR20 (…)` 1 条**：`write`（:2371）lizard 4 / ce 3——`if constexpr`
  的 constexpr 由宏拼写（:2377），tree-sitter 读不成 if_statement（ERROR）。解析器
  恢复，不改。
- **lizard 右值引用 1 条**：`nested_format_specs::write`（:4089）lizard 4 / ce 2——
  `static_cast<T&&>(values)...` 两处的 `&&` 被 lizard 当逻辑算子。最小样本：
  `return static_cast<T&&>(v) ? 1 : 0;` lizard CCN 3。lizard 无类型层，本项目立项
  要解决的那类问题。

两侧独有 18 条（lizard 10 / ce 8）：

- **解析器恢复（宏未展开）14 条**。fmt 的 `FMT_*` 宏让 tree-sitter-cpp 的错误恢复
  移动单位边界，两侧各有各的读法，ce 不追：
  - color.h：`FMT_CONSTEXPR styled_arg(const T& v, text_style s) : value(v), style(s) {}`
    （:474）——宏在构造函数前被读成返回类型，随后的成员初始化列表打断解析，整个
    `struct styled_arg` 折进下一个函数的 type：ce 一单位 `vprint` 起始 :471，lizard
    `styled_arg` 构造 :474 + `vprint` :497（lizard 2 / ce 1）。
  - format.h :2279 `FMT_CONSTEXPR size_padding(int …) : size(…)` 同形——struct 体永不
    闭合，:2296–:4089 的成员在 ce 侧都带上 `size_padding::` 伪 owner，构造函数本身
    lizard 有 ce 无（1 / 0）。
  - format.h :935 `template <…> class basic_memory_buffer : public detail::buffer<T> {`
    折进 placeholder 约束：类体内的方法两侧都读到（:945–:1024 七条一致），但移动
    构造 :1029 被读成 field_declaration（lizard 有 ce 无），`operator=` :1035 成了那个
    折叠定义的声明子——ce 的单位起始位移到 :937（lizard 1035 / ce 937）。同一类里
    `FMT_CONSTEXPR20 ~basic_memory_buffer()`（:984）的 `~` 落进 ERROR 结点，ce 把它
    放回名字（`dropped_tilde`，电池行）。
  - format.h :1138 / :1145 `FMT_EXPORT template <…> constexpr auto compile_string_to_view`
    ——宏在 `template` 前，两条被读成 ERROR / 无名叶的定义：ce 零单位（名叶规则），
    lizard 两条。
  - format.h :1355–:1361 `extern template FMT_API auto …` 五行被折进 `write2digits`
    的前缀：ce 一单位起始 :1358 吞掉两条 `equal2`（:1366 / :1369），lizard 三条
    （3 / 1）。
- **lizard 大括号初始化 1 条**：format.h :323 `return {~n.hi_, ~n.lo_};` 之后的
  `friend FMT_CONSTEXPR auto operator+`（:326）被 lizard 并进 `operator~`（最小样本
  重现：合成一个名为 `U::operator ~( const U & n ) -> U { return {…} ; } friend …`
  的单位）。ce 独有 :326。
- **局部类 4 条**（上）。

lizard 同一起始行重复报出 10 行：:1559 / :2173 / :2296 / :2395 / :2727 / :3141 /
:3277 / :3377 / :3677 / :3734，都是多行签名且形参表含模板实参
（`basic_string_view<WChar>`、`digit_grouping<Char>`…），两行数值相同，按一个单位计。

**对拍修掉的 ce 缺口（真收益，都在 `scan/declarator.rs` 与 `spec_c.rs`）**：

1. 预处理条件里的算子当分支（上，D1）。
2. `= default` / `= delete` 当函数单位（无 body 的 function_definition：format.h
   :2980 / :2981 / :4130 / :4131、color.h :211，共 5 条）→ 定义须有 body。
3. 宏行 + `namespace detail {` 被读成名为 `namespace` 的函数（std.h :80、ostream.h
   :33、format.h :1122）、`namespace detail {` 被读成名为 `detail` 的函数（color.h
   :206）、:2275 的 struct 折叠体（名字是三行文本）→ 声明链须有形参表、叶须是名字
   结点（ERROR / 折进来的类型都不是）。
4. 类内 `FMT_CATCH(...) { }`（std.h :524）→ 类内无 type 的定义须是构造 / 析构 /
   转换运算符（三者带类名或 `~` / `operator`）；无 owner 的保留（:4126 的构造函数
   所在类体被恢复读成了块）。
5. 体内嵌套 function_definition（宏块、GNU 嵌套函数）→ 吸收进宿主（lambda 先例），
   局部类成员照旧独立。
6. 名字含模板实参与换行（std.h :656 `formatter<\n T, char, …>::write`）→ 名字取
   标识符：`formatter::write`、`Box::b`、`spec`。

修后 C++ 单位 440 → 428（12 条伪单位消失：`= default/delete` 5、`namespace` 4、
struct 折叠 1、`FMT_CATCH` 1、`basic_string_view<Char>` 1）。四份既有语料同树重扫
单位数不变（go 52 / python 118 / rust 322 / typescript 25）：新规则只对带 `declarator`
字段的结点生效，`opaque_fields` 只有 C 表非空。

**步 3 补记（重载按实参个数择一，2026-09-24）**：C++ 与 Java 里同名、形参不同的函数是不同
的函数，调用按实参个数落到恰好一个能接纳它的重载（`scan/callees.rs` 的 `pick`；两个都接纳
即不连边）。此前同一作用域里的同名函数被读成一个可调用体，于是「调同名、参数个数不同的另
一个重载」被记成递归、CoC 多 1。fmt 五头文件同树重扫 CC 不动，CoC 四个单位各 −1，四个都是
委托给另一个重载：`styled_arg::vformat_to`（color.h:478，四参调三参）5 → 4、`append`
（format.h:1054，`append(range)` 调 `append(begin, end)`）1 → 0、`to_utf8::convert`（:1552，
调三参的静态重载）2 → 1、`size_padding::nested_format_specs::parse`（:4080，四参调三参）
3 → 2。

## Java 对拍（2026-09-24，计划 v2.30 步 3；fixtures 见 SOURCES.md 的 java 行）

CC 对照物 lizard 1.23.0；CoC 对照物 PMD 7.27.0 的 `CognitiveComplexity` 规则——Java 是第一个
有可跑的独立 CoC 实现的新语言（SonarSource 自家的 sonar-java 读同一份白皮书，但只在 SonarQube
里跑；它的访问器源码按需第一方核对，见下）。复现：

```
python -m lizard -l java contracts/fixtures/crosscheck/java/*.java
pmd check -d contracts/fixtures/crosscheck/java -R ruleset.xml -f csv
ce scan contracts/fixtures/crosscheck --format json        # 按结束行 join
```

`ruleset.xml` 只有两条规则：`category/java/design.xml/CognitiveComplexity`（`reportLevel` = 1，
报出每个非零方法）与 `category/java/design.xml/CyclomaticComplexity`（`methodReportLevel` = 1——
它列出 PMD 读到的每个方法，于是「有 CYCLO 行、无 CoC 行」就是 CoC 0）。

**join 键 = 文件 + 结束行。** tree-sitter-java 的方法结点从它的 `modifiers` 起，注解是方法声明的
一部分（JLS 8.4.3，MethodModifier 含 Annotation），所以 ce 的起始行落在第一个注解上；lizard 与
PMD 从名字所在的行起。五个文件里 16 个带注解的方法起始行差一行，结束行两侧处处相同（D30）。

**CC（lizard）**：lizard 34 行 = 33 个结束行（`getBoundFields` :341 被报两行、数值相同——多行
签名带泛型形参，C++ 节同款），ce 38 单位；join 33，**33/33 数值一致**，形参个数也全等。ce 独有
5 条，全是匿名类的方法（D29）：

- `ReflectiveTypeAdapterFactory.create`（:110）里 `return new TypeAdapter<T>() { … }` 的
  `read`、`write`、`toString`（:124 / :130 / :135）；
- `SqlTypesSupport` 静态初始化块里 `new DateType<java.sql.Date>(…) { … }` 与
  `new DateType<Timestamp>(…) { … }` 各一个 `deserialize`（:65 / :72）。

lizard 的 Java reader 遇到带类型实参的匿名类即把类体并进宿主（宿主是静态初始化块时整段不报）；
不带类型实参的匿名类它自己也单独报成 `(anonymous)::m`——同一文件 :237 / :276 / :290 那三个即
如此，两侧一致。最小样本：`Object f() { return new Box<T>() { public String toString() { if (a)
return "a"; return "b"; } }; }` lizard 只报 `f` CCN 2；删掉 `<T>` 则报 `(anonymous)::toString`
2 与 `f` 1。无体方法（两个接口方法、六个 `abstract`）两侧都不成单位（D24 同一立场）。

**CoC（PMD）**：PMD 46 个方法里 8 个无体（只有 CYCLO 1 那一行），其余与 ce 的 38 单位逐一 join
（同名，且 PMD 报的行落在 ce 单位的起止行之间），**37/38 一致**。一条差；落码前的第二条已随 D31 消掉：

- `createBoundField`（:181）PMD 39 / ce 7：方法体里 `new BoundField(…) { … }` 的三个方法
  （`write` 12、`readIntoArray` 2、`readIntoField` 8，两侧对它们本身的读数相同）被 PMD 再整个
  并进宿主、每个结构多一层嵌套。sonar-java（SonarSource/sonar-java@98c1e25 的
  `CognitiveComplexityVisitor`）也并进宿主（`visitClass` 抬嵌套），但不再单独给这些方法记分
  （`shouldAnalyzeMethod` 跳过匿名类与局部类的成员）；PMD 两处都记。ce 的单位拆分模型下它们是
  独立单位、宿主不含——与 Python 装饰器（白皮书 p.15，上文「立场钉死」）同源，归因保留（D29）。
- `checkAccessible`（:168）：`if (!canAccess(member, isStatic(…) ? null : object))`——三元在 `if`
  的条件里。PMD 与 sonar-java（`visitIfStatement` 在 `nesting++` 之前扫条件）都不给条件加嵌套，读 2；
  ce 落码前给（三元 +2）读 3。2026-09-24 用户裁「条件都不算」，落码后读 2，两侧一致（D31，见「条件
  不抬嵌套」节）。

PMD 的 CYCLO 不作 CC 对照：它是另一种口径——`throw` 计 +1（`createDuplicateFieldException`
PMD 2、lizard 与 ce 1），控制流条件之外的 `&&` / `||` 不计（`BagOfPrimitives.equals` 末尾那句带
三个 `&&` 的 `return`：PMD 3、lizard 与 ce 6）。

## 条件不抬嵌套（2026-09-24，D31，所有语言）

用户裁「条件都不算」：结构的头部——条件、循环子句、switch 的值、catch 的形参——按结构自己的层级计分，
只有语句体抬嵌套；else-if / elif 的条件与首个 if 的条件同在链的层级；三元整个抬嵌套（D4 不变）。每个语言
在 `LangSpec::coc_nesting_kinds` 的项里写明结构的语句体位置：字段名，或者语法没给语句体起字段名时写它的 kind
（Python except 的 `block`、Go switch 的两种 case、Haskell case 的 `alternatives`）。白皮书只列抬嵌套的
结构（p.9、Appendix B2），没说头部算不算在里面。

对照物只在 `if` 自己的条件上一致：sonar-java 的 `visitIfStatement` 与 PMD 7.27.0 都在 `nesting++` 之前扫它。
其余头部 PMD 照样抬嵌套。九个方法的探针（每个方法在一种头部里放一个三元），PMD 读 `if` 2、`while` 3、`for` 3、
for-each 3、do-while 3、`switch` 3、else-if 4、`if` 条件里的 lambda 3、catch 体里 `if` 条件的三元 5；ce 读
2 / 2 / 2 / 2 / 2 / 2 / 3 / 3 / 5——在循环、switch 与 else-if 的头上比 PMD 低 1，立场如此，归因保留。三元的
条件里再套三元，两侧都抬嵌套（PMD 3、ce 3）。电池 `cli/tests/it/coc_headers.rs`（八种语言 25 行，每行记落码
前的读数，Java 行另记 PMD 的读数）。

重跑（落码前后两个二进制，同一棵树，逐单元按起始行对拍）：七个对拍语料 1,098 个单元里移动 2 个——Java
`checkAccessible` 3 → 2（与 PMD 相符，见 Java 节）、C++ `do_write_float` 16 → 15（`if` 条件里的三元）；本仓
（`4f021e2` 的 `cli/src`、`core/app`、`core/test`）与九个语料——cobra `adbc881`、gson `854c825`、jsoup `093e2f5`、
requests `8068356`、ripgrep `3fce3b5`、zod `912f0f5`、junit4 `890f3c9`、mockito `5a2f0f8`、TheAlgorithms/Java
`dd8df80`——43,732 个单元里移动 12 个，全是头部里的三元、lambda 或 Haskell `case` 判断值里的 `if`，每个降 1–2 分
（jsoup `matchesSibling` 降 2：`for` 头里两个三元）。

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
