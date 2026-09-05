# Changelog

> 记录义务来自 DEVELOPMENT_PLAN.md §4.2：“每次默认档位变更在
> CHANGELOG 记录依据（FPR 数据）。”本册记两类：守卫**默认档位变更**
> （依据 = FPR 数据，每个版本条目首句声明有无）与每步落码的**功能 /
> 协议 / 记账变更**（v1.2.0 后的 [Unreleased] 块起，发版时并入版本条目，
> 源码克隆与 crates.io 包内即有全史）；GitHub Releases 留发布说明与分数
> 可比性声明（v1.2.0 及更早的功能面只在那里）。

## [Unreleased]

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
  names ≥ 1、标签 names ≥ 1）、标签 / 散文分账与 `over`（sites > budget）全在核（`TombstoneProps` 七腿：真值表全枚举 /
  无预算恒 false / 边界 / 三类 contract 拒绝 / 降级面），golden 六对随 hello 能力表机器重生（request 行锚仍 6.0.0）；
  PreToolUse 经 daemon **2.1.0** 加性 `tombstone{rows,budget}` 转发到 daemon 持有的核链，Stop / precommit 复用 audit
  那一条核链（一次打开、两个判决）；三腿只读两位——类自己的 `[tombstone] tier` 与核答的 `over`——同真才出声
  （`guard/say.rs` 双语一句；deny 拒写、Stop 阻断、precommit 退 1），observe 只记不说；核不可用 / 旧核无此能力 /
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

## [v1.4.1] — 2026-09-01 — 五条字节门看不见的缺陷

**无默认档位变更。** v1.4.0 发布后的维护批：一轮 21-agent
对抗审计确认的四条，加上回放自己抽出的第五条。

- **分块预算未给类列计价（真缺陷）。** `CE.Scan.overCap`
  把 `rowClasses` 当第六个维度计入上限，而 `scan/chunk.rs` 只算了五个；
  类列每行一项、随块满长发出，却一分钱不付。后果：声明了路径类的大仓
  上 `ce scan` / `ce check` 直接报 “cap mirror drift” 而死，而那是只有我们
  能修的错。模块头注还写着“类列由调用方预留 override 行代付”——
  `overrides` 每个声明类最多几行，`rowClasses` 与块同长，这笔账从来对不上。
  修法：行价在 `plan` 里按是否有类列算 1 或 2，头注改写为实情，并配一条
  `a_class_column_costs_a_row_its_own_seat`（同一批行无类时一块、有类时两块，
  单文件超预算则具名拒绝）。
- **拒绝消息里的 14 个空格。** 块从 `wire.rs` 搬出来时字符串丢了行尾
  续行符，一条故意写成具名拒绝的句子因此读起来像构建坏了。现在它是
  一行，并由既有那条拒绝腿多断言一句“没有连续空格”。
- **两个 README 把 Rust edition 印成了 `2,024`。** `#digits`
  是为 4,096 那种**数量**设计的形，从 1,000 起加千位逗号；而 edition
  是个**名字**（2015 / 2018 / 2021 / 2024），加了逗号就不再是任何合法值，
  读者拿它去 grep 或拄进 Cargo.toml 都落空。已随 v1.3.0 / v1.3.2 / v1.4.0
  发了三次，而每道字节门都绿——门比的是文件与它自己的生成器。新增
  `#name` 形（命名而非计数的整数，不分组），`tool:edition` 改读它。
- **两个 README 的分数可比性句子漏了 1.3.x → 1.4.0。** 发布说明、
  CHANGELOG 两处都声明了这次不可比，只有读者最先看到的那句枚举没跟上；
  先例就是 485f108（v1.3.0 把自己加进同一句）。该句是手写散文，无字节门可守。
- **bench 回放：全序列同状态重跑，并把 BENCH.md 自己的入列规则落成可执行。**
  回放夹具此前测每个 tag，而 docs/BENCH.md 写的是“与前任共享同一份
  `cli/src` 与 `core/app` 的发布不得有自己的行”。两者不一致，于是 v0.7.1 /
  v1.0.1 一直在表里，本次回放又要添上 v1.3.1 / v1.3.2——四行把机器漂移
  当版本差发布，正是该页页眉警告的那件事。现在 `bench_backfill` 自己按规则
  筛 tag 并具名打印被拦下的那几个；序列为 14 个版本 × 7 项 = 98 行，
  全部 measured 2026-09-01 且 dirty=false。另修一条夹具真缺陷：只有新 tag
  才真用到“它自己的 core”——v0.1.0 的 `ce` 把 core 当 `--core` 默认裸名解析，
  只有 daemon 路径读 `CE_CORE_BIN`，所以它会去 PATH 上找**本机安装的**
  core（当时答以 proto 差六个 major 而拒）；`with_core` 两路同指后才兑现
  BENCH.md 对每个 tag 的承诺。docs/BENCH.md 同时补上它一直欠的那句话：
  最新一行是哪个发布（标题是其他文档指向的锚点，故版本写在散文里）。
- **`bench.rs` 拆为两文件。** 加完上面两项后它到 365 行，过了 300
  行敕令；缝本来就在——量一棵树是一件事，决定序列持有哪些树、
  并把每个 tag 自己的二进制建出来是另一件。后者移入
  `cli/tests/it/bench_backfill.rs`（249 + 128 行）。
- **两仓各一次具名棘轮重立。** 主仓 = CHANGELOG 本条（+39 行，宽度
  +10）；子仓 = `unit/scan/chunk.rs`（+43）与 `it/bench_support/render.rs`（+15）。
  重立后主仓 check 953 / dedup 55，子仓 check 984 / dedup 119，两边 scan 0 fail。


## [v1.4.0] — 2026-08-31 — 递归增量：环内每个函数 +1

**无默认档位变更；判决语义有变——分数与 1.3.x 不可比。** 认知复杂度补上
S3776 的递归增量（p.8 / Appendix B1：环内每个函数 +1，直接与间接同价），
所以任何含递归的仓库分数都会动；本仓自身实测 27 个函数各 +1，而 `ce check`
恰好仍是 953。wire `scan/1` 加性升到 **6.5.0**。以下按落码顺序。

