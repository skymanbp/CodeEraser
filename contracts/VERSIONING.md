# contracts/ — 契约版本化机制（M0 冻结机制，内容 M4 定稿 1.0.0）

> 依据 DEVELOPMENT_PLAN.md §7.1 与评审 B1：M0 只冻结**版本化机制**，
> IR/判决 schema 的**内容**在 M4 随真实需求定稿为 1.0（2026-08-11，
> wire 形状与 0.2.0 一致的声明性定稿）。**2.0.0**（M5-1c-iii，
> 2026-08-12）：rem/add 条目携第三元素 = trim 后 alnum 宽度，喂
> Cost.anchorFloor 的站点锚地板——请求形状破坏性变更，按 §2 升 major。

> **以下按版本倒序（最新在前），至 3.0.0 止；2.15.0–2.33.0 已于 2026-10-01 逐字节迁入 [VERSIONING-ARCHIVE-2.15-2.33.md](VERSIONING-ARCHIVE-2.15-2.33.md)（正册离 `ce scan` 的 750 行硬线只剩 5 行），2.1.0–2.14.0 的最初顺序段已于 2026-09-29 迁入 [VERSIONING-ARCHIVE-2.1-2.14.md](VERSIONING-ARCHIVE-2.1-2.14.md)，2.0.0 仍在上方导语段。**
> **7.10.0**（控制台文本，加性 minor，计划 v2.32 步 5，2026-10-01；设计册 `docs/reference/authority-track.md` §6）：
> `document.request` 可带 `lang`（0 en / 1 zh，缺省 0；其余按名拒 `document: lang is not 0 or 1`）；`document.result` 多两个键：
> `lines` = 该族在该语言下的控制台每一行 `[stream, text, ref…]`（stream 0 stdout / 1 stderr；数字与产品词已写进 text，每个引用在 text 里
> 留一个 `{}`、按序跟在后面，引用即文档里的类别化引用）与 `exit: {fail}`（该面的否决位；退出码 2 仍是测量侧自己的拒绝）；超 cap 时
> 两键按空请求答。`family` 多两个句子族 `guard`（PreToolUse 的拒绝理由，`say` 行一条规则一行）与 `audit`（Stop / precommit / commitmsg），
> `document` 为 `{}`、不进 `tables/1` 目录，`tablesDigest` 不变。arch / flow / merge / check / deadcode / graphscreen 的请求多几个只进 `lines` 的
> 测量侧事实（`widths` 表，`check` / `deny` / `only` / `roast` / `files` 事实，均可缺省、缺省读 0 / 空）与引用类 `clipped`，文档逐字节不变。既有判决族字节零变化
> （既有 golden 只动 proto 字面）；`contracts/fixtures/document/golden.ndjson` 加四十三对（每族一条 `lang` 1、守卫与审计每句两语、
> ROI 一位小数电池等）；电池 `DocumentProps5`。同一 minor 内（步 5A，未发布）`family` 再多十个带目录的报告族：`scan` / `dedup` /
> `clone` / `clone-units` / `docdup` / `erase` / `erase-trail` / `churn` / `trend` / `similar`，各带文档、`lines` 与 `exit`（scan 有 fail 条件、
> dedup / docdup / erase 在 `--check` 下越线、erase 轨迹有不可读行、trend 判 fail 或有失败点时为真，其余恒 false）；`tables/1` 的
> `document` 目录多十项（scan 另列规则名与 fail 条件名、dedup 的 `kgram` / `window`、erase 的类名 / reason 名 / kind / diff 上下文
> 行数、erase-trail 的日志路径与记录 schema、churn 的两个上限；每族另多 `pretty`——面按多行缩进印文档与否，scan / dedup 为
> true），`tablesDigest` 随之挪动；只进 `lines` 的请求事实与引用类见设计册
> §13 第 39 条。document golden 再加五十二对（每族非空 en / zh、有否决的族再一对否决例、空或降级请求、一条本族拒绝），原先问 `scan`
> 的未知族拒绝改问 `nosuch`；电池 `DocumentProps6` + `DocumentGen6`。读者（步 5 R0，wire 不动）：Rust 每个文档请求都带 `lang` 并绑定
> `lines`，`churn`、`scan`、`dedup`、`clone`、`clone-units`、`docdup`、`erase`、`erase-trail`、`trend`、`similar` 改印核的 `lines`、
> 退出码读 `exit.fail`、`--format json` 按目录的 `pretty` 印（scan / dedup 的 `--format sarif` 是绑定后文档的投影）——7.10.0 之前的核答不出
> `lines` 或 `pretty`，按名拒。
> **7.9.0**（文档族第二批，加性 minor，计划 v2.32 步 4，2026-10-01；设计册 `docs/reference/authority-track.md` §5）：
> `document/1` 的 `family` 多七个：`check` / `structure` / `join` / `deadcode` / `mentions` / `sites` / `graphscreen`（图屏一次装出画布与内嵌的
> deadcode 文档）；每族的 `ranges` / `rows` / `facts` 键、行宽与每列的宇宙仍按族陈述，契约照旧按名拒，另加各族自己的检查（一行至多的可空表、
> 码表范围、按槽一行、`structure` / `mentions` / `sites` 不收 `degraded` 下标——它们今天没有降级文档）。判决应答自己的降级原因
> （`graph_too_large` / `verdict_too_large`）由核按码直接写进文档的 `degraded`。`tables/1` 的 `document` 目录多七项，包另多 `store` 表
> （`site_kinds`：站点 kind 名按存储码，转录自 `cli/src/graph/store.rs` 的 `KINDS`；4B 起测量侧读这一表、`KINDS` 删除），`tablesDigest` 随之变。
> 目录项另带测量侧送码要用的名字表（4B）：`check` 带 `failed`（fail 条件名，`CE.Verdict.Faces.failConditions` 的次序再加 `degraded`）与 `reasons`，
> `join` / `deadcode` / `graphscreen` 带 `reasons`（`graph_too_large` / `verdict_too_large`）——判决应答给的是名字，测量侧按名字在包里的位置送码，
> 包里没有的名字按名拒，不留副本。七族三面都问 `document/1` 并绑定（4B），判决没发生、文档答不出同 7.8.0。既有判决族字节零变化（既有 golden 只动 proto 字面与 hello / 包的 digest）；
> `contracts/fixtures/document/golden.ndjson` 多十对（七族各一正例 + 三条拒绝）；电池 `DocumentProps4`。
> **7.8.0**（文档族，加性 minor，计划 v2.32 步 3，2026-10-01；设计册 `docs/reference/authority-track.md` §5）：第十八族 `document/1`——不是判决族：不判任何东西，按判决已答的整数把报告文档装出来。
> 请求 `document.request`：`family`（`arch` / `query` / `rules` / `flow` / `merge` 五选一）+ `ranges`（每个可被引用的宇宙的大小，按族定键，含 `why`）+ `rows`（判决已答的行原样回送 + 文档要而判决不要的整数：秩、行号、成员号……，按族定表名、行宽与每列所指的宇宙）+ `facts`（标量计数，按族定键）+ `degraded`（null 或 `why` 的下标）；三对象各须恰好是本族陈述的键。
> 缺键、多键、行宽不对、下标出宇宙、`degraded` 越界、`degraded` 旁带判决的行或事实，都按名拒（code `contract`，`<表> <i>: <原因>` / `document: unknown family <名>` / `degraded: with rows <表>`），再加各族自己的检查（按槽一行的表、码表范围；flow 的 `shown` 行 −1 = `--kind` 给了目录里没有的名字，拒为 `shown <i>: unknown kind; the catalogue lists …`）。
> 应答 `document.result`：`document` = 与该族报告 JSON 同形的文档，被度量仓库的字符串位一律是引用 `{"$": [类别, 整数…]}`（类别按族陈述，如 `path` / `dir` / `unit` / `why`）、产品常量（schema id、kind 名、原因名）直接出字符串；`counts{rows}`；`degraded:false`。行总数 > `docRowCap` 1,048,576 → `degraded:true, reason:"document_too_large"`、`document` 为该族的空文档。
> 定义包（`tables/1`）加顶层键 `document`：每族 `{schema, empty}`（`flow` 另带 `kinds`〔每行 `[名, advisory]`〕与 `judged`），`tablesDigest` 随之变。五族三面都问 `document/1` 并绑定；判决没发生照样问（带 `degraded`），文档答不出 = 具名拒绝、退 2（设计册 §13 第 20 条）。既有十六判决族字节零变化（既有 golden 只动 proto 字面与 hello 能力表 / `tablesDigest`）；新增 `contracts/fixtures/document/golden.ndjson`；电池 `DocumentProps`。
> **7.7.0**（定义包族，加性 minor，计划 v2.32 步 1，2026-10-01；设计册 `docs/reference/authority-track.md` §4）：
> 第十七族 `tables/1`——不是判决族：不读仓库事实、不判任何东西，答的是核判决所用的全部语言与产品定义。请求 `tables.request`
> 只有信封三键（`type` / `id` / `proto`），多出任何键按名拒（code `contract`，「tables: unexpected key <k>」，只点名第一个）；
> 应答 `tables.result`：信封 + `digest`（包的规范字节——aeson 有序键、无空白——的 fnv1a64，JSON 整数）+ 十六个顶层键：
> `languages`（`rows` 每码一行 `{code,name,exts,scan_only,prose_only,judged,document,flow_judged}`、码 0..21；`machine_txt`；
> `mention_whole_run_exts`）、按语言报告名分的 `scan` / `flow` / `slot` / `sites`（无表的语言 `flow` / `slot` 为 `null`）、
> `calls`（`calls` / `protected` 按语言分 + `r_formals`）、`fourclass`、`ladder`（`hs.boot` / `java.packages`·`lang` /
> `ts.builtins`·`prefix_only` / `go.std` / `py.stdlib` / `lua.stdlib` / `rs.builtin`）、`walk`（`secret_globs` / `builtin_excludes`）、
> `outputs`、`docdup`、`keys`、`flags`、`tombstone`、`compdb`、`protocol`；表下的键 = 测量侧字段名或常量名的 snake_case，
> 共用片（TS / TSX、C / C++）只出拼好的每语言最终表。编码：元组为数组、`Option` 为 `null`、`NameStyle` 为 `"snake"` /
> `"mixed_caps"` / `"any"`；`Specifier` 为相邻标签对象 `{"form": <snake_case 变体名>, "arg": <载荷>}`，无载荷的变体无 `arg`
> （`first_named` 的载荷 `{"star": bool}`、`spanned` 的载荷 `{"from","to"}`）；空白分隔的名表出为字符串数组，行表出为
> `[头, [词…]]`。hello 应答加性 `tablesDigest`（同一个数），能力表加 `tables/1`。整数过线（§5.9.2，保护的是仓库数据）不受影响：
> 包里每个值都是产品常量，从核流向测量侧。既有十六族字节零变化（既有 golden 只动 proto 字面与 hello 能力表 / `tablesDigest`）；
> 新增 `contracts/fixtures/tables/golden.ndjson` 两对（全包 / 多一个键被拒）；电池 `LangProps`（码 0..21、扩展名唯一、判决掩码
> 0x17847F、flow 判决掩码 1540127 且蕴含判决、nest-only 种类不是单元种类、slot 种类集两两不交、flow 名表、C / C++ 只在 noreturn 上分、`digest` 是数据的纯函数、
> 族答全包且与 hello 同数）。步 1 时测量侧还不读包（子仓 `it/tables_equivalence.rs` 证核的转录与 Rust 原文逐键相等，步 2 随原文一起退役）。
> **步 2：测量侧改读**（同一未发布 minor，2026-10-01；设计册 §4.5）：既有十六族的请求、应答与 golden 一个字节不动；`tables.result` 的语言行加性一列 `flow_judged`——flow 家族的判决语言集（盲评精度考题过门的十门：python、typescript、tsx、rust、go、c、cpp、java、lua、r，掩码 1540127，蕴含 `judged`），原是 `cli/src/flow/mod.rs` 的 `JUDGED`，tables golden 与 hello-ok 的 `tablesDigest` 随之重答。`ce` 的每张定义表
> 改从 `tables.result` 读（`cli/src/tables/`），Rust 侧的表文本删除；包按「进程内存 → 缓存文件 `<根>/.ce/tables-<ce 版本>-<proto>.json`
> → 核」三级取，缓存以答包那份核二进制的 `{path, len, mtime_ns}` 加 `ce` / `proto` 为身份、以 Rust 自己对包字节算的 fnv1a64 验完整，
> 任一不符即重取、永不因旧缓存拒绝。三条具名拒绝（退 2）：无核（「core unavailable: … to answer tables/1」）、核无 `tables/1` 能力
> （「pre-7.7.0 core …: no tables/1」）、包缺键（「tables/1 from core <版本> (…) lacks a table: missing field `<键>`」）；此后每条核链的
> hello 若点名的 `tablesDigest` 不等于本次读到的包的 `digest`，按名拒（「core … names tables digest <n>, but this run read <m> from <缓存>」）。
> 请求里的 `judgedMask` 照发，值改由包的语言行算出。
> **7.6.0**（架构分析族，加性 minor，计划 v2.31 步 8 / 9，2026-09-30；ADR-008 细则第七期，设计册
> `docs/reference/analysis-track.md` §7）：第十六判决族 `arch/1`——请求 `arch.request`：`files=[[F,D,lines]]`
> （F = 行号自 0 连续，D = 所在目录，`lines` ≥ 0）+ `dirs=[[D,parent]]`（D = 行号；第 0 行是根、父 −1，其余父是更早的行，
> 父关系按构造是树）+ `edges=[[F,G,w]]`（文件 → 文件引用，按 (F,G) 严格升序、F ≠ G、w ≥ 1）+ `pkgEdges=[[F,D,w]]`
> （文件 → 目录的包粒度引用，按 (F,D) 严格升序、w ≥ 1）+ `focus=[F]`（`--impact` 点名的文件，严格升序）；五表缺席读作空；
> 应答 `arch.result`：`layers=[[D,level]]`（每目录一行：去掉 cuts 后无出弧为 0、否则 1 + 出弧目标的最大层）+
> `cuts=[[D,E,w,exact]]`（目录图的反馈弧集，按目录对出：强连通分量 ≤ 14 顶点按顶点子集 DP 求最小、`exact` 1；
> 更大按 Eades–Lin–Smyth 贪心并逐条试放回、`exact` 0 = 极小而未证最小）+ `clusters=[[F,c]]`（文件图上的确定性 Louvain，
> 簇号按簇内最小文件号重编）+ `misplaced=[[F,M]]`（簇的多数目录 M 在簇内的文件数严格多于 F 自己的目录）+
> `impact=[[F,depth]]`（自 `focus` 沿反向文件边 BFS，`pkgEdges` 读作引用该目录直属的每个文件，focus 自身 depth 0）+
> `metrics=[[D,fanIn,fanOut,instability]]`（互异目录数，去 cuts 前；instability = ⌊1000 · out ÷ (in + out)⌋，in + out = 0 记 −1）
> + `counts{files,dirs,edges,pkgEdges,focus,cuts,clusters,misplaced,impact}`；文件数 > `fileCap` 131,072 或
> `edges` + `pkgEdges` 合计 > `refCap` 524,288 → 完整降级应答 `degraded:true, reason:"arch_too_large"`（六表空、五张请求表仍计数）。
> 顾问族：无旋钮、无 fail 档、无条件位。契约拒绝 22 条按名（`ArchRefusals` 逐条钉：文件行形 → 目录行形 → 文件的目录范围 →
> `edges` → `pkgEdges` → `focus`）；电池 `ArchCases` / `ArchFasProps` / `ArchProps`（精确 FAS 对弧子集穷举与全排列两个参考逐权逐弧等价、
> 贪心 cut 无环且极小、对最小值的比只记录不断言、分层对 cuts、Louvain 不劣于全单点划分、影响面对不动点闭包、错位读法、两道上限、空请求、
> 计数九键）。既有十五族字节零变化（既有 golden 只动 proto 字面与 hello 能力表）；新增 `contracts/fixtures/arch/golden.ndjson`
> 七对（两目录环精确拆一条 / 包引用折到目录 / 五目录稠密环 / focus 影响面与错位 / 空目录 / 第二个根拒绝 / 十五目录成环的贪心 `exact` 0）。
> 旧核无此能力 = 测量侧具名降级：`ce arch` 的文档带 `degraded`「core offers no arch/1」，`judged::ask` 点名「pre-7.6.0」。
> **7.5.0**（克隆合并建议族，加性 minor，计划 v2.31 步 6 / 7，2026-09-30；ADR-008 细则第七期，设计册
> `docs/reference/analysis-track.md` §6）：第十五判决族 `merge/1`——请求 `merge.request`：`groups=[[g,family,helper]]`
> （按 g 严格升序；family 0 = T1/T2 组〔≥ 2 个成员，成员树须同构〕/ 1 = T3 对〔恰 2 个成员，按树编辑映射对齐〕；`helper` = 片段组合并后
> 辅助函数的头尾行数〔Python / Haskell 1、其余 2〕、整单元组 0，计入骨架行数）+
> `members=[[g,m,unit,lines,fileIndeg]]`（按 (g,m) 严格升序、m 每组自 0 连续；`unit` = 请求内成员号〔核只回显〕、`lines` =
> 整单元的行数或片段保留段的行数、`fileIndeg` = 成员文件在引用图上的入边数）+ `trees=[{lab,lld,leaf,slot,own,text}]`（按成员序每成员一棵；
> `lab` / `lld` = clone/1 的后序编码并共用它的形状契约，`leaf` = 叶结点源文本的 fnv1a64〔内部结点 0〕，`slot` = 位置类
> 0 语句 / 1 表达式 / 2 类型 / 3 声明名或局部赋值的裸目标 / 4 其他，`own` = 结点自己的匿名记号〔运算符 / 关键字 / 标点〕以 0x00 相隔的
> fnv1a64〔没有 = 0〕，`text` = 子树整条记号流的 fnv1a64；名字、路径、源文本都不过线）；应答 `merge.result`：
> `suggestions=[[g,params,kept,savings,feasible,reason]]`（每组一行按请求序；`kept` = 文件入度最大的成员、并列取最小 m；
> `savings` = 成员行数之和 −（骨架行数 + 成员数 × `callLines` 1）；reason 取洞序里第一个不可行洞：0 可行 / 1 洞在语句或其他位置 /
> 2 洞在类型位置 / 3 缺口洞的森林含语句〔先于 1、2〕/ 4 洞全可行但参数多于 `paramCap` 6 / 5 `no_savings`〔洞与参数都过、省不下一行〕；
> 自己的记号不同的结点是洞〔类其他〕，类其他的叶洞在两侧都是表达式的父结点下拓宽为父结点整棵子树〔类表达式〕，T3 对里一侧为空的间隙在两侧都是表达式的保留父对下同样拓宽，其余间隙类其他，
> 洞序 = 成员 0 的后序，参数按各成员的文本向量去重，各成员文本全同的洞不成参数也不出洞行）
> + `holes=[[g,hole,param,m,post,postEnd]]`（每洞每成员一行，`post` .. `postEnd` = 该成员此洞的首末根，叶洞同一结点两次、空侧 −1 −1）
> + `counts{groups,members,nodes,suggestions,holes,feasible}`；组数 > `groupCap` 4,096 或树结点合计 > `treeNodeCap` 131,072〔两道上限
> 下最宽的请求仍在协议的 32 MiB 行内〕→
> 完整降级应答 `degraded:true, reason:"merge_too_large"`（空表）。同批 clone/1 加性：树可带 `leaf` 列（`CE.Clone.WireTree` 的可选字段，
> `ted` 永不读；长度不符按名拒 `leaf length mismatch`），测量侧的 clone/1 请求仍只发 `lab` / `lld`，判决字节不动。顾问族：无旋钮、
> 无 fail 档、无条件位。契约拒绝 29 条按名（`MergeRefusals` 逐条钉：组行 / 成员行形状、成员对组、组的成员数、树数、每棵树的
> clone/1 形状契约与四列〔缺列 / 长度不等 / 位置类越界 / `own`、`text` 为负〕、T1/T2 组不同构）；电池 `MergeCases` / `MergeRulings`
> 〔合并家族第二代九条裁定各一例与反向探针〕/ `MergeProps`（反合一两律〔每个成员由骨架填入自己的值还原、没有参数可删〕与文本律〔两个
> 非空洞同一参数当且仅当文本向量相同〕
> 200 个生成的 T1/T2 组、`tedMapping` 与 `ted` 同距离且是合法 Tai 映射、洞的首末根、理由一致性〔可行恰为 0、0 必有省行、5 必无省行〕、
> 等缺口折叠、两道上限与两道上限下最宽的请求在行内、空请求）。既有十四族字节零变化（既有 golden 只动 proto 字面与 hello 能力表）；新增
> `contracts/fixtures/merge/golden.ndjson` 七对（无洞可行 / 三成员叶洞一个参数 / T3 缺口洞 / 语句位置洞 reason 1 / 七个参数 reason 4 /
> 不同构拒绝 / T3 一侧为空的间隙在表达式父对下拓宽为一个可行参数〔R3b〕）。旧核无此能力 = 测量侧具名降级：`ce merge` 的文档带 `degraded`「core offers no merge/1」，`judged::ask` 点名「pre-7.5.0」。
> **7.4.0**（函数内死代码族，加性 minor，计划 v2.31 步 3，2026-09-29；ADR-008 细则第七期，设计册
> `docs/reference/analysis-track.md` §5）：第十四判决族 `flow/1`——请求 `flow.request`：`units=[[u,lang,params]]`
> （按 u 严格升序；`params` = 该单元 var 表里形参的个数，不符按名拒 `params disagree with the var table`）+
> `stmts=[[u,seq,parent,kind,flags,aux]]`（每单元前序编号：同单元 seq 自 0 连续、父在前〔−1 = 单元体〕、单元按序出现；
> kind 0 block / 1 stmt / 2 if / 3 loop / 4 switch / 5 case / 6 try / 7 catch / 8 finally / 9 return / 10 throw / 11 break /
> 12 continue / 13 goto / 14 label / 15 noreturn-call；flags 五位 0 has_else〔if 的第二个子结点是 else、switch 有 default〕/
> 1 infinite / 2 fallthrough / 3 dynamic / 4 empty；aux 只在 break / continue / goto 上 = 目标 seq〔break 要包围它的 loop 或
> switch、continue 要包围它的 loop、goto 要本单元的 label〕、其余为 0；树形按名拒：if 的子结点数 = 1 + has_else、switch 的子结点
> 全是 case、case 只在 switch 下、catch / finally 只在 try 下且顺序 = 体 · catch* · finally?、stmt 与出口与跳转无子结点）+
> `vars=[[u,v,declSeq,flags]]`（v 每单元自 0 连续；declSeq −1 ⇔ 形参〔flags 位 0〕，否则本单元某语句；位 1 captured / 2 ignored /
> 3 address_taken）+ `uses=[[u,seq,v,mode]]`（按 (u,seq) 不降、同语句内按求值序可重复；mode 0 read / 1 write / 2 readwrite）；
> 应答 `flow.result`：`findings=[[u,kind,seq,v,seqEnd]]`（0 unreachable 极大连续段〔v −1〕/ 1 dead_store / 2 unused_local
> 〔seq = declSeq〕/ 3 unused_param〔seq −1，顾问〕；整表按元组升序）、`counts{units,stmts,vars,uses,findings,dynamicUnits}`；
> 四表合计 > 524,288 行 → 完整降级应答 `degraded:true, reason:"flow_too_large"`（空表）。判决 = 结构化控制流建图
> （`CE.Flow.Cfg`：try 体内每条语句都可能跳到本 try 的每个 catch、离开 try 的每条边先经 finally〔finally 的汇合点接所有待续目标的
> 并集——只加路径不减〕、return / throw / noreturn 经 finally 到出口、break / continue / goto 经 finally 到目标）→ 可达性
> （`CE.Flow.Reach`）→ 反向活性（`CE.Flow.Live`：同语句内按求值序倒走，同一语句对同一变量的多次写各自判、任一为死即报该行，写后无路径读到即死存储；captured / address_taken
> 不判死存储，ignored / captured / address_taken 不判未用；无任何读的变量只报未用不报死存储；不可达语句的写不重复报）；任一语句
> 带 dynamic 位的单元整体不判并计 `dynamicUnits`。无旋钮、无 fail 档、无条件位（`ce flow --check` 与守卫腿按 `[flow] tier`
> 读发现，步 5）。既有十三族字节零变化（既有 golden 只动 proto 字面与 hello 能力表）；新增 `contracts/fixtures/flow/golden.ndjson`
> 六对（不可达段 / 死存储 / 未用局部量 / 未用形参 / 经 finally 的 return〔不可达 + 死存储〕/ 树形拒绝一例；降级面由电池以
> 524,289 行的运行时请求探——夹具不装 7 MB）；契约拒绝 42 条按名（`FlowRefusals` 逐条钉）；随机结构化程序 200 例与不依赖控制流图的
> 轨迹参考（`ReferenceFlow`：树遍历枚举每条执行轨迹、每个循环头至多两次、try 体内每条语句都可交给 catch）逐条同。旧核无此能力 =
> 测量侧具名降级「core offers no flow/1 (pre-7.4.0)」（步 4 提交 A 已接线 2026-09-30：`cli/src/flow/wire.rs` 四表请求与严格 consume、`wire_batch.rs` 按 `rowCap` 分批与拒绝驱动的剔除；golden 六对的请求行改由真源码降出，应答只动 seq / v / 计数）。
> **7.3.0**（代码查询与架构规则族，加性 minor，计划 v2.31 步 1，2026-09-29；ADR-008 细则第七期，设计册
> `docs/reference/analysis-track.md` §4）：第十三判决族 `query/1`——请求 `query.request`：`program=[[kind,value]…]`
> 记号流（0 谓词码：事实谓词 0..26 / 程序谓词 ≥ 1000 按首现编号；1 变量按子句编号；2 整数；3 集合号；4 名字哈希
> 〔枚举常量同走这一路〕；6 匿名；10..33 标点与关键字——5 空着不用，其余 kind 按记号点名拒 `unknown token kind`；
> 非整数 kind 的负值拒 `negative token value`；不在 schema 且 < 1000 的谓词码拒 `unknown predicate code`）+
> `facts={"<code>":[[…]…]}`（只含程序引用到的事实表，键 = 谓词码十进制——非 schema 码或 `007` / `-1` 一类拼写拒
> `unknown fact predicate`；每行恰为该谓词的元数、值非负、整表按元组严格升序，各按 `facts <code> <i>` 点名）+
> `prelude`（前奏子句数，负值拒）+ `why` / `schema` 两个布尔；应答 `query.result`：`goals=[[goal,kind,sorts…]]`
> （0 查询 / 1 断言；列类别 0 node / 1 dir / 2 unit / 3 int / 4 sym / 5 set / −1 无约束）、`answers=[[goal,args…]]`
> （goal 序、元组升序）、`preds=[[code,sorts…]]`（每个程序谓词各位置推导出的类别，按码升序；有错误或空程序时为空——
> 步 2 加进 7.3.0 未发布的应答形，回标推导链结点的实参用）、`proof=[[goal,answer,node,parent,rule,pred,args…]]`（前序编号、根 parent −1、`rule` = 子句
> 下标或 −1 = 发送的事实、查询根 pred −1；`?-` 只在 `why` 时展开、断言违规恒展开；整棵装不下 `proofCap` 16384 就整棵
> 不出并计 `counts.proofTruncated`）、`errors=[[token,code]]`（1 语法 / 2 未知谓词 / 3 元数 / 4 类别 / 5 未绑定 /
> 6 不可分层 / 7 前奏重定义 / 8 聚合形 / 9 头部匿名；分阶段，第一个出错阶段报它的全部；有错即不求值）、
> `counts{rules,queries,asserts,strata,facts,derived,answers,violations,proofNodes,proofTruncated}`、`schema=[[code,arity,sorts…]]`
> 只在请求要时回显；记号 > 65536 或事实行 > 4,194,304 → 完整降级应答 `degraded:true, reason:"query_too_large"`
> （空表），求值中派生元组 > 2,097,152 → 同一降级应答且 `counts.derived` 给到达上限时的值。语义 = 分层否定与聚合
> （层规则 + SCC 负边检查）、逐层半朴素求值、每层惰性按掩码建索引、首条推导即出处；`Var = expr` 在 `Var` 已绑定时是比较、
> 比较两侧同类别（同类 id 可比大小）、算式只在整数上、除零使文字不成立、空 `count` / `sum` = 0、空 `min` / `max`
> 不成立（`CE.Query` 与 `CE.Query.{Contract,Cost,Schema,Syntax,Parse,Check,Check.Sorts,Check.Safety,Eval,Eval.Index,Eval.Join,Proof}`）。
> 无旋钮、无 fail 档、无条件位：`assert` 的违规数由面（`ce rules`）读成门。既有十二族字节零变化（152 对 golden 只动
> proto 字面与 hello 能力表）；新增 `contracts/fixtures/query/golden.ndjson` 八对（步 1 六对：前奏 + schema 回显 / 断言违规 /
> 带 `why` 的查询 / 语法 / 未绑定 / 不可分层；步 2 加算术 / 集合上的聚合两对）。旧核无此能力 = 测量侧具名降级「core offers no query/1 (pre-7.3.0)」
> （步 2 已接线：Rust `cli/src/query/wire.rs` 经 `corelink::judged::ask` 的能力门，缺席 = 文档里的具名未判决）。
> **7.2.0**（判决语言集上线，加性 minor，计划 v2.30 步 1，2026-09-24）：`scan.request` 与 `graph.request`
> 各加一个可选整数键 `judgedMask`——Rust 侧 `Lang::judged_mask()` 从 LANGS 表按 `scan_only` 列推出的位集
> （落码时 `0x7F` = 码 0..6；各语言随自己的步翻位，今日十三位 `0x17847F` = 码 0..6、10、15..18、20——纯散文臂 21 与哨兵 7 不在集内），ce 恒发。核此前把「lang 在判决集内」写死为 `lang ≤ 6`（`CE.Scan.Contract.namingShape` 与
> `CE.Graph.Contract.unresRow` 各一份常量）；自本版起两处同读 `CE.Wire.judgedLang`：`naming` 行与 `unres` 行的
> lang 位在 mask 内即合法，否则仍按行点名 `lang outside the judged set`。缺席 = `CE.Wire.legacyJudged` 127，
> 行为与 7.1.0 逐字节同；负值 / ≥ 2^63 按名拒绝（`judgedMask: negative` / `judgedMask: outside i64`），位序
> ≥ 63 的 lang 永不在集内。回显：`scan.result` 在 mask 上过线且未降级时携同值 `judgedMask`，`graph.result`
> 在上过线时恒携——Rust 两侧据此钉漂移（无回显 = 7.2.0 之前的核，按名拒绝）。判决字节零变化：既有 130 对
> golden 只有 proto 字面动；新增 scan 三对（16 mask 内 lang 15 判并回显 / 17 缺席拒 / 18 负值拒）与 graph
> 两对（25 mask 内 lang 20 判并回显 / 26 缺席拒）。此键是语言扩展（设计册 `docs/reference/language-expansion.md`）
> 的接线：新语言在各自的步翻 `scan_only` 位即入集，核不必再改。步 2（C / C++，同一未发布 minor 内加性）：核
> `CE.Graph.Cost.roleBits` 加第九行 `(8, 1)`——编译单元角色（Rust 侧 `ROLE_UNIT = 1 << 8`，`.c/.cc/.cpp/.cxx`，
> 设计册 D18）落在可执行位，请求不带该位时字节同前；步 5（HTML，同一未发布 minor 内加性）再加第十行 `(9, 4)`——资产角色（Rust 侧 `ROLE_ASSET = 1 << 9`：走查读到而索引不持有的文件——页面的 `src` / `href` / `link` 目标，样式表、脚本、图片；`nodes.rs` 按构造标出、`node_row` 单独发送）落在 dyn-referenced 位 4，此前该位没有生产者：样式表的 `url()`、脚本的取回、manifest 的图标是图看不见的引用，资产结点永不成候选，请求不带该位时字节同前；Rust 侧另有三件存储与配置事实、皆不过线：`store::KINDS`
> 追加 `include` 站点标签（GRAPH_REV 15 → 16）、clangd 的三个探名 `compile_commands.json` / `build/compile_commands.json` / `compile_flags.txt` 入解析器配置、`[graph.search_roots]`
> 入 resolve_key。步 5b-8（纯散文臂 `.txt`，同一未发布 minor 内，wire 零改动）：`verdict/1` 轴 2 的代码文件数改读「有位置行且不在
> `docFiles` 集内的文件」（`Score.hs` 的 `codeFiles`）——纯文本文件是 docdup 机会而不是结点，`docFiles` 自此可以点名一个没有位置行的
> 文件，7.0.0 条的 `nodes − docFiles` 是历史读法。步 7b（判决回迁，同一未发布 minor 内加性；v2.30 修正案 2026-09-24
> 用户裁「搬前两处，随 1.8.0」，2026-09-28）：① `erase.request` 加性可选表 `targets=[[pathId,start,end]]`——与 `rows`
> 一一对齐（行数不等按名拒 `targets: N rows for M fact rows`）；pathId 按路径首现稠密编号（名不过线）、`0/0` = 整文件、
> 否则 1 基闭区间；每类只能点名一种形（verbatim_doc 须为区间、t1_twin / dead_file 须为整文件）、半开与倒序按名拒、整表按键序
> （`[pathId,start,end]` 字典序不降，否则 `target i: out of key order`）；上过线且未降级时 `erase.result` 多一键 `kept`
> （与 `rows` 同长的 0/1 表）= 目标闭包的答案（`CE.Erase.Cost.keptRows`：可擦的整文件行拥有其路径、同路径区间行出局；
> 同一目标里可擦行按 `licence` 择富〔t1_twin 2 > dead_file 1 > verbatim_doc 0〕；无可擦行时按 `advisoryFirst`
> 〔dead_file > t1_twin〕；同类并列取最早行——键序因此是契约的一部分）；此前这套闭包住在 Rust `erase/mod.rs::close_targets`。
> 缺席 = 7.1.0 字节同、无 `kept`；ce 恒发（候选先按 path / span / 类名排序，故键序即渲染序），无 `kept` 的应答按名拒
> （「7.2.0 之前的核」）。② `structure.request` 加性可选表 `patternShapes=[[dirId,bits,count]]`（bits 0..127 = 词干的七位
> 事实：0 下划线 / 1 连字符 / 2 小写字母 / 3 大写字母 / 4 数字打头 / 5 首字母大写 / 6 不可分类〔空词干或非字母数字连字符下划线〕；
> `count ≥ 1`、按 `(dirId,bits)` 严格升序、越界按名拒 `shape bits outside 0..127`），核按 `CE.Structure.Shape` 的 STYLE 表
> 折成 `[dirId,code,count]` 分布再判 S1——此前分类住在 Rust `structure/tree.rs::pattern_code`；与 `patterns` 同上线按名拒
> （`patternShapes: rides beside patterns (one road)`）；上过线且未降级时 `structure.result` 回显 `patternShapes=<行数>`，
> ce 恒发形状表、不再发 `patterns`，无回显按名拒。golden：既有对不动，新增 erase 六对（9–11 闭包三例〔孪生胜死文件且同路径
> 区间出局 / 两行皆顾问时死文件的类别拒绝立 / 可擦孪生胜顾问死文件〕/ 12 行数不等 / 13 整文件类点名区间 / 14 键序）与 structure
> 三对（18 形状路与 1 号对同判并回显 3 / 19 双路拒 / 20 位越界拒）。③（2026-09-29）`scan.request` 加性表
> `events=[[row,seq,parent,pos,flags,aux,op…]…]`——每个单元的**结构事件流**：测量侧只按 LangSpec 表把单元里每个入类
> 结点前序列出（嵌套结构 / 扁平子句 / 只抬层的 lambda / 带标签跳转 / if 类 / let 链 / 圈复杂度结点与短路算子 / 布尔链根），
> `row` = 该单元的 cognitive 行下标（三条复杂度行须按 [3,4,5] 相邻乘车、值恒 0，否则按名拒 `row i: pre-judged complexity
> value (events ride)` / `complexity rows must ride as [3,4,5] triples`）、`seq` 单元内自 0 连续、`parent` 最近入类祖先的
> seq（−1 = 单元本身）、`pos` 0 头 / 1 体、`flags` 十三位（0 NESTING / 1 FLAT / 2 NEST_ONLY / 3 LABELLED_JUMP /
> 4 IF_KIND / 5 CHAIN / 6 CC_KIND / 7 CC_OP / 8 LOGIC_ROOT / 9 DIRECT〔树父即父事件〕/ 10 IN_ALT〔在父的 `alternative`
> 字段〕/ 11–12 if 的首个 alternative 子结点类别）、`aux` 链的操作数个数、尾列 = 布尔链根的算子序号；表按 `(row,seq)`
> 严格升序，是第七个计入上限的维度（`CE.Scan.overCap`；分块每事件一座，文件不跨块）。核按 `CE.Scan.Complexity` 从事件流
> 折出三数——白皮书的增量与嵌套罚〔else-if 由 DIRECT ∧ IN_ALT 推出、条件不抬嵌套 D31〕、圈复杂度的判定点、最大嵌套深度
> ——回执加性 `derived=[[rowIndex,value]…]`（每个单元三行、升序、递归增量之后；`cocBumped` 照旧回显且与 derived 的
> cognitive 行一致），Rust 只把值写回报告、无 `derived` 的应答按名拒（「7.2.0 之前的核把清零的行当测量值判了」）；此前三条
> 规则住在 Rust `scan/metrics/{cognitive,cyclo}.rs` 与 `scan/coc.rs`，本版删除。其余十四条拒绝按名（`CE.Scan.Events`：
> 行外 / 非 cognitive 行 / 负 seq / seq 不连续 / 父不在前 / pos 越界 / flags 越十三位 / 负 aux / 非链带 aux / 负算子 /
> 根无算子 / 算子无根 / 畸形行 / 非严格升序）。白皮书例题册 `contracts/fixtures/scan/whitepaper.ndjson`（38 行 = 六道
> 例题 + 递归锚 + 步 6 的 Java 9 / C 5 / C++ 8 / Lua 5 / R 4 行）成两半共读的夹具：Rust 半（`sonar_whitepaper.rs`）
> 钉事件（`CE_BLESS=1` 重写）并经核走到页边值，Haskell 半（`ScanEventsProps`）无解析器折到同一值。缺席 = 7.1.0 字节同、
> 无 `derived`（K16）。golden：新增 scan 八对（19 事件路三数回显 / 20 加 `callEdges` 的环记账落在 derived 上 / 21 预判
> 值拒 / 22 非三元组拒 / 23 畸形行拒 / 24 空表〔无事件的单元 (1,0,0)〕/ 25 seq 不连续拒 / 26 旧路无 `events` 字节同），
> 既有 144 对只动 proto。

