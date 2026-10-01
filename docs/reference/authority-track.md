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
6. **口径类办法最后裁**：本轨做完后语言条估算 ≈ 37.4 %（§11）；到 40 % 的约 2.6 点用哪种办法（拆 GUI 子仓或再加一个新判决家族）在步 9 由用户裁（§13 第 1–2 条）。
7. **不留已知限制**（用户令 2026-09-26，分析轨 §1 第 6 条同）：凡想写成「边界」的，先问能不能按规范修对；修不对的在 §13 记立场与证据。

## 2. 不变量（一条不动）

- **整数过线**：这条不变量保护的是**被度量仓库的数据**——仓库里的名字、路径、源码文本永不送进核；它不限制**产品自己的常量**从核送回 Rust。`tables/1` 送的是语法结点 kind 名、标准库名、扩展名、产物目录名、协议词表，都是产品定义、没有一个字节来自用户仓库，方向是核 → Rust（§4）；文档骨架里凡来自仓库的字符串位仍是符号引用 `{"$": k}`，符号表只送**计数** `symbols: N`，符号内容留在 Rust（§5）。
- **判决在核**、**顾问永非判决**：本轨不新增判决，不改顾问的身份。
- **解析 / 索引 / 文件系统 / 进程 = Rust**（硬约束 1）：Rust 仍是定义的执行者；`GRAMMARS` 表（链接编译好的文法）与以代码写成的 kind 启发式（`dedup/tokens.rs`、`similar/bag.rs`）留在 Rust。
- **三面等价**。
- **每一步以「十语料十面旧 / 新二进制逐字节同」为门**。
- **不为占比写代码**——每一步都说得出它让架构更好的那句话。

## 3. wire 契约与版本

| 件 | proto | 请求 | 应答 | 缓存与降级 |
|---|---|---|---|---|
| ① `tables/1` | 7.7.0（加性） | 无参（信封之外多一个键按名拒） | `tables.result` 一次答整包：十六个顶层键按表族分（`languages` / `scan` / `flow` / `slot` / `sites` / `calls` / `fourclass` / `ladder` / `walk` / `outputs` / `docdup` / `keys` / `flags` / `tombstone` / `compdb` / `protocol`），每语言的表在表族键下按语言报告名分，另带 `digest`（规范字节的 fnv1a64，hello 回执的 `tablesDigest` 同数；规范字节 135,145 B） | Rust 第一次用到时要一次，按核 `(version, proto)` 入键落盘缓存 `.ce/tables-<ver>-<proto>.json`；握手已知版本 → 命中无往返；持核链的 daemon 内存缓存；缓存损坏 = 重取；无核 / 旧核无 `tables/1` = 具名拒绝，不留内嵌副本 |
| ② ③ `document` | 各族随自己的 minor（7.7.0 之后依次） | 多送文档需要而判决不需要的整数事实（行号、起止行、符号下标）与符号计数 `symbols: N` | 各族应答加性 `document` 键 = 与今天报告 JSON 同形的骨架（字符串位 `{"$": k}`，句子 `{"$t": key, "args": [...]}`）+ `lines: {en: [...], zh: [...]}`（控制台每一行） | 旧行表键先保留给仍读行表的面 |
| 退役 | 8.0.0（major） | — | 退役不再有读者的旧行表键 | 全部族切完后一次（§12 步 6） |

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

- 请求多送**文档需要而判决不需要的整数事实**（行号、起止行、符号下标；符号表只送计数 `symbols: N`，内容留在 Rust）。
- 核在应答里直接给出**文档骨架** = 与今天的报告 JSON 同形，凡字符串位一律是符号引用 `{"$": k}`。
- Rust 一个通用绑定器 `bind(skeleton, symbols)`（遍历 JSON，`{"$": k}` → `symbols[k]`）；各族的报告装配删除，只剩请求装配（已有）+ 绑定器（≈ 10 KB）。schema id、字段、排序、计数全由核决定——三面等价由构造保证。
- **切换规则**：每族一次 minor（应答加性 `document` 键；旧行表键先保留给仍读行表的面）；切换提交以「旧装配产物 == 新 `bind` 产物逐字节」为门（十语料 × 每族 × 三面），然后删旧装配；全部族切完后一次 major 8.0.0 退役不再有读者的旧键。
- **分组落地**：(e) 新五族 query / rules / flow / merge / arch 作样板（骨架最简单）→ (a) check / score / structure / join → (b) graph：deadcode / mentions / sites / canvas → (c) scan / dedup / clone / docdup / erase → (d) churn / trend / tombstone / similar / audit / update / health。

