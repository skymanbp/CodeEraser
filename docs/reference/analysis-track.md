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
| `flow/1` | 7.4.0 | `units=[[u,lang,params]]`、`stmts=[[u,seq,parent,kind,flags,aux]]`、`vars=[[u,v,declSeq,flags]]`、`uses=[[u,seq,v,mode]]` | `findings=[[u,kind,seq,v,seqEnd]]`、`counts={units,stmts,vars,uses,findings,dynamicUnits}` | 行 524,288（四表合计）；`flow_too_large` |
| `merge/1` | 7.5.0 | `groups=[[g,family,helper]]`、`members=[[g,m,unit,lines,fileIndeg]]`、`trees=[{lab,lld,leaf,slot,own,text}]`（按成员序） | `suggestions=[[g,params,kept,savings,feasible,reason]]`、`holes=[[g,hole,param,m,post,postEnd]]`、`counts` | 组 4,096 / 结点 131,072（两道上限下最宽的请求在协议的 32 MiB 行内）；`merge_too_large` |
| `arch/1` | 7.6.0 | `files=[[F,D,lines]]`、`dirs=[[D,parent]]`、`edges=[[F,G,w]]`、`pkgEdges=[[F,D,w]]`、`focus=[F…]` | `layers=[[D,level]]`、`cuts=[[D,E,w,exact]]`、`clusters=[[F,cluster]]`、`misplaced=[[F,M]]`、`impact=[[F,depth]]`、`metrics=[[D,fanIn,fanOut,instability]]`、`counts` | 文件 131,072（`fileCap`）/ 引用 524,288（`edges` + `pkgEdges` 合计，`refCap`）；`arch_too_large` |

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

结点宇宙 = `graph/1` 请求同一份结点集（文件、包、节、走查到的资产，同一稠密下标；`cli/src/graph/nodes.rs` 一处权威）后接 docdup 族读的纯散文文件；目录树 = `structure/1` 同法在全部结点路径上建的稠密树（根 0 读 `.`，包坐在自己的目录）。落码 = `cli/src/query/legend.rs::SCHEMA`（27 个谓词，码 = 行位）与核 `CE.Query.Schema` 逐行镜像，`schema` 回显钉住。

| 谓词 | 实参类别 | 来源 |
|---|---|---|
| `node(N, K)` `file(F)` | node, sym | 图结点表 + 散文文件；K = `file` / `pkg` / `section` / `asset` / `prose`，`file` 是 kind `file` 的视图 |
| `in_dir(N, D)` `dir(D)` `parent(D, P)` `dir_name(D, "name")` | node, dir, sym | 目录树；`parent(0, _)` 无行 |
| `lang(F, L)` | node, sym | 文件形结点的 `Lang::name`，无语言认领 = `unknown` |
| `role(N, R)` | node, sym | 角色位每位一行：`entry_named` / `entry_dir` / `test` / `glob` / `doc` / `allow` / `declared` / `foreign` / `unit` / `asset` |
| `lines(F, N)` | node, int | 文件行数（引用到才读） |
| `ref(N, M, K, R)` `unresolved(F, N)` | node, node, sym, int | 解出的引用边（import / doc_link / doc_ref / asset / contain / refdef，rung）；External / Unresolved 无边，`unresolved` 按文件计未解析站点数 |
| `unit(U, F)` `unit_kind(U, K)` `unit_lines(U, N)` `unit_at(U, L)` | unit, node, sym, int | 单元表（请求内稠密 id，序 = 文件、起行、末行、键、nth）；kind = `fn` / `named` / `impl` / `section` |
| `named(U, "name")` `exported(U)` `params(U, N)` | unit, sym, int | 提及规约会拼的声明名、可见性位、键上的元数 |
| `coc(U, N)` `cyclo(U, N)` `nesting(U, N)` | unit, int | 核为 `ce scan` 推导的三数，按文件与跨度落座（引用到才量） |
| `clone(U, V, K)` | unit, unit, sym | 已判克隆对：`t1t2` = T1/T2 块两侧整单元按序配对、`t3` = 核判为克隆的对 |
| `dup(F, G, N)` `docdup(F, G)` | node, node, int | T1/T2 块的文件对（记号数）；核判的文档重复段背后的文件对 |
| `mention("name", F)` | sym, node | 提及表（同一 fnv1a64） |
| `class(F, "name")` | node, sym | `[[rules.class]]` 路径类 |
| `set(S, F)` ← `in(F, "glob")` | set, node | 语法糖：Rust 把 glob（与 exclude 同一解析器）展开成 `set(S, F)` |

类别（sort）在核里推导（并查集：子句变量 / 程序谓词位置 / 常量类别）：整数字面量 int、字符串 sym（集合位置 set）、比较两侧同类、算子操作数与聚合结果 int；冲突 = 程序错误 `sort mismatch` 按记号点名。用户谓词的实参类别按各条规则合一，随应答的 `preds` 回给 Rust（回标推导链结点用）；答案与见证列的类别随 `goals` 回。

### 4.3 内置前奏

随二进制的一份 `.rules` 文本（`include_str!`），与用户程序同一条词法路走上线，核不区分来源；前奏谓词名保留，用户程序重定义 = 程序错误。`ce query --prelude` 打印原文。

```
entry(N) :- role(N, _).
reach(N) :- entry(N).
reach(M) :- reach(N), ref(N, M, K, _), K != asset, K != refdef.
dead(F) :- file(F), not reach(F).
depends(N, M) :- ref(N, M, _, _).
depends(N, M) :- depends(N, K), ref(K, M, _, _).
same_dir(F, G) :- in_dir(F, D), in_dir(G, D), F != G.
dir_ref(D, E) :- ref(F, G, _, _), in_dir(F, D), in_dir(G, E), D != E.
```

落码 `cli/src/query/prelude.rules`（八条）：`dead` 逐字镜像 `ce deadcode` 的文件级判决——入口 = 带任一角色位的结点（每个角色都落在核的入口掩码里），任一 rung 的引用都沿、只跳 asset / refdef 两种对存活性无效的边；前奏谓词码 1000–1005，用户谓词自 1006 起。

### 4.4 记号流编码（Rust 词法，核解析）

`program=[[kind, value]]`：0 谓词码（EDB 固定码，IDB 按首次出现 ≥ 1000）/ 1 变量（每条子句内编号，`_` 每次新号）/ 2 整数 / 3 集合号 / 4 名字哈希（枚举常量也走这一路——`entry` 与 `"entry"` 同一个 fnv1a64，事实表的枚举列送的就是名字哈希；5 空着不用）/ 6 匿名 / 10 以上标点与关键字（`:-` `,` `.` `(` `)` `not` `?-` `assert` `=` `!=` `<` `<=` `>` `>=` `+` `-` `*` `/` `%` `count` `min` `max` `sum` `:`）。Rust 记每个记号的 `行:列`；核的解析器手写（递归下降，无解析库——核依赖只有 base / aeson / array / bytestring / containers）。这样分工：词法与解析错误的位置回标是文本活（Rust），文法、类别、安全性、分层是判决活（核）。

### 4.5 应答

- `answers` 按 goal 序再按元组升序；`goals` 给每个目标的种类（0 查询 / 1 断言）与列类别。
- `preds` 按码升序给每个程序谓词各位置推导出的类别（无约束位 −1）；有错误或空程序时为空——步 2 加进 7.3.0 未发布的应答形，回标推导链结点的实参用。
- `proof`：每个答案元组的第一条推导——行 `[goal, answer, node, parent, rule, pred, args…]`，结点按前序编号、根的 parent = −1，`rule` = 子句下标（发送的事实行 −1），`pred` = 结点的谓词码（查询的根 −1）；只在 `?-` 带 `--why` 或断言违规时展开；一棵树装不下预算 `proofCap` 就整棵不出并记 `counts.proofTruncated`，答案不截。
- `errors`：程序错误按记号下标 + 码（1 语法 / 2 未知谓词 / 3 元数不一 / 4 类别不合 / 5 未绑定 / 6 不可分层 / 7 前奏重定义 / 8 聚合形不合 / 9 头部匿名）；检查按这个顺序分阶段，第一个出错的阶段报出它的全部错误；有错误即不求值、`answers` 空。
- 超派生上限 = 完整降级应答 `query_too_large`（`counts.derived` 给到达上限时的值）。

### 4.6 面与配置

- `ce query '<body>' [--why] [--file <path>] [--prelude] [--format json]`：即席查询（体 = 一条 `?-`，`?-` 与结尾 `.` 可省），叠加规则文件里的规则；判出退 0，程序错误或核未判退 2（只报告：所叠加文件里断言的违规不动它的退出码）。
- `ce rules [--file <path>] [--why]`：求值文件里全部 `assert`；每条违规印见证与推导链；有违规退 1，程序错误或核未判退 2。文件 = 命令行 / MCP 点名的（相对根、须存在）> `[rules] file`（须存在）> 仓根存在的 `ce.rules` > 无（零断言、退 0 并说明）；`[rules] file` 是路径不是旋钮，指纹丢弃（canonical 规则 7）。
- MCP `query`（`body` / `why` / `file`）/ `rules`（`file` / `why`）；GUI Query 屏（第十二屏：输入框 + why 开关 + 每目标一张表 + 答案下的推导行 + Rules 按钮）；报告 `ce.query-report/0.1.0` / `ce.rules-report/0.1.0`，三面经 `faces::query` / `faces::rules` 同一份文档。
- 自食（已落）：仓根 `ce.rules` 九条断言（ADR-008 分界三条、scan 不依赖 graph、产品不依赖集成测试与 GUI 不读测试、无死文件、`site/` 无孤页、顶层目录无双向引用），子仓 `cli/tests/ce.rules` 三条；CI 两根狗粮各加 `ce rules` 腿（第七门），`count:gates` 六 → 七。

### 4.7 门

- 核：朴素不动点参考求值器与半朴素 + 索引求值器在枚举程序 × 种子事实集（200 例）上逐元组等价；分层 / 安全性 / 类别每种程序错误按名；推导链每结点重放规则体得回该元组；聚合与参考实现等价；派生上限降级；golden 八对（步 1 六对：前奏 + schema 回显 / 断言违规 / 带 `why` 的查询 / 语法 / 未绑定 / 不可分层；步 2 加算术 / 集合上的聚合两对，子仓 `query_golden.rs` 钉每条请求行 = 前奏 + 程序经真词法）。
- Rust：词法编号与故障位置（`unit/query/program.rs` 六腿）、图例与核回显同表（`unit/query/legend.rs`）、glob 读不出 = 该记号处的程序错误（`unit/query/face.rs` 负向探针）、`consume` 对健康 / 降级 / 偏斜应答（`unit/query/wire.rs`）；集成：`dead(F)` 与 `ce deadcode` 同一文件、糖 / 聚合 / 算术 / 九种错误同一条路、`ce rules` 退出码与控制台、规则文件三种来源（`it/query_face.rs` 四腿）；三面字节同 = CLI JSON 对库面逐字节、GUI 与 MCP 经同两个库函数（`face_parity` 一行）。
- 精度：无统计门（语义精确）；自仓 `ce.rules` CI 常绿即验收。

## 5. 功能 ②：函数内死代码（`flow/1`）

### 5.1 降表（Rust，每语言一张 `FlowSpec` 表，与 `LangSpec` 同目录）

