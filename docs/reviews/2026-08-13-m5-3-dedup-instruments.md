# M5-3 深度去冗 设计定稿 · 卷三：仪器与决策（2026-08-13）

> 本卷是 [M5-3 设计定稿](2026-08-13-m5-3-dedup-design.md) 的第三卷（§9–§12）。
> 卷一 = 缺陷处置 + 模块布局 + wire + 存储；卷二 =
> [T3 / docdup / join / score / Haskell 算法](2026-08-13-m5-3-dedup-algorithms.md)。
> **§12 的十项阻断决策逐项走 AskUserQuestion 上呈**（全局 `CLAUDE.md` 2026-08-07 指令）。
> 缺陷编号 F1–F42 见卷一 §0；`file:line` 均于 HEAD `b6b9fc1` 实读。

---

## 9. 预注册评估仪器

### 9.1 通用纪律（从 M5-2 逐条继承，出处在册）

宇宙先于判决器冻结且分母不得由被测物自选（`EVAL-SET.md:255`）· 常数见数前写死、
单绑双消费 · 抽样 = 哈希排名无 RNG 无时钟 · GT 是**数据**非常量，`include_str!` 按语料名
**首日**挂载（`eval_support/graph.rs:129-138`，M5-1d C3 教训）· truth 词表封闭、每行
`why` 必填且必须**指名机制** · 审计由独立 agent 执行、装配者逐字不改
（`EVAL-SET.md:279-281`）· 祖先门 `is_strict_ancestor`（`a≠d`）+ 两层盲窗**全扫** +
浅克隆响亮拒绝 + `fetch-depth: 0` · 反怯懦三数（全宇宙应答率 / 理由分桶台账 /
检测器盲区巡检）· 未达标协议：缺门语料档**扣发**并写明失败门（requests-L2 先例，
`EVAL-SET.md:177-180`）。

### 9.2 仪器 A — T3 精度（合同门 ≥0.85，plan `:271`）

- **宇宙** = §4.2 四源并集的**候选单元对**，3c 冻结于 TED（3e）之前。
- **冻结档** `contracts/eval/t3-universe-{corpus}-v1.json`：钉定 OID（F10 门 `rev-parse`）、
  文件清单+sha256、逐 (lang, source) 与逐 (lang, sizeBand) 计数、逐剪枝阶段淘汰数、
  **见数前写死的证伪常数**（下表）。
- **分层** (lang) × sizeBand ∈ {S:24-60, M:61-200, L:>200 具名节点}。
- **抽样** `rank_key = sha256("ce-t3-pair-v1|"‖corpus‖commit‖a_path‖a_key‖a_nth‖b_path‖b_key‖b_nth‖source)`；
  主样 **100** = 每语言地板 15 + 25 席最大余数（纯整数分摊，`EVAL-SET.md:270-271` 同法）；
  独立审计序域 `ce-t3-audit-v1|`；后备 **20/语言不跨语言**。
- **分母补足**（`EVAL-SET.md:189-190` 机制逐字复用）：审计沿冻结排名序伸入后备，
  直到被 T3 判 clone 的行数达 `min_answered = 40`。**这是"精度只对已答行"（M5-2
  `:187` 同立场）在 T3 上的落地**，也是驳回"以自身输出为分母"的方式：宇宙是四源
  并集、报告率与输出量地板同时公布。
- **GT** `cli/tests/eval_t3_review/{corpus}.json`，封闭 truth ∈
  `{clone, variant, boilerplate, unrelated, t1t2, generated}`。
  - `boilerplate` **必须单列**：`DEDUP-CALIBRATION.md:70-72` 实测 FP 全类都在这里
    （status_codes 数据行、locale 文件内前后 key 段）；并进 `unrelated` 会把它藏起来。
  - `t1t2` **必须单列**（instrument-first）：否则 T3 的**增量**不可测，recall 被热路径
    t=50 已报的对充胀。
  - `clone` 行另带 `edit ∈ {rename, reorder, insert, type_sub, control_tweak}`——消融轴，
    不是装饰。
- **仲裁规则（见数前写死）**：`clone` = 在钉定 OID 读两侧源后，其一可删除并改写为对
  另一的调用而行为不变。`boilerplate` = 语言惯例致结构同构而无共享意图。
- **指标**：precision = correct/(correct+wrong)，只对判 clone 的已答行，**门 ≥0.85**
  （总体；in-corpus GT 分母 ≥5 的语料逐个设门——`EVAL-SET.md:288` 的 2026-08-12 决策）。
- **发布数（非门）**：全宇宙逐 (lang, sizeBand, source) 应答率与判正率 · `below_floor`
  台账（候选器**结构性看不见**的对——method 字串必须明写"候选召回是 recall 上界"）·
  `unit_gaps`（Rust `macro_rules!` 体、Go `func_literal`、Python 嵌套类等，`site_gaps` 同形）·
  **全 θ cut 表**，发布旋钮 = 过 0.85 的最宽松阈值。

