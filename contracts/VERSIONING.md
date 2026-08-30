# contracts/ — 契约版本化机制（M0 冻结机制，内容 M4 定稿 1.0.0）

> 依据 DEVELOPMENT_PLAN.md §7.1 与评审 B1：M0 只冻结**版本化机制**，
> IR/判决 schema 的**内容**在 M4 随真实需求定稿为 1.0（2026-08-11，
> wire 形状与 0.2.0 一致的声明性定稿）。**2.0.0**（M5-1c-iii，
> 2026-08-12）：rem/add 条目携第三元素 = trim 后 alnum 宽度，喂
> Cost.anchorFloor 的站点锚地板——请求形状破坏性变更，按 §2 升 major。
> **2.1.0**（M5-2a，2026-08-12）：graph/1 族落地——加法 type + 加法
> capability，按 §2 minor；行字节预检同批放宽（见 §1）。
> **2.2.0**（M5-3a，2026-08-13）：clone/1 + docdup/1 + verdict/1
> 三族**一次 minor 同批声明**（capabilities 是纯信息发现，接受/拒绝的
> 唯一权威是 §2 SemVer，故不逐族拆 bump）；桩 handler 对一切输入回
> `error/contract`，各族判决随其批次落地后重生成 golden——graph/1
> 于 2a 声明、2g 实现的同一路径。
> **2.3.0**（ADR-008 首步，2026-08-14）：`verdict.request` 加性
> `ceilings` 表（`[[axis,ceiling]]`，axis 0=size/1=coc，缺席解析为
> 空 = 用 Cost.hs 默认）+ 应答加性 `knobs` 回显生效值——ce.toml 是
> 源、wire 是路、Cost.hs 300/15 降为**默认值**而非镜像另一半；空表
> 回显 == `Thresholds::default()` 由漂移门钉住（M5 收口审计 D2）。
> **2.4.0**（ADR-008 P4，2026-08-17）：`verdict.request` 加性
> `thresholds` 表（codes 0..6 = deadIndegCeil/rewriteNum/rewriteDen/
> cochangeFloor/violCost/defaultWeight/scoreScale）与 `tolerance` 表
> （legs 0..2 = tolNum/tolDen/tolAbs），同一 `[code,value]` 单行文法；
> 应答 `knobs` 回显扩为**全量生效集**（12 键）；weights 通道由
> ce.toml `[score.weights]` 驱动（Rust 恒发空数组退役）。判决语义
> 逐字节不变（判决表化=纯重述）；码表单一权威 = `cli/src/score/knobs.rs`。
> **2.5.0**（ADR-008 P1 判决权回迁，2026-08-17）：clone/docdup 应答
> 加性 `verdicts` 数组（每 score 行一布尔、同序——上报集自此是 **core
> 的判决**，Rust 只转发并以镜像逐行 ensure 抓漂移，绝不再推导）；
> docdup `knobs` 回显加 `verbatimFloor`（=50，判决权随 P1 迁入
> `CE.Docdup.Cost`——run 长度早已过线〔F26〕，文本从不过线）；
> **degraded verdict 应答自带 `ratchet.fail=true`**（"不能判者绝不放行"
> 由 core 自述，Rust 侧 `|| degraded` 再解释退役——语义位翻转仅此一处，
> 属 P1 契约本体，golden 无 degraded 对、由 VerdictWireProps 电池钉住）。
> **2.6.0**（ADR-008 P2 棘轮统一，2026-08-17）：`verdict.request` 加性
> `dedup` 对（`[blocks,budget]`，仅 `ce dedup --check` 发送；缺席=条件
> 不评估，ce check 之路字节不变）+ fail 具名条件表加第四行
> `dedup_budget`——第二棘轮的比较自此在 core（`blocks > budget` 即 fail）；
> Rust 侧退出码消费 fail 位、报告行自渲染；`ce baseline` 的 only-shrink
> 集合再解释同批收敛为消费 fail 位（该线无 floor 无 dedup 对，fail ≡
> added∨over，语义等价）。
> **2.7.0**（ADR-008 P3 scan 分级入 core，2026-08-17）：新家族 `scan/1`
> （加性 type + 加性 capability，2.1.0 先例）——request 携测量行
> `[[code,value]]`（码 0..6 = file-lines/fn-lines/fn-params/cyclomatic/
> cognitive/nesting/fn-naming）+ 可选 `grades` 覆盖表
> `[[code,warn,fail]]`（fail 0 = 无硬线；ce.toml 是源、
> `CE.Scan.Cost.gradeTable` 是默认）；result 回位置对齐 `levels`
> （0/1/2）+ `fail` 位（任一 level-2 即 true；退出码语义在 core）+
> 生效 `grades` 全表回显（Rust 钉镜像）；`degraded.reason ∈
> {scan_too_large}` 且 degraded 自带 fail=true（P1 立场）。主体名/
> 路径永不过线（§5.9.2）；Rust `report.rs::evaluate` 降为钉住镜像
> （mcp 辅面读镜像、score 面只复用测量不读判决——反审 C11 勘误，
> `ce scan` 门以整报告 ensure 逐跑证等）。
> **2.8.0**（ADR-008 反审修复批，2026-08-17）：四路独立反审（亲审+三
> Opus 同路）20 项 confirmed 的契约面偿付——verdict.result 加性
> `weights` 生效表（0..6 全轴，`CE.Verdict.Score.effectiveWeights`
> 与评分折叠共用同一查找；反审 C3：weights 曾是唯一无往返的 knob 族）
> + `ratchet.failed` 持名条件表（消费者按名归因 fail 位，反审 C8）；
> `floor` 改按**生效 scoreScale** 校验（C7）；边界收紧同批记载：
> clone/docdup 拒自环对（C11）、docdup 升序改 (i,j) 身份前缀（C10）、
> 帽盖 knob/grade 表（C15）、scan degraded 回显默认表（C14）。Rust 侧
> =corelink 上浮 error/contract 的 code+message（C4，"desync" 不再吞
> 具名拒绝）、scan 分块（C5，行帽内分请求）、grade_rows 预校验指名
> ce.toml 键（C6）、degraded 读真布尔（C9）、check-report schema
> 0.2.0（C12）。收紧不升 major 之据=同机锁版客户端（挂账清零批先例）。
> **2.9.0**（M6 S2 structure/1，2026-08-17）：新家族（加性 type +
> capability，判决与声明同批）——树尺度熵判决：request 携稠密目录
> `nodes [id,parent,depth,subdirs,files]`（id==下标、parent 先于子、
> 根自环深 0）+ `patterns [dirId,code,count]`（命名模式分布，码 0..6）
> + `conventions [dirId,bits]`（1=README/2=config）+ `fileRefs
> [dirId,inside,outside,count]`（逐文件引用触点聚合）+ `knobs` 表
> （码 0..8，既有文法；`CE.Structure.Cost` 默认）；result 回五判轴
> `axes`（S0 几何/S1 命名/S2 混流/S3 错位/S4 文档）+ `score`
> （Score.hs 公式形等权五轴）+ `entropy`（0=全局命名 Tsallis-2‰、
> 1=扇出分布‰）+ `findings [dirId,axis]` 稀疏下钻 + knobs 全表回显；
> 名/路径永不过线；`degraded.reason ∈ {structure_too_large}` 自带
> fail=true；S2 报告态不设门。本家族 Rust 侧**无判决镜像**（设计册
> 拍板：无冻结仪器需求，反审 C1 缝类在设计期关闭）。
> **2.10.0**（M6 S3a A 层声明覆盖，2026-08-17）：`structure.request`
> 加性 `declared` 表（`[[dirId,weight]]`，dirId 升序、weight≥1；
> ce.toml `[structure.layout]` 编译而来，声明路径查不到走树目录=
> Rust 侧响亮拒绝）；应答**仅声明时**携 `divergence`（`[χ²‰]` 单元素
> 或 `[]`=未声明领土持有质量，数字绝不装）与 `deviations`
> `[[dirId,kind]]` 指名行（kind 0=未声明领土有文件、1=声明 bin 零
> 归属）；归属=最深声明祖先（R1 cabal 先例），`"."` 即兜底 bin；
> 未声明请求的应答与 2.9.0 逐字节同形（键整体缺席）；degraded 应答
> 不携 A 层键。散度=χ²（Σ(p−q)²/q，`CE.Structure.Entropy.chi2`，
> 全程 Data.Ratio、‰ 定标）。
> **2.11.0**（M6 S3b S6 冗余轴，2026-08-17）：`structure.request`
> 可选 `redundancy` 表（`[[dirId,dupBlocks,deadUnits]]`，dirId 升序；
> **缺席=轴 6 不判、空表=判为净**——churn 表诚实缺席立场在 wire 语法
> 的重演，Maybe 解码不设默认）+ knobs 码 9=dupMin/10=deadMin（着陆序
> 编码，S3c 的 staleMin 预留 11）；应答 `axes`/`findings` 仅表在时携
> 码 6 行，score 等权除以**判轴数**；knobs 回显恒 11 行（全表）。
> 测量侧=`ce structure --deep`：dedup 块逐目录卷积（一块记入每个涉及
> 目录一次）+ deadcode 死单元卷积，liveness degraded 时整卷积拒绝
> （伪零不上线）；两者都是既有家族的**判决输出**，树尺度绝不重推导。
> 同批勘误：regen 脚本 pair-9 注入缺幂等门，verdict golden 曾被重复
> 追加（22→26 行，重放同答故 CI 未红）——文件去重回 9 对、脚本改
> 重复请求行断言（注入块随对入档退役，P4 pair-7 先例）。
> **2.12.0**（M6 S3c S5 文档新鲜度轴，七轴面收官，2026-08-17）：
> `structure.request` 可选 `staleDocs` 表（`[[dirId,stale,total]]`，
> dirId 升序、total≥1、stale≤total；缺席=轴 5 不判、空=判净）+ knob
> 11=staleMin；应答轴/findings 码 5 行仅表在时出现（序恒升：5 在 6
> 前）；knobs 回显恒 12 行。测量=`ce structure --days N`：md 出边
> 目标（graph 边端点自带 path，节级目标归其文件）×单遍窗口 git log
> （\x01 哨兵防全数字文件名混入时间行；同 commit 双改=不陈旧）。
> 同批机制修：golden pair 5 的 unknown-knob 探针两次因 knob 面增长
> 转合法——冻结移动边界是错法；改钉稳定未知码 99，精确 max+1 边界
> 由电池随面同步持有。
> **2.13.0**（M7.5b trend/1 第八判决家族，2026-08-18）：
> `trend.request` `rows=[[ts,score,scale]]`（ts **次序不设限**——
> 2026-08-20 评审 #9 放宽：最小二乘与次序无关，而 first-parent 历史
> 合法携带回填/rebase 改期时间戳，原「倒退拒绝」误拒合法窗口，已
> 退役〔纯放宽：原受理请求应答不变，golden 同步重生〕；scale>0、
> 0≤score≤scale）+ 可选 `knobs`（码 0=minPoints 默认 3〔<2 拒绝〕、
> 1=declineFloorMicro 默认 0=report-only）；应答=最小二乘斜率
> `slopeMicroPerDay`（判决在核内全程 Data.Ratio 精确比较，回显整数为
> round 显示值，无客户端重导）+ `verdict`（0 升/1 平/2 恶化）+
> `fail`（**仅声明地板>0 且恶化才置位**）；不足 minPoints 或时间戳
> 零方差（全同秒=欠定）时斜率与 verdict 皆 null=缺席非平；knobs 回显
> 恒 2 行；超帽=完整降级应答 fail=true（P1 立场）。测量侧=`ce trend`
> （缓存 schema v7），ce.toml `[trend]` 两钮仅声明才上 wire。
> **2.14.0**（v0.6 尺寸软区间+相对软线，计划 v2.6 §A/§B，2026-08-20）：
> `verdict.request` 加性可选表 `judgedLoc=[loc,...]`（**判决语言集**每
> 文件行数多重集，非降序、值 <2^64；缺席=[]=S 不可导出；计入 row cap
> ——C15 纪律）；`ceilings` 码域 0..1 扩至 **0..4**（2=sizeHard 默认
> 750〔硬线首次入核〕、3=sizePMax 默认 10、4=softLineK 默认 2，值 ≥1）；
> `newBaseline` 加性键 `softLine`（整数或 null）：establish 时由核按
> **乘法序统计**导出 S=clamp(floor(median·r^k), [200,500])，r=median
> max(x/m,m/x)——log 单调故与 median+k·MAD(log-LOC) 精确等价，全程
> Data.Ratio 零对数（Entropy.hs 纪律）；非 establish 原样携带（重锚仅随
> CE_ACCEPT_BASELINE）。**轴 0 判决语义变更（分数迁移，发版声明义务）**：
> 二值计数改凸罚 p(x)=P_max·((x−S)/(H−S))²（x>S 全程同式，轴内有理
> 累加、轴口一次 floor，axes 行形不变）；S=基线 softLine，缺省回落
> sizeCeil；H≤S 退化为旧二值（防除零，具名测试钉住）。knobsEcho
> 12 键 → **15 键**（+sizeHard/sizePMax/softLineK）。
> 同一 minor 的第二面（§C 拆分 ROI 顾问，structure/1 加性）：
> `structure.request` 可选三表 `seamFiles=[[fileId,total]]`（密集
> id、total≥1，**在场即判**——回执双键随发随在，divergence 先例）、
> `seamUnits=[[fileId,unit,start,end]]`（每文件密集、跨距 1 基、严格
> 有序不重叠、不出文件总长）、`seamRefs=[[fileId,from,to]]`（同文件
> 单元提及边，from≠to，行严格升序）；三表计入 structNodeCap（C15）。
> 回执 `splitCandidates=[[fileId,afterUnit,benefitMilli,costMilli]]`
> （每文件至多一行=ROI 交叉相乘最大且 ≥1 的缝）与
> `sizeExempt=[[fileId,bestBenefitMilli,bestCostMilli]]`（无可行缝；
> 0/0=根本无缝）。定价 v1：benefit=软区间罚回收（与 verdict 同一
> 曲线权威 CE.Verdict.Soft）、cost=跨缝提及边×roiRefMilli+roiPhiMilli；
> 克隆/共变价目=v1.1 预留。knobs 码域 0..11 → **0..16**
> （12=seamSoft/13=seamHard/14=seamPMax/15=roiRefMilli/16=roiPhiMilli），
> knob 回执 12 行 → **17 行**。
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
> **5.1.0**（规则包围栏 + per-class 棘轮容差 minor，K 轮步 4，2026-08-25，用户拍板 v2.14 ②）：
> ①`verdict.request` 加性标量 `classDigest`——对 `[[rules.class]]` 规范化声明（名、**声明序**的 globs、旋钮）
> 的指纹。名与 glob 仍永不过线（§5.9.2）：它们的哈希不是它们。编码为**长度前缀**（netstring 式 `tag:len:bytes`）
> 而非分隔符——首版靠 fnv1a 的 NUL 分隔，自带的腿当场抓到碰撞：名 `a` 带 glob `b` 与名 `a glob b` 无 glob
> 字节流全等（分隔符只能分隔不含它的东西，长度可以）。②`ce-baseline.json` 记录其天花板**在哪套规则包下立的**，
> 核加持名 fail 条件 `class_digest`，判据是**朴素的 Maybe 不等**且是全的：两边皆无=同意；改了规则包=不同意；
> 对着围栏之前的旧基线声明规则包=不同意；把基线记过的规则包删掉=也不同意。四种分歧要的是同一个答案：
> 具名说出来，让人去具名重立一条地板。只有 establish 写 digest（`CE_ACCEPT_BASELINE=1` 走空基线路），
> 故「同意一套新规则包」与「同意一条新地板」是同一个动作。③`classKnobs` 码域 0..2 → **0..3**，码 3 =
> 该类自己的棘轮容差（行数**绝对值**，非比例——想要它的是 vendored 与夹具树，它们要的是零或固定额度，
> 而大文件的百分比正是本旋钮要拿掉的白拿增长）。声明即**取代两条全局腿**，故 0 意味着一行都不许长、
> 全局 max(+2%,+10) 救不了它（因为根本没被查询）。它是唯一「零有意义」的类旋钮，故表的取值下界**按码判**
> 而非一刀切（码 0/1/2 是线，线为零是荒谬）。反事实：K11 = 无类声明仓 digest 缺席（**不是 null**）且 101 对
> 金样中 199 改动行里 197 行只动 proto 字段、另 2 行是不匹配文案内嵌 server 版本串〔核电池另有一腿断言
> newBaseline 无该键〕、K12 = 改规则包即 `failed=["class_digest"]` 而 `over` 为空——不是悄悄放松而是具名停下
> 〔fixtures/verdict pair 16〕、K13 = establish 记下 digest 且棘轮行仍三列、K14 = 类容差 0 时长一行即 over
> 且 allowed=天花板本身〔pair 17；全局 +10 腿够不着〕；另有 Rust 侧三腿钉指纹本身（声明序/名/glob/旋钮各一，
> 「零旋钮」≠「无旋钮」，以及长度前缀的单射性）。请求行随 minor 机器重写为 5.1.0；核电池请求侧 proto 同步 22 处。
> **5.0.0**（graph 节点行 legacy flags 列裁除 **major**，K 轮步 3d，2026-08-25）：节点行降为
> `[lang, kind, roles]` **单一元**——pre-2.28 的 flags 列离场。它自 2.28.0 roles 列成为权威后又被
> 生产、上线、丢弃了七个 minor；4.0.0 想同批砍掉却被实测拦下（flags 位 0 是公私判决轴，可见性无生产者时删列会让
> `unref_public`/`unreach_public` 连夹具都无法表达），4.1.0 的 `symbols` 表补上那个生产者，此条遂解锁。
> **档位**：§2 写死「schema 不兼容变更（删字段/改字段形状）必须 bump major」，删列正是改行形状，故 major——
> 计划原写 minor，2026-08-25 按本仓自己的规则修正（v2.14 就地记账）。代价为零：4.x 全程未发布（v1.1.0 出货 3.2.0）。
> **三列同元不同义**是有意为之：新三列 = lang/粒度/角色事实，旧三列 = lang/粒度/flags；major 在信封处拒绝一切
> 跨版本对话，那道拒绝正是使元数复用安全的机制，故 K1 由「按行元拒」改为「按 major 拒」。表级
> `node rows: mixed arity` 拒绝随之退役——只剩一种合法元数时，宽窄不对的行就是 malformed，且按**行下标**点名。
> Rust 侧 `flags::legacy_flags` 与 `LEGACY` 折叠表一并删除，随之退役的还有 `legacy_fold_is_the_pre_228_bits`
> 一条测试与 allow-claim 测试里的一行断言（电池名集差实测：Rust −1/+0，核 −1/+1 同一探针改口径）。
> 反事实：**语义保持**——夹具 pair 7 把同一批事实改走各自的通道（节点 0 的入口身份走 roles 0→flag 位 1，
> 节点 3/5 的导出面走 `symbols`），回复与 4.1.0 **逐字段相同**（dead 表码 1/2/3/4 齐全、pos、cycles、counts 皆同），
> 证明这是裁除而非语义迁移；99 对金样中 199 行改动、193 行只动 proto 字段，另 6 行 = 三条我方重塑的请求
> （pair 7/11/12）+ pair 13 的新 malformed 文案 + 两条内嵌 server 版本串的错误文案。
> 请求行随 major 机器重写为 5.0.0；核电池请求侧 proto 同步 16 处。
> **4.1.0**（导出面 minor，K 轮步 3c，2026-08-25，用户三度交本代理裁断 v2.14 K7）：`graph.request` 加性一键——
> `symbols=[[node,visibility]]`，node < 节点数、visibility ≥ 0、**严格升序**（该表是去重的 (节点, 可见性) 集合，
> 重复行=生产者丢了集合语义，按名拒 `symbol i: not strictly ascending`）。core 按 `Cost.exportVisBit`
> 读出导出节点、按 `Cost.publicFlagBit` 或上 flags 位 0——那正是 `Dead.deadTable` 一直在分的公私判决轴，
> 而它**从来没有过生产者**（`cli/src/graph/deadcode/flags.rs:9`：文件粒度永不置位，公开性是符号事实）。
> 判决码 2/4（`unref_public`/`unreach_public`）自此首次可达。该位**故意在 entryMask 之外**：导出面是判决轴、
> 不是入口主张（RG10），故它只改死节点报哪个码，永不改哪些节点死。缺席**与空表同路**（`symRows` 只喂 [] ），
> 字节与 4.0.0 客户端所得相同。表另计 `symCap`。**L 轮片 (2)（2026-08-27）起本表的 visibility 是存储字的 bit 0 投影**：
> `symbols.flags` 另存 bit 1（作用域导出）与 bit 2（`pub(crate)` 族受限）供后续 `unmentioned` 表用，`symwire.rs`
> 的 `SELECT DISTINCT … flags & 1` 在查询处掩码，本表字节与 `symCap` 定容皆不动（K34）。**同批未做**：原计划并列的 `symEdges` 不上线——K10 审计量的是
> 精度（683/683），而「无引用」吃的是召回，实测自仓 import 绑定只覆盖 1064 条 Rust 导出声明中的 170 条
> （补模块跳转到 248 条，~23%），漏掉的是全路径调用与方法调用（皆非 import 点位）。详见 DEVELOPMENT_PLAN v2.14 K7。
> **L 轮终裁（2026-08-27，用户拍板 ①）：删**——`symedges.rs`/`bindings.rs` 与 index 的 `bindings` 表随 schema v14 退役
> （提及否决器批片 (1)，DEVELOPMENT_PLAN v2.17 条；包含论证：有符号边必有某 import 行出现过该 token，否决器完全包含它），
> wire 面零变动（`symEdges` 从未上线）。
> 反事实：K5 = 无符号表/空符号表与 4.0.0 逐字节相同（99 对机器重生成后逐行对拍：195 改动行中 193 行只动 proto 字段、
> 2 行是不匹配文案内嵌的 server 版本串；核电池另有一腿直接比 `respond` 两次的字节）、K6 = 请求体无任何字符串叶子
> （`cli/tests/it/graph_export_surface.rs`，结构性断言而非按本夹具的路径列举）、K9 = 导出节点判 2 而其邻居仍判 1，
> 且死集合不动（`fixtures/graph` pair 16 + 核电池 `exportRides`）；两个旋钮各有反事实腿（读错可见性位=无面、
> 置 entryMask 内的位=该节点变入口而离开判决集）。请求行随 minor 机器重写为 4.1.0；核电池请求侧 proto 同步 19 处（Haskell 字面量 11 + Spec.hs 内嵌请求 8；`9.0.0` 的外来 major 探针不动）。
> **4.0.0**（erase class 0 退役 **major**，K 轮步 2，2026-08-24，用户拍板 v2.14）：`erase.request` 的 class 0
> （dead_file 本地计数路）自 2.32.0 被 class 3 取代、Rust 同 minor 起不再铸行，宽限窗至此关闭——**离开判决集**，
> 其冻结位保留并**按名拒绝**（`row i: retired class 0 (superseded by 3 at 2.32.0, retired 4.0.0)`），而非折进
> 「unknown class」：仍在发它的客户端由此得知接替它的是哪条路。位不重编——重编会为省一个数组槽而移动另外三个冻结码，
> `CLASS_NAMES` 改留 `(retired)` 占位（二义的两个 dead_file 同死）。纯裁除故走 major。**同批未做**：graph 节点行的
> pre-2.28 legacy flags 列本拟同批退役，实测拦下——flags 位 0（exported）是公私判决轴，符号表给可见性第一个真生产者
> 之前删列会让 `unref_public`/`unreach_public` 连夹具都无法表达（对拍实证：旧 golden 含码 2/4，删列重生成后归零），
> 故顺延至符号表落地后的 minor。反事实：K2 = class 0 行按名被拒（fixtures/erase pair 8）、K4 = 未受影响九族回复
> 除 proto 串外逐字节相同（98 对机器重生成后逐行对拍，仅 erase 两行按等价迁至 class 3、wire-errors 两条错误文案
> 内嵌 server 版本串）。请求行随 major 机器重写为 4.0.0（3.0.0 先例，§3）；核电池请求侧 proto 同步 19 处。
> **3.2.0**（规则包 scan 旁表 minor，I 轮 P3，2026-08-24，用户拍板 v2.13 ①）：`scan.request` 加性两键——
> `rowClasses=[classId…]` 与 rows **位置对齐**（长度必等、每项 < 64；缺席 = 全行走全局表）与
> `gradeOverrides=[[classId,code,warn,fail]]`（classId ≥ 1、code 0..6、阶梯同 grades 文法〔fail 0 = 无硬线、
> fail ≥ warn〕、(classId,code) 严格升序；仅非空时发）；core 按 (class,code) 查表回落全局有效表，
> `grades` 回显仍为全局表，`gradeOverrides` 到场且非 degraded 时原样回显（客户端断言往返）；两表计入
> scanRowCap；chunk 切分时类列随行同切。ce.toml 侧 `[[rules.class]].knobs` 增 `fn_lines_warn` /
> `fn_lines_fail`（P3 两键）；Rust 镜像 evaluate 按文件类取有效阈值，每判对拍恒等式覆盖到类。
> 无声明仓库 wire 字节不变。
> **3.1.0**（规则包 DSL v1 minor，I 轮 P1+P2，2026-08-24，用户拍板 v2.13 ①）：①`verdict.request` 的
> `continuous` 行可携第 4 列 **classId**——`[u_fp, metricCode, value, classId]`，路径类的 1 基声明
> 序号，0 = 默认类；全表单 arity，混排拒 `continuous rows: mixed arity`；classId < 64（栅栏 classCap）；
> 身份前缀宽 2 不变，棘轮只读三列前缀；②加性新表 `classKnobs=[[classId,code,value]]`——码域 =
> ceilings 恒发子集 {0,1,2}（sizeCeil / cocCeil / sizeHard 的类影子，**不新造码**），classId ≥ 1
> （类 0 即全局表，已有 ceilings 通道）、value ≥ 1、(classId,code) 严格升序；core 建 Map 求值、
> 缺键回落全局线，chargeAt 律与机会数不动；③回复在表到场时**原样回显** `classKnobs`（客户端断言
> 往返；无表 = 无键，旧回复字节不变）；④`newBaseline` **永三列**（类是本 run 收费参数，非棘轮
> 事实）；⑤ce.toml 侧 `[[rules.class]]`：name/globs 仅本地（§5.9.2），globset 与 exclude 同方言，
> 声明序首中，classCap 64，逐类 ladder_fault 于 load 咽喉；无声明仓库的 wire 字节不变（C1）。
> **声明一个类 = 分数迁移**（§2 发版声明义务同款）：类线一经声明，该类文件的轴 0（sizeMass 的 S/H）
> 与轴 1（cocOver 上限）换线收费，分数与声明前**不可比**；未声明 `[[rules.class]]` 的仓库判决与
> wire 字节均不变，分数序列照旧可比。
> 反事实证表 C1–C9 = core/test/ClassProps.hs + cli 侧 config_contract / scan::classes 电池。
> **3.0.0**（churn 行裁列 **major**，I 轮 D3，2026-08-24，用户拍板「现在就删」）：`verdict.request` 的 `churn` 表由五列 `[u,rewrite,append,added,survived]` **收窄为三列**
> `[u,rewrite,append]`——第 4 列恒等于 rewrite+append、第 5 列恒为 0（per-entity 存活从未测量），
> core 自 M5-3i 起两列全弃读（`Score.churnHeavy` / `Verdict.churnMap` 只解 rw/ap）；删列 = 请求形状
> 破坏性变更，按 §2 升 major：两侧实现 + 三个 core 测试 harness 的 proto 字面量 + **全十族 golden**
> 同批重生（请求行 proto 一律改写为 3.0.0；回复行经核机器再生，与旧回复除 proto/server 字串外
> 逐字节相同——判决面零变化的亲证）；「留+记愿望单」落选（用户裁）。同批 daemon 协议独立升
> **2.0.0**（`hello_ok` 砍无读者的 `version` 字段，见 [DAEMON.md](DAEMON.md)）。
> **2.33.0**（join 格深化 minor，H4，2026-08-24，用户拍板）：①verdictTable 增**严重度**列
> （delete 3 > merge 2 > hotspot 1，表数据、电池可置换）；②candidates 行**加宽为六列**
> [u,v,code,reasonBits,legsMask,**confidence**]——腿一致性置信 = 在场且有据的腿数
> （归属表 `legBits`：sim={1}, graph={2..6}, churn={7,8}）——**行变化入册**（五列消费者需随升）；
> ③回复一次性携 `joinSeverity`=[[code,severity]] 表面；④`ce join` 改经与 `ce check` 同一条
> verdict/1 路取判决（一判两面），退出码仍仅报告；check 报告 schema 升 0.3.0、join 报告 0.2.0。
> **2.32.0**（deadcode 置信 minor，H3，2026-08-24，用户拍板）：①graph.request 可携按语言点位
> 台账 `unres`=[[lang, 未解析数, 总数]]（判决集内、计数自洽、严格升序）；台账在场时每条 dead 行
> 增第三列**置信**——0 未担保 / 1 空担保 / 2 已担保（`CE.Graph.Cost.confidence`；擦除家族的信任
> 边界改由持有台账的 graph 家族亲判）；无台账的旧请求两列 dead 行字节不变。②erase 家族增
> **class 3**（dead_file 置信路：fact 1 = graph 亲判置信，仅 0 拒绝、理由码仍为 1
> language_unresolved）；class 0 依 staleDocs 纪律**被取代**——宽限窗内照旧判决，后续 minor 退役；
> 其 golden 中作"unknown class"样本的旧请求自此合法改答边界拒绝（行为变化入册）。③deadcode
> 报告 schema 升 0.2.0（dead 行带 confidence 列）。
> **2.31.0**（trend/2 深化 minor，H2，2026-08-24，用户拍板）：趋势家族能力名升 **trend/2**——
> ①斜率估计量由最小二乘换 **Theil-Sen**（成对斜率中位数，exact Rational）——**行为变化**入册：
> 同一窗口可得不同答案；一个野点可拽动均值、拽不动中位数，TrendProps 钉住两估计量符号相反的
> counterfactual。②回复增列 `cliff`=[后点请求索引, 跌落 micro] 与 `declineRun`=[起点请求索引,
> 点数] 两形状事实（索引过线、hash 永不过线，§5.9.2），低于 minPoints 时与斜率同缺席。③判决窗口
> 封顶 `tsWindow`=512 个最近点（130,816 对成对斜率实测 ~150 ms 端到端），`counts.judged` 具名切口。
> **2.30.0**（fn-naming facts minor，ADR-008 批 7 片 14，2026-08-24，用户拍板）：scan.request
> 增列对齐 `naming` facts 表（`[lang, style, upper, under, test]`，每 code-6 行一行；facts 在场时
> code-6 行 value 必须为 0——判决不再过线）；conforms 判决迁 `CE.Scan.Cost`：godoc 下划线豁免
> 仅限 Go 自己的 lang 码，前缀边界按 go vet 规则——`Testing_helper` 不再豁免、豁免不再漏到
> TS/Haskell 等一切 mixedCaps 语言（Rust 谓词的两处缺陷同死；naming.rs 降为钉住的镜像）。
> 旧路（无 naming 键）字节不变。
> **2.29.0**（H1 三件 minor，ADR-008 批 7 后续，2026-08-24，用户拍板）：
> ① verdict knobs 回声加 `judgedMask`（判决语言集 Lang 码位掩码，客户端声明、每判必钉；
> 谓词本体仍在 Rust——片 2 承诺的回声钉件；0=未声明）。② 未用 Markdown 引用定义改为解析后以
> **边种 5**（`EDGE_REFDEF_UNUSED`）过线，core 第二个惰性边种（`refdefKind` 伴 `assetKind`）
> 排除于存活——片 16 借道新建的 Outcome 通道执行；不可解析的未用定义自此为普通 miss，
> 借用 External 类退役；GRAPH_REV 升 8。③ 预判 `staleDocs` 臂退役：2.23.0 一个 minor 宽限早过，
> 旧键按 §1 未知字段规则被忽略，axis 5 仅由 raw 表判决——**行为变化**：仍发旧键的 2.22
> 客户端自此 axis 5 不判（发 raw 表的 2.23+ 客户端字节不变）。
> **2.28.0**（ADR-008 批 7 片 3 主体 entry-roles minor，2026-08-24，用户拍板）：
> `graph.request` 节点行接受加性第 4 列 role 事实位（0 具名 main / 1 可执行目录 / 2 测试惯例 /
> 3 entry glob / 4 文档入口 / 5 ce:allow 声明 / 6 清单声明构建目标）；role→entry 位表 =
> `CE.Graph.Cost.roleBits`，roles 在场时 legacy flags 列让位（同表混排两种列宽拒绝
> `node rows: mixed arity`）；3 列行字节不变。role 6 修片 3 缺陷：声明的 `[[bin]] path` /
> cabal `main-is` 目标即根，此前只有名字惯例是。
> **2.27.0**（M9 批 9 P8 环轴口径修正 minor，2026-08-22，用户拍板）：
> cycle 轴只计代码文件；`verdict.request` 加性表 `docFiles` 搭载文档语言文件的文件宇宙下标，升序；
> 缺省/空表 = 旧语义，旧 golden 字节不变；v 与 n 同宇宙；起因 = 批 9 P8 导航条实测（`DEVELOPMENT_PLAN.md` v2.12）。
> 规则仍在 Haskell（ADR-008 反抢跑），liveness/死文档检测不变。
> **2.26.0**（M9 批 9 P9 单一密度律 minor，2026-08-21，用户拍板）：
> `structure.result` 的 score 与 axes 行改走 verdict 族密度折算——
> 每轴违规目录数 v 计费 floor(scale·v/(v+N))（N=目录总数），
> 再过 violCost/structViolCostNeutral 表盘；轴行载费额（‰）非
> 计数，findings 行不变。退役的质量法即批 6 在 verdict/1 杀掉的
> 饱和形（均值 100 目录即 0 分且随仓规模线性恶化）。chargeAt 共享自
> CE.Verdict.Score：一个定律两个评分族。
> **2.25.0**（M9 批 7 片 9 豁免权威 minor，2026-08-21）：docdup 回显加
> `licHeadLines`（CE.Docdup.Cost=5，许可证头窗口，镜像钉）。豁免
> 执行留在 Rust 持久化前（豁免段无行不过线——minDocTokens 立场）；
> 标记字符串表不入钉（SKELETON_PREFIXES 定案，护栏 DOCDUP_REV）；
> 裸标记零主张规则的权威为 Cost 模块书面定案 + 2.22.0 已浮出的
> `allow_missing_why` 计数。
> **2.24.0**（M9 批 7 片 7 会话审计 minor，2026-08-21）：第十判决族
> `audit/1`——请求 `rows=[[aTouched,bTouched]]`（每克隆块一行，
> 两侧是否落在会话改动集内，Rust 的集属度量），回复
> `dups=[行序]` + `counts` + `fail`。定罪析取（任一侧被碰即定罪，
> v1 故意不对称）与零容忍阈（CE.Audit.Cost.dupTolerance=0，
> 无旋钮——零容忍不可调）入核；Stop/precommit 腿只转发 fail，
> 核不可用→可见降级跳过（A9f，绝不阻断也绝不默过）。
> 这是最后两处 Rust 常驻执法判决之一的回迁。
> **2.23.0**（M9 批 7 片 11 原始陈旧表 minor，2026-08-21）：`structure.request`
> 加性 `staleDocRows=[[dirId,docTs]]`（docTs=文档窗内最新变更，0=窗内
> 未变——唯一哨兵；文档身份=行序，图节点纪律）与
> `staleEdgeRows=[[docIdx,targetTs]]`（只载窗内变过的目标，
> targetTs>=1）。S5 陈旧谓词（严格 >、同 commit 平局、存在量化）
> 入核 deriveStale；预判 `staleDocs` 行保留一个 minor，原始表在场时
> 让位。S5 是同一 wire 上唯一 Rust 预分类的轴（S2/S3 均发原始行）。
> **2.22.0**（M9 批 7 收尾缺陷清扫 minor，2026-08-21）：docdup 回显加
> `docLineCap`（CE.Docdup.Cost=200，超长行掩码帽，镜像钉等；
> SKELETON_PREFIXES 字符串表不入钉——echo 文法是数字的，其漂移
> 护栏为 DOCDUP_REV，书面定案入 Cost 注释）。同批 Rust 侧：入口
> bit 6 得生产者（行内 `ce:allow(deadcode) -- why`，裸标记零主张）、
> docdup 报告浮出已持久豁免计数、erase class-2 行携真实覆盖位、
> guard 坏配置通知不再被空 reasons 吞、trend 缺 scoreScale 具名拒绝。
> **2.21.0**（M9 批 7 片 5/10 钉底 minor，2026-08-21）：`newBaseline` 加性
> `zoneTiers=[warn,ask]`（‰，CE.Verdict.Cost zoneWarnPermille=250/
> zoneAskPermille=750）——guard 渐进区档位地图核著，经已提交基线文书
> 抵达无 daemon 的 hook（写盘器具名拷贝；默认仍 observe-only，档位
> 只在 `[guard] zone_tiers` 显式声明后生效）。clone knobs 回显加
> `minUnitNodes`（CE.Clone.Cost=24）、docdup 回显加 `minDocTokens`
> （CE.Docdup.Cost=50）——两枚准入地板执行仍在 Rust 侧 wire 前
> （下地板行上线已议价并否决），权威入核、可消融、逐跑镜像钉等。
> **2.20.0**（M9 批 7 片 12/13/15 全证据 minor，2026-08-21）：asset 类边行不再客户端预删——行上 wire，核在与 rung 同一推导式内
> 按 **CE.Graph.Cost.assetKind=3** 排除出存活性（规则自此可消融可测试；
> 反事实测试：翻 kind 即复活）。cochange 表整体上 wire（撤客户端 rank-20
> 截断；地板随配置 cochange_floor，默认 2 与核默认字节等价）。克隆对语言
> 同一性改按语法（Lang）判定。形状零变；契约点=发 asset 行的客户端
> 需要会忽略它们的核。
> **2.19.0**（M9 批 7 片 1 多样性地板入核，2026-08-21）：`verdict.request`
> 加性 `dedupDistinct=[d,...]`（**预过滤**逐块 distinct 计数，随 2.6.0
> `dedup` 对同乘，值域 u64；无对而有行=具名拒绝）与可选
> `dedupMinDistinct`（CLI --min-distinct 覆盖时才发，>=1；缺席=核默认判）。
> 核以 **CE.Dedup.Cost.minDistinct=7**（首个 dedup 族核内常数；M2 标定
> band：仲裁假阳 distinct<=6、真克隆>=7，deny 路径的 FPR 台账即按此数认证）
> 自导准入块数并**以自导数判预算**；应答加性 `dedupBlocks`（行未乘=null，
> trend 缺席立场）供客户端逐跑证明本地过滤器等值（scan 镜像 ensure 的
> dedup 形），knobs 回显 15 键 -> **16 键**（+minDistinct=生效地板）。
> Rust 侧 DEFAULT_MIN_DISTINCT 降为**声明镜像**；探针热路径零新开销
> （报告面继续免核——测量面契约不破）。
> **2.18.0**（M9 批 7 片 4 RG9 回迁，2026-08-21）：`graph.result` 判决
> 分流入核——`dead` 表只承载**文件粒度**行（判红表，亦是 erase class-0
> 授权源），加性新表 `reported=[[i,verdict]]` 承载 package/section 判决
> （RG9：聚合不是代码实体，只报告不判死）；加性 `fail` 位为零容忍门具名
> （任一文件级死判决即 fail；降级应答自带 fail=true——verdict 族 P1 立场）。
> kind 列一直上 wire 且被校验，此前判后即弃——分流以 Rust 无名分支存在，
> 消融不可见。客户端保留分流为边界契约：判红表混入聚合=按 wire skew
> 具名拒绝，绝不授权目录擦除；缺位 `fail` / `reported` 同为 wire skew
> 按名拒绝——2.18 前旧核的「客户端按旧合取顶替」回退自 3.0.0 起已被握手
> 挡在门外，L 轮 #15 O62 裁除该死兼容。
> **2.17.0**（M9 批 6 密度评分，2026-08-21）：`verdict/1` **纯值迁移**
> （行形状零变，2.14.0 轴 0 迁移先例）——axes 行由违规质量改为**有界
> 轴费** `floor(scale·v/(v+n))`（v=轴违规质量、n=轴机会数：尺寸/克隆/
> 文档重复按文件数、复杂度按函数数、死码与循环按图节点数、churn 按
> 窗口内实体数；n=0 零费=诚实缺席），score=各轴费在 violCost 表盘下的
> 加权均值（`violCostNeutral=10` 时恰为加权均值，结构性不可饱和；
> 上调 violCost 属显式选择）。区间罚曲线过硬线 H 改 **C¹ 线性延伸**
> （H 处斜率相接，仍单调恒收费，但平方不出契约域 (S,H]）。动因=
> 批 6 实战：两真实仓库在旧「裸质量线性入有界分再钳 0」聚合下齐测
> 0/1000（轴 0 达 10176‰），区分度全失。knob 面与回执行数不变。
> **2.16.0**（M9 批 3 擦除族，计划 v2.8 ②，2026-08-21）：新家族
> `erase/1` —— 确定性两段式擦除器的**安全谓词**（契约册
> docs/reference/erase.md；ADR-008：字节归测量、可擦性归判决）。
> `erase.request`：`rows=[[class,w,x,y,z]]` 稠密整数事实行（行序即身份，
> 路径永不上 wire——Rust 按序回贴标签），class 冻结位 0=**已退役**（4.0.0，
> 按名拒绝；曾为 dead_file 本地计数路）、1=verbatim_doc
> （w=verbatim 词数，x/y=两侧段词数，z=字节相等布尔）、2=t1_twin
> （w=整单元覆盖，x=字节相等，y=副本文件已判死，z=语言未解析数）；
> **无 knob**——安全谓词不可调，任何 knob 行按名拒绝（`error/contract`）。
> `erase.result`：`rows=[[eraseable,reason]]` 按请求序，reason 冻结位
> 0=eraseable/1=language_unresolved/2=not_full_segment/3=bytes_differ/
> 4=copy_not_dead/5=unit_not_covered；`counts{rows,eraseable,advisory}`；
> 行上限 4096，超限=完整降级应答 `fail:true` 且判决表**为空**——被
> 拒判的计划不授权任何擦除。fail 仅随 degraded（计划本身不设门；
> 自仓零行门是 CLI `--check` 对本表的判读）。
> **2.15.0**（v0.7 拆分 ROI v1.1 价目，计划 v2.7 ②，2026-08-20）：
> `structure.request` 加性可选两表 `seamClones=[[fileId,start,end]]`
> （T1/T2 克隆块跨距——span 契约与 seamUnits 同一检查器）与
> `seamChurn=[[fileId,a,b]]`（churn 窗口〔14 天〕单元共变对，a<b
> 升序）；缺席=该腿零计价，2.14.0 请求原判逐字节不变（SplitProps
> 兼容回归钉住）。缝价 cost 增两腿：切穿克隆块（跨距骑缝线）×
> roiCloneMilli(500) + 跨缝共变对×roiChurnMilli(150)，回执形状不变
> ——两腿并入 costMilli。knobs 码域 0..16 → **0..18**
> （17=roiCloneMilli/18=roiChurnMilli），knob 回执 17 行 → **19 行**。
> 测量侧：克隆跨距直读 dedup 索引、共变对直读 churn 提交台账（当前
> 快照 key 联结）——两者均为既有家族事实的复用，绝不重推导；
> SeamTables 一形两面（测量侧就地装配 wire 同型，无镜像结构）。