**先是一条测试面的修**（改动全在 `#[cfg(test)]` 可达的反事实通道与测试本身）：
v1.3.2 发版夜挂账的 Windows
守时 flake（`an_inert_canceller_leaves_the_worker_parked_and_counted`，CI
elapsed 1.306 s < 2.8 s，本地 30 次未复现）已根修。机制：负载下 worker 线程
在 spawn 与 `register` 之间停滞逾 300 ms，deadline 先落 `cancelled` 旗，而
inert 反事实只废掉了 `fire` 的取消、没废掉 `register` 的拒绝——worker 未及
park 即被拒回，grace 循环提前收到回复即退出，测试要演示的「泄漏」根本没上场。
修法：inert canceller 按其本义（pre-O64 根本没有 canceller）致盲 worker 侧
**全部** deadline 观察点（`register` 拒绝与 `fired()` 两处同扫），并新增
deadline=0 的竞态腿 `an_inert_canceller_arms_even_when_the_deadline_wins_the_race`
把该交错钉成必现——修前 100% 复现 CI 签名（27.6 ms、泛化拒绝、无 detached
残留），修后确定性通过。断言未放宽、未加 sleep、未 ignore。

**demo 从「渲染看到的」改为「断言必须看到的」**（指令彻查销 codex 审查债时，
本地五维对抗审查在现状树上提 20 条，逐条回源码核实后的统一修法）。根因一个、
面孔八张：demo 读的每条通道都按设计 fail-open——`ce probe --hook` 与
`ce audit --hook` 永不向外失败（缺核、索引未建、ce.toml 坏、diff 降级一律回成
沉默），`git` 退出码此前被丢弃，`ce erase --apply` 对任意行数（含 0）都退 0——
于是工具悄悄降级的一次运行会渲染出与量到的运行形状相同的表，`bless` 随即冻住
它：**字节门比对的是文件与它自己的生成器，看不出生成器说谎**（同 v1.3.2 教训）。
唯一看得见的是一份不从自身输出派生的期望，故新增 `demo/expect.js` 作「本次运行
必须观测到什么」的唯一所有者：恰两次拒绝且在第 1、7 步；守卫只能沉默或 `deny`
（`ask` 与带警告的 allow 都会被叙述成「落地」，按名拒绝）；首审必须 block 且理由
点名脚本化修复所答之事；修复后审计必须沉默**且量到了**——读它自己的
`.ce/observe.ndjson`，`degraded`/`skipped` 两字段分开「满意」与「没量成」
（audit/observe.rs 写者契约）；erase 恰好移除那一行具名文档孪生；六门只准退
0 或 1（退 2 是崩溃，旧码渲染成一次 FAIL 发现）。十条负向探针验明期望真咬、
健康形不误伤。`demo/README.md` 早写着「the run asserts the audit falls silent
after it」而当时代码只记录不断言——**改代码兑现文档，不改文档迁就代码**。
同批：`git()` 与同文件 `baseline`/`eject` 对齐，rc≠0 即就地 throw，并在 `run()`
基础环境钉 `GIT_CONFIG_GLOBAL`（指向空文件——`os.devNull` 不行，git 在 Windows
上直接拒 `\\.\nul`）+ `GIT_CONFIG_NOSYSTEM`，使贡献者全局配置到不了 demo 的任何
一次 git（含 `ce` 自己 shell 出去的）：同树一次进程内 A/B 实测，`commit.gpgsign`
为真且无密钥时无覆盖 rc 128、有覆盖 rc 0，投毒态下整条 demo 跑通（修前死在很远
的下游、报「erase --apply: worktree not clean」）；`normalize()` 加擦
`realpathSync.native` 两形（macOS /private/var 与 Windows 短名会把 mkdtemp 路径
漏进字节门产物，而 macOS 腿只在 tag 上跑）。**读者面四处过度声称同批修**：
①「唯一变量是守卫与审计在不在环内」漏说 with 那条环还跑 `ce erase --apply`，而
记分牌五数有两个（重复文档段、仍欠的删除）正是它产出的——四面（README 双语、
demo/README、生成的 cap 双语）改为点名第三件事；② 英文标签
`writes refused before the file existed` 对两次拒绝之一不成立（第 7 步写的
`web/api.ts` 在种子里已存在，是覆盖非创建；中文「文件落盘前」一直对），改为
`before they reached disk`，README 首节同改；③ Markdown 记分牌此前不带「两次都
仍以红色收场」而 HTML 形带——五行全是 with 列净胜，停在这里的读者会带走「有一边
干净了」的错读，故两形同带；④ Stop 判决在表里被截到第一个冒号（其后是审计点名的
证据）而 README 说「逐字输出」——截断处补 `…` 并在「真 vs 脚本」清单写明。另：
分数行 pass/FAIL 改读 `ce check` 退出码而非只匹配散文（措辞漂移的失败此前渲染成
pass）；`count` 同文件克隆收成一个所有者（js 在纯尺寸臂，这类克隆没有门看得见）。
产出 `node demo/run.js --check` 逐字节相同。
**ADR-006 具名（含补记两笔旧欠）**：本批越容差 `demo/tree.js` 88→124、
`demo/table.js` 175→193、`demo/run.js` 234→246、`demo/README.md` 138→152，
新入表 `demo/expect.js` 126 行、生成物 `demo/out/scoreboard.md` 与 `.zh.md` 各
7→9。**补记**：`b7c9786` 曾把 `demo/table.js` 100→157 并新入表
`demo/out/scoreboard.{md,zh.md,html,zh.html}`（7/7/9/9）而该段一个未具名；
`fdd8a6a` 的 `demo/table.js` 157→175 与 `site/style.css` 132→171 只写在提交
信息里，而规则（计划书 :210）指定的登记册是 CHANGELOG 该段——两笔在此补齐。
**勘误**：v2.21 展览段把 README 双语上升写作 158→170，实测 **148→170**
（`git show 991b42f~1:README.md | wc -l` = 148；同段上方那处 148→170 一直是对的，
该段自相矛盾），已就地改正而非在此追加覆盖。