**G 门（照抄 `eval_graph_precision.rs` 的实现形状）**：T-G1 summary 由同一 scorer 从行
重导（`:159-163`）· T-G2 ≥0.85 总体 + 分母 ≥5 每语料（`:207-214`）· T-G3 六类守恒 ==
样本量 · T-G4 rank 双射 + `(a_path,a_key,a_nth,b_path,b_key,b_nth,source)` 逐字身份 echo
（`eval_support/graph.rs:43-66`）· T-G5 **每语言地板 15 = 红不是脚注**（`:235-237`）·
T-G6 存档判决由其自身行重算（`:144-153`）· T-G7 重复台账行在**共享咽喉**拒绝
（`eval_support/graph.rs:61`）· T-G8 wrong 台账冻结，增长需 `CE_ACCEPT_T3=1`（`:59-70`）·
T-G9 反事实电池 ≥5 变体（翻判决 / 幻影 rank / 删行 / 煮 summary / **打桩 TSED 阈值**）·
T-G10 `FROZEN_CORPORA` 锚 + PENDING 变体（`eval_support/graph.rs:28-39,:81-87`）·
T-G11 确定性：文件序洗牌 ⇒ 字节同一；增量 ≡ 全量 · T-G12 `degraded` 回复**直接拒绝**
（实现锚 `eval_graph_precision_parts/mod.rs`，F35）· T-G13 祖先三腿（样本 ≺ 审计 ≺
`cli/src/dedup/t3` ∪ `cli/src/docdup` ∪ `core/app/CE/Clone` ∪ `core/app/CE/Docdup`
**整子树全扫** ≺ 跑分）· **T-G14 输出量地板**：每语料 reported pairs ≥
`min_reported_pairs`（预注册），防"以沉默换精度"（F18）。

### 9.3 仪器 A' — Haskell 侧穷举参照（TED 正确性不靠断言）

`core/test/ReferenceTed.hs`（`Reference.hs:1-9` 模式逐字迁移）：有界族 = 全部有序有根
标签树，**CI 默认 n≤4（Catalan(n−1)×2ⁿ = 2+4+16+80 = 102 棵 → 10,404 有序对）**；
n=5 层（+448 棵 → 302,500 对）由 `CE_DEEP_TED=1` 门控走 nightly/tag（F6）。参照实现 =
**定义式暴力**（枚举全部合法 Tai 映射取最小代价），不是出品算法的另一种写法。比较用
`Data.Set`/规范值非列表（`Reference.hs:125-131`）。首个失配 dump 实例。

`core/test/CloneProps.hs`：(1) §4.3 两条可容许界的**全族属性**（R2，界错即红）；
(2) 度量三公理 `ted a a == 0`、对称、三角；(3) 死旋钮：扰动 `tsedNum`/`unitNodeCap`
必须改判决计数 **且** 非空性前置断言绿；(4) 固定种子 LCG = `Integer` + 显式
`mod 2^64`（`GraphProps.hs:80-81`），非 `Word64` 回绕。

### 9.4 仪器 B — T3 召回 vs mizchi/similarity（plan `:271` 字面门 ≥0.90）

**核实**：对照物不在仓内、未安装、渠道无任何在仓记录。`mizchi/similarity` 全仓只在
`DEVELOPMENT_PLAN.md:34,:67,:169,:201,:271` 以散文出现（无版本、无 commit、无调用行）
——对比 jscpd 有 `5.0.14` + 逐字 npx 命令行在册（`DEDUP-CALIBRATION.md:3,:135-136`）。
三个独立设计各自 spike 均未确认包名与渠道。→ **阻断决策 ①**。

方法学逐条照抄仓内唯一有用户拍板的召回口径（M2/jscpd）：
- 对照物输出 = **召回分母且仅此**，**绝不是精度 oracle**（`DEDUP-CALIBRATION.md:26-31`：
  ce 检出而对照物原理性不可见者，人工仲裁计真阳不计假阳）。
- 其"TSED 0.85 默认"必须**从其源码读出并记页/行**，不得继承 plan `:67` 的转述。
- **双报**：`recall_raw`（对照物全集，plan `:271` 字面合同门 ≥0.90）与
  `recall_incremental`（扣除 T1/T2 在 t=50 已报的对；书面处置触发器 <0.50）。
- **归因排除制是否延用 = 阻断决策 ③**：plan `:267` 的 M2 行**已写入**该口径，
  plan `:271` 的 M5-3 行**没有**。把它带过来是改计划。
- 语料风险：若对照物只覆盖 TS/JS，五语料塌成 zod 一家 ⇒ recall 门作用域缩到被覆盖
  语言并**公布覆盖清单**，precision 门仍跑五语料；**不中途加语料**（会破
  `FROZEN_CORPORA` 锚与祖先链）。
