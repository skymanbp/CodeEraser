# Changelog

> 记录义务来自 DEVELOPMENT_PLAN.md §4.2：“每次默认档位变更在
> CHANGELOG 记录依据（FPR 数据）。”本册记两类：守卫**默认档位变更**
> （依据 = FPR 数据，每个版本条目首句声明有无）与每步落码的**功能 /
> 协议 / 记账变更**（v1.2.0 后的 [Unreleased] 块起，发版时并入版本条目，
> 源码克隆与 crates.io 包内即有全史）；GitHub Releases 留发布说明与分数
> 可比性声明（v1.2.0 及更早的功能面只在那里）。

## [v1.3.0] — 2026-08-29 — L 轮片 (2)+(3)+(4)+(5)+(6)+(7)+(8)（无默认档位变更；bit 0 生产者三修，erase reason 6 人口随之变动；secrets 排除表加宽，判决宇宙随之变动；`ce:allow` 解析合一，旧宽松形三收窄一放宽，自仓一段转活；TS 星号再导出自 `export_from` 分出 `export_star` 站点标签，冻结自仓切片一行改签；graph/1 **6.2.0** 加性两表 + `export_unmentioned` 顾问类，`ce deadcode` 报告 0.3.0 加性两键，片 (7)+(8) 第三键 `unmentioned_cut` + `ce graph --mentions` 报告 0.2.0 加性 `rates` 键；步 #8 提取面补强 GRAPH_REV 13→14，站点标签零新增、边与符号域拓宽；步 #9 自仓可见性候选 38 行逐条处置，零 wire；步 #10 减法批 7 条 + 3 缺陷落码，dedup 预算 185→182 具名下调，零 wire；步 #11 测试子仓 CodeEraser-tests submodule 挂 `cli/tests`，walk/trend/U 公式三处 submodule 语义，零 wire；步 #12 子仓只当读者不当被测者，wire **6.3.0** 节点角色 bit 7 foreign、索引 schema v15 `files.owner`、`[graph] crate_roots` 旋钮、嵌套项目自带 ce.toml 的守卫/审计委托，dedup 182→65 / check 943 地板 940；步 #13 单元测试亦迁子仓 `unit/`（`#[path]` 挂载），dedup 64 / check 949 地板 946，自仓分数与 1.2.0 不可比；步 #14 乙 围栏收尾片 (a) 七条零 wire——`ce dedup --check` 拒绝放宽、类 glob 与 exclude 同一解析器（启类仓库 `dir/` 读法切换、分数不可比）、`ce baseline` 只在根 + 三具名动作（缺基线须 `CE_ACCEPT_BASELINE=1`、`CE_ACCEPT_FENCE=1` 窄动作）、trend 恒等钉基线、控制台逐名报失败条件，dedup 64→63 / check 950 地板 946；片 (b) = wire **6.4.0** 围栏批——verdict 加性 `present`→`ratchet.dropped` + 第六 fail 条件 `rows_dropped`、classKnobs 码 4 仅 CoC 容差、thresholds 码 7 `cycleFloor` 与 graph `sccFloor` 同读 `[graph] scc_floor`、scan `knobsFence`→`failed`、守卫漂移按出厂预算、每份回执核自洽（wire_check）、报告 check 0.5.0 / scan 0.2.0、O66 夹具账本自推导，dedup 60 / check 951 地板 946；片 (c) = O39 旋钮摘要规范形〔有效旋钮集、默认即静默、一份声明字面值冻结、两仓摘要各移一次具名重立〕+ O64 客户端期限拆除〔shutdown / CancelIoEx、2.5 s 宽限、脱离计入 `ce.doctor-report/0.3.0` 加性 `daemon.parkedWorkers`〕，零 wire，dedup 60 / 119 恒；步 #15 丙批 O62/O65/O68/O69/O60/O52/O11 + CI 两修——`wire skew` 具名拒绝、空说明符站点 GRAPH_REV 14→15、`ce.sh` 会话绑定戳（全链 2.0–2.3 s → p95 0.50 s，e2e 15 态）、§2 变短 + 新入场行 + R5 击发裁「观察、路线不动」，零 wire，dedup 60 / 119 恒；步 #16 复核发版前置——46 条已交付 O## 经 codex 四路只读审计逐条取证 35 落地 / 8 具名澄清 / 2 修补〔O42 类名出指纹、O43 三处注释〕/ 1 退役，41 条后置束计数与台账一致、两条新入 M 轮产品小项，bench 回填 seat helper，CI Windows relink 尸案根修〔子仓收尸跟随委托根〕，零 wire，dedup 60 / 119 恒；步 #16–#19（v2.20，用户四令 2026-08-29）更新功能三面等价 + 三面等价门 + README 双语九节重构与仓内 demo + 全量文档同步，零 wire，随后发 v1.3.0）

**步 #16–#19 无默认档位变更**。`ce update`（新子命令，零 wire）：检查 = 读 GitHub `releases/latest` 的 tag 与**该 tag 上已提交**的
`plugin/bin/manifest.env`（pin 提交先于 tag 的发版序即信任锚——前沿项 O83「自更新」以此逆转延期，不等签名），退出码 = 判定
0 最新 / 1 有更新 / 2 未知（无网络或清单不可读，永不读作「最新」）；`--yes` 下载 ce 与 ce-core，**两枚 pin 都通过后**才两次
rename 就地落位（旧副本改名 `.old`，下次 apply 清扫），`--installer` 另存已校验的 GUI 安装包并打印路径；安装归属按二进制自身
位置判：手工放置 / 安装包随附可替换，插件绑定副本按名拒绝并指向 `/plugin update codeeraser`，cargo 副本指向 `cargo install
codeeraser`。报告 `ce.update-report/0.1.0`（current / platform / latest / pins / verdict / action，码不载句）三面一文档：GUI 第十屏
update（同一 apply 库入口）、插件 SessionStart 第二行通知（缓存一天，`CE_UPDATE_CHECK=0` 关，fail-open）+ `/codeeraser:update`
命令、MCP 第十四工具 `update_check` 只到检查。传输 = curl（https、redirect 同守、限时限量）+ `file://` 测试缝（`CE_UPDATE_BASE`、
`CE_UPDATE_TARGET_DIR`）；e2e 七腿 + 单元八腿；测试线束默认 `CE_UPDATE_CHECK=0`。三面等价门 `face_parity`：README 双语「能力 ×
面」表由 clap 枚举 / Tauri 命令与屏 / MCP 目录 / hooks.json / plugin commands+skills 派生，每个派生面恰被一行认领、具名省略写在表内；
demo 回放门 `demo_replay`：`demo/` 同一任务两跑只差 hook，七步真实判决，产出与三处嵌表逐字节门控；demo 语料与两份 README 在 `.gitattributes` 钉 LF——Windows runner 的 autocrlf 检出曾把每个标记移位（CI 33271753662，ndjson 规则第三次应用）。§5.9 网络承诺条改为两种可关网络行为。**daemon 离开根目录**（零 wire）：进程 cwd 在 Windows 上钉住目录，demo 回放在并行
测试套下撞到 eject 的 `bye` 先于退出完成、驱动 rm 即 EBUSY（Node rimraf 不重试顶层 EBUSY）；daemon 启动即 chdir 到系统临时目录并在
serving 行写明，相对含路径的 `CE_CORE_BIN` 先绝对化；`daemon_cwd` 腿以 remove_dir 的 145/32 之别为证。
**记账**：check 主仓 951→953（地板 946）/ 子仓 988→989（地板 983）；dedup 主 60 / 子 119 恒（HEAD 克隆对工作树行集零差）；版本
1.2.0→1.3.0 五处 + 两 Cargo.lock + hello-ok golden 回显。棘轮具名重立（ADR-006 文档规则，自 2aca36f / 子仓 ba5ee10 起）：主仓
CHANGELOG.md 415→442、cli/src/daemon/server.rs 124→158、cli/src/faces.rs 162→173、cli/src/main_cli.rs 220→236、docs/reference/cli.md
427→443、gui/src-tauri/src/commands.rs 244→272、gui/ui/i18n.js 280→319、gui/ui/index.html 177→191，新入表 cli/src/update/{apply,fetch,
install,manifest,mod,notice,version}.rs、cli/src/main_update.rs、gui/ui/update.js、plugin/commands/update.md、demo/（run.js / steps.js /
render.js / README.md / seed 九文件 / out 两 summary）；子仓 it/common/daemon.rs 191→205、unit/mcp/tools.rs 22→38，新入 it/{daemon_cwd,
demo_replay,face_parity,update_e2e}.rs、unit/update/{install,manifest,version}.rs。**全量文档审计**（用户问「全量文档更新了吗」；14 面
读者 × 每条三票反驳，Workflow wf_05aca52d-fac，61 确认 / 22 驳回 / 6 欠票亲核 / 3 额外漂移）全部就地修：README 双语 FPR 双口径句、
扩展名全集、安装包 pin 句、六条 `ce` 命令；`ce baseline --help` 两条围栏条件 + zh 行 deadcode/dedup check 的降级分支（cli.md 机器再生）；
gui.md 命令形例外句；VERSIONING 四面一文档 + `sccFloor` + `knobsFence`/`failed`；RELEASE §0 两套门 + tag 去 v 版本号；计划书六处
（退出码例外、O42 提交号、产品小项 9、发版 #20、载体表 23 子命令、布局 demo/）；site 双语 how/index 四处 + 两 stack.svg「ten screens」+
bench 中文表头（生成器 zh 支）；册 01/05/06/11/13 引文 54 条重瞄 + 册 13 §8 自仓行重取；demo 三注释与 README 「Gated」句；erase.md
子仓句、size-advisory 围栏回落句；plugin README 引文、SKILL `ce:allow` 三读者、update 命令三处；`Cost.hs` 注释「唯一」→「两个」。

**v2.21 S1+S2 无默认档位变更**（计划书 v2.21 修正案就地写入——ADR-009 文档事实派生 + archify 架构图，329 行不变；v1.3.0 顺延其后）。
S1 bless 单读者（子仓）：`facts::blessing()` 是 `CE_BLESS` 的唯一读者（恰 "1"；`CE_BLESS=1 ∧ CI` 按名拒绝——bless 写、bless 不裁），
六处读点（common 金样比对、bench_render、bench_render_dashboard、docs_citations_parts、face_parity、eval_support 再导出）改经它；
`bless_guard` 三腿：源码普查（`CE_BLESS` 一读者、`CE_REFREEZE` 一读者 universe.rs）、workflows 永不拼写二者、子进程探针证 unset/0/1/CI
四态；docs_consts_stack 私有 `repo_root` 与其陈旧「编译两次」理由删除（单 crate 后 `crate::common::repo_root` 即可）。S2 产品暴露（主仓，零
wire、零 pub 放宽）：`churn/report.rs` 与 `report.rs` 的两处 inline schema id 提为具名 `const`（`ce.churn-report/0.2.0`、
`ce.deadcode-report/0.3.0`），报告字节不变；册 06/13 两条 `report.rs` 引文随行移重瞄再签。**记账**：check 主 953 / 子 989 恒；dedup 60 / 119 恒；子仓基线具名重立——新入表
it/bless_guard.rs、it/facts/mod.rs（it/main.rs、daemon_conn_deadline.rs 在容差内）；主仓基线具名重立——CHANGELOG.md 本段在容差内，无超容差文件。

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

