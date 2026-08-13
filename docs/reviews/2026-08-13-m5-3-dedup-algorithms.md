# M5-3 深度去冗 设计定稿 · 卷二：算法（2026-08-13）

> 本卷是 [M5-3 设计定稿](2026-08-13-m5-3-dedup-design.md) 的第二卷（§4–§8）。
> 卷一 = 缺陷处置 + 模块布局 + wire + 存储；卷三 =
> [评估仪器 / 子里程碑 / 风险 / 阻断决策](2026-08-13-m5-3-dedup-instruments.md)。
> 拆卷理由与卷一同：单卷 823 行 > E01 `file_lines_fail = 750`（`config.rs:26-27`），
> 会让本仓自己的 `ce scan ..` 门（`ci.yml:78`）红——狗粮不能只写在别人身上。
> 缺陷编号 F1–F42 见卷一 §0；`file:line` 均于 HEAD `b6b9fc1` 实读。

---

## 4. T3 冷路径——算法与常数

### 4.1 单元粒度与身份

判决对象 = **函数单元**，键 = `(path, units::segments.key, nth)`（F2）。理由全是既有
事实：`units::segments`（`units.rs:21-26`）已给出跨语言单元词汇且**已持久化**
（`store.rs:126-131` 写 `symbols`）；任意 token span 之间不存在"子树"，TSED 无定义。
`symbols` 今天**没有读面**（`load.rs:36-58` 只读 files/edges/sites）——3b 补
`graph/symbols.rs`。准入下界 `T3_MIN_NODES = 24`：低于此的"克隆"是签名不是实现。

### 4.2 候选生成——三源并集，判决自由，先于 TED 冻结

| 源 | 内容 | 为何独立于被测物 |
|---|---|---|
| S1 逐字近邻 | `pairs::extend_anchor`（`pairs.rs:186-190`）今天把 `len < t` 的已验证公共段**静默丢弃**；共享指纹 ⇒ 按构造存在公共 25-gram（`Params{kgram:25}`，`mod.rs:240-245`），故被丢弃population 恰为 `25 ≤ len < 50`。加第二汇取回 | M2 冻结件，先于 M5-3 |
| S2 同键跨文件 | `units::segments` 键相同、跨文件、两端 ≥ `T3_MIN_NODES` | M4 冻结件 |
| S3 原始指纹共现 | `index::all_instances()`（`index.rs:210-221`）的裸 `Instance` 共现，**完全不做 extend** ——有意比任何合理候选器更宽 | M2 冻结件 |
| S4 结构 MinHash | `struct_fp` 的结构 shingle 上跑 MinHash/LSH（判别力存疑，F22：只作补充源，须发布带组分布） | 3c 冻结，先于 TED（3e） |

四源取**并集**。候选器不看 TSED ⇒ 可在 TED 存在之前冻结。

### 4.3 两道可证可容许剪枝（无假阴，Rust/Haskell 双侧，过线之前）

设 `n1,n2` = 具名节点数，`I = Σ_ℓ min(c₁(ℓ),c₂(ℓ))` = kind 直方图多重集交，`ted` = 单位
代价树编辑距离。Tai 映射 `M`（偏序保持部分单射）：

