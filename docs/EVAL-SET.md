# M4 预注册评估集 v1（计划 §6 M4，D2-1 纯净度）

> 冻结于四分类任何实现代码之前（预注册的全部意义）。本文件 +
> [manifest-v1.json](../contracts/eval/manifest-v1.json) 共同构成冻结记录；
> 样本载荷含其它私有仓库全文，按用户拍板（2026-08-10）**不入库**，落本地
> `.ce-eval/`（.gitignore），manifest 逐样本 SHA-256 钉定、可随时重建校验。

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

1. **扫描**：遍历本机 transcripts，配对 Edit/Write `tool_use` 与
   `toolUseResult`，同一 id 只消费一次（compact/resume 重放实测重复至 6 次）。
2. **重建**：before = `originalFile`（Edit 必须非空；Write 空前态仅当 create
   ——缺前态按弃置计，绝不伪造）；after = Write `content` 或对 before 施加
   `structuredPatch`（严格核对，不匹配即弃置，ADR-004 教训）。
3. **视界**：`frozen_at` 在扫描收集点生效，弃置计数器同受界定（双跑曾抓 +2 漂移）。
4. **分层抽样**：(项目, 语言) 分层，最大余数法配额 ∝ 层大小，层内 SHA-256
   哈希序取前 N；标注子集二次哈希序取 200。
5. **弃置全量入册**（manifest `excluded`，无静默截断）：错误/被拒、五语言外、前态
   不可知、超 1 MiB、历史重放、guard 时代无 feed 链接（58，无机器证据）、deny 测试仓（0）。

## Ground truth（标注子集 200，已定稿 2026-08-10）

三层分列存储，锚定偏差可审计：

1. **预标** [prelabels-v1.json](../contracts/eval/prelabels-v1.json)：
   `git diff --no-index -U0 --color-moved=plain --color-moved-ws=allow-indentation-change`
   的 SGR 行分类（31/32=deleted/novel，35/36=moved），逐样本 numstat 交叉断言。
2. **逐条审核**（全 200 条过目真实 diff，用户 2026-08-10 委托）：194 精确；
   **6 条修正根因单一**——plain 空行跨位匹配伪影（11 个 moved 样本逐行甄别）；
   修正只在 novel/moved、deleted/moved 间移行，numstat 守恒；注记留本地。
3. **定稿** [labels-v1.json](../contracts/eval/labels-v1.json)：200 行终值 +
   6 修正显式列出（预标值/终值/机制）。CI 门校验 labels↔prelabels 除声明修
   正外逐行一致、拆分守恒（红证双向）。

终值：novel 10,455 / moved-in 32 / deleted 975 / moved-out 30；真移动样本
6/200。**FPR ground truth：200/200 均 `is_normal=true`**——语料无异常编辑，
M4 误报门（≤1%/500 行）的分母将全部来自正常编辑回放。

## L0 基线（计划 §4.3 B3c 阶梯首级，2026-08-10）

[baseline-l0-v1.json](../contracts/eval/baseline-l0-v1.json) 两变体（CI 从入
库文件全量复核，git 实跑走 ignored）、样本精确同为 194/200：`l0_numstat`
moved 召回 **0/62**（rename/copy 旗标对单文件对惰性逐样本实证）；
`reference_color_moved`（=预标引擎）召回 62/62 但精度 62/125（空行伪影 63）。
行级准确率双双 ~99.5% 无区分力 ⇒ **头名指标 = moved 类召回/精度**；L1 达标
线 = 同时关掉两个缺口。

## L1（函数边界对齐，用户拍板全量实现 2026-08-10）

实现 [cli/src/fourclass/](../cli/src/fourclass/)：自含 Myers 行 diff（探针不
依赖 git，MAX_D 封顶降级显式）+ 显著行移动判定（缩进不敏感、两侧独立标记 =
GT 口径）+ tree-sitter 单元归属。评分 [l1-v1.json](../contracts/eval/l1-v1.json)：
moved 召回 **62/62**、精度 **100%**（L0 两缺口同关）、样本精确 195/200。5 个
失配全是纯 diff 对齐差（GT 继承 git 非最小对齐；difflib 复核 4/5 逐数一致，
第 5 个 L1 更小；moved 全程精确）。**饱和注记（用户已确认）**：本集 moved GT
无法区分全量函数边界对齐与空行过滤——对齐增量由 lib 单测钉定，跨文件/整
commit 区分力留待 L2 与 FPR 重放。

