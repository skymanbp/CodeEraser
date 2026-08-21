# structure/1 七轴语义（S0–S6）

> 蒸馏自 M6 结构管理器设计册（原 docs/reviews/，2026-08-20 回填后
> 清理；全文在 git 历史）。判决全在 Haskell `CE.Structure.*`；这里
> 只记语义契约——每轴一具名谓词一旋钮，逐目录可下钻，CLI 报告与
> GUI 首屏共用一份 JSON 数据面（`ce.structure-report`）。

## 轴表

| 轴 | 信号 | 事实源（测量侧，Rust） |
|---|---|---|
| S0 路径几何 | 深度超顶（depth > 8）、单目录扇出超顶（subdirs+files > 30）——两谓词两旋钮，绝对上限非分布判定 | walk 树形聚合 |
| S1 命名一致性 | 每兄弟集的命名模式熵（case/分隔符/前缀族）——**名不过线**，只过模式码分布 | walk 文件名 |
| S2 正交性/模块度 | 目录内 vs 目录间引用密度（fileRefs 触点为单一事实基） | graph 边表 |
| S3 漂移错位 | 持有错位文件的**目录**计数（修正案①）；文件级谓词不变：outside ≥ min 且 > 2×inside | graph 边表 |
| S4 文档基建 | 文件数 ≥ bigDirFloor（默认 8，旋钮 6）的目录须有 README；根目录须有可识别配置；约定位掩码（1=README，2=config） | walk + entry_globs 同款约定 |
| S5 文档新鲜度 | md 节引用目标在文档最后一改之后的变更次数 | churn 窗口 + md 阶梯 |
| S6 冗余/孤儿卷积 | dedup 块数、deadcode 判决按目录卷积（--deep 才上线） | dedup/deadcode 判决 |

- 评分（2.26.0 密度律）：`charge_i = floor(scale·v_i/(v_i+N))`（N=目录总数），`score = kScale − Σ(charge×violCost)/(neutral×judgedAxisCount)`；轴行载费额（‰）
  （`structViolCost=10`、`structScale=1000`；旋钮回执逐行 pin，
  漂移即错——「一个数字两个主人」拒绝）。
- 熵实现：Shannon/KL 需对数=无理数不可精确判定，取有理判定式；
  观测质量落在参照零质量 bin 上 = Nothing，由 S3 指名偏差行承接，
  绝不塌成 0。
- 语言口径：只有**判决语言集**入树（plan v2.5：尺寸门语言臂不入
  structure——否则 S2 会把正常前端目录判混语）。

## 修正案①（用户拍板 2026-08-19，v0.5.0 起生效）

S3 罚分单位由错位**文件**数改为持有错位文件的**目录**数（去重）。
依据 = 评审实测：七轴罚分等权加和，唯 S3 按文件计——500 个错位
文件的一个垃圾抽屉能把整份结构分打到 0，淹没其余全部轴；改后与
S6 体例一致（过线目录计 1）。

**覆盖映射（一现象一轴，不双计）**：散落 = 每个受害目录各计 1
（散得越广罚越多）；单目录文件爆炸 = S0 扇出 + S4 文档辖区；
堆叠/克隆 = S6 与 dedup 辖区。

> 分数迁移：修正案①改变同一仓库的结构分（先例：自仓 990→992）；2.26.0 密度律再次迁移全部结构分（质量法退役，批 6 verdict 先例），
> 发版说明必须声明（docs/RELEASE.md §0）。

## 拆分 ROI 顾问（v0.6，structure/1 扩展）

`ce structure --split-candidates`：对越过冻结软线的判决文件逐一计
价——最优缝（ROI≥1）或带数字的内聚豁免；v0.7 起缝价含四腿（引用
/克隆块/共变对/φ），MCP 与 GUI 面同批补齐。契约与 as-built 实录见
[size-advisory.md](size-advisory.md)；wire 形状见
[contracts/VERSIONING.md](../../contracts/VERSIONING.md) 2.14.0/2.15.0 条。
