# Changelog

> 记录义务来自 DEVELOPMENT_PLAN.md §4.2：“每次默认档位变更在
> CHANGELOG 记录依据（FPR 数据）。”本册记两类：守卫**默认档位变更**
> （依据 = FPR 数据，每个版本条目首句声明有无）与每步落码的**功能 /
> 协议 / 记账变更**（v1.2.0 后的 [Unreleased] 块起，发版时并入版本条目，
> 源码克隆与 crates.io 包内即有全史）；GitHub Releases 留发布说明与分数
> 可比性声明（v1.2.0 及更早的功能面只在那里）。

## [Unreleased]

**无默认档位变更。** bench 入列规则加第三种情形：申报沿用（2026-09-19，用户裁「更新规则，如果核心部件没动，那该版本沿用先前的数值」）：

- **规则本身一个字不动**。它按路径判「算不算新程序」——改到 `cli/src` 或 `core/app` 就该有一行——而七项指标计时的**正是这两处**。任何窄到能放过 v1.7.4（判决请求生产侧的一个小函数）的机器判据，同时也会放过真正让 `ce check` 变慢的改动。把规则拧窄到不再看见这一类，等于让它以后也看不见。
- **加的是「欠一行时可以怎么诚实地不测」**：维护者在 `contracts/bench/bench.json` 的新表 `inherits` 里**申报**「本版沿用某个具名发布的数值」，并写下理由；七个面（README 双语、docs/BENCH.md、两首页、两 bench 页）把话说完。此前它们挂着半句「它在 tag 之后测量」——那句话从 tag 当天起就不再是实话。`NoRow` 第三支 `Inherited(from)` 与两语句子随之落地；schema `ce.bench/0.1.0` → `0.2.0`。判断是人做的，但它被记在读者能拿公开 diff 去核的地方——这比把规则悄悄拧到不再报警要诚实。
- **新门 `every_declared_inheritance_names_a_measured_release_and_says_why`**：申报的**形状**由机器核，四种不诚实各被点名——`dangling`（它指的那个发布没有行）、`self-serving`（申报的那个发布自己有行，即已经测过还说没测）、`mute`（理由短到没法拿 diff 去核）、`idle`（规则本来就没让它欠行，申报只会糊住到底是哪一种）。四处变异各自转红，还原后 `bench.json` 的 sha256 逐字节相等。`idle` 那条问的是 `release_without_a_row`——**每个渲染面用的同一个读法**，不是规则的第二份抄件，所以它漂不到页面说法之外去。
- **申报的读法收进一个类型**（`Inheritance` / `field` / `inheritances`），而不是在门里裸取三次字段。是查重棘轮点的名：那串连着三行的 `expect` 与 `eval_support/dedup.rs` 逐 token 同形（52 token），子仓 120 块 > 预算 119。**预算没动**，去掉重复回到 119——申报现在是个正式概念，本来就该有自己的形状。
- **顺手修掉 schema 升版自己带出来的一个缺陷**：`BENCH_SCHEMA` 一个常量在干两件事——既是**文件的形状**（`doc.schema`），又被盖进**每一行的 `harness` 字段**（是**谁量的这一行**）。把文件形状升到 0.2.0 之后，下一次真去量时新行会被盖上 `ce.bench/0.2.0`，而在册的 140 行全是 `ce.bench/0.1.0`——而 `harness` 恰恰是读者用来判断两行能不能相比的那个字段。**一次文档改动不该有能力宣布这些数字不再可比**。拆成两个常量（`BENCH_HARNESS` 停在 0.1.0，因为量的代码一个字没动），并把「在册每一行的 harness 等于 `BENCH_HARNESS`」钉成门：把任一个常量重新指向另一个（常量侧 / 盖章侧两种走法）都被它点名拒绝，两处变异各自转红、还原逐字节相等。ADR-006 具名重立（子仓，第二次）：`it/bench_support/mod.rs` 265 → 276 超容差（cap 275，长出来的正是「为什么这是两个概念」那段注释，**没有为了躲闸删注释**），`it/bench.rs` 253 → 263 容差内同定。
- **官网两首页不再把 winget 列为安装方式**（2026-09-20 用户裁「winget PR 以后一个版本只保留一个；更新后续版本时前一个版本 PR 还没过就撤掉，保持最新。在 PR 通过前，官网不标『可 winget 下载』」）：这个包**从没进过 winget**——winget-pkgs 里零个已合入清单，本机实测 `winget search skymanbp.CodeEraser` 找不到——而两个首页的安装芯片一直印着 `winget install skymanbp.CodeEraser`，**读者照着敲必然失败**。芯片改成只讲 Homebrew（那半句是真的：tap 里的公式指向 1.7.4 的资产）。README 双语不动，它本来就写着「须 winget-pkgs 合并之后」。**PR 侧按新规收敛到一个**：1.7.0 / 1.7.2 / 1.7.3 三个自 2026-09-06 起挂着等社区志愿者管理员（十项校验全过、CLA 已签，我方无一处要修）已撤下并写明去向，新开 microsoft/winget-pkgs#437832 只为当前发布 1.7.4（清单钉的安装包 sha256 与发布资产逐字相符）。**规则写进 `docs/RELEASE.md`** 而不是只做一次——这摞 PR 正是「规则只活在某一轮的脑子里」的产物；合并当天再把首页芯片加回去。
- **dependabot 三条车道停到手动**（用户裁「推荐。改成手动触发。」）：`schedule.interval` 没有 `manual` 这个值，所以车道照旧声明、`open-pull-requests-limit: 0` 把它们按在零——这正是官方文档给的「暂时停掉某个包管理器的版本更新」的办法。**安全更新是另一条通道，不受这个上限影响**，仓库照收。要升级某样东西时：把那条车道的上限抬起来、让它开 PR、再设回 0。
- 门：主 check 944 / dedup 55 / scan 88 warn 0 fail，子仓 983 / dedup 119 / scan 42 warn 0 fail，两仓棘轮 pass；lib 375、it 408 (12 ign)、`cabal test` PASS、clippy 与 fmt 清。ADR-006 具名重立（**子仓**）：`it/bench_support/render.rs` 200 → 264 超容差（cap 210）、`it/bench_render.rs` 296 → 358（cap 306），softLine 296 → 297，另三行容差内同定；该次写入同时记下树上早已存在、却从未被写进基线的条目（`check_sim_table.rs`、`unit/score/sim_table.rs`、`site_viewer.rs`、`site_contents.rs`、`site/camera.js` 与四十条单元行），都不是本轮的新活。

**无默认档位变更。** 语言扩展轨道 v2.30 立项（2026-09-24，用户三裁：立项 / HTML 升格为文档类判决语言 / 一版 1.8.0；其余九条按既定原则裁定，逐条记在设计册 §14）：

- **设计先于代码**：`docs/reference/language-expansion.md`（C / C++ / Lua / Java / R 走 Haskell 走过的全套——文法钉版、单元、CC 与 CoC、克隆指纹、引用图阶梯、可见性、提及规约；HTML 从纯尺寸臂升格为文档类判决语言；每个 tree-sitter 结点 kind 与字段名都实探于 0.27.0）与 `scripts/tsprobe`（钉版文法的 AST 探针 + 样本，本机复跑零 ERROR；Ruby 撤出后六套文法、十二个样本）由云端会话落在 6900c3f / 1e1bda8。其 CI 35945413754 三平台各只红一项——册 13 自仓普查行：新文件让 U 1034 → 1053（listed 1046 → 1065）。补救 84af1a4 把十四个样本用 `scripts/tsprobe/.gitignore` 的 `snippets/` 模式挡在两条走查之外（文件仍 `git add -f` 跟踪；公式门记 `pattern-ignored` 14，U = 1065 − 14 − 12 = 1039）并重钉普查行，CI 35947387600 全绿。
- **计划书 v2.30 修正案**：横幅句 + §6 T 轨八步（步 0 计划修正 → 步 1 骨架 → 步 2–5 C/C++ · Java · Lua/R · HTML 可并行 → 步 6 评估 → 步 7 文档 → 步 8 发版 1.8.0）；同日再裁两条（册 §14 第 13、14 条）：**Ruby 退出 v2.30**（tree-sitter-ruby 依赖撤回——两份 Cargo.lock 各 −1 包、NOTICE −1 行、`grammar_pins` 十三 → 十二、`[graph.search_roots]` 去掉 `ruby` 键、探针去掉文法臂与两个样本；语言码 19 只留位、D6 / D7 / D15 空号不复用；设计留在 git 历史 b7e78c7），**每个语言的精度考题先于其阶梯提交**（slice + sample → 独立代理盲判 GT → 阶梯与精度册；C/C++ 在步 6 补做）；设计册改由横幅链接保活（顶部临时 `ce:allow(deadcode)` 行删除）、§14 改为拍板记录、§15 交接段删除；cc-memory 重锁九步。本提交零判决代码改动：主 check 944 / dedup 55 / scan 88 warn 0 fail，子仓 983 / 119 / 42 warn 0 fail；ADR-006 具名重立（主仓）：`CHANGELOG.md` 691 → 696 超容差（cap 693，长出来的就是本块记账），另两行容差内同定。

**无默认档位变更。** 语言扩展 v2.30 步 1 骨架（2026-09-24；判决代码字节零变化，分数与 1.7.4 可比）：

- **六套文法钉版**：`cli/Cargo.toml` 加 tree-sitter-c 0.24.2 / cpp 0.23.4 / lua 0.5.0 / java 0.23.5 / r 1.3.0 / html 0.23.2（两份 Cargo.lock 各 +6 包、十二条 tree-sitter 包的版本与校验和逐条相等；NOTICE 再生 +6 行）。`it/hs_grammar_pin.rs` 扩为 `it/grammar_pins.rs`：十二套文法一张表，每套核 ABI 在 0.27 的 13–15 窗口内、样本解析零 ERROR 结点、根 kind 如表；Haskell 的 bind / conditional 两腿原样保留。首稿把语言构造子写成闭包，八行同形让子仓 dedup 121 > 119（两块都在这张表里，19–36 ↔ 37–54、19–42 ↔ 43–66），改存 crate 自己的 `LanguageFn` 值即消——一行四个记号，八行装不下两个 50 记号的窗口；dev-dependency `tree-sitter-language 0.1.8` 随之入表（tree-sitter 不再导出这个类型，版本就是每个文法 crate 已经解到的那一个）。
- **`Lang` 六行保留码 15–20**（C / Cpp / Lua / Java / Ruby / R；19 号 Ruby 只留位，Ruby 不在 v2.30）：无扩展名、scan_only、无文法——**不在本步翻位**，翻位随各语言自己的步，HTML 在步 5。理由记在设计册 §1：翻了位而没有文法的语言会以 Markdown 的 spec 进索引、留下指纹与单元行，内容哈希刷新永不重算它们；没有阶梯的判决文件在 deadcode 里成孤儿；README 的语言芯片会先于事实说话；逐语言 FPR 发布门要求每个语言能单独不入集。`judged_mask()` 仍是 `0x7F`，`count:langs` 七 / `count:grammars` 六不动；设计册 §13 行 1 与计划书 T 轨步 1 句同批就地改写。
- **新谓词 `Lang::fingerprints()`**（= 有文法 ∧ 非 HTML）接管四处指纹消费者：索引刷新的 token 流与 `has_tokens` 列、走查计数 `tokenized`、daemon 探针的空答、T3 单元事实的空返；两条 FPR 回放仪器的镜像同改。今日它与 `grammar().is_some()` 逐语言相等（HTML 尚无文法臂），故索引行零变化；册 01 的指纹句改引它。
- **wire 7.2.0（加性 minor）**：`scan.request` / `graph.request` 恒发 `judgedMask`（= `Lang::judged_mask()`）；核 `CE.Wire.judgedLang` 取代 `CE.Scan.Contract.namingShape` 与 `CE.Graph.Contract.unresRow` 各自写死的 `lang ≤ 6`（缺席 = `legacyJudged` 127，逐字节同旧；负值 / ≥ 2^63 按名拒）；应答回显（scan 未降级时、graph 恒），Rust 两侧无回显（「pre-7.2.0 core」）或值不等（漂移）按名拒。`CE.Scan` 随之撞核文件墙（291 行；子仓 `core_size_gate` 要求 tolerated(ceiling) ≤ 300，即 ≤ 290），请求边界校验拆出 `CE.Scan.Contract`（144 行，`CE.Graph.Contract` 先例；`Scan.hs` 281 → 167），每个行形改读 `CE.Wire.rowCheck`——搬出时写入门点名手写 `rowShape` 与 trend/2 的同韵（80 记号），改读其他家族已共用的骨架后那一行真消（dedup 55 → 54，预算按台账降到 54）；拒绝文本逐字不变，golden 回放零字节差。golden 十四文件经新核机器重生：既有 135 对只动 proto 字面（脚本逐对断言，非 proto 差异 0），新增 scan 三对（16 mask 内 lang 15 判并回显 / 17 缺席拒 / 18 负值拒）+ graph 两对（25 mask 内 lang 20 判并回显 / 26 缺席拒）；`hello-ok` 握手请求随 server 走 7.2.0；VERSIONING 7.2.0 条 + 两处请求形状 + §3 三元组（130 → 135 行）。电池：`ScanProps.maskRoad` 十腿 / `GraphWireProps.maskRoad` 六腿（K52 的拒绝谓词提升为顶层 `refusedGraph` 共用；`maskReq` 要显式签名——GHC-39999 where 块多态第四例），`cabal test` PASS；子仓 `unit/scan/wire_tests.rs` 钉 Rust 侧回显三态、`graph_export_surface` K16 改六键、`unit/scan/lang.rs` 钉 21 行表与保留行四性质。
- **对拍**：旧二进制（a6d9376 的 ce 1.7.4 + 核 7.1.0）与本树二进制在四份 crosscheck 语料副本上各跑十个报告面（scan / dedup / docdup / deadcode / clone / structure / erase / check 的 JSON 与 graph --sites / --mentions，各自 `.ce`），40/40 逐字节相同。
- 文档：架构图 IR 与四张 stack.svg 的 proto 字面 7.2.0（archify 重渲，docs 与 site 孪生逐字节同）；`count:golden_requests` 130 → 135。
- 门：主 check 944 / dedup 54（55 → 54）/ scan 88 warn 0 fail（deadcode 0 / docdup 0 / erase 0），子仓 983 / 119 / 42 warn 0 fail；lib 376 / it 409 (12 ign)、cabal PASS、clippy（cli 全目标、gui `--locked`）+ fmt 清。ADR-006 具名重立（主根）：`cli/src/scan/lang.rs` 148 → 186、`cli/src/scan/wire.rs` 240 → 253、`core/app/CE/Wire.hs` 251 → 283、`core/test/ScanProps.hs` 227 → 255、`core/test/GraphWireProps.hs` 199 → 229 超容差，`core/app/CE/Scan.hs` 281 → 167（拆出）与新实体 `core/app/CE/Scan/Contract.hs` 144 同批入列，其余容差内同定；（子仓）`it/graph_export_surface.rs` 246 → 257、`unit/scan/lang.rs` 23 → 48、`unit/scan/wire_tests.rs` 69 → 96 超容差，新文件 `it/grammar_pins.rs` 入列、`it/hs_grammar_pin.rs` 退役；`it/baseline_ledgers.rs` 297 → 180（7.0.0 的两份 REANCHORED 台账拆到新实体 `it/baseline_reanchored.rs` 133——第十五条 RETIRED 行让它过 300 行墙）。离散集：Scan.hs ↔ Trend.hs 那对 `rowShape` 成员 2746967240018182176 按名退役（RETIRED 十四 → 十五；HEAD 基线 45 个离散成员 → 44，缺的只有它、零新增），`baseline_bridge` 门据此放行。

**无默认档位变更。** 语言扩展 v2.30 步 2 C / C++（2026-09-24；`Lang` 15 C（`.c`）与 16 Cpp（`.cpp .cc .cxx .hpp .hh .hxx .h .inl`，`.h` 归 C++——册 §14）翻位进判决集，**判决宇宙多两种语言，分数与 1.7.4 不可比**；既有五种语言的判决字节不动——旧二进制对拍见下）：

- **LangSpec 共表 `scan/spec_c.rs`**（C 与 C++ 一张表：C++ 独有的 kind 在 C 的解析里永不出现，共表零代价）+ `LangSpec` 新字段 `opaque_fields`（C 族：`preproc_if` / `preproc_elif` 的 `condition`；其余六表空）——`metrics::walk::measured` 成为函数子树取子结点的唯一喉口（`own_nodes` 与认知走查同读），`#if defined(X) && defined(Y)` 里的 `&&` 不再计入 CC / CoC（D28；lizard 同样不计，对拍抓出）。
- **单元谓词 `scan/declarator.rs`**（D24–D27）：C 族 `function_definition` 只在「有 body ∧ 声明链上有形参表 ∧ 链末是七种名叶之一 ∧ 有 type 或是特殊成员」且未被吸收时成单元。特殊成员 = 构造 / 析构 / 转换算符（类内无返回类型者须名等于类名、`~…` 或 `operator …`；无 owner 者存疑保留——format.h:4126 的构造函数，其类体被解析器读成了块）；吸收 = 最近作用域开启者之前被 unit 形定义或 lambda 包住（C++ 宏块与 GNU 嵌套函数并入宿主；局部类成员先遇到类，仍是独立单元）。名：类链 `Outer::Inner::m`、类外限定 `K::b`、模板实参去掉（`Box<T>::b` → `Box::b`、偏特化 `spec<int>` → `spec`）、空白折叠、`operator bool`、被解析器丢进 ERROR 的 `~` 放回（format.h:984）。六个 ce 缺陷全由对拍抓出：`= default` / `= delete` / `= 0` 成单元、`FMT_BEGIN_NAMESPACE` 读成名为 `namespace` 的单元、struct 折进 type 的垃圾名、类作用域 `FMT_CATCH(...) { }`、宏嵌套块、带换行的模板名——各配电池行。`functions.rs` 收成三条命名路（`name` 字段 / 声明链 / 匿名挂靠），`fourclass/units.rs` 的 `type_definition` 改读 `declarator::chain`；`is_unit_node` / `own_nodes` 多带 `src`（`similar/bag.rs` 同签名）。
- **调用弧**：`scan/calls.rs` 过 300 行拆出 `scan/callees.rs`（WHOLE / BASE 两路两键；BASE = (owner, 去限定名)，`this->m()`、`K::m()` 与成员体内的裸 `m()` 同键；同键两组即弃，规则未动）；`functions::owner_of` 是 owner 半的唯一生产者。
- **四类 / 可见性 / 提及**：`fourclass/kinds.rs` C 族九种声明 kind 一张表（两种宏、四种类型说明符、typedef、namespace、alias；原型 `declaration` 不是单元——头文件拼出名字本身就是提及，册 §6），`BODIED` 四种说明符只在带 `body` 时声明（`struct K x;` 与前置 `class Fwd;` 是引用）；`fourclass/visibility/c.rs`：bit 0 三路（类体成员按最近 `access_specifier` 或体默认——`class` 私有、`struct` / `union` 公开，`protected` 出口 + 受限位；`#define` 双位；其余外部链接，除非 `static` 或匿名 namespace；类外成员定义看不见说明符故读作出口——安全侧 D13），bit 1 = 无函数体 ∧ namespace 链全具名 ∧ 外层类体自身公开；`mention/conv/c.rs` 一位 `Ffi`（`linkage_specification` 祖先，或 `__declspec(dllexport)` / `visibility("default")` / `used` / `constructor` / `destructor` 属性词；裸 `extern` 不算）；`mention/name.rs` C++ 键按 `rsplit("::")` 取名，`selfref.rs` 收 C 族字符串字面量（`dlsym` 参数 / 注册表名）。
- **引用图**：站点 `#include`（`graph/spec.rs`：`path` 是 `string_literal` 或 `system_lib_string`，引号丢、尖括号留——分隔符就是阶梯读的搜索序；宏拼的 `#include HEADER` 是任何梯级答不了的诚实台账行），`store::KINDS` += `include`（GRAPH_REV 15 → 16，图表整库重算一次）。阶梯 `graph/ladder/c.rs` 四级：R1 含者同目录（只引号形）→ R2 `[graph.search_roots] c` 声明根（多根同名 → `AmbiguousRoot` 按名拒）→ R3 含者自己在 `compile_commands.json` 里的条目：`-I` / `-iquote` / `-isystem` 两种拼法按调用序首中（`graph/compdb.rs`：`arguments` / `command` 两形，`directory` + `file` 词法归一到仓根，仓外条目整条丢；`keys::CONFIG_NAMES` += `compile_commands.json`）→ R4 `<x>` 外部第 4 级、`"x"` OutOfScope；永不按文件名全树搜。配置 `[graph.search_roots]`（键 c / lua / java / r / html，五语言一张表——册 §14 裁 8；陌生键载入即拒），声明的目录须含走查到的文件——`[graph.search_roots] c declares "inc", which holds no walked file` 按名拒，与 `crate_roots` 同门 `walkidx::declarations`，两键各一行进 resolve_key；`docs/reference/ce-toml.md` 再生 +1 行。
- **编译单元角色位**（D18）：`ROLE_UNIT = 1 << 8`（`.c / .cc / .cpp / .cxx`；头文件 0），`main.*` 具名入口、`_test.*` 测试；核 `CE.Graph.Cost.roleBits` 加行 `(8, 1)`（7.2.0 同一未发布 minor 内加性，`GraphProps.rolesDerive` 加腿 `deriveFlags roleBits 256 == 2`），VERSIONING 7.2.0 条补句。
- **核 `CE.Verdict.Knobs`**：`cycleFloor` 回执重复的一行删除（用户裁「现在就修，随步 2 首个提交带出」；Aeson 对象同键后者胜，回执字节不变，golden 回放零差）。
- **对拍（lizard 1.23.0，`python -m lizard -l c|cpp`，按起始行 join）**：C 语料 lua/lua@0b29f40 五文件（lzio / ldblib / lbaselib / lfunc / loadlib）118 单元，CC 116/118 相符、2 条 D2（`default:` 不计）；C++ 语料 fmtlib/fmt@6d71f74 五头文件（base / color / format / ostream / std，`.h` 按 C++）：lizard 440 行 = 430 个起点、ce 428、join 420、逐条相符 394，26 条差全归因（D1 `#if` 族行 20、D2 3、局部类成员并入宿主 1 + 4、`if FMT_CONSTEXPR20 (` 1、lizard 数右值引用的 `&&` 1），两侧独有 18 条全归因（解析器恢复 14、lizard 在花括号初始化 `return {…};` 后并函数 1、局部类 4），lizard 对含模板实参的多行签名重复出行 10；四种既有语言单元数不动（go 52 / python 118 / rust 322 / typescript 25）。`DIVERGENCES.md` 新节 + 顶表两行、`SOURCES.md` 两行（两份 MIT）；`contracts/eval/dedup-distinct-v1.json` 与 `DEDUP-CALIBRATION.md` 的 fixtures 行随十个新文件重生（30 / 208 / 24）。
- **对拍旧二进制**：a6d9376 的 ce 1.7.4 + 核 7.1.0 与本树二进制在四份 crosscheck 语料副本上各跑十个报告面（scan / dedup / docdup / deadcode / clone / structure / erase / check 的 JSON 与 graph --sites / --mentions），40/40 逐字节相同——既有语言零判决改动。
- **电池**：子仓 `it/coc_c.rs`（CC / CoC / 嵌套行表含 D1 行、十六个名字、七种定义形）、`unit/graph/compdb.rs` 三腿、`it/graph_ladder_c.rs` C 十例 + compile db 四例、`it/graph_knobs.rs`（取代 `it/crate_roots_knob.rs`：两旋钮各一对声明树 / 对照树 + 四条按名拒绝）、`unit/graph/deadcode/flags.rs` 角色行、`unit/graph/sites_tests.rs` include 行、`unit/fourclass/visibility/tests_c.rs`、`unit/scan/callees.rs`、`unit/mention/*` C 行；探针 `scripts/tsprobe` 每结点打印起始行。
- **计划**：计划书 T 轨步 2 行就地记「已交付」；追加**步 7b 判决回迁三件**（用户两裁「搬前两处，随 1.8.0」+「塞进 1.8.0」：erase 的 `close_targets` + `licence` 择优 → erase/1 加性 `targets` / `kept`；structure 的 `STYLE` + `pattern_code` → structure/1 加性 `patternShapes`；CoC / CC / 最大嵌套的规则应用进核——Rust 只送 LangSpec 表分类后的结构事件流；每件以五语料分数逐字节等价为门；解析不搬、不为占比写代码），横幅八步 → 九步；册 §13 行 2「已交付」、D19 改为实证、D24–D28 入册；README 双语判决语言句加 C / C++，`count:langs` 七 → 九、`count:grammars` 六 → 八、`ver:graph_rev` 15 → 16 随 facts 门重签；how 页双语的 GRAPH_REV 芯片 16，架构图 IR 副标 `eight grammars` / 八套语法重渲（docs 与 site 孪生逐字节同），册 06 两处引文重瞄（`store.rs:105`、`config/graph.rs:77`）按名重签。
- **查重门抓出的两仓新克隆块全部消掉，预算未动**（主 54 / 子 119）：主仓 `functions::extract` 与 `walk::own_nodes` 的同形循环收成 `ast::preorder` 一条路（两个闭包：剪枝与取子结点），`Lang::grammar` 八条同形 match 臂改成 `GRAMMARS` 的 `LanguageFn` 表（第八条臂起两个 50 记号窗口就同形；`tree-sitter-language` 从 dev 依赖升为依赖、两份锁各只多一行、facts 刮取改按 `Lang::` 前缀数表行——`(Lang::` 漏掉 rustfmt 折行的 TypeScript 行，曾把八行读成七）；子仓 `it/coc_c.rs` 与 `unit/scan/calls.rs` 的元组行表改成一整个字面量（块以 `====` 分隔、头行 `ext @@ … @@ why`——任何一种元组行形每十来个记号就重复一次，把数组列换成字符串列只是换一种重复，两表在两文件里同形反而对上 175 记号），`unit/graph/deadcode/flags.rs` 两腿共用 `roles_at`，`it/graph_ladder_c.rs` 的树表用数组形并在测试前隔一个 `ROOTS` 常量（尾部与 `graph_ladder.rs` 的 hs 树尾曾对上 62 记号）。E01 顺手：`config.rs` 349 行拆出 `config/graph.rs`，`walkidx::index_all` 74 → 28 行（`refresh_tree` 拆出），`kinds::extra` 的 C 表提成 `C_FAMILY`，`conv/name.rs::test_file` 圈复杂度 16 → 11（`TEST_SUFFIXES` 表）。
- 门：主 check 944 / dedup 54 / scan 86 warn 0 fail（deadcode 0 / docdup 0 / erase 0），子仓 983 / 119 / 42 warn 0 fail；lib 383 / it 414 (12 ign)——提交前的全量跑是 413 绿 1 红，红的 `graph_provenance::audit_precedes_any_resolver` 对尚未提交的 `graph/compdb.rs` 报 `never committed`（它读 git log 定每个 graph 文件的首提交），是提交序的产物不是缺陷，CI 在提交上跑的全量 it 是它的执行者；cabal PASS、clippy（cli 全目标、gui `--locked`）+ fmt 清。ADR-006 具名重立（主根）：`cli/src/dedup/walkidx.rs` 203 → 256、`cli/src/mention/conv/name.rs` 233 → 256、`cli/src/scan/functions.rs` 136 → 192、`cli/src/scan/lang.rs` 186 → 210、`cli/src/fourclass/kinds.rs` 70 → 105、`cli/src/graph/deadcode/flags.rs` 96 → 119、`cli/src/scan/ast.rs` 56 → 78、`cli/src/scan/metrics/walk.rs` 25 → 44、`cli/src/graph/spec.rs` 146 → 161、`cli/src/scan/spec.rs` 347 → 359、`CHANGELOG.md` 706 → 722 超容差；`cli/src/config.rs` 302 → 255（`config/graph.rs` 105 拆出）与 `cli/src/scan/calls.rs` 277 → 223（`scan/callees.rs` 128 拆出）同批；新实体 `scan/declarator.rs` 248 / `scan/spec_c.rs` 106 / `graph/compdb.rs` 154 / `graph/ladder/c.rs` 91 / `fourclass/visibility/c.rs` 132 / `mention/conv/c.rs` 49 与十个 crosscheck 夹具入列，其余容差内同定。（子仓）`unit/fourclass/units.rs` 94 → 108、`unit/mention/conv/name_tests.rs` 154 → 168、`unit/scan/lang.rs` 48 → 60、`unit/graph/deadcode/flags.rs` 34 → 56 超容差；新文件 `it/coc_c.rs` 253 / `it/graph_knobs.rs` 109 / `it/graph_ladder_c.rs` 115 / `unit/fourclass/visibility/tests_c.rs` 79 / `unit/graph/compdb.rs` 57 / `unit/scan/callees.rs` 21 入列、`it/crate_roots_knob.rs` 退役；`it/graph_ladder.rs` 666 → 666——C 行与 compile db 腿加进去时 759 行撞 750 硬线，搬到 `graph_ladder_c.rs`。

**无默认档位变更。** 语言扩展 v2.30 步 3 Java 提交 A（2026-09-24；`Lang` 18 Java（`.java`）翻位进判决集，**判决宇宙再多一种语言，分数与 1.7.4 不可比**；既有语言的判决字节只在下面三处按意图移动——旧二进制对拍见下。Java 的引用阶梯不在本提交：按设计册 §14 第 14 条，考题先冻结提交，没看过解析结果的独立代理盲判真值再提交，阶梯与精度册最后）：

