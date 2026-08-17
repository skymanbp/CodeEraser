# M5 收口与挂账清零登记册（二次拆册，2026-08-17）

> [EVAL-SET-M5-3.md](EVAL-SET-M5-3.md) 在挂账清零批节落册时越过 300 行
> E01 线——按决策⑨(b) 拆册先例再拆：本册收 **M5-3 收官之后**的批次冻结
> 登记（M5 收口欠账清算、3m recall 仪器 B、审查热修+CI 门补全、ADR-008
> 首步、挂账清零批）。已冻结记录原文迁移不压缩（provenance 代价，决策⑨）。
> 数字直读 `contracts/eval/*.json` 冻结件；重放门 = CI `cargo test`。

## M5 收口（欠账清算，2026-08-14）

**①Go arity**（3h 盲审缺陷）：`param_count` parameters 字段优先（五语言字段
实探：Go/Rust/Py/TS 字段=kind 扫描同节点、Haskell 无字段走回退——冲击面恰限
Go method）；GRAPH_REV 3→4 + STRUCT_REV 1→2（unit key 内嵌 params，两侧缓存
陈旧）；钉 = `fourclass::units` 六 key 表（`(T) mix/2`/`(T) grouped/1`/
`(T) none/0`）；T3 冻结族按 epoch 档立（见 T3 节，用户拍板保审计）；churn
台账零 .go 行实测，活体重放门零冲击。
**②md 节陈旧**（2f 既档）：目标 slug 集哈希入 resolve_key（`md::slug_hash`
单咽喉；anchor() 是唯一跨文件内容读）；钉 = `graph_wire::
target_heading_edit_refreshes_the_source_anchor` 三轮表驱动，反事实（副本法
断 key 折入）实证红。
**③core Haskell 六警清零**（拆分非豁免）：Protocol dispatch 表驱动（五同形
case 臂 → families 表 + familyReply，顺带退休两自对块）；Wire violation
行检器提顶层；Verdict result 拆 candidates 装配；Score penalties 每轴一具名
谓词（正合"每谓词一旋钮"）；Clone reply 计数捆绑 (judged, prefiltered)；
Reference refBlocks 抽 blockOk 五子句谓词（定义誊写保真）——Spec 电池全绿
护航。**预算 149→148 真下棘**（Protocol −2 落袋；本批自增 +5 当场表驱动
偿清）；警台账 18→13。
**④立场档（带界收口，不清而档）**：R6 调用边/RG10 公共位 = 计划内条件项
（M5-2 行：独立 100 调用点审计 ≥90% 方开），M5 以 import-绑定层收口、RG10
对 file-tier 休眠为既定立场；T3 改编与短单元（<floor_nodes 24）= 设计卷
已档域界，仪器地板如实发布；aeson 类 store 安装依赖 = hs.rs 头部 stated
boundary；py-tree-sitter 0.26 结点存活期不安全 = 3h 盲审外部工具注记，
本仓零依赖。

## T3 recall 仪器 B（3m，M5 收口补齐——3a–3g 批缝漏建经验收复核挖出）

**对照物**（provenance 全入档）：similarity-ts 0.5.0（TS，阈值 0.87@main.rs:22，
`-e ts --no-types`=对齐冻结 canonical-extension 域+函数域）、similarity-py 0.5.0
（Py，0.85@main.rs:22）、similarity-generic 0.5.0（Go，0.85@main.rs:31；
**Windows 目录行走缺陷实证（os error 5）→逐文件调用=仅文件内对、名基匹配**）；
ripgrep 排除（similarity-rs="(future)" 官方 --supported 输出）、self 排除
（覆盖语言面全为 crosscheck 夹具=产品排除域）。分母=对照物默认参数全检出集，
**永不缩减**；记功=ce 全层（T1/T2 块或 T3 clone 判决双侧 span 重叠 ≥1 行）。

**根修 = S5 全对候选源**（`candidates::extend_exhaustive`，产品 `ce clone` 专用
——collect() 四源冻结面零扰动，冻结族 digest 门照绿）：同语言按节点数排序，
尺寸窗=§4.3 尺寸剪枝同谓词于生成时执行，标签剪枝照跑；wire 升序契约由终排序
恢复（首跑 desync 教训）。修前候选盲区=硬上限（requests 128 候选对 vs 分母
425、cobra 1,124 vs 9,205）；修后 not_candidate 桶**清零**。

