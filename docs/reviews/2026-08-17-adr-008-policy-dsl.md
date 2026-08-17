# ADR-008 细则设计与迁移契约——策略即数据（2026-08-17）

> **三拍板**（用户，2026-08-17，AskUserQuestion 结构化）：
> ① 边界细则 = **判决/测量分界**（入计划 v1.8 并 ccm 重锁）；
> ② DSL 形态授权"按最优雅定案" → 定案 = **位台账 + 判决表**（§4）；
> ③ 本批切片 = **四片全收**（P1 判决权回迁 / P2 棘轮统一 / P3 scan 分级入 core / P4 配置面）。
>
> 普查 = 两路独立探查代理（Haskell 侧 core/app 全量、Rust 侧 cli/src 全量），
> 行号基于 tip `1fe2501`。本册是普查冻结件与切片契约的单一权威。

## 1. 分界原则（细则正文，进计划 ADR-008 段）

**判决语义**（什么算违规、什么豁免于判决、预算、棘轮、阻断）→ ce-core 求值。
**测量语义**（哪些事实存在：指纹参数、掩码豁免、截断、上限执行）→ Rust，受两条约束：
单点声明（一个常量/spec 模块）；凡镜像 core 常量者必须 knobs 回显钉住（lockstep 抓漂移）。

**机器判据**：迁移该语义是否要求源文本或行级内容过 wire——要求即测量侧
（§5.9.2 索引隐私：不存源文本，一票否决）。

**两条判例**（分界原则的显式例外，均以计划既有条款为据）：
- **热路径判例**：guard（PreToolUse 廉价门）的判决按 §5 职责边界留 Rust
  （"延迟敏感的热路径完全在 Rust 进程内完成，不跨语言"），其阈值已走 ce.toml。
- **协议映射判例**：tier→decision 映射（deny→deny/ask→ask/warn→allow，
  guard.rs:214-219）是 hook I/O 协议翻译，非策略语义，留 Rust。

## 2. 普查冻结——Haskell 侧（core/app）

### 2.1 wire 已通（4 项）
| 项 | 位置 | 现状 |
|---|---|---|
| sizeCeil (axis 0) | Verdict/Cost.hs:97 + Wire.hs:43 + Verdict.hs:107 | wire ceilings 覆盖，默认 300（27b9bc2 首步） |
| cocCeil (axis 1) | 同上 :102 | 同上，默认 15 |
| weights 七轴表 | Wire.hs:36 + Score.hs:122 | wire 传入；Rust 侧 score/wire.rs:59-61 **故意恒发空** → 线上恒等权 |
| floor (--fail-under) | Wire.hs:37 + Verdict.hs:100 | wire Maybe，缺席=不判 |

### 2.2 硬编码策略常量（18 项，均"参数旅行、家族边界绑定"）
| # | 位置 | 策略 | 值 |
|---|---|---|---|
| B1 | Clone/Cost.hs:15,18 | tsedNum/Den（克隆阈，三家复用） | 85/100 |
| B2 | Docdup/Cost.hs:23,26 | jaccardNum/Den | 80/100 |
| B3 | Docdup/Cost.hs:38 | shingleK（仅回显对钉，core 不加窗） | 5 |
| B4-6 | Graph/Cost.hs:35,48,55 | minRung / entryMask / sccFloor | 5 / 126 / 2 |
| B7 | Verdict/Cost.hs:109 | deadIndegCeil | 0 |
| B8 | Verdict/Cost.hs:85-91 | 棘轮容忍 tolNum/Den/Abs（腿交叉于 500） | 102/100, +10 |
| B9-10 | Verdict/Cost.hs:67,75,78 | cochangeFloor / rewriteNum/Den | 2 / 50/100 |
| B11-12 | Verdict/Cost.hs:117,123 | violCost / defaultWeight | 10 / 1 |
| B13 | **Score.hs:128** | 分数上限/地板 1000/`max 0` | **唯一未进 ScoreKnobs 的评分数字** |
| B14 | Join.hs:95-97 | legsMask 位 | 1/2/4 |
| B15-18 | FourClass/{Cost,Verdict,Anchor}.hs | 四分类代价/anchorFloor/stacking/bucketCap | 1,3,0,2 / 19 / 20,10 / 64 |

caps（5 项，over-cap⇒完整 degraded 回复）：verdictNode/Row 131072/524288、
graph 同、clone 256/4096、docdup 8192/4096、行字节 32 MiB（Protocol.hs:40）。