- **LangSpec 拆册与四个新字段**：`scan/spec.rs` 只留契约（结构体、分派、Markdown 空表），四张 M1 启动表逐字迁到 `scan/spec_launch.rs`，Java 表 `scan/spec_java.rs`（C / C++ / Haskell 各在旁）。新字段：`fn_required_fields`（取代 `fn_named_only_kinds`，写成 (kind, 字段) 对：Haskell 两种单元要 `name`，Java `method_declaration` 要 `body`——抽象 / 接口 / native 签名不成单元，D24 同一立场）；`if_kinds`（认知复杂度按精确表认 if，不再 `starts_with("if")`；if 的 `alternative` 字段里既不是 if 也不是 else 结点的东西就是 else 本体、+1，Java 单语句 `else return x;` 此前数不到）；`call_fields`（callee 字段 + 可选接收者字段：Java 的方法名挂 `name`、接收者挂调用自己的 `object`，设计册 §4 (c) 原拟名 `callee_field` 就地改记）；`owner_kinds`（Java 成员的 owner = 包住它的类型声明的名字）与 `overloads`（下条）。启动四表只多这些字段，每个都按该语言原有行为拼写，既有五语言电池逐值不变。
- **重载按实参个数择一**（C++ 与 Java，`scan/callees.rs` 的 `Named::pick`）：同一作用域里同名、形参不同的函数是不同的可调用体，调用落到恰好一个实参个数区间能接纳它的重载，两个都接纳即不连边（少报方向）。C++ 默认实参只抬上界，C 式 `...` 与形参包、Java varargs 去掉上界，C++ 包展开 `f(a...)` 实参个数未知，Java 接收者形参不占实参、构造器不参与（`new` 与 `this(…)` 都不是调用 kind）。此前 C++ 同名函数读成一个可调用体，「调另一个重载」被记成递归：fmt 五头文件同树重扫 CC 不动，CoC 四个单元各 −1（`styled_arg::vformat_to` 5 → 4、`append` 1 → 0、`to_utf8::convert` 2 → 1、`size_padding::nested_format_specs::parse` 3 → 2，`DIVERGENCES.md` 步 3 补记）。
- **Java 的单元、调用与成员路**：单元 = 有体的 `method_declaration`、`constructor_declaration`、`compact_constructor_declaration`（记录的紧凑构造器以记录头的组件为形参，JLS 8.10.4.2）；匿名类与局部类的方法是独立单元（D29）。成员路按成员所在的那一个类型体作键（`callees::Owner::Body`——匿名类、枚举常量体、同名局部类各是一个类）：`this.m()`、裸 `m()` 与 `K.m()`（K = 自己的类名）到达本类的 `m`；`super.m()` 永不是自身——覆盖方法调 super 是委托，不是递归。`Main.java` 是具名入口；Maven Surefire 默认的四种测试类名（`Test*` / `*Test` / `*Tests` / `*TestCase`）与 C 族 `_test` 同表（`mention/conv/name.rs` 的 `RUNNER_TESTS`，死代码角色与提及类别共读一张表）。
- **四类 / 可见性 / 提及**：`fourclass/kinds.rs` 收 Java 五种类型声明（class / interface / enum / record / 注解类型）。`fourclass/visibility/java.rs`：bit 0 = 写出的或隐含的访问（接口与注解类型的成员隐含 public、枚举构造器隐含 private，其余不写即包访问 = 出口 + 受限位，`pub(crate)` 读法；`protected` 同为出口 + 受限），bit 1 = 每层外围类型自身公开，且外面没有名字出不去的体（方法体、初始化块、构造器体、lambda、匿名类、枚举常量体）。`mention/conv`：注解 = `Registration`（框架经反射找到被注解者；`@Override` 也算，安全方向），`main` = `Main`，平台替作者调用的方法名表（Object / Comparable 契约、函数式接口、迭代、序列化钩子、克隆与终结、枚举合成对、servlet 生命周期）= `Protocol`；`.java` 进 `$` 整段臂（`Outer$Inner` 与 `Inner` 是不同的标识符），`MENTION_REV` 2 → 3，提及缓存重算一次。`scan::ast::ancestors` 成为唯一的祖先链（可见性、提及类别词与 Java owner 同读），`ast::entries` 是形参表与实参表的同一读法（重载要比两者个数）；`similar/bag.rs` 的 `ret:` 形状词认 `type` 字段非 `void`（C 族与 Java 把返回类型写在名前，构造器不写）。
- **引用站点**（`graph/spec.rs`、`graph/sites.rs`、新 `graph/sites/java.rs`）：`import`（单类型导入；`static` 形的 spec 从 `static` 起）与 `import_star`（按需导入，同 TS `export *` 单列标签）两行进站点表。`type_ref` 是文件级一遍——哪些名字算数要排除本文件自己声明的类型与类型形参，这是文件事实，表行装不下：类型位置、注解名、记录模式的类型、模块 `uses` / `provides` 指令，以及表达式里像类型的接收者（`Util.f()`、`Mode.FAST`、`Util::f`）与经包名写出的类（`a.b.C.f()`），不含 `var`、本文件自己的类型与类型形参、被同名变量遮住的接收者（lambda 形参与枚举常量也是变量，JLS 6.4.2）；spec 截在类型实参的 `<` 处（设计册 D17：同包引用不经 import，没有这类站点，只在本包被用到的类会读成没人引用）。`store::KINDS` += `import_star` / `type_ref`；**GRAPH_REV 仍 16**：步 2 与步 3 同随 1.8.0，没有任何已发布的索引持有本步会改的行（`store.rs` 注释写明）。本提交**无阶梯**：Java 站点全部进未解析台账，直到阶梯提交。
- **自提及区域的文档代码块**（`mention/selfref/doc.rs` 与 `doc/blocks.rs`，自 `selfref.rs` 拆出）：Javadoc `<pre>` / `{@code}` / `{@snippet}` 与 JDK 23 `///` Markdown 文档注释的围栏，Doxygen `@code…@endcode`（`\code` 同）与围栏；三种 Markdown 读者（rustdoc、Doxygen、JDK 23 的 Markdown 文档注释）另读 CommonMark 缩进代码块（spec 0.31.2 §4.4）。**顺带修掉一个已发布的 Rust 缺陷**：rustdoc 把缩进块当 doctest 编译（`cargo test --doc` 实测：缩进块里一句写错的断言让 doctest 失败），而 ce 此前只认围栏，缩进 doctest 里的提及被漏掉。C++ 语料（fmt 五头文件）顾问行 173 → 166，其余五份语料的顾问与提及计数不动。
- **考题冻结**（设计册 §14 第 14、15 条；新册 `docs/EVAL-SET-LANGS.md` 入冻结集）：站点宇宙 `contracts/eval/lang-slice-gson-v1.json`（google/gson@854c825，264 文件 / 22,698 站点：import 2,674、type_ref 20,024、通配导入 0）与 `lang-slice-jsoup-v1.json`（jhy/jsoup@093e2f5，204 文件 / 24,210 站点：import 1,887、import_star 80、type_ref 22,243——用户裁「加 jsoup」，因为 gson 一处通配导入都没有）；样本 `lang-sample-java-v1.json` 主 100 题（import 20 / import_star 15 / type_ref 65：每类地板 15，剩余座位按剩余池最大余数分）+ 备用 60 题，落在 74 个文件上。仪器 `it/eval_lang_parts/`（考题表、抽样、`#[ignore]` 生成器、样本核对器）；CI 门 `it/eval_lang.rs`（宇宙信封、扩展名 = 产品路径表、样本逐行重导两个哈希、配额从冻结摘要重算、篡改八形全拒、交叉核对夹具按冻结行复核）；顺序门 `it/lang_provenance.rs`（审阅表提交前任何提交不得碰 `cli/src/graph/ladder/java*`、精度册不得存在；两层绊线与 `graph_provenance.rs` 共用 `eval_support::assert_resolver_after_audits`）。三份档的 `generated_from` 记 `bac6169`、dirty = true：检测代码与档同一提交落地。
- **对拍**（`contracts/fixtures/crosscheck/java/`，gson 五文件；`SOURCES.md` 新行与 PMD 工具行）：CC 对照 lizard 1.23.0，按结束行 join 33 条，**33/33**；ce 独有 5 条全是带类型实参的匿名类方法（lizard 并进宿主或整段不报，D29）。CoC 对照 PMD 7.27.0 的 `CognitiveComplexity`（Java 是第一个有可跑的独立 CoC 实现的新语言），join 38 条 **36/38**：匿名类方法 PMD 并进宿主 1（D29，归因保留）；`if` 条件里的三元 1——PMD 与 sonar-java 都不给条件加嵌套，ce 给；用户裁「条件都不算」（所有语言只让语句体抬嵌套），**随下一个提交落码**，届时这一行改为 37/38。ce 起始行早一行的 16 条是注解（D30：方法结点从 `modifiers` 起，JLS 8.4.3），故按结束行 join。五个 Java 样例也进了查重校准的 fixtures 行（`--ignored regenerate` 重量 `contracts/eval/dedup-distinct-v1.json` 与 `DEDUP-CALIBRATION.md`）：30 → 35 文件、208 → 218 块、下限关时 distinct ≤ 6 的 24 → 34——新增 10 块全在 `ReflectiveTypeAdapterFactory.java` 开头那串 import 里，distinct 4，出厂下限 7 本就抑制；四个外部语料行逐字不变。
- **旧二进制对拍**：bac6169 的 ce 与本树二进制在六份既有 crosscheck 语料副本（go / python / rust / typescript / c / cpp）上各跑十个报告面，60 面里 52 面逐字节相同，8 面按意图移动：六份 `graph --mentions` 的头行 `rev 2` → `rev 3`（cpp 那份另有未提及 266 → 259、本文件例外否决 2 → 9），cpp `deadcode` 顾问 173 → 166，cpp `scan` 四个 CoC 各 −1（上面的重载）。
- **电池**：子仓 `it/coc_java.rs`（十行指标 + 一行 `units=` 单元与形参）、`it/sonar_whitepaper_java.rs`（白皮书自己的 Java 例题九行，以页边注为准）、`it/coc_recursion.rs` 的 Java 端到端一例、`unit/scan/calls.rs` Java 十行与 C++ 重载四行、`unit/graph/sites/java.rs`（type_ref 规则；十二处变异——`var`、本文件类型、被遮接收者、限定名作一站、大写判型、包头遮蔽、模块指令、lambda 形参、枚举常量、类型实参截断、类型形参、注解——各自转红，源文件还原后 sha256 相等）、`unit/fourclass/visibility/tests_java.rs`、`unit/mention/*` 与 `unit/graph/*` 的 Java 行。共用件提升：`eval_support::largest_remainder`（t3 与语言考题同一分座）、`generated_from`（L2 FPR 台账与考题同一戳）、`assert_envelope_core`、`site_row` / `site_summary`、`sites_within_windows`（语句窗口核对，自仓漂移门与考题对拍同读）、`common::assert_metric_table`（单字面量电池表，`coc_c.rs` 改用；新增 `units=` 行形，按源序钉单元名与形参数）；过 300 行的三处各拆一刀：`it/eval_lang.rs` 拆出核对器 `eval_lang_parts/verify.rs`，`unit/scan/calls.rs`（220 → 313）拆出调用点作用域一组 `calls_scope.rs`，`it/common/mod.rs`（已 296 行）把指标电池整组连同 `parse` 挪到 `common/metric.rs`。
- **文档**：README 双语判决语言句加 Java（`count:langs` 九 → 十、`count:grammars` 八 → 九随 facts 门重签）；架构图 IR「nine grammars / 九套语法」重渲（docs 与 site 孪生）；设计册 D29 / D30 入册、§4 (c) 改记 `call_fields`、§9 文档代码块句、§11 加 PMD、§13 行 3、§14 第 15 条；册 13 自提及区域句（引文改瞄拆出后的三个文件）、册 01 四处 `literal_delims` 引文改瞄 `spec_launch.rs`、册 06 冻结站点类「十一」改「十四」并补 rev 16 一句（步 2 的 `include` 当时漏记）；计划书横幅与 T 轨步 3 就地记「提交 A 已交付」（334 行不变）。
- **查重门点名的新克隆块全部消掉，预算未动**（主 54 / 子 119）：主仓 `fourclass/units.rs` 与 `graph/sites.rs` 各自手写的「解析失败即返回空」改走 `ast::with_tree`，`cognitive.rs` 的 `has_label` / `has_if_child` 合成一个自由函数 `has_child_of`。子仓：C 与 Java 两套电池各带一个单元清单测试、文件骨架同形（55 记号）——清单改成各自指标表的末行 `units=`；`coc_recursion.rs` 的元组行（配对切片形每十来个记号重复一次）改成四个字符串，期望值按名序写 `name=value`、渲染后按文本比（先写的「解析成映射」一版又与 `graph_mounts_codes.rs` 的解析同形）；两处语句窗口循环收成 `sites_within_windows`；两道信封核对原本各收同样五个参数，改收 `UniverseFamily`（结构体挪到 `universe.rs`，`family.rs` 仍单向引它；考题声明自己的 `eval_lang_parts::SLICE`）；可见性两张语言表原本各带一对测试，改为各留一个、同调 `tests.rs` 的 `run_tables`（把两文件挂进 `visibility/mod.rs` 的一版让那里的挂载串与 `mention/mod.rs` 同形，主仓 54 → 56，退回原挂载）；`testutil::blocks` 以常量泛型 N 自己核列数、列数不符按名拒绝（三个调用各写一遍的 `try_into` 消掉，拆出的 `calls_scope.rs` 开头因此不再与 `sites/java.rs` 同形）。主仓另有一块换了搭档而非新写，见下条。
- 门：主 check 944 / dedup 54 / scan 85 warn 0 fail（deadcode 0 / docdup 0 / erase 0），子仓 983 / 119 / 45 warn 0 fail；lib 384 / it 422 (14 ign，新增两条是考题冻结仪器)——提交前的全量跑是 419 绿 3 红：`lang_provenance` 两条对尚未提交的 `lang-sample-java-v1.json`、`graph_provenance::audit_precedes_any_resolver` 对尚未提交的 `graph/sites/java.rs` 报 `never committed`，与步 2 那条同是提交序的产物；clippy（cli 全目标、gui `--locked`）+ fmt 清、cabal test PASS。ADR-006 具名重立两仓：主 `scan/functions.rs` 192 → 258、`scan/callees.rs` 128 → 187、`graph/sites.rs` 203 → 243、`scan/calls.rs` 223 → 269、`mention/conv/name.rs` 256 → 290、`scan/spec_c.rs` 106 → 141、`graph/spec.rs` 161 → 189、`mention/conv/mod.rs` 228 → 252、`scan/ast.rs` 78 → 97、`similar/bag.rs` 282 → 293 超容差，新文件入基线；子 `it/eval_support/graph.rs` 165 → 223、`it/eval_support/provenance.rs` 138 → 187、`it/eval_support/auditgen.rs` 108 → 136、`unit/mention/selfref_tests.rs` 100 → 134、`unit/testutil.rs` 64 → 88、`unit/fourclass/visibility/tests.rs` 220 → 244、`it/eval_support/universe.rs` 196 → 218、`it/coc_recursion.rs` 181 → 193、`unit/mention/conv/name_tests.rs` 168 → 180、`it/baseline_reanchored.rs` 133 → 167 超容差，新文件入基线。主仓基线的克隆成员 44 → 44 但换了两个：四张启动表迁到 `spec_launch.rs`，TYPESCRIPT / RUST 那对随路径换键（路径是成员身份的一部分）；LangSpec 各表加字段后 GO / HASKELL 那对不再押韵、散了，FAMILY / TYPESCRIPT 同形凑成新的一对（块数 54 不变，字段表同形这一类在基线里此前此后都是两对）。子仓代际门 `baseline_bridge.rs` 为此多一条出路 `RELOCATED`（7.0.0 之后挪文件造成的换键，记在 `baseline_reanchored.rs`；推导 = 一次性仪器把挪过的一侧按旧路径重哈希，同一仪器在挪前的树上复现该提交的 44 个成员一个不差），GO / HASKELL 按名入 `RETIRED` 第十六行；删掉这两笔记账或塞一行过时的 `RELOCATED`，门各自转红，还原后逐字节相等。

**无默认档位变更。** 语言扩展 v2.30 步 3 Java 提交 A′：盲评真值冻结（2026-09-24；只加冻结工件与它的门，判决代码字节零变化）：

- **真值表** `contracts/eval/lang-review-gson-v1.json`（38 行）与 `lang-review-jsoup-v1.json`（62 行）：四个独立 Opus 代理各判一批 25 道主样本（按样本的审阅序切批），只读两个语料在钉住 tip 的克隆和自己那一批，没看过任何解析结果、没跑过 ce；装配逐字照录，判决不动。100 道的 spec 全在记录的行上，零失配，没有动用备用题。真值：external 63、语料内 36（文件 33 / `#成员` 1 / 包目录 2）、`none` 1；两处约定（按需导入本文件的嵌套类判 `none`、拆分包取导入者自己的源根）与 5 条候选漏检照判词记下，见 `docs/EVAL-SET-LANGS.md` 的「真值」节。
- **门**：子仓新 `it/eval_lang_parts/review.rs`，`it/eval_lang.rs` 加两条——`lang_reviews_verify`（逐行对应主样本、身份回显、真值绑冻结宇宙、判词地板、摘要重算、漏检落在冻结文件上）与 `a_tampered_review_is_refused`（八种篡改各自被拒）。考题表 `Exam` 新增 `audited` 旗标，表在不在盘必须与它逐语料相符（删掉一张表即被点名，负向探针实测），顺序门 `lang_provenance.rs` 改读同一个判定；判词地板 `MIN_WHY`（40 字符）从同角色仲裁门移入 `eval_support`，两门共读一个常量。
- 门：主 check 944 / dedup 54 / scan 85 warn 0 fail（deadcode 0 / docdup 0 / erase 0），子仓 983 / 119 / 46 warn 0 fail；lib 384 / it 424 (14 ign，新增两条是审阅表的校验与篡改探针)——提交前的全量跑是 419 绿 5 红：`lang_provenance` 两条对尚未提交的审阅表报 `never committed`，与提交 A 同是提交序的产物；另三条是那一小时别的会话的测试把 CPU 占满（负载 100 %）——`trend_rebuild` 两条的核应答超过 60 s 截止、`guard_hook::deny_mode_intercepts_t1_rewrite` 的探针失败放行后没有输出，单跑 3/3 绿（24.7 s）；判决代码与核未动，`cabal test` 不涉及。子仓查重点名的一块新克隆（审阅表与样本两个校验器各自按语料名查钉住的 tip）收成 `Exam::tip` 一个方法，克隆块逐条与提交 A 的 119 块相同。ADR-006 具名重立：主 `docs/EVAL-SET-LANGS.md` 66 → 90 超容差、`CHANGELOG.md` 738 → 744 容差内同定；子 `it/eval_lang.rs` 150 → 220 超容差，新文件 `it/eval_lang_parts/review.rs` 入基线。

**无默认档位变更。** 语言扩展 v2.30 步 3：条件不抬嵌套（2026-09-24，用户裁「条件都不算」；认知复杂度的读法变了，分数与 1.7.4 不可比——1.8.0 本就不可比）：

- **规则**：结构的头部——条件、循环子句、switch 的值、catch 的形参——按结构自己的层级计分，只有语句体抬嵌套；else-if / elif 的条件与首个 if 的条件同在链的层级；三元整个抬嵌套（D4 不变）。语句体的位置写进 `LangSpec::coc_nesting_kinds` 已有的项里：每项由 kind 扩成「kind 位置…」，位置是字段名，或者语法没给语句体起字段名时写它的 kind（Python except 的 `block`、Go switch 的两种 case、Haskell case 的 `alternatives`）；Python 的 elif 是带条件的平级分支，它那项在 `coc_flat_kinds` 里同样写；只写 kind 的项整个抬嵌套（三元、Go 的 `select`）。`metrics/cognitive.rs` 的 `walk_split` 按项把子结点分给头部与语句体。不另开一张表：先试的独立字段让每个语言的结构体字面量多出一段同形记号，查重从 54 块变 55 块（新出 Java 对 TypeScript、Java 对 Rust 两块，TypeScript 对 Rust 一块消失），改成每张表一个字符串后是 56 块；写进已有的项，字面量的记号序列与改前逐个相同，查重仍是 54 块、与提交 A′ 逐块对应（两块随新注释下移两行），两张表也不会再各写各的。
- **对照**：白皮书只列抬嵌套的结构（p.9、Appendix B2），没说头部算不算。sonar-java 与 PMD 7.27.0 只在 `if` 自己的条件上同此读法；PMD 对循环、switch 与 else-if 的头照样抬嵌套（九个方法的探针，登记册 D31 节），ce 在那几处比 PMD 低 1，归因保留。
- **影响**（同一棵树，新旧两个二进制逐单元按起始行对拍）：七个对拍语料 1,098 个单元里移动 2 个——Java `checkAccessible` 3 → 2，Java 的 CoC 对 PMD 由 **36/38 → 37/38**；C++ `do_write_float` 16 → 15。本仓与九个语料 43,732 个单元里移动 12 个（本仓两个：`CE/Structure.hs` 的 `declaredKeys` / `splitKeys`，都是 `case` 判断值里的 `if`），全是头部里的三元、lambda 或 `case` 判断值里的 `if`，每个降 1–2 分。裁定时问题里写的「9 个语料 24,476 个函数里 32 个降 1–6 分」出自一版试验实现：它把第一个语句体字段之前的子结点都当头部，语句体没有字段名的结构（Go switch 的 case、Python except、Haskell case）因此整块被压平；落码版逐语言写明语句体，这些结构不动，电池各有一行钉住。
- **电池**：子仓新 `it/coc_headers.rs`，八种语言 25 行，每行 why 写明算式与落码前的读数，Java 行另记 PMD 的读数。
- **登记**：`contracts/fixtures/crosscheck/DIVERGENCES.md` 新节「条件不抬嵌套」、Java CoC 行 37/38、一行重跑记录（递归增量那行的「详见末节」顺手改成节名：后面已接了 C / C++ 与 Java 两节）；设计册 §4 加一行语句体位置（Lua / R 两列按钉版文法的 node-types 预写，随步 4 落码）、机制段加 (e)、§5 D31、§14 第 16 条；`cognitive.rs` 头注写明两个对照物各自怎么读。
- **记账**：本册第四次抵 750 行硬线，v1.5.0–v1.5.1 两条逐字节迁入新归档册 `docs/CHANGELOG-ARCHIVE-v1.5.md`；冻结集（子仓 `it/frozen_set.rs`）与引文门豁免表（`contracts/docs-citations-optout.json`）各加一行。
- 门：主 check 945 / dedup 54 / scan 85 warn 0 fail（deadcode 0 / docdup 0 / erase 0），子仓 983 / 119 / 46 warn 0 fail；lib 384 / it 425 (14 ign，新增一条是本批的电池)——提交前的全量跑是 421 绿 4 红，都是那一小时别的会话把 CPU 占满（负载 94–100 %）时钩子与守护进程按设计失败放行：`audit_bypass::a_plain_dot_git_file_cannot_re_root_the_guard` 与 `guard_hook::default_tier_denies_t1_rewrite` 没有输出、`health_plugin` 读到 daemon 未就绪、`observe_feed` 的一条 probe 记成 degraded，单跑 4/4 绿（24.7 s）；clippy（cli 全目标）+ fmt 清；核未动，`cabal test` 不涉及。改后的二进制与量读数时的二进制在上述 17 个语料 44,830 个单元上逐单元相同（0 差）。ADR-006 具名重立：主 `scan/metrics/cognitive.rs` 203 → 256、`scan/spec.rs` 227 → 239 超容差（长出来的是头部与语句体的拆分、两张表项写法的说明），`scan/spec_launch.rs` 226 → 230、`scan/spec_hs.rs` 100 → 102、`docs/reference/language-expansion.md` 214 → 217 容差内同定，`CHANGELOG.md` 744 → 609 随拆册下降，新归档册入基线；克隆成员集与提交 A′ 相同（0 added / 0 removed）。子仓无超线、未重立。

**无默认档位变更。** 语言扩展 v2.30 步 3 Java 提交 B：引用阶梯与精度册（2026-09-25；Java 文件从此在图上有边，Java 文件的死代码、结构与分数随之移动——判决宇宙在提交 A 已变，**分数与 1.7.4 不可比**；既有语言零判决改动，对拍见下）：

- **阶梯 `graph/ladder/java.rs`**（设计册 §8 Java 行按落码改写）：类 `a.b.C` 是声明 `package a.b` 且名为 `C.java` 的那个被走文件——javac 的读法，文件放在哪个目录不作数（设计初稿写的是从目录反推源根，落码时改为按包建索引：目录与包路径对不上的文件照样认得）。R1 单类型导入落它的文件；R2 更短的前缀——嵌套类、static 成员、`import a.b.*` 落该包唯一的目录（ResolvedPackage）、`a.b.C.*` 与 static 通配落 `C.java`；R3 `type_ref` 按 JLS 6.4.1 的序：本文件的单类型导入（static 的也算，类型优先）→ 本包（本目录优先）→ 通配导入的包，限定名看首段，类型注解剥掉；R4 External：JDK 导出包下的名字、`java.lang` 的公开类型、域外通配导入全是 JDK 包时的简单名。同一级两个文件答 = ambiguous_root（跨通配包 = ambiguous_paths），除非 `[graph.search_roots] java` 恰好只含其一；其余 out_of_scope——第三方导入、经继承才看得见的嵌套类、文件名与类名不同的类（一个文件里的第二个顶层类）都是漏答，从不猜边。
- **JDK 名表 `graph/ladder/java_jdk.rs`**：机器生成自 Temurin 25.0.4.1+1——无条件导出的 233 个包（`java --list-modules` 后逐模块 `--describe-module` 的 `exports` 行；限定导出不是 API）与 `java.lang` 的 108 个公开顶层类型（读 `lib/modules` 里类文件自己的访问标志）；缺的名字降为 out_of_scope（精度安全），换新 JDK 重生成即补。
- **文件头 `graph/ladder/java_header.rs`**：包声明与导入按词法读（JLS 7.3–7.5：它们在任何类型声明之前，前面只能有空白、注释和包自己的注解）；遍历（`dedup/walkidx.rs`）每次对每个 Java 文件读一遍，经 `ladder::Scope::java` 交给阶梯，阶梯自己不读文件。声明的包进 `resolve_key`（包变了边要重算），导入不进（它们只管本文件的站点，本文件刷新时自然重解）。
- **精度册**（`contracts/eval/lang-precision-gson-v1.json`、`lang-precision-jsoup-v1.json`；判分 `it/eval_lang_parts/score.rs`，核对 `precision.rs`，CI 门 `it/eval_lang_precision.rs`；登记 `docs/EVAL-SET-LANGS.md`「判分」一节）：gson 12/12、jsoup 22/23，整体 34/35 = 0.971（门 0.90），站内真值 36 道答中 34 道；`type_ref` 26/26。唯一的 wrong：jsoup 一个文件用 static 通配导入自己的嵌套类，阶梯答了这个文件本身，真值按词表是 `none`。两道 missed：测试根里的 `import org.jsoup.nodes.*`——这个包在 main 与 test 两个源根各有文件，阶梯按设计不挑。宇宙台账：gson 22,698 个站点解出 87.0 %、jsoup 24,210 个解出 86.7 %，首级占比 5.0 % / 3.4 %（触发线 0.80）。审阅者记下的 5 条候选漏检逐条核实，没有真漏。
- **考题表 `Exam` 加 `scored` 旗标**（Java 为 true）：精度册在不在盘必须与它逐语料相符，删掉一份会被点名。审阅表与精度册共用一套路径与读取的拼写（`Docs`）和一个核对（`Exam::filed`）；样本、审阅表、精度册三份篡改电池收成一个框架 `it/eval_lang_parts/tamper.rs`——是查重门点的名：新写的精度册电池开头与审阅表那份逐记号同形（子仓 122 > 预算 119），**预算没动**，收成一处后回到 119、克隆清单与改动前逐条相同。
- **电池**：`it/graph_ladder_java.rs`（十一个文件的夹具树：包 a.b 分在三个目录、包 p 在两个根、两个通配包各有一个 S、一个无名包文件；30 行四级答案与拒绝；另 4 行在 `[graph.search_roots] java = ["src"]` 下重答：根恰含一侧的三个平局解开，两侧都在根下的那个照旧拒绝）、`unit/graph/ladder/java_header.rs`（文件头读法 10 块：四种导入形、注释与空白穿插、package-info 的带参注解、注解里的文本块、module-info、缺 `;` 的包声明 / 导入里夹杂的记号 / 第一个类型声明让文件头就此结束、关键字边界、任意文字的标识符；另一腿钉字节序标记）、`unit/graph/ladder/java.rs`（类型注解不属于名字，8 行：带参、嵌套括号、注释、读不完的文本块）。
- **配置说明**：`docs/reference/ce-toml.md` 的 `[graph.search_roots]` 一行写明 `java` 键做什么（一个类或包在两个源根各有一份时，取声明目录下的那一个）。
- **计划**：计划书横幅与 §6 T 轨步 3 记「已交付」（四个提交），设计册 §13 行 3 同改。
- **对拍**（旧 = 2090e57 编出的程序，新 = 本提交）：六个既有对拍语料（go / python / rust / typescript / c / cpp）加 Java 对拍语料，每个语料十个报告面，70/70 逐字节相同，六种既有语言零判决改动。Java 对拍语料没动在预期之内：它的五个文件把目录摊平进了文件名（`gson__src__main__java__com__google__gson__JsonDeserializationContext.java`），文件名与类名对不上，新阶梯一条站内边也解不出（JDK 名字答成外部依赖，本就不产生边）。新阶梯的作用要在真树上看：jsoup（考题钉的 `093e2f58`）新旧同跑，图的保留边 0 → 1,295、没有边的站点 24,452 → 11,935、死文件 129 → 27（unref_public 120 → 16、unref_private 9 不变、新出 unreach_public 2），`ce check` 696 → 698，`ce structure` 960 → 807——有了边，三条结构轴（2、3、7）从 0 开始出分，发现 12 → 63 条。
- 门：主 check 945 / dedup 54 / scan 85 warn 0 fail（deadcode 0 / docdup 0 / erase 0），子仓 982 / 119 / 46 warn 0 fail；lib 387 / it 429 (15 ign)——提交前的全量跑是 426 绿 3 红：`lang_provenance` 与 `graph_provenance::audit_precedes_any_resolver` 对尚未提交的 `graph/ladder/java.rs` 报 `never committed`（它们读 git log 定新文件的首提交，是提交序的产物，提交后单跑），`core_size_gate` 逐文件那一腿报 `index was rebuilt by another process mid-run — not stamping full_build`（全量里几条测试同时给主仓建索引，产品按设计拒绝），单跑 3/3 绿；clippy + fmt 清；核未动，`cabal test` 免跑。ADR-006 具名重立——主根：`cli/src/dedup/walkidx.rs` 256 → 275 超容差（cap 266）、`docs/EVAL-SET-LANGS.md` 90 → 145（cap 100）、`CHANGELOG.md` 609 → 622（cap 621），`cli/src/graph/ladder/mod.rs` 256 → 266 与 `cli/src/dedup/mod.rs` 312 → 313 容差内同定（`cli/src/graph/store.rs` 的注释改动重排回 417 行，本就过 300 行警告线的文件一行不长），三个新文件 `graph/ladder/java.rs` / `java_header.rs` / `java_jdk.rs` 入基线，softLine 359 → 357；子仓：`it/eval_lang_parts/mod.rs` 208 → 258（cap 218）、`it/eval_lang_parts/generate.rs` 148 → 237（cap 158），另四行容差内同定、三行变短，八个新文件入基线——其中 `it/coc_headers.rs` 与 `it/frozen_set.rs` 62 → 63 是上一个提交（条件不抬嵌套，子仓 a3e41af）漏重立子仓基线留下的，这次一并入账；softLine 301 → 304。新写的精度册篡改测试原为 51 行，把首级占比触发线的两向检查抽成 `assert_rg1_both_ways` 后回到 50 行以内。