### 5.3 门

每族切换提交：旧装配产物 == 新 `bind` 产物逐字节（十语料 × 该族 × 三面）；十语料十面旧 / 新二进制字节同；核电池每族骨架腿（字段齐、符号下标在 `symbols` 内）；步 6 退役后 parity 门改读骨架。

### 5.4 估算

Rust −≈ 200 KB，Haskell +≈ 180 KB。

## 6. 件 ③：双语文本进核（与件 ② 同一次应答）

**那句话**：控制台文本只是文档的另一种渲染；渲染规则与消息目录是文档权威的一部分。

### 6.1 今天

各族 `console.rs` 各自渲染控制台文本；中文帮助与句子在 `main_lang.rs` 的 `ZH_TSV`；守卫的句子在 `guard/say.rs`；GUI 的文字在 `gui/ui/i18n.js`。

### 6.2 设计

- 核持有消息目录 `CE.Text`（key → en / zh 模板，槽位 `{0}`）；应答的 `document` 里所有句子是 `{"$t": key, "args": [...]}`，并另给 `lines: {en: [...], zh: [...]}`（控制台每一行，槽位里仍是符号引用）。
- Rust 按 `[ui] lang` 选语言 + 绑定 + 打印；`console.rs` 整族与 `main_lang.rs` 的 `ZH_TSV` 并入核目录。clap `--help` 文本留在 Rust，`zh_surface` 门照旧；GUI 的 `i18n.js` 不动。
- 守卫句（`guard/say.rs`）同理进目录，PreToolUse 腿经 daemon 的核链取句——daemon 已持核链，无额外往返。

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
- **到 40 % 的约 2.6 点**：按用户裁到时再定（步 9）——拆 GUI 子仓（+2.3 点）或再一个新判决家族（+1.8 点）。
- **步 7 实读**（本仓 `git ls-tree` 按上述口径重算，Rust 与其他未动）：Haskell 855,590 → 937,809 B（+82,219）/ Rust 2,534,369 / 其他 300,362 → Haskell 24.9 %（a0417628 为 23.2 %）。
- **记账**：每步收口写一行读数进 CHANGELOG 该步块；v2.31 的语言条读数改以本轨收口（步 8）时为准，那是 1.9.0 发版声明的读数。

## 12. 分步（每步自带门，落码顺序；工作树车道并行开发、按序落地）

