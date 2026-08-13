# M5-3 深度去冗 设计定稿（2026-08-13）

> 产出流程：3 独立设计（detection-first / instrument-first / judgment-first）→ 4 透镜
> 评审（contract / provability / realism / 合成前反审，逐引用对仓核实）→ 本合成。
> 透镜判决 1–1–2 分歧：contract 与 realism 判 judgment-first，provability 判
> instrument-first，第四镜判 instrument-first。**合成不取平均**——骨架取
> instrument-first（三源候选宇宙、暴力精确 Jaccard oracle、Tier F/Tier U 的 `null`
> 诚实、`t1t2` 真值类、`ce-baseline.json` 原样过 wire）+ 全身取 detection-first
> （四路豁免与逐类台账、两条可容许剪枝及其 R2 纪律、G1-G13 逐门实现锚、
> `boilerplate` 席位、requests 六条 docstring 现成验收物、数值化 perf 退出门）
> + 判定安放与门完整性取 judgment-first（`sccId` 保留、awk 门→逐文件棘轮 ceiling
> 而非放宽、非空性+互异性前置、离散指纹与 `ce.toml budget` 共存、语料 generation
> 入基线 schema、只重写 reply 行）。
>
> **基准 HEAD = `b6b9fc1`，工作树干净。** 本文所有行数为字节级 LF 计数（PS 5.1 的
> `Get-Content` 对 LF-only 文件少计——三设计中已有两处因此致错）。所有 `file:line`
> 于本次会话实读核对；未能核实者在同句写明。
>
> **本定稿分三卷（E01 狗粮：单卷 823 行 > `file_lines_fail = 750`，`config.rs:26-27`，
> 会红掉本仓自己的 `ce scan ..` 门 `ci.yml:78`——故拆卷，不压缩证据；与 §12 决策 ⑨
> 对 `EVAL-SET.md` 的建议同法）**：
> · **卷一（本文）** §0 缺陷处置 · §1 模块布局 · §2 wire · §3 存储
> · **卷二** [算法](2026-08-13-m5-3-dedup-algorithms.md) §4 T3 · §5 docdup · §6 join · §7 score · §8 Haskell
> · **卷三** [仪器与决策](2026-08-13-m5-3-dedup-instruments.md) §9 评估仪器 · §10 子里程碑 · §11 风险 · §12 阻断决策

---

## 0. 评审抓出的缺陷与处置（合成层已消化）

三镜共 60 余条，合并同源后 42 条。**每条给处置，无一条"记录不处理"。**

### 0.1 跨设计缺陷（三设计共有）

