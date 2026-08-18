# M7 发布轨 — 章程与切片设计册

> 2026-08-17 · 用户拍板开工（先立章程+设计册）· 契约源 = DEVELOPMENT_PLAN.md v2.0 M7 行
> 本册地位与 M6 设计册同：切片划分与拍板记录的权威载体；计划正文仅在需要修正案时改动。

---

## 1. 契约范围（计划条款逐条溯源）

| 条款 | 出处（DEVELOPMENT_PLAN.md） |
|---|---|
| marketplace 上架、签名/公证、Releases 自动化（含 Linux/macOS 实包）、完整 MCP、许可证合规、文档、GUI 二期（趋势面板+删除候选浏览） | M7 行（§6 表末行） |
| 验收：陌生机器一条命令可用；SHA256 校验链路端到端；转公开前全历史审计（D2-7）；文档过 docdup 自检；默认档位切换依据入 CHANGELOG | 同上 |
| M7 公开上 marketplace + GitHub Releases | §3 分发行 |
| `ce eject` 落地于 M7（2026-08-13 拍板⑩） | §3 CLI 行 |
| 1.0 默认档位：T1/T2 精确重复写入与硬预算超限 → deny，其余 ask/warn 按各自 FPR 记录 | §4.2 演进路线第 3 级 |
| 二进制分发唯一化（Releases 下载 + SHA256 pinned 在插件清单）；代码签名/公证列入 M7 发布验收；air-gapped 手动放置保留 | ADR-007 |
| 签名完成前 README 明示未签名状态 | ADR-007 / R1 |

## 2. 现状清点（2026-08-17 实查，全部一手证据）

| 项 | 现状 | 证据 |
|---|---|---|
| 插件面 | 0.1.0 air-gapped 预览，已预告 "SHA256-pinned binary download ships at M7" | plugin/.claude-plugin/plugin.json |
| MCP | 最小面 2 工具：`scan` + `check_duplication` | cli/src/mcp.rs:71-99 |
| CLI 家族 | 14 臂：doctor/scan/churn/graph/deadcode/clone/docdup/join/structure/check/baseline/dedup/daemon/mcp | cli/src/main.rs:35-151 |
| `ce eject` | 未实现（main.rs 无该臂） | 同上（grep 零命中） |
| NOTICE | 不存在。**D1-7 原始前提已消失**：ast-grep-core 不在依赖树（ADR-005 自研 winnowing 定案）→ 义务收缩为依赖树许可证清单（cli / gui/src-tauri / core 三工程） | cli/Cargo.toml；core/ce-core.cabal:27-32 |
| CHANGELOG | 存在，2026-08-11 已有 ask 档位升级 + FPR 依据的完整先例格式 | CHANGELOG.md |
| 史审计对象 | 64780b9 / e296178 / d3f48df 三 commit 均在 HEAD 可达历史（携 memory/ 会话数据 blob） | `git cat-file -t` 三者均 commit |
| 实包 | Windows NSIS 已本地产出（0.0.1，6.2MB）；Linux AppImage / macOS dmg 未产（v2.0 移入本轨） | PERF-BUDGET.md M6 节 |
| 签名 | 零；README 亦尚未按 R1 明示未签名状态（本轨 P3 补） | 仓内无签名工件 |
| CI | 三平台编译门已建（macOS 随 schedule/tag，D2-8 计费约束落实）；发布产物 workflow 未建 | .github/workflows/ci.yml |

## 3. 切片划分（P1–P6）

依赖关系：P1→P6；P2/P3/P4 互相独立；P5 已依拍板①退役。终态序 = P1→P2→P3→P4→P6。