> **7.1.0**（structure/1 模块度轴 + fourclass/2 声明级搬迁，加性 minor，计划 v2.29 步 10 批 C3 O54 / O48，2026-09-06）：
> `structure.request` 加性可选表 `dirEdges=[[fromDir,toDir,count]]`（**只载跨目录**有向边，
> `from ≠ to`、两端 `< |nodes|`、`count ≥ 1`、按 `(from,to)` 严格升序；缺席 = 轴 7 不判、空表 = 判为净——
> staleDocs / redundancy 的 Maybe 立场原样）。**intra 质量不上 wire**：`fileRefs` 的 `inside` 在一条
> 目录内边的**两端**各加一，故逐目录 `Σ inside×count` 恰为内部边数的两倍，核取半即得——一个数字两个主人
> 正是本族 seamSoft 那笔旧账。凭据 = 只在 `dirEdges` 在场时执行的**跨表律**：每个目录的 `inside` 之和为偶，
> 且 `outside` 之和等于 `dirEdges` 中与之相接（两个方向）的边量，任一不符按目录点名 `error/contract`。
> 应答 `axes`/`findings` 仅表在时携码 7 行（序恒升：7 在 6 后）；knobs 码域 0..18 → **0..20**
> （19=modFloor 默认 1‰、20=modMassFloor 默认 4），knob 回执 19 行 → **21 行**——**既有 golden 应答行随之
> 各多两行**（2.14.0 / 2.15.0 先例），其余键逐字节如前。判决 = 目录分划在有向多重图上的 Newman 贡献
> `q = e/m − o·i/m²`，除以该目录自身质量的上限 `qMax = mu(m−mu)/m²`，判 `rho = q/qMax` 是否低于地板
> （整数不等式 `1000(e·m − o·i) < modFloor·mu·(m−mu)`，全程整数不做除法）；`mu < modMassFloor` 或
> `mu == m` 者整条不判（`qMax = 0`，没有可分离的补集）。`dirEdges` 行计入 `structNodeCap`（C15），
> Rust 镜像同批改为逐项对齐核的 `famOverCap`（此前只计 nodes 行）。测量侧 = `ce structure` 恒发（与
> `fileRefs` 同一次 join，无第二次 walk、无新 I/O），故**自仓结构分迁移、与 1.6.0 不可比**；报告态不设门
> （v2.22 结项 O53 立场不变）。
> 同一未发布 minor 的 O48：`fourclass/2` 的对可加带成对可选两表
> `declRem` / `declAdd=[[keyHash,kind,start,end]…]`——本对**之前有之后无**（rem）/ **之前无之后有**（add）且该侧多重度恰为 1 的
> 声明键：keyHash = fnv1a(单元键)、kind ∈ {1 函数 / 2 具名非函数 / 3 impl / 4 Markdown 节}、跨距 1 基闭区间（名字与路径永不过线）。
> 两键**同生同死**且**必须覆盖整批**（目的地唯一性是批级量词，半批无法可靠回答）——半表 / 半批 / 跨距或 kind 形错 / 同侧同键
> 两行，各按对点名 `error/contract`（`decl tables come in pairs: pair i` / `decl tables must cover every pair: pair i` /
> `malformed decl span: pair i` / `duplicate decl key: pair i`）。回执随之带 `unitEdges=[[源i,宿i,keyHash]…]`（严格升序）：同 kind 的键在**恰好一个**
> 对里出现（两个即按名弃该键，无平局裁决、无相似度）、且两侧跨距内的 leftover 内容共享 ≥ `CE.FourClass.Cost.declFloor` 个互异
> 哈希时成边。`declFloor` 是**推导**的：一个「从一处消失、在唯一另一处出现」的声明键**就是**跨站要买的出处身份，故它付掉
> `siteCostCross`，余下要付的只有内容 ⇒ `declFloor = 1`（`declCredit = 0` 时塌回 `destFloor = 2`，旋钮可消融）。0 个共享行 =
> **改编**，本级对它不出声。两张声明表的总行数 > `CE.FourClass.Decl.declCap` 65536 ⇒ `unitEdges:[]` + `unitEdgesDropped:true`
> （拒绝，绝不截断）。**请求不带两表 = 回执不带 `unitEdges`，与本批前逐字节相同**：无行改类、无 block 移动、无 suspicion 变化，
> 故 L2 的七道行级门与 81,640 穷举参照等价全部不动。fourclass golden 新增三对（10 一行证据开边 / 11 双目的地拒 + 改编零边 /
> 12 半表按名拒）；§3 锚行数由 `fixture_contract` 从文件推导。报告新边 `lines:0`，行级已具名的边优先、只报一次。