**冻结 = `t3-recall-{zod,requests,cobra}-v1.json`**（`ce.eval-t3-recall/1.0.0`；
门 `eval_t3_recall` 信封重放+封闭词表+回归地板+覆盖清单封闭）：recall_raw
zod **3/6=0.50** / requests **67/425=0.158** / cobra **1417/9205=0.154**；
recall_incremental 0.0 / 0.058 / 0.083（触发器 <0.50 书面处置=本节即处置：
增量低因 T3 域与对照物测度轴不同，见下）。**miss 100% 机械归因且全部定义性**：
size_bound_not_clone 1/135/4453（ce 注册 TSED 下 best-case sim=min/max<0.85
=数学不可能）+below_floor 0/0/2578（注册短单元域界 T3_MIN_NODES=24）+
judged_not_clone 2/223/757（真送 TED、按 θ=85/100 拒）。**结论=测度分歧非
盲区**：mizchi similarity 轴≠ce TSED 轴，0.90 字面门对该对照物可证不可达
→计划 v1.6 修正案（用户拍板 2026-08-14）：门改只升不降回归地板。
PERF：S5 后 `ce clone` 冷 requests 1.8s / cobra 3.6s / zod 47.1s
（19,193→~40k 判决对量级，pre-S5 zod 24.9s@3e——全对候选的代价如实入册）。

## 审查热修批（M5 收口审计响应，2026-08-14）

HIGH-1：候选路径只读化——`walkidx::read_streams`（鲜读+不写；缺流/陈旧偏移
是 `extend_anchor` 既有守卫案），`candidates::collect` 收 `&Index`，S1 的
load_streams 写路径退役（评审实证：中途保存的文件其级联删边被静默孤儿化）。
HIGH-2：`Verdict/Wire.hs` tierOf 线性扫（O(F²)）→ 懒 IntSet（O(F)；tier 稠密
性由 asum 首元先证）。MED：sim 行域检查（kind>2 拒 "unknown sim kind"、
den=0 拒 "zero denominator"，VerdictProps +2 具名探针）；`idx_edge_site`
索引 + SCHEMA_VERSION 6（唯一无 FK 子键索引的级联子表）；rel_str 咽喉收拢
（walkidx::rel_of 删、daemon 尾拼写换 throat 调用——双审查员收敛项）；
CloneProps +prefilter 族性质（shipped `provablyBelow` ⇒ 真 ted 非克隆，
补上转写零执行缺口）；Go receiver 限定上移抽取根 `functions::name_of`
（fourclass 后置 qualify 删除——D4 的 baseline 键重拼按构造消亡；键值字面
不变由 units 电池逐 key 证同，rev 零 bump）。
**HIGH-3 = 计划 v1.7 修正案（用户拍板：根治不偷懒）**：「daemon 唯一写者」
（审计实证从未成立）改写为**收敛式多写者缓存**契约——写路径全内容门控+
幂等+IMMEDIATE 锁内自检，WAL 逐事务串行 ⇒ 并发写者对静止树收敛于串行序
终态；HIGH-1 恰移除了最后一个非幂等写者（候选路径），契约由此可证。
验收件=`concurrent_writers` 双电池：双进程同库 dedup 收敛于串行 digest、
daemon 冷启动 vs 外部写者收敛（coldstart 竞态注记转为按构造良性）；
M6 GUI 直写同库自此有据（风险 R1 解除）。