### P1 Releases 自动化（ADR-007 分发链路闭环）
tag 触发 release workflow：三平台构建 `ce` 二进制 + GUI 实包（NSIS/AppImage/dmg），产
SHA256SUMS；插件清单写入 pinned hash；SessionStart 下载→校验→拒绝执行（篡改样本）三态
端到端验证；air-gapped 手动放置路保留并回归。鸡蛋序设计：workflow 先产工件与校验和 →
pinned hash 提交 → 再打正式 tag（两段式，清单永不引用未产出的 hash）。
**验收**：draft Release 上三平台工件齐备；校验链自动验证（含故意篡改一例拒绝执行）。
**As-built（本批）**：plugin/bin/{manifest.env,ce.sh}——清单=sh 可 source 键值对（零
解析依赖，空 pin=air-gapped 回归按构造成立）；垫片解析序=数据目录已验证副本→带 pin
下载+校验+原子落位→PATH 兜底，**校验失败响亮拒绝不落位不转 PATH**；hooks.json 三钩改走
`${CLAUDE_PLUGIN_ROOT}/bin/ce.sh`；release.yml 两段式（dispatch 三平台 draft+SHA256SUMS，
tag 段**不重建**只验 pin 后 publish——Rust/GHC 构建非位可复现，重建即假 pin）；
bootstrap_e2e.sh **四态**电池（超合同一态：+PATH 回归）入 ci.yml 三平台，file:// 测试缝
（https 传输属 curl 契约，如实注记）；本地 Windows 4 态 PASS + 跳校验突变体反事实红。
draft 实弹（run 32087836041）：v0.1.0 七工件=三平台 ce 二进制 14.9/15.8/16.2MB（全落
ADR-007 预测 8–19MB 带）+ NSIS 3.9MB/dmg 5.7MB/AppImage 80MB + SHA256SUMS；本地下载
`sha256sum -c` OK——验收两腿（工件齐备+校验链三层）全过，P1 收口 ccm #1126。

### P2 产品面收尾（完整 MCP + eject + 1.0 档位）
MCP 面按拍板③扩展；`ce eject` 实现（清基线、`.ce/`、`CLAUDE_PLUGIN_DATA` 索引——§5.9.4
清单为准）；默认档位切 1.0 第 3 级（deny/ask），依据（各规则 FPR 记录）按 CHANGELOG
既有先例格式发布。
**验收**：MCP 新工具逐个 golden 对拍；eject 后目录状态断言（建→驻→eject→零残留）；
档位切换有 FPR 依据行。
**As-built（本批）**：MCP=mcp/{mod,tools}.rs，十工具**单一权威表**同时生成 tools/list 与
dispatch（判决族复用 daemon 的 core 解析器；写动词零暴露=拍板③硬形）；对拍电池=十面
MCP 文本逐一等于家族公共库序列化（暖索引稳态后字节相等——Summary 携刷新计数器，冷暖
态差异被电池首跑抓获后以稳态化根修）；库侧三处单喉上提（report::envelope 半函数、
graph::sites_json 公开、deadcode::report_json 出二进制入库）；eject=dry-run 默认+
`--yes` 实删（关 daemon 先行+有界拆卸重试），三电池=dry 零删/带活 daemon 零残留/
CLAUDE_PLUGIN_DATA 只扫 `ce-*`；档位 1.0=`config::PROMOTED_DEFAULT` 单常量（guard 执行
面与 health 报告面同源不可漂移），两晋级类 deny，Stop/precommit 无 FPR 记录维持
observe（如实缺席），CHANGELOG 第 3 级条目引第 2 级同一账本。

### P3 合规与文档
NOTICE（三工程依赖许可证清单，机器可复核：从 Cargo.lock/cabal 冻结面生成）；README 与
安装文档全翻新（含未签名明示——若拍板①走修正案）；全部对外文档过 `ce docdup` 自检
（M7 行验收原文）。
**验收**：NOTICE 与依赖冻结面零漂移（生成器入 CI 比对）；docdup 自检零上报。
**As-built（本批）**：NOTICE 563 行 548 依赖行=Rust 双 workspace 并集（cargo metadata
直读 license，无 SPDX 者具名注记）+ Haskell **exe 依赖闭包**（诚实测度：plan.json 步行
exe:ce-core 传递闭包 61 包，测试链 7 包按"不随二进制分发"出局）；许可证冻结表 64 行
机采（ghc-pkg 全局+store 双 db，2026-08-17 SPDX 原样）+覆盖门=闭包新包无表行指名红
（hs_boot 机生成表先例）；门=notice_gate.rs 字节比对（CE_BLESS=1 重生成），篡改反事实
红实证+复原绿；README 全英文事实版重写：0.x 状态/源码安装/**未签名明示+SHA256 校验指引
（拍板①修正案落地，R1 兑现）**/命令表/守卫档位与诚实边界/NOTICE 指针；docdup 自检腿=
CI 既有门对新 README/设计册增量保持零上报。首点火 ubuntu 红抓获**闭包按平台分歧**
（Win32 仅经 `time` 的 Windows 条件依赖进入；Linux 闭包恰少一行且无 unix——实探证
directory/process 均不在 exe 闭包）→修正案：NOTICE 测度改**三平台发布物联集**，
HS_PLATFORM_ONLY 冻结表补缺席平台行，walker 静默跳过改指名红（同覆盖门立场），
Linux 反事实（剥 Win32 复刻联集）与在仓 NOTICE 逐行同一。