**步 #16 复核发版前置：87/87 具名归属复核 + 发布前置门全绿 + 发布准备；零 wire、无默认档位变更；v1.3.0 的 tag / GitHub Release / crates.io / npm / 官网 deploy 按用户令 2026-08-29 一律未动，待 README 重构指示**（2026-08-29，ccm 步 #16）。
**复核**：46 条已交付 O##（甲 18 / 乙 19 / 丙 9）派 codex 中转站四路只读审计逐条取证（每条须给 CHANGELOG 行 + 提交 + 代码/测试/文档 file:line），46/46 到齐：35 条落地零缺口；8 条「带缺口」经第一方核为封版口径或既有裁定——O01 「符号行第 3 列」形已于 2026-08-27 计划对齐改为两表（计划书 :4）、O16 为甲批动机项由 mentions 表正面回应（草稿 :48）、O26 `import_alias` 按构造不开站点（spec.rs:76-79，:210 已记）、O28 SOURCE 导入指向同模块 `.hs`（boot 非判决语言，:210-211）、O59 草稿明裁「出旋钮、默认 2」、O29 构造子/记录选择子是其 `data` 声明的成员（无独立符号行，`T(..)`/`T(MkT)` 取头名，裸选择子导入退文件边——**具名后置** M 轮产品小项：选择子名并入类型单元的导入绑定与提及匹配）、O56 `__all__` 改名再导出为可见性字的边界句（visibility/mod.rs:38-43「floor, never an invention」——**具名后置** M 轮产品小项：Python `__all__` 再导出跳入 mounts `via_reexport` 臂）、O52 余句改实（PERF-BUDGET.md:42-44 行 1 注尾句：fork 前宿主开销已由 2026-08-29 全链注两端宿主时钟实录覆盖）；2 条真缺口本步修补——**O42** 类名退出规范树（`config/canonical.rs` 第五律：名是标签——加载期唯一性键与错误文本标签，不达任何阈值 / wire 字段 / 渲染行——改名即静默，声明序仍经第三律计数；单测 `the_rulepack_still_moves_it_in_every_part` 反转为 `assert_eq`、`a_glob_cannot_impersonate_the_encoding` 改以 glob 承载编码标点、`the_tree_holds_declarations_only` 期望树去 `name`；冻结字面值腿不含类故不动；启类仓库摘要因此再移一次——自 1.2.0 升级本就须一次 `CE_ACCEPT_FENCE=1` 具名重钉，同一次动作）与 **O43** 余项（`score/wire.rs:81-87`、`score/baseline.rs:230-234`、`Verdict/Ratchet.hs:23-28` 三处 5.1.0「无类」口径改为 6.0.0/O39 口径；Faces.hs 数字段已在片 (b) 按「记录才是具名动作」改到位）；同句文档：README 双语「整份 ce.toml」→「每个有效旋钮」、计划书 ADR-006「类围栏」条改「旋钮围栏」（`knobsDigest`、缺省 = 判决等于出厂默认，K11）、VERSIONING O39 注、册 05 决定论条、生成参考 ce-toml.md 行源；O11 = 竞品复扫例行已按用户令退役（:100、计划书 :314）。41 条后置束：计划书 §6 K–L 行「M 12+7 / N 20 / 证据门 4」= 草稿 丁-1 11 + 丁-4 7 / 丁-2 18 / 丁-3 3 + 双票翻入 C 类 6（甲 2、丁-1 1、丁-2 2、丁-3 1），O06 外部等待、O44 由 plan-set 自身清偿；本步两条具名新入 M 轮产品小项 7→9。
**发布准备**：bench 回填 seat helper（四裁 1de6696 所欠）——`bench_support::seat_submodules` 在 `git worktree add` 后 `submodule update --init --recursive` 并以产品自身 `gitmodules::seated` 为证（无 `.gitmodules` 的树零 git 调用），新腿 `bench_seat.rs`，BENCH.md 页眉重签；前置门（RELEASE.md §0）：两仓六门 + doctor rc=0、`cargo test --release` 263 passed / 3 ignored（lib 230）、clippy `--all-targets -D warnings` 0、fmt clean、GUI 四腿 ok、bootstrap e2e 15 态（CI 33257054625）、版本五处 1.2.0 一致（未 bump）、VERSIONING 6.2.0/6.3.0/6.4.0 在册；引文门 22 条随 Ratchet.hs +1 / canonical.rs +17 位移重瞄（`canonical()` 范围尾手校 68-96）后重签；dedup 主 60（行集与 HEAD 唯一差 = `score/wire.rs:120-128→122-130` 同一对随注释位移）/ 子 119 逐行同；棘轮具名重立：主 `config/canonical.rs` 106→123，子 `unit/config_knob_fingerprint.rs` 252→259、`it/bench_support/mod.rs` 213→237、`it/bench.rs` 296→300、`it/bench_render.rs` 154→155、`it/main.rs` 109→110、新 `it/bench_seat.rs` 64；check 951（地板 946）/ 子仓 988（地板 983）。**CI Windows relink 尸案根修**：af45fcf 两跑 Dogfood 步 `cargo run` 重链 `target/debug/ce.exe` 遭 `Access is denied`（全测试绿、本地不可复现）——c1f42d1 在 relink 前加**只打印不杀**的进程普查步，33262869599 点名持有者 = `ce.exe daemon …/foreign-readers-guard/suite`：`the_guard_is_inert_on_a_foreign_write` 第三次 hook 按 `root::judging_root` 委托到子仓根起 daemon，而 `run_hook` 的收尸只问超仓根；红/绿 = 该 daemon（继承 `CE_DAEMON_IDLE_SECS=120`，看门狗 60 s 步长 → 120–180 s 死期）与 relink（套件结束后 67–189 s）的先后；修法 = 子仓 `common::shutdown_daemon` 对 `dir` 及其下每个带 ce.toml 的嵌套根「关停并确认」（`client::is_running` 连接被拒才算走，30 s 活性界同 `spawn_daemon_ready`，超时 panic；冷启中拒第一次 shutdown 者靠循环补问），该腿末尾断言 `!is_running(sup/suite)`（未修先红 foreign_readers.rs:250、修后绿），全套 `it` 263 passed / 3 ignored 后普查零残留；子仓棘轮具名重立 `it/common/daemon.rs` 140→191、`it/foreign_readers.rs` 257→265、`it/common/hooks.rs` 148→149（子仓 e247d77，check 988 / dedup 119 恒）；主仓 ci.yml 418→425 容差内、CHANGELOG 415 行不动，六门全绿 check 951 / dedup 60 恒；ci.yml 两处 Rust 步注释纠正「e2e 腿泄漏」误判、普查步常设；具名残余：hook 放弃等待（2 s 预算）的 spawn 尚无 socket 可问，其探针必 ≥ 2 s degraded。教训入 memory：delegated-root-daemon-leak（管道版 `cargo test | tail` 停滞 180 s 本身即泄漏探测器——daemon 经 hook 继承了 cargo 的管道句柄，普查须在管道关闭前跑或把 cargo 输出落文件）；长跑 codex 须 WMI 隐窗拉起（Bash job 连坐与可见窗 CTRL_CLOSE 两次夭折）；插件绑定 1.2.0（索引 schema 12）与开发版（15）共用 `.ce/index.db` 互相整库重建，门里瞬时 `no such column` 是竞态非缺陷。