- `stmts=[[u, seq, parent, kind, flags, aux]]`：单元内语句按前序编号；`kind`：0 block / 1 stmt / 2 if / 3 loop / 4 switch / 5 case / 6 try / 7 catch / 8 finally / 9 return / 10 throw / 11 break / 12 continue / 13 goto / 14 label / 15 noreturn-call；`flags` 位：0 has_else / 1 infinite（常真循环头：Rust `loop`、Go `for {}`、Python `while True`、C `for(;;)`）/ 2 fallthrough / 3 dynamic（单元含 `eval` / `exec` / `load` 一类名的调用，按语言表）/ 4 empty；`aux` = break / continue / goto 的目标 seq（无标签 = 最近的循环或 switch）。
- `vars=[[u, v, declSeq, flags]]`：局部量与形参；`flags`：0 param / 1 captured（被嵌套单元或闭包引用；嵌套单元是独立单元，捕获在宿主侧记位）/ 2 ignored（以 `_` 开头、或语言的丢弃名）/ 3 address_taken（C 族 `&x`、Rust `&mut x` 传出）。
- `uses=[[u, seq, v, mode]]`：按求值序，`mode`：0 read / 1 write / 2 readwrite（复合赋值、`x++`）。
- 树形约定（步 3 定；核 `CE.Flow.Tree` / `CE.Flow.Shape` 按名拒，每条点名行）：`parent` −1 = 单元体，同单元 `seq` 自 0 连续、父在前、单元按序出现；if 的子结点 = then〔+ else〕，`has_else` 位 = 第二个子结点在；switch 的子结点全是 case，`has_else` 位 = 有 default（空 switch 不得带）；try 的子结点顺序 = 体 · catch* · finally?（至多一个 finally）；case 只在 switch 下、catch / finally 只在 try 下；stmt / return / throw / break / continue / goto / noreturn 无子结点；`has_else` 只在 if / switch、`infinite` 只在 loop、`fallthrough` 只在 case，`dynamic` / `empty` 任意；`aux` 只在 break / continue / goto 上（break 指包围它的 loop 或 switch、continue 指包围它的 loop、goto 指本单元的 label），其余为 0；`units` 行的 `params` = 该单元 var 表里形参的个数；`vars` 每单元 `v` 自 0 连续，形参 `declSeq` −1 且位 0 为 1、局部量 `declSeq` 指本单元某语句；`uses` 按 `(u, seq)` 不降、`v` 须已声明。
- 不入表的：表达式内部结构、名字、字面量；只有 `kind` / `flags` / 序号。名字与位置只进 Rust 侧的图例（wire 不送）：每条语句记起点（行，列）、首行文本与终点行，面把类 0 的 `lineEnd` 写成不可达段末句的终点行（提交 E，LEG-1）。
- **降表规则**（步 4 定；每条都是从源码读出的事实，不是判决——核只见整数，这些规则决定它见到什么）：
  1. **单元与容器**：单元 = `scan::functions::extract` 的同一喉口（同一 `fn_kinds`、同一名字、同一序）；单元体 = 文法的 body 字段（表达式体的箭头函数 = 一条 `return`）；语句 = 容器的具名子结点（注释除外）；`block` 类容器 → kind 0，空容器带 `empty`；嵌套单元（闭包 / lambda / 局部函数）不下降——它在自己的单元里判，宿主只记捕获（第 10 条）。
  2. **结构语句的子结点**：if → then〔+ else〕各恰一个子结点（分支是块 → kind 0 块，是裸语句 → 该语句），`elif` / `elseif` 链改写成 else 位上的嵌套 if；loop → 体的语句（或一个块）；switch / match → 每臂一个 case（子结点 = 臂体）；try → 体〔一个块〕· catch* · finally?；Python try 的 `else:` 作体的第二个块子结点（其语句仍可交给 catch——只加路径）。
  3. **终结与跳转**：`return` 9 / `throw`、`raise` 10 / `break` 11 / `continue`、`next` 12 / `goto` 13 / 标签 14 / 名表里的 noreturn 调用 15。noreturn 只认表达式语句顶层调用的被调名与限定名逐字相符：Python `sys.exit` `exit` `quit` `os._exit` `os.abort`；TypeScript `process.exit`；Rust `panic!` `unreachable!` `todo!` `unimplemented!` `std::process::exit` `process::exit`；Go `panic` `os.Exit` `log.Fatal` `log.Fatalf` `log.Fatalln` `log.Panic` `log.Panicf` `log.Panicln` `runtime.Goexit`；C / C++ `exit` `_exit` `_Exit` `abort` `quick_exit` `longjmp` `siglongjmp` `__builtin_unreachable` `__builtin_trap` `std::exit` `std::abort` `std::terminate` 加本文件里带 `[[noreturn]]` / `_Noreturn` / `__attribute__((noreturn))` 的声明名；Java `System.exit`；Lua `error` `os.exit`；R `stop` `quit` `q` `abort` `cli_abort` `rlang::abort` `cli::cli_abort`（R 的 `return(x)` 是 call，按被调名 `return` 入 9）。接收者不定的名（Go `t.Fatal`）不入表：漏报安全、误报不安全。表达式位置的控制流（Rust `?`、`match` 臂里的 `return`、Java switch 表达式、`let v = loop { break 5 }`）不降：核的两种判决都是「存在路径」的否定（不可达 = 无路径到达，死存储 = 无路径读到），有条件的出口只加路径，加路径既不能把可达变成不可达、也不能把活的存储变成死的；漏掉的只有「每个臂都出口」这一种无条件出口——后续语句本应不可达而不报，安全侧（2026-09-30 裁定，§13 第 19 条）。
  4. **表达不了的跳转改写成 goto / label 对**（树形契约只许 break 指包围的 loop / switch、continue 指包围的 loop、goto 指本单元的 label）：C 族 `for (init; cond; update)` = init 作循环前的兄弟语句、循环头只带 cond 的访问、update 作循环体末尾一个 label 结点的子语句，体内直属本循环的 `continue` 降成 `goto` 该 label；Python 的 `for / while … else` = 循环 · else 块 · 一个 label，循环内直属的 `break` 降成 `goto` 该 label；Rust 带标签的块 `'a: { … break 'a; }` = 块 · label，`break 'a` 降成 goto；Java `yield` = `break` 到包围的 switch；Go `fallthrough` = 该 case 置 `fallthrough` 位。
  5. **常真循环头**（`infinite`）：无条件的 `loop` / `for {}` / `for (;;)`；条件是字面 `true` / `True` / `TRUE` / 非零整数字面；Lua `repeat … until false | nil`；`do … while (1)`。
  6. **switch 的两位**：`has_else` = 有 default 臂（C / Java / TypeScript / Go `default`、Python `case _:`）；Rust `match` 恒有（穷尽）；Go `select` 恒有（无 default 也不跳过所有臂），`select {}` 降成 noreturn。`fallthrough` = C / C++ / Java 冒号 case / TypeScript 的每个 case 恒置（语言语义：不 break 即落入下一臂，break 已是独立语句），Java 箭头 case 与 Rust / Python 臂不置，Go 只在臂末是 `fallthrough` 语句时置。
  7. **`dynamic` 位**（单元整体不判；Rust 侧的读法比核注释里的例子宽——凡单元的行为读不出来的）：求值器调用（Python `eval` `exec` `compile` `locals` `globals` `vars` `__import__`；TypeScript `eval` `Function` 与 `with` 语句；Lua `load` `loadstring` `dofile` `setfenv` `getfenv` `debug.*`；R `eval` `evalq` `eval.parent` `parse` `assign` `get` `get0` `mget` `exists` `rm` `environment` `sys.function` `parent.frame` `local` `with` `within` `attach` `source`）、C 族单元体内的预处理条件（`preproc_if` / `preproc_ifdef` / `preproc_elif` / `preproc_else`：两臂互斥，按顺序降表会把两臂读成先后）、计算 goto、`asm`、`setjmp`。
  8. **变量**：形参 = 形参表的每个绑定名（解构逐名；Python 方法的首参 `self` / `cls` 置 `ignored`；Rust `self`、Go 接收者、Lua `self`、Java / TypeScript `this` 不是形参）。局部量按语言表：Python 首次成为绑定目标的名（赋值 / 增量赋值 / `for` / `with … as` / `except … as` / 海象），`global` / `nonlocal` 声明的名不入，推导式与 lambda 的绑定名不入；TypeScript `let` / `const` 块作用域、`var` 函数作用域、解构逐名、`catch (e)`、`for (… of / in)`；Rust `let` 模式、`if let` / `while let` / `match` 臂 / `for` 的模式逐名，块作用域可遮蔽；Go `:=`（同作用域已声明的名是写不是新变量）、`var`、range 变量、`if` / `switch` 的初始化语句；C / C++ `declaration` 的每个声明子、`for` 头声明、`catch` 形参、结构化绑定逐名，`static` / `extern` 局部量不入（跨调用存活）；Java `local_variable_declaration` 逐声明子、增强 for、catch 形参、try 资源、`instanceof` 模式变量；Lua `local` 声明逐名、数值 / 泛型 for 的变量，无 `local` 的赋值是全局写不入；R 首次被 `<-` / `=` / `->` 绑定的名、`for` 变量，`<<-` / `->>` 不入。嵌套函数 / 类的名字在任何语言都不作变量。C / C++ 块里被文法读成函数原型的声明（`T x(a, b);`，最令人烦的解析）不声明局部量，但它形参表里的类型名若解析到域内变量即是读——那是直接初始化的实参，歧义按读解、安全侧（提交 E，CPP-2）。
  9. **访问序**：语句的表达式子树按源序，赋值先右后左（`x = x + 1` 读后写）；复合赋值 / 自增自减 = 2 readwrite；成员 / 下标 / 解引用写（`a.b = v` `a[i] = v` `*p = v` `p->f = v`）与方法调用是对基变量的**读**（别名可见，不算写）；条件求值上下文里的写（`&&` / `||` / `??` / `and` / `or` 的右操作数、三元 / 条件表达式的分支、表达式位置上的 if / match 表达式）降成 readwrite（写不一定发生，读保住此前的存储）；结构语句只带头部的访问（if 条件、循环的条件 / 迭代器与目标写、switch 主语、case 的模式绑定写与 guard 读、catch 形参写、try 资源写）。名字只在「是变量引用」的位置解析（成员名、属性名、键、关键字实参名、类型位置、标签不是；简写字段 `S { x }` / `{ x }` 是读）；解析到最近的可见声明（块作用域按遮蔽，函数作用域按单元）；解析不到的名（全局、函数、字段）不入表。Rust 宏调用：记号树里的每个标识符是读；每个字符串字面量（含 raw string）按 std 的占位符文法读——`{name}` / `{name:…}` 与格式说明里的 `name$`（`{:width$}` / `{:.prec$}` / `{:>w$.p$}`）是读，`{}` / `{0}` 不是，`{{` / `}}` 是字面括号；任何宏一视同仁（用户宏转发 `format_args` 同形，多记一次读只会少报；提交 E，RS-1）。TypeScript 类型位置（类型注解、类型别名、声明的类型）里的 `typeof x` 是读——类型取自那个值，删掉值就编不过（提交 E，TS-2）。默认实参的表达式（R / Python / C++ / TypeScript）作单元首条合成语句的读；其后一条合成语句（`<synthetic:entry>`，只在写了时才有）读单元入口初始化的东西：TypeScript 带 `public` / `private` / `protected` / `readonly` 的构造器形参（参数属性声明并赋值一个字段，TS-3）、C++ 构造器成员初始化列表里的每个实参（CPP-1）。R 单元体里的 `UseMethod` / `NextMethod` / `standardGeneric` / `callNextMethod` 调用把整个调用原样交给方法：该语句读每个形参（提交 E，R-1）。
  10. **豁免位**：`captured` = 单元体内任何嵌套作用域（嵌套单元、Go `func_literal`、C++ lambda、Java lambda 与匿名类体、Python 嵌套 def / class）按名提到的宿主变量；`ignored` = `_` 开头的名、语言的丢弃名（Go `_`）与第 8 条的 `self` / `cls`；`address_taken` = C / C++ / Go `&x`、C++ 引用绑定 `T& r = x` / `auto& r = x`、Rust `&mut x`。TypeScript 的块级绑定到调用时才过暂时性死区：嵌套作用域按名提到一个此刻还解析不到的名，其后在当时仍开着的作用域里声明它，该局部量置 `captured`（自身初始化式里的闭包 `const a = z.lazy(() => a)`、更早的闭包 / getter 引用稍后的 `const`；只在 TS 表开——Lua `local f = function() return f end` 里的 `f` 是全局；提交 E，TS-1）。
  11. **总性**：降表按构造只产合法树形；一个降不出合法形的单元不发（Rust 侧记 `unlowered` 并计数）；核若仍按名拒绝某行，测量侧把该行映射回它的单元、去掉该单元重发（拒绝理由与单元记进图例），一批请求里一个单元的缺陷不连累别的单元。

### 5.2 判决（核 `CE.Flow.*`）

