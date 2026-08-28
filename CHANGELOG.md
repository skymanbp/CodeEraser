# Changelog

> 记录义务来自 DEVELOPMENT_PLAN.md §4.2：“每次默认档位变更在
> CHANGELOG 记录依据（FPR 数据）。”本册只记守卫档位变更；
> 功能发布史见 GitHub Releases。

## [Unreleased] — L 轮片 (2)+(3)+(4)+(5)+(6)+(7)+(8)（无默认档位变更；bit 0 生产者三修，erase reason 6 人口随之变动；secrets 排除表加宽，判决宇宙随之变动；`ce:allow` 解析合一，旧宽松形三收窄一放宽，自仓一段转活；TS 星号再导出自 `export_from` 分出 `export_star` 站点标签，冻结自仓切片一行改签；graph/1 **6.2.0** 加性两表 + `export_unmentioned` 顾问类，`ce deadcode` 报告 0.3.0 加性两键，片 (7)+(8) 第三键 `unmentioned_cut` + `ce graph --mentions` 报告 0.2.0 加性 `rates` 键；步 #8 提取面补强 GRAPH_REV 13→14，站点标签零新增、边与符号域拓宽）

**片 (6) 无默认档位变更**。wire ce↔core **6.1.0→6.2.0**（加性 minor，三面原子：`corelink.rs::PROTO` + `Version.hs::proto` +
全族 golden 机器再生，非 handshake 的 request 行按 §3 立场滞留 6.0.0——105 行）。`graph.request` 加性**两键同生同死**：
`unmentioned=[[node,vis,conv]]`（`id` 投影升序）与 `mounts=[[node,private,total,bits]]`（全节点恒一行、`take 1` 投影）；
只发其一 ⇒ 配对拒绝（`unmentioned: mounts table required alongside` / 反向），占 `violation` asum 最前；行级 5+4 条具名
拒绝；两表**各自**析取项计价（`mountCap`=131072 / `unmentionedHardCap`=524288〔= 最大既有表 `edgeCap` 同阀，非软 cap 倍数〕，
节点净空不动；本方生产者自限 131072 行下顾问表永不把门翻红，超硬阀只对缺陷/敌意客户端可达）。回复
加性 `exportUnmentioned=[[node,vis,conv,code]]`（`vis∧3==3` 且 `conv` 无 0..10 任一位者出行；code 全序 1 private > 2
restricted > 3 reexported > 0 public，`CE.Graph.Advisory` 具名谓词链 `mountedPrivate`/`pkgPrivate`/`reexported`，缺 mounts
行读作 `[0,0,0]`）；> `unmentionedCap`(131072) 行 ⇒ `exportUnmentioned:[]` + `unmentionedDropped:true`（只在掉表时在场）。
**铁律腿**：`AdvisoryProps` 十电池（K16 核半 / K19+K36 旋钮 / K33 七拒 + `total=0` + 131072/131073 + `nodeCap−|unres|`
双向反事实 + 零行格 / K35 四格 / codeOrder 九节点矩阵〔含 1>2 碰撞格〕/ ironRule 五种表形 dead 集恒等）；`Verdict.hs rowTotal`
补计 `symbols`（K47，`VerdictWireProps` 双向腿：仅 `symbols` 524289 行 ⇒ `verdict_too_large`、524288 行不降级）。Rust 生产者：`mention/candidates.rs`（域 = (文件, `mention_name`) 折叠、否决序 = 他文件提及 → 折叠门
`segments≥2∧chars≥FOLD_MIN_CHARS` → 自文件具名区域〔Go 模板串 / TS 字符串与模板字面量 / Python doctest / Rust 宏定义与
文档围栏块 / Haskell haddock 围栏〕→ conv = AST 半 | 名表半 | 路径半 | 文本半，`take UNMENTIONED_SOFT_CAP` 自限）、
`mention/conv/name.rs`（名表半：Test 路径族 + `benches`/`examples` 仅限 Cargo 包内、Ambient `.d.ts` 族、Main py/hs、
Protocol = Python unittest/xunit/pluggy/Django/反射前缀 + TS 文件名×导出名表 + Haskell `Paths_*`/hspec、Go 接收者 ⇒
MemberApi/MemberDispatch、`ce:allow(unmentioned)`）、`selfref.rs`（tree-sitter 区域抽取，行注释节点含尾换行故末行 = `end.row−1`
——首稿 split 案由此抓出；文件按索引侧同法 lossy 解码，杂字节不再清空自文件例外区与 allow 声明——对抗审查抓出）；`wire_of(root, idx, db, Advisory::{No,Yes})` 独家参数化（`ce deadcode`/GUI/MCP 走 Yes，`erase`/
`join`/`score`/`structure`/canvas 走 No 各带 why），`request_body` 零行为提取为 `pub fn`（K6 三腿共用，`graph_export_surface.rs`
手抄体删除；spec 写 `pub(crate)`，集成测试够不着，**勘误 ⑩** 记为 `pub`），`consume` 第四参名表 → `Report.unmentioned:
Option<UnmentionedFace{Rows,Dropped}>`（K38 两腿各自具名拒绝：第一等式按封版后勘误 ⑨ 改「核出行 (node,vis,conv) 键集 ⊆
生产者名表键集」= 查表即腿、第二腿值侧非空；对抗审查抓出首稿节点集 ensure 为恒真死码，已删；回复既非 degraded 又无
`exportUnmentioned` ⇒ 具名拒绝——前 6.2.0 核合法偏斜不得读作「已问且干净」）；`ce.deadcode-report` **0.2.0→0.3.0** 加性两键〔片 (7) 补第三键〕
`unmentioned`（行 `{name,symbol,line,code,why}`）/`unmentioned_dropped`，只在 Yes 路在场（K43）。it 腿：K6 双路 + K16
(a)(b)(c1 全提及树空表 + c2 mounts 全节点)、K30 code 半（真核 14 行：六格矩阵 + 两碰撞格 + 再导出 + Go 两形 + TS 星 + cabal
两事实）、K43、K46 `UNMENTIONED_SOFT_CAP`↔`unmentionedCap` 源对源一数、K31 分段器直接表（`PyProject`/`HTTPServer`/`RULES`）+
e2e `PyProject`↔`pyproject` 折叠得救。**封版后勘误 ⑨**：K36「`exportVisBit` ∉ `exemptCategories`」字面不可满足（vis 字与
conv 字异域），落码为「豁免只读 conv、vis 永不豁免」+ `11 ∉ exemptCategories`；K38 第一等式在核按掩码/豁免筛行后不可能相等，
落码为键集查表。对抗审查 Workflow（6 镜头 23 条 → 14 确认〔去重 8 缺陷〕/ 9 驳回 / 0 存疑）全修：cap 注释改封版口径、K47 补腿、
selfref lossy、K38 恒真 ensure 删除+两腿拆分+前 6.2.0 核拒绝、golden 20 补 code 0 行、ironRule/codeOrder 补格、分段器两腿、
`MENTION_REV` 台账补分段器、`*.test.*` 任一出现位命中；E01 拆分：`wire_of`→`advisory::tables`、`Graph.hs result`→
`advisoryKeys`/`liveness`、K30 code 半独立模块。

