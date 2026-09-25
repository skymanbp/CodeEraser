# Changelog 归档册（v1.5.x）

> v1.5.0–v1.5.1 两条版本条目。2026-09-24 从 [CHANGELOG.md](../CHANGELOG.md) 迁出：
> 那一册第四次抵到 750 行硬线（计划 v2.30 步 3「条件不抬嵌套」一批），按仓规「拆分优先于豁免」
> 另起一册。条目逐字节搬家，未改写；记录义务与两类内容的定义仍由原册声明。

## [v1.5.1] — 2026-09-01 — 维护：死件理由码、三处双语缺口、两条规则的执行者、官网八页整理

**无默认档位变更。** v1.5.0 发布后的收尾维护批（计划 v2.25 修正案，2026-09-01）：
一轮 43-agent 清扫「能做但没做」+ 一轮 13-agent i18n 同步审计的确认项，全部处置。

- **死件 why 句改为编码（O23 复活，用户裁定 2026-09-01）。** `DeadRow.why` 曾在
  Rust 里铸成英文句子，直出到中文控制台（`死件：…（no kept in-edge and no entry
  flag）`）与 GUI 引用图屏的两种语言。现在测量侧只出代码（`WHY_CODES` 两行：0 =
  无保留入边，1 = 仅被死代码引用），机器面（`ce.deadcode-report` **0.4.0**、
  `ce.graph-canvas` **0.4.0**）在英文 `why` 旁加性带 `whyCode`，控制台与 GUI 各按
  自己的语言表渲染；旧文档在 GUI 里回落到它自带的英文句而不是 `undefined`。
- **GUI 体检屏的 OK/FAILED 是硬编码英文**（i18n 审计确认）：改走 `handshakeOk` /
  `handshakeFailed` 两键，中文与 CLI 的「握手：正常/失败」同词。
- **erase 建议行带上未解析站点数**（FIELD-TEST 记的显示项，K 轮步 6 曾列入又无声
  丢掉，用户裁定做掉）：`language_unresolved` 的行尾附「该语言尚有 N 个未解析引用
  点位」；`Row.sites` 加性进 `ce.erase-plan` **0.2.0**，wire 的 reason 位不动。
- **措辞对齐结项裁定**：六处源注释与一册方法学把已裁定「不做」的事写成「等 X 落地」
  （structure 分数地板 O53 ×3、R-L2-4 多文件 FPR 仪器 ×2、ce.toml `[ui]` 语言路）；
  计划书 §4.2 `UserPromptSubmit` 行、§6 M5-2 的 R6 条件项、§8 R5 的 dupehound 去向
  就地补上裁定。均为措辞，无功能变化。
- **ADR-006 具名重立账补进本册**：c3a2198 与 ba067cf 两次重立只在提交信息里具名，
  按 ADR-006 规则须在该提交段逐个具名——已补入 [v1.5.0] 两节末尾。
- 方法学册 08 一条引文标签重瞄（`size-advisory.md:46-48` → `:48-49` + `:52`，§C 改写
  后被引行位移）；站点 how 页英文「never contributes to the score」补回中文页与册 08
  都有的限定词 **structure**。
- **bench.json 冻结点的引文有了执行者**（新门 `bench_frozen_sources.rs`）：九个冻结
  评测点各以散文写着 `docs/X.md:A-B + contracts/eval/*.json`，而 `docs_citations` 只
  读 `[label](path#L)` 链接、`source_citations` 只读 Rust 注释，这九条从未被任何门
  解析——被引行位移后会一路绿着印在 BENCH.md、两个 README 与两个站点页上。现在每段
  须解析（文件在、行段落在文件内、通配至少命中一个文件），且值里的**每个数**（`17/17`、
  `1.000`、`0.90`、`1%` 这类整 token）必须逐字出现在被引行内；三条负向探针各按名拒。
- **docs_consts 六枚芯片改绑源常量，豁免名单 22→16**：`kgram` / `window` 绑
  `impl Default for Params` 的字段值、`scale` 绑 `structScale`、`row + knob cap` 绑
  `trendRowCap`、`feed schema` 绑 `OBSERVE_SCHEMA`、四个家族的 `schema` 芯片按文件各
  绑自家 `SCHEMA_ID`——此前全在豁免名单里以「散文事实」为名，而绑定就在一步之外。
- **ADR-006 具名重立（本批）**：主仓 CHANGELOG.md 587→636（本节）、`cli/src/erase/render.rs`
  138→159（`reason_detail` + 测试挂载）、`gui/ui/i18n.js` 329→340（`deadWhy` 两语表 + 体检两键）；
  子仓 `it/docs_consts.rs` 270→287（六芯片绑定 + 四家族 `schema` 路由）、`unit/graph/deadcode.rs`
  79→103（O23 两语腿）。`WHY_CODES` 与两个读法拆进新文件 `graph/deadcode/why.rs`，
  `deadcode.rs` 555→560 落在容差内、不入此账。
