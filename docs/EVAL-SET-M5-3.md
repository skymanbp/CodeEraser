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
  `.hs` size-only 行走 continuous 指纹表（`Lang::Haskell` 留 3k 语料 generation
  预注册，见设计卷 §8.4）。

## 复跑

```
cd cli && cargo test --test eval_t3_universe --test eval_t3_candidates \
  --test eval_t3_sample --test eval_t3_precision --test eval_docdup_universe \
  --test eval_docdup_oracle --test eval_docdup_sample \
  --test eval_docdup_precision --test eval_churn_ledger \
  --test baseline_bridge --test core_size_gate
```

外部四语料需本地 `.ce-eval/` 克隆（tip 见上表，CI 门 `rev-parse` 复核 RM19）；
冻结件重生成走各 `--ignored` 生成器测试，diff 为空即完整复现。