## 1. 信封（envelope）

ce ↔ ce-core 的每条消息 = 一行 NDJSON（UTF-8，无 BOM，`\n` 结尾，binary-mode I/O）。
每条消息必带信封字段，其余字段由 `type` 决定：

```json
{"proto": "<SemVer>", "type": "<message-type>", ...}
```

- `proto`：协议版本，当前 **6.4.0**（单一来源：`cli/src/corelink.rs::PROTO`
  与 `core/app/CE/Protocol/Version.hs::proto`，两处必须一致——core 侧由共享
  fixture 钉住，两侧相等由 `cli/tests/it/core_wire.rs::corelink_open_and_desync`
  的 PROTO 断言焊住）。
- 未知**额外**字段必须被接收方忽略（同 major 内前向兼容）。
- 未知 `type` → **`error` 应答**（0.2.0 起；此前实现以 hello 形状拒绝，属缺陷已修）：
  `{"proto","type":"error","id":<回显|null>,"code","message"}`，
  `code ∈ {unknown_type, bad_request, too_large, contract, internal}`——`internal`
  为第五席，2.3.0 同代的 Main.hs 异常屏障引入（挂账清零批 2026-08-17：纯判决
  计算内任何缺陷成为 error/internal 行而非进程崩溃，id 恒 null——计算死在可信
  回显之前；Spec `refusalProbes` 钉其 code 字符串）。core 侧在 JSON 解析
  **之前**先做行字节上限预检（2.1.0 起 32 MiB，此前 1 MiB——2026-08-12 决策：
  唯一客户端是同机受信 daemon，而 graph 请求在 100k LOC 量级合法地 ~1 MB；
  真防护 = 各族容量护栏），超限即 `too_large`，不解析。