### M7.5 清理批（v2.3 插片，用户三拍板 2026-08-18 ccm #1152）
深度瘦身+trend/1 入核，P6 前置。**As-built（瘦身半场）**：普查=7 代理测试级全量
census（65 文件逐 #[test] 二分 ignored/活门+工件读者映射+CI 接线枚举）推翻文件级
预估——22 个 eval 文件为混载体（活门=每 push CI 跑的一致性/精度/审计门，冻结 JSON
多数被活门钉住必须保留）；执行=8 整文件退役（5 仪器+eval_extract 三件）+27 个
generate_*/scout/重放 fn 割除+支援域零引用不动点清扫约 77 项（eval_support/
eval_extract/各 *_parts；#![allow(dead_code)] 静默死件靠引用普查非编译器）+
冻结件退役仅 manifest-v1.json（首拟 6 件被活门 t3_candidates_consistent 红回
5 件——each_frozen_doc 动态拼路是字面普查盲区，退役判据改"删后全量绿"实证）；
**crosscheck fixtures 保留**（graph 宇宙设计内负对照，活门在读——文件级预估的
第二处反例）；净 −11.2k 行；EVAL-SET 修正案记退役名录+git 复活路径（压缩至
256 行过 RM20 300 硬线，3j 门首触发即咬），FPR-REPLAY/PERF-BUDGET 封册；
世代门二次点火=3 员具名入 RETIRED（M7.5 类目）；全量测试+clippy 双绿证活门
零缺员；dedup 158 块（162→158 下棘 4 全入账）。教训=①普查必须测试级（同文件
混载）②动态拼路读者字面普查不可见=删冻结件必以全量绿实证 ③多行字符串 const
分号终止启发会误切 ④glob 重导出下词频普查须不动点迭代补岛效应。

### P4 GUI 二期（v2.0 移入项）
趋势面板：历史时序数据面按拍板②设计；删除候选浏览：消费 `ce join` 报告
（Tier F 三腿判决）+ dedup 块清单，GUI 列表+定位跳转，判决零渗入 JS（M6 立场延续）。
**验收**：趋势数据可从 git 历史重导出（缓存可重建性断言）；删候列表与 CLI 报告行
集合相等（同一 report_json 喉）。
**As-built（本批）**：trend.rs=`ce trend` 第八报告家族（ce.trend-report/0.1.0）：
每点=该 commit 在 detached 临时 worktree 里的 score::run 绝对分（NULL baseline/无
churn 窗/树自带 ce.toml），缓存表入 index.db **同一 wipe 生命周期（schema v7）=
可比性契约**（测量 rev 变更即趋势点失比较性，随库同批失效）；重建断言电池
trend_rebuild.rs 三腿=全量测→删库重measure逐行相等→纯缓存读，另 batch=1 增量算术；
MCP 第 11 工具+对拍行（库先行暖缓存）；删候浏览=join/dedup 两 report_json 原喉
（文档序渲染零排序零判决）；GUI 三 tab（structure/trend/candidates）+SVG 折线+
批量续测按钮（pending 算术）；worktree 名 sha+pid+seq 三元唯一（并发同 sha 撞车
实抓）；churn::git 报错补 stderr（贫瘠报错吞败因实抓）；棘轮第十四咬 +9 偿 8=
gui judged()/main_judge::family_cmd/mcp doc()/report::print_doc（join 同迁）/
commit_all+表驱动，残 1=同元数 tauri 委托构造性孪生具名入 ce.toml 161→162；
PERF 自仓=冷回填 16.5s/点（每点全冷索引）、暖读 1.1s——GUI 默认 30 点冷≈8min，
批量 5/次即此实证的设计。

