# Codex 评审记录 — M5-1c-ii 影子消融仪器（2026-08-11）

评审对象：commit 371e9a2（消融仪器 + 双语料矩阵）。评审通道
codex:codex-rescue（gpt-5.6-sol，xhigh，READ-ONLY）。八项发现逐条独立核
实后处置如下；修复落在本记录同一提交。

| # | 级别 | 发现 | 核实 | 处置 |
|---|---|---|---|---|
| F1 | blocker | moved-map 等价证不到站点**分解**——变体过滤的是 block，同一 mark 并集可来自不同 block 集 | 属实（断言只比 delta map） | **用真核作 block 级 oracle**：重放同一 wire 请求，影子站点集必须与核 reply blocks 逐字全等（比 Codex 建议的合成测试更强）；双语料重生成全过 |
| F2 | blocker | CI 门可被"同时改 rows+summary"的伪造通过（全零省行 + 标量自洽） | 属实 | 门加固：row sha 须为切片唯一成员；**每变体 hits+misses == labels 锚定的 GT 总量守恒**（删非零行即破外锚）；quality 判决列 pin（misses/id_misses/invention 全零，双语料） |
| F3 | major | "质量地板"实为**单行锚**规则，非聚合地板 | 属实且关键 | 命名精确化（method + EVAL-SET）；并入册反向论据：聚合-20 分离**不了** requests 发明站（7+16=23≥20）——单行锚形式才是有效形式 |
| F4 | major | chain 是 starts-only 宽松形式，端点/重叠未查 | 属实 | 如实标注"最宽松非交叉形式"；其在最宽松形式下已破自仓召回 8 行（真实交叉搬迁），端点感知成员未测但同族更严 |
| F5 | major | "flow 全程无操作"未证——输出相等 ≠ 未开火 | 属实，**且数据证明原论断错误**：flow 自仓 drop 41 块（多对一目的地竞争真实存在），输出被 phase3 兜底吸收 | 撤回原措辞；drops 计数入册（第 8 槽，可守恒复推导）；结论改为"flow 开火但零输出差 = 竞争在成本模型下不产生错误" |
| F6 | major | 阈值窗口 "(16,19]" 表述含糊/无 sweep 支撑 | 属实 | 精确化：**发明站死∧实测最险真站(19 锚)活的整数阈值 = 17..19**；t=20 双语料亦过门（19 锚站转靠冗余）；>20 未扫描，最弱非冗余幸存锚未测定 |
| F7 | minor | freq 是硬唯一门，非"频率加权"族 | 属实 | 命名 base-tree-uniqueness gate；结论收窄：硬门失效 + 方向倒置数据（巧合行 freq=1）否定"越稀有越强"的分离方向，连续加权族未逐一测 |
| F8 | minor | hits/misses 是计数级 | 属实（id_misses 列已并行承载行身份缺口） | 更名 count_hits/count_misses（method + Row 注释） |

无发现项（Codex 复核通过）：base_freq 树扫描范围与 COMMIT_SCOPE 一致；
width_ledger 差集与 in_gt 归属正确（false=GT 外，非独立证伪）；alnum 为
Unicode 感知；fnv1a 碰撞响亮拒绝。

评审结论边界（如实记录）：quality 的"唯一赢家"地位成立于**这六个精确谓
词、双冻结语料之内**；推广到"quality 族优于 freq/chain/flow 族"需以 F3
的反向论据（聚合形式无效）与 F5/F7 的机制数据（竞争无错误、稀有度倒置）
为限定语引用。阈值取值（17..19 vs 20）为用户决策点。
