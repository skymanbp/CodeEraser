# 分析轨设计册：代码查询与架构规则 / 函数内死代码 / 克隆合并建议 / 架构分析（计划 v2.31，ADR-008 细则第七期）

> 本册是 v2.31 四个新判决家族的落码权威（计划书横幅 v2.31 条与 §6 T 轨 v2.31 行指向这里）。每个家族的规格在本册各占一章；落码时以本册为准，改设计先改本册再改计划书再动代码。语言扩展轨的设计册 [language-expansion.md](language-expansion.md) 是本册的体例先例。

## 0. 一句话定位

四个新功能全按既有分工落地：Rust 只把源码降成整数事实（§5.9.2：名字、路径、源文本永不过线），分析与判决全在 Haskell 核，每个功能三面同落（CLI / MCP / GUI）、各过自己的门、各一册方法学。它们是 2026-09-25 用户两裁的功能轨（语言条 Haskell ≥ 40 % / 1.8.0 之后开新功能轨，不改写现有代码），随 1.9.0 一版发出。

## 1. 范围与裁定

1. **四个家族**（落码顺序即计划书 T 轨顺序）：① `query/1` 代码查询与架构规则；② `flow/1` 函数内死代码；③ `merge/1` 克隆合并建议；④ `arch/1` 架构分析。每个家族一次 proto minor（7.3.0 → 7.6.0，§3），随各自的步落下，golden 六对起。
2. **三面同落**：每个家族 = 一个 CLI 子命令（`ce query` 与 `ce rules` 两个、`ce flow`、`ce merge`、`ce arch`）+ MCP 工具（`query` / `rules` / `flow` / `merge_suggestions` / `architecture`）+ GUI 屏（Query〔含规则〕/ Flow / Merge / Arch）+ 报告文档 `ce.<family>-report/0.1.0` + `main_lang.rs` 中文帮助 + parity 表一行。缺一面即该步未完成（三面等价常设指令）。
3. **判决还是顾问**：`query/1` 的 `assert` 是门（违规退 1，CI 狗粮加腿）；`flow/1` 是判决，按语言过精度门后才成门（§5.7）；`merge/1` 与 `arch/1` 是顾问（报告面自陈 advisory，永不产条件位，册 13 / 15 先例）。
4. **语言**：`query/1` 与 `arch/1` 读索引里的事实，语言无关；`flow/1` 只对有语句的文法（Python / TypeScript / TSX / Rust / Go / C / C++ / Java / Lua / R 十种；Haskell 无语句、Markdown / HTML / 纯文本无单元，按构造不入）；`merge/1` 对克隆族已判的对与组（`Lang::fingerprints()` 的十一种）。
5. **不改写现有代码**（用户裁）：四个家族只追加模块、只对既有 wire 做加性 minor；既有十二个家族的判决字节不动，落码每步以「旧二进制 / 新二进制在十语料十面对拍逐字节同」为门。
6. **不留已知限制**（用户令 2026-09-26）：凡本册想写成「边界」的，先问能不能按规范修对；修不对的在 §13 记立场与证据，不记「已知限制」。
7. **语言条**：GitHub 语言条读的是 `.gitattributes` 之后每种语言的字节数（`gh api repos/<owner>/<repo>/languages`）。立项日读数 Rust 2,144,472 / Haskell 592,241 B（Haskell 19.8 %）；目标 ≥ 40 %；每步收口时实测一次写进 CHANGELOG 该步的块，发版前的读数写进本册 §11 与发布说明。占比是副产品：不为占比搬迁代码、不为占比写死代码。

## 2. 不变量：一处权威、整数过线、判决在核、顾问永非判决

- **一处权威**：每个家族的请求形状只在核的 `CE.<Family>.Contract`（或 `CE.<Family>` 的解码段）与 Rust 的 `cli/src/<family>/wire.rs` 各写一次，两边由 golden 对拍钉住；上限与地板只在 `CE.<Family>.Cost` 一处，Rust 只镜像并在 `consume` 处校验回执。
- **整数过线**：程序文本、名字、路径、glob 都在 Rust 侧解析成整数（请求内稠密 id、fnv1a64 哈希、枚举码）；核的应答只有下标与整数；Rust 用自己的图例把下标翻回名字与行号。查询语言的记号流也是整数（§4.4），核只见记号 kind 与值。
- **判决在核**：Datalog 求值、分层、推导链；控制流图、可达性、活性；反统一、参数计数、可行性；分层、最小反馈弧集、聚类、影响面——全在核。Rust 侧凡「镜像」核规则的地方（如 `flow` 的语句类表）只作请求形状的自检，不作判决。
- **顾问永非判决**：`merge/1` 与 `arch/1` 的报告面第一行自陈 advisory；不进 `ce check`、不进守卫、不进基线。
- **既有家族不动**：`clone/1` 的树只加一列可选的叶子哈希（§6.2），`ted` 判决不读它；`scan/1` / `graph/1` / `structure/1` 的请求与应答字节不动，新家族各自装配自己的表。

## 3. wire 契约与版本