**计划 v2.23 + 本册拆分**（记账，程序未动）。CoC 的递归增量（S3776 §1，M1 起
挂账）经三侧取证与两次用户拍板立册：口径站白皮书侧（v1.7 p.8 与 Appendix B1
皆写 each method in a recursion cycle, whether direct or indirect），形态定为
「能证明的那部分调用关系上的精确 SCC」——调用边是每解析单元内的词法作用域事实
归 Rust，环的判定归 Haskell，跨文件环据两个量过的数具名不做（名字铸边精度
0.576 / 绑定派生符号边召回 ~23%）。细则见 DEVELOPMENT_PLAN.md 的 ADR-008
第四期；计划书三行就地扩写，行数守 332。
新实体 `docs/CHANGELOG-ARCHIVE.md`（520 行）：本册抵到 750 行硬线，下一条版本
条目会被守卫当场拒绝，故按仓规「拆分优先于豁免」把 v1.3.0 及更早整体迁出——
条目逐字节搬家未改写，正册 749 → 251 行（净减，不触棘轮），册末留指向归档册的
一节，归档册头部反向指回本册。

**调用边（递归增量的语法半边，计划 v2.23 步 3）。** 新实体
`cli/src/scan/calls.rs`：给出**一个解析单元内**的 `(调用者, 被调者)` 边，
判据不是「函数体里出现了自己的名字」——那个读法在对拍语料上 Rust 命中 59 次、
Python 8 次而几乎全错（`DirEntry::path` 体内的 `self.dent.path()`、
`Request.prepare` 体内的 `p.prepare()`：同名不同类型）。被调者只有两条解析路：
裸名按**整名**匹配（Go 方法拼作 `(T) g`，故裸 `g()` 永远够不着方法），
或经调用者**自己的接收者**（`self` / `Self` / `this` / Go 的接收者绑定）取成员，
再按剥掉接收者前缀的名匹配。两条路都**按作用域**解析而非全文件查表：取成员只到
调用者**自己那个容器**里找，裸名只认容器是调用点祖先的可调用体。其余一律不建边；
一个名字被两个可调用体共有时也不建。
可调用体按 (父结点, 名字) 分组——Haskell 每方程一单元（D7）是**同一个函数**而非歧义，
而 `where` 局部的 `go` 与顶层的 `go` 父结点不同，仍互相抵消。
`LangSpec` 加四个字段 `call_kinds` / `call_name_kinds` / `call_member_kinds` /
`call_self_words`（结点名逐个用**仓内钉死的语法**探过，非凭印象），六张表各自填齐；
`cli/tests/unit/scan/calls.rs` 十四腿，负向四条各钉一种实测过的假阳性。
本片只出事实、尚无消费者，环的判定与 wire 属步 4。

**作用域是这条读法的承重件，不是修饰**——首版把查表摊平成全文件，对抗审查随即
在两条路上各给出一个**假环**，本机逐条复现：`class A: def run(self): self.step()`
与 `class B: def step(self): self.run()` 同处一文件，两个成员各自来自别处，摊平的
查表却铸出 `run ↔ step`；裸名同理，`def top(): helper()` 里的 `helper` 是 import
进来的，摊平查表把它接到了 `A.helper` 上。两个假环都会让核给根本不递归的函数各 +1,
而 `+1` 正是流进 `ce check` 分数与 E01 敕令的那个数——模块头写着「宁可少计不可错计」，
首版并没有做到。修法 = 容器化查表（`Named.containers` + `sees`）；两条路各配一条
**反证腿**：摘掉作用域检查即分别重现 `[run↔step]` 与 `[helper↔top]`，装回即消失。
顺带回收一个此前被误伤的真阳性：`impl A { fn path(&self) { self.path() } }` 与
`impl B { fn path }` 并存时，`self.` 已经证明了接收者，摊平查表却因重名而放弃。
棘轮具名重立：`cli/src/scan/calls.rs` 162 → 212 行（容器化查表 + `Caller` 记录 +
两个谓词 + 头注新增的作用域段），子仓 `cli/tests/unit/scan/calls.rs` 149 → 165 行
（两条反证腿）、本册自身 283 → 299 行。dedup 恒 56，`ce scan` 函数数 3825 → 3827。
`LangSpec` 此前把 `&'static [&'static str]` 逐字段拼了十八遍，任一窗口都与另一窗口
同韵——四个新字段把这段推过克隆阈值，代价 +7 块。修法是给这个类型起名
（`pub type Kinds`）而不是抬预算：重复本身消失，连带偿掉那段旧欠的 4 块，
**dedup 预算 60 → 56 具名下调**（净 −4，按台账规矩量在 fmt 之后的代码上）。
棘轮具名重立：`cli/src/scan/spec.rs` 277 → 315 行（四字段 × 五张表 + 字段文档 +
类型别名），`cli/src/scan/spec_hs.rs` 与 `cli/src/scan/mod.rs` 在容差内；本册自身
268 → 283 行（长出来的正是本条目）超 +10 容差，一并重立——**基线是快照，只能最后取**：
首次取在本条目写完之前，committed 树遂红了两腿（CI 33440319848 两平台同签名，
`baseline_bridge` 与 `site_roast` 都是这一条棘轮的下游）。
连带四处，全部经其自有通道具名而非静默：① 冻结成员 17681371623117319386 随那段
同韵消失，进 `RETIRED` 台账（12 → 13 条）；② 册 01 的七条 `spec.rs` 引文按新行重瞄
（改的是**引文标签**——resolve 按引文自述的行认领，只改账本会落到旧行），
随之一条三行锚点（`literal_delims` 到 `};`，被新字段从中截断）经
`CE_DROP_VANISHED` 具名销账；③ `gate:dedup.main` 芯片随预算 60 → 56 重投影；
④ 首页终端块按本仓新实测重出（check 950 → 953、dedup 60 → 56），中英两页各一处
——**站点需随下次发布重新部署**。归档册进 `facts_chips::DESCRIBES`：它在代码跨度里
**描述** chip 语法而非携带 chip，挂进去后门会真去验这条性质（把文件挪到仓根只是躲开门）。

