# 语言扩展设计册：C / C++ / Lua / Java / Ruby / R / HTML（计划 v2.30 修正案草案）

<!-- ce:allow(deadcode) -- 设计稿尚未被计划书链接；v2.30 修正案并入计划横幅并链接本册后删除本行 -->

> 状态：**设计稿，待用户拍板**（§14 列出全部待裁项）。本册只设计、不落码：计划书 `docs/DEVELOPMENT_PLAN.md` 已由 cc-memory 锁定，流程是「改计划 → 重新锁定 → 才能动代码」，本册即拟并入计划横幅的 v2.30 修正案正文与 §6 新轨步表。表内每个 tree-sitter 结点 kind 与字段名都于 2026-09-24 在 tree-sitter 0.27.0 上**实探**（两轮，§10），不是回忆；标 † 的条目未入本轮样本，落码时由电池实证。先例：M5-3k（Haskell 全文法入判决集，`contracts/coc-haskell-divergences.md`）与计划 v2.5（尺寸门语言臂）——本册是前者的七倍，边界沿用后者。 探针与十四个样本随本册入库：[`scripts/tsprobe`](../../scripts/tsprobe/README.md)；交接事项见 §15。

## 0. 一句话定位

七个语言进入判决集：六个代码语言（C、C++、Lua、Java、Ruby、R）走 Haskell 走过的全套——文法、单元、两个复杂度、克隆指纹、引用图阶梯、可见性、提及规约；HTML 作为**文档类判决语言**（Markdown 的同类：文档重复、引用图、节锚，**不取 token 指纹、无函数单元**）从纯尺寸臂升格。既有七个语言码的每一行判决字节不变；新增人口改变分数基数，发版声明「分数与 1.7.x 不可比」。

## 1. 范围与裁定

| 语言 | 扩展名 | 文法 crate（crates.io 最新稳定版，2026-09-24 查） | ABI | 臂 | 语言码 | 命名规范 | CC 外部对照 |
|---|---|---|---|---|---|---|---|
| C | `c` | tree-sitter-c 0.24.2 | 15 | 判决·代码 | 15 | Any | lizard |
| C++ | `cpp cc cxx hpp hh hxx h inl` | tree-sitter-cpp 0.23.4 | 14 | 判决·代码 | 16 | Any | lizard |
| Lua | `lua` | tree-sitter-lua 0.5.0（tree-sitter-grammars 组织） | 15 | 判决·代码 | 17 | Any | lizard |
| Java | `java` | tree-sitter-java 0.23.5 | 14 | 判决·代码 | 18 | MixedCaps | lizard |
| Ruby | `rb rake gemspec` | tree-sitter-ruby 0.23.1 | 14 | 判决·代码 | 19 | Snake | lizard |
| R | `R r` | tree-sitter-r 1.3.0（r-lib 组织） | 14 | 判决·代码 | 20 | Any | 无（D0） |
| HTML | `html htm` | tree-sitter-html 0.23.2 | 14 | 判决·文档 | **10（既有码，翻 scan_only 位）** | Any | 无 |

- **语言码追加式**（lang.rs 头注 RM15）：六个新变元追加在 `Yaml = 14` 之后，`Lang as i64` 冻结位不重排；HTML 沿用 10，只把 LANGS 表该行的 scan_only 列翻为 false——码 10 此前从未上过 wire，翻位不改任何既有 golden。`judged_mask()` 由 `0x7F` 变为 `0x1F847F`（位 0–6、10、15–20）。七个 crate 全部接受核心 0.27 的 ABI 13–15 窗口，与 Haskell 的 `hs_grammar_pin` 同一条门。
- **`.h` 归 C++ 文法**：tree-sitter-cpp 是 tree-sitter-c 的超集，C 头文件在其下零 ERROR（§10 实探，含 `int class;` 这种 C 合法而 C++ 保留字的字段名，仍只是 `field_identifier`）；反之 Qt / LLVM / Chromium 式 C++ 项目把类体写在 `.h` 里，用 C 文法解析会整块 ERROR、方法从度量里消失。代价是纯 C 项目的 `.h` 挂 cpp 码：ledger 多一行语言，S2 混流轴读的是命名模式分布而非语言（structure/judge.rs 批-7 勘误），无判决影响。两文法共享一张 LangSpec 表（TS/TSX 先例），C++ 独有 kind 在 C 树里永不出现。
- **纯尺寸臂**收窄为 js/mjs/cjs/jsx、css/scss/less、vue、svelte、sh/bash、yml/yaml。HTML 内嵌的 `<script>`/`<style>` 是 `raw_text`，不解析——JS/CSS 仍在尺寸臂，本册不动。
- 不在本册：`.C`/`.H` 大写扩展、`.Rmd`/`.qmd`、`.erb`/`.haml`、Objective-C、模板语言；C/C++ 宏展开与预处理条件求值（D1）；Ruby `private_constant`（D15）；R `NAMESPACE`（D16）。

## 2. 不变量：一处权威、一条新谓词、三处不动

- `Lang::judged_path` 仍是唯一边界权威。structure / churn / fourclass / mention / tombstone / erase / score 全部经它分派，六个代码语言在这些面上**零改动**流入；`erase` 的 `language_unresolved` 理由位自动覆盖新阶梯。
- 新谓词 `Lang::fingerprints(self) = grammar().is_some() && self != Html`：T1/T2 token 流（`tokens::stream`、索引刷新）、T3 单元宇宙与 guard 的克隆探针改读它；站点、docdup 段、节锚等其余文法消费者仍读 `grammar()`。理由：HTML 叶结点没有标识符，`tag_name`/`attribute_name` 按 kind 取哈希，任意两页都会成为一对克隆。
- `MENTION_WHOLE_RUN_EXTS` 不动（Ruby 的 `$stdout` 走并集臂，两片都发 = 更多提及 = 安全方向）；`MENTION_REV = 2`、`TOKENIZER_REV = 3` 不动（既有语言的 token 流逐字节不变，新语言的行是新行）；索引 schema 16 不动（无表形变化）。
- 走查表：`BUILTIN_EXCLUDES` 已含 `vendor/ build/ target/ dist/ node_modules/`，Java 的 `target/` 与 Gradle 的 `build/` 已被覆盖；新增 R 的 `renv/`、`packrat/` 与 Ruby 的 `.bundle/`。

