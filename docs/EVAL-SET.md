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

1. **扫描**：遍历本机 transcripts，配对 Edit/Write `tool_use` 与
   `toolUseResult`，同一 id 只消费一次（compact/resume 重放，实测重复至 6 次）。
2. **重建**：before = `originalFile`（Edit 必须非空；Write 空前态仅当
   create——缺前态按弃置计，绝不伪造）；after = Write `content` 或对
   before 施加 `structuredPatch`（严格核对，不匹配即弃置，ADR-004 教训）。
3. **视界**：`frozen_at` 在扫描收集点生效，弃置计数器同受界定（双跑确定性
   检查曾抓 +2 漂移）。
4. **分层抽样**：(项目, 语言) 分层，最大余数法配额 ∝ 层大小，层内 SHA-256
   哈希序取前 N；标注子集二次哈希序取 200。
5. **弃置全量入册**（manifest `excluded`，无静默截断）：错误/被拒、五语言
   外、前态不可知、超 1 MiB、历史重放、guard 时代无 feed 链接（58 个，纯净
   但无机器证据）、deny 测试仓（0）。

## Ground truth（标注子集 200，已定稿 2026-08-10）

三层分列存储，锚定偏差可审计：

1. **预标** [contracts/eval/prelabels-v1.json](../contracts/eval/prelabels-v1.json)：
   `git diff --no-index -U0 --color-moved=plain --color-moved-ws=allow-indentation-change`
   的 SGR 行分类（31/32=deleted/novel，35/36=moved），逐样本 numstat 交叉断言。
2. **逐条审核**（全 200 条过目真实 diff，用户 2026-08-10 委托）：194 条
   精确；**6 条修正根因单一**——plain 模式空行跨位匹配伪影（11 个 moved
   样本逐行 dump 甄别）；修正只在 novel/moved、deleted/moved 间移行，
   numstat 守恒；注记留本地 `.ce-eval/review/reviewed.ndjson`。
3. **定稿** [contracts/eval/labels-v1.json](../contracts/eval/labels-v1.json)：
   200 行终值 + 6 修正显式列出（预标值/终值/机制）。CI 门校验
   labels ↔ prelabels 除声明修正外逐行一致、拆分守恒（红证双向）。

终值：novel 10,455 / moved-in 32 / deleted 975 / moved-out 30；真移动样本
6/200。**FPR ground truth：200/200 均 `is_normal=true`**——语料无异常编辑，
M4 误报门（≤1%/500 行）的分母将全部来自正常编辑回放。

## L0 基线（计划 §4.3 B3c 阶梯首级，2026-08-10）

[contracts/eval/baseline-l0-v1.json](../contracts/eval/baseline-l0-v1.json)，
两变体（CI 门从入库文件全量复核，git 实跑走 ignored 测试）、样本精确同为
194/200：`l0_numstat`（计划命名命令；rename/copy 旗标对单文件对惰性已逐样本
实证）moved 召回 **0/62**；`reference_color_moved`（= 预标引擎）召回 62/62
但精度 62/125——空行跨位匹配发明 63 条假移动。行级准确率两者都 ~99.5% 无
区分力 ⇒ **头名指标 = moved 类召回/精度**；两变体失误集只交于 1 个混合样本，
L1 达标线 = 同时关掉两个缺口。

## L1（函数边界对齐，用户拍板全量实现 2026-08-10）

实现 [cli/src/fourclass/](../cli/src/fourclass/)：自含 Myers 行 diff（探针路径
不依赖 git，MAX_D 封顶降级显式标记）+ 显著行移动判定（缩进不敏感、两侧独立
标记 = GT 口径）+ tree-sitter 单元归属（moved 行标注离开/加入的函数）。评分
[contracts/eval/l1-v1.json](../contracts/eval/l1-v1.json)：moved 召回
**62/62**、精度 **100%**（L0 两缺口同关）、样本精确 195/200。5 个失配全是
纯 diff 对齐差（GT 总量继承 git 非最小对齐；difflib 独立复核 4/5 逐数一致，
第 5 个 L1 更小；moved 全程精确）。**饱和注记（用户已确认）**：本集 moved
GT 无法区分全量函数边界对齐与单纯空行过滤——对齐增量由 lib 单测钉定，跨
文件/整 commit 区分力留待 L2 与 FPR 重放。

## 整 commit 切片 v1（L2 增量仪器，预注册于任何 L2 代码之前，2026-08-10）