| 步 | 内容 | 门 |
|---|---|---|
| 0 | 计划修正案与立项（2026-10-01）：本册 + 计划书 v2.32（横幅立项句、ADR-008 细则第八期、§6 T 轨 v2.32 行）+ CHANGELOG `[Unreleased]` 块 + cc-memory 重锁（硬约束 2） | docs 门全绿、基线具名重立 |
| 1 | 定义进核 A（2026-10-01 已交付）：核 `CE.Lang.*` 三十五个模块（`Toml` / `Table` 两个读者、`Spec` / `Spec.Flow` 两个模式、`Contract`、十三个语言模块与八个 `.Flow` 模块、`Common` 九模块）+ `CE.Lang` 汇总 + `CE.Tables`，转录自 a378e78c 的 Rust 表；`LangProps` 十一腿 + `tables/1` golden 两对（全包 / 多一个键被拒）+ proto 7.7.0（hello 加性 `tablesDigest`）；子仓 `it/tables_equivalence.rs` 三腿证核的包与 Rust 临时镜像 `cli/src/tables/native.rs` 逐键相等、三方同数、多键被拒 | `cabal test` 全绿、`tables/1` golden 一对、既有 golden 只动 proto |
| 2 | 定义进核 B（2026-10-01 已交付，§4.5）：Rust `tables.rs` + 缓存 + 消费者改读 + 定义文本删除（二十四个文件）；精度册两族（语言十一份、flow 十份）退役再在干净树上重生成，两个提交 | 十语料 + 自仓十三面旧 / 新二进制 143 对字节同、子仓表对钉版文法腿改读、无核 / 旧核 / 缺键具名拒绝 |
| 3 | 文档骨架样板：新五族（query / rules / flow / merge / arch）+ 绑定器 + 切换门 | 旧装配产物 == 新 `bind` 产物逐字节（十语料 × 五族 × 三面） |
| 4 | 骨架 (a)(b)：check / score / structure / join；graph 的 deadcode / mentions / sites / canvas | 同步 3 的切换门 |
| 5 | 骨架 (c)(d) + `CE.Text` 双语目录 + 守卫句：scan / dedup / clone / docdup / erase；churn / trend / tombstone / similar / audit / update / health | 同步 3 的切换门 + `lines` en / zh 逐字节同、`zh_surface` 绿 |
| 6 | 删旧 face / console / `ZH_TSV`、major 8.0.0 退役旧键、parity 门改读骨架 | 十语料十面字节同、`face_parity` 改读骨架后绿 |
| 7 | 老家族参考实现与电池（六族：Verdict / Score、Structure、Erase、Trend、Tombstone、Similar；可与 3–6 并行车道）——**已交付**（§7.5） | `cabal test` 全绿、每族等价 200/200 |
| 8 | 全量文档 + 语言条读数记账（§11 口径；v2.31 步 10 的文档面一并） | docs / site / facts 门全绿、引文重签 |
| 9 | 口径类裁定（用户：拆 GUI 或加家族） | AskUserQuestion |
| 10 | 发版 1.9.0（v2.31 步 11 即此步） | RELEASE.md 链 |

依赖：0 先于一切；1 → 2；3 → 4 → 5 → 6；7 可与 3–6 并行车道开发；8 在 0–7 之后；9 → 10 最后。

## 13. 拍板记录（2026-10-01）

1. **用户令与三题裁定**（2026-10-01）：用户令「想办法减少rust比例或者增加haskell。任何办法。」→ AskUserQuestion 三题：① 真改代码选「报告文档与双语文本进核」+「十三语言的定义表进核」，并要「思考更加优雅的方法」（本册件 ① ② ③ 的形即那个回答：核持有陈述、Rust 只执行与绑定，而不是把 Rust 改写成 Haskell）；② 口径类办法做完再视结果定（步 9）；③ 1.9.0 推迟到所选办法做完再发（§1 第 5 条）。
2. **范围再裁**（2026-10-01，量化后 AskUserQuestion）：「两项 + 老家族参考实现（推荐）」——件 ① ② ③ 加老家族的参考第二实现与等价电池（件 ④；裁时按设计稿的十族估 ≈ 38.5 %，立项按实改为六族、≈ 37.4 %，见第 10 条）；最后的缺口到时再裁（拆 GUI 子仓 +2.3 点或再加一个新家族 +1.8 点）；「统一走查引擎」量下来只换 0.6 点，不做。
3. **提交署名**（用户令 2026-10-01，常设）：「不要加claude提交署名，署名只写我自己」——本轨起每个提交说明末尾不加任何署名行，作者只有 git 配置里的用户本人。
4. **不变量一条不动**（主会话按原则自答）：整数过线、判决在核、顾问永非判决、硬约束 1、三面等价、逐字节门、不为占比写代码（§2）——本轨换的是陈述的持有者，不是分工。
5. **整数过线靠符号引用保持**（主会话按原则自答）：文档骨架的字符串位一律 `{"$": k}`，符号表只送计数，内容留在 Rust；句子是 `{"$t": key, "args": [...]}`，槽位里仍是符号引用（备选「把路径 / 名字送进核让核拼字符串」违反 §5.9.2）。
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
