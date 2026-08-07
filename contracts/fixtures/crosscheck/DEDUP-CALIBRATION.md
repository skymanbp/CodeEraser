# M2 dedup 对拍标定记录（2026-08-07，第一轮探索）

> 对照物：jscpd 5.0.14（npx，--min-tokens 50，json reporter）跑钉定
> crosscheck fixtures（SOURCES.md 同一批）。ce 侧：`ce dedup` 默认参数
> （k=25 / window=26 / t=50）。本文档记录标定过程与设计决策，终版
> 召回/精度数字待第二轮（扩展验证重构后）复测出具。

## jscpd 可检出集

17 条克隆 / 142 重复行（go 1、python 2、rust 14；TS locale 文件因
字符串内容不同 jscpd 不可见——见"价值差异"）。

## ce 第一轮原始成绩

- **召回 15/17**。两条 miss 均已根因定位（非指纹层缺陷，指纹锚点存在）：
  1. `models.py 837-847 <-> 847-857`：同文件**边界相接**的两段重复被
     pair 层的行级自重叠过滤错杀（`b_start <= a_end` 把相接当重叠）。
  2. `walk.rs 2364-2377 <-> 2380-2391`（公共段手数 ≈124 归一化 token，
     远超 t）：测试样板前缀在文件内十余处重复，hash 组实例密集，
     扁平排序贪心合并把区域配对**碎片化**（证据：`2314-2316 <->
     2385-2388` 等 2fp 碎片 + 病理块 `2345-2376 <-> 2726-2729`
     116fp——链式误并把 31 行 A 侧接到 4 行 B 侧）。
- **精度未标定**：195 块中 108 块单指纹（仅保证 ≥k=25 token 公共段，
  低于宣称的 min-tokens 50 检出意图），须先重构再测。

## 价值差异（ce 检出、jscpd 原理性不可见）

zod locale 文件互为整文件 T2 克隆（字符串按语言不同、结构全同）：
`ota.ts 18-76 <-> ru.ts 74-131` 共 455 指纹。ce 的 LIT 折叠归一化
正是为此设计；jscpd 逐 token 比对字符串内容故不可见。此类块在精度
评测中按人工仲裁计真阳性，不按 jscpd 集合计假阳性。

## 第二轮设计决策（SIGMOD'03 经典两段式）

winnowing 只做**候选锚点选择**；配对层重构为**token 流扩展验证**：

1. fingerprints 表加 token 起始索引列（现仅存行号，无法精确扩展）；
2. 每个共享 hash 锚点对，在两侧 token-hash 流上做双向精确最长公共
   延伸，得到极大公共段；同一极大段从多个锚点扩展到相同 span，去重；
3. 仅报告延伸长度 ≥ t=50 token 的段（报告项从"启发式合并块"变为
   "已验证精确匹配段"）——同时修复两条 miss（相接判定改 token 偏移、
   碎片化由极大段替代）、消灭单指纹噪声、使精度可按构造验证。

## 第二轮结果（扩展验证重构后，2026-08-07）

pair 层已重构为锚点-扩展-验证（fingerprints 表加 start_tok 列，schema
user_version=2 门控重建；报告项 = 已验证精确归一化公共段 ≥ 阈值；
同文件对按包含支配合并平移变体：321 → **170 块**，全部 ≥50 token）。

- **召回 15/17（默认 t=50）**，两条 miss 归因关闭（性质与第一轮不同，
  第一轮的 pair 层缺陷已被重构消灭，2364 条已命中）：
  1. `models.py __bool__ <-> __nonzero__`：函数体 = 9 行 docstring +
     `return self.ok`，LIT 折叠后每函数仅 ≈12 token < k=25，**锚点
     原理性不存在**（任何报告阈值都不可见）。这是 T2 归一化的刻意
     语义：docstring 重复属 `docdup` 域（计划 §4.1，M5），不是代码
     克隆热路径的目标物。M2 收口评审时按此口径仲裁。
  2. `walk.rs 1224 <-> 1249`：首 token `mut` 断链后极大公共段 47
     token，差 3 到线。`--min-tokens 40` 实测召回 **16/17**——能力
     在，默认档更严；根因 = jscpd 的 min-tokens 50 用其细粒度 token
     度量，ce 的 50 用折叠后 token（同数字不同测度）。
- **精度**：170 块全部为扩展验证过的精确归一化公共段（T2 定义下
  构造性真阳）；风险类 = 纯字面量结构退化匹配（LIT 折叠使不同字面量
  列表同构），人工仲裁抽样（N=20 + 风险类定向）下轮出数。

## 复现

- jscpd：`npx jscpd --min-tokens 50 --reporters json --output <tmp>
  --format "python,typescript,go,rust" contracts/fixtures/crosscheck/`
- ce：`ce dedup contracts/fixtures/crosscheck --format json --db <tmp>/ce.db`