- **每条非 hello 消息的 `proto` 由 core 强制校验**（1.0.0 定稿修正，攻击评审 F8：
  0.x 实现只在 hello 协商，裸发/错 major 的请求曾被静默应答）：缺失或 major
  不符 → `error/bad_request`。hello 自身仍走 §2 协商应答（`accept:false` 更富）。
- `hello` 应答自 0.2.0 起带 `capabilities`（当前 `["hello","fourclass/2","graph/1",
  "clone/1","docdup/1","verdict/1","scan/1","structure/1","trend/2","erase/1","audit/1"]`；fourclass/2 =
  2.0.0 的锚宽请求形状——旧客户端探 /1 得缺席，响亮降级 L1 而非发不可解析的二元形状；
  graph/1 = M5-2 图族；clone/docdup/verdict = M5-3 三族，2.2.0 同批声明；scan/1 =
  ADR-008 P3 分级判决族，2.7.0 声明；structure/1 = M6 结构族，2.9.0 声明；
  trend/2 = M7.5b 趋势族，2.13.0 以 trend/1 声明、2.31.0 随 Theil-Sen 行为变化升 /2；erase/1 = M9 批 3 擦除谓词族，2.16.0
  声明）——**纯信息发现**，接受/拒绝的唯一权威仍是
  §2 的 SemVer；能力缺席 = 客户端走 L1 并显式降级（A9f）。