## 3. wire 契约与版本

| 项 | 今日 | 改动 | 升版 |
|---|---|---|---|
| scan/1 `naming` 行 | 核 `CE.Scan.namingShape` 判 `lang > 6` 为「lang outside the judged set」 | 请求加性键 `judgedMask`（缺席 = 旧集 `0x7F`，行为逐字节同旧）；核按 `testBit` 校验；应答回显该值供 Rust 钉漂移 | proto 7.1.0 → 7.2.0（minor，VERSIONING §2 放宽=minor） |
| graph/1 `unres` 行 | `CE.Graph.Contract.unresRow` 同一常量 6 | 同一 `judgedMask` 键；node 行 `[lang,kind,roles]` 本就只查非负；`roleBits` 表加角色位 8「编译单元」（§8） | 同上 |
| verdict/1 `judgedMask` | 已是 knob 回显（2.29.0 H1 slice 2） | 值随 LANGS 表移动，围栏不由它触发；判决人口 S 变 → `CE_ACCEPT_BASELINE=1` 具名重立一次 | 发版声明 score migration |
| docdup/1 段 kind | `KIND_NAMES = [md_para, comment_block, docstring]` | 追加 `html_text = 3`（位置冻结、只追加）；核若校验 kind 上界同批放宽 | `DOCDUP_REV` +1 |
| 图存储 | GRAPH_REV 15 | `store::KINDS` 追加十二个标签（§8）、六条阶梯、六个 conv 产者、三种新解析器配置（`compile_commands.json`、`DESCRIPTION`、`*.gemspec`） | GRAPH_REV 16 |
| 事实面 | `count:grammars` 6、`count:langs` 7 | `Lang::grammar` Some 臂 13、`judged_mask` 置位 14；`count:grammars` 那条 debt（「Lang 无变元迭代器」）借本批偿付：LANGS 表即迭代器 | docs-facts 重派生 |
| 包 | codeeraser 1.7.4 | 1.8.0：judged 人口变 = 分数不可比（1.4.0 先例） | minor + 声明 |

golden：scan（naming 行带码 15–20）、graph（unres 行）、docdup（kind 3）各加一对请求/应答，CE_BLESS 纪律重生；`contracts/eval/pre-haskell-members-v1.json` 的子集门保持绿（新语料只加成员）。`Version.hs` 的条目按「替换不追加」规则换成 7.2.0 一段。

## 4. 度量表（LangSpec，按语言列）

