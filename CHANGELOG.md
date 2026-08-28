# Changelog

> 记录义务来自 DEVELOPMENT_PLAN.md §4.2：“每次默认档位变更在
> CHANGELOG 记录依据（FPR 数据）。”本册只记守卫档位变更；
> 功能发布史见 GitHub Releases。

## [Unreleased] — L 轮片 (2)+(3)+(4)+(5)（无默认档位变更；bit 0 生产者三修，erase reason 6 人口随之变动；secrets 排除表加宽，判决宇宙随之变动；`ce:allow` 解析合一，旧宽松形三收窄一放宽，自仓一段转活；TS 星号再导出自 `export_from` 分出 `export_star` 站点标签，冻结自仓切片一行改签）

**片 (5) 无默认档位变更**。`mounts` 表生产者落地（`cli/src/graph/mounts.rs`：每节点 `[privateMounts, totalMounts, bits]`，
全节点无条件一行；private/total 由 `mod_decl` 边 join 声明 `mod` 单元的存储可见性 bit 0；bits bit 0 = 再导出目标 = Rust
`via_reexport=1` 边 ∪ TS `export_star` 站点目标；bit 1 = 包私有 = Go `package main` ∨ `internal/` 段〔`_test.go` 除外〕、Cargo
无 lib target ⇒ 整包否则 `[[bin]]` 根〔`Package::bin_roots`〕、cabal 无 `library` stanza ⇒ 整包否则模块仅在 `other-modules`
〔`Cabal::{has_library, hidden_modules}` 两字段〕），wire 未接（片 (6)）。**站点宇宙变动一处**：TS `export * from` /
`export * as ns from` 自本版起开 `export_star` 站点（`store::KINDS` 末位追加，位次 10），不再计作 `export_from`——GRAPH_REV
12→13：全体站点重检、缓存键失配走整库重建、trend 同清（发版说明义务）；冻结自仓切片 `contracts/eval/graph-slice-v1.json`
`packages__zod__src__v4__mini__index.ts` 行 `export_from: 1` 改签 `export_star: 1`、summary `ts/export_from` 键同改（RG3 常备
成本，活门 `eval_graph` 两腿重新绿）。**冻结册口径分界**：只有自仓切片被活门重检并改签；zod/requests/ripgrep/cobra 四册切片、
审计样本 `graph-sample-v1.json` 与精度册均早于 rev 13，其中 TS 星号再导出仍计在 `export_from` 下（zod 切片 `ts/export_from`
118 含此类），冻结对冻结不动、审计记录不改——跨语料的站点标签自本版起不再逐字可比，重算须复活生成器（EVAL-SET.md）。
**bit 0 的 Rust 臂继承 R5 召回界，具名**：`rs_use::bound` 只在「单终点走完仍有剩余段」且门面条目以 `crate::`/`self::`/`super::`
开头时绑定（K30 形：包名梯级 `use fixture::Thing` 落 `lib.rs` 剩 `Thing`，再走门面 `pub use crate::source::Thing` 到定义文件；
非根门面 `crate::facade::Thing` 同样可绑——两覆盖根降到同一文件）；不绑定的三形：通配门面（`pub use x::*`，rs_reexport 不跟）、
uniform-path 门面（`pub use source::Thing`，裸头按 crate 名走 R4 即止）、lib+bin 同包内自非根模块出发且走到根为止的导入
（`use crate::Thing` / `use crate::{…}`：两覆盖根两终点 ⇒ `AmbiguousRoot`）。自仓索引实数（rev 13 树）：`via_reexport=1` 边
**0**；`cli/src` 217 个 `crate::` 站点里无边 2（皆 `crate::{…}` 组形）、裸本地模块头无边 26（`report::`/`model::`/`main_cli::`…）；
`cli/tests/it` 另有 108 个 `crate::` 无边站点，成因是嵌套测试根 `tests/it/main.rs` 不在 `crate_roots` 模型内（`cargo.rs` 已具名
的深层自动发现召回界），与 R5 无关。这是梯级的召回界不是本表的；用户裁 2026-08-27 作 §4 修正案、入提取面补强步（裸头先读
本地模块再按 crate 名走 R4、lib+bin 包根终端在包内绑定，随 R5 精度册重跑 + RG3 具名），本版按封版口径只读事实。对抗审查（16 agent）：
确认 3 全修（cabal 首根遮蔽改全根任一、Go 块注释内 `package` 两向误读改带状态读子句、Rust 臂逐文件重扫改逐清单一次），
分歧 1 修（同上第三项），未核 6 与 nit 13 逐条处置（含本段两处措辞与实数改正、命名子库 `library x` 不算公共库、
幻影节点零事实由走集键控见证），驳回 2（`pub(crate) mod` 按封版 §4 bit 0 仍为导出挂载）。自仓差分：`ce dedup .`
恒 185（K30 e2e 首稿的元组表被自家门抓出，改文本表）、`ce scan .` 0 fail、lib 193 / it 201 + 2 ignored、`ce check` 954。

