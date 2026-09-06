# Changelog

> 记录义务来自 DEVELOPMENT_PLAN.md §4.2：“每次默认档位变更在
> CHANGELOG 记录依据（FPR 数据）。”本册记两类：守卫**默认档位变更**
> （依据 = FPR 数据，每个版本条目首句声明有无）与每步落码的**功能 /
> 协议 / 记账变更**（v1.2.0 后的 [Unreleased] 块起，发版时并入版本条目，
> 源码克隆与 crates.io 包内即有全史）；GitHub Releases 留发布说明与分数
> 可比性声明（v1.2.0 及更早的功能面只在那里）。

## [Unreleased]

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
- 子仓：`unit/update/version.rs` 六行表 + 花名册往返腿、`unit/update/manifest.rs` 按 `TARGETS` 逐目标（已建者三枚 64 位 hex pin，其余 `pins()` 具名拒绝）、`it/facts/count.rs` 的 `roster()` 从 `TARGETS` 推导 binaries / platforms / installers 三个事实。
  dedup **55 / 119 恒**——本批两块新克隆（`packaging.rs` / `release_roster.rs` 各一份 `read(rel)`、`packaging.rs` / `docs_diagrams.rs` 同形的 node 驱动调用）消掉：读者统一走 `facts::read`（同类的 `face_parity.rs` 一并改），node 运行器 + 双流断言进 `common/gates.rs`
  （`demo_replay.rs` 同改；先试开新模块 `common/script.rs`，`common/mod.rs` 的索引随即与 `eval_support/mod.rs` 同形、实测多出一块，故并入既有模块）。
- 记账修正：步 9 批 A / 批 B 与步 10 第一组的三块自 b8d3c1e（第三次拆册）起被追加在 v1.5.0 段末、「更早的版本」之前，现移回 `[Unreleased]` 段（字节不变，只挪位置）。
- Opus 只读审阅 22 条，落 18 条：**4 blocker**——tag 门的等待环把本 run 自己在 `needs:` 上排队的三条可选腿也算进 pending，v1.7.0 首打必等满两小时被拒 → 按 check suite 过滤本 run 未完成项（已完成的 skipped `build` / `draft` 仍按名赦免）；build-target.yml 新加的 `[ "$(ls dist | wc -l)" = 3 ]` 在两条 macOS 腿上是 BSD `wc` 带前导空格的串比较、正确构建也红 → `set -- dist/*; [ "$#" -eq 3 ]`；§5.10 布局树缺 `packaging/` 一行（`layout_tree` 门在 `git add` 后才红）；README 双语 / 官网两首页四枚数词芯片（三 / 九 → 五 / 十五）随 `facts_` bless。**major**：winget `ProductCode` 由「猜是 productName」改为本机注册表实测 `HKLM\...\Uninstall\CodeEraser`（1.5.1 装机，2026-09-06）；winget 三份 yaml 改纯 ASCII（winget-pkgs 对非 ASCII 要 BOM）——生成器 `render()` 与子仓门各一道断言；bundle 表四处拼写加门 `every_host_spells_the_same_bundle_per_os`；`bootstrap_e2e.sh` Linux 臂与 `ce.sh` 锁步（未知架构 = `unsupported`）；`packaging-live` 只在周程 / 手动跑 → RELEASE.md §2.1 要求打 tag 前手动跑一次。**minor**：`winget_pr.sh` 可重跑（分支已在则 PATCH、PR 已开则 notice）、`homebrew_tap.sh` 印的安装命令用 tap 名而非仓库名、`packaging.js::winget` 拆两表 ≤ 50 行、`RELEASE_VERSION` 经 `GITHUB_ENV` 覆盖后断言非空、`bundle()` 的拒绝在 `$(...)` 里不可达 → 顶层先校验一遍花名册、`packaging-live` 按清单版本取 winget 目录、gui.md 断句。**不加 formula `version` 行**：Homebrew 从 url 的 `/v1.x.y/` 段与 `ce-1.x.y-` 词干都能识别版本，显式行会被 `brew audit` 判冗余——由 `packaging-live` dispatch 实证；可复用工作流 caller 被 skip 时的 check 名（`release-rehearsal`）待本提交推上后按 `check-runs` 实测再定。
- ADR-006 具名重立（两仓）：主 CHANGELOG 506→531 / cli/src/update/version.rs 66→139 / docs/RELEASE.md 114→149 / scripts/pin_release.js 128→140，scripts 四新文件 + packaging/winget 三份 yaml 入基线（.rb 与 .github/ 不在度量宇宙）；子 it/common/gates.rs 65→87 / unit/update/version.rs 69→97 / unit/update/manifest.rs 75→92，it/packaging.rs / it/release_roster.rs 两新文件入基线。

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

## 更早的版本

v1.4.1 及更早移入归档册：[v1.4.0–v1.4.1](docs/CHANGELOG-ARCHIVE-v1.4.md)（2026-09-05 迁出）、
[v1.3.2 及更早](docs/CHANGELOG-ARCHIVE.md)（2026-08-31 迁 v1.3.0 及更早、2026-09-05 迁 v1.3.1–v1.3.2）；
三次都是本册抵 750 行硬线所致的拆分，条目逐字节未改。