**无默认档位变更。** Haskell 占比目标立项记录（2026-09-25，用户两裁：GitHub 语言条上 Haskell 提到 40 % 以上；路线 = 1.8.0 发布之后开新功能轨、核的测试计入语言条，不改写现有代码）：

- **语言条口径**：`.gitattributes` 撤掉 `core/test/** linguist-vendored`，核的性质电池（204,477 字节 Haskell）从此计入 GitHub 语言条（改前 API 读数：Rust 1,830,565 / Haskell 341,694 字节）。Rust 的测试在 CodeEraser-tests 子仓，语言条读不到，所以口径偏向 Haskell；该文件注释照实写明，不再自称两边对称排除。
- **计划书**：横幅记 v2.31 功能轨（查询与架构规则、函数内死代码、克隆合并建议、架构分析，分析全在核、Rust 只送整数事实），§6 T 轨行尾同记；ADR-008 那句占比纪律补明它管的是搬迁与改写已有代码。细则与 plan-set 在 1.8.0 发布后另立。

**无默认档位变更。** 语言扩展 v2.30 步 4 Lua + R 提交 A（2026-09-25；`Lang` 17 Lua（`.lua`）与 20 R（`.R` `.r`，扩展名按字面匹配）翻位进判决集，**判决宇宙再多两种语言，分数与 1.7.4 不可比**；既有语言零判决改动——旧二进制对拍见下。两种语言的引用阶梯不在本提交：按设计册 §14 第 14 条，考题先冻结提交，没看过解析结果的独立代理盲判真值再提交，阶梯与精度册最后；在那之前 Lua 与 R 的每个站点都是一行 `unsupported` 台账行，不静默跳过）：

- **两张 LangSpec 表** `scan/spec_lua.rs`、`scan/spec_r.rs`（文法步 1 已钉：tree-sitter-lua 0.5.0、tree-sitter-r 1.3.0）：Lua 的 else 是结点（`else_statement`，与 C / Python 同读），`elseif` 挂在 if 的 `alternative` 字段、条件与整条链同级；R 的 else 没有结点，`alternative` 就是 else 表达式或下一个 if（Go / Java 的形状），不带花括号的 else 照样计分。R 的 `&&` `||` 短路计分、向量化的 `&` `|` 不计，`repeat` 是循环（D9）；Lua 的 `pcall` 与 R 的 `switch()` `ifelse()` `tryCatch()` 是函数调用、不计（D8）；Lua 的 `goto` +1，R 没有带标签的跳转（D5）。认知复杂度的运算数字段补上 tree-sitter-r 的 `lhs` / `rhs`。
- **函数值命名 `scan/binding.rs`**（设计册 §4 名字一行）：两种语言的多数具名函数写成匿名 `function_definition`，名字来自外面那条语句——Lua 的表字段（`{ f = function() end }`；方括号里只有字符串键算名字）、`local` 与普通赋值（第 i 个值取第 i 个变量）；R 的 `<-` `=` `<<-` `:=` 取左边、`->` `->>` 取右边（只经括号：`function(x) x -> f` 是一个把 x 赋给 f 的函数体）、链式赋值取最内层目标、字符串目标取内容；实参里的函数与 `attr(x, "a") <-` 这类目标不命名。同一个读者（`binding::of`，答一个 `Binding`：名字结点、绑定语句、名字所在的作用域、`local` 的可见起点）管三件事：单元叫什么（`functions::name_of`）、名字住在哪个作用域（`callees::container_of`：语句所在的块，或字段所在的表）、Lua `local` 从哪个字节起可见——`local f = function() f() end` 里的 `f` 是此前那个 `f`、不是它自己（Lua 5.4 手册 §3.5），`local function f` 则先绑名再有函数体（§3.4.11）。没被绑定的函数值自成单元，名 `(anonymous)`（D3：函数值是这两种语言的声明形式，具名与否、嵌套与否都不并入宿主）。R 的 `function_definition` 的 `name` 字段装的是关键字记号，`name_of` 从此只认具名的 `name` 结点。未提及声明顾问为此多报一行 `Binding`：别的模块只经 `binding::of` 拿到它、只读字段不写类型名，而它只能对 crate 可见（`of` 本身 crate 可见），保留——本条写出它的名字即是处置（文档拼写就是提及，册 13 §2），自仓普查仍是零行。
- **形参数**：tree-sitter-r 把逗号做成具名结点 `comma`（十一套文法里只此一家），`ast::entries` 一处把它读作分隔符——形参与实参同一个读法，重载择一比的正是这两个数（设计初稿的「只数某 kind」旋钮因此不设）；两种语言的 `...` 都计一个形参（D32，签名里写出的都算，Python 的 `*args` / `**kw` 同读）。
- **递归弧**（`scan/callees.rs`、`scan/calls.rs`）：成员形名（`M.f`、`M:f`、`x$f`）的对象是 owner，新 `Owner::Table`——`M.f()`、`self:f()` 与 `M` 的成员体内的 `M.g()` 到达 `M` 自己的成员；但表不是类，`M.a` 里的裸 `b()` 是块里看得见的那个 `b`，永不是 `M.b`（`Owner::is_class` 只对 C++ / Java 的类型体为真）。Lua `local` 只被声明之后的调用看见（`Named::visible`）：`local f = function() f() end` 不是递归，写在下面的 local 对上面的函数尚不在作用域。
- **可见性**（`fourclass/visibility/lua.rs`、`r.rs`，只读声明所在的文件）：Lua 的 `local` 对别的文件隐藏，全局、表成员、表字段都算出口（模块经 `require` 交出的就是它返回的表）；R 用 roxygen 的文件（顶层有 `#'` 注释）按块上的 `@export` / `@exportS3Method` 标签定出口，不用 roxygen 的按 R 自己的约定：点开头的名字隐藏。`NAMESPACE` 是另一个文件、永不读，所以 roxygen 文件里只在 `NAMESPACE` 手写导出的名字会读成私有——这种读法唯一漏掉的形状（D16，`r.rs` 头注写明）。bit 1 = 外面没有函数包着。
- **提及与约定**：Lua 的成员名按最后一个 `.` 或 `:` 取末段（`M.f`、`M:f` 以 `f` 被提及）；R 的 `x$f` 与 `print.foo` 保持整体、落到记号不变量。Protocol 名表：Lua 的元方法（手册 §2.4 与标准库读的 `__name` `__pairs` `__metatable` `__mode`）与 Neovim 插件管理器调用的 `setup` / `config` / `on_attach`，LÖVE 回调只在 `main.lua` / `conf.lua` 里算；R 的 Shiny `server` / `ui`（与旧拼写 `shinyServer` / `shinyUI`）和 golem 的 `run_app`——R 自己调用的 `.onLoad` 一类以点开头，本就不进提及域。测试名：busted 的 `_spec.lua` 与 `_test.lua`，testthat 的 `test-` / `test_` 两种前缀、两种扩展名大小写。`listed`（空白分隔的名表）提给死代码的入口名表共用。Lua 与 R 的表让 `mention/conv/name.rs` 过了 300 行：各语言的 Protocol 名表连同读它们的 `protocol` / `ts_protocol` 与共用的 `listed` / `starred` 拆到新 `mention/conv/protocol.rs`（拆后 203 + 154 行），测试文件的路径一半留在原处；新文件是叶子——`name.rs` 读它、它不读 `name.rs`，两者不成环。多了 Lua 与 R 两臂，`protocol` 的圈复杂度从 12 升过 15 的警告线，拆成每语言一行（名表，前缀）的 `tables` 与按文件名和位置判的 `by_file`（三个函数圈复杂度 3 / 7 / 9）。
- **自提及区域**（`mention/selfref.rs`、`selfref/doc.rs`、`doc/blocks.rs`）：Lua 与 R 的每个字符串都算（Lua 长字符串、R 原始字符串 `r"(…)"` 在内——`_G["name"]`、`get("name")`、`do.call("name", …)` 与别的语言的字符串同一个理由）；文档里被文档工具当代码渲染的段：LDoc 的 `@usage`、roxygen 的 `@examples` 与 `@examplesIf`（`R CMD check` 真的会跑它们），段落到下一个标签为止。Lua 的每条注释都是文档行（LDoc 的 `---` 块在普通 `--` 行上延续），长注释按等号层级剥掉 `--[==[ … ]==]`；R 只读 roxygen 的 `#'` 行。行注释与块注释两个读者各自重写的「剥掉起始标记」合成 `past_marker` 一处。
- **引用站点** `CallSite` 表（`graph/spec.rs`、新 `graph/sites/call.rs`）：两种语言都没有 import 语句，引用是调用——Lua `require`（按模块名）与 `load`（`dofile` / `loadfile` 按路径），R `source`（`source`、`sys.source`）与 `library`（`library` / `require` 读不加引号的包名，除非传了 `character.only` 且不是字面 `FALSE`；`requireNamespace` / `loadNamespace` 求值实参，那里的裸名是变量）。每行只写 callee 名、实参名与标签；哪个结点是调用、callee 挂在哪个字段读 LangSpec 的 `call_kinds` / `call_fields`（递归弧读的同一拼写）——设计初稿写的是每语言一条 `CallArg`，两条同形臂被查重门点名后收成一张表。R 先按名匹配实参再按位置；目标必须是字面文本，拼出来的实参（`require(prefix .. name)`）不开站点；限定的 callee（`base::source("x.R")`）不是调用站点，它的 `base::` 是一个 `library` 站点——R 唯一不是调用的站点是 `pkg::name` 运算符。`store::KINDS` 加 `require` `load` `source` `library` 四个标签（`GRAPH_REV` 仍是 16：步 2 没有随任何发布出去；`graph/store.rs` 本就过 300 行，版本说明折回原来的八行，只长四个标签行）；阶梯在提交 B 之前对 Lua 与 R 答 `Unresolved(Unsupported)`。
- **入口角色**（`graph/deadcode/flags.rs`）：按名字——LÖVE 的 `main.lua` 与 `conf.lua`、Shiny 的 `app.R` / `ui.R` / `server.R` / `global.R`，Neovim 的 `init.lua` 只在仓库根算（别处的 `init.lua` 是模块自己的文件，`require "a"` 读 `a/init.lua`，图够得着）；按位置——`ENTRY_DIRS` 一张表按语言分行：Neovim 按路径加载 Lua 的运行时目录（`plugin/` `ftplugin/` `indent/` `syntax/` `colors/` `compiler/` `ftdetect/` `lsp/` `after/`；`autoload/` 是 Vim script 的、`lua/` 是 `require` 的，都不算），R 包里被 R 与工具按路径运行的 `inst/` `vignettes/` `data-raw/` `exec/` `demo/`，别的语言的文件放在这些目录里不算。走查内置排除加 `lua_modules/`（`luarocks init` 建的项目树，`node_modules` 的孪生）与 renv / packrat 的项目库 `renv/` `packrat/`（`scan/walk.rs` 本就过 300 行，说明折进原有的三行注释，只长三个排除项）。
- **考题冻结**（设计册 §14 第 14、17 条；`docs/EVAL-SET-LANGS.md` 新两节）：用户裁「两个都加」——两种语言从一开始各取两个语料，一个包、一个应用：包按模块名找到自己的文件，应用还按路径，一个语料考不全一门语言的站点类。Lua：`lang-slice-luarocks-v1.json`（luarocks/luarocks@2d2cc8e，162 文件 / 607 站点，全是 require）、`lang-slice-koreader-v1.json`（koreader/koreader@d9cd278，594 文件 / 5,025 站点：require 4,938、load 87；AGPL-3.0，冻结档只记路径、哈希、计数与 spec 片段）；样本 `lang-sample-lua-v1.json` 主 100 题（require 84 / load 16）+ 备用 40，落在 87 个文件上。R：`lang-slice-stringr-v1.json`（tidyverse/stringr@ae054b1，67 文件 / 55 站点，全是 library）、`lang-slice-covid19model-v1.json`（ImperialCollegeLondon/covid19model@fcc30e2，177 文件 / 1,697 站点：library 1,647、source 50）；样本 `lang-sample-r-v1.json` 主 100 题（library 84 / source 16）+ 备用 40，落在 62 个文件上——stringr 只抽到 2 题（期望 2.7）：抽样按站点人口、不设语料地板，这是预登记的规则，包的读法另由精度册的宇宙台账逐站点检验。考题表 `EXAMS` 加两行（`audited` / `scored` 都还是 false，盲评与判分在后两个提交）；`eval_support::lang_of` 改读产品自己的路径表。
- **对拍**（`contracts/fixtures/crosscheck/lua/` 取 luarocks 五文件、`r/` 取 stringr 五文件，与考题同一 tip；`SOURCES.md` 两行、`DIVERGENCES.md` 新节）：对照物 lizard 1.23.0 的 CCN 与形参数——两种语言都没有认知复杂度对照物，CoC 的执行者是白皮书电池。**lizard 有 R reader**（`lizard_languages/r.py`）：设计册钉版时核对 reader 列表漏了它，把 R 记作「无外部对照」（D0），对拍时发现并启用，D0 改写为只管 R 的 CoC。Lua：lizard 22 行、ce 28 单元，join 22 条 **CC 22/22、形参 22/22**；ce 独有 6 条全在两个 busted 测试文件里——lizard 的 Lua reader 继承 Ruby 的状态机，为 RSpec 写的规则把 `it` 当作 `it … do` 块的开头，Lua 的 `it("…", function() … end)` 永远等不到 `do`，此后的记号全被吞掉。R：lizard 43 行、ce 44 单元，join 43 条 **CC 34 条一致**，9 条差全部归因：`switch(…)` 5 条（D8）、原生管道 `|>` 1 条（lizard 的分词没有 `|>`，切成 `|` 与 `>` 再按向量化的 `|` 计分支）、lizard 在函数体里遇到 `name <- function` 就结束宿主 3 条；ce 独有 1 条是 `lapply(…, function(i) …)` 的匿名函数（D3）。形参 R 28 条一致、15 条差全在 lizard 一侧：它不计 `...`，还把形参表截在默认值里第一个 `)` 上（D32）。查重校准记录的 fixtures 行随这十个样例重量（`contracts/eval/dedup-distinct-v1.json` 与 `DEDUP-CALIBRATION.md`，`--ignored regenerate`）：文件 35 → 45、块 218 → 240，新增 22 块的 distinct 都在 7 以上，出厂下限一块不抑制（distinct ≤ 6 仍是 34）；四个外部语料行逐字不变。
- **旧二进制对拍**（旧 = ced7ea8 编出的程序，新 = 本树）：九份对拍语料副本（七份既有 go / python / rust / typescript / c / cpp / java，加新的 lua 与 r）各跑十个报告面，90 面里 71 面逐字节相同——七份既有语料 70/70，七种既有语言零判决改动；第 71 面是 lua 的 `docdup`，两个版本都是 0 段（luarocks 的 `.lua` 是 Teal 源码 `.tl` 的编译产物，五个文件里带 `--` 的行共 10 行，没有一段够 50 词的准入线）。其余 19 面按意图移动：旧程序不读 `.lua` / `.R`（`scan` 0 个文件，`check` 在空索引上按名拒绝、退 2），新程序读到 Lua 5 个文件 28 个单元、30 个 `require` 站点，R 5 个文件 44 个单元、2 个文件里 10 个 `library` 站点、8 段注释够 50 词的准入线、10 对候选送核判（0 对重复），`check` 分别 843 与 923。
- **电池**：子仓 `it/coc_lua.rs` 与 `it/coc_r.rs`（各六行指标 + 一行 `units=` 单元与形参）、`it/coc_recursion.rs` 的 Lua 与 R 端到端各一例（经核：Lua 的 `local function` 看得见自己、`local shadow = function` 看不见，`M.` 与 `self:` 到达本表成员，表不是类所以 `M.a` 调裸 `b()` 不成环；R 的表达式 else 与自调用各 +1、`obj$` 成员、互递归一对各付一次）、`unit/scan/binding.rs`（命名读法四块）、`unit/graph/sites/call.rs`（站点两块：三种 `require` 写法与长字符串、拼出的实参与对象上的 callee 不开站点；R 按名先于位置、`character.only`、`requireNamespace` 的裸名、`pkg::`、原始字符串、跨行调用）、`unit/fourclass/visibility/tests_lua.rs` 与 `tests_r.rs`、`unit/scan/calls.rs` 与 `calls_scope.rs` 的 Lua / R 块（local 的可见起点、表不是类、R 成员只到自己的对象）、`unit/graph/deadcode/flags.rs`（名字与位置决定的角色改成一张 `路径 ⇒ 字母` 文本表，C 与 Java 的旧元组行并入）、`unit/mention/conv/name_tests.rs`、`unit/mention/name.rs`、`unit/mention/selfref_tests.rs`、`unit/scan/lang.rs`——其中 `unit/mention/name.rs` 与 `unit/scan/lang.rs` 各有一个测试函数过了 50 行：前者的 44 个用例按语言代际拆成两个测试、共用一个 `check`（写成常量表或文本表都会与邻近测试逐记号同形，查重门两次点名），后者的路径一半拆成独立测试 `paths_reach_their_languages`。
- **文档**：README 双语判决语言句加 Lua、R（`count:langs` 十 → 十二、`count:grammars` 九 → 十一随 facts 门重签）；架构图 IR「eleven grammars / 十一套语法」重渲（docs 与 site 孪生）；设计册 §1 R 行的对照物、§2 走查排除、§4 名字 / 形参 / 嵌套 / 作用域四格、§5 D0 改写 + D3 补 Lua / R + 新行 D32、§8 调用站点与入口角色、§9 Protocol 与自提及、§11 lizard 的 reader 列表、§12 步 7 的清单补上技术栈图四张（判决两行至今仍是 1.7.4 的七种，仅扫描一行还列着 HTML）、§13 行 4、§14 第 17 条、(b) 形参旋钮不设；`spec_r.rs` 与 `it/coc_r.rs` 头注里「R 没有外部对照」一句随 D0 改写；册 13 名表半一段的引文随上面的拆分改瞄（Protocol 两条改指 `protocol.rs`，`main` 一条补全为 Python / Haskell / C / C++ / Java，Protocol 句补上 v2.30 四种语言的表，引文台账以 `CE_DROP_VANISHED` 点名退役旧锚）；册 06 冻结站点种类十四 → 十八（加 `require` `load` `source` `library`，rev 16 一句同补），角色表按现码重写（入口名补上 C / Java / Lua / R，入口目录按语言分行，测试名补上各语言运行器自己的表，补上步 2 漏记的编译单元角色 8，表内行号全部改瞄），核 `roleBits` 与「没有阶梯的语言答 `Unsupported`」两条引文原本各指偏了几行（后者指到 Rust 那一臂），改指定义本身；册 13 自提及区域一句补上 Lua 与 R，测试名一句补引 `RUNNER_TESTS` 表；五条区间里被插了行的引文把终行改对，拒绝原因表那条补回早先就漏掉的末项 `Empty`——引文门只按旧跨度平移终行，看不见区间里多出的行。
- **计划**：计划书横幅与 §6 T 轨步 4 记「提交 A 已交付」，站点改记 `CallSite` 表、R 形参旋钮不设、两语言考题各两语料。
- **查重预算 54 → 59 具名入账**（`ce.toml` 台账）：本批落下十行、收回五行。在机制上收回的三行：上面的 `CallSite` 表（两条同形臂 51 与 56 记号）、`past_marker`（50）、`callees::owner_key` 改读 `functions::owner_of`（54）；一行自己来又自己走（`store::KINDS` 长进 `walk::BUILTIN_EXCLUDES` 的链距内，三个新排除又把它拉长出去）。留下的五行都是 LangSpec 表与表同形（50–104 记号）：两套文法把结点起了同样的名字（`for_statement body`、`parameters`）是两条事实，共用常量会把 Lua 的表绑在 R 的文法上——v2.24 那条台账记过的记录面第六次出现。试过用 `..BARE` 缺省底座还账并实测：59 → 57、五行里三行还在（`spec_c.rs` ↔ `spec_lua.rs` 仍 90 记号），而且带底座的结构体字面量不再逐字段写全，拆掉的正是这些表存在的理由——LangSpec 加一个字段就逼每种语言的表做一次决定（步 3 那样加过四个）。不采。与 HEAD 工作树对拍行集：恰好这五行是新的，没有旧行消失。
- 门：主 check 945 / dedup 59 / scan 84 warn 0 fail（deadcode 0 / docdup 0 / erase 0），子仓 982 / 119 / 43 warn 0 fail；lib 393 / it 431 (15 ign)——提交前的全量跑是 427 绿 4 红：查重校准的 fixtures 行（随即重量，单跑绿，见对拍一条），与三条出处门对本提交才加的文件（`lang-sample-lua-v1.json`、`graph/sites/call.rs`）报 `never committed`——它们读 git log 定首提交，是提交序的产物，提交后单跑与 CI 的全量是执行者；clippy（cli 全目标）+ fmt 清；核未动，`cabal test` 免跑。本批自己的门先抓出五处越线，五处都改掉、没有豁免：`mention/conv/name.rs` 328 行（拆出 `protocol.rs`，见上）、`protocol` 圈复杂度过 15（见上）、`graph/spec.rs` 的 `sites` 64 行（原本就过线，步 2–4 各加一臂：十五个表行改成一行一个的 `site!(结点, 标签, 取法)`，回到 44 行，值与顺序不变——是宏而不是 const fn，因为 match 臂里的 `&[…]` 要提升成 `'static`，行必须是常量表达式，而 const fn 调用从不提升）、子仓两个测试函数过 50 行（一个本批才过线，一个原本就过线、本批又长了四行；见电池一条）；`graph/store.rs` 与 `scan/walk.rs` 本就过 300 行，只长必须的代码行。ADR-006 具名重立——主根：`cli/src/graph/spec.rs` 189 → 263 超容差（cap 199）、`cli/src/scan/callees.rs` 187 → 233（cap 197）、`cli/src/mention/selfref/doc/blocks.rs` 190 → 219（cap 200）、`cli/src/graph/deadcode/flags.rs` 120 → 146（cap 130）、`cli/src/mention/selfref/doc.rs` 125 → 145（cap 135）、`cli/src/scan/calls.rs` 269 → 280（cap 279）、`cli/src/scan/functions.rs` 258 → 269（cap 268）、`docs/EVAL-SET-LANGS.md` 145 → 189（cap 155）、`CHANGELOG.md` 627 → 647（cap 639），七个新代码文件入基线（冻结考题档是 JSON、对拍夹具在走查排除里，都不在度量之内），查重的五个新成员（上面那五处 LangSpec 表）与 `knobs_digest`（查重预算 54 → 59）随之重签，softLine 357 → 362；子仓：`unit/graph/deadcode/flags.rs` 63 → 96（cap 73）、`it/coc_recursion.rs` 193 → 228（cap 203）、`unit/mention/name.rs` 52 → 79（cap 62）、`unit/scan/calls_scope.rs` 117 → 142（cap 127）、`it/eval_lang_parts/mod.rs` 258 → 286（cap 268）、`unit/mention/conv/name_tests.rs` 180 → 203（cap 190）、`unit/mention/selfref_tests.rs` 134 → 155（cap 144）、`unit/scan/lang.rs` 65 → 78（cap 75）、`unit/scan/calls.rs` 202 → 214（cap 212），六个新文件入基线，softLine 307 → 300。

**无默认档位变更。** 语言扩展 v2.30 步 4 Lua + R 提交 A′：盲评真值冻结（2026-09-25；只加冻结工件与它的门，判决代码字节零变化）：

- **真值表** `contracts/eval/lang-review-luarocks-v1.json`（10 行）、`lang-review-koreader-v1.json`（90 行）、`lang-review-stringr-v1.json`（2 行）与 `lang-review-covid19model-v1.json`（98 行）：八个独立 Opus 代理各判一批 25 道主样本（两种语言各四批，按样本的审阅序切批），只读两个语料在钉住 tip 的克隆和自己那一批，没看过产品的任何解析结果、没跑过 ce；装配逐字照录，判决不动。200 道的 spec 全在记录的行上，零失配，没有动用备用题。Lua：语料内文件 88、external 12；R：external 82、语料内文件 15、包目录 3（`covid19AgeModel`）；两种语言都没有 `ambiguous` / `dynamic` / `none`。判词的依据、代理记下的约定与 22 条候选漏检见 `docs/EVAL-SET-LANGS.md` 两节「真值」。
- **宇宙外的候选漏检**：两个代理都记下了 luarocks 命令行启动脚本 `src/bin/luarocks`（没有扩展名的 Lua 脚本）里的 7 处 `require`。按扩展名走的冻结宇宙看不到这个文件，所以它们不算本册的候选漏检；审阅表加一栏 `scope_gaps` 照录，与 `site_gaps` 分开。
- **门**（子仓）：`it/eval_lang_parts/review.rs` 的包目录真值由「只给 Java 的按需导入」改为一张 `PACKAGE_KINDS` 表，按站点类说包的代码在目录下哪里——Java 的按需导入是直接装着该包冻结文件的目录，R 的包装载是包根、其 `R/` 下直接有冻结文件，`.` 是语料根；`check_gaps` 分两栏核：`site_gaps` 必须落在冻结文件上，`scope_gaps` 必须落在冻结宇宙之外（Java 的两张表早于这一栏，没有它）。`it/eval_lang.rs` 原来只瞄 jsoup 的「包目录真值挪到别类行」一腿，改成通用的 `a_misplaced_package_truth_is_refused`：每张含包目录真值的表（jsoup、covid19model）先确认原表通过，再验「挪到别类行」与「用包的代码目录冒充包」两种伪造都被拒；新腿 `a_gap_on_the_wrong_side_is_refused` 验两栏互换都被拒。两处反向探针（去掉栏位判定；让本身直接装着冻结文件的目录也算包）各让对应的腿变红，源码还原逐字节相同——第一轮探针先抓出通用腿没确认原表通过：原表坏了时伪造品也被当成「被拒」，补上这一句后第二处探针才变红。考题表 Lua、R 的 `audited` 翻为 true。
- 门：主 check 945 / dedup 59 / scan 84 warn 0 fail（deadcode 0 / docdup 0 / erase 0），子仓 982 / 119 / 43 warn 0 fail；lib 393 / it 433 (15 ign，新增两条是包目录真值与两栏漏检的拒绝腿)——提交前的全量跑是 431 绿 2 红：`lang_provenance` 两条对尚未提交的审阅表报 `never committed`，与步 3 的提交 A′ 同是提交序的产物，提交后单跑复核；clippy（cli 全目标）+ fmt 清；判决代码与核未动，`cabal test` 不涉及。ADR-006 具名重立：主 `docs/EVAL-SET-LANGS.md` 189 → 243 超容差（cap 199，长出来的是两节「真值」），`CHANGELOG.md` 647 → 654 容差内同定，softLine 362 → 366；子 `it/eval_lang.rs` 206 → 277（cap 216）、`it/eval_lang_parts/review.rs` 118 → 143（cap 128）超容差，`check_gaps` 的认知复杂度 1 → 5 与判包目录的那个闭包 0 → 1 容差内同定，本提交新写的十五个函数与闭包入基线；两仓克隆成员集不变（0 added / 0 removed）。

**无默认档位变更。** 语言扩展 v2.30 步 4：Lua 考题升为第二代（2026-09-25；用户裁「现在支持」——受保护的加载 `pcall(require, "x")` 读作一次加载。Lua 的站点宇宙因此变大，按考题协议整门考题换代、重新盲评，阶梯仍在其后；判决面只是 Lua 多读了这些站点，分数本就与 1.7.4 不可比）：