| 家族 | proto | 请求表 | 应答表 | 上限（`Cost`）与降级 |
|---|---|---|---|---|
| `query/1` | 7.3.0 | `program=[[kind,value]]` 记号流；`facts={"<code>":[[…]]}` 只含程序引用到的表（键 = 谓词码的十进制，`set` 表是其中一张），每表按元组严格升序；`prelude` 前奏子句数；`why` / `schema` 两个布尔 | `goals=[[goal,kind,sorts…]]`、`answers=[[goal,args…]]`、`proof=[[goal,answer,node,parent,rule,pred,args…]]`、`errors=[[token,code]]`、`counts={rules,queries,asserts,strata,facts,derived,answers,violations,proofNodes,proofTruncated}`、`schema=[[code,arity,sorts…]]`（只在请求要时） | 记号 65,536 / 事实行 4,194,304 / 派生元组 2,097,152 / 推导结点 16,384；`query_too_large` |
| `flow/1` | 7.4.0 | `units=[[u,lang,params]]`、`stmts=[[u,seq,parent,kind,flags,aux]]`、`vars=[[u,v,declSeq,flags]]`、`uses=[[u,seq,v,mode]]` | `findings=[[u,kind,seq,v,seqEnd]]`、`counts={units,stmts,vars,uses,findings}` | 行 524,288（四表合计）；`flow_too_large` |
| `merge/1` | 7.5.0 | `groups=[[g,family]]`、`members=[[g,m,unit,lines,fileIndeg]]`、`trees=[{lab,lld,leaf,slot}]`（按成员序） | `suggestions=[[g,params,kept,savings,feasible,reason]]`、`holes=[[g,hole,param,m,post]]`、`counts` | 组 4,096 / 结点 1,048,576；`merge_too_large` |
| `arch/1` | 7.6.0 | `files=[[F,D,lines]]`、`dirs=[[D,parent]]`、`edges=[[F,G,w]]`、`pkgEdges=[[F,D,w]]`、`focus=[F…]` | `layers=[[D,level]]`、`cuts=[[F,G,w,exact]]`、`clusters=[[F,cluster]]`、`misplaced=[[F,D]]`、`impact=[[F,depth]]`、`metrics=[[D,fanIn,fanOut,instability]]`、`counts` | 文件 131,072 / 边 524,288；`arch_too_large` |

- 四个 minor 各随其步，`contracts/VERSIONING.md` 各一条（最新在前），`Protocol.hs` 的 `families` 表各加一行（hello 能力表由它派生），`Version.hs` 与 `corelink.rs` 的 `PROTO` 同步；旧核缺能力时 Rust 经 `corelink/judged.rs::ask` 具名降级「core offers no `<family>/1` (pre-7.x.0)」。
- 拒绝（contract）按既有形：表形不合、非严格升序、下标越界、负值、成对表缺一——每条点名行号；程序错误（语法 / 未绑定 / 不可分层 / 类别不合 / 未知谓词 / 元数不一）**不是拒绝**，是正常应答里的 `errors` 表（§4.5）。
- golden：`contracts/fixtures/<family>/golden.ndjson` 六对起（请求 `"proto":"7.0.0"` 锚，回复按核逐字节），十四份既有 golden 只动 proto 字面与 hello 能力表；重生器改为测试子仓的 `--ignored` 腿（`CE_BLESS=1 … fixture_contract::regen`，与 daemon golden 同法），不再靠会话草稿目录里的脚本。

## 4. 功能 ①：代码查询与架构规则（`query/1`）

### 4.1 语言（CE Datalog v1）

```
program  := clause*
clause   := rule | query | assert
rule     := atom ':-' body '.'  |  atom '.'
query    := '?-' body '.'
assert   := 'assert' atom ':-' body '.'
body     := lit (',' lit)*
lit      := atom | 'not' atom | cond | bind
atom     := pred '(' term (',' term)* ')'
term     := Var | '_' | int | "string" | enum
cond     := expr ('=' | '!=' | '<' | '<=' | '>' | '>=') expr
bind     := Var '=' expr | Var '=' agg
expr     := term (('+' | '-' | '*' | '/' | '%') term)*
agg      := ('count' | 'min' | 'max' | 'sum') '(' Var (',' Var)* ':' body ')'
```

- 谓词名与枚举常量小写开头；变量大写开头；`_` 每次出现是新变量；`#` 到行尾是注释（`%` 是取余算子，不能兼作注释符——落码时改定）。
- 字符串常量的类别由所在实参的类别决定（§4.2）：路径类位置 = 路径或 glob（Rust 展开成文件集，线上只有集合号）；名字类位置 = 名字（线上只有 fnv1a64）；用户谓词位置 = 原子（同样只有哈希；回标时 Rust 用请求图例还原拼写）。类别不合 = 程序错误，按 `行:列` 点名。
- `assert` 的头部列出见证列；违规行 = 头部元组；断言通过 ⇔ 零元组。`?-` 的答案 = 体内变量的绑定，按首次出现序列出。
- 安全性：头部、否定、比较、算式里的每个变量必须先被同一体内的正原子（或 `=` 绑定）绑定；否则程序错误。
- 分层：谓词依赖图上否定与聚合的边是负边；负边落在强连通分量内 = 不可分层 → 程序错误，点名那条规则。递归只经正原子。
- 语义：集合语义、最小模型、逐层半朴素求值，每层建索引；无函数符号；算式只在体内、两侧已绑定，结果可绑定新变量（推导量受派生元组上限保护）；`Var = expr` 在 `Var` 已绑定时是比较。比较的两侧须同一类别——同类 id 也按编号比大小（`F < G` 打破对称对），算式只在整数上；除以零让那条文字不成立而不是报错。
- 聚合：`N = count(Y : body)` 对外层已绑定变量分组；`min` / `max` / `sum` 取第一个变量的值；聚合体属更低层。

