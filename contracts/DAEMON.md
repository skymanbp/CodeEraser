# contracts/DAEMON.md — ce ↔ ce daemon 协议契约册

> M5-close 审计 R2/LOW 的偿清：daemon 协议 1.0.0 此前无契约册、无
> golden——M6 GUI 是第二个客户端，无册的协议只能靠读源码对齐。本册 +
> `contracts/fixtures/daemon/` 形状 golden（`cli/tests/daemon_proto.rs`
> 重放）即漂移门：serde 改名/换 tag/字段重排在第二客户端失步**之前**变红。

## 1. 信封

- 传输：本地 socket（Windows named pipe / Unix domain socket，
  `interprocess` GenericNamespaced）。socket 名 =
  `ce-daemon-<fnv1a(canonicalized root) 十六进制 16 位>`
  （`cli/src/daemon/proto.rs::socket_name`）——每项目根一个 daemon，
  凭据即本地用户（ADR-003）。
- 消息：NDJSON，一行一条；serde 外部 tag `type`，snake_case
  （`{"type":"hello","proto":"1.0.0"}`）。
- 每连接可承载多条请求（server.rs `handle` 循环）；解析失败回
  `error` 行并**继续**本连接，绝不崩连接。
- 版本常量：`cli/src/daemon/proto.rs::DAEMON_PROTO`（当前 **1.0.0**），
  与 ce↔ce-core 的 handshake proto（VERSIONING.md §1）**相互独立**。

## 2. 生命周期

- **懒启动**：客户端 `client::request` 连接失败时从自身二进制 respawn；
  测试必须走 `request_if_running`（永不 spawn——嵌套测试二进制风险，
  common/mod.rs 注记）。
- **协商**：每连接首条应为 `hello{proto}`。major 相符 →
  `hello_ok{proto,version}`；major 不符 → `restart{reason}` 且 daemon
  **退出**——客户端从自己（更新的）二进制重启一个。
- **空闲退出**：30 分钟无活动（`CE_DAEMON_IDLE_SECS` 仅测试可调）；
  watchdog 读 lock-free 时间戳，卡住的请求挡不住它。
- **shutdown** → `bye` 后退出。
- 冷启动索引在 bind **之后**起线程（抢 bind 失败的进程不建库）；
  写路径遵循 ADR-003 v1.7 收敛式多写者契约——daemon 是收敛写者
  **之一**（自身内部按连接串行），不是「唯一写者」。

## 3. 消息表（形状以 golden 为准，此表是语义）

| Request | 语义 | 正常应答 |
|---|---|---|
| `hello{proto}` | 版本协商，连接首条 | `hello_ok{proto,version}` / `restart{reason}` |
| `ping` | 活性 | `pong{uptime_ms}` |
| `dedup{min_tokens?,min_distinct?}` | 对 daemon 根跑 dedup 管线 | `dedup_report{report}` |
| `probe{file_path,content}` | M3 廉价门：即将写入内容的 T1/T2 证实匹配（排除自身路径，零刷新） | `probe_report{matches,elapsed_ms}` |
| `four_class{pairs}` | M4 判决：(before,after) 路径对走 daemon 持有的 ce-core link；只有**路径**过 socket，内容 daemon 侧读（ADR-002） | `four_class_report{report}` |
| `shutdown` | 退出 | `bye` |

- 任何请求的失败面 → `error{message}`；降级信息在 report **内部**
  显式携带，绝不静默（A9f）。
- 未知/坏行 → `error{message:"bad request: …"}`，连接存续。

## 4. 版本纪律

- SemVer：形状破坏（改 tag、删字段、字段改义）= major——触发
  restart-respawn 链路，故 major 就是「强制客户端与 daemon 同代」。
  加性可选字段 = minor（旧客户端缺省解析）。
- 形状唯一权威 = `proto.rs` 两枚 enum；本册**不抄写字段清单**
  （单源纪律，M5-close D7 先例）——golden 冻结每个变体的规范字节，
  `cargo test --test daemon_proto`（CE_BLESS=1 蓄意重生成）守护。

## 5. 复跑

```
cd cli && cargo test --test daemon_proto        # 形状漂移门
cd cli && cargo test --test daemon_e2e          # 活体生命周期
cd cli && cargo test --test concurrent_writers  # v1.7 收敛契约
```