## 整 commit 切片 v1（L2 增量仪器，预注册于任何 L2 代码之前，2026-08-10）

200 样本集 moved GT 已饱和（上节）⇒ L2 增量须在**整 commit / 跨文件 /
单元归属**维度证明。切片全部派生自本仓库自身 git 历史（零伪造，可复现）：

- **构成** [commit-slice-v1.json](../contracts/eval/commit-slice-v1.json)：宇宙
  = 至 2f40f22（L1 落地，仪器在自身宇宙外）线性历史 61 commit，五语言 scope
  （排除 memory/，D2-7）后 47 入册、14 无涉排除。预标引擎 `--color-moved=blocks`
  （≥20 字母数字 block 下限：plain 在 commit 粒度发明假移动实测 153 条；代价
  漏 sub-block 小移动，如实入册）。配对 `-M -C`（纯改名不计移动行）。双跑逐
  字节复验；M7 历史改写后确定性重生成。
- **GT**（[commit-labels-v1.json](../contracts/eval/commit-labels-v1.json)，22
  个 moved-bearing commit 逐条审核）：moved 采用**来源语义**——新写行哪怕与
  别处删除同文也**不是**移动（计入 moved 会把重复藏进健康信号，恰是要抓的复
  制信号）。三层可审计：①机械显著性过滤（182 行，与 labels-v1 同约定）；
  ②机械跨/内划分（两可优先文件内）；③审核修正（5 条 6 行巧合，各带机制，逐
  条对过原始 diff）。终值：**跨文件 moved 366 出/181 进 = 547 行**（11 个真
  实重构 commit，35 搬迁单元具名）；文件内 112/107。CI 门逐行校验账本守恒。
- **L1-on-slice 基线** [commit-baseline-l1-v1.json](../contracts/eval/commit-baseline-l1-v1.json)：
  47 commit / 214 对全跑。**检出 219/766 = 恰好全部文件内 GT，跨文件 0/547，
  巧合抵扣上界 0**——结构性盲区钉死（超出 32 行 = GT 漏标 sub-block，如实注记）。

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
| 跨文件召回 | **547/547（366 出 + 181 入），misses = 0**（L1 基线 0/547）；2026-08-11 攻击评审后升级为**行身份门**：无修正文件逐行覆盖 GT 行号（同数替换无所遁形），5 个巧合修正文件保持计数门+精确门（修正行身份未存档，如实分层） |
| 巧合门 | 5 个已审文件逐一 pred==gt 精确（配合逐文件 miss=0 ⇒ 无余地收留被剔行） |
| 零虚报门 | 无跨移动 GT 的 commit 上跨预测 = 0（CI 门逐行断言） |
| 单调 + L1 复现 | L2≥L1 且逐对总和守恒；单文件批在 200 样本集逐字复现 l1-v1.json |
| extras 台账 | 24 行/17 文件逐条具名+内容入册（全部为 GT blocks 引擎地板漏标的 sub-block 真移动，逐条审读，无一属巧合类） |
| 确定性 | 逆序输入 delta 逐字节一致；三份 doc 重构前后哈希一致 |
| 代价敏感性 | Spec.hs 钉死 s_cross∈{0,2,4,6}→地板{1,2,3,4}（死旋钮无处藏身） |
| 参照等价 | 81,640 个穷举小实例上，生产实现与独立参照（双向极大性 block 枚举 + 相位集合式定义）逐一等价——实现↔规范穷举钉死；规范↔真值由上表七门钉 |
| 搬迁登记 | 35 单元名册：32 具名（含 CLASSES 常量洞与扩展相位归属；块内逐行
  归属修掉"大块只记头单元"）+ 3 个 `~` 改编单元（GT dump 证零行同一存活，
  行级归属结构性不可能——如实入册而非硬凑） |