**片 (7)+(8) 无默认档位变更**（2026-08-28，ccm 步 #7）。渲染面：`ce.deadcode-report` 0.3.0 加**第三键** `unmentioned_cut`
（生产者在 `UNMENTIONED_SOFT_CAP` 处整对截断的事实，核看不见故只能本地说——K38 自限腿落为生产者单元腿：`candidates::cut`
恰 cap 不截、cap+1 截且前缀恒同；131,073 节点 e2e 不可造，一个节点的 (vis,conv) 变体只有数种）；`Report.unmentioned` 三态
`Rows{rows,cut}`/`Dropped`/`None`；控制台每行 `advisory: name:line  symbol  code  (why)` + 按码普查行 + 两条本地降级行
（截断/掉表），中文同形；MCP `deadcode` 工具同文档（描述句补顾问）；GUI Graph 屏加载 `deadcode_report` 与画布并列、按路径
join（渲染 join，非判决）：选中文件列出其未提及声明（行/符号/码词），根视图带全树按码普查 + 两条通知，悬停计数；i18n 五键 +
`graphNullWhy[1]`/`emptyGraph` 重写（O09/O17）；K44 门 `gui/tests/hub_projection.js`（0.3.0 行过 `hubTable` 后 `symbol` 列仍在，
CI 两平台接线）。语料面：`ce graph --mentions` 0.1.0→**0.2.0** 加性 `rates` 键 + 每语言一行控制台——K23 普查 `mention::rates`
（`declared{all,exported}` / `unmentioned{all,exported}` / `vetoed{other,fold,self_text,**collision_saved**}`——末者 = 只因他
文件同名声明得救者，§6 碰撞失明成数字）；`store::mentioners`；否决序抽为 `candidates::veto` 单喉供生产者与普查共用。**U 公式钉**
（`tests/it/mention_universe.rs`）：git 在 `.gitignore` 单源下的列表（`--cached --others --exclude-per-directory=.gitignore`——
walk 不读 `info/exclude`/`core.excludesFile`，本机排除文件不得移动 U）− walk 每条规则一项：按名剪除（`.git`/`.ce`）、嵌套仓
（git 列作 `sub/` 一项）、tracked 而被 `.gitignore` 模式命中者（walk 读模式不读索引，`--ignored` 取）、排除表、盘上无常规文件
者（已删未暂存/目录链接）、4 MiB、早 NUL——每项用 walk 自己发布的谓词（`mention::{cut,excluded,FILE_CAP,decode}`）算，
`Formula` 八项在打印行内各成列（`listed − Σ = U` 行内自闭），scratch 仓逐形见证腿 + 自仓 CI 常钉 + 外部四语料 `--ignored`
腿同式钉（自仓本树 627 = 640 − 13；cobra 65 = 66 − 1 / requests 118 = 130 − 7 − 5 / ripgrep 230 = 237 − 7 /
zod 536 = 583 − 45 − 1 − 1）。
**K23 仪器腿** `tests/it/eval_mention.rs` + `eval_support/mention.rs`（产品自己的 `token::{emit,runs,whole_run_only}` 与
`rates::declarations` 域，一次运行出全部数——L7-F6 律）：`$` union 臂 ② 行成本两 U 列（zod 438/146、requests 449/3、ripgrep
61/61、cobra 24/24、自仓 62/62，与封版 spec 参照列逐数相同）；① 顾问结果差**预登记 0 五树全成立**——仪器按生产者的三条否决通道（身份 / Rust 折叠 / 自文件例外区）两臂同问；只看身份
通道曾示 zod 2 行（`ZodBase64URL` `schemas.ts:939` 与 `ZodExactOptional` `:2148`，裸名无他文件拼写而 `$` 孪生见于
`core.mdx`/`wiki/optionality.md`），但两者在严格臂下亦被自文件字符串字面量 `$constructor("…")`（:943/:2155）否决——
对抗审查抓出的假警，腿断言空并打印行；JS 臂连带 zod 624 / 444 / **0 域内**（自仓 8/5/0）；`$` run 形 2392/5416/119/10 与参照同；
`test` 单数四语料 0（自仓 22，只报）；ripgrep 包根 Test 规则恰四目录。**258 行外部顾问逐条处置**（Workflow 10 处置者 + 10
推翻式核验者，8 条改判、**0 条 veto 缺陷**——仪器主张 258/258 成立）：218 公开 API 面、2 loader 拼写（cobra `Gt`/`Eq` 经
`text/template` FuncMap `gt`/`eq`）、5 仅测试、31 限制/私有声明仅自文件用或无人用、2 条域读法入残余风险台账（requests
`_types.py:157` `if TYPE_CHECKING:` 下划线模块类判 public——Python 模块私有非 mount 事实，具名交提取面补强步——2026-08-28 用户裁**入 #8 一起做**；ripgrep
`matcher.rs:548` 同文件 `pub(crate)` 属文件粒度设计）。**K45 双腿**（A/B 两棵 HEAD 树各自 `.ce/`，旧客户端 1f493df vs 本批，
n=9 交错，静默窗）：`ce audit --hook` 1.186→0.954 s、`ce erase` 1.526→1.493 s、`ce check` 1.786→1.802 s（散布 1.729–1.928）——
传否路零代价成立，PERF-BUDGET 立节。方法学册 **13**（`docs/reference/methodology/13-unmentioned-declaration-advisory.md`，
全引文 file:line，含 §7 残余风险与 §8 验收表）+ 06 §4 净空散文按 `Cost.hs` 现注重写 + README 双语第 5 条补顾问句
+ 官网 how 页双语第 13 节（`.fam` 13：公式块 + 注 + chip `MENTION_REV`/`FOLD_MIN_CHARS`/`unmentionedVisMask`/
`unmentionedCap`/`unmentionedHardCap` 各绑一处源常量，过 `docs_consts` 门）+ 册数十二→十三六处（how 标题/meta/h2/调和注、
stack 卡、README 双语技术栈条，`site_counts` 门同改）；`contracts/docs-citations.json` +86 条册 13 钉锚，06 册 13 条引文随
`deadcode.rs` 三处 hunk（+1/+3/±0）位移按序号重瞄后重签。**对抗审查 Workflow**（6 镜头 37 条 → 25 确认 / 11 推翻 / 1 核验者
被 API 过滤、亲核成立）全修：U 公式四缺陷（`--exclude-standard` 连 `info/exclude`/`core.excludesFile` 一起读而 walk 不读 →
`--exclude-per-directory=.gitignore` + `--ignored`；漏嵌套仓项且把 git 的 `sub/` 目录项当文件读；tracked 已删未暂存 `expect` panic；
打印行缺 cut 项 → `Formula` 八项各成列 + scratch 仓逐形见证腿）；① 仅问身份通道 → 三通道两臂同问（zod 2 行假警归零）；
`pkg_test_dir` 重述包根规则 → 单调前缀探针问产品自己的 `PathWords::bits`；`ce graph --mentions` 零测试 → `tests/it/mentions_face.rs`
两腿（字段名 / 折叠通道 fixture / 中英九孔实数）+ `zh_surface` 形状（scratch `--db`）；GUI `Promise.all` 令顾问路承重（前 6.2.0 核
使整图空白）→ `allSettled` 画布为准 + 顾问第三态通知 `advisoryUnavailable`（中英）；`walk::cut` 单谓词发布（walk 过滤器 / census /
公式三处同读）；`rates.rs` 文档 0.64 %/14.9 % 归 X-1 全规则仿真、并列本层 65.8 %/16.0 %；册 13 五处语义（vis bit 1 具名、
code 1 `total > 0` 空真线、碰撞得救率分母改 unmentioned〔§0 条款 3 第二数〕、自仓行按本提交重取、① 叙述反转）+ 五处引文重瞄
（`rates.rs` 两处曾钉在 `}` 与 fixture 串上）；`docs_citations` 门新增标签尾行 past-EOF 检查——抓出 04/05/08/11 册 6 条旧标签
（含 11 册「决策 JSON 发射点」误指 `observe_log`，改瞄 `emit_decision`）一并重瞄；CHANGELOG 头补第三键；gui.md「同一判决」改
「第二次判决」。**ADR-006 具名重立**（`CE_ACCEPT_BASELINE=1 ce baseline .`，softLine 332 不动）：超 max(+2 %, +10) 者 15 文件——
`graph.js` +79、`deadcode/report.rs` +67、`candidates.rs` +61、`deadcode_e2e.rs` +50、CHANGELOG +46、`face.rs` +38、
`candidates_tests.rs` +35、`site/how` +33、`i18n.js` +27、`site/zh/how` +27、`walk.rs` +17、`advisory.rs` +15、`store.rs` +15、
PERF-BUDGET +13、`zh_surface.rs` +12；新入表 7（`rates.rs`、`eval_mention.rs`、`eval_support/mention.rs`、`mention_universe.rs`、
`mentions_face.rs`、册 13、`hub_projection.js`）；`ce dedup` 185 恒（行集对片 (4) 零差）、`ce scan` 0 fail、`ce check` 952。

