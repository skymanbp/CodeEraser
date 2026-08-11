# M4 攻击评审（2026-08-11）——Codex gpt-5.6-sol（effort xhigh），独立只读

> 评审者：OpenAI Codex（`gpt-5.6-sol`，最高推理档，READ-ONLY），经
> codex-rescue 通路；范围 = M3 收口 8a71f12 → d086950（M4 全量）。
> 评审判定"验收在修复前不成立"（2 blocker + 10 major + 2 minor）。
> 每条经主线独立核实后处置；修复批全部经全语料复验。

| # | 级别 | 论断 | 核实 | 处置 |
|---|---|---|---|---|
| F1 | blocker | CI 门只读已提交 doc，活体重放不在 CI；结果未绑定源版本 | 属实（架构性：D2-7 样本载荷含私有仓全文，不可入 CI——用户拍板边界） | doc 增 `generated_from`（HEAD sha+dirty+ce 版本），约定 doc 落子提交；EVAL-SET 明文边界与复跑命令。CI 直跑活体重放**不做**（D2-7 优先） |
| F2 | blocker | GT 降维为计数，行身份替换与新 extras 不可见 | 属实（cross_gt 只有 per-file 计数） | SGR walker 增行号追踪；无修正文件升级**行身份门**（全语料一次通过）；extras 台账**冻结**（新行须审读+`CE_ACCEPT_EXTRAS=1`）；5 个修正文件如实保持计数门（修正行身份未存档） |
| F3 | major | 降级应答仍套用部分 L2，违背 pure-L1 承诺 | 属实（merge 先于 reason 判定） | reason 先判、降级即纯 L1；core_wire 增 65 洪泛+真移动 e2e 钉住（degraded=bucket_cap ∧ 零 delta ∧ 计数=纯 L1） |
| F4 | major | phase3 无边要求，pair0 的 X removal 被跨归因 | 行为属实，**判 disputed/按设计**：Provenance.hs:6-11 明文产品论点（删侧宽松=去重多对一形态），golden id=4 有意封存；25 无移动 commit 零虚报门在此语义下通过 | 不改语义；风险并入 R-L2-2（M5 第二语料外验，若现目的地竞争/过度归因形态则改判） |
| F5 | major | 重复哈希 run 凑满行数地板（[x,x]↔[x,x] 开站） | 属实（tryBlock 只计长度） | 地板改计**去重内容数**（Anchor+Reference 同步，81,640 穷举等价重跑）；golden 增 id=8 负例；全语料复验零漂移 |
| F6 | major | 非显著桥无上界，1000 行标点桥可压缩融合远端巧合 | 属实（side_runs 无桥长界） | 实测冻结切片桥宽直方图 {0:7037,…,7:1}，上界 = 实测最大 7（MAX_BRIDGE，注释带直方图）；全语料复验零漂移 |
| F7 | major | Rust impl 方法/Go 接收者方法伪顶层 → 假堆叠证据 | 属实（kinds 无 impl 容器；Go 方法键无接收者） | impl 成具名容器单元（trait 限定键）；Go 方法键并入接收者类型；impl 容器整类排除堆叠身份。首版修复被 FPR 重放抓到 `impl Foo`/`impl Advisor for Foo` 碰撞（1/600）→ 根修回 **0/600** |
| F8 | major | 非 hello 消息不校验 proto（裸发/9.0.0 被静默应答） | 属实（Envelope 只解 type/id） | core 逐消息强制 proto major（缺失/不符 → error/bad_request）；wire-errors 增两负例（双侧逐字节）；core_wire 补上 fourclass golden 消费（Rust 侧此前只消费 handshake 两件） |
| F9 | major | `i` 文档称不透明、实现按稠密位置回查 | 属实 | 契约 1.0.0 定稿明确：`i` = 稠密 0 基对位置（VERSIONING §1） |
| F10 | major | in_scope 只探文件路径，目录型模式（`vendor/`、`generated/`）不命中 → 预算规则误问 | 属实（Override/Gitignore 均未父目录匹配） | 祖先目录逐级探测 + `matched_path_or_any_parents`；e2e 覆盖 glob/ce.toml 目录/.gitignore 目录三形态 |
| F11 | major | Edit 重放语义分歧：非唯一 old_string 真工具拒绝而 guard 照判；CRLF 不归一可逃逸 | 属实 | 唯一性强制（非唯一→静默）+ CRLF 归一；表驱动 e2e 四形态（唯一/缺失/歧义/CRLF） |
| F12 | major | root commit `sha^..sha` 失败被静默吞，churn 账本失衡 | 属实（unwrap_or_default 吞错） | commit_pairs 以空树为基（`hash-object -t tree --stdin` 现算，兼容 sha256 仓）；最小 surviving-root 用例钉住 |
| F13 | minor | `--name-status` C 记录两路径只消费一个 → 流解析失步 | 属实（且 head_pairs 带 `-C` 会真实遇到） | C → (None, 目的地)（复制目的地=新增文件，恰是产品要抓的复制信号）；fixture 增 C 行+失步探针（copy 后记录仍正确解析） |
| F14 | minor | 文档 error code 闭集缺 `contract`（自相矛盾） | 属实 | VERSIONING §1 code 集补 `contract` |

## 复验总账（修复批收口时）

- 全电池 35 套 0 失败；clippy/fmt 净；棘轮 **201/201**（修复批自引入的
  9 块克隆全数根治：git 生成器归 session::git_stdout、堆叠证据拆
  stacking.rs、测试骨架合并）；scan 3 warn = 修前既存集合（batch.rs
  355 行 warn 因拆分反而消除）。
- L2 doc 与 FPR doc 重生成：语义加固（F5/F6/F7）后**零漂移**——
  547/547 行身份召回保持、0/600 保持；新增字段仅 gt_lines 与
  generated_from（F1/F2 落地本身）。
- 穷举参照等价 81,640 实例重跑通过（Reference 与 Anchor 同步 F5 语义）。

## 评审质量注记

14 条中 13 条经独立核实属实（其中 F1 的"CI 不跑活体"部分属用户拍板的
D2-7 边界而非疏漏）；F4 与既定设计冲突，按 ADR 维持并挂 M5 外验条件。
评审者对"评估仪器循环论证"的攻击（F1/F2）催生了行身份门与 extras 冻结
——本轮最有价值的两条。FPR 重放在修复批内当场抓到 F7 首版修复的新碰撞
（impl 键），是仪器有效性的实战自证。
