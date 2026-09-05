# v2.29 同角色顾问 ROI 度量册（第三次拆册，2026-09-05）

> 计划 v2.29 步 2「ROI 度量先行：不达标即止步入账」的冻结登记。母册链：
> [EVAL-SET.md](EVAL-SET.md) → [EVAL-SET-M5-3.md](EVAL-SET-M5-3.md) →
> [EVAL-SET-M5-CLOSE.md](EVAL-SET-M5-CLOSE.md)（303 行）→ 本册。度量定义唯一权威 =
> `.ccm/similar-spec-2026-09-05.md`（本机件，§三 词袋 / §四 PPMI / §七 评估）。本册每个数字直读
> `contracts/eval/similar-sample-v1.json`（样本）、`contracts/eval/similar-oracle-v1.json`（oracle）与
> `cli/tests/it/eval_similar_review/sample.json`（仲裁记录），由渲染脚本一次写出后冻结；
> 重放门 = CI `cargo test --test it -- eval_similar_precision::`。本册与三册前身同入冻结集
> （`frozen_set.rs`：不扫芯片、不生成、退出引文门），行号引文一律不写。

## 仪器（`cli/tests/it/similar_replay.rs`，常驻 `--ignored` 腿，release 跑）

五语料各自成库：自仓走产品 walk + scratch 索引（`refreshed_index` → `unit_rows` → 索引看到的文本 →
`similar::file_bags`，单元多重集与索引逐一对拍，漂移即拒）；四份 crosscheck 夹具目录被 ce.toml 排除在自仓外，
各自单独成库。每个单元对本库其余单元查两臂 top-k：**裸臂** = 自身袋按通道乘子；**扩展臂** = 裸臂 + 每个
拼出的词通道词的 top-m PPMI 邻词（权重 ≤ 一半，邻词只计分不成证据）。候选 = 两臂 top-k 并集，每候选一行
六通道命中整数 + 形状全等位 + 同角色位 + 同文件位 + 两臂名次与分数。同角色位 = (N ≥ 1 ∧ C ≥ 1) ∨ (N ≥ 2 ∧ 形状全等)，
与步 5 将落在 Haskell `CE.Similar` 的合取同式（步 2 只在 Rust 镜像以便度量）。

| 常量 | 值 | 常量 | 值 |
|---|---|---|---|
| `SIMILAR_REV` | 1 | k（top-k） | 5 |
| k1 | 6/5 | b | 3/4 |
| idf 定点位 | 8 | 分数定点位 | 16 |
| `W_UNIT`（查询权重单位） | 256 | `TERM_CAP`（PPMI 每单元词上限） | 96 |
| `MIN_COOC` | 2 | `MIN_PPMI`（8 位定点 = 2 bit） | 512 |
| `PPMI_CAP`（= 4 bit） | 1024 | `PPMI_SCALE`（= 8 bit，扩展权重 = w·min(ppmi, cap)/scale） | 2048 |
| `TOP_M` | 3 | 文档归属 `LEAD_GAP` / `HEAD_GAP` | 3 / 2 |
| 样本配额 self（role 1 / role 0） | 35 / 35 | 样本配额每夹具 | 7 / 6 |

这些常量随样本冻结（`constants` 对象），CI 门 `similar_oracle_consistent` 断言它们等于仪器当下的常量——
改一个常量即换一套仪器，旧 oracle 随之作废。

## 全量普查（树 `a48f8fe`，dirty = true：similar 模块本身在度量时尚未提交；四份夹具是钉住的上游切片，自仓行按文件 sha 锚定）

| 语料 | 单元 | 裸臂 top-1 role=1 | 扩展臂 top-1 role=1 | 两臂 top-1 相同 | 裸臂 top-1 同文件 | 平均袋长 | PPMI 截断单元 |
|---|---|---|---|---|---|---|---|
| self | 5378 | 1114 | 1092 | 3968 | 1376 | 76 | 17 |
| go | 59 | 29 | 27 | 47 | 53 | 127 | 0 |
| python | 125 | 46 | 44 | 98 | 100 | 140 | 0 |
| rust | 426 | 131 | 123 | 320 | 343 | 109 | 3 |
| typescript | 26 | 3 | 3 | 26 | 13 | 180 | 0 |

读法：role 位在自仓只对约五分之一的 top-1 亮起，在 typescript 夹具（26 单元）只亮 3 次——夹具太小，
样本配额取不满属实。扩展臂改变的 top-1 是少数（两臂相同列），PPMI 的作用只能在仲裁后看精度差。

## 样本（118 个查询 / 700 个候选对，`contracts/eval/similar-sample-v1.json`）

每库把裸臂 top-1 按 role 位分两层，各层按 `sha256("similar|corpus|path|key|nth")` 的字典序取前缀到配额——
秩序由身份决定、与分数无关，仲裁者看不到分数（RM18）。

| 语料 | 查询 | 其中 role=1 | 候选对 |
|---|---|---|---|
| self | 70 | 35 | 424 |
| go | 13 | 7 | 76 |
| python | 13 | 7 | 78 |
| rust | 13 | 7 | 77 |
| typescript | 9 | 3 | 45 |