| # | 缺陷（实读证据） | 处置 |
|---|---|---|
| F1 | **评估仪器行数三设计全部漏算**。M5-2 单个 graph 精度仪器实测 = Rust 1,902 行（`eval_graph{,_audit,_precision,_sample}.rs` + 两个 `_parts/mod.rs` + `eval_support/graph.rs` + `graph_provenance.rs`）+ GT JSON 1,235 行。M5-3 提三套 | 本文 §1 把仪器行数**单列入预算**（~3,900 新增），并作为工期拆分决策 ①的依据数字，不藏在"测试"里 |
| F2 | **`(path, unit_key)` 不是键**。`units.rs:36` `format!("{}/{}", f.name, f.params)`——Rust 方法不带 impl 限定，`impl A{fn add}` 与 `impl B{fn add}` 同文件同键；Go 收方限定是 attack-review F7 的**单语言**补丁（`units.rs:52-69`）；`store.rs:47` `idx_sym_file` 非 UNIQUE | schema v5 给 `symbols` 加 `nth`（同 `(file_id,key)` 内按 `start_line` 的出现序）+ `UNIQUE(file_id,key,nth)`；一切单元身份、join key、离散指纹一律带 `nth`（§3、§7.2）。**同键兄弟被删导致 nth 位移**是已知降级，写进模块头并配反事实，不粉饰 |
| F3 | **`graph::md::content_lines` 不做行内码掩码**。三设计都声称复用它即得"围栏+HTML 注释+行内码"三重掩码；实读 `md.rs:33-55` 只返回 `comment_mask`（`:52`），行内码掩码是 `scan_line` 里调用的私有 `merge_code_spans`（`:68`, `:118+`）。detection-first 的 3g 红条件"复用它、另写即红"因此自相矛盾 | **加法**：`md.rs` 新增 `pub(crate) fn masked_content_lines()` = `content_lines` + `merge_code_spans`，不改任何既有调用路径。docdup 只准调它。红条件：重构后 `ce graph --sites` 五语料站点数与 `graph-slice-*-v1.json` **字节同一**，`GRAPH_REV` 不动（`store.rs:24-30`） |
| F4 | **退休 awk 门换 `ce scan core/` 是放宽不是迁移**。`ci.yml:49-56` 是硬 300 红；`scan/report.rs:46-50` 300 = `Level::Warn`，`scan/mod.rs:29-33` 只对 Fail(>750) 退 1。detection-first 3k 与 instrument-first 3h 都把"逐文件行数一致"当退出判据，数字一致而红→黄 | 采 judgment-first：`core/**` 迁到 **`ce check` 的逐文件棘轮 ceiling**（`CE/Graph.hs` ceiling=155 而非 300），严格强于被退休的门。红条件写成"替代门在每个既有文件上不弱于 awk 门"，而不是"数字一致" |
| F5 | **Zhang-Shasha 代价三设计均未算**。ZS = O(n₁n₂·min(d,l)₁·min(d,l)₂)。judgment-first 的 `treeCap 512 × pairCap 65536` 无算术支撑；instrument-first 连剪枝都没有 | §4.4 给出运算次数公式并据此定 `unitNodeCap = 256 / pairCap = 4096`；两条可容许剪枝**先于** TED（§4.3）；3e 退出含**实测** PERF-BUDGET 行（自仓 + ripgrep 冷路径），超预算则收紧 `pairCap` 并公布触发器；`array`（freeze:13 已钉 0.5.8.0）晋升 `build-depends` 为**预授权的独立 PR**，仅在实测失败时启用 |
| F6 | **穷举 TED 族大小是 CI 时间炸弹**。有序有根标签树 n≤5、\|Σ\|=2 = Catalan(n−1)×2ⁿ = 2+4+16+80+448 = **550 棵 → 302,500 有序对**，每对还要枚举全部 Tai 映射，跑在三平台 `cabal test all` 里 | 两层：**CI 默认 n≤4（102 棵 → 10,404 对）**；n=5 层（550 棵）由 `CE_DEEP_TED=1` 门控，nightly/tag 触发。族大小与算术写进 `ReferenceTed.hs` 头部（`Reference.hs:1-9` 的"枚举覆盖"论证体例） |
| F7 | **`Protocol.hs` proto 行号争议**。三设计写 `:26`，第二镜判 `:25` 有误 | 实读：`:25` = `proto :: String`，**`:26` = `proto = "2.1.0"`**。三设计正确，第二镜的更正被驳回。另：`maxLineBytes = 33554432`（`:34`，M5-2a 已从 1 MiB 放宽），detection-first 引"1048576"陈旧 |
| F8 | **D2-5 出处三设计全部引错**：写 `2026-08-07-plan-v1.2-delta-review.md:18`，实际 `:15`（`:18` = D1-5 `--roast`） | 全文改正为 `:15` |
| F9 | **proto 爆炸半径被高估**。41 行带 `"proto":"2.1.0"` 属实，但 golden 是请求/应答**交替**（`Spec.hs:64,:76-78`），且 `majorMatches` 只比 major（`Protocol.hs:81-83`）。实测拆分：**reply 22 行**（fourclass 9 / graph 8 / hello-ok 1 / wire-errors 4）、request 19 行 | 采 judgment-first：**只重写 22 条 reply 行**，request 行有意留在 `2.1.0` ——它成为"minor 偏斜必须被接受"（`VERSIONING.md:69`）的常设回归 fixture。此立场写进 `VERSIONING.md` §3，防后人"修好它" |
| F10 | **`requests` 语料双钉**。`SOURCES.md:10` = `1f6589ec…`（M1 crosscheck 抽样源）；`.ce-eval/corpora/requests` HEAD = `8068356288978c…` = `EVAL-SET.md:259`。两者均真实存在、用途不同 | 每份 M5-3 宇宙档必须内载其**实际使用的 OID**，且 CI 门 `rev-parse` 之（`eval_support/graph.rs` 的 sha 重建体例）。混用即红 |
| F11 | **T3 不重解析**为假。`dedup::analyze` 只持 token 流：`walkidx.rs:86` → `tokens::stream` 在 `tokens.rs:46-49` 解析后丢弃 `Tree`；`FnUnit<'t>` 的活 `Node`（`functions.rs:10-16`）只活在 `code_segments`/`stream` 内 | 承认第 4 次 per-file 解析（现已 3 次/脏文件），把它记进 3b 的 PERF-BUDGET 行；不写"零重解析" |

### 0.2 detection-first 专有