**步 #8 提取面补强，无默认档位变更**（2026-08-28，ccm 步 #8；GRAPH_REV **13→14**，缓存键失配整库重建，发版说明义务归 #12）。
八片全做不记限档。(O55/O29) `kinds::extra` 收 Go `type_spec`/`type_alias` 与 Haskell `data_type`/`newtype`/`type_synomym`/`class`/
`type_family`/`data_family`，visibility/hs.rs 出口表项 `T(..)`/`C(m)`/`type Fam`/`pattern P` 取头名——自仓 haskell declared 1242→1284
（exported 274→296）、unmentioned 295→299（exported 4 恒）、vetoed other 947→985；cobra go 591 (468)→613 (481) / 394 (310)→403 (313)。
(O56 + 拍板 ⑤) 新 visibility/py.rs：字面 `__all__`（`=`/`+=` 的 list/tuple 并集，任一非字面 ⇒ 约定）**收窄** bit 0 至所列名——Haskell
出口表先例，与下划线约定同为收窄；读图工作流读者曾主张只放宽以保 erase reason 6，按 §4 原则句「导出机制命名本声明」取收窄并具名差分：
requests exported 645→644 = `__init__.py:60 check_compatibility`；mounts.rs Python 臂 `py_private` 读下划线路径段（dunder 除外）；conv/py.rs
`if TYPE_CHECKING:` 体（裸名或 `typing.` 限定、经 `consequence` 进入）⇒ `Ambient`——requests `_types.py:157` 判私有且豁免出顾问，requests
顾问人口 16→14、unmentioned exported 432→431。(O26/O27) spec.rs `Specifier::Literal`：Python `future_import_statement` ⇒ `import_from
"__future__"`（ladder/py.rs 具名 External 4：PEP 236 真模块而 public 名表不含），TS `import_require_clause` ⇒ `import`（`import X = A.B.C`
书面豁免：命名空间别名，文件已由 `import * as A` 承载）；站点标签零新增，自仓站点 3519 恒。(O28) `{-# SOURCE #-}` 导入指向同名模块 `.hs`
（boot 是同模块接口，非判决语言）：hs.rs 边界句 + sites_tests 例。(O05/O15) R6 侧门反事实两行 `searcher_lib::{searcher::Binary…}` ⇒ ok(门面,4)，
计划书 :4 句改写。(O58) cabal_parse.rs `Region{Live,Common,Dead}` + `Walk.commons`，`import:` 经 `merge` 单喉合并 roots/exposed/other
（common 可再 import、未知名不拉、common 内 main-is 不落、只靠 import 得根者不落 `.` 默认）；core/ce-core.cabal 无 common ⇒ 自仓差分空具名。
(O57) 新 graph/md_mask.rs `Blocks`（围栏 + 缩进码块：四列且无开段落〔空行或围栏后〕且列表上下文外、注释内不开，保守侧具名）+ ladder/md_slug.rs
（`render_text` 渲染级 slug：码段留内容、链接/图片取文本、内联 HTML 掉、`*` 掉、`_` 仅作定界时掉〔CommonMark 词内规则〕、反斜杠转义、尾空白
修剪；`html_anchors` `<a name/id>`/`<h1..6 id/name>` 原样入集；`percent_decode` 路径与片段在 `#` 分裂后各自解码）；自仓 md 锚点差分空具名
（1326 站点皆 `#Lnn` 行锚 ⇒ 文件级）。(R5 拓宽，拍板 ④，spec 勘误 ⑬) rs_use.rs `local_module`/`namespace_mod`（裸头先读站点命名空间里
**声明**的 `mod H`：带体 ⇒ Resolved{本文件,3}，声明 ⇒ 经 mod 梯级挂载〔`#[path]` 仍胜〕再 descend rung 3；盘上有文件无声明不算模块）+
`crate_walk`（lib+bin 两根终端由顶层定义/导入〔含私有 use〕首个未消费段的根裁定，`rs_reexport::owns`；皆有/皆无仍 `AmbiguousRoot`）+
rs_tree `walk_hits`/`settle`/`mod_named`；**cargo.rs `auto_targets` 补 Cargo 自动发现第二形 `tests|examples|benches|src/bin/<name>/main.rs`**——
差分抓出：无此项时 `tests/it/*.rs` 的覆盖根落到两兄弟根二进制，裁定把恰好也 `#[path]` 挂 `common` 的 `daemon_conn_deadline.rs` 记成 78 条假边，
建模缺口不得变成错文件，先修根再裁。自仓同树差分（HEAD a033ce2 工作树，新旧二进制）：边 2430→2675（rung 1 419→507、rung 2 239→350、
rung 3 219→260、`via_reexport=1` 0→5：`crate::config::ClassCfg`/`RulesCfg`→config/rules.rs、`crate::graph::deadcode::UnmentionedFace`→
deadcode/advisory.rs、`crate::mention::UNMENTIONED_SOFT_CAP`→candidates.rs、`super::LangRates`→rates.rs）；deadcode `unresolved_sites`
1089→844、`kept_edges` 1311→1488、dead 0→0、顾问行 37→37；ripgrep 顾问 41 恒（7 行转 `reexported_unmentioned`）、zod 197 恒，四语料
① 预登记零全立，外部顾问总 258→257（cobra 4→5 = `doc/man_docs.go:84 GenManTreeOptions` 类型形入域）。测试 lib 210、it 全绿；
`ce dedup` 185 恒（三处新克隆结构消除：cabal_tests 逐 stanza 串、md_tests spec 列、spec.rs `FUTURE_IMPORT`/`IMPORT_REQUIRE` 具名常量）。
冻结册五行改签（生成器已退役，按门报出的 `right:` 值逐行改签、摘要由行重算，docs/EVAL-SET.md 同句）：graph-slice requests `models.py`/
`structures.py`/`utils.py` `import_from` 各 +1（`from __future__`，py/import_from 47→50、站点 667→670），t3-universe `go/args.go` 11→12 与
`completions.go` 22→28（`type` 形入单元，go 52→59、单元 2028→2035），t3-candidates `admitted_by_lang.go` 43→44（对级字段仍是快照）。`ce scan` 唯一 FAIL `graph_ladder.rs rust_cases` 89 行按梯级拆三
（mount / walk / member）归 0 fail。ADR-006 棘轮具名重立（审查修法落定后 `CE_ACCEPT_BASELINE=1 ce baseline .`，`ce check` 951）：超 +10 者 22 文件——
graph_ladder.rs 507→662、rs_use.rs 192→320、cabal_parse.rs 178→262、rs_reexport.rs 220→294、tests_hs.rs 159→227、visibility/hs.rs 92→153、
visibility/tests.rs 165→220、ladder/md_tests.rs 31→77、cabal_tests.rs 144→187、spec.rs 115→146、conv/py.rs 56→87、mounts_tests.rs 146→176、
rs_tree.rs 262→291、kinds.rs 51→77、md_tests.rs 120→140、rs.rs 151→170、mounts.rs 288→302、sites_tests.rs 115→129、cargo.rs 125→137、
方法学册 06 377→388、册 13 341→354、CHANGELOG 253→304；新文件 visibility/py.rs 119 / md_mask.rs 179 / md_slug.rs 276。
对抗审查 Workflow `wf_f9654713-030`（6 lens finders → 9 refuters）23 条：9 条经反驳者确认全修，14 条因 cap 未核者逐条处置（13 修 + 1 由生产者
证实为真后修，零推翻）。修法：(1) rs.rs `conv_base` 单一权威——`mod inner { mod deep; }` 的约定查找与 `#[path]` 同读 child_dir + 内联 mod
名（此前约定查找只读文件级目录，内联声明挂浅一层成错文件；PATH_TREE 加 conv/shallow 两行 + use 行）；(2) rs_use.rs 全局 `::foo::Bar` 形
不再被同名本地模块劫持（`use_path` 带 global 位，rs_reexport `split` 保留空首段供 hop 走）、BUILTIN 名改在本地模块**之后**读（勘误 ⑬ ①
字面：`test`/`alloc` 不在 extern prelude，`mod test; use test::Helper` 是模块）、R5 hop 站点取 `pub use` 自身行号而非 1（文件开头的 bodied
mod 曾夺走 hop 命名空间；REEXPORT_TREE 门面加 shadow 块）；(3) rs_reexport `pubuse_hash` 折进 `owns` 所读的两项事实（私有 use 绑定名 +
顶层项名），battery 翻两例加一例，册 06 :90 句改写；(4) visibility/py.rs `__all__` 顶层扫描未消费的任一拼写（`.extend`/守卫 `+=`/
docstring 提及）⇒ 不可读走约定，f-string/转义项 ⇒ 不可读（5 例入表）；(5) md_mask 围栏关闭须同标记且 run ≥ 开栏长、`Blocks` 初态无开段落
（文首缩进码块）；md_slug `html_anchors` 先并码段掩码、`_` 只在**成对**时掉（`_private_helper()`/`foo_` 保留）、`#\t` 标题、标题内
`<!-- -->` 掉；(6) hs.rs `head` 首 `)` 截断（`(:^:)(..)` 读作类型算子）+ 关联族随类导出（`C(..)`/`C(Fam)`/`C(type DF)`）、kinds.rs
`REDECLARING` 排除 `data_instance` 包裹的 `data_type`/`newtype`（每个实例曾多铸一行家族名）；(7) Go 函数体内 `type` 只留 bit 0；cabal 续行块内
列 0 注释不再截断；conv/py.rs `elif TYPE_CHECKING:` 同 Ambient；册 13 §5 bit 1 补 Python 下划线路径臂、两处旧数字改 66/1249 = 5.3 % 与
313/481 = 65.1 %。修后复测：四语料顾问行 5/14/41/197、边、未解析、census 逐行零差；自仓顾问 38 恒（一行仅行号位移）、dead 0、haskell census
恒、rust declared 2972→2988 = 新增测试/函数；lib 211 + it 212（3 ignored）。

