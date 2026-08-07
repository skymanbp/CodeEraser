# contracts/ — 契约版本化机制（M0 冻结机制，内容随 M4 定稿）

> 依据 DEVELOPMENT_PLAN.md §7.1 与评审 B1：M0 只冻结**版本化机制**，
> IR/判决 schema 的**内容**在 M4 随真实需求定稿为 1.0。

## 1. 信封（envelope）

ce ↔ ce-core 的每条消息 = 一行 NDJSON（UTF-8，无 BOM，`\n` 结尾，binary-mode I/O）。
每条消息必带三个信封字段，其余字段由 `type` 决定：

```json
{"proto": "<SemVer>", "type": "<message-type>", ...}
```

- `proto`：协议版本，当前 **0.1.0**（单一来源：`cli/src/handshake.rs::PROTO`
  与 `core/app/CE/Handshake.hs::proto`，两处必须一致，由 handshake fixture 测试钉住）。
- 未知**额外**字段必须被接收方忽略（同 major 内前向兼容）。
- 未知 `type` → 回错误应答，不崩溃。

## 2. SemVer 协商规则

- **major 不同 = 拒绝**：应答 `accept:false` + `reason`，调用方报错退出。
- minor/patch 不同 = 接受（新字段走"忽略未知字段"规则）。
- 破坏性变更（删字段/改语义）必须 bump major，并同步更新两侧实现 + fixtures。

## 3. Fixtures 约定

- `fixtures/handshake/`：握手 golden（请求行 + 期望应答行），Rust/Haskell 两侧
  测试共同消费——同一份文件，防两侧实现漂移。
- `fixtures/hook-payloads/`：Claude Code `PreToolUse(Edit|Write)` 的**实测** stdin
  dump（官方文档无逐字示例，ADR-007 ⚠️ 项）。采集方式见该目录 README。
- fixture 变更 = 契约变更，走 §2 规则。

## 4. 工具链锁定（M0 验收项）

| 组件 | 锁定 | 载体 |
|---|---|---|
| Rust | 1.94.1 | `cli/rust-toolchain.toml` |
| GHC | 9.14.1（LTS） | CI `ghc-version` + 本文件 |
| 依赖快照 | cabal freeze | `core/cabal.project.freeze`（GHC 就绪后 `cabal freeze` 生成入库） |
| 协议 | 0.1.0 | §1 所列两处常量 |