| # | 缺陷 | 处置 |
|---|---|---|
| F12 | 3g 旗舰退出判据自相矛盾：要求 `DEDUP-CALIBRATION.md:107` 六条 requests docstring 互拷全命中，同时 §2.2 剥离 `:param`/`:return` 骨架行、§2.3 设 `min_doc_tokens=50`。api.py post/put/patch 三连剥离后 ≈15 词 < 50 | 采纳该验收物但**改判据**：六条中 `models.py __bool__/__nonzero__`（散文 ~66 词）为**必须命中**；api.py 三连与 sessions 参数段为**必须以 `below_floor` 具名台账行出现**（不是静默 miss）。骨架剥离是行级不是整块（plan `:79` 字面"模板行"） |
| F13 | 追加 `GRAN_UNIT = 3` 使 join 成为删除候选机器。实读 `wire.rs:56-69`：**任何代码解析都发 `String::new()`**，非空 `dst_unit` 只来自 `ResolvedSection{slug:Some}`；`nodes_of`（`deadcode.rs:101-120`）只从文件∪边目标造节点 ⇒ 单元节点 indeg 恒 0 ⇒ `delete_candidate` 普遍点火 | **不加 `GRAN_UNIT`**。采 instrument-first 的 Tier F / Tier U：Tier U 的图腿发 `null`，绝不编造（先例：md `ref_def/ref_link/url` 零 GT 席位仍入册，`EVAL-SET.md:289-290`） |
| F14 | 重开已拍板决策 ②：单元级图位置需要 R6，而 R6 被冻结为条件项（须独立 100 调用点审计 ≥0.90，`2026-08-12-m5-2-graph-design.md:254-256`），设计全文未引 | F13 的 Tier U 方案即处置；join 每条输出**逐条带 caveat**："图腿=import 粒度，符号级入度不存在" |
| F15 | `verdict.request.pos` 丢 `sccId`，而 `merge_candidate` 要求"不同 SCC"——判据不可从自身 wire 算出 | `Position.hs:13-21` 实读六元组 `[idx,indeg,outdeg,sccId,sccSize,reachIn]` 原样过线，一位不删 |
| F16 | 敏感性电池可空转：`score = 100 − Σw·pen/Σw` 是加权**均值**，决策 ⑦推等权，前置只要求 penalty 非零——等 penalty + 等权 ⇒ 每次扰动皆无操作 | 前置断言**两条**：逐轴基线 penalty > 0（非空性，`Spec.hs:114-116` 的 `edgeCap` 教训）**且**七轴 penalty 两两互异、权重两两互异（互异性）。缺任一即测试作废 |
| F17 | 离散指纹只用 unitKey 且不含路径，叠加 F2 ⇒ 新克隆映射到既有成员，棘轮绿着而重复增长 | §7.2 的身份含 kind‖两侧 path‖key‖nth，三条反事实钉死 |
| F18 | 无输出量地板：精度只在判 clone 的行上算，候选器紧一点就抬精度 | 采 judgment-first 的 T-G14：**预注册每语料 `min_reported_pairs`**为硬红条件 + 全 θ cut 表 |
| F19 | "`deadcode.rs` 零 diff"与"join 复用 `nodes_of`"不可同时成立（`nodes_of:101`、`Node:54`、`judge:204` 全私有） | 弃"零 diff"。改为**抽出 `graph/nodes.rs` 身份咽喉**（deadcode 与 join 共用），退出判据 = 两者在同一棵树上**断言产出逐字节相同的节点 id 分配**——身份唯一性优先于文件字节稳定性 |
| F20 | LSH `b=16,r=8` 在其自身 0.80 报告地板处召回 1−(1−0.8⁸)¹⁶ = **0.947**，却被描述为"留召回余量"，且只配软触发器 | 改用 instrument-first 的 `b=32,r=4`：0.80 处 1−(1−0.8⁴)³² ≈ 1−5×10⁻⁸；拐点 (1/32)^(1/4) ≈ 0.42。并把召回从软触发器升为**对 oracle 的硬门 ≥0.99** |
| F21 | `mix64` 不存在，同段却称"复用既有原语零新依赖" | 置换实现只用 `tokens::fnv1a`（`tokens.rs:134-141`）：`sig[i] = min fnv1a(x ‖ i.to_le_bytes())` |
| F22 | 结构 shingle MinHash 判别力存疑：\|Σ\|≈150 kind、k=4，同语言无关函数共享大量 k-gram ⇒ 带普遍变热 | 保留但降级：结构 MinHash 只作**第三路**候选，且必须发布"每带组大小分布 + `hot_chained` 计数"；若热带链化产出 >50% 候选则该路以数据除名（预写触发器，`RG12` 同形） |
| F23 | §3.2 称 12 新模块，§4.2 列 11 | 本文 §1 的模块表即唯一计数源 |
| F24 | G2 称"阈值从档推导不硬编"；实读 `eval_graph_precision.rs:207-214` 0.90 是硬编，只有 ≥5 分母来自档 | 本文如实陈述：合同阈值硬编、分母规则从档推导 |

