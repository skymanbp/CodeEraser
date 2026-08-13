# M5-2 `graph` 设计定稿（2026-08-12）

> 产出流程：设计工作流 10 agents——3 读者（cli-Rust / core-Haskell / 评估仪器模式）
> → 3 独立设计（resolution-first / judgment-first / instrument-first）→ 3 透镜评审
> （contract / provability / realism，逐引用对仓核实）→ 合成。聚合分 39/39/33，
> 透镜判决 2–1 归 instrument-first。合成 = instrument-first 供骨架（SITES 冻结宇宙、
> 祖先断言、GRAPH_SCOPE、三数诚实包、zod/cobra 语料）+ resolution-first 供全身
> （5×6 阶梯表、两相失效、dst_path-TEXT、Markdown 入 files 论证、流形↔判决双射、
> E01 拆分入退出判据）+ judgment-first 供判定安放（rung=Rust 测的事实 / 判定=Haskell
> 常数、entryMask、Position.hs、死旋钮测试、枚举扩到判决函数、wire 体积计算）。
> 用户四项 blocking 拍板 + 六项默认采纳见 §8。评审全文存工作流 transcript（机器本地）。

## 0. 评审抓出的缺陷与处置（合成层已消化）

| # | 缺陷（出处） | 处置 |
|---|---|---|
| D1 | instrument-first 的 resolution_rate ≥ 66.7% 门把 stdlib/外部记为未解析→requests 上结构性不可达 | rate=(resolved+external)/N；锚可比数=GT 推导 recall；门 vs 报告→决策 6 |
| D2 | 负对照不可达：ce.toml:7 排除 crosscheck/** | GRAPH_SCOPE 显式覆盖该排除纳入孤岛 fixture（实冻结 23 行 = 20 个代码孤岛四语言各 5 + 3 个 md 文档；负对照义务只落在代码孤岛的 import 上——2b-iii Opus 反审核正，设计原文"10"仅数了 ts+go）。**2cd 反审 F3 修订（2026-08-12）**：负对照义务限于**跨文件/跨包**引用——孤岛 rs 的 `self::`/`super::` 站点目标在本文件内（冻结 GT 行 walk.rs:220 `use self::DirEntryInner::*` → 同文件 :131 的 enum，审计表 self.json 实证 ≥13 站同形），此类站点的正确判决 = 解析回本文件，不是 external |
| D3 | 祖先断言需 .git，但 eval 族声明无 git | 独立 git 测试 cli/tests/graph_provenance.rs；eval_graph.rs 族保持纯 |
| D4 | resolution-first 判决词汇缺"该解析而未解析"→recall 不可导 | 判决加 `missed`；GT 记 truth，precision/recall 同出 100 行 |
| D5 | dedup/mod.rs 拆分低估（index_all 仅 27 行） | index_all + load_streams（:133-181）同迁 dedup/walkidx.rs |
| D6 | 前两设计未算 wire 体积；judgment-first 算出 ~1.1 MB > maxLineBytes=1048576（Protocol.hs:29-30 已核） | 编码前 nodeCap/edgeCap 检查→degraded/graph_too_large 绝不截断；预检上调→决策 7 |
| D7 | judgment-first 的 u64 nid 上 wire 引入碰撞类 | 弃——只走稠密 0 基索引；ADR-002 A6 不放宽 |
| D8 | judgment-first 把 maxLineBytes 提到 64 MiB 放松文档化常数 | 不作默认；上呈为决策 7 |
| D9 | SCHEMA_VERSION 3→4 连 fingerprints 一起 DROP（index.rs:19/:192 已核）⇒ 存量用户全量重建 | 上呈为决策 8；默认=接受一次重建、单 DB |
| D10 | 每语言地板：10 vs 15 | 15，冻结为 min_per_lang 且 CI 断言（G5） |
| D11 | 两处误引（review/mod.rs 路径缩写；session.rs:142 是 Stop 汇总非健康行） | 全文改正：cli/tests/eval_commit_review/mod.rs；健康行=cli/src/health.rs:79-97 |

## 1. 模块布局（E01：文件 300 警/750 拒；core/** CI 硬 300）

**新 Rust——`cli/src/graph/`**（兄弟模块；scan/spec.rs 不得增边 kind）：
`mod.rs` 90（`ce graph`/`ce deadcode` 入口，analyze(root)→(Nodes,Edges,Ledger)）·
`spec.rs` 150（EdgeSpec 表驱动，kinds.rs:42-47 元组表形状）·
`sites.rs` 130（**免解析**站点检测=冻结宇宙；复用 scan::ast::children、fourclass::units::owner）·
`md.rs` 120（Markdown 链接扫描 + GitHub anchor slugger，围栏/行内码感知；Markdown 无 grammar，行扫描零新依赖）·
`roots.rs` 130（tsconfig(+extends)/package.json/pyproject/go.mod/Cargo workspace；serde_json+toml 已有）·
`ladder/mod.rs` 70（Rung、Outcome{Resolved,External,Unresolved(reason)}、分发器）·
`ladder/{ts,py,rs,go,md}.rs` 150/120/140/90/70 ·
`store.rs` 150（symbols/sites/edges DDL + resolve_key 门）·
`wire.rs` 110（稠密索引构建、回复消费、回复后名字归属——batch.rs:265-293 形状）。
**合计 ≈1520 新 Rust 行**（决策 10 的依据数字，如实陈列）。

**改动 Rust——拆分是退出判据不是意向**：dedup/index.rs 263→SCHEMA/ensure_cache_key/
meta_entries 迁新 dedup/schema.rs ~80，然后 SCHEMA_VERSION 3→4 + graph_rev 入
meta_entries；dedup/mod.rs 271→index_all+load_streams 迁 dedup/walkidx.rs ~80；
daemon/proto.rs 93→115；daemon/server.rs 253→285（越 300 则拆 daemon/replies.rs）；
main.rs 263→288（**余量 12 行，已标记**，再增即拆 main_cmds.rs）；config.rs 90→115
（[graph]）；corelink.rs:18 PROTO→"2.1.0"。

**新 Haskell——`core/app/CE/`**（零新依赖：containers==0.8 双 stanza 已在
ce-core.cabal:20,40 + freeze:26，Data.Graph 可用⇒计划 R6 不触发）：
`Graph.hs` 45（边界契约+respond，FourClass.hs:17-27 形状）· `Graph/Wire.hs` 95 ·
`Graph/Build.hs` 90（Data.Set 去重→graphFromEdges）· `Graph/Dead.hs` 85（四路判决）·
`Graph/Cycles.hs` 60 · `Graph/Position.hs` 55（仅按请求 pos 节点算）·
`Graph/Cost.hs` 55（**纯常数**，唯一消融靶——FourClass/Cost.hs:1-6 先例）。
改动：Protocol.hs 82→85、Handshake.hs +"graph/1"、cabal **双** other-modules stanza。
**测试**：core/test/ReferenceGraph.hs ~120——暴力参照住 core/test/ 绝不入 core/app/
（否则吃 300 门）；不并入 Reference.hs（已 ~150）。

## 2. Wire——族 `graph/1`，proto 2.1.0

加法 type + 加法 capability ⇒ minor（VERSIONING.md:51）。成本前置声明：proto 回显在
每条回复（Protocol.hs:77），**全部现有 golden 重生成**——在 M5-2a 以独立机械提交支付，
diff 只准触 "proto"/"capabilities" 两值，grep 可验。

```jsonc
{"proto":"2.1.0","type":"graph.request","id":7,
 "nodes":[[lang,kind,flags], …],          // 索引=身份；无名字、无哈希
 "edges":[[src,dst,kind,rung], …],        // 稠密索引；严格升序、去重
 "unresolved":[[lang,kind,reason,count], …],   // 诚实台账是 wire 级事实
 "pos":[idx, …]}                          // join 焦点；空 ⇒ 只回计数
{"proto":"2.1.0","type":"graph.result","id":7,
 "dead":[[idx,verdict], …],               // 1 unref_private 2 unref_public 3 unreach_private 4 unreach_public
 "pos":[[idx,indeg,outdeg,sccId,sccSize,reachIn], …],
 "cycles":[[sccId,[idx, …]], …],
 "counts":{"nodes":n,"edges":e,"kept":k},
 "degraded":false}                        // true 时带 reason ∈ {graph_too_large}
```

**无文本形物过 wire**——稠密 0 基索引（fourclass `i` 惯例），ADR-002 A6 不放宽。四路
verdict 把 dead vs unreachable 两轴（入度×可达性）结构化分开，`unreferenced_public`
与 dead 结构隔离——R4 信任崩塌风险拿到防火墙而非政策。
**判定安放**：rung 是 **Rust 测的事实**；"哪些 rung 算引用"= `Cost.minRung`（Integer，
纯常数模块，可消融，死旋钮藏不住）。flags 位（0 exported·1 main·2 test·3 entry-glob·
4 dyn-referenced·5 doc-entry·6 ce:allow(deadcode) 豁免）由 Rust 按 ce.toml glob 与
语法置位；**`Cost.entryMask` 在 Haskell 决定什么算入口**——驱动 deadcode FPR 的唯一常数。
**边界契约**（code:"contract" 指名违约者）：所有端点与 pos 索引 < |nodes|；edges 严格
升序无重复——Rust 的 sort+dedup 成为被检查的前置条件；graphFromEdges 按排序键编号 ⇒
结果是边集的函数，字节确定性由结构证明而非断言。
**体积（算出来的）**：100k LOC ≈ 20k 节点/60k 边 ≈ 1.1–1.2 MB > maxLineBytes=1048576
（Protocol.hs:26-30 已核）。默认：编码前 `Cost.nodeCap`/`edgeCap`（Integer 化——
Anchor.hs 溢出教训）→ degraded/graph_too_large，绝不截断；graph_too_large 入
VERSIONING.md:45 reason 词汇。预检上调=决策 7（已拍板 b）。

## 3. SQLite 增量——schema v4，单 DB（daemon 唯一写者）

```sql
CREATE TABLE symbols (id INTEGER PRIMARY KEY,
  file_id INTEGER NOT NULL REFERENCES files(id) ON DELETE CASCADE,
  key TEXT NOT NULL, start_line INTEGER NOT NULL, end_line INTEGER NOT NULL,
  flags INTEGER NOT NULL);
CREATE TABLE sites (id INTEGER PRIMARY KEY,               -- 冻结宇宙
  file_id INTEGER NOT NULL REFERENCES files(id) ON DELETE CASCADE,
  kind INTEGER NOT NULL, line INTEGER NOT NULL, spec TEXT NOT NULL, owner TEXT);
CREATE TABLE edges (site_id INTEGER NOT NULL REFERENCES sites(id) ON DELETE CASCADE,
  dst_path TEXT NOT NULL, dst_unit TEXT NOT NULL,          -- TEXT，有意不做 FK
  kind INTEGER NOT NULL, rung INTEGER NOT NULL, granularity INTEGER NOT NULL);
ALTER TABLE files ADD has_tokens INTEGER NOT NULL;         -- Markdown 以 0 进入
CREATE INDEX idx_sym_file ON symbols(file_id, key);
CREATE INDEX idx_site_file ON sites(file_id);
CREATE INDEX idx_edge_dst ON edges(dst_path);
```

`key` 复用既有单元词汇（name/arity、(T) add/1、impl Tr for T——units.rs）。所有读取
先排序再返回。**dst_path 是 TEXT 不是 FK**（已核理由：站点可指向缺席/被排除文件，
为满足 FK 物化幻影 files 行会腐蚀 dedup）；file_id 读取时按 files.path join。
**两相失效，因为边不是按文件可失效的——假装可失效正是 bug**：相 1（symbols/sites）
= 单文件字节的纯函数 ⇒ 在 refresh_file 既有单事务内删后插（content_hash 门不变）；
相 2（edges）依赖全文件集**与配置文件**——`resolve_key = fnv1a(sorted in-scope paths
‖ 各 tsconfig/package.json/pyproject/go.mod/Cargo.toml 字节哈希)`；未变 ⇒ 相 2 整体
跳过；变 ⇒ DELETE FROM edges + 对**缓存**站点一次解析（免重 parse），单事务。
`graph_rev` 与 tokenizer_rev 并列入 meta_entries——抽取/阶梯语义改动即清陈旧行。
**Markdown 入 files**（token_count=0, has_tokens=0，零指纹行）。已核安全：
all_instances 从 fingerprints 侧 join ⇒ 零 Markdown 指纹 ⇒ 零实例 ⇒ 201 块棘轮
结构性不动。二阶陷阱：Summary.files 只数 has_tokens=1，否则 dedup-report/0.5.0 须升版。
**诚实极限（写进模块头不粉饰）**：图行与指纹行由各自 content-hash 门控，两事务间
崩溃留单侧陈旧——各自可自检，不存在单一原子刷新，我们不声称有。

## 4. 解析阶梯——逐级、逐语言、带立场

统一契约：站点按序走 rung；**首个恰给一个 in-scope 候选**的 rung 解析之。**某 rung
>1 候选 ⇒ Unresolved(ambiguous_*)，绝不猜最优**——挑选即发明路径。终态 External
（stdlib/registry/依赖）是**正确答案**不是 miss。每条边存 rung ⇒ 精度可按级归因，
脏 rung 以数据除名。Unresolved 理由：external·dynamic·ambiguous_*·macro·
config_depth·out_of_scope·unsupported。

| lang | R1（确定） | R2 | R3 | R4 | R5 |
|---|---|---|---|---|---|
| TS/TSX | 相对 `./x` + TS 扩展序 `.ts,.tsx,.d.ts,.mts,.cts,/index.*`——**顺序即规范**，多命中非歧义 | ESM `./x.js`→`x.ts` 改写，仅当 x.ts 存在∧x.js 不存在 | 最近 tsconfig baseUrl+paths，extends 链深 ≤8 带环检 ⇒ 否则 config_depth；双 paths 命中 ⇒ ambiguous_paths | workspace 成员 name/exports 子路径 | 裸说明符在 dependencies 或 node_modules/ ⇒ External；否则 Unresolved |
| Python | `from .`/`..`——点数走 `__init__.py` 包链 | 绝对点路 vs 源根 {repo 根, src/, pyproject packages}；双根命中 ⇒ ambiguous_root | `a/b/__init__.py` 存在而 `a/b/c.py` 不存在 ⇒ **文件级边指 __init__.py，符号有意留未解析**——降级非猜测 | stdlib/site-packages ⇒ External | importlib(var)/`__import__` ⇒ dynamic——Python 主要 recall 损失，设计使然 |
| Rust | `mod foo;`→dir/foo.rs \| dir/foo/mod.rs；crate 根出 [[bin]]/lib.rs——唯一建模块结构的 rung | `use crate::a::b::C` 走 **R1 导出**的树 | self::/super:: 同树 | workspace 成员或 path= 依赖→其根文件；registry 依赖 ⇒ External | `#[path="…"]` 字面量回 R1；宏生成 mod/use ⇒ macro；`pub use` 跟 **≤1 跳**边指再导出者，标 via_reexport |
| Go | 最近 go.mod module 前缀→包**目录**；边扇出到每个非 _test.go 文件，granularity=package（归到单文件即猜测）；节点身份 (pkg_dir,"") | replace 指令 + 嵌套 go.mod | stdlib 表，或首段带点不匹配本地 module ⇒ External | //go:build 排除文件仍发边，标 build_constrained——tag 求值需要我们没有的构建配置，交判定层 | 否则 Unresolved |
| Markdown | 带扩展相对链接→join+exists | +#anchor 按 GitHub slug 规则（含 -1/-2 重复后缀）对 ATX 标题 ⇒ 节级边；slug 集非 1:1 ⇒ 降文件级+ambiguous_anchor | 引用式 [t][id] + 文内 [id]: ./x.md，再走 R1/R2 | 裸 #anchor ⇒ 文内节边 | http(s)/mailto ⇒ External；图片 kind=asset 不入 deadcode；否则 Unresolved |

**每级精度立场（待测量，绝不断言已达）**：R1 ~100%（miss 是 bug 不是旋钮）；R2 ≥98%；
R3 ≥95%；R4（单跳再导出，跳数入档）≥90%；R5（已解析文件内符号绑定；未绑 ⇒ 文件级边）
≥90%。**R6=全仓唯一名调用匹配不实现**——锚点用精度换 recall 的唯一处，我们唯一拒绝处
（条件解锁见 §6 2i）。
**Markdown 语义显式**：节点=每文件 1 + 每 ATX 节 1。边 kind：doc_link（doc→doc 节级）、
doc_ref（doc→代码文件）——计划 :74 "无引用文档段落"双向可算。**排除**：围栏/行内码内
一切、裸 URL、未用引用定义——均计为站点并解析为 external，排除在台账可见而非静默。
doc 入口默认 README.md + docs/ 索引 + CLAUDE.md（[graph] entry_globs）；无入口规则则
每个 doc 平凡地死。
**调用边**骑阶梯不与之竞争：被调方由已解析 import 边绑定或同文件定义才成边。入度依赖
未解析调用的符号报 unresolvable_indegree **绝不声称 dead**（决策 5）。

## 5. 预注册评估仪器

**宇宙 = SITES 非边**。精度分母若=边集则解析器自选分母——拒绝难边即抬精度。站点宇宙
只需 tree-sitter kind 表 ⇒ 在 ladder/ 存在前冻结；precision/recall/resolution_rate
同出 100 行。
**范围**：GRAPH_SCOPE（克隆自 COMMIT_SCOPE 冻结扩展 pathspec，colordiff.rs:209-217），
**显式覆盖** ce.toml:7 的 crosscheck/** 排除仅纳 10 孤岛 fixture——设计内负对照：其
import 无 in-corpus 目标，必须落 external；"解析"了它们的解析器被审计当场抓获。
**语料**：self + requests + ripgrep（已克隆）+ **zod@912f0f5 + cobra@adbc881**
（SOURCES.md:11-12 已钉，决策 1 拍板）。
**抽样**——哈希排名、无 RNG 无时钟（freeze.rs 先例）：`rank_key = sha256("ce-graph-
site-v1|"‖corpus‖commit‖path‖line‖kind‖spec)`；分层 (lang,kind)；**预注册地板
min_per_lang=15** 先于最大余数法分配剩余 25（5×15+25=100；纯比例给 TS/Go ~2 = 静默
砸 D2-4）。第二独立域分隔抽样 `ce-graph-audit-v1|` 定审计序 + **100 行后备**。第三
重叠抽样 `ce-graph-rung-v1|`（每 rung 15）供 minRung 定值，**明确不是门**；重叠允许
且入档——同站点双标签必须同判决=免费审计者一致性检查（**2cd 反审 F13 澄清**：
rung 需解析器实测方存在，故 rung 重叠抽样只能在 2f 后抽取，该一致性检查结构性
非盲——价值是审计者自一致性，不是盲评）。verify() 重哈希载荷拒重复 id。
**冻结档**（contracts/eval/，按语料名后缀）：graph-slice-{corpus}-v1.json（宇宙：
钉全 OID、GRAPH_SCOPE、文件清单+sha256、(lang,kind) 站点计数、逐项 excluded 计数、
**测量前写死**的证伪常数 min_per_lang=15 / r0_share_trigger=0.80）·
graph-sample-v1.json（100+后备；**路径包含**——freeze.rs 无路径隐私规则是因其样本
内嵌他人私仓，本语料公开或自有，区别陈述不照搬）· graph-audit-{corpus}-v1.json（GT）·
graph-precision-{corpus}-v1.json（判分+三台账+每级 cut 表）。
**人工审计流**：逐站点在钉定 OID 读源：{corpus,path,line,kind,spec,truth:
"path#unit"|external|dynamic|ambiguous|none, why}；why 必填且须指名**机制**。附带
**site_gaps** 巡检：文件打开时顺手记录检测器漏掉的带边构造。审计表是**数据**非 Rust
常量——cli/tests/eval_graph_review/{self,requests,ripgrep,cobra,zod}.json，经
include_str!+OnceLock，**首日即按语料名解析**（gate_docs——M5-1d C3 教训：门读错
语料的书还绿着）。
**指标**：precision=correct/(correct+wrong) 只对已答站点——**门 ≥0.90**（计划 §6）。
recall=correct/|truth 在语料内|——与 66.7% 锚可比、免疫 stdlib 配比。
resolution_rate=(resolved+external)/N。r0_share=R1 解析占比。分母补足：审计沿冻结
排名序伸入后备直到 **100 已答行**——保住计划"100 条"字面功效。
**CI 门**（cli/tests/eval_graph.rs，纯 #[test] 无 git 无 core 二进制；summary 由
生成器同一 scorer 从行重导）：G1 summary 重导相等 · G2 每语料 precision ≥0.90
（**2cd 反审 F2 拍板 2026-08-12**：每语料门仅对 in-corpus GT 分母 ≥5 的语料生效
——实测 ripgrep 10/zod 22 达标，cobra 1/requests 3/self 4 带分母发布不设门；
计划字面的总体 100 条 ≥0.90 合同门不变）·
G3 守恒 correct+wrong+missed+external_ok+unresolved_ok==100 三层 · G4 **双射**
manifest id↔判决 id（缺行与幻影行同样响亮红）· G5 每语言 ≥15 审计站点**且各自报
带分母的精度**，低于地板即红非脚注 · G6 自证伪：wrong 行的站点在 (path,line,spec)
已不存在则自我作废；被豁免 unresolved 站点解析器居然答了同理 · G7 重复台账行在
共享咽喉拒绝 · G8 unresolved+wrong 台账冻结，新行须 CE_ACCEPT_GRAPH=1 赐福 ·
G9 反事实：幻影审计行必须变红（断言而非假设）· G10 配套完整性 corpus_doc_pairs
+ _frozen PENDING 变体——整语料不得静默失明 · G11 确定性：文件序洗牌 ⇒ 边表字节
同一；增量 ≡ 全量重建 · G12 生成器对 degraded 回复直接拒绝——被截断的跑分绝不入册 ·
G13（独立 git 测试 graph_provenance.rs）`git merge-base --is-ancestor <审计档提交>
<精度档 generated_from.commit>`——"先审计后跑分"成为被检查的事实。
**反怯懦三数+一规则**：(a) 全语料（非只样本）的 (lang,kind,rung) resolution_rate；
(b) 按理由分桶的 unresolved 台账；(c) site_gaps。method 字串明写**解析率是 recall
上界**（未检出构造不在分母）。**预注册 cut 规则（见数前写死）**：发布过 90% 的最
宽松 minRung，且无论如何公布完整每级 cut 表。r0_share>0.80 = 书面处置触发器（非红灯，
决策 6）。**未达标协议**：缺门语料档**扣发**，失败与失败 rung 写明——requests-L2 先例。
**预算注**：EVAL-SET.md 恰 300 行 = E01 警戒线且自身入 CI 扫描——M5-2 章须在同一 PR
内由压缩旧章支付。

## 6. 子里程碑序列

| # | 内容 | 退出（红条件） |
|---|---|---|
| 2a | proto 2.1.0 + graph/1 capability + 空 graph.respond 对一切输入回 contract。机械、独立、先行——在语义存在前支付 golden 翻批 | 两常数读 2.1.0；全部现有 golden 重生成且两侧字节同一消费；新 contracts/fixtures/graph/golden.ndjson 5 对（含悬空端点 contract；graph_too_large 落地时改运行时结构检查——Spec.hs oversize 先例，超容量 fixture 行 ~1.5 MB 不入库）。红：损坏单 golden 字节没让**两套**电池都红，或 diff 触及 proto/capabilities 之外 |
| 2b | GRAPH_SCOPE、graph/{sites,spec,md}.rs、`ce graph --sites`、slice 档（**无**解析器） | slice 档两次干净树跑字节同一；五语言站点数全非零；每站点 spec 是其源行子串。红：字节 diff，或围栏内链接形字串发出站点 |
| 2c | 抽样 + graph-sample-v1.json + 后备 | 两跑 id 同一；verify() 绿含重复 id 拒绝；每语言 ≥15；git log 证样本提交**先于**任何 cli/src/graph/ladder/ 文件。红：任一 |
| 2d | 人工审计（100+后备）+ site_gaps | 100/100 行带指名机制的非空 why；G13 祖先断言绿；每语料 site_gaps 巡检有交代（可为空） |
| 2e | schema v4、symbols/sites/edges、resolve_key、Markdown 入 files、两处 dedup/ 拆分 | (i) `ce dedup --check` 恰报 201 块；(ii) report.golden.json 字节同一；(iii) 增量≡全量；(iv) index.rs/dedup mod.rs/server.rs/main.rs 全 ≤300；(v) 100k LOC 全量重解析 <2s。红：任一 |
| 2f | 阶梯 R1-R5，每语言一 PR：TS→Py→Rust→Go→Md | 每 rung：≥1 恰在该级解析的 fixture + ≥1 必须保持 Unresolved 的歧义/动态 fixture。红：歧义 fixture 被解析，或 20 个 crosscheck 代码孤岛（2b-iii 核正计数）的**跨文件引用**解析成 external 之外任何东西；孤岛 intra-file `self::`/`super::` 站点须解析回其本文件（2cd 反审 F3 修订——原"一切孤岛站点必须 external"被冻结 GT 证伪：walk.rs:220→:131 同文件目标，一个正确的解析器会触发原红条件） |
| 2g | Haskell core + ReferenceGraph.hs | 暴力≡出品：(a) 4 标注顶点全部 2^16 有向图 × 3 固定入口集（入度/可达/SCC）；(b) 3 顶点全部 2^9 图 × 全部 2^3 入口集（四路判决全函数——唯一覆盖**入口集维度**的 pass）；前 3 失配报告。**死旋钮测试**：逐一扰动 minRung/entryMask/sccFloor 必须改变某 fixture 判决计数——直接对准计划 :75 指名的 fuck-u-code 死字段 bug |
| 2h | **达标线** + `ce deadcode` + Request::Graph + join 面 | 冻结 100 上每语料 precision ≥0.90；recall/resolution_rate/r0_share 带分母发布；每级 cut 表公布；自仓 deadcode 全处置（每发现带判决类+入口理由，unresolvable_indegree 零声称）；≥1 死符号经外部工具（rustc dead_code/vulture）佐证且分歧逐项列明；冷索引回显式 Error 绝不静默空图；降级理由到达 SessionStart 健康行（health.rs:79-97）与 Stop 汇总，e2e 断言。红：任一，或档静默扣发без PENDING 记录 |
| 2i（条件） | R6 全仓调用边 | 仅当授权：自己的 100 调用点审计样本 ≥0.90，否则 OFF 出厂并公布实测数 |

## 7. 风险登记册

| id | 风险 | 缓解/预写证伪触发器 |
|---|---|---|
| RG1 | 怯懦式精度（只解析平凡边） | GT 推导 recall + r0_share 0.80 重开触发器 + 理由分桶台账，全部测量前预注册 |
| RG2 | 检测器盲区使 recall 不可知 | site_gaps 巡检；"解析率是 recall 上界"写进 method 非暗示 |
| RG3 | 样本宇宙依赖检测器 | 检测器 2b 冻结先于解析器；**后续检测器改动 bump graph_rev、重冻 slice、样本作废 ⇒ 重审计**——常设成本，明言 |
| RG4 | resolve_key 抖动：加一文件重解析全仓 | 2e 退出实测；>2s/100k LOC 则收窄到 rung 咨询过变更目录的站点——触发器现在写死 |
| RG5 | tsconfig extends/paths 实践中无界 | 深 ≤8+环检+config_depth。**TS 站点 >5% 落 config_depth 即重开**——该 rung 欠建 |
| RG6 | wire ~1.1-1.2 MB vs 1 MiB 预检 | 编码前 cap→graph_too_large 绝不截断；决策 7 已拍板 b（同机受信放宽预检，a 为兜底） |
| RG7 | proto 2.1.0 重生成全部 golden | 2a 独立机械提交；diff 限两值，grep 可验 |
| RG8 | Markdown 入 files 动 dedup 棘轮 | 结构安全已核（读取从 fingerprints join）；2e 以 201+golden 字节同一钉死；预注册兜底=独立 doc_files 表 |
| RG9 | 文档节级死区 FP——多数标题从未被链接，入度 0 是常态 | 默认判决=文件级从 doc 入口集不可达；节级入度仅报告（决策 4） |
| RG10 | 库公共 API 被标死 ⇒ R4 信任崩塌 | unreferenced_public 独立 wire 判决类永不并入 dead——结构性非政策（决策 5） |
| RG11 | 反射/动态分发沉精度且造 deadcode FP | dyn_referenced 标志位，独立类目审计使成本可见 |
| RG12 | rung 膨胀：为 recall 加级静默降精度 | 每级精度冻结；加 rung 须重冻并审计该级 15 站点；每个 wrong 判决须指名其 rung |
| RG13 | E01 余量：main.rs +25/+37 | 子命令分发拆分预案；四文件全在 2e 退出判据内 |
| RG14 | 枚举 harness 撑爆 core/app 300 门 | 住 core/test/ReferenceGraph.hs，绝不入 core/app/ |
| RG15 | 审计者 100 判漂移 | why 强制指名机制；重叠抽样一致性检查；审计先于跑分提交（G13）。分歧**入档**不调和 |
| RG16 | 图表与指纹表跨崩溃瞬时不一致 | 独立 content-hash 门控 ⇒ 陈旧自检；模块头明言，永不声称原子性 |

## 8. 决策记录（2026-08-12 全部拍板/采纳）

Blocking 四项（AskUserQuestion，用户逐项确认，均选推荐项）：
**①TS/Go 语料** = zod@912f0f51b0ced654d0069741e7160834dca742ee +
cobra@adbc8813901bba65827259daa8e22ff94ec1f30e（crosscheck SOURCES.md:11-12 已钉，
继承既有 provenance）。**②调用边范围** = import + import-绑定调用；R6 为条件项 2i，
须独立 100 调用点审计 ≥90% 方开（计划 §4.1/§6 两处同时满足，零修订）。
**③wire 上限** = 同机受信子进程放宽预检，真保护移 nodeCap/edgeCap（Integer）→
graph_too_large；方案 a 保留为兜底；VERSIONING.md §1 常数文档随 2a/2g 更新。
**④M5 工期** = 拆 M5-3（T3/docdup/join/score/Haskell 语言支持顺延），计划 §6 已
就地改写（316 行棘轮守恒），M5-2=graph+deadcode 3–4 周 ±。
非 blocking 六项（按合成推荐采纳，已向用户明示可推翻）：Markdown 边只认链接语法
（行内码裸路径不计，代价入台账）；文档死区=文件级不可达为判决、节级入度仅报告；
unreferenced_public 独立类不 fail + unresolvable_indegree 带 caveat 报告；只门精度、
recall/r0_share 为发布数+书面处置触发器（0.80/0.667）；接受 schema v4 一次性全量
重建（pre-1.0 索引即缓存）；审计量=100+每级 15 重叠抽样（~175 行，minRung 数据定值）。