> **7.0.0**（判决正确性批，**major**，计划 v2.29 步 8，2026-09-05）：`fourclass/1` 对级 `dup=[hash…]` 改为
> `dupSpans=[[hash,start,end]…]`（每个 after 侧出现一行，1 基闭区间；旧键不再读——请求形状变 = major），堆叠规则据此
> 改为「≥ `stackingNovelFloor` 条 novel 行落在新重复单元的跨度内」（O47；`start < 1 ∨ end < start` 按对点名拒绝）。同批
> 随行三条判决改动不改形状：`verdict/1` 轴 2 / 3 的质量改为**被判定对触及的互异文件数**、分母改为各自的机会宇宙——
> 轴 2 = 代码文件 `nodes − docFiles`（5b-8 起改读有位置行且不在文档集内的文件，见 7.2.0 条）、轴 3 = 文档文件 `docFiles`（O22；此前两轴数对、分母全体节点，分数与 1.6.0 不可比随
> CHANGELOG 声明）；`ce check` 起送 kind = 2 的 docdup 对行（O46；此前该轴在产品里恒零）；`erase/1` 第 2 类第 3 事实由
> 死亡位改为**死亡判决码 0..4**（0 = 不死；2 / 4 命中 `publicDeadVerdicts` 按 reason 6 拒绝——O51，RG10 自此真的到达
> 孪生路；`> 4` 按行点名拒绝）。golden 全族 request 行随 major 机器重写为 7.0.0（§3 锚 7.0.0）。
> **6.7.0**（同角色顾问族，加性 minor，计划 v2.29 步 5，2026-09-05；ADR-008 细则第六期）：第十二判决族
> `similar/1`——请求 `query=[[termHash,weight]…]`（可缺省 = 空袋；哈希严格升序、weight ≥ 1）+
> `rows=[[nHit,pHit,cHit,dHit,sHit,lHit,shapeEqual,bm25Num,bm25Den]…]`（每个候选一行九整数：六通道各自的
> 共享拼写项数、形状相等位 ∈ {0,1}、BM25 分数 = Num/Den 有理数——Rust 送 16 位定点的分子与 2^16 分母；名字与
> 路径永不过线）；回复 `order=[候选下标序]`（Num/Den 有理数降序、同分按请求下标升序，`Data.Ratio` 精确比较、
> 永不取浮点）+ `roles=[bool 每行]`（同角色位 = `(nHit ≥ 1 ∧ cHit ≥ 1) ∨ (nHit ≥ 2 ∧ shapeEqual)`，地板在
> `CE.Similar.Cost`）+ `counts{rows,queryTerms,role}`；query + rows > 65536 → 降级回执 `degraded:true,
> reason:"similar_too_large"`（空 order / roles）；行宽错 / 负计数 / shapeEqual 非布尔 / 负分子 / 非正分母 /
> 查询项形错 / 哈希非严格升序 → `error/contract` 按行点名。无旋钮、无 fail 档：顾问族只排序与打位，判决权
> 留给读者（spec §一）；旧核无此能力 = 测量侧具名降级「core offers no similar/1 (pre-6.7.0)」，绝不阻断（A9f）。
> **6.6.0**（墓碑残留族，加性 minor，计划 v2.27 步 4，2026-09-04；ADR-008 细则第五期）：第十一判决族
> `tombstone/1`——请求 `rows=[[kind,marks,erasedNames]…]`（每个候选面一行：kind 0 = 带括号标签 / 1 = 裸标签 /
> 2 = 散文句；marks = 句内回溯记号数；erasedNames = 该面拼出的被删名字数——Rust 的集属度量，两者皆零的面不送，
> 名字与路径永不过线）+ `knobs=[[0,budget]]`（码 0 = 预算，缺席 = 不评条件；码域 0..0）；回复 `sites=[行序]`
> （升序）+ `counts{rows,label,prose}` + `over`（sites > budget，无预算恒 false）。合取（散文行 marks ≥ 1 ∧
> names ≥ 1；标签行 names ≥ 1）与地板（`CE.Tombstone.Cost.minMarks` / `minName` = 1）入核；rows + knobs >
> 65536 → 降级回执 `degraded:true, reason:"tombstone_too_large"`（空 sites、over false）；kind 越域 / 负计数 /
> 未知旋钮码 → `error/contract` 按行点名。三腿（PreToolUse / Stop / precommit）只转发行序与 `over`、按
> `[tombstone] tier` 定档；核不可用或无此能力 = 具名降级，绝不阻断也绝不默过（A9f）。旧核缺席 = 字节不变（K16）。
> **6.5.0**（递归增量，加性 minor，计划 v2.23 步 4，2026-08-31；ADR-008 细则第四期）：
> `scan.request` 加性 `callEdges=[[from,to]…]`——测量侧在**一个解析单元内**证明的调用弧，
> 两端都是 `rows` 里的 cognitive 行下标，表严格升序（名字与路径永不过线，§5.9.2）；
> 核对这些弧求 SCC，环内每个单元 +1（S3776 §1「each method in a recursion cycle, whether
> direct or indirect」；自环 = 环长 1，无特判），回执加性 `cocBumped=[[rowIndex,生效值]]`
> 升序，恰在 `callEdges` 乘车且非降级时乘车。送的是**值**不是增量：`+1` 这个政策常数全仓
> 只在 `CE.Scan.Cycles` 一处。分块新增不变量「一个文件的行不得跨 chunk」——弧以行下标表述，
> 边界落在文件内部会把弧拦腰截断；装不下的单个文件按名拒绝而非劈开。缺席 = 字节不变（K16）。