**步 #15 丙批：计划本体/契约文案、代码到期承诺、PreToolUse 实录、竞品复扫例行——O62 / O65 / O68 / O69 / O60 / O52 / O11 + CI 两修；GRAPH_REV 14→15、零 wire、无默认档位变更**（2026-08-29，ccm 步 #15）。
**O62 死兼容裁除**：`graph/deadcode.rs` 对 6.2.0 前核的两条静默回退（缺 `fail` 位读作 false、缺 `reported` 表读作空）改为具名拒绝 `wire skew: graph reply carries no \`fail\` bit / \`reported\` table (a pre-2.18.0 core cannot reach this client)`——握手已把此类核拒于门外，回退是永不可达的死路；VERSIONING 2.18.0 条款同句改写；子仓 `unit/graph/deadcode.rs` 两形回复一表驱动。**O65** DAEMON.md §5 复跑补 `corelink_deadline` / `daemon_conn_deadline` 两独立二进制；**O68** VERSIONING 冻结行记 `cabal.project.freeze` 入库提交 378fe40（2026-08-07）与再生规则；**O69** 本册页眉改记两类义务（默认档位变更 + 每步功能/协议/记账）并声明 GitHub Releases 留发布说明。
**O60 空说明符**：TS `import ""` / Go `import ""` 不再在检测端丢弃——站点保留、spec 为空，阶梯以 `Reason::Empty` 入未解析台账（`ladder/mod.rs` 调度前置守卫，Markdown 空锚除外）；`sites.rs` node_text 剥引号后再 trim；GRAPH_REV **14→15**（存储站点行变动，全站点重检）；册 06 词表补 `Empty`；子仓 `sites_tests.rs` TS/Go 两空例 + `graph_ladder.rs` 两 Empty 行；站点 how 页两语 GRAPH_REV 芯片 15。
**O52 PreToolUse 全链实录与根修**：真 headless 会话（`claude -p`，`--settings` 只启 codeeraser 插件，`--debug-file` + `-d hooks`，每条 assistant 消息恰一次 Write，n=20）——修前 13/20 次触发宿主 `Slow PreToolUse hooks` 告警 2016–2344 ms，根因不在 ce 侧：`plugin/bin/ce.sh` 每 hook 重走全链（两次 SHA256 + ~15 次 fork，Windows 每 fork 70–100 ms，单 wrapper 1.7 s）。修法 = **校验按会话一次**：验证腿把已验证绝对路径写入 `CLAUDE_PLUGIN_DATA/bound-<清单版本>.env`（tmp+mv 原子，含 `'` 的路径拒写），后续 hook 经戳直接 exec（戳比清单旧、或任一绑定二进制比戳新即重走全链；`health` 恒全链；`run_path_ce` 永不写戳）；信任边界不变（ADR-007 句）。修后单 wrapper 0.21 s，全链 min 322 / median 385 / **p95 502** / max 568 ms，宿主告警 0（PERF-BUDGET 合计行 + 全链注）。`bootstrap_e2e.sh` 11→**15 态**：11 戳快路零哈希（PATH 上投毒 `sha256sum`/`shasum` 不被听见）、12 更新的二进制不受戳信任（投毒经效果被听见：R3 降级 PATH ce 带 `pin unverified`，真哈希回来后重签戳〔哨兵行消失〕、快路复活）、13 `health` 恒全链、14 未验证腿永不留戳；态 9 计数排除戳文件。plugin/README、README 双语同句。
**O11 竞品复扫（plan-set 例行）**：67-agent 核验 Workflow（8 行逐条重扫 + 逐 claim 反驳 + 完整性批评）+ 亲核四份新入场 README。§2 就地改写 1424→1358 字符（只准变短）：jscpd v5.0.16 master 2026-08-19 合入 `--baseline`/`--fail-on-new-clones`/`--baseline-from-ref`（未发版）；mizchi/similarity「无 gating」勘误为 `--fail-on-duplicates` 批扫门；CodeScene MCP 提交前 staged pass/fail；停更日期入行。**R5 触发器：击发，带两条保留**（jscpd 门未发版；新入场 dupehound 89★ 已发布变更函数级 diff 门 + MCP，同栈 winnowing，但不做写入时拦截；Claude Code 2.1.251 无内置查重，#10170/#34535 均 closed not-planned）——用户裁（2026-08-29，委托取效果最好最优雅）：**击发-观察、路线不动**，引擎复用不做（ADR-005 三条否决理由对 dupehound 逐字成立），dupehound 转 M 轮作 T1/T2 召回对拍方，R5 触发器改写为「写入当轮拦截 + 克隆判定兼备」；§2 加新入场行（dupehound / nose / vibeguard / slopo，1358→1959 字符），例行规则改「除新入场行外只准变短」。
**CI 两修**：Windows 腿 `cargo package` 校验挪到 `cargo test` 之前（e2e 遗留的 `ce daemon` 持有 `target/debug/ce.exe`，共享 CARGO_TARGET_DIR 下校验重编译撞锁，33227318755 / 33232925443 实锤）；ubuntu 腿子仓狗粮 `ce scan` 硬线红（33249612101）——`knob_default_drift_gate` 99 行拆三（`knobless_echo` 助手 + 镜像两腿）、`the_named_acts_write_what_they_promise` 80 行拆二（`established_then_fenced` 夹具 + 增长腿），子仓 scan 0 fail。
**记账**：dedup 主仓 **60** / 子仓 **119** 行集零差（子仓曾长出三块——两个拆出测试的同形前奏折进助手、`sites_tests.rs` 表列改 `kind=spec|…` 单字面量〔元组切片在 ID/LIT 归一下同流〕）；lib 230 / it 262 (3 ignored)；主仓 scan 349 文件 0 fail；引文台账 22 条随行移重瞄再签（ADR-007 +5 行、PERF-BUDGET +11 行、store.rs +4 行、deadcode/ladder 改行；两条锚文本改变者〔GRAPH_REV 常量、deadcode 信息表注释〕人工重瞄）；check 951（地板 946）/ 子仓 988（地板 983）；棘轮具名重立 主仓 `plugin/bin/ce.sh`（223→293 行，会话戳 + 审查修）、`graph/ladder/mod.rs`（248→249）；子仓 `it/core_wire.rs`（313→323）、`it/baseline_policy.rs`（249→252）、`unit/graph/deadcode.rs`（76→79）。

**步 #14 乙 围栏收尾片 (c)：O39 旋钮摘要规范形 + O64 客户端期限拆除；零 wire、无默认档位变更**（2026-08-29，ccm 步 #14 收官；乙 19 条至此全落：片 (a) 9 / 片 (b) 8 / 片 (c) 2）。
**O39 规范形**：`knobs_digest` 的哈希对象从「整份序列化 Config」改为**规范树**——与出厂默认不同的**有效**旋钮集（`config/canonical.rs`，序列化树对序列化有效默认树逐节点剪枝，四律：`null` 叶即未声明、等于默认叶即默认、数组整值比较且类对象内同律、空对象不计），有效默认 = `Config::default()` 叠核常量（`score::knobs::core_defaults` 12 值 + `CORE_SCC_FLOOR` 2 + `TrendCfg::core`，由 `core_wire::knob_default_drift_gate` 对核回显活钉：三张 rows 表逐码对拍、trend 回显 `[[0,3],[1,0]]`、graph 缺 `sccFloor` 与显式 2 同判而与 1 异判）；`[thresholds]` 拆 `config/thresholds.rs`。**效果**：写成默认值的旋钮、没人声明过的可选项、注释与键序都不动摘要（6.0.0 起每个 `Option` 旋钮写 `null`，schema 新增可选项即移动全部非默认仓的摘要——修的是这条）；一份固定声明的字面值冻结在 `config_contract::the_digest_of_a_fixed_declaration_is_frozen`（`[dedup] budget = 182` + `file_lines_warn = 250` → 13_320_460_457_564_820_659；silent / frozen / moved 五行 + 类 glob 与 exclude 相异 + Windows 拼写按名拒绝〔O42〕而非第二摘要）；单元 7 腿（默认即静默六行表、树只含声明）。**一次性迁移**：本仓与测试子仓的摘要各移一次，随本片具名重立；下游非默认仓升级首跑 `knobs_digest` 具名停下一次，`CE_ACCEPT_BASELINE=1 ce baseline` 重立即可（VERSIONING 6.0.0 条目补注、§4 一句；README 双语、stack 卡双语、册 05 两处引文同批）。
**O64 期限拆除**：`daemon/cancel.rs`——客户端 75 s 期限到时主线程**拆除**工人而非任其停读：Unix 经复制的描述符 `shutdown(Both)`（读返回 0 → 「daemon closed the connection」）；Windows 对管道句柄 `CancelIoEx`（interprocess 2.4.3 以 `ReadFileEx` + 可唤醒等待读管道，`CancelSynchronousIo` 够不着；只取消在途 I/O 故每 20 ms 重发；`windows-sys` 加 `Win32_System_IO`）；先置标志再取消 / 工人先登记流再读标志，重连竞态闭合；懒启动重试环在两次尝试之间读标志即止。宽限 = 工人自己的重试预算 20 × 100 ms + 500 ms = **2.5 s**，其内返回的应答作废；唯有内核仍持着的 `connect` 无物可取消——宽限过后按名**脱离**（错误文本带阶段 `still connecting/starting/reading … detached`），计入进程级 `PARKED` gauge → doctor 文档 **`ce.doctor-report/0.3.0`** 加性键 `daemon.parkedWorkers`（健康进程恒 0；控制台一行与 GUI 一行**仅非零时**渲染，双语键 `parkedWorkers`；`doctor_face` 钉 0）。DAEMON.md §2 那句「GUI 的 doctor 探针每次撞上卡死 daemon 至多滞留一条停读线程」明记撤销、gui.md Doctor 行同批。**实测**：静默 daemon 持连接 5 s，客户端 300 ms 期限后 **< 2.8 s** 内带着「did not answer within」返回、无 detached、gauge 不动；反事实惰性取消器（`Canceller::inert`，即 O64 前行为）同一 daemon 下耗尽整个宽限、错误具名 `still reading … detached`、gauge +1，daemon 放手后回落——取消即拆除的因果由这一对腿钉住（`client_tests.rs` 共享 `probe` 测量 + 进程级 `GAUGE` 互斥）。`bounded_with` 与宽限常量住进 cancel.rs（client.rs 322→268）。
**记账**：dedup 主仓 **60** / 子仓 **119** 行集零差（子仓 O64 两腿曾长出两行——测量前奏与 eject.rs `deaf_daemon` 的 bind 前奏——分别以 `probe` / `listen` 助手结构消解）；check 主仓 951 / 子仓 988，两仓具名重立（主仓过容差 2：`score/knobs.rs` 145→174〔核默认镜像〕、`health/doctor.rs` 127→144；新文件 `config/canonical.rs` 106、`config/thresholds.rs` 66、`daemon/cancel.rs` 253；子仓过容差 4：`it/config_contract.rs` 159→228、`unit/config_knob_fingerprint.rs` 116→252、`it/core_wire.rs` 239→303、`unit/daemon/client_tests.rs` 143→225）；lib **230** / it **259** / clippy 0 / GUI 四腿 8/8；`docs_citations` 门随 config.rs 拆分与 VERSIONING 增行重瞄 7 条后重签，(target,text) 多重集对 HEAD 账本零锚点消失；`zh_surface` 两腿改为各自的 scratch 库（曾共用同名目录且 `tmp` 每次调用先 wipe，305 s 全量并跑下一腿的 wipe 落在另一腿的 mention pass 上——`database is locked` 一次实锤，单跑绿；修后全量重跑）。