- 建控制流图（`CE.Flow.Cfg`）：结构化控制流按 `kind` 展开；`try` → `catch` 保守（`try` 体每条语句都可能跳到本 try 的每个 `catch`）；`finally` 在每条离开 try 体或 catch 的边上——return / throw / noreturn / break / continue / goto 一律先经 finally 再到目标或出口，finally 的汇合点接所有待续目标的并集（只加路径不减，比逐目标复制 finally 多出的路径只会让判决更保守）；`infinite` 位的循环头不出去；`goto` / `label` 按 `aux`；`noreturn-call` 与 `throw` / `return` 是出口。
- 可达性 → `unreachable`（kind 0）：从入口不可达的语句，一条极大连续段报一条（`seq`…`seqEnd`）。
- 反向活性 → `dead_store`（kind 1）：写入后在所有路径上被覆盖或到出口前从未读（同语句内的访问按求值序倒走——`x = x + 1` 先读后写，`readwrite` 的读也算读；同一语句对同一变量的多次写各自判，任一为死即报该 `(seq, v)` 行一条；`address_taken` / `captured` 的变量不判；无任何读的变量只报未用、不报死存储；不可达语句里的写已由不可达段覆盖，不重复报）。
- `unused_local`（kind 2）：声明后无任何读（`ignored` / `captured` / `address_taken` 不判）；`unused_param`（kind 3，顾问）：形参无读（同三种豁免；接口 / 重载 / 覆写的形参本就可能不用——只报不门）。
- 任一语句带 `dynamic` 位的单元整体不判（记 `counts.dynamicUnits`；`counts` 的其余五个数照记）。

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
- MCP `flow`；GUI 报告枢纽的 flow 族（自有渲染器：计数芯片、四个种类筛选芯片、发现表带判决 / 顾问标、被拒单元表；§13 第 26 条）。

### 5.5 门

- 核：判决与**不建控制流图**的轨迹参考（`ReferenceFlow`：树遍历枚举每条执行轨迹——每个分支、每个 case、循环头至多两次、try 体内每条语句都可交给 catch、finally 后按待续完成的并集续走——从轨迹读发现）在 `ReferenceFlowGen` 的 200 个有籽随机结构化程序上逐条同；契约拒绝 42 条按名（`FlowRefusals` 文本表逐条钉：行形与域值 14、表间一致 14、树形 14）；golden 六对（四种发现各一 + 经 finally 的 return〔不可达 + 死存储〕+ 树形拒绝一；请求行按 §5.1 手写、步 4 起由 Rust 降表重生；降级面由电池以 524,289 行的运行时请求探——夹具不装 7 MB）。
- 精度考题（每语言；登记册 `docs/EVAL-SET-FLOW.md`，仪器在测试子仓 `it/eval_flow_parts/`）：**语料** = v2.30 考题登记表的第一个语料（C lua、C++ fmt、Java gson、Lua luarocks、R stringr）+ M1 对拍语料（Python requests、TypeScript zod、Go cobra、Rust ripgrep）+ TSX 取 zod 的 `.tsx` 文件 + Rust 另加本仓（钉在降表落地的提交，克隆名与 HTML 考题的本仓克隆分开）。**宇宙**（`flow-slice-<语言>-<语料>-v<代>.json`；zod 在 TypeScript 与 TSX 两门考题里各出一份）= 走查读到的每个该语言文件的每个单元：文件 sha、单元序号 / 名 / 行段、四表行数、每类每层的候选池大小。**候选池**按类从降出的四表读，不问核（所以能在判决之前冻结）：0 不可达 = 前一兄弟本身是终结叶（return / throw / break / continue / goto / noreturn）或常真循环的语句〔层 A〕、前一兄弟是子树含终结叶的结构语句的语句〔层 B，工具最容易误标的一类：`if … return` 之后〕、前一兄弟是其余结构语句的语句〔层 C〕；1 死存储 = 有读的非豁免变量（含形参）上，同一变量按 seq 序的下一次访问是写的写〔A〕、是该变量最后一次访问的写〔B〕、其余写〔C〕；2 未用局部量 = 文本上零读的局部量〔A〕、有读的〔B〕；3 未用形参同 2（顾问，只记不门）。**抽样**（`flow-sample-<语言>-v<代>.json`）：每类每层各取 min(15, 池)，秩 = sha256(域 | 语料 | commit | 路径 | 单元序号 | 类 | 层 | 行 | nth)，域 `ce-flow-site-v1`，审阅序另按 `ce-flow-audit-v1` 排列；一道题 = 语料、路径、单元名与行段、类、行号、该行第 nth 个同形锚（语句 / 对该名的写 / 声明）与变量名——层留在样本档里（门用它重算秩；层是源码事实，不是答案），**交给审阅代理的那一批不带层、不带产品的答案**。**盲判**（`flow-review-<语言>-v<代>.json`）：没看过判决的独立代理只读钉住克隆与自己那一批，逐题答 `unreachable | reachable`、`dead | live`、`unread | read`（各带 `cannot_tell`）与理由。**精度册**（`flow-precision-<语言>-v<代>.json`）：在审阅表之后的干净树上生成，每题记产品的答案（该语句落在某条不可达段内 / 该 `(seq, v)` 是死存储行 / 该变量是未用行）与判词 tp / fp / tn / fn / unjudged（单元未降出、被核拒或 dynamic）/ cannot_tell，每类另记样本里的正例数 `positives` = tp + fn 与负例数 `negatives` = tn + fp。**门 = 每语言每种非顾问发现（0 / 1 / 2）读四态：`fail`（fp ≥ 1）、`pass`（fp = 0 ∧ tp ≥ 1）、`vacuous`（fp = tp = fn = 0：样本里没有正例可找，准入靠负例上的零误报；零行也归此态）、`silent`（fp = tp = 0 ∧ fn ≥ 1：有正例而一个没报，不准入）；三门各 ∈ {pass, vacuous} 即 `judged`（§13 第 25 条）**；召回（fn）只记不门；达门的语言进 `flow::judged_mask()`，未达的只 observe（逐语言发布门，v2.30 §14 第 9 条先例）。**顺序**：降表与语言表先于抽样落地（题从降出的表里抽；与 C / C++ 阶梯先于考题同一种 `ladder_first` 形，v2.30 §14 第 14 条），抽样 ≺ 审阅表 ≺ 精度册由三档 `generated_from` 的提交祖先证明（门 `cli/tests/it/flow_provenance.rs`），降表的首个提交是抽样提交的祖先或就是它，抽样到审阅表之间 `cli/src/flow`（`mod.rs` 除外）零提交（盲窗只读降表），精度册从自己的提交起钉在回答它的代码上（`ANSWERED_BY` = `cli/src/flow/` 下降表与 wire 的文件〔`mod.rs` 除外：模块表与 `judged_mask` 是政策不是答案，填掩码不退役精度册〕、`cli/src/scan/functions.rs`、`cli/src/scan/walk.rs`、`cli/src/scan/lang.rs`、`cli/Cargo.lock` 的钉版）——降表一动即退役到重生成为止；审阅之后为修缺陷改表是允许的，在登记册具名记一条。**代际**：降表改动挪了某门的候选池（逐（单元，格）比池计数），这一门才出下一代——考题表的 `generation` 列加一，宇宙与样本在改动后的树上重导，全部题重判；前一代的宇宙 / 样本 / 审阅档留在盘上作记录，门只读考题表指向的代；池一格没动的门保留原代。题的身份是审阅域哈希 `audit`，两代的共有 / 新增 / 消失按它数（§13 第 28 条）。
- 回放台账（每语言）：仪器 `it/fpr_flow_replay.rs`（`--ignored`，release），冻结件 `contracts/eval/fpr-flow-v1.json`，执行者 `it/fpr_flow_gate.rs`（行内算术从冻结行重算、`FPR-REPLAY.md` 引用表逐字）；语料 = 该语言考题的第一个语料，窗口 = 从 tip 往回 400 个第一父提交；单元身份 = (路径, 单元名, 形参数)，发现身份 = (类, 锚)——锚在 0 是不可达段首语句的源文本、在 1 是变量名 + 写语句的源文本、在 2 / 3 是变量名；一条发现在后来的提交里随该单元被改而消失 = 真阳，随该单元被改却留下 = 误拦（严格口径），窗口末仍在且单元未再被改 = 未定（不计）；读数写进 `FPR-REPLAY.md` 新节，两口径并记，门读精度考题。

## 6. 功能 ③：克隆合并建议（`merge/1`）

### 6.1 组

- T1/T2：`ce dedup` 的组（端点相同的块连通分量）里覆盖整单元的成员——每成员一棵树；只覆盖片段的块按整块（片段树）成组——测量侧包一个合成根送来，核只认 `clone/1` 的单树契约——建议面标 `fragment`。
- 片段按成员间共同的顶层结点形修边（成员 0 的最长连续段且每个成员都连续含有；无共同段 = 不同形、不送）；成员的身份仍是克隆族行段（`ce dedup` 点名的 `path:a-b`），保留段的行段另记为 `run`——请求 `members` 行的 `lines` 送 `run` 的行数，核按要合并的那段计价，整单元成员 `run` = 身份段（§13 第 32 条）。
- T3：`t3ted` 判 1 的对，每对一组（两成员）。
- 一个成员集只出一条建议：两族按成员身份的有序元组去重，同集时 T1/T2 盖过 T3（精确同构是比近似更强的主张，与 `score::one_row_per_pair` 同一立场）；去重在本地预检之后、送核之前，被盖掉的组计 `merged_duplicates`、不进 `unsendable`（§13 第 33 条）。
- 组内成员同语言（克隆族按构造保证）。

### 6.2 树（与 `clone/1` 同一后序编码加四列）

`trees=[{lab, lld, leaf, slot, own, text}]`：`lab` / `lld` 与 `clone/1` 同（命名结点后序、kind 哈希、最左叶）；`leaf` = 叶结点原文的 fnv1a64（标识符与字面量各自的文本；内部结点 0；核以 `Integer` 读，fnv1a64 可超 Int64）；`slot` = 结点位置类：0 语句位 / 1 表达式位 / 2 类型位 / 3 名字位（声明名与局部赋值的裸目标）/ 4 其他；`own` = 结点**自己的匿名记号**（直接子结点里的匿名记号，穿过匿名中间结点取其叶、不进任何命名子结点——正是树的子结点遍历跨过的那些记号）按源码序、记号之间以一个 0x00 相隔的 fnv1a64，一个也没有 = 0；`text` = 子树**整条记号流**（命名叶与匿名记号，源码序，同样以 0x00 相隔）的 fnv1a64——空白不是记号，`()` 与 `( )` 的 `text` 相同而 `leaf` 不同，单记号叶的 `text` 恰等于 `leaf` 是巧合不是契约（§13 第 42 条）。六列出自 `emit` 的同一次遍历（`own` / `text` 在 `cli/src/dedup/t3/tree_text.rs`）；片段的合成根 `own` 0、`text` = 各顶层结点的记号流以 0x00 首尾相接。`clone/1` 的请求同批接受可选的 `leaf` 列（长度须等于结点数，`ted` 不读它——判决字节不变），测量侧的 `clone/1` 请求只发 `lab` / `lld`；`merge/1` 要求四列都在，缺列 / 长度不等 / 负值按名拒。组行 `groups=[[g, family, helper]]`：`helper` = 片段组合并后辅助函数的头尾行数（该语言 slot 表的 `helper_lines`：Python 与 Haskell 1、其余十种 2），整单元组 0（§13 第 47 条）。

位置类由每语言一张表给（`cli/src/merge/slot*.rs`：语句类读 `FlowSpec` 既有字段、不另立语句表；没有流表的 Haskell 在自己的表里带 `stmt_kinds` / `container_kinds`；表不住 `cli/src/flow/`——那个目录去掉 `mod.rs` 是 flow 精度册回答的降表，§13 第 34 条），是语法事实。分类次序（§13 第 49 条）：语句位 → 声明名（`name_fields`；排在类型之前，`struct_item` / `class_declaration` / `type_spec` 的名字因此可达）→ 类型 → 赋值目标（`target_fields` 与 R 的 `target_ops`：目标根是裸 `identifier` = 3〔局部重命名〕，否则 4〔成员 / 下标 / 解引用目标，参数接不到写入〕；`target_lists` 是穿过的目标列表）→ 表达式的组成名与字面量内文（`part_fields` / `part_kinds` = 4，由一级拓宽交给父表达式）→ 表达式 → 其他 4。merge 送的树不含文法的 extra 结点（注释）——T1/T2 的记号流本来没有注释；`clone/1` 的判决树照旧带（§13 第 31 条）。

### 6.3 判决（核 `CE.Merge.*`）