> **6.4.0 附注（零 wire，L 轮 v2.20 步 #16，2026-08-29）**：新增报告 schema `ce.update-report/0.1.0`——`ce update`（CLI）/ GUI update 屏 /
> 插件 SessionStart 通知 + `/codeeraser:update` / MCP `update_check`（只读）四面一文档（`current` 含安装归属码 0..3、`platform`、`latest`、`pins`、`verdict` 0..2、
> `action` 0..4；码不载句，各面自持词表）；`--yes` 的落位回执 `{version, placed, sweptOld, installer}` 不设 schema（非报告面）。
> **6.4.0**（围栏批，加性 minor，L 轮 v2.18 步 #14 片 (b)，2026-08-29；O32/O33/O37/O38/O40/O43/O59/O66）：
> `verdict.request` 加性 `present=[u64…]`（严格升序；作用域内在盘、本次无连续行的文件实体——实体按**项目根**
> 键控，走无 ignore 文件、无 exclude 的第二条 walk，内置排除/秘密表/隐藏规则/归属剪枝照旧）→ 回执
> `ratchet.dropped=[[entity,code,committed]]`（`present` 上过线即在、空表亦答，降级面同）+ **第六具名 fail
> 条件 `rows_dropped`**（排除藏起的文件其已提交行是「掉线」而非「移除」，仅 `CE_ACCEPT_FENCE=1` 可认领——写入
> 不含这些行的基线）；`classKnobs` 码域 0..3 → **0..4**（4 = 仅 CoC 的棘轮容差：声明即对 metric 1 取代码 3，
> 零有意义）；类 id 域 1..=**64**（64 自此在栏内、65 越栏——四处读者同一谓词 `classIdPastFence`）；
> `thresholds` 码域 0..6 → **0..7**（7 = `cycleFloor`，与 `graph.request` 加性 `sccFloor` 同读一份
> `[graph] scc_floor`，上过线才回显，≥1）+ 加性 `cycleSelfLoops=[idx…]`（cycleFloor 1 时**必须**在场、他处
> 按名拒绝；带自环的单点 SCC 计入 cycle 轴）；每份回执（含降级）`newBaseline` 回显 `knobsDigest`、缺席 ⇔
> 未发。`scan.request` 加性 `knobsFence`（`null` = 无基线未围；`[current,recorded]` 两摘要各可 null）→
> 回执 `failed` 具名序 `hard_line, knobs_digest, degraded`（fence 上过线即在；`fail ⇔ failed ≠ []`）。
> `graph.request` 加性 `sccFloor`（≥1 否则按名拒绝，上过线即回显；1 时单点 SCC 仅在自环时成环）。Rust 侧
> `score/wire_check.rs` 对**每份**回执核 fail/failed 律、围栏策略（基线摘要 ≠ 声明 ⇔ `knobs_digest`）、摘要
> 回显、newBaseline 形（写者要落盘的文档）、present ⇔ dropped（缺 dropped = 6.4.0 前的核，按名拒绝）；`ce scan`
> 同围栏具名退 1、报告 0.2.0 `failed`；守卫在配置漂移或基线不可读时按**出厂** thresholds/exclude/classes 判预算
> 并在拒绝理由具名围栏。全部新键缺席时十二 golden 逐字节如前（仅 proto 戳改动，K16）；`fixture_contract.rs`
> 自文件推导 §3 三元组并对拍 Spec.hs 的清单。
> **6.3.0**（外来读者角色，加性 minor，L 轮 v2.18 步 #12，2026-08-28，用户裁「子仓只当读者、不当被测者」）：
> `graph.request` 节点行的 `roles` 得 **bit 7 = foreign**：该节点（文件、包或节）属于超仓 `.gitmodules`
> 声明的 submodule。核侧 `roleBits` 把它落到与测试约定同一入口位（`(7, 2)`）——其引用播种可达性、
> 永不被判；Rust 侧由索引自有事实 `files.owner`（schema v15；0 = own / 1 = foreign）标记，外来节点
> 只发 bit 7、其余角色一律不测，且被逐出每个判决宇宙（score / join / structure 的 `measured_nodes`、
> 克隆对与 docdup 的实例查询、顾问域 `f.owner = 0`）；未声明的嵌套仓在两条 walk 与守卫 Scope 处整体裁除
> （`gitmodules::owner` 三态 Own / Foreign / Cut 是唯一谓词）。无 submodule 的树不发 bit 7，十键与
> 判决字节逐位如前（K16）。graph golden 新增一对（24：外来文件节点只带 bit 7 而活、其引用使本仓文件活）。
> **6.2.0**（符号层顾问两表，加性 minor，L 轮片 (6) / ccm 步 #6，2026-08-27，口径 = 封版 spec v9）：
> `graph.request` 加性**两键同生同死**——`unmentioned = [[node, vis, conv]]`（声明文件的 node、
> 可见性三位字、约定类别字；`id` 投影严格升序；一行 = 本文件里一组无他文件提及的声明域，其名载荷
> `AdvisoryName` 留在 Rust 侧永不过线，K6 第三腿）与 `mounts = [[node, private, total, bits]]`
> （全节点恒一行、`take 1` 投影升序、`private ≤ total`、bit 0 再导出目标 / bit 1 包私有）。配对检查
> 占 `violation` asum **最前**：只发其一 ⇒ `unmentioned: mounts table required alongside` /
> `mounts: unmentioned table required alongside`；行级五条 `mount i: …` + 四条 `unmentioned i: …`
> 具名拒绝；两表各自析取项计价（`mountCap` 131072、`unmentionedHardCap` 524288，节点净空不动）。
> 回复加性 `exportUnmentioned = [[node, vis, conv, code]]`：`vis ∧ unmentionedVisMask(3) == 3` 且
> `conv` 无 `exemptCategories`（0..10）任一位者出行；code 全序 **1 > 2 > 3 > 0**（`mountedPrivate ∨
> pkgPrivate` ⇒ 1 private / vis bit 2 ⇒ 2 restricted / mounts bit 0 ⇒ 3 reexported / 否则 0 public；
> `CE.Graph.Advisory` 具名谓词链，缺 mounts 行读作 `[0,0,0]`）；行数 > `unmentionedCap`（131072）⇒
> `exportUnmentioned: []` + `unmentionedDropped: true`（**只在掉表时在场**）。**铁律**（K16/K33）：
> 两键缺席 = 十键回复字节逐位不变；带表与不带表 dead 集相同；顾问表永不能把门翻红——超硬阀
> `graph_too_large` 是唯一带 `fail` 的顾问路，本方生产者自限 131072 行不可达。`verdict/1` 的
> `rowTotal` 补计 `symbols` 行（K47）。graph golden 新增六对（18–23：配对两拒、六节点四码齐出、
> 空表、`private above total`、`malformed row`）。回复既非 degraded 又无 `exportUnmentioned`
> ⇒ Rust 侧具名拒绝（前 6.2.0 核的合法 minor 偏斜不得读作「已问且干净」）。
> **6.1.0**（RG10 防火墙抵达会动手的两个面，加性 minor，K 轮步 5，2026-08-25）：
> `CE.Graph.Dead` 把 dead 沿 indegree × reachability 分成四码，正是为了让
> **「库的公开 API 无人引用」永远塌不成普通 dead**——RG10 是一个**判决码**，不是一条策略。
> 4.1.0 给了 flag 位 0 生产者、判决码 2/4 首次能点火之后，**下游两个会照着这个判决动手的面
> 仍在读它的旁边**：①`ce erase` 的 class 3 只看置信度（`judgeRow [3, _verdict, conf, _, _]`），
> 于是一个 `unref_public` 文件成了可擦除行——**这一点被冻在契约夹具里**：erase golden 第 6 对
> 原本答 `[[0,1],[1,0],[1,0]]`，即公开未引用 API「可擦」；②join 格的 `Candidates.hs` 合成
> `pFlags = 0`，`publicGuard` 在生产态恒不点火，于是 `delete` 可以指着一个导出面提出。
> 改法两片，都是加性：`verdict/1` 接受 `symbols` 表——**就是 graph/1 自 4.1.0 起载的那张
> `[node, visibility]`**，只是改按 tier 宇宙下标；过线的是**原始可见性字**而非派生的 exported 列表，
> 因为「哪一位算导出」是判决（`Graph.Cost.exportVisBit`），留在核里（ADR-008）。erase 理由码新增
> 位置 **6 `public_surface`**——冻结码域只增不改号。**反事实**（K15）：不带表 = 带空表 = 旧路
> 逐字节相同；导出**死侧**则 delete 退位且理由位 6 亮；导出**活侧**判决不动（否则守卫成了静音而非防火墙）；
> 可见性字不含导出位则判决不动（决定权在**位**，不在这一行是否存在）。全族 golden 机器再生后
> 104 行变化中 **103 行只差版本串**，唯一实变正是上面那对 erase golden。