### 4.2 事实库（EDB；Rust 装配，只送程序引用到的表）

结点宇宙 = `graph/1` 请求同一份结点集（文件、包、资产，同一稠密下标；`cli/src/graph/nodes.rs` 一处权威），目录树 = `structure/1` 同法在全部结点路径上建的稠密树（根 0）。

| 谓词 | 实参类别 | 来源 |
|---|---|---|
| `node(N, K)` `file(F)` `pkg(P)` `asset(A)` | node, enum kind | 图结点表；`file` / `pkg` / `asset` 是 kind 的三条视图 |
| `in_dir(N, D)` `dir(D)` `parent(D, P)` | node, dir | 路径的目录前缀；`parent(0, _)` 无行 |
| `lang(F, L)` | file, enum lang | `Lang` 码 |
| `role(N, R)` | node, enum role | 角色位（entry / test / declared / unit / asset / foreign …）每位一行 |
| `lines(F, N)` | file, int | 文件行数 |
| `ref(N, M, K, R)` | node, node, enum refkind, int | 解出的引用边（import / doc_link / doc_ref / asset / contain / refdef，rung 1–5）；External / Unresolved 无行，`unresolved(F, K, N)` 另表 |
| `unit(U, F)` `unit_kind(U, K)` `unit_lines(U, N)` `unit_at(U, L)` | unit, file, enum unitkind, int | 单元表（请求内稠密 id，序 = 文件、起行、nth） |
| `named(U, "name")` | unit, name | 声明名（简单名的哈希） |
| `exported(U)` | unit | 可见性位 0 |
| `coc(U, N)` `cyclo(U, N)` `nesting(U, N)` `params(U, N)` | unit, int | 度量（引用到才量） |
| `clone(U, V, K)` | unit, unit, enum clonekind | 已判克隆对（T1/T2 块的整单元孪生、T3 判 1 的对） |
| `docdup(F, G)` | file, file | 已判文档重复对 |
| `mention("name", F)` | name, file | 提及表 |
| `class(F, "name")` | file, class | `[[rules.class]]` 路径类 |
| `in(F, "glob")` | file, path-set | 语法糖：Rust 展开 glob（与 exclude 同一解析器）为 `set(S, F)`，`in` 改写成 `set(s, F)` |

类别（sort）在核里推导：变量类别取自首次出现的正原子；冲突 = 程序错误 `sort mismatch`。用户谓词的实参类别按各条规则合一；答案与见证列的类别随应答回给 Rust（回标用）。

### 4.3 内置前奏

随二进制的一份 `.rules` 文本（`include_str!`），与用户程序同一条词法路走上线，核不区分来源；前奏谓词名保留，用户程序重定义 = 程序错误。`ce query --prelude` 打印原文。

```
reach(N) :- role(N, entry).
reach(M) :- reach(N), ref(N, M, _, _).
dead(F) :- file(F), not reach(F).
depends(N, M) :- ref(N, M, _, _).
depends(N, M) :- depends(N, K), ref(K, M, _, _).
same_dir(F, G) :- in_dir(F, D), in_dir(G, D), F != G.
dir_ref(D, E) :- ref(F, G, _, _), in_dir(F, D), in_dir(G, E), D != E.
```

### 4.4 记号流编码（Rust 词法，核解析）

`program=[[kind, value]]`：0 谓词码（EDB 固定码，IDB 按首次出现 ≥ 1000）/ 1 变量（每条子句内编号，`_` 每次新号）/ 2 整数 / 3 集合号 / 4 名字哈希（枚举常量也走这一路——`entry` 与 `"entry"` 同一个 fnv1a64，事实表的枚举列送的就是名字哈希；5 空着不用）/ 6 匿名 / 10 以上标点与关键字（`:-` `,` `.` `(` `)` `not` `?-` `assert` `=` `!=` `<` `<=` `>` `>=` `+` `-` `*` `/` `%` `count` `min` `max` `sum` `:`）。Rust 记每个记号的 `行:列`；核的解析器手写（递归下降，无解析库——核依赖只有 base / aeson / array / bytestring / containers）。这样分工：词法与解析错误的位置回标是文本活（Rust），文法、类别、安全性、分层是判决活（核）。

### 4.5 应答