### 0.3 instrument-first 专有

| # | 缺陷 | 处置 |
|---|---|---|
| F25 | §0 "corrections" 表自称"`core/app` 978 / `core/test` 542 逐文件与合计均已确认"——**实测 984 / 543**。在以核实为唯一目的的章节里失准 | 本文全部行数重测（§1）。另 `Protocol.hs` 93→**94**、`sonar_whitepaper.rs` 177→**178** |
| F26 | `docdup.result.verbatimN` 不可从 `docdup.request.shingles:[[segIdx,hash]]` 算出——连续 run 需要顺序 | 逐字 run 移回 Rust（它就是 `pairs::extend` 的文本版，`pairs.rs:198-214`），只把整数 `verbatimRun` 过线。docdup wire 只走**升序去重的 shingle 哈希集**（§2.2） |
| F27 | 120 行样本按判决边界 60/60 分层且无重加权、无估计量声明；\|N\| ≫ \|P\| ⇒ 发布的 precision 不估计任何总体量 | 弃 60/60。采仓内既有分配法：主样 **100**，每语言地板 15 + 最大余数（`EVAL-SET.md:270-273` 同法）；分母补足机制照抄 `EVAL-SET.md`（沿冻结排名序伸入后备直到 `min_answered` 行），后备 20/语言不跨语言 |
| F28 | 3e 的红条件（LSH 召回 / 复核已接线 / 打桩变红）全部指向 3g 才落地的管线——退出判据在自己那批不可评估 | 3e 只交付 **oracle 档 + GT**；LSH/复核类红条件全部移到 3g（§6） |
| F29 | 离线 oracle 与 Rust 粗筛两套 shingle 实现，无相等门 | 单一实现：Rust `docdup::shingle` 同时供 oracle 与产品；oracle 是同一函数的 O(n²) 全枚举驱动。另加 judgment-first 的**旋钮回显相等断言**（`docdup.result.knobs.shingleK` == Rust `DOC_SHINGLE`） |
| F30 | docdup 100 行样本无分层无每 kind 地板，整类可零席位 | 分层 = `unit_kind` × lang，**`min_per_kind = 15`**，后备 20/kind |
| F31 | oracle 代价无界：全段集 O(n²) 精确 Jaccard，无尺寸预算、无产物上限 | oracle 走**两阶段**：先按 `min_doc_tokens` 与语料内段数上限 `DOCDUP_ORACLE_SEGCAP` 分片，超限语料**扣发**并写明（而非静默降采样）；产物只记 `J ≥ jaccard_universe_floor` 的对 |
| F32 | 只实现 plan `:79-80` 四条豁免路中的两条（缺行内 `ce:allow(docdup) -- <why>`、缺基线豁免存量） | 采 detection-first 的四路豁免全表（§5.2），逐类台账键 |
| F33 | 把结构化 docstring 骨架当整块排除类；plan `:79` 字面是"模板**行**" | 行级剥离（同 F12） |
| F34 | cabal 编辑算术："10 app + 3 test = 26"，实为 10×2+3 = 23 | 本文 §2.3 逐项列出 |
| F35 | 引 `eval_graph_precision.rs:107` 作"degraded 直接拒绝"的先例——该行在 `method` 字串内，讲的是物化污染 | 锚点改为 `eval_graph_precision_parts/mod.rs`（G12 实现处） |

### 0.4 judgment-first 专有