- 客户端规则：应答 `type` 非预期或 `id` 不回显 = 失步 → 视为 L2 不可用，
  回退 L1 且降级可见——绝不给错答案，只给响亮的答案。
- `fourclass.request`（2.0.0 形状）：`{"id","pairs":[{"i","rem":[[[行,hash,宽],…],…],
  "add":[…],"dup":[keyhash]}]}`——rem/add 为 L1 判 novel/deleted 的**显著**行按
  **run 分组**（run 结构=对齐产物，Rust 侧产出），hash = fnv1a(trim)，宽 =
  trim 后 alnum 计数（行事实，Cost.anchorFloor 的判定输入）；`dup` =
  after 侧新出现重复的**顶层具名单元**键哈希（堆叠证据，符号知识留在 Rust，
  仅哈希过线——ADR-002 A6）；`i` 为**不透明的文件对键**：批内唯一、由客户端选定，
  接收方只拿它当 Map/Set 键，**绝不按它下标回查**（`CE/FourClass/Wire.hs:31` 的
  pIdx 自述 "an opaque pair index"；重复 `i` 由 `CE.FourClass.violation` 判
  `error/contract`——Anchor 的 (pair,run) 图会静默丢掉重复者的 run；跨匹配要求
  `i` 不同）。批内合法地**稀疏**——发送方在滤掉空对**之前**取下标
  （`cli/src/fourclass/batch.rs:194-204`，enumerate 先于 filter）；旧文"稠密 0 基
  文件对位置"两侧实现从未成立，2026-08-20 就地更正（纯勘误，wire 字节不变）。
  within-first 前置（同对 add∩rem 必空）由 core 在边界校验，违反 → `error/contract`。
