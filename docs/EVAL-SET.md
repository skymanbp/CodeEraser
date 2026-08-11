# M4 预注册评估集 v1（计划 §6 M4，D2-1 纯净度）

> 冻结于四分类任何实现代码之前（预注册的全部意义）。本文件 + 
> [contracts/eval/manifest-v1.json](../contracts/eval/manifest-v1.json)
> 共同构成冻结记录；样本载荷含其它私有仓库全文，按用户拍板
> （2026-08-10）**不入库**，落本地 `.ce-eval/`（.gitignore），由
> manifest 的逐样本 SHA-256 钉定、可随时重建校验。

## 构成（用户拍板 2026-08-10）

| 项 | 值 |
|---|---|
| 总量 | **600**，100% 真实 agent transcript（计划下限 ≥200、≥50%） |
| observe 档（feed 链接，机器可证未塑形） | 400（候选池 3,915） |
| 无 guard 时代（< 2026-08-07 18:20 装机，UTC 2026-08-07T10:20） | 200（候选池 8,271） |
| 标注子集 | 200（四分类 ground truth 用） |
| 视界 frozen_at | 2026-08-10T15:24:50 UTC |
| 语言分布 | py 380 / md 169 / rs 46 / ts 3 / go 2（按池比例，未人为配平） |
| 工具分布 | Edit 323 / Write 277 |

## 方法（全程无 RNG、无时钟——同输入必同输出，已双跑逐字节复验）

1. **扫描**：遍历本机 Claude Code transcripts，配对每个 Edit/Write
   `tool_use` 与其 `toolUseResult`，同一 id 只消费一次（compact/resume
   会重放历史行，实测单会话 1,242/3,282 个 id 重复至 6 次）。
2. **重建**：before = `originalFile`（Edit 必须非空；Write 空前态仅当
   `type=create`——update 缺前态不可知，按弃置计，绝不伪造）；after =
   Write 的 `content` 或对 before 施加 `structuredPatch`（上下文与删除行
   严格核对，不匹配即弃置——不复刻 Edit 匹配语义，ADR-004 教训）。
3. **视界**：`frozen_at` 在扫描收集点生效，弃置计数器同样被视界界定
   （transcripts 持续生长，双跑确定性检查曾抓到 +2 漂移）。
4. **分层抽样**：按 (项目, 语言) 分层，最大余数法配额 ∝ 层大小，层内
   SHA-256 哈希序取前 N；标注子集对 600 个 id 二次哈希序取 200。
5. **弃置全量入册**（manifest `excluded`，无静默截断）：错误/被拒结果、
   五语言外、前态不可知、超 1 MiB、历史重放、guard 时代无 feed 链接
   （58 个编辑，纯净但无机器证据，不采）、deny 测试仓（0）。

## Ground truth（标注子集 200，已定稿 2026-08-10）

三层分列存储，锚定偏差可审计：

1. **预标** [contracts/eval/prelabels-v1.json](../contracts/eval/prelabels-v1.json)：
   `git diff --no-index -U0 --color-moved=plain --color-moved-ws=allow-indentation-change`
   的 SGR 行分类（31/32=deleted/novel，35/36=moved），逐样本 numstat 交叉断言。
2. **逐条审核**（agent 全 200 条过目真实 diff，用户 2026-08-10 委托，替代计划
   原文的人工标注）：194 条预标精确；**6 条修正，根因单一**——plain 模式对
   **空行**也做跨位匹配，纯空行"移动"是伪影（5 条全伪影 + 1 条真移动 17/17
   混伪影 7/4，全部 11 个带 moved 计数样本逐行 dump 甄别）。修正只在
   novel/moved、deleted/moved 间移行，numstat 总和守恒。
   完整审核注记（含各样本判定依据）留本地 `.ce-eval/review/reviewed.ndjson`。
3. **定稿** [contracts/eval/labels-v1.json](../contracts/eval/labels-v1.json)：
   200 行终值 + 6 条修正显式列出（预标值/终值/机制）。CI 门
   `cargo test --test eval_labels` 校验 labels ↔ prelabels 除声明修正外逐行
   一致、每行拆分总和守恒（红证：扰动任一数字即红）。

终值：novel 10,455 / moved-in 32 / deleted 975 / moved-out 30；真移动样本
6/200。**FPR ground truth：200/200 均 `is_normal=true`**——语料无异常编辑，
M4 误报门（≤1%/500 行）的分母将全部来自正常编辑回放。

## L0 基线（计划 §4.3 B3c 阶梯首级，2026-08-10）

[contracts/eval/baseline-l0-v1.json](../contracts/eval/baseline-l0-v1.json)，
两变体（CI 门 `eval_labels`/`eval_baseline` 从入库文件全量复核；git 实跑
逐样本验证走 ignored 测试）：