| # | 缺陷 | 处置 |
|---|---|---|
| F36 | **`docdup.request {"docs":[[u64]]}` 违反 ADR-002 A6**。plan `:176` 字面："token 流只入本地索引，**不跨进程**"；按文档序的词哈希序列就是哈希形式的 token 流。且 plan `:177` 只把 docdup 的**复核判定**给 Haskell | 弃。wire 走升序去重 shingle **集合**（detection-first §2.3.5）。`DOC_SHINGLE` 因此是 Rust 侧预注册常数，由 slice 档双消费（`min_per_lang=15` 的绑定体例），并以 F29 的回显相等断言防漂移 |
| F37 | `lld` 边界契约数学上错误："`lld[i] ≤ lld[i+1]` 失败即 contract"。反例：根 r，子 a（有子 c）与 b；后序 c,a,b,r，lld = [0,0,2,0]——最后一棵非平凡子树的树全被拒 | 契约只保留 `0 ≤ lld[i] ≤ i` + **前序/后序可重建性机检**（`Graph.hs:65-104` 的逐行首违例体例）。单调性断言删除 |
| F38 | `nth` 序号型克隆指纹对回归失明：基线 {(A,B,1),(A,B,2)}，删 1 加一条全新克隆 ⇒ 集合不变 ⇒ `added=∅` ⇒ 净新违规过门 | §7.2 的指纹以两侧 `(path,key,nth_in_file)` 为身份，与块序无关 |
| F39 | 三次 proto bump，理由是"2.2.0 有时含 docdup 即 F8 偏斜"。实读 `VERSIONING.md:32-35`：capabilities 是**纯信息发现**，接受/拒绝的唯一权威是 §2 SemVer；`fourclass/2` vs `/1` 正是"固定 proto 下能力缺席"的已演练案例 | **一次 minor bump 2.1.0 → 2.2.0，三族同批声明**（`graph/1` 于 2a 声明、2g 才实现的先例，`Graph.hs:11`） |
| F40 | 引 `contracts/SOURCES.md`——仓内只有 `contracts/fixtures/crosscheck/SOURCES.md`，且 `ce.toml:7` 把该子树排除在自扫之外，两条路径的狗粮后果不同 | 全文用实际路径 |
| F41 | 称"freeze 里没有 vector/可变数组"——`core/cabal.project.freeze:95` 有 `vector==0.13.2.0`，`:13` 有 `array==0.5.8.0` | 如实陈述；`array` 为 F5 的预授权晋升目标 |
| F42 | 60 对 docdup 样本挂 ≥0.90 门：比 T3 的 0.85/100+ 更严的门配更薄的样本，且低于仓内 15/层地板 | 采 100 行 + `min_per_kind=15` + 门 ≥0.85（与 T3 齐），理由写进决策 ④ |

---

## 1. 模块布局与 300 门余量（E01：300 警 / 750 拒；`core/**` CI 硬 300）

### 1.1 Rust 新增（`cli/src/`）

`dedup/struct_fp.rs` 110（前序具名 kind 序列、kind→u64、结构 shingle、kind 直方图）·
`dedup/minhash.rs` 120（签名+分带+确定性盐；**T3 与 docdup 唯一实现**）·
`dedup/unitcache.rs` 130（单元签名/直方图/节点数的 SQLite 读写；**不动 `index.rs` 254**）·
`dedup/t3/mod.rs` 160（单元枚举、三路候选、两道可容许剪枝、结果回映）·
`dedup/t3/tree.rs` 140（子树→后序 `(lab, lld)` + 请求内 kind 字典 + `unitNodeCap`）·
`dedup/t3/wire.rs` 90 ·
`docdup/mod.rs` 150（编排 + `ce.docdup-report/0.1.0`）·
`docdup/segments.rs` 170（三类文本单元；**只调 `md::masked_content_lines`**）·
`docdup/spec.rs` 110（`DocSpec`：docstring_hosts / skeleton_prefixes / license_markers）·
`docdup/exempt.rs` 90（四路豁免 + 逐类台账）·
`docdup/shingle.rs` 110（词化 + fnv1a + shingle + 逐字 run；oracle 与产品同一实现）·
`docdup/wire.rs` 90 ·
`graph/nodes.rs` 110（**从 `deadcode.rs:54-150` 抽出的身份咽喉**，F19）·
`graph/symbols.rs` 60（`symbols` 读面——今天没有：`load.rs:36-58` 只读 files/edges/sites）·
`join/mod.rs` 140（三腿装配、Tier F/U）· `join/churn_unit.rs` 150（逐 (file,unit) churn）·
`score/mod.rs` 130（`ce score` / `ce check` / `--fail-under`）·
`score/baseline.rs` 160（`ce-baseline.json` I/O + 成员身份 + 名字回挂）·
`score/wire.rs` 110 · `main_score.rs` 120（三个子命令体，`main_cmds.rs` 220 不动）·
`scan/spec_hs.rs` 55（Haskell `LangSpec`——`spec.rs` 259/300，GO 块 39 行，塞不下）。
**新 Rust 产品行 ≈ 2,505 / 21 文件。**

### 1.2 Rust 改动（拆分是退出判据不是意向）