- 不可得 ⇒ 该门**扣发并写明**；替换工具须改计划（`CLAUDE.md` 硬约束 2 + ccm 重锁）。

### 9.5 仪器 C — docdup（双仪器）

**C1 oracle 仪器（无人工，definitional）**：对冻结段集**暴力**精确 Jaccard，产出
`contracts/eval/docdup-oracle-{corpus}-v1.json` = 全部 `J ≥ jaccard_universe_floor`
的对 + 全部逐字 run ≥ `verbatim_floor` 的对。MinHash/LSH 是它的估计量，所以 oracle
不需要人也不可争辩（instrument-first 的最强单点）。代价有界化见 F31。

**C2 审计仪器**：`contracts/eval/docdup-sample-v1.json` = `J ≥ jaccard_report_floor`
的对中 hash-rank **100 行**，域 `ce-docdup-pair-v1|`，分层 `unit_kind` × lang，
**`min_per_kind = 15`**，后备 20/kind（F30/F42）。GT
`cli/tests/eval_docdup_review/{corpus}.json`，封闭 truth ∈
`{redundant, paraphrase, license, skeleton, tabular, quoted, deliberate_xref, unrelated}`。
`paraphrase` **必须单列**：docdup 是词汇级的，改写重复是**设计内 miss** 不是 bug。

> **2026-08-14 落地勘误（普查制）**：报告地板人口 30（含边际带 47）＜100 ⇒ 样本 =
> **全量普查**（零选择；边际带入审），`min_per_kind` 受人口约束；实测五独立审计 47/47，
> `paraphrase` 席位为空（J ≥ 0.30 处已被词汇塌缩淘汰，入档不造席）；修正案见算法卷 §5.2。

**D 门**：D1 oracle 召回 ≥**0.99**（硬门，逐语料）· D2 **可靠性**：每条上报对的精确
Jaccard ≥ 报告地板 **或** 逐字 run ≥ 50 ——这条证明 Haskell 复核**确实点火**了 ·
D3 精度 ≥**0.85**（`md_para` + `comment_block`；决策 ④）· D4 八类守恒 == 100 ·
D5 双射 + 身份 echo · D6 豁免台账：每个被过滤段有理由行，且 `license`/`skeleton` 类
**零行存活进上报集** · D7 自证伪 · D8 台账冻结（`CE_ACCEPT_DOCDUP=1`）·
D9 反事实电池含**复核打桩**与**豁免打桩**两变体（F10 教训：变体不齐时文档过度声称
覆盖）· D10 `FROZEN_CORPORA` + `include_str!` 按名挂载 · D11 确定性（MinHash 是段的
纯函数，无时钟无 RNG）· D12 祖先（与 T-G13 共实现）· **D13 旋钮回显相等**：
Rust `DOC_SHINGLE` == `docdup.result.knobs.shingleK`（F29）。

**自仓即首语料**（plan `:273`/`:289` 已把 `docs/` 定为终局对象，且它是我们自己的）：
3g 内先跑**报告态**，3j 起设门；真发现在原地偿付（`CLAUDE.md` 硬约束 4：不堆叠编辑）。

### 9.6 预注册常数（见数前写死；单绑双消费）

| 常数 | 值 | 住处 | 出处状态 |
|---|---|---|---|
| `T3_MIN_NODES` | 24 | `dedup/t3` + universe 档 | **decided, not derived**，需窗口 + 反事实 |
| `STRUCT_SHINGLE` | 4 | `dedup/struct_fp.rs` | decided |
| `T3_ANCHOR_FLOOR` | 25 | `dedup/t3` | **导出** = `Params::default().kgram`（`mod.rs:242`） |
| `MINHASH_PERMS` / `LSH_BANDS` / `LSH_ROWS` | 128 / 32 / 4 | `dedup/minhash.rs` | **导出**：0.80 地板处召回 ≈1−5×10⁻⁸，拐点 0.42 |
| `HOT_BAND_CAP` | 64 | `dedup/minhash.rs` | **导出** = `pairs.rs:19` 的 `HOT_CAP` |
| `tsedNum` / `tsedDen` | 85 / 100 | `CE/Clone/Cost.hs` | plan `:67` 给了数但**未给可核出处** ⇒ 本仓自定义并文档化（决策 ②） |
| `unitNodeCap` / `pairCap` | 256 / 4096 | `CE/Clone/Cost.hs` | decided + **sizing anchor 注释**（`Graph/Cost.hs:15-26` 体例）+ 3e 实测 |
| `min_doc_tokens` / `verbatim_floor` | 50 / 50 | `docdup/spec.rs` + slice 档 | **有出处**：plan `:68`（Lee et al. 2107.06499） |
| `DOC_LINE_CAP` | 200 | `docdup/spec.rs` | decided, not derived（2026-08-14 达标线 B 修正案：注释/docstring 段内可见内容超此长度 = 正则/数据/生成物非散文；FP 实证两行 300+/600+ 字符，硬换行散文惯例 <120；播种反事实钉死） |
| `DOC_SHINGLE` | 5 | `docdup/spec.rs` | ⚠️ 落地批次二选一：实读论文记节/页，**或**写"decided, not derived"+ 实测窗口 + 反事实（`FourClass/Cost.hs:53-58` 体例）。**不许凭记忆写出处** |
| `jaccardNum/jaccardDen` | 80 / 100 | `CE/Docdup/Cost.hs` | 同上 |
| `jaccard_universe_floor` | 30/100 | oracle 档 | decided（oracle 发射地板） |
| `LICENSE_HEAD_LINES` | 5 | `docdup/spec.rs` | decided |
| `min_per_lang` / `min_per_kind` | 15 / 15 | 两份 slice 档 + `eval_support/dedup.rs` | **导出** = M5-2 同名常数 |
| `min_answered` | 40 | t3 sample 档 | decided（分母补足停止条件） |
| `min_reported_pairs` | 逐语料，3c 写死 | t3-universe 档 | decided（T-G14 输出量地板） |
| `lsh_oracle_recall` | 0.99 | docdup 档 | **硬门**（非软触发器，F20） |
| 容差 | `max(c*102 div 100, c+10)` | `CE/Verdict/Cost.hs` | **有出处**：ADR-006 `:209-210` |
| 权重表 | 七轴等权（各 1），`wTotal` 导出 | `CE/Verdict/Cost.hs` | 决策 ⑦ |

