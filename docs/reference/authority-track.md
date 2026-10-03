# 权威轨设计册：定义进核 / 文档骨架进核 / 双语文本进核 / 老家族参考实现（计划 v2.32，「一处权威」第二期，ADR-008 细则第八期）

> 本册是 v2.32 四件事的落码权威（计划书横幅 v2.32 条与 §6 T 轨 v2.32 行指向这里）。每件事在本册各占一章；落码时以本册为准，改设计先改本册再改计划书再动代码。分析轨设计册 [analysis-track.md](analysis-track.md) 是本册的体例先例。

## 0. 一句话定位

判决者持有判决的全部陈述：语言的定义、报告文档的形状、控制台的句子都由 Haskell 核给出，Rust 只做解析、索引、文件系统、进程与打印（硬约束 1 不动）；六个还没有独立参考的老家族补上新家族已有的「参考第二实现 + 等价电池」。四件事都不改判决——每一步以「十语料十面旧 / 新二进制逐字节同」为门。它们来自 2026-10-01 用户令（§13 第 1–2 条），排在 v2.31 步 10 / 11 之前，1.9.0 在本轨之后发。

## 1. 范围与裁定

1. **四件事**（落码顺序即 §12）：① 定义进核（`tables/1`，proto 7.7.0 加性，§4）；② 文档骨架进核（每族应答加 `document`，各族随自己的 minor，§5）；③ 双语文本进核（与 ② 同一次应答，§6）；④ 六个老家族的参考第二实现与等价电池（只在 `core/test`，§7；另四族已有穷举参考，不再加）。
2. **不改判决**：四件事都只换「谁持有这份陈述」，判决字节不动；每步以十语料十面旧 / 新二进制逐字节同为门（§9），切换类的步另以「旧产物 == 新产物逐字节」为门（§5）。
3. **三面等价**：CLI / MCP / GUI 今天吃同一份报告 JSON；骨架进核后三面只是三种打印，等价由构造保证（§5）。
4. **不为占比写代码**：每一步都说得出它让架构更好的那句话（§4–§7 各章首句）。量过、说不出那句话的三条不做（§13 第 8 条）：统一走查引擎、前端进核、阶梯即数据。
5. **1.9.0 推迟**：v2.31 的步 10（全量文档与语言条）与步 11（发版 1.9.0）排在本轨步 0–7 之后，由本轨步 8 / 10 一并收口（§12）。
6. **口径类裁定由算法轨取代**（2026-10-03）：本轨做完后语言条估算 ≈ 37.4 %（§11）；用户改令「要求改成：haskell占比比rust高。不固定比例阈值。」并裁路线 = 把算法搬进核、不拆仓，原定步 9 的口径类裁定不再做，目标与路线见 [algorithm-track.md](algorithm-track.md)（计划 v2.33）。
7. **不留已知限制**（用户令 2026-09-26，分析轨 §1 第 6 条同）：凡想写成「边界」的，先问能不能按规范修对；修不对的在 §13 记立场与证据。

## 2. 不变量（一条不动）

- **整数过线**：这条不变量保护的是**被度量仓库的数据**——仓库里的名字、路径、源码文本永不送进核；它不限制**产品自己的常量**从核送回 Rust。`tables/1` 送的是语法结点 kind 名、标准库名、扩展名、产物目录名、协议词表，都是产品定义、没有一个字节来自用户仓库，方向是核 → Rust（§4）；文档骨架里凡来自仓库的字符串位仍是**类别化引用** `{"$": [类别, 整数…]}`——类别加判决自己已在用的整数（文件号、目录号、成员号……），字符串内容留在 Rust、由它按类别解出（§5）。
- **判决在核**、**顾问永非判决**：本轨不新增判决，不改顾问的身份。
- **解析 / 索引 / 文件系统 / 进程 = Rust**（硬约束 1）：Rust 仍是定义的执行者；`GRAMMARS` 表（链接编译好的文法）与以代码写成的 kind 启发式（`dedup/tokens.rs`、`similar/bag.rs`）留在 Rust。
- **三面等价**。
- **每一步以「十语料十面旧 / 新二进制逐字节同」为门**。
- **不为占比写代码**——每一步都说得出它让架构更好的那句话。

## 3. wire 契约与版本

| 件 | proto | 请求 | 应答 | 缓存与降级 |
|---|---|---|---|---|
| ① `tables/1` | 7.7.0（加性） | 无参（信封之外多一个键按名拒） | `tables.result` 一次答整包：十六个顶层键按表族分（`languages` / `scan` / `flow` / `slot` / `sites` / `calls` / `fourclass` / `ladder` / `walk` / `outputs` / `docdup` / `keys` / `flags` / `tombstone` / `compdb` / `protocol`），每语言的表在表族键下按语言报告名分，另带 `digest`（规范字节的 fnv1a64，hello 回执的 `tablesDigest` 同数；规范字节 135,145 B） | Rust 第一次用到时要一次，按核 `(version, proto)` 入键落盘缓存 `.ce/tables-<ver>-<proto>.json`；握手已知版本 → 命中无往返；持核链的 daemon 内存缓存；缓存损坏 = 重取；无核 / 旧核无 `tables/1` = 具名拒绝，不留内嵌副本 |
| ② ③ `document/1` | 每批一个 minor（步 3 = 7.8.0，新五族；步 4 = 7.9.0；步 5 = 7.10.0，控制台文本） | `document.request`：`family` + `ranges`（可被引用的宇宙大小）+ `rows`（判决已答的行原样回送 + 文档要而判决不要的整数：秩、行号、成员号）+ `facts`（标量）+ `degraded`（null 或 `why` 下标）+ 步 5 起可选 `lang`（0 en / 1 zh，缺省 0，其余按名拒）；每族的键、行宽、每列所指的宇宙写成一张陈述文本表 | `document.result`：`document` = 与今天报告 JSON 同形的骨架（字符串位 `{"$": [类别, 整数…]}`，产品常量直接出字符串）；步 5 起另带 `lines` = 请求语言的控制台每一行 `[stream, text, ref…]`（stream 0 stdout / 1 stderr，数字与产品词已写进 text，每个引用在 text 里留一个 `{}`、按序跟在后面）与 `exit: {fail}`（该面的否决位）；`tables/1` 包加 `document` 目录（每族 schema id 与空文档；`guard` / `audit` 两个句子族没有文档、不进目录） | 判决没发生而核够得着 → 照样问，带 `degraded` 与测量侧留下的事实（§13 第 20 条）；文档时刻核够不着、请求被拒、行总数 > 1,048,576（核答该族空文档 + `document_too_large`）= 具名拒绝、不是降级；包里的空文档只作陈述与电池的锚，测量侧永不绑定 |
| 退役 | 8.0.0（major） | — | 退役失去一切读者的键（§13 第 18 条：判决应答的行表仍是 `document/1` 的输入；已知一个：请求键 `judgedMask`） | 全部族切完后一次（§12 步 6） |

- 每个 minor 与 8.0.0 各一条 `contracts/VERSIONING.md`（最新在前），`Version.hs` 与 `corelink.rs` 的 `PROTO` 同步；`tables/1` 进 `Protocol.hs` 的 `families` 表（hello 能力表由它派生）。
- golden：`tables/1` 一对；各族切换时既有 golden 随 `document` 键重答（子仓 `fixture_contract::regen` 腿，分析轨 §3 的法子）。

## 4. 件 ①：定义进核（`tables/1`）

**那句话**：语言的定义（什么是语句、什么是表达式位置、哪些 kind 是循环、标准库叫什么、哪些路径是构建产物）是判决的一部分，应当由判决者持有；Rust 只是执行者。

### 4.1 今天

定义文本散在测量侧：`scan/spec_*` 34.5 KB、`flow/spec_*` 25.1 KB、`merge/slot_*` 16.6 KB、`fourclass/kinds.rs` 7.2 KB、`graph/ladder/hs_boot.rs` 42.4 KB、`java_jdk.rs` 7.2 KB、`ts_node.rs`、Go / Python 标准库名、Lua 标准库名、walk 的 secrets 与内建排除、`outputs.rs` 产物规则、`lang.rs` 扩展名表……（测量侧七目录内合计 204 KB）；七目录外 `graph/spec.rs` + `spec/calls.rs` 14.7 KB、`mention/conv/protocol.rs` 8.5 KB、`docdup/spec.rs` 2.8 KB、`graph/keys.rs` 2.0 KB、`deadcode/flags.rs` 1.9 KB、`tombstone/vocab.rs`、`compdb_flags.rs` 等。可由核持有的纯定义文本 ≈ 235 KB（只读盘点，HEAD blob 字节）。读法有两种：`const` 表，与 TOML 片段经 `toml::from_str` 读入（`flow/spec.rs`、`merge/slot.rs`）。

判决语言集今天有两处权威：Rust `Lang::judged_mask()` 算出掩码、随 `scan/1` / `graph/1` 请求送 `judgedMask`，核 `CE.Wire.Mask.judgedLang` 按请求里的掩码判「这一行的语言在不在判决集」——核信的是 Rust 报的掩码。语言表进核后掩码的所有者是核、Rust 是读者：判决集由判决者声明，这是定义进核让架构变好的那句话之一。

### 4.2 设计

- **范围** = 上面盘点的 ≈ 235 KB 纯定义文本。判据：**描述语言或产品语义、与文件系统状态无关、改一处就改判决**（§13 第 6 条）。留在 Rust 的：`GRAMMARS` 表、以代码写成的 kind 启发式（`dedup/tokens.rs`、`similar/bag.rs`）。
- **核的形**（步 1 落码定稿）：一门语言的全部定义是一份 TOML 文档（`name` 与顶层名表、`[scan]`、`[flow]`、`[[slot]]` 片），数据模块 `CE.Lang.<Lang>`（流表另放 `CE.Lang.<Lang>.Flow`）只装这份文档的字符串块、每块一个绑定（函数行线内），`CE.Lang` 按顺序拼块、一个读者（`CE.Lang.Toml` + `CE.Lang.Spec` 的`LangTables`）读成记录；跨语言表同形（`CE.Lang.Common` 与 `Common.*`，GHC 全局包表是一段行文本）；TSX 读 TypeScript 的文本加自己的 slot 片，C++ 读 C 的文本加 overloads、三个 `std::` 名与自己的 slot 片——与测量侧原来拼文本的法子一致。十三个同形的 record 模块在查重门下互成克隆（初稿实测主根 85 块），一门一份文档就没有这个形。JSON 键名 = 测量侧字段名 / 常量名的 snake_case。
- **Rust 的形**：`tables.rs` 按 §3 取包、缓存；消费者（`scan::spec::spec(lang)` / `flow::spec::spec` / `merge::slot::slot_spec` / 阶梯名表 …）签名不变，来源从 `const` 表与 `toml::from_str` 变成 `tables::get(...)`；Rust 的结构体（模式）保留——它们是消费者的形状；定义文本删除。
- **无兜底副本**：无核 / 旧核无 `tables/1` = 具名拒绝（§13 第 7 条）。

### 4.3 门

`tables/1` golden 一对；十语料十面旧 / 新二进制字节同；子仓「表对钉版文法」单元腿改读 `tables::get`（kind 名存在性仍在 Rust 腿验——核没有文法）；核电池 `LangProps`：每表非空、字段齐、语言间约束（statement ∩ expression = ∅、每个 `name_fields` 的 kind 在该语言 kind 集合里 …）；无核 / 旧核无 `tables/1` 的具名拒绝各一腿。

### 4.4 估算

Rust −≈ 235 KB，Haskell +≈ 250 KB（record 语法比 TOML 略长）。

### 4.5 已交付（步 2，2026-10-01）

