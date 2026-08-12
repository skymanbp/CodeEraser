# contracts/ — 契约版本化机制（M0 冻结机制，内容 M4 定稿 1.0.0）

> 依据 DEVELOPMENT_PLAN.md §7.1 与评审 B1：M0 只冻结**版本化机制**，
> IR/判决 schema 的**内容**在 M4 随真实需求定稿为 1.0（2026-08-11，
> wire 形状与 0.2.0 一致的声明性定稿）。**2.0.0**（M5-1c-iii，
> 2026-08-12）：rem/add 条目携第三元素 = trim 后 alnum 宽度，喂
> Cost.anchorFloor 的站点锚地板——请求形状破坏性变更，按 §2 升 major。
> **2.1.0**（M5-2a，2026-08-12）：graph/1 族落地——加法 type + 加法
> capability，按 §2 minor；行字节预检同批放宽（见 §1）。

## 1. 信封（envelope）

ce ↔ ce-core 的每条消息 = 一行 NDJSON（UTF-8，无 BOM，`\n` 结尾，binary-mode I/O）。
每条消息必带信封字段，其余字段由 `type` 决定：

```json
{"proto": "<SemVer>", "type": "<message-type>", ...}
```

- `proto`：协议版本，当前 **2.1.0**（单一来源：`cli/src/corelink.rs::PROTO`
  与 `core/app/CE/Protocol.hs::proto`，两处必须一致，由共享 fixture 钉住）。
- 未知**额外**字段必须被接收方忽略（同 major 内前向兼容）。
- 未知 `type` → **`error` 应答**（0.2.0 起；此前实现以 hello 形状拒绝，属缺陷已修）：
  `{"proto","type":"error","id":<回显|null>,"code","message"}`，
  `code ∈ {unknown_type, bad_request, too_large, contract}`。core 侧在 JSON 解析
  **之前**先做行字节上限预检（2.1.0 起 32 MiB，此前 1 MiB——2026-08-12 决策：
  唯一客户端是同机受信 daemon，而 graph 请求在 100k LOC 量级合法地 ~1 MB；
  真防护 = 各族容量护栏），超限即 `too_large`，不解析。
- **每条非 hello 消息的 `proto` 由 core 强制校验**（1.0.0 定稿修正，攻击评审 F8：
  0.x 实现只在 hello 协商，裸发/错 major 的请求曾被静默应答）：缺失或 major
  不符 → `error/bad_request`。hello 自身仍走 §2 协商应答（`accept:false` 更富）。
- `hello` 应答自 0.2.0 起带 `capabilities`（当前 `["hello","fourclass/2","graph/1"]`；
  /2 = 2.0.0 的锚宽请求形状——旧客户端探 /1 得缺席，响亮降级 L1 而非发不可解析的
  二元形状；graph/1 = M5-2 图族）——**纯信息发现**，接受/拒绝的唯一权威仍是
  §2 的 SemVer；能力缺席 = 客户端走 L1 并显式降级（A9f）。
- 客户端规则：应答 `type` 非预期或 `id` 不回显 = 失步 → 视为 L2 不可用，
  回退 L1 且降级可见——绝不给错答案，只给响亮的答案。
- `fourclass.request`（2.0.0 形状）：`{"id","pairs":[{"i","rem":[[[行,hash,宽],…],…],
  "add":[…],"dup":[keyhash]}]}`——rem/add 为 L1 判 novel/deleted 的**显著**行按
  **run 分组**（run 结构=对齐产物，Rust 侧产出），hash = fnv1a(trim)，宽 =
  trim 后 alnum 计数（行事实，Cost.anchorFloor 的判定输入）；`dup` =
  after 侧新出现重复的**顶层具名单元**键哈希（堆叠证据，符号知识留在 Rust，
  仅哈希过线——ADR-002 A6）；`i` 为**稠密 0 基文件对位置**（与发送方批内文件对
  数组共同索引；1.0.0 定稿明确——攻击评审 F9：接收方按位置回查，稀疏 id 不受支持，
  跨匹配要求 `i` 不同）。
  within-first 前置（同对 add∩rem 必空）由 core 在边界校验，违反 → `error/contract`。
- `fourclass.result`：`{"id","moved":[[i,出行,入行]],"blocks":[[源i,源行,宿i,宿行]],
  "suspicions":[[i,规则名]],"degraded"(,"reason"∈{bucket_cap})}`——moved 为单调
  重分类 delta；blocks 为 ≥2 行站点证据（扩展/归因行只进 moved 不进 blocks）；
  suspicions 为 M4 判定规则点火记录（堆叠常数在 CE.FourClass.Verdict）。
- `graph.request`（2.1.0 起）：`{"id","nodes":[[lang,kind,flags]],"edges":
  [[src,dst,kind,rung]],"unresolved":[[lang,kind,reason,count]],"pos":[idx]}`——
  稠密 0 基索引即节点身份，**无文本形物过线**（ADR-002 A6）；边严格升序且去重、
  端点与 pos 越界 → `error/contract`（边界契约由 core 机检）；超
  `CE.Graph.Cost` 节点/边护栏 → `graph.result` 带 `degraded:true,
  "reason":"graph_too_large"`（绝不截断）。**M5-2a 桩**：通过校验的请求一律
  `error/contract`（算法在 M5-2g 随穷举参照 harness 落地，桩不发明判决）。
- `graph.result`：`{"id","dead":[[idx,verdict]],"pos":[[idx,indeg,outdeg,sccId,
  sccSize,reachIn]],"cycles":[[sccId,[idx]]],"counts":{"nodes","edges","kept"},
  "degraded"(,"reason"∈{graph_too_large})}`（形状 2.1.0 预留，语义 M5-2g）。

## 2. SemVer 协商规则

- **major 不同 = 拒绝**：应答 `accept:false` + `reason`，调用方报错退出。
- minor/patch 不同 = 接受（新字段走"忽略未知字段"规则）。
- 破坏性变更（删字段/改语义）必须 bump major，并同步更新两侧实现 + fixtures。
- **信封常数变更**（行字节预检、错误码/reason 词汇扩充）：放宽 = minor（旧客户端
  照常工作），收紧 = major；变更必须在 §1 就地改写并注明日期与依据（2.1.0 的
  32 MiB 放宽为首例）。

## 3. Fixtures 约定

- `fixtures/handshake/`：wire golden（请求行 + 期望应答行交替；`hello-ok` 握手、
  `wire-errors` 错误应答），Rust（`cli/tests/core_wire.rs`）与 Haskell
  （`core/test/Spec.hs`）**逐字节**共同消费——同一份文件，防两侧实现漂移。
  字节比较可靠因为 freeze 钉 `aeson +ordered-keymap`（键序确定）。
- `fixtures/hook-payloads/`：Claude Code `PreToolUse(Edit|Write)` 的**实测** stdin
  dump（官方文档无逐字示例，ADR-007 ⚠️ 项）。采集方式见该目录 README。
- fixture 变更 = 契约变更，走 §2 规则。

## 4. 工具链锁定（M0 验收项）

| 组件 | 锁定 | 载体 |
|---|---|---|
| Rust | 1.94.1 | `cli/rust-toolchain.toml` |
| GHC | 9.14.1（LTS） | CI `ghc-version` + 本文件 |
| 依赖快照 | cabal freeze | `core/cabal.project.freeze`（GHC 就绪后 `cabal freeze` 生成入库） |
| 协议 | 2.1.0 | §1 所列两处常量 |