---

## 10. 子里程碑序列（退出 = 红条件）

| # | 内容 | 退出（红条件） |
|---|---|---|
| **3a** | proto 2.2.0 + `clone/1`+`docdup/1`+`verdict/1` 三 capability + 三个对一切输入回 `contract` 的空 handler。**机械、独立、先行**（2a 先例） | 两常数（`Protocol.hs:26`/`corelink.rs:18`）读 2.2.0；**22 条 reply 行**重生成、19 条 request 行**未动**且该立场入 `VERSIONING.md` §3；三份新 golden 各 ≥5 对（含畸形树/非升序集/越界索引）；`Spec.hs` ≤300（否则同批拆 `SpecStructural.hs`）。**红**：损坏单 golden 字节没让**两套**电池都红；或 diff 触及 proto/capabilities/golden/cabal 之外任何东西 |
| **3b** | `md::masked_content_lines`（F3）+ `graph/nodes.rs` 抽取（F19）+ `graph/symbols.rs` 读面 + `symbols.nth` + schema v5 + `dedup/struct_fp.rs` + `dedup/unitcache.rs` + `ce clone --units`（列单元宇宙，**无判决**）+ 五语料 `t3-universe` 冻结 | `ce graph --sites` 五语料站点数与既有 slice 档**字节同一**且 `GRAPH_REV` 未动；`deadcode` 与 join 在同一棵树上产出**逐字节相同**的节点 id 分配（断言非假设）；`symbols` 的 `UNIQUE(file_id,key,nth)` 在五语料上无冲突；两次干净树跑字节同一；五语言单元数全非零；`TOKENIZER_REV` 仍为 2 且 `ce dedup --check` 恰报 187；第 4 次 per-file 解析入 PERF-BUDGET 行（F11）。**红**：任一 |
| **3c** | `dedup/minhash.rs` + 四源候选并集 + 两道可容许剪枝 + `t3-candidates` 档 + `t3-sample-v1.json`（100 + 后备 20/语言）+ 逐语料 `min_reported_pairs` 写死。**TED 不存在** | 两跑 rank id 同一；`verify()` 重哈希绿含重复 id 拒绝；每语言 ≥15；`git log` 证样本提交**先于** `cli/src/dedup/t3` 与 `core/app/CE/Clone` 下任何文件；S4 带组大小分布已发布。**红**：任一；或两道剪枝中任一被实现为**非可容许**形式（近似剪枝 = 假阴 = 分母污染） |
| **3d** | docdup 抽取器 + 四路豁免实装 + `docdup-segments` 宇宙冻结 + `docdup-oracle` 暴力档 | 围栏/行内码/HTML 注释内文本**零单元**（且实现只调 `masked_content_lines`——另写掩码即红）；四类豁免各有非零计数或写明为何为零；**播种反事实**：N 条 Apache-2.0 头 + M 条 `Args:/Returns:` 骨架 ⇒ 该两类零段存活；`DOCDUP_REV` 入 meta 键且 bump 清缓存；`tokens.rs` **零 diff**。**红**：任一；或某段 kind 五语料全零且 `doc_gaps` 无解释行 |
| **3e** | T3 判决：`CE/Clone{,/Ted,/Prefilter,/Cost}.hs` + `ReferenceTed.hs` + `CloneProps.hs` + Rust `t3/{mod,tree,wire}.rs`。**参照 harness 与出品同批，绝不后补** | 暴力 ≡ 出品：n≤4 全族对全绿（n=5 层 `CE_DEEP_TED=1` nightly 绿）；**§4.3 两条界作为全族属性绿**；度量三公理绿；死旋钮扰动改判决计数**且**非空性前置绿；**实测 PERF-BUDGET 行**：自仓 + ripgrep 冷路径全跑 <60s，超则收紧 `pairCap` 并公布触发器；`ReferenceTed.hs`/`CloneProps.hs` 落在 `core/test/`（进 `core/app/` 即红——吃 300 门） |
| **3f** | T3 精度 100+ 后备逐对人工审计（独立 agent，装配 verbatim）+ `unit_gaps` 巡检 + `cli/tests/dedup_provenance.rs` + **达标线 A** | 100/100 行带**指名机制**的非空 why；truth 词表封闭且 `boilerplate`/`t1t2` 各有实际席位；总体 precision ≥0.85，分母 ≥5 的语料逐个 ≥0.85；T-G1..T-G14 全绿（含输出量地板与 θ cut 全表）；祖先三腿绿含浅克隆响亮拒绝。**红**：任一；或档静默扣发**无** PENDING 记录 |
| **3g** | docdup 判决：`CE/Docdup{,/Jaccard,/Cost}.hs` + `ReferenceJaccard.hs` + Rust MinHash/LSH 粗筛 + 100 行审计 + **达标线 B** | Jaccard 有界族穷举等价绿；D1 oracle 召回 ≥0.99 逐语料；D2 可靠性绿（证明复核点火）；D3 精度 ≥0.85；D13 旋钮回显相等绿；LSH 分带对签名的单调性以**集合包含**陈述；`paraphrase` 有席位；`DEDUP-CALIBRATION.md:107` 的 `models.py __bool__/__nonzero__` **被命中**，api.py 三连以具名 `below_floor` 行出现（F12）；`tokens.rs` 零 diff 且 `--check` 恰报预算。**红**：任一 |
| **3h** | 三信号 join：`join/churn_unit.rs` + `join/mod.rs` + `graph.request{pos:[…]}` 消费 + `CE/Verdict/Join.hs`（Tier F/U） | 逐单元 churn 与**手工审计的 40 提交台账**逐行相符（守恒检查抓不到系统性错归属）；拆分后逐文件求和 == 拆分前全仓数（逐语料守恒）；`reply["pos"]` 行数 == 请求 `pos` 长度（断言）；Tier U 的图腿是 `null`**从不编造**；每位 `reasonBits`/`legsMask` 在 `Cost.hs` 逐位有注释；公共 API 相似绝不进 `delete_candidate`（RG10 同形反事实钉死）；`git blame` 代价入 PERF-BUDGET 且 join 保持报告态。**红**：任一 |
| **3i** | `CE/Verdict/{Score,Ratchet,Cost}.hs` + `score/{mod,baseline,wire}.rs` + `main_score.rs` + `ce check`/`ce baseline`/`--fail-under` + `VerdictProps.hs` + CI 棘轮 | 电池七项全绿，**含两条前置**（非空性 + 互异性，F16）；容差两支（c<500 / c>500）各一条 `costModel` 断言；离散指纹三条反事实（搬移不红 / 新增红 / 改长由连续 ceiling 红）；`baseline_bridge.rs` 断言 `len(discrete.clone) == ce.toml budget`；`ce check` 与 `ce dedup --check` 在自仓同判；`--fail-under` 与棘轮**各自可独立 fail**（两向测试）；`ce-baseline.json` 入库且**原样过 wire、Rust 不解释**（`plan:232`）。**红**：任一 |
| **3j** | `core/**` 门迁移：awk 门 → `ce check` 逐文件棘轮 ceiling | 替代门在**每个既有 `core/**/*.hs` 文件上不弱于** awk 门（逐文件断言，非"数字一致"，F4）；同批删 awk 门（两门并存即红——静默漂移）；`EVAL-SET.md` 拆册落地（决策 ⑨） |
| **3k** | Haskell 语言支持：`scan/spec_hs.rs` + `Lang::Haskell`（追加）+ `LangUnknown` 哨兵 + CoC 规范 + 分歧登记册 + 棘轮重基线 | grammar spike 结论公开记录（不可得 ⇒ 落回 size-only，`grammar()→None` 路径）；Sonar 白皮书共通例题 Haskell 译本全绿；跨语言等价对拍绿；分歧登记册逐条带立场与 S3776 条款（含"无外部 oracle"作为一条分歧）；`Lang::Haskell` 位于枚举**末位**且 `graph/spec.rs` 有显式无站点臂（非 `_ =>`）；棘轮 delta 记为 `budget_before_haskell=187 + haskell_delta=N` 且 CI 断言 pre-Haskell 成员是**真子集**。**红**：任一；或预算被抬高而无计划修订 + 子集门 |