| 字段 | C / C++（共表） | Lua | Java | Ruby | R |
|---|---|---|---|---|---|
| fn_kinds | `function_definition` | `function_declaration` `function_definition` | `method_declaration` `constructor_declaration` `compact_constructor_declaration` | `method` `singleton_method` | `function_definition` |
| 单元名（name_via） | `Declarator`：沿 `declarator` 链下探 `pointer_declarator`/`reference_declarator`/`parenthesized_declarator` → `function_declarator.declarator` → `identifier`/`field_identifier`/`qualified_identifier`/`destructor_name`/`operator_name`；类内定义拼 `Class::name`，与类外 `K::b` 同形 | `name` 字段文本原样（`M.method`、`M:colon`）；`function_definition`：父 `field` 取其 `name`，父 `expression_list` 取祖父 `assignment_statement.variable_list` 同序位，否则 `(anonymous)` | `name` 字段 | `name` 字段（`identifier`/`operator`/`setter` 文本，`name=`、`ok?`、`<=>` 原样）；`singleton_method` 拼 `(self) make`（Go `(T) add` 先例，`object` 文本入括号） | `Assign`：父 `binary_operator` 的 `operator` 为 `<-` `=` `<<-` `:=`† 取 `lhs` 文本（含反引号名、`x$y$z`），为 `->` `->>`† 取 `rhs`；其余（`lapply(x, function…)`、`setMethod`、具名实参）`(anonymous)`——**`name` 字段持的是 `function`/`\` 关键字记号，不可取** |
| 形参 | 解析到的 `function_declarator.parameters`（`parameter_list`）具名子数；`(void)` 记 0（D12） | `parameters` 字段 | `parameters` 字段（`formal_parameters`） | `parameters` 字段（`method_parameters`） | `parameters` 字段，**只数 `parameter` kind**（`comma` 是具名结点） |
| cc_kinds | `if_statement` `for_statement` `for_range_loop` `while_statement` `do_statement` `case_statement` `conditional_expression` `catch_clause` | `if_statement` `elseif_statement` `for_statement` `while_statement` `repeat_statement` | `if_statement` `for_statement` `enhanced_for_statement` `while_statement` `do_statement` `switch_label` `ternary_expression` `catch_clause` | `if` `unless`† `elsif` `while` `until` `for` `when` `in_clause` `rescue` `conditional` `if_modifier` `unless_modifier` `while_modifier` `until_modifier`† `rescue_modifier` | `if_statement` `for_statement` `while_statement` `repeat_statement` |
| cc_operators | `&&` `\|\|`（C++ 替代记号 `and` `or`†） | `and` `or` | `&&` `\|\|` | `&&` `\|\|` `and` `or` | `&&` `\|\|`（D9） |
| coc_nesting | 同 cc 去 `case_statement`，加 `switch_statement` | `if_statement` `for_statement` `while_statement` `repeat_statement` | 同 cc 去 `switch_label`，加 `switch_expression` | 同 cc 去 `elsif` `when` `in_clause`，加 `case` `case_match` | 同 cc |
| coc_flat / else | `else_clause`（具名结点；包 `if_statement` 时让位给内层 if） | `elseif_statement` `else_statement` | 无结点：`alternative` 字段直接是 `block`/`if_statement`（Go 形） | `elsif` `else` | 无结点：`alternative` 直接是表达式（`braced_expression`/`call`…） |
| coc_nest_only | `lambda_expression`（C++） | 无（匿名函数是独立单元） | `lambda_expression` | `lambda` `block` `do_block` | 无 |
| coc_jump + label_kinds | `goto_statement`（恒带 `label: statement_identifier`）；`break`/`continue` 无标签不计 | `goto_statement`（子 `identifier`，无字段名） | `break_statement` `continue_statement`，标签为裸 `identifier` 子结点 | 无 | 无 |
| comment_kinds | `comment` | `comment`（复合：start/content/end；按 kind 整棵跳过即可） | `line_comment` `block_comment` | `comment`（`=begin…=end` 亦是） | `comment`（roxygen `#'` 亦是） |
| literal_delims | `"` `'` `character` `R"`（`string_content`/`raw_string_content`/`escape_sequence`/`number_literal` 由通用规则吃） | `"` `'` `[[` `]]`（长括号任意等级 kind 恒此二者） | `"` `"""`（`string_fragment`/`multiline_string_fragment`/`character_literal` 通用规则） | `"`（所有引号记号 kind 恒 `"`，含 `'`、`%q(`）、`` ` ``、`heredoc_beginning` `heredoc_content` `heredoc_end` | `string_open` `string_close`（具名 kind 亦可入表） |
| call_kinds / callee 字段 | `call_expression` / `function` | `function_call` / **`name`** | `method_invocation` / **`name`**（接收者在 `object`） | `call` / **`method`**（接收者在 `receiver`） | `call` / `function` |
| call_name / member / self / scopes / import | `identifier` / `field_expression` `qualified_identifier` / `this` / `class_specifier` `struct_specifier` `union_specifier` / `using_declaration` | `identifier` / `dot_index_expression` `method_index_expression` / `self` + 调用者自身的表前缀（`M.`/`M:`） / 无 / 无（D21） | `identifier` / `object` 字段在场 / `this` `super` + 封闭类名（静态调用 `Probe.id()`） / `class_body` `interface_body` `enum_body` `annotation_type_body` / 无 | `identifier` / `receiver` 字段在场 / `self` / `class` `module` `singleton_class` / 无 | `identifier` / `extract_operator` `namespace_operator` / 无 / 无 / 无 |

HTML 的 LangSpec 为空表（MARKDOWN 同款），只提供 `comment_kinds = [comment]` 给 selfref 的注释判断。四条**机制扩展**（皆向后兼容，既有五语言电池同批证明逐值不变）：(a) `functions::name_of` 的父结点表 `variable_declarator|pair|assignment` 扩为 §4「单元名」列的三种新读法；(b) `param_count` 加「解析到的声明子结点」与「只数某 kind」两个旋钮；(c) `calls.rs` 的 `CALLEE = "function"` 常量变为 LangSpec 字段 `callee_field`，接收者由 `receiver`/`object`/`table` 字段或 `call_member_kinds` 结点给出，`call_self_words` 允许「调用者自身容器名」这一动态词；(d) `cognitive.rs` 两处泛化——`field_else_bonus` 从 `alternative.kind == "block"` 改为「`alternative` 非 if 类结点即 +1」（Java 单语句 else、R 的表达式 else 今日都数不到），`starts_with("if")` 改为精确的 `if_kinds` 表（Ruby `if_modifier` 会误入 else-if 路径）；Go 的 alternative 恒为 `block`/`if_statement`，五语言等价由既有 `sonar_whitepaper` 电池证明。

## 5. 分歧登记册草案（落码时进 `contracts/fixtures/crosscheck/DIVERGENCES.md`，与电池 why 串互为索引）

| # | 构造 | 裁定 | 依据 / 先例 |
|---|---|---|---|
| D0 | R、HTML | 无外部 CC/CoC oracle：一切映射为仓内立场，电池是唯一执行者 | Haskell D0 |
| D1 | C/C++ 预处理条件 `preproc_if/ifdef/elif` | 不计 CC/CoC——编译期分支不是控制流；lizard 计 `#if`，rust-code-analysis 不计 | 分歧如实登记 |
| D2 | C/C++ `case_statement`、Java `switch_label` | 一 kind 两义（`case` 与 `default`），全计：case 是全函数、default 是真路径；lizard/gocyclo 不计 default | Rust match_arm、Haskell D2 |
| D3 | C++ lambda、Java lambda、Ruby `block`/`do_block`/`lambda` | 吸收进宿主、只抬嵌套；Lua/R 的匿名函数是**独立单元**（它们是该语言的函数声明形），无名者 `(anonymous)` | 白皮书 p.13；Go func_literal 与 TS arrow 两先例各取其一 |
| D4 | 三元 `?:`（C/C++/Java/Ruby `conditional`） | 结构增量带嵌套罚 | TS ternary 先例 |
| D5 | 带标签跳转 | C/C++ `goto` 恒带标签 +1；Java `break L`/`continue L` +1；Lua `goto` +1；Ruby/R 无 | 白皮书 p.8 |
| D6 | Ruby 修饰符形（`x if y` 等五种） | 按结构增量（带嵌套罚）；`rescue` 按 catch；`unless` 按 if；`elsif`/`else` 平 +1 | SonarRuby 立场 |
| D7 | Ruby 无括号裸名调用 | 文法里是 `identifier` 非 `call`：递归边少计（安全方向） | calls.rs 少计原则 |
| D8 | Lua/R/Ruby 以调用承担的控制流（`pcall`、`tryCatch`、`ifelse`、`switch()`、`each` 块） | 语法层不可见，无增量 | Haskell D5「try 透明」 |
| D9 | R `&`/`\|` | 向量化运算符不计，仅 `&&`/`\|\|`；`repeat` 按循环 | 短路才是分支 |
| D10 | R `1L`/`3i`、Lua/R 字符串 | `integer`/`complex` 是复合结点（数字无叶），T2 只见后缀记号；`string_open/close` 入表使 `'a'` ≡ `"a"` | 实探事实 |
| D11 | C `"a" "b"` | `concatenated_string` 下两个 `string_literal` 父 → 两个 LIT | 片段只在同父合并（M2 D3） |
| D12 | C `f(void)`、K&R 声明 | 记 0 形参（lizard 同）；K&R 不建模 | — |
| D13 | C++ 类外定义 `K::b` | 访问级写在类体（另一结点或另一文件）：可见性取安全侧 = 导出 | 「只读本文件」原则 |
| D14 | Java 包私有 / 接口成员 | 无修饰 = bit0\|bit2（包内导出且受限）；`interface_body` 下隐式 public | Rust `pub(crate)` 的 bit2 语义 |
| D15 | Ruby 顶层 `def` | 技术上是 Object 的私有方法，按导出（安全侧）；`private_constant` 不读 | 误判「死」比漏判贵 |
| D16 | R 可见性 | 文件有 roxygen 串则 `#' @export` 为准，否则点号约定（`.name` 内部）；`NAMESPACE` 是另一文件不读——Python `__all__` 前的同一路，后置 | visibility/mod.rs 原则 |
| D17 | Java 同包引用、Ruby 常量自动加载 | 不经 import/require：以 `type_ref`/`const_ref` 名字站点补（§8），否则每个只被同包/自动加载引用的文件都成「未引用」 | 存活判决可用性 |
| D18 | C/C++ 翻译单元、R 包 `R/` | `.c/.cc/.cpp/.cxx` 从不被 include：按「编译单元」角色入口，死候选是无人 include 的头；R 包的 `R/*.R` 由加载器整体 collate：按声明目标角色（DESCRIPTION 即清单） | roleBits 是核的表 |
| D19 | C++ `and`/`or`、`attribute_specifier`、Ruby `unless`/`until_modifier`、Java 带实参 `annotation` | 本轮样本未含，表内先列（†），电池落地时实证 | 诚实登记 |
| D20 | HTML 文本 | `text` 叶被内联元素切开，段 = 块级元素文本后代的串接；`pre/code/script/style/textarea` 与 `comment` shed 计数 | md 围栏与 HTML 注释 mask 先例 |
| D21 | Lua 局部绑定遮蔽（`local helper = other.helper`） | 不建模（`call_import_kinds` 空），递归边可能多计——语料实测命中则补 `variable_declaration` 遮蔽读法 | ignore 仓 `use` 先例 |
| D22 | 命名轴 | C/C++/Lua/R 无社区统一规范 → Any（style 0）；Java MixedCaps；Ruby Snake（`?`/`!`/`=` 后缀与运算符名不含大写下划线，不受罚） | 规范出处各语言风格指南 |
| D23 | `Member` 类别 | 不扩展到 Java/Ruby/C++（会让未提及顾问对它们空转；Python 的 Member 有其动态访问缘由），协议名走 Protocol 表 | 顾问要锋利 |