## 仲裁（`cli/tests/it/eval_similar_review/sample.json`）

仲裁者：codex exec gpt-6-astra (reasoning max, read-only sandbox), batches of 24 queries; the prompt carried identities and verbatim source only — no scores, no role bits, no arm placements (RM18)。每批一份 packet 带每个单元的逐字源码（`path:start-end key#nth` 头），
仲裁者必须读源码后才答；词表 = `same_role`（两单元在各自程序里扮演同一角色）/ `related`（同一机制的不同腿、
调用方与被调方、同族不同职）/ `unrelated`；另标 `clone`（代码本身近似相同——同角色的平凡形，单独计一档）；
`why` ≥ 40 字符否则合并脚本拒收。转录修复 2 处（仲裁者抄错 rank 哈希、
仅当唯一的 ≥ 16 位十六进制前缀匹配时按名修复，逐条列在记录的 `transcription_repairs`）。

| 判决 | 候选对 |
|---|---|
| same_role | 177 |
| related | 355 |
| unrelated | 168 |
| 其中 clone = true | 67 |

## 结果（`contracts/eval/similar-oracle-v1.json` `summary`，CI 门从行重导必须相等）

p@1 = 该臂第 1 名被仲裁为 same_role 的比例（分母 = 该臂有答案的查询）；hit@5 = 该臂 top-k 里至少一个
same_role；role 位混淆 = 同角色位对全部候选对的判决（tp / fp / fn / tn）。**不报 recall**：oracle 只知道
它见过的候选，没有一个单元的全部同角色伙伴，分母不存在。

| 语料 | 查询 | p@1 裸臂 | p@1 扩展臂 | p@1 裸臂 role=1 | p@1 裸臂 role=0 | p@1 裸臂非 clone | hit@5 裸臂 | hit@5 扩展臂 |
|---|---|---|---|---|---|---|---|---|
| all | 118 | 67/118 = 56.8 % | 63/118 = 53.4 % | 39/59 = 66.1 % | 28/59 = 47.5 % | 38/89 = 42.7 % | 74/118 = 62.7 % | 75/118 = 63.6 % |
| self | 70 | 40/70 = 57.1 % | 38/70 = 54.3 % | 24/35 = 68.6 % | 16/35 = 45.7 % | 27/57 = 47.4 % | 45/70 = 64.3 % | 46/70 = 65.7 % |
| go | 13 | 7/13 = 53.8 % | 6/13 = 46.2 % | 4/7 = 57.1 % | 3/6 = 50.0 % | 5/11 = 45.5 % | 8/13 = 61.5 % | 8/13 = 61.5 % |
| python | 13 | 4/13 = 30.8 % | 3/13 = 23.1 % | 3/7 = 42.9 % | 1/6 = 16.7 % | 3/12 = 25.0 % | 4/13 = 30.8 % | 4/13 = 30.8 % |
| rust | 13 | 7/13 = 53.8 % | 7/13 = 53.8 % | 5/7 = 71.4 % | 2/6 = 33.3 % | 3/9 = 33.3 % | 8/13 = 61.5 % | 8/13 = 61.5 % |
| typescript | 9 | 9/9 = 100.0 % | 9/9 = 100.0 % | 3/3 = 100.0 % | 6/6 = 100.0 % | 0/0 | 9/9 = 100.0 % | 9/9 = 100.0 % |

| 语料 | 候选对 | role 位 tp | fp | fn | tn | role 位精度 | role 位对候选的召回 |
|---|---|---|---|---|---|---|---|
| all | 700 | 101 | 64 | 76 | 459 | 101/165 = 61.2 % | 101/177 = 57.1 % |
| self | 424 | 70 | 30 | 49 | 275 | 70/100 = 70.0 % | 70/119 = 58.8 % |
| go | 76 | 10 | 15 | 3 | 48 | 10/25 = 40.0 % | 10/13 = 76.9 % |
| python | 78 | 6 | 12 | 2 | 58 | 6/18 = 33.3 % | 6/8 = 75.0 % |
| rust | 77 | 9 | 7 | 8 | 53 | 9/16 = 56.2 % | 9/17 = 52.9 % |
| typescript | 45 | 6 | 0 | 14 | 25 | 6/6 = 100.0 % | 6/20 = 30.0 % |

扩展臂的 role=1 / role=0 / 非 clone 切片同在 `summary`（`p_at_1_widened_*`），本表只列裸臂切片以免两表同形。

## 读数

- **裸臂 top-1 的三分**：same_role 67 / related 40 / unrelated 11（118 查询）——顶 1 与查询「同角色或同机制」占 107/118；
  67 个同角色顶 1 里 29 个是 clone（平凡形）、34 个与查询同文件。样本按 role 位对半分层，故 67/118 不是自仓
  全量的 p@1（自仓 role=1 的 top-1 只占 1114/5378）；分层各自的数才是可外推的：role=1 顶 1 同角色 39/59、
  role=0 顶 1 同角色 28/59。
