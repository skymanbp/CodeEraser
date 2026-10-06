# contracts/ — 契约版本化机制 归档册（proto 3.0.0–5.1.0）

> 3.0.0–5.1.0 七条版本条目，即 [VERSIONING.md](VERSIONING.md) 倒序段在 2.15.0–2.33.0 迁出之后最老的一截（更早的 2.15.0–2.33.0 在 [VERSIONING-ARCHIVE-2.15-2.33.md](VERSIONING-ARCHIVE-2.15-2.33.md)、2.1.0–2.14.0 在 [VERSIONING-ARCHIVE-2.1-2.14.md](VERSIONING-ARCHIVE-2.1-2.14.md)，
> 2.0.0 一条仍在正册的导语段里）。2026-10-06 从正册迁出：算法轨 v2.33 W6 的 9.0.0 同版加性段落下后正册读 751 行，过了 `ce scan` 的
> 750 行硬线，按仓规「拆分优先于豁免」与前两册同法另起一册。条目逐字节搬家，未改写，仍按版本倒序；信封、SemVer 协商规则与
> fixtures 约定只由正册声明，本册与正册同一立场（不冻结：条目所述的规则若改，仍在条目上就地改写）。

> **5.1.0**（规则包围栏 + per-class 棘轮容差 minor，K 轮步 4，2026-08-25，用户拍板 v2.14 ②）：
> ①`verdict.request` 加性标量 `classDigest`——对 `[[rules.class]]` 规范化声明（名、**声明序**的 globs、旋钮）
> 的指纹。名与 glob 仍永不过线（§5.9.2）：它们的哈希不是它们。编码为**长度前缀**（netstring 式 `tag:len:bytes`）
> 而非分隔符——首版靠 fnv1a 的 NUL 分隔，自带的腿当场抓到碰撞：名 `a` 带 glob `b` 与名 `a\0glob\0b` 无 glob
> 字节流全等（分隔符只能分隔不含它的东西，长度可以）。②`ce-baseline.json` 记录其天花板**在哪套规则包下立的**，
> 核加持名 fail 条件 `class_digest`，判据是**朴素的 Maybe 不等**且是全的：两边皆无=同意；改了规则包=不同意；
> 对着围栏之前的旧基线声明规则包=不同意；把基线记过的规则包删掉=也不同意。四种分歧要的是同一个答案：
> 具名说出来，让人去具名重立一条地板。只有 establish 写 digest（`CE_ACCEPT_BASELINE=1` 走空基线路），
> 故「同意一套新规则包」与「同意一条新地板」是同一个动作。③`classKnobs` 码域 0..2 → **0..3**，码 3 =
> 该类自己的棘轮容差（行数**绝对值**，非比例——想要它的是 vendored 与夹具树，它们要的是零或固定额度，
> 而大文件的百分比正是本旋钮要拿掉的白拿增长）。声明即**取代两条全局腿**，故 0 意味着一行都不许长、
> 全局 max(+2%,+10) 救不了它（因为根本没被查询）。它是唯一「零有意义」的类旋钮，故表的取值下界**按码判**
> 而非一刀切（码 0/1/2 是线，线为零是荒谬）。反事实：K11 = 无类声明仓 digest 缺席（**不是 null**）且 101 对
> 金样中 199 改动行里 197 行只动 proto 字段、另 2 行是不匹配文案内嵌 server 版本串〔核电池另有一腿断言
> newBaseline 无该键〕、K12 = 改规则包即 `failed=["class_digest"]` 而 `over` 为空——不是悄悄放松而是具名停下
> 〔fixtures/verdict pair 16〕、K13 = establish 记下 digest 且棘轮行仍三列、K14 = 类容差 0 时长一行即 over
> 且 allowed=天花板本身〔pair 17；全局 +10 腿够不着〕；另有 Rust 侧三腿钉指纹本身（声明序/名/glob/旋钮各一，
> 「零旋钮」≠「无旋钮」，以及长度前缀的单射性）。请求行随 minor 机器重写为 5.1.0；核电池请求侧 proto 同步 22 处。
> **5.0.0**（graph 节点行 legacy flags 列裁除 **major**，K 轮步 3d，2026-08-25）：节点行降为
> `[lang, kind, roles]` **单一元**——pre-2.28 的 flags 列离场。它自 2.28.0 roles 列成为权威后又被
> 生产、上线、丢弃了七个 minor；4.0.0 想同批砍掉却被实测拦下（flags 位 0 是公私判决轴，可见性无生产者时删列会让
> `unref_public`/`unreach_public` 连夹具都无法表达），4.1.0 的 `symbols` 表补上那个生产者，此条遂解锁。
> **档位**：§2 写死「schema 不兼容变更（删字段/改字段形状）必须 bump major」，删列正是改行形状，故 major——
> 计划原写 minor，2026-08-25 按本仓自己的规则修正（v2.14 就地记账）。代价为零：4.x 全程未发布（v1.1.0 出货 3.2.0）。
> **三列同元不同义**是有意为之：新三列 = lang/粒度/角色事实，旧三列 = lang/粒度/flags；major 在信封处拒绝一切
> 跨版本对话，那道拒绝正是使元数复用安全的机制，故 K1 由「按行元拒」改为「按 major 拒」。表级
> `node rows: mixed arity` 拒绝随之退役——只剩一种合法元数时，宽窄不对的行就是 malformed，且按**行下标**点名。
> Rust 侧 `flags::legacy_flags` 与 `LEGACY` 折叠表一并删除，随之退役的还有 `legacy_fold_is_the_pre_228_bits`
> 一条测试与 allow-claim 测试里的一行断言（电池名集差实测：Rust −1/+0，核 −1/+1 同一探针改口径）。
> 反事实：**语义保持**——夹具 pair 7 把同一批事实改走各自的通道（节点 0 的入口身份走 roles 0→flag 位 1，
> 节点 3/5 的导出面走 `symbols`），回复与 4.1.0 **逐字段相同**（dead 表码 1/2/3/4 齐全、pos、cycles、counts 皆同），
> 证明这是裁除而非语义迁移；99 对金样中 199 行改动、193 行只动 proto 字段，另 6 行 = 三条我方重塑的请求
> （pair 7/11/12）+ pair 13 的新 malformed 文案 + 两条内嵌 server 版本串的错误文案。
> 请求行随 major 机器重写为 5.0.0；核电池请求侧 proto 同步 16 处。
> **4.1.0**（导出面 minor，K 轮步 3c，2026-08-25，用户三度交本代理裁断 v2.14 K7）：`graph.request` 加性一键——
> `symbols=[[node,visibility]]`，node < 节点数、visibility ≥ 0、**严格升序**（该表是去重的 (节点, 可见性) 集合，
> 重复行=生产者丢了集合语义，按名拒 `symbol i: not strictly ascending`）。core 按 `Cost.exportVisBit`
> 读出导出节点、按 `Cost.publicFlagBit` 或上 flags 位 0——那正是 `Dead.deadTable` 一直在分的公私判决轴，
> 而它**从来没有过生产者**（`cli/src/graph/deadcode/flags.rs:9`：文件粒度永不置位，公开性是符号事实）。
> 判决码 2/4（`unref_public`/`unreach_public`）自此首次可达。该位**故意在 entryMask 之外**：导出面是判决轴、
> 不是入口主张（RG10），故它只改死节点报哪个码，永不改哪些节点死。缺席**与空表同路**（`symRows` 只喂 [] ），
> 字节与 4.0.0 客户端所得相同。表另计 `symCap`。**L 轮片 (2)（2026-08-27）起本表的 visibility 是存储字的 bit 0 投影**：
> `symbols.flags` 另存 bit 1（作用域导出）与 bit 2（`pub(crate)` 族受限）供后续 `unmentioned` 表用，`symwire.rs`
> 的 `SELECT DISTINCT … flags & 1` 在查询处掩码，本表字节与 `symCap` 定容皆不动（K34）。**同批未做**：原计划并列的 `symEdges` 不上线——K10 审计量的是
> 精度（683/683），而「无引用」吃的是召回，实测自仓 import 绑定只覆盖 1064 条 Rust 导出声明中的 170 条
> （补模块跳转到 248 条，~23%），漏掉的是全路径调用与方法调用（皆非 import 点位）。详见 DEVELOPMENT_PLAN v2.14 K7。
> **L 轮终裁（2026-08-27，用户拍板 ①）：删**——`symedges.rs`/`bindings.rs` 与 index 的 `bindings` 表随 schema v14 退役
> （提及否决器批片 (1)，DEVELOPMENT_PLAN v2.17 条；包含论证：有符号边必有某 import 行出现过该 token，否决器完全包含它），
> wire 面零变动（`symEdges` 从未上线）。
> 反事实：K5 = 无符号表/空符号表与 4.0.0 逐字节相同（99 对机器重生成后逐行对拍：195 改动行中 193 行只动 proto 字段、
> 2 行是不匹配文案内嵌的 server 版本串；核电池另有一腿直接比 `respond` 两次的字节）、K6 = 请求体无任何字符串叶子
> （`cli/tests/it/graph_export_surface.rs`，结构性断言而非按本夹具的路径列举）、K9 = 导出节点判 2 而其邻居仍判 1，
> 且死集合不动（`fixtures/graph` pair 16 + 核电池 `exportRides`）；两个旋钮各有反事实腿（读错可见性位=无面、
> 置 entryMask 内的位=该节点变入口而离开判决集）。请求行随 minor 机器重写为 4.1.0；核电池请求侧 proto 同步 19 处（Haskell 字面量 11 + Spec.hs 内嵌请求 8；`9.0.0` 的外来 major 探针不动）。
> **4.0.0**（erase class 0 退役 **major**，K 轮步 2，2026-08-24，用户拍板 v2.14）：`erase.request` 的 class 0
> （dead_file 本地计数路）自 2.32.0 被 class 3 取代、Rust 同 minor 起不再铸行，宽限窗至此关闭——**离开判决集**，
> 其冻结位保留并**按名拒绝**（`row i: retired class 0 (superseded by 3 at 2.32.0, retired 4.0.0)`），而非折进
> 「unknown class」：仍在发它的客户端由此得知接替它的是哪条路。位不重编——重编会为省一个数组槽而移动另外三个冻结码，
> `CLASS_NAMES` 改留 `(retired)` 占位（二义的两个 dead_file 同死）。纯裁除故走 major。**同批未做**：graph 节点行的
> pre-2.28 legacy flags 列本拟同批退役，实测拦下——flags 位 0（exported）是公私判决轴，符号表给可见性第一个真生产者
> 之前删列会让 `unref_public`/`unreach_public` 连夹具都无法表达（对拍实证：旧 golden 含码 2/4，删列重生成后归零），
> 故顺延至符号表落地后的 minor。反事实：K2 = class 0 行按名被拒（fixtures/erase pair 8）、K4 = 未受影响九族回复
> 除 proto 串外逐字节相同（98 对机器重生成后逐行对拍，仅 erase 两行按等价迁至 class 3、wire-errors 两条错误文案
> 内嵌 server 版本串）。请求行随 major 机器重写为 4.0.0（3.0.0 先例，§3）；核电池请求侧 proto 同步 19 处。
> **3.2.0**（规则包 scan 旁表 minor，I 轮 P3，2026-08-24，用户拍板 v2.13 ①）：`scan.request` 加性两键——
> `rowClasses=[classId…]` 与 rows **位置对齐**（长度必等、每项 < 64；缺席 = 全行走全局表）与
> `gradeOverrides=[[classId,code,warn,fail]]`（classId ≥ 1、code 0..6、阶梯同 grades 文法〔fail 0 = 无硬线、
> fail ≥ warn〕、(classId,code) 严格升序；仅非空时发）；core 按 (class,code) 查表回落全局有效表，
> `grades` 回显仍为全局表，`gradeOverrides` 到场且非 degraded 时原样回显（客户端断言往返）；两表计入
> scanRowCap；chunk 切分时类列随行同切。ce.toml 侧 `[[rules.class]].knobs` 增 `fn_lines_warn` /
> `fn_lines_fail`（P3 两键）；Rust 镜像 evaluate 按文件类取有效阈值，每判对拍恒等式覆盖到类。
> 无声明仓库 wire 字节不变。
> **3.1.0**（规则包 DSL v1 minor，I 轮 P1+P2，2026-08-24，用户拍板 v2.13 ①）：①`verdict.request` 的
> `continuous` 行可携第 4 列 **classId**——`[u_fp, metricCode, value, classId]`，路径类的 1 基声明
> 序号，0 = 默认类；全表单 arity，混排拒 `continuous rows: mixed arity`；classId < 64（栅栏 classCap）；
> 身份前缀宽 2 不变，棘轮只读三列前缀；②加性新表 `classKnobs=[[classId,code,value]]`——码域 =
> ceilings 恒发子集 {0,1,2}（sizeCeil / cocCeil / sizeHard 的类影子，**不新造码**），classId ≥ 1
> （类 0 即全局表，已有 ceilings 通道）、value ≥ 1、(classId,code) 严格升序；core 建 Map 求值、
> 缺键回落全局线，chargeAt 律与机会数不动；③回复在表到场时**原样回显** `classKnobs`（客户端断言
> 往返；无表 = 无键，旧回复字节不变）；④`newBaseline` **永三列**（类是本 run 收费参数，非棘轮
> 事实）；⑤ce.toml 侧 `[[rules.class]]`：name/globs 仅本地（§5.9.2），globset 与 exclude 同方言，
> 声明序首中，classCap 64，逐类 ladder_fault 于 load 咽喉；无声明仓库的 wire 字节不变（C1）。
> **声明一个类 = 分数迁移**（§2 发版声明义务同款）：类线一经声明，该类文件的轴 0（sizeMass 的 S/H）
> 与轴 1（cocOver 上限）换线收费，分数与声明前**不可比**；未声明 `[[rules.class]]` 的仓库判决与
> wire 字节均不变，分数序列照旧可比。
> 反事实证表 C1–C9 = core/test/ClassProps.hs + cli 侧 config_contract / scan::classes 电池。
> **3.0.0**（churn 行裁列 **major**，I 轮 D3，2026-08-24，用户拍板「现在就删」）：`verdict.request` 的 `churn` 表由五列 `[u,rewrite,append,added,survived]` **收窄为三列**
> `[u,rewrite,append]`——第 4 列恒等于 rewrite+append、第 5 列恒为 0（per-entity 存活从未测量），
> core 自 M5-3i 起两列全弃读（`Score.churnHeavy` / `Verdict.churnMap` 只解 rw/ap）；删列 = 请求形状
> 破坏性变更，按 §2 升 major：两侧实现 + 三个 core 测试 harness 的 proto 字面量 + **全十族 golden**
> 同批重生（请求行 proto 一律改写为 3.0.0；回复行经核机器再生，与旧回复除 proto/server 字串外
> 逐字节相同——判决面零变化的亲证）；「留+记愿望单」落选（用户裁）。同批 daemon 协议独立升
