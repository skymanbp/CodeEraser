# Changelog 归档册（v1.6.x）

> v1.6.0 一条版本条目。2026-09-26 从 [CHANGELOG.md](../CHANGELOG.md) 迁出：
> 那一册第五次抵到 750 行硬线（计划 v2.30 步 5b 边界清零第一小批），按仓规「拆分优先于豁免」
> 另起一册。条目逐字节搬家，未改写；记录义务与两类内容的定义仍由原册声明。

## [v1.6.0] — 2026-09-05 — 墓碑残留判决进核、`ce commitmsg`、docdup `///` 合段（docdup 行与 1.5.x 不可比）

**无默认档位变更。** v1.5.1 发布后的 bench 落表（07b9155）与其补账：
- **bench 全序列在同一机器状态落表**：17 tag × 7 = 119 行；v1.4.1 / v1.5.0 / v1.5.1 各经
  `CE_BENCH_TAGS` 单 tag 重量（另一会话的 AutoShade 测试与前台游戏抢核，v1.5.1 量八次取一）；
  那次坐下跨了 UTC 午夜、28 行日期不同，而表头「每行同一个测量日期」此前没有执行者——`bench_render.rs`
  新增 `every_row_shares_one_measured_date` 门，并趁重启后的安静窗口把整条 17-tag 序列同一次坐下重量（UTC 09-03）。
- **ADR-006 具名重立账**，超容差上升的文件（旧→新行）：07b9155 三个序列多两版本的生成件
  `docs/BENCH.md` 169→183、`site/bench/index.html` 187→200、`site/zh/bench/index.html` 186→199；
  本批子仓 `it/bench_render.rs` 247→266（那条门与表头改句），主仓 `CHANGELOG.md` 637→656（本节两条）。
- **收尾清点（70-agent 九切面只读清点 + 逐条双人反驳核验，仓内确认项全落）**：`memory/` 改名 `.ccm/` 后
  计划书 :4 / :291 散文里的旧路径就地改；归档册两条根相对链接改为相对 docs/，归档册入冻结集
  （`frozen_set.rs` + 引文豁免表）；册 06 角色表 `deadcode.rs:296-300` 重瞄 `304-307, 325-326`；
  **`source_citations.rs` 只认 `.md:` 目标却自称「整个总体」**——拓宽到注释行里的 `.rs:` / `.hs:`，
  走遍 core/app、core/test、gui/src-tauri/src，扫描器拆入 `source_citations_parts/`；三处漂移引文重瞄
  （`ladder/md.rs:75→86`、`conn.rs:35→daemon/server/conn.rs:46-51`、`flags.rs:9` 两处引语按现文重引）
  并全部补锚文本，册 03 因 segments.rs 多一行位移的九条引文重渲染；`docs/assets/gui-structure.png`
  无读者删除（站点副本由 `shoot_gui.js` 生成并有 `site_screenshots` 门）。

**无默认档位变更。** 计划 v2.26 第一段（2026-09-04；用户三裁：分两段先量 FPR / 出处叙事算残留但 changelog
定位的文档豁免 / 命名 `tombstone`）——**墓碑残留只度量、不判决、零面变化**：
- **feed schema `ce.observe/0.7.0` → 0.8.0（具名断点）→ 0.9.0（v2.27 具名断点：判决键 `judged`，段级 `exempt` 条目带起始 `line`——下节步 4）**：PreToolUse 新事件 `tombstone`（仅当本次删了名字
  或命中时写，带 `erased_hashes` / `session_erased`，名字只以 fnv1a64 键出现）；Stop / precommit 行加性对象
  `tombstone`（`label` / `prose` / `erased` / `exempt` / `sites`，站点只写 `file:line kind`）；golden 重 bless，
  `plugin/README.md` feed 段与册 11 同步；precommit 命中时多印一行人读摘要，任何档位不阻断。
- **度量层 `cli/src/tombstone/`**（frames / names / marked / surfaces / role / texts / mod）：名字只出自结构位
  （非注释行且字面量之外的标识符 + 单元名、md 标题与列表首词；内联代码跨度只保活不声明），框架窗口两侧对称
  不成名，标记与名字的合取以句为单位、只读新增行，changelog 定位（路径或版本台账形）整文豁免并入账。