- `fourclass.result`：`{"id","moved":[[i,出行,入行]],"blocks":[[源i,源行,宿i,宿行]],
  "suspicions":[[i,规则名]],"degraded"(,"reason"∈{bucket_cap})}`——moved 为单调
  重分类 delta；blocks 为 ≥2 行站点证据（扩展/归因行只进 moved 不进 blocks）；
  suspicions 为 M4 判定规则点火记录（堆叠常数在 CE.FourClass.Verdict）。
- `graph.request`（2.1.0 起）：`{"id","nodes":[[lang,kind,roles]],"edges":
  [[src,dst,kind,rung]],"pos":[idx],"unres":[[lang,unresolved,total]],
  "symbols":[[node,visibility]],"unmentioned":[[node,vis,conv]],
  "mounts":[[node,private,total,bits]],"sccFloor":u64}`——稠密 0 基索引即
  节点身份，**无文本形物过线**（ADR-002 A6；6.2.0 的两张顾问表同律——候选名 `AdvisoryName`
  留在 Rust 侧，过线的只有整数）；节点行**三元组、单一合法元数**（5.0.0 起：
  pre-2.28 的 flags 列裁除，宽窄不对的行按**行下标**报 `node i: malformed row (need
  [lang,kind,roles])`；表级 `node rows: mixed arity` 随之退役）、边严格升序且去重、端点与
  pos 越界 → `error/contract`（边界契约由 core 机检）；`symbols` 是 4.1.0 起的可选导出面
  表——去重的 (节点, 可见性) 对、严格升序，核按 `Cost.exportVisBit` 读出导出节点并按
  `Cost.publicFlagBit` 或上 flags 位 0，判决码 2/4 由此首次可达；缺席或空表 = 字节不变。
  `unmentioned`/`mounts` 是 6.2.0 起的可选**顾问两表，同生同死**（只发其一 ⇒ `error/contract`
  具名配对拒绝，占校验 asum 最前）：`unmentioned` 按 `id` 投影严格升序、每行 `[node, vis, conv]`；
  `mounts` 全节点恒一行、`take 1` 投影升序、`private ≤ total`、bits bit 0 再导出目标 / bit 1
  包私有；两表各自析取项计价（`mountCap` 131072 / `unmentionedHardCap` 524288），节点净空不动；
  缺席 = 十键回复字节不变、dead 集不变（K16/K33）。`sccFloor` 是 6.4.0 起的可选环底（与 `verdict` 的 `cycleFloor` 同读一份 `[graph] scc_floor`；≥1 否则按名拒绝，上过线即在 `graph.result` 回显）。
  `unres` 是 2.32.0 起的可选按语言站点
  台账，是**判决输入**：在场时每条 dead 行增置信列（`CE.Graph.Cost.confidence`），缺席 = 旧
  两列 dead 行、字节不变；总数 `unresolved_sites` 仍只进 Rust 侧报告与摘要行（请求体见
  `cli/src/graph/deadcode.rs` `GraphWire`，核侧 `core/app/CE/Graph/Contract.hs` `GraphReq`——4.1.0 符号表落地时随解码与边界校验自 `CE.Graph` 拆出）；
  超 `CE.Graph.Cost` 节点/边护栏 → `graph.result` 带 `degraded:true,
  "reason":"graph_too_large"`（绝不截断）。