| 变体 | 样本精确 | moved 行召回 | moved 行精度 | 失误模式 |
|---|---|---|---|---|
| `l0_numstat`（计划命名的 `git diff --numstat -M -C --find-copies-harder`，单文件对上 rename/copy 旗标惰性，已逐样本实证） | 194/200 | **0/62** | — | 漏掉全部真移动（6 个真移动样本全错） |
| `reference_color_moved`（git 自带移动检测 = 预标引擎） | 194/200 | 62/62 | 62/125（49.6%） | 空行跨位匹配发明 63 条假移动（6 个修正样本全错） |

两者行级准确率都 ~99.5%——**头名指标必须是 moved 类召回/精度**，总行数
准确率无区分力。两变体失误样本集只交于 1 个（混合样本）：L1（函数边界
对齐）的达标线 = 同时关掉两个缺口——找回 62 条真移动且不吞空行伪影。

## L1（函数边界对齐，用户拍板全量实现 2026-08-10）

实现 [cli/src/fourclass/](../cli/src/fourclass/)：自含 Myers 行 diff（探针路径
不依赖 git，MAX_D 封顶降级显式标记）+ 显著行移动判定（缩进不敏感、仅含字母
数字行、两侧独立标记=GT 口径）+ tree-sitter 单元归属（moved 行标注离开/加入
的函数，整函数原样搬迁摘要）。评分 [contracts/eval/l1-v1.json](../contracts/eval/l1-v1.json)：

| 指标 | L1 | 对照 L0 numstat | 对照 color-moved |
|---|---|---|---|
| moved 行召回 | **62/62** | 0/62 | 62/62 |
| moved 行精度 | **62/62（100%）** | — | 49.6% |
| 样本精确 | 195/200 | 194/200 | 194/200 |

L0 的两个缺口同时关闭。5 个失配样本**全部是纯 diff 对齐差**：GT 总量经
numstat 不变量继承了 git 的非最小对齐，L1 的 Myers 把 git 拆成 del+add 的
相同空行对直接匹配（对称 -k/-k，无任何显著行分类不同，moved 全程精确）；
python difflib 独立复核 4/5 样本逐数一致，第 5 个 L1 比两者都更小。
**饱和注记（用户已确认）**：本集 moved GT 无法区分全量函数边界对齐与单纯
空行过滤——对齐机制的增量（单元归属、整函数搬迁摘要）由 lib 单测钉定，
跨文件/整 commit 场景的区分力留待 L2 对比与 FPR 重放。

## 整 commit 切片 v1（L2 增量仪器，预注册于任何 L2 代码之前，2026-08-10）

本仓库 200 样本集的 moved GT 已饱和（上节）；按拍板，L2 的增量须在
**整 commit / 跨文件 / 单元归属**维度证明——逐文件对的 L1 结构性看不见
"函数离开 A 文件、落进 B 文件"。切片全部派生自本仓库自身 git 历史
（真实编辑，零伪造，仓内可完整复现）：

- **构成** [contracts/eval/commit-slice-v1.json](../contracts/eval/commit-slice-v1.json)：
  宇宙 = 至 2f40f22（L1 落地 commit，仪器在自身宇宙之外）的线性历史 61 个
  commit，五语言 scope（排除 memory/ 机器本地态 = M7 filter-repo 面，D2-7）
  后 47 个入册、14 个无涉排除。预标引擎 `--color-moved=blocks`（git 自带
  ≥20 字母数字 block 下限——plain 模式在 commit 粒度会为跨文件同文琐碎行
  发明假移动，实测 2f40f22 下 153 条；代价是漏 sub-block 小移动，如实入册）。
  文件配对 `-M -C`（纯改名由配对解释，不计移动行）。双跑逐字节复验。
  M7 历史改写会更名全部 sha，届时切片确定性重生成。
- **GT**（[contracts/eval/commit-labels-v1.json](../contracts/eval/commit-labels-v1.json)，
  22 个 moved-bearing commit 逐条审核）：commit 粒度 moved 采用**来源语义**
  而非内容集合语义——新写的行哪怕与别处删除同文也**不是**移动，恰是本产品
  要抓的复制信号，计入 moved 会把重复藏进健康信号。三层可审计：① 机械显著性
  过滤（无字母数字的 moved 标记 → novel/deleted，与 labels-v1 同约定，182 行）；
  ② 机械跨文件/文件内划分（trim 内容配对，两可优先文件内）；③ 审核修正
  （5 条 6 行内容巧合，各带机制，逐条对过原始 diff）。终值：**跨文件 moved
  366 出/181 进 = 547 行**（11 个 commit，全部真实重构：tests/common 抽取、
  eval_support 抽取、observe writer 进 hookio、冷启动状态机拆模块……35 个
  搬迁单元具名入册）；文件内 112/107。CI 门逐行校验账本守恒。