**递归增量：环内每个函数 +1（计划 v2.23 步 4，wire `scan/1` **6.5.0**）。** S3776 白皮书
v1.7 p.8 与 Appendix B1 写的是 `each method in a recursion cycle, whether direct or
indirect`，而 SonarSource 自家 java/python/js 三个分析器一个都没实现（各自源码 `recurs`
命中 0）——**站规范侧做全**，与真实 SonarQube 分数的系统性差走 crosscheck 的归因栏。
分界照 ADR-008 细则第四期：调用弧是测量侧事实（步 3 的 `scan/calls.rs`），环的判定是
核的判决。新模块 `core/app/CE/Scan/Cycles.hs` 自己求 SCC——**不复用 `CE.Graph.Cycles`
本体**，它第一行就自陈 cycles are REPORTED never judged（RG9），本期是判决，两面不得混；
逐字沿用的只有它对单点的读法（`cyclic [v] = member (v,v) arcs`），故**直接递归无需特判**，
它就是环长 1。`+1` 这个政策常数全仓只在这一处。

wire 加性两键：请求 `callEdges=[[from,to]…]`（两端都是 `rows` 里的 cognitive 行下标、
表严格升序，名字与路径永不过线），应答 `cocBumped=[[rowIndex,生效值]]`。送**值**不送
增量，是为了测量侧渲染核判过的那个数而**永不自己重导**环或增量——`cli/src/scan/coc.rs`
只做下标算术与一条单调 `ensure`。四个读者（findings / 钉住镜像 `evaluate` / ADR-006
棘轮的复杂度列 / JSON 报告）因此读的是同一个整数：`scan::settle` 是唯一那条路，
`score::size_facts` 也改走它，否则一道门会对着另一道门从不展示的值收紧。

分块新增可测不变量「**一个文件的行不得跨 chunk**」：弧以行下标表述，边界落在文件内部
会把弧拦腰截断，而现有分块论证（rows grade independently，C5 评审）对跨行判决不成立。
分块因此改走**文件**而非行；装不下的单个文件按名拒绝，不劈开——劈开会静默丢掉跨切口的
弧，而丢一条弧就是丢一个环。`wire.rs` 被这段推过 300 行敕令，按「拆分优先于豁免」把
分块整体搬出为 `cli/src/scan/chunk.rs`（wire 336 → 243，chunk 105）。

**本仓库实测被记分的函数：27 个**——同一棵树上跑两个二进制（HEAD 的与本批的）逐函数
对拍，全部恰好 +1，名字清一色是 `walk` / `go` / `loop` / `visit` / `flatten` / `pairs`
这类递归形。这次度量当场抓出一个**错记**并已根修：`cli/src/corelink.rs` 的
`impl Drop { fn drop(&mut self) { drop(...) } }`——那个裸 `drop` 是 prelude 的自由函数，
而裸名根本够不着方法。新增 `LangSpec::call_member_scopes`（五张表逐个用钉死语法探过：
Rust 的 `declaration_list` 被 impl/trait/mod 共用故看父节点，TS 的 `class_body`/`object`
自己就说了算，Go 方法在顶层且名字带接收者、Haskell 类方法本就是顶层名，两者皆空），
裸名永不进入成员作用域；配一正一反两条腿——摘掉即重现 `[("drop","drop")]`，而
`mod m { fn a(){b()} fn b(){a()} }` 必须仍然成环，证明规则没有过宽。

核电池 `core/test/ScanCyclesProps.hs` 七腿，主腿是**全部 65536 张四顶点图**与一个
不碰 `Data.Graph` 的独立预言机（`ReferenceGraph.reachB` 的朴素不动点）逐图对拍；另有
单向调用不成环、只有 cognitive 行动且只动 1、加边不减免、无表即无变化无键、空的
`cocBumped` 也要答（「这里没环」与「压根没问」不得同形）、四条具名拒绝。golden：12 份
应答行按新核重出——**脚本先证明每一行除版本戳外逐字节不变才允许写盘**（第一版用文本管道，
Windows 把 `\n` 变成 `\r\n`，核把那个 CR 原样回显进了错误消息，改二进制管道）；
`scan/golden.ndjson` 另加 6 对，三接受三拒绝。VERSIONING §1 加 6.5.0 账本行，§3 三元组
105 → 111 行、server 恒答 6.5.0。

**先补一笔上一提交欠的账**：`9625914` 推上去时 CI 是红的（run 33448106023，两个平台同一步
`Dogfood the test suite`）——测试子仓的 dedup 从 119 涨到 122，越过它自己的预算。机制是**表
写得太长开始跟自己押韵**：`unit/scan/calls.rs` 的 `CASES` 每行都是 `(Lang::X, 片段, &[…], 为什么)`
这一个骨架，13 行时零克隆，我给作用域规则补的两行让它到了 15 行，就出现三块自相似。修法不是
抬预算也不是把行删掉，而是让那四行**说出它们本来的意思**：它们回答的是同一条规则（被调用者
只在调用点看得见的地方才被认领），且天然成对——一个必须拒的形状，配一个必须仍然认领的邻居。
于是它们独立成 `SCOPE_CASES`，**每行同时装两个片段**，元组形状与主表不同，两张表不再互相押韵。
配对本身就是断言：只有反证腿证明不了规则没有变宽。结果：子仓 dedup 回到 **119**，且**行集与
上一次绿（`7c0c637`）逐对相同、零差**；子仓棘轮具名重立一行 `unit/scan/calls.rs` 165 → 205。