- **受保护的调用**（`graph/spec.rs` 新表 `LUA_PROTECTED`、`graph/sites/call.rs` 的 `unprotected`）：`pcall(f, …)` 调用 f、把错误交回而不抛出，是可选模块的惯用写法（`local ok, m = pcall(require, "x")`）。检测器把受保护的调用读成它保护的那次调用：包装的第一个实参就是被保护的函数，它的文本按各行的裸名匹配（`pcall(m.require, "x")` 拼不出任何一行的名字），函数自己的实参跟在前导实参之后——`pcall` 一个（函数），`xpcall` 两个（函数与消息处理函数；Lua 5.2 起与 LuaJIT 把其余实参传下去，5.1 的 `xpcall` 不传，手册 5.1 / 5.2 / 5.4 与 LuaJIT 扩展页逐一核过）。标签不变（`require` / `load`）；R 的调用表没有受保护的写法，`LUA_PROTECTED` 只在 Lua 一臂。初稿对第一个实参另加了一道「必须是裸名结点」的过滤，反向探针证明它是死逻辑（非裸名的文本本就不等于任何裸名），删掉；留下的三处探针（`xpcall` 只跳一个前导实参、`pcall` 一个不跳、Lua 不设包装）各让电池变红，源码还原逐字节相同。电池 `unit/graph/sites/call.rs` 加一块：两种包装各自的读法，与拼出来的目标、对象上的函数、没有实参的包装、没有哪一行叫这个名字的函数四种不开站点。
- **考题换代**（`it/eval_lang_parts/mod.rs` 的 `Exam::generation`）：顺序门读一份档的**首个**提交（`intro_commit`），原地重写的样本与审阅表会留着第一代的提交、把第二代的先后证明成第一代的，所以重冻结改为代数加一、用新文件名。第一代五份档按名退役：`lang-slice-luarocks-v1.json`、`lang-slice-koreader-v1.json`、`lang-sample-lua-v1.json`（冻结于 9d28d6b）与 `lang-review-luarocks-v1.json`、`lang-review-koreader-v1.json`（冻结于 8f823c0）。第二代三份入册：luarocks 702 个站点（第一代 607，多出的 95 个里 73 个是 Teal 编译产物首行的 `compat53.module` 兼容前言）、koreader 5,058（5,025；require 4,967、load 91）；多出的 128 个逐文件对过受保护加载的字面写法，没读成站点的 6 处全是不该读的（拼出的模块名 3、字符串里的生成代码 2、块注释 1）。样本配额不变（require 84 / load 16），97 道主样本与第一代相同，3 道新进、3 道被挤出。Java 与 R 的四份站点宇宙用新检测器重生成，除 `generated_from` 外逐字相同。Lua 的 `audited` 回到 false：第二代的真值在下一个提交重新盲评。
- **门**（子仓）：考题的档由 `Docs` 家族（宇宙 `SLICES`、样本 `SAMPLES`、审阅表、精度册）按考题与代数取路径与文件——`eval_support::eval_doc_path`（git 读的仓库相对路径）与 `eval_doc_v`（门打开的文件）是同一条命名规则；`frozen_docs` 与 `doc_suffix` 认任何代数，一个家族每个语料只留一代，退役的一代留在树里就读成那个语料出现两次（G10）。`lang_slices_consistent` 加一条「宇宙在考题的代数上」，审阅表加上冻结集门（此前只有旗标与盘面相符一条）；`assert_docs_postdate_audits` 改收文件而不是词干，M5 的出处门同读。两处反向探针：把退役的第一代宇宙与审阅表放回树里，两条 G10 各自变红并点名多出的那个语料。`a_gap_on_the_wrong_side_is_refused` 原本瞄 luarocks 那张同时有两栏候选漏检的表，那张表随第一代退役，改成取第一张有站点漏检的已审阅表、在旁边放一条宇宙外的漏检。
- **拆分**：审阅表的门离开 `it/eval_lang.rs` 成为 `it/eval_lang_review.rs`（与 `eval_lang_precision.rs` 同理：宇宙与样本、审阅表、精度册各一个门文件），三条门共用一次「每张已审阅的表连同它的考题与样本」的遍历 `audited_tables`——初稿在新的 G10 门里又写了一遍那个双层循环，被查重门点名（子仓 120 > 119）；抽样算法离开 `eval_lang_parts/mod.rs` 成为 `draw.rs`。两个文件本批都过了 300 行（304 / 311），拆后 148 + 169 与 235 + 85。
- **文档**：`docs/EVAL-SET-LANGS.md` 的重冻结规则改为按代数，三处档名改成 `-v<代>.json`，CI 门一条补上代数与审阅表的冻结集，Lua 一节加「第二代：受保护的加载」一段（第一代的记录原样留着）；设计册 §8 Lua 调用站点一行补上受保护的调用；`contracts/fixtures/crosscheck/SOURCES.md` 的 Lua 行改指第二代宇宙；计划书横幅与 §6 T 轨步 4。
- 门：主 check 945 / dedup 59 / scan 84 warn 0 fail（deadcode 0 / docdup 0 / erase 0），子仓 982 / 119 / 43 warn 0 fail；lib 393 / it 433 (15 ign，条数不变：拆分只挪测试，审阅表的冻结集门长在原有的校验里)——提交前的全量跑是 431 绿 2 红：`lang_provenance` 两条对尚未提交的 `lang-sample-lua-v2.json` 报 `never committed`，与提交 A′ 同是提交序的产物，提交后单跑复核；clippy（cli 全目标）+ fmt 清；核未动，`cabal test` 不涉及。ADR-006 具名重立：主 `docs/EVAL-SET-LANGS.md` 243 → 278（cap 253）、`cli/src/graph/sites/call.rs` 96 → 117（cap 106）、`cli/src/graph/spec.rs` 263 → 283（cap 273）超容差，`CHANGELOG.md` 654 → 663 容差内同定，新写的三个函数入基线，softLine 366 不动；子 softLine 300 → 299，两个新文件 `it/eval_lang_review.rs` 169 与 `it/eval_lang_parts/draw.rs` 85 入基线，搬走的函数按新路径换键（路径是成员身份的一部分），十个文件的行数都在容差内（`it/eval_lang.rs` 277 → 148、`it/eval_lang_parts/mod.rs` 286 → 235）；两仓离散集与克隆成员集不变。


**无默认档位变更。** 语言扩展 v2.30 步 4 Lua 考题第二代提交 A2′：盲评真值冻结（2026-09-25；只加冻结工件与一个旗标，判决代码字节零变化）：

- **真值表** `contracts/eval/lang-review-luarocks-v2.json`（12 行）与 `lang-review-koreader-v2.json`（88 行）：四个新起的独立 Opus 代理各判一批 25 道主样本（按样本的审阅序切批），只读两个语料在钉住 tip 的克隆和自己那一批，没看过第一代的表、没看过产品的任何解析结果、没跑过 ce（派卷前查过两个副本都没有 `.ce/`）；简报只比第一代多一句「受保护的调用装载的就是不受保护时装载的那个文件」。装配逐字照录，判决不动。100 道的 spec 全在记录的行上，零失配，没有动用备用题；语料内文件 88、external 12，没有 `ambiguous` / `dynamic` / `none`。
- **盲评的噪声读数**：97 道两代共有的题（秩是站点自身的哈希，同秩即同站点），两批互不相识的代理判词 97/97 相同、判到的文件逐题相同。3 道新进的题：luarocks 两处 `compat53.module` 前言由两个不同的代理各自判 `vendor/compat53/module.lua`，并各自记下同一条保留意见（`make bootstrap` / `--with-system-rocks` / busted 测试环境装的是另装的 compat53 rock）；koreader `frontend/userpatch.lua:7` 的 `pcall(require, "android")` 判 `external`（Android 启动器子模块提供，未检出）。候选漏检 5 条（三条 compat53 前言第二代宇宙已读到，判分时按宇宙核实即销）+ 宇宙外 4 条，逐条见 `docs/EVAL-SET-LANGS.md`「第二代真值」。
- 考题表 Lua 的 `audited` 翻回 true（子仓 `it/eval_lang_parts/mod.rs` 一个词）；审阅表的校验门与冻结集门就此覆盖第二代的两张表。
- 门：主 check 945 / dedup 59 / scan 85 warn 0 fail（deadcode 0 / docdup 0 / erase 0；多出的一条 warn 是 `docs/EVAL-SET-LANGS.md` 过 300 行），子仓 982 / 119 / 43 warn 0 fail；lib 393 未动（`cli/src` 本提交零改动）/ it 433 (15 ign)——提交前的全量跑是 431 绿 2 红：`lang_provenance` 两条对尚未提交的两张 v2 审阅表报 `never committed`，与 A′ 同是提交序的产物，提交后单跑复核；clippy（cli 全目标）+ fmt 清；核未动，`cabal test` 不涉及。ADR-006 具名重立（主根）：`docs/EVAL-SET-LANGS.md` 278 → 308 超容差（cap 288，长出来的是「第二代真值」一节），`CHANGELOG.md` 663 → 671 容差内同定，实体零增零减；子仓无棘轮移动。

**无默认档位变更。** 语言扩展 v2.30 步 4 Lua + R 提交 B：引用阶梯（2026-09-25；Lua 与 R 文件从此在图上有边，它们的死代码、结构与分数随之移动——判决宇宙在提交 A 已变，**分数与 1.7.4 不可比**；既有语言零判决改动，对拍见下。判分与精度册在下一个提交 B′：顺序门要精度册的 `generated_from` 严格晚于每张审阅表的首个提交，而 Lua 第二代的两张审阅表就在 HEAD〔f473c39〕——在本提交的树上生成的档与它同一个提交，门按名拒绝；阶梯先落地，下一个提交在它之上生成四份档、翻 `scored`）：

- **阶梯 `graph/ladder/lua.rs`**（设计册 §8 Lua 行按落码改写）：`require "a.b"` 是 `package.searchers` 找到的那个文件（Lua 5.4 手册 §6.3）——先看 `package.loaded` 已有的名字，再逐个试 `package.path` 的模板。R1 模块在每个搜索目录下的路径 `a/b`，先 `a/b.lua` 再 `a/b/init.lua`（标准序）：仓根、`src`、`lua`（luarocks 与 Neovim 的布局）、声明的 `[graph.search_roots] lua`，外加树里文件自己写进 `package.path` 的字面模板（下条）；两个目录命中不同文件 = ambiguous_root——一次运行先试哪个模板取决于路径是怎么拼起来的，不是文本事实。R2 `dofile` / `loadfile`（标签 `load`）：引用文件同目录，再仓根，取第一个（路径相对工作目录，脚本自己的目录或项目根是惯例）。R3 External：标准库与 LuaJIT 内建 18 个名字（`string` `table` `os` … `ffi` `bit` `jit.util` `string.buffer`），**先于搜索**——`package.loaded` 先答，树里有同名文件也轮不到它。其余 out_of_scope：rock 或 C 库提供的模块、运行时算出来的目录、空段的名字（`a..b`）——漏答，从不猜边。
- **`package.path` 模板 `graph/ladder/lua_path.rs`**：`package.path = "frontend/?.lua;" .. package.path` 把搜索目录写在了文本里。遍历（`dedup/walkidx.rs`）对每个 Lua 文件读一遍（先按 `package.path` 字面预筛，不含的文件不解析），只认字面文本：赋给拼写为 `package.path` 的目标（多重赋值按位置取值）的字符串、或 `..` 链里的字符串操作数（括号看穿）；调用（`string.format("%s/?.lua", dir)`）与变量不算。模板要相对（无前导 `/` `~`、无盘符与 `$`）、`./` 前缀去掉、恰一个 `?`、以 `.lua` 结尾，反斜杠读作分隔符；`/usr/share/lua/5.1/?.lua` 这种绝对模板不指树里任何文件。并集经 `Scope::lua` 交给阶梯，阶梯自己不读文件；一个文件的模板进 `resolve_key`（模板变了边要重算），没有模板的文件不加键输入，所以没有模板的树键与步 4 前相同。这一读是落码时加的：koreader 的 `frontend/` 只写在它自己的 `package.path` 里，只按默认根设计时它大半的 `require` 指不到文件（读数在设计册 §8）。
- **阶梯 `graph/ladder/r/`**（设计册 §8 R 行按落码改写）：R 没有 import 语句。R1 `source("x.R")`：引用文件同目录，再仓根（RStudio 项目与从根跑的 `Rscript` 共有的工作目录惯例），取第一个；再声明的 `[graph.search_roots] r`（两根命中不同文件 = ambiguous_root）；`source` 一个 URL → External。R2 `library(x)` / `requireNamespace("x")` / `x::`：在域 `DESCRIPTION`（`r/description.rs` 读 Debian control 格式的 `Package:`——一个词，`Packaged:` 是另一个字段——与 `Collate:`，续行以空白起头，引号名可含空格）的 `Package: x` → ResolvedPackage（其目录）；两个包同名 = ambiguous_workspace。R3 External：没有在域包声明这个名字——base R、CRAN 与 Bioconductor 构造上在语料外，不需要名表。绝对路径、`~` 路径与带盘符的路径 out_of_scope。`DESCRIPTION` 进 `store::CONFIG_NAMES`（解析器配置：字节入 `resolve_key`，改了边重算）；**GRAPH_REV 仍 16**（步 2 起同随 1.8.0，`store.rs` 注释同改）。
- **三条阶梯共用 `graph/ladder/paths.rs`**：`one_of`（候选集：零个让下一级问、一个解出、两个 ambiguous_root）、`declared`（声明根逐一 join）、`beside_or_root`（引用文件同目录再仓根）——C 的声明根、Lua 的搜索目录与 R 的声明根用的是同一个判法，Lua 的 `dofile` 与 R 的 `source` 用的是同一个读法，各写两遍是查重门要点的名。`ladder::resolve` 的分派：按站点种类分支的阶梯（Markdown / Java / Lua / R，Rust 早已如此）收 `&Site`，其余只收路径与 spec。
- **角色**（`deadcode/flags.rs`、`deadcode/targets.rs`）：R 包的 `DESCRIPTION` 声明它随包加载的代码——`Collate` 列出的 `R/` 文件（列了而不在的不声明任何东西，没列的不是目标），否则 `R/` 直属的每个 R 文件（嵌套的不算）——入 `ROLE_DECLARED`；每个含 R 文件的目录向上找最近的 `DESCRIPTION`，一份读一次；没有 `Package` 的 `DESCRIPTION` 不是包。R 的入口目录（`inst/ vignettes/ data-raw/ exec/ demo/`）改按每个包根读——`DESCRIPTION` 所在目录之下才算，包外的同名目录不算；其余语言的入口目录仍在仓根。
- **R 包结点只展开到它的代码**（`graph/nodes.rs::contain`；真树对拍抓出）：包结点的合成包含弧此前一律指向目录下的每个文件；stringr 的包在仓根，测试里一句 `library(stringr)` 就让整棵树可达——死文件 40 → 0，`NEWS.md`、`revdep/*.md` 全被算成被引用。现在 `contain` 收 `Declared` 的包表（包根 → 代码），有表的包只到它的 `Collate` / `R/` 直属文件，没表的包（Go、Java 的目录包）照旧到目录下的每个文件：`library(pkg)` 跑的是 `R/`，不是 NEWS.md。子仓 `it/graph_containment.rs` 加一腿：声明了代码的包在根上也只到代码，测试文件、别的包的代码、代码为空的包都不在弧上。
- **电池**：`it/graph_ladder_lua.rs`（十六个文件的夹具树 + 23 行：`main.lua` 写两个模板、`loader.lua` 运行时拼一个〔不算〕、`twin` 在两个默认目录、`both` 一个目录里两形取 `.lua`、`vendor/` 只在 `@rooted vendor` 下解出、`dofile` 同目录 / 仓根 / `../` / 绝对路径 / 缺失、标准库三名）、`it/graph_ladder_r.rs`（根包 + `sub/` 包 + 两个同名包 + 两个声明根共有一名，14 行）、`unit/graph/ladder/lua_path.rs`（12 块：拼接、括号、`./`、长字符串、`;;`、拼写 `package . path` 不算、多重赋值按位置、反斜杠、计算值、四种域外模板、错形、`cpath` / 注释 / 局部变量）、`unit/graph/ladder/r_description.rs`（5 块：续行与引号、无 `Collate`、值起于下一行、只有 `Packaged`、两个词的包名）；三条阶梯的电池改成一份文本（`common::text_ladder`：`==== path` 块是夹具文件，`==== @cases` 与 `==== @rooted <dir>…` 块是两种作用域下的行）——Java 那份原来的「树 + 表 + 每个作用域一个测试」在三个文件里逐记号同韵，查重门先点了名；`unit/graph/deadcode/{flags,targets}.rs` 各加 R 包的行（`pkg/` 下的入口目录与 `R/` 直属文件、包外同名目录不算；`Collate` 三种情形、无 `Package` 的 `DESCRIPTION`、每个包的代码表），后者 57 行的夹具测试把五份清单抽成 `MANIFESTS` 常量回到 50 行以内。
- **配置说明**：`docs/reference/ce-toml.md` 的 `[graph.search_roots]` 一行写明 `lua` 与 `r` 键做什么（`lua = ["vendor"]` 加入 `require` 的搜索目录；`r = ["lib"]` 是 `source` 在引用文件同目录与仓根之后找的地方）。
- **依赖**：`serde_json` 开 `float_roundtrip`——冻结的评测档把存下的比值原样读回（默认解析器尽力而为，偶尔差一个 ULP：`lang-precision` 的 559/607 不开就低一个 ULP）。
- **计划**：计划书横幅与 §6 T 轨步 4 记 B「已交付」、B′ 待交，设计册 §8 Lua / R 行按落码改写、§13 行 4 同改，册 06 包含弧一段补 R 包的例外。
- **对拍**（旧 = f473c39 编出的程序，新 = 本提交）：九个对拍语料（go / python / rust / typescript / c / cpp / java / lua / r）每个十个报告面，90/90 逐字节相同——七种既有语言零判决改动；lua 与 r 两份对拍语料不动在预期之内（同 Java：目录摊平进了文件名，`require "luarocks.core.cfg"` 在摊平的树里指不到文件，`library(stringr)` 答的 External 本就不产生边）。阶梯的作用在四棵考题真树上看（钉住 tip 的干净导出；koreader 去掉五个未检出子模块的声明后才跑得动——`.gitmodules` 声明而未就位的子模块按名拒绝是产品既有立场，两臂同拒）：luarocks（244 结点）保留边 176 → 708、没有边的站点 760 → 220、死文件 117 → 68（unref_public 98 → 38、unref_private 15 → 9、unreach_public 4 → 21），`ce check` 805 → 825、`ce structure` 947 → 799；koreader（602 结点）保留边 8 → 3,921、没有边的站点 5,138 → 995、死文件 474 → 173（unref_public 336 → 51、unref_private 138 → 97，新出 unreach 25），check 733 → 732、structure 952 → 754；stringr（结点 90 → 91，`DESCRIPTION` 成了配置结点）保留边 1 → 37、死文件 40 → 5（`R/` 的 35 个文件按 `DESCRIPTION` 声明进 `ROLE_DECLARED`，剩下的 5 个是 `LICENSE.md`、`NEWS.md`、`cran-comments.md` 与 `revdep/` 两份 md，本就无人引用），check 909 → 946、structure 903 不动（它的边全是包级的）；covid19model（185 → 186）保留边 3 → 135、没有边的站点 1,776 → 1,652、死文件 178 → 64（`covid19AgeModel/` 包的 114 个文件全部离开——`R/` 十个按声明、`inst/` 104 个按包根下的入口目录；留下的 64 个在 `usa/` `Italy/` `nature/` 这些没有 `DESCRIPTION` 的脚本目录里，其中 35 个从「无人引用」变成「有引用但不可达」），check 698 → 749、structure 946 → 907。有了文件级的边，结构分的三条轴（2、3、7）从 0 开始出分，所以三棵树的 structure 往下走。
- 门：主 check 944 / dedup 59 / scan 85 warn 0 fail（deadcode 0 / docdup 0 / erase 0），子仓 982 / 119 / 43 warn 0 fail；lib 395 / it 433 绿 2 红 (15 ign)——两条红是提交序产物：`lang_provenance::lang_audit_precedes_the_ladder` 与 `graph_provenance::audit_precedes_any_resolver` 用 `git log` 取新阶梯文件的首个提交（`never committed`），提交后单跑绿；重立前同一棵树 check 945（size 轴 81，重立让软线移动）；全量跑的第一遍 432 绿 3 红，第三条是子仓 `it/graph_mounts.rs` 头注引 `ladder/md.rs:86` 的行号随本提交漂到 87，重瞄后复跑

## [v1.7.4] — 2026-09-19 — 官网八页的图片取景器、整站元件对齐、how 页目录面板与中文散文不再断在句中；判决请求的 `sim` 表改成一对文件只出一行（两个家族都判过的那一对曾让核拒收整份请求，`ce check` 在那棵树上一个判决也给不出）

**无默认档位变更。** 官网与 README 读者面（2026-09-14，用户四条：架构图要能放大缩小 / 拖动〔参照 AutoShade README〕、README 首张架构图与正文宽度岔开、网页各元件长宽对齐、截图区一大坨难看 + 全面美化）：

- **官网八页第一次带脚本：图片取景器 `site/viewer.js` + `site/viewer.css`**（挂图的六页加载，两个 bench 页不加载）。每张图——两语架构图、判决图、常数总表、技术栈图、三张 GUI 截图——包在 `.stage` 里，脚本给它一台相机：拖动平移，滚轮 / 双指 / `+` `−` 键与按钮条缩放（1×–6×，步进 1.4 几何对称，进 n 步出 n 步回原点），`0` / Reset 复位，全屏，方向键平移；滚轮只在读者点选或聚焦该图后才归它（Ctrl/Cmd + 滚轮例外），贴合时纵向手势归页面（`touch-action: pan-y`）、放大后归图。相机移动的是 `<img>` 自己的布局尺寸与位移而不是 SVG viewBox：一条路同时管矢量图与位图，不把 SVG 内联进页（archify SVG 的 `<style>` 声明 `--panel` / `--text`，内联会覆盖页面自己的记号），引擎对改了尺寸的 SVG `<img>` 重新栅格化故放大仍清晰。相机是纯函数、Node 下导出，`cli/tests/site/camera.js` 钉四条性质（贴合 / 缩放锚点不动 / 夹紧无缝 / 倍率上下界；去掉夹紧的反向探针三条红），`it/site_viewer.rs` 五腿另钉：挂图页与脚本、样式表三者同在同缺，三份缓存文件全站各一个 `?v=`，每个 stage 恰一张带 width/height 的图（相机读它取纵横比），按钮条文字表两语同键各说各话（docs_lang 看不见脚本写的字），README 里每条 `codeeraser.dev/…#id` 深链都落在页上真有的 id。无脚本时退回原样：图在流内按列宽显示、按钮条不建、页签隐藏三张图全显（无头 Edge 拦掉 viewer.js 实测）。
- **README 首张架构图与正文对齐**：`scripts/diagram_svg.mjs` 不再往 SVG 根写 `width` / `height`——只留 viewBox 的 SVG 在 `<img>` 里取容器宽（CSS 2.1 §10.3.2；无头 Edge 实测 896px 容器里 viewBox-only 得 896、带 `width="650"` 的仍是 650），判决图（650 宽）与架构图在 GitHub 上遂撑满正文列；`demo/render.js` 的三张终端卡同改（849 宽曾比列窄约 50px）。四张 archify 图与三张 demo 卡各只改根行一处，判决句零变化（`node demo/run.js` 后 `demo/bless.js` 报 0 blocks rewritten）。两份 README 的两张图下各加一行 `<sub>` 指向官网可缩放的那张（`/#architecture`、`/how/#verdict` 与 zh 对应），仿 AutoShade README 的做法；图上原来那层指向 SVG 文件的链接去掉（GitHub 自己给图加同样的链接）。
- **网页元件对齐**（1280px 下八页每个块 x=184 / 宽 912 逐块实测，此前 hero 里的安装芯片 712、计分板 676）：首屏安装芯片移出 `.hero` 占满列宽；键 / 值芯片改 CSS subgrid——同一叠芯片的键取最宽键的宽度、值从同一条竖线起（此前 `min-width: 7.5em` 让「Homebrew · winget」把自己的值推右 50px），窄屏改键上值下（此前 400px 下值被裁掉）；计分板改占满列宽、行线通栏；卡片网格 4 列改 3 列（12 个判决家族恰 4 行、技术栈 9 张恰 3 行），插件钩子那张不是家族、改横条 `.card.band` 而非第四行孤卡；三处画框（终端 / 图 / 截图）统一为一个 frame（ink 底、1px 边、12px 圆角、说明与按钮条在框内）；纵向过长的判决图（650×760）在 stage 里限高 78vh、居中留边而不再是 1066px 的一堵墙；bench 仪表盘的 p50 / p95 / n 三列由生成器写 `class="num"` 右对齐等宽数字（冻结点表第三列是路径，不受影响）、隔行淡底。`style.css` 265 → 280 行（取景器的规则拆去 `viewer.css` 69 行，两文件都在 300 软线内）。
- **截图区**：三张 1424×892 的窗口此前挤在 449px 宽的 2+1 网格里（第三张孤悬、缩到不可读），改为页签式取景器——一次一张、占满列宽（912×571）、可缩放；无脚本时三张竖排全显。`site_screenshots.rs` 的五腿不动（`<img src width height>` 形与 alt / 说明文字都保留，截图与收据未重拍——`gui/ui` 未动）。
- **how 页开篇**（用户指着自己截图里的那一坨：「How 那里，那一坨你动都没动」——上一条读成了首页的三张 GUI 截图，读错）：三段导语下那行等宽的跳转串（`Families 01 02 … 15 FPR discipline …`，21 个链接一行排开、无层级）换成目录面板 `<nav class="toc">`——一节一行、该节的家族作带编号的药丸排在旁边（家族 01–10 / FPR 纪律 11 / 据判决行动 12–13 / 改动时残留 14 / 同角色顾问 15 / 诚实边界两枚无号药丸「不是安全边界」「fail-open」），与安装芯片同一套两列 subgrid（节名同宽、药丸行同一竖线起），640px 以下节名在上药丸在下；导语首段作提要（正文色、1.18rem），后两段保持次级；两语同改，`style.css` `?v=5 → ?v=6` 八页同 bump。**中文页的半角空格**：Chromium 不实现 CSS Text 3 §4.1.2「两个 CJK 字符之间的换行符删除」，四个 zh 页里硬换行的中文散文在线上句中都带一个空格（「写代码， 会漂向」「追加的 形式落地」，1280px 与 400px 截图都看得见）——95 处 CJK 与 CJK 之间的换行按脚本接成一行（`<pre>` / `<script>` / 生成块逐字节不动；CJK 与拉丁 / 标签之间的换行是站点约定的那个空格，保留），zh how 页 619 → 534 行。**新门 `it/site_contents.rs` 两腿**：目录面板的每个链接落在页上一个 id、每个 `<h2 id>` 与 `<h3 id="fNN">` 在面板里有链接、药丸编号等于它链的家族号、两语面板同一份大纲——旧跳转串没有读者，家族从十二长到十五全靠手记；三个变体探针（悬空链接 / 漏列标题 / 错号药丸）各被点名。
- 门：主 check 944 / dedup 55 / scan 0 fail（deadcode 0 / docdup 0 / erase 0），子仓 983 / 119 / 0；lib 373、it 404 (12 ign)；ADR-006 具名重立（主根）：`site/style.css` 265 → 280 超容差（cap 275），其余 11 文件容差内同定（两首页 175 → 182 / 172 → 179、两 how +2、两 stack +2、两 README 199 → 203、CHANGELOG 664 → 672、`diagram_svg.mjs` 231 → 236、`render.js` 145 → 148）；子仓 2 tolerance drawn 不重立；首页自测块 944 / axes 0:86 → 0:87（多两个 scan 尺寸臂文件）；bless 序 site_roast → docs_citations（册 methodology.md:34 引 README:176 → 180 纯位移重签）→ eval_mention（册 13 自仓行 U 1027 → 1031）→ facts_（无变）。 追加提交（how 开篇）：主 check 945（size 轴 84，zh how 页少 85 行），dedup 55 / scan 88 warn 0 fail / deadcode 0 / docdup 0 / erase 0；子仓 983 / 119 / 0（2 tolerance drawn 不重立）；ADR-006 具名重立（主根）：`site/style.css` 280 → 296 超容差（cap 290），`site/how/index.html` 712 → 723 容差内同定；首页自测块随 945 重 bless。

**无默认档位变更。** 缺陷修复（2026-09-19，在一棵外部的树上撞见：`ce check` 一个判决也给不出）：

- **判决请求里的 `sim` 表改成「一对文件只出一行」**：这张表的线上身份是**文件对本身**（`table "sim" (simRow n) 2`，按身份前缀严格递增；核那边的注释写得很清楚——「同一身份带不同载荷，正是它拒绝放进来的那种漂移」），而 `score::measure` 是把两个家族**各自有序的集合首尾相接**。7.0.0 之前接不出重复（只有克隆家族出行）；7.0.0 起 docdup 也出行，于是**两个家族都判过的那一对文件出现两行**，核按合约拒收整份请求（`contract: sim <i>: not strictly ascending`），`ce check` 在那棵树上什么也判不出来。触发条件只是「两个文件既共享代码又共享散文」，也就是兄弟模块的常态：首个撞上它的仓库里，一个 55 文件的目录就占十对（克隆 30 对 ∩ 文档重复 55 对 = 重合 10 对）。合并放在这张表**收尾的那一处**（`one_row_per_pair`），留**更强**的那一条：排序把 kind 排在文件对之内、kind 枚举本身按强度排（0 t1t2 / 1 t3 / 2 docdup），所以每段留下的是「它是个克隆」而不是「它的散文也像」。今天这个选择改不了任何判决——两行都带各自家族**已核实**的 100/100，kind 只决定拿哪条线跟它交叉相乘（克隆 85/100、docdup 80/100，都过）——所以它由单元测试单独钉住。新门两道（子仓）：`it/check_sim_table.rs` 造一棵「两个文件既是克隆对、又是文档重复对」的树，**先钉夹具自己**（两个家族确实都判到这一对，共同的那段是 75 词的逐字命中），再钉 `ce check` 出判决、`sim` 一行、候选一行；`unit/score/sim_table.rs` 钉合并的三条性质。两处变异各自转红：拿掉合并 → 三条全红，且 e2e 报的正是线上那句拒收；把留存翻成弱的那条 → 只有单元测试红、e2e 仍绿（这正是单元测试要存在的理由）。册 07 补一句记下这条规则。
- 门：主 check 944 / dedup 55 / scan 88 warn 0 fail（deadcode 0 / docdup 0 / erase 0），子仓 983 / 119 / 0（2 tolerance drawn 不重立）；lib 375、it 407 (12 ign)；ADR-006 具名重立（主根）：`cli/src/score/mod.rs` 432 → 462 超容差（cap 442），`CHANGELOG.md` 673 → 677 容差内同定，另两行新函数（`one_row_per_pair` 与它的 dedup 闭包）；重签：两首页自测块 945 → 944（尺寸轴 84 → 85）、引文台账、册 13 自仓普查 U 1032 → 1034。

## [v1.7.3] — 2026-09-12 — 插件 hooks.json 说明键改名 description；dependabot 第二批（interprocess 2.4.4、upload-sarif 4.38.0）随本版发出；判决代码与 1.7.2 相同

**无默认档位变更。** 插件读者面（2026-09-12，用户报 `hooks.json: unknown key "_why_timeout" ignored`）：

- **`plugin/hooks/hooks.json` 顶层那个说明键改名 `description`**：Claude Code 从 2.1.267 起按白名单核该文件的顶层键（`description` / `hooks` / `modules` / `surface`；本机留存的三个二进制实测：2.1.266 无此检查，2.1.267 与 2.1.269 有；其 changelog 未记），不在名单的键每次开会话点名一次后忽略。原键 `_why_timeout` 自 2026-08-19（cd11fda）起未动，装机 1.7.2 那份与仓内逐字节同；改名只换键名，说明文字逐字保留、行数不变。三条钩子的接线从未受影响（键被忽略、钩子照常跑），子仓读该文件的三个门（`face_parity` / `facts/count` / `health_plugin`）只读 `hooks` 键。装机上的副本要等下个版本发出并 `claude plugin update` 后才换，此前告警仍在。ADR-006 具名重立：CHANGELOG 651 → 655。

**无默认档位变更。** dependabot 每周批第二轮（2026-09-12；PR #11 / #12，用户裁「修两行注释后并入」与「现在并、做全套记账」）：