> **6.0.0**（旋钮指纹拓宽 **major**，K 轮步 4b，2026-08-25）：5.1.0 的 `classDigest` 改名 `knobsDigest`
> 并覆盖**整份解析后的 ce.toml**，而非仅 `[[rules.class]]` 一张表。起因是一轮 52-agent 五镜头对抗审查，
> 发现**在一小时内**就把范围判错的地方指了出来，而且我第一手复现无误：
> ①`[score] viol_cost = 0` 两行把一个仓从 **939/1000 FAIL 变成 1000/1000 pass**，而 axes 仍老实报着 `4:428` 的违规费；
> ②`[score] tol_abs = 100000` 把 +280 行的增长从 `1 over -> FAIL` 变成 `0 over, 1 tolerance drawn -> pass`；
> ③`exclude` glob 把文件连同它的棘轮行一起移出，无人喊停。三条都在挪与改 glob 同样的门，
> 三条都没碰类表，三条都不要求任何人具名。**「挑哪几张表来围」正是这类漏洞的成因**，所以不再挑：
> 标量 = fnv1a over 序列化后的 Config。它自动覆盖**还没有人添加的那个旋钮**，且只随**解析结果**变——
> ce.toml 里的注释与键序不动它。配置等于出厂默认的仓仍然什么都不发（K11 不变）。
> （L 轮步 #14 O39 起，零 wire：哈希对象改为**规范树**——与出厂默认不同的**有效**旋钮集，`config/canonical.rs`
> 四律：空叶即未声明、等于默认叶即默认（核默认由 `score::knobs::core_defaults` 镜像、经 core_wire 镜像门活钉）、
> 数组整值比较且类对象内同律、空对象不计、类名为标签不入树（步 #16 O42 收口：改名即静默）——故写成默认值的旋钮与没人声明过的可选项都不动它，一份固定声明的字面值
> 冻结在 `config_contract::the_digest_of_a_fixed_declaration_is_frozen`；本仓与测试子仓的摘要因此各移一次，具名重立。）
> 判决条件随键改名 `knobs_digest`；**请求字段改名属 schema 变更，故按 §2 走 major**——与 5.0.0 同一把尺。
> 反事实：五条 Rust 腿（出厂默认无指纹、两个绕过旋钮各自移动指纹且彼此不同、规则包四要素含声明序、
> exclude glob、JSON 转义使值无法冒充结构）+ 核内 K11–K14 原样迁移；本仓自身是活演示——它有 ce.toml，
> 指纹从无到有，`ce check` 当场报 `failed: ['ratchet_over','knobs_digest']`，具名重立后基线记下 `knobsDigest`。
> 金样 203 行改动中 199 行由 proto 串与改名解释，另 4 行 = 2 条内嵌 server 版本串 + pair 16/17 逐字段核实
> 只差改名与版本（改名改变了 aeson 的键序，故字符串级对拍不成立，需逐字段比）。
> 请求行随 major 机器重写为 6.0.0；核电池请求侧 proto 同步 22 处。
> **5.1.0**（规则包围栏 + per-class 棘轮容差 minor，K 轮步 4，2026-08-25，用户拍板 v2.14 ②）：
> ①`verdict.request` 加性标量 `classDigest`——对 `[[rules.class]]` 规范化声明（名、**声明序**的 globs、旋钮）
> 的指纹。名与 glob 仍永不过线（§5.9.2）：它们的哈希不是它们。编码为**长度前缀**（netstring 式 `tag:len:bytes`）
> 而非分隔符——首版靠 fnv1a 的 NUL 分隔，自带的腿当场抓到碰撞：名 `a` 带 glob `b` 与名 `a\0glob\0b` 无 glob
> 字节流全等（分隔符只能分隔不含它的东西，长度可以）。②`ce-baseline.json` 记录其天花板**在哪套规则包下立的**，
> 核加持名 fail 条件 `class_digest`，判据是**朴素的 Maybe 不等**且是全的：两边皆无=同意；改了规则包=不同意；
> 对着围栏之前的旧基线声明规则包=不同意；把基线记过的规则包删掉=也不同意。四种分歧要的是同一个答案：
> 具名说出来，让人去具名重立一条地板。只有 establish 写 digest（`CE_ACCEPT_BASELINE=1` 走空基线路），
> 故「同意一套新规则包」与「同意一条新地板」是同一个动作。③`classKnobs` 码域 0..2 → **0..3**，码 3 =
> 该类自己的棘轮容差（行数**绝对值**，非比例——想要它的是 vendored 与夹具树，它们要的是零或固定额度，
> 而大文件的百分比正是本旋钮要拿掉的白拿增长）。声明即**取代两条全局腿**，故 0 意味着一行都不许长、
> 全局 max(+2%,+10) 救不了它（因为根本没被查询）。它是唯一「零有意义」的类旋钮，故表的取值下界**按码判**
> 而非一刀切（码 0/1/2 是线，线为零是荒谬）。反事实：K11 = 无类声明仓 digest 缺席（**不是 null**）且 101 对
> 金样中 199 改动行里 197 行只动 proto 字段、另 2 行是不匹配文案内嵌 server 版本串〔核电池另有一腿断言
> newBaseline 无该键〕、K12 = 改规则包即 `failed=["class_digest"]` 而 `over` 为空——不是悄悄放松而是具名停下
> 〔fixtures/verdict pair 16〕、K13 = establish 记下 digest 且棘轮行仍三列、K14 = 类容差 0 时长一行即 over
> 且 allowed=天花板本身〔pair 17；全局 +10 腿够不着〕；另有 Rust 侧三腿钉指纹本身（声明序/名/glob/旋钮各一，
> 「零旋钮」≠「无旋钮」，以及长度前缀的单射性）。请求行随 minor 机器重写为 5.1.0；核电池请求侧 proto 同步 22 处。
> **5.0.0**（graph 节点行 legacy flags 列裁除 **major**，K 轮步 3d，2026-08-25）：节点行降为
> `[lang, kind, roles]` **单一元**——pre-2.28 的 flags 列离场。它自 2.28.0 roles 列成为权威后又被
> 生产、上线、丢弃了七个 minor；4.0.0 想同批砍掉却被实测拦下（flags 位 0 是公私判决轴，可见性无生产者时删列会让
> `unref_public`/`unreach_public` 连夹具都无法表达），4.1.0 的 `symbols` 表补上那个生产者，此条遂解锁。
> **档位**：§2 写死「schema 不兼容变更（删字段/改字段形状）必须 bump major」，删列正是改行形状，故 major——
> 计划原写 minor，2026-08-25 按本仓自己的规则修正（v2.14 就地记账）。代价为零：4.x 全程未发布（v1.1.0 出货 3.2.0）。
> **三列同元不同义**是有意为之：新三列 = lang/粒度/角色事实，旧三列 = lang/粒度/flags；major 在信封处拒绝一切
> 跨版本对话，那道拒绝正是使元数复用安全的机制，故 K1 由「按行元拒」改为「按 major 拒」。表级
> `node rows: mixed arity` 拒绝随之退役——只剩一种合法元数时，宽窄不对的行就是 malformed，且按**行下标**点名。
> Rust 侧 `flags::legacy_flags` 与 `LEGACY` 折叠表一并删除，随之退役的还有 `legacy_fold_is_the_pre_228_bits`
> 一条测试与 allow-claim 测试里的一行断言（电池名集差实测：Rust −1/+0，核 −1/+1 同一探针改口径）。
> 反事实：**语义保持**——夹具 pair 7 把同一批事实改走各自的通道（节点 0 的入口身份走 roles 0→flag 位 1，
> 节点 3/5 的导出面走 `symbols`），回复与 4.1.0 **逐字段相同**（dead 表码 1/2/3/4 齐全、pos、cycles、counts 皆同），
> 证明这是裁除而非语义迁移；99 对金样中 199 行改动、193 行只动 proto 字段，另 6 行 = 三条我方重塑的请求
> （pair 7/11/12）+ pair 13 的新 malformed 文案 + 两条内嵌 server 版本串的错误文案。
> 请求行随 major 机器重写为 5.0.0；核电池请求侧 proto 同步 16 处。
> **4.1.0**（导出面 minor，K 轮步 3c，2026-08-25，用户三度交本代理裁断 v2.14 K7）：`graph.request` 加性一键——
> `symbols=[[node,visibility]]`，node < 节点数、visibility ≥ 0、**严格升序**（该表是去重的 (节点, 可见性) 集合，
> 重复行=生产者丢了集合语义，按名拒 `symbol i: not strictly ascending`）。core 按 `Cost.exportVisBit`
> 读出导出节点、按 `Cost.publicFlagBit` 或上 flags 位 0——那正是 `Dead.deadTable` 一直在分的公私判决轴，
> 而它**从来没有过生产者**（`cli/src/graph/deadcode/flags.rs:9`：文件粒度永不置位，公开性是符号事实）。
> 判决码 2/4（`unref_public`/`unreach_public`）自此首次可达。该位**故意在 entryMask 之外**：导出面是判决轴、
> 不是入口主张（RG10），故它只改死节点报哪个码，永不改哪些节点死。缺席**与空表同路**（`symRows` 只喂 [] ），
> 字节与 4.0.0 客户端所得相同。表另计 `symCap`。**L 轮片 (2)（2026-08-27）起本表的 visibility 是存储字的 bit 0 投影**：
> `symbols.flags` 另存 bit 1（作用域导出）与 bit 2（`pub(crate)` 族受限）供后续 `unmentioned` 表用，`symwire.rs`
> 的 `SELECT DISTINCT … flags & 1` 在查询处掩码，本表字节与 `symCap` 定容皆不动（K34）。**同批未做**：原计划并列的 `symEdges` 不上线——K10 审计量的是
> 精度（683/683），而「无引用」吃的是召回，实测自仓 import 绑定只覆盖 1064 条 Rust 导出声明中的 170 条
> （补模块跳转到 248 条，~23%），漏掉的是全路径调用与方法调用（皆非 import 点位）。详见 DEVELOPMENT_PLAN v2.14 K7。
> **L 轮终裁（2026-08-27，用户拍板 ①）：删**——`symedges.rs`/`bindings.rs` 与 index 的 `bindings` 表随 schema v14 退役
> （提及否决器批片 (1)，DEVELOPMENT_PLAN v2.17 条；包含论证：有符号边必有某 import 行出现过该 token，否决器完全包含它），
> wire 面零变动（`symEdges` 从未上线）。
> 反事实：K5 = 无符号表/空符号表与 4.0.0 逐字节相同（99 对机器重生成后逐行对拍：195 改动行中 193 行只动 proto 字段、
> 2 行是不匹配文案内嵌的 server 版本串；核电池另有一腿直接比 `respond` 两次的字节）、K6 = 请求体无任何字符串叶子
> （`cli/tests/it/graph_export_surface.rs`，结构性断言而非按本夹具的路径列举）、K9 = 导出节点判 2 而其邻居仍判 1，
> 且死集合不动（`fixtures/graph` pair 16 + 核电池 `exportRides`）；两个旋钮各有反事实腿（读错可见性位=无面、
> 置 entryMask 内的位=该节点变入口而离开判决集）。请求行随 minor 机器重写为 4.1.0；核电池请求侧 proto 同步 19 处（Haskell 字面量 11 + Spec.hs 内嵌请求 8；`9.0.0` 的外来 major 探针不动）。
> **4.0.0**（erase class 0 退役 **major**，K 轮步 2，2026-08-24，用户拍板 v2.14）：`erase.request` 的 class 0
> （dead_file 本地计数路）自 2.32.0 被 class 3 取代、Rust 同 minor 起不再铸行，宽限窗至此关闭——**离开判决集**，
> 其冻结位保留并**按名拒绝**（`row i: retired class 0 (superseded by 3 at 2.32.0, retired 4.0.0)`），而非折进
> 「unknown class」：仍在发它的客户端由此得知接替它的是哪条路。位不重编——重编会为省一个数组槽而移动另外三个冻结码，
> `CLASS_NAMES` 改留 `(retired)` 占位（二义的两个 dead_file 同死）。纯裁除故走 major。**同批未做**：graph 节点行的
> pre-2.28 legacy flags 列本拟同批退役，实测拦下——flags 位 0（exported）是公私判决轴，符号表给可见性第一个真生产者
> 之前删列会让 `unref_public`/`unreach_public` 连夹具都无法表达（对拍实证：旧 golden 含码 2/4，删列重生成后归零），
> 故顺延至符号表落地后的 minor。反事实：K2 = class 0 行按名被拒（fixtures/erase pair 8）、K4 = 未受影响九族回复
> 除 proto 串外逐字节相同（98 对机器重生成后逐行对拍，仅 erase 两行按等价迁至 class 3、wire-errors 两条错误文案
> 内嵌 server 版本串）。请求行随 major 机器重写为 4.0.0（3.0.0 先例，§3）；核电池请求侧 proto 同步 19 处。
> **3.2.0**（规则包 scan 旁表 minor，I 轮 P3，2026-08-24，用户拍板 v2.13 ①）：`scan.request` 加性两键——
> `rowClasses=[classId…]` 与 rows **位置对齐**（长度必等、每项 < 64；缺席 = 全行走全局表）与
> `gradeOverrides=[[classId,code,warn,fail]]`（classId ≥ 1、code 0..6、阶梯同 grades 文法〔fail 0 = 无硬线、
> fail ≥ warn〕、(classId,code) 严格升序；仅非空时发）；core 按 (class,code) 查表回落全局有效表，
> `grades` 回显仍为全局表，`gradeOverrides` 到场且非 degraded 时原样回显（客户端断言往返）；两表计入
> scanRowCap；chunk 切分时类列随行同切。ce.toml 侧 `[[rules.class]].knobs` 增 `fn_lines_warn` /
> `fn_lines_fail`（P3 两键）；Rust 镜像 evaluate 按文件类取有效阈值，每判对拍恒等式覆盖到类。
> 无声明仓库 wire 字节不变。
> **3.1.0**（规则包 DSL v1 minor，I 轮 P1+P2，2026-08-24，用户拍板 v2.13 ①）：①`verdict.request` 的
> `continuous` 行可携第 4 列 **classId**——`[u_fp, metricCode, value, classId]`，路径类的 1 基声明
> 序号，0 = 默认类；全表单 arity，混排拒 `continuous rows: mixed arity`；classId < 64（栅栏 classCap）；
> 身份前缀宽 2 不变，棘轮只读三列前缀；②加性新表 `classKnobs=[[classId,code,value]]`——码域 =
> ceilings 恒发子集 {0,1,2}（sizeCeil / cocCeil / sizeHard 的类影子，**不新造码**），classId ≥ 1
> （类 0 即全局表，已有 ceilings 通道）、value ≥ 1、(classId,code) 严格升序；core 建 Map 求值、
> 缺键回落全局线，chargeAt 律与机会数不动；③回复在表到场时**原样回显** `classKnobs`（客户端断言
> 往返；无表 = 无键，旧回复字节不变）；④`newBaseline` **永三列**（类是本 run 收费参数，非棘轮
> 事实）；⑤ce.toml 侧 `[[rules.class]]`：name/globs 仅本地（§5.9.2），globset 与 exclude 同方言，
> 声明序首中，classCap 64，逐类 ladder_fault 于 load 咽喉；无声明仓库的 wire 字节不变（C1）。
> **声明一个类 = 分数迁移**（§2 发版声明义务同款）：类线一经声明，该类文件的轴 0（sizeMass 的 S/H）
> 与轴 1（cocOver 上限）换线收费，分数与声明前**不可比**；未声明 `[[rules.class]]` 的仓库判决与
> wire 字节均不变，分数序列照旧可比。
> 反事实证表 C1–C9 = core/test/ClassProps.hs + cli 侧 config_contract / scan::classes 电池。
> **3.0.0**（churn 行裁列 **major**，I 轮 D3，2026-08-24，用户拍板「现在就删」）：`verdict.request` 的 `churn` 表由五列 `[u,rewrite,append,added,survived]` **收窄为三列**
> `[u,rewrite,append]`——第 4 列恒等于 rewrite+append、第 5 列恒为 0（per-entity 存活从未测量），
> core 自 M5-3i 起两列全弃读（`Score.churnHeavy` / `Verdict.churnMap` 只解 rw/ap）；删列 = 请求形状
> 破坏性变更，按 §2 升 major：两侧实现 + 三个 core 测试 harness 的 proto 字面量 + **全十族 golden**
> 同批重生（请求行 proto 一律改写为 3.0.0；回复行经核机器再生，与旧回复除 proto/server 字串外
> 逐字节相同——判决面零变化的亲证）；「留+记愿望单」落选（用户裁）。同批 daemon 协议独立升
> **2.0.0**（`hello_ok` 砍无读者的 `version` 字段）——daemon 协议自有台账，见
> [DAEMON.md](DAEMON.md) §1（其后每一次 daemon bump 只记在那里）。