**片 (4) 无默认档位变更**。`symbols`（SQLite）新列 `conv` 存约定类别字的 **AST 半**（GRAPH_REV 11→12：全体
符号行重导出，缓存键失配走整库重建，trend 同清——发版说明义务）：bit 1 TEST 的 Rust `cfg` 谓词臂、bit 2 FFI
（Rust 导出属性/`extern`、Haskell `foreign export`、Go `//export`）、bit 3 REGISTRATION（Python 注册装饰器表、
TS 任意装饰器）、bit 5 MEMBER（Python 类体）、bit 8 DEFAULT_EXPORT、bit 9 AMBIENT、bit 10 ALLOW 的
`allow/expect(dead_code)` 臂、bit 11 CFG；名表半在 wire 组装时算、零存储（片 (6)）。`mention_name`
（`cli/src/mention/name.rs`）钉判决域 = 声明文件自身分词臂下的单 token 名。**`ce:allow(<tag>)` 解析合一**为
`crate::allow::allow_claim`，文法即计划 §4.1 成文形 `ce:allow(<rule>) -- <why>`（标记—空白—`--`—空白—同行非空
why）。与两处旧解析器的差分具名四条：docdup 原容许标记与 `--` 之间夹任意文字、原容许 `--why` 不带空白（两形
收窄）；deadcode 原接受空 why（收窄）、原只认 `--` 后一个空格（`--<TAB>why` 自本版起算作声明——唯一放宽项，
方向 = 多一文件免死判）。自仓差分：源码零标记（`git grep` 实测，仅测试与解析器自身）；判决文档四处标记里
`contracts/VERSIONING.md:353`、`docs/reference/methodology/06…md:214`（`ce:allow(deadcode) -- why` 散文，两文件
因此带 ROLE_ALLOW，新旧同判）与 `plugin/skills/erase/SKILL.md:57`（`ce:allow(docdup) -- <why>`，新旧同判）
不动，`docs/reference/methodology/03…md:25` 标记后先接反引号再 `--`——旧文法算声明、新文法不算，该段自
`inline_allow` 转活：`ce docdup .` exempt_allow 2→1、allow_missing_why 1→2、live segments 673→674，dups 恒 0。

**片 (3) 无默认档位变更**。提及语料宇宙 pass 落地（`ce graph --mentions`，索引两张加性表 `mention_files`/`mentions`
只存 fnv1a64 哈希，`MENTION_REV` 自有版本行，不在 guard/audit 热路径）。与之同表的 secrets 排除 glob 自四条
（`.env`/`*.pem`/`id_*`/`*.key`）加宽为八条（`.env*`/`*.pem`/`*.key`/`id_*`/`.npmrc`/`.pypirc`/`.netrc`/`*credentials*`，
口径册 S-A9）：**判决 walk 同一张表**，故名含 `credentials` 的源文件自本版起离开 scan/dedup/graph 判决宇宙——
隐私失效方向为安全（少判一文件，永不把密钥写进索引）；自仓差分具名为空（无此类文件）。