- **Rust 的形**：`cli/src/tables.rs` + `tables/{fetch,cache,pack,leak}.rs`。包读进 `Tables` 一个记录（`Pack` 解引用到它，另带 `digest`、来源文件与扩展名 → 码的索引），每个表结构是 `leaked!` 的紧凑声明：serde 读它的所有权孪生、再把每个字段泄漏成消费者原来的 `&'static` 形——包有 JSON 转义（Doxygen 的 `\brief`、`"` 定界符），借用读不出，泄漏一次活到进程结束，消费者签名不变（`scan::spec::spec(lang)` 仍答 `&'static LangSpec`，阶梯名表、墓碑词表、docdup 的五个数各一个读者函数）。三级来源：进程内存 → `<根>/.ce/tables-<ce 版本>-<proto>.json` → 核；缓存身份 = `ce` / `proto` + 答包那份核二进制的 `{path, len, mtime_ns}`（一次 stat），完整性 = Rust 自己对包字节算的 fnv1a64；任一不符、文件坏了都重取并覆盖，写 = 临时名 + rename、写失败不是错。根由每个入口设：CLI 的 `main_cmds::or_cwd`（所有带根参数的命令与 MCP）、钩子的 `hookio::gated_envelope`、daemon 的 `serve`（离开根之前）、GUI 的 `commands::task`；没设时回落 `root::project_root(当前目录)`。`ce eject` 只读不写缓存（`tables::transient`）：它的活是删 `.ce/`，不能顺手建一个。
- **拒绝**（都退 2，都具名）：无核、核无 `tables/1`、包缺键（serde 的「missing field `<键>`」）；此后每条核链的 hello 若点名的 `tablesDigest` 与本次读到的包不等，按名拒（`corelink::Link::open` 里一处）。钩子拿不到包时不退 2（那在 PreToolUse 里读作拒写），而是在 stderr 说一句、照无钩子放行；GUI 把同一句作为任务的错误显示。
- **删除**：`scan/spec_{c,hs,java,launch,lua,r}.rs`、`flow/spec_{c,go,java,lua,py,r,rs,ts}.rs`、`merge/slot_{c,hs,java,launch,lua,r,ts}.rs`、`graph/ladder/{hs_boot,java_jdk}.rs`、`tables/native.rs` 二十四个文件，`lang.rs` 的 `LANGS` / `MACHINE_TXT` / `MENTION_WHOLE_RUN_EXTS`、`graph/spec.rs` 与 `spec/calls.rs` 的站点表与调用表、`fourclass/kinds.rs` 的种类表、`walk.rs` 的两张排除表、`outputs.rs`、`keys.rs`、`deadcode/flags.rs`、`compdb_flags.rs`、`docdup/spec.rs`、`tombstone/vocab.rs`、`mention/conv/protocol.rs` 与五个阶梯的名表的文本；子仓 `it/tables_equivalence.rs` 随镜像退役。语言条（§11 口径，`git ls-tree -l`、`cli/tests` / `contracts` / `site` 除外、CRLF 折 LF）：Rust 2,555,451 → 2,406,418 B（−149,033），Haskell 1,183,246 → 1,185,149 B（`CE.Lang.Toml` 头注一段、`CE.Docdup.Cost` 接过被删 Rust 注释里的出处、语言行的 `flow_judged` 列与它的 `LangProps` 腿），Haskell 占全部 29.00 % → 30.13 %、占 Rust + Haskell 31.65 % → 33.00 %。
- **门**：十个对拍语料与 e877f389 的自仓工作树（测试子仓 7462018 就位）各十三面（scan / dedup / docdup / deadcode / clone / structure / erase / check / graph --sites / graph --mentions / flow / merge / arch），e877f389 与本步的 release 两臂 143 对逐字节同、`ce flow` 在内（另 e877f389 的 `git archive` 树十三面的拒绝 13/13 同）；子仓新腿 `it/tables_package.rs` 两条（无核 / 旧核 / 缺键三种拒绝；缓存的写、命中不再问核、核换了重取、`ce` / `proto` 不符重取、文件坏了重取、hello digest 不符拒绝）；代价见 PERF-BUDGET「v2.32 步 2」一节（暖跑三面与钩子探针量不出，未命中的第一跑多付 ≈ 131 ms）。

## 5. 件 ②：文档骨架进核（每族应答加 `document`）

**那句话**：报告文档的形状（字段、排序、计数、schema id）是判决的陈述，应当只有一处权威；三面只是三种打印。

### 5.1 今天

核答整数行表 → 各族的报告装配把行表 + 索引里的路径 / 名字 / 片段 / 行号拼成 `ce.<family>-report/x.y.z`（设计稿盘点记 16 份 `face.rs`；以同名文件计今天 6 个，其余装配在各族的 `report.rs` / `judge.rs` / `model.rs` / `mod.rs` 里，`cli/src` 里的 `ce.*-report/` schema id 共 22 种）→ 各族 `console.rs` 渲染文本 → GUI / MCP 吃同一份 JSON。

### 5.2 设计

- **一个族、一次装配**：文档由独立的 wire 族 `document/1` 装出（`CE.Document` 分派 / 契约 / cap + 每族一个 `CE.<Fam>.Document`），不随各族应答走——flow 按 `rowCap` 分批、merge 按两个 cap 分块判决，文档若在 Rust 侧折叠各批就等于把装配留在 Rust。请求 = `family` + `ranges` + `rows`（判决已答的行原样回送 + **文档需要而判决不需要的整数事实**：秩、行号、成员号）+ `facts` + `degraded`；每族把自己的请求写成一张陈述文本表（`range` / `fact` / `rows` 名、行宽、每列所指的宇宙 / `ref` 类别），契约按它逐条核（`CE.Document.Contract`）。
- 核直接给出**文档骨架** = 与今天的报告 JSON 同形：字段集、数组内的排序、计数、schema id、`degraded` 位全在核；来自被度量仓库的字符串位一律是**类别化引用** `{"$": [类别, 整数…]}`（类别 + 判决自己已在用的整数；不预建符号表、不送符号计数），产品常量（schema id、kind 名、原因名、error 名、`t1t2` / `t3`、`?-`）核直接出字符串；按字符串排序的地方 Rust 送**秩**、核按秩排。
- Rust 一个通用绑定器 `bind(skeleton, resolve)`（遍历 JSON，`{"$": [类别, 整数…]}` → 按类别解出的字符串）；各族的报告装配删除，只剩请求装配 + 解析器。三面等价由构造保证。
- **降级与空文档**：陈述把表与事实分 `kept` / `judged` 两半，`degraded` 时只留 `kept` 的；判决没发生而核够得着，Rust 照样问 `document/1`、印核答的；核够不着 = 具名拒绝（§13 第 20 条）。每族的空文档（同一个装配函数对空请求的结果）进 `tables/1` 的 `document` 目录，作陈述与电池的锚，测量侧永不绑定。
- **切换规则**：每批一个 minor（`document/1` 加性；步 3 = 7.8.0）；切换提交以「旧装配产物 == 新 `bind` 产物逐字节」为门（十语料 × 每族 × 三面），然后删旧装配；全部族切完后一次 major 8.0.0 退役失去一切读者的键（范围见 §13 第 18 条）。
- **分组落地**：(e) 新五族 query / rules / flow / merge / arch 作样板（骨架最简单）→ (a) check / score / structure / join → (b) graph：deadcode / mentions / sites / canvas → (c) scan / dedup / clone / docdup / erase → (d) churn / trend / tombstone / similar / audit / update / health。(d) 里 update / doctor / setup / health 留在 Rust（它们描述的是 Rust 进程自己的环境，`ce doctor` 必须在没有核时也能跑）；audit / probe / precommit / commitmsg 没有 `ce.*` 文档，它们的句子由 `document/1` 的两个句子族 `guard`（PreToolUse 拒绝理由）与 `audit`（Stop / precommit / commitmsg）给出，`document` 为 `{}`。

### 5.3 门

每族切换提交：旧装配产物 == 新 `bind` 产物逐字节（十语料 × 该族 × CLI 的 JSON 与控制台两面；MCP / GUI 读同一份文档，由构造等价）；十语料十面旧 / 新二进制字节同；核电池每族骨架腿（字段齐、符号下标在 `symbols` 内）；步 6 退役后 parity 门改读骨架。

### 5.4 估算

Rust −≈ 200 KB，Haskell +≈ 180 KB。Rust 的减量在步 6 兑现（旧面、控制台、`print_*`、名字表与镜像退役）；步 3–5 各批 Rust 先增（读者结构、请求装配、解析器），见 §5.5。

### 5.5 已交付（步 3，2026-10-01）

- **核**（3A）：`CE.Document`（族表、目录、`docRowCap`、应答）+ `CE.Document.Contract`（请求记录、陈述读者 `readSpec`、通用校验 `offence`、类别引用 `ref`）+ `CE.{Arch,Query,Flow,Merge}.Document`（query 模块出 query 与 rules 两族），`document/1` 7.8.0，`tables/1` 的 `document` 目录；flow 的判决语言集由 `CE.Lang` 语言行的 `flow_judged` 派生（§13 第 19 条）。
- **Rust 的形**（3B）：`cli/src/document.rs`——`Request`（`range` / `rows` / `fact` / `degraded`，`empty` / `zero` 补齐陈述要的每个键）、`Held`（一个面一条核链：判决与文档同走；判决请求失败的链作废，文档时刻另起一条）、`assemble_over` / `assemble`（问 `document/1`、核答不出即具名拒绝）、`bind` + `trait Resolve`（`{"$": [类别, 整数…]}` → 解析器的字符串）、`Why`（本侧自己的文本，类别 `why`）、`ranks`（按字符串序的处所，请求送秩）。每族 `face.rs` 只剩判决、请求装配与解析器 `Names`；`<族>/report.rs` 是读者（`Deserialize`，键为 `String`），控制台与退出码读它；四个 CLI 面经 `main_prelude::document_face` 一条路：读、退出码规则（merge 的 `--group` 越界在此拒绝）、再打印。删除：各族的 `label` / `assemble` / `document` / `report_json`、query 的 `ERROR_NAMES` 与 sort 名、arch 的切点与簇的标注、`arch::face::document`。
- **拒绝**（都退 2、都具名 `<族> document: …`）：文档时刻核够不着、核无 `document/1`（`pre-7.8.0`）、拒绝、`document_too_large`；判决没发生而核够得着（核无该族、请求被拒、传输错位）照样问、印核答的降级文档（§13 第 20 条）。
- **不留副本**（3C）：Rust 里五个 schema id 与 flow 的 kind 名常量删除——面印绑定后文档的 `schema`；钩子两腿的 feed 读包的 `document.flow.kinds`（`flow_report::kind_name` / `kinds_json`）；`--kind` 按包的目录译码，目录里没有的名字送 −1，由 `CE.Flow.Document` 拒为 `shown <i>: unknown kind; the catalogue lists …`，Rust 只把行号换回用户给的名字（`flow document: unknown kind "dead"; the catalogue lists …`，退 2）；3D：目录的 `kinds` 行为 `[名, advisory]`，`judged()` / 钩子 feed / 守卫腿读这一列，`flow_report::ADVISORY` 删除。事实登记表的 `report:{arch,flow,merge,query,rules}#schemaver` 改读核的常量（`schemaId` / `querySchemaId` / `rulesSchemaId`），方法学册 16–19 的 `schema` 芯片同改。核答出文档之前就要 schema id 的 Rust 路径：没有（§13 第 21 条）。
- **门**：切换门——十个对拍语料与 e877f389 的自仓干净树各 21 面（十三个 JSON 面 + query / rules JSON + 五族控制台），e877f389 的 release 与终树 release（3A–3D 变基到 0a128885 后）两臂 231 对逐字节同（自仓的 query / rules JSON 里 `rules_file` 是含臂目录名的绝对路径，换成同名后同）；对 baa4f8af 213 同、18 不同全在 merge（baa4f8af 没有合并第二代）；判决没发生的一腿（中继核从 hello 能力表里藏掉一族，四族 × 各自的面在 python 语料上）三臂 10/10 同；子仓 `it/document_catalogue.rs` 两腿（包的目录 = 冻结的 `tables/golden`；五族绑定后文档的 `schema` = 目录、flow 发现的 kind 都在目录里）；代价见 PERF-BUDGET「v2.32 步 3B」一节（arch / flow / query 在噪声内、rules 两坐都是本侧快约 180 ms（未定因），merge 两坐 +417 / +408 ms〔+4.8 % / +4.1 %，量时处理器负载 59–77 %〕是 2.8 MB 文档本身的一来一回）。
- **体积**（对 3A 的提交树、CRLF 折 LF）：`cli/src` Rust +15,141 B（删 910 行、加 714 行，另加 `document.rs` 与四个 `report.rs`、`query/rows.rs` 共 16.6 KB）、核 +1,037 B——本批 Rust 净增，§5.4 的减量在步 6 兑现：旧面的装配本就薄（serde 序列化同一组结构），切换后读者结构留下，另多了请求装配与解析器。