**步 #14 乙 围栏收尾片 (b)：wire 6.4.0 围栏批——O32 / O33 / O37 / O38 / O40 / O43 / O59 / O66；核·CLI·GUI·文档四面同批**（2026-08-29，ccm 步 #14；O39 规范摘要与 O64 daemon 取消器另记）。
**O40 出处表**：`verdict.request` 加性 `present=[u64…]`——作用域内在盘、本次无连续行的文件实体（`scan::walk::collect_unignored`：与测量 walk 同树、同内置排除/秘密表/隐藏规则/归属剪枝，**唯独不读** ignore 文件与 `exclude`，因为那正是被看守的两条路；实体自此按**项目根**键控〔`score/provenance.rs::Keys`，`ce check cli` 的行与根基线同键〕）→ 回执 `ratchet.dropped=[[entity,code,committed]]` + 第六具名 fail 条件 **`rows_dropped`**：被排除藏起的文件其已提交行是「掉线」而非「移除」，仅 `CE_ACCEPT_FENCE=1 ce baseline` 可认领（写入不含这些行的基线）；删除文件仍是移除、不成立任何条件（`fence_wire.rs` 反事实两腿）。报告 **check 0.5.0**（`ratchet.dropped`，仅在表上过线时在场）、GUI Score 屏第五寄存器。
**O33 scan 围栏**：`scan.request` 加性 `knobsFence`（`null` = 无基线未围；`[current,recorded]`）→ 回执 `failed` 具名序 `hard_line, knobs_digest, degraded`（`fail ⇔ failed ≠ []`；核 `CE.Scan.Fence` 自有模块）；`ce scan` 在 `ce check` 失败的同一漂移上退 1、控制台 `-> FAIL (failed: knobs_digest)` 双语、报告 **scan 0.2.0** `failed`、GUI 诊断枢纽以 chip 显示；PreToolUse 守卫在配置漂移或基线不可读时按**出厂** thresholds/exclude/classes 判预算并在拒绝理由具名围栏（`guard/budget.rs::fenced`；`[guard]` 模式照声明）。
**O59 环底**：`[graph] scc_floor`（0 在 `Config::load` 按名拒绝）一份配置两张脸——`graph.request` 加性 `sccFloor`（上过线即回显；1 时单点 SCC 仅在自环时成环，`Graph/Cycles.hs`）与 `thresholds` 码 7 `cycleFloor`（上过线才回显）+ `cycleSelfLoops`（cycleFloor 1 必须在场、他处拒绝；由 graph 回执单点环投影，check 与 join 两路同发）。**O37** classKnobs 码 4 `cognitive_ratchet_tolerance`（仅对 metric 1 取代码 3；零有意义）。**O38** 类 id 域 1..=64（四读者同一谓词 `classIdPastFence`）。**O43** 每份回执（含降级）`newBaseline` 回显 `knobsDigest`。**O32** `score/wire_check.rs` 对每份回执核六律：fail/failed 析取、降级具名、围栏策略（基线摘要 ≠ 声明 ⇔ `knobs_digest`）、摘要回显缺席 ⇔ 未发、newBaseline 形（写者要落盘的文档）、present ⇔ dropped（缺 dropped = 6.4.0 前的核，按名拒绝）——纯函数，单元表 28 例。**O66** `fixture_contract.rs` 自十二 golden 推导 VERSIONING §3 三元组并对拍 Spec.hs 清单。
**协议**：ce↔core **6.4.0**（十二 golden 机器再生，全部新键缺席时回执逐字节如前——仅 proto 戳改动；VERSIONING §1 条目、§3/§4 同批）。**核电池**：K48 码 4 / K49 present→dropped / K50 cycleFloor / K51 knobsFence / K52 sccFloor 五腿 + ScanProps/ClassProps/VerdictWireProps 栏位探针移到 65。**减法**：四处 core_size_gate 撞线各拆一块——Scan.hs 拆 `CE/Scan/Fence.hs`（291→270）、Verdict/Wire.hs 拆 `CE/Verdict/Baseline.hs`（`parseBaseline` 读文档不读请求，295→237，另删三段随检查器早已迁 Rows.hs 的孤儿文档）、Verdict.hs 拆 `CE/Verdict/Faces.hs`（回执四面：具名 fail 行 / ratchet / newBaseline / digest，312→240）、VerdictWireProps.hs 拆 `VerdictFenceProps.hs`（K49/K50，319→245）；`ClassKnobs` 得具名 `Knob` 型；present 集由 Ratchet 自建。**记账**：dedup 主仓 **60** / 子仓 **119**（行集对拍：主仓 63→60 三行离场零新行——Docdup/Cost.hs↔Verdict/Cost.hs（84 tokens）随类表第四码消解，GraphWireProps.hs 七腿 battery 改写为契约三面 `refusals <> roads <> export` 后对 AuditProps.hs/SplitProps.hs 两行离场，Graph/Cost.hs、Structure/Cost.hs 对 Verdict/Cost.hs 两行收窄 111/169→52 同对保留，ce.toml 台账入账；子仓 119 恒、行集零差——fence_wire.rs 的 `report` 收退出码、`refused` 助手（1 = 判决拒写 / 2 = 配置故障）、common 共享 `WHOLESALE`/`FENCE`/`declare`/`ce_triple`、wire_check 单元表三构造器）；`ce check` 主仓 951 / 子仓 988，地板 946 / 983 不动；两基线具名重立——主仓超容差 19 文件（cli/src/config/rules.rs 114→126、core/app/CE/Scan.hs 252→270、core/app/CE/Verdict/Rows.hs 142→177、core/test/ClassProps.hs 223→253、core/app/CE/Verdict/Cost.hs 229→254、cli/src/config.rs 296→317、core/app/CE/Verdict/Ratchet.hs 127→147、core/app/CE/Verdict/Knobs.hs 111→122、cli/src/score/baseline.rs 209→253、cli/src/graph/deadcode.rs 513→550、contracts/VERSIONING.md 612→630、cli/src/score/mod.rs 351→382、cli/src/scan/walk.rs 319→358、cli/src/join/verdicts.rs 137→149、cli/src/scan/wire.rs 215→252、core/app/CE/Graph/Contract.hs 193→208、cli/src/guard/budget.rs 219→264、core/test/GraphWireProps.hs 148→198、core/test/ScanProps.hs 184→227）+ 新实体 6（core/app/CE/Verdict/Baseline.hs 48、cli/src/score/wire_check.rs 186、cli/src/score/provenance.rs 73、core/test/VerdictFenceProps.hs 96、core/app/CE/Verdict/Faces.hs 87、core/app/CE/Scan/Fence.hs 30）；子仓超容差 1 文件（it/common/mod.rs 229→249）+ 新实体 3（it/fixture_contract.rs 125、it/fence_wire.rs 240、unit/score/wire_check.rs 239）；CHANGELOG.md 本段自身 389→395。文档：方法学 05/06/07/11、站点 how 双语（第六条件）、stack 双语 + svg（proto 6.4.0）、`ce-toml.md` 再生（`graph.scc_floor` / `cognitive_ratchet_tolerance` / exclude 掉线句）、gui.md、计划书 ADR-008 两处就地注记；引文门按 git diff 机械重瞄 172 + 手核 14 后再签。

**步 #14 乙 围栏收尾片 (a)：七条零 wire 项——O41 / O42+O20 / O30 / O31 / O34 / O35 / O36；无默认档位变更**（2026-08-29，ccm 步 #14；片 (b) = wire 6.4.0 批〔O32/O33/O37/O38/O40/O59/O43/O66〕、O39 规范摘要、O64 daemon 取消器另记；每条先由 codex 中转站勘察 HEAD 与设计稿的错位〔k15，read-only〕再亲手落码）。
**O41** `ce dedup --check` 的 `--min-tokens`/`--min-distinct` 只许收紧：高于 50 / 高于 7 或 0 的值在任何测量与 core 接触之前按名拒绝（`dedup/budget.rs::gate_filters`；此前一个放宽的覆盖能让预算门无声变绿）。
**O42+O20+O18** 类 glob、exclude 列表与 `[graph] entry_globs`（O18，`deadcode.rs` 经 `globs::compile_inclusions` 编译一次）同一解析器（`scan/globs.rs`：gitignore 文法、`/` 分隔、`\` 为转义、前导 `!`/`#` 按名拒绝、`dir/` = 目录内容）——此前类走 globset 方言，同一串在两张表里两种读法；`[[rules.class]]` 的 `dir/` 自此选中目录内容（旧读法下零命中），启类仓库经此切换分数不可比、须具名重立（自仓不启类，刻度不动）。
**O30** `ce baseline` 只在项目根持久化：`ce baseline pkg` 在读、量、spawn 之前按名拒绝（`main_score::preflight` 走 `root::resolve`，`baseline::write` 另有库级同一守卫 `root::same_dir` 兜每个后来的调用方），`ce check pkg` 仍是作用域测量；`root::project_root` 无锚回退改返绝对路径（相对根曾让同一性判断失真）。
**O31** `baseline::read` 语义化：无文件 = `None`，在场但非基线文档（`null`/数组/缺两表）= 错误（退出 2，文件留作证据），`score::run` 永不自行读盘——基线由调用方钉进 `Opts.baseline`（`faces::check`、`check_cmd`、`baseline_cmd` 各读一次原样发送）；缺基线不再无声建立，须 `CE_ACCEPT_BASELINE=1`（唯一会创建文件的动作）。
**O34** trend 的 pinned-soft 基线改恒等基线（`score/pinned.rs`：请求自己的三列连续行 + 成员集 + 钉住的软线 + 本次摘要，摘要缺席时不写键——core 围栏是 Maybe 相等，双缺席按规则相等）——此前两张空表无摘要让每个非默认 ce.toml 的历史点都是假 `knobs_digest` 漂移、每个成员都「新增」，trend 因忽略 `fail` 而不见；现 `trend::measure` 以 `ensure!(!r.fail)` 具名拒绝而非静默记点。**具名缺口**：该绊线无 fake-core 反例——间谍 core 须重实现核心的回显策略，违反语言分工；由 `unit/score/pinned.rs`（形状 + 摘要缺席）与 `it/trend_rebuild.rs` 同摘要绿腿覆盖。
**O35** `CE_ACCEPT_FENCE=1` 窄动作：`FENCE = [knobs_digest]`（片 (b) O40 加 `rows_dropped`），成立条件全在围栏内才接受——上限与旧值取 min、成员取当前、在声明旋钮下重钉；任何增长同时成立即按名拒绝并点出两种动作；无动作的例行 `ce baseline` 只许违规集收缩；降级判决永不持久化。
**O36** `ce check` 控制台 `-> FAIL (failed: ratchet_over, knobs_digest)` / 中文 `-> FAIL（失败条件：…）`：按核心序原样、不排序不过滤、pass 行字节不变（`report::fail_suffix` 落在共享 report.rs——scan 也将印它〔片 (b) O33〕，scan 引 score 是图轴自己计费的模块环）。
**测试**（子仓）：`it/baseline_policy.rs` 三腿（拒绝表以不存在的 core 路径证「拒绝先于测量」、三动作各写所诺、控制台英中 + floor 居中序证渲染器不排序）、`unit/score/pinned.rs`、`unit/scan/globs.rs`（平表 + 拒绝）、`unit/root.rs`/`unit/score/baseline.rs` 扩腿、`common::run_ce_env` 清两动作 env、`fixtures::seed_budget`；`config_contract`/`trend_rebuild`/`gate_e2e`/`baseline_bridge` 随读法改。
**记账**：dedup 主仓 **64→63** 具名下调（行集差恰一行：`main_erase.rs:6-13↔main_score.rs:8-15` use 块克隆消失，零新行），子仓 **119 恒**（`gate_e2e.rs:30-41↔guard_hook.rs:77-87` 为旧行换伴）；`ce check` 主仓 950 / 子仓 987，地板 946 / 983 不动；两基线具名重立——主仓超容差 8 文件：root.rs 163→177、report.rs 154→171、config.rs 271→296、score/baseline.rs 187→209、graph/deadcode.rs 498→513、main_score.rs 157→248、score/mod.rs 339→351、dedup/budget.rs 81→113，新入表 scan/globs.rs 93、score/pinned.rs 44，CHANGELOG.md 本段自身 371→389（上限→现值）；子仓超容差 7 文件：it/deadcode_e2e.rs 337→365、unit/score/baseline.rs 14→54、it/config_contract.rs 117→159、it/trend_rebuild.rs 75→126、unit/root.rs 124→145、it/common/mod.rs 218→229、it/gate_e2e.rs 61→117，新入表 it/baseline_policy.rs 249、unit/score/pinned.rs 22、unit/scan/globs.rs 78。文档：README 双语、方法学 05/10、计划书 ADR-006/008 四处、站点 how 双语、`ce-toml.md`/`cli.md` 再生，引文门 04/06/07/10/12/13 共 14 条标签按源重瞄后再签。