- `answers` 按 goal 序再按元组升序；`goals` 给每个目标的种类（0 查询 / 1 断言）与列类别。
- `proof`：每个答案元组的第一条推导——行 `[goal, answer, node, parent, rule, pred, args…]`，结点按前序编号、根的 parent = −1，`rule` = 子句下标（发送的事实行 −1），`pred` = 结点的谓词码（查询的根 −1）；只在 `?-` 带 `--why` 或断言违规时展开；一棵树装不下预算 `proofCap` 就整棵不出并记 `counts.proofTruncated`，答案不截。
- `errors`：程序错误按记号下标 + 码（1 语法 / 2 未知谓词 / 3 元数不一 / 4 类别不合 / 5 未绑定 / 6 不可分层 / 7 前奏重定义 / 8 聚合形不合 / 9 头部匿名）；检查按这个顺序分阶段，第一个出错的阶段报出它的全部错误；有错误即不求值、`answers` 空。
- 超派生上限 = 完整降级应答 `query_too_large`（`counts.derived` 给到达上限时的值）。

### 4.6 面与配置

- `ce query '<body>' [--why] [--file <path>] [--prelude] [--format json]`：即席查询（体 = 一条 `?-`），叠加规则文件里的规则。
- `ce rules [--file <path>] [--why]`：求值文件里全部 `assert`；每条违规印见证与推导链；有违规退 1，程序错误退 2。默认文件 = 仓根 `ce.rules`（无文件 = 零断言、退 0 并说明）；`[rules] file` 改路径（与 `[[rules.class]]` 同节；`docs_gate.rs` 的键表加行、`ce-toml.md` 再生）。
- MCP `query` / `rules`；GUI Query 屏（输入框 + 答案表 + 断言状态 + 推导链折叠）；报告 `ce.query-report/0.1.0` / `ce.rules-report/0.1.0`。
- 自食：仓根 `ce.rules` 写本仓的架构断言（例：`core/app` 不引用仓内 Haskell 之外的文件、`cli/src/scan` 不依赖 `cli/src/graph`、`site/` 无孤页、无跨目录环——每条按当日事实写、先跑绿再入库），CI 两根狗粮各加 `ce rules` 腿，`count:gates` 六 → 七。

### 4.7 门

- 核：朴素不动点参考求值器与半朴素 + 索引求值器在随机程序 / 随机事实上逐元组等价；分层 / 安全性 / 类别每种程序错误按名；推导链每结点重放规则体得回该元组；聚合与参考实现等价；派生上限降级；golden 六对（前奏 + 一条断言 + 一条带 `--why` 的查询 + 三种程序错误）。
- Rust：词法与错误位置回标；glob 展开与 exclude 同一解析器（负向探针）；事实装配与索引逐表对拍（`ref` 表 = `graph/1` 请求的边表逐行同）；三面字节同（parity）。
- 精度：无统计门（语义精确）；自仓 `ce.rules` CI 常绿即验收。

## 5. 功能 ②：函数内死代码（`flow/1`）

### 5.1 降表（Rust，每语言一张 `FlowSpec` 表，与 `LangSpec` 同目录）

- `stmts=[[u, seq, parent, kind, flags, aux]]`：单元内语句按前序编号；`kind`：0 block / 1 stmt / 2 if / 3 loop / 4 switch / 5 case / 6 try / 7 catch / 8 finally / 9 return / 10 throw / 11 break / 12 continue / 13 goto / 14 label / 15 noreturn-call；`flags` 位：0 has_else / 1 infinite（常真循环头：Rust `loop`、Go `for {}`、Python `while True`、C `for(;;)`）/ 2 fallthrough / 3 dynamic（单元含 `eval` / `exec` / `load` 一类名的调用，按语言表）/ 4 empty；`aux` = break / continue / goto 的目标 seq（无标签 = 最近的循环或 switch）。
- `vars=[[u, v, declSeq, flags]]`：局部量与形参；`flags`：0 param / 1 captured（被嵌套单元或闭包引用；嵌套单元是独立单元，捕获在宿主侧记位）/ 2 ignored（以 `_` 开头、或语言的丢弃名）/ 3 address_taken（C 族 `&x`、Rust `&mut x` 传出）。
- `uses=[[u, seq, v, mode]]`：按求值序，`mode`：0 read / 1 write / 2 readwrite（复合赋值、`x++`）。
- 不入表的：表达式内部结构、名字、字面量；只有 `kind` / `flags` / 序号。

### 5.2 判决（核 `CE.Flow.*`）

- 建控制流图：结构化控制流按 `kind` 展开；`try` → `catch` 保守（`try` 体每条语句都可能跳到 `catch`）；`finally` 在每条离开边上；`goto` / `label` 按 `aux`；`noreturn-call` 与 `throw` / `return` 是出口。
- 可达性 → `unreachable`（kind 0）：从入口不可达的语句，一条极大连续段报一条（`seq`…`seqEnd`）。
- 反向活性 → `dead_store`（kind 1）：写入后在所有路径上被覆盖或到出口前从未读（`address_taken` / `captured` 的变量不判）。
- `unused_local`（kind 2）：声明后无任何读（`ignored` 不判）；`unused_param`（kind 3，顾问）：形参无读（接口 / 重载 / 覆写的形参本就可能不用——只报不门）。
- `dynamic` 位为 1 的单元整体不判（记 `counts.dynamicUnits`）。

### 5.3 语言对照（每语言先按 tree-sitter 实探建表，与 v2.30 §4 同法）