### 5.6 已交付（步 4B，2026-10-01）

- **Rust 的形**：七族（check / structure / join / deadcode / mentions / sites / 图屏）的面判决后问 `document/1` 并绑定，七份 Rust 装配删除。请求装配 + 解析器：`score/document.rs`、`structure/document.rs`、`join/document.rs`、`graph/deadcode/document.rs`，mentions（`mention/face.rs`）、sites（`graph/mod.rs::sites_document`）、图屏（`graph/canvas.rs`，送 deadcode 的表加每个结点的 `[node, kind, fileRow]`、边、位置、环成员）就地改为请求；读回的 `Report` 只是控制台与退出码的读者——check / structure / join 共一个 `report::Bound`（`bound!` 写实现、`read_bound` 读回并留住文档、`print_bound` 印 JSON 或控制台），`ce graph --sites` / `--mentions` 共一个 `report::print_read`，可选的单行表共一个 `Request::single`。删除：check / join / structure / mentions 的 `report_json`、`report::deadcode_json` 与 `DEADCODE_SCHEMA`、`graph::sites_json`、图屏的 Rust 装配（`document` / `file_edges` / `file_cycles`）、structure 的 `tree_rows` / `split_relabel` / `relabel`、join 的 `file_rows` 装配半与 `pair_verdicts`、两份 `VERDICT_NAMES`、`WHY_CODES` 的英文半（中文半 `WHY_ZH` 与 `why_line` 留到步 5）、`ADVISORY_NAMES`、`GRAPH_NULL_IMPORT_GRANULARITY`、`store::KINDS`（改读包的 `store` 表）与七个 schema id 常量。`ce baseline` 的写半不动。
- **不留副本**（第 21 条照做，第 31 条）：判决应答给名字的两处——check 的 `failed` 与四族的降级原因——由目录带名字表，Rust 按名字在包里的位置送码；事实登记表的七个 schema id 与两个码数（`count:join_codes` / `count:deadcode_codes`）改读核的常量。structure 的 S6 汇总只读判决的死行，不问文档（第 32 条）。
- **门**：切换门——十个对拍语料（各提交成一个 git 仓库，好让 `--days 14` 有历史可读）与 b3443723 的自仓干净克隆，每棵树 35 面（本步七族的 JSON / 英文控制台 / 中文控制台、`check --days 14`、`structure --deep --split-candidates`、`join --days 14`、`deadcode --check` 的退出码，加既有十族 JSON），b3443723 的 release 配它自己的核对终树 release 配车道的核，770 对逐字节同 768、不同 2（自仓 query / rules JSON 的 `rules_file` 是含臂目录名的绝对路径，换名后同）；判决没发生的一腿（两臂各自核的 hello 桩：只答握手与定义包，不给任何判决族、不给 `document/1`）在 rust 语料上 25 面 50 对同 38、不同 12，全在 `graph --mentions` / `--sites`：旧路无判决可等、照印测量并退 0，新路按名拒 `ce graph: sites|mentions document: core offers no document/1 (pre-7.8.0)` 退 2。图屏：b3443723 的 release 与它自己的核在两棵子仓夹具树上答的 `faces::graph_screen` 冻结为子仓 `it/golden/graph_screen.b3443723.ndjson`，常设腿 `graph_screen_frozen` 读终树，逐字节同。
- **代价**：PERF-BUDGET「v2.32 步 4B」一节（自仓克隆，ABAB ×7）：check / deadcode / structure --deep 两坐的差都在噪声内（第一坐 +198.7 / +8.7 / +7.9 ms，第二坐 +22.9 / −26.3 / +156.3 ms）；文档沿用判决的核链，一个面一个核进程。
- **体积**（对 093ee88e 的提交树、CRLF 折 LF）：cli/src +6,452 B（41 个文件改动、4 个新文件）、core/app +618 B（4 个文件）、core/test +357 B（1 个文件）、gui −24 B（1 个文件）；Rust 的减量不在本步，七族的读者结构（控制台与退出码）仍在 Rust

## 6. 件 ③：双语文本进核（与件 ② 同一次应答）

**那句话**：控制台文本只是文档的另一种渲染；渲染规则与消息目录是文档权威的一部分。

### 6.1 今天

各族 `console.rs` 各自渲染控制台文本；中文帮助与句子在 `main_lang.rs` 的 `ZH_TSV`；守卫的句子在 `guard/say.rs`；GUI 的文字在 `gui/ui/i18n.js`。

### 6.2 设计

- 核持有消息目录：`CE.Text` 是机制（模板按 key 取 `(en, zh)`，槽位 `{}` 自左至右填，与 Rust `i18n::line` 的读法同），每族的模板在 `CE.Text.<Fam>`，逐字节转录自今天的 Rust 打印处（只有英文的句子 zh = en）。文档本身不带句子：应答另给 `lines`（请求 `lang` 那一种语言的控制台每一行 `[stream, text, ref…]`）与 `exit: {fail}`——一次请求只答一种语言，Rust 按 `[ui] lang` 选语言后发请求、把每个 `{}` 按序换成引用解出的字符串、按 stream 打印。
- 数字与产品词由核写：整数按 Rust `Display`，`{:+}` 一处（`CE.Text.signed`），一位小数按 Rust `{:.1}` 的读法（取除得的 double 的精确二进制值、半数取偶、负零留负号、零分母 `inf` / `-inf` / `NaN`，`CE.Text.fixed`；子仓 `it/document_number_format.rs` 对 golden 的 ROI 电池逐个用 Rust 格式化比对）；按显示宽度对齐的地方（arch 的度量表）宽度是测量侧事实（`widths` 表送每个目录路径的字节数与字符数）；merge 片段的 40 字符截断是新引用类 `clipped`，由 Rust 截。
- Rust 留下的：clap `--help` 与 `main_lang.rs` 的 `ZH_TSV`（只服务 `--help`，`zh_surface` 门照旧）、SARIF、写入者（observe feed 行、erase 日志记录）、`hookio` 的截断、退出码 2（降级 / 无核 / 程序错误是测量侧自己的拒绝）；GUI 的 `i18n.js` 不动。
- 守卫句（`guard/say.rs`）与审计句同理进目录（`guard` / `audit` 两个句子族），PreToolUse 腿经 daemon 的核链取句——daemon 已持核链，无额外往返。

### 6.3 门

与件 ② 同一切换门（`lines` 与旧 `console.rs` 的输出逐字节同，en / zh 两语）；守卫句的双语逐句同；`zh_surface` 照旧绿。

### 6.4 估算

Rust −≈ 40 KB，Haskell +≈ 40 KB。

## 7. 件 ④：老家族的参考第二实现与等价电池

**那句话**：新家族都有「参考第二实现 + 200 例等价电池」（`ReferenceQuery` / `ReferenceFlow` / `ReferenceArch`），同一标准该一视同仁。

### 7.1 今天

新家族三份参考与 200 例对拍在 `core/test`。老家族里已有四份独立参考，都是穷举小实例（比 200 例种子电池更强）：`Reference.hs`（FourClass 的归属判决，有界族全枚举）、`ReferenceGraph.hs`（图判决：4 顶点全部有向图的可达与 SCC、3 顶点全图 × 全入口子集的四路判决）、`ReferenceTed.hs`（按定义枚举 Tai 映射的树编辑距离）、`ReferenceJaccard.hs`（docdup 判决，8 元宇宙全子集对）。Verdict / Score、Structure、Erase、Trend、Tombstone、Similar 六族没有独立参考。

### 7.2 设计

- 六族：Verdict / Score（评分）、Structure、Erase（`keptRows`）、Trend、Tombstone、Similar（BM25 合取）。每族 `Reference<Family>.hs` + `<Family>EquivProps.hs`（生成器覆盖拒绝边界与上限）。
- 已有穷举参考的四族（FourClass、Graph、Docdup、Clone，§7.1）不再加参考实现——加了就是为占比写代码（§13 第 10 条）。
- 参考实现换一种写法（foldM / 运算符表 / mapMaybe），语义等价靠生成的 200 例对拍，查重门不把它当克隆（§13 第 9 条）。
- 参考实现只在 `core/test`，不进 `core/app`、不进 wire。

### 7.3 门

`cabal test` 全绿；每族等价 200/200；查重预算主 50 / 子 91 不动（新块先消）。

### 7.4 估算

Haskell +≈ 85 KB：按新家族参考的实测密度（`ReferenceFlow.hs` + `ReferenceFlowGen.hs` 331 行 14,047 B、`ReferenceQuery.hs` 212 行 9,963 B），每族参考 + 电池 ≈ 14 KB × 6。

### 7.5 已交付（步 7，2026-10-01）

六族的参考第二实现与等价电池在 `core/test`，共十八个新模块、1,809 行、81,829 B（实测比 §7.4 的估算少约 3 KB）：`ReferenceContract`（六族共用的拒绝与应答比较词汇，不 import `core/app`）、`ReferenceScore` + `ReferenceRatchet` + `VerdictGen` + `VerdictEquivProps`（Verdict / Score：七轴计价、加权折叠、软线、区间曲线、单次增长容差、棘轮、join 格与置信、`verdict/1` 应答）、`ReferenceStructure` + `ReferenceSplit` + `StructureGen` + `StructureEquivProps`、`ReferenceErase` + `EraseEquivProps`、`ReferenceTrend` + `TrendEquivProps`、`ReferenceTombstone` + `TombstoneEquivProps`、`ReferenceSimilar` + `SimilarEquivProps`。每份参考只 import 本族入口与本族 `Cost` 常量，按册里的定义换一种写法（计数代替差分折叠、次序统计代替排序、条件表代替守卫链、真值表代替合取式）；每族 200 例种子对拍（`ReferenceFlowGen` 的生成器），域小处另加穷举（Tombstone 27,055 例、Erase 31,975 例、Trend 22,143 例、join 格 39,015 例、模块化轴 13,824 例），每族至少一条拒绝腿与一条上限腿；十条变异探针（每族至少一条）各自让本族电池转红、还原后 sha 逐字节同。`cabal test` 663 ok / 0 FAIL；查重主 50 不动（落码中途最高到 58，共用词汇模块、文本拒绝表、条件表三种写法消回）。

## 8. 核的模块布局与尺寸

- 件 ①：`CE.Lang`（汇总）+ `CE.Lang.Contract` + 每语言一个数据模块 `CE.Lang.<Lang>` + `CE.Lang.Common`。
- 件 ③：`CE.Text`（消息目录）。
- 件 ②：每族一个 `CE.<Family>.Document`（骨架装配）。
- 件 ④：`core/test/Reference<Family>.hs` + `core/test/<Family>EquivProps.hs`。
- 每文件 ≤ 290 行（子仓 `core_size_gate` 的真实上限）。按分析轨 §11 的 ~44 B / 行计，单份定义超过约 12 KB 就放不进一个模块（如 `graph/ladder/hs_boot.rs` 的 42.4 KB 名表），这类表按 290 行墙分片成同一前缀下的若干模块，分片名落码时定。
- 电池各一（`WireHarness.runLegs` 两条平行列表形，避开查重门的表同韵），注册在 `core/test/Spec.hs`。

## 9. 验收与门（每步共用）

- 两仓 `cargo test / clippy --all-targets -- -D warnings / fmt --check`、`cabal test`、七条产品腿（主根与 `cli/tests`）、golden 重生（`CE_BLESS=1`）、`fixture_contract`、`core_wire` 往返、`face_parity`、`facts_*`、`docs_*`、`site_*`；两仓 ADR-006 具名重立；查重预算只降不升（新块先消后入账）。
- 判决不动：每步旧二进制 / 新二进制在十语料十面 + 自仓干净树十面对拍逐字节同（v2.30 步 7b 的形）。
- 切换类的步（2、3、4、5）另加本册 §4.3 / §5.3 / §6.3 的逐字节门。
- 全量 it 在每步提交前跑一次，红先单跑读 panic 再定抖动 / 真红。

## 10. 文档与事实面