| 文件 | 今 | 后 | 变更 |
|---|---|---|---|
| `graph/deadcode.rs` | **286** | ~180 | 身份咽喉迁 `graph/nodes.rs`（F19）；join 与它共用，不再各铸 id |
| `graph/md.rs` | 240 | ~258 | 加法 `masked_content_lines`（F3）；`GRAPH_REV` 不动 |
| `scan/spec.rs` | **259** | ~262 | 只加一条 dispatch 臂；表体住 `spec_hs.rs` |
| `graph/spec.rs` | 87 | ~95 | Haskell 的**显式**无站点臂（非 `_ =>`） |
| `dedup/schema.rs` | 117 | ~145 | `SCHEMA_VERSION 4→5`；`meta_entries` `[…;4]`→`[…;6]`（+`struct_rev`,`docdup_rev`）；新表 DROP |
| `dedup/pairs.rs` | 240 | ~272 | `by_hash` 分组从两处局部（`:75-78`/`:95-98`）提为 `pub(crate)` 单例；`extend_anchor`（`:186-190`）加近似锚点第二汇 |
| `dedup/mod.rs` | 246 | ~205 | `Report`/`emit`/`Params` 迁 `dedup/report.rs` ~90，腾位给 T3 相 |
| `churn.rs` | 227 | ~185 | 逐单元分解迁 `join/churn_unit.rs` |
| `main.rs` | 154 | ~205 | +`docdup`/`join`/`score`/`check`/`baseline` |
| `config.rs` | 100 | ~135 | `[docdup]` `[score]` `[t3]` |
| `scan/lang.rs` | 53 | ~64 | **只准追加** `Haskell = 6` + `LangUnknown = 7` 哨兵 |
| `graph/store.rs` | 272 | ~285 | `symbols` 加 `nth` 列（F2）+ UNIQUE 索引 |

### 1.3 Haskell（`core/app` 今 **984** / 15 文件；`core/test` 今 **543** / 4 文件）

`CE/Clone.hs` 150（族：解码、容量、边界契约、结果、降级——`CE/Graph.hs` 155 是一个
完整族的实测体量）· `CE/Clone/Ted.hs` 140（Zhang-Shasha，`Data.IntMap.Strict`）·
`CE/Clone/Prefilter.hs` 50（两条可容许界，带证明注释）· `CE/Clone/Cost.hs` 55 ·
`CE/Docdup.hs` 120 · `CE/Docdup/Jaccard.hs` 55 · `CE/Docdup/Cost.hs` 50 ·
`CE/Verdict.hs` 160（`verdict/1` 族）· `CE/Verdict/Join.hs` 95 ·
`CE/Verdict/Score.hs` 90 · `CE/Verdict/Ratchet.hs` 110 · `CE/Verdict/Cost.hs` 80。
改动：`Protocol.hs` 94→~106（3 条 guard）、`Handshake.hs` 61→63。
**新 app ≈ 1,155 行 / 12 模块。**注释密度house 比 40–90%（`Graph/Cycles.hs` 16 行装
5 行代码，`Graph/Build.hs` 49 行装 build+reach+SCC），故 `Ted.hs` 140 是最紧的一格；
越 300 则拆 `Ted/Forest.hs`（预写预案）。

`core/test/`：`ReferenceTed.hs` ~150 · `ReferenceJaccard.hs` ~90 · `CloneProps.hs` ~130 ·
`VerdictProps.hs` ~140 · `Spec.hs` 143→~172。**`Spec.hs` 是 300 门咽喉**（golden 列表
`:32-35`、`costModel` `:46-57`、`structural` 手工合取链）——预写拆分预案
`Spec.hs` + `SpecStructural.hs`，列进 3a 退出判据。

### 1.4 评估仪器（F1：单列，不藏在"测试"里）

`cli/tests/eval_support/dedup.rs` ~170（共享咽喉）· `eval_t3_precision.rs` + `_parts/` ~520 ·
`eval_t3_sample.rs` + `_parts/` ~480 · `eval_t3_recall.rs` ~230 · `eval_docdup.rs` + `_parts/` ~600 ·
`eval_docdup_sample.rs` ~330 · `dedup_provenance.rs` ~160 · `baseline_bridge.rs` ~120。
GT JSON：`eval_t3_review/{5 语料}.json` ~1,300 行 · `eval_docdup_review/{5 语料}.json` ~900 行。
**仪器 ≈ 2,610 Rust + 2,200 GT 行**，锚 = M5-2 单仪器实测 1,902 + 1,235。

**M5-3 总量 ≈ 2,505 产品 Rust + 1,155 app Haskell + 510 test Haskell + 2,610 仪器 Rust
+ 2,200 GT = 8,980 行。**这是决策 ① 的依据数字，如实陈列。

