# contracts/DAEMON.md — ce ↔ ce daemon 协议契约册

> M5-close 审计 R2/LOW 的偿清：daemon 协议 1.0.0 此前无契约册、无
> golden——M6 GUI 是第二个客户端，无册的协议只能靠读源码对齐。本册 +
> `contracts/fixtures/daemon/` 形状 golden（`cli/tests/it/daemon_proto.rs`
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
- 版本常量：`cli/src/daemon/proto.rs::DAEMON_PROTO`（当前 **<!--ce:ver:daemon#v-->2.0.0<!--/ce-->**：
  `hello_ok` 砍去无读者的 `version` 字段〔I 轮 D8，2026-08-24，用户拍板
  「现在就删」；删字段 = major，1.x 客户端得 `restart` 后自 respawn〕；1.1.0 =
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
  `proto` 旧于自身（同 major、低 minor；判据 = `client.rs::stale` 的 major.minor 比较，
  `refuse_stale` 只对同 major 者代发 shutdown。daemon 2.0.0 起 1.x 旧进程走 major 不符的
  重启路而非本条；2.0.0 为 2.x 首版，本条今日无实例、是前瞻契约）即判 stale：`request` 令其
  shutdown 后 respawn 一轮；`request_if_running` 只放行 Shutdown
  （eject 仍可令其退役），其余请求报错拒信。
- **空闲退出**：30 分钟无活动（`CE_DAEMON_IDLE_SECS` 仅测试可调）；
  watchdog 读 lock-free 时间戳，卡住的请求挡不住它。
- **工作目录**：daemon 启动即离开项目根（`set_current_dir` 到系统临时目录，
  serving 行写明 cwd）——进程 cwd 在 Windows 上钉住目录，空闲中或退出中的
  daemon 会让根目录删不掉（demo 回放在并行测试套下撞到：`bye` 先于退出完成，
  驱动随即 rm 即 EBUSY）；根以绝对路径入参、子进程各持 `-C root` 或纯管道，
  相对且含路径分隔符的 `CE_CORE_BIN` 启动时绝对化，裸名仍走 PATH。
- **超时与上限（本节 2026-08-25 补齐——此前只有字节上限入册、时间上限
  全在源码里，第二个客户端无从得知）**：
  - **每连接静默期限**：hello 前 **5 s**、hello 后 **60 s**
    （`server/idle.rs`）。与 §1 的 4 KiB 未认证行长上限是同一道门的
    两个量纲：字节管一行多长，秒管一行多久不来。
  - **daemon → ce-core 应答期限**：默认 **60 s**，可由
    `CE_CORE_DEADLINE_SECS` 覆盖；超时**杀掉**该 core 进程而非等它
    （`corelink/pipe.rs`）。core 单行应答上限 **64 MiB**，超限按同一条
    降级路处理——这是**客户端自己的帧上限**，与核各族的 `*_too_large`
    是两回事：后者是核判自己算不动，前者是客户端拒绝读下去。
  - **core 重启预算**：连续 **3** 次开链/请求失败后，daemon 在**其整个
    生命周期内**不再重试，全程 L1（`daemon/judge.rs`）。修好 ce-core
    也不会被这个 daemon 捡起来——要么等它空闲退出，要么 shutdown。
    该状态经报文的 `degraded` 字段可见；那条 stderr 提示在懒启动的
    daemon 上写进空句柄，不是可依赖的通道。
  - **客户端 → daemon 整场对话期限**：默认 **75 s**（daemon 自己的
    60 s core 期限加余量——最慢的合法应答是等核的 four_class 批，客户端
    期限短于服务端会把健康的慢判决误读成挂死），`CE_CLIENT_DEADLINE_SECS`
    覆盖。K 步 10 兑现审计 #85 的「1.0 后再议」：此前三边只有这一边
    没期限，卡死的 daemon 把 PreToolUse 钩子无限期挂住。实现是工人线程
    持连接、主线程限时收（interprocess <!--ce:tool:interprocess#v-->2.4.3<!--/ce--> 对命名管道无读超时 API，
    仅有弃用的 PIPE_NOWAIT 轮询）。超时后主线程**拆除**工人（L 轮步 #14
    O64，`daemon/cancel.rs`）：Unix 经复制的描述符 `shutdown` 套接字，工人
    的读返回 0；Windows 对管道句柄 `CancelIoEx`（interprocess 以
    `ReadFileEx` + 可唤醒等待读管道，故 `CancelSynchronousIo` 够不着），
    因其只取消在途 I/O 而每 20 ms 重发；先置标志再取消、工人先登记流再
    读标志，重连竞态由此闭合。宽限 = 工人自己的重试预算 20 × 100 ms +
    500 ms = **2.5 s**，其内返回的应答作废；唯有内核仍持着的 `connect`
    无物可取消——宽限过后按名**脱离**（错误文本带阶段），计入 doctor
    文档 `daemon.parkedWorkers`（`ce.doctor-report/0.3.0` 加性键，健康
    进程恒 0，控制台与 GUI 仅非零时渲染）。此前「GUI 的 doctor 探针每次
    撞上卡死 daemon 至多滞留一条停读线程」的明记至此撤销。懒启动另有
    **20 × 100 ms = 2 s** 的上线上限。
- **shutdown** → `bye` 后**即刻**退出——`bye` ⇒ 进程随即放手是承重
  契约：eject 的有界拆除窗口与 stale/skew respawn 链都建立在它上面。
  在途冷启首建被 shutdown **有意斩断**（join-on-exit 变体经三组 A/B
  eject 复现否证后撤除，2026-08-26）；被斩（含 kill/断电）的窗口由库
  内 `resolve_pending` 债务行兜底——掉边与记债同事务提交（index
  schema v13），任一后续 run 的 sweep 或 phase-1.5 结清（v1.2.0 发版
  夜 CI 32964681934 的永久缺边即无账本时的此调度）。
- 冷启动索引在 bind **之后**起线程（抢 bind 失败的进程不建库）；
  写路径遵循 ADR-003 v1.7 收敛式多写者契约——daemon 是收敛写者
  **之一**（自身内部按**请求**串行：连接是并发线程，dispatch 过
  judge 互斥锁），不是「唯一写者」。

## 3. 消息表（形状以 golden 为准，此表是语义）

| Request | 语义 | 正常应答 |
|---|---|---|
| `hello{proto,token}` | 凭证 + 版本协商，连接首条（token 缺省=空，必被拒） | `hello_ok{proto}` / `error{unauthorized}` / `restart{reason}` |
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
  `cargo test --test it daemon_proto::`（CE_BLESS=1 蓄意重生成）守护。
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
cd cli && cargo test --test it daemon_proto::        # 形状漂移门
cd cli && cargo test --test it daemon_e2e::          # 生命周期+凭证门+界读（daemon_auth 已并入）
cd cli && cargo test --release --lib -- daemon::     # 凭证落盘/staleness/常数时间 单元层
cd cli && cargo test --test it concurrent_writers::  # v1.7 收敛契约
cd cli && cargo test --test corelink_deadline        # §2 core 应答期限：不应答的核被收割（独占二进制：改进程 env）
cd cli && cargo test --test daemon_conn_deadline     # §2 每连接静默期限 + 连接槽上限（独占二进制：64 静默 peer 须同窗）
```