本仓库 200 样本集的 moved GT 已饱和（上节）；按拍板，L2 的增量须在
**整 commit / 跨文件 / 单元归属**维度证明——逐文件对的 L1 结构性看不见
"函数离开 A 文件、落进 B 文件"。切片全部派生自本仓库自身 git 历史
（真实编辑，零伪造，仓内可完整复现）：

- **构成** [contracts/eval/commit-slice-v1.json](../contracts/eval/commit-slice-v1.json)：
  宇宙 = 至 2f40f22（L1 落地，仪器在自身宇宙外）的线性历史 61 commit，五语言
  scope（排除 memory/ 机器本地态，D2-7）后 47 入册、14 无涉排除。预标引擎
  `--color-moved=blocks`（≥20 字母数字 block 下限：plain 在 commit 粒度为跨
  文件琐碎行发明假移动，实测 153 条；代价漏 sub-block 小移动，如实入册）。
  配对 `-M -C`（纯改名不计移动行）。双跑逐字节复验；M7 历史改写后确定性重生成。
- **GT**（[contracts/eval/commit-labels-v1.json](../contracts/eval/commit-labels-v1.json)，
  22 个 moved-bearing commit 逐条审核）：commit 粒度 moved 采用**来源语义**
  而非内容集合语义——新写的行哪怕与别处删除同文也**不是**移动，恰是本产品
  要抓的复制信号，计入 moved 会把重复藏进健康信号。三层可审计：① 机械显著性
  过滤（无字母数字的 moved 标记 → novel/deleted，与 labels-v1 同约定，182 行）；
  ② 机械跨文件/文件内划分（trim 内容配对，两可优先文件内）；③ 审核修正
  （5 条 6 行内容巧合，各带机制，逐条对过原始 diff）。终值：**跨文件 moved
  366 出/181 进 = 547 行**（11 个 commit 全真实重构，35 个搬迁单元具名入册）；
  文件内 112/107。CI 门逐行校验账本守恒。
- **L1-on-slice 基线** [contracts/eval/commit-baseline-l1-v1.json](../contracts/eval/commit-baseline-l1-v1.json)：
  47 commit / 214 对全跑。**检出 219/766 = 恰好全部文件内 GT，跨文件
  0/547，巧合抵扣上界 0**——结构性盲区实测钉死（预测超出 32 行 = GT block
  下限漏标的 sub-block 移动，如实注记）。

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
| extras 台账 | 24 行/17 文件逐条具名+内容入册（全部为 GT blocks 引擎地板漏标
  的 sub-block 真移动，逐条审读，无一属巧合类） |
| 确定性 | 逆序输入 delta 逐字节一致；三份 doc 重构前后哈希一致 |
| 代价敏感性 | Spec.hs 钉死 s_cross∈{0,2,4,6}→地板{1,2,3,4}（死旋钮无处藏身） |
| 参照等价 | 81,640 个穷举小实例上，生产实现与独立参照（双向极大性 block 枚举
  + 相位集合式定义）逐一等价——实现↔规范穷举钉死；规范↔真值由上表七门钉 |
| 搬迁登记 | 35 单元名册：32 具名（含 kinds.rs 关闭的 CLASSES 常量洞 + 经扩展相位
  归属的 mark_ready 类；块内逐行归属修掉"大块只记头单元"缺陷）+ 3 个 `~` 注记
  改编单元（out_dir/labeling_rows/write_doc 搬迁中被泛化，GT dump 证零行同一
  存活，行级归属结构性不可能——如实入册而非硬凑） |

行级总量：moved detected 766/766（547 跨 + 219 文件内；开发中曾短 5 行，
根因 = 显著行行号连续划 run 切碎 git 整块，按"run 结构归对齐者"修正闭
合）。M4-6/7 狗粮门实战四轮（465 行拆分、CoC 24、12+ 对克隆根治，棘轮回
201）。**外部效度已证——R-L2-2 关闭（2026-08-12 用户授权解锁）**：
requests（py，18/18 零发明）与 ripgrep（rs，held-out 988+6 守恒零发明）
两个第二语料以同一 GT 流水线复测通过，Codex 独立评审无 blocker；deny 档
位自此可倚赖跨文件 moved 信号（默认档位变更仍按计划 §4.2 在 M7 裁决），
Stop 汇总该信号的 informational 前置解除。

## FPR 主门（M4 验收主门，2026-08-11 通过）