### 2.3 内联判决策略与三缺口
校验层内联（Wire.hs）：ceilings 轴上限=1、weights 七轴全零拒、floor 0..1000、
sim kind>2 与 den==0 拒、metricCode≤6、tier 0..1。判决层内联：**Join 判决码格
（gated=simOver∧graphBoth；1 merge/2 delete/3 churn/0 report，优先级=guard 顺序隐式，
Join.hs:132-137）**、RG10 公开守卫（:125-126）、fail 合取（Verdict.hs:101）、
棘轮 min 收紧+ESTABLISH（Ratchet.hs:64,81）、缺席语义（churn 真零/cochange
unknown-small/graph leg 拒 gate，Verdict.hs:139-141）。

**三缺口**（普查副产物，P4 偿还）：
1. Score.hs:128 的 1000/`max 0` 在 knobs 外，扰动电池碰不到；
2. metricCode 2..6 合法但无人产出（Rust 只产 0/1，baseline.rs:65,71）——契约单向缺口；
3. **Join 判决优先级无表无常量，改序不被任何电池抓获**（JoinProps census 按类计数）。

### 2.4 表驱动雏形（五处，DSL 收敛的地基）
knobTable 单行文法（Wire.hs:216，weights/ceilings 两实例）；penalties 七行表
（Score.hs:77）；reasons 位表（Join.hs:139，8 行 (bit,held)）；effectiveKnobs
覆盖折叠（Verdict.hs:107，仅 2 分支硬编码）；Protocol.families dispatch 表。
验证半边：每 knob 一杠杆的扰动电池已全（VerdictProps/JoinProps/GraphProps/
CloneProps/ReferenceJaccard）。

## 3. 普查冻结——Rust 侧（cli/src）与归边

ce.toml 全 12 键：exclude、thresholds×8（file_lines_warn/fail、fn_lines_warn/fail、
params_warn、cyclomatic_warn、cognitive_warn、nesting_warn）、guard.mode、
dedup.budget、graph.entry_globs（config.rs:13-87）。
**陷阱：Config 无 deny_unknown_fields——未知策略键静默丢弃（P4 修）。**

② 类可容许剪枝镜像（合规，knobs echo 钉住）：TSED 85/100（candidates.rs:27）、
JACCARD 80/100（judge/wire.rs:27）、DOC_SHINGLE 5、四 caps 镜像。

③ 类独立语义 17 项**逐项归边**：

| # | 位置 | 语义 | 归边 | 去处 |
|---|---|---|---|---|
| 1 | scan/report.rs:27-99 | warn/fail 分级+退出码（无 wire） | **判决** | **P3** |
| 2 | dedup/mod.rs:63-80 | 预算 145 第二棘轮 | **判决** | **P2** |
| 3 | guard.rs:109-127 | guard 硬预算门 | 判决·热路径判例 | 留 Rust（§1） |
| 4 | audit.rs:92-101 | mode=deny 阻断映射 | 协议映射判例 | 留 Rust（§1） |
| 5 | main_score.rs:83-101 | baseline only-shrink 再解释 | **判决** | **P2** |
| 6 | score/mod.rs:108 + main_score.rs:67 | degraded→FAIL | **判决** | **P1** |
| 7 | graph/deadcode.rs:145-196 | entry 位事实生产（globs/约定） | 测量（事实生产；豁免判决=core entryMask） | 留 Rust |
| 8 | docdup/exempt.rs+spec.rs | license/骨架/allow/地板掩码 | 测量（行级内容不过 wire） | 留 Rust，单点已聚 spec.rs |
| 9 | docdup/judge/mod.rs:50-53 | **verbatim>=50 析取半边** | **判决** | **P1** |
| 10 | dedup/t3/mod.rs:121-124 | **is_clone 最终判决** | **判决** | **P1** |
| 11 | t3/mod.rs:136 等 | caps 执行丢弃（ledger 已诚实） | 测量 | 留 Rust |
| 12 | score/mod.rs:154 | T1/T2 对=100/100 比率 | 测量（exact-run 恒等式编码） | 留 Rust+契约注记 |
| 13 | score/mod.rs:89 | ceilings 轴映射（键→轴） | 配置翻译 | **P4** 表化 |
| 14 | dedup/mod.rs:266 等 | winnowing/LSH/地板/稀释器 | 测量 | 留 Rust |
| 15 | churn/mod.rs:220,44 | top_pairs 截断/大提交跳过 | 测量（缺席=unknown-small 已建制） | 留 Rust |
| 16 | fourclass/mod.rs | L1 分类语义 | 测量（阶梯 L1 本属 Rust，L2=core） | 留 Rust |
| 17 | scan/walk.rs:11-30 | 内建 exclude 表 | 测量（走树范围，ce.toml 可配） | 留 Rust |