- **PR #12 `github/codeql-action/upload-sarif` 4.37.9 → 4.38.0**：ci.yml 两处 SARIF 上传 action 的 sha `cdf488f5…` → `b96794f0…`（= v4.38.0 tag 解引用，GitHub API 核过）。dependabot 换 sha 不改行尾注释，两行仍写 v4.37.9 / 2026-09-06 / #7——`.github/` 不在度量宇宙、无门可抓——在 PR 分支补一笔注释提交后合并。
- **PR #11 `interprocess` 2.4.3 → 2.4.4**（daemon 的命名管道 IPC 库）：上游只改文档与打包——错字、死链、`checks-and-tests.py` 不再入包（免得发行版打包器以为要 Python）；两版 Cargo.toml 的依赖段逐字节同，windows-sys 要求仍是 0.61。gui 锁文件手动同点 2.4.4（version + checksum 两行，两工作区 `cargo metadata --locked` 过）：`cargo update -p interprocess --precise 2.4.4` 会顺带把七个无关包（anstyle-query、anstyle-wincon、dirs-sys、socket2、winapi-util 等）的 windows-sys 引用从 0.61.2 改到 0.60.2 / 0.59.0——cargo 解锁被点名包的依赖子树、其余包偏好仍锁着的版本——比这次升级该动的宽，不采。NOTICE 一行随之再生。
- **bench 七个面翻成「该有行、tag 后测量」**（README 双语、docs/BENCH.md、两首页、两 bench 页）：`cli/Cargo.lock` 是构建输入（`bench_support/joins.rs` 按内容比、只剔版本戳），锁一动即不再是 v1.7.0 那份被测程序；PR 首跑 CI 的 5 条红（三平台同）正是这四条 bench 门 + NOTICE 门。判决 / 分数算法 / schema id / wire 不变；下个发布按 BENCH.md 入列规则须量一次 bench。
- 门：主 check 944 / dedup 55 / scan 0 fail（deadcode 0 / docdup 0 / erase 0），子仓 983 / 119 / 0，lib 373、it 399 (12 ign；`layout_tree` / `docs_diagrams` 两腿只在度量用的 worktree 里红——前者要机器本地的 `.ccm/`、后者要 `cli/target/archify` 缓存——CI 为准)，两锁 `--locked` 过；ADR-006 具名重立：CHANGELOG 655 → 662（首页自测块 944 / 尺寸轴 86 不动，无需重 bless）。

## [v1.7.2] — 2026-09-10 — GUI 根目录选择器；1.7.1 只到了 Release 与插件的五条渠道在本版补齐

**无默认档位变更。** GUI 读者面（2026-09-10，用户报两条）：

- **根目录有了选择入口**：头部根路径框旁一枚文件夹按钮，点开系统的目录选择器——`tauri-plugin-dialog` 2.7.3，`capabilities/default.json` 只授 `dialog:allow-open`（不取插件的 `default` 集：message / save / ask / confirm 一个不授），`withGlobalTauri` 下插件脚本落在 `window.__TAURI__.dialog`，webview 仍零打包零框架。选中的路径写进根路径框并派发它自己的 `change` 事件，记忆（localStorage）与锚定回显（`resolve_root`）与手打是同一条路；取消（null）不改任何东西；对话框标题与按钮 tooltip 共用一枚 i18n 键 `pickRoot`。子仓 `face_parity.rs` 新腿钉住授权集恰为三条并要求 gui.md 那句「granted `dialog:allow-open` and nothing else」在场。第一方核实：debug 构建的真 app 经 WebView2 远程调试读到 `typeof window.__TAURI__.dialog.open === "function"`；无头 Edge 下桩掉 `open` 驱动按钮——选中即写入、存储、回显，第二次打开的 `defaultPath` 是上次选中值，取消不动。
- **头部一行版图的地板随之重量**（`scripts/measure_header.js`，临时关掉本查询）：英文 1205px / 中文 1082px——2026-09-06 的 1163 / 1040 各加按钮自己的 42px（34px 按钮 + 8px 间距）——英文为约束方 ⇒ `@media (max-width: 1204px)`；1280px 默认窗口下状态列在英文里余 50px。
- **「中文」按钮溢出的第二次报告不是回归**：用户机器上的 GUI 本体是 1.7.0（`ce-gui.exe` sha `a0e84680…`、注册表 DisplayVersion 1.7.0）——`ce update` 只换 ce 与 ce-core，GUI 本体要跑安装包；修复（`white-space: nowrap`，e9323d1）已在 1.7.1 发出。第一方复现：v1.7.0 的 `gui/ui` 在无头 Edge 1280px 下语言按钮盒 31×32、文字盒 13×36（两行各一字，scrollHeight 38 > clientHeight 30）；HEAD 44×32 / 26×17。本批不改码，随本版安装包一并到位。
- NOTICE 随 gui 锁文件重生 +14 行（rfd 0.16.0、tauri-plugin 2.6.3、tauri-plugin-dialog 2.7.3、tauri-plugin-fs 2.5.2、windows-sys 0.60.2 与九个 windows_* 目标 0.53.1）；三张官网 GUI 截图重拍（头部多一枚按钮），收据 `ui` 摘要随之。ADR-006 具名重立：`gui/ui/app.js` 104 → 123（picker 一函数）、子仓 `it/face_parity.rs` 275 → 304（一腿）；`gui/ui/style.css` 269 → 275、`gui/src-tauri/src/main.rs` 40 → 45、`docs/reference/gui.md` 116 → 124 在容差内同定。check 944 / dedup 55 / scan 0 fail 不动，子仓 983 / 119 / 0。

## [v1.7.1] — 2026-09-07 — Windows 上 pin 校验读错自己算出的哈希，插件三钩子与 MCP 面一起静默失效

**无默认档位变更。** 一处分发链路缺陷（2026-09-07，用户报「`.ce` 没建、插件没生效」后第一方定位）：

- **`sha_of` 在带反斜杠的路径上交出的不是哈希**：GNU coreutils 对含反斜杠或换行的文件名做转义，并以行首一个字面 `\` 标记该行，于是 `sha256sum "$1" | cut -d' ' -f1` 读出 `\<64 位十六进制>`——比 pin 多一个字符，与任何 pin 都不可能相等。这在 Windows 上不是假设：`CLAUDE_PLUGIN_DATA` 是原生路径，由它拼出的每个候选都带反斜杠，data 目录里那枚**与 pin 逐字节相同**的 `ce-1.7.0-x86_64-windows.exe` 因此被当成篡改品拒绝（`REFUSING on-disk ce — SHA256 mismatch`），`ce.sh` 随即按 R3 fail-open 退出 0，SessionStart / PreToolUse / Stop 三个钩子与 MCP 报告面**一起静默失效**：没有健康行、没有 `.ce/`、没有守卫，而失效的方式恰好是不出声。
- **改为经 stdin 哈希**（`sha256sum < "$1"`，`shasum -a 256` 同改）：输出里根本没有文件名可转义，两者都只印 `<hash>  -`，反斜杠路径与 POSIX 路径同解。第一方复核：`Get-FileHash` 与清单 pin 同为 `051184…5680f`，改前 `sha_of` 报 `\051184…5680f`、改后报 `051184…5680f`。
- **另一条腿不在本仓，一并具名**（不改码，两份 plugin README 记为前置）：钩子与 `.mcp.json` 都以裸 `sh` 起头，而 Claude Code 是原生 Windows 进程、按 Windows PATH 找第一个 token；Git 装机默认只把 `Git\cmd` 放进 PATH（内有 `git.exe`、无 `sh.exe`），`sh.exe` 在 `Git\bin` 与 `Git\usr\bin`。PATH 上没有 `sh` 时三个钩子与 MCP 都起不来——起不来的正是本该报错的那个脚本，所以同样不出声。

**无默认档位变更。** 两处读者面缺陷（2026-09-06，用户报）：
- **GUI 的语言按钮与中文页签裂成两行**：CSS 允许在任意两个汉字之间断行，中文标签的 min-content 宽度因此只有一个字，flex 子项默认的 `min-width: auto` 不再撑住控件——被挤窄后标签折到第二行，而这些控件各自钉死了 `height`，第二行遂溢出；英文标签词内没有断点故永不折行，这就是只有翻译面出事的原因。一条规则收拢整类（`#tabs .tab, #lang, .bar button { white-space: nowrap }`），不逐个补丁。
- **量它的仪器只问了一个轴**：`scripts/measure_header.js` 判裁切用 `scrollWidth > clientWidth`，而标签是靠折行、不是靠变宽溢出固定高度的，于是它把裂开的按钮读成「装得下」；判据改为两轴同问并把十一个页签逐个纳入，`floor` 的 fits 补 `rows === 1`（不补则二分越过断点走进已换行的两行版图，答出的是那一版的极限 601px 而非断点）。断点按新判据重量（临时关掉本查询以免自指）：英文 1163px、中文 1040px，英文为约束方 ⇒ `@media (max-width: 1162px)`；此前钉的 1150 / 844 是该缺陷的产物——控件当时以裂行代替撑宽，头部遂量得比实际窄 13px（英）与 196px（中）。
- **八张站点图与四张 README 图现在能看原尺寸**：图本就是带 `viewBox` 的真矢量（零位图），缺的只是走出栏宽的路——每张图包一个指向其自身 SVG 的链接，点开即由浏览器无级缩放，站点保持零脚本（`<script>` 计数仍为 0）；锚点与图同行，故十个文件行数零变化。
- **README 双语与官网八页跑了一遍 sci-paper 级去 AI 味 + 精简**（用户令）：按 `sci-paper` 的 de-ai 标准做——L0（破折号）在散文里归零，L1–L4 只作有处置的顾问项。散文破折号 README 52 处 → 0、README.zh 53 处 → 0（按字符数是 79 → 27 与 138 → 32，余数全落在生成块里），官网八页 406 字符 → 38，plugin/README 与 demo/README 另清 42 处（`demo/seed/**` 具名不动——它是重放判决的那棵夹具树，改它的散文就是改 demo 报出来的数）；改法逐句按功能选（成对同位语进圆括号、列表与题注用冒号、并列转分号、长句直接断句），不做一律换成冒号那种以一个 tell 换另一个 tell 的替换。
- **冒号那一轴逐条核过，不是无代价**：`ai_ism_lint.py` 报的 `colon-elaboration` 在英文 README 上 20 → 25（散文区 11 → 16，生成块与表格那 9 条一条没动），在 how 页文本投影上 24 → 36。README 净增的 5 条：2 条是「已知限制」拆成条目后旧句各自计数（`word boundaries:` 与 `softLine:` 改前就在同一段里）、1 条是小标题、2 条引出枚举；how 页新增的 12 条：4 条是小节标题、4 条引出枚举，剩下 4 条是真同位语冒号，按标准记 accepted（题注、标题与列表规格是标准自带的豁免）。改写一度引入的 README 同位语冒号（`honest boundary` 那句）已改回 `, since`。
- **留在原地的破折号都是具名的**（38 字符：27 在两页九条方法学册标题链接里——链接文字就是 `docs/reference/methodology/` 那份文档的标题，只改一面等于把同一个名字劈成两个；2 在两条 `<pre>` 规格行；9 在两首页的 scoreboard 与 bench 生成块）——生成块的字改在生成器，不在页面。两页 `<title>` 由破折号改冒号，`facts_registry.rs::LITERALS` 的两条字面随之同改（子仓）。
- **精简的实测结论：没有可删的冗余**。`condense_map.py` 在 README 上只报 1 条 `condense-restatement`（可删 19 词 = 0.48 %），在 how 页上只报 1 条 `condense-dead:acronym` 且是投影造成的假阳（TSED 在页面上出现三次，两次在 `<pre>` 公式里），首页 0 条；真正缩短的是改写本身——`site/how` 散文 4,122 → 4,046 词，按句末标点切句计 ≥ 45 词的长句 33 → 25 句，其中 136 词那句「三种家族计数」拆成 13 / 15 / 32 / 75 词四句。
- **README 的「已知限制」由一段变六条**：英文原是 3,073 字符 / 20 句的单段，是两份 README 上仅剩的大段文字；改成一句引言加六条具名条目（语言 / 只当顾问 / 不替你画的线 / 分发与接线 / 墓碑残留 / 分数可比性）。每句都是从原段按下标切出来再拼回，脚本断言切片能逐字节重组成原段，故无一事实、数字或芯片经手重打；两份各 192 → 199 行。
- ADR-006 具名重立：`gui/ui/style.css` 254 → 269、`scripts/measure_header.js` 124 → 135（皆为注释块增长）；check 945 / dedup 55 / scan 0 fail 不动，子仓未动。

## [v1.7.0] — 2026-09-06 — 同角色顾问 + 稀疏检索 RAG 三面同落、复活批 35 条、wire 7.1.0、五目标十六资产（check 与 structure 分数与 1.6.0 不可比；index schema 16 整库重建一次）

> 本节按计划步序升序排列；同一步的多个批次按交付时间先后。

**无默认档位变更。** 计划 v2.29 步 1（2026-09-05，143cfe0）——计划书 v2.29 语义层修正案：
- 用户三问 + 深夜三裁后立项：先发 v1.6.0（已发，tag af8dbf8）；语义层 #3「确定性同角色顾问」+ #4「轻量代码 RAG = 稀疏检索（BM25 形，索引已有事实拼词袋，倒排表进 index.db）+ 仓内 PPMI 联想，零外部模型」随 v1.7.0；npm 与桌面装机等 1.7.0 同批；v1.6.0 不单独跑 bench（一日期门下单量一个 tag 就得整条重量），其行随 1.7.0 整条序列入列；45 条后置束按「有收益且代价可接受」逐条复活——35 条具名做，O08 / O80 无收益不做，O03 / O14 / O53 / O67 与 README 永久立场冲突不翻，O23 / O83 已做（清单为本机件）。
- 计划书三处就地改、332 行不变：banner 版本行 + 修正案句、ADR-008 细则第六期（分词 / 倒排 / BM25 / PPMI 全是需源文本的测量居 Rust，排序与「同角色」合取居 Haskell，顾问永不产条件位故无 tier）、§6 T 轨十五步；ADR-008 那一行是册 methodology.md:37 的整行引文锚，追加后按名重签（`CE_DROP_VANISHED=533be90842d8cac2`）。

**无默认档位变更。** 计划 v2.29 步 2（2026-09-05）——同角色顾问的 **ROI 度量先行**，只加度量与冻结件、零面变化：
- `cli/src/similar/`（词袋六通道 / Porter 词干 / 整数 BM25 / 仓内 PPMI，`SIMILAR_REV` 1）只被回放仪器
  `it/similar_replay.rs`（常驻 `--ignored`）读；自仓 + 四份 crosscheck 夹具各自成库，每单元查两臂 top-5。
- 冻结件：样本 `contracts/eval/similar-sample-v1.json`（118 查询 / 700 候选对，sha256 秩序分层抽样）、
  仲裁记录 `cli/tests/it/eval_similar_review/sample.json`（codex gpt-6-astra 逐候选 same_role / related / unrelated + clone）、
  oracle `contracts/eval/similar-oracle-v1.json`；门 `eval_similar_precision.rs` 三腿（一致性 / 夹具逐字节回放 / 60 % 地板）。
- 读数（册 `docs/EVAL-SET-SIMILAR.md`，第三次拆册）：裸臂顶 1 同角色 67/118，role=1 子集 39/59，hit@5 74/118；
  同角色位对候选精度 101/165；PPMI m = 3 扩展臂顶 1 63/118（配对 6 : 2 反向）。k1 / b / 合取维持 spec 形。
- **调优与联想探索（用户裁定后；codex gpt-6-astra 写评测器、本仓第一方复跑）**：`it/similar_tune.rs` + `similar_tune_parts/`（常驻 `--ignored`）
  在冻结 oracle 的候选池上重排 84 个打分配置 + 16 个同角色谓词，无一过显著线——最好的 `field_binary` p@1 69/116 对基线 66/116
  （配对 6 : 3），PPMI 原形扩展臂 62/116（配对 2 : 6）；`SIMILAR_REV` 1 不动。裁定：默认臂 = 裸臂，PPMI 联想改为三面 opt-in 联想视图
  （spec §四数学不动、§六 加开关）；通道分别归一 / 查询 tf 截 1 / `spec ∧ 2N ≥ QN` 三个候选留待步 5 第二份样本留出集复测。册追记节记全表。
- **冻结行身份改为 CRLF 折叠 LF 后的 sha**（`identity_sha`，仪器与评测器同一所有者）：自仓行冻结时哈希的是本机混行尾工作树的字节，
  干净检出只对上 13/70 自仓查询；83 行（29 文件）按 LF 字节重识别，冻结后改过的 2 文件（17 行）按实保留，复跑对上 68/70 查询、407/424 候选对（e300386 钉夹具目录 LF 是同一缺陷的另一半）。
- 子仓教训：`use super::{a, b}` 花括号组在依赖图里落到父模块（前缀 `super::` 即全部消费），子模块与 `mod.rs` 成环——评测器 15 个子模块改逐行 `use super::x;`、
  `strongest` 搬进 `feedback.rs` 消掉互引，子仓 check 980 → 985（地板 983）。
- ADR-006 具名重立：`cli/src/fourclass/units.rs` 196→217（`node_segments` 让词袋与 unitsig 同一宇宙）；本批 `docs/EVAL-SET-SIMILAR.md` 157→272（追记节）。

**无默认档位变更。** 计划 v2.29 步 3（2026-09-05）——词袋与倒排表进 `.ce/index.db`（**索引 schema 15 → 16，整库 wipe 一次**；`similar_rev` 入缓存键），仍零面变化：
- `cli/src/similar/store.rs`：两表只存 fnv1a64 与计数——`bag(term_hash, unit, tf, channel)` 以 `unitsig.id` 为座位（`unitsig` 得 `id INTEGER PRIMARY KEY`，外键级联：词袋宇宙 = unitsig 宇宙；
  WITHOUT ROWID，主键 (term_hash, unit) 即倒排表）与 `df(term_hash, df, marg)`（`marg` = 该词在 PPMI 96 词帽内被计数的单元数，即共现对计数的边际）。**不建 cooc 对表**：自仓 688k 行、
  库 10.7 → 58 MB、冷索引 5 → 35–50 s，而联想视图是 opt-in——共现对改由 `similar/reader.rs` 在查询时从携带该词的单元的 bag 行推导，与内存表逐格相同（五语料回放每单元两臂逐位断言）。
- 随 refresh 差分更新：`retire`（旧袋 −1，在 unitsig 行替换级联掉 bag 行之前）+ `refresh_bags`（新袋 +1，落在新 unitsig id 上），只有净非零的词动 SQL（update-else-insert + 清零行扫除；
  未动的单元自我抵消）；外来文件（`files.owner` = 1）不写行，`remove_missing` 先退休再删 `files`。SQLite 教训：CHECK 约束在 upsert 冲突消解**之前**按插入值算，下行移动不能骑 `ON CONFLICT DO UPDATE`。
- 排序只有一条路：`bm25::Postings` / `ppmi::Cooc` 两个 trait 各配自由函数（`top_k` / `neighbours` / `expand`），内存 `Corpus` / `Table`（仪器与单测）与 `similar::reader::Reader`（SQL）各实现一次；
  邻词精确剪枝 `4·n_a > N ⇒ 无邻词`（PPMI ≤ log2(N/n_a) < 2 bit，门槛之下）。
- 分词路一处：索引写下的袋 = `file_bags` 现算的袋（身份、行距、逐词相等；单测钉 fetch / user / query / row 与形状词 `p:1`，停用词不入），改一处分词即两边同动；
  bag 行落在外来文件的单元上 = 读者打开即具名拒绝（`outside the own universe`）。
- 代价（PERF-BUDGET 新节，自仓 687 文件 release A/B）：冷 `ce dedup` 5.1–5.3 → 8.0–8.7 s（+0.65 s 第六次解析与 docdup 段再抽取、≈2.3 s SQL 其中 ≈1.5 s 是随机键倒排索引在逐文件事务下的写放大）、
  暖 0.51–0.56 → 0.50–0.55 s 不变、库 10.7 → 18.0 MB（bag 177,536 行 / df 5,697 行 / 自有单元 5,458）；五种布局微基准里取 WITHOUT ROWID (term, unit)（时间与 rowid 双索引打平、体积少 2.5 MB）。
- 测试（子仓）：`unit/similar/store.rs`（差分随每次 refresh 与全量重算对账、未动单元抵消）、`unit/similar/reader.rs`（持久路 = 内存路、一条分词路、宇宙外 bag 行拒绝）、
  `unit/dedup/index.rs`（五个 rev 行各自在缓存键里）；回放 `it/similar_replay.rs` 改经 `Reader` 读库、两条路逐位对拍后再量（release 1062 s，五语料零分歧）。
- ADR-006 具名重立（旧→新行）：`cli/src/similar/ppmi.rs` 130→193（trait + 精确剪枝）、`bm25.rs` 226→273（trait + 自由函数）、`bag.rs` 269→281、
  `cli/src/dedup/index.rs` 396→407（两半刷新）、`docs/PERF-BUDGET.md` 342→362（新节）、`CHANGELOG.md` 605→624（本块）；子仓 `unit/dedup/index.rs` 27→56（rev 行循环腿）。
  dedup 预算主 56 / 子 119 不动：本批新出的三块克隆（两个 `Postings` 实现同形、`Reader` 两条 SQL 读法同形、两处 `QueryTerm` 投影助手）全部消掉——读者的均长在打开时定一次、
  两列查询收成一个助手、`QueryTerm` 派生 `PartialEq` 后直接比。

**无默认档位变更。** 计划 v2.29 步 5 前置（2026-09-05）——步 2 留下的三个候选在**第二份样本的留出集**上复测，测过再落 wire：
- 仪器得留出集通道：`CE_SIMILAR_SAMPLE_GEN=<n>` 以同一配额、同一 sha256 秩序抽第 n 代，只跳过更早各代 oracle 仲裁过的 rank（排除集从冻结件重导）；
  `similar-sample-v2.json` 115 查询 / 668 候选对（typescript role=1 层 3 个 top-1 全在 v1，抽到 0 如实报），与 v1 零重叠；codex gpt-6-astra max 五批仲裁
  → `contracts/eval/similar-oracle-v2.json`（`generation` 2、`holdout_of` v1）+ 记录 `eval_similar_review/sample-v2.json`（转录修复 1 处）。
- 留出集读数比 v1 低一档：裸臂顶 1 46/115（v1 67/118）、role=1 子集 30/56 = 53.6 %（v1 66.1 %）、非 clone 29/98、hit@5 69/115 = 60.0 %、
  role 位精度 86/177 = 48.6 %（v1 61.2 %）；PPMI 扩展臂配对 2 : 6 与 v1 同形。此后引用顾问精度两代并列。
- **三个候选一个不采**（评测器 `CE_SIMILAR_ORACLE=2`，668 对全部对上）：`field_binary` +3 / role=1 +1（配对 5 : 2，hit@5 −1）不过预登记的 +8 线；
  `binary_query` +2 且自仓 2 : 4 变差；谓词 `spec ∧ 2N ≥ QN` 在留出集上少 3 个真阳（go）、不再 Pareto。v1 最差三行（`k0` / `lm100` / `translation_quarter`）
  在留出集上成了最好三行（各 52/115、10 : 4）——±6 查询的赢面是噪声。**`SIMILAR_REV` 1 与 spec 合取原样进步 5 的 `CE.Similar`。**
- 门 `eval_similar_precision.rs` 改按 `GENERATIONS` 表逐代执行（v1 地板 60 %、v2 地板 40 % = 三数最小向下取整到一成；每代与更早各代零重叠、
  `holdout_of` 点名前一代、夹具行逐字节回放——四夹具各重量一次）；度量算术拆入 `eval_similar_precision_parts/`（E01 文件预算，268 → 251 + 90）。
  评测器 `similar_tune` 的 metadata 曾把 oracle sha 写死在 v1 路径上，改记运行时读入的那份。
- 册 `docs/EVAL-SET-SIMILAR.md` 加「留出集复测」节（抽样 / 仲裁 / 两表 / 三候选 / 裁定三条）；步 2 裁定 2 那句「cooc 表」改按步 3 实测（不建对表、边际进 `df.marg`）。册 13 自仓普查行随树重取（U 923→927；rust 未提及 301→264、haskell 278→247——两份 v2 JSON 拼写了单元键，拼写即提及）。
- ADR-006 具名重立（旧→新行）：`docs/EVAL-SET-SIMILAR.md` 274→351（新节）、`CHANGELOG.md` 624→639（本块）；子仓 `it/similar_replay.rs` 212→229、`it/similar_replay_parts/mod.rs` 234→257（留出集通道）。

**无默认档位变更。** 计划 v2.29 步 5（2026-09-05）——同角色顾问的**判决进核**：wire **6.7.0** 第十二族 `similar/1`（加性 minor，ADR-008 细则第六期），仍零面变化：
- 请求 = 查询袋 `query=[[termHash,weight]…]`（可缺省；哈希严格升序、weight ≥ 1）+ 候选行 `rows=[[nHit,pHit,cHit,dHit,sHit,lHit,shapeEqual,bm25Num,bm25Den]…]`；
  回执 = `order`（BM25 有理数 `Num/Den` 降序、同分按请求下标升序，`Data.Ratio` 精确比较）+ `roles`（同角色位）+ `counts{rows,queryTerms,role}`；
  query + rows > 65536 具名降级 `similar_too_large`；九种行 / 项形错按行点名 `error/contract`。无旋钮、无 fail 档：顾问只排序与打位（spec §一）。
- **计划就地修正为九整数**（`DEVELOPMENT_PLAN.md` ADR-008 第六期句 + spec §五）：`shapeEqual` 是落码时补的第七位——idf 为 0 的形状词既不计分也不算证据，`pHit` 还原不出形状相等，
  合取第二臂 `(nHit ≥ 2 ∧ shapeEqual)` 若不给这一位就只能退回 Rust。合取与地板（`CE.Similar.Cost`：1 / 1 / 2，按留出集复测原样）从 `bm25.rs` 的声明镜像回迁进核。
- Rust 只做行 ↔ 面映射（`similar/wire.rs`）：`Hit` 加 `score_fp`（16 位定点全值，即排序与过线的那个数；`score` 仍是文档冻结的整数部分），分母恒 `1 << SCORE_FRAC_BITS`，
  查询袋 = 词哈希→权重和的有序表；`consume` 严格（回执须是 0..n 的置换、roles 同长、counts 相符、degraded 即 Err 具名），无此能力的旧核 = `core offers no similar/1 (pre-6.7.0)` 具名降级。
- golden 六对（乱序 + role 混合 / 10²⁰+1 : 10²⁰ 对 1 : 1 的有理数精确性 / 空请求 / 三种契约拒绝）；`SimilarProps` 六腿（真值表、排序 = 有理数排序、精确性、空、十种拒绝、降级面）；
  十三个既有 golden 由新核机器再答、只允许 proto 与 `capabilities` 漂移（`hello-ok` 握手 request 随 server 走 6.7.0，§3）；子仓 `unit/similar/wire.rs` 四腿 + `it/similar_wire.rs` 两腿
  （go 夹具全体单元真核复判与度量侧同序同位；同一条链上能力在座 → 判 → 按名拒 → 仍在步）。
- 记账：VERSIONING §1 6.7.0 条 + 能力表 + §3 三元组「123 行，server 恒答 6.7.0」（引文锚三处经 `CE_DROP_VANISHED` 点名重签）；架构图 IR 双语「proto 6.7.0 · twelve / 十二个家族」
  按 pin 重渲、stack.svg 四份同句；README 双语 / 站点芯片 `ver:proto` `count:families` 随 bless；`CE.Similar.Cost` 的三个地板名带 `roleMin` 前缀——`docs_consts` 按裸名把册 14 的
  `minName` 芯片绑到 `CE.Tombstone.Cost` 且要求唯一。
- dedup 预算 56 恒、零新克隆块——第十二族照抄第十一族的五处同形当场消掉：Rust 问答壳提升为 `corelink/judged.rs`（能力门 `ask` / `degraded` / `table` / `count`），
  `tombstone/wire.rs` 与 `similar/wire.rs` 同改（101→91）；`CE.Similar` 的越界 lambda 抄了 `Graph.hs` 的，改具名 `overCap`；`SimilarProps` 请求构造并成一个 `request`、电池改对表形。
- ADR-006 具名重立（旧→新行）：`contracts/VERSIONING.md` 658→669、`CHANGELOG.md` 639→658（本块）；新文件入基线 `core/app/CE/Similar.hs` 128、`core/app/CE/Similar/Cost.hs` 64、
  `core/test/SimilarProps.hs` 96、`cli/src/similar/wire.rs` 105、`cli/src/corelink/judged.rs` 52、`contracts/fixtures/similar/golden.ndjson`；子仓 `it/similar_wire.rs` 63、`unit/similar/wire.rs` 98。

**无默认档位变更。** 计划 v2.29 步 6（2026-09-05）——同角色顾问**三面同落**（ADR-008 细则第六期；三面等价、仍只当顾问不判决）：
- 一份文档 `ce.similar-report/0.1.0`（`similar/face.rs::report_json`）：`query{label,terms,widen}`、`candidates` 行 `{at,key,nth,role,score,hits[6],shape_equal,widened}`
  （前五个标量按字母序即 hub 投影列）、`counts{candidates,role,widened}`、`degraded`；排序与角色位只来自核 `similar/1`，无核 / 旧核 = 具名降级、按度量序、`role` 为 null（A9f）。
  问法三选一 `similar/query.rs::Ask`：`at file:line` 取最内层座位、`unit key` 唯一否则点名前五处、`text` 只作 Name + Doc 证据（无形状无被调者，核给的角色位构造性为假）。
- CLI `ce similar --at|--text|--unit [--widen] [--format json]`（`main_similar.rs`，clap 组恰一）；MCP 第十五个只读工具 `similar_units`（`{at,text,unit,widen}`，经 `faces::similar` 同一文档）；
  GUI 第十一屏 `similar`（`gui/ui/similar.js` + Tauri `similar_report`，i18n 双语 13 键，`reports.css` 复用 hub 表样式）；控制台双语一句头 + 每候选一行 `at key  N P C D S L  role`；`main_lang.rs` zh 帮助表加 `similar` 与四个参数（zh_surface 门抓出英文 about）。
- Stop 审计 `audit/similar.rs`：本会话新增的单元（after 有、HEAD 无的 (key, nth)）逐个问索引，核判 top-1 role=1 才写 `{unit,twin,score}`；feed **`ce.observe/0.9.0 → 0.10.0` 加性**——
  `stop_audit` 行可选 `similar{rev,new_units,queried,rows[,degraded]}`，无话可说即无键；墓碑腿与 similar 腿共读一次 git 批（`audit/tombstone.rs::loaded` 拆自 `measured`）。
- 门与记账：parity 表新行（README 双语 parity 块）、`count:mcp_tools` 14→15、`count:screens` 10→11 随 bless；`docs/reference/cli.md` 再生；`docs/reference/gui.md` 十一屏 + Similar 行；
  plugin/README 15 工具；官网 how 双语第十五工具句 + feed schema 0.10.0；册 11 feed 版本史补 0.9.0 / 0.10.0、册 14 三处引文重瞄（`CE_DROP_VANISHED` 两枚）；facts `report:similar` 入 LINKED 表。