---

## 2. Wire——一次 minor bump `2.1.0 → 2.2.0`，三族

### 2.1 为什么一次 bump、三族（驳 F39）

加法 type + 加法 capability = minor（`VERSIONING.md:66-73`，`:8-9` 记 graph/1 就是这么
走的）。capabilities 是**纯信息发现**，接受/拒绝的唯一权威是 §2 SemVer，能力缺席 =
客户端响亮降级（`VERSIONING.md:32-35`）——`fourclass/2` vs `/1` 是已演练案例。故
"2.2.0 有时含 docdup"不是偏斜，是文档化的正常态。三族在 3a 同批声明（handler 对
一切输入回 `contract`），语义在后续批次填入——`Graph.hs:11` 的先例逐字复用。

**为什么 `clone/1` 与 `docdup/1` 不合并**：一个是树、一个是集合，边界机检完全不同；
合并会退化成 optional-field 多态，`Graph.hs:65-104` 那种"按请求顺序报第一违例"的
确定性会丢。**为什么 join+score+棘轮合成 `verdict/1`**：三者共用同一张事实表，分族
会让同一批事实过线三次；且这正是 ADR-008 要接管的形状——请求携事实、策略住
`CE/Verdict/Cost.hs`，DSL 落地时换求值器不换 wire，`plan:233` 的"迁移前后判决字节
等价"才可达。

### 2.2 形状（整数与索引，无文本形物——ADR-002 A6，`VERSIONING.md:43-45,:53`）

```jsonc
// clone/1
{"proto":"2.2.0","type":"clone.request","id":N,
 "trees":[{"lab":[Int],"lld":[Int]}],   // 后序；lab=请求内稠密 kind 码；lld[i]=最左叶后代后序下标
 "pairs":[[i,j]]}                        // 严格升序、去重
{"proto":"2.2.0","type":"clone.result","id":N,
 "scores":[[i,j,ted,n1,n2]],             // 回传原始 ted 与规模，不回传比值——cut 表一跑重算
 "counts":{"trees":T,"pairs":P,"judged":J,"prefiltered":F},
 "knobs":{"tsedNum":85,"tsedDen":100},
 "degraded":false}                        // true 时 reason ∈ {clone_too_large}

// docdup/1
{"proto":"2.2.0","type":"docdup.request","id":N,
 "sets":[[u64 升序去重]],                 // shingle 哈希集合，非序列（F36）
 "pairs":[[i,j,verbatimRun]]}             // verbatimRun 由 Rust 算（F26）
{"proto":"2.2.0","type":"docdup.result","id":N,
 "scores":[[i,j,inter,union]],
 "counts":{"docs":D,"pairs":P,"judged":J},
 "knobs":{"shingleK":5,"jaccardNum":80,"jaccardDen":100,"verbatimFloor":50},
 "degraded":false}                        // reason ∈ {docdup_too_large}

// verdict/1
{"proto":"2.2.0","type":"verdict.request","id":N,
 "sim":[[u,v,simKind,num,den]],           // simKind ∈ {0 t1t2, 1 t3, 2 docdup}
 "pos":[[u,indeg,outdeg,sccId,sccSize,reachIn]],   // sccId 保留（F15）；Tier U 不出现在此表
 "tier":[[u,tierCode]],                   // 0=F(三腿齐) 1=U(图腿缺席)
 "churn":[[u,rewrite,append,added,survived]],
 "cochange":[[u,v,count]],
 "continuous":[[u,metricCode,value]], "discrete":[u64 升序],
 "baseline":<ce-baseline.json 原样、未解释>,       // ADR-008 反抢跑（plan:232）
 "weights":[[metricCode,numerator]], "floor":perMille|null}
{"proto":"2.2.0","type":"verdict.result","id":N,
 "candidates":[[u,v,verdictCode,reasonBits,legsMask]],
 "score":perMille,"axes":[[axisCode,penalty]],
 "ratchet":{"added":[u64],"removed":[u64],"over":[[u,metricCode,value,allowed]],
            "toleranceDrawn":[[u,metricCode,drawn]],"fail":bool},
 "newBaseline":{"continuous":[…],"discrete":[…]},
 "degraded":false}                        // reason ∈ {verdict_too_large}
```

