# R8/A9f 批次对抗审阅（2026-08-07，多代理工作流）

- 范围：ec2d007..f8ab86e（R8 克隆组 / A9f 降级可见性 / hookio / parse harness 四批）
- 方法：4 维度审阅者（算法/fail-open/契约/测试保真）各自读实码产出发现，
  每条经 2 名独立反驳者复读现行代码试图证伪，双票存活才确认。
  Codex 本机不可用，按既定指令以此替代独立评审。46 代理，0 错误。
- 结果：**17 确认（6 defect / 8 risk / 3 nit），4 证伪**。处置全部落在
  同日修复批（含 e2e 回归），棘轮 202→201。

## 确认项与处置

| # | 级别/维度 | 发现 | 处置 |
|---|---|---|---|
| 1 | defect/algo | 同文件重叠跨度链式合并让"枢纽文件"桥接无关克隆家族（x.rs/y.rs 零共享 token 却入同组；自仓 155 块巨型组即此产物），反驳者双双以 e2e 实跑证实 | ✅ 根修：删除重叠合并，仅完全相同跨度记同一出现点（groups.rs 重写）；`hub_file_does_not_bridge_families` 回归 |
| 2 | defect/algo | 同文件相邻对（a_end==b_start）塌缩为单成员"家族"，组视图否认块视图报告的重复（自仓实存 5 个单成员组） | ✅ 同根修；`adjacent_self_pair_keeps_two_members` 回归 |
| 3 | defect/failopen | doctor 硬编码 `"."` 且无 root 参数：在子目录/CI（working-dir=cli）报告错误项目的状态 | ✅ Doctor 增 root 位置参数（与 scan/dedup 对齐）；CI 传 `..` |
| 4 | risk/failopen | doctor 的暖机 ping 惰启 30 分钟 daemon（每调用目录一个；锁 exe、CI runner 遗留进程——也解释了本机当日链接器锁死） | ✅ 新 `client::request_if_running`（只连不拉起）；doctor e2e 连跑两次证明无侧效 |
| 5 | risk/failopen | 降级计数在只增不删的 feed 上永不归零，单独一个数误导 | ✅ 改报 `N of M entries` 终身口径；窗口语义挂 M4 feed 正式化 |
| 6 | risk/failopen | precommit 观察条目伪装成 `stop_audit` 事件 | ✅ gather 带事件名：precommit/stop_audit 分明 |
| 7 | nit/failopen | precommit 降级路径丢掉健康路径必给的 staged/净 LOC 摘要 | ✅ 降级行保留摘要 |
| 8 | defect/contract | 同 #1（契约面：0.5.0 载荷违反 Group 文档语义） | ✅ 同 #1 |
| 9 | defect/contract | 同 #2（wire 序列化出单成员组） | ✅ 同 #2 |
| 10 | risk/contract | observe feed 无 schema id / golden / 统一事件甄别字段 | 🔶 部分：全条目已带 `event` + 测试断言 ts_ms/event；每行 schema id 与 feed golden 挂 M4 正式化 |
| 11 | risk/contract | doctor 的 daemon 侧效未反映在帮助文本/CI 用法中 | ✅ 侧效已除；帮助文本改述 |
| 12 | defect/tests | R8 核心修复路径（重叠合并）零测试覆盖 | ✅ 该路径已删；两条回归钉住新语义 |
| 13 | defect/tests | doctor 新增面零覆盖，CI 结构性读 0 | ✅ `doctor_reports_project_without_spawning_daemon` e2e |
| 14 | risk/tests | observe feed 无 golden，ts_ms 无断言 | 🔶 部分：ts_ms/event 已断言；feed golden 挂 M4 |
| 15 | risk/tests | guard 侧 degraded=true 从未被测试 | ✅ `probe_failure_is_stamped_degraded`（daemon 下索引损坏） |
| 16 | risk/tests | 共享 fail-open stdin 进气无畸形输入测试 | ✅ `malformed_envelope_fails_open_everywhere`（三 hook） |
| 17 | nit/tests | g.tokens 断言无法区分 max 与全局值 | ✅ hub 回归断言两组 tokens 互异 |

## 证伪项（不处置）

- min_distinct retain 后建组会拆散热链家族——反驳：retain 前后集合关系不支持该场景。
- observe_append 对非对象 Value panic——反驳：所有调用点均传 json! 对象字面量。
- precommit 条目标签问题的另一表述（与 #6 重复上报，已随 #6 修复）。
- 0.5.0 后无传输层 schema id 钉定——反驳：daemon_e2e 改引常量属蓄意（形状钉定职责在 golden）。

## 语义变更记录

组语义自本批起：**成员 = 被验证块连接的精确端点跨度**；重叠但不相同的
跨度保持独立成员。自仓 201 块从假融合的 16 组变为真实的 67 组——组数
上升是诚实化，不是回归。