[contracts/eval/fpr-fourclass-v1.json](../contracts/eval/fpr-fourclass-v1.json)：
全部 **600** 个真实编辑样本（全语料 reviewed-normal，计划下限 500）经真实
流水线（classify_batch + ce-core 链）重放，M4 判定规则（堆叠嫌疑 =
显著 novel ≥20 ∧ 删除 < novel/10 ∧ **顶层具名单元新出现重复键**，常数在
CE.FourClass.Verdict 一处）——**误报 0/600 = 0% ≤ 1%**，CI 门断言。
首测曾 8/600（1.33%）红：根因非阈值而是证据语义——匿名闭包键（3 条 rs）
与跨类同名方法平键（含 `__init__`×2，2 条 py）不构成堆叠身份；dup 证据
收窄到**顶层具名单元**后 0/600，且红绿双向单测钉住真堆叠仍点火、两类
误报形状不点火。**Recall 如实记 undefined**：语料按构造零异常样本，
按计划"recall 报告但不设作弊性 100% 门"。

## 攻击评审加固轮（2026-08-11，Codex gpt-5.6-sol 独立评审后）

评审 14 项（2 blocker + 10 major + 2 minor）逐条独立核实后处置，全档
[docs/reviews/2026-08-11-m4-attack-review.md](reviews/2026-08-11-m4-attack-review.md)：
行身份召回门（F2：跨文件 GT 计数升为行号集合，全语料一次通过）；extras
冻结（新行须先审读再 `CE_ACCEPT_EXTRAS=1` 赐福）；源版本绑定（F1：
`generated_from`，doc 落所记 commit 的**子提交**，活体重放本地跑）；判决
语义加固（F5/F6：去重内容数地板 + run 桥长上界 7，全语料零漂移）；堆叠
证据归属（F7：具名容器/trait 限定键/Go 接收者键；首版修复被 FPR 重放当
场抓 1/600，根修回 0/600——仪器抓自己的修复批，机制在工作）。

## requests 外验切片 v1（M5-1b 冻结，2026-08-11，预注册于任何外验评分之前）