**CI 门补全批（审计 D5/D6/D7/D8/D9 响应）**：①`ce check --fail-under 800`
入双平台 dogfood（floor 腿活化；800=实测 866–872 带下 66‰ 塌方地板，
决定值非推导值）；churn 腿（--days 14，axis 5 活=实测 2 hit）实测 215.8s
⇒ 仅 ubuntu 一腿承担（成本有界诚实覆盖非全平台结构性死亡）。
②`ce deadcode --check`/`ce docdup --check` 新旗入 CI（emit_checked 单咽喉
=dedup --check 同形；deadcode_e2e 红绿双向钉：孤儿必红、entry_globs 处置
必绿）——M5-2「全处置」与 §7.5 docdup 条款自此代码执行非纪律执行。
③`regen_tables` 三 --ignored 漂移检测（D8 生成器回仓）：go STD/py STDLIB/
hs_boot 各按其记录管线对工具链重导出集合比对——**首跑即抓获自身滤网语义
缺陷**（Go internal 规则=任一路径段，`log/internal` 逃过前缀检查而冻结表
本身正确），修后 3/3 全表零漂移证毕。④D7 根修=删 README 版本抄本立单源
（hookio.rs::OBSERVE_SCHEMA 唯一权威）。

## ADR-008 首步（300/15 入 wire，2026-08-14）

**镜像退役**（M5 收口审计 D2 最后一对无检镜像）：`verdict.request` 加性
`ceilings` 表（`[[axis,ceiling]]`，axis 0=size/1=coc；缺席=空=Cost.hs
**默认值**——proto 2.3.0 加性 minor，request 行偏斜 fixture 纪律照旧）；
Rust 侧 `score::run` 经 `Config::load` 单咽喉发
`[[0,file_lines_warn],[1,cognitive_warn]]`，应答 `knobs` 回显生效值、
`wire::judge` 断言往返。**漂移门 = `core_wire::ceilings_default_drift_gate`**：
空表回显 == `Thresholds::default()`（300/15）——Cost.hs 默认与 ce.toml
默认在同一条断言相会，任一侧独动即红（此前两常量互为无检镜像）。
Haskell 权威 = `ceilingsOffence` 域检（axis>1/值<1/降序拒绝，VerdictProps
+3 具名探针）+ `effectiveKnobs`（scoreBound 覆盖式）+ `ceilingKnob` 性质
（真 respond 驱动：310 尺寸行默认 300 下受罚、请求 400 下净，回显双态钉）。
golden 翻批 41 reply 行逐行审（40 = proto 位 + verdict 两档默认 knobs；
新 pair 6 = ceilings [[0,400],[1,20]] 生效+回显）。**棘轮咬偿**：ceilings
检器克隆 weights 脚手架当场被抓（149>148）→ `knobTable` 单文法咽喉
（两表差异降为数据：axis 界/拒绝文/值判），148==148 净、零豁免。

## 挂账清零批（M6 前清零，2026-08-14 → 2026-08-17）

**账本**：M5 收口审计（artifact 4548131c）残留 MED×2 + LOW 开放项 + R2
契约册 + RG10 条件项 + 五批审查欠账，逐项清偿至零。六提交链
68cf2eb→ef2ce91→06ef3e1→9cda405→67f4f68(+f1ed34d 活化修)→a049275。

**收敛热修（68cf2eb）**：夜跑 31991997431 抓获 v1.7 契约真破口——相 1.5
裸 INSERT 的防重护栏是进程内知识，跨进程交错（B 刷文件→A sweep 盖到 B 的
新 site 并落 key→B 跳 sweep 后相 1.5 重插）双插边。根修=同锁内
delete-then-insert；确定性电池=CI 交错投影单连接（`phase_15_is_idempotent_
over_a_swept_file` 修前红 2≠1；本机 40 环进程竞态 0 命中=Windows 调度打不中
窗口，ubuntu 打中）。store.rs 297 行按 E01 拆 keys.rs。

**C1（ef2ce91+06ef3e1）**：verdict baseline 单跑+行数入 cap；graph pos 严格
升序=应答按契约有界；fourclass 重复 pIdx 具名拒绝；信封解码失败仍回显在场
id；Main 异常屏障；axisCodes 死导出删；siteOpens 唯一乘法点 Integer 化。
golden 全字节稳定；**预算真下棘 148→145**（三跨模块块退休）。

**C2（9cda405）**：sweep memo=Scope 携每 sweep 键控 any-cache（members()
咽喉+rs 目录级 Cargo/crate-root+每文件 tree-sitter+go importable+hs cabal+
md slug/ref 表+ts walk-up 与 node_modules）；PERF release 冷 sweep：ripgrep
3.33→2.83s、self 2.11→1.92s（渐近类 O(sites×files)→O(dirs×files+sites)，
冷跑主导在索引侧如实入册）。LOW 四件：节点铸造改读存储 granularity（资产/
悬空 ref 不再铸 package）；cabal common 死区；`.markdown` 单谓词
`md::is_md_path`；走树删文件经 `each_surviving` 降级；t3/tree 三递归换显式
栈（lld=进入时 lab 长可证=最左叶后序位）。

