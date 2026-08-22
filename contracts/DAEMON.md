# contracts/DAEMON.md — ce ↔ ce daemon 协议契约册

> M5-close 审计 R2/LOW 的偿清：daemon 协议 1.0.0 此前无契约册、无
> golden——M6 GUI 是第二个客户端，无册的协议只能靠读源码对齐。本册 +
> `contracts/fixtures/daemon/` 形状 golden（`cli/tests/daemon_proto.rs`
> 重放）即漂移门：serde 改名/换 tag/字段重排在第二客户端失步**之前**变红。

## 1. 信封

- 传输：本地 socket（Windows named pipe / Unix domain socket，
  `interprocess` GenericNamespaced）。socket 名 =
  `ce-daemon-<fnv1a(canonicalized root) 十六进制 16 位>`
  （`cli/src/daemon/proto.rs::socket_name`）——每项目根一个 daemon。
- 凭据（1.1.0，`daemon/auth.rs`）：socket 名可推导，连接本身不设防；
  真正的门是 `hello.token` 必须等于 `<root>/.ce/daemon.token`（每次
  serve 在 bind 后新铸，**先删后独占创建**——绝不写穿符号链接、绝不
  沿用旧文件的宽模式；Unix 0600，Windows 继承项目目录 ACL）。能力
  边界 = 能读项目目录——daemon 每条应答都派生自项目自身内容，文件
  系统放行的人在这里学不到新东西，拒绝的人连 probe/shutdown 都到
  不了。未认证行 → `error{unauthorized}` 且**连接**关闭，daemon 存续；
  token 比较为常数时间（2026-08-20 加固批）。
- 消息：NDJSON，一行一条；serde 外部 tag `type`，snake_case
  （`{"type":"hello","proto":"1.1.0","token":"…"}`）。**未认证行长
  上限 4 KiB**（超限 → `error` 并关连接，超出部分不缓冲不解析）；
  认证后不限长（probe/four_class 合法携带整文件内容）。
- 每连接可承载多条请求（server/conn.rs `handle` 循环）；解析失败回
  `error` 行并**继续**本连接，绝不崩连接。**连接并发、请求串行**：
  每连接一线程（静默连接只占住自己，卡不住 accept 循环；线程上限
  64），dispatch 经 judge 互斥锁逐条执行——ADR-003 的一次一请求
  纪律不变。
- 版本常量：`cli/src/daemon/proto.rs::DAEMON_PROTO`（当前 **1.1.0**：
  加性 `hello.token`，1.0.0 行仍可解析、得 unauthorized 拒绝），与
  ce↔ce-core 的 handshake proto（VERSIONING.md §1）**相互独立**。

## 2. 生命周期

- **懒启动**：客户端 `client::request` 连接失败时从自身二进制 respawn；
  测试必须走 `request_if_running`（永不 spawn——嵌套测试二进制风险，
  common/daemon.rs::spawn_daemon_ready 注记）。
- **协商**：每连接首条必须为 `hello{proto,token}`，token **先于**
  major 校验——否则一条无凭证的假 skew 就能随意退出 daemon（未认证
  击杀）。token 不符 → `error{unauthorized}` 且连接关闭；major 不符 →
  `restart{reason}` 且 daemon **退出**——客户端从自己（更新的）二进制
  重启一个。客户端在 unauthorized 上重读 token 文件再试**一轮**
  （刚 spawn 的 daemon 在 bind 后毫秒级才落盘新 token）；
  `request_if_running`（doctor/eject/audit）同享这一轮重试——eject
  曾撞上该窗口后把 .ce 从活 daemon 脚下删走。
- **回执校验（2026-08-20 #1）**：客户端**不盲信 `hello_ok`**——回执
  `proto` 旧于自身（同 major、低 minor，如仍在跑的 1.0.x 旧进程：
  它无视 token 字段、不查任何凭证）即判 stale：`request` 令其
  shutdown 后 respawn 一轮；`request_if_running` 只放行 Shutdown
  （eject 仍可令其退役），其余请求报错拒信。
- **空闲退出**：30 分钟无活动（`CE_DAEMON_IDLE_SECS` 仅测试可调）；
  watchdog 读 lock-free 时间戳，卡住的请求挡不住它。
- **shutdown** → `bye` 后退出。
- 冷启动索引在 bind **之后**起线程（抢 bind 失败的进程不建库）；
  写路径遵循 ADR-003 v1.7 收敛式多写者契约——daemon 是收敛写者
  **之一**（自身内部按**请求**串行：连接是并发线程，dispatch 过
  judge 互斥锁），不是「唯一写者」。

## 3. 消息表（形状以 golden 为准，此表是语义）

| Request | 语义 | 正常应答 |
|---|---|---|
| `hello{proto,token}` | 凭证 + 版本协商，连接首条（token 缺省=空，必被拒） | `hello_ok{proto,version}` / `error{unauthorized}` / `restart{reason}` |
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
- **覆盖边界（清零批审查注记）**：golden 冻结的是 enum 变体的**信封
  形状**；`dedup_report.report`、`probe_report.matches`、
  `four_class_report.report` 三个嵌套载荷是 `serde_json::Value` 直通，
  其内部键**不在**本门覆盖内（fixture 里的载荷是示意占位）。各自的
  权威与钉点：dedup report = `dedup::report_json`（report_schema
  golden `fixtures/dedup-report/` + daemon_e2e 消费）、fourclass
  report = `fourclass::session` 形状（daemon_e2e 断言 + wire_indices
  索引钉）、probe matches = probe.rs 报告形（guard 电池）。

## 5. 复跑

```
cd cli && cargo test --test daemon_proto        # 形状漂移门
cd cli && cargo test --test daemon_e2e          # 生命周期+凭证门+界读（daemon_auth 已并入）
cd cli && cargo test --release --lib -- daemon::  # 凭证落盘/staleness/常数时间 单元层
cd cli && cargo test --test concurrent_writers  # v1.7 收敛契约
```