**片 (2) 无默认档位变更**。`symbols.flags` bit 0 是 erase reason 6 `public_surface` 与 join `publicGuard`
的输入，其三处生产者缺陷按口径册 §4 修复；许可增量与一次口径收窄不可区分，故按 §4.2 铁律与晋级
同门记依据（K27 差分 = 片 (1) 后二进制 vs 本修，fixture 语料逐文件；自仓差分表**具名为空**）：
T3（TS，扩宽）恒等守卫下 zod 912f0f5 差分 83 行 0→1、0 行 1→0（16 文件，皆 `export const f = () =>`
第三跳形）；H2/H4（Haskell，收窄）hsprobe2/3 差分 3 行 1→0（where 同名 ×1、class 缺省体/instance 体
×2）；H5（Haskell，扩宽）hsprobe4/6/8/9 + `module M` 自再导出差分 6 行 0→1。存储字另加 bit 1/2
（作用域导出/受限），wire 只发 bit 0（`symwire.rs` 掩码，十族 golden 逐字节不变）。

## [v1.2.0] — 2026-08-26（无默认档位变更；T1/T2 写入类规则语义收窄）

**无默认档位变更；T1/T2 写入类的规则语义收窄为「只拦新引入的重复」**
（K 步 11 根修，2026-08-26；仲裁 = 用户授权下的模型仲裁，授权 2026-08-24）。
复活 M3 重放仪器全史复测：requests 尾段 487 事件 0 误拦；自仓 2274 事件在
出厂全文语义下 505 拦截（122 对首燃 = 90 真引入 + 32 拆叶/并叶中间态），
32 条中间态若计误拦 = 1.41% > §4.2 M4 门（≤1%）。处置（用户拍板 2026-08-26）
= 根修而非降档：`guard::novel_matches` 以被替换内容为基线减法（Write=盘上
旧全文、Edit=old_string），复测 505→139（84 真引入 + 32 中间态 + 23 延展
复燃），拦截词教搬移安全序（先削源后写即过）；两类已晋级规则**维持 deny**。
observe feed 0.7.0（`matches` = 新引入数，语义断点具名，前后不可直比）。
Stop 审计 / precommit / zone 三类**维持 observe**：现行代 observe 台账
stop_audit/precommit 样本数为 0，§4.2 类准入门（每会话 M4 台账、≥200 样本、
纯净度）今日不可满足，如实缺席。全记录：docs/FPR-REPLAY.md「K 轮复跑」节。

## [v1.1.0] — 2026-08-24（无默认档位变更）

无档位变更：两类晋级规则仍 deny、其余仍 observe、渐进区仍须显式
`[guard] zone_tiers = true`；本周期无新增独立 FPR 记录，按 §4.2 维持原档。
硬预算规则的**取线来源**改为文件所属 `[[rules.class]]` 的 `file_lines_fail`
（无类 = 全局 750，`guard::budget::lines_for`，44a0abd）——同一规则、同一档，
只是 deny 的那条线与 CI 硬墙同源；未声明类的仓库逐字节不变。功能面与
分数可比性声明见 GitHub Release v1.1.0。

## [v1.0.1] — 2026-08-22（无默认档位变更）

无档位变更、无判决语义变更，分数与 v1.0.0 完全可比。本版是分发面
维护：Windows 安装器装机时探测 Claude Code 并自动接入 codeeraser
插件（一次安装即整个产品；卸载只对称拆除它自己添加的注册）。

## [v1.0.0] — 2026-08-22（无默认档位变更；v0.7.1–v0.7.3 亦均无）

自 v0.7.0 起无默认档位变更：`PROMOTED_DEFAULT` 两类（T1/T2 重复写入、
硬预算超限）仍 deny，其余规则仍默认 observe，渐进区仍须显式
`[guard] zone_tiers = true`。本周期无新增独立 FPR 记录，按 §4.2
「无记录即无晋升」维持原档。边界修正一处（非档位）：hook 车道的
usage error 现打印后退 **1**——clap 原退 2 正是 PreToolUse deny 码，
usage 失败必须 fail-open，不得拦停会话。功能发布史与分数迁移声明
见 GitHub Release v1.0.0。

## [v0.7.0] — 2026-08-20（无默认档位变更；v0.4.0–v0.6.0 亦均无）

