# 墓碑残留度量 FPR 回放记录（2026-09-04，计划 v2.26 第一段 + v2.27 第二段步 2）

> 门（计划 v2.26 步 6）：500 次真实正常编辑误报 ≤ 1%；未达即停在 observe /
> feed-only；第二段 = 计划 v2.27（wire 族 `tombstone/1`、`[tombstone]` 档位表、册 14），步 2–4 已落，deny 晋级仍以本册为门。
> 方法：`cli/tests/it/tombstone_replay.rs`（`#[ignore]`，release 跑，常设）——
> 把 git first-parent 历史当真实编辑流：**每提交 = 一个改动集**（父 blob → 子 blob，
> `session::scoped_pairs` 配对、`texts::load` 一次 `cat-file --batch` 取文），跑钩子
> 同一个 `tombstone::measure`；每处站点印出**绑定的名字与摘录**供仲裁。
> 事件口径 = 提交（Stop 腿的单位就是一次改动集；PreToolUse 腿按文件度量，同一改动集只会
> 少不会多）。

## 语料

| 语料 | 事件（有父的提交） | 取法 |
|---|---|---|
| requests 尾段 400 提交（钉定 `1f6589ec` 止，与 FPR-REPLAY.md 同一钉定） | 400 | `git fetch --depth 401 origin 1f6589ec3a1ee910f9a65cc3ceac60b26677bc0e` 后 `checkout FETCH_HEAD`，`CE_TOMBSTONE_LIMIT=400` |
| CodeEraser 自仓全史（真实 agent 编辑流，文档极重） | 530（531 提交，根提交无 before） | 默认 |

## 七轮：每轮修一类机制缺陷再全量重跑

自仓每一处站点都读过原文；requests 只在第一至三轮出过同一提交的两处标签站点。

| 轮 | 自仓 命中提交 / 站点（标签 + 散文） | requests | 这一轮修掉的机制（修在定义，不是修在阈值） |
|---|---|---|---|
| 1 | 123 / 223（4 + 219） | 1 / 2（标签） | 段落被碰到即整段判：版本号改在一个列表项里，同段旧行写着「此前」——散文面改为**只读本次新增的行** |
| 2 | 68 / 88（4 + 84） | 1 / 2 | 三条准入：框架绑定的窗口在 before 侧也不是名字（`def test_header_no_return_chars` 搬一行被读成「删了 `return_chars` 又写回」——requests 那两处的全部来源）；含虚词的窗口（`the_pre`、`budget_is`）不是名字；仪器自己的词汇（`longer`、`removed`）不拼名字 |
| 3 | 64 / 81（0 + 81） | 0 / 0 | md 内联代码跨度**只保活、不声明**：计划书横幅一行 5000 字符就地重写，自己的跨度被读成「删了又提」（42/81） |
| 4 | 11 / 17（0 + 17） | 0 / 0 | 合取**以句为单位**：同一行里相隔 3000 字符的标记与名字不再合成一次命中 |
| 5 | 7 / 10（0 + 10） | 0 / 0 | 字符串字面量的内容不声明名字（`independent` 出自一条 caveat 消息、`linux` 出自 cfg 字符串） |
| 6 | 7 / 9（0 + 9） | 0 / 0 | —（第一段终轮：剩余 9 处逐条仲裁见下表，没有可修的机制缺陷） |
| 7 | **4 / 6（0 + 6）** | 0 / 0 | 第二段步 2（用户裁定 2026-09-04「段级见证 + `[tombstone] ledger` 兑底」）：changelog 定位的**第三见证 = 段级**——被触段（`>` 引用块连续行，或标题到下一标题的正文）自身含 ≥ K 个互异版本 / ISO 日期 / 短哈希记号即只豁免该段并入账（`role::segment`，`Witness::Segment`，feed 条目带起始 `line`）；三处中间态（全在计划书横幅）转为段级豁免，6 处真阳一个不少 |

changelog 定位豁免（`CHANGELOG.md` 路径见证）在每轮都做工：自仓 170 → 109 → 106 → **109**（第七轮 = 106 处整文件 + 3 处段级）次（第三轮起
R 为空的改动集提前返回、不再计豁免，数字是「豁免真正拦下站点的次数」而非「碰过 CHANGELOG 的
提交数」）；requests 的 `HISTORY.md` 同理 30 → 29 → 2（第六轮起字面量不再造名，requests 是
Python 语料、此前的 R 多由字符串里的英文词充数，R 为空的改动集不再走到豁免）。

## 第一段终轮逐条仲裁（自仓第六轮 9 处；requests 0 处）

仲裁尺度 = 用户 2026-09-04 裁定：出处叙事（注释 / 散文里追述「X 曾经在这、现在没了」）**算残留**，
只有 changelog 定位的文档豁免。三档：**真阳** = 规则要抓的；**中间态** = 文档本身不是 changelog
定位、但被命中的那一行是版本台账体裁，规则按字面抓对、按意图存疑；**误报** = 名字或标记读错。