### P5 签名/公证（已依拍板①退役为修正案）
裁决=本轨零外部付费：签名/公证整体后置 post-1.0（计划 v2.1 落款），README 未签名明示
归 P3、SHA256 校验链路归 P1 承重。本片不再推进，序退为 P1→P2→P3→P4→P6。

### P6 转公开与上架（不可逆动作集中片，末位）
镜像仓 filter-repo 演练（三 commit 的 memory/ blob 清除）→ 全史敏感扫描（密钥/绝对
路径/transcript/`.ce-eval` 泄漏——D2-7 全项）→ bundle 备份 → **执行清史（单独用户
确认，不可逆）** → 转公开 → marketplace 上架（公开仓即 marketplace，§5.10 布局）→
陌生机器一条命令验收（干净 VM/Actions 环境实测）。
**验收**：M7 行验收全表逐项 ✓；清史后全史扫描零命中；陌生机器 transcript 入册。

## 4. 拍板记录（2026-08-17 用户逐项裁决，四项全落 (a)）

| # | 决策 | 裁决 |
|---|---|---|
| ① | 签名/公证投入 | **(a) 修正案**：本轨不购证书/账号，README 明示未签名 + SHA256 链路承重，签名后置 post-1.0；计划 M7 行随改（v2.1）。落选：(b) 全投入 Windows 证书+Apple Developer；(c) 仅一平台 |
| ② | 趋势面板存储 | **(a) SQLite 缓存 + git 历史按需回填**——历史即真源，缓存可整表重建，零仓噪声。落选：(b) 同库只记增量（新 clone 趋势为空）；(c) 仓内时序文件（逐条追加=本项目要消灭的堆叠形态） |
| ③ | 完整 MCP 面范围 | **(a) 全家族只读报告面**：十家族 JSON 报告 + 既有 2 工具，零写动作——agent 不得自我重立基线/改配置。落选：(b) 报告+写动作（agent 自我豁免，与反堆叠立场相抵）；(c) 维持最小面（验收弱化） |
| ④ | 切片顺序 | **(a) P1→P2→P3→P4→P6**（P5 依①退役）：发布链先行，不可逆片末位单独确认。落选：(b) GUI 先行；(c) P6 演练先行 |

## 5. 验收映射（M7 行验收 ↔ 切片）

| M7 行验收项 | 承接切片 |
|---|---|
| 陌生机器一条命令可用 | P6（依赖 P1） |
| SHA256 校验链路端到端 | P1 |
| 转公开前全历史审计 | P6 |
| 文档过 docdup 自检 | P3 |
| 默认档位切换依据入 CHANGELOG | P2 |
| （签名/公证） | 拍板①修正案后置 post-1.0（P3 明示 + P1 校验链承重） |

## 6. 风险登记

| 风险 | 处置 |
|---|---|
| filter-repo 清史不可逆 | 镜像演练先行 + bundle 备份 + 执行前单独用户确认（P6 内置三道闸） |
| pinned hash 与 Releases 鸡蛋序 | P1 两段式发布流程（build→pin→tag），清单永不引用未产 hash |
| 未签名安装警告（SmartScreen/Gatekeeper） | 拍板①已裁决本轨不投入证书（后置 post-1.0）；两平台警告/拒开行为如实入 README 明示（P3），SHA256 链路为信任锚（P1） |
| 陌生机器验收环境 | 干净 VM 或 Actions 一次性 runner，transcript 入册 |
| marketplace 上架后 0.x 期望管理 | README 版本地位声明（0.x 预览→1.0 语义按 §4.2 档位联动） |