## 6. 单元键与声明域（`fourclass::kinds::extra`，只列带 `name` 字段者；`type_definition` 的名在 `declarator` 字段，表加一列字段名即可）

- C：`preproc_def`、`preproc_function_def`（宏是真声明）、`struct_specifier`、`enum_specifier`、`union_specifier`（`name: type_identifier`）、`type_definition`（`declarator`）；函数原型 `declaration` 不是单元（头文件对名字的拼写本身就是「提及」）。
- C++：以上加 `class_specifier`、`namespace_definition`（有名者）、`alias_declaration`、`enum_specifier`（`enum class` 同 kind）；`template_declaration` 是包裹结点，走查本就下探，无需登记。
- Java：`class_declaration`、`interface_declaration`、`enum_declaration`、`record_declaration`、`annotation_type_declaration`；字段不入（Go `const_spec` 同理，类内引用为主）。
- Ruby：`class`、`module`（`name: constant`）；`singleton_class` 无名不入。Lua、R：无（表与 S4 类都是值/调用）。
- HTML：带 `id` 属性的元素是 `KIND_SECTION` 单元，键 `#id`，粒度 GRAN_SECTION（md 标题先例）；四分类、churn、拆分缝据此对齐。
- 同名同元数（C++ 重载、Ruby 多次 `def`）由既有 `with_nth` 区分（Rust impl 同胞先例）。

## 7. 可见性（`fourclass/visibility/<lang>.rs`，每位只读本文件本结点）

| 语言 | bit0 导出 | bit1 作用域开放 | bit2 受限 |
|---|---|---|---|
| C | 无 `storage_class_specifier` 为 `static`（`static inline` 私有，裸 `inline` 导出） | = bit0 | 0 |
| C++ | 类内：最近前置 `access_specifier` 兄弟（`class` 默认 private，`struct`/`union` 默认 public）public→1、private→0；类外：`static` 或匿名 `namespace_definition`（无 `name`）祖先→0，否则 1（D13） | 祖先链无匿名命名空间、无函数体，每个封闭类体在其父体内的访问级为 public；`template_declaration` 透明 | `protected` |
| Lua | `local`（`function_declaration` 带 `"local"` 子记号，或 `function_definition` 处于 `variable_declaration` 之下）→0；全局函数、表字段 `M.f`/`M:f`、非 local 赋值→1 | 无封闭函数体 | 0 |
| Java | `modifiers` 含 public→1；protected 或无修饰→1；private→0；`interface_body` 下无修饰→1 | 每个封闭 class/interface/enum/record 体按同一规则开放，且无 `block`（方法体）祖先——局部类与匿名类成员为 0 | protected、包私有（D14） |
| Ruby | 节状态机读同一 `body_statement` 的前序兄弟：裸 `identifier` `private`/`protected`/`public` 切换（默认 public）；`private def x`（方法作实参）、`private :x`/`private :a, :b`（def 之后的符号表）、`private_class_method :x`（单例）逐名生效；`module_function` 后的 def 视为导出；`singleton_class` 体同规则；顶层 def→1（D15） | 类/模块不封闭，方法体封闭 | protected |
| R | 文件含 roxygen 串（`#'` 注释）：紧邻上方串内有 `@export`→1，否则 0；无 roxygen 串：名不以 `.` 开头→1（D16） | 无封闭 `function_definition` | 0 |
| HTML | 节锚恒 `HTML_VIS = EXPORTED \| SCOPE_EXPORTED`（MARKDOWN_VIS 同款） | | |

## 8. 引用站点、阶梯、根与角色

站点（`graph/spec.rs`；标签追加进 `store::KINDS`，位置冻结）：