**片 (5) 无默认档位变更**。`mounts` 表生产者落地（`cli/src/graph/mounts.rs`：每节点 `[privateMounts, totalMounts, bits]`，
全节点无条件一行；private/total 由 `mod_decl` 边 join 声明 `mod` 单元的存储可见性 bit 0；bits bit 0 = 再导出目标 = Rust
`via_reexport=1` 边 ∪ TS `export_star` 站点目标；bit 1 = 包私有 = Go `package main` ∨ `internal/` 段〔`_test.go` 除外〕、Cargo
无 lib target ⇒ 整包否则 `[[bin]]` 根〔`Package::bin_roots`〕、cabal 无 `library` stanza ⇒ 整包否则模块仅在 `other-modules`
〔`Cabal::{has_library, hidden_modules}` 两字段〕），wire 未接（片 (6)）。**站点宇宙变动一处**：TS `export * from` /
`export * as ns from` 自本版起开 `export_star` 站点（`store::KINDS` 末位追加，位次 10），不再计作 `export_from`——GRAPH_REV
12→13：全体站点重检、缓存键失配走整库重建、trend 同清（发版说明义务）；冻结自仓切片 `contracts/eval/graph-slice-v1.json`
`packages__zod__src__v4__mini__index.ts` 行 `export_from: 1` 改签 `export_star: 1`、summary `ts/export_from` 键同改（RG3 常备
成本，活门 `eval_graph` 两腿重新绿）。**冻结册口径分界**：只有自仓切片被活门重检并改签；zod/requests/ripgrep/cobra 四册切片、
审计样本 `graph-sample-v1.json` 与精度册均早于 rev 13，其中 TS 星号再导出仍计在 `export_from` 下（zod 切片 `ts/export_from`
118 含此类），冻结对冻结不动、审计记录不改——跨语料的站点标签自本版起不再逐字可比，重算须复活生成器（EVAL-SET.md）。
**bit 0 的 Rust 臂继承 R5 召回界，具名**：`rs_use::bound` 只在「单终点走完仍有剩余段」且门面条目以 `crate::`/`self::`/`super::`
开头时绑定（K30 形：包名梯级 `use fixture::Thing` 落 `lib.rs` 剩 `Thing`，再走门面 `pub use crate::source::Thing` 到定义文件；
非根门面 `crate::facade::Thing` 同样可绑——两覆盖根降到同一文件）；不绑定的三形：通配门面（`pub use x::*`，rs_reexport 不跟）、
uniform-path 门面（`pub use source::Thing`，裸头按 crate 名走 R4 即止）、lib+bin 同包内自非根模块出发且走到根为止的导入
（`use crate::Thing` / `use crate::{…}`：两覆盖根两终点 ⇒ `AmbiguousRoot`）。自仓索引实数（rev 13 树）：`via_reexport=1` 边
**0**；`cli/src` 217 个 `crate::` 站点里无边 2（皆 `crate::{…}` 组形）、裸本地模块头无边 26（`report::`/`model::`/`main_cli::`…）；
`cli/tests/it` 另有 108 个 `crate::` 无边站点，成因是嵌套测试根 `tests/it/main.rs` 不在 `crate_roots` 模型内（`cargo.rs` 已具名
的深层自动发现召回界），与 R5 无关。这是梯级的召回界不是本表的；用户裁 2026-08-27 作 §4 修正案、入提取面补强步（裸头先读
本地模块再按 crate 名走 R4、lib+bin 包根终端在包内绑定，随 R5 精度册重跑 + RG3 具名），本版按封版口径只读事实。对抗审查（16 agent）：
确认 3 全修（cabal 首根遮蔽改全根任一、Go 块注释内 `package` 两向误读改带状态读子句、Rust 臂逐文件重扫改逐清单一次），
分歧 1 修（同上第三项），未核 6 与 nit 13 逐条处置（含本段两处措辞与实数改正、命名子库 `library x` 不算公共库、
幻影节点零事实由走集键控见证），驳回 2（`pub(crate) mod` 按封版 §4 bit 0 仍为导出挂载）。自仓差分：`ce dedup .`
恒 185（K30 e2e 首稿的元组表被自家门抓出，改文本表）、`ce scan .` 0 fail、lib 193 / it 201 + 2 ignored、`ce check` 954。

