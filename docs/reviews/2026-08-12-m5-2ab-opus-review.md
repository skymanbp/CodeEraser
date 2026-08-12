# M5-2a/2b Opus 反审处置（2026-08-12）

> 评审通道变更后首战：Codex 不可用（用户指令 2026-08-12），改由 Opus 三透镜
> 工作流（wf_0b8340fd-4a1，model=opus effort=high）对 e2440df..127da23 六提交
> 做只读对抗审查——wire 契约 / 仪器完整性 / 检测器正确性。24 项发现（去重后），
> 每条经主线独立核实后处置；修复批 = M5-2b-iii。原始三份报告存工作流 transcript
>（机器本地）。清洁面（评审确认无恙）：freeze-before-resolver 成立（自仓 tip
> 恰=检测器提交，ladder/ 从未存在于任何 ref）、2.1.0 minor 合法（纯加法）、
> 四语法表字段对钉定 grammar 版本逐一核实为真。

## 处置表（severity 为评审原判；全部主线复核成立）

| # | 发现 | 处置 |
|---|---|---|
| W1 major | edgeCap 零测试覆盖=死旋钮（Spec.hs 只探 nodeCap；护栏一半失守） | Spec.hs 增 edgeCap+1 边行运行时探针（cap 先于校验故同行边合法）；nodeCap/edgeCap 双探针 |
| W2 major | 冻结档与检测器零 CI 耦合：spec.rs/md.rs 改动五档全部静默失效（RG3 无 CI 化） | 新门 self_universe_tracks_detector：自仓行 sha256 未变者重跑 detect 断言逐 kind 计数相等 + spec 逐行子串（2b 判据 CI 化）；近空转地板 25 行 |
| W3 major | ref_def spec 合成"{id}: target"非源行子串，测试被弱化迁就 | md.rs spec 改 target 本体（子串恢复）；md 子串测试改严格 line.contains 全站点 |
| W4 minor | caps Int ≠ 拍板③ Integer（规格漂移，无活溢出） | Cost.hs Integer 化 + Graph.hs toInteger 比较 + why 注释 |
| W5 minor | degraded 回复 8 字段仅断 2 个（type 错值=客户端全链失步而无红） | Spec.hs 补 type/id/counts.nodes/kept 断言 |
| W6 minor | 族级解码失败丢 envelope id → 单条坏请求引发全会话 L2 失步 | Protocol.hs 分发 rid <\|> envId 回退（fourclass/graph 同形类级修）；graph golden 新增第 6 对钉 id 回显 |
| W7 minor | sites.rs 模块头声称不存在的"slice gate 子串断言" | 头部改指真实的 drift 门（W2 使声明为真） |
| W8 note | nodes 行完全未校验 | 2g 语义落地时校验（flags 位域彼时才有定义）；如实推迟 |
| W9 note | 32 MiB 预检后置于整行物化，解码成本未被 caps 保护 | 如实注记（同机受信模型下接受）；不改 |
| W10 note | §2 变更分类规则未涵盖信封常数变更 | VERSIONING §2 增列（放宽=minor/收紧=major，2.1.0 为首例） |
| I1 major | 门无语料集锚：任一冻结档被删/改名 CI 仍绿（G10 缺位） | FROZEN_CORPORA 五名单常量 + 集合相等断言（排序后比较） |
| I2 major | 负对照计数误记：实为 23 行（20 代码孤岛+3 md）非设计的"10" | 设计档两处核正；负对照义务限定代码孤岛 import |
| I3 minor | 证伪常数在生成器/门间裸字面量重复 | constants() 单绑定双消费 |
| I4 minor | excluded 逐类计数零断言；SCOPE_EXCLUDES 无语料行使 | check_slice 增类目/正值断言；SCOPE_EXCLUDES 现状 inert（memory/ 未被任何 tip 追踪）如实接受为防护性 |
| I5 note | 冻结提交消息"clean tree"与档内 dirty:true 表述张力 | dirty 脏源=先落兄弟档（M5-1d 惯例），本档如实说明；消息已推不改 |
| I6 note | ce graph --sites 无自动化测试 | mod.rs 增 analyze 端到端温测（含非 UTF-8 存活） |
| D1 major | badge `[![alt](img)](url)` 发 1 站错标 link 载 img 目标，真 url 丢（语料 34 处） | 深度匹配 matching_close + 返回 start+1 允许嵌套自扫 → link(url)+image(img) 双站点；测试钉死 |
| D2 major | 双反引号行内码不掩码（单反引号逐个配对）——字面 2b 红条件违例 | CommonMark N 长 run 配对 merge_code_spans；测试钉死 |
| D3 major | mod_decl owner 自指退化（自仓 89/89） | detect() 过滤恰为站点单行的 unit；owner 测试钉死 |
| D4 minor | 多行 HTML 注释内链接发幻影站（语料 4 处） | comment_mask 跨行状态机；测试钉死。缩进代码块不建模（块上下文列表敏感）——头部如实声明 |
| D5 minor | 模块头declared裸 URL 发站与实现不符 | 头部改真：裸 URL 非站点=D3 拍板本义；`<…>` autolink 才是站点 |
| D6 minor | 空 spec 站点无声消失无台账 | why 注释如实声明；台账归 2f unresolved ledger |
| D7 minor | md owner 继承 units.rs 围栏盲标题扫描 | **推迟登记**：改 units.rs 触判决层输入（冻结 labels 链），单独批处理 |
| D8 minor | (path,line,kind,spec) 非唯一身份，2c rank_key 需唯一 | RawSite 增 nth 行内序号（文档序），JSON 输出携带；测试钉死 |
| D9 minor | 非 UTF-8 文件中止整个 analyze | lossy 读（与仪器哈希口径一致）；温测钉死 |
| D10 note | TS `import x = require("y")` 漏检（source 属 import_require_clause） | 2f TS 阶梯落地时补 SiteKind；登记为已知检出缺口（recall 侧，台账可见） |

## 重冻结（RG3 首次点火）

检测器语义变更（D1-D4）⇒ GRAPH_SELF_TIP 换钉至修复批代码提交、五档全量重生成
（外语料 tip 不变但 md 计数移动——badge 双站点/注释剔除的直接可见证据），
EVAL-SET graph 章 tip 同步。W2 的 drift 门自此让此类失效永不再静默。