棘轮具名重立六行：`cli/src/scan/mod.rs` 189 → 248（`settle` 那条唯一通路）、
`cli/src/scan/spec.rs` 315 → 333、`cli/src/scan/report.rs` 290 → 307、
`cli/src/scan/calls.rs` 212 → 239、`core/app/CE/Scan.hs` 270 → 281、本册自身
299 → 362 行；其余在容差内。**dedup 预算 56 → 55**（台账在 `ce.toml` 注释块）：
行集是拿 HEAD 工作树逐对比出来的，最终只差一行，其余差异全是行号位移的换键。
本批曾花掉一行——`scan/wire.rs` 长出第三张可选表之后，它组装请求的形状与
`structure/wire.rs` 一模一样——**按机制退回**：三个 `if` 改写成一张可选表，我们自家工具
匹配上的那个形状就此不存在，而不是把它记进豁免。偿的一行：`scan/ast.rs` 的 `children`
与 `named_children` 除了调哪对访问器之外一字不差，折成一个 `kids` 走查。净 −1。
分数：CoC 变了，**与 1.3.x 不可比**。

**交叉校验重标定：四份语料零移动，而重标定本身抓出一个缺陷（计划 v2.23 步 5）。**
拿新旧两个二进制在同一棵树上逐函数对拍四份语料（go 52 / python 118 / rust 319 /
typescript 25，键 = 路径 + 起始行 + 名字），**没有任何一个单位的 CoC 变化**——
`DIVERGENCES.md` 里 gocyclo 52/52、lizard 102/104、RCA 322/322、gocognit 29/32 那几行
全部原封不动，因为四份语料里没有一个我们能证明的文件内环。

首轮曾有**唯一一条**移动，且是**误记**：`ignore` crate 的 `walk.rs:2215`
`fn symlink { use std::os::unix::fs::symlink; symlink(src, dst) }`——体内那句 `use`
是最内层绑定，裸调用是被导入的那个函数。这与 `impl Drop { fn drop }` 是同一族错误
（裸名不是它看起来的那个可调用体），修法也同族：新增 `LangSpec::call_import_kinds`，
**一个单元自己体内的导入所绑定的名字，裸调用永不认领**；Rust 填 `use_declaration`、
Python 填两种 import 语句，Go/TS/Haskell 留空并写明理由（Go 导入是包限定的，TS 与
Haskell 直接拒绝同作用域重名，语言本身就不允许这种撞名）。配一正一反两条腿——摘掉
规则即重现该误记，而导入的是**别的**名字时递归仍照计。修后四语料全部零移动。

**锚是推导的，且推导成两面可对。** 白皮书六道例题无一含递归调用，所以没有现成的
计分例题可抄。底数取 p.10 的 `sumOfPrimes`（页边判分 7，`sonar_whitepaper.rs` 已对着
同一页边钉住），给它加一句丢弃返回值的自调用——调用本身不是结构增量——于是**同一份
源码**的环前读数必须仍是 7，结清后必须是 8。差值就是被测的那条规则。新电池
`cli/tests/it/coc_recursion.rs` 五腿：这个两面锚、互递归与直接递归同价、**只有环内
成员付钱**（环外调用者与无环链各 0，这条腿否掉「能到达环就 +1」的实现）、跨文件环
不建边（具名不做的立场以断言留存）、以及与 **gocognit 实测对拍**——同一份探针
`fact`（一个 if + 自递归）两侧都是 2、`plain` 两侧都是 1，**直接递归逐值一致**；
互递归那一对就是两个实现分手的地方。这些腿走 `scan::settle`，因为
`common::measure_units` 按设计答的是环前的数。

**三处登记就地改写，不追加平行条目（步 6）。** `cognitive.rs` 头注那句「recursion
is not detected — needs a call graph (M5)」改写为这个模块**为什么**仍然只答环前的
数（结构族是唯一不看复杂度的读者，它应该继续拿到那个数）；`coc-haskell-divergences.md`
的 D4 改写为「循环由惯用递归承担，而递归轴自本期起收费」并指向新电池；
`crosscheck/DIVERGENCES.md` 那句「仍未实现」改写为已补齐并加一节，登记与三个
SonarSource 自家分析器（java/python/js，2026-08-31 三仓源码第一方核对，`recurs`
零命中）之间的**系统性正偏**：凡在递归环里的函数我们比 SonarQube 高 1 分。这不是
缺陷，也不会在任何语料上被「修掉」——是两侧对同一份规范的取舍差。

**勘误（本册自身）**：上一条提交把步 4 的条目插进了步 3 条目的**段落中间**——
我匹配的锚是一行软换行，不是段落结尾——于是「`LangSpec` 逐字段拼十八遍」那段被
劈成两半、夹着整条步 4。三条条目已按顺序复位（步 3 整条 / 步 4 整条 / 本条），
条目文字逐字未改。

棘轮具名重立：`cli/src/scan/spec.rs` 333 → 347、`cli/src/scan/calls.rs` 239 → 274、
本册自身 362 → 407 行；子仓新入表 `it/coc_recursion.rs`，`unit/scan/calls.rs`
205 → 216 行。dedup 主仓恒 55、子仓恒 119。分数：与上一条提交相比只有
`ignore` 语料那一处误记消失，本仓自身零变动（`ce check` 恒 953）。

**两个 README 改由「产品在做它那件看得见的事」开场（用户 2026-08-31 提）。** 用户看到
Claude Code 里守卫拒写的截图后问：开头能不能放这种效果图，直观且简短，一眼看到效果。
此前首图是架构 SVG——它讲的是机器，不是效果。现在开场是一张**终端卡**：代理请求新建
一个文件，`ce` 当场答出这段内容重复了哪块已索引区域、以及换成什么顺序就能通过。架构图
下移到「技术栈」一节，那一节本来就是讲机器的。

