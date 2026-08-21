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
`ce.eval-t3-universe/1.0.0`，生成于 `a7c2672`；门 `eval_t3_universe`）：
units self 2,028（141 文件）/ requests 807（50）/ ripgrep 4,397（133）/
cobra 595（53）/ zod 7,078（393）；常数 floor_nodes 24、s_max 60、m_max 200。

**候选**（`t3-candidates-*-v1.json`，`ce.eval-t3-candidates/1.0.0`，生成于
`ea494f4`；门 `eval_t3_candidates`）：四源并集（S1 段二汇 / S2 同键 / S3 裸指纹
/ S4 MinHash-LSH (128,32,4)）+ 两道可容许剪枝与判决同界（85/100 整数交叉相乘）。
survivors self 490 / requests 128 / ripgrep 6,201 / cobra 1,158 / zod 21,740
（zod 剪枝实证：pruned_size 1,844,192 + pruned_label 264,236 → 21,740）。
`pairs_sha256` 五 digest = 抽样册 `pool_digests` 锚链；min_reported 地板
53 / 18 / 617 / 63 / 3,253。

**抽样**（`t3-sample-v1.json`，`ce.eval-t3-sample/1.0.0`；原门 `eval_t3_sample`
随 v0.5.0 瘦身退役，工件保留，完整性腿迁入
`eval_t3_precision::t3_sample_verifies`）：
main 100 + backup 20×4，配额 ts 44 / rs 24 / go 17 / py 15（最大余数 ∝ 池
21,750 / 6,676 / 1,161 / 130），哈希序无 RNG。

**精度 = 达标线 A**（`t3-precision-*-v1.json`，`ce.eval-t3-precision/1.0.0`，
生成于 `dfba03a`；门 `eval_t3_precision`）：answered 61 全 correct、wrong 0 →
**总体 precision 1.000**（合同 ≥0.85）；分母 ≥5 语料逐个 1.000（requests 11 /
ripgrep 18 / cobra 12 / zod 19；self answered 1 不设门）。余量 not_clone 31
（zod boilerplate 16 全落 TSED<0.85 的机制性分离）+ dropped 8 = 100 全记账。
θ 全表 70..100 入档、逐格 wrong 恒 0（合同格点 85 在档）；审计 = 五独立审计员 100/100
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
`b70909e`，DOCDUP_REV **4**（rev 2→3 修正案后重冻结；rev 3→4 = NFC 组字修复
（2026-08-21，M9 批 1），按 EVAL-SET.md 复活协议重冻结——五语料逐文件字段与
summary 零漂移实证，precision 32 行普查同证）；门 `eval_docdup_universe`）：live
段 self 114 / requests 98 / ripgrep 251 / cobra 59 / zod 117；常数 DOC_SHINGLE 5、
verbatim_floor 50、DOC_LINE_CAP 200、license 头 5 行豁免、SEGCAP 8192。

**oracle**（`docdup-oracle-*-v1.json`，`ce.eval-docdup-oracle/1.0.0`；原门
`eval_docdup_oracle` 随 v0.5.0 瘦身退役，工件保留，活读者 =
`eval_docdup_precision` 直读冻结 oracle JSON 做 D2 回声断言）：live 段全对
枚举整数交叉 Jaccard ≥80/100 ∨ verbatim ≥50，
oracle 对 self 4 / requests 4 / ripgrep 8 / cobra 8 / zod 8 = **32**。

**普查**（`docdup-sample-v1.json`，`ce.eval-docdup-sample/1.0.0`；原门
`eval_docdup_sample` 随 v0.5.0 瘦身退役，冻结件保留、由
`eval_docdup_precision` 活读）：32 全量普查零选择（勘误：报告地板人口
30<100 抽样算术不可能）；by_kind comment 17 / docstring 8 / md_para 7；
population = report_floor 23 + margin 9。census v2 = v1 的严格子集
（retired 15 对冻结存证）。

**精度 = 达标线 B 修正案**（`docdup-precision-*-v1.json`，
`ce.eval-docdup-precision/1.0.0`，生成于 `0b13d4b`；门 `eval_docdup_precision`）：
D3 scoped **17/17** correct（ripgrep 7 + cobra 4 + zod 6）+ docstring **6/6**
（self 3 + requests 3，不设门）+ not_reported 9 台账化；D1 oracle 召回
self 3/3、requests 3/3、ripgrep 7/7、cobra 4/4、zod 6/6 全 100%（硬门 0.99）；
修正案 = 三条类别级行掩码（html_line / fenced_code_line / overlong_line），
DOCDUP_REV 2→3 五语料重冻结，J-floor 全表 50..100 逐档 wrong 恒 0。

## Churn 台账（3h 盲审）

`churn-ledger-v1.json`（`ce.churn-ledger/1`；冻结件与门 `eval_churn_ledger`
已随 v0.5.0 瘦身退役，全档在 git 历史）：self tip
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
  `contracts/eval/pre-haskell-members-v1.json` 冻结 3j 收口树（`fb9c139`）的
  40 个 pre-Haskell discrete 成员；门 `baseline_bridge::
  pre_haskell_members_survive_every_generation` 断言其为每个后继基线的真子集。
  预算 97→149 双笔具名（+13 generation / +39 表族，`ce.toml` 历史段；本行
  3l 勘误——本册曾记 150/+40，权威数是 fmt 后量得的 149/+39）；churn
  台账重放改按工件冻结语言域比对（五语言白名单，`eval_churn_ledger`——
  已退役，见上节）。
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
全局包库 43 包 1,371 模块；滤除 = 库自记 exposed 位（`ghc`）+ 零模块包两条
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

## M5 收口之后（拆册指针）

M5 收口欠账清算、3m recall 仪器 B、审查热修+CI 门补全、ADR-008 首步、
挂账清零批的冻结登记 → [EVAL-SET-M5-CLOSE.md](EVAL-SET-M5-CLOSE.md)
（本册 300 行线的二次拆册，2026-08-17）。

## 复跑

```
cd cli && cargo test --test eval_t3_universe --test eval_t3_candidates \
  --test eval_t3_precision --test eval_docdup_universe \
  --test eval_docdup_precision --test baseline_bridge \
  --test core_size_gate --test core_wire
```

外部四语料需本地 `.ce-eval/` 克隆（tip 见上表，CI 门 `rev-parse` 复核 RM19）；
冻结件重生成：生成器已随 M7.5 退役、五条整件门（t3_sample/t3_recall/
docdup_oracle/docdup_sample/churn_ledger）随 v0.5.0 加码批退役——均先
`git checkout <父提交> -- cli/tests/<仪器>` 复活再跑（EVAL-SET.md 再生成节
同律），diff 为空即完整复现。