- `graph.result`（语义 M5-2g 落地，穷举参照 harness 见 core/test/）：
  `{"id","dead":[[idx,verdict]],"reported":[[idx,verdict]],"fail",
  "pos":[[idx,indeg,outdeg,sccId,sccSize,reachIn]],"cycles":[[sccId,[idx]]],
  "counts":{"nodes","edges","kept"},"degraded"(,"reason"∈{graph_too_large})}`——
  `dead` 只承载文件粒度判决，`reported` 承载 package/section 聚合判决；非降级时
  `fail` 当且仅当 `dead` 非空，降级应答恒 true。两表 verdict ∈ {1 unref_private,
  2 unref_public, 3 unreach_private, 4 unreach_public}（入度×可达两轴 + 公私隔离）；
  判定旋钮全在 `CE.Graph.Cost`：`minRung`(=5，边计为引用的 rung 上限)、
  `entryMask`(=126，flags 位 1-6 为入口根；位 0 exported 有意不入——公私是判决轴
  不是活性声明)、`sccFloor`(=2，环报告的最小 SCC)；kept = 去重后被判定采用的边数。
- **M5-3 三族（2.2.0 同批声明，桩期对一切输入回 `error/contract`；契约形状 =
  设计定稿卷一 §2.2，各族判决批落地时在此就地实体化并重生成 golden）**：
  - `clone/1`（判决落 T3 批）：request 携后序树
    `{"trees":[{"lab":[Int],"lld":[Int]}],"pairs":[[i,j]]}`（`lld[i]` = 最左叶后代
    后序下标，`0 ≤ lld[i] ≤ i` + 后序可重建性机检；pairs 严格升序去重、端点在界内）；
    result 回原始 `ted` 与规模不回比值，2.5.0 起并回加性 `verdicts` 布尔数组
    （每 score 行一位，`CE.Clone.Cost.cloneDecides` 的输出——上报集的唯一权威）；
    `degraded.reason ∈ {clone_too_large}`。
  - `docdup/1`（判决落 docdup 批）：request 携**升序去重 shingle 哈希集**
    `{"sets":[[u64]],"pairs":[[i,j,verbatimRun]]}`（集合非序列——token 流不跨进程，
    ADR-002 A6；逐字 run 在 Rust 算好只过整数）；result 回 `[i,j,inter,union]`，
    2.5.0 起并回加性 `verdicts` 数组（`CE.Docdup.Cost.dupVerdict` = Jaccard 半
    ∨ verbatim 半的全析取——run 过线正是为让 core 持有全部判决输入）；
    `degraded.reason ∈ {docdup_too_large}`。
  - `verdict/1`（判决落 score 批）：request 携三信号事实表 + `baseline` 原样字节
    （Rust 不解释，ADR-008 反抢跑），2.6.0 起并可携加性 `dedup`
    `[blocks,budget]` 对（第二棘轮判决输入，`ce dedup --check` 专用），6.1.0 起并可携
    加性 `symbols`:`[[u,visibility]]`——**与 graph/1 同一张导出面表**，只改按 tier 宇宙
    下标；核按 `Graph.Cost.exportVisBit` 读出导出集并给 `Pos.pFlags` 置 `publicFlagBit`，
    join 格的 `publicGuard`（RG10）自此在生产态可点火；缺席或空表 = 字节不变；
    result 回判决四码 + `reasonBits`/`legsMask` 自陈 + 棘轮集合 delta，
    2.8.0 起并回生效 `weights` 表与 `ratchet.failed` 持名条件表；
    5.1.0 起 request 并可携标量指纹与 `classKnobs` 码 3（该类自己的棘轮容差，行数绝对值）；
    6.0.0 起该标量名 `knobsDigest`、覆盖**整份解析后的配置**（O39 起为其规范化有效旋钮集），`ratchet.failed` 的持名条件为
    `knobs_digest`，`newBaseline` 在指纹到场时加同名键（**缺席而非 null**——出厂默认配置的仓字节恒等）；
    `degraded.reason ∈ {verdict_too_large}`。
  - `scan/1`（2.7.0，判决与声明同批）：request 携测量行 `{"rows":[[code,value]],
    "naming":[[lang,style,upper,under,test]]}`（码 0..6，主体名/路径不过线；naming 自 2.30.0
    由 ce 恒发、与码 6 行逐位对齐，core 容其缺席）+ 可选 `grades` 覆盖 `[[code,warn,fail]]`
    + 规则包两键（3.2.0）：`rowClasses`（与 rows 逐位对齐的 classId）与 `gradeOverrides`
    `[[classId,code,warn,fail]]`（码 ∈ {0,1,4}，回复原样回显）
    （fail 0=无硬线、fail==warn=合法单线配置、码严格升序）+ 围栏键 `knobsFence`（6.4.0，ce 恒发：`null` = 无基线未围、`[current,recorded]` 两摘要各 u64 或 null）；result 回
    `{"levels":[0|1|2 逐行],"counts":{rows,warns,fails},"fail",生效 "grades" 全表,"failed":[名…]}`（`failed` 具名序 `hard_line, knobs_digest, degraded`，`knobsFence` 上过线即在；`fail ⇔ failed ≠ []`）；
    `degraded.reason ∈ {scan_too_large}` 且自带 fail=true。

