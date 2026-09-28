# Changelog 归档册（v1.7.0–v1.7.3）

> v1.7.0–v1.7.3 四条版本条目（v1.7.4 是当前发布，留在原册）。2026-09-28 从 [CHANGELOG.md](../CHANGELOG.md) 迁出：
> 那一册第六次抵到 750 行硬线（计划 v2.30 步 6 提交 D），按仓规「拆分优先于豁免」
> 另起一册。条目逐字节搬家，未改写；记录义务与两类内容的定义仍由原册声明。

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

