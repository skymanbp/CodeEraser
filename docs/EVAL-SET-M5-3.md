# M5-3 评估仪器册 v1（拆自 EVAL-SET.md，决策 ⑨(b)）

> 2026-08-14 拆册：母册 [EVAL-SET.md](EVAL-SET.md) 持 M0–M5-2 冻结记录与方法学，
> 本册收 M5-3 各仪器批的冻结登记。拆册而非压缩——已冻结记录不压缩（provenance
> 代价，决策 ⑨）。本册每个数字直读自 `contracts/eval/*.json` 冻结件；重放门 =
> CI `cargo test`（逐族测试名见各节）。母册 300 行 = E01 硬线（RM20，3j 起由
> `core_size_gate.rs` 的 ceiling 断言执行）；本册同受 E01 扫描。

## 语料树钉定（外部四语料 tip 全族一致；self 按生成批各钉）

| corpus | tip |
|---|---|
| requests | `8068356288978c4f54661ae6f95afe0e0831885e` |
| ripgrep | `3fce3b5bb0236da2df6d99672afb8a719642eca7` |
| cobra | `adbc8813901bba65827259daa8e22ff94ec1f30e` |
| zod | `912f0f51b0ced654d0069741e7160834dca742ee` |
| self（t3/docdup 系） | `60f73e3bea7681721a2f572e64788948a17830f6` |
| self（churn 台账，3h 收口树） | `473adfcea187f0f35e93dd3ac33e33a2bd16bb5c` |

## T3 仪器链（3b 宇宙 → 3c 候选/抽样 → 3f 精度）

**宇宙**（`t3-universe{,-requests,-ripgrep,-cobra,-zod}-v1.json`，
`ce.eval-t3-universe/1.0.0`，生成于 `c21d47f`；门 `eval_t3_universe`）：
units self 2,028（141 文件）/ requests 807（50）/ ripgrep 4,397（133）/
cobra 595（53）/ zod 7,078（393）；常数 floor_nodes 24、s_max 60、m_max 200。

**候选**（`t3-candidates-*-v1.json`，`ce.eval-t3-candidates/1.0.0`，生成于
`c6d2465`；门 `eval_t3_candidates`）：四源并集（S1 段二汇 / S2 同键 / S3 裸指纹
/ S4 MinHash-LSH (128,32,4)）+ 两道可容许剪枝与判决同界（85/100 整数交叉相乘）。
survivors self 490 / requests 128 / ripgrep 6,201 / cobra 1,158 / zod 21,740
（zod 剪枝实证：pruned_size 1,844,192 + pruned_label 264,236 → 21,740）。
`pairs_sha256` 五 digest = 抽样册 `pool_digests` 锚链；min_reported 地板
53 / 18 / 617 / 63 / 3,253。

**抽样**（`t3-sample-v1.json`，`ce.eval-t3-sample/1.0.0`；门 `eval_t3_sample`）：
main 100 + backup 20×4，配额 ts 44 / rs 24 / go 17 / py 15（最大余数 ∝ 池
21,750 / 6,676 / 1,161 / 130），哈希序无 RNG。

**精度 = 达标线 A**（`t3-precision-*-v1.json`，`ce.eval-t3-precision/1.0.0`，
生成于 `2c67ebe`；门 `eval_t3_precision`）：answered 61 全 correct、wrong 0 →
**总体 precision 1.000**（合同 ≥0.85）；分母 ≥5 语料逐个 1.000（requests 11 /
ripgrep 18 / cobra 12 / zod 19；self answered 1 不设门）。余量 not_clone 31
（zod boilerplate 16 全落 TSED<0.85 的机制性分离）+ dropped 8 = 100 全记账。
θ=85 为过合同最宽松格点（θ 全表 50..100 入档）；审计 = 五独立审计员 100/100
封闭六类 truth；T-G14 clones 地板 219 / 84 / 3,773 / 842 / 10,036。