**步 #13 单元测试迁子仓：cli/src 内全部 `#[cfg(test)]` 代码迁 CodeEraser-tests `unit/`，源文件 `#[path]` 挂载；主仓分数只量产品代码；无 wire / 无默认档位变更**
（2026-08-28 晚，ccm 步 #13；主仓 d7f71f8 推后用户问「rust 测试代码不算在主仓行数里了吧」——实测判决 361 文件 / 60,983 行中 `cli/tests/` 已为 0，但 cli/src 内按 Rust 惯例内联的单元测试仍占 23 独立 `*_tests.rs` 文件 3,176 行 + 64 内联 `mod tests` 块 3,119 行 ≈ 10 %，经 AskUserQuestion 裁「迁出到子仓」；Haskell 电池 core/test 22 文件 3,802 行另核 cabal 包外 `hs-source-dirs` 后再议，本步不含）。
**普查**（确定性脚本，非估计）：64 个列 0 内联块（1 个名 `knob_fingerprint`，config.rs；无文档注释前缀、无内部属性、无嵌套 cfg(test)）+ 23 个独立文件声明（13 个已带 `#[path]`）+ 2 个零散 cfg(test) 项（`main_lang.rs` 的 `SHARED_ARGS` 常量、`mention/conv/mod.rs` 的 `pub(super) fn bit_of`）+ `lib.rs` 的 `testutil`；无 `cfg_attr(test)`。
**迁移**：内联块**逐字**抽出（不缩进——`cargo fmt` 会重排代码缩进而永不动字面量内部，脚本缩进反而会改坏多行字面量）到 `cli/tests/unit/<镜像 cli/src 路径>.rs`（`mod.rs` 宿主以目录名命名：`graph/mod.rs` → `unit/graph.rs`；非 `tests` 名后缀：`config_knob_fingerprint.rs`），源文件留 `#[cfg(test)] #[path = "<相对声明文件目录>"] mod tests;`（Rust 参考：非内联模块的 `#[path]` 相对当前文件所在目录，mod-rs 与非 mod-rs 同律）；23 个独立文件同名镜像迁移、声明改路径；`SHARED_ARGS` 随 `unit/main_lang.rs`，`bit_of` 落 `unit/mention/conv/tests.rs` 为 `pub(in crate::mention::conv)`、`name_tests` 经 `conv::tests::bit_of` 导入（私有模块对后代可见）；`lib.rs` `testutil` 同法挂载。验证：87 块/文件逐一与 `git show HEAD:` 做去空白去逗号的记号序列比对**零差**；lib **219 passed**（迁前 218 + #12 的 seating 测试 = 同数，87 个挂载模块全部编译运行）。教训：脚本第二阶段按普查行号定位声明，同文件前一处插入使后一处行号失效——改按形态定位并可续跑；抽出体首行留空一行经 64 文件统一剥除。
**两仓门**：子仓 `ce.toml` `entry_globs` 加 `unit/`（挂载文件在子仓自身图里无根可达，作入口；deadcode 0 dead 维持，顾问 27 恒）；dedup 主仓 **65→64** / 子仓 **118→119** 同一行对迁（`dedup/t3` 与 `docdup/judge` 两个测试块的克隆：主仓行集 GONE 恰此一行、NEW 空；子仓 +1 即它），两侧 ce.toml 账本各记；主仓 scan 判决 **338 文件 / 54,859 行**（−23 文件 −6,124 行），`ce check` **943→949**（轴 6 环轴 248‰→202‰：含测试模块的模块环随之出宇宙）⇒ CI 地板按「同咬合」**940→946**（ci.yml ×2 + 注释、config.rs、RELEASE.md、站点 index/stack 英中四页、stack.svg 两份、docs_consts_stack 钉、CLAUDE.md），子仓 **979→987** ⇒ 地板 **975→983**；两基线具名重立。
**引文与散文**：方法学 02/03/06/07/13 共 8 条指向已迁测试块的标签结构越界（EOF 之外）——按账本锚文本在单元文件中重定位（去缩进比较）后重瞄至 `cli/tests/unit/…`，其中 06「shuffle-proof 断言」曾钉在裸 `}` 上、手工改指 `assignment_is_shuffle_proof`；`bit_of`/常量迁出使 conv/mod.rs 与 main_lang.rs 中段上移，册 13 十条标签整体重瞄；再签。册 13 §8 自仓行重取：U **706**（719 − 13）/ rust **1881 (1019) / 281 (0) / 14.9 % / 14 / 281 = 5.0 % / 14 / 1580**（测试函数声明离开自有域：declared −298、unmentioned −256），haskell 不变；README 双语与子仓 README（步 #11 时的「分数继续含测试」句已过期，一并改为读者立场 + `unit/` 行）。**冻结自仓切片**：三份自仓视图（graph-slice / t3-universe / docdup-segments，141 行钉 tip 60f73e3）的漂移门自 K+1 以来靠「sha 仍匹配的行 ≥ 25」撑住，本步 `fourclass/diff.rs`、`fourclass/stacking.rs` 两行的测试块迁出使匹配数 23 < 25 三腿同红；曾试「CE_BLESS=1 全量改签所有变更行」两次均回退——candidates 文档把 rs 准入单元钉在宇宙 s+m+l 带上、样本钉在池摘要上，全量改签即换 tip 重冻结、`t3_candidates_consistent` 红回（left 1246）；定案 = **按名改签**：`eval_support::universe::refreeze_self` 只改 `CE_REFREEZE=<冻结路径,…>` 点名且文件仍在且已变的行（未变/已删的行永不动，检测器漂移无法被 bless 掉），经家族自有行喉 `row(live,lang,text)` 重签、`summarize` 重算包络，三视图必须同批改签（sibling anchor 钉同一份 path→sha 清单）；`live_text` 学第二条搬迁别名（`cli/src/<测试文件>` → `cli/tests/unit/`），graph 家族补 `FAMILY`/`graph_row` 行喉并入 `assert_self_tracks(&UniverseFamily, row, floor)` 共用骨架，`eval_support::blessing()` 与 golden 门同律只认 `"1"`；本步改签恰两行（diff 34/33 行），candidates 锚按 #5/#8 先例手改 rs 1246→1233、units_admitted 1409→1396（同减 13 = 两文件迁出的 rs 单元），eval 16 腿全绿；EVAL-SET.md 冻结册账本句就地续记「步 #13 起」段；子仓棘轮为此二文件具名重立（`eval_support/universe.rs` 143→196、`eval_graph.rs` 104→119 超 +10 容差；新文件 `unit_mounts.rs` 同批入列），`live_text` 依子仓顾问改私有（顾问 27 恒）。codex 对抗审查（gpt-5.6-sol xhigh 经中转站，只读沙箱，2.9 MB 读源实录）判 REFUTED 2 条全确认全修：must-fix——`cli/Cargo.toml` `exclude = ["tests/"]`（2026-08-26 裁定，理由是集成测试依赖仓根 fixtures 进包也跑不了）现在会让发布 tarball 缺 87 个 `#[path]` 挂载目标，而 `cargo publish` 的 verify 构建不启 cfg(test) 看不见——下载者 `cargo test` 编译即错（反事实：解包后删 `tests/unit` 再 `cargo check --tests`，实测 `couldn't read src\..\tests\unit\main_lang.rs`）→ 单元测试是随 lib 测试目标编译的活重、不在原裁定理由内，exclude 收窄为 `tests/*` + `!tests/unit/`（`cargo package --list` 196→283 项 = +87 恰单元文件，`.ce/`、`it/`、fixtures 零入包），ci.yml 两 Rust 步加「`cargo package`（共用 CARGO_TARGET_DIR，verify 24 s 只重编本 crate）→ 解包 → `cargo check --locked --tests`」腿（本地全链 exit 0），RELEASE.md §3 crates.io 条加子仓在座铁则；should-fix——`unit/` 作入口后子仓 deadcode 看不见孤儿单元文件 → 新门 `it/unit_mounts.rs` 钉「src 声明集 = unit/ 磁盘集 = 打包集」三集合并拒任何非「cfg(test)/path/mod」三行形的 `#[cfg(test)]`（孤儿探针 `unit/orphan_probe.rs` 实测 `only right` 红），其递归 walk 与 `fixtures_why::fixtures` 同形被子仓 dedup 抓出（119→120）→ 抽 `common::files_with_ext` 两处共用（119 恒）；checked_and_fine 9 项：87 条 `#[path]` 逐一手算路径、super/self 语义不变、字面量零动、梯级跟随挂载且 owner 过滤保持只读不测、引文与地板全对。 lib 219 / it 245 passed; 3 ignored / clippy 0 / fmt clean / 核未动。