| 语言 | 语句容器 | 终结语句 | 局部声明 | 备注 |
|---|---|---|---|---|
| Python | `block` | return / raise / break / continue（`pass` 是 stmt） | 赋值目标首现 | `global` / `nonlocal` 名不作局部；推导式与 lambda 是嵌套作用域 |
| TypeScript / TSX | `statement_block` | return / throw / break / continue | `lexical_declaration` / `variable_declaration` | 箭头函数是独立单元；`var` 提升按声明所在单元 |
| Rust | `block` | return / break / continue 表达式；`panic!` / `unreachable!` / `todo!` 宏按名表入 noreturn | `let_declaration` | 表达式尾值 = 读；`loop` 常真 |
| Go | `block` → `statement_list` | return / break / continue / goto / fallthrough / `panic` / `os.Exit` 按名表 | `short_var_declaration` / `var_declaration` | 未用局部量 Go 编译器已拒，故 kind 2 在 Go 只可能是被 `_ =` 抑制的写——按 ignored 不判；dead_store 与 unreachable 照判 |
| C / C++ | `compound_statement` | return / throw / break / continue / goto / `exit` / `abort` / `noreturn` 属性函数按名表 | `declaration` 每个 declarator | `#if` 条件区按 `opaque_fields` 不透明整段不判 |
| Java | `block` | return / throw / yield / break / continue | `local_variable_declaration` | 编译器已拒不可达语句，故 kind 0 在 Java 只可能出现在编译器不查的形（`while(true)` 后）——照判，量得到即报 |
| Lua | `block` | return / break / goto | `local` 声明 | `if` 分支体可为空（无子结点）= empty 位 |
| R | `braced_expression` | `break` / `next`；`return(x)` / `stop()` 是 `call`，按被调名表入终结 | 赋值首现（`<-` / `=` / `->`） | 全局赋值 `<<-` 不作局部 |

### 5.4 面与档位

- `ce flow [--check] [--kind …] [--format json]`：报告 `ce.flow-report/0.1.0`（每条 `{path, unit, kind, line, lineEnd, var?}`，名字与行号由 Rust 图例回标）；`--check` 在 `[flow] tier` 为 `deny` 且有非顾问发现时退 1。
- 守卫腿：PreToolUse 对本次写入的单元跑一次（复用 guard 的探针路，daemon `2.1.0` → `2.2.0` 加性请求）；档位 `[flow] tier = observe | warn | ask | deny`（出厂 observe；晋级按 §4.2 铁律：有台账才可 deny）；Stop 审计行加性对象 `flow{findings, kinds}`，feed `ce.observe` minor 加性。
- MCP `flow`；GUI Flow 屏（按单元列发现、按 kind 筛）。

### 5.5 门

- 核：CFG 与参考实现（朴素传递闭包）在随机结构化程序上逐语句等价；活性与参考（逐路径枚举，小程序）等价；十六种拒绝按名；golden 六对（四种发现各一 + try/finally 一 + 降级一）。
- 精度考题（每语言）：语料 = v2.30 考题登记表的第一个语料 + 本仓（Rust / TypeScript / Python 用既有对拍语料）；抽样冻结（每语言每种发现 ≥ 15 道，池不满整类取尽）→ 没看过判决的独立代理盲判 → 精度册 `contracts/eval/flow-precision-<lang>-v1.json`；顺序由提交先后证明（v2.30 §14 第 14 条同法）。**门 = 每语言每种非顾问发现的精度 ≥ 99 %（误报 ≤ 1 %）**，达门的语言进 `flow` 的 `judged` 掩码，未达门的语言只 observe（逐语言发布门，v2.30 §14 第 9 条先例）。
- 回放台账（每语言）：常设台账的 `replay` 加 flow 腿——窗口内每个提交对改动单元判决，一条发现在后来的提交里随该单元被改而消失 = 真阳，随该单元被改却留下 = 误拦（严格口径），窗口末仍在且单元未再被改 = 未定（不计）；读数写进 `FPR-REPLAY.md` 新节，两口径并记，门读精度考题。

## 6. 功能 ③：克隆合并建议（`merge/1`）

### 6.1 组

- T1/T2：`ce dedup` 的组（端点相同的块连通分量）里覆盖整单元的成员——每成员一棵树；只覆盖片段的块按整块（片段树）成组，建议面标 `fragment`。
- T3：`t3ted` 判 1 的对，每对一组（两成员）。
- 组内成员同语言（克隆族按构造保证）。

### 6.2 树（与 `clone/1` 同一后序编码加两列）

`trees=[{lab, lld, leaf, slot}]`：`lab` / `lld` 与 `clone/1` 同（命名结点后序、kind 哈希、最左叶）；`leaf` = 叶结点原文的 fnv1a64（标识符与字面量各自的文本；内部结点 0）；`slot` = 结点位置类：0 语句位 / 1 表达式位 / 2 类型位 / 3 名字位（声明名）/ 4 其他。`clone/1` 的请求同批接受可选的 `leaf` 列（长度须等于结点数，`ted` 不读它——判决字节不变），`merge/1` 要求两列都在。位置类由每语言一张表给（`FlowSpec` 同目录），是语法事实。

### 6.3 判决（核 `CE.Merge.*`）