- 测试（子仓）：`unit/similar/query.rs` 4 腿、`unit/similar/face.rs` 2 腿、`it/similar_face.rs` 4 腿（CLI == faces 文档且核判同角色、widen 打标、MCP 转发与恰一拒绝、Stop 腿写行 / 提交后无键）；
  MCP 会话壳提升为 `common/mcp.rs`（`mcp_precommit.rs` 376→339），catalog 断言 15 名；`hub_projection.js` 加 similar 行投影；feed golden 重 bless（只 schema 串）。
  dedup 主 56 / 子 119 恒：query 测试建库改走 `refreshed_index` 消掉与 store 测试的孪生块。
- ADR-006 具名重立（旧→新行）：主 `cli/src/audit.rs` 260→273、`cli/src/faces.rs` 173→190、`cli/src/mcp/tools.rs` 191→208、`cli/src/mcp/adapters.rs` 166→178、
  `gui/src-tauri/src/commands.rs` 272→293、`gui/ui/i18n.js` 340→356、`gui/ui/index.html` 194→210、`docs/reference/cli.md` 460→483、`CHANGELOG.md` 658→676（本块）；
  新文件入基线 `cli/src/similar/face.rs` 231、`cli/src/similar/query.rs` 144、`cli/src/main_similar.rs` 55、`cli/src/audit/similar.rs` 110、`gui/ui/similar.js` 76；
  子仓 `gui/hub_projection.js` 73→92，新 `it/common/mcp.rs` 56、`it/similar_face.rs` 203、`unit/similar/query.rs` 109、`unit/similar/face.rs` 89。

**无默认档位变更。** 计划 v2.29 步 7（2026-09-05）——方法学册 15、十四→十五全扫、两张图、`ce similar` 代价、步 6 CI 唯一红腿：
- 册 15 `docs/reference/methodology/15-same-role-advisor-sparse-retrieval-and-in-repo-association.md`（296 行，九节：六通道词袋 / 倒排两表与不建对表的裁定 / 整数 BM25 一条排序路 / PPMI 联想 opt-in / wire `similar/1` 与核内合取 /
  三面 + Stop 腿 / 两代 oracle 与两道地板 / 残余风险 / 验收；每条引文瞄到实现行，`docs_citations` 按 `git add` 后播种）；`methodology.md` 目录行 15、
  册 14 尾接「→ 15」（`docs_nav` 派生导航绿）。
- 十四→十五：`count:booklets#word` 芯片随 bless（README 双语、how 双语 h2 与调和句、stack 双语卡）；how 双语 `<title>` / meta 字面手改（facts `LITERALS` 门）；
  how 双语新节 `#advisor` + f15 卡（`<pre>` 四行公式；consts 芯片 SIMILAR_REV / K / W_UNIT / TERM_CAP / TOP_M / MIN_COOC / roleMinName / roleMinCallee /
  roleMinNameShape / similarCap 按裸名绑源常量，`docs_consts` 绿）、跳转条 15 与「同角色顾问」；README:171 的 Philosophy 句含该芯片，
  册 `methodology.md:34` 引它的锚随之重签（`CE_DROP_VANISHED=b37893c848038ca7`）。
- 图：架构图 IR 双语 `Tauri · eleven screens` / `fifteen read-only tools`；判决图 IR 双语**第 1 行合并**——archify dataflow 只许 0..4 五行（`invalid row 5`），
  故 `Tokens & term bags` → `Clones & roles`（tag `clone/1 · fourclass/2 · similar/1` 需 119 px，节点 width 124→132）；四张 SVG 按 pin 重渲、
  `node scripts/diagram.mjs --check` 清；README 双语 / how 双语 alt 加「克隆与同角色顾问」。
- PERF-BUDGET 新节「v2.29 步 7 `ce similar` 查询代价」（HEAD worktree 自带 `.ce/`，n=5 三臂交错）：裸臂 0.71–0.76 s、`--text` 0.70–0.74 s、
  `--widen` 1.78–1.88 s（+1.1 s = 步 3 不存 cooc 对表的直接代价，只由 opt-in 者付）、一文件改动后 0.99–1.09 s（+0.3 s 差分）；
  教训：PowerShell 函数参数名 `$args` 遮蔽自动变量，首轮全部 27 ms 是 `ce` 的 usage 行——计时脚本先看 rc 与产物再信数字。
- 步 6 CI 33980319590 唯一红腿 `site_screenshots::the_pictures_are_no_older_than_the_screens_they_show`：GUI 第十一屏让 `gui/ui` 的提交比三张截图新
  ——`node scripts/shoot_gui.js --out site/assets --ce <release ce>` 重拍三张，`contracts/gui-shots.json` 回执随之（本地全跑绿是因为截图与屏在同一未提交树里）；首次在主根重拍撞上插件 1.6.0 的 daemon（索引 schema 15）改写共享 `.ce/index.db`（`ce dedup: no such column: u.id`），改在 HEAD worktree 自带 `.ce/` 里拍——`ce join --days 14` 对 211 提交 / 728 文件的窗口逐文件 `git blame --line-porcelain` 约 7 min，代价事实记此，随步 11 清点归位。
- ADR-006 具名重立（旧→新行）：`docs/PERF-BUDGET.md` 362→381、`docs/reference/methodology.md` 66→67、`site/how/index.html` 652→695、`site/zh/how/index.html` 562→597、`CHANGELOG.md` 676→694（本块）。

**无默认档位变更。** 计划 v2.29 步 8（2026-09-05）——复活批 A「判决正确性」七条，每条根修不打补丁；**wire 7.0.0（major）**、基线 schema **`ce.baseline/2`**、**`ce check` 分数与 1.6.0 不可比**（自仓 949 → 943、子仓 985 → 983，CI 地板同咬合重定价 946 → 939 / 983 → 979）：
- O22 / O46 评分：`CE.Verdict.Score` 轴 2（clone）/ 轴 3（docdup）的质量 = 被判定对**触及的互异文件数**（`touched`），分母 = 各自的机会宇宙
  （代码文件 `nodes − docFiles` / 文档文件 `docFiles`）——此前两轴数对、分母全体节点，一份文件抄进三处按六对计费、分母却是文件；`ce check` 的 `sim` 表
  首次带 kind = 2 的 docdup 对行（`score/mod.rs::pair_rows`，dedup 与 docdup 同一条路、同一快照）——此前该轴在产品里恒零。`VerdictProps` 夹具按新分母重配
  （穷举搜索满足可区分性 / 旋钮探针 / 权重探针三组断言）。
- O47 堆叠：`fourclass/1` 对级 `dup=[hash]` → `dupSpans=[[hash,start,end]]`（`stacking.rs::dup_spans`，每个 after 侧出现一行；请求形状变 = **major**，
  信封拒绝外来 major，十三族 golden request 行机器重写为 7.0.0、畸形 / 9.0.0 探针不动），核 `suspicions` 只数落在新重复单元跨度内的 novel 行
  （`inDup ≥ stackingNovelFloor`），删除比仍读整次编辑；`start < 1 ∨ end < start` 在 `violation` 按对点名；`StackingProps` 六腿新电池。
- O51 擦除：第 2 类事实 3 由死亡位改为死亡判决码 0..4（`gather.rs::verdict_code`；`> 4` 按行点名拒绝），核 `judgeRow` 对 2 / 4 按 reason 6
  `public_surface` 拒绝——此前 RG10 从未到达孪生路，公开 `copy.py` 只因同路径的 `dead_file` 行按类名排序先赢才被拒；`close_targets` 改按 `licence`
  （t1_twin 2 > dead_file 1）在可擦整文件行里择优；子仓 `it/erase_e2e.rs` 夹具加私有孪生 `spare.py`（`_spare_total`）——可擦 3、apply 3、
  `## t1_twin` 进 diff 面、再计划收敛 0；`EraseProps` 加四条第 2 类真值行。
- O63 daemon：`daemon/judge.rs` 核链「三振永闭」改指数退避（1 s · 2^(n−1)，上限 60 s，永不永久关闭，装完 / 修好 PATH 的核下次尝试即被接回），
  恢复后首份 classify 报告带 `recovered: n`（feed 0.10.0 `fourclass` 对象加性键；daemon 内层载荷不在 DAEMON 门内，DAEMON_PROTO 2.1.0 不动）；子仓 unit 腿钉
  倍增到帽、恢复计数只报一次、新一轮从头起。
- C-nth §7.2：基线成员身份由 `nth`（同键单元的起始行序）改为**容器链锚** `fnv1a(外层单元键链，最外层在前)#同链同键序`（`score/anchor.rs`，读索引
  单元表不重切）——删掉前面的兄弟不再挪动幸存者的身份，`impl A { fn add }` 与 `impl B { fn add }` 按各自容器分开，同一行两个闭包按扫描序对到索引的第 k 个同跨度单元（重立时两根各撞出一次「continuous entity fingerprint collision」，据此根修）；RM14 门第四本账 `REANCHORED` / `REANCHORED_SUITE`（子仓 `it/baseline_ledgers.rs`：主 45 / 子 37 对 old→new，一次性仪器按同一 `member_id` 喉两种编码推导、与两份提交基线的离散表全等），门把每个期望键映射到锚后继并断言账本只描述现在；`ce-baseline.json` schema
  `ce.baseline/2`，`ce.baseline/1` 按名拒绝并给出 `CE_ACCEPT_BASELINE=1 ce baseline .`（一次具名重立即迁移）；索引 `nth` 列不动（join / churn /
  similar 仍以它为键）。
- O21 守卫：子仓新腿 `it/guard_budget_parity.rs`——PreToolUse 硬预算（`guard/budget.rs::budget_breach`）与 `ce scan`（核 `gradeWith`）对同一字节逐格
  对拍：全局表 30 / 31 行、`[[rules.class]]` 表 20 / 21 行与类外 25 行、`file_lines_fail = 0` 的 5000 / 400 行，断言两侧判决相等且表跨线两侧。
- 记账：VERSIONING 7.0.0 条 + §3 三元组「123 行，锚 7.0.0，server 恒答 7.0.0」（`ver:anchor` 事实随 `facts/ver.rs::ANCHOR`）；册 05 轴 2 / 3 行与
  7.0.0 段、册 09 堆叠节、册 12 第 2 类事实与 reason 4 / 6、erase.md 类表与验收段、DAEMON.md fourclass 载荷注、README 双语可比性句、计划书 v2.29
  步 8 细则就地改。上一提交 9a90dd9（gui `cargo fmt`）落在重立之后，`site_roast` 块因此红一次（CI 33983490751）——本批重立后重 bless 修正。
- dedup 预算 **56 → 55**（ce.toml 台账具名）：本批落下的三块全部消掉（`anchor::units_by_path` 单一所有者、`EraseProps` 两张行表、子仓 parity 腿走
  `common::write_all` / 基线单测一个闭包），其中 EraseProps 的折叠顺带化掉 6.1.0 起就在的八探针自重叠块，行集对 HEAD 工作树差分净 −1；子仓 119 恒。
- 步 8 提交 dcbf8ba 的 CI 33995985204 双平台唯一红腿仍是 `site_screenshots::the_pictures_are_no_older_than_the_screens_they_show`
  （`gui/ui/score.js` 的地板字面 939 让屏比图新；本地全绿因该改动尚未提交——步 6 同病第二次）：重拍三张 + **根修**——收据 `contracts/gui-shots.json`
  加 `ui` = `gui/ui` 树内容摘要（sha256 over `路径\0内容\0`，CRLF 折 LF、两端同算），收据腿对**当前工作树**比对而不只读提交；
  `scripts/shoot_receipt.js` 拆自 `shoot_gui.js`（300→269），子仓 `it/site_shots_receipt.rs` 拆自 `site_screenshots.rs`（333→282）。
- ADR-006 具名重立（两仓；`ce.baseline/2` 迁移 = 离散集整体换键）：主 `cli/src/fourclass/stacking.rs` 62→75、`cli/src/erase/mod.rs` 120→130、
  `cli/src/daemon/judge.rs` 175→227、`core/app/CE/Verdict/Score.hs` 261→270、`cli/src/score/mod.rs` 398→432、CHANGELOG 707→725，新文件 `cli/src/score/anchor.rs` 158 /
  `core/test/StackingProps.hs` 48；子 `it/erase_e2e.rs` 235→269、`unit/fourclass/stacking.rs` 70→75、`it/baseline_ledgers.rs` 172→297、`it/baseline_bridge.rs` 180→210，新文件 `it/guard_budget_parity.rs` 94 / `unit/score/anchor.rs` 132 /
  `unit/daemon/judge.rs` 36。

**无默认档位变更。** 计划 v2.29 步 9 批 A（2026-09-05）——复活批 B 的前半：发布信任链九条 + O87 dependabot + C-macOS 每推，外加步 8 修补提交的 CI 红腿与 CHANGELOG 第三次拆册：
- O84 来源绑定：draft 以 `--target "$GITHUB_SHA"` 建在构建提交上（重传走 `gh release edit --target`）；verify-publish（`fetch-depth: 0`）对 `cli/src` / `cli/Cargo.toml` / `cli/Cargo.lock` /
  `core/app` / `core/ce-core.cabal` / `core/cabal.project` / `core/cabal.project.freeze` / `gui/src-tauri` / `gui/ui` 九条路径逐条比 `git rev-parse <构建提交>:<路径>` 与 tag 提交的树哈希，
  任一不等即按名拒发；draft 的 target 不是提交 sha 也拒——二进制自此绑定到它真正出自的源码树，不只绑定到 pin 的哈希。
- O85 占位说明：占位句由 workflow env `DRAFT_NOTES_MARKER`（"Draft build phase"）单一所有者，publish 前读 release 正文——仍含占位句或为空即拒发；说明改在打 tag **之前**写
  （`gh release edit vX.Y.Z --notes-file`），RELEASE.md §2.2 / §2.3 就地改序。
- O86 并发组：`concurrency.group` 由 `release-${{ github.ref }}` 改常量 `release`——dispatch（建 draft）与 tag（校验发布）此前不互斥，同跑会对同一 draft 边传边验。
- O77 pin 生成器：`scripts/pin_release.js <版本> [--bless]` 读 draft 自己的 SHA256SUMS（须是 draft、恰十资产、名字全在九人名册内），改写 `plugin/bin/manifest.env` 的十一行并以
  `git diff --numstat` 断言恰 11（版本变）/ 9（同版本重 pin）行移动；`--bless` 顺带跑 `facts_` 刷第十二行 `ver:pin#v`；非 draft / 名册外资产 / 重复键各按名拒绝。
- O74 npm 指针包入库 `npm/`（package.json + README，逐字复制已发布 1.5.1 的元数据与指针文，`files: []` 零二进制）；子仓 `it/health_plugin.rs::version_mirrors_move_with_the_crate`
  的 json 镜像表加 `npm/package.json`——版本必须等于 crate 版本；RELEASE.md 六处一致加它，§3 npm 一腿 `cd npm && npm publish`。
- O76 部署后校验进部署脚本：`scripts/deploy_site.js` 在 wrangler 之后自己跑 `verify_site.js`（最多 8 次 × 15 s 等边缘节点刷新，8/8 逐字节等于提交的 blob 才退 0）——此前是
  RELEASE.md §3 里部署、校验两步手动链。
- O79 气隙手放：`plugin/bin/ce.sh` 候选循环——data 目录手放副本与 PATH 上的 `ce` 都先过 sha256 对 pin；手放副本不等 pin 时**按名拒绝并保留文件**
  （`REFUSING on-disk ce — SHA256 mismatch, not running <路径>`，不删不覆盖）再试下一候选；`bootstrap_e2e.sh` 15 → 17 态（15 = `CE_AIRGAPPED` 下手放匹配副本直接执行且落戳；
  16 = pin 为零值时手放副本被拒、stdout 仍是回落 `ce` 的输出、文件保留、无戳）。
- O78 真 HTTPS 腿：ci.yml 周程 schedule 新 job `starter-https`——对已提交的 manifest 跑一次真 starter（从 GitHub Release 下载 ce 与 ce-core），断言 stderr 静默、`--version` 等
  `CE_MANIFEST_VERSION`、会话戳落下、`ce-core` 放到位、`ce doctor` 握手 OK；此前只有 `file://` 语料。
- O81 dedup SARIF：ci.yml main 推送加 `ce dedup --format sarif` 上传腿（category `ce-dedup`，与 `ce-scan` 各自成族）——该格式自 v2.14 起存在而无消费者。
- O87 `.github/dependabot.yml`：github-actions（`/`）+ cargo（`/cli`、`/gui/src-tauri`）三条周程；npm 指针包无依赖不设。
- C-macOS：`build-macos` 改每推——此前只在 tag 与周一 schedule 跑，v1.3.0 首打红即此类（`daemon_cwd` 入库后第一次 macOS 运行就是 tag）；公共仓 Actions 分钟免费。
- 记账：RELEASE.md（§1.3 `--target`、§2.1 生成器十一 / 九行、§2.2 说明先于 tag、§2.3 tag 后顺序 = checks → 来源名册 → 说明 → pin → publish、§3 npm 与官网校验、§4 手放两态）、
  plugin/README 手放副本一句、ci.yml / release.yml 头注。
- 步 8 修补提交 08e4e3b 的 CI 33997942800 双平台唯一红腿 `eval_mention::the_self_corpus_holds_the_preregistered_zeros`：`scripts/shoot_receipt.js` 与子仓 `it/site_shots_receipt.rs`
  两个新文件进了提及宇宙（U 950 → 952）而册 13 自仓普查行未重取——普查数字要在**全部文件到位之后**取；本批文件到位后重 bless（U → 957 = 969 − 12 early-NUL；npm/package.json 亦入宇宙）。
- CHANGELOG 729 / 750 第三次抵硬线：v1.4.0–v1.4.1 两条目（314 行）逐字节迁入第二归档册 `docs/CHANGELOG-ARCHIVE-v1.4.md`（第一归档册 685 行装不下），冻结集 `frozen_set.rs` /
  引文 opt-out 同批登记；本册 729 → 447。
- ADR-006 具名重立（主）：`scripts/deploy_site.js` 93→119（+26，容差 10；`ce check --format json` 的 `over` 唯一一项）；新文件 `scripts/pin_release.js` 128 /
  `npm/README.md` 19 / `docs/CHANGELOG-ARCHIVE-v1.4.md` 320 随重立入基线；`plugin/bin/ce.sh` 293→303（容差恰用尽）与 RELEASE.md 104→111 在容差内。`.github/` 是隐藏目录、
  在 walk 之外不入棘轮，其三文件的增长如实记：`bootstrap_e2e.sh` 335→378、`ci.yml` 440→489、`release.yml` 349→407、`dependabot.yml` 新 20。`docs/DEVELOPMENT_PLAN.md` 332→333（§5.10 布局树加 `npm/` 一行——`layout_tree` 门抓出）随重立。子仓 `it/health_plugin.rs` 163→166 在容差内、无重立。

**无默认档位变更。** 计划 v2.29 步 9 批 B 后半（2026-09-05）——产品小项六条 O24 / O50 / O61 / O25 / O45 / O49，判决面零变化：`ce.erase-plan` 0.2.0 → **0.3.0 加性**、第十六个 MCP 工具、`[ui] lang` 第三选择器、FPR 回放仪器复立为常设腿：
- O24 擦除建议行点名家族命令：`erase/model.rs::family_command` 是唯一一张表（`t1t2_block_no_whole_unit` → `ce dedup`），控制台句「见 `ce dedup`」、GUI 摘要芯片 `→ ce dedup` 与计划文档新键 `families`
  （`ce.erase-plan/0.3.0`，加性）同源；表不认识的 kind 保留旧句「见对应家族命令」、不进 map、不编造命令（子仓 `unit/erase/render.rs` 一腿钉三面同表）。
- O50 擦除审计轨迹有了读者（三面）：`erase/log.rs` 读 `.ce/erase-log.ndjson` 出一份文档 **`ce.erase-trail-report/0.1.0`**（行 `{ts_ms, class, path, span, provenance, plan}`；读不出的行按行号进 `unreadable`，
  不作工具错误吞掉整份）——CLI `ce erase --log`（与 `--apply` / `--check` 互斥；有不可读行退 1）、MCP 第十六工具 `erase_log`（只读，永不 apply / append）、GUI 擦除屏「审计日志」段
  （已打开的日志随 apply 重读；i18n 六键）；`faces::erase_log` 一具身体、`LOG_SCHEMA` 从 `apply.rs` 私有常量提到 `model.rs` 公开导出——facts 登记 `report:erase-log#schemaver` scraped → linked、
  新 `report:erase-trail#schemaver`、SCRAPED 22 → 21；parity 行「擦除审计日志」、README 双语能力表与 MCP 计数 fifteen → sixteen（架构图双语同改）、erase.md 第 6 条（此前写「今日无 CLI 或 GUI 面渲染它」）、
  gui.md Erase 行、plugin README、erase skill 各就地改；子仓 `it/erase_e2e.rs` apply 后读轨迹逐行对 applied 行、`unit/erase/log.rs` 两腿、`mcp_precommit` 目录 16 名。
- O61 `[ui] lang`：ce.toml 新节（`config/ui.rs`；`en` / `zh` 之外按名拒载），第三选择器 `--lang` > `CE_LANG` > 文件，在控制台面加载配置处生效——`i18n::init_from_config` 只在 `progress::armed()`
  （`ce` 的 main 武装过控制台面、与 TTY 无关）时写入，GUI / MCP / daemon / 测试进程内 `Config::load` 永不切语言；`main_cmds::or_cwd` 解析根目录后立即钉住项目语言，钩子经各自的 `Config::load` 走同一条路
  （SessionStart 健康行按项目语言答）；`--help` 在项目已知前渲染，只读前两者；canonical 指纹规则 6：`[ui]` 整表丢弃（表现不是旋钮，`knobs_digest` 不动；子仓 `config_contract` 加两行）；
  `docs/reference/ce-toml.md` / `cli.md` 再生（`ui.lang` 一行、cli 页横幅三选择器）、README 双语一句、`main_lang.rs` zh 帮助；子仓 `it/ui_lang.rs` 三腿（八行选择器矩阵 / `--help` 只读旗与变量 / SessionStart 中文健康行）、
  `unit/config/ui.rs`；`common::run_ce_env` 清 `CE_LANG`（开发者 shell 导出过会让每条英文断言红）。
- O25 GUI 断点实测钉数：新 `scripts/measure_header.js`（无头 Edge = 应用自带的 WebView2 引擎；二分求「一行装下十一 tab」的地板；`shoot_gui.js` 导出 `launch / attach / serve / teardown` 供其复用）
  ——英文 1150 px / 中文 844 px（未挤压自然宽 1618 / 1496；批 9 提案的 1120 是字宽估算），钉 `@media (max-width: 1149px)`：tab 条独占一行、tab 内边距 s4 → s3 让十一 tab 在 860 px 最小窗装下
  （实测 841 对 828）；三张截图重拍逐字节相同、收据 `contracts/gui-shots.json` 随 `ui` 摘要更新；子仓 `site_screenshots` 腿 1 的「拍摄时刻」见证改为图与收据两者提交的较新者（逐字节相同的重拍动不了图的提交）。
- O45 `min_distinct` 校准可复现：子仓新 `it/eval_dedup_distinct.rs` 用 `dedup::analyze` 关下限（`min_distinct = 0`）重量五语料在 t = 50 处全部块的 `distinct` 直方图与出厂 7 抑制的块（两端 `文件:行`），
  冻结 `contracts/eval/dedup-distinct-v1.json`（`ce.eval-dedup-distinct/1.0.0`），表渲染进 `DEDUP-CALIBRATION.md` 新节（`<!-- distinct:begin/end -->`；CI 腿实测 fixtures 并对表，四外部语料 `--ignored regenerate` 重量）；
  fixtures 8 / cobra 13 / requests 12 抑制块与 2026-08-07 记录逐条相同（pygments 字典族 = `flask_theme_support.py` ×11），ripgrep 17 / zod 623 首次入册；册 01 §7 末段由「本节未复现」改为「产品复现」；
  `eval_support::corpus::PINNED_CORPORA` 四 tip 单一所有者（`eval_mention` 同读）。
- O49 FPR 回放仪器复立为常设腿：子仓新 `it/fpr_replay.rs` + `fpr_replay_parts/`（`--ignored`，release 约 7 min；`CE_FPR_REPO` / `CE_FPR_TIP` / `CE_FPR_LIMIT`）——每条拦截对**父版基线**读一次
  （与孪生共享的 token 和：父版 0 = 新、更大 = 延展、否则 = 漂移；novelty 减法同守卫 `carried` 的重叠规则），对**提交整体落地后的子状态**再读一次（落地 / 写先于削的中间态），孪生同提交去向随行；
  Markdown 判决但无语法、归 docdup 不入事件（daemon 探针同读法）。全史复跑：requests 窗口（1f6589ec 止，M5-3 钉定克隆内）365 事件 0 拦截；自仓 555 提交 3164 事件 176 拦截事件 / 257 行 =
  落地 178（账本真阳类）+ 中间态 79（全部孪生同提交被动：改名 30 / 拆并叶 49；b4a0b642 一个提交 26 行；12.48/500 全文写口径、按事件 9.80）；复燃 35 = 延展 11（共享片段多 63～389 token）+
  归零复引 24 + 漂移 0——K 轮「23 延展复燃按倾向判真阳」改为度量；FPR-REPLAY.md 新节 + 横幅 + 复现（历史配方保留）、EVAL-SET.md 退役行、册 11 立场段、bench.json `guard_fpr_per500` 冻结点
  source 重瞄 :18-38 + :49-98 + :101-148（BENCH.md / 两 bench 页随 bless）。
- dedup **55 / 119 恒**，本批落下的五个新克隆块全部消掉而不买单：主仓 `mcp/adapters.rs` 第三个同形壳 `erase_log` 让 `erase` / `doctor` 连成块——三者并入既有 `plain!` 家族（`plain_face` 一具身体，
  原 `judged` 改名、六面一张 match）；子仓 SessionStart 健康行读取提升 `common::session_start_line`（health_plugin / ui_lang 同读）、`unit/erase/log.rs` 时间戳四连断言改数组一判、
  文档字段断言串改 `counts` 整对象一判。
- ADR-006 具名重立（两仓）：主仓 `ce check --format json` 的 `over` 十三项——`cli/src/i18n.rs` 85→136（第三选择器与 `init_from_config`）、`config.rs` 289→302、`progress.rs` 206→220、
  `faces.rs` 190→201、`main_erase.rs` 58→82、`erase/model.rs` 98→124、`erase/render.rs` 159→181、`gui/ui/erase.js` 75→142、`gui/ui/i18n.js` 356→368、`gui/ui/style.css` 223→244、
  `scripts/shoot_gui.js` 269→282、`docs/FPR-REPLAY.md` 158→221、CHANGELOG 447→483（本块）；`mcp/adapters.rs` 178→181 在容差内；新文件 `cli/src/config/ui.rs` 33 / `cli/src/erase/log.rs` 184 /
  `scripts/measure_header.js` 114 随重立入基线（`contracts/eval/dedup-distinct-v1.json` 是 json、不在尺寸臂内）。子仓 `over` 四项——`unit/erase/render.rs` 34→70、`it/eval_support/corpus.rs` 97→129、
  `it/common/hooks.rs` 172→193、`it/erase_e2e.rs` 269→290；新文件 `it/eval_dedup_distinct.rs` 238 / `it/fpr_replay.rs` 298 / `it/fpr_replay_parts/mod.rs` 120 / `it/ui_lang.rs` 86 /
  `unit/config/ui.rs` 28 / `unit/erase/log.rs` 108 入基线。分数主 945（地板 939）/ 子 984（地板 979），两仓 `added` 皆 0。

**无默认档位变更。** 计划 v2.29 步 10 批 C 第一组（2026-09-05）——分发接线 O72 / O73 / O82 + C-installer 动态腿 + `update_e2e` 竞态根修，判决面零变化：
- O72 新子命令 **`ce setup`**（`cli/src/setup/`，文档 `ce.setup-report/0.1.0`）：找到 Claude Code（PATH 上的 `claude.exe` / `.cmd` / `.bat` 或 `claude`，兜底 `~/.local/bin`；`.cmd` 经 `%ComSpec% /c`）
  → `plugin marketplace list --json`（旧 CLI 回落散文表，`❯` / `>` 行取裸名）→ 未注册才 `marketplace add skymanbp/CodeEraser@release`（已注册者保留、永不替换——开发 clone 的目录注册是人做的）
  → `plugin install codeeraser@codeeraser` → 尽力 `plugin update` → 只在本次注册时写 `claude-plugin-wired` 标记（默认在本二进制旁，`--marker-dir` 可指）→ 报告该目录是否在 PATH 上（不在则多一行提示）；
  退出码沿用 v1.0.1 安装日志图例 0 已接 / 5 保留 / 10 无 Claude Code / 11 add 失败 / 12 install 失败，新增 **13**；`--unwire` 以标记为凭只拆自己接的（uninstall + marketplace remove + 删标记），无标记即什么都不问退 0；
  `--format json` 出整份文档，控制台双语一句 + PATH 提示（`main_lang.rs` zh 帮助三键）。名字只拼一处（`setup::NAMES`：source / marketplace / plugin / marker）；NSIS `hooks.nsh` 的 POSTINSTALL / PREUNINSTALL
  改为调 `"$INSTDIR\ce.exe" setup` / `setup --unwire`、只印图例、自身不再拼任何 claude 命令（v1.0.1 起的内联 PowerShell 接线程序删除）。
- O73 提权账户 ≠ 登录用户：`setup::env::users` 读运行账户（USERNAME / USER / LOGNAME）与登录账户（`CE_SETUP_LOGON_USER` 测试缝 → `SUDO_USER` → Windows `Win32_ComputerSystem.UserName`），
  `DOMAIN\name` 与 `name` 大小写不敏感同账户；两者皆知且不同才退 13、什么都不接、句子点名两个账户；登录未知（CI runner、服务会话）永不拒绝。README 双语限制段以此句替换「marketplace 跟 main」。
- O82 marketplace 改跟 `release` 分支（`claude plugin marketplace add owner/repo@ref` 亲测支持，`list --json` 回 `"ref": "release"`；分支已建在 af8dbf8 = v1.6.0 pin 提交）：release.yml verify-publish 在 publish 之后
  新增一步 `git push origin HEAD:refs/heads/release`（只快进；非快进即红并给手动命令）；README 双语安装段 / 命令表、plugin README 安装节、官网两首页安装行、docs/RELEASE.md §2.3 + §3、gui.md「Getting it」同批改。
