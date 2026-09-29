# contracts/ — 契约版本化机制 归档册（proto 2.1.0–2.14.0）

> 2.1.0–2.14.0 十四条版本条目，即 [VERSIONING.md](VERSIONING.md) 最初的顺序段（2.0.0 一条仍在正册的导语段里）。
> 2026-09-29 从正册迁出：步 7b ③ 的条目落下后正册读 765 行，过了 `ce scan` 的 750 行硬线（计划 v2.30 步 7b），
> 按仓规「拆分优先于豁免」另起一册——CHANGELOG 六次拆归档的同一做法。条目逐字节搬家，未改写；信封、SemVer 协商
> 规则与 fixtures 约定只由正册声明，本册与正册同一立场（不冻结：条目所述的规则若改，仍在条目上就地改写）。

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