**卡片不是截图，是生成物。** 仓规只收生成件，所以它由已有的那条链产出：`demo/vignettes.js`
的第一幕本来就要演一遍（README 下方那段 console 块就是它），现在同一次演出的行
经 `demo/render.js` 再画成 `demo/out/hero.svg` 与 `hero.zh.svg`——**同一次捕获出两形**，
不存在第二份会漂走的素材；每一幕都自带 `expect`，降级成 `allow` 的运行会当场抛而不是
把「守卫什么也没做」的图冻进仓库；`node demo/run.js --check` 逐字节守。

**中文卡片当场逼出渲染器的一个真缺陷**：`wrap()` 按**字符数**折行，而画布宽度钉死在
`COLS * CHAR_W`——一个汉字是一个字符、两列宽，于是中文行会画出右边界而没有任何东西
察觉（此前这个渲染器只见过 ASCII）。改为按**显示列**折（CJK 与全角计 2）。纯 ASCII 行
折在原处一字不差，证据是旁边两张既有 transcript SVG **一个字节没动**。

棘轮具名重立：`demo/render.js` 104 → 145（列宽三个小函数 + 反向读 `typed`）、
`demo/vignettes.js` 119 → 133、`demo/README.md` 152 → 165（新增说明两张卡片的那段——
每个产物都靠被说明挣到入边，`ce deadcode` 恒 0 死件），两个 README 各 182 → 184（容差内）。本册自身
407 → 429 行。

**官网对拍从「每次凭记忆重写一遍」变成仓里的一支脚本** `scripts/verify_site.js`。
它把八页逐个与 `git show HEAD:<page>` 的 blob 对拍，判据写死为「至多一处 Cloudflare
边缘注入的 beacon，剥掉它之后与 blob **逐字节相等**」——不是「剥完摘要相同」，
后者会把第二处真差异连同第一处一起吞掉。这支仪器此前被写错过两次，两处都记进头注：
beacon 是**带属性**的 `<script>`，按 `<script type="module">` 字面写的正则永不匹配、
会把整站报成变了（红的是仪器不是站点）；它还**连所在行的换行一起注入**，只剥标签会让
每一页都差一个字节。本次落地前先拿线上跑过：四页未改的逐字节相同，四页本批改过的
按名报差（首页与 zh 首页的 roast 块、两张 stack 页的 proto 6.4.0 → 6.5.0）。
`docs/RELEASE.md` §3 的「部署后对拍才算上线」改为指向它。

## [v1.3.2] — 2026-08-31 — 结项：路线图改写为永久立场

**无默认档位变更；判决语义零变动，分数与 v1.3.0 / v1.3.1 完全可比**——`git
diff v1.3.1..v1.3.2 -- cli/src core/app` 为空，wire 亦未动（proto 6.4.0、
graph/1 6.4.0、索引 schema 15 原样）。**故为 patch 而非 minor**，与 v1.3.1
同理：版本号描述的是制品，不是仪式，而本批改的是文档、站点、演示与发版门。

**项目结项（计划 v2.22）。** 三个具名后置束——M（评分与评测 12 + 产品小项
9）、N（分发 20）、证据门 4，共 45 条——**一次性裁定不做**：不是遗忘，也不是
无限期后置，v1.3 的形态即成品形态，此后只做维护。读者可见的那几条从「路线图」
改写为「已知限制」里的**永久立场**，逐条今日复核而非照抄 2026-08-26 台账
（该台账已至少一处过时：O83「桌面无应用内更新通道」在 v2.20 被逆转，已交付为
`ce update`）：符号层存活性只出顾问行（`ce deadcode` 末行自陈）、守卫类在拿出
自己的误报记录前停在 `observe`、`ce structure` 不设分数地板故只报不守、发布只
构建三键而启动器能算出五键（多出的两键无 pin，回落到 PATH `ce` 或源码安装）、
marketplace 条目跟 `main` 而非发布。任何一条日后复活须走一次新的 plan-set。

**门比对的是文件与它自己的生成器**——生成器说了假话，它一个字也看不出来。
v1.3.1 发布后，README 双语与官网两张首页仍写着 v1.3.0 是「最新版本」，而每道
字节门全绿：那句标题读的是 bench 系列里**实测最新**的版本，而 v1.3.1 因
`cli/src`/`core/app` 未动**不入列**（同一程序再测一行，等于把机器漂移当版本差
发布，正是 BENCH 页眉警告的那件事）。修法不是改那个数：标题只说实测版本，另派生
一句「当前发布 vX 没有自己的行」，并补 `names_the_release`——全仓唯一读
`CARGO_PKG_VERSION` 而非读合同的断言（负向探针：版本调回即四面同时红）。
`docs/BENCH.md` 补上从未写下的规则：一次发布何时入列，不入列时页面欠读者什么。

**演示以记分牌开场。** 十二行散文标签把结论埋在表里，现在五个数字先说话——
写入前被拒 0→2、残留克隆块 4→0、重复文档段 1→0、仍欠的删除 1→0、检查分数
952→979；表留在下面作细节。首页此前**根本没展示 before/after**（「看它工作」
是三张 GUI 在**读**报告的图）。一份行表出两形，故首页不可能引用 README 表里
没有的数：两面同经 `EMBEDS` 由既有 `demo/run.js` 检查、`demo/bless.js` 拼接。
形状先拍后改——`.install` 芯片只有一个值槽，五个数字会从五个不同的 x 开始；
改真表三列（两数对齐、表头点名哪次运行、640px 以下每行堆叠且每数自带运行名），
CSS 一份进 `site/style.css` 而非两份页内，八页随之 `?v=2`→`?v=3`。两张表此前把
同两列拼成两种写法（`Without` / `without`），现由 `heading()` 一处产出。
`ce deadcode` 当场判两个新 .md 产物为死件**且判对**——根修是 `demo/README.md`
补上说明它们的那段，不是加个链接把门哄绿。

**分句符收成唯一所有者 `join(zh)`。** 中文句号后不加空格是排版事实，不是偏好；
`unmeasured_note` 早知道这件事，而三行外 README bench 块接链接时两语都硬编码
一个空格，中文遂读作 `。 [`。全仓只动一个字符。仓内中文面已全扫，`。 ` 零残留。