**片 (4) 无默认档位变更**。`symbols`（SQLite）新列 `conv` 存约定类别字的 **AST 半**（GRAPH_REV 11→12：全体
符号行重导出，缓存键失配走整库重建，trend 同清——发版说明义务）：bit 1 TEST 的 Rust `cfg` 谓词臂、bit 2 FFI
（Rust 导出属性/`extern`、Haskell `foreign export`、Go `//export`）、bit 3 REGISTRATION（Python 注册装饰器表、
TS 任意装饰器）、bit 5 MEMBER（Python 类体）、bit 8 DEFAULT_EXPORT、bit 9 AMBIENT、bit 10 ALLOW 的
`allow/expect(dead_code)` 臂、bit 11 CFG；名表半在 wire 组装时算、零存储（片 (6)）。`mention_name`
（`cli/src/mention/name.rs`）钉判决域 = 声明文件自身分词臂下的单 token 名。**`ce:allow(<tag>)` 解析合一**为
`crate::allow::allow_claim`，文法即计划 §4.1 成文形 `ce:allow(<rule>) -- <why>`（标记—空白—`--`—空白—同行非空
why）。与两处旧解析器的差分具名四条：docdup 原容许标记与 `--` 之间夹任意文字、原容许 `--why` 不带空白（两形
收窄）；deadcode 原接受空 why（收窄）、原只认 `--` 后一个空格（`--<TAB>why` 自本版起算作声明——唯一放宽项，
方向 = 多一文件免死判）。自仓差分：源码零标记（`git grep` 实测，仅测试与解析器自身）；判决文档四处标记里
`contracts/VERSIONING.md:353`、`docs/reference/methodology/06…md:214`（`ce:allow(deadcode) -- why` 散文，两文件
因此带 ROLE_ALLOW，新旧同判）与 `plugin/skills/erase/SKILL.md:57`（`ce:allow(docdup) -- <why>`，新旧同判）
不动，`docs/reference/methodology/03…md:25` 标记后先接反引号再 `--`——旧文法算声明、新文法不算，该段自
`inline_allow` 转活：`ce docdup .` exempt_allow 2→1、allow_missing_why 1→2、live segments 673→674，dups 恒 0。