行级总量：moved detected 766/766（547 跨 + 219 文件内；曾短 5 行 = 连续划
run 切碎 git 整块，按"run 结构归对齐者"闭合）。狗粮四轮（465 行拆分、CoC
24、12+ 对克隆根治，棘轮回 201）。**外部效度已证——R-L2-2 关闭（2026-08-12
用户授权）**：requests 18/18 零发明 + ripgrep held-out 988+6 守恒零发明，
同一 GT 流水线，Codex 评审无 blocker；deny 档自此可倚赖跨文件 moved（默认
档位变更仍按计划 §4.2 在 M7 裁决），Stop 汇总 informational 前置解除。

## FPR 主门（M4 验收主门，2026-08-11 通过）

[contracts/eval/fpr-fourclass-v1.json](../contracts/eval/fpr-fourclass-v1.json)：
全部 **600** 个真实编辑样本（全语料 reviewed-normal，计划下限 500）经真实
流水线（classify_batch + ce-core 链）重放，M4 判定规则（堆叠嫌疑 =
显著 novel ≥20 ∧ 删除 < novel/10 ∧ **顶层具名单元新出现重复键**，常数在
CE.FourClass.Verdict 一处）——**误报 0/600 = 0% ≤ 1%**，CI 门断言。首测
8/600 红 = 证据语义非阈值（匿名闭包键/跨类同名平键非堆叠身份）→ dup 收窄
**顶层具名单元**后 0/600，红绿双向单测钉死。**Recall 如实 undefined**：语
料按构造零异常，计划本就不设作弊性 100% 门。

## 攻击评审加固轮（2026-08-11，Codex gpt-5.6-sol 独立评审后）

评审 14 项逐条独立核实后处置（[归档](reviews/2026-08-11-m4-attack-review.md)）：
行身份召回门（F2）；extras 冻结（`CE_ACCEPT_EXTRAS=1` 审读赐福）；源版本
绑定（F1：`generated_from`，doc 落所记 commit 的子提交）；判决语义加固
（F5/F6：去重内容地板 + 桥长上界 7，全语料零漂移）；堆叠证据归属（F7：
首版修复被 FPR 重放当场抓 1/600，根修回 0/600——仪器抓自己的修复批）。

## requests 外验切片 v1（M5-1b 冻结，2026-08-11，预注册于任何外验评分之前）