**步 #12 子仓只当读者、不当被测者：wire ce↔core 6.2.0→6.3.0（加性 minor，节点角色 bit 7 foreign），索引 schema v14→v15（`files.owner`），主仓分数与 1.2.0 不可比；无默认档位变更**
（2026-08-28 晚，ccm 步 #12；用户发现主仓门实测 480 判决文件 / 77,462 行中 cli/tests 占 120 / 17,050 = 22 %——「分离测试代码不就没啥意义了」——经 AskUserQuestion 改裁步 #11 的「分数继续含测试」为**子仓只当读者、不当被测者**；步 #11 段相应句就地改）。
**唯一谓词** `gitmodules::owner(root, declared, rel)` 三态：路径段自上而下，先命中 `.gitmodules` 声明前缀 ⇒ **Foreign**（就位与否）、先命中未声明的**真** git 锚（`root::is_git_anchor`：`.git` 目录或指针可解析的 gitfile；纯文件 `.git` 不算——与 root.rs 同律，mention walk 原 `.exists()` 判法收拢）⇒ **Cut**、否则 Own。
三面同读：判决 walk `scan/walk.rs::collect` 产 `Walked{path,foreign}`（Cut 目录在门口剪枝、`Owners` 按目录记忆化；`scoped_lang_files` 只取 own；`Scope::contains` 只认 Own ⇒ 守卫/审计对外来与被裁路径惰性）、mention walk `is_cut`/`cut()`、root 解析。
**读而不测**：外来文件进索引（`files.owner` 列，refresh 快路径键 (content_hash, owner)、rescache 摘要 `rescache/2` 含位）、进图（`Node.foreign` 由索引 `foreign_paths()` 标记，包/节节点随其下外来文件而外来、根包 "" 永不）、进 U 与提及表；每个测量面剔除：克隆实例查询 `fl.owner = 0`、docdup 活行 `f.owner = 0`、顾问域 `f.owner = 0`、score/join/structure 改走 `deadcode::measured_nodes`（`file_nodes` 只剩 canvas）、`symwire::rekeyed` 跳外来、churn `unhistoried` 不计。
**wire 6.3.0**：graph/1 节点行 `roles` 得 bit 7 `ROLE_FOREIGN`（外来节点只发此位、其余角色一律不测），核 `roleBits` 加 `(7, 2)` 落测试约定同一入口位——其引用播种可达性、永不被判；`GraphProps` `deriveFlags roleBits 128 == 4`；十二 golden 机器再生（request 行按 §3 立场滞留 6.0.0——105 行，hello-ok 随 server）+ graph golden 新增第 24 对（外来节点 `[0,0,128]` 引本仓节点：外来活、被引者活、孤儿死）；无 submodule 的树不发 bit 7、十键字节如前（K16）；`contracts/VERSIONING.md` §1 6.3.0 条 / §4 表 / :455、`corelink.rs`/`Version.hs` 常量与替换式注释、Haskell 电池 25 处 request 字面量、站点 stack 芯片两页 + `stack.svg` 两份。
**子仓自带门**：`cli/tests/ce.toml`（`[graph] crate_roots = ["it/main.rs", "corelink_deadline.rs", "daemon_conn_deadline.rs"]` + `entry_globs = ["gui/"]`，`[dedup] budget = 118`）+ `cli/tests/ce-baseline.json`；主仓 CI 两个 Dogfood 步各加子仓六腿（root `tests`，`--fail-under 975`）；子仓实测 scan 0 fail / dedup 118 / check **979** / deadcode 0 dead（27 条顾问）/ docdup 0 / erase 0。
**`[graph] crate_roots`**（零 wire，新旋钮）：无 Cargo.toml 的树声明 Rust crate root——发现的机制缺口是子仓单独判决时 `it/main.rs` 不是 root，`mod x;` 挂到 `it/main/` 下、`crate::` 无锚 ⇒ 105/117 文件假死；声明根经 `GraphCfg::declared_roots()` 一处规范化，ladder `ctx_for` 与 manifest 根并集、`targets::Declared::gather` 计为 role 6 目标、进 resolve_key（改声明即重扫）；`ce-toml.md` 行由 docs_gate TSV 再生；`crate_roots_knob.rs` 两树对照。
**嵌套项目的门就地生效**（守卫/审计委托）：`root::judging_root(session, target)`——目标属 Own 由会话根判；属 Foreign/Cut 时取目标向上最近根，仅当它在会话根之内**且自带 ce.toml** 才委托（其配置、索引、基线、观察流各自），否则惰性（无门 = 无人测）；Stop 审计 = 会话根自身 + `gitmodules::gated()` 每个自带 ce.toml 的就位 submodule 各审一次（各自 git/索引/预算），判决以挂载名前缀合入同一次 Stop；`audit/changes.rs` 两条 git 腿不再遍历 submodule（超仓 diff 只见 `0 0 sub` gitlink 行；子仓改动归子仓审计）。测试：`audit_bypass` 三腿改写（gated 拦、ungated 不拦且 `changed_files` 0、纯文件 `.git` 不再改根）、`foreign_readers` 四腿（walk 标记/scan 行/dedup 预算 0/基线等于自有树；外来读者使被引本仓文件活、拼写本仓声明即否决顾问且对照树反向；守卫惰性→自有拦→gated 委托拦；U 读外来切嵌套）、`guard_hook` 边界腿改 owner 三态、`trend_submodule` 就位行 = 挂载前行（读者不入分）、`erase_e2e` 外来死文件永不入计划 + 手造行仍按名拒绝、`gate_e2e` 零克隆块树 `--check` 通过（根修 `budget.rs`：核对空 distinct 行答 `dedupBlocks: null`，原 `== Some(blocks)` 把干净树误报「pairs.rs and CE.Dedup.Cost have drifted」）、`common::walked`/`doc_tree`/`expect_write_denied` 三个共享形（各消一次克隆）。
**记账**：dedup 主仓 **182→65** 具名下调（行集对拍：HEAD 二进制旧语义在同一棵树 = 184 行 = 主仓 65 ∪ 子仓 119，双向零差；65 行集 = 182 行集去外来行）；`ce check` **952→943**（轴 6 环轴 248‰：120 个测试文件退出被测宇宙，cli/src 既有模块环占比升）⇒ CI 地板 **950→940** 同咬合度重锚（ci.yml 注释、config.rs、RELEASE.md、站点两页、docs_consts_stack 钉）；RM14 冻结 40 成员：22 个 REKEYED 继任键恰为迁入子仓者，第二代账本 `REKEYED_SUITE`（继任键 → 子仓根拼写键，一次性仪器同 `member_id` 喉咙重哈希，全部在座）+ 门读主仓 ∪ 子仓基线，三本账本拆到 `baseline_ledgers.rs`；册 13 §8 自仓行重取 U 640 (653 − 13) / rust 2179 (1028) / 537 (0) / 24.6 % / 17 / 537 = 3.2 %；册 06 角色表 role 7 + crate_roots 句；README 双语句改；主仓基线与子仓基线各具名重立（本段落地后 CHANGELOG 371 行超容差 +11 > +10，再次具名重立）。**对抗审查**（codex 中转站，只读沙箱，10 条：8 确认全修 / 1 推翻 / 1 过程态）——确认：① `structure::rows::stale_doc_rows` 遍历全部边，外来 md 的链接成边后 `dir_of` 在自有树找不到目录即整个 structure 报错「md node suite/README.md outside the walked tree」（`ce check --days` 形状；今日子仓 README 只有外链故 CI 未炸，夹具再触发实证），修为跳过外来源节点、外来文档的陈旧性归其自身树，`foreign_readers` 新腿（自有 NOTES.md 为唯一 doc 行、边挂在自有文档上）；② T3 单元读者 `unitcache::unit_rows`/`fact_rows` 无 owner 谓词，外来单元进克隆宇宙，补 `f.owner = 0`；③ docdup `exempt_counts` 读全部 owner，外来许可头计入本树豁免数，改 JOIN files + owner（夹具：root.rs 与 suite/lic.rs 同一份完整 Apache 头——单行许可头够不着 `MIN_DOC_TOKENS` 50 词根本不成段——豁免数 (1,0)）；④ `gitmodules::unseated` 只认空目录，声明路径为普通文件 / 无锚的非空目录 / 指针落空的 gitfile 时都被当外来读者而非按名拒绝；⑤ 被裁（Cut）祖先之下的声明可触发拒绝而不是随祖先剪掉；⑥ trend 座位用 `.git.exists()` 而非真锚谓词（gitfile 落空时 git worktree 泛错而非具名拒绝），`seated`/`gated` 同病——三修合一：`gitmodules::seated_at` = `root::is_git_anchor` 唯一就位谓词，`declared_where` 先过 owner 规则（Cut 之下的声明不可达、名无所指）、`unseated` = 存在且无真锚，trend/`seated`/`gated` 同读；gitmodules 测试模块拆到 `gitmodules/tests.rs`（245 + 128 行），owner 与就位用例改在**同一棵**夹具树上问（两表同形曾成新克隆块 65→66，合表后 65 行集零差）；`guard_hook` 边界夹具的声明 submodule 改为真 checkout（`.git/modules/sub/HEAD`）；`trend_submodule` 新腿 gitfile 落空 = trend 具名拒绝 + 测量 walk 具名拒绝；⑦ `[graph] crate_roots` 接受任何活路径（非 Rust、缺席者静默丢弃），改为按名拒绝「不是已 walk 的 Rust 文件」（`[structure] layout` 立场；静默丢弃会把树放回旋钮本要消灭的假死形），`crate_roots_knob` 新腿（README.md / 缺席文件两例）、ce-toml.md 行再生、册 06 补句；⑧ stack 页芯片（英/中）与 `stack.svg` 两份仍写地板 950、`docs_consts_stack` 也钉 950 而只查 ci.yml 含 940——同修为 940（历史记录 FIELD-TEST/EVAL-SET-M5-CLOSE 的 950 是当时事实、不动）。推翻：「信任账本/未解析计数聚合了外来站点」——读者的未解析引用正是「可能漏掉指向本树的边」的证据，剔除是危险方向（erase 会在漏边时放行），保留为设计。过程态：「CI 依赖未提交的子仓文件」= 子仓尚未 commit，随本次提交序列自愈。册 03 candidates.rs 八处标签 +4 重瞄、册 13 owner 标签重瞄 89-132（bless 曾把它钉到 `declared_where` 注释——引文门标签漂移教训再现）、册 06 walkidx.rs:94 新引；子仓棘轮超容差具名重立 3 文件（foreign_readers 255 / crate_roots_knob 72 / trend_submodule 185）；lib 219 / it 243 passed; 3 ignored / clippy 0 / fmt clean / 核未动（Haskell 电池承前）。教训：`foreign` 是 SQL 保留字（报错落在类型词上）；零克隆块树的 `--check` 从未被夹具走过；两个测试各写一张 `write_tree` 表即一个克隆块。

