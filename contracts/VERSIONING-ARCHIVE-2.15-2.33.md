# contracts/ — 契约版本化机制 归档册（proto 2.15.0–2.33.0）

> 2.15.0–2.33.0 十九条版本条目，即 [VERSIONING.md](VERSIONING.md) 倒序段最老的一截（更早的 2.1.0–2.14.0 在 [VERSIONING-ARCHIVE-2.1-2.14.md](VERSIONING-ARCHIVE-2.1-2.14.md)，2.0.0 一条仍在正册的导语段里）。
> 2026-10-01 从正册迁出：7.9.0 的条目落下后正册读 745 行，离 `ce scan` 的 750 行硬线只剩 5 行，下一条（计划 v2.32 步 5）必过线，
> 按仓规「拆分优先于豁免」与 2.1.0–2.14.0 那一册同法另起一册。条目逐字节搬家，未改写，仍按版本倒序；信封、SemVer 协商规则与
> fixtures 约定只由正册声明，本册与正册同一立场（不冻结：条目所述的规则若改，仍在条目上就地改写）。

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