- T1/T2 组：成员树同构（记号流相等）；逐结点对齐，`leaf` 不全同的位置 = 洞。
- T3 对：Zhang–Shasha 的最优映射（`CE.Clone.Ted` 扩为回映射，距离值不变）；映射内的叶差 = 叶洞，映射外的子树 = 子树洞。
- 洞按「各成员取值向量」合并：向量相同的洞是同一个参数；参数数 = 互异向量数。
- 可行：每个洞在表达式位或名字位（reason 0）；否则按首个不可行洞记 reason（1 语句位 / 2 类型位 / 3 子树洞跨语句 / 4 参数 > 6）。
- 收益 = Σ 成员行数 − （骨架行数 + 成员数 × 1）；骨架行数 = 任一成员行数（同构）或映射公共部分所占行数（T3）。
- 保留者 = 文件入度最大的成员，并列取 `m` 最小（Rust 送 `fileIndeg`，图侧事实）。

### 6.4 面

`ce merge [--group <n>] [--format json]`：报告 `ce.merge-report/0.1.0`（每组 `{family, members[{path, unit, lines}], params, kept, savings, feasible, reason, holes[{param, values[{member, text}]}]}`——`text` 由 Rust 用成员源文件按 `post` 回标）；MCP `merge_suggestions`；GUI Merge 屏（组列表 + 参数对照表）。顾问：不进门、不进基线。

### 6.5 门

- 核：反统一定律——实例化（骨架 + 每成员的参数向量）逐树复原每个成员；极小性（少一个参数即有成员复原不出）；映射版 TED 距离与原 TED 逐对相等（随机树电池）；golden 六对。
- 冻结建议集：本仓 + 四语料的建议集冻结（`contracts/eval/merge-suggestions-v1.json`），改动即漂移门；盲判精度（可行 / 参数数由独立代理按源码判），读数入册。

## 7. 功能 ④：架构分析（`arch/1`）

### 7.1 表

`files=[[F, D, lines]]`（measured 文件结点）、`dirs=[[D, parent]]`（稠密树，根 0）、`edges=[[F, G, w]]`（文件到文件，w = 该对之间的引用边数，任何 kind 与 rung）、`pkgEdges=[[F, D, w]]`（包粒度的引用——Go / R / Java 的包导入与 Markdown 的目录链接，`structure/1` 的 `dirEdges` 至今丢掉的那一类——核折成 dir(F) → D）、`focus=[F…]`（`--impact` 点名的文件）。

### 7.2 判决（核 `CE.Arch.*`）

- 目录图：文件边按目录聚合（同目录内不计），加 `pkgEdges`。
- 分层：去掉最小反馈弧集后的拓扑层级（`layers`）；反馈弧集 = `cuts`（拆环最省改法）——强连通分量 ≤ 12 条边穷举精确（`exact = 1`），更大用 Eades–Lin–Smyth 贪心（`exact = 0`）；权 = 边的引用数。
- 聚类：文件图上的确定性 Louvain（结点按 id 序、并列取小 id，两遍即停）→ `clusters`；文件所在目录与其簇的多数目录不同 = `misplaced`。
- 影响面：从 `focus` 反向 BFS，`impact=[[F, depth]]`。
- 度量：每目录 fanIn / fanOut / 不稳定度 I = out / (in + out)（千分整数）。

### 7.3 面

`ce arch [--impact <path>…] [--format json]`：报告 `ce.arch-report/0.1.0`（分层表、拆环建议〔每条边带两端路径与引用数〕、簇、错位文件、影响面、目录度量）；MCP `architecture`；GUI Arch 屏（分层图 + 拆环表 + 影响面输入）。顾问。

### 7.4 门

- 核：分层与「去掉 cuts 后无环」互证；精确 FAS 与穷举参考在随机小图上逐权等价；贪心 FAS 结果无环且 ≤ 穷举的 2 倍（记录界，随机电池）；Louvain 确定性（同图两跑逐字节同）；影响面与朴素闭包等价；golden 六对。
- 冻结自仓读数：本仓的分层 / cuts / 簇冻结（`contracts/eval/arch-self-v1.json`），改动即漂移门（自仓是活语料，漂移随代码变，按 EVAL-SET 复活协议改签）。

## 8. 核的模块布局与尺寸

每个家族 = `CE/<Family>.hs`（respond、解码、应答）+ `CE/<Family>/Cost.hs`（上限、地板、码域）+ 判决模块若干（`Query`: `Contract`（请求形与拒绝）、`Syntax` / `Parse`（记号流 → 子句）、`Check`（安全性 / 类别 / 分层；`Check/Sorts`、`Check/Safety`）、`Eval`（半朴素 + 索引；`Eval/Index`、`Eval/Join`）、`Proof`、`Schema`；`Flow`: `Cfg`、`Reach`、`Live`；`Merge`: `Align`、`Holes`、`Ted` 的映射扩展；`Arch`: `Layers`、`Fas`、`Louvain`、`Impact`），每文件 ≤ 290 行（`core_size_gate` 的真实上限）。电池 `core/test/<Family>Props.hs` 各一（`WireHarness.runLegs` 两条平行列表形，避开查重门的表同韵）。`core/test/Spec.hs` 现 289 行，每加一家族 +3 行——步 1 先把电池清单拆到 `core/test/Batteries.hs`（`fixture_contract.rs` 读的那三行形不变、只换文件）。