R-L2-2 的解药第一料：同一 GT 仪器（M5-1a 泛化，commit e7aa3f8）对准
[psf/requests](https://github.com/psf/requests) 的 first-parent 窗口
`00fd4c8e..8068356` （2018-05→克隆 tip，626 commit = 可见链全量，零主观
筛选；merge 对第一父 diff = 主线增量，110 个 merge 行带 `parents` 标注供
审读分层）。冻结档
[contracts/eval/commit-slice-requests-v1.json](../contracts/eval/commit-slice-requests-v1.json)：
**341 入册 + 285 无涉排除**；预标 moved 286 进/287 出，47 个 moved-bearing
commit（21 个多文件对 = 跨文件候选）；对语言 py 486 / md 161。双跑逐字节
一致；CI 一致性门枚举全部 commit-slice\*-v1.json 共同校验。

方法差异一处（如实入册）：`added/deleted` 改由**同一编辑脚本的 hunk 头
算术**推导而非 numstat——默认 myers 下 git 的 numstat 会对自家 patch 超
计（requests 28d537dd 实测 15/6 vs patch/difflib/minimal/patience/histogram
一致的 14/5，守恒断言当场逮住）；钉 histogram 会漂移已冻结的自仓切片，故
弃；自仓 doc 在 hunk 推导下逐字节不变。复跑：M5-1a 的四个 CE_SLICE_* 环
境变量 + 本节两端 sha。

**GT 审读**（[commit-labels-requests-v1.json](../contracts/eval/commit-labels-requests-v1.json)）：
机械两层后跨文件 15 出/15 进全部集中在 3 个 commit，逐行对原始 diff 审读
→ **12 条巧合修正**（5aeec8b6 文档同步重写 1 删 2 增；2a6f290b black/isort
全仓格式化 9 行：import 就地合并/炸开/收拢的内容巧合）+ **1 个真搬迁**保
留（99b3b492：`rebuild_proxies` 判定体 9 行 sessions.py 缩进 8 → utils.py
新函数 `resolve_proxies` 缩进 4，逐行同一）。终值跨 **9/9**、文件内
255/257、非显著 15/16。审读记录 = eval_commit_review/requests.json（数据
即数据——平行 Rust 常量表会被自家棘轮判克隆）。

**L1-on-requests 基线**（[commit-baseline-l1-requests-v1.json](../contracts/eval/commit-baseline-l1-requests-v1.json)）：
**cross_credit_upper_bound = 0**——自仓"跨文件 moved L1 结构性零召回"
（0/547）在第二语料复现，R-L2-2 外验第一料落地；detected 510 / GT 530 /
predicted 636（超出方向同自仓 = L1 合法声称 GT blocks 引擎漏标的
sub-block 移动）；320/341 commit 逐对精确。附带清偿自仓欠账一笔：`~` 标
记后未重生成的 labels doc（代码表↔doc 漂移 CI 不可见）掩码比对抓出重生成。

**L2-on-requests 外验判决（2026-08-11，当时 doc 未冻结——这正是判决）**：
召回门过（跨文件 **18/18 行身份级**，resolve_proxies 完整回收）；**虚报
门破**——black/isort 格式化 commit 2a6f290b 上发明 2 站/4 行（`Timeout,`/
`TooManyRedirects,` 收拢↔炸开，两行去重内容恰够 destFloor=2；对照
5aeec8b6 同内容×2 被 F5 地板正确拒绝）⇒ doc 悬置、F4 改判条件点火，升级
候选交由下节影子消融双语料裁决（自仓 547/547 与 FPR 0/600 不可破）。

**影子消融裁决（M5-1c-ii，[commit-ablation-v1.json](../contracts/eval/commit-ablation-v1.json) / [-requests-](../contracts/eval/commit-ablation-requests-v1.json)，
Codex 评审处置 [2026-08-11-m5-1c-ii](reviews/2026-08-11-m5-1c-ii-ablation-review.md)）**：
Rust 影子引擎镜像判决核（Anchor sites + Provenance phase2/3），吃
leftovers() 同一 run 结构；**双重保真门逐 commit 断言**（自仓 47/47 +
requests 341/341）：baseline 影子 == 活核 delta，且影子站点集 == 核 reply
blocks **逐字全等**（分解也被证明，评审 F1）。矩阵（drops=被滤 block 数，
输出相等≠未开火）：

| 变体（精确谓词） | 自仓（GT 547） | requests（GT 18） |
|---|---|---|
| baseline | 547 hit / 0 miss / 0 发明 | 18 / 0 / **4 发明** |
| quality（**单行** ≥20-alnum 锚） | 547 / 0 / 0，drop 14 全冗余 | 18 / 0 / **0**，pred=GT，drop 恰 = 发明站 |
| freq（base 树唯一硬门） | **103 miss**，drop 55 | 未修（drop 0——发明行恰唯一，方向倒置） |
| chain（starts-only 非交叉，最宽松形式） | **8 miss**，drop 6（真交叉搬迁） | 未修（drop 0） |
| flow（贪心整块目的行独占） | **drop 41 输出全等**——竞争被 phase3 兜底吸收 | drop 0 |
| phase3_edge（F4 探针） | 1 miss（宽度 +1 真召回/+1 已审 extra） | 宽度贡献 **0**（台账空） |

裁决：**quality 在此六谓词、双语料内唯一赢家**；聚合-20 地板分离不了发明
站（7+16=23）——单行锚才是有效形式。阈值：发明站死∧最险真站（19 锚）活
= **17..19**；t=20 双语料亦过门（19 锚站转靠冗余）；>20 未扫描。**F4 以
数据结案，维持原设计**：requests 零 error 贡献，自仓加边反破 547→546。
**升级已落地（用户拍板阈值 19，wire 2.0.0）**：行级 alnum 随 rem/add 过
线（判定全在 Haskell 的 Cost.anchorFloor），自仓 L2 summary 逐项不变、
FPR 0/600 保持、升级后消融 baseline 七项 == 升级前 quality 列（双语料预
注册全中）、**requests L2 首冻结：18/18、0 miss、0 extras、0 发明**——
本节上文的"悬置"自此解除。

**边级 GT（M5-1c-iii 尾款）**：审读表新增 relocation_edges（来源→目的地
+经边单元；自仓 16 边行/37 边-单元对，requests 1/1），每行对原始 diff 核
实——名字存活者按定义删除扫描，改名收敛者按体行同一（project_dir /
corpus_dir 即 common tmp 的同体异名）。验收反向设门：**edge_violations =
L2 声称的边必须被审读认可，越界即红**（生成器+CI 双点；首跑即逮出定义扫
描漏掉的 5 条改名体边——按 diff 补全 GT，非按 pipeline）。正向为测量：
边覆盖 **自仓 31/37、requests 1/1**——未覆盖 = 短体单元开不了站 + ~ 改编
无行级证据，与改编谱系（~×3、run_hook 签名改编、resolve_proxies 改名搬
迁）同为下一升级方向（改编/短单元对齐）的量化基线。

## ripgrep 外验切片 v1（M5-1d 冻结，2026-08-12，Rust 补充语料）

同一 GT 仪器对准 [BurntSushi/ripgrep](https://github.com/BurntSushi/ripgrep)
first-parent 窗口 `14860b0f..3fce3b5`（tip 即 M1 crosscheck/M2 性能钉定点，
可见链全量 699 commit 零主观筛选）：**433 入册 + 266 排除**、57
moved-bearing，双跑字节一致。语料首现 `C` 状态，copy 语义就此拍板：copy 对
按 (源, 新) 配对但**不消费 before 侧**（单 `-C` 只把已修改文件当源），pair
带 `copied: true` 记号——literal.rs→literalold.rs 旧版留档正是整文件复制
膨胀形态。**GT 审读**：机械跨 506/499 集中 11 个 commit 逐条对原始 diff 审
读——6 个真搬迁 commit 全量保留（PathPrinter 跨 crate 整迁、clap→lexopt
重构 767 行、pcre2 版本块、hyperlink 别名表解体、max_matches
printer→searcher、别名测试改编迁移），6 条修正/11 行巧合（就地 import 改
写、重写文档首行、use 块嵌套化、被删 word.rs 的通用惯用行）。终值跨
**498 出/496 进**、22 单元登记 + 22 边-单元对。**L1 基线**：盲区三度复现
——巧合抵扣上界 26（首个非零，仍 ≪ 994）、387/433 逐对精确。

**语料逼出两项真产品缺陷（根修 + 重钉）**：① 7,625 行纯新建文件的空侧
diff 被 D 受限搜索误标 degraded → 空侧 = 构造性精确解短路（diff.rs，生产
路径同愈）；② bucketCap 单侧计数在零配对成本处点火（66 条同文 `#[inline]`
纯加入、5×118 样板桶）→ **乘积形工作量护栏** |rem|×|add| > 64²（最坏界不
变、点火仍全请求纯 L1 = F3 语义、洪泛 e2e 翻双侧重钉；旧式在自仓/requests
从未点火 ⇒ 判决零漂移，81,640 穷举等价与 FPR 0/600 保持）。**L2 冻结：
hits 988 + 地板下 6 == 994 守恒、发明 0**（锚地板 held-out 零发明）、22
登记单元全具名、8 边零越界、逆序确定性过。**地板下登记制**（用户拍板
2026-08-12）：6 条真搬迁行在目的地无 ≥2-distinct 连续伙伴、结构性低于
destFloor（单行注释 args.rs:512→parse.rs:87 双侧 + 改编 find 两对孤立行）
——审读逐行入册（审读表 `below_floor`，extras 台账的 miss 侧镜像），门语
义 = **零未审读 miss**（自仓/requests 登记空、bar 不变；hits+misses+
below_floor==cross GT 三层 CI 锚定）。extras 505 行/15 文件逐行台账赐福
（可回收基线 gt−below_floor 计费），两类机制：GT blocks 引擎漏标的真搬迁
（旗标文档段落、改编函数体内的同一行）+ 亚锚样板搭车（`Ok(())`/import 片
段，F4 删侧宽松的既定代价）。**消融三度确认**：quality==baseline
[988,0,0,0]、freq 13 miss / chain 9 miss 三语料全灭、F4 宽度探针零召回贡
献、保真门 433/433（影子==活核逐字全等）。

**Codex 评审处置（2026-08-12，无 blocker，3 major + 2 minor 全核实落地，
[归档](reviews/2026-08-12-m5-1d-codex-review.md)）**：extras 统一可回收基
线计费（消 500≠505 口径分歧与 waiver 隐身通道）；登记三重锚（waived 行被
预测即自证非地板下 + 咽喉拒重复 + labels CI 行身份锚）；**by-name 评审表
解析修复单元/边登记门在外语料 CI 的静默空转**（幻影单元反事实即红）；
copy 不消费落进 labels 机器；乘积护栏 Integer 化 + 两真实形状非降级回归钉。

## 复跑 / 校验

```
cd cli && CE_EVAL_TRANSCRIPTS=<transcripts root> CE_EVAL_FEEDS=<projects root> \
CE_EVAL_FROZEN_AT=2026-08-10T15:24:50 cargo test --test eval_extract -- --ignored
```

重建 `.ce-eval/` 并重写 manifest，与已提交 manifest diff 为空即完整复现。
约束：依赖本机 transcripts 留存；transcripts 清理后仅 manifest 哈希可证
完整性（样本另行备份归用户）。