**依赖**：3b←3a · 3c←3b · 3d←3b · 3e←3a,3c · 3f←3e · 3g←3d,3e（复用 minhash）·
3h←3f,3g · 3i←3h · 3j←3i · **3k←3i（强制，见 §8.4）**。

---

## 11. 风险登记册

| id | 风险 | 缓解 / 预写证伪触发器 |
|---|---|---|
| RM1 | 对照物 mizchi/similarity 不可得（**三设计各自 spike 均未确认渠道**） | 3a 起并行 spike；不可得 ⇒ **扣发该门并写明**，替换须改计划（决策 ①）。jscpd 是 T1 工具，**不能**顶替 |
| RM2 | 候选生成器自选分母（"拒绝难对即抬精度"的 T3 版） | 四源并集（三源为 M2/M4 冻结件）3c 冻结先于 TED；`dedup_provenance.rs` 两层盲窗全扫；**T-G14 输出量地板**；候选召回上界写进 method 非暗示 |
| RM3 | §4.3 两条界是**本设计推导的**，错了就是静默假阴 | `CloneProps.hs` 全族属性对暴力 TED 断言；界错即红。**不许以"显然"落地** |
| RM4 | Zhang-Shasha 实现错误 | `ReferenceTed.hs` 定义式 Tai 映射暴力枚举；出品与参照同批 |
| RM5 | **TED 代价在真仓爆掉** | 两道剪枝 + `unitNodeCap=256`/`pairCap=4096` + 3e 数值 PERF 退出门 + `array` 预授权晋升（F5） |
| RM6 | **穷举 harness 撑爆 CI 时长**（n≤5 = 302,500 对 × 三平台） | CI 默认 n≤4（10,404 对），n=5 走 `CE_DEEP_TED=1` nightly（F6） |
| RM7 | 改 `tokenize` 收注释 ⇒ `TOKENIZER_REV` bump ⇒ 全索引清空 + T1/T2 重标定 + 187 棘轮重来 | docdup 走**独立加法抽取器**；3b/3d/3g 各有"`tokens.rs` 零 diff + `--check` 恰报预算"红条件 |
| RM8 | docdup 另写掩码 ⇒ 判决器看见检测器看不见的东西 | 强制只调 `md::masked_content_lines`（`md.rs:29-32` 已立"ONE masking implementation"之约）；另写即红 |
| RM9 | **MinHash 精度不可穷举验证**（`Reference.hs:6-8` 的枚举论证对估计量不成立） | 暴力精确 Jaccard oracle 让精度主张**经验化于冻结宇宙**；Haskell 复核让上报集按构造精确；D2 证明复核点火 |
| RM10 | `(path,key)` 碰撞使新克隆映射到既有成员，棘轮绿着而重复增长 | `nth` 消歧 + `UNIQUE(file_id,key,nth)`（F2）；三条反事实（§7.2） |
| RM11 | 离散指纹带行号 ⇒ 每次编辑全红，团队关掉门 | 指纹**行号与块序无关**；存在归离散集、量级归连续 ceiling（ADR-006 自身分工） |
| RM12 | 敏感性测试空转（等权 + 等 penalty ⇒ 加权均值对扰动免疫） | **两条**前置断言：非空性 + 互异性（F16） |
| RM13 | docdup 在自仓文档上自爆（plan `:289` 把计划文件也纳入约束） | 豁免是带台账的过滤器不是抑制；3g 报告态、3i 起设门、真发现原地偿付 |
| RM14 | Haskell 入语料 ⇒ 棘轮上升，撞"never raise" | 3k **必须**在 3i 之后；集合基线让增长成为具名成员集；先跑只测不改探针；generation 入基线 schema |
| RM15 | `Lang` 枚举中插 ⇒ 静默重标全部 lang 码；且今天"未知扩展名"≡Python | 只准追加 + `LangUnknown = 7` 哨兵，契约惰性（`Graph.hs:78-82` 只拒负数） |
| RM16 | `deadcode.rs` 286/300、`spec.rs` 259/300、`Spec.hs` 143→~172 三处门咽喉 | 拆分是**排期交付物**（§1.2）不是顺手：`graph/nodes.rs` 先于 join、`scan/spec_hs.rs` 先于 Haskell 表、`SpecStructural.hs` 进 3a 退出判据 |
| RM17 | ADR-008 抢跑或反被堵死 | wire 携**事实**、策略住 `CE/Verdict/Cost.hs`、`ce-baseline.json` **原样过线不解释**（`plan:232`）；DSL 落地换求值器不换 wire |
| RM18 | 审计者 100 判漂移 | why 强制指名机制；审计先于跑分（T-G13）；分歧**入档不调和** |
| RM19 | 语料双钉（`SOURCES.md:10` vs `.ce-eval` HEAD） | 每份宇宙档内载实际 OID，CI 门 `rev-parse`（F10） |
| RM20 | 文档预算：plan **316**（棘轮上界，只准变短）、`EVAL-SET.md` **恰 300** = E01 警戒线且自身入 CI 扫描；3j 后 300 从警戒变硬红 | 决策 ⑨：`EVAL-SET.md` **拆册**而非压缩已冻结记录（压缩冻结档有 provenance 代价）；plan 的编辑限于决策 ④ 所需的验收行就地改写 |
| RM21 | **工期**：8,980 新行、三新 wire 族、三套仪器、两套穷举 harness vs plan `:271` 的"3 周 ±" | 决策 ① |