**步 #11 测试子仓：cli/tests + gui/tests 迁 public 子仓 skymanbp/CodeEraser-tests，主仓以 submodule 挂回 `cli/tests`，零 wire、无默认档位变更**
（2026-08-28，ccm 步 #11；用户裁「测试不推远端／改 Haskell」经量化否决后立公开子仓）。历史用 `git filter-repo` 抽出（`--path cli/tests
--path gui/tests --path-rename cli/tests/: --path-rename gui/tests/:gui/`，230 提交、135 文件，只改路径不改内容），子仓根 = 原 cli/tests，
`gui/` = 原 gui/tests 四节点门（主仓路径 `cli/tests/gui/*.js`，脚本 root 由两级改三级上溯）。**分数不再含测试**（步 #11 曾裁「继续含」，步 #12 改裁「子仓只当读者、不当被测者」——见其段；本段三处 submodule 语义不变，只是 walk 自 #12 起给子仓文件打 foreign 标）——步 #11 三处，加后续提交（2026-08-28；codex 中转站勘察 21 条经 Workflow 对抗核验：15 确认落码、6 驳回或无动作〔#6/#11/#13/#14/#19/#20〕）的 ④–⑩：
① mention walk 的「嵌套仓整体切出」学 `.gitmodules`——[gitmodules.rs](cli/src/gitmodules.rs)（后续提交自 walk.rs 抬出的**唯一**读者：git-config 文法——节名/键名不分大小写、引号值、`#`/`;` 注释、尾 `\` 续行、同行变量，只认 `[submodule …]` 节下的 `path`；`declared`/`unseated`/`seated`/`refusal` 四口，mention walk、root、scan walk、audit、churn、trend 同读；与 `git config -f .gitmodules` 对拍）读**被跟踪的**声明文件，声明的
路径是本树的（其 `.git` 文件不再当外来仓），「一提交一 U」在有无 checkout 时同样成立；`cut()` 同律，K23 公式 `mention_universe.rs` 改为**每仓各问
git 两次**（根 + 每个已就位的声明 submodule，子仓列表加前缀、超仓的裸 gitlink 行跳过），自仓 U 627→633 = 原 627 + `.gitmodules` + 子仓 README/
LICENSE/.gitignore + 新测试文件 `trend_submodule.rs`（后续提交 →635：+ `gitmodules.rs` + `history_recipes.rs`），`outside.nested` 0；册 13 §8 自仓行重取 rust 2993 (1244) / 932 (31) 31.1 %（后续提交 3044 (1257) / 953 (31) 31.3 %，碰撞得救 40 / 953）；② `ce trend` 的临时 worktree 把 gitlink 渲染成空目录，会让切换后每个提交少了测试而打分——
`trend/worktree.rs`（自 mod.rs 拆出，283→239 行）按 `ls-tree -r` 的 160000 行给每个 gitlink 在超仓自己的 submodule checkout 上挂嵌套 worktree
（`git -C <root>/<path> worktree add --detach <wt>/<path> <sha>`，离线、确定；未初始化 = 具名拒绝「not checked out」而非空树打分），Drop 先拆座再拆树；
③ CI 两处 checkout `submodules: true`，八条 `node gui/tests/*.js` 改 `node cli/tests/gui/*.js`（release.yml 不动：矩阵只 build，crates.io `exclude = ["tests/"]`
不动）；④ 未就位 = 具名拒绝，四面同一句「submodule … is not checked out … `git submodule update --init` first」：scan `walk::collect`（scan/score/check/baseline/dedup/graph 共用——`ce baseline` 因而写不出缩水基线）、mention `universe`（`MENTION_REV` 1→2，spec 勘误 ⑮）、trend（既有），且 `ce trend` 退出码改按 `failed` 读、永不按 `pending`（原缺陷：拒绝点上了控制台/JSON/GUI 却不上退出码——P15 同类）；排除模型剪掉的声明路径（`vendor/`）不拒绝；⑤ root：声明 submodule 归超仓（`root.rs::superproject_of`——2026-08-21 审计锚定先例的细化：`.git` 锚先问超仓 `.gitmodules`，未声明嵌套仓与嵌套 ce.toml 仍自立；根因标本 = guard 信封 cwd 落在 cli/tests 时在 `cli/tests/.ce` 铸了一份只含测试的索引），`ce eject` 扫尾在他人项目边界止步并具名「left to its own eject」；⑥ Stop 审计两条 git 腿对 submodule 全盲（超仓 `diff --numstat` 记子仓为 `0 0 sub`、`ls-files --others` 永不列子仓文件，实测）→ `audit/changes.rs` 对每个已就位 submodule 问子仓自己的 git（路径加前缀），未就位的进 observe 行加性字段 `unmeasured`（partial，永不 `skipped`）；⑦ scan walk 的范围判定改 `ignore::IncrementalIgnore`（`Scope`）——根 `.gitignore` 在嵌套 `.git` 处止步，与 git 及 `collect` 同律，手写祖先环 `ignored_by` 删除，guard 预算规则同读；⑧ churn 报告 0.1.0→0.2.0 加性键 `submodules_without_file_history`（声明 ∩ 有判决文件；超仓只见指针提交，行缺席具名而非静默零），structure/fourclass 的历史面注明超仓专属；⑨ erase `--apply` 第四前置：目标落在 HEAD gitlink 之下按名拒绝（erase.md 第 3 条），CI `--check` 仍规划；⑩ 复活配方换成只读历史的 `git show <sha>:<path>` / `git archive <sha> <dir> | tar -x` + 纯 `rm` 退役（实测 2026-08-28：对 gitlink 之下 `git checkout <sha> -- <路径>` 退出 0 且静默把超仓索引里的 gitlink 换成历史 blob，旧的 `git rm` 退役放不回去）——FPR-REPLAY / PERF-BUDGET / EVAL-SET / EVAL-SET-M5-3 四册与九个退役仪器头同改，`history_recipes.rs` 三腿门（语料无 `git checkout`/`git rm` 指向声明路径、每条配方在超仓历史可解析、git 行为本身）；三处路径指针修正（gui-lens.json `_why`、commands.rs、Spec.hs）+ `lens_invariant.js` 自指自检；子仓 `.gitignore` 补 OS/编辑器规则（超仓的在边界止步）。scan/dedup/graph 的 `ignore` 走查本就进入 submodule 目录（dedup 索引 403→404 文件 = 子仓 README.md；行集与步 #10 恒等 182）。子仓第一方腿：
`mention_universe::a_declared_submodule_is_not_a_nested_repository`（声明路径及其下不切、未声明兄弟仍切、名切仍在）与 `trend_submodule` 双腿
（seated 行 = 活 checkout 同钉软线判决、且 ≠ 挂载前一行；`deinit` 后具名拒绝）；后续提交再加 19 条 it 腿——trend_submodule 五（seated 同判／unseated 拒绝／测量 walk 拒绝且 baseline 不落盘／vendor 不拒／退出码，缓存教训：seated 时量过的行按提交缓存、deinit 后仍真，批次腿自备夹具）、history_recipes 三、mention_universe 三（文法喂 cut、与 `git config -f .gitmodules` 对拍、unseated 具名拒绝）、audit_bypass 三（子仓内已跟踪编辑／子仓内新克隆仍拦、unseated 入 feed）、root_anchor 二（eject 表驱动六格：嵌套 ce.toml／未声明 .git 留给自己的 eject、声明 submodule 的 stray 归我们；声明 submodule 的 config/baseline/index 归超仓）、churn 一、guard_hook 一（VCS 边界）、erase_e2e 一，及 gitmodules 文法表／root 归超仓／erase 组件边界四条单元腿；夹具 `seed_superproject`/`unseat` 抬进 fixtures.rs。ADR-006：四脚本文件实体换路径，基线具名重立；引文 13 标签重瞄 + 两处
手瞄（册 10:9 trend `run`、册 13:46 walk 排除表）。子仓 README 写明「只在超仓内可跑」（44/80 白盒调库 API）。子仓 dedup 教训：两条新腿的「写文件 + git init + 首提交」三连与 fixtures/trend_rebuild 同韵（185 > 182 被门抓住）→ `common::init_and_commit` 一处、夹具复用 `seed_clone_pair`，行集回到 182 恒；夹具挂 `vendor/` 时活树同分 929（vendor 不进判决，实测）故挂 `suite/`。门：clippy 净、lib 212、it 215（3 ignored）、corelink 1/1、daemon 单跑 1/1、`ce check` **952**/1000、scan 0 fail、dedup/deadcode/docdup/erase --check 全 0、GUI 四脚本自新路径 8/8、docs_citations 重签后绿（台账 3 退 3 新 = trend stamp 钉文、册 13 walk 排除段、计划 281 行，按 HEAD 钉文逐条推）；棘轮具名重立（四脚本实体换路径、walk.rs +31、trend/worktree.rs 新）。子仓提交：a2cfe1e / 42de57e / 9550f77 于 skymanbp/CodeEraser-tests。**后续提交门**：clippy 净（lib + it）、lib 216、it 231（3 ignored）、corelink 1/1、daemon 单跑 1/1、core cabal 1/1、`ce check` **952**/1000（基线具名重立后 0 added / 0 over）、scan 0 fail、dedup **182 恒且行集与 HEAD 零差**（首跑 +13 新行全消：eject 走查前奏与 docdup/segments.rs 三行同形、夹具「三连写」改 `materialize` 表或循环、`answers_to_the_project` / `common::append` 抬出）、deadcode/docdup/erase --check 全 0、GUI 四脚本 27 ok、docs_consts how 页双语 chip `MENTION_REV` 1→2、docs_citations 25 条纯位移（HEAD 行块 +d 逐条对拍为证）+ 4 条手瞄再重签 1341→1343；棘轮具名重立 18 文件（root.rs 205→249、mention_universe.rs 289→343、erase_e2e.rs 184→204、erase/apply.rs 165→198、audit/changes.rs 248→271、scan/walk.rs 249→269、churn/mod.rs 267→286、guard_hook.rs 301→324、audit_bypass.rs 101→131、root_anchor.rs 103→140、trend_submodule.rs 113→135、fixtures.rs 152→175、churn.rs 112→126、churn/gitio.rs 29→38、churn/report.rs 131→137、main_judge.rs 288→292、eject.rs 272→277、audit/observe.rs 61→62）+ 新 `gitmodules.rs` 211 / `history_recipes.rs` 178；`ce eject` 干跑把 `cli/tests/.ce` 标本列为 would remove（四语料目录仍 left to its own eject），标本手动清除；教训：git 2.52 实测反斜杠换行在值里**保留换行**（非续行）、Bash 工具 heredoc 会吞一半反斜杠（含反斜杠的编辑脚本走 Write 落盘）、seated 时量过的 trend 行按提交缓存 deinit 后仍真（批次腿自备夹具）。子仓后续提交 e2ffe52 于 skymanbp/CodeEraser-tests。**四裁落地**（同日 AskUserQuestion）：① cargo test 未就位静默绿 → 不动；② bench per-tag 回填 v1.3.0 前 → bench 脚本加 seat helper（trend/worktree.rs 同机制，归复核发版步）；③ ADR-006 具名重立完整性 → 只文档，计划书 ADR-006 条文补「超容差文件与新实体逐个具名于 CHANGELOG、靠纪律不设门」一句（209 行就地，引文零位移）；④ 路径 token 门 → 仅夹具 `_why`：新腿 `fixtures_why.rs`（60 行，contracts/fixtures/**/*.json 的 `_why` 所拼仓内路径必须存在 + 反空腿 + 窄读腿），基线具名重立（新实体 fixtures_why.rs）、check 952、dedup 182 零差；子仓 67d8edb。**格式修补**（CI 33208269240：ubuntu/windows 两腿在 Rust 步末尾的 `cargo fmt --check` 红，测试全绿——步 #11 后续提交与四裁的 13 个 `.rs` 经脚本落盘未过 rustfmt）：`cargo fmt` 净化 3 源文件 + 子仓 10 测试文件，语义零变；dedup 182 且行集与 HEAD **span 级零差**；棘轮具名重立 8 文件超容差（history_recipes.rs 178→242、mention_universe.rs 343→365、fixtures_why.rs 60→80、root_anchor.rs 140→158、guard_hook.rs 324→339、trend_submodule.rs 135→150、gitmodules.rs 211→226、root.rs 249→261）+ 5 文件容差内（churn.rs +10、fixtures.rs +8、erase_e2e.rs +7、erase/apply.rs +5、audit_bypass.rs +3）；教训：脚本写码后、提交前必跑 `cargo fmt`（预算与棘轮只在格式化后的代码上量）。

