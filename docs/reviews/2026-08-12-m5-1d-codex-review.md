# Codex 评审记录 — M5-1d ripgrep 外验批（2026-08-12）

评审对象：commit 65fc3ab（ripgrep 三语料闭环 + 两项核缺陷根修 + 地板下
登记制）。评审通道 codex:codex-rescue（gpt-5.6-sol，xhigh，READ-ONLY）。
结论：**无 blocker**，3 major + 2 minor；五项逐条独立核实后全部成立，
处置落在本记录前两个提交（代码）与同一提交（重生成档）。

| # | 级别 | 发现 | 核实 | 处置 |
|---|---|---|---|---|
| C1 | major | extras 计数制按**全量 gt** 计费，waiver 空隙可让 ≤\|登记\| 条非 GT 预测隐身不入台账；且与消融的 eff 口径分歧（L2 500 vs 消融 505） | 属实：hyperlink_aliases.rs out 侧 4 条非 GT 身份只计 2；hyperlink/mod.rs in 侧 p==g 干脆无行（2 换 2 抵消）；两冻结档 500≠505 | extras 计费统一到**可回收基线** gt−below_floor（点火条件 pred+bf>gt），台账行带 below_floor 字段；重生成后 L2==消融==505/15 文件，新增 1 行 + 2 行加字段（内容逐字不变，集合 diff 复核）；waived∉pred 断言使 eff 计费恰为身份精确 |
| C2 | major | copy 对与源对共享 before 路径，take_delta 按路径聚合首对全吞——moved-bearing copy 将顺序依赖（够扣则静默错记、不够则 panic）；labels 投影丢 copied 记号 | 属实（潜伏：当前唯一 in-scope copy 零 moved 未点火） | "copy 不消费 before 侧"落进 labels 机器：copied 透传入 labels 对 + copy 对跳过 out 侧消费（git 保证非 copy before 路径唯一 ⇒ 歧义消除）+ CI 断言记号零漂移；重生成仅加 copied 字段、数字零变 |
| C3 | major | 登记行"结构性低于地板"性质无处断言；重复行静默放宽 miss 额度；**审读表编辑对默认 CI 不可见**（totals-only 锚） | 属实；且核实中发现**同形更重**：register_misses/edge_violations 经活动语料解析，CI 对外语料查 self.json 空表——requests/ripgrep 的单元/边登记门一直**空转**（绿灯来自查空表） | 三重锚定：cross_rows 断言 waived 行**绝不被预测**（被预测即自证非地板下）+ below_floor_for 咽喉拒绝重复行 + labels CI 门对审读表做**行身份锚**；评审表解析全面 **by-name 化**（tables_for/doc_corpus_name），两注册检查带语料参数；反事实：注入幻影单元 PhantomUnit 即红（旧解析静默通过） |
| C4 | minor | 乘积护栏机器字长（Int/usize）在 32 位目标可溢出绕过；66×0 / 5×118 非降级形状无回归钉（revert 旧护栏测试不红） | 属实（Spec.hs 零 bucket 覆盖确认） | 核 Integer 化 + 影子 u64 化（64 位行为不变：81,640 穷举等价重验绿）；新 e2e product_cap_spares_one_sided_and_small_product_shapes 钉两形状非降级（走真 core） |
| C5 | minor | generated_from 只记 HEAD+dirty，65fc3ab 的档生成于**代码也脏**的树——"档由该实现产出"从提交证据不可独立验证 | 属实（约定"dirty 只该覆盖 doc 文件"本批被违反） | 流程归位：修复批代码先行两提交，四档在**干净代码树**上重生成（labels dirty:false，其余 dirty 仅含先落的 doc）；CE_CORE_BIN 身份仍未入档，如实挂账（活体重放约束见 EVAL-SET 复跑章） |

Codex 复核通过的两项请求不变量（与我方独立验证一致）：旧护栏未点火 ⇒
双侧 ≤64 ⇒ 乘积 ≤4096 ⇒ 新护栏零判决漂移（单调性证明，冻结语料免重生
成）；空侧修剪窗口的精确解 = 对侧全量索引，与 Myers 完成时输出（含前缀
偏移）逐点一致；F3 全有全无由 client 端 reason 检查保持。

评审结论边界（如实记录）：C3 的"结构性低于地板"如今由生成期 waived∉pred
断言 + 复跑指引背书，仍非 CI 可重演（需活语料）；C5 的完全闭环需要档内
记录生成器二进制身份，成本收益比暂不支持，挂账。ripgrep 上游 082245dad
的网页 diff 截断使 Codex 未能独立复核全部登记行判断——登记行的原始 diff
审读记录在审读表 why 字段内，本地 git show 可全量重演。