**提及宇宙增量快路：先量 ROI，然后划掉。** 提及那一遍按设计把宇宙内每个文件读
出来算内容哈希，不信 mtime。快路（(mtime,size) 预筛 + 内容哈希兜底）省下的不是
这一遍，而是「读」换成「stat」的那点差：自仓宇宙 832 文件 / 10.29 MiB，实测读
155 ms、stat 32 ms，净上限 **123 ms**——比它前面那一问 `git ls-files`（151 ms）
还小，最多触及全命令的 ~9 %；而 `Advisory::Yes` 全树只有一处生产点
（`cli/src/graph/deadcode.rs`），门、`ce scan` 与钩子都不付这笔钱。故划掉，
算式与理由记在 `docs/PERF-BUDGET.md` 的读法行。

**发版门等的是 macOS 排队，不是构建。** v1.3.1 首打被 `verify-publish` 按
`checks did not settle in 30 min` 拒发，而谁都没错——`build-macos` 那一刻排队
30.6 min 从未开工，两条 push 腿已绿 17.7 min；原预算照的是「推送后 18m46s 跑完」
这个**构建**耗时。预算 30 min → 2 h，且每条超时消息点名未完成的腿；runbook §2.2
同批写明这段等待。（同批把该注释里两个凭印象的数字改成实测值。）

**记账。** npm 1.3.1 已由用户在自己终端发出（`dist-tags.latest` 实测 = 1.3.1），
v1.3.1 自此四渠道齐平——CHANGELOG 的渠道行按实测而非按「应该发了」记。

## [v1.3.1] — 2026-08-30 — 站点图改生成 + 两个分数各自具名

**无默认档位变更；判决语义零变动，分数与 v1.3.0 完全可比**——`git diff
v1.3.0..v1.3.1 -- cli/src core/app` 为空：度量侧与判决核一行未动，改的是
截图的产生方式、GUI 两处文案、文档与站点；wire 亦未动（proto 6.4.0、
graph/1 6.4.0、索引 schema 15 全部原样）。**故为 patch 而非 minor。**

**两个分数各自具名。** `ce structure` 与 `ce check` 都给 0–1000 的分数，
却不是同一把尺子：前者量树尺度的熵，后者量门自己的七轴并背着棘轮与地板。
同一个仓库在两者上分别是 832 与 952，哪个都不是「那个分数」。控制台一直
分得清（`structure score …` / `检查分数 …`），两块屏幕却都只印数字——读者
在标签页之间一走，或在官网的终端块与截图之间一看，就无从分辨。现在两屏
各自带上控制台的原词（en/zh 皆有，`gui/ui/i18n.js` 加 `scoreStructure` /
`scoreCheck` 两键），官网图注也点明哪个是哪个，`docs/reference/gui.md`
记下理由：它们无法对齐，因为量的根本不是同一件事。

**首页三张 GUI 截图改由产品自己生成**——ADR-009
（随版本变动的文档一律派生）最后一个未派生的面。`scripts/shoot_gui.js`
用无头 Edge 跑真 `gui/ui`：Tauri 在 Windows 上就是画在 WebView2 上，
所以这是产品自己的像素而非仿制；webview 的 `invoke` 桥是浏览器唯一没有
的东西，用 CLI 出的三份报告文档（structure / join / dedup）顶上——那**就是**
webview 会收到的那份：`ce structure --format json` 打印的是
`structure::report::report_json`，而 Tauri 命令返回的 `faces::structure` 调的是
同一个 `report_json`，跑在同一次 `structure::judge::run` 上（join / dedup 同形）。
新门 `it/site_screenshots.rs` 五腿：图不得比 `gui/ui` 旧（git 祖先关系；无历史的
浅克隆按名拒绝而不空过）、每张是整窗 1424×892、alt 文本不得手抄数字、
`contracts/gui-shots.json` 收据里的 schema 必须等于代码当下声明的三个
`SCHEMA_ID`（第一腿拦不住的那条路：界面不动而报告形状动了）、以及
`site/assets/` 里每个文件都得有门认领。2026-08-22 手摆的旧图曾同时错报四项——
八格页签（现十格）、`ce.join-report/0.1.0`（现 0.3.0）、把已是子模块且
只读不测的 `cli/tests` 量在树内、以及 alt 文本里一个结构分数 854（现
832）——而站点上没有一处能发现：它们是唯一无法被重新推导的面。重拍步骤
记在 RELEASE.md §3。测试侧 `git_out` 提到 `common/gitio.rs`（原址
`history_recipes.rs` 退让，克隆行清零——由 `ce` 自己的写入守卫拦下）。

**拍摄可复现**（发版前自查抓出）：同一份报告集连拍三次，`gui-tree.png`
出三个不同摘要、`gui-candidates.png` 出两个——每张图都以点一个按钮开
场，而 `gui/ui/style.css` 给按钮 0.12s 的背景过渡，两帧后按下快门就落在
插值途中的随机一点。这不是观感问题：图不可复现，就没法再问「committed
的这张还是不是当前的」——重拍永远不一样，而不一样永远不说明什么。修法
用产品自己的开关：应用本就应答 `prefers-reduced-motion`，于是取景器用
`Emulation.setEmulatedMedia` 声明它，过渡根本不开始。修后三连拍三张全部
逐字节相同；`gui-candidates.png` 随之重拍（旧图正是拍在过渡途中的那张）。
第六腿 `it/site_shoot_motion.rs` 守住这对耦合——CSS 规则与 CDP 调用分处
两文件、各自失效都无声，删任一半即红（两侧都实测过）。同批修掉另一处：
`--out` 指向仓外时 `shoot_gui.js` 仍改写 `contracts/gui-shots.json`，一次
探针跑就让收据记上站点根本没有的那三张图的摘要；收据只在拍进
`site/assets` 时才写。