## 9. 验收与门（每步共用）

- 两仓 `cargo test / clippy --all-targets -- -D warnings / fmt --check`、`cabal test`、六条产品腿（主根与 `cli/tests`）、golden 重生（`CE_BLESS=1`）、`fixture_contract` 的 golden 清单、`core_wire` 往返、`face_parity`、`facts_*`、`docs_*`、`site_*`；两仓 ADR-006 具名重立；dedup 预算只降不升（新块先消后入账）。
- 既有判决不动：每步旧二进制 / 新二进制在十语料十面 + 自仓干净树十面对拍逐字节同（步 7b 的形）。
- 全量 it 在每步提交前跑一次（release 或 debug 按步的重活定），红先单跑读 panic 再定抖动 / 真红。

## 10. 文档与事实面

- 计数事实：`count:families` 十二 → 十六、`count:screens` 十一 → 十五、`count:mcp_tools` 十六 → 二十一（`it/facts/form.rs` 的数词表到 twenty，先扩到 thirty）、`count:booklets` 十五 → 十九、`count:gates` 六 → 七、`count:golden_requests` 152 → ≥ 176；`LITERALS` 里手改的字面（stack.svg 两对的家族行、how 页标题、六处 alt 文字「N families」）逐处改。
- 官网 how 页现 732 行、硬线 750：四张家族卡放新页对 `site/how/analysis/`（en / zh 各一），how 页目录面板与两首页各加一行入口；`site_contents.rs` 假定 f15 是末卡，改读 f16–f19 在新页；`verify_site.js` 与部署清单加两页。
- 方法学册 16–19（每家族一册：判决定义、表、门、读数）+ `methodology.md` 索引四行 + 前后导航；架构图 IR（家族数 / 屏数 / 工具数子标签）与判决图（tag 行加四个家族）重渲；README 双语命令表加五行、parity 块随 bless、「三面一体」句、How it works 四行、技术栈条目；plugin/README 工具句；`VERSIONING.md` 四条；`ce-toml.md` 与 `cli.md` 再生；`DAEMON.md` 一行（flow 腿）；CHANGELOG 每步一块；计划书横幅与 T 轨行随步。

## 11. 语言条：量法与记账

读数 = `gh api repos/skymanbp/CodeEraser/languages` 的字节表（`.gitattributes`：`cli/tests/**` 与 `contracts/**` vendored、`site/**` documentation、`core/test` 计入）。立项日 2026-09-29：Rust 2,144,472 / Haskell 592,241 / 其余约 252,000 B，Haskell 19.8 %；40 % 需要 Haskell 约 +1.0–1.2 MB（核与电池各按 ~44 B/行计 ≈ 两万行加八千行电池），Rust 的新增同时抬分母。每步收口写一行读数进 CHANGELOG 该步块；步 10 的读数是发版声明的读数——若低于 40 %，差距与可选路线（都是新功能、不改写）以 AskUserQuestion 上呈，不自行拉伸。

## 12. 分步（每步自带门，落码顺序）

| 步 | 内容 | 门 |
|---|---|---|
| 0 | 细则与立项（2026-09-29）：本册 + 计划书 v2.31（横幅细则句、ADR-008 细则第七期、§6 T 轨十二步）+ CHANGELOG `[Unreleased]` 块 + cc-memory 十二步锁定 | docs 门全绿、基线具名重立、CI 绿 |
| 1 | 查询 A（2026-09-29 已交付）：核 `CE.Query.*`（Contract / Syntax / Parse / Check〔Sorts · Safety〕/ Eval〔Index · Join〕/ Proof / Schema / Cost，十三模块 1,501 行）+ `QueryProps` 十二腿 + `ReferenceQuery` 朴素参考 200 例 + golden 六对 + proto 7.3.0（`Protocol.hs` 一行、`Version.hs`、`corelink.rs`、VERSIONING 一条）+ `Spec.hs` 拆 `SpecProbes.hs`（四条探针腿；设计名 `Batteries.hs` 按搬出的内容改，电池表留在 `Spec.hs`）+ 子仓 `fixture_contract::regen` 腿（`LineSession` 三件套同供 MCP 会话与 golden 往返） | `cabal test` 439 ok、参考求值器等价 200/200、既有 golden 只动 proto |
| 2 | 查询 B：Rust `cli/src/query/`（词法、图例、glob 展开、事实装配、wire、回标）+ `ce query` / `ce rules` + `[rules] file` + 前奏 + MCP 两工具 + GUI Query 屏 + 自仓 `ce.rules` + CI 狗粮腿 + 册 16 + 事实与 parity | 三面字节同、`ce rules` 自仓绿、十语料十面对拍同 |
| 3 | 死代码 A：核 `CE.Flow.*`（Cfg / Reach / Live / Cost）+ `FlowProps` + golden + 7.4.0 | 参考实现等价、十六种拒绝按名 |
| 4 | 死代码 B：Rust `FlowSpec` 十语言表（实探建表）+ 降表 + wire + 考题冻结（每语言）→ 盲判 → 精度册 + 回放台账 + `judged` 掩码 | 逐语言精度 ≥ 99 %、顺序门 |
| 5 | 死代码 C：`ce flow` + 守卫腿（daemon 2.2.0）+ Stop 行 + `[flow] tier` + MCP + GUI + 册 17 | 三面字节同、FPR 台账 |
| 6 | 合并 A：核 `CE.Merge.*`（Align / Holes / Ted 映射）+ `MergeProps` + golden + 7.5.0（`clone/1` 可选 `leaf` 列） | 定律电池、TED 距离不变 |
| 7 | 合并 B：Rust 叶子哈希与位置类列、组装配、wire、`ce merge` + MCP + GUI + 冻结建议集 + 盲判精度 + 册 18 | 三面字节同、冻结集门 |
| 8 | 架构 A：核 `CE.Arch.*`（Layers / Fas / Louvain / Impact）+ `ArchProps` + golden + 7.6.0 | 穷举参考等价、确定性 |
| 9 | 架构 B：Rust 表装配（含 `pkgEdges`）、wire、`ce arch` + MCP + GUI + 冻结自仓读数 + 册 19 | 三面字节同、冻结读数门 |
| 10 | 全量文档与事实（§10）+ 语言条实测记账（§11）+ 官网新页对部署 | docs / site / facts 门全绿、引文重签 |
| 11 | 发版 1.9.0：分数可比性声明（判决轴不动即与 1.8.0 可比；`flow` 若进 `ce check` 另声明）、基线具名重立、bench 入列 | RELEASE.md 链 |

