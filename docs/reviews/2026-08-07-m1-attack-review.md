# M1 代码攻击式审阅与处置（2026-08-07）

- 审阅者：Opus 攻击视角（用户指令：Codex 额度不足，以 Opus 替代）
- 范围：`19a07a5..be6e6ac`（RCA harness、ce.toml dogfood 排除、白皮书
  golden / labeled-jump / coc_operators 分表 / naming / schema 契约）
- 处置提交：本文件所在提交（修复批与本记录同批入库）
- 复核纪律：每条 defect 级声称均由主线独立核实（gocyclo v0.6.0 源码
  原文、fixture 行内容、`default:` 计数 grep、let_chain AST 探针）后才动代码。

## 处置表

| # | 级别 | 发现 | 处置 |
|---|---|---|---|
| 1 | defect | Go `default_case` 计入 CC，与 gocyclo v0.6.0（"ignore default case"）和白皮书 p.5 margin 双双相悖；fixtures 零 `default:`，52/52 对此轴沉默 | ✅ 修复：spec.rs 移除 `default_case`（含出处注释）；getWords golden 补 CC=4 断言；DIVERGENCES 缺口 #3 |
| 2 | defect | harness 按 (start,end) 单键 join，同行嵌套闭包同 span 对称覆盖，双侧各静默丢 1 单位（walk.rs:1529/2718 实存）；"320/320" 实为幸存映射数 | ✅ 修复：FnMap 改多重映射 + 同 span 排序多重集合比较；总数断言钉 322；复跑 322/322 全对 |
| 3 | defect | Rust let-chain 的 `&&` 为匿名 token（无 operator 字段），CC/CoC 双漏计，且 ce 自身代码正用此惯用法，fixtures/dogfood 均不可见 | ✅ 修复：新增 `chain_kinds` 表驱动机制（CC +N-1，CoC +1 run）；AST 探针核实结构；钉定测试 cc=4/coc=3 |
| 4 | risk | TS/Rust 的 coc_nest_only 死表项（fn_kinds 早退遮蔽，永不触发）——正是 fuck-u-code 死字段形态 | ✅ 清除：TS/Rust 置空并注释为何为空；活项仅 Go func_literal / Python lambda |
| 5 | risk | 括号同算子子链 `x && (a && b)` 计 2 run 未钉定 | ✅ 钉定为正确行为：白皮书 p.8 `if (a && !(b && c))`=3 原例入 golden（括号/取反开启新序列） |
| 6 | risk | `CE_BLESS` 任意值（含空/0）即静默 bless-and-pass | ✅ 修复：仅接受 `CE_BLESS=1` |
| 7 | risk | naming 误报：godoc 强制下划线族（Example_x/TestT_M）、TS 引号/计算键泄漏进 subject | ✅ 修复：Go Test/Benchmark/Example/Fuzz 前缀放行；`"` `'` `[` 哨兵跳过；Python unittest camelCase 保持警告（warn 级，PEP 8 祖父条款记录于此，不改） |
| 8 | risk | Python for/while/try 的 `else` 计 flat +1 无出处无测试 | ✅ 立场钉定：divergence_stances 新测试（白皮书对此沉默，ce 读作阅读流分支，如实记录） |
| 9 | risk | ce.toml 排除过宽：`contracts/fixtures/**` 把 ce 自著 dump.py/文档一并豁免，且理由陈述不实 | ✅ 收窄至 `contracts/fixtures/crosscheck/**`；dogfood 34 文件 0 fail |
| 10 | nit | `is_attributed` 用 ends_with，扁平文件名后缀碰撞可误豁免 | ✅ 改精确路径相等 |
| 11 | nit | harness 无总数下限、陈旧文件防护吞错、`.round() as u32` 静默饱和 | ✅ 总数 `assert_eq!(322)`；删除后 `assert!(!exists)`；own≥0.5 断言（CI 不跑 `#[ignore]` 维持现状：RCA 为本机对照工具，CI 无此依赖——审阅注记接受） |
| 12 | nit | walk.rs 用户 glob 带 `!` 双反转成静默空操作；全 `!` 构造依赖未防护 | ✅ 用户 glob 带 `!` 即报错拒绝（BUILTIN 全 `!` 为内部不变式，注释已述） |
| 13 | nit | 白皮书无钉定副本/哈希，违反自家"防作弊"标准 | ✅ SOURCES.md 记 URL + v1.7 (2023-08-29) + SHA-256（版权原因不入库，凭 hash 复验） |

## 审阅明确"无发现"项（攻击后存活）

labeled-jump 机制（含 Rust `break 'l value`、Go switch-break 惯用法、
labeled_statement 非跳转 kind 不误计）；`??` 分表后的 run 语义（含
`(a||b) ?? c` 等 6 组构造推演）；白皮书 5 例题判分独立重推导全部复现
（p9 lambda 例仅 Go 可忠实移植——TS/Rust 闭包为独立单位，单位拆分
模型差异已在 divergence_stances 以 Python 装饰器钉定，同型）。

## 遗留（显式不修，有出处）

- 递归 +1（规范 §1）：需调用图，M5 实现；cognitive.rs 头部注记。
- Python unittest 的 camelCase 覆写方法会收 fn-naming warn：PEP 8
  祖父条款场景，warn 级可接受，M4 判决层再论豁免。
