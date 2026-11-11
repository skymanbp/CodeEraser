# contracts/ — 契约版本化机制 归档册（proto 6.0.0–7.1.0）

> 6.0.0–7.1.0 十一条版本条目（含 6.4.0 附注），即 [VERSIONING.md](VERSIONING.md) 倒序段在 3.0.0–5.1.0 迁出之后最老的一截（更早的 3.0.0–5.1.0 在 [VERSIONING-ARCHIVE-3.0-5.1.md](VERSIONING-ARCHIVE-3.0-5.1.md)、2.15.0–2.33.0 在 [VERSIONING-ARCHIVE-2.15-2.33.md](VERSIONING-ARCHIVE-2.15-2.33.md)、2.1.0–2.14.0 在 [VERSIONING-ARCHIVE-2.1-2.14.md](VERSIONING-ARCHIVE-2.1-2.14.md)，
> 2.0.0 一条仍在正册的导语段里）。2026-11-11 从正册迁出：守卫车道的 9.1.0 同版加性段落下后正册读 747 行，判决车道的三条同版注记再进来就过 `ce scan` 的
> 750 行硬线，按仓规「拆分优先于豁免」与前三册同法另起一册。条目逐字节搬家，未改写，仍按版本倒序；信封、SemVer 协商规则与
> fixtures 约定只由正册声明，本册与正册同一立场（不冻结：条目所述的规则若改，仍在条目上就地改写）。

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