## 1. 信封（envelope）

ce ↔ ce-core 的每条消息 = 一行 NDJSON（UTF-8，无 BOM，`\n` 结尾，binary-mode I/O）。
每条消息必带信封字段，其余字段由 `type` 决定：

```json
{"proto": "<SemVer>", "type": "<message-type>", ...}
```

- `proto`：协议版本，当前 **<!--ce:ver:proto#v-->7.10.0<!--/ce-->**（单一来源：`cli/src/corelink.rs::PROTO`
  与 `core/app/CE/Protocol/Version.hs::proto`，两处必须一致——core 侧由共享
  fixture 钉住，两侧相等由 `cli/tests/it/core_wire.rs::corelink_open_and_desync`
  的 PROTO 断言焊住）。
- 未知**额外**字段必须被接收方忽略（同 major 内前向兼容）。
- 未知 `type` → **`error` 应答**（0.2.0 起；此前实现以 hello 形状拒绝，属缺陷已修）：
  `{"proto","type":"error","id":<回显|null>,"code","message"}`，
  `code ∈ {unknown_type, bad_request, too_large, contract, internal}`——`internal`
  为第五席，2.3.0 同代的 Main.hs 异常屏障引入（挂账清零批 2026-08-17：纯判决
  计算内任何缺陷成为 error/internal 行而非进程崩溃，id 恒 null——计算死在可信
  回显之前；Spec `refusalProbes` 钉其 code 字符串）。core 侧在 JSON 解析
  **之前**先做行字节上限预检（2.1.0 起 32 MiB，此前 1 MiB——2026-08-12 决策：
  唯一客户端是同机受信 daemon，而 graph 请求在 100k LOC 量级合法地 ~1 MB；
  真防护 = 各族容量护栏），超限即 `too_large`，不解析。