- **FPR 回放仪器** `it/tombstone_replay.rs`（`#[ignore]`，git 历史驱动，`CE_TOMBSTONE_REPO` / `_LIMIT`）+ 新册
  `docs/FPR-TOMBSTONE.md`：六轮各修一类定义缺陷（自仓命中提交 123 → 68 → 64 → 11 → 7 → 7，requests
  1 → 1 → 0 → 0 → 0 → 0）；终轮 9 处逐条仲裁 = 真阳 6 / 中间态 3（全是计划书横幅）/ 误报 0；门 ≤ 1 % 达成
  （requests 0/400，自仓 3/530 = 0.57 %；把真阳也当误报的保守读法 1.32 % 单独超线，如实写明）。
- **Stop 腿代价**（PERF-BUDGET.md v2.26 节）：干净树与 HEAD 二进制打平（0.580 vs 0.592 s）；27 文件改动树
  +0.65 s、72 % 是两个 git spawn；首测多付的 `rev-parse --show-prefix` 改 `HEAD:./path` 消掉，零改动不再配对。
- **十九条夹具全落测试**：`it/tombstone_guard.rs` 8 腿、`it/tombstone_audit.rs` 5 腿、`unit/tombstone/` 44 腿；
  `ce:allow(tombstone)` 刻意不接线；feed 站点串在测试里由部件拼出（字面 `file:line` 会被引文门当成引文）。
- **ADR-006 具名重立账**（旧→新行）：主仓 `hookio.rs` 248→260（schema 0.8.0 头注）、`proc.rs` 55→79
  （`git_feed`：一次 `cat-file --batch`）、`fourclass/session.rs` 156→169（`scoped_pairs`）、
  `docs/PERF-BUDGET.md` 296→317（v2.26 A/B 节）、`CHANGELOG.md` 656→678（本节）；软线 372→370 随重立挪动；
  子仓无超容差文件（两条 discrete 行随克隆消除退场）；册 13 自仓普查行由其腿重取（U 843→864、顾问行仍 0：`STOP_EN` 改私有、`Marked` 由读者拼写）；三份自仓冻结切片六行按名改签，t3 候选册 rs 准入 1233→1244 随之手改同增。

**无默认档位变更。** 计划 v2.27 第二段（2026-09-04 立项；用户三裁：立项 / 计划书横幅一类文档「段级台账见证 +
`[tombstone] ledger` 声明表兑底」两者都做 / 单词名字继续算、ASCII 3 字符地板维持）——按步就地记账：
- **步 2 段级台账见证**（`role::segment` / `Witness::Segment`）：changelog 定位的第三见证——被触段（`>` 引用块
  连续行，或标题到下一标题的正文）自身含 ≥ 3 个互异版本 / ISO 日期 / 短哈希记号即只豁免该段并入账（feed 条目带
  起始 `line`，schema 0.9.0）；K = 3 由第七轮回放定（真阳所在段记号 0、横幅段 33 / 75 / 77，窗口 [1, 33]，
  与整文件见证「至少三个标题」同一地板）；第七轮：自仓命中提交 7 → 4（三处横幅中间态转段级豁免、6 处真阳原样，
  保守读法 1.32 % → 0.75 %），requests 0/400 不变（`docs/FPR-TOMBSTONE.md` 第七轮节）。
- **引文门补一扇门**（子仓 `docs_citations_parts/passes.rs`）：`CE_DROP_VANISHED` 点名的条目在按行认领之前退役，
  原地重写的被引行（同一行号）可按名重签——此前点名只对孤儿条目生效，同号改文只能改标签绕行。
- **步 3 `[tombstone]` 配置节**（`config/tombstone.rs` / `tombstone/policy.rs`）：`tier`（类自己的档位，默认 observe，
  `[guard] mode` 不及；四档之外按名拒载）/ `budget`（缺席 = 不判，只入 feed）/ `ledger`（声明台账文件，任何语言整文
  豁免，feed `why` = `declared`）/ `terms`（仓库自有词汇永不成名，含复合词）；四键皆入 `knobs_digest`（默认档位拼写即
  静默）；度量层只多一个 `Policy` 参数（钩子与审计从同一次配置加载建它，回放与无表的仓库用默认）。