**探测器 epoch 边界（M5 收口拍板，2026-08-14）**：本族五档整体冻结于 Go arity
修复前的探测器（3h 盲审缺陷：method receiver 计入元数，`(T) f/1` 塌缩；修复 =
`param_count` parameters 字段优先，仅 Go method 受影响）。一手拓扑核实：
sample 行内嵌 unit key 且抽样为 key 哈希序 → 重冻 candidates 必断
`pool_digests` 锚链、重冻 sample 即换样即作废五审计员 GT——**部分重冻不存在**。
处置 = 族内锚链自洽照旧、CI 信封门照跑（从冻结行自重导，不触现探测器）、
审计判决为内容级（key 只是标签）故 GT 有效；重生成走 `--ignored` 生成器时
必须**整族连审计一起**按新 epoch 重立。churn 台账零 .go 行（实测），
活体重放门零冲击。

## Docdup 仪器链（3d 段宇宙/oracle → 3g 普查/修正案/精度）

**段宇宙**（`docdup-segments-*-v1.json`，`ce.eval-docdup-segments/1.0.0`，生成于
`1cd3115`，DOCDUP_REV **3**（修正案后重冻结）；门 `eval_docdup_universe`）：live
段 self 114 / requests 98 / ripgrep 251 / cobra 59 / zod 117；常数 DOC_SHINGLE 5、
verbatim_floor 50、DOC_LINE_CAP 200、license 头 5 行豁免、SEGCAP 8192。

**oracle**（`docdup-oracle-*-v1.json`，`ce.eval-docdup-oracle/1.0.0`；门
`eval_docdup_oracle`）：live 段全对枚举整数交叉 Jaccard ≥80/100 ∨ verbatim ≥50，
oracle 对 self 4 / requests 4 / ripgrep 8 / cobra 8 / zod 8 = **32**。

**普查**（`docdup-sample-v1.json`，`ce.eval-docdup-sample/1.0.0`；门
`eval_docdup_sample`）：32 全量普查零选择（勘误：报告地板人口 30<100 抽样算术
不可能）；by_kind comment 17 / docstring 8 / md_para 7；population =
report_floor 23 + margin 9。census v2 = v1 的严格子集（retired 15 对冻结存证）。

**精度 = 达标线 B 修正案**（`docdup-precision-*-v1.json`，
`ce.eval-docdup-precision/1.0.0`，生成于 `5f3b73b`；门 `eval_docdup_precision`）：
D3 scoped **17/17** correct（ripgrep 7 + cobra 4 + zod 6）+ docstring **6/6**
（self 3 + requests 3，不设门）+ not_reported 8 台账化；D1 oracle 召回
self 3/3、requests 3/3、ripgrep 7/7、cobra 4/4、zod 6/6 全 100%（硬门 0.99）；
修正案 = 三条类别级行掩码（html_line / fenced_code_line / overlong_line），
DOCDUP_REV 2→3 五语料重冻结，J-floor 全表 50..100 逐档 wrong 恒 0。

## Churn 台账（3h 盲审）

`churn-ledger-v1.json`（`ce.churn-ledger/1`；门 `eval_churn_ledger`）：self tip
`473adfce`，40 提交、1,993 行，totals appended 16,327 / rewrote 1,461；五盲审
代理 × 8 席蛇形均衡，**26/40 逐行零仲裁**；14 提交差异全过逐文件守恒证书
（equal-minimal-alignment-slide 类）后按产品 Myers 仲裁双方存证；重放门 40/40 +
结构门 + 五语料守恒门。

## Score / 棘轮门（3i–3j，活体基线——本节登记机制，不登记活数字）

- `ce-baseline.json`（仓根，`ce.baseline/1`）：continuous 指纹实体天花板 +
  discrete 违规成员集，随每次 `ce baseline` 演进（活体，非冻结件——文件本身
  即记录）。守恒桥 members+collapsed==blocks==budget（3i 收口实测
  40+57==97==97，`ce.toml` budget 97）；门 `baseline_bridge`。
- 3j 门迁移：CI awk 300 门退役（同批删除）；替代 = 逐文件棘轮 ceiling +
  `core_size_gate` 三腿逐文件断言（产品发射 / 基线覆盖 / tolerated≤300 不弱于）
  + no-awk 守卫（两门并存即红）+ 母册 EVAL-SET.md ceiling≤300 硬红（RM20）。