**片 (3) 无默认档位变更**。提及语料宇宙 pass 落地（`ce graph --mentions`，索引两张加性表 `mention_files`/`mentions`
只存 fnv1a64 哈希，`MENTION_REV` 自有版本行，不在 guard/audit 热路径）。与之同表的 secrets 排除 glob 自四条
（`.env`/`*.pem`/`id_*`/`*.key`）加宽为八条（`.env*`/`*.pem`/`*.key`/`id_*`/`.npmrc`/`.pypirc`/`.netrc`/`*credentials*`，
口径册 S-A9）：**判决 walk 同一张表**，故名含 `credentials` 的源文件自本版起离开 scan/dedup/graph 判决宇宙——
隐私失效方向为安全（少判一文件，永不把密钥写进索引）；自仓差分具名为空（无此类文件）。

**片 (2) 无默认档位变更**。`symbols.flags` bit 0 是 erase reason 6 `public_surface` 与 join `publicGuard`
的输入，其三处生产者缺陷按口径册 §4 修复；许可增量与一次口径收窄不可区分，故按 §4.2 铁律与晋级
同门记依据（K27 差分 = 片 (1) 后二进制 vs 本修，fixture 语料逐文件；自仓差分表**具名为空**）：
T3（TS，扩宽）恒等守卫下 zod 912f0f5 差分 83 行 0→1、0 行 1→0（16 文件，皆 `export const f = () =>`
第三跳形）；H2/H4（Haskell，收窄）hsprobe2/3 差分 3 行 1→0（where 同名 ×1、class 缺省体/instance 体
×2）；H5（Haskell，扩宽）hsprobe4/6/8/9 + `module M` 自再导出差分 6 行 0→1。存储字另加 bit 1/2
（作用域导出/受限），wire 只发 bit 0（`symwire.rs` 掩码，十族 golden 逐字节不变）。

