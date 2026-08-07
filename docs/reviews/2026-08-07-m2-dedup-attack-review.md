# M2 dedup 子系统攻击式审阅与处置（2026-08-07）

- 审阅者：Opus 攻击视角（用户指令替代 Codex）；亮点 = 全部发现**带可执行
  复现**（评审自建 path-dependency 探针 crate + 真 CLI 实验：65 份全同
  文件→0 块、越界 panic 栈、参数换挡陈旧索引、跨语句吞并样本）。
- 范围：`551fa41..ffe4ad0`（dedup 子系统全量）+ 标定文档。
- 处置提交：本文件所在提交。核实纪律：每条 defect 修复前由主线以
  回归测试独立复现（5 个新回归全部先红后绿或构造性验证）。

## 处置表

| # | 级别 | 发现 | 处置 |
|---|---|---|---|
| D1 | defect | extend() 用未校验的 DB token 偏移索引流：refresh↔load 竞窗或分词器变更即 panic（复现：len 55 index 61） | ✅ 双层修复：load_streams 把刚读的字节回灌 refresh_file（内容哈希快路径免费，变更即原子重索引 + 重取 instances）；pairs 层越界防卫 skip + `stale_skipped` 计数入报告；回归测试钉死 |
| D2 | defect | 缓存键缺 winnowing 参数与分词器版本：换挡后未变文件静默沿用旧指纹（复现：p2 下 fps 仍 11 应 37） | ✅ meta 表（kgram/window/tokenizer_rev）入缓存键，失配整库重建；`TOKENIZER_REV` 常量随归一化语义 bump（本轮 rev 2）；param-wipe 回归测试 |
| D3 | defect | LIT 折叠按词法相邻合并，跨语句吞并整条语句（Python 属性 docstring、TS ASI 指令消失且行跨度膨胀） | ✅ 合并加同父节点约束（同一字面量节点的碎片才合并）；Python/TS 双语言回归测试 |
| D4 | defect | HOT_CAP 悬崖：65 份全同文件 → 0 块（重复越多检出越少的非单调性） | ✅ 热组改邻接链配对（n-1 对线性成本，每实例入 ≥1 验证对）；`hot_skipped`→`hot_chained`；70 份合成回归 = 69 块 |
| R5 | risk | 94.7% 精度的抽样不支撑：walk.rs 家族 128/170 只抽 4，n=4 零失败的 95% 上界约 53% 类错误率 | ✅ 标定文档改口径：点估计降级为"初步仲裁"，注明区间与家族抽样局限；终版精度数字挂 M2 收口（扩样 ≥12/52 区域） |
| R6 | risk | 跨/内 locale 的 TP/FP 判定同源同机制、无外部 oracle，翻转即 85.3% 跌破门 | ✅ 文档显式记录该判定为实现者仲裁 + 判据（同 key 序）+ 敏感性（翻转跌破门）；M2 收口交叉仲裁 |
| R7 | risk | 计划 §6 M2 召回门 ≥95% 未达（88.2%/94.1%）而文档未明说；按 CLAUDE.md#2 改口径须先改计划 | ✅ 文档补"数值门未达"直陈；docdup 域排除口径 = **用户决策项**，已在会话汇报中上呈（计划契约不擅改） |
| R8 | risk | dominant 只删严格包含，170 块中 97 对互相重叠，分母通胀 | 🔶 记录保留：极大段语义的固有表现；克隆"组"表示（k 路家族 k-1 块）列入 M2 收口设计项 |
| R9 | risk | candidate_files 不知热组，白白 tokenize 永不贡献的文件 | 🔶 记录保留：热组现已链接（D4）不再"永不贡献"；成本优化挂 ⑧ 性能轮实测后做 |
| R10 | risk | dedup 报告无 golden/e2e/契约测试（scan 有） | 🔶 挂下轮：schema 0.3.0 刚定型，golden + e2e 随 daemon 轮补 |
| R11 | risk | JSON 报告不自述参数，--min-tokens 40 与默认档字节不可分 | ✅ Summary 增 kgram/window/min_tokens（schema 0.3.0） |
| R12 | risk | dedup dogfood 未入 CI；本仓库现有 6 处 cli/src 真重复（如 ast.rs children/named_children 孪生） | 🔶 记录：去冗 dogfood 门随 M3 棘轮入 CI（计划 §7.5 M1 只要求 scan）；6 处真重复列入 M3 首批治理 |
| N13 | nit | is_literal 对复合 kind（composite_literal/func_literal）误真 | ✅ 文档化前置条件（仅叶子 kind；tokenize 先递归） |
| N14 | nit | Rust `'`（lifetime tick）被当字符串定界符 → `&'a str` 签名成假克隆驱动 | ✅ literal_delims 按语言入 LangSpec 表（Rust 仅 `"`）；回归测试 |
| N15/16/17 | nit | 布尔/None 不折叠、TS 仅 number 折叠、blank_identifier→ID 未文档化 | ✅ tokens.rs 头部文档化为 M2 显式立场（行为不动，动 = tokenizer_rev bump + 重标定） |
| N18 | nit | extend_anchor 死守卫 + 双重查找 | ✅ 清除（get_key_value 单查；同 tok 同文件由 cap=0 自然拒绝） |
| N19 | nit | files.token_count 全库无读者 | ✅ 保留并注记：陈旧校验的边界数据源，daemon 轮接线（D1 已由回灌方案根治主路径） |
| N20 | nit | remove_missing 每行一事务；refresh 快路径无 mtime 预滤 | ✅ 单事务合并；mtime 预滤挂 ⑧ 性能轮（预算实测驱动） |
| N21 | nit | 标定文档 10.2 均值不可复现（实为 walk 9.84 / hyperlink 9.0 的抽样均值误作类统计） | ✅ 文档更正为按类精确均值 |
| N22 | nit | 双版本 SCHEMA_VERSION 乒乓重建 | ✅ 已有 pre-release 注记，不动 |

## 审阅"无发现"清单（攻击后存活）

winnow 短序列路径逐点核验（24/25/26/49/50/51）；last_recorded 按索引去重
的单调性证明；同文件 cap 间距不变式；dominant 确定性（三次运行字节一致）
与贪心正确性（包含传递性）；五语法全 kind 枚举证 `ends_with("identifier")`
无误报；注释子树无非注释子节点；路径归一化一致；TOCTOU 无损坏；E01 全
合规（最大文件 pairs.rs 194 行、最大函数 30 行）；周期三副本 = 2 偏移类
run 的数学正确性（k 路家族 k-1 块信息完备，已注记给 M3 消费者）。