- **每条非 hello 消息的 `proto` 由 core 强制校验**（1.0.0 定稿修正，攻击评审 F8：
  0.x 实现只在 hello 协商，裸发/错 major 的请求曾被静默应答）：缺失或 major
  不符 → `error/bad_request`。hello 自身仍走 §2 协商应答（`accept:false` 更富）。
- `hello` 应答自 0.2.0 起带 `capabilities`（当前 `["hello","fourclass/2","graph/1",
  "clone/1","docdup/1","verdict/1","scan/1","structure/1","trend/2","erase/1","audit/1","tombstone/1",
  "similar/1","query/1","flow/1","merge/1","arch/1","tables/1","document/1"]`；fourclass/2 =
  2.0.0 的锚宽请求形状，7.1.0 加性 `declRem` / `declAdd` → `unitEdges`（能力名不变）——旧客户端探 /1 得缺席，响亮降级 L1 而非发不可解析的二元形状；
  graph/1 = M5-2 图族；clone/docdup/verdict = M5-3 三族，2.2.0 同批声明；scan/1 =
  ADR-008 P3 分级判决族，2.7.0 声明；structure/1 = M6 结构族，2.9.0 声明；
  trend/2 = M7.5b 趋势族，2.13.0 以 trend/1 声明、2.31.0 随 Theil-Sen 行为变化升 /2；erase/1 = M9 批 3 擦除谓词族，2.16.0
  声明；audit/1 = M9 批 7 会话审计族，2.24.0 声明；tombstone/1 = 墓碑残留族，6.6.0 声明；similar/1 = 同角色顾问族，6.7.0 声明；query/1 = 代码查询与架构规则族，7.3.0 声明；flow/1 = 函数内死代码族，7.4.0 声明；merge/1 = 克隆合并建议族，7.5.0 声明；arch/1 = 架构分析族，7.6.0 声明；tables/1 = 定义包族〔不是判决族〕，7.7.0 声明，同版 hello 应答加性 `tablesDigest`；document/1 = 文档族〔不是判决族〕，7.8.0 声明）——**纯信息发现**，接受/拒绝的唯一权威仍是
  §2 的 SemVer；能力缺席 = 客户端走 L1 并显式降级（A9f）。
- 客户端规则：应答 `type` 非预期或 `id` 不回显 = 失步 → 视为 L2 不可用，
  回退 L1 且降级可见——绝不给错答案，只给响亮的答案。
- `fourclass.request`（2.0.0 形状，7.0.0 起 `dupSpans`，7.1.0 起可带声明两表）：
  `{"id","pairs":[{"i","rem":[[[行,hash,宽],…],…],"add":[…],"dupSpans":[[keyhash,起,止]],
  "declRem":[[keyhash,kind,起,止]],"declAdd":[…]}]}`——rem/add 为 L1 判 novel/deleted 的**显著**行按
  **run 分组**（run 结构=对齐产物，Rust 侧产出），hash = fnv1a(trim)，宽 =
  trim 后 alnum 计数（行事实，Cost.anchorFloor 的判定输入）；`dupSpans` =
  after 侧新出现重复的**顶层具名单元**每次出现的键哈希与跨度（堆叠证据，符号知识留在 Rust，
  仅整数过线——ADR-002 A6）；`i` 为**不透明的文件对键**：批内唯一、由客户端选定，
  接收方只拿它当 Map/Set 键，**绝不按它下标回查**（`CE/FourClass/Wire.hs:36` 的
  pIdx 自述 "an opaque pair index"；重复 `i` 由 `CE.FourClass.violation` 判
  `error/contract`——Anchor 的 (pair,run) 图会静默丢掉重复者的 run；跨匹配要求
  `i` 不同）。协议允许**稀疏**键；7.1.0 的生产者发送全部已度量对，空 leftover 对也可能是第二声明目的地，
  不得在判唯一性之前丢掉。`declRem` / `declAdd` 同生同死、覆盖整批，缺席与空表不同（上方 O48 条）。
  within-first 前置（同对 add∩rem 必空）由 core 在边界校验，违反 → `error/contract`。
- `fourclass.result`：`{"id","moved":[[i,出行,入行]],"blocks":[[源i,源行,宿i,宿行]],
  "suspicions":[[i,规则名]],"degraded"(,"reason"∈{bucket_cap})(,"unitEdges":[[源i,宿i,keyhash]]
  (,"unitEdgesDropped":true))}`——`unitEdges` 当且仅当请求带声明两表时在场；moved 为单调
  重分类 delta；blocks 为 ≥2 行站点证据（扩展/归因行只进 moved 不进 blocks）；
  suspicions 为 M4 判定规则点火记录（堆叠常数在 CE.FourClass.Verdict）。