- T1/T2 组（`family` 0，≥ 2 成员）：成员树同构 = 每成员 `lld` 逐位相等且内部结点的 `(lab, leaf)` 逐位相等（T2 把标识符与字面量归一到同一记号流，叶的 kind 可不同；同构不读 `own` / `text`）；不同构按名拒 `group <i>: members are not isomorphic`；逐结点对齐：`(lab, leaf)` 不全同的叶 = 重标洞，内部结点 `own` 不全同 = 自己的记号洞（类 4）。
- T3 对（`family` 1，恰 2 成员）：Zhang–Shasha 的最优映射（`CE.Clone.Ted.tedMapping`，与 `ted` 同一单元格递推、距离值不变，并列先取匹配、再删、再插）自顶向下收窄——只保留祖先对也在映射里的对，根对不在映射里 = 无骨架、整对一个间隙洞；保留对的 key（`lab`，叶加 `leaf`）不等 = 重标洞，key 相等而 `own` 不等 = 自己的记号洞（类 4，差在运算符 / 关键字 / 标点；§13 第 43 条）；保留对的子结点序以保留子对为界切段，两侧未映射的子树序列各成一个间隙（一侧可空），两侧森林逐结点 `(lab, leaf, 相对 lld)` 相等则并入骨架不成洞，否则成间隙洞——间隙是结构差：森林含语句位结点 = `spans_statements`，否则 `position`，不可行（§13 第 45 条）；唯一例外是一侧为空、另一侧有子树、且保留父对两侧都在表达式位的间隙，它按下条一级拓宽（§13 第 52 条）；洞的取值格 = 该侧森林的后序 `(lab, leaf, 相对 lld)` 序列。
- 一级拓宽：重标洞落在类 4 的叶上（两族同此），或一侧为空、另一侧有子树的间隙落在保留父对之下（只 T3 对有间隙；§13 第 52 条），且这个父对两侧都在表达式位（1）→ 洞改覆盖父对整棵子树（骨架处一个间隙、取值格 = 子树、首末根 = 子树首尾），类 1；父对内部的其他洞（父结点自己的 `own` 洞、子间隙）被吸收。只拓一级、只拓类 4 的叶与一侧为空的间隙；父对不全是表达式位（例如赋值目标的成员访问、实参表）→ 留在原处、`position`（§13 第 44 / 52 条）。
- 空洞：取值文本向量各成员全等的洞（只差空白）留在骨架里供复原，但不是参数、不定理由、不出洞行。
- 洞序 = 键 `(anchor, rank, tie)` 升序（§13 第 46 条）：结点 n 处的洞（重标 / 自己的记号 / 拓宽后的父结点）= `(n, 1, 0)`；成员 0 侧非空的间隙 = `(首根的 lld, 1, 0)`；成员 0 侧为空的间隙排在它前一个保留子对之后、下一个之前：`anchor` = 下一个保留子对在成员 0 的结点的 lld，没有则 = 父结点自身，`rank` 0，`tie` = 成员 1 侧首根下标（同锚的外层间隙先于内层）。即成员 0 的后序：内层先于外层、兄弟从左到右、结点自己的记号在它里面的一切之后。
- 参数身份按文本（§13 第 48 条）：每洞每成员一个文本向量——重标叶 `[text]`、自己的记号洞 `[own]`、拓宽洞 `[父的 text]`、间隙 `[各根的 text]`（空侧 `[]`）；向量相同的洞是同一个参数（同文本不同 kind 的两处也是一个），参数按首现编号，参数数 = 互异向量数；取值格 `(lab, leaf, 相对 lld)` 只留给复原律。
- 可行：每个非空洞在表达式位（1）或名字位（3）→ reason 0；否则按洞序第一个不可行洞记 reason——间隙洞的森林含语句位结点 = 3、其余间隙与语句位或其他位（0 / 4）的洞 = 1、类型位（2）= 2；洞全可行但参数 > 6 = 4；洞与参数都过而收益 ≤ 0 = 5 `no_savings`（省不下一行的合并没人会采，与 4 同一立场；§13 第 32 条）；`feasible` = reason 为 0。
- 收益 = Σ 成员行数 − （骨架行数 + 成员数 × 1）；骨架行数：T1/T2 = 保留者行数；T3 = ⌈保留者行数 × 保留对数 ÷ 保留者结点数⌉（wire 无逐结点行号，整数估计）；片段组两族都再加 `helper`（§13 第 47 条）；可为负，照报。
- 保留者 = 文件入度最大的成员，并列取 `m` 最小（Rust 送 `fileIndeg`，图侧事实）。

### 6.4 面

`ce merge [--group <n>] [--format json]`：报告 `ce.merge-report/0.1.0`（每组 `{family, members[{path, unit, lines, run}], params, kept, savings, feasible, reason, holes[{param, values[{member, text}]}]}`——`text` 由 Rust 用成员源文件按 `post` 回标）；MCP `merge_suggestions`；GUI = reports 枢纽里的 merge 报告（自定义渲染：组列表 + 参数对照表；§13 第 26 条的读法）。顾问：不进门、不进基线。

### 6.5 门

- 核：反统一定律——实例化（骨架 + 每成员的参数向量）逐树复原每个成员；极小性（少一个参数即有成员复原不出）；映射版 TED 距离与原 TED 逐对相等（随机树电池）；golden 六对。
- 冻结建议集：本仓 + 四语料的建议集冻结（`contracts/eval/merge-suggestions-v<代>.json`，门读最新一代，旧代留盘作记录），改动即漂移门；盲判精度（可行 / 参数数由独立代理按源码判，第二代起批次提示带 `## Reading rules`——与核的规则逐条对应，§13 第 50 条），读数入册。

## 7. 功能 ④：架构分析（`arch/1`）

### 7.1 表

`files=[[F, D, lines]]`（measured 文件结点）、`dirs=[[D, parent]]`（稠密树，根 0）、`edges=[[F, G, w]]`（文件到文件，w = 该对之间的引用边数，任何 kind 与 rung）、`pkgEdges=[[F, D, w]]`（包粒度的引用——Go / R / Java 的包导入与 Markdown 的目录链接，`structure/1` 的 `dirEdges` 至今丢掉的那一类——核折成 dir(F) → D）、`focus=[F…]`（`--impact` 点名的文件）。

### 7.2 判决（核 `CE.Arch.*`）

- 目录图：结点 = 全部目录；文件边按目录聚合（同目录内不计）、`pkgEdges` 的 F → D 折成 dir(F) → D（同目录不计），同一对目录的两种边合并、权相加。
- 分层：反馈弧集按目录图的每个强连通分量各算——分量 ≤ 14 顶点用按顶点子集记忆化的排序 DP 精确（线性序的代价 = 后向弧的 (权和, 弧数, 按 (D,E) 排序的弧表) 三元组字典序；`cost(S) = min over v ∈ S of cost(S∖{v}) ⊕ v 排在 S 末尾时 v → S∖{v} 的弧`，2^14 × 14 状态，回溯取序、cuts = 后向弧集），`exact = 1`；更大用 Eades–Lin–Smyth 加权贪心（汇点入尾段、源点入首段、否则取出权 − 入权最大者、并列取 id 小，每摘一点重判）再按 (D,E) 升序逐条试放回、放回不成环即留下，`exact = 0`；权 = 边的引用数；`cuts=[[D,E,w,exact]]` 按目录粒度出（文件级展开归测量侧——包边没有 G，目录粒度是唯一无洞的形）；去掉 cuts 后 level(D) = 0 若无出弧、否则 1 + max level(E)，`layers` 每目录一行。
- 聚类：文件图（只用 `edges`，无向、两向权相加）上的确定性 Louvain：从单点社区起、按结点 id 升序局部移动，增益按 2m·k_{i,C} − Σ_tot(C)·k_i 的整数形比（留在原社区的增益也按先把 i 摘出算）、并列取社区 id 小、只在严格优于留在原社区时移动，一遍无移动即收敛；只聚合一次（超结点按最小成员 id 编号，第二层再局部移动到收敛）即停；簇号按簇内最小文件 id 升序重编 → `clusters`；簇的多数目录 M = 簇内文件最多的目录（并列取 D 小）；文件 F 错位 ⇔ M 在簇内的文件数**严格大于** dir(F) 在簇内的文件数（并列 = 无证据、不错位；单文件簇由此不错位；2026-09-30 由 golden 第 7 对的环形收紧：15 个单文件目录成环时旧规则判 11/15 错位而无一处有证据）= `misplaced=[[F, M]]`。
- 影响面：从 `focus` 沿反向文件边 BFS（`pkgEdges` 的 F → D 读作 F 引用 D **直属**的每个文件，不含子目录），`impact=[[F, depth]]`，focus 自身 depth 0。
- 度量：每目录 fanIn / fanOut = 有弧指向它 / 它指向的互异目录数（去 cuts 前）；不稳定度 I = ⌊1000 · out ÷ (in + out)⌋，in + out = 0 记 −1。

### 7.3 面

`ce arch [--impact <path>…] [--format json]`：报告 `ce.arch-report/0.1.0`（分层表、拆环建议〔目录对带引用数与 `exact`，其下由 Rust 展开成文件级边——`edges` 里落在该目录对的行加 `pkgEdges` 里落在它的行〕、簇〔簇号是核的，Rust 只回标路径与多数目录〕、错位文件、影响面、目录度量〔−1 → null〕；核起不来或答不了、核不提供家族、本地越上限 = `degraded` 具名、六表空，与查询族同一姿态；核对已计价的请求答 degraded 是上限镜像漂移 = 错误）；MCP `architecture`（`impact` 路径数组）；GUI = reports 枢纽里的 arch 报告（自定义渲染：分层带 + 拆环表 + 错位 / 簇 / 影响面 / 度量表，枢纽的路径输入喂 `impact`；§13 第 26 条的读法）；`--impact` 点名不是 measured 文件的路径按名拒、退 2。顾问。测量侧的表：文件结点 = `structure` 同一条取图路的 measured 文件结点按路径序，目录 = `structure::tree::build` 的树（parent < D），`lines` 与 `structure` 的缝定价同一来源，`edges` 收任何 kind / rung 的文件 → 文件边（section 目标折到其文件，自环丢），`pkgEdges` 收 `GRAN_PACKAGE` 目标且目录在树内的边，两表各按 (F, G) / (F, D) 合并计数。

### 7.4 门

- 核：分层与「去掉 cuts 后无环」互证；精确 FAS（顶点子集 DP）与两个独立参考——弧子集穷举（≤ 12 弧）与全排列枚举——在 4 顶点全体有向图与两族有籽随机图上逐权、逐弧表等价；贪心 FAS 结果无环且极小，对 DP 最小值的比只记录不断言（ELS 是启发式、无常数界；2026-09-30 首轮电池在 8–10 顶点强连通图上观测到最差 4×，正是把精确范围从 ≤ 12 弧抬到 ≤ 14 顶点的缘由）；`exact` 位 ⇔ 分量 ≤ 14 顶点；Louvain 确定性（同图两跑逐字节同）；影响面与朴素闭包等价；golden 七对。
- 冻结自仓读数：本仓**一个钉住提交**的分层 / cuts / 目录度量 / 错位 / 簇计数冻结（`contracts/eval/arch-self-v1.json`；门用 `git archive <commit>` 去掉 `.gitmodules` 物化那棵树再量——树不动，读数只随测量或核的行为动，漂移即真漂移；换钉提交 = 按 EVAL-SET 复活协议改签）。

## 8. 核的模块布局与尺寸

每个家族 = `CE/<Family>.hs`（respond、解码、应答）+ `CE/<Family>/Cost.hs`（上限、地板、码域）+ 判决模块若干（`Query`: `Contract`（请求形与拒绝）、`Syntax` / `Parse`（记号流 → 子句）、`Check`（安全性 / 类别 / 分层；`Check/Sorts`、`Check/Safety`）、`Eval`（半朴素 + 索引；`Eval/Index`、`Eval/Join`）、`Proof`、`Schema`；`Flow`: `Contract`（请求形与拒绝）、`Tree`（四表 → 单元树）、`Shape`（树形契约）、`Cfg`、`Reach`、`Live`；`Merge`: `Align`、`Holes`、`Ted` 的映射扩展；`Arch`: `Layers`、`Fas`、`Louvain`、`Impact`），每文件 ≤ 290 行（`core_size_gate` 的真实上限）。电池 `core/test/<Family>Props.hs` 各一（`WireHarness.runLegs` 两条平行列表形，避开查重门的表同韵）。`core/test/Spec.hs` 现 289 行，每加一家族 +3 行——步 1 先把电池清单拆到 `core/test/Batteries.hs`（`fixture_contract.rs` 读的那三行形不变、只换文件）。