```
cost = n1 + n2 − 2|M| + r ,  r = |M| 中标签不等的对数
零代价对至多 I 个 ⇒ |M| − r ≤ I ⇒ r ≥ |M| − I
cost ≥ n1 + n2 − |M| − I ≥ max(n1,n2) − I          （|M| ≤ min(n1,n2)）
```
故 **`ted ≥ max(n1,n2) − I`**，于是 `TSED ≤ I / max(n1,n2)`。剪枝：
`I · tsedDen < tsedNum · max(n1,n2)` ⇒ 判负不过线。推论（O(1) 前置）：
`I ≤ min(n1,n2)` ⇒ `min·tsedDen < tsedNum·max` ⇒ 判负。尺寸带被标签界蕴含，
两道都保留（一道 O(1)、一道 O(#kinds)）。

> **R2 纪律（detection-first 的最强条款，逐字继承）**：这两条界是本设计**推导**的，
> 不是引用。**不许以"显然"落地**——`core/test/CloneProps.hs` 必须把
> `ted a b ≥ max (size a) (size b) − labelInter a b` 与 `ted a b ≥ |size a − size b|`
> 作为**穷举族属性**对着 `ReferenceTed` 的暴力 TED 断言。界若错，CI 当场红。

### 4.4 TSED 与容量（`CE/Clone/Cost.hs`，整数，零浮点）

```
TSED(t1,t2) = max(0, 1 − ted / max(n1,n2))
clone ⇔ (max(n1,n2) − ted) · tsedDen ≥ tsedNum · max(n1,n2)   -- 85/100，整数交叉相乘
```
浮点绝不出现（`FourClass/Cost.hs:2-3`："floats tie-break differently across platforms"；
核实 `core/**/*.hs` 全树零 `Double`/`Float`）。回传原始 `ted` 与 `n1,n2` **不回传比值**
——预注册 cut 表 {0.75,0.80,0.85,0.90,0.95} 可由一次跑分整表重算（M5-2 逐 rung cut 表
同构）。

**容量算术（F5）**：ZS = O(n₁n₂·min(d,l)₁·min(d,l)₂)。取 `unitNodeCap = 256`、典型
`min(d,l) ≈ 16` ⇒ 单对约 `256·256·16·16 ≈ 1.7×10⁷` 次 `IntMap` 更新。`pairCap = 4096`
是**在两道剪枝之后**的上限，超限 ⇒ `degraded:true, reason:"clone_too_large"`，绝不截断。
3e 退出含**实测**：自仓 + ripgrep（`PERF-BUDGET.md` 已录 1.29s 全量索引）冷路径全跑
预算，超则收紧 `pairCap` 并公布触发器（detection-first 的数值门，唯一能在上线前抓住
本类失败的机制）。`array==0.5.8.0` 已在 freeze（`:13`），晋升 `build-depends` 为
**预授权的独立 PR + golden 全绿**（R2 纪律，`plan:298`），仅在实测失败时启用。

---

## 5. docdup

### 5.1 语料抽取（检测器，冻结于判决之前）

| kind | 抽取 | 边界 |
|---|---|---|
| `md_para` | **`md::masked_content_lines`**（F3 新增：围栏 `md.rs:39-48` + HTML 注释 + 行内码等长反引号 run 配对）；段落 = 非空内容行的极大连续段 | 排除 ATX 标题行、表格行、纯列表标记行。缩进代码块**不建模**（`md.rs:14-16` 明言），其罕见链接形内容留在 `doc_gaps` 台账 |
| `comment_block` | 同一 parse 树上收 `spec.comment_kinds`（Python `["comment"]` `spec.rs:110`；TS `:159`；Rust `["line_comment","block_comment"]` `:197`；Go `:238`；Md `[]` `:256`），同起始列 + 行连续者合并 | Rust `///` 词法上就是 `line_comment`，自动入域 |
| `docstring` | `DocSpec.docstring_hosts` 驱动：Python = module/function/class 体首个 `expression_statement > string` | docstring 今天**不是**注释——它作为单个 `\x02LIT` 存活（`tokens.rs:107-131` + 同父合并 `:70-78`）。实测后果在册：`DEDUP-CALIBRATION.md:107` 的 requests 六条 miss |

### 5.2 四路类别级豁免（plan `:78-80`，**实装非文档**）

| 豁免 | 规则 | 台账键 |
|---|---|---|
| license 文件头 | 文件**首个** comment_block 且起始行 ≤ `LICENSE_HEAD_LINES = 5` 且命中 `license_markers`（`SPDX-License-Identifier` / `Licensed under the Apache License` / `Copyright (c)` / `Permission is hereby granted` / `MIT License`） | `excluded.license_header` |
| 结构化 docstring 骨架 | **行级剥离**（plan `:79` 字面"模板行"，F12/F33）：`Args:` `Arguments:` `Returns:` `Raises:` `Yields:` `Parameters` `Attributes:` `Example(s):` `Note:` `:param ` `:return` `:rtype` `@param` `@returns` `@throws` 及 `^-{3,}$` | `excluded.skeleton_line`（计数） |
| 行内 `ce:allow(docdup) -- <why>` | **无 why 即违规**（plan `:79-80`）；作用于其所在单元 | `excluded.inline_allow` |
| `.ceignore` / `ce.toml exclude` / 基线豁免存量 | 复用 M1 排除模型 + `ce-baseline.json` 的 `exempt` 段 | `excluded.path` / `excluded.baseline` |

豁免来源是这条评审的起因：`2026-08-07-plan-v1.2-delta-review.md:`**15**（Apache-2.0 头
自爆，F8）。**每一类必须计数入档，不得静默**——`pairs.rs:51-53` 的
`low_diversity_suppressed`、`churn.rs:123-126` 的无声上限禁令同一纪律。

### 5.3 管线

Rust 粗筛：词化（小写、剥标点、折叠空白、去 md 行内语法标记，每词 `fnv1a`）→ 准入
`min_doc_tokens = 50`（plan `:68`，Lee et al. 2107.06499）→ `DOC_SHINGLE = 5` 的
k-shingle（`winnow::kgram_hashes` 的滚动 Rabin-Karp，`winnow.rs:17-40`，`BASE=1_000_003`）
→ MinHash `MINHASH_PERMS = 128`，`sig[i] = min fnv1a(x ‖ i.to_le_bytes())`（确定性盐，
无 RNG，`plan:283-285`；F21）→ LSH `LSH_BANDS = 32 × LSH_ROWS = 4`（0.80 处召回
`1−(1−0.8⁴)³² ≈ 1−5×10⁻⁸`，拐点 `(1/32)^(1/4) ≈ 0.42`；F20）→ 热带链化按
`HOT_BAND_CAP = 64`（= `pairs.rs:19` 的 `HOT_CAP`，同一 D4 教训："skipping hot groups
made detection FALL TO ZERO"）。

**逐字路径**：两单元词 shingle 序列上的最长公共连续 run ≥ `verbatim_floor = 50` 词 ⇒
硬命中，与 Jaccard 结果并集。走 Rust（它就是 `pairs::extend` 的文本版，`pairs.rs:198-214`），
只把整数 `verbatimRun` 过线（F26）。

**精确 Jaccard 复核在 Haskell**（plan `:68`、`:177`）：Rust 送**升序去重的 shingle 哈希集**，
`CE.Docdup.Jaccard` 用 `Data.Set` 算 `|A∩B|`/`|A∪B|`，判定 = 整数交叉相乘
`inter · jaccardDen ≥ union · jaccardNum`（`siteOpens` 形状，`FourClass/Cost.hs:38-41`）。
u64 过线有先例（`FourClass/Wire.hs:8-9`：aeson 必须让 u64 走 Scientific 的 Integer 系数）。
**为什么是集合而不是"Rust 先算好 inter/union"**：那样"复核判定在 Haskell"就成了空壳，
且 `Reference.hs:6-8` 的穷举等价论证将无物可证。集合让 Jaccard 判定成为两个有界 u64
集的**全函数**，`ReferenceJaccard.hs` 的有界族穷举模式逐字迁移。

---

## 6. 三信号 join 与 score

### 6.1 三腿现状（实读）

| 腿 | 现状 | 缺口 |
|---|---|---|
| 相似度 | `pairs::Block`（`pairs.rs:22-38`）只有行 span；`to_block`（`:222-240`）吃掉 token 偏移后丢弃 | 用 `units::owner`（`units.rs:176-181`）把行 span 归到单元；token 精确重叠须先加宽 `Block` = `ce.dedup-report/0.5.0` 升版，本里程碑不做 |
| 图位置 | **已算好且被丢掉**：`Position.positions`（`Position.hs:13-21`）返 `[idx,indeg,outdeg,sccId,sccSize,reachIn]`，由 `Graph.hs:117` 调用；Rust 侧 `deadcode.rs:212` 发 `"pos": []`，`consume`（`:221-251`）只读 `dead`/`counts.kept`/`reason` | **零 wire 变更、零新图算法**——`pos` 已在请求与结果形状内（`VERSIONING.md:52,:58`） |
| churn | `churn::Report`（`churn.rs:29-37`）除 `cochange` 外全是全仓聚合；`classify_commit`（`:106-112`）用 `units::owner` 算出逐单元归属后**立刻折进两个全局计数器**；`survival`（`:137-156`）累一个总数 | **M5-3 唯一需改既有产品代码的腿**；`git blame --line-porcelain` per touched file（`:145`）无缓存 ⇒ 必须先入 PERF-BUDGET |

### 6.2 两层实体（F13/F14）

节点身份 = `(path, unit, nth)`，粒度码 `GRAN_FILE=0 / GRAN_PACKAGE=1 / GRAN_SECTION=2`
（`wire.rs:29-32`）。**不追加 `GRAN_UNIT`**——实读 `wire.rs:56-69`：任何代码解析都发
`String::new()`，非空 `dst_unit` 只来自 `ResolvedSection{slug:Some}`，故单元节点入度
恒 0。

- **Tier F（文件）**：三腿齐备，是**设门**的产品信号。
- **Tier U（单元）**：相似度 + churn 齐备，**图腿发 `null`，绝不编造**。仅报告。
  每条输出带 caveat："图腿=import 粒度；符号级入度需 R6，R6 是须独立 100 调用点
  审计 ≥0.90 的条件项（`2026-08-12-m5-2-graph-design.md:254-256`），本里程碑不解锁"。

### 6.3 判决格（`CE/Verdict/Join.hs`，常数在 `Cost.hs`）

| verdict | 条件（仅 Tier F 设门） | legsMask |
|---|---|---|
| 1 `merge_candidate` | 相似度过阈 ∧ 双方 indeg ≥ 1 ∧ **sccId 不同** | 0b111 |
| 2 `delete_candidate` | 相似度过阈 ∧ 一侧 indeg=0 ∧ reachIn=0 ∧ 非入口（`entryMask`，`Graph/Cost.hs:39-46`）∧ 伙伴 survive 更高 | 0b111 |
| 3 `churn_hotspot` | 相似度过阈 ∧ cochange ≥ `cochangeFloor` ∧ rewrite 占比过阈 | 0b111 |
| 0 `report_only` | 其余 | 实际到齐的腿 |

`legsMask` 是强制输出：三信号合取**就是**产品主张（plan `:43-44`），两腿点火的判决必须
自陈。`reasonBits` 逐位含义在 `Cost.hs` 逐位注释，含**有意缺席的位**（`entryMask` bit 0
的体例）。**RG10 同形防线**：公共 API 相似绝不并入 `delete_candidate`——公私是判决轴
不是活性声明（`Graph/Cost.hs:41-46`）。

---

## 7. score + 棘轮（ADR-006，100% greenfield）

核实：`--fail-under` 在 `cli/` **0 命中**；`main.rs:28-125` 的十三个子命令中无
`check`/`report`/`baseline`/`eject`，而 plan `:52` 承诺"M1 起"。

### 7.1 连续半（plan `:208-210`），整数

```haskell
tolerated :: Integer -> Integer
tolerated c = max (c * tolNum `div` tolDen) (c + tolAbs)   -- 102/100, 10
```
`div` 向下截断 = 保守侧（"平局不开"同立场，`FourClass/Cost.hs:38-41`）；交叉点在
c = 500，两支各钉一条 `costModel` 断言（`Spec.hs:46-57` 体例）。低于 ceiling 自动收紧；
容差消耗计入 Stop 汇总；`drawn + remaining == granted` 由属性测试守恒。

### 7.2 离散半与成员身份（F17/F38 的正面解）

```
member id = fnv1a(kind ‖0‖ a_path ‖0‖ a_key ‖0‖ a_nth ‖0‖ b_path ‖0‖ b_key ‖0‖ b_nth)
            kind ∈ {clone, t3, docdup, deadcode}；两侧按 (path,key,nth) 字典序规范化
```
`nth` = 同 `(file, key)` 内按 `start_line` 的出现序（F2）。**行号与块序有意不入身份**：
搬动一块克隆不变红；**新增**一处克隆变红；**改长**既有克隆由 `clone_tokens` 连续
ceiling 抓——这正是 ADR-006 自己的两机制分工（`:208-212`），不是把每一 token 增长都
造成新违规。三条反事实各一条测试钉死：

| 反事实 | 期望 |
|---|---|
| 整体搬移 fixture 克隆（改行号） | 离散集不变，**不红** |
| 新增一处克隆 | `added` 非空，**红** |
| 把既有克隆改长 | 离散集不变，`clone_tokens` 超 ceiling，**红** |

已知降级（写进模块头，不粉饰）：删除同键更早的兄弟单元会使 `nth` 位移 ⇒ 该成员 id
变化，读作一删一增。

### 7.3 与 `ce.toml [dedup] budget = 187` 的关系

**共存，不替代**（judgment-first 的取舍，最低风险）。今天的 CI 门是标量
（`ci.yml:79` → `check_budget`，`dedup/mod.rs:57-74`）；集合门并行运行，
`cli/tests/baseline_bridge.rs` 断言 `len(discrete.clone) == ce.toml budget`。
"只缩"不变量从计数强化为**集合包含**：`new ⊆ old`，否则须 `CE_ACCEPT_BASELINE=1`。
`ce.toml:32` 的"never raise without a plan amendment"跨格式存活。
**语料 generation 入基线 schema**——Haskell 入语料是 generation 变更，不能伪装成回归
或胜利（见 §8.4）。

### 7.4 评分与敏感性电池（`core/test/VerdictProps.hs`）

极性"越高越好"（plan `:74`），整数 per-mille，`wTotal` **导出**不是字面量
（`destFloor` 体例，`FourClass/Cost.hs:43-47`——手打字面量正是权重悄悄死掉的方式）。
七轴：`size` / `complexity` / `clone` / `docdup` / `deadcode` / `churn` / `graph_cycle`，
起手等权（决策 ⑦）。`--roast`（plan `:74`,`:312`）只留开关位不实现表。

电池六项：
1. **两条前置断言（F16）**：逐轴基线 penalty > 0（非空性，`Spec.hs:114-116` 的
   `edgeCap` 死旋钮教训）**且**七轴 penalty 两两互异、权重两两互异（互异性——等权
   等 penalty 下加权均值对扰动免疫）。缺任一即测试作废。
2. 逐权重 +1 扰动 ⇒ `(score, violation count)` 元组必须变（plan `:72`,`:286-287`，
   `GraphProps.deadKnobs:43-58` 形状）。
3. 逐轴阈值常数扰动 ⇒ 总分必须变。
4. **join 腿消融**：置零任一腿 ⇒ verdict 计数元组必须变。
5. **棘轮幂等**：把 `newBaseline` 施于同一批测量 ⇒ 零判决。
6. **单调性以集合包含陈述**，非计数比较（`maskMono`/`rungMono`，`GraphProps.hs:91-99`）。
7. 负性质驱动真实 `CE.Verdict.respond`，**同时匹配 code 与 message 子串**
   （`GraphProps.shuffledRefused` 形状，`:112-129`）。

权重全部作为**参数**传入，仅在族边界绑到 `Cost` 常数（`Graph/Cost.hs:4-7`）——这正是
让电池能在不碰生产代码的前提下扰动旋钮的机制，也是 ADR-008 迁移只换求值器的前提。
`-Wall -Werror`（`ce-core.cabal:24,:47`）**不**强制常数活性：导出但无人读的常数编译干净。
活性永远是测试义务。

---

## 8. Haskell 语言支持

### 8.1 范围（建议：size + CC + CoC + `comment_kinds`；**不含 graph 阶梯**）

`ladder/mod.rs:10-12` 已写好诚实降级路："a future language without rungs must return
Unresolved(Unsupported) — an honest ledger row, never a silent skip"。一条 Haskell 阶梯
要重走 M5-2f 的"每 rung ≥1 恰在该级解析的 fixture + ≥1 必须保持 Unresolved 的歧义
fixture"——那是一个完整子里程碑。plan `:288` 的承诺是"M5 起 core/（Haskell 支持就位）
+ 棘轮"，未承诺图。

### 8.2 两处强耦合（必须同批）

1. `scan/lang.rs:8-15` **只准追加** `Haskell = 6`，并同批加 `LangUnknown = 7` 哨兵修
   `deadcode.rs:141-143` 的 `Lang as i64 … unwrap_or(0)` 与 `Python = 0` 撞码（今天
   "未知扩展名"在 wire 上与 Python 不可区分）。契约惰性：`Graph.hs:78-82` 只拒负数。
2. `scan/spec.rs` 259/300、`GO` 块 39 行（`:204-242`）⇒ 表体住新 `scan/spec_hs.rs`，
   `spec.rs` 只加一条 dispatch 臂。`graph/spec.rs:41-87` 给 Haskell **显式无站点臂**
   （非 `_ =>`）。

### 8.3 CoC 无外部对照物（plan `:266` B2 的原因）——三件替代

`SOURCES.md:38` 显示 gocognit 是唯一外部 CoC 对照（Go）。诚实答案不是造一个：
1. **白皮书移植**：Sonar S3776 共通例题的 Haskell 译本必须得白皮书判分——扩展
   `cli/tests/sonar_whitepaper.rs`（**178** 行）。
2. **跨语言等价对拍**：语义等价构造跨语言同分（guard ≈ `match`/`switch`，`case` alt ≈
   `switch`，`if-then-else` ≈ `if/else`），与既有 Go/Rust spec 逐对断言。
3. **立场登记册** `docs/HASKELL-COC.md`：逐条给 S3776 条款与适配理由
   （`cli/tests/divergence_stances.rs` **198** 行的文献式体例）。需要立场的构造：
   guard 分支、`where`/`let` 嵌套、`do` 绑定、点自由组合链、算子截断、嵌套模式 `case`、
   `MultiWayIf`、view patterns。**"无外部 oracle"本身逐条登记为分歧**（plan `:281-282`
   要求逐条归因；此处诚实的归因就是缺席），不制造假 oracle。

### 8.4 grammar 与棘轮（未核实项 + 必须先付的账）

`cli/Cargo.toml:25-29` 无 `tree-sitter-haskell`（tree-sitter 0.26.11 + go 0.25.0 /
python 0.25.0 / rust 0.24.2 / typescript 0.23.2）。**其在 0.26 ABI 下的可得性本次未能
联网核实**——3k 首个动作是 spike；不可得 ⇒ 落回 size-only（`lang.rs` 的
`grammar() → None` 路径现成，Markdown 同档）并公开记录，CoC 那半顺延（决策 ⑧）。

`ce.toml:22-34` 的 187 是在**零 Haskell** 语料上测的（`.hs` 不是 `Lang`）。
`Lang::Haskell` 一落地，`core/app` 984 + `core/test` 543 首次进入 dedup 语料，块数
极可能上升，直撞"never raise without a plan amendment"。**处置（预注册）**：3k **必须
排在 3j 之后**，让集合基线先就位——语料增长从"数字变大"变成"一批具名新成员"，逐条
可审可归因可豁免；先跑**只测不改**的探针批次；delta 以
`budget_before_haskell = 187` + `haskell_delta = N` 记入 `ce.toml` 历史段，并配 CI 门
断言 pre-Haskell 成员是新集合的**真子集**。