- C-installer 动态腿，**先核实再加**：空 `CLAUDE_CONFIG_DIR`（无账户、无既有 marketplace）下真 `claude` 2.1.259 对 `ce setup` 答 added / installed 1.6.0（13 s）、第二次退 5 保留、`--unwire` 干净（本机第一方）；
  据此 ci.yml 新增 `setup-wiring` job（ubuntu + windows：`npm i -g @anthropic-ai/claude-code` 后跑真 `ce setup` 三幕，jq 断言文档与 `plugin marketplace list --json` 的 name / ref），随周程 schedule 跑、新增 `workflow_dispatch` 可按需触发；
  `installer_wiring.js` 门改读 `setup::NAMES` 四字段：钩子须委托 `ce setup` / `--unwire`、自身不得拼 `marketplace add`、slug 整词等于 `skymanbp/CodeEraser` 且 ref 为 `release`、插件目标在仓根清单里可解析。
- 子仓 `it/setup_e2e.rs`：脚本化假 `claude`（Windows `.cmd` / unix sh，记录每次调用，按行剧本答 listing / add / install 退出码）——五行图例表（接 / 保留 / add 败 / install 败 / 他人账户）+ 无 Claude Code + 接 / 拆 / 再拆三幕表；
  `unit/setup/{claude,env}.rs` 四腿（JSON 与散文 listing、账户等价表、PATH 成员）；parity 表新行「Claude Code 接线」只在 CLI（安装包调用、AppImage / dmg 用户跑一次），README 载体表补 `ce setup`，`docs/reference/cli.md` 再生。
- `update_e2e` 竞态根修（CI 34006228086 ubuntu 红 `run ce: NotFound`）：`check()` 曾共用一个 `tmp("update-cwd")`——`tmp` 先删再建，并行的腿把兄弟的 cwd 删掉、spawn 找不到目录；改为每腿传自己的目录。
- dedup **55 / 119 恒**，本批落下的十个新克隆块全部消掉：主仓 `setup/mod.rs` 五个 `&str` + 六个 `u8` 常量与 `graph/wire.rs` / `tombstone/vocab.rs` 的常量表同形（六块）→ 名字并成一张 `Names` 结构常量、退出码改 `#[repr(u8)] enum Exit`；
  `Exit` 的派生列表与 `mention::conv::Conv` 同形（一块）→ 只派生用到的 `Clone, Copy`；子仓 `setup_e2e.rs` 两处断言元组同形、四常量表与 `eval_dedup_distinct.rs` 同形、「删日志 → 跑 → 断言」三连（三块）→ `states()` 四词一串 + listing 由名字生成 + 三幕表 `ACTS`。
- ADR-006 具名重立（两仓）：主仓 `over` 两项——`docs/reference/cli.md` 484→500（`ce setup` 节）、CHANGELOG 483→506（本块）；新文件 `cli/src/setup/mod.rs` 271 / `setup/claude.rs` 116 / `setup/env.rs` 114 / `cli/src/main_setup.rs` 39
  入基线（`hooks.nsh` 与 `.github/` 不在度量宇宙）。子仓 `over` 一项——`gui/installer_wiring.js` 88→105；新文件 `it/setup_e2e.rs` 309 / `unit/setup/claude.rs` 23 / `unit/setup/env.rs` 32 入基线；`it/update_e2e.rs` 300→303、
  `it/face_parity.rs` 277→278 在容差内。分数主 945（重立前读 946——重立把软线挪了一分，尺寸轴 73 → 74；地板 939）/ 子 984（地板 979），两仓 `added` 皆 0。

**无默认档位变更。** 计划 v2.29 步 10 批 C 第二组（2026-09-06）——分发面 O70 五目标 / O71 Homebrew + winget / O75 crates.io 腿，外加 verify-publish 一处潜在拒发的根修与第一组 CI 红腿的修补，判决面零变化：
- O70 五目标：花名册只拼一处 `update::version::TARGETS`（`x86_64-windows` / `x86_64-linux` / `aarch64-macos` / `x86_64-macos` / `aarch64-linux`），`FULL_ROSTER_SINCE = "1.7.0"`、`built(version)`（更早的版本只前三个）、
  `Bundle {Setup, AppImage, Dmg}` 由键的 os 半推出（`-setup.exe` / `.AppImage` / `.dmg` 与清单尾 `SETUP` / `APPIMAGE` / `DMG`）；读不到常量的宿主——`plugin/bin/manifest.env` 键集（十五枚 pin，六枚新键在 1.7.0 前为空）、
  release.yml 构建矩阵与 `TARGETS="…"` 校验环、ci.yml `release-rehearsal` 矩阵、`scripts/roster.js`、`bootstrap_e2e.sh` 键表——由子仓门 `it/release_roster.rs` 逐个对拍（流式矩阵行 `key: x, triple: …` 的读者第一版把整段尾巴读进值里，当场修）。
  每目标的构建配方搬进可复用工作流 `.github/workflows/build-target.yml`（`workflow_call`）：release.yml `build`（上传）与 ci.yml `release-rehearsal`（不上传、版本 = crate 自己的、周程 + 手动触发）同一份；新 runner `macos-15-intel` / `ubuntu-24.04-arm`
  （第一方核过 actions/runner-images 标签与 GHC 9.14.1 两平台 bindist；apt 镜像按 `dpkg --print-architecture` 选）。一次发布 = 十五个二进制 + SHA256SUMS = 十六资产；`scripts/pin_release.js` 移十五枚 pin（+ 两行版本）并再生 `packaging/`；
  `update` 的安装器资产按 Bundle 命名（x86_64-macos dmg / aarch64-linux AppImage）。README 双语 / 官网两首页 / plugin README / gui.md / RELEASE.md §1.2 §2.1 §2.3 的「三个目标 / 十资产 / 十二行」改为五 / 十六 / 十七（数词走 `count:platforms` 芯片），
  并写明 1.7.0 前的清单在新两目标上只有空 pin、插件启动器回落 PATH 上的 `ce` 或源码安装。
- O71 Homebrew + winget 作为清单的**生成投影**：`scripts/packaging.js` 从 pin 清单渲染 `packaging/homebrew/Formula/codeeraser.rb`（`on_macos` / `on_linux` × `on_arm` / `on_intel` 只写有 pin 的目标，`resource "ce-core"`）
  与 `packaging/winget/manifests/s/skymanbp/CodeEraser/<版本>/` 三份 yaml（schema 1.10.0、`nullsoft` / `/S` / machine 作用域、`ProductCode` = NSIS 卸载键 `CodeEraser`，本机注册表核过）；`--check` 逐字节比对、陈旧版本目录点名；
  `.gitattributes` 钉 `packaging/** eol=lf`。子仓门 `it/packaging.rs` 用自己的行读者把 pin 从提交的文件里读回来对清单（生成器说谎也过不了）。release.yml 在 verify-publish 之后加三条**按 secret 存在与否**走的可选腿：
  `publish-crate`（O75：永远 `cargo publish --dry-run --locked`；有 `CARGO_REGISTRY_TOKEN` 且 crates.io 尚无该版本才真发，否则 `::notice::` 跳过）、`homebrew-tap`（`HOMEBREW_TAP_TOKEN` → `scripts/homebrew_tap.sh` 经 contents API 写 `skymanbp/homebrew-codeeraser`，仓库尚未建）、
  `winget-pr`（`WINGET_TOKEN` → `scripts/winget_pr.sh`：fork 同步 / 分支 / 三文件 / `gh pr create` 到 microsoft/winget-pkgs）——token 只经环境变量，永不落码、不打印。ci.yml `packaging-live`（周程 + 手动；ubuntu + macos）：`brew style` / `brew audit --formula --strict` /
  `brew install --formula` / `ce --version` 对清单 / `brew test`，ubuntu 再以 `check-jsonschema` 对 1.10.0 三份 schema 校验 winget yaml。`tauri.conf.json` `bundle.publisher = "skymanbp"` 让 ARP 发行者与 winget 标识 `skymanbp.CodeEraser` 同名。
  **deb / rpm / AUR 不做**（RELEASE.md §3 记理由：与 AppImage 同一批二进制再钉四枚 pin 却无消费者；AUR 需维护者账户与第三方 PKGBUILD；Linuxbrew 已覆盖 Linux）。推 tap / 提 winget PR 是对外动作——发版时 AskUserQuestion 裁三枚 secret 与 tap 仓库，不设则发版前把 README / 官网的「Homebrew · winget」行摘掉。
- verify-publish 潜在拒发根修：tag 推送上，ci.yml 里只跑 schedule / workflow_dispatch 的 job（步 9 的 `starter-https` 起、第一组的 `setup-wiring`、本批的 `release-rehearsal` / `packaging-live`）在 tag 提交上以 **SKIPPED** check 出现，原来的门只赦 `build` / `draft`，
  v1.7.0 首打必被拒——改 `SKIPPED_OK` 按名赦免，子仓门 `release_roster.rs::the_tag_gate_excuses_every_schedule_only_job_by_name` 从 ci.yml 各 job 的 `if:` 行推导期望集并要求全等。
- 第一组 CI 红腿修补（`setup_e2e` kept 行，ubuntu + macOS 各一）：假 `claude` 的 sh 脚本用外部 `cat`，而 setup 跑时 PATH 已清空只剩假目录，`cat: command not found` 让 listing 为空、被读成 fresh 而 `add`——改 builtin `read` / `printf`（cmd 侧本就是内建 `type`）；CI 34010686941 三平台绿。
- `packaging-live` 首跑双红（dispatch 34015989581）的根修 9a385c8：在分支 ci/packaging-live 上 dispatch 迭代、双平台绿后随 O54 的推送并回 main。三条 Homebrew 事实：① ubuntu runner 的 Homebrew 装在 `/home/linuxbrew` 而不在 PATH（runner-images 自述）→ `brew shellenv`；② Homebrew 4 拒绝 tap 之外的公式文件（`brew style` / `audit` / `install --formula <路径>` 一律 `Homebrew requires formulae to be in a tap`）
  → `brew tap-new --no-git ci/codeeraser` 建一次性本地 tap、公式 `cp` 进其 `Formula/`、四条命令按全名调——布局与真 tap 相同，验的仍是仓内那份字节；③ 裸二进制 url 的公式装出来没有 x 位——Homebrew `UnpackStrategy::Executable` 只认 `#!` 与 `MZ`，ELF / Mach-O 走 `Uncompressed` 原样拷贝（curl 落盘 0644），`Cleaner` 对非可执行文件 chmod 0444（三处读自 Homebrew 源码）
  → 生成器在 `def install` 末尾加 `chmod 0755, [bin/"ce", bin/"ce-core"]`（rubocop `zero_only` 写 `0755`）。分支上双平台绿：`brew style` no offenses / `brew audit --strict` 过 / 装 5 files 71.2 MB / `ce --version` 1.6.0 对清单 / `brew test` 过。教训：主树 `scripts/packaging.js` +2 行就让首页 roast 块的 `tolerance drawn` 0→1（js 在纯尺寸臂、进棘轮），CI-only 修补在分支上迭代、与下一次重立同批并回。
- 子仓：`unit/update/version.rs` 六行表 + 花名册往返腿、`unit/update/manifest.rs` 按 `TARGETS` 逐目标（已建者三枚 64 位 hex pin，其余 `pins()` 具名拒绝）、`it/facts/count.rs` 的 `roster()` 从 `TARGETS` 推导 binaries / platforms / installers 三个事实。
  dedup **55 / 119 恒**——本批两块新克隆（`packaging.rs` / `release_roster.rs` 各一份 `read(rel)`、`packaging.rs` / `docs_diagrams.rs` 同形的 node 驱动调用）消掉：读者统一走 `facts::read`（同类的 `face_parity.rs` 一并改），node 运行器 + 双流断言进 `common/gates.rs`
  （`demo_replay.rs` 同改；先试开新模块 `common/script.rs`，`common/mod.rs` 的索引随即与 `eval_support/mod.rs` 同形、实测多出一块，故并入既有模块）。
- 记账修正：步 9 批 A / 批 B 与步 10 第一组的三块自 b8d3c1e（第三次拆册）起被追加在 v1.5.0 段末、「更早的版本」之前，现移回 `[Unreleased]` 段（字节不变，只挪位置）。
- Opus 只读审阅 22 条，落 18 条：**4 blocker**——tag 门的等待环把本 run 自己在 `needs:` 上排队的三条可选腿也算进 pending，v1.7.0 首打必等满两小时被拒 → 按 check suite 过滤本 run 未完成项（已完成的 skipped `build` / `draft` 仍按名赦免）；build-target.yml 新加的 `[ "$(ls dist | wc -l)" = 3 ]` 在两条 macOS 腿上是 BSD `wc` 带前导空格的串比较、正确构建也红 → `set -- dist/*; [ "$#" -eq 3 ]`；§5.10 布局树缺 `packaging/` 一行（`layout_tree` 门在 `git add` 后才红）；README 双语 / 官网两首页四枚数词芯片（三 / 九 → 五 / 十五）随 `facts_` bless。**major**：winget `ProductCode` 由「猜是 productName」改为本机注册表实测 `HKLM\...\Uninstall\CodeEraser`（1.5.1 装机，2026-09-06）；winget 三份 yaml 改纯 ASCII（winget-pkgs 对非 ASCII 要 BOM）——生成器 `render()` 与子仓门各一道断言；bundle 表四处拼写加门 `every_host_spells_the_same_bundle_per_os`；`bootstrap_e2e.sh` Linux 臂与 `ce.sh` 锁步（未知架构 = `unsupported`）；`packaging-live` 只在周程 / 手动跑 → RELEASE.md §2.1 要求打 tag 前手动跑一次。**minor**：`winget_pr.sh` 可重跑（分支已在则 PATCH、PR 已开则 notice）、`homebrew_tap.sh` 印的安装命令用 tap 名而非仓库名、`packaging.js::winget` 拆两表 ≤ 50 行、`RELEASE_VERSION` 经 `GITHUB_ENV` 覆盖后断言非空、`bundle()` 的拒绝在 `$(...)` 里不可达 → 顶层先校验一遍花名册、`packaging-live` 按清单版本取 winget 目录、gui.md 断句。**不加 formula `version` 行**：Homebrew 从 url 的 `/v1.x.y/` 段与 `ce-1.x.y-` 词干都能识别版本，显式行会被 `brew audit` 判冗余——由 `packaging-live` dispatch 实证；可复用工作流 caller 被 skip 时的 check 名已按 79611e8 的 `check-runs` 实测：就是裸 job 名 `release-rehearsal`（无 `/ build-target` 后缀），该提交的 skipped 名集 = packaging-live / release-rehearsal / setup-wiring / starter-https ⊆ `SKIPPED_OK`；dispatch 34015989581 五目标 rehearsal 5/5 绿（跑起来的 check 名是 `release-rehearsal (<runner>, <key>, <triple>) / build-target`）。
- ADR-006 具名重立（两仓）：主 CHANGELOG 506→531 / cli/src/update/version.rs 66→139 / docs/RELEASE.md 114→149 / scripts/pin_release.js 128→140，scripts 四新文件 + packaging/winget 三份 yaml 入基线（.rb 与 .github/ 不在度量宇宙）；子 it/common/gates.rs 65→87 / unit/update/version.rs 69→97 / unit/update/manifest.rs 75→92，it/packaging.rs / it/release_roster.rs 两新文件入基线。

**无默认档位变更。** 计划 v2.29 步 10 批 C3 O54（2026-09-06）——structure/1 有向目录边表上 wire，第八条判轴「模块度」判它（**wire 7.1.0 加性 minor**）：
- 请求加性可选表 `dirEdges=[[fromDir,toDir,count]]`（只载跨目录有向边，`from ≠ to`、按 `(from,to)` 严格升序）。
  **intra 质量不上 wire**——`fileRefs` 的 `inside` 在目录内边的两端各加一，逐目录之和恰为内部边数两倍，核取半即得；
  一个数字两个主人正是本族 `seamSoft` 那笔旧账。凭据 = 只在该表在场时执行的**跨表律**（`inside` 之和为偶、
  `outside` 之和 == `dirEdges` 与之相接的边量），不符即按目录点名拒绝。
- 判决 `core/app/CE/Structure/Modularity.hs`：目录分划在有向多重图上的 Newman 贡献 `q = e/m − o·i/m²`，
  除以该目录自身质量的上限 `qMax = mu(m−mu)/m²`，判归一化后的 `rho` 是否低于地板——整数不等式
  `1000(e·m − o·i) < modFloor·mu·(m−mu)`，全程整数、不做除法、无浮点。**判 rho 而不判 q** 是因为
  `Σ q_c = Q ≤ 1`：只对 q 设地板会按仓规模成比例地误判大树，正是 2.26.0 密度律退役掉的那个形状。
  `mu < modMassFloor` 或 `mu == m`（`qMax = 0`，没有可分离的补集）者整条不判——既不算净也不算犯。
- 旋钮 19 `modFloor=1`（‰，零模型那条线；1/3 局部性线归 S2，一现象一轴）/ 20 `modMassFloor=4`
  （四条边以下贡献的正负由单条引用决定）；knob 回执 19 → **21 行**，既有 golden 应答各多两行、其余键字节如前。
- 三面：GUI `axisNames[7]` = modularity / 模块度；控制台印判轴**码**故无新字串；`ce.structure-report/0.6.0`
  形状不变（`axes` 多一行、不多一键）故不升 schema。MCP 工具说明 seven → eight axes。
- **自仓结构分迁移、与 1.6.0 不可比**：`ce structure` 恒发该表，判轴数每次都多一，且新轴入等权折叠。
  实测：本批前的树 **820**（五轴 `0:13 1:26 2:321 3:345 4:195`），本批后的树 **831**（六轴，新增 `7:108`）；
  同一棵**本批后**的树按旧轴表只判五轴是 819，故 819 → 831 才是轴 7 自己那一笔（余下差额是本批新增文件让树本身动了）。
  `ce check` / `ce scan` / 其余分数不动；structure 仍报告态不设门（v2.22 结项 O53 立场不变）。
- Rust cap 镜像同批对齐核的 `famOverCap`（此前只计 nodes 行，seam 表与新表都没计价）；
  `rows::ref_rows` 一次 join 出两张表——它们必须描述同一张图，而一次 join 是保证不是断言。
- 记账：`contracts/VERSIONING.md` 7.1.0 条 + §3「127 行，server 恒答 7.1.0」；册 04 改题「eight axes」并新增 S7 行与推导节
  （文件名保留历史 slug——它是已发布的 URL 与 92 条引文的键）；`structure-axes.md` 改题八轴 + S7 行 + S7/S2 分界；
  README 双语 / how 双语 / stack 双语随 bless；架构图 IR proto 串重渲；判决图 IR「结构与分数」节点副标改 `8 + 7 axes`（结构八轴、判决分七轴，此前二者恰同为七）、stack.svg ×4「seven structure axes」→ eight；GUI 三张截图随 `scripts/shoot_gui.js` 在 HEAD worktree 重拍（结构屏多一轴）+ `site_shots_receipt` 重签。
- ADR-006 具名重立（两仓）：主 core/app/CE/Structure/Cost.hs 156→187 / Structure/Axes.hs 225→253 / Structure.hs 276→288 / cli/src/structure/edges.rs 27→52 / structure/wire.rs 182→206 / structure/rows.rs 244→259 / 册 04 312→345 / contracts/VERSIONING.md 677→693 / CHANGELOG 531→559，`core/app/CE/Structure/Modularity.hs`（114）+ `core/test/StructureModularityProps.hs`（146）两新文件入基线；子 unit/structure/edges.rs 14→28，`it/structure_modularity.rs`（167）新文件入基线。