---

## 12. 阻断用户决策（逐项：问题 / 2-4 个具体选项 / 推荐与理由）

> 十项已于 2026-08-13 经 AskUserQuestion 逐项拍板（全局 `CLAUDE.md` 2026-08-07 指令）；
> 拍板结果就地记在各项「→」行。③ 经用户委托评估后采精化形（归因不排除）；⑧ 用户
> 选了高于推荐的全套范围。

**① M5-3 工期与拆分。** 本设计合计 ≈8,980 新行（2,505 产品 Rust + 1,155 app Haskell +
510 test Haskell + 2,610 仪器 Rust + 2,200 GT），plan `:271` 记"3 周 ±"。M5-2 用 3–4 周
完成 8 个子里程碑，且当时机器多已就位。
选项：(a) 维持单一 M5-3，接受工期超标；(b) **拆 M5-3A = 3a–3g（T3 + docdup，含两套
仪器）/ M5-3B = 3h–3k（join + score/棘轮 + Haskell）**；(c) 砍 Haskell 语言支持到 M6。
→ **推荐 (b)；拍板 = (b)**。与用户已做过一次的 M5→M5-2/M5-3 拆分（`2026-08-12-m5-2-graph-design.md:258-259` ④）同形；
plan `:271` 的退出行**天然可切**：T3 召回/精度归 3A，"本仓库自身跑通棘轮入 CI" 归 3B；
也让 ADR-008（plan `:230` 排在 M5-3 后）更早开工。