R-L2-2 的解药第一料：同一 GT 仪器（M5-1a 泛化，commit e7aa3f8）对准
[psf/requests](https://github.com/psf/requests) first-parent 窗口
`00fd4c8e..8068356`（2018-05→克隆 tip，626 commit = 可见链全量零主观筛选；
merge 对第一父 diff = 主线增量，110 个 merge 行带 `parents` 标注）。冻结档
[commit-slice-requests-v1.json](../contracts/eval/commit-slice-requests-v1.json)：
**341 入册 + 285 无涉排除**；预标 moved 286 进/287 出，47 moved-bearing
（21 个多文件对）；py 486 / md 161。双跑逐字节一致；CI 一致性门枚举全部
commit-slice\*-v1.json。方法差异一处（如实入册）：`added/deleted` 改由**同
一编辑脚本 hunk 头算术**推导——默认 myers 的 numstat 对自家 patch 超计
（28d537dd 实测 15/6 vs 五算法一致的 14/5，守恒断言当场逮住）；钉
histogram 会漂自仓冻结档故弃；自仓 doc 在 hunk 推导下字节不变。复跑：
CE_SLICE_* 四变量 + 本节两端 sha。

**GT 审读**（[commit-labels-requests-v1.json](../contracts/eval/commit-labels-requests-v1.json)）：
机械两层后跨文件 15 出/15 进集中在 3 个 commit，逐行对原始 diff 审读 →
**12 条巧合修正**（5aeec8b6 文档同步重写；2a6f290b black/isort 全仓格式化
9 行内容巧合）+ **1 个真搬迁**（99b3b492：`rebuild_proxies` 判定体 9 行
sessions.py → utils.py `resolve_proxies`，逐行同一）。终值跨 **9/9**、文件
内 255/257、非显著 15/16。审读记录 = eval_commit_review/requests.json。

**L1-on-requests 基线**（[commit-baseline-l1-requests-v1.json](../contracts/eval/commit-baseline-l1-requests-v1.json)）：
**cross_credit_upper_bound = 0**——自仓跨文件 L1 结构性零召回（0/547）在
第二语料复现，R-L2-2 外验第一料落地；detected 510 / GT 530 / predicted 636
（超出 = L1 合法声称 GT blocks 漏标的 sub-block 移动）；320/341 逐对精确。
附带清偿：`~` 标记后未重生成的 labels doc 由掩码比对抓出（漂移 CI 不可见）。

**L2-on-requests 外验判决（2026-08-11）**：召回门过（跨 **18/18 行身份
级**，resolve_proxies 完整回收）；**虚报门破**——black/isort commit 发明
2 站/4 行（恰够 destFloor=2）⇒ doc 悬置、F4 改判条件点火，交影子消融裁决。

**影子消融裁决（M5-1c-ii，[commit-ablation-v1.json](../contracts/eval/commit-ablation-v1.json) / [-requests-](../contracts/eval/commit-ablation-requests-v1.json)，
Codex 评审处置 [2026-08-11-m5-1c-ii](reviews/2026-08-11-m5-1c-ii-ablation-review.md)）**：
Rust 影子引擎镜像判决核（Anchor sites + Provenance phase2/3），吃
leftovers() 同一 run 结构；**双重保真门逐 commit 断言**（自仓 47/47 +
requests 341/341）：baseline 影子 == 活核 delta、影子站点集 == 核 reply
blocks **逐字全等**（评审 F1）。矩阵（drops=被滤 block，输出相等≠未开火）：

| 变体（精确谓词） | 自仓（GT 547） | requests（GT 18） |
|---|---|---|
| baseline | 547 hit / 0 miss / 0 发明 | 18 / 0 / **4 发明** |
| quality（**单行** ≥20-alnum 锚） | 547 / 0 / 0，drop 14 全冗余 | 18 / 0 / **0**，pred=GT，drop 恰 = 发明站 |
| freq（base 树唯一硬门） | **103 miss**，drop 55 | 未修（drop 0——发明行恰唯一，方向倒置） |
| chain（starts-only 非交叉，最宽松形式） | **8 miss**，drop 6（真交叉搬迁） | 未修（drop 0） |
| flow（贪心整块目的行独占） | **drop 41 输出全等**——竞争被 phase3 兜底吸收 | drop 0 |
| phase3_edge（F4 探针） | 1 miss（宽度 +1 真召回/+1 已审 extra） | 宽度贡献 **0**（台账空） |

裁决：**quality 在此六谓词、双语料内唯一赢家**；聚合-20 地板分离不了发明
站（7+16=23）——单行锚才是有效形式。阈值：发明站死∧最险真站（19 锚）活 =
**17..19**；t=20 双语料亦过门（靠冗余）；>20 未扫描。**F4 以数据结案维持
原设计**：requests 零 error 贡献，自仓加边反破 547→546。**升级已落地（拍
板阈值 19，wire 2.0.0）**：行级 alnum 随 rem/add 过线（判定全在 Haskell
Cost.anchorFloor），自仓 L2 summary 逐项不变、FPR 0/600 保持、升级后消融
baseline 七项 == 升级前 quality 列（预注册全中）、**requests L2 首冻结：
18/18、0 miss/0 extras/0 发明**——上文"悬置"自此解除。

**边级 GT（M5-1c-iii 尾款）**：审读表新增 relocation_edges（自仓 16 边行/
37 边-单元对，requests 1/1），每行对原始 diff 核实——名字存活者按定义删除
扫描，改名收敛者按体行同一。验收反向设门：**edge_violations 越界即红**
（生成器+CI 双点；首跑逮出 5 条改名体边——按 diff 补全 GT，非按
pipeline）。正向为测量：边覆盖 **自仓 31/37、requests 1/1**——
未覆盖 = 短体单元开不了站 + ~ 改编无行级证据，即下一升级方向（改编/短单
元对齐）的量化基线。

## ripgrep 外验切片 v1（M5-1d 冻结，2026-08-12，Rust 补充语料）

同一 GT 仪器对准 [BurntSushi/ripgrep](https://github.com/BurntSushi/ripgrep)
first-parent 窗口 `14860b0f..3fce3b5`（tip 即 M1 crosscheck/M2 性能钉定点，
可见链全量 699 commit 零主观筛选）：**433 入册 + 266 排除**、57
moved-bearing，双跑字节一致。语料首现 `C` 状态，copy 语义就此拍板：copy 对
按 (源, 新) 配对但**不消费 before 侧**（单 `-C` 只把已修改文件当源），pair
带 `copied: true` 记号——literal.rs→literalold.rs 旧版留档正是整文件复制
膨胀形态。**GT 审读**：机械跨 506/499 集中 11 个 commit 逐条对原始 diff
审读——6 个真搬迁 commit 全量保留（PathPrinter 跨 crate、clap→lexopt 767
行、pcre2 版本块、hyperlink 别名表解体、max_matches printer→searcher、别
名测试改编迁移），6 修正/11 行巧合（就地 import 改写、重写文档首行、use
嵌套化、被删 word.rs 惯用行）。终值跨 **498 出/496 进**、22 单元 + 22 边-
单元对。**L1 基线**：盲区三度复现——巧合抵扣上界 26（首非零，≪ 994）、
387/433 逐对精确。

**语料逼出两项真产品缺陷（根修 + 重钉）**：① 7,625 行纯新建文件的空侧
diff 被 D 受限搜索误标 degraded → 空侧构造性精确解短路（diff.rs，生产路径
同愈）；② bucketCap 单侧计数在零配对成本处点火（66 条同文 `#[inline]`、
5×118 样板桶）→ **乘积形护栏** |rem|×|add| > 64²（最坏界不变、点火仍纯
L1 = F3 语义、洪泛 e2e 翻双侧重钉；旧式在自仓/requests 从未点火 ⇒ 判决零
漂移，81,640 穷举等价与 FPR 0/600 保持）。**L2 冻结：hits 988 + 地板下 6
== 994 守恒、发明 0**（held-out 零发明）、22 单元全具名、8 边零越界、逆序
确定性过。**地板下登记制**（用户拍板 2026-08-12）：6 条真搬迁行结构性低于
destFloor（单行注释双侧 + 改编 find 孤立行）——审读逐行入册（`below_floor`，
extras 的 miss 侧镜像），门语义 = **零未审读 miss**（自仓/requests 登记空；
hits+misses+below_floor==cross GT 三层锚定）。extras 505 行/15 文件台账赐
福（可回收基线 gt−below_floor 计费）：GT blocks 漏标真搬迁 + 亚锚样板搭车
（`Ok(())`/import 片段，F4 删侧宽松既定代价）。**消融三度确认**：
quality==baseline [988,0,0,0]、freq 13 miss / chain 9 miss 三语料全灭、F4
宽度零召回贡献、保真门 433/433（影子==活核逐字全等）。

**Codex 评审处置（2026-08-12，无 blocker，3 major + 2 minor 全核实落地，
[归档](reviews/2026-08-12-m5-1d-codex-review.md)）**：extras 统一可回收基线
计费（消 500≠505 口径分歧与 waiver 隐身通道）；登记三重锚（waived 被预测即
自证 + 咽喉拒重复 + labels 行身份锚）；**by-name 评审表解析修复外语料登记门
静默空转**（幻影单元反事实即红）；copy 不消费落 labels 机器；乘积护栏
Integer 化 + 两真实形状非降级回归钉。

## graph 站点宇宙 + 审计抽样 v1（M5-2b/2c 冻结，2026-08-12，均先于解析器）

精度仪器的分母 = **免解析 SITES**（宇宙先于解析器冻结，解析器不得自选分
母——[设计定稿](reviews/2026-08-12-m5-2-graph-design.md) §5）。五语料
graph-slice\*-v1.json：self@60f73e3（**含 crosscheck 孤岛 = 设计内负对照**；
2b-iii [Opus 反审](reviews/2026-08-12-m5-2ab-opus-review.md)加固后 RG3 首
次点火换钉重冻结）、requests@8068356、ripgrep@3fce3b5、zod@912f0f5、
cobra@adbc881（TS/Go 复用 SOURCES.md 已钉 commit）。范围 = 五门正典扩展
名 − memory/；档载清单 + sha256 + 逐 (lang,kind) 站点计数 + 逐类排除 + **测量
前写死的证伪常数**（min_per_lang=15、r0_share_trigger=0.80）。CI 门：summary
从行重导、常数/范围/语料集钉死（删档即红）、档名↔内嵌名一致、五语言联合非零
（D2-4）、**检测器漂移门**（自仓 sha 未变行重检=RG3 CI 化+spec **语句窗口**
子串——站点行=语句头，TS 多行 spec 落后行，14/4269 zod 站，反审 F1）；双跑一致。

**审计抽样**（[graph-sample-v1.json](../contracts/eval/graph-sample-v1.json)，
提交**先于任何 ladder/ 解析器**——2d 起 G13 祖先断言化）：rank id =
sha256(域|corpus|commit|path|line|nth|kind|spec)，nth = 2b-iii 入身份的行内
序号，spec 居末保单射；主样 **100** = 每语言地板 15 + 25 席按语言池最大余数
（纯整数分摊）再按 kind 摊——go 16/md 25/py 17/rs 21/ts 21，语料谱 cobra 12/
requests 15/ripgrep 25/self 18/zod 30；行序 = 独立 audit 域（审计者不见 rank
序）；**后备每语言 20**（补分母不跨语言，护地板）；rung 域预注册、2f 实体化
（非门）。池 = 逐文件对冻结 sha256+计数重建（凭核对不凭信任）；双跑
modulo-provenance 全同。CI 门：verify() 重哈希 + 拒重复 id、字面量 100/20×5、
审计序、配额↔行逐格且从 slice summary 经**同一分摊代码**重导、逐行宇宙绑定、
反事实（篡改/重复必拒——断言非假设）。

**精度审计 GT**（M5-2d，cli/tests/eval_graph_review/{五语料}.json；执行 = Opus
独立代理——用户委托 2026-08-12 镜像 M4-2c，装配 verbatim 判决不动）：100/100
逐站在钉定 OID 读源、零失配零后备消耗；truth = path[#unit]/external/dynamic/
ambiguous/none，why 全带指名机制；37 条 site_gaps（HTML 块引用、GFM 裸 URL、
别名 import、字符串/doc 注释假站点双向危害）。判例：Go 包导入=目录级 truth、
rs 取定义点并记再导出链、TS 多行 import 首行=站点行。CI 门：rank 双射+身份
echo+truth 宇宙绑定（反审 F4）+why 地板+反事实六连+语料谱常数；G13 强祖先门
（a≠d，样本→审计→graph 代码盲窗全扫，fetch-depth:0）。**带分母登记（F2）**：
60 external / 40 in-corpus——in 分母 cobra 1/requests 3/self 4/ripgrep 10/
zod 22（拍板 2026-08-12：**分母 ≥5 才设每语料门**=ripgrep/zod，其余带分母只
报，总体 ≥0.90 合同门不变）；md ref_def/ref_link/url 零席位（44 站无 GT）+
dynamic/ambiguous/none 零 GT = 2f fixture 面补；rank 极小性门不验——冻结
100 由反审独立重导逐字节认证（[2cd 归档](reviews/2026-08-12-m5-2cd-opus-review.md)）。

## 复跑 / 校验

```
cd cli && CE_EVAL_TRANSCRIPTS=<transcripts root> CE_EVAL_FEEDS=<projects root> \
CE_EVAL_FROZEN_AT=2026-08-10T15:24:50 cargo test --test eval_extract -- --ignored
```

重建 `.ce-eval/` 重写 manifest，diff 为空即完整复现；依赖本机 transcripts 留存，清理后仅 manifest 哈希可证（样本备份归用户）。