依赖：0 先于一切；1 → 2、3 → 4 → 5、6 → 7、8 → 9 各成链，四条链按序落（一条链收口再开下一条，每链收口时既有家族对拍同）；10 / 11 最后。

## 13. 拍板记录（2026-09-29）

用户两裁在先（2026-09-25：语言条 ≥ 40 % / 1.8.0 之后开四件新功能轨，不改写现有代码；2026-09-29：「推 1.9.0」）。以下各条按既定原则（最完整最彻底、一处权威、整数过线、有先例照先例、顾问永非判决、不留已知限制）自答，只剩一个完整选项的直接定；括号里是被排除的备选。

1. 查询语言的解析放哪——**词法在 Rust、文法在核**：记号流是整数（§4.4），文法 / 类别 / 安全性 / 分层是判决活，核手写递归下降（备选「Rust 解析成表送核」把文法错误的判定留在测量侧）。
2. 每家族一次 minor——**7.3.0 → 7.6.0 各随其步**：每步都是可发的状态，`judged::ask` 的 since 串、VERSIONING 条、how 页芯片三处同一版本（备选「四家族共用 7.3.0」要四步都落完才能写 since 串）。
3. `query/1` 的事实宇宙——**与 `graph/1` 同一份结点集与下标**：`ref` 表逐行等于图请求的边表，事实装配可对拍（备选「只送文件」丢掉包与资产结点，`dead` 前奏就与 `ce deadcode` 不一致）。
4. 程序错误的形——**正常应答里的 `errors` 表**，拒绝只留给表形（备选「按拒绝答」让一条拼错的规则读成「核坏了」）。
5. 前奏谓词名保留——**保留**：一处权威（备选「允许覆盖」让同名谓词两处定义）。
6. `flow/1` 的门——**精度考题 ≥ 99 %，回放台账并记**：静态发现的误报直接由盲判量；回放的「随单元被改却留下」是第二读数（备选「只读回放」把「人没修」读成误报）。
7. `flow/1` 的语言集——**十种有语句的文法**；Haskell 无语句按构造不入，不记为限制（备选「Haskell 按 do 块判」是另一种判决，另立）。
8. `unused_param` 顾问——**顾问不门**：接口 / 覆写 / 回调的形参本就可能不用，误报无界（备选「入门」）。
9. `merge/1` 的树——**`clone/1` 同编码加 `leaf` 与 `slot` 两列**：计划书写的是叶子哈希一列，位置类是可行性判决必需的第二个语法事实，同批加（备选「核从 kind 哈希猜位置类」——核不知道文法）。
10. T3 对的反统一——**扩 `CE.Clone.Ted` 回映射，距离值不变**（备选「只做 T1/T2」丢掉近似克隆这一半）。
11. `arch/1` 的包粒度边——**`pkgEdges` 单独一表折进目录图**：`structure/1` 至今丢掉 Go / R / Java 的包导入与 Markdown 目录链接（盘点所见），新家族不继承这个洞（备选「沿用 `dirEdges`」）。
12. 反馈弧集——**≤ 12 条边穷举精确、更大贪心并标 `exact`**：报告面说清哪条是最优哪条是近似（备选「全贪心」）。
13. 官网卡片放哪——**新页对 `site/how/analysis/`**：how 页 732 行贴着 750 硬线，四张卡放不下；页面也不该为了塞卡片而删既有内容（备选「压缩既有卡片」）。
14. golden 重生器——**测试子仓的 `--ignored` 腿**：会话草稿目录里的脚本随会话消失（盘点所见），daemon golden 已有同形先例（备选「入库 python 脚本」引入第三种工具语言）。
15. 语言条低于 40 % 时——**上呈用户，不自行拉伸**：占比是副产品、不为占比写代码（§1 第 7 条）。