- 计划书横幅与 T 轨行随步；CHANGELOG 每步一块。
- `contracts/VERSIONING.md`：7.7.0 一条、各族 `document` minor 各一条、8.0.0 一条；`Protocol.hs` 的 `families` 表加 `tables/1`。
- `zh_surface` 门照旧（clap `--help` 留在 Rust）；GUI `i18n.js` 不动。
- 步 8 全量文档：README 双语的技术栈 / How it works 里关于「谁持有定义与文档」的句子、方法学册里引到被删定义文件的引文（引文门在删文件的那一步就会点名，随该步重瞄）、parity 块随 bless；语言条读数入 §11 与发布说明。
- 本轨不新增判决家族，事实计数（家族数 / 屏数 / 工具数）不因本轨而动；新增事实随步登记，不预先加芯片。

## 11. 语言条：基线与估算

- **量法**同分析轨 §11：`gh api repos/skymanbp/CodeEraser/languages` 的字节表（`.gitattributes`：`cli/tests/**` 与 `contracts/**` vendored、`site/**` documentation、`core/test` 计入）。
- **基线（a0417628，GitHub linguist）**：Haskell 855,590 / Rust 2,534,369 / 其他 300,362 B，Haskell 23.2 %。
- **Rust 构成**：graph 550 KB、根文件 227 KB、scan 221 KB、dedup 173 KB、flow 164 KB、fourclass 129 KB、mention 126 KB、similar 79 KB、score 79 KB、daemon 79 KB、tombstone 73 KB、query 73 KB、merge 65 KB、structure 62 KB、docdup 58 KB、guard 48 KB、erase 45 KB、audit 42 KB、update 32 KB、config 31 KB、join 29 KB、arch 27 KB、mcp 27 KB、trend 23 KB、churn 21 KB、setup 19 KB、flow_report 13 KB。
- **只读盘点（Explore 代理，HEAD blob 字节）**：测量侧七目录 999 KB = 表与模式 204 KB（模式 38 KB）+ 表驱动走查 224 KB（其中 120 KB 是语言无关算法：`calls.rs` / `callees.rs`、flow 降表）+ 语言特判 306 KB（阶梯 211 KB）+ 接线 266 KB；可由核持有的纯定义文本 ≈ 235 KB（§4.1）；渲染类（face / console / main_* / say / i18n）≈ 240 KB；GUI Rust 17 KB。
- **估算**：件 ① + ② + ③ → Haskell ≈ 1,326,000 / Rust ≈ 2,059,000 / 其他 300,000 → **≈ 36.0 %**；加件 ④ → Haskell ≈ 1,411,000 / 总 ≈ 3,770,000 → **≈ 37.4 %**（设计稿按十族记 ≈ 38.5 %，立项时按实改为六族，§13 第 10 条）。
- **目标**（2026-10-03 改）：不再是 40 %，而是 Haskell 字节数高于 Rust、不固定比例；缺口由算法轨（[algorithm-track.md](algorithm-track.md)）补，拆 GUI 子仓与再加家族两条口径办法不做。
- **步 7 实读**（本仓 `git ls-tree` 按上述口径重算，Rust 与其他未动）：Haskell 855,590 → 937,809 B（+82,219）/ Rust 2,534,369 / 其他 300,362 → Haskell 24.9 %（a0417628 为 23.2 %）。
- **记账**：每步收口写一行读数进 CHANGELOG 该步块；v2.31 的语言条读数改以本轨收口（步 8）时为准，那是 1.9.0 发版声明的读数。

## 12. 分步（每步自带门，落码顺序；工作树车道并行开发、按序落地）

| 步 | 内容 | 门 |
|---|---|---|
| 0 | 计划修正案与立项（2026-10-01）：本册 + 计划书 v2.32（横幅立项句、ADR-008 细则第八期、§6 T 轨 v2.32 行）+ CHANGELOG `[Unreleased]` 块 + cc-memory 重锁（硬约束 2） | docs 门全绿、基线具名重立 |
| 1 | 定义进核 A（2026-10-01 已交付）：核 `CE.Lang.*` 三十五个模块（`Toml` / `Table` 两个读者、`Spec` / `Spec.Flow` 两个模式、`Contract`、十三个语言模块与八个 `.Flow` 模块、`Common` 九模块）+ `CE.Lang` 汇总 + `CE.Tables`，转录自 a378e78c 的 Rust 表；`LangProps` 十一腿 + `tables/1` golden 两对（全包 / 多一个键被拒）+ proto 7.7.0（hello 加性 `tablesDigest`）；子仓 `it/tables_equivalence.rs` 三腿证核的包与 Rust 临时镜像 `cli/src/tables/native.rs` 逐键相等、三方同数、多键被拒 | `cabal test` 全绿、`tables/1` golden 一对、既有 golden 只动 proto |
| 2 | 定义进核 B（2026-10-01 已交付，§4.5）：Rust `tables.rs` + 缓存 + 消费者改读 + 定义文本删除（二十四个文件）；精度册两族（语言十一份、flow 十份）退役再在干净树上重生成，两个提交 | 十语料 + 自仓十三面旧 / 新二进制 143 对字节同、子仓表对钉版文法腿改读、无核 / 旧核 / 缺键具名拒绝 |
| 3 | 文档骨架样板：新五族（query / rules / flow / merge / arch）+ 绑定器 + 切换门；3A 核（`CE.Document` + `CE.Document.Contract` + 四个 `CE.<Fam>.Document`、`document/1` 7.8.0、`tables/1` 的 `document` 目录、`DocumentProps` 十腿、golden 十四对）；3B Rust `cli/src/document.rs`（`bind` + `Resolve` + `Held`）、四个 face 只留判决、请求装配与解析器，`<族>/report.rs` 读绑定后的文档、切换门；3C Rust 不留目录副本、`--kind` 的未知名由核拒（golden 加一对）；3D 目录的 kind 行带 advisory（§5.5，§13 第 17–22 条） | 旧装配产物 == 新 `bind` 产物逐字节（十语料 + 自仓 × 五族 × CLI 两面） |
| 4 | 骨架 (a)(b)：check / score / structure / join；graph 的 deadcode / mentions / sites / canvas；4A 核（七个 `CE.<Fam>.Document`〔`Score` / `Structure` / `Join` / `Graph` / `Graph.Screen` / `Graph.Sites` / `Mention`〕、`document/1` 7.9.0、目录多七项、`DocumentProps4` 九腿 + `DocumentHarness` 共用四腿、golden 十对；裁定后续：名字各回一个所有者、包多 `store` 表、VERSIONING 拆第二册）先在车道提交，4B Rust 绑定与切换门（2026-10-01 已交付，§5.6）：七族的面问 `document/1` 并绑定、七份 Rust 装配与名字表删除、目录多 `failed` / `reasons` 名字表、图屏冻结金样 | 同步 3 的切换门 |
| 5 | 骨架 (c)(d) + `CE.Text` 双语目录 + 守卫句：scan / dedup / clone / docdup / erase；churn / trend / tombstone / similar / audit / update / health；flow 文档带各 kind 的标签，`gui/ui/hub_flow.js` 的标签表退役、芯片集改读文档的 kind 列表（§13 第 22 条）；5C 核（`CE.Text` + 十二个 `CE.Text.<Fam>` 目录、十个 `CE.<Fam>.Lines`、`guard` / `audit` 两个句子族、`document/1` 7.10.0、`DocumentProps5` 九腿、golden 加三十余对）先在车道提交，5A 核（scan / dedup / clone / clone-units / docdup / erase / erase-trail / churn / trend / similar 十族的文档、八份 `CE.Text.<Fam>` 目录与十个 `CE.<Fam>.Lines`、仍是 7.10.0、`DocumentProps6` 十腿、golden 加五十二对）同样先在车道提交；Rust 改读 `lines` 在 3B / 4B 落地后接上；R0（车道 `lane/v232-step5a`）先接管道并切 churn 一族作试点（第 45–49 条） | 同步 3 的切换门 + `lines` en / zh 逐字节同、`zh_surface` 绿 |
| 6 | 删旧 face / console / `ZH_TSV`、major 8.0.0 退役旧键、parity 门改读骨架 | 十语料十面字节同、`face_parity` 改读骨架后绿 |
| 7 | 老家族参考实现与电池（六族：Verdict / Score、Structure、Erase、Trend、Tombstone、Similar；可与 3–6 并行车道）——**已交付**（§7.5） | `cabal test` 全绿、每族等价 200/200 |
| 8 | 全量文档 + 语言条读数记账（§11 口径；v2.31 步 10 的文档面一并） | docs / site / facts 门全绿、引文重签 |
| 9 | 口径类裁定——2026-10-03 由算法轨取代（[algorithm-track.md](algorithm-track.md)，计划 v2.33），本步不再做 | — |
| 10 | 发版 1.9.0（v2.31 步 11 即此步）；等算法轨达标（Haskell > Rust）再发 | RELEASE.md 链 |

依赖：0 先于一切；1 → 2；3 → 4 → 5 → 6；7 可与 3–6 并行车道开发；8 在 0–7 之后；10 在算法轨达标之后。

## 13. 拍板记录（2026-10-01）