| # | 提交 | 站点 | 绑定的名字 | 判 | 理由 |
|---|---|---|---|---|---|
| 1 | b883b5a1 | `cli/src/health.rs:48` | `doctor_line` | 真阳 | 头注追述被删的孪生函数「used to sit beside this one」 |
| 2 | b883b5a1 | `cli/src/health.rs:74` | `index_summary` | 真阳 | 「The WORDS used to be the fact — `index_summary`」：被删函数的出处叙事 |
| 3 | 9b9c8599 | `cli/src/graph/deadcode.rs:219` | `legacy` | 真阳 | 追述已裁除的 pre-2.28 `legacy` flags 列（wire 5.0.0 断代） |
| 4 | 9b9c8599 | `core/app/CE/Graph.hs:94` | `legacy` | 真阳 | 同一段叙事的 Haskell 侧 |
| 5 | e9e4f0ee | `docs/reference/methodology/07-the-three-signal-join.md:73` | `truncate` | 真阳 | 方法学册散文追述 Rust 侧不再 `truncate(20)`（册不是 changelog 定位） |
| 6 | f7812fac | `cli/tests/eval_graph_audit.rs:44` | `path_shaped` | 真阳 | 测试头注「any single-line string used to pass as "path-shaped"」 |
| 7 | bb09cc3e | `docs/DEVELOPMENT_PLAN.md:4` | `nul` | 中间态 | 计划书横幅：5000 字符的版本台账一行就地重写 |
| 8 | 9bedcc4c | `docs/DEVELOPMENT_PLAN.md:4` | `mentioned` | 中间态 | 同一行、下一个提交 |
| 9 | 7a8343df | `docs/DEVELOPMENT_PLAN.md:3` | `批_1_7` | 中间态 | 「批 1–7 全部执行但不再中间发版」：横幅里的版本台账句 |

第五轮仲裁出的唯一误报（同册 :75 的 `independent`，出自一条 caveat 字符串）已由第六轮的
字面量掩码修在定义上；第六轮 9 处里没有误报。

## 第七轮：段级见证的阈值 K 由本表定

第七轮先把见证解除武装跑一遍，给第六轮每处站点印出**所在段的互异记号数**（`ledger=` 列，记号 =
三段式 semver 或带 `v` 的两段式、ISO 日期、含数字与字母的 7–40 位十六进制短哈希；`§4.2`、`0.57 %`、
`Cost.hs:102-103`、纯数字的 CI 运行号都不是）：

| 站点（第六轮编号） | 判 | 所在段记号数 |
|---|---|---|
| 1–6 | 真阳 | **0**（五处在代码文件，见证只看 Markdown；册 07 :73 所在节本身无记号） |
| 7 `bb09cc3e` 横幅 | 中间态 | **77** |
| 8 `9bedcc4c` 横幅 | 中间态 | **75** |
| 9 `7a8343df` 横幅 | 中间态 | **33** |

窗口 = [1, 33]：K 取 **3**（`role::SEGMENT_TOKENS`）——与整文件见证「至少三个标题」同一地板，
三条记号是一张台账、两条是一次对比，且离最近的横幅还有 30 条余量。再全量重跑：自仓 4 / 6（6 处真阳原样，
3 处中间态转为 `docs/DEVELOPMENT_PLAN.md:3 segment` 豁免各一），requests 0 / 0、2 处整文件豁免不变。
`[tombstone] ledger` 声明表（第二段步 3）是它的兑底：横幅之外的台账性文件由用户按仓点名。

## 折算与门

事件 = 有父提交的提交（每提交一个改动集，见页眉口径）。

| 语料 | 事件 | 命中提交 | 严格读法（中间态计误报） | 保守读法（命中即误报） |
|---|---|---|---|---|
| requests 尾段 400 | 400 | 0 | 0 / 400 = **0.00 %** | 0.00 % |
| CodeEraser 自仓全史（第六轮） | 530 | 7 | 3 / 530 = **0.57 %**（折 /500 = 2.8） | 7 / 530 = 1.32 % |
| CodeEraser 自仓全史（**第七轮**，历史多一提交） | 531 | 4 | 0 / 531 = **0.00 %** | 4 / 531 = **0.75 %** |
| 两语料合计（第六轮） | 930 | 7 | 3 / 930 = 0.32 % | 7 / 930 = 0.75 % |
| 两语料合计（**第七轮**） | 931 | 4 | 0 / 931 = 0.00 % | 4 / 931 = 0.43 % |

**门（≤ 1 %）达成**：requests 两种读法都是 0；第一段自仓按裁定尺度（出处叙事算残留）为 0.57 %，
只有把 4 个真阳提交也当误报的保守读法在自仓单独超线（1.32 %），而那 6 处正是规则受命要抓的。
**第七轮起两种读法都在线内**：中间态归零，保守读法 0.75 %。第二段据此立项（计划 v2.27）；
本册所有数字由下方命令复现。

## 第二段的两条口径（2026-09-04 用户裁定）

1. **计划书横幅算不算 changelog 定位**：三处中间态全出自 `docs/DEVELOPMENT_PLAN.md` 前四行的
   版本横幅——文档整体是计划，那几行是台账。裁定 = **两者都做**：段级见证（第七轮，上表）
   + `[tombstone] ledger` 声明路径表兑底（整文件豁免、入账）。
2. **单词名字**：`legacy`、`truncate`、`nul`、`mentioned` 这类单词名字都在 3 字符地板之上、
   也不在词表里；6 处真阳里 4 处绑的是单词名。裁定 = **继续算，地板维持**；写进已知限制。

## 复现

```sh
export CE_CORE_BIN=…/ce-core.exe            # 判决核，替换为本机路径
cd cli
cargo test --release --test it -- --ignored tombstone_replay        # 自仓全史（默认）
CE_TOMBSTONE_REPO=<requests 尾段的 checkout> CE_TOMBSTONE_LIMIT=400 \
  cargo test --release --test it -- --ignored tombstone_replay      # requests 400
```

每行印 `| commit | label | prose | erased | exempt | sites |`，站点带绑定的名字与句摘录，
末行 `walked N of M commits … K with a site`；表里的数字即这些末行。