- **42/118 个查询在 700 个候选里没有一个 same_role**——这些单元在两臂的 top-5 里都没有同角色伙伴被翻出来
  （或本就没有）；hit@5 74/118 的另一面是：在 76 个「有同角色候选出现」的查询里，裸臂把它放在第 1 名的有 67 个，
  藏在第 2–5 名的 9 个。
- **PPMI 扩展臂（m = 3）**：27/118 个查询的顶 1 被扩展改变；配对看，裸臂对、扩展臂错 **6**，裸臂错、扩展臂对 **2**，
  其余 110 个两臂同对（61）或同错（49）；hit@5 +1（74 → 75）。在这份 oracle 上，联想扩展对顶 1 是净负（−4/118），
  对 top-5 覆盖持平。`ppmi_capped_units` 自仓 17 / rust 3 / 其余 0。
- **同角色位的替代合取**（对 700 个已仲裁候选，精度 / 对候选的召回）：spec 形 `(N≥1∧C≥1)∨(N≥2∧形状全等)`
  0.612 / 0.571；`N≥1` 0.402 / 0.695；`N≥1∧C≥1` 0.612 / 0.480；`(N≥1∧C≥1∧形状)∨(N≥2∧形状)` 0.658 / 0.412；
  `N≥1∧C≥2` 0.651 / 0.316；spec ∧ 非同文件 0.678 / 0.333；spec ∨ `(D≥2∧形状)` 0.483 / 0.627。没有一个替代形
  同时高于 spec 形的精度与召回——收紧买精度赔召回、放宽反之。
- **typescript 夹具退化**：26 单元里 role=1 顶 1 只有 3 个，抽到的 9 个查询顶 1 全是 clone（非 clone 子集 0/0），
  该语料的行只作存在证明，不作精度证据。

## 裁定（步 2 权限内：阈值、m、k1、b）

1. **k1 = 6/5、b = 3/4 不动**——只量过这一组，没有换组的证据；`SIMILAR_REV` 1 的输入即此。
2. **同角色合取维持 spec 形**——替代形无一双优（上表）；步 5 落 Haskell `CE.Similar` 时逐字沿用，
   本册这张表是它的对拍基线。
3. **地板 60 %（`similar_oracle_floors`，只升不降）**：role=1 顶 1 同角色 39/59 = 66.1 %、hit@5 74/118 = 62.7 %、
   同角色位对候选的精度 101/165 = 61.2 %，三数各向下取整到一成 = 60 %；重冻结的 oracle 落到线下即是更差的仪器，
   门红而不是改数。没有预登记的「达标线」——计划 v2.29 步 2 把线留给度量本身，故这里给的是回归地板，不是验收线。
4. **PPMI 的 m 与默认臂、以及「是否值得往步 3 走」**：两者都超出仪器能自答的范围——前者是 #4 的产品形态
   （用户三裁点名「稀疏检索 + 仓内 PPMI 联想」），后者是验收线的解释——按 2026-08-07 用户指令经
   AskUserQuestion 呈裁。
5. **用户裁定（2026-09-05）**：① 让 gpt-6-astra 以本 oracle 为靶尝试调优，能显著增强就采、不能就按此精度继续
   步 3；② PPMI 不按「默认开 / 关」二选一，改为从数学与模型两个角度探索更优雅的联想方案再定；③ 顺带回答
   「只报不判的顾问怎样才能被更高效地用起来」——调优与探索的结果另起一节追记，本册以上数字是它们要打败的基线。

## 门（`cli/tests/it/eval_similar_precision.rs`，非 ignored）

- `similar_oracle_consistent`：常量 = 仪器当下常量；词表；≥ 100 个查询；rank 升序且由身份重导；每候选
  truth 在词表、clone 为布尔、why 过地板；`summary.all` 与每语料块 = 从行重导的度量。
- `similar_fixture_rows_replay_byte_for_byte`：四份夹具语料按当下代码重量，冻结行的候选表（身份、六通道
  整数、形状位、role 位、同文件位、两臂名次与分数）逐字节回来，且行所指文件 sha 未变。自仓行只按 sha 锚定不重量
  （自仓文件一动 df 就动，重量无意义）。
- `similar_oracle_floors`：本册「裁定」节定下的地板由该腿执行（见裁定节的数字）。
- 样本 ≺ 判决子树的出处腿（RM2，姊妹族 `dedup_provenance` 同形）待步 5 `core/app/CE/Similar` 落地后加：
  `assert_subtree_postdates` 对空子树按名拒绝（「空子树让门空转」），步 2 加不了。

## 复跑

```
cd cli && cargo test --test it -- eval_similar_precision::
export CE_CORE_BIN=…   # 自仓索引刷新要核
cd cli && cargo test --release --test it -- --ignored similar_replay::similar_replay --nocapture
CE_BLESS=1 …           # 重冻结样本（常量改动后）；CE_SIMILAR_PACKET=<dir> 另写仲裁 packet
```

仲裁批处理（prompt / packet / 合并脚本）在会话 scratchpad，不入库；重仲裁 = 换仲裁者重跑合并脚本，
样本不动。