1. **用户令与三题裁定**（2026-10-01）：用户令「想办法减少rust比例或者增加haskell。任何办法。」→ AskUserQuestion 三题：① 真改代码选「报告文档与双语文本进核」+「十三语言的定义表进核」，并要「思考更加优雅的方法」（本册件 ① ② ③ 的形即那个回答：核持有陈述、Rust 只执行与绑定，而不是把 Rust 改写成 Haskell）；② 口径类办法做完再视结果定（步 9）；③ 1.9.0 推迟到所选办法做完再发（§1 第 5 条）。
2. **范围再裁**（2026-10-01，量化后 AskUserQuestion）：「两项 + 老家族参考实现（推荐）」——件 ① ② ③ 加老家族的参考第二实现与等价电池（件 ④；裁时按设计稿的十族估 ≈ 38.5 %，立项按实改为六族、≈ 37.4 %，见第 10 条）；最后的缺口到时再裁（拆 GUI 子仓 +2.3 点或再加一个新家族 +1.8 点）；「统一走查引擎」量下来只换 0.6 点，不做。
3. **提交署名**（用户令 2026-10-01，常设）：「不要加claude提交署名，署名只写我自己」——本轨起每个提交说明末尾不加任何署名行，作者只有 git 配置里的用户本人。
4. **不变量一条不动**（主会话按原则自答）：整数过线、判决在核、顾问永非判决、硬约束 1、三面等价、逐字节门、不为占比写代码（§2）——本轨换的是陈述的持有者，不是分工。
5. **整数过线靠符号引用保持**（主会话按原则自答；步 3 细化为类别化引用，见第 17 条）：文档骨架的字符串位一律 `{"$": k}`，符号表只送计数，内容留在 Rust；句子是 `{"$t": key, "args": [...]}`，槽位里仍是符号引用（备选「把路径 / 名字送进核让核拼字符串」违反 §5.9.2）。
6. **定义文本的判据**（主会话按原则自答）：描述语言或产品语义、与文件系统状态无关、改一处就改判决——三条都满足才进核；`GRAMMARS`（链接编译好的文法）与以代码写成的 kind 启发式不满足「是文本」这一条，留 Rust。
7. **无核不留内嵌副本**（主会话按原则自答）：无核 / 旧核无 `tables/1` = 具名拒绝；两处权威正是要消掉的东西（备选「Rust 内嵌一份兜底」把一处权威改回两处）。
8. **三条不做的理由**（主会话按原则自答，数字量过）：统一走查引擎——可折叠的走查器只 64 KB（其余 120 KB 是语言无关算法要留作库代码），+0.6 点，风险大于收益，定义进核已拿走它真正的价值（表）；前端进核（daemon / MCP / 守卫 / update / setup ≈ 530 KB）——用户未选；阶梯即数据——阶梯 211 KB 里是读构建 / 配置 / 文件系统的真逻辑，不是表。
9. **参考实现换一种写法**（主会话按原则自答）：foldM / 运算符表 / mapMaybe 等第二种写法，语义等价靠生成的 200 例对拍，查重门不把它当克隆（memory `reference-evaluators-need-a-second-spelling` 的先例）。
10. **件 ④ 按实改为六族**（2026-10-01，主会话自答）：设计稿写「老家族没有参考实现」是盘点漏了 `core/test` 的四个 Reference 模块（`Reference.hs` / `ReferenceGraph.hs` / `ReferenceJaccard.hs` / `ReferenceTed.hs`，§7.1），立项时按实改为六族，估算 38.5 % → 37.4 %；那四族不再加参考（备选「十族照做」= 为已有穷举参考的判决再写一份抽样参考，只为占比）。
11. **整数过线保护的是仓库数据，不是产品常量**（2026-10-01，主会话按原则自答）：§5.9.2 不让被度量仓库的名字、路径、源码文本进核；`tables/1` 的内容（kind 名、标准库名、扩展名、产物目录名、协议词表）是产品定义，方向核 → Rust，没有一个字节来自用户仓库，故不触这条不变量；骨架里来自仓库的字符串位仍是符号引用（§2、第 5 条）。
12. **判决掩码分两步换主**（主会话按原则自答，步 2 落码）：`judgedMask` 照发、值改由包的语言行算出——掩码的内容已由核声明，线上那一列与核的读法（`CE.Wire.Mask`）不动；把这一列从请求里拿掉是 wire 断代，留给 8.0.0 与骨架退役一起做（§12 步 6），不为它单开一次 major。flow 家族的判决语言集同一道理进核（主会话 2026-10-01 补入步 2）：步 1 的盘点漏了 `cli/src/flow/mod.rs` 的 `JUDGED`，它是一张定义表——哪几门语言的 flow 发现算判决——唯一诚实的主人是语言表；语言行加 `flow_judged` 列（十门，掩码 1540127，`LangProps` 一腿钉住且蕴含 `judged`），`flow::judged_mask()` 改折包里的这一列，步 3 的文档骨架在核里算每条发现的 `judged` 时读同一列。
13. **缓存的身份是三件加一个 Rust 自己的数**（主会话按原则自答）：`ce` 版本、`proto`、核二进制的 `{path, len, mtime_ns}` 决定「这份缓存是不是这份核答的」，Rust 对包字节的 fnv1a64 决定「文件有没有坏」；`digest` 是核的数，Rust 不重算它，只在之后每条核链的 hello 上对照——身份对上而 digest 不对，是缓存检查漏了一次换核，按名拒而不是读过去。
14. **`query/prelude.rules` 不进包**（主会话按原则自答）：它是 `ce query` 的前奏规则，用户可读可改的程序文本，不是语言或产品的定义表；§4.2 的判据第三条（改一处就改判决）对它不成立——它不改任何家族的判决。
15. **冻结的降表只守输出**（主会话按原则自答）：flow 的降表读的仍是核那份表，表进核后精度册的出处门照旧按路径守 `cli/src/flow/` 与 `scan/{functions,walk,lang}.rs`；表文本不在那些路径里了，门守的是读者与降表——一份表改了而读者没动，由十语料十三面的字节门与 `LangProps` 抓，不由出处门抓。
16. **精度册退役与重生成是两个提交**（主会话按原则自答，813f4976 / e1a6b520 先例）：生成器把 `git status --porcelain` 非空读作 dirty、拒绝覆盖冻结档，所以「在步 2 的树上重生成」按构造是两步——退役提交删档、考题翻回审阅档阶段，下一提交在退役提交的干净树上逐份生成；flow 的判决掩码在退役提交上不清（判决零改动是本步的不变量，提交 E 清掩码的理由——降表改了——这里不成立），掩码腿在那一个提交上按构造红。
17. **字符串位按类别引用**（步 3A 落码者按原则自答；主会话 2026-10-01 核准，「任务书的表点的是类别，不是拼法」）：第 5 条的 `{"$": k}` 细化为 `{"$": [class, 整数...]}`——类 = `path` / `dir` / `slashed` / `unit` / `var` / `text` / `value` / `goal_name` / `column` / `pred` / `rules_file` / `query` / `at` / `why`，整数是行里本来就有的下标（文件号、单元序号、变量号、文档内成员号、跨度），Rust 按类与整数从自己的表取字符串（`cli/src/document.rs` 的 `bind` + 每族一个 `Resolve`）；符号表不单独上线，它的计数就是请求的 `ranges`。产品常量（schema id、kind 名、reason 名、错误名、`t1t2` / `t3`、`?-`、sort 名）由核直接写字符串（第 11 条）。与步 3 任务书的引用表有三处不同、按写定的收：flow `var` 带 `(f, nth, v)`（一个单元有多个变量）；merge 的引用用文档内成员号 `k`，不用 `(g, m)`；query 的程序错误位置与消息都是 `why` 文本。每族陈述文本的注释一行写明每个类别的参数顺序。
18. **`document/1` 是独立能力，不是判决族的加性键；8.0.0 只退役失去一切读者的键**（步 3A；主会话 2026-10-01 裁）：骨架装配不进 `arch/1` / `query/1` / `flow/1` / `merge/1` 的应答，单独成一族——请求送判决应答已有的行表（Rust 按类重编号）与测量侧事实，核按每族一份陈述文本（`range` / `fact` / `rows` / `ref` 四种行，`CE.Document.Contract.readSpec` 读）校验并装配。判决族的行表因此仍是 `document/1` 的输入，§1 原定 8.0.0 退役的「旧键」不包括它们；8.0.0 只退役失去一切读者的键，目前已知一个：请求键 `judgedMask`（步 2 起值由包的语言行算出）。
19. **每批一个 minor；flow 的判决语言集读语言表；一份包一个摘要**（步 3A；主会话 2026-10-01 裁）：步 3 = 7.8.0（`document/1` 加性 + `tables/1` 加性 `document` 键），以后每切一批族一个 minor。flow 的判决语言集属于核的语言表：步 2 给 `CE.Lang` 的语言行加 `flow_judged`、Rust 的 `flow::judged_mask()` 读包；本步不再持副本，`CE.Flow.Document` 每条发现的 `judged` 与目录 `document.flow.judged` 都从 `CE.Lang` 的语言行派生，子仓 `it/document_catalogue.rs` 对 `flow::judged_mask()` 逐位比（比的是包自己的位）。包多了一个所有者（`CE.Lang.pack` + `CE.Document.catalogue`），`tablesDigest` 从 `CE.Lang` 挪到 `CE.Tables`（`CE.Lang` 唯一的改动：删一个导出与定义）。
20. **判决没发生时照样问核；目录里的空文档测量侧永不绑定**（步 3B；主会话 2026-10-01 裁）：陈述把表与事实分成 `kept` / `judged` 两半，`degraded` 时 `judged` 的必须空 / 为 0，`kept` 的可以留（query 的程序字段与 `faults` / `heads`；flow 的 units / stmts / vars / uses 与 `langs` / `rankFiles` / `shown` / `unlowered`）。判决没发生而核够得着（超 cap、请求被拒、传输错位、核没有该族）→ Rust 照样发 `document.request`，带 `degraded: <why>` 与留下的事实与行，印核答的文档；文档时刻核够不着（没有核、核没有 `document/1`、拒绝、`document_too_large`）→ 具名拒绝（退 2；MCP 报错），不印半真的文档。目录里每族的 `empty` 是该族陈述对空请求的装配，只作陈述与电池（`DocumentProps` 第一腿）的锚。步 2 起没有 `tables/1` 的核在任何走查之前就被拒，旧的「无核 → 降级文档」路已不存在；本步的可见变化是另两种：只答 hello 的核、`--core` 指向不存在的路径——此前印降级文档，此后具名拒绝（子仓 `arch_face` / `flow_face` / `merge_face` 三条腿改断言拒绝）。
21. **目录在 Rust 不留副本**（步 3C；主会话 2026-10-01 裁）：第 7 条对文档目录同样成立——schema id 只从绑定后的文档读，kind 名只从包读；`--kind` 的未知名由核拒（请求只送整数：译码在 Rust 按包做，译不出的送 −1，核点名并列出目录）；事实登记表与方法学册芯片改读核的常量。步 2 已退役的形（Rust 常量 + 对包的等价门）不再出现：子仓的门把包的目录钉在冻结的 golden 与各族绑定后的文档上。代价记账：名字写错时判决照常走完、到文档时刻才由核拒——只有写错的名字付这笔钱，而拒绝只有核这一句（Rust 先查一遍就成了同一拒绝的第二句，步 5 还得去对齐）。哪个 kind 只当顾问同是定义（步 3D）：目录的 `kinds` 行是 `[名, advisory]`，核的 `CE.Flow.Document` 判 judged 与答目录读同一张 `kindTable`，Rust 的 `judged()`、钩子 feed 与守卫腿读包里的这一列，Rust 的 `ADVISORY` 常量删除；落点读 wire 自己的说法（参数的发现 `declSeq` 为 −1），不再借 kind 号。
22. **GUI 的 kind 标签与芯片集等步 5**（步 3C；主会话 2026-10-01 裁）：`gui/ui/hub_flow.js` 的 kind → 标签表是显示文本，留到步 5 由 `CE.Text` 把各 kind 的标签按请求语言放进文档再退役；芯片的初始集合是定义副本，同在步 5 改读文档的 kind 列表。
23. **步 4 的四条前提**（步 4 任务书 §0，4A 落码时照做）：一批一个 minor（7.9.0；步 5 = 7.10.0、步 6 = 8.0.0）；控制台文本本步不进核（句子、roast 档、deadcode 的 `entry_globs` 提示、structure 的 ROI 留 Rust 到步 5 一次换）；文档一个键不多不少（退出码仍读判决应答自己的位，`ce baseline` 写的 `ce-baseline.json` 不是报告文档、不动）；任何代码不依赖键序。
24. **无判决的文档也进核**（步 4 任务书 §0 第 5、9 条）：mentions 与 sites 没有核判决，形状、schema id、语言名（`CE.Lang` 的行）与站点 kind 名照样由核陈述，Rust 只送整数；`mention_rev` 是测量侧的索引修订号，作事实送、原样放进文档。
25. **降级分两种来源；无核一律具名拒**（步 4A，主会话裁 2026-10-01）：判决应答自己的降级原因（`graph_too_large` / `verdict_too_large`，各由答它的模块 `CE.Graph.graphTooLarge` / `CE.Verdict.verdictTooLarge` 持有，`CE.Document.Contract.coreReasons` 引用）按码送、核直接写字符串；判决没发生的原因仍是 Rust 的 `why` 文本。structure / mentions / sites 的文档没有 `degraded` 键——这三族的陈述不立 `why`，契约按名拒任何 `degraded` 下标。核起不来时：步 2（ab16e390）起每一次走查都先要核——语言表本身来自 `tables/1`、Rust 不留内嵌副本（第 7 条；文档目录同，第 21 条）——所以步 2 之后没有哪个面能在无核时跑，「无核 = 具名拒绝、退 2」是所有面的同一条规则。mentions 与 sites 今天不用核照常印文档，那是步 2 之前的状态；切到核之后它们与 check / structure / join / deadcode 一样具名拒，这是一致，不是 4B 新引入的行为变化。
26. **按字符串排序的只有两处**（步 4A 逐处读码）：join 的 `files[]`（`BTreeMap<(a, b)>`，按路径对）与 sites 的行（`analyze` 按路径排文件、文件内按检测序）——这两处送秩、核稳定排序。任务书 §0 第 8 条点名的 structure `tree` / `splitCandidates` 与 deadcode `unmentioned` 今天不按字符串排：`tree` 按目录号、`splitCandidates` / `sizeExempt` / `findings` / `deviations` 按应答序、`unmentioned` 按核行序再按 Rust 名表序；check 的候选是整数下标、没有字符串。
27. **join 的单元键按行引用**（步 4A）：任务书写 `unit_key f nth`，但 `nth` 是同一个 key 里的序号（`fourclass::units::with_nth`），`(f, nth)` 定不出 key；改为 `{"$": ["key", k, side]}`——`k` 是单元行在请求里的位置、`side` 0 / 1，Rust 按行取。
28. **图屏的节 → 文件是测量侧事实**（步 4A）：每个结点一行 `[node, kind, fileRow]`，`fileRow` = 与该结点同路径的文件行（无则 −1），Rust 按路径算出；包端点按 kind 在核丢、自环与重复在核丢、文件层 SCC 在核数。Rust 今天对越界的边端点 `continue`，核的契约改为按名拒（按构造不会发生）。
29. **一个名字一处拼写**（步 4A，主会话裁 2026-10-01）：check 的 fail 条件名由 `CE.Score.Document` 从 `CE.Verdict.Faces.failConditions` 读出（只取名字），`degraded` 读 `CE.Verdict.degradedCondition`（降级应答 `failed` 的同一个词），钉两份相等的那一腿随之删掉；降级原因见第 25 条；站点 kind 名是产品定义表，进定义包的 `store` 表（`CE.Lang.Common.Graph.store` 的 `site_kinds`，`CE.Lang.siteKinds` 读，`CE.Graph.Sites` 引用；`tables/1` 因此多一个键、digest 变；目录不再带 `sites.kinds`），子仓 `it/document_catalogue.rs` 对冻结的 tables golden 钉它——4B 让 Rust 改由包读 `store::KINDS`，第二份拼写随之消失。join / deadcode 的 verdict 名、deadcode 的 why 句、顾问名与读法：核的新表从 4B 起是唯一所有者，4B 删除 Rust 的副本；本步不改。
30. **七族的空文档只作内部一致**（步 4A，主会话裁 2026-10-01，与任务书预期同）：目录给七族各留一份结构零值 `empty`，`DocumentProps4` 第一腿（空文档 = 目录）只作核内一致；Rust 对这七族永不印 `empty`——核不可达 = 具名拒绝、退 2，与今天相同。
31. **判决给名字的地方，目录带名字表**（步 4B，按原则自答）：check 的判决应答把成立的 fail 条件写成名字（`failed`），四族的降级原因也是名字（`reason`）；文档请求只送整数，所以 Rust 要一张名字 → 码的表。这张表不在 Rust 留（第 21 条）——`CE.Score.Document` 的目录项带 `failed`（`failConditions` 的次序再加 `degraded`）与 `reasons`，`CE.Join.Document` / `CE.Graph.Document` / `CE.Graph.Screen` 的带 `reasons`（`coreReasons`），Rust 按名字在包里的位置送码，包里没有的名字按名拒；`DocumentProps4` 的目录腿钉这两张表等于核自己的常量。备选「请求里送名字」违反整数过线（§2），「Rust 留一张常量表」是第二份拼写。
32. **structure 的 S6 汇总只读判决**（步 4B）：`--deep` 的死单元按目录汇总只要死行的结点号（`deadcode::judged`），不要任何名字，所以它不问 deadcode 文档——问了就是为了把名字读回来再按路径查目录，多一次往返、多一份依赖。
33. **图屏的金样冻在旧码上**（步 4B，任务书 §3）：两棵子仓夹具树（一个带环与孤儿的 crate、一组按节互链的文档与资产）在 b3443723 的 release 与它自己的核上答的 `faces::graph_screen` 写进 `it/golden/graph_screen.b3443723.ndjson`，文件名记来源；腿 `graph_screen_frozen` 只读不写——没有 bless 通道，要重答只能检出名字里那份代码。
34. **单元测试的去向**（步 4B，一条不无声删）：`unit/graph/canvas.rs` 的 `file_cycles_take_the_core_report_and_restrict_it_to_files` 拆两半——Rust 半（环成员展平、缺键拒绝、节归其文件、包无文件行）进新腿 `graph_rows_name_the_file_and_cycles_flatten`，核半（只含节的 SCC 不计、成员出界拒绝）进 `sections_collapse_packages_drop_and_cycles_count`（改由真核装配）与 `DocumentProps4` 的图屏腿；`unit/graph/deadcode.rs` 的 `every_liveness_reason_has_both_words` 改为 `every_liveness_reason_has_its_chinese`（英文半是核的 `whyCodes`，由 `DocumentProps4` 与 golden 钉）、`reported_rows_and_fail_bit_consume_and_skew_refuses` 改钉判决的码；`unit/graph/deadcode/advisory.rs` 改钉 `Advised`；`unit/graph/store_tests.rs` 读包；`unit/mention/tests.rs` 的 `caps_and_face_identity_are_stated` 改钉请求形（十九个事实、每语言一行，schema id 是核的，由 `it/document_catalogue.rs` 钉）；新 `unit/graph/deadcode/document.rs` 两腿（请求即判决、字符串留在本侧；包里没有的降级原因按名拒）。`unit/score` / `unit/structure` / `unit/join` 无一条读被删的函数，未动。
35. **控制台的形：一次一种语言，句子不进文档**（步 5C）：§6.2 原稿的 `{"$t": key, "args": [...]}` 与 `lines: {en, zh}` 两份不做——文档的字段集是今天的报告 JSON，加句子键会改变三面都在读的文档；改为请求带 `lang`、应答带那一种语言的 `lines` 与 `exit`，另一种语言再问一次（Rust 一次只印一种）。每行的引用就是文档里那套类别化引用，Rust 的绑定器原样复用。
36. **控制台要而文档不要的整数作测量侧事实**（步 5C）：arch 的 `widths`（目录路径的字节数与字符数——Rust `{:w$}` 按字节量宽、按字符补齐，核量不了字符串）、flow 的 `check` / `deny`（`--check` 与 `[flow] guard` 档）、merge 的 `only`（`--group` 号加一，0 = 全部）、check 的 `roast`、deadcode 与图屏的 `files`（`--entry` 提示要的文件数）与 `check`；merge 片段截断改为新引用类 `clipped [k, 起, 止, 40]`。这几处都只进 `lines`，文档逐字节不变（既有 golden 的 `document` 只有 arch 的 `counts.rows` 因多一张表而变）。它们在请求里可缺省（陈述行 `optional`，变基到 cacc2741 时补）：3B / 4B 落地的 Rust 发这六族的文档请求却还不送它们，缺省读作 0 / 空表、在场照陈述校验，文档两种情形逐字节同（`DocumentProps5` 第九腿）；Rust 半让面送齐之后，这几行 `optional` 可以删。
37. **否决位只给「判了且不通过」**（步 5C）：`exit.fail` = rules 判了且有违规、flow `--check` 且 deny 档且有判决发现、check 的 `fail`、deadcode `--check` 且有死文件或降级、审计的拦截（git 收得到、提交说明读得到、墓碑腿 deny 档且核答 over 且度量完整，或去重腿 deny 档且失败）；其余族恒 false。退出码 2（降级 / 无核 / 程序错误）仍由 Rust 读自己的拒绝，不经 `exit`。
38. **守卫的规则码与围栏**（步 5C）：`say` 行 `[rule, a..f]`，rule 0 重复 / 1 硬预算 / 2 分级区 / 3 墓碑 / 4 新增死代码 / 5 ce.toml 不可读，六种；围栏是 1 / 3 两条的尾句（1 配置偏离基线、2 基线不可读），不另占规则码——任务书写的「0..6、七句」是把两句尾句数成了规则。`guard` / `audit` 的 `document` 为 `{}`、陈述表 schema 为空串，不进 `tables/1` 目录，所以 `tablesDigest` 不动。
39. **5A 十族只进 `lines` 的请求事实与引用类**（步 5A，落码者按原则自答、待主会话核）：scan 的 `rows` 范围（= files + 6 × fns，`levels` 行号的宇宙）与 `levels` 第三列（该行的类：0 全局 / k = 第 k 个声明类，找它的 override）；dedup 的 `check` / `budget` / `fail` 事实（`--check`、预算、越线——Rust 照旧算，核只读位）；docdup 的 `check`；erase 的 `check`、`apply`（印不印 apply 那一行，0 / 1）与 `applied`（`--apply` 擦掉的行数本身，0 也是个数；`apply` 为 0 时 `applied` 非零按名拒）、引用类 `provenance` / `diff`（一组同文件可擦行的首末下标，Rust 按行取 diff 正文）；erase-trail 的引用类 `hash` / `plan` / `unreadable`；churn 的 `days`；trend 的引用类 `short` / `sha` / `reason`；similar 的引用类 `label`（查询的显示名，不带整数）。文档逐字节不受这些键影响。
40. **一个名字一处拼写（5A）**：erase 的类名与 reason 名由 `CE.Erase.Cost`（`classNames` / `reasonNames`）持有，计划与轨迹两份文档、目录同读；scan 的规则名 `CE.Scan.Cost.ruleNames`、fail 条件名 `CE.Scan.conditionNames`（读 `conditions` 自己的名字）；docdup 段 kind 名读定义包 `docdup.kind_names`（`CE.Lang.segmentKinds`）；dedup 的 `kgram` / `window` 由 `CE.Dedup.Cost` 以 `dedupKgram` / `dedupWindow` 持有——叫这两个名字是因为 `docs_consts` 按裸名找唯一源常量，`kgram` / `window` 已是 Rust `Params` 的芯片。
41. **5A 的否决位**（补第 37 条「其余族恒 false」）：scan = 有 fail 条件（`failed` 非空）；dedup = `--check` 且越预算；docdup = `--check` 且有重复；erase = `--check` 且有可擦行；erase-trail = 有不可读行；trend = 判 fail 或有失败点（stderr 只出一句：判了下行先说下行，否则说失败点）；clone / clone-units / churn / similar 恒 false。dedup 的预算建议句（低于预算）走 stdout、越线句走 stderr，`--format json` 时也在 `lines` 里，Rust 半要照 deadcode 的先例在 json 面也印 stderr 行。
42. **核算而不引用的三处**（步 5A）：erase-trail 记录的 UTC 时间戳由核按 Hinnant `civil_from_days` 拼（毫秒整数过线、字符串在核），不立 `civil` 引用类；docdup 文档的 `a` / `b` 是 `seg [s]` 引用、kind 名由 Rust 读定义包拼进——`lines` 里核直接写 kind 名；erase 的 diff 上下文行数 3 进目录、不进文档（文档要与今天逐字节同）。轨迹记录的类按码送（`classNames` 的下标），今天的「类名漂移」在码表里表达不了——Rust 半把类码读不出来的记录送作一条不可读行，它的 `why` 写出读到的那个类字符串，信息不丢。
43. **文档的键序与排版**（步 5A，主会话裁 2026-10-01）：每族文档的键一律排序（Aeson 与 `serde_json` 的默认），Rust 不留键序表——今天 `ce scan --format json` 与 `ce dedup --format json` 由结构体按字段序印的键序随切换变为排序，按第 23 条「任何代码不依赖键序」不算行为变化。排版不变：scan 与 dedup 仍按今天的多行缩进印，其余各族仍是今天的单行；这一选择是目录事实 `document.<family>.pretty`（scan / dedup 为 true，其余为 false，`tablesDigest` 随之挪动），Rust 半从加载的定义包读它。切换门对 scan / dedup 的 `--format json` 解析成值再比，其余各面逐字节比。
44. **Lines 模块拿到 `say` 与文档，不再自己取**（步 5A，落码者按原则自答、待主会话核）：5A 十族照 5C 的形写完后，查重门在主根点名二十四块新克隆，其中十块是 Lines 模块头（`import` 自己的文档模块与目录、`say = phrase T.catalogue lang`、`doc = dfAssemble X.doc req`）、三块是 `CE.Document` / `CE.Document.Fifth` / `DocumentProps5` 的导入串。根因是每个 Lines 模块都在重做族已经有的两件事，于是改 `spoken`（变基到 cacc2741 时它与 `Say` 从 `CE.Document.Contract` 挪进 `CE.Document.Read`——契约加了 `optional` 行后 Contract 到 294 行，过核的 290 行墙）：它收族的目录，`dfLines` / `dfExit` 把该语言的 `say` 与族装好的文档（`dfAssemble` 一次）递进去，Lines 的签名成 `Say -> Value -> DocReq -> [Line]`、否决成 `Value -> DocReq -> Bool`；5C 的十个 Lines 模块与守卫 / 审计同改，`CE.Document.Fifth` 并回 `CE.Document`，`DocumentProps5` 的目录覆盖读 `dfText`。其余新块各按其形消：clone 与 docdup 的信封收成 `CE.Document.Envelope`（一行文本），erase-trail 并进 `CE.Erase.Document`（任务书「第二个 DocFamily 放同一模块」），schema id 只拼一次，dedup 的事实行走 `judgedFacts`，测试的十七条拒绝与腿名改文本表、四份种子生成器共用 `seededBy`。主根预算未动（落码时 50，变基后 41），行集与基底相同；判决、文档与 `lines` 逐字节不变（`fixture_contract::regen` 空跑无一对移动）。
45. **`Answer`：一次问答三样东西**（步 5 R0，裁定 R1–R3）：`document::assemble` 返回 `Answer { document, lines, fail }`——每个面的请求都带 `lang`（`document::lines::lang()`，读控制台已定的那一种语言），应答的 `lines` 先于文档绑定，引用走同一个 `Resolve`，缺 `lines` / `exit` 的应答按名拒（7.10.0 之前的核）。`cli/src/document/lines.rs` 与它的单元测试自车道 B2（3a7c7046 / 子仓 bc6c7cb7）逐字节拷入（`cmp` 相同），B2 的守卫 / 审计 / daemon 改动不在 R0。既有面（arch / flow / merge / query / check / structure / join / deadcode / mentions / sites / 图屏）只把 `assemble` 的结果改读 `.document`，打印照旧。
46. **merge 的 `Resolve` 补 `clipped`**（步 5 R0）：`lines` 对每个面都绑定，核写进 merge `lines` 的片段截断引用 `clipped [k, 起, 止, 40]`（第 36 条）在 Rust 侧若无解析者即按名拒——所以 merge 面的 `Resolve` 多一类，按 `merge/console.rs::clip` 原样截断；merge 的打印仍是 Rust 自己的，切换在 Rust 半。
47. **打印器读目录，退出码读否决位**（步 5 R0，裁定 R4 / R5）：`document::emit(family, answer, json)`——控制台面按 stream 印全部 `lines`；`--format json` 先按目录的 `document.<family>.pretty` 印文档（缺这一项 = 7.10.0 之前的核，按名拒），再只印 stream 1 的行。切过来的面退出码 = `Answer.fail`（0 / 1），2 仍是测量侧自己的拒绝（无核、核拒、绑定失败）。`pretty` 与 churn 的 `cochangeFileCap` 两个目录事实不经 `leaked!` 宏（宏内的 serde 字段属性编译不过）：`tables::pack::DocCatalogue` 手写一层，按值读出这两项、其余照旧泄漏；包里没有 `cochangeFileCap` 即加载失败。
48. **churn 试点端到端**（步 5 R0）：`churn::answer` 送 `ranges` paths / submodules、`facts` days / commits / appended / rewrote / surviving / skipped、`rows` cochange `[a, b, n]`，核装文档与 `lines`；Rust 的 `report_json` / `print_console` / `SCHEMA` / `COCHANGE_FILE_CAP` 删除，配对上限改读目录（`tables::get().document.cochange_file_cap`），schema id 只在 `CE.Churn.Document.schemaId` 拼一次（facts 的 `report:churn` 改绑核）。三面同读一条路：CLI `ce churn`、MCP `churn`、GUI `churn_report` 都走 `faces::churn(root, core, days)`——三面从此都要核，无核按名拒、退 2（今天 churn 不问核）。
49. **R0 的切换门**（步 5 R0）：旧臂 = R0 父提交的 release 与它的核（Rust 与 cacc2741 只差 `corelink.rs` 的 PROTO 一行，churn 仍由 Rust 装配打印），新臂 = R0 的 release 与它的核；十个 crosscheck 语料各提交成一个仓、自仓一份带历史的拷贝，面 = `churn --days 14` / `--days 400` 控制台与 `--format json`、en 与 zh，两臂逐字节比；另一腿给新旧两臂各配一个只答握手、不答 `document/1` 的桩核——旧臂照印（不问核），新臂具名拒、退 2，这是第 48 条说的行为变化。读数见提交说明。
50. **R0 的三块新克隆按所有者消掉**（步 5 R0）：churn 的请求照 join 的形写，查重门在主根点名三块（44 > 41）——两块是路径表（churn 与 join 各一份「每个路径一次、按首见次序」的 `Paths`），一块是 MCP 的两个窗口面（`churn` / `join` 各一个三行适配器）。路径表收成 `document::Paths`（`cli/src/document/paths.rs`，join 与 churn 同读），适配器照 `plain!` 的先例收成宏 `windowed!`；主根预算 41 未动，行集与基底相同（只有 `faces.rs` 一块随注释下移一行）。
51. **scan 与 dedup 切到核**（步 5 车道 A）：`scan::document::answer` 送 files / fns / rows 三个范围、`files` 行 `[f, 总行, 注释行, 语言码]`、`fns` 行（每个函数的十个整数）、`levels`（每行非零的等级与它的类）、`grades` / `overrides` / `failed`，引用类 `path` / `fn`；`dedup::report::answer` 送路径表与 groups 两个范围、块 / 组 / 成员三张表、走查的计数器与两个报告阈值，`--check` 时再送 `check` / `budget` / `fail`（`budget::check` 返回 `Gate`，预算与越线位照旧由 verdict/1 判，句子改由核出）。Rust 的 `scan::report` 只留 R10 镜像（`evaluate` / `RULES` / `Finding` 只喂漂移 ensure），`SCHEMA` / `Report` / `Summary` / `print_console` / `sarif_string` 删除，fail 条件名的线上列表 `CONDITIONS` 留在 `scan/document.rs`（它是线上词表的检查，不是展示）；dedup 的 `SCHEMA_ID` / `Serialize` 与两句 ratchet 打印删除。SARIF 两面都是绑定后文档的投影（`scan::document::sarif`、`dedup::report::sarif`）。GUI 与 daemon 的签名不动（车道不碰 `gui/**` / `daemon/**`）：`faces::dedup` 与 daemon 的 `dedup` 应答经 `dedup::report_json` 用本进程的核（`tables::core_flag()`）装文档。
52. **clone、clone-units 与 docdup 切到核**（步 5 车道 A）：`t3::answer` 送单元宇宙的大小、十四个计数器与**每一条**判过的行 `[a, b, ted, n1, n2, 判决位]`（缓存回放的在内，`Judged` 多一份 `judged`），引用类 `unit` 由 Rust 拼成 `path:key#nth`；`--units` 由 `t3::units_answer` 送路径表与 `[u, f, nth, nodes]`，引用类 `path` / `key`，`faces::clone_units(root, core)` 改要核（MCP 同改）；`docdup::judge::answer` 送路径表、每个活段 `[s, f, 起, 止, kind 码]`、每条判过的行 `[a, b, inter, union, 逐字长, 判决位]` 与计数器、`check`，引用类 `seg` 由 Rust 按定义包的 kind 名拼成 `path:起-止 kind`。`rows_of` 的三元组签名不动（它的读者 score 与 query 在车道之外，erase 照读）。`cli/src/report.rs` 的 `Pair` / `Report` / `emit` / `envelope` 随最后一个读者删除；两族的 `print`、`SCHEMA_ID`、`unitcache::UNITS_SCHEMA_ID` 删除，核的信封文本改读具名常量 `schemaId`（clone / docdup），facts 的 `report:clone` / `report:clone-units` / `report:docdup` 改绑核（`unitsSchema`）。命令体收成一条 `main_cmds::answered`（churn 同读）。
53. **车道 A 第一批的行为变化**：（1）这五族从此要核——核不答 `document/1` 时具名拒、退 2（此前 scan / dedup / clone / docdup 的报告由 Rust 印，只问各自的判决族）；（2）`ce dedup --check --format json` 低于预算时不再在文档后印那句 stdout 建议（裁定 R3：json 面 stdout 只有文档与 stream 1 的行），控制台面照旧；（3）`ce scan --format json` 与 `ce dedup --format json` 的键改为排序（第 43 条），值逐个相同——`contracts/fixtures/scan-report/report.golden.json` 由真管线重生成、dedup 的 golden 不动。
54. **车道 A 第一批的切换门**：旧臂 = 7d69cf27 的 release 与它的核，新臂 = 本树的 release 与它的核；十个 crosscheck 语料各提交成一个仓、自仓一份带历史的拷贝，每臂各一份拷贝、逐语料背靠背；面 = scan / dedup / clone / clone --units / docdup 的控制台 en 与 zh、`--format json`（scan / dedup 解析成值比、带尾行的 `dedup --check --format json` 先比值再逐字节比尾巴，其余逐字节比）、scan / dedup 的 `--format sarif`、dedup / docdup 的 `--check`（退出码在比较之内）；另配两种桩核（转发真核、只从握手里藏掉一个能力）：藏掉判决族时两臂同拒，藏掉 `document/1` 时旧臂照印、新臂具名拒——即第 53 条（1）。自仓拷贝的预算恰在线上（42 / 42），低于与越过预算两腿另在改了预算的拷贝上跑。读数见提交说明。
55. **文档在判决那条核链上问，新克隆按所有者消掉**（步 5 车道 A）：第一版每个面为文档另起一个核，自仓 `ce scan` 的 ABAB 中位多 835 ms；scan / clone / docdup 改为把判决用过的那条链交给 `document::assemble_over`（`scan::wire::judge`、`t3::judge` 与 docdup 的 `judged_over` 把链原样交回，`rows_of` / `judge_index` / `settle` 的签名不动），dedup 不判时没有链可复用。查重门在主根点名六块新克隆（44 > 41）、子仓一块（92 > 91）：两份 SARIF 投影收成 `sarif::projected` + `num` / `text`，四个面的「下标 → 字符串」解析器收成 `document::Lists`，clone 与 docdup 的计数器表改走 `Request::counters`（一行名字、一列值），核里 scan / dedup / clone / docdup 的 `schemaId` 挪到模块末尾（紧挨 `doc` 与 `statement` 时两两成块），`it/docdup_text.rs` 改为一次比三元组。主根 38 块（预算 41 → 38 具名下调、`ce.toml` 入账，三个成员在子仓 `it/baseline_ledgers.rs`（今 `it/baseline_retired.rs`）的 RETIRED 按名退役；消掉的三块是 `config/thresholds.rs` ↔ `dedup/t3/mod.rs`、`dedup/t3/mod.rs` ↔ `docdup/judge/mod.rs`（两族删掉的序列化结构体与报告形）和 `faces.rs` 自身两个信封面），子仓 91 块、行集与基底相同。
56. **erase 与 erase-trail 切到核**（步 5 车道 A 第二批）：`erase::document::answer` 送路径表、计划里每一行 `[类码, 路径, 有跨度, 起, 止, 未解析点位, 内容哈希, 可擦位, reason 码, 1]`（只送闭包留下的行——计划本就只存它们，与送全部候选、`kept` 位各自标出的文档逐字节同）、out-of-class 计数 `[kind 码, 数]` 与 `check` / `apply` / `applied` 三个事实，引用类 `path` / `provenance` / `diff`；diff 仍是这一侧的度量（`render::file_diffs`），在动手之前逐文件渲染、先验内容哈希，核按「一个文件一条引用」放进 `lines`。`--apply` 的顺序照旧：`--check` 有可擦行即否决、不擦；擦完以 `applied` 的数装文档，擦败先印计划（`apply` 为 0）再按名退 2。`ce erase --log` 由 `trail_answer` 送每条记录 `[r, 毫秒戳, 类码, 路径, 有跨度, 起, 止]` 与每条读不出的行 `[k, 行号]`，时间戳由核拼；记录的类不在 `CLASS_NAMES` 里时读者把它算作读不出的一行，`why` 写「class "…" is not an erase class」（第 42 条）。删除：`render::print` / `out_of_class_line` / `reason_detail` / `span_str`（「见对应家族命令」那句随之退役——核的目录里每个 kind 都有命令），`log::print` / `report_json` / `REPORT_SCHEMA` / `utc_stamp` / `civil` 与它们的单元测试，`main_erase` 的 `--check` 拒绝句与 `--apply` 那句。`render::report_json` 只为 GUI 的擦除预览留着（`gui/src-tauri/src/commands.rs` 要一个 `Value` 且不带核路径，GUI 不在本车道），MCP 与 CLI 读核的文档。核：`CE.Erase.Document` 的两个 schema id 成具名常量 `schemaId` / `trailSchemaId`，facts 的 `report:erase-trail` 改绑核。
57. **trend 与 similar 切到核**（步 5 车道 A 第二批）：`trend::document` 送每个点 `[i, 戳, 分, 刻度, 轴码, 值…]`、`window` / `pending` / `fail`、trend/2 应答的 `slope` / `verdict`（各是零或一行的表——缺一张表核即拒）/ `cliff` / `declineRun` / `knobs`，引用类 `commit` / `short` / `sha` / `reason`；`main_judge` 的否决闭包删除（否决位在核：判 fail 或有拒绝测量的提交）。`similar::document` 送每个候选 `[座位, nth, 分, N P C D S L, 形状位, 联想位, 角色（2 = 未判）]`、`terms` / `widen` / `similarRev`，未判时以 `degraded` 指向原因文本，引用类 `at` / `key` / `label`（零个整数）/ `why`。删除：`trend::report` 的 `SCHEMA_ID` / `report_json` / `print` / 控制台三函数、`judge::verdict_str` / `judgment_json`、`similar::face` 的 `SCHEMA_ID` / `report_json` / `console` 与它们的单元测试。核：`CE.Trend.Document` 与 `CE.Similar.Document` 的 `schemaId` 成具名常量，facts 的 `report:trend`（原为刮取层，`SCRAPED` 18 → 17）与 `report:similar` 改绑核。
58. **车道 A 第二批的行为变化**：（1）erase / trend / similar 从此要核答 `document/1`，否则具名拒、退 2；`ce erase --log` 此前不要核，现在要（文档与行由核给出）。（2）`ce erase --apply --format json` 不再在文档后印「erase applied」那句（它是 stream 0；裁定 R3：json 面 stdout 只有文档与 stream 1 的行），控制台面照旧。（3）轨迹里类不在表内的记录从「读得出」变为「读不出」、退 1（第 42 条）。（4）erase、trend 与 similar 把判决用过的核链交给文档（`erase::planned` / `wire::judge` 交回 `Held`，`trend::judged` 交回 trend/2 的链，similar 的 `Judge::held`）；erase-trail 没有判决，另起一个核。
59. **车道 A 第二批的切换门**：同第 54 条的两臂与语料，另加一棵 `erase_e2e` 夹具树（三条可擦行：孤儿、整段文档、私有死孪生）与每个语料一条四行轨迹（三类各一条、一条非 JSON）；面 = erase 的控制台 en / zh、`--format json`、`--check`（en / zh / json）、`--log`（en / zh / json）、`--apply`（控制台与 json）与擦后的 `--log`，trend `--commits 5` 的 en / zh / json，similar `--at` 每个语料两个座位（一个带 `--widen`）的 en / zh / json，以及每族两条中继腿（藏判决能力 / 藏 `document/1`）。读数：erase + trend 259 行 218 同；41 条不同全归因——12 条 `--apply --format json`（上条（2））、12 条 erase 藏 `document/1`（轨迹现在要它）、11 条 trend 藏 `document/1`、6 条擦后 `--log`（只差擦除那一刻的墙钟时间戳，抹掉时间戳后 6/6 逐字节同）；夹具树与 python 语料的 `--apply` 两臂逐字节同（diff、两条建议行、三行擦除）。similar 77 行 67 同，10 条不同全是藏 `document/1`（html 语料没有代码单元，两臂同样按名拒）。
60. **第二批的新克隆按所有者消掉**（步 5 车道 A）：查重门在主根点名三块（41 > 38）。（1）`main_judge` 的 `trend_cmd` 与 `docdup_cmd`（第一批已与 `clone_cmd` 同形）、`main_similar` 的命令体是同一段「解出根与库、问核、按形印、按否决退」——收成 `JudgeArgs::answered`，三个命令体各剩一个闭包；（2）erase 计划的字符串表与 `flow_report` 的 `Names` 拼出同一个按类分派的 `resolve`——`PlanStrings` 改为持一个 `document::Lists`（路径与出处），自己只答 `diff`；（3）trend 的 `Row` 去掉只为旧 JSON 面而派生的 `Serialize` 后与 `dedup/groups.rs` 的 `Member` 同韵、去掉 `Clone` 后又与 `dedup/probe.rs` 的 `Match` 同韵（都是「导入块之后一个五字段的结构体」）——`Row` 挪到 `Judgment` 之后，`Clone` 本就无人用。主根 38 块（预算 38 未动）、子仓 91 块，行集与第一批落地时相同。
61. **守卫与审计经核说话（步 5 Rust 半第一部分）**：PreToolUse 的每条规则留下一个 `Said`（规则码、数字、字符串），钩子先定档位，档位要出一行时才把全部规则拼成一份 `guard` 请求（`say` 行 + 引用）经 daemon **2.3.0** 加性的 `document{body}` 送到 daemon 持有的核链，按自己的字符串绑回 `lines`、以空格接成那一句（`cli/src/guard/speech.rs`）；observe 档不出行也就不问核。Stop / precommit / commitmsg 在审计自己的核链（`verdict::open`）上问 `audit`（`cli/src/audit/speech.rs`），印它的行、按 `exit.fail` 拦或退 1。`guard/say.rs` 与审计的六个句子函数删掉；`Verdict.shown` 与墓碑腿的 `shown` 改存块与站点本身（路径成引用），墓碑腿的「度量不完整」由一句话改成两个计数 `unread` / `bounded`。daemon 的应答变体照同族旧例写作 `document_report`（主会话裁 2026-10-02：2.3.0 未发布，与 `tombstone_report` / `flow_report` 同形；任务书原写 `document`），golden 一对。
62. **核说不出时的兜底**（步 5 Rust 半第一部分，主会话裁 2026-10-02）：钩子绑不出句子（无核、核无 `document/1`、降级应答、引用解不出）时决定照本地规则不变，句子换成一句英文。守卫是 `ce: rule {codes} fired; the core could not phrase the reason: {why}`，warn / ask / deny 各档同一拼写：句子只点名触发的规则，不用任何宣称决定的动词——决定由钩子自己的 allow / ask / deny 通道表达，任务书原句的「write denied」在 warn / ask 档是假话。审计照同一原则：本地规则要拦时是 `ce {face}: rule {codes} fired; the core could not phrase the verdict: {why}`，不拦时是 `ce {face}: the core could not phrase the verdict: {why}`，挂在子模块前缀后、走该面原来的流，拦与否由退出码与 Stop 的 block 通道表达。代价具名、照记不改：今天无核时 precommit 仍印本地的暂存摘要，切换后印兜底句；Stop 只在本地规则要拦时才问核（「信息不付 spawn」不变）；precommit / commitmsg 在空改动集上也起一次核（今天不起），实测中位 +106.4 ms / +111.0 ms，见 `docs/PERF-BUDGET.md`「v2.32 步 5 Rust 半第一部分 precommit / commitmsg 空改动集 A/B」一节。
63. **GUI 的 kind 标签（R8）**（主会话裁 2026-10-02，落在第二部分）：kind 的两种显示标签进目录、建在主干 3D 的 `kindTable` 之上——`document.flow.kinds` 的每行在 `[name, advisory]` 之后多中英两个显示标签；GUI 经既有的目录读者（`tables/pack.rs` 的 `DocCatalogue`）读它们，`gui/ui/i18n.js` 删掉自己的 flow kind 表；`ce flow --format json` 的字节不动（标签只在目录里、不进文档）。落在第二部分：`CE.Flow.Document.kindTable` 的行成 `(名, advisory, en, zh)`、`kinds` 由它取名，`CE.Guard.Document` 改读 `Flow.kinds`；Rust 的 `tables::KindRow` 按序读四列，`flow_report::kinds()` 是唯一读者；GUI 经新的 Tauri 命令 `flow_kinds`（`gui/src-tauri/src/commands_flow.rs`，在任务里加载定义包后答 `[{name, en, zh}]`）取标签——任务书写「既有的目录命令」，GUI 侧没有这样的命令，目录今天只在 Rust 里读；`hub_flow.js` 在第一份文档到后问一次、按 `ceLang` 选标签，种类芯片与发现表的种类列都读它；parity 表 flow 行认领 `flow_kinds`。`tablesDigest` 随之挪动，tables golden 与 hello-ok 由 `fixture_contract::regen` 重答。
64. **十二族改印核的 `lines`（步 5 Rust 半第二部分）**：arch / flow / merge / query / rules / check / structure / join / deadcode / `graph --mentions` / `graph --sites` 的控制台每一行与退出码改读核（图屏没有控制台面，只补事实）。CLI 面一条路 `main_prelude::answered(name, family, json, answer, unjudged)`（车道 A 的 `main_cmds::answered` 是同一条路少一个 `unjudged`，变基后并成这一个：它的九族与 churn 读 `whole`；`name` 是命令、`family` 是文档，`ce erase --log` / `ce clone --units` 两处不同）：`document::emit` 先印，再按面自己的读法判「判决没发生」退 2（arch / flow / merge 读 `degraded`，query / rules 另读程序错误，merge 的 `--group` 越过最后一组按名拒、什么都不印），否则退 `fail`；两个图面走 `report::graph_face`（错误照旧以 `ce graph:` 开头、退 2）。请求补第 25 条的事实：arch `widths`、flow `check` / `deny`、merge `only`（两条路都送：降级路的签名多一个 `only`）、check `roast`、deadcode 与图屏 `files` / `check`（merge 的 `clipped` 在 R0）。删掉的打印器与只为它们存在的读者：`arch/{console,report}.rs`、`flow_report/{console,report}.rs`、`merge/{console,report}.rs`、`query/{console,report}.rs`、`score/report.rs`（`Report` / `Ratchet` / `Counts`）、`graph/deadcode/{report,why}.rs`（`conf_word`、`WHY_ZH`、`why_line`），structure / join 的 `console`、mentions 的 `Read` / `console` / `rates_console`、sites 的 `SitesRead` / `SiteRead` / `print_counts`、`report.rs` 的 `print_bound` / `print_read` / `colon_pairs`（`Bound` 只剩 `keep`）。`faces::flow` / `faces::merge` 送 `(false, false)` / `None`——机器面不印行，事实取 0。
65. **json 面的 stream 1（R3）逐面核过**：切换门十个语料 + 自仓的 `--format json` 面里，stderr 上有字的只有四种，新旧两臂字节相同——Rust 在印之前的拒绝（`arch --impact` 点名的文件不在测量里、`flow --kind` 的未知种类、`merge --group` 越过最后一组，退 2、stdout 无字）、`deadcode --check` 的死文件句（stream 1，`deadcode check: N dead file(s) — …`，退 1）、以及无核 / 核不答 `document/1` 的具名拒绝；query 的程序错误在文档里（`errors`），不在 stderr。没有哪一面在 json 时往 stdout 印过 stream 0 的句子，所以 R3「json 面只印 stream 1」在这十二族没有可见的行为变化。
66. **留下的读者结构**：join / structure / deadcode 的 `Report` 留作类型化读者——库的调用者与测试经 `report::read_bound` / `document::read` 读绑定后的文档（`baseline_bridge`、`mcp_precommit`、`structure_knobs`、`common::join_report`），它们问的是文档里的数，不是控制台；arch / flow / merge / query / check 的读者只服务控制台，随打印器删除，测试改读 `Value`（`eval_arch_self`、`structure_modularity`、`merge/face`）或在测试文件里带自己的小结构（`flow_report/face`、`query/face`）。
67. **删掉的一条单元腿**：`unit/graph/deadcode.rs` 的 `every_liveness_reason_has_its_chinese` 钉 Rust 的 `WHY_ZH` 每个原因都有中文；那张表与 `why_line` 随打印器删除，原因句的两语只在核的目录（`why_unref` / `why_unreach`），由核的文档电池与 golden 覆盖。
68. **第二部分的三块新克隆按所有者消掉**（主根查重门点名 43 > 41）：一块是本部分加的——`faces.rs` 的 flow / merge / arch 三个面各写一遍 `Ok(…?.document)`，收成 `faces::laid`，十一个读核文档的面同读；一块是第一部分变基后才相遇的——守卫句子的字符串表（`guard/speech.rs`）与 sites 文档的字符串表（`graph/mod.rs`）各手写一份「类 → 列表」的 `Resolve`，变基前收成宏 `document::listed!`；变基后车道 A 的 `document::Lists` 答的是同一个问题，宏删掉、两处在调用点建一份 `Lists`（一个问题一种写法）；一块在核——`DocumentProps` 照 `kindTable` 的行逐行重拼期望值，改按列写（`zip4`）。structure 控制台随删除带走一块既有的文件内克隆，主根在变基到 389c7750 之后读 37，预算 38 → 37 按名下调（ce.toml 台账），那个成员在子仓 RETIRED 按名退役——第四十五行让 `it/baseline_ledgers.rs` 撞 E01 的 300 行墙，RETIRED 照 `baseline_reanchored.rs` 的先例搬进自己的文件 `it/baseline_retired.rs`；子仓 91。