- **L1-on-slice 基线** [contracts/eval/commit-baseline-l1-v1.json](../contracts/eval/commit-baseline-l1-v1.json)：
  L1 逐文件对跑全部 47 commit / 214 对。**检出 219/766 = 恰好全部文件内
  GT，跨文件 0/547，巧合抵扣上界 0**——结构性盲区实测钉死。预测 251 vs
  检出 219 的 32 行超出 = GT 引擎 block 下限漏掉的 sub-block 移动（如实
  注记，非错误）。

**L2 达标线**：在不虚报 25 个无移动 commit 的前提下召回 547 条跨文件
moved 行（及其单元归属）。

## L2 达标（2026-08-11，计划 §4.3 修订版口径，真实流水线）

实现：Rust 跑 L1 不动，把 L1 判为 novel/deleted 的显著行以 `[行, fnv1a(trim)]`
按 **run 分组**（run 结构 = 对齐产物，归对齐者产出：空行/纯标点变更行桥接、
未变更缺口与 within-moved 行断链）经 NDJSON 发给 ce-core；Haskell
（[core/app/CE/FourClass/](../core/app/CE/FourClass/)）以整数代价模型判决：
m=1/v=3/s_cross=2 ⇒ **≥2 行跨文件证据地板由模型推导**（单跨行 1+2=3=v 平局
不开站 = 巧合拒绝本身）；锚定块（无排他、无贪占——去重多对一形态下确定性是
结构性的）+ run 域扩展 + 非对称来源归因（加侧需站点证据，删侧对已开站点
宽松附着——正是产品论点：新写行与别处删除同文是复制信号不是移动）。回传
单调 delta + blocks 证据；Rust 侧做单元归属。评分
[contracts/eval/commit-l2-v1.json](../contracts/eval/commit-l2-v1.json)：

| 门 | 结果 |
|---|---|
| 跨文件召回 | **547/547（366 出 + 181 入），misses = 0**（L1 基线 0/547） |
| 巧合门 | 5 个已审文件逐一 pred==gt 精确（配合逐文件 miss=0 ⇒ 无余地收留被剔行） |
| 零虚报门 | 无跨移动 GT 的 commit 上跨预测 = 0（CI 门逐行断言） |
| 单调 + L1 复现 | L2≥L1 且逐对总和守恒；单文件批在 200 样本集逐字复现 l1-v1.json |
| extras 台账 | 24 行/17 文件逐条具名+内容入册（全部为 GT blocks 引擎 ≥20 字母数字
  地板漏标的 sub-block 真移动：use 行随 helper 搬家、`read_envelope` 首行、
  多次出现的 sleep/kill 尾巴——沿用逐条审核委托审读，无一属巧合类） |
| 确定性 | 逆序输入 delta 逐字节一致；三份 doc 重构前后哈希一致 |
| 代价敏感性 | Spec.hs 钉死 s_cross∈{0,2,4,6}→地板{1,2,3,4}（死旋钮无处藏身） |
| 搬迁登记 | 35 单元名册：32 具名（含 kinds.rs 关闭的 CLASSES 常量洞 + 经扩展相位
  归属的 mark_ready 类；块内逐行归属修掉"大块只记头单元"缺陷）+ 3 个 `~` 注记
  改编单元（out_dir/labeling_rows/write_doc 搬迁中被泛化，GT dump 证零行同一
  存活，行级归属结构性不可能——如实入册而非硬凑） |

行级总量：moved detected 766/766（547 跨 + 219 文件内）。开发中真实流水线
曾比设计实测短 5 行（common/mod.rs 63→60 等）——根因是 Haskell 端以显著行
行号连续划 run，`}}`/空行把 git 视为整块的区域切碎；按"run 结构归对齐者"
修正后闭合，且 5 个巧合文件门全程保持精确。M4-6/7 期间狗粮门另实战四轮
（465 行文件拆分、CoC 24 重构、共 12+ 对克隆全数根治入 eval_support/
eval_l2_parts，棘轮始终回 201）。**外部效度未证**（R-L2-2）：模型设计与
评分同源于本仓库历史；M5 须在第二仓库以同一 GT 流水线复测，届时 deny
档位方可倚赖跨文件 moved 信号；Stop 汇总中该信号先行 informational。

## 复跑 / 校验

```
cd cli
CE_EVAL_TRANSCRIPTS=<transcripts root> CE_EVAL_FEEDS=<projects root> \
CE_EVAL_FROZEN_AT=2026-08-10T15:24:50 \
cargo test --test eval_extract -- --ignored --nocapture
# 重建 .ce-eval/ 并重写 manifest；与已提交 manifest diff 为空即完整复现
```

约束：重建依赖本机 transcripts 留存；`.ce-eval/` 丢失可重建，
transcripts 被清理后仅 manifest 哈希可证完整性（样本另行备份归用户）。