## 4. DSL 形态定案：位台账 + 判决表

**原则**：具名 Haskell 谓词只产**条件位**（reasonBits 机制泛化——已是 §6.3 建制）；
**判决本身是数据**。零新求值器、零代数机器；每张表配双电池
（扰动=每 knob 一杠杆〔已有〕；**重排=改行序必红**〔新增，堵缺口 3〕）。

两种表形（覆盖现有全部判决形态）：
1. **格判决表**：有序行 `[(verdictCode, requiredBits)]`，首行 `required ⊆ held` 即判——
   Join 四路（merge>delete>churn>report 显式化）、graph 四路、fail 合取皆此形。
2. **分级判决表**：行 `[(metricCode, warnKnob, Maybe failKnob)]`，值对阈交叉比较出
   Level——scan 分级（P3）与 E01 双级（file 300/750、fn 50/75）皆此形。

配置通道 = knobTable 文法泛化：`[axis|knobCode, value]` 行、严格升序、逐值判官——
weights/ceilings 已是两实例，P4 扩为全 knob 面（thresholds/tolerance 第三四表）。
Cost.hs 常量全体降级为**默认值**（27b9bc2 模式推广到整个 knob 面）。

## 5. 四片契约（实施顺序 P4→P1→P2→P3，每片一提交链一 CI 绿）

**验收细化**（ADR-008 "判决字节等价"的精确义）：**产品判决面字节等价** =
上报集、退出码、报告行逐字节不变（golden 全绿）；wire 回复按 proto minor
加性演进（新字段允许，语义位翻转不允许）。每片验收含反事实：翻一个新表行
序/新 knob 值，指定判决必变（证表在承重，非装饰）。

- **P4 配置面与表化**（基建先行）：knobTable 泛化承载全 knob 配置行；
  Cost.hs 常量→默认值；`Config` 加 `deny_unknown_fields`；ceilings 轴映射
  表化单点；Join 优先级/graph 四路/fail 合取落格判决表+重排电池；
  Score 1000/`max 0` 入 knobs（缺口 1）；weights 通道打通（ce.toml 键→wire，
  Rust 恒空退役）。判决字节等价（纯重述+配置默认不变）。
- **P1 判决权回迁**：t3 `is_clone`、docdup `is_dup`（含 verbatim 半边——
  verbatim run 长度数值过 wire，文本不过）判决入 core；core degraded 回复
  携 fail 语义（degraded→FAIL 规则入 core，Rust 侧输入降级拒绝保留=测量诚实）；
  Rust 剪枝常量降为 ② 类镜像并保持 echo 钉。上报集字节等价。
- **P2 棘轮统一**：dedup budget 145 比较入 core 求值（wire 携 blocks+budget，
  判决与失败文案语义等价、退出码等价）；main_score only-shrink 再解释收敛为
  消费 core fail 位；CE_ACCEPT_BASELINE 作为操作员出口留 Rust（非判决）。
- **P3 scan 分级入 core**：scan 获 `--core`（与 check/deadcode/docdup 同律，
  不读 env）；测量与报告渲染留 Rust，Level 判定+退出码语义走分级判决表；
  门链/CI scan 腿加 `--core $core`；冷延迟入 PERF-BUDGET 实测（现 0.52s 基线），
  预算超标则本片单独回滚不连坐。

**留守台账**（判决侧但明确不迁，防"字面全量"追责）：guard 热路径（判例一）、
audit 阻断映射（判例二）。测量侧留守 11 项见 §3 表。

## 6. 风险与回退

- **计划行数棘轮**：DEVELOPMENT_PLAN.md 只准变短——细则以就地改写入 ADR-008 段。
- **P3 性能**：scan 首次依赖 core 二进制；验收挂 PERF 实测，超标单片回滚。
- **proto 演进**：P1/P2/P3 各自加性 minor；golden 翻批必逐行审（3a 纪律）。
- **dedup 预算压力**：新增 Rust 桥接代码计入 145 预算——每片 fmt 后量，
  超则当场机制偿（棘轮咬作者已七次，第八次照例）。