**② TSED 定义与 0.85 的出处。** 仓内无任何 TSED 定义，plan `:67` 的"mizchi/similarity
默认"在仓内无可核来源（grep 确认 `tsed` 只出现在文档里）。
选项：(a) 从对照物源码读出其定义与默认值并逐字记录，本仓对齐；(b) **在仓内自定义
并文档化（§4.4 公式），0.85 预注册为报告阈值，回传原始 `ted`/`n`，全 θ cut 表发布**；
(c) 推迟到 3a spike 结论。
→ **推荐 (b)，并把 (a) 作为 3a 的并行动作；拍板 = (b)**。与 M5-2 的 minRung cut 表同法：
无论对照物是否可得，本仓的判定都自洽可核；对照物默认值若不同，作为分歧入册而非改数。

**③ T3 召回是否延用「归因排除制」。** plan `:267`（M2 行）**已写入**该口径（用户
2026-08-07 AskUserQuestion 拍板）；plan `:271`（M5-3 行）**没有**，只给了裸 ≥0.90。
选项：(a) 严格按字面：`recall_raw ≥ 0.90` 对对照物全集，不排除；(b) **延用归因排除制，
预注册三类（docdup 域重叠 / TSED 测度失配 / T1-T2 已报），双报 raw 与排除后**；
(c) 只发布不设门。
→ 推荐 (b)；**拍板（用户委托"更优则用"，评估后精化）= 归因不排除制**：分母永不缩减
（对照物全集），检出按 **ce 产品全层记功**（T1/T2 已报 = 产品真阳，非排除项），每条
miss 入冻结台账按封闭词表归因 `{tsed_measure_mismatch, below_candidate_recall,
unit_gap, other}`（增长需 `CE_ACCEPT_T3R=1`），门挂 `recall_raw ≥0.90`（`:271` 字面
分母与数字），`recall_incremental`（扣 T1/T2 已报后的 T3 增量）并列发布、书面处置
触发器 <0.50。较 (b) 更强：排除机制整个消失而非被缓解——没有可加宽的类目；对 AST 基
对照物，注释不在其可检出集，docdup 排除类本就近空。**这是改计划**（`:271` 就地加句），
须 ccm 重锁。

**④ docdup / join / score / Haskell 四项无量化门。** plan `:271` 的退出只写了 T3 两个数
与"本仓库棘轮入 CI"。
选项：(a) 维持无门，仅以反事实电池验收；(b) **加：docdup oracle 召回 ≥0.99（硬）+
精度 ≥0.85（`md_para`+`comment_block`）+「豁免类零行进上报集」；join 不设数值门
（验收 = 诚实包 + Tier U 的 `null` 纪律）；score 只门敏感性电池 + 棘轮入 CI**；
(c) 给 join 也设数值门。
→ **推荐 (b)；拍板 = (b)**，另定 docdup 逐语料门沿用「in-corpus GT 分母 ≥5 才逐语料
设门」既有规则（`EVAL-SET.md:288` 同法）。docdup 两门都有**无人参与的分母**（oracle 是
definitional）故设门不引入自评风险；join 继承 M5-2 对 `unreferenced_public` 的立场
（独立类不 fail + caveat 报告）。**这是改计划**（316 行棘轮守恒 + ccm 重锁）。