| 语言 | 结点 | 标签 | 说明符来源 |
|---|---|---|---|
| C/C++ | `preproc_include` | `include` | `Field("path")`：`"x.h"` 去引号，`<x>` 保留尖括号，阶梯据此分形 |
| Java | `import_declaration` | `import` / `import_star` | 新 `FirstNamed`：首个具名子结点（`scoped_identifier`/`identifier`）文本，无字段名；有 `asterisk` 兄弟→`import_star`；`static` 记号→spec 前缀 `static ` |
| Java | `type_identifier`（排除本文件声明的类/接口/枚举/记录/注解名与 `type_parameter` 名——文件局部事实，仍是解析无关的检测） | `type_ref` | 结点文本 |
| Ruby | `call` 且 `method` 文本 ∈ {require, require_relative, load, autoload} | `require` / `require_relative` / `load` | 新 `CallArg`：`argument_list` 内首个 `string` 的 `string_content`（autoload 取第二实参） |
| Ruby | 表达式位的 `constant` / `scope_resolution`（排除 class/module 的 `name`、`superclass` 保留、本文件定义的常量排除） | `const_ref` | 完整路径文本 `Outer::Probe` |
| Lua | `function_call` 且 `name` 文本 ∈ {require, dofile, loadfile} | `require` / `load` | `CallArg`（`arguments` 内首个 `string`；`require "x"` 无括号形亦是 `arguments`） |
| R | `call` 且 `function` 文本 ∈ {source, sys.source} | `source` | `CallArg` |
| R | `call` 且 `function` ∈ {library, require, requireNamespace, loadNamespace}（实参 `identifier` 或 `string`）；`namespace_operator.lhs` | `library` | 实参 / lhs 文本 |
| HTML | `attribute`（`start_tag`/`self_closing_tag` 内）按 (tag_name, attribute_name) | `href`（a/area/link/base/use）、`src`（script/img/iframe/embed/source/track/video/audio、object·data、video·poster）、`srcset`（逗号表，每候选一站，`nth` 序位）、`action`（form） | 新 `Attr`：`attribute_value` 文本，只解 `&amp;` |

阶梯（`graph/ladder/<lang>.rs`；候选只来自被走集，多候选即 ambiguous，External 是正确终态）：

| 语言 | R1 | R2 | R3 | External / 拒绝 |
|---|---|---|---|---|
| C/C++ | `"x"`：引用文件同目录 join | `[graph.search_roots] c` 声明目录逐一（两目录命中不同文件 = ambiguous_root） | 在域 `compile_commands.json`（解析器配置，字节入 resolve_key）中该文件条目的 `-I`/`-iquote`/`-isystem` 目录 | `<x>` 在 R2/R3 无命中→External（系统/工具链头）；`"x"` 无命中→OutOfScope；**永不按 basename 搜树** |
| Java | 源根 = 每个被走 `.java` 的 `package` 声明反推（目录去掉包路径；sweep memo 一次）；`import a.b.C`→各根下 `a/b/C.java`（跨根异文件 = ambiguous_root） | `import_star`→包目录 ResolvedPackage（须直接持有被走 `.java`）；static/嵌套类：最长文件前缀（`a/b/C.java`，Python `__init__` 降级先例） | `type_ref`：同包同目录 `<Name>.java`（构造上唯一）；否则被 `import_star` 的在域包持有 `<Name>.java` | `java.lang` 名与 JDK 模块表（机器生成，CPython/Go/Haskell 表先例）→External；其余 OutOfScope |
| Lua | `require "a.b"`→`a/b.lua` 或 `a/b/init.lua`，根集 {仓根, `src`, `lua`, 声明根}（跨根异文件 = ambiguous_root） | `dofile`/`loadfile`：引用文件同目录，再仓根 | — | 标准库名表（`string` `table` `os`…）→External；其余 OutOfScope |
| Ruby | `require_relative`：同目录 join + `.rb`；`load`：同目录再仓根 | `require`：根集 {`lib`, gemspec `require_paths`（在域配置）, 声明根} + `.rb` | `const_ref`：Zeitwerk 映射 `Outer::Probe`→`outer/probe.rb`，根集 {`app/*/` 各直接子目录, `lib`, 声明根}；`concerns` 目录按 Zeitwerk 默认折叠 | 标准库/默认 gem 名表与核心常量表→External；其余 OutOfScope |
| R | `source("x.R")`：同目录 join，再仓根（R 工作目录 = 项目根惯例），再声明根 | `library(x)`/`pkg::`：在域 `DESCRIPTION` 的 `Package: x`→ResolvedPackage（其目录） | — | 无在域包→External（CRAN 构造上域外，无需名表） |
| HTML | 相对：同目录；`/abs`：`[graph.search_roots] html`（默认仓根）；`?query` 去掉；目录目标→`dir/index.html`（Web 服务器语义，与 md「目录=包」不同，明说） | 跨文件 `#frag`：目标页 `id` 集（零/多命中降级到文件级，md slug 先例）；裸 `#frag`：本文件节声明如实取 | — | 任何 scheme、`//`、`mailto:`/`tel:`/`javascript:`/`data:`→External（md R5） |

- 边种类：HTML `href`/`action` 到页面 = 链接边（存活）；`src`/`srcset`/`link rel=stylesheet|icon|preload` 到资产 = asset 边（核的 `assetKind` 惰性规则原样适用，目标是幻影结点）。`type_ref`/`const_ref` 与 import 边落同一目标时由 BTreeSet 去重。
- 配置：`[graph.search_roots]` 表，键 = 语言名（`c`〔cpp 共用〕、`lua`、`ruby`、`java`、`r`、`html`），值 = 目录数组，交被走集；声明了不存在的目录按名拒绝（`crate_roots` 先例）；进 resolve_key；canonical 规则 3/4 下未声明者不动指纹。
- 角色（`deadcode/flags.rs`，事实；入口决定仍是核的 roleBits）：命名入口加 `main.c main.cc main.cpp Main.java main.lua conf.lua app.R server.R ui.R global.R index.html 404.html config.ru`；目录入口加 Ruby `app/ config/ db/ bin/ exe/ script/`、Lua `plugin/ ftplugin/ after/ colors/ syntax/ autoload/`、R `inst/ vignettes/ data-raw/ exec/ demo/`（Java 无：类以名字被引用）；新位 `ROLE_UNIT = 1 << 8` 给 `.c .cc .cpp .cxx`（D18）；`ROLE_DECLARED` 加 R：`DESCRIPTION` 所在目录的 `R/*.R`（`Collate:` 有则按其表），`targets::Declared::gather` 多读一种清单；测试模式加 `*_test.c/.cc/.cpp`、`*Test.java`、`*Tests.java`、`*_spec.rb`、`*_test.rb`、`test_*.rb`、`*_spec.lua`、`*_test.lua`、`test-*.R`、`test_*.R`、目录 `testthat/`——flags.rs `is_test` 与 conv/name.rs `test_file` 两处产者借此合为一表（查重棘轮会先抓到孪生）。

## 9. 提及规约、文档重复、自提及