## [v1.2.0] — 2026-08-26（无默认档位变更；T1/T2 写入类规则语义收窄）

**无默认档位变更；T1/T2 写入类的规则语义收窄为「只拦新引入的重复」**
（K 步 11 根修，2026-08-26；仲裁 = 用户授权下的模型仲裁，授权 2026-08-24）。
复活 M3 重放仪器全史复测：requests 尾段 487 事件 0 误拦；自仓 2274 事件在
出厂全文语义下 505 拦截（122 对首燃 = 90 真引入 + 32 拆叶/并叶中间态），
32 条中间态若计误拦 = 1.41% > §4.2 M4 门（≤1%）。处置（用户拍板 2026-08-26）
= 根修而非降档：`guard::novel_matches` 以被替换内容为基线减法（Write=盘上
旧全文、Edit=old_string），复测 505→139（84 真引入 + 32 中间态 + 23 延展
复燃），拦截词教搬移安全序（先削源后写即过）；两类已晋级规则**维持 deny**。
observe feed 0.7.0（`matches` = 新引入数，语义断点具名，前后不可直比）。
Stop 审计 / precommit / zone 三类**维持 observe**：现行代 observe 台账
stop_audit/precommit 样本数为 0，§4.2 类准入门（每会话 M4 台账、≥200 样本、
纯净度）今日不可满足，如实缺席。全记录：docs/FPR-REPLAY.md「K 轮复跑」节。

## [v1.1.0] — 2026-08-24（无默认档位变更）

无档位变更：两类晋级规则仍 deny、其余仍 observe、渐进区仍须显式
`[guard] zone_tiers = true`；本周期无新增独立 FPR 记录，按 §4.2 维持原档。
硬预算规则的**取线来源**改为文件所属 `[[rules.class]]` 的 `file_lines_fail`
（无类 = 全局 750，`guard::budget::lines_for`，44a0abd）——同一规则、同一档，
只是 deny 的那条线与 CI 硬墙同源；未声明类的仓库逐字节不变。功能面与
分数可比性声明见 GitHub Release v1.1.0。

## [v1.0.1] — 2026-08-22（无默认档位变更）