- 3k 语料 generation（预注册 RM14/§8.4 兑现）：
  `contracts/eval/pre-haskell-members-v1.json` 冻结 3j 收口树（`4b39695`）的
  40 个 pre-Haskell discrete 成员；门 `baseline_bridge::
  pre_haskell_members_survive_every_generation` 断言其为每个后继基线的真子集。
  预算 97→149 双笔具名（+13 generation / +39 表族，`ce.toml` 历史段；本行
  3l 勘误——本册曾记 150/+40，权威数是 fmt 后量得的 149/+39）；churn
  台账重放改按工件冻结语言域比对（五语言白名单，`eval_churn_ledger`）。
  CoC 立场登记册 =
  [coc-haskell-divergences.md](../contracts/coc-haskell-divergences.md)
  （D0 无外部 oracle 起十三条），机检半身 = `coc_haskell` 电池 + 五语言等价对拍。

## Graph 阶梯（3l，M5-3B 收官）

**阶梯**（`ladder/hs.rs` 两 rung + `graph/cabal.rs` 解析面；站点 =
`graph/spec.rs` import 臂，`module` 字段单咽喉——anon `import` token 与
`foreign_import` 内层 token 撞 kind 名（D11 类）但无该字段，构造性丢弃）：
R1 = cabal 源根行走（owner 按目录前缀最深、同深两 cabal = ambiguous_workspace；
stanza 归属并集立场——双 stanza 共持文件跨组件分歧即 ambiguous_root 拒绝；
无 cabal 锚仓根 = 裸 ghc 语义）；R2 = `hs_boot.rs` 机生成表（GHC 9.14.1
全局包库 43 包 1,510 模块；滤除 = 库自记 exposed 位（`ghc`）+ 零模块包两条
db 事实，零圈选），有 cabal 时按 owner build-depends 门控、无 cabal 时整库
默认可见。库外声明依赖（aeson，store 安装）拒绝 out_of_scope——module→package
无证据绝不猜。

**电池**（门 `graph_ladder::rungs_resolve_and_refuse`，hs 15 行）：每 rung
≥1 恰级 fixture + 歧义保持 Unresolved（跨根 Dup 双席、双 cabal 工作区）+
build-depends 门双向（bytestring 声明 → ext(2)；containers 未声明 → 拒绝）+
裸 ghc 双向 + 块内注释不吃后续 deps；反事实（copy-restore）翻 Data.Map 门
案例实证见红。cabal 真钉 = `cabal::tests` 解析真 `core/ce-core.cabal`
（两 stanza 根 + 五 deps）。

**自仓活化实测**（release，fresh `.ce`）：deadcode **34 → 0**——32 个 .hs
孤岛误判全经真 import 边 + Main.hs/Spec.hs 机械入口约定（cabal main-is
惯例）活化，余 2 同批处置归零：`segments_tests.rs` 入 entry_globs（第三个
`#[path]` 档，rs.rs R5 既档）、CoC 登记册由上节 md 真链接活化（unlinked
doc IS reported 立场的机制性偿还）。勘合逐数分解 **176 站点 = 78 R1
入库边 + 80 声明外部（base/containers/bytestring/array）+ 18 aeson 库外**。
GRAPH_REV 2→3（.hs 缓存行空表陈旧）；`.cabal` 入 resolve_key。自仓警
台账 16→18：+`hs_boot.rs` 676 行（机生成表，头部 why）+`graph_ladder.rs`
364 行（共享树纪律，头部 why）；本批新码三处 E01 警（cabal parse 三连、
sites 测试、spec sites()）全按拆分机制清零非豁免。dedup 预算 149==149
零抬升（go↔hs 阶梯前导 T2 块以 helper-first 序断流偿清）。
冻结五语言宇宙全族不动（SCOPE_EXTS 按构造封闭；2h 精度门版图不含
Haskell = 既定验收边界，3l 验收线 = rung fixture 全绿，plan M5-3B 行）。

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

## 复跑

```
cd cli && cargo test --test eval_t3_universe --test eval_t3_candidates \
  --test eval_t3_sample --test eval_t3_precision --test eval_t3_recall \
  --test eval_docdup_universe --test eval_docdup_oracle \
  --test eval_docdup_sample --test eval_docdup_precision \
  --test eval_churn_ledger --test baseline_bridge --test core_size_gate \
  --test core_wire
```

外部四语料需本地 `.ce-eval/` 克隆（tip 见上表，CI 门 `rev-parse` 复核 RM19）；
冻结件重生成走各 `--ignored` 生成器测试，diff 为空即完整复现。