- conv AST 半：C/C++ `Ffi` = `linkage_specification` 祖先（`extern "C"`，实探）与 `__attribute__((visibility("default")))`/`__declspec(dllexport)`（`attribute_specifier` 文本†）；Java `Registration` = `modifiers` 内任一 `annotation`/`marker_annotation`（TS decorator 同款；`@Override` 亦是——被分派抵达，沉默正确）；其余语言 AST 半为 0。名字半 `Main` 加 C/C++/Java 的 `main`。
- `Protocol` 名表（每语言一段常量，装载器替作者拼写的名字）：Java `toString equals hashCode compareTo compare run call get accept apply test close iterator hasNext next readObject writeObject readResolve writeReplace finalize clone valueOf values doGet doPost doPut doDelete init destroy service`；Ruby `initialize to_s to_str to_a to_ary to_h to_hash to_proc to_sym inspect each <=> == eql? hash call method_missing respond_to_missing? coerce === =~ [] []= << included extended prepended inherited method_added perform`，加路径 × 名：`app/controllers/**` × `index show new create edit update destroy`；Lua 元方法 `__index __newindex __call __tostring __eq __lt __le __add __sub __mul __div __mod __pow __unm __idiv __band __bor __bxor __shl __shr __bnot __concat __len __gc __close __mode __name __metatable __pairs` 与插件惯例 `setup config on_attach`、`main.lua`/`conf.lua` 内的 `love.*` 回调（`load update draw keypressed…`）；R `.onLoad .onAttach .onUnload .onDetach .Last.lib .First .Last server ui shinyServer shinyUI run_app` 与 S3 方法形 `<generic>.<class>`（generic ∈ 一份 base/stats 泛型表 `print format summary plot as.character as.data.frame length names c mean toString str update predict residuals coef anova`）；C/C++ `main DllMain WinMain wmain _start JNI_OnLoad napi_register_module_v1 LLVMFuzzerTestOneInput` 与前缀 `PyInit_ luaopen_ Java_`。
- docdup：注释 kind 走 §4；骨架前缀表加 Javadoc/Doxygen/YARD/roxygen/LDoc 标签（`@brief @return @see @since @author @tparam @treturn @usage @examples @export @importFrom @rdname @details @inheritParams @describeIn` 与 Doxygen 反斜杠形 `\brief \param \return`）——DOCDUP_REV 同批；HTML 段：块级元素 `p li dt dd td th h1–h6 blockquote figcaption caption summary label legend title` 的 `text` 后代串接为一段（内联 `code` 按 md 行内代码 mask），shed 计数分 `code`（pre/code/textarea）与 `script`（script/style）两栏并入既有 `MdShed`；kind 3 `html_text`；带 why 尾注的 `ce:allow(docdup)` 标记写在 HTML 注释里即可用（README 末行已这么写，`allow.rs` 一处文法不改）。
- selfref（二级解释器区域，安全方向 = 更多提及）：六个代码语言的字符串字面量整体——Ruby `send(:x)`/插值、Lua `_G["x"]`/`require`、R `get("x")`/`do.call`、Java 反射、C/C++ `dlsym`/方法表，皆是真实的字符串分派惯用法，TS 计字符串的同一理由；文档注释串的代码块加四形：Javadoc `<pre>`/`{@code}`、Doxygen `@code…@endcode`、roxygen `@examples` 段（到下一 `@tag`）、LDoc `@usage`；HTML：属性值与 `script_element` 的 `raw_text`（本页 `href="#id"` 是对本页节锚的自提及）。

## 10. 实探记录（2026-09-24，tree-sitter 0.27.0，探针 crate 在会话草稿目录，两轮十四个样本全部零 ERROR）

| 文法 | 样本覆盖的构造 |
|---|---|
| C 0.24.2 | include 两形、`#define` 两形、`#ifdef`、static、if/else if/else、for/while/do、switch/case/default、`?:`、goto/标签、struct/typedef/enum/union、char/string/拼接串、函数指针、数组形参、`(void)`、原型、`p->cb(1)`/`(*p->cb)(2)` |
| C++ 0.23.4 | `extern "C"` 两形、有名/匿名/嵌套 namespace、class 三访问级与 struct 默认、构造/析构/const 成员/static 成员/模板成员/运算符、类外定义 `K::b`、try/catch 两臂、lambda 两形、范围 for、`if constexpr`、`using` 两形、`enum class`、typedef/alias、模板 struct、raw string、`nullptr`、`this->`、`obj.m()`/`ptr->m()`；另以 C 头文件（`int class;`）验证超集性 |
| Lua 0.5.0 | `require` 两形、dofile、三种注释、local/全局/`M.f`/`M:f`/匿名/表字段/多重赋值函数、if/elseif/else、数值与泛型 for、while/repeat、goto/标签、四种字符串与 `[==[`、`#`/`M[1]`/`M["a"]`、嵌套 local function |
| Java 0.23.5 | package、四种 import、Javadoc、注解、字段三访问级 + 包私有、构造、泛型静态方法、`@Override`、if/else if/else、三种 for、do/while、语句与箭头 switch、三元、多重 catch/finally、标签 break/continue、lambda、`this.`/`super.`/静态调用、匿名类、文本块、内部/局部/嵌套/静态嵌套类、interface/enum/record/@interface |
| Ruby 0.23.1 | require/require_relative/load/autoload、两种注释、module/class/superclass、`include`/`attr_reader`、`def self.`、访问节三形（裸词、`private def`、`private :sym`）、`private_class_method`、`module_function`、`class << self`、if/elsif/else/`unless_modifier`/`if_modifier`/`while_modifier`/`rescue_modifier`、while/until/for、case/when/else、case/in、三元、begin/rescue/retry/ensure、`and`/`or`/`&&`/`\|\|`、lambda/proc/块两形、`define_method`、字符串七形与 heredoc、`$stdout`、`Outer::Probe.make`、`<=>`/`name=`/`ok?`、方法链 |
| R 1.3.0 | library/require/requireNamespace/source、roxygen、五种赋值形（`<-` `=` `<<-` `->` `\(x)`）、反引号名、`.hidden`、S3 命名、setGeneric/setMethod、if/else if/else、for/while/repeat、next/break、`&`/`\|`/`&&`/`\|\|`、tryCatch/switch/ifelse、四种字符串与 `r"(…)"`、`1L`/`3i`、`$`/`@`/`::`/`:::`、默认实参与 `...`、嵌套函数、`helper(1)(2)`、`x$y$z <- function` |
| HTML 0.23.2 | doctype、注释、head 元数据、`link href`、`script src`/内联脚本/内联样式、`id`/`class`/`data-*`、被 `<em>` 切开的段落、四种 href（相对带 `#`、绝对 URL、裸 `#`、根相对）、`img src`+`srcset`、列表、空元素、form action、无引号属性、自定义元素、`pre>code`、内联 svg `<use href>`、`video poster`/`source src` |