- **步 4 wire 6.6.0 `tombstone/1` + 三腿档位路由**（`core/app/CE/Tombstone.hs` / `CE.Tombstone.Cost` / `cli/src/tombstone/wire.rs`）：
  第十一判决族——Rust 只送每个候选面的三个整数 `[kind, marks, erasedNames]` 与预算旋钮（码 0），合取（散文 marks ≥ 1 ∧
  names ≥ 1、标签 names ≥ 1）、标签 / 散文分账与 `over`（sites > budget）全在核（`TombstoneProps` 六腿：真值表全枚举 /
  无预算恒 false / 边界 / 三类 contract 拒绝 / 降级面），golden 六对随 hello 能力表机器重生（request 行锚仍 6.0.0）；
  PreToolUse 经 daemon **2.1.0** 加性 `tombstone{rows,budget}` 转发到 daemon 持有的核链，Stop / precommit 复用 audit
  那一条核链（一次打开、两个判决）；三腿只读两位——类自己的 `[tombstone] tier` 与核答的 `over`——同真才出声
  （`guard/say.rs` 双语一句；deny 拒写、Stop 阻断、precommit 退 1），observe 档钩子不出声、终端面仍印一行信息；核不可用 / 旧核无此能力 /
  回执越界 = feed `judged.degraded` 具名，绝不阻断也绝不默过。**feed schema 0.9.0 改为具名断点**（无任何发布带过
  0.8.0 形）：`tombstone` 对象的计数与站点搬进 `judged{sites,label,prose,over}`，`rows` 计候选面，`tombstone` 事件的
  `mode` = 类档位；`frames::marks` 计数取代 `has_mark`、`names::spelled_all` / `wide_all` 取代首个命中。自仓 ce.toml 不声明 `[tombstone]`。
- **步 5 `ce commitmsg <file>`**（`audit/commitmsg.rs`，git commit-msg 钩子之面）：与 `ce precommit` 同一具身体
  （`precommit::run(face, message)`）再跑一次，把 git 交给钩子的提交说明当作多一个 Markdown 面——站点记
  `COMMIT_EDITMSG:行 prose`；注释行按仓库自己的 `core.commentChar` / `core.commentString`（二者互为别名、后设者胜，
  git 2.52 亲证）原地置空而不删除，行号即文件行号，`auto` 读作 `#`；deny 档越预算退 1，读不到文件退 2 而非默过；
  feed 事件 `commitmsg`（precommit 的行形、`session_id` 同为 null，golden 第 12 条）；parity 表与 `ce precommit`
  同行具名（GUI / MCP / 插件无此面：钩子在 git 里）、README 载体表按名省略、zh 面门加一形、`docs/reference/cli.md`
  再生；PR 正文存成文件即同一个面（CI 配方，不做腿）。两钩子都装时暂存集判两次；只装 commit-msg 即两者兼得。
- **步 6 方法学册 14 + 十三→十四全扫**（`docs/reference/methodology/14-tombstone-residue-the-erased-name-conjunction.md`）：九节——改动集与两个面、
  R 的定义与地板、标签框架、逐句合取、四见证豁免（含 K = 3 推导）、`tombstone/1` 与核的三行判决、三腿一档位一 feed、
  已知限制七条、验收（FPR 七轮 + 六探针 + 逐模块测试），每个数字引到 `file:line`；索引表第 14 行、册 13 导航行；how 页两语
  第十四张家族卡另立 `#residue` 节置于常数总表之后（该表手绘、只载树家族常数），十二枚常量芯片逐枚绑源常量（不含
  `PAIR_CAP`——树内两处同名、门解析不唯一；不含 `READ_CAP`——值 `4 << 20` 非字面数）；页题 / meta 十三→十四手改
  （facts_registry 字面位），其余 `count:booklets#word` 芯片由 bless 再生；判决数据流图两语第四行并入本族（archify 数据流布局只许
  0..4 五行，实渲亲证）：度量侧「Git windows & diffs · erased names」→ 判决侧「Change verdicts · Theil–Sen · join · conjunction」tag 加 `tombstone/1`（archify 还校验标签宽度：22 字符 136 px 超 124 px 节点即拒，故取伞名），
  几何不动重渲，README ×2 / how ×2 的 alt 同改；常数总表 alt
  的「每个常数」改「常数」——册 13 起图上已不载全部常数，句子早已不真。
