# M5-1a 仪器泛化评审（2026-08-11）——Codex gpt-5.6-sol（effort xhigh），独立只读

> 范围 = 切片仪器泛化到外仓的未提交工作树 diff（eval_commits.rs、
> eval_support/{mod,colordiff}.rs）。8 条发现，每条经主线独立核实后处置；
> 修复批全部经字节不变回归 + 红/绿测试复验。

| # | 级别 | 论断 | 核实 | 处置 |
|---|---|---|---|---|
| R1 | high | 浅克隆的 shallow 边界伪装成 root：`--max-parents=0` 返回它、断言通过、截断历史静默重生成冻结自仓切片 | 属实（git 将 shallow 边界视作无父） | 自仓路径（base=None）强制 `rev-parse --is-shallow-repository` == false |
| R2 | high | CE_SLICE_BASE 未验证为 first-parent 窗口边界：merge 第二父是合法 rev 但窗口静默错位；BASE==TIP 静默写空切片 | 属实（主线自行复现：requests 1b40fdd 的 ^2 → 最早窗口 commit 的第一父 ≠ base） | 空窗断言 + 最老窗口 commit 的第一父必须等于 base（邻接断言一并杀死离链/跳段两类）；两条红测试钉住（rc=101 + 预期消息） |
| R3 | high | `git diff -M -C` 在 renameLimit 超限时静默降级为 D+A，仅 stderr 告警、退出码 0，守恒断言照过 | 属实（实证：limit=1 时 3 个改名对全降级 + 告警；limit=0 = 无限，R093 正常检出，git 2.52） | git_run 统一 `-c diff.renameLimit=0` + 成功却有 stderr 一律 panic（GT 生成无良性告警；自仓 61-commit 走查全程零触发） |
| R4 | medium | 端点接受可移动 rev 表达式（HEAD/分支/短 sha），doc 记录原文不可复现 | 属实 | corpus() 经 `rev-parse --verify <rev>^{commit}` 钉全 OID 后才入 doc；空字符串一并拒绝（原守卫仅查存在性） |
| R5 | medium | 一致性门只读自仓 doc，外仓切片无门；自仓 doc 大部分字段不受门保护 | 属实 | 门改为枚举 `contracts/eval/commit-slice*-v1.json` 全部校验（双 doc 在场实测通过）；字段级冻结依赖 git 提交审读 = 既定 F1/D2-7 边界，不另建摘要机制 |
| R6 | medium | merge 聚合 diff 可把不同构成 commit 的删/增行配成"移动"（跨构成巧合），无重复计数但伤 GT 精度 | 属实为风险（无重复计数经 Codex 与主线双方确认） | 行加 `parents` 字段（仅 >1 时写入，自仓字节不变）；1b 审读分层处理 merge 行 |
| R7 | medium | 五语言 scope 非全后缀集（.tsx/.mts/.markdown 不入）且 memory/ 排除是自仓专属 | 属实为设计选择 | 注释明文定性：canonical-extension 基准，排除项对无此路径的外仓惰性，统一 scope 保跨语料可比 |
| R8 | low | `-C` 请求 copy 检测但解析器拒绝 C 状态，外仓窗口遇 copy 即 panic | 属实且有意 | 保持响亮失败，注释升级为"deliberate stop"：真遇到时强制做 copy 语义决策而非静默猜测 |

## 复验总账（修复批收口时）

- 自仓切片重生成**字节不变**（修复批前后各一次 + 收尾一次，共三次
  `git diff` 均空）——泛化 + `--no-merges` 移除（自仓 0 merge 实测）+
  renameLimit=0 对冻结仪器零扰动。
- 红测试：裸 CE_SLICE_REPO / BASE==TIP / 第二父 BASE 三条全部 rc=101
  且消息精确命中。绿测试：requests 100-commit 干跑窗口 47 入册 + 53
  排除、真 merge（2 父）正确落 excluded、merge 标注布线经正反两向验证。
- 全电池 0 FAILED；fmt/clippy 净；scan 3 warn 逐项 = 既存集合；
  棘轮 201/201、67 组持平。三文件 ≤300 行（E01）。

## 评审质量注记

8/8 属实（R7/R8 为"属实的设计选择"，处置为显式定性而非改行为）。R2 与
主线独立复现同一危害（评审流式痕迹与主线 Bash 复现在同一小时内互不知情
地指向 merge 第二父），R3 的实证（limit=1 降级 + limit=0 语义）由主线
scratchpad 小仓补齐——评审给方向、主线给证据的分工成立。
