# M6 结构管理器设计册——树尺度的熵判决（structure/1）

> **两拍板**（用户，2026-08-17，AskUserQuestion 结构化）：
> ① 语言分工 = **判决全 Haskell、测量复用既有 Rust 管线**（walk/graph/churn/
> dedup/deadcode/docdup 是事实源；structure/1 新家族的全部数学与判决表纯
> Haskell——零抵触硬性约束 1 与 ADR-008 判决/测量分界）；
> ② 轨道 = **并入 M6**：结构家族即 GUI 首屏数据面（机器可读 JSON 树报告
> 与 Tauri 可视化一次建成），设计册先行 → 计划 v1.9 → 切片实施。
>
> 本册是参照系选型、信号普查与切片契约的单一权威（ADR-008 设计册体例）。

## 1. 使命定位

现有七家族全部在**文件/单元尺度**判熵。结构管理器把同一使命抬到**目录树
尺度**：打开任意项目跑一次全量扫描，产出（a）**规范化程度**（结构分，
per-mille，scoreScale 纪律）、（b）**树相对熵**（定标有理数，见 §2）、
（c）逐目录可下钻的轴分解——CLI 报告 + GUI 首屏共用一份 JSON 数据面。

## 2. 参照系选型（相对熵"相对于什么"——本设计的第一决策）

| 方案 | 形 | 判 |
|---|---|---|
| A 声明式模板 | ce.toml `[structure]` 自述理想布局（目录→角色、glob→归属），KL(观测‖声明) | **可选覆盖层**：声明后获得目标化散度与指名偏差（"tests/ 里住着 3 个非测试文件"） |
| B 内建生态先验 | 内置 rust-crate/cabal/node… 原型分布 | **不做**：猜原型违反"歧义一律不猜"立场（RG 先例）；跨生态先验必然漂移 |
| C 自参照统计 | 无外部 Q：树自身的规整度——兄弟命名分布熵、深度/扇出集中度、目录话题分离度 | **地板**：零配置必产完整报告 |

**定案 = C 地板 + A 覆盖**（Cost 默认 + ce.toml 覆盖的既有 knob 模式在
树尺度的重演）。整数纪律：core 无浮点铁律不破——熵与散度全程
`Data.Ratio`/定标整数（ReferenceJaccard 先例），发布值为 ‰ 定标。
**S1 具体化（实现定案）**：Shannon/KL 需对数=无理数不可精确判定，故取
同族有理闭成员——熵 = **Tsallis-2**（= Gini–Simpson，1−Σp²，两次独立
抽取相异概率）、散度 = **χ²**（Σ(p−q)²/q，f-散度族，序语义与 KL 同向；
观测质量落在参照零质量 bin 上 = Nothing，由 S3 指名偏差行承接而非塌成
数字）；参照电池 = 代数 vs 逐对枚举在 125 向量穷举族上相等
（`CE.Structure.Entropy` + `EntropyProps`）。

## 3. 信号普查与轴表（七轴，S0..S6；每轴一具名谓词一旋钮，Score.hs 体例）

| 轴 | 信号 | 事实源（测量侧） | 新建量 |
|---|---|---|---|
| S0 路径几何 | 深度分布集中度、扇出失衡（定标基尼）、超长/超深路径计数 | walk 既有 | 树形聚合（Rust 小） |
| S1 命名一致性 | 每兄弟集的命名模式熵（case/分隔符/前缀族） | walk 文件名 | 模式分类器（Rust 小）——**名不过线**，只过模式码分布 |
| S2 正交性/模块度 | 目录内 vs 目录间引用密度（Newman 模块度，目录即划分） | graph 边表既有 | 逐目录边聚合（Rust 小） |
| S3 漂移错位 | 引用多数邻域在别目录的文件计数（社区错配） | graph 边表既有 | 同上聚合 |
| S4 文档基建 | 大分类目录（top-k 扇出）README/配置存在性；约定表 | walk + entry_globs 同款约定机制 | 约定位掩码（Rust 小） |
| S5 文档新鲜度 | md 节引用目标在文档最后一改之后的变更次数 | churn 窗口 + md 阶梯既有 | join（Rust 中） |
| S6 冗余/孤儿卷积 | dedup 块数、deadcode 判决按目录卷积 | dedup/deadcode 既有判决 | 逐目录 rollup（Rust 小） |

