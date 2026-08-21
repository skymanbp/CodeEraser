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
> **2.18.0**（M9 批 7 片 4 RG9 回迁，2026-08-21）：`graph.result` 判决
> 分流入核——`dead` 表只承载**文件粒度**行（判红表，亦是 erase class-0
> 授权源），加性新表 `reported=[[i,verdict]]` 承载 package/section 判决
> （RG9：聚合不是代码实体，只报告不判死）；加性 `fail` 位为零容忍门具名
> （任一文件级死判决即 fail；降级应答自带 fail=true——verdict 族 P1 立场）。
> kind 列一直上 wire 且被校验，此前判后即弃——分流以 Rust 无名分支存在，
> 消融不可见。客户端保留分流为边界契约：判红表混入聚合=按 wire skew
> 具名拒绝，绝不授权目录擦除；对 2.18 前旧核，缺位 fail 由客户端按旧
> 合取自算顶替（字节等价回退）。
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
> 路径永不上 wire——Rust 按序回贴标签），class 冻结位 0=dead_file
> （w=死判决 1..4，x=该文件语言的未解析站点数）、1=verbatim_doc
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

- `proto`：协议版本，当前 **2.18.0**（单一来源：`cli/src/corelink.rs::PROTO`
  与 `core/app/CE/Protocol.hs::proto`，两处必须一致，由共享 fixture 钉住）。
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
  "clone/1","docdup/1","verdict/1","scan/1","structure/1","trend/1","erase/1"]`；/2 =
  2.0.0 的锚宽请求形状——旧客户端探 /1 得缺席，响亮降级 L1 而非发不可解析的二元形状；
  graph/1 = M5-2 图族；clone/docdup/verdict = M5-3 三族，2.2.0 同批声明；scan/1 =
  ADR-008 P3 分级判决族，2.7.0 声明；structure/1 = M6 结构族，2.9.0 声明；
  trend/1 = M7.5b 趋势族，2.13.0 声明；erase/1 = M9 批 3 擦除谓词族，2.16.0
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
- `graph.request`（2.1.0 起）：`{"id","nodes":[[lang,kind,flags]],"edges":
  [[src,dst,kind,rung]],"pos":[idx]}`——稠密 0 基索引即节点身份，**无文本形物
  过线**（ADR-002 A6）；节点行三元组、边严格升序且去重、端点与 pos 越界 →
  `error/contract`（边界契约由 core 机检）；未解析站点计数**不过线、留在客户端**
  （`unresolved_sites` 只进 Rust 侧报告与摘要行，判决从不消费）——旧文曾列的
  `"unresolved":[[lang,kind,reason,count]]` 两侧都不发不解析：请求体 =
  `{nodes,edges,pos}`（`cli/src/graph/deadcode.rs:174-178`），`GraphReq` 无此字段
  （`core/app/CE/Graph.hs:33-46`），2026-08-20 就地删除，wire 字节不变；
  超 `CE.Graph.Cost` 节点/边护栏 → `graph.result` 带 `degraded:true,
  "reason":"graph_too_large"`（绝不截断）。
- `graph.result`（语义 M5-2g 落地，穷举参照 harness 见 core/test/）：
  `{"id","dead":[[idx,verdict]],"pos":[[idx,indeg,outdeg,sccId,sccSize,reachIn]],
  "cycles":[[sccId,[idx]]],"counts":{"nodes","edges","kept"},
  "degraded"(,"reason"∈{graph_too_large})}`——verdict ∈ {1 unref_private,
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
    `[blocks,budget]` 对（第二棘轮判决输入，`ce dedup --check` 专用）；
    result 回判决四码 + `reasonBits`/`legsMask` 自陈 + 棘轮集合 delta，
    2.8.0 起并回生效 `weights` 表与 `ratchet.failed` 持名条件表；
    `degraded.reason ∈ {verdict_too_large}`。
  - `scan/1`（2.7.0，判决与声明同批）：request 携测量行 `{"rows":[[code,value]]}`
    （码 0..6，主体名/路径不过线）+ 可选 `grades` 覆盖 `[[code,warn,fail]]`
    （fail 0=无硬线、fail==warn=合法单线配置、码严格升序）；result 回
    `{"levels":[0|1|2 逐行],"counts":{rows,warns,fails},"fail",生效 "grades" 全表}`；
    `degraded.reason ∈ {scan_too_large}` 且自带 fail=true。

## 2. SemVer 协商规则

- **major 不同 = 拒绝**：应答 `accept:false` + `reason`，调用方报错退出。
- minor/patch 不同 = 接受（新字段走"忽略未知字段"规则）。
- 破坏性变更（删字段/改语义）必须 bump major，并同步更新两侧实现 + fixtures。
- **信封常数变更**（行字节预检、错误码/reason 词汇扩充）：放宽 = minor（旧客户端
  照常工作），收紧 = major；变更必须在 §1 就地改写并注明日期与依据（2.1.0 的
  32 MiB 放宽为首例）。

## 3. Fixtures 约定

- `fixtures/handshake/`：wire golden（请求行 + 期望应答行交替；`hello-ok` 握手、
  `wire-errors` 错误应答），Rust（`cli/tests/core_wire.rs`）与 Haskell
  （`core/test/Spec.hs`）**逐字节**共同消费——同一份文件，防两侧实现漂移。
  字节比较可靠因为 freeze 钉 `aeson +ordered-keymap`（键序确定）。
- **request 行的 proto 有意滞留（2.2.0 立场声明，M5-3a）**：2.2.0 翻批只重写
  22 条 reply 行；既有 19 条 request 行**有意留在 2.1.0**——它们是"minor 偏斜
  必须被接受"（§2：minor/patch 不同 = 接受）的**常设回归 fixture**。后人把
  request 行"修"成与 server 同版 = 删除该回归覆盖，禁止；新增 fixture 的
  request 用当前 proto。
- `fixtures/hook-payloads/`：Claude Code `PreToolUse(Edit|Write)` 的**实测** stdin
  dump（官方文档无逐字示例，ADR-007 ⚠️ 项）。采集方式见该目录 README。
- fixture 变更 = 契约变更，走 §2 规则。

## 4. 工具链锁定（M0 验收项）

| 组件 | 锁定 | 载体 |
|---|---|---|
| Rust | 1.94.1 | `cli/rust-toolchain.toml` |
| GHC | 9.14.1（LTS） | CI `ghc-version` + 本文件 |
| 依赖快照 | cabal freeze | `core/cabal.project.freeze`（GHC 就绪后 `cabal freeze` 生成入库） |
| 协议 | 2.18.0 | §1 所列两处常量 |
| daemon 协议 | 1.1.0 | [DAEMON.md](DAEMON.md) + `cli/src/daemon/proto.rs::DAEMON_PROTO`（形状 golden：`fixtures/daemon/`；反引号拼写无入边——dogfood deadcode 门在 CI 首点火即抓获，链接语法即活化） |