**判决流图右侧 39% 是空的**（用户报，发版前）：`docs/diagrams/judgment.{en,zh}.json`
的 `meta.viewBox` 是手写画布尺寸 `[1000,760]`，而内容只到 x=614——archify 左对齐、
按内容排版，不会把泳道摊满画布，那 386px 纯属多申请。收到 `[650,760]`（右边距 36px
与左边距对称）重渲后**版面零重排零裁切**（节点坐标一个未动，overflow=0），中文孪生
另单独渲染核对——汉字宽于拉丁字，收窄画布最可能在这里裁字，实测未裁。`architecture`
两图边距 3%，按名不动。同批修掉中文图的语言泄漏：图例标题 `Legend` 是 archify 模板里
的字面量，任何 IR 键都够不着，故在我们自己的序列化器里加 chrome 映射表
（`scripts/diagram_svg.mjs` 的 `CHROME`，`extractSvg` 收 `lang`）；门腿
`docs_diagrams.rs` 第五腿**读回那张表**再逐项断言 zh SVG 不含该词——表与文件不会各说
各话，负向探针双侧成立。

**README 双语对齐最新内容并瘦身 170→168 行**（六路读者 + 逐条对抗核验，41 条确认）：
① 「文档要么生成要么门控」那份清单漏了自己的三个成员——本页由代码派生的数字、上方两张图、
官网终端块与 GUI 截图，全部补上；② 设计规则列到 ADR-008 为止，ADR-009 已于 2026-08-29
通过并随本发布线交付，补入双语；③ 「挪几行骗不过的分数」改称**检查分数**并点名它自己的
七轴——上一条讲的是结构的七轴，两者不是同一张表，这正是本版 GUI 两屏具名要解决的混淆；
④ **`count:axes#word` 一个事实服务两张名册的登记册缺陷**：该芯片 linked 到
`score::knobs::AXES`（检查分数的轴），却渲染在结构句里，两者今日恰好都是 7 故散文读着
为真，任一族增减即另一族说谎且无人察觉——新登记 `count:structure_axes#word`，scraped
自持有该名册的判官 `core/app/CE/Structure/Axes.hs::axes`，SCRAPED 档 21→22，双语两处
结构芯片改指；⑤ 五处复述式冗余就地删（哲学条已有的整句、图注上方两行已说过的、开场
第 1 步的预告、pin 校验的三次重述、误报记录那句的第三次），并把开头两段并作一段。
引文 `methodology.md:34` 的锚随之重瞄（键 b86dae35→ae06d5b3，标签 README.md:156→154）。

**记账**：版本 1.3.0→1.3.1 五处 + 两 Cargo.lock + hello-ok golden 回显 +
`contracts/docs-facts.json` 投影一条。check 主仓 953→952（地板 946）/ 子仓
984 恒（地板 983）；dedup 主 60 / 子 119 恒。首页终端块两语随树重取
（`site_roast` 门量了才写，两语一次取完）。棘轮具名重立一处：`CHANGELOG.md` 544→557 行，增量
即本条发布说明，容差 +10 不够。cc-memory 插件把本地状态目录 `memory/`
改名 `.ccm/`（2026-08-30），`.gitignore` 按同样的锚定理由钉住新名，计划
§5.10 的布局树与 `layout_tree` 门里 ignored-by-design 的那个名字同批改到
新名——一次 `git add -A` 曾把 106 份本地笔记推上公开仓（32b65f6），该
提交已从 main 上撤出。

**v1.3.1 发版 无默认档位变更**（两段式按 RELEASE.md §1–§3 走完）：draft run
33340289362 出十资产（二进制只来自三 OS 矩阵）→ pin `plugin/bin/manifest.env`
十一行 f43c099〔`ver:pin#v` 由清单派生，投影同批重 bless〕→ tag。**tag 首打的
verify-publish 按超时拒发**：`checks did not settle in 30 min`——而本次谁都没错，
`build-macos` 压根没出队。它只在 tag 与每周一的 schedule 上跑（ci.yml:311），
故每次发版都是它的冷启动，而 GitHub 何时调度一台 macOS runner 不由本仓决定：
拒发那一刻（00:19:39Z）它已排队 30.6 min 且**从未开工**，两条 push 腿则已绿了
17.7 min。原预算是照着 v1.3.0 那次 macOS 腿「推送后 18m46s 跑完」定的——那量的是
构建，而循环等的是排队。根修在 c167ab7：预算改按排队定（30 min → 2 h），且每条
消息点名还没完成的腿（旧行只打个计数，拒发记录因此说不出缺的是谁；新过滤器对该
commit 实跑打印 `build-macos(queued)`），runbook §2.2 同批写明「发布前先等 tag
自己那次 CI」。tag 未挪：macOS 腿出队后三平台全绿，重跑该 job 即 publish 成功
（2026-08-31T00:58:01Z，十资产）。12fa295 另把该注释里两个凭印象写下的数字
（50 min / 40 min）改成实测的 30.6 / 17.7。渠道：GitHub Release 十资产、crates.io
1.3.1（305 文件，含 93 份 `tests/unit/**`）、官网 20/20 逐字节对拍相同（HTML 剥掉
Cloudflare 注入的那一条 beacon 后比；本次实测该 beacon 是 `<script type="module">`
形，不是旧脚本假设的 `<script defer>`）、npm 1.3.1（指针包，2 文件 1430 B，
2026-08-31T01:33:33Z 由用户在交互终端发出——passkey-only 账户在非交互 shell 里
恒 EOTP，见 RELEASE.md §3）。**四渠道齐上。**`ce update` 实网冒烟：读到 tag v1.3.1 与
**该 tag 上**的 pin，ce / ce-core / installer 三值与 SHA256SUMS 相同，本机 1.3.1
退 0。

## 更早的版本

v1.3.0 及更早移入 [归档册](docs/CHANGELOG-ARCHIVE.md)（2026-08-31，本册抵 750 行
硬线所致的拆分；条目逐字节未改）。