- **codex 审阅批**（2026-09-04，用户令「用 codex 走 gpt-6-astra max 审阅和打磨」；20 条发现 19 落码 1 记账）。
  **完整性**：`texts::load` 把缺失 / 二进制 / 超 `READ_CAP` 的一侧也计入 `unread`（此前只计超 `PAIR_CAP`），
  `surfaces::added` 带回四分类 diff 的 `degraded` 位（有界 diff 把每行都当新增），feed 加 `unread_pairs` / `degraded_pairs`，
  三腿凡度量不完整**只记不判**（`Leg::blocks` 与 PreToolUse 的出声各加一位，终端行加注）；`cat-file --batch` 回执改流式读
  （`proc::git_feed` 返回子进程，超 cap 的 blob 经固定缓冲跳过、不再整份驻留）。**会话并集**：`tombstone` 事件等钩子决定后
  再落行并带 `applied`（deny = false，该次擦除从未发生，`session_keys` 跳过；ask = null），本次 after 侧重新声明的会话键记
  `revived_hashes` 并从并集减去（`tombstone::declared_keys`），并集按 feed 序折叠。**定义**：散文句在**整段**里切再按新增行
  认领（只拼新增行曾把 `We no longer` + `Consult X.` 隔行读成一句）；正文段记号不借 `>` 引用行；`role::version` 恰好三段
  （IPv4 不是版本）；`vocabulary` 收 `MARKS_ZH`；`frames::closes` 认独立成词的中文后缀（`cache已移除`）；`[tombstone] terms`
  经 `names::canon` 规范化并整词匹配（复合词此前永不命中）；同面同名只计一次。**提交说明只当面不当侧**（`measure_with`：
  主题行为标签、句子为散文，不声明、不保活、不受见证——`- X is no longer needed` 作列表首词此前把 X 保活成漏报）；
  `ce commitmsg` 改 `texts::read_capped` 有界读（超 cap / 二进制退 2），`comment_prefix` 正则收成
  `^core\.comment(char|string)$`（`core.commentary` 曾被读作前缀）且值逐字节保留（`## ` 的空格曾被 trim 掉）。
  **wire**：`consume` 要求 `over` 为布尔、站点表严格升序、`label + prose == |sites|`，畸形回执一律具名拒绝。**守卫**：
  `budget::shipped_budgets` 在配置漂移下保留声明的 `budget`（出厂值缺席 = 不评条件，抹掉它等于把类关掉），栅栏注记随
  拒绝句带出；Stop / precommit 的对按配置 `exclude` 走同一 walk 作用域。**拆分**：`tombstone/candidates.rs`（候选面 + 第三
  见证）、`guard/probe.rs`（重复探针，guard.rs 303→229）；precommit help 补墓碑类（`docs/reference/cli.md` 再生）。
  **FPR 第八轮**：自仓 536 事件 4 / 6（六处真阳原样）、requests 0 / 400；册加 Clopper–Pearson 95 % 区间列并改口
  「校准证据」（`docs/FPR-TOMBSTONE.md`）；**第九轮**（`///` 合段后）自仓 537 事件 6 / 9 = 六处原真阳 + 两处新真阳（本批
  自己的散文追述被删的 `added_lines`，随即改写）+ 一处误报（单词名 `docs`）——严格 0.19 %、保守 1.12 % 如实入册；requests 0 / 400。ADR-006 具名重立：主仓 `tombstone/surfaces.rs` 146→196、`tombstone/texts.rs`
  175→205、`guard/tombstone.rs` 112→178、`audit/tombstone.rs` 212→265、册 14 374→425、本文件 718→741（本节）；子仓
  `it/tombstone_audit.rs` 210→233、`it/tombstone_commitmsg.rs` 107→141、`unit/tombstone/surfaces.rs` 86→124、
  `unit/tombstone/wire.rs` 71→105（子仓五个新克隆块全部消掉，dedup 119 恒）。记账不改码：每候选面重解析 `role::segment`。
  顺带亲证 docdup 一条缺陷并按用户裁**当批修掉（v2.28 修正案）**：tree-sitter-rust 的 `///` / `//!` 节点跨到下一行列 0，
  `merge_comments` 只见 `//`，连续 `///` 永不合段且 `end_line` 多一行——合段与 `end_line` 改按节点**最后内容行**，`DOCDUP_REV`
  4→5 清缓存；同树前后对拍：ripgrep 段 251→568、重复对 7→50（printer / matcher 各 crate 逐字相同的 `///` 块），cobra / zod /
  requests 零变，自仓段 845→1462、重复对 0→1——`corelink.rs` 与 `Version.hs` 各带一段逐字相同的「台账为何不镜像」说明，收成
  corelink 一处、Version.hs 一句指回（0→0）；check 949 不动。冻结仪器按 EVAL-SET.md 再生成协议重冻结（复活 0c7c936^ 的三个
  生成器，五语料 docdup-segments / oracle / precision 同钉 tip；32 行普查零漂移，只有计数回声与 `docdup_rev` 变；self 八行按名
  改签过的内容按兄弟锚 sha 取回；requests 3 行 / cobra 1 行 md 段早随 5df1dad 的块规则漂移而外部语料无门，本次一并吸收）。
  **docdup 行与 1.5.x 不可比**（Rust 树的段几何整体变了）；check 分数不受影响。