表 §4 所依赖的关键事实（各语言一行，全部来自上表样本）：C 的 `function_definition` 无 `name` 字段而名在 `declarator` 链底、`else_clause` 是具名结点、`case_statement` 兼 `default`、`goto_statement.label: statement_identifier`（与 TS 同 kind 名）、`char_literal` 三片、`preproc_include.path` 为 `string_literal` 或 `system_lib_string`；C++ 类内方法 `declarator: field_identifier`、类外 `qualified_identifier{scope,name}`、`access_specifier` 是 `field_declaration_list` 的兄弟、`lambda_expression{captures,declarator,body}`、`linkage_specification{value,body}`、匿名 `namespace_definition` 无 `name`、`raw_string_literal` 五片；Lua 的 `function_call.name`、`local_declaration:` 是字段名而 `"local"` 是子记号、`if_statement.alternative` 可多个、`comment` 复合、长括号 kind 恒 `[[`/`]]`、`goto` 的标签是无字段名 `identifier`；Java 的 `import_declaration` 子结点无字段名而 `asterisk` 是兄弟、`method_invocation{object?,name,arguments}`、`switch_expression` 统管两形且 `switch_label` 兼 `default`、`if_statement.alternative` 直接是 `block`/`if_statement`、标签是裸 `identifier`；Ruby 的 `call{receiver?,method,arguments?,block?}`、`method.name` 三形、引号记号 kind 恒 `"`、裸 `private` 是 `identifier` 而带实参是 `call`、`=begin` 块是 `comment`；R 的 `function_definition.name` 持关键字记号、赋值是 `binary_operator{lhs,operator,rhs}`、`parameters` 的 `comma` 具名、`argument{name?,value}`、`string{open,content,close}` 三具名 kind、`next`/`break` 具名叶；HTML 全树无字段名、`text` 被内联元素切开、`script_element`/`style_element` 持 `raw_text`。

## 11. 验收与门

- 交叉核对语料（钉 commit、SHA1 抽样规则不变，`SOURCES.md` 加七行，拍板项 10）：候选 C `lua/lua`、C++ `fmtlib/fmt`、Java `google/gson`、Lua `luarocks/luarocks`、Ruby `rack/rack`、R `tidyverse/stringr`、HTML 本仓 `site/`（自食）+ `h5bp/html5-boilerplate`。对照物 lizard 1.23.0（其 reader 列表含 C/C++/Java/Lua/Ruby，钉版时核实）；CoC 走 `sonar_whitepaper.rs` 的语言等价表（五 → 十一；R 无 switch/goto 的例题登记不可移植，Haskell D4 同款）；分歧全部落 `DIVERGENCES.md`（§5 即草案）。
- 图精度：每语言一份 graph-sample → slice → precision 三件套（M5-2 的仪器优先脊柱：站点宇宙先冻结，再有解析器），按 rung 出精度，达 M5-2 的门；`type_ref`/`const_ref` 单独出行——它们是新的站点类，精度须单独可归因。
- FPR：guard 的 T1/T2 探针对新语言首次生效，每语言一份 `fpr_replay` 回放（≤ 1 % 门，R4）；**未达门的语言不随该版发布**（逐语言发布门）。
- 自食：HTML 入判决后本仓 `site/`（八页）与 `gui/ui/index.html` 首次进 docdup 与图。预期：README ↔ 首页段落命中（本就同源）——真重复者改写，或加带 why 尾注的 `ce:allow(docdup)` 标记；孤页与断链由图报出；基线具名重立一次；`ce docdup --check` 与 `ce deadcode --check` 两条腿在实现分支上先跑绿再合。
- 性能：release strip 后七文法合计约 +6 MB（探针二进制 7.6 MB 含 tree-sitter 运行时与七文法；C++ 与 Ruby 文法各占大头），`PERF-BUDGET.md` 记一行；解析成本按 bench 七指标随版入列。
- 门：`cargo test/clippy/fmt`、`cabal test`、六条产品腿、golden 重生（CE_BLESS）、`pre-haskell-members` 子集门、`hs_grammar_pin` 扩为七文法 ABI 钉（`abi_version() ∈ 13..=15`）。

## 12. 文档与事实面（步 7 一次落完）

README 双语「范围」句与「语言」行（纯尺寸臂列表随之收窄）；官网八页中提语言集与文法数的句子；册 01（`literal_delims` 表、无文法语言段）、册 06（阶梯表加六行）、册 09（单元键）、册 13（Protocol/Ffi/Test 表与自仓普查行）、`methodology.md` 索引；`docs-facts.json` 两条计数与一条 debt；`docs-citations.json` 随 lang.rs / spec.rs 行移重签；`ce-toml.md` 加 `[graph.search_roots]`；`VERSIONING.md` 7.2.0 台账；`PERF-BUDGET.md` 体积行；NOTICE 随七个 MIT crate 重生；CHANGELOG `[Unreleased]` 块；计划横幅 v2.30 句与 §6 新轨步表；`plugin/README` 无需改（钩子语义不变）。

## 13. 分步（每步自带门，落码顺序）