- **官网八页整理（用户令「GUI 样式美化 + 重复内容精简/合并 + 排查错误 + 细节优化」，裁定只做官网八页；1.5.0 发完后作为独立提交再部署）**：74 条经对抗核验的发现全落。**样式**：八页页内 `<style>` 全部并入 `site/style.css`（bench 双页 12 行与 how 双页 50 行各是一份逐字节副本，html/css 属 scan-only 门看不见），中文微调走 `:lang(zh)`；三张表统一表头声线；`.cap` / `.note` 全局化——首页记分牌与 bench 芯片下的说明此前根本没被任何规则碰到；bench 芯片得 `install metric` 类（长标签不再全大写、值列对齐），单位随值不留尾空格；`.card b` → `h3`；七个子页面的 logo 锁定成回首页链接；页脚八页统一（兄弟页按导航名 + 源码 + 最新发布，how 页保留完整方法学）；bench 页导航顺序与其余页对齐；`theme-color` + `color-scheme: dark` + OG/Twitter 元数据 ×8（不含任何登记册事实，故无需新钉）；每张图带 `width`/`height`（子仓新腿 `every_page_reserves_the_window_before_the_picture_loads`），窄屏下架构图横向滚动而不再缩成一团；`?v=3` → `?v=4`。**内容**：methodology.svg 的「five named conditions」实为六个，补 `rows_dropped` 并由 LITERALS 钉 `count:fail_conditions`；how 页册 04 标题芯片 `count:axes` → `count:structure_axes`（两者今日同值，但背书的不是同一件事）；13 个 `<h3 id="fNN">` + 三个 h2 id + 章节跳转条，`scroll-behavior: smooth` 自此有处可去；册 12 之前补 h2「据判决行动——擦除与顾问」（此前 12、13 两册挂在 FPR 纪律标题下）；方法学图下移到「两条诚实边界」之前、改题为常数总表、alt 去掉常数；how 页裸版本号一律前缀 `proto` / `CodeEraser`；zh how 页 13 对直引号改「」、两处半角括号改全角；EN 册 04 `floor (scale` 对齐为 `floor(scale`；首页信任锚删去与架构图说明重复的一句、`ce deadcode` 卡精简、Update 芯片改为命令 + 说明段、记分牌下补一句定义种子（`demo/seed/`）；stack 页 `[[rules.class]]` 卡收成门级陈述并链到 how 页评分节；stack 页三枚 `data-const` 改为普通芯片（facts_chips 2→5），`docs_consts_stack.rs` 因此退役——LITERALS 已钉两张 stack 图的三个值，页面上的三枚归 facts_chips；stack.svg 信封框第四行 55 字符溢进邻框（截图实拍），缩为 36 字符并同改 zh 映射。**新增中文图两张**：`methodology.zh.svg` 与 `stack.zh.svg`（几何逐字节同英文、逐文本节点翻译、字体栈补 CJK；docs_lang SVGS 登记、LITERALS 钉 zh 值），zh how / stack 页改挂中文图。**archify 图**：四份 IR 补 `meta.subtitle`（`<desc>` 此前是 archify 的英文默认句，两语皆然）；渲染器改为整文件映射 chrome——`Focus` 与 `Architecture component` 藏在 aria-label 里，旧的 `>term</text>` 判据看不见，docs_diagrams 第五腿同改为整文件断言；根元素钉 `data-theme="dark"`（自动主题在浅色系统上把嵌在深色页面里的图翻成浅色）。**生成器**：bench 芯片 dedup 标签点名 `dedup_warm`（此前「增量索引」在仪表盘上无从对应）、说明句点名两个写手、dashboard 冻结表说明移出面板、stack FPR 卡按语言选标点（zh `：；。`）；`no_generated_sentence_carries_a_lost_continuation` 连芯片一起查尾空格。ADR-006 具名重立：site/index.html 160→172、site/stack/index.html 82→94、site/style.css 171→261、site/zh/index.html 157→169、site/zh/stack/index.html 80→92、子仓 it/facts_registry.rs 129→176、子仓 it/site_screenshots.rs 312→333。

## [v1.5.0] — 2026-09-01 — 复杂度轴的 opt-in 绝对上限；文档规则全部有了执行者

### 复杂度轴补上绝对上限（计划 v2.24 修正案，2026-09-01）

**无默认档位变更**——新键出厂即 0，等于既定的「无硬线」，任何没声明它的仓库
判决一个字节不变。用户三问拍板：尺寸硬线 H=750 维持声明式、不改动态
（2026-08-20 裁定不重开）；复杂度**给墙不给曲线**；顺带查出的维护缺陷全修。