**无默认档位变更。** 计划 v2.29 步 10 批 C3 O48（2026-09-06）——声明级搬迁，折入同一未发布的 **wire 7.1.0**：
- `fourclass/2` 请求加性 `declRem` / `declAdd`，回执 `unitEdges` / `unitEdgesDropped`；Rust 量名字、种类与跨度，Haskell 判同名同种、唯一目的地与跨度内共同内容。声明支付跨站成本，导出 `declFloor = 1`；多源可汇一处，两目的地拒绝，`declCap = 65536` 超限整表放弃。
- 搬迁表补 `lines = 0` 行，已有行级记录优先；**分数和行分类不变**，既有 L2 冻结件与七条行门不动。声明边是报告信息，不进入守卫判决。
- 真核回放 `commit-edges{,-requests,-ripgrep}-v1.json`：边覆盖 **自仓 31/37 → 36/37、requests 1/1、ripgrep 22/22**；五对短体补齐，`~out_dir` 零共同内容仍未覆盖。六对诊断、改编边界与 ripgrep 六条登记外发现见 [续测台账](docs/EVAL-SET-M5-3.md#声明级搬迁o48)，不改 GT。
- 三对 golden 由核重答、旧回复字节不动；§3 由夹具门导出 **130 行，server 恒答 7.1.0**。主册 EVAL-SET 保持 300 行，册 09 英文推导与中文续测就地对齐。
- ADR-006 具名重立（两仓）：主 `cli/src/fourclass/batch.rs` 211→226、`core/app/CE/FourClass/Wire.hs` 94→113、`core/app/CE/FourClass/Cost.hs` 64→89、`contracts/VERSIONING.md` 693→708、`docs/EVAL-SET-M5-3.md` 182→213、册 09 163→179；新文件入基线 `cli/src/fourclass/decls.rs` 96、`cli/src/fourclass/batch/edges.rs` 125、`core/app/CE/FourClass/Decl.hs` 93、`core/test/DeclProps.hs` 89；子 `it/eval_commit_review/mod.rs` 76→90，新文件 `it/eval_l2_edges.rs` 191、`it/eval_l2_edges_parts/mod.rs` 128、`it/fourclass_decls.rs` 99、`unit/fourclass/decls.rs` 72、`unit/fourclass/batch/edges.rs` 83。dedup 55 / 119 恒；门主 945 / 55 / 0、子 984 / 119 / 0、cabal PASS（DeclProps 11 检查）、lib 366、clippy + fmt 清。

**无默认档位变更；采用变体 B，`[guard] zone_tiers` 默认维持 `false`。** 计划 v2.29 步 10 C-zone_tiers（2026-09-06）：
- 纯映射迁入 `guard::zone`（`landing` / `envelope` / `table_for`），钩子与回放共用；`budget.rs` 缩小，基线每次写入只解析一次。零 wire 变化。
- `fpr_zone_replay` 逐父提交物化策略、记录档位与硬线遮蔽；`fpr_zone_gate` 复算冻结行并把默认值钉到两语料 `rate_ppm <= 10000` 的合取。四份回放共用 `common/history.rs`，区间算术共用 `common/stats.rs`。
- **实测依据**：自仓 568 提交，28 / 5196 事件 = **0.5388 %**（CP 95 % 上界 **0.7778 %**）；requests 钉定尾段 400 提交，11 / 448 = **2.4553 %**（上界 **4.3507 %**），后者超过 1 %，不满足两语料合取。ask 被硬线遮蔽分别 5 / 190，配置不可读均 0；表与全部拦截冻结于 `contracts/eval/fpr-zone-v1.json`，正文见 `docs/FPR-REPLAY.md`。消重后复冻测试体耗时 408.44 / 69.16 s，冻结件与首测逐字节相同。
- ADR-006 具名重立（两仓）：主 `docs/FPR-REPLAY.md` 221→283（本节）、`CHANGELOG.md` 566→581（本块与上块）；新文件入基线 `cli/src/guard/zone.rs` 108、`docs/FPR-L2.md` 149；子仓无超线，新文件入基线 `it/common/history.rs` 78、`it/common/stats.rs` 97、`it/fpr_zone_replay.rs` 240、`it/fpr_zone_replay_parts/mod.rs` 134、`it/fpr_zone_gate.rs` 158、`it/fpr_zone_gate_helpers/mod.rs` 10、`it/l2_fpr_replay.rs` 245、`it/l2_fpr_replay_parts/{mod,fixtures,render,tally}.rs` 101 / 106 / 81 / 85、`it/l2_fpr_gate.rs` 207、`it/l2_fpr_gate/checks.rs` 87、`unit/guard/zone.rs` 106。dedup 55 / 119 恒；门主 945 / 55 / 0、子 983 / 119 / 0、cabal PASS、lib 369、clippy + fmt 清（cli / gui）、GUI 四腿 ok；codex gpt-6-astra 落码（`guard_say` / `observe_feed` / 四条 `guard_hook` 与五条 lib 管道腿只在沙箱红，沙箱外全绿），Claude 审阅与最终改动。

**无默认档位变更。** 计划 v2.29 步 10（C-R-L2-4，证据门四条之一）——**跨文件搬迁 / 堆叠判定的改动集级 FPR 仪器与账本首立**，零面变化、零 wire、零 ce.toml：
- `Judge::judge_changeset` 供 `classify` 与仪器共用：判决仍走 `classify_batch`，保留核链失败记账。仪器与 O48 共用 `pair_inputs`，按冻结切片批量取父/子 blob；不完整的改动集按六种原因记跳过。行的文本与 JSON 输出统一由 Serde 序列化，区间表迭代归算术检查宿主；子仓新增五块消重后回到 119，不抬预算。
- 拦截只读 Haskell 的 `suspicions`（M4 堆叠合取）；跨文件搬迁量随行作证。标签按冻结切片与 labels 的 sha 读取，严格 / 宽读法、两组 CP 95 % 区间、召回均入表。
- `l2_fpr_gate` 七腿核对提交守恒、逐提交仲裁计数、全部表格单元、区间与晋级蕴含式；合成改动集另验搬迁放行、堆叠命中。冻结只准同次完整回放 self / requests / ripgrep。
- `session.rs` 注释更新使旧自仓视图的逐字节匹配数 25→24；以 `CE_REFREEZE=cli/src/fourclass/session.rs` 对 graph-slice / t3-universe / docdup-segments 三视图具名续冻该行，25 行覆盖地板不降，八条关联门通过。
- **首测冻结**：self / requests / ripgrep 分别 **47 / 341 / 433** 个完整事件，六类跳过均 0，`INTERCEPT` 共 0 条、仲裁行 0。严格 **0/125**（CP 95 % **0.000–2.908 %**）、宽读法 **0/820**（**0.000–0.449 %**）；唯一 copy 正例 ripgrep `1035f6b1` 未被拦截，**漏 1、召回 0/1**，保留标签与门线、不翻档。消重后三语料同次复冻测试体 **328.41 s**，冻结件与首测逐字节相同；逐行表与漏报 diff 复核见 `docs/FPR-L2.md`，机器件 `contracts/eval/fpr-l2-v1.json` 由仪器写入。

**无默认档位变更。** 计划 v2.29 步 12 依赖批 DEP-B：
- getrandom 升至 0.4、sha2 升至 0.11、rusqlite 升至 0.40.2（保留 `bundled`，启用 `fallible_uint` 的检查式无符号转换）、toml 升至 1.1.5；CLI 与 GUI 两份锁文件同步。
- 两处 archify 缓存准备移到 Rust 缓存恢复之后，`scripts/diagram.mjs` 遇到被 rust-cache 掏空的缓存（只剩目录骨架，git 会向上认领 CodeEraser 的 checkout，fetch 报 `not our ref`）即重新初始化；NOTICE 门的 `cargo metadata` 加 `--locked`。按两工作区依赖重生 NOTICE，清除 `CE_BLESS` 后复验。
- 更新 pin 与截图收据共用逐字节小写十六进制；发布资产经旧、新拼写与系统 SHA-256 对拍，pin 字节不变。旧开发版所建索引可复用，暖路径克隆行一致。
- **判决、分数算法、schema id 均不变**；不改索引版本与 wire。dependabot PR 编号 1 / 2 / 4 / 6 随本批关闭（升版落在本地锁文件，不合并 PR）。ADR-006 具名重立：主 `cli/src/update/apply.rs` 172→187（`hex` 与测试挂载）；子仓新文件入基线 `unit/update/apply.rs` 57；dedup 55 / 119 恒；门主 945 / 55 / 0、子 983 / 119 / 0；codex gpt-6-astra 落码（五条 lib 管道腿、八条 daemon 腿、两条 git 夹具腿只在沙箱红，沙箱外全绿），Claude 审阅与最终改动。

**无默认档位变更。** 计划 v2.29 步 12 依赖批 DEP-TS（dependabot PR 5；codex gpt-6-astra 落码到对拍与测试全落时订阅额度用尽，Claude 审阅收口）：
- tree-sitter 0.26.11 → 0.27.0（CLI 与 GUI 两份锁文件同步，`tree-sitter-language` 0.1.8；NOTICE 重生）。`Node::child_count` 回到 `u32`，三处显式转换随之删除，`scan/ast.rs` 头注写明子计数与命名子计数各自的宽度、两个取子都收 `u32`（0.27.0 源码核对）。
- `TOKENIZER_REV` 2 → 3，缓存失效拆两层：存储 / 算法键不符仍整库重建；**只有解析器修订不符**时走 `dedup/schema/parser.rs::invalidate`——清 13 张解析派生表与 `full_build` / `resolve_key` / `mention_rev` 三枚戳、换 epoch，`trend` 行保留。trend 的工具链戳自此含 `tokenizer{REV}`，旧戳行在复用前重量（`it/trend_parser.rs`；单测两条：迁移保史、失败回滚）。`schema_current` = 存储键 ∧ 解析器键，`index::peek` 仍只读。meta 键名只拼一处（`parser::KEY`）。
- 不变性协议（同树 f9a0775，旧 / 新二进制各自 worktree 与 `.ce`）：16 族 JSON 报告逐字节相同——codex 首跑 churn 一族差异 = 14 天滚动窗口在两次运行之间前移，同窗口背靠背重跑逐字节同（54361 字节）；自仓索引 12 张表逐行相同（files 758 / fingerprints 38774 / symbols 9847 / sites 5082 / edges 3696 / unitsig 9425 / docsegs 2074 / bag 190398 / df 5834 / mentions 336356 / mention_files 1019 / result_cache 1）；四外部语料 mention 报告相同；FPR 全史回放 257 行相同；similar 五语料回放 ok（SQL 读者与内存语料逐位一致，tally 是索引表与 `SIMILAR_REV` 1 的函数）；`ce check` 945 / 棘轮零动用。冷 `ce dedup` 中位 10545 → 10232 ms、暖 668 → 672 ms（三轮各），库大小逐字节同——在噪声内，PERF-BUDGET 只改失效口径那一句。
- eval：`tokens.rs` 因常量改动丢冻结锚（三份自仓切片 25 → 24 行，覆盖率非判决），按 EVAL-SET.md 复活协议 `CE_REFREEZE=cli/src/dedup/tokens.rs` 具名续冻——graph-slice / t3-universe 只换 sha，docdup-segments 该文件多出一段 live 注释段（`TOKENIZER_REV` 的文档注释过了长度地板：live 259 → 260）；25 行地板复位。
- 文档：册 01 的失效句改为「清解析派生表、trend 保留并重量」并引 `schema/parser.rs` 与 trend 头注（`tokens.rs:21` 锚随常量值改签）；site how 双页 `TOKENIZER_REV` 3；README 双语 tree-sitter 芯片与事实投影随 bless；册 13 自仓普查行重取。
- 子仓 `it/docs_diagrams.rs` 删掉 archify 缓存的第二读者 `cache_head`（骨架缓存下 `git -C cli/target/archify rev-parse HEAD` 会答 CodeEraser 的 HEAD），`--check` 的 exit 2 具名拒绝是唯一谓词；`hs_grammar_pin` 随 `child_count` 宽度改。
- **判决、分数算法、schema id 均不变**；wire 不动。ADR-006 具名重立与门数见提交说明。

**无默认档位变更。** 计划 v2.29 步 12 清点落实批 INV-FIX（codex 清点的 23 条采纳项；codex 额度用尽后由 Claude 分四包落码——三个 Opus 子代理各一包、Claude 一包并逐 diff 审阅）：
- 手写事实配执行者（1–3、21）：架构图 IR 的四个子标签（wire / scan / gui / mcp）经注册表模板渲染后对拍（`it/docs_diagrams.rs` 新腿；`{id}` 渲染器提为 `facts::template` 单一所有者）；VERSIONING §3 三元组里的行数与答版改成 chip——新事实 `count:golden_requests#digits`（= 130，linked 档：golden 文件本身就是源，无债可记），`fixture_contract` 腿同时对拍注册表计数与 Spec.hs 名单；plugin/README 的三钩 / 一 skill / 一命令 / 十六工具四枚 chip，该页入 `facts_chips::SURFACES`；VERSIONING.md:304 两个裸 NUL 字节改成可见的 `\0`，文件回到文本（grep 不再答 Binary）。
- 双语渲染器（4–6）：冻结评估点的值列有了语言——`bench_support/frozen.rs` 一张按指标键的中文模板表（一个 `|` 串而非二元组表：后者与 `unit/structure/tree.rs` 同韵成克隆），数字只从台账的英文值里抽出再拼，多重集不等即回落英文并由门抓出；仪表盘表 / stack 卡 / 首页芯片三处渲染器同改，README zh 表头 `percentile` → `百分位`，`measured()` 的脏树后缀按语言；GUI `bench.js` 经 `benchFrozenWords` 键走 `i18n.js` 的 `BENCH_ZH`（9 指标 × 22 模板，同样只拼台账数字，不等即回落）；新门 `docs_lang_generated`（十词拒绝表 flagged / answered / scoped / held / wrong / attributed / raw / per / percentile / dirty）修前 13 处红、修后绿，外加回放已发布 stack 卡块的负向探针；`every_frozen_point_has_a_chinese_sentence` 拒绝半翻译的新冻结点。zh 三页生成块随 `bench_render` bless 重写（如 `范围内 17/17（100%）`、`600 个样本命中 0 个（门 ≤ 1%）`、`每 500 次编辑 0.00 次误报`）。
- 立场文本（7–10、17、19、20）：CHANGELOG 补步 1 块；计划书横幅首句改「v2.29 修正案执行中」、步 11「发版后」→「发版前」（334 行不动）；README 双语「成品的形状」段改 v1.7.0 范围段（45 条裁定作带日期的历史留一句）、「语义判决覆盖六套语法」改「基于 AST 的判决…Markdown 没有 tree-sitter 语法，由文档与图规则判决」、源码安装序列补 `cd ..`；plugin/README 钩子两行按 guard.rs / audit.rs 现状重写（PreToolUse：T1/T2 探针 + 硬预算 + 分级区记账 + 墓碑类按自己的档位；Stop：净行数 + 涉改重复块 + 墓碑腿 + 只记不拦的同角色顾问行）；册 15 的 VERSIONING 链接、册 11「两类规则」→「三类」（含墓碑类，引文按行播种）；`ce audit` 帮助文案（en 源 + zh 表）描述今日的 Stop 审计，`cli.md` 再生。
- 发版工具（11–16、22、23）：RELEASE.md §2.1 改「tag 前只跑离线三检，`packaging-live` 是公开渠道验收——周程 + publish 后手动 dispatch 一次」；verify-publish 来源环加 `contracts/bench/bench.json`（`include_str!` 编进 GUI）/ `rust-toolchain.toml` / `build-target.yml` 共十二条；check 环 `--paginate` 并按名要求 `build (ubuntu-latest)` / `build (windows-latest)` / `build-macos` 三条 success（空 check 表不再读成「全绿」；缺席 = pending，2 h 超时按名列出），`release_roster` 新腿从 ci.yml 无 `if:` 的 job 与其矩阵推出同一集合（ci.yml 的 job 模型拆入 `release_roster_parts/`，schedule-only 扫描同读一份解析）；`pin_release.js` 改终态判定（键集 = 花名册、每枚 pin 64 位十六进制且等于 draft 报的、两个版本行 = 本次 tag），行数只从本进程读写的那一对算，`CE_PIN_SANDBOX` 离线缝 + `it/pin_release.rs` 四腿六练（首钉 / 原样重跑 / `--bless` 重跑 / 少资产 / 多资产 / 花名册外的键）；bless 命令补 `--manifest-path`（runbook 与脚本同一拼写 `BLESS`）；`starter-https` 可 dispatch；tauri-cli 2.11.4 源码核实 `-- --locked` 直达 cargo（`desktop.rs:259`），该步不改；`brings_something_new` 搬 `bench_support/joins.rs`——两源目录按名比、六个构建输入（两 manifest / 两 lock / cabal.project(.freeze) / rust-toolchain）按内容比并剔除发布自身的版本戳（不剔则每个发布都「有新东西」），21 对 tag 回放拦下的仍是 v0.7.1 / v1.0.1 / v1.3.1 / v1.3.2 四个，BENCH.md 页眉句随之；`.gitattributes` 在宽规则之后重申 `contracts/fixtures/**/*.ndjson -text`（实测：宽规则让 `git add` 把 CRLF 中毒的 golden 归一成 LF 存进索引，字节门就看不见中毒）。
- 记账（18）：ci.yml:82 注释路径改 `tests/it/core_size_gate.rs`；`cli/Cargo.toml` 那半没有此字面，无事可做。dedup 主 55 / 子 119 恒（frozen.rs 的二元组表折成 `|` 串消掉唯一的新块）；ADR-006 具名重立与门数见提交说明。

**无默认档位变更。** 计划 v2.29 步 13 全量文档——三个只读 Opus 审计（README 双语 + plugin README / 参考页 + 合约 + runbook / 官网八页 + 十五册 + 图 IR）61 条发现逐条裁「做」并全落，无一条被推回；改动分三包（Claude：README 双语 / plugin README / CHANGELOG / 事实登记册；两个 Opus 子代理：官网 + 册 + 图、参考页 + 合约 + runbook），每包逐 diff 审阅：
- 手打数字改事实：新事实 `gate:size.file_lines_fail#digits`（= 750，linked 档，源 `config/thresholds.rs::Thresholds::default`）与 `count:assets#word`（= 二进制数 + 1 = 16，源 `update/version.rs::TARGETS`）；README 双语的 750、plugin README 的 750 与「五目标十五枚」、RELEASE.md 的五目标 / 十五工件 / 十六资产、gui.md 的十一屏 / 五安装包全走芯片，`facts_chips` 面表随之（README 双语 36 → 37、plugin 4 → 7、RELEASE.md 5 → 13、gui.md 新登 4）；RELEASE.md「v1.7.0 起五个，此前三个」是历史句、具名不芯片化；parity 表两行不再手打「八轴」、接线行点名 Windows 安装包。
- README 双语：拒绝者三处点名（Stop 审计拒回合、`ce precommit` / `ce commitmsg` 拒提交、CI 退出码拒合并）；判决图 alt 文字按当前五行 IR 重写；插件全链 p95 0.50 s 加测量日期与「墓碑腿并入之前」；新增「同角色建议，零模型」一行（名字 / 形状 / 被调用者 / 文档 / 结构 / 字面量六通道词袋、整数 BM25 k1 = 6/5、b = 3/4、角色位只在名字 / 被调用者 / 形状三通道同意时成立、`--widen` 仓内 PPMI 联想；无退出码、无门、无钩子拦停）；结构行补「文档覆盖」；证据段去重；Homebrew · winget 行改条件句（tap 与 winget token 配好才发、winget-pkgs 合并后）；`ce erase` 行补 `--log`；v1.7.0 范围段改写；限制段三句（顾问永非判决：`ce similar` 恒退 0、`ce check` 不读该族、Stop 顾问行只进 observe；`ce structure` 分数因模块化轴新入与 1.6.0 不可比；软线随每次具名重立移动，不再冻数字）；文档清单补 EVAL-SET-SIMILAR / FPR-TOMBSTONE / FPR-L2。
- plugin/README：`## 配置` 标题；pins 行改五目标 × 三枚十五枚芯片 + 「v1.7.0 前的清单只钉前三个目标」；feed 的 `similar` 对象（`rev` / `new_units` / `queried` / `rows{unit,twin,score}` / `degraded`）一行。
- 参考页 / 合约 / runbook：`ce probe` 帮助补「墓碑类按它自己的 `[tombstone] tier` 记账」、`ce mcp` 帮助改「每个判决家族的只读报告面 + 本机与本构建的诊断面，无一能写」（双语，`main_lang.rs` 同行）；ce.toml 参考的 `[guard] mode` 行点名它只管的两类（T1/T2 重复写入、硬预算越线）而墓碑类按自己的键判、`zone_tiers` 行点名 FPR-REPLAY 台账与 `fpr_zone_gate.rs`；生成横幅改 `--manifest-path cli/Cargo.toml`，`docs/reference/cli.md` / `ce-toml.md` 再生；gui.md 状态横幅改「已发布，十一屏」（逐版本史指向 CHANGELOG）、例外命令三条 → 四条（`bench_doc` 读编译进二进制的序列而非打开的树）；size-advisory 软线链改「现行值恒以 `ce-baseline.json` 为准」（标定期五个链节留给 git 历史）；DAEMON.md 核重启预算改 O63 指数退避（1 s·2^(n−1) 帽 60 s、永不永久关闭、恢复首报 `recovered`）；VERSIONING §1 顺序段与倒序段之间补断句与说明行、3.0.0 条的 daemon 行改指 DAEMON.md §1；RELEASE.md 截图门四腿 → 五腿；PERF-BUDGET 无标题的探针表补节标题（口径 / release / n = 30 自块内取，块内无日期即写明）；册 11 引 `ce-toml.md:32` 的锚随 `zone_tiers` 行重签。
- 官网八页 + 册 + 图：首页双语 `ce similar` 卡、结构卡模块化轴、信任行；how 双语 `erase_log` 工具、序数去除、`judgedAxisCount` 5 到 8、`modFloor` / `modMassFloor` 常量芯片、f09 声明级搬迁段、f11 常设仪器句、区档台账句、九轮；stack / bench 页脚与发布卡；册 01 复现节、02 注释、04 文件名注、05 新段「成员身份（7.0.0）」引 `score/anchor.rs` 与 `baseline.rs`、13、14 九轮、15 spec §2；判决图 IR zh 标签「克隆与角色」、架构图 IR revision 重钉 HEAD、`judgment.zh.svg` 重渲（docs/assets 与 site/assets 同字节）。
- CHANGELOG `[Unreleased]` 改按计划步序升序（规则行入节首；逐行字节搬运、行数不变）。**判决、分数算法、wire、schema 均不变**；ADR-006 具名重立与门数见提交说明。

**无默认档位变更。** 计划 v2.29 步 15 发版前置（2026-09-06，用户四裁）：
- 版本 1.6.0 → 1.7.0：九处字面（两 Cargo.toml / 两 Cargo.lock / `ce-core.cabal` / `plugin.json` / `tauri.conf.json` / `npm/package.json` / 握手 golden `hello-ok.ndjson`）+ 本标题；README 双语 / RELEASE.md / `docs-facts.json` 的 `ver:ce` 芯片随 bless。
- bench 一日期规则改为允许错开（用户裁）：BENCH 页眉改「每行自带测量日期；跨日期读数在版本差之上还含机器日漂移——同日期行是序列、跨日期行只是界」，子仓门 `every_row_shares_one_measured_date` → `every_row_names_its_measured_date`（每行 ISO 日期形）；「尚无自己的行」双语句改「打 tag 之后才被测量」；v1.6.0 / v1.7.0 两行按 `CE_BENCH_TAGS` 单量、不再重放整条序列，2026-09-03 的 119 行原样保留。
- 三条渠道腿首次启用（用户裁）：空 tap 仓 `skymanbp/homebrew-codeeraser` 与 `skymanbp/winget-pkgs` fork 已建；`CARGO_REGISTRY_TOKEN` / `HOMEBREW_TAP_TOKEN` / `WINGET_TOKEN` 三枚仓库 secret 由用户放置，tag 腿按 secret 在座与否自动发或按名跳过（RELEASE.md §3）。
- 判决、分数算法、wire、schema 均不变；ADR-006 具名重立与门数见提交说明。

## [v1.6.0] — 2026-09-05 — 墓碑残留判决进核、`ce commitmsg`、docdup `///` 合段（docdup 行与 1.5.x 不可比）

**无默认档位变更。** v1.5.1 发布后的 bench 落表（07b9155）与其补账：
- **bench 全序列在同一机器状态落表**：17 tag × 7 = 119 行；v1.4.1 / v1.5.0 / v1.5.1 各经
  `CE_BENCH_TAGS` 单 tag 重量（另一会话的 AutoShade 测试与前台游戏抢核，v1.5.1 量八次取一）；
  那次坐下跨了 UTC 午夜、28 行日期不同，而表头「每行同一个测量日期」此前没有执行者——`bench_render.rs`
  新增 `every_row_shares_one_measured_date` 门，并趁重启后的安静窗口把整条 17-tag 序列同一次坐下重量（UTC 09-03）。
- **ADR-006 具名重立账**，超容差上升的文件（旧→新行）：07b9155 三个序列多两版本的生成件
  `docs/BENCH.md` 169→183、`site/bench/index.html` 187→200、`site/zh/bench/index.html` 186→199；
  本批子仓 `it/bench_render.rs` 247→266（那条门与表头改句），主仓 `CHANGELOG.md` 637→656（本节两条）。
- **收尾清点（70-agent 九切面只读清点 + 逐条双人反驳核验，仓内确认项全落）**：`memory/` 改名 `.ccm/` 后
  计划书 :4 / :291 散文里的旧路径就地改；归档册两条根相对链接改为相对 docs/，归档册入冻结集
  （`frozen_set.rs` + 引文豁免表）；册 06 角色表 `deadcode.rs:296-300` 重瞄 `304-307, 325-326`；
  **`source_citations.rs` 只认 `.md:` 目标却自称「整个总体」**——拓宽到注释行里的 `.rs:` / `.hs:`，
  走遍 core/app、core/test、gui/src-tauri/src，扫描器拆入 `source_citations_parts/`；三处漂移引文重瞄
  （`ladder/md.rs:75→86`、`conn.rs:35→daemon/server/conn.rs:46-51`、`flags.rs:9` 两处引语按现文重引）
  并全部补锚文本，册 03 因 segments.rs 多一行位移的九条引文重渲染；`docs/assets/gui-structure.png`
  无读者删除（站点副本由 `shoot_gui.js` 生成并有 `site_screenshots` 门）。

**无默认档位变更。** 计划 v2.26 第一段（2026-09-04；用户三裁：分两段先量 FPR / 出处叙事算残留但 changelog
定位的文档豁免 / 命名 `tombstone`）——**墓碑残留只度量、不判决、零面变化**：
- **feed schema `ce.observe/0.7.0` → 0.8.0（具名断点）→ 0.9.0（v2.27 具名断点：判决键 `judged`，段级 `exempt` 条目带起始 `line`——下节步 4）**：PreToolUse 新事件 `tombstone`（仅当本次删了名字
  或命中时写，带 `erased_hashes` / `session_erased`，名字只以 fnv1a64 键出现）；Stop / precommit 行加性对象
  `tombstone`（`label` / `prose` / `erased` / `exempt` / `sites`，站点只写 `file:line kind`）；golden 重 bless，
  `plugin/README.md` feed 段与册 11 同步；precommit 命中时多印一行人读摘要，任何档位不阻断。
- **度量层 `cli/src/tombstone/`**（frames / names / marked / surfaces / role / texts / mod）：名字只出自结构位
  （非注释行且字面量之外的标识符 + 单元名、md 标题与列表首词；内联代码跨度只保活不声明），框架窗口两侧对称
  不成名，标记与名字的合取以句为单位、只读新增行，changelog 定位（路径或版本台账形）整文豁免并入账。
- **FPR 回放仪器** `it/tombstone_replay.rs`（`#[ignore]`，git 历史驱动，`CE_TOMBSTONE_REPO` / `_LIMIT`）+ 新册
  `docs/FPR-TOMBSTONE.md`：六轮各修一类定义缺陷（自仓命中提交 123 → 68 → 64 → 11 → 7 → 7，requests
  1 → 1 → 0 → 0 → 0 → 0）；终轮 9 处逐条仲裁 = 真阳 6 / 中间态 3（全是计划书横幅）/ 误报 0；门 ≤ 1 % 达成
  （requests 0/400，自仓 3/530 = 0.57 %；把真阳也当误报的保守读法 1.32 % 单独超线，如实写明）。
- **Stop 腿代价**（PERF-BUDGET.md v2.26 节）：干净树与 HEAD 二进制打平（0.580 vs 0.592 s）；27 文件改动树
  +0.65 s、72 % 是两个 git spawn；首测多付的 `rev-parse --show-prefix` 改 `HEAD:./path` 消掉，零改动不再配对。
- **十九条夹具全落测试**：`it/tombstone_guard.rs` 8 腿、`it/tombstone_audit.rs` 5 腿、`unit/tombstone/` 44 腿；
  `ce:allow(tombstone)` 刻意不接线；feed 站点串在测试里由部件拼出（字面 `file:line` 会被引文门当成引文）。
- **ADR-006 具名重立账**（旧→新行）：主仓 `hookio.rs` 248→260（schema 0.8.0 头注）、`proc.rs` 55→79
  （`git_feed`：一次 `cat-file --batch`）、`fourclass/session.rs` 156→169（`scoped_pairs`）、
  `docs/PERF-BUDGET.md` 296→317（v2.26 A/B 节）、`CHANGELOG.md` 656→678（本节）；软线 372→370 随重立挪动；
  子仓无超容差文件（两条 discrete 行随克隆消除退场）；册 13 自仓普查行由其腿重取（U 843→864、顾问行仍 0：`STOP_EN` 改私有、`Marked` 由读者拼写）；三份自仓冻结切片六行按名改签，t3 候选册 rs 准入 1233→1244 随之手改同增。

**无默认档位变更。** 计划 v2.27 第二段（2026-09-04 立项；用户三裁：立项 / 计划书横幅一类文档「段级台账见证 +
`[tombstone] ledger` 声明表兑底」两者都做 / 单词名字继续算、ASCII 3 字符地板维持）——按步就地记账：
- **步 2 段级台账见证**（`role::segment` / `Witness::Segment`）：changelog 定位的第三见证——被触段（`>` 引用块
  连续行，或标题到下一标题的正文）自身含 ≥ 3 个互异版本 / ISO 日期 / 短哈希记号即只豁免该段并入账（feed 条目带
  起始 `line`，schema 0.9.0）；K = 3 由第七轮回放定（真阳所在段记号 0、横幅段 33 / 75 / 77，窗口 [1, 33]，
  与整文件见证「至少三个标题」同一地板）；第七轮：自仓命中提交 7 → 4（三处横幅中间态转段级豁免、6 处真阳原样，
  保守读法 1.32 % → 0.75 %），requests 0/400 不变（`docs/FPR-TOMBSTONE.md` 第七轮节）。
- **引文门补一扇门**（子仓 `docs_citations_parts/passes.rs`）：`CE_DROP_VANISHED` 点名的条目在按行认领之前退役，
  原地重写的被引行（同一行号）可按名重签——此前点名只对孤儿条目生效，同号改文只能改标签绕行。
- **步 3 `[tombstone]` 配置节**（`config/tombstone.rs` / `tombstone/policy.rs`）：`tier`（类自己的档位，默认 observe，
  `[guard] mode` 不及；四档之外按名拒载）/ `budget`（缺席 = 不判，只入 feed）/ `ledger`（声明台账文件，任何语言整文
  豁免，feed `why` = `declared`）/ `terms`（仓库自有词汇永不成名，含复合词）；四键皆入 `knobs_digest`（默认档位拼写即
  静默）；度量层只多一个 `Policy` 参数（钩子与审计从同一次配置加载建它，回放与无表的仓库用默认）。
- **步 4 wire 6.6.0 `tombstone/1` + 三腿档位路由**（`core/app/CE/Tombstone.hs` / `CE.Tombstone.Cost` / `cli/src/tombstone/wire.rs`）：
  第十一判决族——Rust 只送每个候选面的三个整数 `[kind, marks, erasedNames]` 与预算旋钮（码 0），合取（散文 marks ≥ 1 ∧
  names ≥ 1、标签 names ≥ 1）、标签 / 散文分账与 `over`（sites > budget）全在核（`TombstoneProps` 六腿：真值表全枚举 /
  无预算恒 false / 边界 / 三类 contract 拒绝 / 降级面），golden 六对随 hello 能力表机器重生（request 行锚仍 6.0.0）；
  PreToolUse 经 daemon **2.1.0** 加性 `tombstone{rows,budget}` 转发到 daemon 持有的核链，Stop / precommit 复用 audit
  那一条核链（一次打开、两个判决）；三腿只读两位——类自己的 `[tombstone] tier` 与核答的 `over`——同真才出声
  （`guard/say.rs` 双语一句；deny 拒写、Stop 阻断、precommit 退 1），observe 档钩子不出声、终端面仍印一行信息；核不可用 / 旧核无此能力 /
  回执越界 = feed `judged.degraded` 具名，绝不阻断也绝不默过。**feed schema 0.9.0 改为具名断点**（无任何发布带过
  0.8.0 形）：`tombstone` 对象的计数与站点搬进 `judged{sites,label,prose,over}`，`rows` 计候选面，`tombstone` 事件的
  `mode` = 类档位；`frames::marks` 计数取代 `has_mark`、`names::spelled_all` / `wide_all` 取代首个命中。自仓 ce.toml 不声明 `[tombstone]`。
- **步 5 `ce commitmsg <file>`**（`audit/commitmsg.rs`，git commit-msg 钩子之面）：与 `ce precommit` 同一具身体
  （`precommit::run(face, message)`）再跑一次，把 git 交给钩子的提交说明当作多一个 Markdown 面——站点记
  `COMMIT_EDITMSG:行 prose`；注释行按仓库自己的 `core.commentChar` / `core.commentString`（二者互为别名、后设者胜，
  git 2.52 亲证）原地置空而不删除，行号即文件行号，`auto` 读作 `#`；deny 档越预算退 1，读不到文件退 2 而非默过；
  feed 事件 `commitmsg`（precommit 的行形、`session_id` 同为 null，golden 第 12 条）；parity 表与 `ce precommit`
  同行具名（GUI / MCP / 插件无此面：钩子在 git 里）、README 载体表按名省略、zh 面门加一形、`docs/reference/cli.md`
  再生；PR 正文存成文件即同一个面（CI 配方，不做腿）。两钩子都装时暂存集判两次；只装 commit-msg 即两者兼得。
- **步 6 方法学册 14 + 十三→十四全扫**（`docs/reference/methodology/14-tombstone-residue-the-erased-name-conjunction.md`）：九节——改动集与两个面、
  R 的定义与地板、标签框架、逐句合取、四见证豁免（含 K = 3 推导）、`tombstone/1` 与核的三行判决、三腿一档位一 feed、
  已知限制七条、验收（FPR 七轮 + 六探针 + 逐模块测试），每个数字引到 `file:line`；索引表第 14 行、册 13 导航行；how 页两语
  第十四张家族卡另立 `#residue` 节置于常数总表之后（该表手绘、只载树家族常数），十二枚常量芯片逐枚绑源常量（不含
  `PAIR_CAP`——树内两处同名、门解析不唯一；不含 `READ_CAP`——值 `4 << 20` 非字面数）；页题 / meta 十三→十四手改
  （facts_registry 字面位），其余 `count:booklets#word` 芯片由 bless 再生；判决数据流图两语第四行并入本族（archify 数据流布局只许
  0..4 五行，实渲亲证）：度量侧「Git windows & diffs · erased names」→ 判决侧「Change verdicts · Theil–Sen · join · conjunction」tag 加 `tombstone/1`（archify 还校验标签宽度：22 字符 136 px 超 124 px 节点即拒，故取伞名），
  几何不动重渲，README ×2 / how ×2 的 alt 同改；常数总表 alt
  的「每个常数」改「常数」——册 13 起图上已不载全部常数，句子早已不真。
- **codex 审阅批**（2026-09-04，用户令「用 codex 走 gpt-6-astra max 审阅和打磨」；20 条发现 19 落码 1 记账）。
  **完整性**：`texts::load` 把缺失 / 二进制 / 超 `READ_CAP` 的一侧也计入 `unread`（此前只计超 `PAIR_CAP`），
  `surfaces::added` 带回四分类 diff 的 `degraded` 位（有界 diff 把每行都当新增），feed 加 `unread_pairs` / `degraded_pairs`，
  三腿凡度量不完整**只记不判**（`Leg::blocks` 与 PreToolUse 的出声各加一位，终端行加注）；`cat-file --batch` 回执改流式读
  （`proc::git_feed` 返回子进程，超 cap 的 blob 经固定缓冲跳过、不再整份驻留）。**会话并集**：`tombstone` 事件等钩子决定后
  再落行并带 `applied`（deny = false，该次擦除从未发生，`session_keys` 跳过；ask = null），本次 after 侧重新声明的会话键记
  `revived_hashes` 并从并集减去（`tombstone::declared_keys`），并集按 feed 序折叠。**定义**：散文句在**整段**里切再按新增行
  认领（只拼新增行曾把 `We no longer` + `Consult X.` 隔行读成一句）；正文段记号不借 `>` 引用行；`role::version` 恰好三段
  （IPv4 不是版本）；`vocabulary` 收 `MARKS_ZH`；`frames::closes` 认独立成词的中文后缀（`cache已移除`）；`[tombstone] terms`
  经 `names::canon` 规范化并整词匹配（复合词此前永不命中）；同面同名只计一次。**提交说明只当面不当侧**（`measure_with`：
  主题行为标签、句子为散文，不声明、不保活、不受见证——`- X is no longer needed` 作列表首词此前把 X 保活成漏报）；
  `ce commitmsg` 改 `texts::read_capped` 有界读（超 cap / 二进制退 2），`comment_prefix` 正则收成
  `^core\.comment(char|string)$`（`core.commentary` 曾被读作前缀）且值逐字节保留（`## ` 的空格曾被 trim 掉）。
  **wire**：`consume` 要求 `over` 为布尔、站点表严格升序、`label + prose == |sites|`，畸形回执一律具名拒绝。**守卫**：
  `budget::shipped_budgets` 在配置漂移下保留声明的 `budget`（出厂值缺席 = 不评条件，抹掉它等于把类关掉），栅栏注记随
  拒绝句带出；Stop / precommit 的对按配置 `exclude` 走同一 walk 作用域。**拆分**：`tombstone/candidates.rs`（候选面 + 第三
  见证）、`guard/probe.rs`（重复探针，guard.rs 303→229）；precommit help 补墓碑类（`docs/reference/cli.md` 再生）。
  **FPR 第八轮**：自仓 536 事件 4 / 6（六处真阳原样）、requests 0 / 400；册加 Clopper–Pearson 95 % 区间列并改口
  「校准证据」（`docs/FPR-TOMBSTONE.md`）；**第九轮**（`///` 合段后）自仓 537 事件 6 / 9 = 六处原真阳 + 两处新真阳（本批
  自己的散文追述被删的 `added_lines`，随即改写）+ 一处误报（单词名 `docs`）——严格 0.19 %、保守 1.12 % 如实入册；requests 0 / 400。ADR-006 具名重立：主仓 `tombstone/surfaces.rs` 146→196、`tombstone/texts.rs`
  175→205、`guard/tombstone.rs` 112→178、`audit/tombstone.rs` 212→265、册 14 374→425、本文件 718→741（本节）；子仓
  `it/tombstone_audit.rs` 210→233、`it/tombstone_commitmsg.rs` 107→141、`unit/tombstone/surfaces.rs` 86→124、
  `unit/tombstone/wire.rs` 71→105（子仓五个新克隆块全部消掉，dedup 119 恒）。记账不改码：每候选面重解析 `role::segment`。
  顺带亲证 docdup 一条缺陷并按用户裁**当批修掉（v2.28 修正案）**：tree-sitter-rust 的 `///` / `//!` 节点跨到下一行列 0，
  `merge_comments` 只见 `//`，连续 `///` 永不合段且 `end_line` 多一行——合段与 `end_line` 改按节点**最后内容行**，`DOCDUP_REV`
  4→5 清缓存；同树前后对拍：ripgrep 段 251→568、重复对 7→50（printer / matcher 各 crate 逐字相同的 `///` 块），cobra / zod /
  requests 零变，自仓段 845→1462、重复对 0→1——`corelink.rs` 与 `Version.hs` 各带一段逐字相同的「台账为何不镜像」说明，收成
  corelink 一处、Version.hs 一句指回（0→0）；check 949 不动。冻结仪器按 EVAL-SET.md 再生成协议重冻结（复活 0c7c936^ 的三个
  生成器，五语料 docdup-segments / oracle / precision 同钉 tip；32 行普查零漂移，只有计数回声与 `docdup_rev` 变；self 八行按名
  改签过的内容按兄弟锚 sha 取回；requests 3 行 / cobra 1 行 md 段早随 5df1dad 的块规则漂移而外部语料无门，本次一并吸收）。
  **docdup 行与 1.5.x 不可比**（Rust 树的段几何整体变了）；check 分数不受影响。

## 更早的版本

v1.5.1 及更早移入归档册：[v1.5.0–v1.5.1](docs/CHANGELOG-ARCHIVE-v1.5.md)（2026-09-24 迁出）、
[v1.4.0–v1.4.1](docs/CHANGELOG-ARCHIVE-v1.4.md)（2026-09-05 迁出）、
[v1.3.2 及更早](docs/CHANGELOG-ARCHIVE.md)（2026-08-31 迁 v1.3.0 及更早、2026-09-05 迁 v1.3.1–v1.3.2）；
四次都是本册抵 750 行硬线所致的拆分，条目逐字节未改。