## 9. 验收与门（每步共用）

- 两仓 `cargo test / clippy --all-targets -- -D warnings / fmt --check`、`cabal test`、七条产品腿（主根与 `cli/tests`，含 `ce rules`）、golden 重生（`CE_BLESS=1`）、`fixture_contract` 的 golden 清单、`core_wire` 往返、`face_parity`、`facts_*`、`docs_*`、`site_*`；两仓 ADR-006 具名重立；dedup 预算只降不升（新块先消后入账）。
- 既有判决不动：每步旧二进制 / 新二进制在十语料十面 + 自仓干净树十面对拍逐字节同（步 7b 的形）。
- 全量 it 在每步提交前跑一次（release 或 debug 按步的重活定），红先单跑读 panic 再定抖动 / 真红。

## 10. 文档与事实面

- 计数事实：`count:families` 十二 → 十六、`count:screens` 十一 → 十五、`count:mcp_tools` 十六 → 二十一（`it/facts/form.rs` 的数词表到 twenty，先扩到 thirty）、`count:booklets` 十五 → 十九、`count:gates` 六 → 七、`count:golden_requests` 152 → ≥ 176；`LITERALS` 里手改的字面（stack.svg 两对的家族行、how 页标题、六处 alt 文字「N families」）逐处改。
- 官网 how 页现 732 行、硬线 750：四张家族卡放新页对 `site/how/analysis/`（en / zh 各一），how 页目录面板与两首页各加一行入口；`site_contents.rs` 假定 f15 是末卡，改读 f16–f19 在新页；`verify_site.js` 与部署清单加两页。
- 方法学册 16–19（每家族一册：判决定义、表、门、读数）+ `methodology.md` 索引四行 + 前后导航；架构图 IR（家族数 / 屏数 / 工具数子标签）与判决图（tag 行加四个家族）重渲；README 双语命令表加五行、parity 块随 bless、「三面一体」句、How it works 四行、技术栈条目；plugin/README 工具句；`VERSIONING.md` 四条；`ce-toml.md` 与 `cli.md` 再生；`DAEMON.md` 一行（flow 腿）；CHANGELOG 每步一块；计划书横幅与 T 轨行随步。

## 11. 语言条：量法与记账

读数 = `gh api repos/skymanbp/CodeEraser/languages` 的字节表（`.gitattributes`：`cli/tests/**` 与 `contracts/**` vendored、`site/**` documentation、`core/test` 计入）。立项日 2026-09-29：Rust 2,144,472 / Haskell 592,241 / 其余约 252,000 B，Haskell 19.8 %；40 % 需要 Haskell 约 +1.0–1.2 MB（核与电池各按 ~44 B/行计 ≈ 两万行加八千行电池），Rust 的新增同时抬分母。每步收口写一行读数进 CHANGELOG 该步块；发版声明的读数以 v2.32 收口时为准（2026-10-01 用户裁 1.9.0 推迟到 v2.32 做完再发，读数随 [authority-track.md](authority-track.md) 步 8 记账）——若低于 40 %，差距与可选路线以 AskUserQuestion 上呈（已排为 v2.32 步 9），不自行拉伸。

## 12. 分步（每步自带门，落码顺序）