- **`[thresholds] cognitive_fail`：复杂度轴此前没有任何绝对上限。**
  `CE.Scan.Cost.gradeTable` 的 code 2..6 fail 列字面量为 0；写入时守卫对复杂度
  零命中（§4.2 明令 PreToolUse 不做 AST，这一面按设计不接）；ADR-006 棘轮按设计
  只止涨，对全新实体 bootstrap 取自身值（`Ratchet.hs` 自陈 not a violation）——
  所以一个全新的高复杂度函数从来不被任何东西拦。新键把 fail 档补上。**默认 0
  是立场不是遗漏**：计划 §4.1 自己记着 CoC 在正确率轴无证据支持（r=−0.13、CI
  跨零），不为一个自陈无支持的指标设默认拦截。曲线实测过并否决：把尺寸轴的
  `zonePenalty` 套到 CoC 上电荷为 0‰——3900 个测量单元里 2732 个 CoC 为 0，
  分母稀释才是主导，换分子形状买不到东西，却要付一次分数断代。
- **落点只有 scan 与类通道，评分与指纹都不动。** `ce scan` fail 档（具名条件仍是
  `hard_line`，阶梯不合法在载入时退 2）+ `[[rules.class]] knobs.cognitive_fail`
  （同一棵树可以按路径挂两堵不同的墙，实测判别腿：全局 30 + 类 20，只有类内
  文件 FAIL）。score 复杂度轴仍只读 `cocCeil`，**分数与 1.4.x 完全可比**；wire
  未升版——grade 行本就是 `[code, warn, fail]` 三列、`gradeWith` 本就通用处理
  `failLine > 0`，**核心一个字符未改**，golden 逐字节不变；指纹由 canonical
  规则 1/2 自动保证不动（默认值叶子与 null 叶子都不进 digest）。
- **`grade_rows` 的阶梯校验收归 `Thresholds::ladder_fault`**——同一条规则原本在
  两处各列一份 (warn, fail, keys) 表，现在一条规则一个所有者。实测它没有还回
  克隆块（56 有它没它都一样），保留是因为它本来就该这样写；预算 55→56 的真实
  原因是 Thresholds 第九个字段的声明串（见 ce.toml 台账，量法：只删那个字段
  即回 55）。
- **`docs_consts` 豁免名单 30 条减到 22 条，八条负向探针逐一验红。** 门按芯片
  显示名找同名源常量，而 `sizeHard H` / `seamHard H` / `file_lines_fail H` 这类
  标签带着文档的角色字母，永远匹配不上 `sizeHard`——于是硬线印在四个面上却
  没有任何执行者，正是 v1.4.1 批「生成器说谎门看不见」的同类。修法三件：
  `label_binding`（标签→它真正命名的常量）、`numbers`（数对芯片
  `[softMin, softMax]` 两半都查，此前 `first_number` 只能看见前一半）、
  `default_impls_in`（采集 `impl Default` 字段默认值——ce.toml 每个
  `[thresholds]` 键的权威住在那里，`const` 文法根本看不见，新键的 0 从第一天
  就有执行者）。留下的 22 条逐条注明不可绑定的原因。
- **`softLineK` 的 ±6% 是一次观测被写成了不变量。** `Cost.hs` 与方法学册 05 都
  说 k=2 让 S 落在「历史 300 的 ±6%」内；实测今日 S=372，偏 +24%。S 是相对线、
  随分布走，与 300 的任何固定距离都不是 k 的性质——两处改写为它本来的身份，
  引文九条纯位移重瞄再签。
- **软线搬家补进两个 README 的不可比清单。** 原清单只列改判决法的四条原因，而
  `softLine` 从 304（v0.7.3）走到 372（v1.4.1）——把两条线同时套在 v1.4.1 的树
  上差三分，一次具名重立能在零代码改动下挪分数，此前无一处文档说过。同批把
  「复杂度轴出厂不带硬线」写进两个 README 的已知限制，读者第一次能从产品面
  看出这是设计而非遗漏。
- 审计中一条被对抗核验**推翻**的发现留档：`ce join` 的尺寸计价「不设围栏不认类」
  不成立——该路 `continuous` 表整个为空、其轴按注释明言被忽略，join 只消费
  candidates/severity，从不读分。
- **ADR-006 具名重立账（本批 ba067cf / 子仓 bc04231）**，超容差上升的文件（旧→新行）：
  主仓 `CHANGELOG.md` 536→587、`cli/src/config/thresholds.rs` 66→81；子仓
  `it/docs_consts.rs` 224→270、`it/docs_consts_parts/mod.rs` 125→187、
  `it/scan_classes.rs` 46→125。

### 只写在文档里的规则，现在都有了执行者