**C3（67f4f68+f1ed34d）**：daemon 契约册 contracts/DAEMON.md+形状 golden
fixtures/daemon/+daemon_proto 重放门（tag 函数无通配臂=新变体编译断裂）；
server.rs sole-writer 陈旧注改写；VERSIONING §4 daemon 行。CI 首点火即被
自家 deadcode 门抓获（DAEMON.md 反引号拼写零入边）→真链接活化 f1ed34d
（3l 先例；教训=本地门链必须镜像全部 CI dogfood 腿）。

**RG10 条件项收案（永闭）**：计划 M5-2 条件（独立 100 调用点审计 ≥90% 方开
R6 全仓同名匹配）实测=5 语料×20 确定性采样+3 镜对抗复审：行为正确率
**72/100**（true_unique 38+correct_absent 34；false_ambig 23、false_wrong
5）、铸边精度 **38/66=0.576**；四条复审异议全按有利翻正仍 ≤74。按计划自身
判据**永久关闭**；证据冻结 `contracts/eval/rg10-callsite-audit-v1.json`。

**全量独立审查（ba1c0f5..HEAD，60 代理六维×双怀疑者）**：27 项判决=15
confirmed+6 contested+6 refuted；**C4（a049275）全数偿清**：HIGH=
`remove_missing` DEFERRED 读改写（v1.7 漏掉的最后一条，WAL 下
BUSY_SNAPSHOT 不可重试）→IMMEDIATE+变异树竞态腿；deadcode --check 与
ce check 的 degraded 放行双洞→双拒（"不能判决的门绝不放行"扫至最后两个
漏站）；C2 自造 cabal 回归（列 0 注释杀 live 位+common deps 消失）→注释
免杀+deps 全档并集+双反事实探针；`internal` 入 §1 闭集+Spec 钉；corelink
PROTO 假焊称→core_wire 真焊；Main 异步重抛；daemon 非 UTF-8 行活连接；
TS 双 fs 事实（js 孪生+node_modules）入 resolve_key（md slug 先例）；
coldstart 完整性=meta full_build 正事实（行数≠完整）；recall 仪器四修
（反真空改真表求和/分母只增断言〔contested-HIGH〕/双比率重放〔首点火即钉
4 位舍入惯例〕/method 词表修正案入冻结档带日期）；daemon_proto 数钉+
DAEMON.md 载荷覆盖边界；rg10 档 40 个 corpus 域按位修复（prose 存
corpus_notes）。契约立场档一条：sweep 持 IMMEDIATE 锁跨 resolver I/O
（contested，memo 已使 sweep 亚秒级+busy_timeout 5s 容），不改码入册。
工具再咬作者四次全偿（recall 测试第二次撞 75 硬线、race 孪生、Spec 检查梯、
score run 超警）；预算 145==145 零豁免。

**边界确认**：ADR-008 规则 DSL=计划内 M6 并行轨非欠账；R4/R5=M7 验收行
自身内容；R1 由 v1.7 解除、R2 由 C3 清、R6 由 regen_tables 已清、R7 由
CI 门批已清。**挂账清零终态：账本空。**

## ADR-008 集中收口批（计划 v1.8，2026-08-17 起）

**章程（6159bde）**：三拍板=①判决/测量分界（判决·豁免判定·预算·棘轮·阻断
入 core；测量参数留 Rust 单点声明+回显钉；判据=源文本/行级内容过 wire 即
测量侧；guard 热路径与 hook 协议映射两判例）②DSL 形态=位台账+判决表
③四片全收（P4→P1→P2→P3）。两路独立普查冻结+17 项 Rust 独立语义逐项归边
（6 迁 9 留 2 判例）：[reviews/2026-08-17-adr-008-policy-dsl.md](reviews/2026-08-17-adr-008-policy-dsl.md)。

