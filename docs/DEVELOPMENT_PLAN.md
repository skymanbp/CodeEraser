# CodeEraser 开发计划

> **版本** v2.18 · 2026-08-28 · 状态：🔒 已由 cc-memory 锁定 · **M0–M9 全交付（批 0 发 v0.7.3；批 1–9 与终扫落 main；v1.0.0 项目收口 2026-08-22）· K 轮已交付随 v1.2.0（2026-08-26）· L 轮 v2.18 在轨（v2.17 + 2026-08-28 修正案：减法批、测试子仓、二进制只出自 Actions）** · 本文件是本仓库唯一权威计划。修改流程：改本文件 → 重新 ccm 锁定 → 才能动代码。
> v1.0→v1.3 经两轮攻击评审收口（记录见 docs/reviews/（已清理，全档在 git 历史））；v1.4 增补 ADR-008 + 判定属性电池；v1.5 = M5-3 拆 3A/3B + 验收门修订（十项拍板：reviews/2026-08-13-m5-3-dedup-instruments.md（git 历史） §12）；v1.6 = M5-3A recall 门修正案；v1.7 = ADR-003 收敛式多写者修正案（两案均用户拍板 2026-08-14，全档见 [EVAL-SET-M5-3.md](EVAL-SET-M5-3.md)）；v1.8 = ADR-008 细则（判决/测量分界+四片契约，三拍板 2026-08-17：reviews/2026-08-17-adr-008-policy-dsl.md（git 历史））；v1.9 = M6 并入结构管理器（structure/1 树尺度熵判决=GUI 首屏数据面，两拍板 2026-08-17：reviews/2026-08-17-m6-structure-manager.md（git 历史））；v2.0 = M6 收口修正案（用户拍板 2026-08-17：趋势面板+删除候选浏览移 M7——趋势需历史存储设计、删候浏览宜与发布后反馈同批；Linux/macOS 实包归 M7 Releases 自动化，M6 以 Windows 实包+三平台编译门收口）；v2.1 = M7 签名后置修正案（用户拍板 2026-08-17：0.x 不购证书/账号，README 明示未签名 + SHA256 校验链路承重，签名/公证后置 post-1.0；M7 章程四拍板与切片 P1→P2→P3→P4→P6：reviews/2026-08-17-m7-release-track.md（git 历史））；v2.2 = M8 成长轨立册（用户五条+三拍板 2026-08-17：IP=软著+商标〔发明专利落选→P6 零专利时序约束；商标宜先于 P6 提交防抢注〕、全量文档对齐+生成器门控、i18n en 默认+zh 查表切换、GitHub 可见度；契约正文=reviews/2026-08-17-m8-growth-track.md（git 历史））；v2.3 = M7.5 清理批修正案（用户三拍板 2026-08-18 ccm #1152：休眠评估仪器深度瘦身走 [EVAL-SET.md](EVAL-SET.md) 修正案〔CI 活门全保、复核链交 git 历史〕+ trend/1 趋势判决入核=Haskell 合约内抬占比、语言分工边界不动；P6 前置执行）；v2.4 = 收口记账修正案（2026-08-19：M7/M8 收口标注、签名裁定统一为"不做"、guard 第 3 级 as-built=observe、分发行更新为已公开——纯记录性同步，零架构变更）；v2.5 = 尺寸门语言臂修正案（用户拍板 2026-08-20：前端/脚本常用扩展 js/mjs/cjs/jsx、css/scss/less、html/htm、vue、svelte、sh/bash、yml/yaml 入 **scan 尺寸门 + guard 硬预算 + score/棘轮（纯 size 事实）**，**永不进** index/clone/graph/docdup/fourclass/churn/structure——判决语义零变动〔S2 混语轴若计入 css 会把正常前端目录误判混语〕；`Lang` 追加式扩展，wire 语言码不重排，谓词 `Lang::scan_only`/`judged_path` 为唯一边界权威；起因 = GUI 的 .js 长期无门漏缺陷，评审 2026-08-19 在册）；v2.6 = 尺寸软区间+拆分 ROI 修正案（用户拍板 2026-08-20 A+B+C 全采纳、契约先行实现留 v0.6：A 软区间凸罚替换 warn/fail 双档悬崖〔硬线 750 不变，分数迁移一次并负发版声明义务〕+ B 基线锚定相对软线 S=clamp(median+k·MAD(log-LOC),[200,500])〔冻结时计算防自指漂移〕+ C 拆分 ROI 顾问面〔无可行缝 ROI<1 → 自动豁免带 why；判决/排名入核 = Haskell 占比第二叉首弹〕；设计册 [reference/size-advisory.md](reference/size-advisory.md)；痛点实证 = v0.5.0 评审周 6 拆分/3 删注释/299 停车在册）；v2.7 = 尺寸顾问收尾批（用户拍板 2026-08-20 三项全推进：①guard 区内档位映射接线——默认全 observe，`ce.toml` 显式声明才启用 warn/ask，默认翻档仍以各档 FPR 重放记录为准〔纪律不破〕；②拆分 ROI v1.1 价目——克隆块与单元 co-change 跨缝价，外语料标定定数，wire 加性 minor；③split-candidates 补 MCP 与 GUI 面；三项齐落后发 v0.7.0）；v2.8 = M9 打磨与完整面立项（用户 2026-08-21 八项使用汇报 + 四拍板全走推荐线：①v0.7.3 折入桌面三修复一起出货〔draft 未建，折入只多一轮门禁+CI〕；②「擦除」= 确定性两段式 `ce erase`——只对可确定性安全消除的类别动手（T1/T2 精确重复块、graph 判死代码、docdup 重复段），plan/apply 分离、dry-run 默认、要求干净工作区，可擦除性判决入核，**永不接 LLM 重写**〔立身之本 = 确定式计算审阅不确定式输出〕；③「降 Rust 占比提 Haskell 占比」= 判决逻辑回迁核第三弹——盘点仍居 Rust 的判决性逻辑成迁移清单逐片入核，解析/索引/前端仍 Rust，ADR-002 语言分工不动；④bench 历史 = 按 release tag 用冻结黄金集回放回填，此后每版自动追加。两项现场根因第一手实录：GUI 弹窗风暴 = release 版 windowed 子系统下全仓 spawn 站点零处 CREATE_NO_WINDOW〔corelink/churn/fourclass/daemon 四站点〕，trend 一次量 30 提交、每提交多次 git+一次核判决 → 上百控制台窗循环弹出且期间无进度反馈；终端「报错」= 裸 `ce` 走 clap usage 到 stderr+退 2，PowerShell 5.1 再把 stderr 包成 NativeCommandError 红错。批序 0–7 与验收见 §6 M9 行）；v2.9 = 1.0 收口令（2026-08-21 用户令「全部推完，计划上不要留任何项，确保无欠账/遗留，彻底检查并全量更新所有仓库内文档，最后发布 1.0.0 收口」：批 0 照发 v0.7.3；批 1–7 全部执行但不再中间发版〔原 v0.8.0 计划位并入终版〕，落 main 后以 **v1.0.0** 一次收口；1.0 语义与 §4.2 档位路线一致——两类规则默认 deny 已 as-built，v1.0.0 即完成度与收口声明）；v2.10 = 1.0 前审查与打磨批（用户令 2026-08-21：收口前插入批 **8** 架构级全仓代码审查〔多 agent 审查+逐条对抗核验，发现要么修复要么书面 disposition，不许悬空〕与批 **9** 全域优雅性终打磨〔数学/算法/架构/GUI 再设计思考，含 GUI 美化挂账项；提案册先行，涉契约/门槛/不可逆变更逐条走 AskUserQuestion 拍板〕，批序插于批 7 与终扫之间）；v2.12 = 环轴口径修正案（用户拍板 2026-08-22：**cycle 轴违规集只计代码文件**——批 9 P8 册间双向导航〔index 回链 + prev/next〕把 12 册+methodology.md 熔成一个 SCC、score 948<950 实测暴露：文档超链互引是导航本意、不是重构阻力，却被当结构环收费；处置 = verdict/1 请求**加性**新表 `docFiles`〔文档语言文件的宇宙下标，升序；缺省/空=旧语义，旧 golden 字节不动〕，核内 Score.hs cycleMembers 排除 doc 下标——规则仍居 Haskell〔ADR-008 反抢跑〕；liveness/死文档检测不受影响；proto minor bump + VERSIONING.md 入账 + 相关册就地修 + 引文台账 CE_BLESS 复审；P8 导航条按原双向设计随后落地）；v2.13 = 规则包 DSL v1 修正案 + 协议死数据裁除（用户拍板 2026-08-24 四连：①规则包 v1 = **路径类分参**——ADR-008 细则第二期：`ce.toml` 新增 `[[rules.class]]` 数组〔name+globs 仅本地渲染，**永不过线**（§5.9.2）〕，glob→classId 指派居 Rust 测量侧（与 exclude 同一 ignore::overrides/globset 编译器——已有两方言并存是债，**第三方言零容忍**），类尺求值居 Haskell 数据表；类通道 = verdict/1 `continuous` 行加宽第 4 列 classId（单表单 arity、混排拒——graph/1 nodes 三或四列先例）+ 新表 `classKnobs`（码域 = ceilings 恒发子集 {0,1,2} 的类影子：sizeCeil/cocCeil/sizeHard，**不新造码只加类维**）；多类重叠 = 声明序首中（镜像 verdictTable 首中语义）；classCap 64 分配栅栏；逐类 ladder_fault 于 load 咽喉；**baseline 永三列**（类 = 本 run 收费参数、非棘轮事实，core 构造 newBaseline 剥类列；per-class 棘轮 = 未来 major、v1 非目标）；per-class weights/棘轮容差/判决表/软线 S 均为非目标（各有其因，防「为占比写代码」）；落码四片 **P1 编译搭载 + P2 类尺求值**（同一 proto minor）→ **P3 scan 旁表**（rowClasses+gradeOverrides+镜像扩类维，scan/1 minor）→ **P4 guard 类预算**（零 wire）；验收 = 反事实证表 C1–C9（无声明全 golden 字节不变、类尺/序/回落反事实、五种拒绝各一腿红）；自仓 ce.toml 不启类先行，启类后分数与未启用不可比〔发版声明义务〕；E01 敕令独立不随类动；设计册 2026-08-24 经四路侦察复核，依 85f9308 关架令不入库，本条即持久记录；②协议死数据裁除：verdict/1 churn 行恒值第 4/5 列（added 恒=rw+ap、survived 恒 0，core 全弃读）砍除 = proto **3.0.0**，daemon `HelloOk::version`（无客户端读）砍除 = daemon proto **2.0.0**——用户裁「现在就删」，弃愿望单缓议，两断代先行于 P1〔类通道随后以 3.1.0 加性入线，C1 字节恒等以 3.0.0 goldens 为基〕；③六镜头深扫九决策件收口：D1/D5/D6 可见性缩窄+错述修正、D2/D4/D7 复活配方族同律改写〔9e05f53 处置 eval_support 先例〕、D9 ladder ensure 环留任〔审计 C6 书面处置纪律〕；④设计册归宿 = 本地 memory/+artifact，不入库。**已落地 2026-08-24**（I 轮五提交：1114aed D 清扫 → 958ab96 proto 3.0.0 + daemon 2.0.0 → 8f309b9 P1+P2 = 3.1.0 → d6d195b P3 = 3.2.0 → 44a0abd P4 零 wire；C1–C9 全部有腿；dedup 预算 191→186 只降入账；自仓 ce.toml **不启类**——用户拍板 2026-08-24「维持现状」，规则包留给下游仓库，自仓分数序列连续））。v2.14 = K 轮欠账清零修正案 + 三次 wire 演进（用户两问拍板 2026-08-24：**范围** = 桶 1〔53 条真缺陷/真缺口，来自 8 名只读读者的全仓盘点：事实错误 4、官网缺口 2、GUI 丢判决 6、GUI 缺屏 4、MCP 缺口 5、插件面 3、i18n 泄漏 5、CLI 行为 2、宽限窗到期 2、实战欠账 3、文档-代码不符 4、honest gaps 5、性能与发布门 4、CI 覆盖 3、审计册 #85〕+ 桶 2〔4 条 FPR 纪律件〕；**协议** = 允许断代 4.0.0 一次清掉两个宽限窗到期件。桶 4〔15 条历史裁定不做〕经评估**追加 1 条**：SARIF 输出复活——原 2026-08-19 与 Markdown 同批退役之由是「CLI 只出结构化事实、解读归消费方 LLM」，而 SARIF **是事实的另一种编码而非解读**，故裁定未被冒犯；成本 = JSON→SARIF 纯投影〔零新判决、零 wire〕，收益 = 接 GitHub code scanning 一条真实分发面。代码签名/公证单列请用户商业决定〔R1 留有「有商业需求再议」口子；装机路径已是主推线，SmartScreen/Gatekeeper 警告是真实摩擦，但属花钱的商业裁量非工程欠账〕，不入本轮；其余 13 条维持原裁定并在文档如实标注为非目标。**两条原非目标经重新设计后折入**（第三问拍板）：①**符号事实搭 import 绑定**——R6 之死是「全仓同名匹配」这一**解析策略**之死〔铸边精度 38/66=0.576〕，不是「符号事实不可得」；梯子已高精度解析 import 绑定，而**绑定了哪些名字是语法事实不是猜**，故一条 import 绑定符号边的可信度**恒等于**其所骑的 import 边〔已过 ≥0.90 精度审计〕。graph/1 加性 `symbols`/`symEdges` 两表〔符号名永不过线、线上只走下标——与 classId 同律，§5.9.2〕，一次解开 entry bit 0 文件粒度永不置位、bit 4 无生产者、`unref_public` 结构性不可达、join `publicGuard` 生产态休眠、unit 层 graph 腿恒 null 五条 honest gap；GUI 符号级下钻随之降为**渲染**而非新判决〔2026-08-21「不入围」之由是「需新 wire」，属里程碑末的范围裁量而非语义否证〕。**纪律**：不铸调用边〔函数体内 `foo()` 除非该文件 import 绑定过 `foo`，否则不是边——文件内调用本就不影响文件级存活性〕、任何处不做同名匹配、动态引用构造在场则该文件符号退 `unresolved_dyn` **永不判 dead**；落码前须过与杀死 R6 的**同一把尺**——import 绑定符号边独立 N 站点审计 ≥0.90，不过则整片回滚。**绑定名 ≠ 目标声明**（2026-08-24 对抗验证实证：四语言提取规格各派一名反驳者，**四驳全中**，19 条发现含 8 条 breaks-correctness）——import 路径的叶子在**语法上不可区分**「目标文件内的声明」与「子模块 / 再导出」：Python `from . import certs`（certs 是子模块）与 `from . import X`（X 真在 `__init__.py` 声明）同形，Rust `use crate::{churn, dedup, join, scan}`（皆模块）与 `use crate::config::Config`（叶子是条目）同形，`from m import Y` 的 Y 也可能是 m 的再导出而非声明。故符号边判据加一条**必要条件**：绑定名须在解析出的目标文件的 `symbols` 表里**命中一个声明**（该表已持每文件命名单元，可见性片起并持 vis 位），命中不了即判为模块或再导出，**退回文件级边**。这不是语法猜测而是对实测事实的连接，退化方向诚实——宁可少一条边，不可铸错边（R6 之死正是铸错边）。别名两侧都要取：本地绑定名供本文件的使用点，目标侧名供查表。②**per-class 棘轮**——原不做之由是「改一下 glob 就悄悄放松全部上限，基线随配置漂而非随树漂」，此忧属实，但现行以**截肢**〔禁类归属进基线〕防之过重；正解是把漂移**检测**出来：基线加标量 `classDigest`〔对 `[[rules.class]]` 规范化声明〔名 + 声明序 glob + 旋钮〕的哈希〕，`ce check` 时 digest 不符即**硬拒**并要求 `CE_ACCEPT_BASELINE=1` 具名重立——与分数地板漂移同一机制，「悄悄放松」自此从可能变为硬停。围栏立起后 per-class `ratchet_tolerance` 方为安全〔`vendored`/夹具可冻结为 0；今日真缺陷正是它们享用着不该有的增长额度、吃掉手写代码需要的预算〕；**棘轮行仍三列只记树事实**——进基线的是**指纹**不是旋钮，v2.13「baseline 永三列」字面保住。per-class weights/判决表/软线 S 仍为非目标〔各有其因不变〕。三次 wire 演进按 §2 分开走，与 3.0.0→3.1.0→3.2.0 同律：**4.0.0**〔纯裁除 major：graph 节点行 pre-2.28 legacy flags 列删除、erase class 0 退役、CLASS_NAMES 去二义〕→ **4.1.0**〔符号表加性——**只上 `symbols` 一表**，`symEdges` 缓议之由见 K7 条〕→ **5.0.0**〔graph 节点行 pre-2.28 legacy flags 列裁除；**档位按 §2 修正 2026-08-25**：原文写「顺延至符号表落地后的 minor」，但 contracts/VERSIONING.md §2 写死「schema 不兼容变更（删字段/改字段形状）必须 bump major」，删列正是改行形状，故走 major 而非 minor——这不是范围变更，是把本仓自己的规则施于本条。代价为零：4.x 全程未发布（v1.1.0 出货的是 3.2.0），用户看到的仍是一次协商断代。行降为三列 `[lang,kind,roles]`，与旧三列同元不同义——major 的信封拒绝正是使这种复用安全的机制，K1 由此从「按行元拒」改为「按 major 拒」〕→ **5.1.0**〔classDigest 加性 + per-class 棘轮容差；**已落地 2026-08-25**，K11–K14 四腿俱全，指纹编码首版被自带的腿抓到 NUL 分隔碰撞、改长度前缀〕→ **6.0.0**〔指纹拓宽 major，**已落地 2026-08-25**：52-agent 对抗审查实证 `classDigest` **范围判错**——`[score] viol_cost = 0` 使 939/1000 FAIL 变 1000/1000 pass、`tol_abs=100000` 抹掉棘轮容差、`exclude` 静默移走棘轮行，三者皆挪同样的门却都不碰类表。改为对**整份解析后配置**打指纹（`knobsDigest`），「挑表来围」这一失效模式自此从设计中消失；改键名属 schema 变更故按 §2 走 major。用户拍板 2026-08-25 采纳 knobsDigest 方案〕 → **6.1.0**〔RG10 抵达会动手的两个面，加性 minor，**已落地 2026-08-25**：4.1.0 让判决码 2/4 首次点火后，`ce erase` 的 class 3 仍只看置信度、join 格仍合成 `pFlags=0`，于是**导出面成了可擦除行、`delete` 可以指着导出面提出**——erase golden 第 6 对把前者冻在契约里（原答「公开未引用 API 可擦」），e2e 夹具把它演在真树上（`copy.py` 声明 `def compute_total` 即导出面）。改法两片加性：`verdict/1` 收 `symbols`〔graph/1 同一张表改按 tier 下标，过线的是**原始可见性字**，「哪一位算导出」仍是核的判决〕+ erase 理由码位 6 `public_surface`。K15 五腿：缺席=空表=旧路字节相同、导出死侧则 delete 退位、导出活侧不动〔否则是静音不是防火墙〕、可见性字不含导出位则不动〕；各片反事实证表 K1–K14 为验收正典（**4.0.0**：K1 旧三列节点行整表拒、K2 class 0 请求自此答边界拒绝而 class 3 路字节不变、K3 CLASS_NAMES 四元唯一任何面可辨两路、K4 未受影响八族回复逐字节相同；**4.1.0**：K5 无符号两表的旧请求字节恒等、K6 符号名零过线〔wire 断言只有整数下标〕、K7 非 import 绑定的同名符号**不**产生边〔反 R6 腿；已在生产者侧夹具化（cli/tests/symbol_visibility.rs 的 other.rs 同名不同元 + refuser.rs 路径调用两条拒绝）〕〔**口径再定 2026-08-24**，用户三度交本代理裁断：`symEdges` **本轮不上 wire**。K10 审计量的是**精度**（铸出的边有多少为真：683/683），而任何「无引用」断言吃的是**召回**（真引用有多少被看见）。实测自仓：1064 条 Rust 导出声明中仅 170 条被 import 绑定覆盖，补上「模块绑定 → 该模块 mod_decl 站点再跳一跳 → 成员查表」也只到 248 条，**召回 ~23%**。手工双端复核 13 条样本 ≥7 条确凿在用，其引用点全是 `crate::graph::md::is_md_path(dst)`（cli/src/graph/wire.rs:111）这类**全路径调用**与 `idx.resolve_refreshed(..)`（cli/src/dedup/mod.rs:212）这类**方法调用**——两者都不是 import 点位，绑定表结构性看不见（方法调用更需类型推断，tree-sitter 前端做不到，召回无天花板可言）。故符号层「无引用」即便只出 reported 也是 ~90% 噪声，不落——K8 修正案说的是「reported 档与枚举完备性无关地成立」，不是「证据不足也可以出行」。**唯一方向可靠的仪器是否决式的**：「该标识符在语料库其他文件是否出现过一次 token」——不出现 ⇒ 确无静态引用（注释/字符串命中即弃权，过保守方向安全），自仓真候选 ~101 条且多为「pub 但只在本文件用」的可见性缩窄；但它**完全包含** symEdges（有符号边必然意味着某 import 语句里出现过该 token），故两条路下 symEdges 皆不上线。该否决器另需 `mentions` 每文件标识符表〔SCHEMA_VERSION + GRAPH_REV 双升、全体用户索引重建〕，且其可靠方向取决于「注释/字符串字面量算不算提及」这类**未经对抗审查**的口径，塞进本片正是 K8 刚惩罚过的赶工，故**单列为步 3b 符号层顾问**：符号行加第 3 列 `mentioned`，走「列数即 road」的加性小版本（node 行 3/4 列、dead 行 2/3 列先例），核侧出 `export_unmentioned` **自有类**（不进 dead 码域——RG10 同律：库的公开 API 无人引用不是死）。symedges.rs 与其 K10 审计留库内不上 wire，去留随步 3b 定〕、K8 动态引用构造在场则该文件符号退 unresolved_dyn 永不 dead〔**口径两定 2026-08-24**，用户两次交本代理裁断。①先定「置信列而非静音」：原文「该文件」四读法均不取——护该文件自己的符号**护错了端**（`getattr(mod,x)` 是 F 指向别人的引用，威胁的是目标的符号判决）；护 F 能到达的文件在最要紧处沉默（`importlib.import_module(s)`、`__all__` 再导出、插件注册表都不需要一条已解析的 import 边）；同语言全域静音把分级事实压成二值；只做信息层是弃判决而非解之。取 2.32.0 同一办法：per-language 动态点位台账与 unres 台账同形，同一个 `Cost.confidence` 消费，不新造判据。②**随即一轮 10-agent 对抗审查推翻了「可枚举」这个前提**（5 语言各一份规格 + 各一名反驳者，**5 份全驳**，31 条正确性/漏报发现，全档 memory/k-dynsite-specs.md）：84 条构造写完，反驳者**又找出 19 条漏的**——`//go:wasmexport`（Go 1.24 新增）、doc-test 代码围栏、`#[cfg_attr(feature=…, derive(…))]`、instance-only `import Data.Aeson ()`、`globalThis.Function`、pickle `__getstate__` 族……**该集合开放且随语言版本增长**。更要命的是同一轮暴露出**第二个、更大的类**：**约定式可达**——pytest 的裸 `def test_*`、Go 的 `TestXxx/BenchmarkXxx/FuzzXxx/ExampleXxx/TestMain`、`main`、`#[no_mangle]`/`extern "C"`、`foreign export`、框架装饰器、DI 注册——它们根本不是「动态构造」，却同样让「无人引用」成为假命题。结论：**符号层 dead 判决无法靠枚举变可靠**，故本轮符号判决**只出 `reported` 信息行、永不让门失败**——该档**与枚举完备性无关地成立**（reported 行不作门断言），正是 RG9 dead/reported 分家下沉一层。K8 因此**本轮不约束任何判决**（没有可失败的判决需要保护）；①的置信列是**将来晋级为可失败档**时的机制，晋级须按 §4.2 走 FPR 证据，不得凭枚举自称完备〕、K9 `unref_public` 在有导出面且文件层零入边时首次可达〔此前结构性不可达——`cli/src/graph/deadcode/flags.rs:9` 起 bit 0 在文件粒度永不置位，判决码 2/4 从无一次机会。导出面由 `symbols` 表派生：「本文件是否声明了导出符号」是**与召回无关**的声明事实，而「无引用」仍由阶梯全覆盖的**文件层**边集裁定，故这条不吃 K7 的召回问题〕、K10 独立 N 站点审计 ≥0.90 不过则整片回滚〔**已核 2026-08-24**：自仓 754 条边，站点表与源码独立重算的连接与实现逐条集相等；全量机检——每条边的绑定名都实见于其引用点的 import 语句，0 例外；种子随机 30 条双端人工复核 30/30 真。唯一系统性错类 = 71 条指向 Rust `mod` 声明（units.rs 因 kinds::extra 收录 mod_item 而把子模块记成命名单元），计为假则 **0.9058** —— 刚过线不入账，按根因修：目标文件自己的 mod_decl 站点即权威，命中即拒，零新存储；修后 683 条零已知错类，且该 71 条全中、好边零误伤。理论上的 R6 侧门（花括号 use 树里带自有路径段的叶子落在前缀文件上）本仓实测 0 例；L 轮步 #8 立反事实守卫——`searcher_lib::{searcher::Binary…}` 两行钉 ok(门面,4)：走法止于花括号、前缀文件即边、绑定器只读走法未消费的前缀段永不读花括号段，symEdges 退役后侧门结构上已闭〕；**4.2.0**：K11 无类声明仓 digest 缺席且基线与判决字节恒等、K12 改 glob 即 digest 变而 `ce check` 硬拒并印具名重立指令〔不是静默放松〕、K13 具名重立后新 digest 入册而棘轮行仍三列、K14 per-class `ratchet_tolerance=0` 的类文件一增即 fail 且全局容差不救它）。**已落地 2026-08-26，K 轮 12/12 全交付**（wire 4.0.0→6.1.0 四演进 + guard novel 语义根修 + SARIF 复活 + dedup 结果缓存全落 main；发版链 draft 32937349986 → pin 9567cc5 → verify-publish 绿门根修 d8a49e2 → tag publish 32966421482；四渠道齐上 v1.2.0——GitHub Release 十资产、crates.io、codeeraser.dev 8 页 sha256 对拍、npm 用户 passkey 亲发 latest=1.2.0；bench 7 行 dirty=false@d8a49e2 入列 503bcf3；发版夜挂账的 daemon 冷启丢边稀发竞态〔CI 32964681934〕随后同批根修——掉边与记债同事务的 `resolve_pending` 债务行〔index schema v13，弃建/被杀的 run 留账、任一后续 run 结清；flush-on-exit 变体经对抗核验三组 A/B eject 复现否证后撤除，Bye⇒即刻放手是承重契约〕，确定性重现入 store_tests，ADR-003 v1.7 收敛契约补上崩溃调度这一面）。）。v2.15 = 呈现散文不铸于测量层修正案（用户拍板 2026-08-25：**问题** = ADR-008 的「判决出码、呈现出词」在两处未守住——`churn_unit::GRAPH_CAVEAT` 是 Rust 侧铸出的整句英文散文，随 join 报告 JSON 一路带到 GUI 与 MCP；`health::index_summary` / `daemon_status` 的 `385 files`、`not running (…)`、`unreachable (DEGRADED: …)` 同理进 doctor 文档。i18n.rs 章程写着「报告 JSON 永不翻译——schema 是机器面」，于是这些字**按构造**够不着 `--lang zh`，中文读者必见英文，而 M8-G3b「en 默认 + zh 查表切换」的承诺在这几处落空。**修法** = 测量层只出编号（code）与机器可读的结构化事实，文字由各面自己的表渲染，与 erase 的 reason 码 0..6、join 的判决码同形——这不是新原则，是把 ADR-008 既有原则应用到未守住的几处。**代价** = report schema 断代（`ce.join-report` / `ce.doctor-report` 升 id），GUI 与 MCP 同批改，golden 机器再生。**边界** = anyhow 错误链里的英文**不在**本案范围〔那是生产者自己的话，逐条编号化的收益不抵成本〕，如实标注为 residue，并由 `cli/tests/it/zh_surface.rs` 的行级规则在其模块文档里具名暴露该盲区。同批书面结账：K 轮步 8 的「拆分顾问单位可读化」经核为 batch 9 P15 议决并落地的原文（`recover N vs cost M` 不带单位词；milli 刻度已公布于方法学 08 册与 `benefitMilli`/`costMilli` 字段名），用户 2026-08-25 拍板**维持现状、不推翻**，该项以「已议决」结账而非「已交付」。**已落地 2026-08-25**（b883b5a，随 v1.2.0 出货）。）；v2.16 = 长测量停止静默修正案（用户 2026-08-25 授权「最彻底、最高质量、最优雅、最正确的做法」，K 轮步 8 末项）。**病灶** = 一次跑满数十至数百秒的测量在结束前不吐一个字节（本机实测 `ce churn . --days 1` = 102 s、stderr 0 字节；PERF-BUDGET 已录 `--days 14` = 278.4 s、`ce join` = 265.0 s），操作者无从把「在跑」与「卡死」分开；而 main_cli.rs 首段自陈 help 要回答「它值多少代价」，churn / join / trend 三条恰恰不答。**立场**（与 v2.15 同源）：测量只发**阶段码 + 两个计数**，词由各面自持；进度走 **stderr**，stdout 永远只承报告，故 `--format json` 与全部机器面按构造字节不变。**闸门** = 仅当 stderr 是终端才绘制，`CE_PROGRESS=0/1` 双向覆写（CI 与门测取证走这条）；由 `main` 一次性武装，与 `i18n::init` 同形——把 sink 穿过 `churn::run` 的五个调用点，会把两个机器面的沉默降格为「每个调用点记得传 None」的自觉，而不是结构保证。**收尾** = RAII `progress::span()`，与开始同函数、覆盖 `?` 早退，故无须在各命令喉口逐个补 finish。**范围** = churn（窗口/提交/blame 三相）、join（索引/图/装配）、trend（逐提交测量）；三条 about 各补代价句并同步 main_lang zh 表与双语 README，`docs/reference/cli.md` 由 docs_gate 机器重生成。**边界** = 不新增全局 flag：`--lang` 之外再加一个全局开关会落进每条子命令的 help，而其中只有三条会绘制；环境变量是显示类开关的既有形制（CE_LANG / CE_BLESS / CE_ACCEPT_BASELINE 同族）。**已落地 2026-08-25**（0c420be，随 v1.2.0 出货）。）；v2.17 = L 轮收口修正案（用户拍板 2026-08-26，四问 + 七条逐条裁定：**起因** = 用户重定义「收口」= 完全没有、绝对没有、零可继续推的内容；据此以 7 路只读读者 × 逐条核验清点全部残余前沿——**182 项**（计划本体/横幅/本地设计册/方法学 caveat/代码标记/契约+CHANGELOG/发布分发七源，最大单源 47 < cap 50 零截断），87 项仍开放、95 项已落地或已明确裁掉，其中 134 项双票复核 118 一致、16 分歧经逐条仲裁〔6 条自「已关」翻入计划〕；台账 memory/frontier-2026-08-26.{md,json}（依 85f9308 关架令不入库）。**范围**（三批零耦合、可并行）：**甲** = ADR-008 细则第三期**步 3b 符号层顾问**全链——口径对抗审查先行〔「注释/字符串字面量算不算提及」+ 两道必答题：约定式可达类（K8 ②）须免疫，嵌套可见性（visibility.rs 交给图而图在文件粒度从不问）须定；**甲-1 已封版 2026-08-27**——口径规格 memory/l-mention-criterion-spec.md **v9**（615 行，依 85f9308 关架令不入库，本条即其持久结论）经九轮对抗审查：第一轮七镜头〔五语言 + wire + 语料〕对 v0，第二轮 wire/storage/xlang 三镜头 56 条，第三至九轮 wire + language 双把各攻上一版增量 41/43/33/31/26/17/16 条，第七轮起每条 finding **及其修法**另派独立核验者 Workflow 推翻式复核〔13/11/11 把：19+15+12 CONFIRMED、7+2+4 PARTIAL、**0 REFUTED**；报告自带修法 20/26、14/17、14/16 有错，只落核验者修正后的修法〕，**第九轮两把双零 breaks-correctness 即封版**。口径定案：① **提及** = 判决文件之外任何 U 文件（自有 walk、纳入生成/供应目录——用户拍板 ③；前 8000 字节 NUL 判二进制、4 MB 上限）出现同一 token，注释/字符串字面量**算**，同文件例外按语言分治（Go `{{}}` 模板、TS 模板串递归采集、Python doctest 行、Rust 宏体/文档围栏、Haskell haddock 围栏）；token = 扩张字母表 run（`start` = 字母|_|$）经脚本边界切分与 `$` 两臂分档（JS 族整 run only / 其余 union），存 fnv1a64 不存明文，≥7 字符另填 `folded_hash`，Q1 折叠只对 Rust 声明开（≥2 段 ∧ ≥7）；声明名须自身是一个 token 否则出域（`foo'`/`(<+>)`/`r#type`/`图_report` 皆安全侧）；② **约定式可达免疫** = 每声明一个 `conv` 类别字（bit 0..11：MAIN/TEST/FFI/REGISTRATION/PROTOCOL/MEMBER/MEMBER_DISPATCH/MEMBER_API/DEFAULT_EXPORT/AMBIENT/ALLOW/CFG；判据 = 性质不是名单，bit 9 仅宽容解析可达形以 tsc 报错号免见证）；③ **嵌套可见性** = 新表 `mounts`（每节点恒一行 `[node,private,total,bits]`，bit 0 再导出目的、bit 1 包私有，覆盖面由建造者全量 `enumerate().map()` 承）+ vis bit 2 restricted，核侧 code 全序 1>2>3>0（`private/restricted/reexported/public_unmentioned`）；④ **wire** = graph/1 **6.2.0 加性 minor**：请求加 `unmentioned=[[node,vis,conv]]`（去重升序，软 cap 131072 生产者自限、硬阀 524288 入 `famOverCap`）与 `mounts`（自有 `mountCap` 131072），两表同生同死（第六/七条具名拒绝占 `violation` asum 最前），回复加 `exportUnmentioned=[[node,vis,conv,code]]` + 掉表位 `unmentionedDropped`；`symbols` wire 表与 `verdict/1` 一字不动；核侧 `CE.Graph.Advisory` 具名谓词可消融；名字永不过线（K6 第三腿）；⑤ **存储** = `mention_files`/`mentions` 两表 + `MENTION_REV` 自有 meta，SCHEMA 13→14 wipe，GRAPH_REV 11/12/13 逐片，mention pass 自有入口只由 deadcode 路触发（六消费者路仅一路传是）；⑥ **片序** (1) symEdges 退役〔用户拍板 ①：**删**〕→ (2) visibility 拆目录 + T3/H2/H4/H5 守卫修复〔用户拍板 ②：随甲批各带 K27 差分 + release note〕→ (3) mention walk/store → (4) `conv` 列 → (5) `mounts` → (6) 6.2.0 三面原子 + `Advisory.hs`/`Cost.hs` → (7) 渲染/report 0.3.0/GUI/MCP → (8) K23 四语料处置 + PERF + 方法学册；验收腿 K16–K47 与 K 腿劈半纪律、残余风险台账（§6 19 条具名，含晚 NUL `.ai` 进 U、单字符 sigil 永久 mentioned、`md.rs:172` 唯一幻影铸造路、fold 通道数字起头键缺口）、折叠纪律（文本锚不写册内行号 + 四道回证）全在册〕→ 落码按 ⑥ 片序 (1)→(8) 走（**计划对齐 2026-08-27**，用户裁「不再问、按优雅/高质量/完整/彻底取效果最好的做法」：原「`symbols` 第 3 列 `mentioned`」形经 W2-F10 档位引证改定为 ④ 两表、`symbols` 出口面不动，核侧 `export_unmentioned` 仍为**自有类**（不进 dead 域，RG10 同律）；原「`symEdges` 终裁」随拍板 ① 定删并依 W3-F12 提到片 (1)；ccm 计划同日换版重锁，处置记录在 `memory/.plan_history/`）→ 提取面补强**全做**〔Go/Haskell named forms、Python `__all__` + Python 下划线模块 / `if TYPE_CHECKING:` 私有性（用户裁 2026-08-28 入 #8：片 (8) K23 requests `_types.py:157` 具名）、K3b 四限档（TS import-equals / `__future__` / hs-boot / Haskell type forms）、R6 侧门守卫、Markdown 锚点四限档全建模、cabal `common` stanza 展开、**R4/R5 梯级召回拓宽**（用户裁 2026-08-27，片 (5) 落 `mounts` 时具名：`rs_use::bound` 不绑 uniform-path 门面 `pub use source::Thing`〔裸头按 crate 名走 R4 即止；自仓 cli/src 26 处无边 use〕、不绑 lib+bin 同包内自非根模块走到根为止的 `use crate::Thing`〔两根两终点 AmbiguousRoot；2/217〕，自仓索引 0 条 via_reexport 边 ⇒ mounts bit 0 恒 0；拓宽 = 裸头先读本地模块再读 crate 名 + 根终端在同包内绑定，作 spec §4 修正案随 K 系差分 + R5 精度册重跑 + RG3 具名；片序不变、#6 先行）〕→ 验收 = 自仓 ~101 条可见性候选逐条处置〔顾问落地后实测 **38 行**（rust 36 + haskell 2；K23 的 ~101 是仪器前估计，差额在否决通道账上），裁定流 wf_a0724240-731 九判官九核验者：21 转私有、4 pub(crate)、4 pub(super)、2 内联、1 删、6 保留具名——pub(crate)/pub(super) 仍带 VIS_EXPORTED 故行改码不消行，只有去 pub 才消行〕→ GUI 符号级下钻纯渲染（随片 (7)）→ **减法批**（v2.18 用户裁 2026-08-28：「rs 能否瘦身」经勘察 wf_9bba332a-39e〔5 读者 20 候选、25 agent 逐条反驳〕实证**体量是 7 语言 × 梯级 × wire 的承载，可安全去掉的只有 7 条 ≈116 行 = 0.3 %**，13 条推翻〔最大一条 696 行语言表改数据文件是项目早裁不做的方向〕；落 7 条 + 反驳者顺手证实的 3 处真缺陷〔u64-LE blob 解码截断尾静默丢、`PairVerdict.legs_mask/.reasons` 存而不读、`DirEdges.inter/intra` 只喂自己的单测〕，dedup 预算随之具名下调、棘轮具名重立）→ **测试子仓**（v2.18 用户裁 2026-08-28：「测试能否改 Haskell」经量化否决——`cli/tests/it` 80 文件中 44 个白盒直接调 Rust 库 API〔125 测试、7.2k 行〕、3.0k 行 `cfg(test)` 按语言规则离不开 crate，黑盒 42 % 改写只换语言比重观感且要倒置 CI 顺序；故立 **public 子仓 skymanbp/CodeEraser-tests** 承载 cli/tests 全树 + gui/tests 四脚本〔迁 cli/tests/gui〕，主仓 submodule 挂 cli/tests 原路径〔ADR-006 成员 id、引文标签、crates.io `exclude` 全不动；gui 四脚本 REKEYED 账本〕，CI checkout / `ce trend` 回放 worktree / mention_universe U 公式三处皆初始化 submodule，**自仓分数继续含测试、与 1.2.0 可比**，`git filter-repo` 保留历史；「打包单文件上传」否决：合成 .rs 撞自家 scan 硬门与 E01，tar 资产多一条会断的链而公开仓 CI 免费）；**乙** = 围栏收尾 19 条〔k4 围栏攻击册 14 条：removal 无 fail 条件、digest 单读者、trend 逐点误报、无 digest-only 重立、类容差公制盲、第 64 类 off-by-one、subdir 重写基线、缺档无名重立、round-trip echo、console 具名、拼写敏感、注释夸张、冻结测试向量、`--min-distinct` 无界；同域 5 条：golden 三元组门、daemon 客户端 reader 线程泄漏、glob 两方言统一到 globset（v2.13 之债）、sccFloor 出 ce.toml 旋钮〕；**丙** = 文档/记账/实录清欠 9 条〔v2.15/v2.16 落地戳与 §6 K–L 行（本条同批完成）、zh_surface 路径、DAEMON.md 重放清单、VERSIONING §4 freeze 措辞、pre-2.18 死兼容裁除、CHANGELOG 维持窄章程 + 指路句、空 import specifier 进 unres 台账（到期承诺）、PreToolUse 全链延迟一次实录、竞品复扫并立为每次 plan-set 例行项〕。**具名后置**：丁-1 评分/评测束 12 条 → M 轮（docdup 轴恒 0、stacking 落点、GT 31/37、re-fire 判定、clone/dup 二次质量、min_distinct 出处、budget_breach 例外、structure 无地板、S3+ 边表不上 wire、daemon 重启预算、§7.2 nth 退化）；丁-2 分发束 20 条 → N 轮（provenance / draft 门 / concurrency / dependabot / SARIF 消费者与 PR lane / bootstrap HTTPS / 气隙 / crates / pin 手抄 十条 automation；两平台键资产 / deb-rpm / AppImage-dmg 接线 / perMachine / npm 源仓 / 站点手动链 / marketplace 渠道 / updater / macOS 每推覆盖 / 安装器动态腿 十条平台扩张）；丁-3 证据门 4 条（符号可失败档、guard 档位晋级、zone_tiers 翻档、R-L2-4 多文件 FPR 仪器）——§4.2 铁律不动；丁-4 产品小项 7 条 → M 轮（why 英文、erase advisory 家族、P17 断点、erase-log 渲染、t1_twin 演练、`[ui]` 路、UserPromptSubmit 钩评估）；软著 eCO 为外部等待。原拟「永久立场」8 条经逐条再议**全部入轮**〔用户令「按最完整、最彻底、最优雅、最高质量做，不省成本，做正确的不做简单的」，记 cc-memory directive quality-over-cost〕。**判定式** = 87/87 具名归属（甲 18 + 乙 19 + 丙 9 + 丁 11+18+3+7+1 + O44 自偿）。**落地序** = 本条 → ccm 退役 K 计划立 L 计划 → 才动代码；工期 3–4 周 ±。）
> 本文件行数以锁定时为棘轮上界：只准变短，不准变长；更新必须就地改写；调研依据：2026-08-06 七路并行实证调研（GitHub API / 官方文档 / 论文原文），关键事实附 URL。