- `graph.request`（2.1.0 起）：`{"id","nodes":[[lang,kind,roles]],"edges":
  [[src,dst,kind,rung]],"pos":[idx],"unres":[[lang,unresolved,total]],
  "symbols":[[node,visibility]],"unmentioned":[[node,vis,conv]],
  "mounts":[[node,private,total,bits]],"sccFloor":u64,"judgedMask":u64}`——稠密 0 基索引即
  节点身份，**无文本形物过线**（ADR-002 A6；6.2.0 的两张顾问表同律——候选名 `AdvisoryName`
  留在 Rust 侧，过线的只有整数）；节点行**三元组、单一合法元数**（5.0.0 起：
  pre-2.28 的 flags 列裁除，宽窄不对的行按**行下标**报 `node i: malformed row (need
  [lang,kind,roles])`；表级 `node rows: mixed arity` 随之退役）、边严格升序且去重、端点与
  pos 越界 → `error/contract`（边界契约由 core 机检）；`symbols` 是 4.1.0 起的可选导出面
  表——去重的 (节点, 可见性) 对、严格升序，核按 `Cost.exportVisBit` 读出导出节点并按
  `Cost.publicFlagBit` 或上 flags 位 0，判决码 2/4 由此首次可达；缺席或空表 = 字节不变。
  `unmentioned`/`mounts` 是 6.2.0 起的可选**顾问两表，同生同死**（只发其一 ⇒ `error/contract`
  具名配对拒绝，占校验 asum 最前）：`unmentioned` 按 `id` 投影严格升序、每行 `[node, vis, conv]`；
  `mounts` 全节点恒一行、`take 1` 投影升序、`private ≤ total`、bits bit 0 再导出目标 / bit 1
  包私有；两表各自析取项计价（`mountCap` 131072 / `unmentionedHardCap` 524288），节点净空不动；
  缺席 = 十键回复字节不变、dead 集不变（K16/K33）。`sccFloor` 是 6.4.0 起的可选环底（与 `verdict` 的 `cycleFloor` 同读一份 `[graph] scc_floor`；≥1 否则按名拒绝，上过线即在 `graph.result` 回显）。`judgedMask` 是 7.2.0 起的可选判决语言位集（ce 恒发 `Lang::judged_mask()`；`unres` 行的 lang 按位校验，缺席 = 127 即旧七码；上过线即在 `graph.result` 回显）。
  `unres` 是 2.32.0 起的可选按语言站点
  台账，是**判决输入**：在场时每条 dead 行增置信列（`CE.Graph.Cost.confidence`），缺席 = 旧
  两列 dead 行、字节不变；总数 `unresolved_sites` 仍只进 Rust 侧报告与摘要行（请求体见
  `cli/src/graph/deadcode.rs` `GraphWire`，核侧 `core/app/CE/Graph/Contract.hs` `GraphReq`——4.1.0 符号表落地时随解码与边界校验自 `CE.Graph` 拆出）；
  超 `CE.Graph.Cost` 节点/边护栏 → `graph.result` 带 `degraded:true,
  "reason":"graph_too_large"`（绝不截断）。
- `graph.result`（语义 M5-2g 落地，穷举参照 harness 见 core/test/）：
  `{"id","dead":[[idx,verdict]],"reported":[[idx,verdict]],"fail",
  "pos":[[idx,indeg,outdeg,sccId,sccSize,reachIn]],"cycles":[[sccId,[idx]]],
  "counts":{"nodes","edges","kept"},"degraded"(,"reason"∈{graph_too_large})}`——
  `dead` 只承载文件粒度判决，`reported` 承载 package/section 聚合判决；非降级时
  `fail` 当且仅当 `dead` 非空，降级应答恒 true。两表 verdict ∈ {1 unref_private,
  2 unref_public, 3 unreach_private, 4 unreach_public}（入度×可达两轴 + 公私隔离）；
  判定旋钮全在 `CE.Graph.Cost`：`minRung`(=5，边计为引用的 rung 上限)、
  `entryMask`(=126，flags 位 1-6 为入口根；位 0 exported 有意不入——公私是判决轴
  不是活性声明)、`sccFloor`(=2，环报告的最小 SCC)；kept = 去重后被判定采用的边数。
- **M5-3 三族（2.2.0 同批声明，桩期对一切输入回 `error/contract`；契约形状 =
  设计定稿卷一 §2.2，各族判决批落地时在此就地实体化并重生成 golden）**：
  - `clone/1`（判决落 T3 批）：request 携后序树
    `{"trees":[{"lab":[Int],"lld":[Int]}],"pairs":[[i,j]]}`（`lld[i]` = 最左叶后代
    后序下标，`0 ≤ lld[i] ≤ i` + 后序可重建性机检；pairs 严格升序去重、端点在界内）；
    result 回原始 `ted` 与规模不回比值，2.5.0 起并回加性 `verdicts` 布尔数组
    （每 score 行一位，`CE.Clone.Cost.cloneDecides` 的输出——上报集的唯一权威）；
    `degraded.reason ∈ {clone_too_large}`。
  - `docdup/1`（判决落 docdup 批）：request 携**升序去重 shingle 哈希集**
    `{"sets":[[u64]],"pairs":[[i,j,verbatimRun]]}`（集合非序列——token 流不跨进程，
    ADR-002 A6；逐字 run 在 Rust 算好只过整数）；result 回 `[i,j,inter,union]`，
    2.5.0 起并回加性 `verdicts` 数组（`CE.Docdup.Cost.dupVerdict` = Jaccard 半
    ∨ verbatim 半的全析取——run 过线正是为让 core 持有全部判决输入）；
    `degraded.reason ∈ {docdup_too_large}`。
  - `verdict/1`（判决落 score 批）：request 携三信号事实表 + `baseline` 原样字节
    （Rust 不解释，ADR-008 反抢跑），2.6.0 起并可携加性 `dedup`
    `[blocks,budget]` 对（第二棘轮判决输入，`ce dedup --check` 专用），6.1.0 起并可携
    加性 `symbols`:`[[u,visibility]]`——**与 graph/1 同一张导出面表**，只改按 tier 宇宙
    下标；核按 `Graph.Cost.exportVisBit` 读出导出集并给 `Pos.pFlags` 置 `publicFlagBit`，
    join 格的 `publicGuard`（RG10）自此在生产态可点火；缺席或空表 = 字节不变；
    result 回判决四码 + `reasonBits`/`legsMask` 自陈 + 棘轮集合 delta，
    2.8.0 起并回生效 `weights` 表与 `ratchet.failed` 持名条件表；
    5.1.0 起 request 并可携标量指纹与 `classKnobs` 码 3（该类自己的棘轮容差，行数绝对值）；
    6.0.0 起该标量名 `knobsDigest`、覆盖**整份解析后的配置**（O39 起为其规范化有效旋钮集），`ratchet.failed` 的持名条件为
    `knobs_digest`，`newBaseline` 在指纹到场时加同名键（**缺席而非 null**——出厂默认配置的仓字节恒等）；
    `degraded.reason ∈ {verdict_too_large}`。
  - `scan/1`（2.7.0，判决与声明同批）：request 携测量行 `{"rows":[[code,value]],
    "naming":[[lang,style,upper,under,test]]}`（码 0..6，主体名/路径不过线；naming 自 2.30.0
    由 ce 恒发、与码 6 行逐位对齐，core 容其缺席；行内 lang 自 7.2.0 按判决语言位集 `judgedMask` 校验——ce 恒发
    `Lang::judged_mask()`，缺席 = 127 即旧七码，未降级的 result 回显同值）+ 调用弧 `callEdges=[[from,to]]`（6.5.0，
    两端都是 cognitive 行下标）+ 结构事件表 `events=[[row,seq,parent,pos,flags,aux,op…]]`（7.2.0 ③，ce 恒发，三条
    复杂度行清零乘车、核折出三数）+ 可选 `grades` 覆盖 `[[code,warn,fail]]`
    + 规则包两键（3.2.0）：`rowClasses`（与 rows 逐位对齐的 classId）与 `gradeOverrides`
    `[[classId,code,warn,fail]]`（码 ∈ {0,1,4}，回复原样回显）
    （fail 0=无硬线、fail==warn=合法单线配置、码严格升序）+ 围栏键 `knobsFence`（6.4.0，ce 恒发：`null` = 无基线未围、`[current,recorded]` 两摘要各 u64 或 null）；result 回
    `{"levels":[0|1|2 逐行],"counts":{rows,warns,fails},"fail",生效 "grades" 全表,"failed":[名…],"cocBumped":[[rowIndex,值]]（弧上过线）,"derived":[[rowIndex,值]]（事件上过线，每单元三行）}`（`failed` 具名序 `hard_line, knobs_digest, degraded`，`knobsFence` 上过线即在；`fail ⇔ failed ≠ []`）；
    `degraded.reason ∈ {scan_too_large}` 且自带 fail=true。

## 2. SemVer 协商规则

- **major 不同 = 拒绝**：应答 `accept:false` + `reason`，调用方报错退出。
- minor/patch 不同 = 接受（新字段走"忽略未知字段"规则）。
- **schema 不兼容变更**（删字段/改字段形状）必须 bump major，并同步更新两侧实现 +
  fixtures；major 不同按上条拒绝。
- **分数语义迁移**（轴语义/阈值/量纲）可随 minor，但 release notes 必须声明
  score migration。
- **信封常数变更**（行字节预检、错误码/reason 词汇扩充）：放宽 = minor（旧客户端
  照常工作），收紧 = major；变更必须在 §1 就地改写并注明日期与依据（2.1.0 的
  32 MiB 放宽为首例）。

## 3. Fixtures 约定

- `fixtures/handshake/`：wire golden（请求行 + 期望应答行交替；`hello-ok` 握手、
  `wire-errors` 错误应答），Rust（`cli/tests/it/core_wire.rs`）与 Haskell
  （`core/test/Spec.hs`）**逐字节**共同消费——同一份文件，防两侧实现漂移。
  字节比较可靠因为 freeze 钉 `aeson +ordered-keymap`（键序确定）。
- **request 行的 proto 有意滞留（2.2.0 立场声明，M5-3a；每次 major 重锚）**：2.2.0 翻批只重写
  reply 行、request 行留在 2.1.0；此后每次 major 都把全部 request 行随之机器重写
  （3.0.0 / 4.0.0 / 5.0.0 / 6.0.0 / 7.0.0 各一次），minor 之间有意滞留——今日锚在 **<!--ce:ver:anchor#v-->7.0.0<!--/ce-->**
  （<!--ce:count:golden_requests#digits-->302<!--/ce--> 行，server 恒答 <!--ce:ver:proto#v-->7.10.0<!--/ce-->）——它们是"minor 偏斜
  必须被接受"（§2：minor/patch 不同 = 接受）的**常设回归 fixture**。后人把
  request 行"修"成与 server 同版 = 删除该回归覆盖，禁止；新增 fixture 的
  request 沿用当前 major 锚（今日 <!--ce:ver:anchor#v-->7.0.0<!--/ce-->；唯 `handshake/hello-ok` 的握手 request 随
  server 走 <!--ce:ver:proto#v-->7.10.0<!--/ce-->）。这组「行数/锚/答版」三元组里，行数与答版是派生值——行数由 `contracts/fixtures/*/golden.ndjson` 数出、答版即 `PROTO`，两者都以 chip 落在本页；锚是手写常量（`cli/tests/it/facts/ver.rs::ANCHOR`），每逢 major 随请求行一起重锚并复核。
- `fixtures/hook-payloads/`：Claude Code `PreToolUse(Edit|Write)` 的**实测** stdin
  dump（官方文档无逐字示例，ADR-007 ⚠️ 项）。采集方式见该目录 README。
- fixture 变更 = 契约变更，走 §2 规则。

## 4. 工具链锁定（M0 验收项）

| 组件 | 锁定 | 载体 |
|---|---|---|
| Rust | <!--ce:tool:rust#v-->1.94.1<!--/ce--> | `rust-toolchain.toml`（仓库根） |
| GHC | <!--ce:tool:ghc#v-->9.14.1<!--/ce-->（LTS） | CI `ghc-version` + 本文件 |
| 依赖快照 | cabal freeze | `core/cabal.project.freeze`（378fe40 入库，2026-08-07；升级依赖时 `cabal freeze` 重生成） |
| 协议 | <!--ce:ver:proto#v-->7.10.0<!--/ce--> | §1 所列两处常量 |
| daemon 协议 | <!--ce:ver:daemon#v-->2.2.0<!--/ce--> | [DAEMON.md](DAEMON.md) + `cli/src/daemon/proto.rs::DAEMON_PROTO`（形状 golden：`fixtures/daemon/`；反引号拼写无入边——dogfood deadcode 门在 CI 首点火即抓获，链接语法即活化） |