**边界契约**（`error/contract`，指名违约者，`Graph.hs:65-104` 体例）：
`length lab == length lld`；`0 ≤ lld[i] ≤ i`；**后序可重建性机检**（单调性断言已删——F37）；
一切字段非负（`Graph.hs:80,89` 的 `any (<0)` 拒绝式，故不能用 `parent = −1` 表示根）；
pair 端点在界内且严格升序（`notAscending` 装置，`Graph.hs:96-99`）；
`sets[i]` 严格升序去重。超 `Cost` 容量 ⇒ `degraded` **完整键集**回复，绝不截断
（`Graph.hs:137-155`，`VERSIONING.md:55-56`）。

### 2.3 成本清单（逐项，可直接切批）

| 触点 | 变更 |
|---|---|
| `core/app/CE/Protocol.hs:26` + `cli/src/corelink.rs:18` | 孪生 `proto` 常数 → `"2.2.0"`，必须同批 |
| `core/app/CE/Protocol.hs:73-79` | +3 条 guard（各 4 行，复用 `rid <|> envId env` 回退式）；94 → ~106 ✔ |
| `core/app/CE/Handshake.hs:29-30` | capabilities +3 串 ⇒ hello 应答字节改变；61 → 63 ✔ |
| golden **reply** 行 | **22 行**（fourclass 9 / graph 8 / hello-ok 1 / wire-errors 4）；request 19 行**有意不动**（F9） |
| `contracts/fixtures/{clone,docdup,verdict}/golden.ndjson` | 新建，各 ≥5 对（含畸形树 / 非升序集 / 越界索引三类 contract），**双侧消费**（`core/test/Spec.hs:32-35` + `cli/tests/core_wire.rs`） |
| `core/ce-core.cabal:12-17` **与** `:34-40` | `other-modules` 双 stanza：12 app 模块 ×2 + 5 test 模块 ×1 = **29 处编辑**（F34 的正确算术） |
| `cli/tests/core_wire.rs` | +3 `assert!(link.has("…/1"))` |
| `contracts/VERSIONING.md` | 顶部横幅（`:5-9` 体例）+ §1 三条族项（`:51-64` 体例，含 reason 词汇）+ §3 加"request 行有意留 2.1.0"的立场声明 + §4 proto 行（`:92`） |

---

## 3. 存储——schema v5，单 DB（daemon 唯一写者）

```sql
-- 既有 symbols 加身份消歧（F2）
ALTER 语义由 wipe 模型承载：CREATE TABLE symbols (…, nth INTEGER NOT NULL);
CREATE UNIQUE INDEX idx_sym_ident ON symbols(file_id, key, nth);

CREATE TABLE unitsig (                     -- T3 单元结构指纹缓存
  file_id INTEGER NOT NULL REFERENCES files(id) ON DELETE CASCADE,
  key TEXT NOT NULL, nth INTEGER NOT NULL,
  nodes INTEGER NOT NULL, sig BLOB NOT NULL, hist BLOB NOT NULL);
CREATE TABLE docsegs (                     -- docdup 段与 shingle 集
  file_id INTEGER NOT NULL REFERENCES files(id) ON DELETE CASCADE,
  kind INTEGER NOT NULL, start_line INTEGER NOT NULL, end_line INTEGER NOT NULL,
  words INTEGER NOT NULL, shingles BLOB NOT NULL, exempt INTEGER NOT NULL);
CREATE INDEX idx_unitsig_file ON unitsig(file_id);
CREATE INDEX idx_docsegs_file ON docsegs(file_id);
```

`SCHEMA_VERSION 4 → 5`；DROP 半住 `dedup/schema.rs` 的 `SCHEMA` 常量，CREATE-only DDL
住各自域（`store::GRAPH_SCHEMA` 体例，`store.rs:32-50`，在 `schema.rs:62` 执行）。
`meta_entries` 从 `[(&'static str, i64); 4]`（`schema.rs:79-86`）扩到 `; 6]`，新增
`struct_rev` / `docdup_rev`——抽取语义一改即清陈旧行。IMMEDIATE 先锁锁内复查的
竞态根修（`schema.rs:55-72`）与 WAL 切换有界重试（`index.rs:53-64`）原样继承。

**`TOKENIZER_REV` 绝对不动**（`tokens.rs:20` = 2）。注释今天在唯一一处被整子树丢弃
（`tokens.rs:62-64`），改它就要 bump，而它在缓存键里 ⇒ 全索引清空 + T1/T2 全部重标定
+ 187 棘轮重来。**docdup 走独立加法抽取器**，与 `tokenize` 各走各的树。3d/3g 各有
"`tokens.rs` 零 diff + `ce dedup --check` 恰报当期预算"的红条件。

一次性全量重建的立场沿用 M5-2 决策 8：索引即缓存，pre-1.0 接受。
