# M5-2c/2d Opus 对抗评审与处置（2026-08-12）

评审对象：`b4ad643^..5b65f8d`（抽样仪器 + 样本冻结 + 审计冻结）。评审通道 =
Opus 独立子代理（三透镜：仪器正确性 / 门强度 / 审计数据质量），全部发现经
主代理逐条第一方复核后处置。**结论：无 blocker；4 MAJOR + 6 MINOR + 4 NOTE
全处置**（本批 = 硬化提交，逐项见下表）。

## 攻而不破面（评审独立重导认证——2c 最强外部证据）

- **rank 极小性**：评审独立重建 4269 站点池（与五冻结 slice 总数逐语料相等）
  并以 Python 重实现全套选择逻辑，重导出的主样与后备**与冻结档逐字节同序全
  同**——冻结 100 确为逐 cell site-域 rank 极小值、后备确为每语言 audit-域
  极小值。最大余数分摊手算逐格核对（语言 16/25/17/21/21、cell 全对）。
- **nth 扩展**：全池 per-line 序号恰 0..n-1 无洞，零载荷碰撞、零 (corpus,
  path,line,nth) 重复——无 nth 则同行站点必撞 id，verify() 拒收整档。
- **审计数据**：100/100 echo 逐字节完整；39 条路径 truth + cobra "." 全部在
  钉定 tip 实存（38 blob + 1 树 + 根树）；#unit 锚全部在目标文件内实证；五
  个最难判例（cobra 14 候选、zod workspace 条件导出、Hangul/括号 slug、
  BinaryDetection 定义点+再导出链、FAQ 27 个 HTML 锚）全部攻而不破；零
  external 判决掩盖可证 in-corpus 目标；site_gaps 37 条计数与机制精确。
- git_in 重构语义保持；fetch-depth:0 影响面 = 仅克隆耗时。

## 发现与处置（severity / 位置 / 处置）

| # | 严重度 | 发现 | 处置（本批） |
|---|---|---|---|
| F1 | MAJOR | 14/4269 站（全为 zod TS 多行 import/export）违反"spec ⊂ 源行"——site() 取语句头行而 TS source 字段落后行；漂移门只测 self、单元表无多行 TS 用例，双盲 | 定约定：站点行 = 语句头。单元表 stray 规则与漂移门统一改 **spec ⊂ 语句窗口**（16 行前视，注 why）；sites.rs 增多行 TS 用例钉 (kind,spec,line=头)；EVAL-SET 子串句就地标注例外 |
| F2 | MAJOR | in-corpus 分母未登记未设门：60 external/40 in——cobra 1/requests 3/self 4/ripgrep 10/zod 22；G2 每语料精度门在 cobra 分母=1 | EVAL-SET 带分母登记（含 2h 待拍板标记）；门语义三选项（分母<5 只报不门 / 耗后备扩审 / 原样）走 AskUserQuestion，2h 前裁决 |
| F3 | MAJOR | 负对照被自家 GT 证伪：孤岛 rs 的 self::/super:: 站点 ≥13 个目标在本文件内（walk.rs:220→:131，且该行在冻结 100 中）——正确解析器会触发 2f 预注册红条件 | 设计档 D2 + §6 2f 红条件修订（注日期引 GT 行）：负对照义务限**跨文件引用**；intra-file self/super 站点须解析回本文件 = 更强重述 |
| F4 | MAJOR | truth 词表门任意单行串可过（"probably exceptions.py" 即"path-shaped"）；truth 列 = 2d 全部交付物，CI 零校验 | 审计门加 **truth 宇宙绑定**：base ∈ {"."} ∪ slice 文件 ∪ 目录前缀（评审已证 39/39 在宇宙内，强门今日安全）；反事实新增"发明路径必拒" |
| F5 | MINOR | 门不验 rank 极小性——评审构造 rank-last 伪样（跨语料换行）全门绿 | EVAL-SET 明写盲口 + 引本档独立重导认证；不加 ceiling 机械（威胁模型 = 生成期偏差，已由独立重导覆盖；生成器可随时在克隆上复跑复证） |
| F6 | MINOR | merge-base --is-ancestor 自反：样本+审计同提交可过 G13 | is_strict_ancestor（a≠d）双测试全用 |
| F7 | MINOR | 引线只认 ladder/ 字面路径；resolve.rs 或内联进 spec.rs 可绕 | 双层引线：ladder/ 须严格晚于审计 + **cli/src/graph 全树扫**——凡非样本前既有文件必须晚于审计（盲窗内落任何 graph 代码即红） |
| F8 | MINOR | bind_row 的 line 无上界、nth 界宽 | 注释如实改口：范围检查的真实边界 + 篡改由 rank 哈希抓 |
| F9 | MINOR | requests.json notes 手数 "13 of 15 external" 实为 12/3（同句自列三 in-corpus 行） | 表 notes 勘误（附 erratum 标注，rows 不动）——与主代理 ccm #631→#632 同族手数错误，规则重申：分布数字一律机算 |
| F10 | MINOR | 反事实未演练"缺行"与"echo 漂移"（doc 注释却声称覆盖） | 补两变体（remove(0) / spec 换假值）+ F4 的发明路径变体，共六连 |
| F11 | NOTE | md ref_def/ref_link/url 三 kind 零席位（44 站无 GT），2h 不可测 | EVAL-SET 明写不可测面（比例制无 kind 地板 = 设计忠实，如实陈列） |
| F12 | NOTE | dynamic/ambiguous/none 三词表值零 GT | EVAL-SET 注明由 2f 歧义/动态 fixture 面补 |
| F13 | NOTE | rung 重叠抽样结构性非盲（rung 需解析器实测） | 设计档 §5 澄清：价值 = 审计者自一致性，非盲评 |
| F14 | NOTE | 语料谱（12/15/25/18/30）登记而未设门——F5 伪样正是跨语料未被抓 | strata 门加语料谱常数断言 |

## 评审自报未查面（scope 如实）

未跑 ignored 生成器（以独立重实现替代——检测器代码两方共用，故"检测语义"
未被独立验证，spec⊂窗口性质除外——全池扫过）；未复审 2b slice 冻结本体；
why 全文精读仅 ~20/100（其余查目标存在性+echo+地板）；未验 site_gaps 穷尽
性；未审 100 后备行判决（设计即不审）；未跑全 CI 链；未验 SOURCES.md 上游
provenance（只证本地克隆自一致）。