## 2. SemVer 协商规则

- **major 不同 = 拒绝**：应答 `accept:false` + `reason`，调用方报错退出。
- minor/patch 不同 = 接受（新字段走"忽略未知字段"规则）。
- **schema 不兼容变更**（删字段/改字段形状）必须 bump major，并同步更新两侧实现 +
  fixtures；major 不同按上条拒绝。
- **分数语义迁移**（轴语义/阈值/量纲）可随 minor，但 release notes 必须声明
  score migration。
- **信封常数变更**（行字节预检、错误码/reason 词汇扩充）：放宽 = minor（旧客户端
  照常工作），收紧 = major；变更必须在 §1 就地改写并注明日期与依据（2.1.0 的
  32 MiB 放宽为首例）。

## 3. Fixtures 约定

- `fixtures/handshake/`：wire golden（请求行 + 期望应答行交替；`hello-ok` 握手、
  `wire-errors` 错误应答），Rust（`cli/tests/it/core_wire.rs`）与 Haskell
  （`core/test/Spec.hs`）**逐字节**共同消费——同一份文件，防两侧实现漂移。
  字节比较可靠因为 freeze 钉 `aeson +ordered-keymap`（键序确定）。
- **request 行的 proto 有意滞留（2.2.0 立场声明，M5-3a；每次 major 重锚）**：2.2.0 翻批只重写
  reply 行、request 行留在 2.1.0；此后每次 major 都把全部 request 行随之机器重写
  （3.0.0 / 4.0.0 / 5.0.0 / 6.0.0 各一次），minor 之间有意滞留——今日锚在 **6.0.0**
  （105 行，server 恒答 6.4.0）——它们是"minor 偏斜
  必须被接受"（§2：minor/patch 不同 = 接受）的**常设回归 fixture**。后人把
  request 行"修"成与 server 同版 = 删除该回归覆盖，禁止；新增 fixture 的
  request 沿用当前 major 锚（今日 6.0.0；唯 `handshake/hello-ok` 的握手 request 随
  server 走 6.4.0）。这组「行数/锚/答版」三元组是手写值，每逢 major 必须复核。
- `fixtures/hook-payloads/`：Claude Code `PreToolUse(Edit|Write)` 的**实测** stdin
  dump（官方文档无逐字示例，ADR-007 ⚠️ 项）。采集方式见该目录 README。
- fixture 变更 = 契约变更，走 §2 规则。

## 4. 工具链锁定（M0 验收项）

| 组件 | 锁定 | 载体 |
|---|---|---|
| Rust | 1.94.1 | `rust-toolchain.toml`（仓库根） |
| GHC | 9.14.1（LTS） | CI `ghc-version` + 本文件 |
| 依赖快照 | cabal freeze | `core/cabal.project.freeze`（378fe40 入库，2026-08-07；升级依赖时 `cabal freeze` 重生成） |
| 协议 | 6.4.0 | §1 所列两处常量 |
| daemon 协议 | 2.0.0 | [DAEMON.md](DAEMON.md) + `cli/src/daemon/proto.rs::DAEMON_PROTO`（形状 golden：`fixtures/daemon/`；反引号拼写无入边——dogfood deadcode 门在 CI 首点火即抓获，链接语法即活化） |