**P4 配置面与表化**：proto 2.4.0 加性 minor——`thresholds`（codes 0..6）+
`tolerance`（legs 0..2）两表入 wire（`knobTable` 文法泛化：judgeV 收 code
参、逐码域判）；应答 `knobs` 回显扩为 12 键全量生效集；weights 通道打通
（ce.toml `[score.weights]` 按轴名驱动，Rust 恒空数组退役）；`Config`
全节 `deny_unknown_fields`（错拼策略键静默丢弃陷阱修）；码表单一权威=
`score/knobs.rs`（code=下标的名单表）。**隐式策略表化**：Join 判决优先级
落 `verdictTable`（(code,必需位,禁止位) 有序行，delete 行=旧 deleteReady
的位等价——至多一死翼引理；`judgeWith` 参数化+JoinProps 重排电池=转表
必红，堵普查缺口 3）；graph 四路码落 `deadTable`（(public,referenced)→code
总查表，暴力参照即其重排反事实）；fail 合取落具名条件表；Score 1000/`max 0`
入 `scoreScale` knob（缺口 1 清）；`effectiveJoin` 直读 score 生效 rewrite
比（一权威两读者）。golden 翻批=proto 位全册+verdict 回显扩展+新 pair 7
（scoreScale 500→499 分、tolAbs 0→310>306 over 触发 fail）逐行审；
VerdictProps 拆 VerdictWireProps（300 线拆分非豁免）。**棘轮第八咬十块
全偿**（四 setter 姊妹表→双表+双直构、knobs 双表→下标名单、Request 字段
带→KnobTable 别名、body json 行群→四表单循环、import/电池脚手架→
qualified+表驱动）；145==145 零豁免。判决字节等价=判决字段（score/axes/
candidates/ratchet）逐字节不变于 golden 全册。

**P1 判决权回迁**：proto 2.5.0 加性 minor——三处判决权入 core：①t3
`is_clone` → `CE.Clone.Cost.cloneDecides`（clone.result 加性 `verdicts`
布尔数组，每 score 行一位同序；上报集自此由 core 的位构建）②docdup
`is_dup` 全析取 → `CE.Docdup.Cost.dupVerdict`（Jaccard 半 ∨ verbatim 半；
`verbatimFloor=50` 判决权迁入 core 并入 knobs 回显——run 长度过线〔F26〕
文本不过〔§5.9.2〕正是为此）③degraded verdict 回复自带 `ratchet.fail=true`
（"不能判者绝不放行"由 core 自述；`main_score` 的 `|| degraded` 再解释
退役——语义位翻转仅此一处=P1 契约本体，golden 无 degraded 对、由
VerdictWireProps `degradedFails` 探针钉住〔verdictNodeCap+1 行 tier 真过
respond〕）。Rust 侧=`lockstep::verdict_bits` 单咽喉（长度锁：位数≠行数即
拒）+两家 `parse_result` 拉链；`is_clone`/`is_dup` 降为**镜像**（3f 冻结
仪器语义零扰动继续走本地绑定），产品 run() 逐行 ensure 位==镜像——公式漂移
指名即死，仪器与产品由证明而非信任保持相等。**反事实杠杆**：CloneProps
边界（ted 15/16@max100，86/100 翻）+ ReferenceJaccard verbatim 半
（run 50/49，地板 51 翻）+ degradedFails 全绿。golden 翻批 43 行逐审=
41 行纯 proto 位+加性字段（判决字段逐字节不变）+docdup 新 pair 6 三形
判决 `[true,false,true]`（verbatim 独断/近界拒/Jaccard 恰在 80/100）与
手算逐格吻合。**棘轮第九咬**：Docdup/Cost 增 knob 使两家 Cost 模块的
声明梯同形越 50-token 地板（distinct 8）——跨家抽取被 Verdict/Cost 自家
立场明文禁止（one authority per family），按史例改偿最近可偿形：JoinProps
三行 delete 同形 fixture 表驱动化为 `deadFlankRow flags` 单构造子，
145==145 零豁免。

## 复跑

本册各节的重放命令与母册一致（见 [EVAL-SET-M5-3.md](EVAL-SET-M5-3.md) 复跑节）；另加 `cargo test --test daemon_proto --test concurrent_writers` （清零批新门）。