**无默认档位变更。** v1.4.1 发布后的对抗审计批：31 个 agent、六个维度扫已发布的树，
12 条经对抗核验存活，全修。共性仍是同一条——**每道字节门比的都是文件与它自己的
生成器**，所以一个说了假话的生成器、一条只写在散文里没人执行的规则、一处瞄错行的
引文，都能一路绿着发出去。这一批的修法统一是：给规则配一个真读源头的执行者。

- **bench 全序列同状态重跑，v1.4.1 入列。** 15 个版本 × 7 项 = 105 行，全部
  measured 2026-09-01、dirty=false、同一台机器；被规则挡下的四个 tag（v0.7.1 /
  v1.0.1 / v1.3.1 / v1.3.2）由回放自己具名打印。
- **「这个发布没有自己的行」有两种原因，页面只会说其中一种。** 生成器无法区分
  “规则把它挡下了”与“它该有行而回放还没跑”，于是四个面对 v1.4.1 给出的是**对它为假**
  的那一个理由。修法：把入列规则搬到 `bench_support::brings_something_new`
  单一所有者、对失败的 git **拒绝而非当成“没变化”**，生成器与回放夹具读同一个谓词，
  `bench_append` 也按它拒绝重测同一份程序；`NoRow` 两态各配中英一句。
- **印着版本的面有七个，门只盯住五个。** 漏掉的正是最详细的那两个——网站
  `/bench/` 与 `/zh/bench/` 仪表盘，它们把 1.4.0 当最新行印着，而站点发的是 1.4.1，
  且一个字都没解释。两页标题现在点名被测版本，差异时下方补一句说明；新增一条
  `every_version_bearing_surface_names_the_release`，读**提交的文件**而不是读生成器。
- **NOTICE 与 15 个测试头写着已不存在的 `--test <target>`。** 76 个测试二进制
  2026-08-26 并入单个 `it` 之后，`cargo metadata` 只剩三个目标，而 NOTICE 是随
  crates.io 包发出去的。全部改为 `cargo test --test it -- <模块>::`；
  docs/PERF-BUDGET.md 与 docs/FPR-REPLAY.md 里那两处是“从 git 历史复活仪器”配方的
  一部分，经核实正确，不动。
- **Rust 注释里的 `file:line` 引文一直没有执行者。** 引文门只扫 Markdown 面，
  于是四处 `PERF-BUDGET.md:60-62` 在规则搬到 :82-84 之后又发了五个版本，被引的那几行
  当时已经是图缓存预算的表头。新增 `source_citations.rs`：全仓只六条这样的引文，
  现在每条必须**用反引号写出被引行必须包含的锚文本**，无锚即拒（负向探针实测：把
  引文调回 60-62，门当场点名站点、目标、行与锚）。同批把 release-only 规则收成
  `bench_support::release_only` 一个所有者，两个驱动各自的 `panic!` 副本随之消失。
- **方法学册 13 的 self 普查行陈了 63 个提交、5 个发布。** 那一行写着
  “@ this commit”，散文还承诺它随树移动，而没有任何东西执行这句话；行下方的 restate
  芯片读的正是这张陈表，所以字节上处处自洽。它的测量 CI 本来每次都在做（self 腿不是
  `--ignored`），现在那条腿直接把行**写出来**：U 764 → 837、rust 2021 → 2065、
  haskell 1333 → 1372，`CE_BLESS=1` 重写，只动数字故不动点成立。
- **拒绝消息里的杂空格，第二例与后续三例。** `dedup/budget.rs` 那条与 v1.4.1 修的
  同病同源（搬文件丢了行尾续行符）。新增 `refusal_text.rs` 系统扫 `cli/src` 每个
  拒绝宏的字面量；本批落码时**我自己又犯了三次**，其中两处是直接印在 BENCH.md、
  两个 README 和四个站点页上的句子——故把那四句拆成 `no_row_sentence` 并配门逐句问。
- **RELEASE.md §2 说 pin 提交是十一行，其实是十二行、跨两个文件。** 第十二行是
  `contracts/docs-facts.json` 的 `ver:pin#v`，从 `CE_MANIFEST_VERSION` 派生；只推十一行
  已在 v1.3.0 与 v1.4.1 两次把 pin 提交打红，而 tag 腿等的正是这个提交的全部 check。
- **两个 README 的可比性句子补上 v0.7.3 → v1.0.0 密度计费改判**（此前只列了三处断代
  中的两处），BENCH.md 页眉散文改为只声称两个写手都真在执行的部分。
- **ADR-006 具名重立账（本批 c3a2198 / 子仓 2ea881b）**，超容差上升的文件（旧→新行）：
  主仓 `CHANGELOG.md` 491→536；子仓 `it/bench_render.rs` 186→238、
  `it/bench_render_dashboard.rs` 222→233、`it/bench_support/mod.rs` 237→289、
  `it/bench_support/render.rs` 124→200。