## 0. 一句话定位

**在 LLM 写入代码/文档的当轮拦截熵增** —— diff 级、实时、可强制。
"可强制"指能力而非默认值：deny 能力从 M3 起存在，默认档位按 §4.2 的演进路线随证据升级。

## 1. 问题与证据（立项依据）

- GitClear《The Maintainability Gap》（6.23 亿次变更，2023–2026，指标按变更行归一化）：
  **重复代码块 40.3 → 73.0/百万变更行（+81%，历史最高）**；moved/重构占比 21%(2022) →
  **3.8%(2026)**；copy/paste 升至 **15.7%**。<https://www.gitclear.com/the_ai_code_quality_maintainability_gap>
- 「Volume-Quality Inverse Law」（arXiv 2605.02741）：代码总量与架构级 smell 计数相关
  **ρ=0.94 (p<0.001)**（相关非因果、smell 计数含规模效应——只主张强相关，不主张因果）。
- 反证诚实纳入：arXiv 2603.27130 发现代码层差异极小 → 卖点锚定在**编辑/提交行为层**
  （重复、堆叠、净增长、churn），不是"AI 代码天生更烂"。

## 2. 竞争格局与差异化

| 先例 | 占据的位置 | 没做的（= 我们的空位） |
|---|---|---|
| [jscpd](https://github.com/kucherenko/jscpd) v5（Rust 引擎，自带 MCP/skill/SARIF；jscpd-rs 0.1.12 文档只称 "incremental detector facade"，无 diff/changed-files API） | Token 级 T1 查重 + 事后报告 | T3 near-miss；写入当轮强制拦截；结构度量（`--summary` 仅体量 + 复杂度估计） |
| [desloppify](https://github.com/peteromallet/desloppify)（3k★） | 全库健康扫描 | "True incremental or diff-only scanning is not the supported model **yet**"（README 原文，复扫 2026-08-27 原句仍在；注意 yet——它可能补上，见 R5 触发器） |
| [CodeScene](https://codescene.com/engineering-blog/codescene-ci-cd-quality-gates/) | PR 级 Code Health 门禁 | 商业闭源；不做写入时拦截 |
| [colbymchenry/codegraph](https://github.com/colbymchenry/codegraph)（68k★） | 代码图谱喂 LLM 上下文 | 图谱项目均不做克隆/重复判定 |
| [mizchi/similarity](https://github.com/mizchi/similarity)（TSED） | T3 跨文件相似检测（最近先例） | 无 gating（README 仅建议接 pre-commit/CI）、无插件面（仓内 `.mcp.json` 是自用配置）、文档查重仅实验性 `similarity-md` |
| [fuck-u-code](https://github.com/Done-0/fuck-u-code)（7.3k★） | 快照式质量评分 + 幽默输出 | 重复检测仅文件内且正则实现；无时间维度 |
| [betterer](https://github.com/phenomnomnominal/betterer) | ratchet 棘轮范式（JS 生态） | 无跨语言主导者 |

**空位（调研 agent 逐 README 核对）：「每次编辑判定真修改 vs 堆叠新增」无人在做。**
三个差异化判决：

1. **写入当轮拦截**：编辑落盘当轮完成判定与强制（PreToolUse 廉价门 + PostToolUse/Stop
   深判与阻断，见 ADR-004），超限时给出量化依据并指回既有实现；
2. **三信号 join**：克隆相似度 × 依赖图位置 × git 历史（co-change/churn）联合给出
   "删除/合并候选"判决——调研确认没有任何项目同时具备三者；
3. **编辑四分类**：把一次 diff 分解为 matched / novel / moved / deleted，直接度量
   "是更新还是堆叠"。

## 3. 产品形态

| 形态 | 载体 | 里程碑 |
|---|---|---|
| **主动**：`ce` CLI | 单二进制（Rust），`codeeraser` 为等价 alias；`ce scan / check / dedup / structure / baseline / doctor / eject` | M1 起（`check`/`baseline` 落地于 M5-3B、`structure` M6、`eject` M7——2026-08-13 拍板⑩） |
| **被动**：Claude Code 插件 | hooks（as-built 三钩：SessionStart/PreToolUse/Stop）+ skills（erase 删除引导，2026-08-19）+ `bin/` 垫片；PostToolUse 深判**退役**（职责归 Stop 审计+CI） | M3 |
| **被动**：通用 agent 集成 | pre-commit、CI（退出码 + `--fail-under`）、**最小 MCP server（M3）**、完整 MCP（M7） | M3/M7 |
| GUI | Tauri（复用 Rust 前端） | M6 |
| 分发 | M3 后 0.x 预览（air-gapped 手动放置，D2-3）；M7 起已公开（as-built 2026-08-19）：marketplace + GitHub Releases + crates.io（`cargo install codeeraser`）+ npm 指针 + 官网 codeeraser.dev | M3/M7 |

## 4. 功能规格

### 4.1 主动模块（用户按需启用，`ce.toml` 配置，纯声明式）

| 模块 | 检查项 | 默认阈值（出处经核实） | 里程碑 |
|---|---|---|---|
| `size` | 文件 LOC；函数长度；参数个数 | 文件 300 警告 / 750 阻断（ESLint max-lines=300；Sonar S104=750）；函数 50/75（ESLint=50；Sonar S138=75）；参数 5（Pylint）。尺寸门语言臂（v2.5）：js/jsx/css/scss/less/html/vue/svelte/sh/yml 仅入本模块 + guard 硬预算 + 棘轮，不入任何判决族。v2.6（as-built v0.6.0）：scan 双档不变；score size 轴改软区间凸罚（S=基线 softLine，缺则回落 300；H=750）+ `ce structure --split-candidates` 顾问面，契约见 [reference/size-advisory.md](reference/size-advisory.md) | M1 |
| `complexity` | Cognitive Complexity 主判罚；Cyclomatic 辅助 | CoC 15（Sonar S3776）；CC 10–15（Sonar S1541=10 / lizard=15）。证据边界如实声明：ESEM 2020 元分析中 CoC 仅在理解耗时（r=0.54）与主观评分轴有支持，正确率轴无支持（r=−0.13 CI 跨零）；arXiv 2303.07722 中 CC 略优于 CoC。选 CoC 主判罚的理由是其对嵌套的惩罚正对准"堆叠"形态，而非"已证明的可维护性代理" | M1 |
| `readability` | 命名规范、嵌套深度、注释密度 | 不用 Maintainability Index 作主分（van Deursen 批判：1994 系数从未重标定、与 LOC 共线）。主判罚永远 = LOC + CoC + 重复率 | M1 |
| `clone` | 跨文件 T1/T2（热路径）；T3 near-miss（冷路径）；**不承诺 T4**（arXiv 2606.25272：SOTA 在 T4 全线退化） | T1/T2 min-tokens 50（jscpd 默认）；T3 TSED 0.85（定义与阈值仓内自定义并文档化——2026-08-13 拍板②；对照物读出值作分歧入册不改数） | M2/M5-3 |
| `docdup` | Markdown/纯文本段落 + **代码注释/docstring** 查重（与 `clone` 联动）：shingle + MinHash/LSH 粗筛 → Jaccard 复核 | 段落粒度；逐字下界 50 tokens（Lee et al. 2107.06499） | M5-3 |
| `churn` | 函数级追加 vs 重写比例、两周 churn、co-change 纠缠对 | 先例：GitClear 指标 + ops-codegraph-tool co-change | M4 |
| `graph` | import/调用边抽取、跨文件符号解析、入度/环 | 工程量锚点：ops-codegraph-tool 用 6 级 import 解析达 precision 94.9%/recall 66.7%——这不是一行验收能带过的子系统 | M5-2 |
| `deadcode` | 无引用符号/文档段落（图入度 = 0 ∧ 非入口） | 依赖 `graph` | M5-2 |
| `score` | 综合评分 + 棘轮基线（语义见 ADR-006） | 权重表配**敏感性测试**：扰动任一权重断言总分变化（fuck-u-code 的真实 bug 是权重字段从未被评分路径读取——"权重和=1"断言测不到死字段） | M5-3 |

评分极性全程统一"越高越好"。幽默评语彩蛋 as-built：`ce check --roast`（i18n 查表，2026-08-19）。

**排除模型（M1 起内置，A2d）**：默认排除 lockfile、minified/生成物（`*.min.js`、
protobuf/OpenAPI 产物）、vendored、快照测试、migration、二进制/数据文件；叠加
`.gitignore`、`.ceignore` 与 `ce.toml` 的 glob。**类别级豁免（D2-5）**：license 文件头、
结构化 docstring 骨架（`Args:`/`Returns:` 等模板行）不入 docdup 语料。豁免三条路：行内
`ce:allow(<rule>) -- <why>`（无 why 即违规）、`.ceignore`、基线豁免存量（JSON 等无注释语法文件用后两条）。

### 4.2 被动模块（guard，Claude Code 插件）

拦截点依据官方 hooks 文档（<https://code.claude.com/docs/en/hooks.md>，2026-08-06 核实；
PostToolUse 不能阻断工具执行，但可反馈；强制阻断点 = PreToolUse 与 Stop）：

| Hook | 职责（与 ADR-004 混合强制点一致） |
|---|---|
| `PreToolUse`（`Edit\|Write`） | 只做**无需 AST 的廉价检查**：路径排除、目标文件当前 LOC 预算、`new_string` 片段对指纹索引的 T1/T2 探针。超限 → `permissionDecision:"deny"/"ask"` + 指回既有 `file:line`。不做 AST diff（避免重放 Edit 落盘语义这一隐藏子系统，评审 A2a） |
| `PostToolUse`（`Edit\|Write`）/ `FileChanged` | **退役（裁定 2026-08-19）**——官方语义不能阻断只能反馈，深判职责由 Stop 审计（git diff 净效果）与 CI 门承担；再加一层反馈面即上下文熵源（B4 立场） |
| `Stop` | 本轮净效果审计（基于 **git diff**，因此对 Bash/`>>` 写入同样生效）：净 LOC、新增重复块、（M4 起）四分类汇总。引入净冗余而声称完成 → `decision:"block"` 要求返工 |
| `SessionStart` | 引导二进制（见 §5.9）；注入 guard 健康状态一行（daemon 是否存活、索引 freshness、guard 档位；降级计数归 `ce doctor`，§5.9-5） |
| `UserPromptSubmit`（可选） | 廉价启发式标记本轮意图（更新 vs 新增），仅作 §4.3 的可选辅助信号，非判定前提 |

**诚实边界（A2b）**：PreToolUse 是**行为塑形层，不是安全边界**——agent 可用
`Bash: echo >>`/`sed -i` 绕过。兜底 = Stop 审计走 git diff（与写入工具无关）+ CI 门禁。
文档必须如实写明这一点，不得宣传为"不可绕过"。

**默认档位演进路线（A1，写死在此，不许默认永远 warn）**：

1. 0.x（M3–M4）：默认 `warn`；`deny` 能力存在，用户可按规则开启；
2. M4 的 FPR 门（§6）通过后：**T1/T2 精确重复写入**与**硬预算超限（文件 >750 行）**两类
   规则默认升为 `ask`；
3. 1.0（M7）：上述两类默认 `deny`；其余规则 as-built 默认 **observe**（无各自 FPR
   记录即无晋级资格，如实缺席——2026-08-17 切换生效，和解记录见 CHANGELOG）。
   每次默认档位变更在 CHANGELOG 记录依据（FPR 数据）。

### 4.3 F4「更新监督」判定模型（核心创新）

四分类：**matched / novel / moved / deleted**。借鉴 difftastic 代价模型思想但自研
（<https://difftastic.wilfred.me.uk/diffing.html>；其 JSON unstable、有硬上限、且
**不识别 moved**——moved 恰是关键健康信号）：整数常数代价模型，跨文件开站成本推导出
≥2 行证据地板，站点另须**一条 ≥19-alnum 锚证据行**（M5-1c-ii 双语料影子消融，拍板
2026-08-11：单行锚地板唯一保 547/547 满召回且清零 requests 发明；freq/chain/flow/图
分配全以数据除名——目的地竞争被删侧归因吸收零错误，F4 删侧宽松以数据结案维持）。

**Fallback 阶梯（B3c，先易后难，各级都是上一级的对照组）**：
L0 = `git diff --numstat -M -C --find-copies-harder`（零自研）；L1 = L0 + 函数边界
对齐（tree-sitter 符号表）；L2 = 跨文件来源判定（自研整数代价模型；AST 单元用于归属
与搬迁登记）。M4 从 L0/L1 建立 baseline，L2 须证明相对 L1 的增量收益，不达标退 L1。

判定规则（**意图无关**，A2c 修复；意图信号仅可选增强）：

- novel 内容与仓库既有代码/段落结构指纹相似度超阈值 → **重复实现嫌疑**（主规则，
  不依赖任务意图；写新测试/新 endpoint 等正常新增不含相似既有实现，不触发）；
- `novel ≫ deleted` **且** novel 与被修改函数同名/同签名/高相似 → **堆叠嫌疑**
  （旧实现没删、新实现又写一份的典型形态）;
- 文档编辑中新增段落与既有段落 MinHash 相似 → **重复陈述嫌疑**；
- 每个判定输出量化依据（novel/moved/deleted 行数、相似度、指回位置），
  绝不输出裸"感觉太长"。不承诺语义级矛盾检测。

### 4.4 CLI UX 与输出

- 退出码：`0` 通过 / `1` 违规 / `2` 内部错误；`--fail-under <score>`（与棘轮合成语义见 ADR-006）。
- 格式（as-built）：console、JSON 全命令，SARIF 限 scan/dedup 两个发现形命令（`--format sarif`，v2.14 复活、2026-08-25 落地：纯投影+CI 上传腿）；Markdown 维持退役（裁定 2026-08-19）。
- 分工（as-built）：CLI 只出结构化事实（报告族 JSON + `ce mcp` 只读面 + erase skill 引导删除），解读归消费方 LLM。
- **hook 输出 token 预算（B4，anti-bloat 工具不得自己成为上下文熵源）**：
  warn 注入 ≤ 200 tokens/事件；同一 `(rule, file)` 每会话只报一次，后续静默累积；
  深度报告落盘 `.ce/`，由 skill 按需读取；Stop 汇总 ≤ 400 tokens。预算进 M3 验收。

## 5. 架构

```
┌ Claude Code / 其他 agent ──────────────────────────────────┐
│ hooks(SessionStart/PreToolUse/Stop) · MCP 只读面 · bin/ce  │
└──────────────┬（hook 每次触发 = 短命 ce 进程）─────────────┘
               ▼
┌ 前端 ce (Rust 单二进制) ───────────────────────────────────┐
│ CLI/配置/排除 · tree-sitter 解析(官方 Rust 绑定) · 热路径  │
│ 廉价检查 · hook I/O · GUI(Tauri) ──┐                       │
└────────────────────────────────────┼───────────────────────┘
              named pipe(Win)/UDS ─► ▼
┌ ce daemon (同一 Rust 二进制, per-project, 懒启动) ─────────┐
│ 指纹索引(SQLite WAL, 多写收敛) · git 历史抽取 · 文件监听   │
│ 子进程: ce-core (Haskell) ↕ NDJSON over stdio(均长驻)      │
│   判决层: 规则引擎(hlint 式双层) · 四分类(L2) · TSED       │
│   依赖图/三信号 join · 评分与棘轮                          │
└────────────────────────────────────────────────────────────┘
```

**职责边界（B1 采纳后）**：延迟敏感的热路径（PreToolUse 廉价检查、索引探针）完全在
Rust 进程内完成，不跨语言；**一切"判决"**（四分类 L2、规则引擎、评分、棘轮、图分析、
TSED）在 Haskell——这些全部位于放宽预算的路径上（Stop 秒级 / 批扫），
Haskell 承重且不背 1s 预算。

### 架构决策记录（ADR，偏离须先改本文件）

**ADR-001 前端语言 = Rust（Go 落选）。**
tree-sitter Rust 绑定是官方一等公民（核心 crate 0.26.x，语法 crate 各按其 ABI 锁定）；最近先例（difftastic、
mizchi/similarity、jscpd v5 引擎、ast-grep）全是 Rust，可参考复用（ast-grep-core，MIT）；
GUI 由 Tauri 覆盖。Go 无以上任何优势。

**ADR-002 Haskell 不拥有解析层；职责 = 判决层（2026-08-12 拍板细化：判定半落 Haskell）。**
实证：Hackage `tree-sitter` 停在 0.9.0.3（2022-04-12），包描述自劝退
（<https://hackage.haskell.org/package/tree-sitter>）；github/semantic 已 archived；
唯一活跃替代 hs-tree-sitter 是 AGPL-3.0-only + 单人维护。→ Rust 止于解析/抽取/索引并输出
归一化 IR（符号、span、结构指纹、import 边；**token 流只入本地索引，不跨进程**——A6）；
图算法（入度/环/SCC）与 docdup 复核判定在 Haskell。wire format 借鉴 ast-grep `--json=stream`。

**ADR-003 进程模型（A3 拆分后）。**
- hook 触发 = 短命 `ce` 进程；重活委托给 **per-project daemon**（同一二进制 `ce daemon`，
  首次使用懒启动，空闲 30 min 自动退出）。
- 通道：Windows named pipe / Unix domain socket，管道名 = 项目路径哈希；连接本身不设防，
  凭据 = `<root>/.ce/daemon.token`（每次 serve 在 bind 之后重铸——能力边界 = 读得到项目
  即可连；契约与协议版本见 contracts/DAEMON.md，daemon proto 1.1.0 起）。
- 索引=**收敛式多写者缓存**（v1.7，审计实证唯一写者从未成立）：写路径全内容门控+幂等+IMMEDIATE 锁内自检，WAL 逐事务串行 ⇒ 并发写者对静止树收敛于串行序终态（验收=`concurrent_writers` 双进程电池）；
  daemon 角色=性能（热缓存+探针热路径）非正确性，M6 GUI 直写同库有据。
- 版本 skew：连接握手带协议版本，不匹配 → daemon 自杀重启（新二进制路径由客户端传入）。
- 冷启动：首次索引后台异步构建；未就绪期间 guard **显式降级**为廉价检查档（降级状态
  进 SessionStart 健康行与 Stop 汇总，不静默——A9f）。
- Haskell core 是 daemon 的长驻子进程，NDJSON over stdio，全程 `ByteString` +
  `hSetBinaryMode`（规避 GHC #10762/#15021 的 Windows code page 坑）；
  **禁止 Haskell DLL**（GHC #16429/#23644 未解决；`foreign export`+DllMain 官方警告冻结）。

**ADR-004 强制点 = 混合（B3a 采纳，替代 v1.0 的 PreToolUse 独担）。**
PreToolUse 只做无需 AST 的廉价检查（见 §4.2）；AST 深判在 Stop 审计
（已落盘全文，无需重放 Edit 语义）；强制力 = PreToolUse（廉价规则）+ Stop（深判结果）。
否决"PreToolUse 独担"：需自建与 Claude Code 逐字节等价的 Edit 落盘语义重放器（unique
匹配、replace_all、空白/CRLF 归一化），任何偏差 = 判定错文件；且 `new_string` 片段常含
ERROR 节点无法可靠建树。代价（文件短暂脏后被要求返工）在 agent 工作流中可接受。

**ADR-005 克隆检测两层；自研 winnowing 而非复用 jscpd（否决理由记录，B3b）。**
热路径：归一化 token 流 winnowing/Rabin-Karp 指纹倒排索引（Schleimer et al. SIGMOD'03
的无漏检下界保证），SQLite 分片、增量失效。冷路径：候选集 → AST 结构指纹 → TSED（T3）。
不复用 jscpd 引擎：①需进程内毫秒级探针（jscpd=批扫 CLI/Node）；②需增量失效常驻索引；
③避免 Node 运行时依赖。代价：自研索引正确性风险，用"增量 ≡ 全量重建"property 覆盖（§7）。
检出能力对齐 jscpd 可检出集（验收含配对精度，§6 M2）。embedding 只进离线报告。
排除：后缀数组（构建不增量）、全仓 pairwise 树编辑、全仓 embedding。

**ADR-006 棘轮语义（B5 修复）。**
- **连续型指标**（文件 LOC、函数 CoC）：per-file/per-function ceiling = 基线值；
  超 ceiling 即 fail，低于 ceiling 自动收紧到新值。修 bug 需要加行时：ceiling 有
  单次编辑 +2% 或 +10 行（取大）的容差，容差消耗计入 `ce check` 棘轮行。
- **离散型违规**（clone 实例、deadcode 符号）：基线是**违规集合**（指纹标识）；
  新增成员即 fail，移除成员自动收基线。
- **类围栏（v2.14，4.2.0 起）**：基线另存标量 `classDigest` = `[[rules.class]]` 规范化声明的哈希。
  digest 不符 → 棘轮**硬拒**并要求具名重立（`CE_ACCEPT_BASELINE=1`）——规则集变更不得被无声吸收。
  围栏在场时容差可按类声明（`ratchet_tolerance`，vendored/夹具可为 0=冻结）；棘轮行仍三列只记树事实，
  进基线的是指纹而非旋钮。缺省无 digest = 无类语义，未声明类的仓字节恒等。
- 与 `--fail-under` 合成：有基线的仓库以棘轮为主门，`--fail-under` 为下限保险；
  两者任一 fail 即 fail。`ce-baseline.json` 提交进仓库（betterer 范式）。

**ADR-007 插件工程约束（官方文档核实，2026-08-06）。**
- 布局：`plugin/.claude-plugin/plugin.json`（省略 `version` 则 commit SHA 即版本；**显式
  version**——D2-2）；仓根 `.claude-plugin/marketplace.json`（source "./plugin"：`owner/repo` 一键添加只认仓根清单——官方文档核实 2026-08-18）。
- 安装拷贝进 `~/.claude/plugins/cache` → 禁止越界相对引用。
- 二进制分发路径**唯一化（A9a）**：仓库 `bin/` 只放轻量启动脚本；真身二进制由
  SessionStart 从 GitHub Releases 下载到 `CLAUDE_PLUGIN_DATA`（跨版本保留），
  **HTTPS + SHA256 pinned 在插件清单内**，校验失败拒绝执行并明示。三平台二进制
  预期 8–19 MB/个（shellcheck 7.69 MB ~ hlint 18.99 MB 区间），不塞仓库。
  air-gapped 模式：允许用户手动放置二进制 + 本地校验。**二进制只出自 release.yml 三 OS 矩阵**（用户令 2026-08-28，RELEASE.md §1 铁则；本地构建物永不上传、永不 pin）。代码签名/公证**裁定不做**
  （2026-08-19，成本/收益不成立；有商业需求再议）——SHA256 链为永久信任锚，README 明示。
- DENY 协议：exit 2 + stderr，或 exit 0 + `{"hookSpecificOutput":{"permissionDecision":"deny",...}}`；
  自设 `timeout` 并按 R3 fail-open + 显式记录。
- ⚠️ 官方文档无 Edit/Write hook payload 逐字示例 → M0 用 echo-hook 实测 dump 固化 fixture。

**ADR-008 策略即数据 = Haskell 规则 DSL（2026-08-12 拍板；细则 v1.8 三拍板 2026-08-17）。**
判决·豁免判定·预算·棘轮·阻断语义在 ce-core 以"位台账+判决表"求值（具名谓词产条件位，判决=有序数据表）；测量语义（指纹/掩码豁免/截断/上限执行）留 Rust，须单点声明+knobs 回显钉住；判据=需源文本或行级内容过 wire 即测量侧（§5.9.2 一票否决）；热路径 guard 与 hook 协议映射按 §5 边界留 Rust（两判例）。
Rust 解析 `ce.toml` 原样过 wire 不解释语义。四片：P4 配置面与表化→P1 判决权回迁（is_clone/verbatim 半边/degraded→FAIL）→P2 棘轮统一（budget 比较入 core）→P3 scan 分级入 core（scan 获 --core，PERF 超标单片回滚）。
验收 = 产品判决面字节等价（上报集/退出码/报告行，golden 全绿）+每片反事实证表承重；wire 按 proto minor 加性演进。占比提升是副产品，禁止为占比写代码。全档：reviews/2026-08-17-adr-008-policy-dsl.md（git 历史）。**细则第二期 = 规则包 DSL v1 路径类分参（2026-08-24 拍板，v2.13）**：glob→classId 指派居 Rust 测量侧（globset 与 exclude 同方言），类尺求值居 Haskell（`classKnobs` 数据表，码域 = {0,1,2} ceilings 类影子）；类名与 glob 永不过线，线上只走整数（continuous 第 4 列 classId + classKnobs 三元组）；声明序首中、classCap 64、baseline 永三列；四片 P1→P4 与反事实证表 C1–C9 为验收正典（全文见 v2.13 banner 条）。**细则第三期 = 符号事实搭 import 绑定（2026-08-24 拍板，v2.14）**：符号的导出性是每文件**本地语法事实**（`pub`/`export`/`__all__`），指派居 Rust 测量侧；符号存活性求值居 Haskell（4.1.0 先上 `symbols` 一表派生文件层导出位，判决码域沿用 graph/1 四态不新造；步 3b 符号层顾问 = **v2.17 L 轮甲批**（口径对抗审查册 v9 **已封版 2026-08-27**——提及口径 / 约定式可达免疫 / 嵌套可见性三题定案，见 v2.17 banner 条；`symEdges` 终裁 = **删**，用户拍板 ①），缓议因由见 v2.14 K7 条）。判据仍是 §5.9.2 一票否决——符号**名**属文本形物故永不过线，线上只走下标；边只从 import 绑定派生，不由名字搜索铸造（R6 判例：同名匹配铸边精度 0.576，永久关闭不复议），且**须在目标文件的 symbols 表命中声明**方成符号边，否则退回文件级边（对抗验证 2026-08-24：语法无法分辨叶子是声明还是子模块/再导出，详见 v2.14 ① 条）。

### 5.9 安全与隐私（A9，上市场的准入条件）

1. **网络承诺**：`ce` 与 `ce-core` 在分析路径上**绝不联网**；唯一网络行为是 SessionStart
   二进制下载（可关）。embedding 特性仅限本地模型；任何云 API 需按仓库显式 opt-in。
2. **索引隐私**：SQLite 索引只存 token 哈希指纹、span、符号名，**不存源代码文本**（提及表
   §5.1 亦只存 fnv1a64 哈希）；位置恒为项目 `.ce/index.db`（入 `.gitignore`）；`CLAUDE_PLUGIN_DATA` 只放钉版启动二进制。
   默认排除 secrets（内置 glob `.env*`/`*.pem`/`*.key`/`id_*`/`.npmrc`/`.pypirc`/`.netrc`/`*credentials*`——v2.17 S-A9 自四条加宽，判决 walk 与提及宇宙 walk 同一张表 + `.gitignore` 项）。
3. **配置信任模型**：`ce.toml` 纯声明式（阈值/开关/glob），**不可指定可执行命令**——
   clone 恶意仓库不产生代码执行。
4. **卸载**：`ce eject` 清除基线、`.ce/` 与 `CLAUDE_PLUGIN_DATA` 下的 `ce-*` 启动
   二进制；插件卸载文档含 eject 指引。
5. **可见性**：`ce doctor`（daemon 健康、索引 freshness、降级计数）；SessionStart 健康行；
   降级事件计入 Stop 汇总——fail-open 但绝不静默失效。

### 5.10 仓库布局（M0 建立）
```
CodeEraser/
├── plugin/       # 插件根：.claude-plugin/plugin.json + hooks/hooks.json（marketplace 清单在仓根 .claude-plugin/，source "./plugin"）
├── cli/          # Rust workspace：ce（CLI+daemon，含 hookio/probe/audit）
│   └── tests/    # git submodule → skymanbp/CodeEraser-tests（Rust 集成套件 + gui/ 四节点门；v2.18 用户裁 2026-08-28，历史经 filter-repo 保留）
├── gui/          # Tauri 外壳 + vanilla JS 界面（消费 CLI 同一报告 schema）
├── core/         # Haskell cabal：ce-core（判决层）
├── contracts/    # 契约版本化机制 + 双语言共享 golden fixtures
├── docs/         # 本计划、协议文档、评审记录
└── memory/       # cc-memory 本地状态（.gitignore 排除，不入库）
```

## 6. 里程碑（工期为单人 + agent 协作的粗估，标 ± 者不确定度高）

| # | 内容 | 工期 | 验收标准（量化、可复跑、防作弊） |
|---|---|---|---|
| **M0** 契约机制与骨架 | License 已拍板 **Apache-2.0**（LICENSE 已入库）；`ce` 命名撞名核查（crates.io/npm/brew）；契约**版本化机制**（信封格式 + SemVer 协商，内容不冻结——B1）；echo-hook 实测 Edit/Write payload 固化 fixture；双工程骨架 + 三平台 CI（私有仓计费额度评估：macOS 10× 倍率，限 tag/夜间触发——D2-8）；工具链锁定并**实测依赖可解**（C3：`cabal build` 全依赖集在 GHC 9.14 LTS 通过，Stackage 快照记录在案）；热路径延迟分解表（fork→ce 冷启动→探针→回传各项预算） | 1–2 周 | CI 三平台绿；`ce --version`↔`ce-core --version` 握手；payload fixture 入库；ce 冷启动实测 < 100ms（Windows 含 Defender 首扫除外，单列记录） |
| **M1** 度量 MVP | `size`+`complexity`+`readability`+排除模型；首发语言 **TypeScript / Python / Rust / Go / Markdown**（Markdown 仅 size；Haskell 支持后移 M5——无外部对照物，B2）；`ce scan` console/JSON | 3 周 ± | fixtures 从**钉死 commit 的真实仓库随机抽样**（清单入 contracts/）；CC 与 lizard(TS/Py)、rust-code-analysis(Rust)、gocyclo(Go) 一致率 100%；CoC 与 gocognit(Go) 对拍且分歧全部清单化归因（规范差异注明出处，无未解释分歧——D2-6）；CoC 过 Sonar 白皮书共通例题 + 自建 golden；分歧 case（短路、装饰器、可选链）显式收录不回避 |
| **M2** 克隆热路径 + 进程模型 | winnowing 指纹索引（token 归一化覆盖全部五门首发语言——D2-4）、`ce dedup`、daemon（ADR-003 全项：懒启动/握手/WAL/冷启动降级） | 3 周 ± | 10 万 LOC 全量索引 < 30s；单文件增量 < 200ms；探针往返（含管道）p95 < 150ms；对 jscpd 可检出集召回 ≥ 95%（属 docdup 域〔docstring/注释重复〕或阈值测度差异的条目可逐条证据归因排除，排除项入册——用户拍板 2026-08-07）**且**在同一真实仓库上精度 ≥ 90%（召回必配精度，B2）；property：增量 ≡ 全量重建 |
| **M3** 被动 guard v1 | 插件成型：PreToolUse 廉价门（预算+T1/T2 探针）、Stop 审计 v1（git diff 净 LOC + 新增重复块，**不含四分类**——A4）、SessionStart 引导+健康行、hook 输出 token 预算、pre-commit 模式、**最小 MCP server**（`check_duplication`/`scan`，对标 jscpd 已在位的位置——A8）；**收尾发 0.x 预览**（本地/私有 marketplace，自有真实项目 dogfood；部分会话跑**静默观察档**——只记录判定不注入不拦截，为 M4 积累未被 guard 塑形的 transcript；plugin.json 自此带显式 version） | 2–3 周 ± | 本地 marketplace 安装 → 测试仓库端到端拦截 T1 重复写入（transcript 为证）+ **500 次真实正常编辑重放误拦 ≤ 1 次**（N=1 演示不算数，B2）；hook 端到端 p95 < 1s 且分解表各项达标；会话累计 hook 延迟中位数 < 15s/百次编辑；0.x 预览在干净环境安装成功，dogfood 会话 ≥ 10（其中观察档 ≥ 5——D2-2） |
| **M4** 更新监督 + Haskell 判决层引入 | 四分类 fallback 阶梯 L0→L1→L2（L2 = Haskell 承重首战）；`churn`；契约内容随真实需求定稿为 1.0 | 3–4 周 ± | **预注册**评估集（实现前冻结、≥200 编辑样本、≥50% 来自真实 agent transcript，**样本纯净度（D2-1）**：只采观察档会话与 M3 前无 guard 历史会话，被 guard 干预过的编辑排除并报告排除比例——否则 FPR 被 guard 塑形向下偏，deny 准入门自证）；主门 = **FPR：500 次真实正常编辑误报 ≤ 1%**；recall 报告但不设作弊性 100% 门；moved 以 `git -M -C` 交叉 + 人工标注为 ground truth（difftastic 不识别 moved，不能当对照——A5）；L2 需证明对 L1 的增量收益，否则产品走 L1 |
| **M5-2** 图 + 死码 | `graph`（独立子系统，验收对齐 ops-codegraph-tool 锚点；调用边=import-绑定层，R6 全仓同名匹配为条件项：须独立 100 调用点审计 ≥90% 方开——2026-08-12 拍板）、`deadcode` | 3–4 周 ± | import 边 precision ≥ 90%（抽样人工核对 100 条，覆盖五门首发语言——D2-4；TS/Go 语料=crosscheck 已钉 zod/cobra commit）；`unreferenced_public` 独立报告类不并入 dead；本仓库 deadcode 发现全处置；core 判定不变量属性电池入 CI（2g 起——2026-08-12 拍板） |
| **M5-3A** 深度去冗·检测 | T3 冷路径（TSED 定义仓内自定义并文档化）、`docdup`（含代码注释/docstring 域），各配预注册评估仪器 | 3 周 ± | T3 recall 对 mizchi/similarity 可检出**全集**（分母永不缩减；检出按 ce 全层记功——T1/T2 已报 = 产品真阳非排除项；miss 按封闭词表归因入冻结台账，增长需显式 accept；`recall_incremental` 并列发布，书面处置触发器 <0.50——2026-08-13 拍板③；**v1.6 修正案（2026-08-14 拍板）**：字面门 ≥0.90 经仪器实证在仓内 TSED 定义下对该对照物可证不可达〔miss 100% 定义性：size_bound/below_floor/judged_not_clone，候选盲区已由 S5 全对候选源根修清零〕，门改挂**只升不降回归地板**（冻结 epoch zod 3/6、requests 67/425、cobra 1417/9205；门 `eval_t3_recall` 与冻结件随 v0.5.0 加码批退役（[EVAL-SET.md](EVAL-SET.md)），全档在 git 历史）〕）；T3 精度 ≥ 85%（四源冻结候选宇宙 + 独立审计 GT + 只对已答行 + 输出量地板——拍板⑤）；docdup：LSH 对暴力精确 Jaccard oracle 召回 ≥ 99%（硬）+ 审计精度 ≥ 85%（in-corpus GT 分母 ≥5 才逐语料设门）+ license/骨架豁免类零行进上报集（拍板④） |
| **M5-3B** 深度去冗·判决 | 三信号 join、`score`+棘轮（`check`/`baseline` 子命令归此）、Haskell 语言支持**全套**（size+CC+CoC+注释域 + graph 阶梯按 M5-2f 每 rung fixture 纪律；先决 = tree-sitter-haskell 0.26 ABI 可得性 spike——拍板⑧） | 3 周 ± | join 不设数值门（验收 = 诚实包 + 图腿缺席发 null 绝不编造——拍板④）；score 敏感性电池绿（非空性 + 互异性双前置）；本仓库自身跑通棘轮入 CI；Haskell 阶梯每 rung fixture 全绿（grammar 不可得 ⇒ size-only 落回并公开记录，CoC 与阶梯顺延） |
| **M6** GUI+结构管理器（**已收口 2026-08-17，v2.0 修正案**） | structure/1 家族（树尺度熵判决：C 自参照地板+A 声明覆盖、七轴 S0-S6、判决全 Haskell 测量复用 Rust——设计册 reviews/2026-08-17-m6-structure-manager.md（git 历史） 四切片）+ `ce structure` JSON 树报告 + Tauri 可视化（树图首屏消费同一 schema） | 3–4 周 | 熵原语过穷举参照电池 ✓；每轴 F16 非真空前置 ✓；每片反事实杠杆+golden 手算 ✓；对 10 万 LOC 仓库**从冷启动 scan 到首屏** < 60s、已扫描报告打开 < 3s（实测 zod 71.6k 冷 8.36s/暖 2.66s，[PERF-BUDGET.md](PERF-BUDGET.md) M6 节）✓；Windows 实包（NSIS）+三平台编译门 ✓（Linux/macOS 实件=M7） |
| **M7** 发布（**已收口 2026-08-18**） | marketplace 上架、未签名明示（签名/公证裁定不做——2026-08-19，见 ADR-007/R1）、Releases 自动化（**含 Linux/macOS 实包**——v2.0）、完整 MCP（只读报告面——章程拍板③）、许可证合规（NOTICE/第三方 MIT 署名清单——D1-7）、文档、**GUI 二期：趋势面板+删除候选浏览**（v2.0 移入）、**M7.5 清理批（v2.3，P6 前置）：休眠评估仪器深度瘦身走 EVAL-SET 修正案 + trend/1 趋势判决入核（Haskell 合约内抬占比——2026-08-18 拍板 ccm #1152）** | 1–2 周 | 陌生机器一条命令可用 ✓；二进制 SHA256 校验链路端到端验证 ✓；**仓库转公开前全历史审计**（历史内 cli/memory/memory.db 三处 blob〔64780b9/e296178/d3f48df〕必须 filter-repo 清除、transcript、密钥、路径泄漏——D2-7）✓；文档过 `docdup` 自检 ✓；默认档位切换依据（各规则 FPR 数据）发布在 CHANGELOG ✓；M7.5 后 CI 活门集合零缺员（普查测试级二分实证）✓ |
| **M8** 成长轨（**已收口 2026-08-19，随 v0.3.0 发布**） | IP（软著 eCO 已递交待批 + 商标裁定不注册：™ 主张即可，用户拍板 2026-08-20——G1）、全量文档对齐 + 生成器门控（docs/reference/{cli,ce-toml}.md 由二进制/schema 生成，`docs_gate.rs` CI 门）、i18n（en 默认 + zh 查表：CLI/GUI/roast，机器面永不翻译）、GitHub 可见度（README 徽章、官网 codeeraser.dev、crates.io + npm 指针、应用图标）；契约=reviews/2026-08-17-m8-growth-track.md（git 历史） | 1–2 周 | 生成式参考漂移即 CI 红 ✓；`--lang zh` 全行覆盖且 JSON/FAIL 词汇不译 ✓；v0.3.0 发布链十资产 + 六 pin 四重复验全绿 ✓ |
| **M9** 打磨与完整面（v2.8 立项 2026-08-21） | 批 **0** v0.7.3 出货批：spawn 全站点无窗化（单一 helper 统一 corelink/churn/fourclass/daemon 四站点——unified fix）+ 裸 `ce` 印 help 退 0 + trend 默认分批渲染进度，随后完整发版链；批 **1** 两车道审阅 113 条全量落地（台账=项目 memory `audit-2026-08-21-two-lane-findings.md`；CONFIRMED 33 优先、minors 全清、与后续批重叠项归并不重复干）+ 全 md 防漂移扫尾；批 **2** 方法学册 `docs/reference/methodology.md`（每判决族数学实现——winnowing 指纹/TSED/结构熵七轴/评分与棘轮/图存活性/三信号 join/ROI 缝价，公式逐条引 file:line 禁凭印象）+ 细节版原理图 + 官网 How-it-works 页 + README 挂链；批 **3** `ce erase` 两段式（契约册 `docs/reference/erase.md` 先行）；批 **4** GUI 完整化：全报告族入面、erase 预览-应用、长任务进度事件、root 选择器/记忆（装机版 CWD=安装目录之弊）、设计系统翻新（v2.9 起并入终版不单发）；批 **5** bench dashboard：`eval/` 回放 harness 产 `bench.json` 单源（延迟 P50/P95、精度、召回、FPR），per-tag 回填 + 官网/README/GUI 三面同源渲染；批 **6** 实战检验：≥2 真实仓库全家桶实测入案例册（反哺批 5 与缺陷修复）；批 **7** 判决回迁第三弹（盘点 Rust 内残余判决性逻辑→迁移清单→逐片入核，wire 按 proto minor 加性；禁止为占比写代码——ADR-008 纪律沿用）；批 **8** 架构级全仓代码审查（v2.10 用户令 1.0 前置：模块边界与依赖结构〔import SCC 清单为线索〕、wire 十族契约一致性、核模块架构、GUI/测试架构、错误处理与降级路径；多 agent+逐条对抗核验，修复或书面 disposition 不悬空）；批 **9** 全域优雅性终打磨（数学/算法/架构/GUI 再设计思考，含 GUI 美化挂账项；提案册先行，契约/门槛变更走拍板；**v2.11 增补**：GUI Graph 可视化收进本批 GUI 波——文件级引用图 canvas 视图+死码/环判决叠加，复用 deadcode 判决运行的 Rust 投影零 proto 变更，符号级下钻不入围，2026-08-21 拍板） | 分批推进 | 每批五门+CI 全绿；批 0 发 v0.7.3，批 1–9 落 main，终版 **v1.0.0** 走完整发版链收口（v2.9/v2.10）；erase 触碰用户文件必 dry-run 默认+干净工作区前置+补丁与判决同源可审计；dashboard 每个数字可由回放复现（禁手填）；回迁片产品判决面字节等价 golden 承重；FPR 纪律与 guard 档位 opt-in 不破；弹窗根修以 windowed 进程实测零弹窗验收（**已收口 2026-08-22**：批 0–9 + 终扫〔113 对账 81 修 29 处置、716 断言终核、截图/官网真值重取〕全清，v1.0.0 发版链走毕即项目收口） |
| **K–L** 收口轨（K 轮 v2.14 **已交付 2026-08-26** 随 v1.2.0；K+1 CI 减重批 7dc391d→cd4ea86 用户拍板；**L 轮 v2.17 立项 2026-08-26，甲-1 封版 2026-08-27**） | 甲 步 3b 符号层顾问全链 + 乙 围栏收尾 19 条 + 丙 文档/记账/实录清欠 9 条（全文见 v2.18 banner 条）+ **v2.18 两步**：减法批（7 条 ≈116 行 + 3 缺陷）+ 测试子仓（cli/tests + gui/tests → skymanbp/CodeEraser-tests submodule）；具名后置束 M（评分/评测 12 + 产品小项 7）/ N（分发 20）/ 证据门 4 | 3–4 周 ± | 甲：口径对抗审查册**已封版**（v9，2026-08-27：九轮双把、第七至九轮核验者 Workflow 零 REFUTED、第九轮双把零 breaks；定案见 v2.17 banner 条）→ 片序 (1)→(8)：symEdges 退役（SCHEMA 14 wipe，**已落 dd20947**）→ 可见性三位字 + bit 0 三修 T3/H2+H4/H5（GRAPH_REV 11，wire 掩码 bit 0，K27 差分入 CHANGELOG，**已落 2026-08-27**）→ mentions 表（自有第二 walk + 三发射器分词 + 两加性表存 fnv1a64、`MENTION_REV` 1 自有版本行、自有入口 `ce graph --mentions`，K39–K42 四腿 + secrets 表八条加宽同表，**已落 2026-08-27**）→ conv 类别字列（`symbols.conv` 存 AST 半八位、名表半 wire 时算，`mention_name` 域抽取器 + `ce:allow` 解析合一 `crate::allow`，GRAPH_REV 12，K24/K25/K28/K37 生产者半，**已落 2026-08-27**）→ `mounts` 表生产者 + cabal 两字段 + TS `export_star` 站点（GRAPH_REV 13，冻结自仓切片一行改签，K30 生产者半 e2e 六格 + 再导出/Go/cabal/无 lib 包，R5 梯级召回界具名——用户裁同日入提取面补强步作梯级修正案〔见本条 v2.17 提取面补强句〕，**已落 2026-08-27**）→ graph/1 6.2.0 加性两表 `unmentioned`/`mounts`（`symbols` 不动、legacy 请求字节恒等）+ `export_unmentioned` golden 反事实（约定式可达名零误报腿 K33；`CE.Graph.Advisory` 自有类 + Rust 候选生产者 `mention/candidates` + report 0.3.0 三面原子，**已落 2026-08-27**）→ 渲染面 + K23 四语料 + K45 双腿 + 方法学册 13（0.3.0 第三键 `unmentioned_cut`、GUI 符号级下钻、`ce graph --mentions` 0.2.0 `rates` 普查、U 公式钉 + 258 行外部处置零 veto 缺陷，**已落 2026-08-28**）+ 自仓 101 条候选逐条处置；乙：19 条各一腿红/绿反事实；丙：9 条就地改 + PreToolUse 全链实录数入 PERF-BUDGET + §2 复扫只变短；收口判定式 87/87 具名，台账 memory/frontier-2026-08-26 |

**依赖**：M2←M1；M3←M2；M4←M3；M5-2←M4（churn 是三信号一腿，**串行**——A4）；M5-3A←M5-2；M5-3B←M5-3A；M6 可与 M5 并行；M7 收尾；M8←M7（成长轨，契约在册——v2.2；G1 IP 材料可与 M7 并行，商标先于 P6）；M9←M8（批 0 先行，批 1–7 依批序，批 6 可与 4/5 并行；批 8←批 7，批 9←批 8，终扫←批 9——v2.10；K+1/L 为收口后轨：L←K，甲乙丙三批可并行；M←L（评分束吃甲的符号事实），N 独立——v2.17）。总计粗估 4–6 个月。

## 7. 质量与测试策略

1. **跨语言契约测试**：contracts/ 同一批 golden fixtures，Rust 生成 IR、Haskell 判决，
   round-trip 断言；schema 变更必须 bump 版本（机制 M0 起，内容 M4 定稿）。
2. **交叉核对**：度量与 lizard / radon / rust-code-analysis / Sonar 例题对拍，
   fixtures 来自钉死 commit 的真实仓库随机抽样，分歧 case 显式收录。
3. **property-based**：Rust 侧 proptest（索引增量 ≡ 全量重建）；Haskell 侧 M4 用 base-only
   确定性测试；M5-2g 起判定不变量属性电池（caps 单调/边表严格升序/SCC 性质/Cost 消融；
   固定种子 base-only 生成器，不动 freeze——2026-08-12 拍板，取代"推迟 M5 再评"）。
4. **评分敏感性测试**（B2）：扰动任一权重断言总分变化——直接针对 fuck-u-code 的
   死字段 bug 形态。
5. **Dogfooding**：M1 起 CI 对仓库强制 `ce scan` 退出码门；M5 起 core/（Haskell
   支持就位）+ 棘轮（`--fail-under` 属 `ce check`）。本文件受行数棘轮与 `docdup` 约束。
6. **性能预算进 CI 基准**，回归即 fail。锚点如实标注：jscpd 3.44s/17K 文件是**批扫**
   数据，只锚定 M2 的全量索引预算；热路径预算不由它推出，由 M0 分解表实测建立（A6）。

## 8. 风险登记册

| # | 风险 | 缓解 |
|---|---|---|
| R1 | Haskell/Windows 工具链 | GHC 9.14 LTS 锁版 + M0 实测依赖可解；CI 必含 windows-latest；stdio 全程 binary mode；禁 DLL；未签名二进制的 Defender/EDR 误报风险 → 签名裁定不做（2026-08-19），SHA256 链承重，README 明示 |
| R2 | tree-sitter 语法 crate 漂移 | 核心锁 0.26.x、语法 crate 按 lockfile 钉住；升级走独立 PR + golden 全绿 |
| R3 | hook 延迟劣化 → 用户关插件 | daemon + 增量索引；分解表预算进 CI；超时 fail-open：探针不可达时**不输出任何决定**（guard.rs 的 reasons 为空即早返），该次运行以 degraded 记入 observe feed 并进 doctor 降级计数——A9f；「降级为 warn」只发生在 `ce.toml` 无法解析这一条路上 |
| R4 | 误报 → 信任崩塌 | 分级 warn/ask/deny + 演进路线（§4.2）；deny 准入 = M4 FPR 门（≤1%）；豁免带 why；每判决附量化依据 |
| R5 | 竞品挤压（**触发器式**，A8） | 监测触发器：jscpd/desloppify 发布 diff 级 gating，或 Claude Code 内置类似能力 → 差异化收缩至三信号 join + 四分类，届时 M5 join 提前、热路径查重改评估复用竞品引擎。**复扫 2026-08-27（v2.17 换版 plan-set 例行）：未击发**——jscpd 5.x README 与 jscpd-rs 0.1.12 无 diff/changed-files gating，desloppify 原句仍在，Claude Code 官方 changelog 无内置查重（仅社区 hook 方案与 issue #10170 功能请求） |
| R6 | 双语言成本先于价值支付（B1） | Haskell 承重首战后移到 M4（判决层），M0 只付骨架+握手的最小成本；契约内容不提前冻结；core 不依赖跟随 GHC 版本发布的库（stan 教训） |
| R7 | "处处 deny"招致反感 | 默认档位演进路线写死在 §4.2，不是永久 warn 也不是上来就 deny；排除模型 M1 内置 |
| R8 | 四分类（L2）不达标 | fallback 阶梯：退 L1（git+函数边界）仍可交付；deny 降级 ask 如实标注 |

## 9. 已拍板决策（2026-08-06 grill 问答，12 项，用户逐项确认）

License=**Apache-2.0**（LICENSE 已入库）· 前端=**Rust** 定案 · 仓库**私有开发、M7 公开** ·
guard 默认档位=**渐进路线**（§4.2）· M1 语言集=**TS/Python/Rust/Go/Markdown** ·
集成优先级=**Claude Code 优先** · GUI=**按 M6** · 发布节奏=**M3 后 0.x 预览** ·
Haskell 边界=**判决层**（ADR-002 确认）· docdup 域=**Markdown/纯文本 + 代码注释/docstring** ·
CLI=**`ce`**（codeeraser 作 alias，M0 撞名核查）· 幽默评语=**默认关闭彩蛋**（`--roast` 开启）。

*里程碑推进以本计划锁定版为准；任何"顺手先写点代码"的行为违反本计划。*