无档位变更、无判决语义变更，分数与 v1.0.0 完全可比。本版是分发面
维护：Windows 安装器装机时探测 Claude Code 并自动接入 codeeraser
插件（一次安装即整个产品；卸载只对称拆除它自己添加的注册）。

## [v1.0.0] — 2026-08-22（无默认档位变更；v0.7.1–v0.7.3 亦均无）

自 v0.7.0 起无默认档位变更：`PROMOTED_DEFAULT` 两类（T1/T2 重复写入、
硬预算超限）仍 deny，其余规则仍默认 observe，渐进区仍须显式
`[guard] zone_tiers = true`。本周期无新增独立 FPR 记录，按 §4.2
「无记录即无晋升」维持原档。边界修正一处（非档位）：hook 车道的
usage error 现打印后退 **1**——clap 原退 2 正是 PreToolUse deny 码，
usage 失败必须 fail-open，不得拦停会话。功能发布史与分数迁移声明
见 GitHub Release v1.0.0。

## [v0.7.0] — 2026-08-20（无默认档位变更；v0.4.0–v0.6.0 亦均无）

新增 **opt-in** 渐进区位置→档位映射（计划 v2.7 ①）：`ce.toml` 显式
声明 `[guard] zone_tiers = true` 后，软线→硬线区间按位置映射
<25% observe / 25–75% warn / >75% ask（>H 仍走硬预算规则）。
**默认恒 observe**——默认翻档仍以观察台账攒出的各档 FPR 记录为准
（§4.2 纪律同款，无记录即无晋升）；为此观察 schema 0.5.0 → 0.6.0，
`zone` 事件行在武装仓库加记 `zone_tier` 映射档，为未来晋升留账。

## [v0.1.0] — 2026-08-18 首发（下列两条均随其发布；v0.2.0、v0.3.0 均无档位变更）

### 2026-08-17 — 默认档位晋升：§4.2 路线第 3 级（1.0，M7-P2）

**变更**：`[guard]` 未显式设 `mode` 时，两类已晋级 PreToolUse 规则
默认从 ask 升为 **deny**（单一权威常量 `config::PROMOTED_DEFAULT`，
guard 执行面与 health/doctor 报告面同源）：

1. **T1/T2 精确/改名重复写入**；
2. **硬预算超限**（写后文件 > 750 行）。

Stop 审计 / precommit **不晋级**，默认维持 observe：两者无独立
FPR 记录在案（§4.2 第 3 级原文"其余规则默认 ask/warn 按各自 FPR
记录决定"——无记录即无晋级资格，如实缺席）。显式 `mode` 依旧
统一覆盖全部规则类。

**依据（FPR 数据，与第 2 级同一账本，无新增拦截记录在案）**：

- T1/T2 探针：630 次真实编辑重放 0 误拦（docs/FPR-REPLAY.md
  2026-08-07：requests 487 + 自仓 143）；
- 判定层主门：600 全语料真实正常编辑重放 0 标记 = 0% ≤ 1%
  （contracts/eval/fpr-fourclass-v1.json）;
- 硬预算：自仓全史 first-parent 380 个 A/M 事件按排除模型 0 触发
  （2026-08-11 条目的复现命令不变）。

### 2026-08-11 — 默认档位晋升：§4.2 路线第 2 级

**变更**：`ce.toml` 的 `[guard]` 未显式设 `mode` 时，两类 PreToolUse
规则默认从 observe 升为 **ask**：

1. **T1/T2 精确/改名重复写入**（M3 探针，daemon 索引比对）；
2. **硬预算超限**——写后文件 > 750 行（本级新增规则：按 Edit/Write
   自身语义在内存精确套用后计行，走 scan 同源排除模型
   `scan::walk::in_scope`，免 daemon）。

Stop 审计 / precommit 不在晋升类，默认仍 observe；显式 `mode` 统一
覆盖全部规则类。观察档 schema 随之 0.3.0 → 0.4.0（新增 `budget`
事件行，为 §4.2 第 3 级的按规则 FPR 记录留账）。

**触发条件**：计划 §4.2 第 2 级 “M4 的 FPR 门（§6）通过后”——已于
2026-08-11 通过。

**依据（FPR 数据）**：

- T1/T2 探针：630 次真实编辑事件重放（requests 完整历史尾段 487 +
  CodeEraser 自仓全史 143），仲裁后误拦 **0**，折算 0.00/500 ≤ 1
  （docs/FPR-REPLAY.md，2026-08-07）。
- M4 判定层主门：600 全语料真实正常编辑重放误报 **0/600** = 0% ≤ 1%
  （contracts/eval/fpr-fourclass-v1.json：samples 600、flagged 0、
  gate_max_percent 1）。
- 硬预算规则：判定为精确行数算术，无统计误报模态；对自仓全史
  first-parent 重放（五门首发语言扩展名，380 个 A/M 事件）超限事件
  5 个，全部落在 `contracts/fixtures/crosscheck/**`（钉定第三方对照
  物，ce.toml 排除项覆盖）——按已装排除模型计发火 **0/380**。复现：
  `git rev-list --reverse --first-parent HEAD` 逐提交
  `git diff-tree -r --root --no-renames --diff-filter=AM` 取
  `rs|py|ts|tsx|go|md` 文件，`git cat-file -p <sha>:<file> | wc -l`
  与 750 比较。