| 步 | 内容 | 门 |
|---|---|---|
| 0 | 计划修正 v2.30（横幅句 + §6 新轨表）+ §14 拍板 → cc-memory 重锁 | 用户拍板 |
| 1 | 骨架：七 crate 钉版、`Lang` 六行 + HTML 翻位、`fingerprints()`、`judgedMask` 上 scan/graph 请求（proto 7.2.0）、核两处边界改读 mask、`Version.hs`/VERSIONING 台账、golden 重生、`grammar_pins` 门 | 既有五语言四语料对拍逐字节不动（DEP-TS 的 16 家族对拍先例）；五语言电池绿 |
| 2 | C/C++：LangSpec 表、`Declarator` 命名、类内键拼 `Class::`、可见性、include 站点/阶梯、`compile_commands.json` 配置、编译单元角色（核 roleBits）、`coc_c.rs` 电池、交叉核对、D 表落册 | 对拍全归因；D1/D2/D12/D13 各有电池行 |
| 3 | Java：表、`callee_field` 机制、包反推源根、`type_ref` 站点与 JDK 名表、注解 Registration、可见性、电池/对拍 | 同上；`type_ref` 精度单独出行 |
| 4 | Lua + Ruby + R（共享 `CallArg` 站点机制与 `Assign`/`LuaBind` 命名）：各自表、阶梯、可见性节状态机、Protocol 表、R 的 `param` 旋钮、电池/对拍 | 同上；Ruby 节状态机每形一电池行 |
| 5 | HTML：`Attr` 站点、`html.rs` 阶梯（复用 md 链）、节锚单元、docdup kind 3 与 shed、自食处理 | `site/` 八页孤页/断链为零或已豁免有据 |
| 6 | 评估：三件套 × 7、FPR 回放 × 7、`sonar_whitepaper` 扩表、cognitive.rs 两处泛化的等价证明 | 逐语言发布门 |
| 7 | 文档与事实（§12） | docs 门全绿、引文重签 |
| 8 | 发版 1.8.0：分数不可比声明、基线具名重立、bench 入列 | RELEASE.md 链 |

依赖：1 先于一切；2/3/4/5 只追加各自的行与文件，可并行；6 随各语言步收口；7/8 最后。可选拆两版：1.8.0 六个代码语言，1.9.0 HTML（拍板项 3）。

## 14. 待拍板

1. `.h` 归 C++ 文法（§1）——备选：`.h` 归 C，C++ 项目须改用 `.hpp`。
2. HTML 升格为文档类判决语言（docdup + 图 + 节锚，无指纹）——备选：维持纯尺寸臂，本册去掉 §5 D20、§8 HTML 行与步 5。
3. 一版（1.8.0 七语言）还是两版（代码语言先、HTML 后）。
4. 复杂度立场 D1（预处理不计）、D2（default 全计）、D3（Lua/R 匿名函数为独立单元）、D9（R 向量化运算符不计）。
5. 可见性安全侧 D13 / D14 / D15 / D16。
6. `type_ref` / `const_ref` 名字站点入图（D17）——不入则 Java/Ruby 的文件级存活判决不可用，须在 README 明示。
7. 编译单元入口角色与 R 包 `R/` 声明目标（D18；核 roleBits 表加一位）。
8. `[graph.search_roots]` 键形（每语言一数组）——备选：每语言各一键 `include_dirs` / `lua_paths` / `site_roots`。
9. 逐语言发布门 = FPR ≤ 1 %（未达门的语言不随版发布）。
10. 交叉核对语料候选（§11 七个仓）。
11. 版本号 1.8.0 与「分数与 1.7.x 不可比」声明。
12. `Member` 不扩展（D23）；Ruby 顶层 def 按导出（D15）。

## 15. 交接（本节随 v2.30 并入计划后删除）

- **本会话（2026-09-24，claude.ai/code 云会话）做了什么**：读码定位语言接入的全部触点（`scan/lang.rs`、`scan/spec*.rs`、`fourclass/kinds.rs`、`fourclass/visibility/`、`graph/spec.rs`、`graph/ladder/`、`graph/mounts.rs`、`graph/deadcode/flags.rs`、`mention/conv/`、`mention/selfref.rs`、`docdup/spec.rs`、`docdup/segments.rs`、`dedup/tokens.rs`，核侧 `CE/Scan.hs` 与 `CE/Graph/Contract.hs` 的两处 `lang > 6`）；查 crates.io 取七套文法的最新稳定版；探针两轮十四个样本；写本册；本册与探针分别以两个提交落在分支 `claude/loving-planck-n8sha8`，随后快进并入 `main`。
- **没做什么**：未改计划书、README、`Cargo.toml`、任何代码；未跑 `cabal test`、`cargo test`、六条自食腿——容器无 GHC（`downloads.haskell.org` 被出站代理以 403 拒绝），tests 子仓可克隆但每条门都要 `ce-core`。本册与探针文件按门的源码逐个核过（`docs_nav` 只读 `methodology.md` 目录表、`layout_tree` 只看顶层目录、`docs_lang` 不管 `docs/`、`mention_universe` 钉的是公式不是字面量），但没有跑过。
- **并入 main 后预期会红、请本地先重钉的门**：① `it/eval_mention.rs`（`parts::check_booklet`）——册 13 自仓普查行随树重取，新文件改变 U；② `it/site_roast.rs`——两首页自测块随判决人口移动。两者都是 `CE_BLESS=1` 重写（子仓 `it/eval_mention_parts/mod.rs` 与 `facts::block` 的说明；既往顺序 site_roast → docs_citations → eval_mention → facts）。③ `ce check` 若分数因新文件移动：容差内同定或 `CE_ACCEPT_BASELINE=1` 具名重立——新文件都在软线之下，不应新增违规行。④ `deadcode --check` 由本册顶部的 allow 行与本册到探针 README 的链接保活；`docdup` / `erase` / `dedup` 应绿（探针的 `grammar()` 每臂六个 token，不够一个块）。
- **本地起手式（§13 步 0–1）**：拍板 §14 → 改计划横幅与 §6 → 重锁；`cli/Cargo.toml` 加七行依赖（版本同 `scripts/tsprobe/Cargo.toml`）；`scan/lang.rs` LANGS 表六行 + Html 翻位 + `grammar()` 七臂 + `fingerprints()`；`scan/spec.rs` 的 `spec()` 分派到 `spec_c.rs` / `spec_lua.rs` / `spec_java.rs` / `spec_ruby.rs` / `spec_r.rs`（RM16 拆文件先例）；核 `namingShape` 与 `unresRow` 改读 `judgedMask`；`corelink.rs::PROTO` 与 `Version.hs` 7.2.0；golden 重生；子仓 `it/hs_grammar_pin.rs` 扩为七文法；落地后把 `scripts/tsprobe/snippets/**` 加进 `ce.toml` 的 `exclude`（README 已写）。
- **文件清单**：`docs/reference/language-expansion.md`（本册）；`scripts/tsprobe/{Cargo.toml, .gitignore, README.md, src/main.rs, snippets/×14}`。探针转录不入库：`cargo run --release -- <文法> snippets/<样本>` 十秒内重出。