判决侧（Haskell，全新）：`CE.Structure.Cost`（轴旋钮+参照默认）、
`CE.Structure.Entropy`（有理数熵/散度原语+穷举小树参照电池）、
`CE.Structure`（structure/1 respond：分级判决表给逐目录 level、七轴罚分
+结构分、A 层声明存在时的 KL 与指名偏差行）。CE.Wire respondWith 直入
（第七家族零骨架复刻——第十咬的机制红利）。

## 4. wire 形（structure/1，加性 minor；§5.9.2 名不过线）

- request（S2 as-built）：`nodes` 稠密表 `[id, parent, depth, subdirs,
  files]`（目录树，parent 先于子、根自环深 0；路径名留 Rust 按位置回排）、
  `patterns [dirId, code, count]` 命名分布行、`conventions [dirId, bits]`
  约定位、`fileRefs [dirId, inside, outside, count]` 逐目录触点分桶
  ——**`dirEdges` 撤出 v1**：S2 混流轴以 fileRefs 触点为单一事实基，
  目录×目录边表等 S3+ 配 Newman 模块度一起加行（一轴一基，不留两套
  引用事实并存的缝）；可选 `declared`（A 层模板行，S3）+ knob 表
  （既有文法，code 0..8）。
- reply（S2 as-built）：五判轴 `axes [code, penalty]`（S0 几何/S1 命名/
  S2 混流/S3 错位/S4 文档；S5/S6 行 S3+ 加入，缺席=不判）、`score`
  （Score.hs 公式形，等权判轴数）、`entropy` 头行（定标 ‰：全局命名+
  目录文件数分布）、`findings [dirId, axis]` 稀疏下钻、全量 knobs 回显、
  degraded 自带 fail=true（P1 立场）。

## 5. 可视化数据面（CLI 与 GUI 共用一份）

`ce structure [--format json] --core <exe>`：JSON = `{schema, score, entropy,
axes, tree: [{id, parent, name, kind, level, score, axes…}], deviations}` ——
flat nodes+parent（机器友好、流式可增量）；CLI 渲染 = 顶部总分/熵 + 逐轴
概要 + top-N 违规下钻；GUI（Tauri，M6 主体）= 树图/treemap 首屏直接消费
同一 schema，无第二报告形。

## 6. 切片（每片一提交链一 CI 绿，ADR-008 纪律）

- **S1 测量+熵核**：Rust 聚合面（§3 新建量）+ `CE.Structure.Entropy`
  有理数原语+参照电池（穷举小树 vs 定义式）——先证数学再接线。
- **S2 家族接线**（已落地，proto 2.9.0）：structure/1 wire（§4 as-built
  形）+ 五轴罚分判决 + golden 五对 + `ce structure` CLI
  （JSON=ce.structure-report/0.1.0 / console）——报告态，不设门：
  新轴先观测后挂门，recall 仪器教训。
- **S3 A 层覆盖**：ce.toml `[structure]` 声明→KL 与指名偏差；
  deny_unknown_fields 与响亮降级从第一天起（反审 C2 教训）。
- **S4 GUI 首屏**：Tauri 树图消费 §5 schema（M6 主体并轨；验收=计划 M6
  行既有数字：10 万 LOC 冷启动到首屏 <60s、报告打开 <3s）。

## 7. 风险与预先立场

- **参照系空转**：C 层轴在小仓可能全 0——每轴带 F16 非真空前置电池。
- **镜像纪律**：本家族 Rust 侧**不建判决镜像**（无冻结仪器需求——P1 的
  镜像存在理由在此不成立），判决位唯一权威即 core，省一类反审 C1 缝。
- **规模**：nodes ≤ 目录+文件数（自仓 ~250，10 万 LOC 仓 ~1 万行级）——
  远低于 verdict 帽；分块非必需但 wire 走 scan/1 分块先例。
- **计划棘轮**：M6 行就地改写（315 只准变短）；ccm 计划重锁随 v1.9。
- **每片验收**：反事实杠杆（翻参照/旋钮指定判决必变）+ golden 手算
  + 新契约扫全读者（反审三例教训制度化）。