**步 #10 减法批：勘察幸存 7 条 + 3 处真缺陷落码，零 wire、无默认档位变更**（2026-08-28，ccm 步 #10；v2.18 用户裁「rs 能否瘦身」）。勘察
wf_9bba332a-39e 的 20 候选经 25 agent 逐条反驳后幸存 7 条，全落：**①** graph/ladder 三家私有 `relabel`（py `__init__` 降级 R3、go replace 改写
R2、md 引用机器 R3）折为 `Outcome::with_rung(rung)` 一方法（`Unresolved` 穿透不改）；**②** `lockstep::parse_scores` 吸收两家 `parse_result` 的
zip-and-shape 尾巴（第五参 `row: Fn(R, bool) -> (i, j, E)`，`verdict_bits` 随之转私有），t3/wire.rs 与 docdup/judge/wire.rs 各成一次调用；**③**
mention/conv/{go,hs,ts}.rs 三个 ≤49 行的 AST 半折入 conv/mod.rs（145→250 行，三段 `// ---- <lang> (AST half) ----` 段注，函数改 `go_bits`/
`hs_bits`/`ts_bits`/`foreign_exports` 私有）；**④** `fourclass::kinds::typed` 单行表退役——Rust `impl_item` 的 `impl Foo`/`impl Trait for Foo` 键由
`units::impl_key` 直接产出（表从未长出第二行）；**⑤** structure/edges.rs `DirEdges` 退役——`aggregate(edges, file_dirs) -> Vec<[u32; 2]>` 只留 wire
上的每文件 (inside, outside)，`inter`/`intra` 两表自 Axes.hs 起即无读者（「未判决的表是死货」），单测改一份手算几何；**⑥** scan/report.rs 三处
并列的指标登记合一——`fn_values(&FnMetrics) -> [usize; 6]` + `ladders(&Thresholds) -> [(warn, fail); 7]`，`rows_of` 与 `evaluate` 同走一张表
（fail 0 = 无硬线契约随表注释迁移；`check_file`/`check_fn` 删）；**⑦** main_judge.rs 两个无否决包装 `family_cmd`/`emit` 删，structure/join/clone
调用点自己落 `|_| None`。三缺陷：**u64-LE 截断尾**——`unitcache::fact_rows` 的 `chunks_exact(8)`/`(12)` 静默丢尾（少 sig = 少候选 = 静默漏报），
新 `whole_rows(blob, width)` 按名拒绝非整行 blob（docdup/judge/candidates.rs 自 2026-08-19 已守的同一规则搬到 dedup 侧）+ 单测三腿；
`PairVerdict.legs_mask/.reasons` 已随步 #9 删；`DirEdges.inter/intra` 即 ⑤。附带：join/mod.rs SCHEMA_ID 注释误列 legsMask/reasons 为渲染项，
改为「留在 wire 上未渲染」。**dedup 预算 185→182 具名下调**（ce.toml 台账入账；行集差 = 恰好两家 parse_result 尾〔t3/wire.rs:81-89/86-95 ↔
judge/wire.rs:60-66/69-78〕+ `family_cmd`↔`family_checked`〔main_judge.rs:91-103 ↔ 106-115〕，零新行）。差分：四语料 deadcode 报告**字节恒等**
（cobra 5 / requests 14 / ripgrep 41 / zod 197 顾问行，边/未解析/dead 零差）；自仓同树同库 HEAD 二进制 vs 新二进制逐家族报告字节对拍（结果见本段末）。
**自仓顾问 13→0 不是本批所致，是准则如实工作**：步 #9 记账段在本册拼写了那 13 个符号名，md 在 mention 宇宙内、拼写即提及（9b 取样 11:15 早于
该段写入 11:27）——文档提到的名字不再是「无人拼写」；HEAD 二进制在纯 HEAD 工作树上同为 0 行，册 13 §8 自仓行按本提交重取（U 627 = 640 − 13；rust 2981 (1241) / 927 (31) 31.1 %、haskell 1283 (294) / 295 (2) 23.0 %；出口存活句 31 / 1241 = 2.5 %）。对拍结果：自仓 scan/structure/clone/clone --units/docdup/check/deadcode/graph/graph --sites/dedup/erase/join 十二家族 + 四语料 scan/structure/clone --units/graph 各四家族，**28 路 stdout 全部字节恒等、退出码相同**（k10/bidiff.sh）。RM14 门抓出本批唯一的 pre-Haskell 成员退役（clone t3/wire.rs `parse_result/1` ↔ judge/wire.rs `parse_result/1`，两 dedup 行折成的同一成员）→ baseline_bridge.rs `RETIRED` 账本第 12 条具名。引文：14 标签重瞄（册 02/03/06/09/13）+ 册 09:32 改引 `units::impl_key`，台账重签 3 退 2 新（按 doc/target/text 多重集对拍，零无声重钉）。门：clippy `-D warnings` 净、lib 212、it 212（3 ignored）、corelink_deadline 1/1、daemon_conn_deadline 单跑 1/1、docs_citations 绿、`ce check` **952**/1000（地板 950）、`ce scan` 0 fail、`ce dedup` 182 = 预算、GUI 四脚本 8/8；core 未动。棘轮具名重立（超 +10）：mention/conv/mod.rs +105、dedup/unitcache.rs +32、graph/ladder/mod.rs +19、CHANGELOG.md +19；Rust 净 −70 行。**教训**：含引号的编辑脚本用 Write 落盘再 `python file` 跑，bash heredoc 会被噎住；自仓普查/顾问数字必须在记账段写完之后取。

**步 #9 自仓「pub 但仅本文件用」候选逐条处置，零 wire、无默认档位变更**（2026-08-28，ccm 步 #9；O04）。K23 时估的 ~101 条是仪器前数字；
顾问（步 #6–#8）落地后自仓实测 **38 行**（rust 36 + haskell 2；rust 出口未提及 67 / 1269），裁定流 wf_a0724240-731（9 判官 + 9 核验者，每行 grep
全仓含 cfg(test)/it/GUI/核/契约）：**转私有 21**——health.rs `status_line`、dedup/struct_fp.rs `STRUCT_SHINGLE`、erase/apply.rs `LOG_SCHEMA`、
visibility/py.rs `public_by_convention`、main_cmds.rs `findings_fmt`、graph/canvas.rs `FileCycles`（连带 `document` 同私有：collision blindness——
`doctor::document` 同名遮住了它本该出的行）、deadcode/flags.rs `ROLE_ENTRY_DIR`/`ROLE_TEST`/`ROLE_GLOB`/`ROLE_DOC`/`ROLE_DECLARED`、lockstep.rs
`pin_knobs`/`scores_and_counts`、scan/walk.rs `scoped_lang_files`、score/baseline.rs `BASELINE_FILE`/`fn_entity`、score/wire.rs `CAPABILITY`、
structure/tree.rs `PATTERN_COUNT`/`CONV_README`/`CONV_CONFIG`、core Scan/Cost.hs `goLang` 出导出表；**pub(crate) 4**——config.rs `GraphCfg`/
`StructureCfg`/`TrendCfg`（字段同缩，否则 `private_interfaces` 在 `-D warnings` 下红）、progress.rs `Span`+`span()`；**pub(super) 4**——unitcache.rs
`FactRow`+`fact_rows`、join/verdicts.rs `PairVerdict`+`Judged.pairs`、structure/judge.rs `TreeRow`+`tree` 字段、mention/rates.rs `Vetoed`+`vetoed`
字段（核验者纠正原「保留」）；**内联 1**——docdup/judge/mod.rs `run_rows` 折入 `run`；**删 1**——core Clone/Prefilter.hs `labelInter`（无读者，
haddock 归 `interH`）；**保留具名 6**——churn `pair_texts`（1114aed 复活配方裁定）、unitcache `UnitFact`（`unit_facts` 返回类型，it 外部读者）、
docdup `Cand`（再生成契约返回类型）、graph `FileSites`（`analyze` 返回类型）、mention/store `Tuned`（RAII 守卫已是最窄）、docdup/shingle.rs
`shingle_set_k`（核验者纠正原「内联」：退役 k-window 探针的复活基，注释改为 pair_texts 式具名）；**转减法批 1**——structure/edges.rs `DirEdges`
（核验者：只缩不删会把死字段 `inter/intra` 留在更窄的可见性里，随 v2.18 减法批整体退役）。编译级联即顾问的第二次判决：`PairVerdict` 缩窄后
编译器暴露 `legs_mask`/`reasons` 自 2.33.0 起存而不读——删两字段（reply 列仍在 wire 上，`_reasons`/`_legs_mask` 具名弃用），模块注释改写。
处置后自仓顾问 **38→13**：8 行 `restricted_unmentioned`（pub(crate)/pub(super) 仍带 VIS_EXPORTED，行改码不消行——只有去 pub 才消行，册 13 §3）
+ 5 行具名保留；rust 出口未提及 67→44 / 1247、haskell 4→2 / 294；dead 0 恒。册 13 §8 自仓行按本提交重取（U 630 = 643 − 13；rust 2987 (1247) /
963 (44) 32.2 %、haskell 1283 (294) / 297 (2) 23.1 %；出口存活句 44 / 1247 = 3.5 %）。门：lib 211、it 全绿、core 全绿（Prefilter/Cost 导出表改）、
`ce dedup` 185 行集零差（三块仅行号位移）、`ce scan` 0 fail。**教训**：一次「pub 仅本文件用」候选里藏着两种真相——名字被同名遮蔽（`document`）
与从未被读的字段（`legs_mask`）——都要让编译器说话再记账，不靠顾问一家之言。

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