**⑤ T3 精度分母。** plan `:271` 定义了召回分母（对照物），**精度分母未定义**。
选项：(a) ce 自身上报对（judgment-first 的选择）；(b) **四源冻结候选对宇宙 + 独立
审计 GT + 「只对已答行」+ 分母补足到 `min_answered=40` + 输出量地板**；
(c) 对照物集合（被 `DEDUP-CALIBRATION.md:26-31` 明令禁止：对照物绝不是精度 oracle）。
→ **推荐 (b)；拍板 = (b)**。(a) 是被三镜之一判为 BLOCKER 的自选分母；(c) 违反已拍板
口径。(b) 是 M5-2 唯一被证明能防"自选分母"的形状（宇宙先于判决器、只对已答、分母补足）。

**⑥ `ce-baseline.json` 与 `ce.toml [dedup] budget = 187`。** 替代还是共存？
选项：(a) 3i 内替代，`budget` 退休；(b) **共存：标量门保留为 CI 主门，集合门并行绿，
`baseline_bridge.rs` 断言两者相等；集合连绿两个 PR 后是否退休标量另议**；
(c) 永久共存。
→ **推荐 (b)；拍板 = (b)**。两个廉价门都不脆，且 `ce.toml:32` 的"只缩、never raise
without a plan amendment"跨格式无损存活；(a) 要求桥测试自身永远诚实，风险更高。

**⑦ 评分权重表。** 哪几轴入分、相对权重。仅受"越高越好"（plan `:74`）与敏感性测试
（plan `:72`）约束。
选项：(a) **七轴等权（各 1），`wTotal` 导出，逐轴 why 注释**；(b) 按经验加权；
(c) 先只上三轴（size/clone/deadcode）。
→ **推荐 (a)；拍板 = (a)**。未经证据的加权比"明说是任意的"更糟；敏感性电池证明无一轴
是死的；权重调整在 ADR-008 后是数据变更。**注意**：等权必须配 §7.4 的**互异性**前置
断言，否则电池空转（F16）。

**⑧ Haskell 语言支持范围。** size-only（如 Markdown）/ +CC+CoC / +graph 阶梯？
选项：(a) size-only；(b) **size + CC + CoC + `comment_kinds`（docdup 域），
graph 阶梯顺延**；(c) 全套含 graph 阶梯。
→ 推荐 (b)；**拍板 = (c) 全套含 graph 阶梯（高于推荐）**：阶梯按 M5-2f 纪律（每 rung
≥1 恰级 fixture + ≥1 歧义保持 Unresolved）作为 M5-3B 内独立子里程碑 **3l**（3l←3k）；
3k 首个动作仍是 `tree-sitter-haskell` 0.26 ABI 可得性 spike（本次未能联网核实），
不可得 ⇒ 落回 size-only 公开记录，CoC 与阶梯顺延，`Unresolved(Unsupported)` 为降级路。

**⑨ 文档预算支付方式。** plan 316（棘轮上界，只准变短）、`EVAL-SET.md` **恰 300**
（E01 警戒线，自身入 CI 扫描，且 3j 后 300 变硬红）。M5-3 要加三个仪器章。
选项：(a) 压缩 `EVAL-SET.md` 的 M4/M5-1 章为 M5-3 腾位；(b) **拆册：`EVAL-SET.md`
（索引 + M0–M5-2 冻结记录）+ `EVAL-SET-M5-3.md`**；(c) 压缩 M5-2 图章。
→ **推荐 (b)；拍板 = (b)**（用户曾问抬线，按代价对账驳回：阈值出处 = 计划 §4.1 狗粮
契约，抬线全仓生效且悖于 never raise）。压缩已冻结记录有 provenance 代价（是仍在跑的
门的可复现性依据）；(c) 尤其危险。plan 编辑限于 ①/③/④/⑧/⑩ 所需验收行就地改写。

**⑩ `check` / `report` / `baseline` / `eject` 的归属。** plan `:52` 承诺"M1 起"，
实测 `main.rs:28-125` 十三个子命令中四个全无。
选项：(a) 四个全部塞进 M5-3；(b) **`check` + `baseline` + `score` 归 3i（ADR-006
天然领地），`report` 归 M6、`eject` 归 M7（plan `:244` 已提及），在计划就地标注**；
(c) 维持不标注。
→ **推荐 (b)；拍板 = (b)**。消灭"承诺了但没人拥有"的悬空；(a) 会把 M6 的报告面提前
进一个已经超载的里程碑。