| 步 | 内容 | 门 |
|---|---|---|
| 0 | 细则与立项（2026-09-29）：本册 + 计划书 v2.31（横幅细则句、ADR-008 细则第七期、§6 T 轨十二步）+ CHANGELOG `[Unreleased]` 块 + cc-memory 十二步锁定 | docs 门全绿、基线具名重立、CI 绿 |
| 1 | 查询 A（2026-09-29 已交付）：核 `CE.Query.*`（Contract / Syntax / Parse / Check〔Sorts · Safety〕/ Eval〔Index · Join〕/ Proof / Schema / Cost，十三模块 1,501 行）+ `QueryProps` 十二腿 + `ReferenceQuery` 朴素参考 200 例 + golden 六对 + proto 7.3.0（`Protocol.hs` 一行、`Version.hs`、`corelink.rs`、VERSIONING 一条）+ `Spec.hs` 拆 `SpecProbes.hs`（四条探针腿；设计名 `Batteries.hs` 按搬出的内容改，电池表留在 `Spec.hs`）+ 子仓 `fixture_contract::regen` 腿（`LineSession` 三件套同供 MCP 会话与 golden 往返） | `cabal test` 439 ok、参考求值器等价 200/200、既有 golden 只动 proto |
| 2 | 查询 B（2026-09-29 已交付）：Rust `cli/src/query/`（lexer / program / legend / columns / facts〔graph · units · pairs · text〕/ wire / face / console，前奏 `prelude.rules` 八条）+ `ce query` / `ce rules`（`main_query.rs`）+ `[rules] file`（canonical 规则 7 丢弃）+ MCP `query` / `rules` + GUI Query 屏（第十二屏）+ 自仓 `ce.rules` 九条与子仓三条 + CI 两根第七腿 + 册 16 + 官网 `site/how/analysis/` 页对 + 事实与 parity；golden 六 → 八对 | 三面字节同、`ce rules` 自仓绿、十语料十面对拍同 |
| 3 | 死代码 A（2026-09-29 已交付）：核 `CE.Flow.*`（Contract / Tree / Shape / Cfg / Reach / Live / Cost + `Flow.hs`，八模块 890 行）+ `FlowProps` 六十腿（`FlowCases` 十三例判决 + `FlowRefusals` 四十二条拒绝——两张文本表，一例一腿——+ 四探针） + `ReferenceFlow` 轨迹参考 200 例（`ReferenceFlowGen` 有籽生成，两文件各在 290 行墙内）+ golden 六对 + proto 7.4.0（`Protocol.hs` 一行、`Version.hs`、`corelink.rs`、VERSIONING 一条、子仓 `core_wire.rs` 阶梯与 `fixture_contract` 清单各一行） | `cabal test` 507 ok、轨迹参考等价 200/200、契约拒绝 42 条按名、既有 golden 只动 proto |
| 4 | 死代码 B（提交 A 已交付 2026-09-30：`FlowSpec` 十语言表〔A1〕+ 降表十三模块〔A2〕+ `flow/1` 接线〔A3：按 `rowCap` 分批、拒绝驱动的剔除、严格 consume；golden 六对的请求行改由真源码降出〕；九语料 1,173 单元真核回放 refused 0）；提交 B 已交付 2026-09-30：考题仪器七模块 + 二十一份冻结档〔十一份宇宙、十份样本，`unlowered` 全 0〕+ 登记册 `docs/EVAL-SET-FLOW.md`；提交 B′ 已交付 2026-09-30：盲判 53 批 1,197 题零 `cannot_tell`、十份审阅档 + 盲判仪器四件 + 登记册盲判节；提交 C 已交付 2026-09-30：十份精度册 + 四态门 + 出处门 + `judged` 掩码〔Python / TSX / Go / C / Java / Lua / R〕；提交 E 已交付 2026-09-30：降表回修八类 + 图例终点行、十份第一代精度册退役、掩码清空；提交 F 已交付 2026-09-30：第二代考题四门（cpp / r / rust / typescript）的五份宇宙与四份样本在 E 的树上重导，考题表加代际列，批次提示加语言读法一节（`readings` 2）；提交 D1 已交付 2026-09-30：回放台账仪器 `fpr_flow_replay` + 执行者 `fpr_flow_gate`（台账阶段 `Pending`，§13 第 29 条）；提交 F′ 已交付 2026-09-30：第二代盲判四门 20 批 454 题零 `cannot_tell`、四份第二代审阅档（`readings` 2），与第一代共有 448 题一致 447，登记册拆出第一代归档册 `EVAL-SET-FLOW-GEN1.md`；提交 G 已交付 2026-09-30：十份精度册在 F′ 的干净树上重生成（四门读第二代审阅档），类 0–2 十门零误报、`judged` 十门皆真、掩码十位全填，六门退役的第一代精度节搬进归档册（§13 第 30 条）；提交 D2 已交付 2026-09-30：十行冻结件在 D1′ 的干净树上重量入库（`dirty` = false）、台账阶段翻 `Frozen`、三张表与读数落 FPR-REPLAY.md；步 4 至此收口 | 逐语言精度 ≥ 99 %、顺序门 |
| 5 | 死代码 C（2026-09-30 已交付）：`ce flow`（`main_flow.rs` + `cli/src/flow_report/` 面与控制台）+ 守卫腿（`guard/flow.rs` + `guard/flow_novel.rs`，daemon 2.2.0 加性 `flow` 请求）+ Stop / precommit / commitmsg 行的 `flow` 对象（feed `ce.observe` 0.12.0 加性）+ `[flow] tier`（出厂 observe、进指纹）+ MCP `flow` + GUI 报告枢纽 flow 族 + 册 17 + parity 行 | 三面字节同、守卫在判决语言（Python）一侧出声 / 在顾问语言（Rust）一侧静默；FPR 台账随回放台账 |
| 6 | 合并 A（落地第一部分 2026-09-30，与步 7 同一提交）：核 `CE.Merge.*`（Contract / Tree / Align / Mapped / Holes / Cost + `Merge.hs`，七模块 645 行）+ `CE.Clone.Ted.tedMapping`（与 `ted` 同一单元格递推）+ `MergeCases` / `MergeRefusals`（24 条拒绝按名）/ `MergeProps` + `ReferenceTreeGen` + golden 六对 + proto 7.5.0（`clone/1` 可选 `leaf` 列；十四份既有 golden 只动 proto 字面与 hello 能力表） | 定律电池、TED 距离不变 |
| 7 | 合并 B（落地第一部分 2026-09-30：`cli/src/merge/`〔groups / groups_trim / wire / face / console + slot 表八个文件〕、`dedup/t3/tree.rs` 一次遍历出 `lab` / `lld` / `leaf` / `slot`、`ce merge`〔`main_merge.rs`，头部读 `main_prelude.rs`〕+ MCP `merge_suggestions` + GUI 报告枢纽 merge 族〔`hub_merge.js`〕+ 冻结建议集五语料 + 对判决盲的抽样 100 行与四批批次 + 册 18；落地第二部分 2026-09-30：四个独立判官的盲判〔四批各 25 题、100 行〕、审阅档 `merge-review-v1.json`、精度册 `merge-precision-v1.json`〔由三档按定义算出、与审阅档同一提交，第 36 条〕、登记册 `docs/EVAL-SET-MERGE.md`〔三读数、核 reason × 判官 reason 混淆表与 23 条理由分歧逐条带判官 note〕；读数 feasible_agree 83 / 100、reason_agree 77 / 100、params_agree 45 / 100，只入册不设门；第二代第一部分 2026-10-01：23 条理由分歧逐条对源核实后的九条裁定〔§13 第 42–50 条：`own` / `text` 两列与组行 `helper`、自己的记号洞、一级拓宽与空洞、间隙一律结构差、洞序按成员 0 后序、片段骨架计辅助函数头尾、参数身份按文本、位置类表三修、批次读法即定义〕落进核与降表，proto 7.5.0 同一未发布 minor 内加性，`clone/1` 旧 / 新二进制在本仓 + ripgrep + zod 上 `ce clone` / `ce check` / `ce dedup --check` 逐字节同；冻结集 / 抽样 / 批次第二代 `merge-suggestions-v2.json` / `merge-sample-v2.json` / `merge-batches-v2`，第一代四档留盘作记录；第二代的盲判与精度册由下一提交落） | 三面字节同、冻结集门 |
| 8 | 架构 A（落地 2026-09-30，与步 9 同一提交）：核 `CE.Arch.*`（Contract / Cost / Dirs / Fas / Layers / Louvain / Impact + `Arch.hs`，八模块 653 行）+ `ArchCases` / `ArchRefusals`（22 条拒绝按名）/ `ArchFasProps` / `ArchProps` + `ReferenceArch` + golden 七对 + proto 7.6.0（`Protocol.hs` 一行、`Version.hs`、`corelink.rs`、VERSIONING 一条；十五份既有 golden 只动 proto 字面与 hello 能力表） | 精确 FAS 对两个参考逐权逐弧等价、贪心 cut 无环且极小、Louvain 确定性、`cabal test` 624 ok |
| 9 | 架构 B（落地 2026-09-30）：`cli/src/arch/`（tables / wire / face / console）+ `ce arch`（`main_arch.rs`，头部读 `main_prelude.rs`）+ MCP `architecture` + GUI 报告枢纽 arch 族（`hub_arch.js`；枢纽长出 `registerHub`、路径列表参数、自定义渲染器三个钩子）+ 冻结自仓读数 `contracts/eval/arch-self-v1.json`（钉 f2a7a2b4 的 `git archive` 树；在落地提交的干净树上重生成，是落地后的第二个提交）+ 册 19 | 三面字节同、冻结读数门 |
| 10 | 全量文档与事实（§10）+ 语言条实测记账（§11）+ 官网新页对部署（排在 v2.32 步 0–7 之后，随 [authority-track.md](authority-track.md) 步 8 一并做） | docs / site / facts 门全绿、引文重签 |
| 11 | 发版 1.9.0（在 v2.32 之后，即 [authority-track.md](authority-track.md) 步 10）：分数可比性声明（判决轴不动即与 1.8.0 可比；`flow` 若进 `ce check` 另声明）、基线具名重立、bench 入列 | RELEASE.md 链 |

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
12. 反馈弧集——**≤ 14 顶点的强连通分量按顶点子集 DP 精确、更大贪心并标 `exact`**（2026-09-30 修正：立项写的是 ≤ 12 条弧穷举；A8 的随机电池量到 ELS 贪心对最优最差 4×，精确范围改按顶点计的排序 DP〔2^14 × 14 状态、三元组并列规则有最优子结构〕，弧子集穷举退为参考；贪心无常数界，电池只记录最大观测比不断言）：报告面说清哪条是最优哪条是近似（备选「全贪心」）。
13. 官网卡片放哪——**新页对 `site/how/analysis/`**：how 页 732 行贴着 750 硬线，四张卡放不下；页面也不该为了塞卡片而删既有内容（备选「压缩既有卡片」）。
14. golden 重生器——**测试子仓的 `--ignored` 腿**：会话草稿目录里的脚本随会话消失（盘点所见），daemon golden 已有同形先例（备选「入库 python 脚本」引入第三种工具语言）。
15. 语言条低于 40 % 时——**上呈用户，不自行拉伸**：占比是副产品、不为占比写代码（§1 第 7 条）。
16. `flow/1` 考题的顺序形（2026-09-30）——**降表先于抽样，`ladder_first` 形**：题是从降出的四表里按类按层抽的（不问核），产品不先落地就没有池可抽；盲判的独立性靠题不带答案、盲窗内 `cli/src/flow` 零提交、精度册钉在回答它的代码上（§5.5），与 C / C++ 的先例同一把尺（备选「先写一个只为出题的第二套降表」是同一张表写两遍）。
17. 精度优先的三条读法（2026-09-30）——**条件上下文里的写降 readwrite、成员 / 下标 / 解引用写与方法调用记作对基变量的读、宏与默认实参里的名一律是读**：三者都只多加读、少报发现（§5.1 第 9 条）；备选（按字面记写）在 `x = 0; c && (x = 1); use(x)` 与 `list.append(a); a.b = 1` 上各出一个假死存储。
18. `dynamic` 的宽读法（2026-09-30）——**凡单元的行为读不出来的都置位**（求值器、体内预处理条件、计算 goto、`asm`、`setjmp`），核不改（位的语义「该单元不判」不变，`counts.dynamicUnits` 照记）；备选「加第二个位」要动 7.4.0 已发的契约形而判决不变。
19. A1 报告的十四条裁定（2026-09-30）——**文法读不出的结构按第 7 条置 `dynamic`**（计算 goto、GNU fallthrough 属性、TS `with`），不是新规则；**规则 1 / 4 里的语言名是例子不是范围**（带标签块的 goto / label 改写对 TS / Java / Rust、三段式 for 的 update 改写对 C / C++ / TS / Java / Go、表达式体 = 一条 `return` 对 TS 箭头 / Rust 闭包 / Java lambda / C++ lambda 各做）；**表达式位置的控制流不降**（§5.1 第 3 条末句）；**Python `with` 降 try + 空 catch**（`as` 目标的写与上下文表达式的读记在 try 结点自己的 seq 上；`__exit__` 可吞异常故后续可达）；**Go 具名返回值作形参行**（`declSeq` −1、位 0；裸 `return` 在自己的 seq 上读全部，`return a, b` 不读；未读的只出 `unused_param`，写了却在显式 return 前从未读照报死存储）；**C 体内原型跳过**（声明子链顶层直接是 `function_declarator`；`(*fp)(int)` 是局部量）；**Rust 模式裸名首字母大写 = 路径**；保守读法七条接受（Python `conditional_expression` 用 `*`、`del x` 读、函数内 `import` 不作局部量、`and` / `or` 同 kind 一律算、R `x$a <- v` 读基变量、Go 局部 `const` 不作变量、Go 复合字面量键按变量读——解析不到的名按第 9 条不入表）；**常真循环的数字字面按文本「不全为 0」**（Lua `while 0 do` 当有限循环，少报）；**Lua 空 if 的 then 位补一个带 `empty` 位的空块**；腿 4 放宽与腿 2 多核接受、R 的 `return(x)` 是 call 按第 3 条入 9；Java 样本只经 tree-sitter 零 ERROR 即够（本机无 JDK）。
20. A2 报告的十一条裁定（2026-09-30）——C 式 for 的 init 落在包住 init 与循环的作用域；表达式位置的循环把读过的名再读一遍（隐藏的回边用「再读」补，少报不多报）；未知语句带直接块子结点降成 kind 0 + 头部读（Java `synchronized`）；Rust if-let 绑定落分支作用域、Java `instanceof` 绑定落正在降的那条语句所在的作用域（elif 的绑定落包围整条 if 链的块——比 Java 的流敏感作用域宽 = 安全侧；A2 只设不还原 `stmt_scope` 的缺陷随 A3 修）；C switch 里不属于任何 case 的语句只挂读不建结点；Go type switch 的 `v := x.(type)` 在 switch 头声明一次；do-while / repeat 的条件在体作用域里挂在循环结点上（多出的路径只会少报）；后绑定的函数级名字跑两遍（Python / R / TS `var`，第二遍下标确定性相同，越界的 declSeq 按名拒兜底）；R 的 glue 字符串 `{name}` 算读；Python `with` / `except` 绑定沿用「首写即声明」；闭包内同名新声明仍让宿主被标 captured。
21. `flow/1` 接线的两条姿态（2026-09-30）——**单个超 `rowCap` 的单元按名记进 `refused`、不整份失败**（第 11 条总性：每个单元要么被判、要么按名拒、要么 dynamic）；**核对本侧已计价的批次答 degraded = 上限镜像漂移 = 错误**，与查询 / 架构族同姿态（备选「降级文档」把测量侧的缺陷读成核的状态）。
22. 提交 B 报告的六条裁定（2026-09-30）——**身份字段末位带 `name`**（同一行两个声明只差名字，不带名就撞）；**死存储层 A = 下一次访问是纯写**（readwrite 先读再写，不算）；**dynamic 单元不入候选池**（核对它整体不判，题按构造只能答 unjudged，问了等于没问；dynamic 位是降表侧的源码事实，抽样仍对判决盲）；阶梯常量 = `cli/src/flow/` 目录 + 具名例外 `mod.rs`；`Lang::extensions` 只读面让门读产品表不手抄；考题表写成一段文本经 `LazyLock` 解析（十行构造会互为克隆，查重门点名后改的形）。
23. 提及宇宙的「产品签名」规则（提交 B 收尾，2026-09-30，主会话按原则自答）——本仓的 flow 考题宇宙档按构造拼写每个单元的名字，让普查自仓行的 rust 未提及声明 424 → 90；这是 `tags` 那一类（把每个名字拼一遍的文件，提及不带信息），且是产品自己从语料生成的派生物、不是对语料的引用。规则按内容不按路径（产品里不写本仓的目录名）：JSON 顶层 `schema` 以 `ce.` 开头或顶层 `generated_from` 带 `ce` 键——产品给自己每个文档家族的两种签名——即离开 U，与二进制规则同位（`MENTION_REV` 4）；用户裁定 ③「生成树在 U 里」不动：它说的是可能引用名字的生成代码，签名文档只能引用自己。
24. 提交 B′ 的三条口径（2026-09-30，主会话按原则自答）——① 批次提示里的 nth 从 0 数（判官读法），样本与审阅档一律从 1 数、审阅档回显样本的 nth；② 审阅档的 `generated_from` 记生成时的树（2ea957d8 / dirty = true）——归档工具与档在同一提交里落地，与提交 B 的宇宙 / 样本档同一读法，顺序门（提交 C）读的是三档 commit 的祖先序：sample 5278e747 ≺ review 2ea957d8 ≺ precision；③ 判官句（auditor）是审阅档的散文出处，逐字进登记册，不另立字段。
25. 提交 C 的四态门与出处门三口径（2026-09-30，主会话裁定）——**门读四态**：`fail` = fp ≥ 1；`pass` = fp = 0 ∧ tp ≥ 1；`vacuous` = fp = tp = fn = 0（样本里没有正例可找，准入靠负例上的零误报——`per_kind` 记 `positives` = tp + fn 与 `negatives` = tn + fp 两个整数，登记册照抄「0 / n 个负例」；零行也归此态）；`silent` = fp = tp = 0 ∧ fn ≥ 1（有正例、一个都没报——家族在这里是瞎的，不是误报问题，但也不能准入，逐条列 fn 作回修降表的清单）；`judged` = 类 0 / 1 / 2 三门各 ∈ {pass, vacuous}，类 3 只记不判。理由：盲判的正例极少（类 0 不可达 2 / 323、类 1 死存储 10 / 393、类 2 未用局部量 45 / 226），八种语言的类 0、六种语言的类 1 与类 2 在样本里一个正例也没有——候选池本身就没有；原判据「fp = 0 ∧ tp ≥ 1 → pass；样本行数为 0 → vacuous；其余 fail」让这些语言仅因无正例可找而 fail，对「行数为 0」的空证据反倒判 vacuous——证据更多反判更差，是判据自身的不一致（原判据即被排除的备选）。**出处门三口径**：`ladder_first` 写成祖先或相等（降表的首个提交与抽样同为 5278e747：抽样在提交 A 的树上生成，门问的是「抽样时降表已在树上」，写成严格序按构造红）；盲窗 `touched_between(sample, review, …)` 只读 `LOWERING`（`cli/src/flow/` 去 `mod.rs`；提交 B 在两者之间改过 `cli/src/scan/lang.rs`，那是 `ANSWERED_BY` 的路径、不是盲窗的，读进盲窗按构造红）；`ANSWERED_BY` 那腿从精度册的提交起算到 HEAD，锁文件按钉版读。
26. 步 5 的四条读法（2026-09-30，落地时主会话按原则自答）——**面住在 `cli/src/flow_report/`、新颖度相减住在 `cli/src/guard/flow_novel.rs`**，不进 `cli/src/flow/`：那个目录回答精度册（出处门按路径读），面不是答案，放进去一改就让十份精度册退役；**GUI 是报告枢纽里自带渲染器的 flow 族**，不另立屏（§5.4 的「Flow 屏」就地改）：发现是一张平表加种类筛选，枢纽的钩子足够；**与任务书读法不同的三处**：① 不可达发现的 `lineEnd` = 该段末句的起始行（图例只存每句起点，跨行的末句不追到尾；随下一代降表补终点）；② 守卫走 daemon 的路一侧一问：核拒一个单元即该侧整体降级、行里具名，没有剔除重问——那是 `ce flow` 批处理路（§5.5 接线）的姿态，钩子把一次写入的代价钉在每侧一问；③ feed 的 `novel` 只数类 ≠ 3（未用形参带进来的发现只进 `kinds`）；**掩码分支的测试**：守卫出声 / 静默两侧都按 `flow::judged_mask()` 分支写，落地时的掩码下出声一侧由 Python 夹具跑到、静默一侧由 Rust 夹具跑到，掩码再变只挪腿不断腿（册 17 §10）。
27. 提交 E 的降表回修与退役（2026-09-30，主会话裁定 TS-2，其余按原则自答）——**TS-2 类型位置的 `typeof x` 是读**：类型别名从那个值的声明取类型，删掉值就编不过；C2 那条类 1 漏报（zod `partials.test.ts:152` 的 `requiredObject`）是真值争议，真值不改，本条同时是第二代考题批次提示的读法规则（由 F′ 落地）；**CPP-2 歧义按读**：`T x(a, b);` 在文法里恒是原型，形参类型名解析到域内变量即读，真原型的类型名解析不到、不受影响；**R-1 派发读全部形参**：代价是 stringr `type` 的 `error_call`（默认值形参，判官读作未读）由 tp 变 fn——类 3 顾问、只记不门；**TS-1 只在 TS 表开**（`forward_captures`）；**CPP-3** 两条类 1 漏报就是第 20 条登记的 do-while 少报（fmt `core.h:1311` 的 `prev` 在循环后的 `prev * 10ull` 被读，跳过循环体的那条多出的路径让初值读成活），**LUA-1** 是路径不敏感（该写已入表——题就是从它抽出的；luarocks `fetch.lua:248` 的 `errcode` 经「`cachefile` 真 → 第一个 `if not file` 不进 → 第二个 `if not file` 进」这条图上路径被读），两条都不改；**退役与第二代**：降表的文件在 `ANSWERED_BY` 里，十份第一代精度册按名退役、十门回 `audited`、判决掩码随门清空（`flow::judged_mask()` 的每一位 ⇔ 一份 judged 的精度册；第 26 条的守卫出声 / 静默两侧与 `flow_report::judged` 都按掩码分支，空掩码下所有发现只作顾问、守卫一律不出声，到 G 重生成精度册再填）；**图例终点行**（LEG-1）供面把类 0 的 `lineEnd` 改读末句终点，第 26 条 ① 那处读法随之可改，面的改动不在本提交；新降表下十语料重导的宇宙逐（单元，格）比池——cpp、r、rust（两个语料）、typescript 的池动了，这四门出第二代考题，python、tsx、go、c、java、lua 的池一格没动，保留第一代宇宙 / 样本 / 审阅档，在新降表的干净树上重生成精度册即可。
28. 提交 F 的第二代读法与代际规则（2026-09-30，主会话按原则自答）——**读法是定义**：批次提示第二代起多一节 `## Language readings`（TypeScript / TSX 三条、Rust 一条、C++ 一条、R 一条；原文在 `it/eval_flow_parts/prompt.rs`，登记册「第二代的读法」一段有中文复述），它与降表自提交 E 起的读法（第 27 条、§5.1 第 8–10 条）是同一份定义：产品按它降表，判官按它判；第一代没写明、判官凭常识判，C2 的读数（六个池未动的语言类 0–2 fp 0、fn 逐条有归因）说明那六门的第一代真值与定义相容，不重判；**代际**：池动了才出下一代（逐（单元，格）比池计数），题的身份是 `audit` 哈希不是秩，前一代的宇宙 / 样本 / 审阅档留在盘上作记录，门只读考题表第五列指向的代；**自证**：第二代`manifest.json` 带 `"readings": 2`，归档时抄进审阅档信封，`verify_review` 要第二代 = 2、第一代缺席或 1；读法一节只在第二代渲染，第一代批次逐字节不变；**出处**：九份档记 E 的树、dirty = true（子仓代际列与仪器未提交），同第 24 条读法。
29. 提交 D 的两提交与回放台账口径（2026-09-30，协调会话三裁 + 主会话裁定分期）——**两提交**：台账行带 `harness{ce, commit, dirty}`，读数取决于降表（产品），与精度册同类，照 813f4976 / e1a6b520 与 C1 / C2 的先例拆开：D1 = 仪器 + 执行者 + 文档骨架，D2 = 在 D1 的干净树上重量的十行冻结件、三张表与读数（`harness.commit` = D1、`dirty` = false）；执行者的台账阶段常量 `FLOW_LEDGER`（`Pending` → `Frozen`）让读冻结件的三腿在 D1 断言冻结件不在盘上（在则按路径点名），合成两语言史那一腿不分期；车道上先量的一份不入库，只作 D2 逐格对拍。**三条口径**：① 「单元被改」按记号读——单元行段内每个 tree-sitter 叶结点的源文本序列，注释子树跳过、空白不成叶，只改格式或注释不算修改（查重家族的记号流把标识符折成 ID、字面量折成 LIT，改名变量会读成没改，故不用）；② 每条发现在严格与窄两口径里各只结一次账（严格 = 单元第一次被修改时消失 = 真阳、留下 = 误拦；窄 = 留下且改动行与发现行段相交才记误拦，不相交不结账），两口径各记一套计数，各自封闭 `发现 = 真阳 + 误拦 + 移除 + 未定`；③ 率只算类 0 / 1 / 2（精度门读的三类），类 3 顾问单列、不进率。台账不是门：准入只读精度册（第 6 条）。D2 已落（2026-09-30）：十行在 D1′ 的干净树 5e02af4a 上 release 串行重量、`dirty` = false，与车道先量的一份十语言各 63 格逐格相同，阶段翻 `Frozen`，三张表与读数在 FPR-REPLAY.md；满载机器上的挂钟秒只作上界，车道安静窗口的读数并列作对照。
30. 提交 G 的十份精度册与掩码重填（2026-09-30，主会话按原则自答）——**读数**：十份精度册在 F′ 提交 5023273 的干净树上重生成（cpp / r / rust / typescript 读第二代审阅档，其余六门读第一代），类 0 / 1 / 2 十门 fp 皆 0——python vacuous / pass / pass、cpp 与 lua 三门 pass、java vacuous / vacuous / pass、其余六门三门 vacuous——`judged` 十门皆真，`flow::judged_mask()` 十位全填；漏报三条都是既有登记（cpp 类 1 两条 = 第 20 条 do-while 少报、lua 类 1 一条 = 第 27 条 LUA-1 路径不敏感），真值与降表都不改。**新登记 CPP-4**：顾问类 3 一条误报——fmt `chrono.h` 的 `format_tm` 形参 `cb` 经 `(w.*cb)(args...)` 被调用，C++ 文法把 `w.*cb` 读成 `field_expression`（operator `.*`、field 是 `field_identifier`，探针核过），降表只读 `field_expression` 的 argument 一侧、field 一侧从不解析成变量，`cb` 零读；类 3 只记不门，留给下一代降表（修它 = 新一代考题，照第 28 条）。**顺序**：G 是精度册的提交，自此到 1.9.0 发版 `cli/src/flow/`（`mod.rs` 除外）、`cli/src/scan/{functions,walk,lang}.rs` 与锁文件的钉版不得再动——出处门要 `touched_between(precision, HEAD, ANSWERED_BY)` 为空；D2 的回放台账行在 G 之后的干净树上量，与精度册同读一份降表。**归档**：六门只有一代的语言退役的「精度（第一代，提交 E 退役）」一节逐字节搬进 `EVAL-SET-FLOW-GEN1.md`、主册留一行指向，主册 558 → 658 行（硬线 750）。
31. M2 报告的九条裁定（2026-09-30，主会话）——**冻结建议集整份保留、五语料逐行**：子仓两条 CI 腿读外部语料的行（`the_sample_is_the_draw_of_the_frozen_set` 用全体 id 与 `feasible` 重算抽样、`readings` 读每条抽中行的 `feasible` / `reason` / `params`），`contracts/` 不进 crate 包、语言条不计 JSON，先例 `fpr-lang-v1.json`；**抽样对判决盲、对核的参数化不盲**：审阅者没有洞的原文判不了可行性，抽样行给 `param_texts`（可推翻），`feasible` / `reason` / `savings` / `kept` 不给，审阅表每行多答 `reason`，读数 `feasible_agree` / `reason_agree` / `params_agree`（后者 = 看过核的参数化后给的个数与核相同）；**片段组的两条不同构根因都修、不记限制**：merge 送的树不含 extra 结点（注释；`clone/1` 的判决树照旧带），片段按成员共同的顶层结点形修边（§6.1）；**核对本侧已计价的请求答 degraded = 上限镜像漂移 = 错误**，核起不来 / 不提供家族 / 答不了 = 降级文档（第 21 条同姿态）；**`nodeCap` 改名 `treeNodeCap`**（`CE.Graph.Cost.nodeCap` 已是 how 页芯片，docs_consts 按裸名全仓唯一）；**洞行加第六列 `postEnd`**（该侧森林末根，叶洞 = `post`、空侧 −1 −1），报告只留 `text`；**reason 1 叫 `position`**（语句位与其他位共用一个码，叫 `statement` 对其他位说谎）；**Haskell 有自己的位置类表**（`stmt_kinds` / `container_kinds` 只在无流表的语言里填，门守两向；Markdown / HTML 无函数可合并，计 `no_slot_table`）；**册 18 §7 只留设计边界**（T3 骨架行数是整数估计、md / html 无函数可合并、不问类型），可修的都修了。
32. M2c 的两条修正与窗口长度（2026-09-30，主会话）——**修边后的行段如实计价**：成员分两个行段，`lines` = 克隆族行段（身份，报告面照印 `path:a-b`）、`run` = 保留段（首个保留顶层结点的起始行到末个的结束行，整单元 `run` = `lines`），请求的 `lines` 列送 `run`，报告 / 冻结集 / 抽样的成员写 `{at, run}`；**省不下行的建议按名不可行**：`CE.Merge.Cost.reasonNoSavings = 5`，取法 = 洞的第一个不可行理由 > 参数多于 `paramCap` > 收益 ≤ 0 → 5 > 0（`MergeCases` 加 `noSavings`、`MergeProps.reasonsCohere` 钉「0 必有省行、5 必无省行」）；**窗口长度不设地板**：一个顶层结点可以是整条 `match` / `if`，片段的大小按保留段的行与结点计、不按顶层结点个数计，太短省不下行的自己按 5 出局（册 18 §7 一句）。修正后五语料冻结集重生成（本仓可行 170 → 51，119 条为 `no_savings`），对判决盲的抽样逐字节未动。
33. M2d 一个成员集只出一条建议（2026-09-30，主会话）——同一对整单元既是 `ce dedup` 的 T1/T2 族又是 T3 近似对时出两组（cobra 1597 组 / 1593 个互异成员集，requests 139 / 138，ripgrep 6387 / 6385，zod 22989 / 22985，本仓 0），两行的抽样 id 相同；裁定 = 按成员身份的有序元组去重、T1/T2 盖 T3（与 `score::one_row_per_pair` 的 0 > 1 留强同一立场），去重在本地预检之后，故 T1/T2 孪生送不出时 T3 那条留着；弃掉的计 `merged_duplicates`（报告 `counts` 加性），冻结集门加「每语料成员集互异」一腿，抽样 id 按构造唯一。
34. 落地时的位置类归类与表的住处（2026-09-30，主会话复核）——**slot 表住 `cli/src/merge/`**：八个文件原挂在 `cli/src/flow/slot*.rs`，而那个目录去掉 `mod.rs` 就是降表（`flow_provenance` 的 `ANSWERED_BY`），放在那里一落地就退役十份 flow 精度册；`git mv` 到 `cli/src/merge/`、`#[path]` 挂载删除、`use` 路径不变，`cli/src/flow` 与步 4 提交 E 逐字节同。**提交 E 的探针段带进来的三处归类**：Rust `raw_string_literal` = 表达式（与 `string_literal` 并列，Go 的表早已如此）、TypeScript `type_query`（类型位置的 `typeof x`）= 类型、C++ `field_initializer_list` / `field_initializer` = 其他（构造函数的成员初始化表是结构，实参各按自己的类，Rust 表里同名结点同读）。
35. CLI 面的公共导入一处（2026-09-30，主会话复核）——`main_flow.rs` / `main_query.rs` / `main_similar.rs` / `main_merge.rs`（步 9 的 `main_arch.rs` 同）五个头是同四行公共导入（`main_cmds::{fail, json, or_cwd}`、`main_judge::JudgeArgs`、`report::print_doc`、`std::process::ExitCode`），查重门把两个头读成一块；拆导入行、换记号序是躲门不是消重复，不收——新文件 `cli/src/main_prelude.rs` 承这四行 `pub(crate) use`（`main_judge.rs` 已 292 行，不往里塞），各头改一行 `use crate::main_prelude::*;` 加自己家族的一行。
36. 合并家族的审阅档与精度册同一提交（2026-09-30，主会话按原则自答）——flow 的精度册要在审阅档之后的干净树上另起一提交，是因为它记产品在生成那一刻的答案（降表一动读数就变，`generated_from` 钉住那棵树）；合并家族的精度册是三份冻结档（建议集、样本、审阅档）的纯函数，不读产品、没有 `generated_from`，在哪棵树上算都一样——为它单开一提交记不下任何可观测的事，`§12` 第 7 行「在审阅档提交之后的干净树上生成」据此改为「由三档按定义算出、与审阅档同一提交」。读数只入册不设门：合并建议是顾问（§6.4 的立场），`feasible_agree` / `reason_agree` / `params_agree` 三读数与分歧表供下一代参数化与理由排序回修；分歧逐条带判官的 note 入册，真值不改、核不改。
37. `arch/1` 测量侧的六条读法（2026-09-30，A8 / A9 落码时定）——**包边 F → D 只到 D 直属文件、ELS 先摘汇点再摘源点各取 id 最小、Louvain 的留守增益也按摘出后算、三条越界拒绝各具名、拒绝序 = 文件行形 → 目录行形 → 文件的目录范围 → edges → pkgEdges → focus、section 目标折到文件而 package 目标进 `pkgEdges`**：每条都是两种读法里更保守、更可核的那种（备选各条的另一读法都要多一个没人送的事实）。
38. A9 审阅裁定（2026-09-30，主会话）——**测量侧与三面整体接受**：`tables.rs::folded` 的段折叠按结点 `path`（段结点的 `path` 是文件路径、slug 另存）成立；包引用只在树里有该目录时入 `pkgEdges`（§7.3）；`wire.rs::consume` 按键核每张答表的行数（layers / metrics 每目录一行、cuts / clusters / misplaced / impact 对计数）、宽度与 id 范围，`face.rs::label` 的 zip 不会静默截断；多数目录的读法（最少 id 平局）与核对 misplaced 的读法一致，簇与它的错位行不会各说各的多数。**落地三件**：册 19 §6 不叫「Known limits」而是「设计边界」（与册 18 §7 同形；贪心路的 `exact` 0 是算法边界、包引用到达包目录直属的文件是 Go / Java 的包模型、簇是整树的函数而错位行是顾问、引用只来自阶梯解出的站点是图家族的定义——用户令「不要留已知限制」，四条都是定义句，没有可修的项）；`contracts/eval/arch-self-v1.json` 在落地提交的干净树上重生成、作第二个提交（车道档的 `generated_from` 是变基后不存在的提交且 dirty；amend 会换掉第一个提交的 sha，所以照第 30 条 G 对 F′ 的先例分两个提交）；proto 7.6.0 随落地（备选「车道上先升版」让车道与主干各持一个 7.6.0）。
39. 车道变基到合并车道后的三个克隆块（2026-09-30，A9b）——**一个所有者，不躲门**：`arch/wire.rs` 的能力常量与计数键对 `merge/wire.rs` 同形 → 计数键改成一张空格分隔的文本表经 `wire::count_keys()` 读（同文件 TABLES / IDS / DENSE 已是此形）；能力门对 `flow_report/face.rs` 同形 → 改经提升过的 `corelink::judged::ask`（同一句具名缺席「core offers no arch/1 (pre-7.6.0)」）；`faces.rs` 里 merge 与 arch 各拼一次 `report_json(&run(…)?)` → `arch::face::document` 持有一个根的 arch 文档、`faces::arch` 只返回它；子仓 MCP 腿对 `flow_face.rs` 同形 → `common::McpSession::relayed`；两根查重回到预算 50 / 91、未动，flow 与 merge 的文件一个没碰。
40. 上限常量名 `refCap`（2026-09-30，A9）——**`CE.Arch.Cost.edgeCap` 改名 `refCap`**：`docs_consts` 按裸名在全仓找 how 页芯片的唯一源常量，`CE.Graph.Cost` 已导出 `edgeCap`（图家族 06 号卡的芯片）；而这个上限本就数两张引用表（`edges` + `pkgEdges`），`ref` 是更准的名（备选「芯片加限定」让一个裸名在两处各有一义）。
41. `arch/1` 的三种具名降级与两种错误（2026-09-30，A9）——**降级**：核起不来或答不了、核不提供 `arch/1`（「pre-7.6.0」）、请求在本侧越过 `fileCap` / `refCap` 镜像——三者都出完整文档、`degraded` 具名、六表空、`ce arch` 退 2；**错误**：核对本侧已计价的请求答 degraded（上限镜像漂移）、应答表的行数 / 宽度 / id 范围与请求不符（偏斜应答）——两者不是降级，是本侧或核的缺陷，按名失败（与 `flow/1` 第 21 条、`merge/1` 同一姿态；备选「偏斜按降级读」把缺陷读成正常的不判）。
42. 合并家族第二代 R1：两列新事实（2026-10-01，主会话裁定；起因 = 第一代盲判的 23 条理由分歧逐条对源核实，`0116334e9e04` 的 `ch.value < max` 对 `ch.value > min` 被读成同形——匿名记号从不进树）——**每个树结点多 `own` 与 `text` 两个整数**（定义见 §6.2），组行多 `helper`；7.5.0 未发布，同一 minor 内加性，缺列 / 长度不等 / 负值按名拒。落码时自答两处：**`own` 读的是树自己的子结点遍历跨过的记号**——`tree.rs` 的 `kids` 一次返回命名子结点与匿名叶两半，`emit` 取前一半、`tree_text.rs` 取后一半，两处不可能读出两棵不同的树；**片段合成根的 `text` = 顶层结点记号流以 0x00 首尾相接**、`own` 0。`clone/1` 零改动：请求只送 `lab` / `lld`、t3 缓存键不变，旧 / 新二进制在本仓 + ripgrep + zod 上 `ce clone` / `ce check` / `ce dedup --check` 逐字节同。
43. R2 自己的记号差是洞（2026-10-01，主会话裁定）——key 相同的保留对（或 T1/T2 的同位内部结点）`own` 不同 → 该结点一个洞，类 4、取值 = 各侧 `own`；同构判定与映射都不读 `own`。运算符、关键字、标点的差按 `position` 不可行，不再读成同形。
44. R3 一级拓宽与空洞（2026-10-01，主会话裁定）——类 4 叶上的重标洞、父对两侧都在表达式位 → 洞覆盖父对整棵子树、类 1、父对里的其他洞被吸收；只拓一级、只拓叶、只拓类 4，类 2 不拓。落码时自答：**空洞（各成员文本向量全等）不删出骨架**——复原律要它，它只是不成参数、不定理由、不出洞行；拓宽后若成空洞同此。
45. R4 间隙一律结构差（2026-10-01，主会话裁定；`00fe2f72ad8f` / `0173d31a603e` 多一个实参）——一侧为空的间隙不再取在场侧的类：森林含语句位 → `spans_statements`，否则 `position`，不可行、不拓宽；两侧非空的间隙同此。落码时自答：**根对不在映射里的 T3 对**是整对一个间隙洞，同读此条。（一侧为空的间隙在表达式父对下的拓宽由第 52 条修订。）
46. R5 洞序 = 成员 0 的后序（2026-10-01，主会话裁定；成员 1 侧独有的间隙原先一律排最后，与批次提示的「源码序」不符）——洞键三元 `(anchor, rank, tie)`，定义见 §6.3；批次提示同批改成同一句后序读法（第 50 条）。
47. R6 片段骨架计辅助函数头尾（2026-10-01，主会话裁定；`0164bed5a39c`）——组行第三列 `helper`：片段组 = 该语言 slot 表的 `helper_lines`（Python 与 Haskell 1、其余十种 2，每张表都写、子仓单元腿逐表钉住），整单元组 0；`skeletonLines` 片段族 = 原公式 + `helper`（T3 的按比例估算照旧再加）。
48. R7 参数身份按文本（2026-10-01，主会话裁定；`0003206e2776` 的 identifier `dotall` 与 field_identifier `dotall` 曾算两个参数）——`Hole` 加文本向量，参数按文本向量去重（定义见 §6.3），`paramCap` 6 不变、`wire.rs::consume` 的 `max(param) + 1 == params` 校验不变；格 `(lab, leaf, 相对 lld)` 只留给复原律。
49. R8 位置类表三修（2026-10-01，主会话裁定；裸成员名在 TS / Rust / Go / C / C++ 落类 4 判 `position`、在 Python / Java / Lua / R 是 `identifier` 判表达式）——① 次序改为语句 → 声明名 → 类型 → 赋值目标 → 组成名与内文 → 表达式 → 其他；② `target_fields`；③ `part_fields` / `part_kinds`（字面量内文从 `expr_kinds` 移出）。落码时按钉版文法实探自答（`scripts/tsprobe`）：Lua 赋值目标是 `variable_list` 在字段 `name` 上的各项；Go `inc_statement` / `dec_statement`、Java `update_expression` 的操作数不带字段名，表里写空字段 = 任一子结点；Go `assignment_statement.left` 与 Python 的 `pattern_list` / `tuple_pattern` / `list_pattern` 是目标列表（`target_lists`，穿过它取各项）；R 的赋值是 `binary_operator`，按 `operator` 字段区分方向（`target_ops`：`<-` / `<<-` / `=` 取 `lhs`，`->` / `->>` 取 `rhs`）；Haskell 的组成名是 `projection.field`。十一张表的移位分类由子仓单元腿在探针源码上逐条钉住（目标根、裸目标、组成名、内文、类型自己的声明名）。
50. R9 读法即定义（2026-10-01，主会话裁定）——第二代批次提示在「How to judge」之后加一节 `## Reading rules`（五条，逐条对应 R2–R7，原文在 `it/eval_merge_batches/prompt.rs`），理由次序那句改成后序（「a place inside another first, siblings left to right, and a construct's own operator / keyword / punctuation after everything inside it」）；判官照读法判、核照规则判，是同一张定义表的两种拼写。落码时自答代际：**考题的文档按代**——`eval_merge_parts::GENERATIONS` 一代一行（建议集 / 样本 / 审阅档 / 精度册四份路径），门读最新一代，旧代四档留盘并照旧核对（审阅档仍是样本的审阅、精度册仍是三档读出的数）；抽样 id 的哈希域保持 `merge-sample-v1`（题的身份跨代可比，共有 / 新增 / 消失按 id 计），批次的哈希域随代 `merge-batches-v2`，清单带 `"readings": 2`。
51. `0003206e2776` 维持 2 个参数（2026-10-01，主会话裁定，不改码）——两侧差在被声明的方法名 `dotall` / `ucp` 与被调用的 `self.builder.dotall` / `self.builder.ucp`：裸方法名不是能传的值，拓宽到被调用表达式才是诚实的参数，所以声明名一个参数、被调用表达式一个参数，共 2 个；第一代判官的「1 个」靠的是声明名与方法名恰好同拼写的巧合（R7 按文本去重，两处文本不同）。任务书里「参数数减一」的验收期望写错，撤回。
52. R3b 一侧为空的间隙拓宽到表达式父对（2026-10-01，主会话裁定；`001e5f83c730` 的 `b""` 对 `b"\n"`、`r""` 对 `r"\n"`——空字节串没有内容结点，第 45 条把两处读成间隙结构差）——T3 对的保留对两侧都在表达式位（1），且它的子结点序里有一段间隙一侧有子树、另一侧没有 → 与 R3 同形一级拓宽：洞覆盖保留对整棵子树、类 1、参数文本 = 两侧父结点的整条记号流，拓宽后文本全同的成空洞，父对内的其他洞被吸收；两侧都有子树的间隙、或父对不全是表达式位（`foo(a, b)` 对 `foo(a)` 的间隙在实参表之下，类 4）仍按第 45 条是结构差。落在 `CE.Merge.Widen.bare` 与 `CE.Merge.Mapped.kept`；`MergeRulings` 三腿（拓上去的、拓不上去的、拓宽后同文本成空洞的）；`001e5f83c730` 读成可行、3 个参数（名 + 两个字面量），与第一代判官一致。批次提示的结构差一条加同一例外（第 50 条，读法即定义），冻结集 / 样本 / 批次按新核在第二代内重出。「间隙」取宽读法（主会话复核确认）：保留父对的子结点序里以保留子对切出的任一段，一侧为空、另一侧非空即是，不只限一侧完全没有子结点——`001a2c1754a1` 的对象字面量里一侧多出的元素因此拓宽到字面量，诚实的参数就是那个表达式。