新增 **opt-in** 渐进区位置→档位映射（计划 v2.7 ①）：`ce.toml` 显式
声明 `[guard] zone_tiers = true` 后，软线→硬线区间按位置映射
<25% observe / 25–75% warn / >75% ask（>H 仍走硬预算规则）。
**默认恒 observe**——默认翻档仍以观察台账攒出的各档 FPR 记录为准
（§4.2 纪律同款，无记录即无晋升）；为此观察 schema 0.5.0 → 0.6.0，
`zone` 事件行在武装仓库加记 `zone_tier` 映射档，为未来晋升留账。

## [v0.1.0] — 2026-08-18 首发（下列两条均随其发布；v0.2.0、v0.3.0 均无档位变更）

### 2026-08-17 — 默认档位晋升：§4.2 路线第 3 级（1.0，M7-P2）

**变更**：`[guard]` 未显式设 `mode` 时，两类已晋级 PreToolUse 规则
默认从 ask 升为 **deny**（单一权威常量 `config::PROMOTED_DEFAULT`，
guard 执行面与 health/doctor 报告面同源）：

1. **T1/T2 精确/改名重复写入**；
2. **硬预算超限**（写后文件 > 750 行）。

Stop 审计 / precommit **不晋级**，默认维持 observe：两者无独立
FPR 记录在案（§4.2 第 3 级原文"其余规则默认 ask/warn 按各自 FPR
记录决定"——无记录即无晋级资格，如实缺席）。显式 `mode` 依旧
统一覆盖全部规则类。

**依据（FPR 数据，与第 2 级同一账本，无新增拦截记录在案）**：

- T1/T2 探针：630 次真实编辑重放 0 误拦（docs/FPR-REPLAY.md
  2026-08-07：requests 487 + 自仓 143）；
- 判定层主门：600 全语料真实正常编辑重放 0 标记 = 0% ≤ 1%
  （contracts/eval/fpr-fourclass-v1.json）;
- 硬预算：自仓全史 first-parent 380 个 A/M 事件按排除模型 0 触发
  （2026-08-11 条目的复现命令不变）。

### 2026-08-11 — 默认档位晋升：§4.2 路线第 2 级

**变更**：`ce.toml` 的 `[guard]` 未显式设 `mode` 时，两类 PreToolUse
规则默认从 observe 升为 **ask**：

1. **T1/T2 精确/改名重复写入**（M3 探针，daemon 索引比对）；
2. **硬预算超限**——写后文件 > 750 行（本级新增规则：按 Edit/Write
   自身语义在内存精确套用后计行，走 scan 同源排除模型
   `scan::walk::in_scope`，免 daemon）。

Stop 审计 / precommit 不在晋升类，默认仍 observe；显式 `mode` 统一
覆盖全部规则类。观察档 schema 随之 0.3.0 → 0.4.0（新增 `budget`
事件行，为 §4.2 第 3 级的按规则 FPR 记录留账）。

**触发条件**：计划 §4.2 第 2 级 “M4 的 FPR 门（§6）通过后”——已于
2026-08-11 通过。

**依据（FPR 数据）**：

- T1/T2 探针：630 次真实编辑事件重放（requests 完整历史尾段 487 +
  CodeEraser 自仓全史 143），仲裁后误拦 **0**，折算 0.00/500 ≤ 1
  （docs/FPR-REPLAY.md，2026-08-07）。
- M4 判定层主门：600 全语料真实正常编辑重放误报 **0/600** = 0% ≤ 1%
  （contracts/eval/fpr-fourclass-v1.json：samples 600、flagged 0、
  gate_max_percent 1）。
- 硬预算规则：判定为精确行数算术，无统计误报模态；对自仓全史
  first-parent 重放（五门首发语言扩展名，380 个 A/M 事件）超限事件
  5 个，全部落在 `contracts/fixtures/crosscheck/**`（钉定第三方对照
  物，ce.toml 排除项覆盖）——按已装排除模型计发火 **0/380**。复现：
  `git rev-list --reverse --first-parent HEAD` 逐提交
  `git diff-tree -r --root --no-renames --diff-filter=AM` 取
  `rs|py|ts|tsx|go|md` 文件，`git cat-file -p <sha>:<file> | wc -l`
  与 750 比较。
