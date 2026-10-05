# v2.30 语言精度考题册·重生成记录归档（2026-10-03）

> 本册是 [EVAL-SET-LANGS.md](EVAL-SET-LANGS.md) 的归档：主册在算法轨 v2.33 W2a 记第五次退役并重生成时过了 750 行硬线，此前四次的记录（步 7、v2.31 步 4 提交 B、v2.32 步 2、v2.32 步 6）于那时从主册逐字节搬到这里，只搬不改——节内「本提交」「下一提交」指的都是各节自己那一次。第五至第九次（算法轨 v2.33 W2a、W2-text 与它的阶段 B、C、D）于阶段 E 记第十次时主册再过线，同样逐字节搬来、接在后面。本册与主册同入冻结集（`frozen_set.rs`：不扫芯片、不生成、退出引文门），行号引文一律不写。

## 步 7 之后：十一份精度册退役并重生成（2026-09-28）

步 7（3e3daa27，文档与事实）给语言表 `cli/src/scan/lang.rs` 加了一个只读面 `Lang::with_grammar()`（文法计数从此读它、不再刮取源码文本），
答案不动一处；但 `lang.rs` 在精度册的「答案出自哪些代码」清单里（`lang_provenance.rs` 的 `ANSWERED_BY`：共用阶梯、路径助手、站点检测器、
解析器配置名、选宇宙的走查、语言登记表、钉住的文法），门只按路径读提交历史、分不出只读面与表的改动，CI 在该提交上按名拒了全部十一份（gson 第一个
被点名："the code its answers come from moved after 92ed92e - regenerate it"）。本地收尾链没看见——门读的是提交，未提交的 `lang.rs` 改动对它不存在，
全量跑那一腿是绿的。两件事随之落下：

- **退役**（本提交）：十一份精度册删档、六门考题的 `stage` 翻回 `Audited`（与步 5 提交 A 的先例同形——档在不在与旗一致，`Exam::filed` 按每个语料核）；
  门 `lang_docs_answer_the_code_they_name` 补一条工作树腿：`ANSWERED_BY` 与各阶梯的路径在工作树里有未提交的改动（暂存或未暂存、含未跟踪）就按名拒，
  本地先于提交看见同一件事。
- **重生成**（下一提交）：在本提交的干净树上按语料逐份生成、每份生成后挪出树再生成下一份（生成器把 `git status --porcelain` 非空读作 dirty），
  十一份齐了放回，六门 `stage` 翻回 `Scored`；读数与退役前逐份比对，记在下文。

### 读数（重生成，2026-09-28）

十一份在退役提交 813f4976 的干净树上逐份生成（gson → jsoup → luarocks → koreader → stringr → covid19model → codeeraser → html5-boilerplate →
learning-area → lua → fmt，每份生成后挪出树、`git status --porcelain` 回到空再生成下一份；两份 Lua 档带退役前那两段 RG1 处置原文），
全部记 `generated_from` = 813f4976 / dirty = false。逐份与退役前的档（3e3daa27 上的 blob）比对：除 `generated_from`
（92ed92e，lua / fmt 两份为 95521640 → 813f4976）外**逐字节相同**——判分行、宇宙台账、站点缺口、走查记录、处置一字未动，行数不变，
册 06 §9 引的十一处 summary 行号照旧。这正是只读面的预期：`Lang::with_grammar()` 不改语言表的任何一行，答案没有理由移动；门按路径拒、
重生成按字节证，两者各守各的。六门考题的 `stage` 随本提交翻回 `Scored`。

## 步 4 提交 B 之后：十一份精度册第二次退役并重生成（2026-09-30）

分析轨 v2.31 步 4 提交 B（flow 考题冻结）给语言表 `cli/src/scan/lang.rs` 加了第二个只读面 `Lang::extensions()`（flow 考题表的扩展名列由门与产品的行相等地钉住、不再手抄），答案不动一处；`lang.rs` 仍在 `ANSWERED_BY` 清单里，门按路径读，2026-09-28 加的工作树腿在本地先于提交按名拒了十一份——与那次同一机制、同一处置：

- **退役**（本提交）：十一份精度册删档、六门考题的 `stage` 翻回 `Audited`；册 06 §9 引的十一处 summary 行在本提交上按构造缺目标，下一提交回绿。
- **重生成**（下一提交）：在本提交的干净树上逐份生成、每份生成后挪出树再生成下一份，十一份齐了放回，六门 `stage` 翻回 `Scored`；读数与退役前逐份比对，记在下文。

### 读数（重生成，2026-09-30）

十一份在退役提交 2ea957d8 的干净树上逐份生成（gson → jsoup → luarocks → koreader → stringr → covid19model → codeeraser → html5-boilerplate → learning-area → lua → fmt，每份生成后挪出树、`git status --porcelain` 回到空再生成下一份；两份 Lua 档带退役前那两段 RG1 处置原文），全部记 `generated_from` = 2ea957d8 / dirty = false。逐份与退役前的档（5278e747 上的 blob，`generated_from` = 813f4976）比对：除 `generated_from`（ce 1.7.4 → 1.8.0、commit 813f4976 → 2ea957d8，dirty 前后皆 false）外逐字节相同——判分行、宇宙台账、站点缺口、走查记录、处置一字未动，行数不变，册 06 §9 引的十一处 summary 行号照旧。`Lang::extensions()` 只读语言表的一行、不改它，答案没有理由移动；门按路径拒、重生成按字节证，两者各守各的。六门考题的 `stage` 随本提交翻回 `Scored`。

## 计划 v2.32 步 2 之后：十一份精度册第三次退役并重生成（2026-10-01）

权威轨 v2.32 步 2（定义进核 B）把测量侧的每张定义表改从核的 `tables/1` 定义包读（设计册 `docs/reference/authority-track.md` §4.5），语言表 `cli/src/scan/lang.rs`、走查 `scan/walk.rs`、产物规则 `scan/outputs.rs`、解析器配置名 `graph/keys.rs`、Java 阶梯与 JDK 名表 `graph/ladder/java*`、站点检测器 `graph/sites/call.rs` 都在 `ANSWERED_BY` 清单里，表的文本删除、读者改读包；十个对拍语料与本仓 e877f389 干净工作树（测试子仓就位）各十三面、新 / 旧二进制 143 对逐字节同（读数在设计册 §4.5），答案没有理由移动。门按路径读、工作树腿在本地先于提交按名拒了十一份（java 第一个被点名）——与 2026-09-28、2026-09-30 两次同一机制、同一处置：

- **退役**（本提交）：十一份精度册删档、六门考题的 `stage` 翻回 `Audited`；册 06 §9 引的十一处 summary 行在本提交上按构造缺目标，下一提交回绿。
- **重生成**（下一提交）：在本提交的干净树上逐份生成、每份生成后挪出树再生成下一份，两份 Lua 档带退役前那两段 RG1 处置原文，十一份齐了放回，六门 `stage` 翻回 `Scored`；读数与退役前逐份比对，记在下文。
- **读数**（ab16e390 的干净树，2026-10-01）：十一份与退役前的 blob 逐字节同，只差 `generated_from`（2ea957d8 → ab16e390，dirty 仍 false）；两份 Lua 档的处置文本原样带回；六门 `stage` 翻回 `Scored`。

## 计划 v2.32 步 6 之后：十一份精度册第四次退役并重生成（2026-10-03）

权威轨 v2.32 步 6 退役请求键 `judgedMask`（proto 8.0.0，设计册 `docs/reference/authority-track.md` §13 第 69 条），`cli/src/scan/lang.rs` 只改了 `Lang::judged_mask()` 与模块头的两段文档注释（请求不再带这一键），代码一字未动；但它在 `ANSWERED_BY` 清单里，门按路径读、工作树腿在本地按名拒了十一份（gson 第一个被点名）。十个对拍语料与本仓自身的切换门新 / 旧二进制逐字节同（设计册 §12 步 6 行），答案没有理由移动。处置与前三次同形：**退役**（本提交）十一份删档、六门考题 `stage` 翻回 `Audited`，册 06 §9 的十一处 summary 行在本提交上按构造缺目标；**重生成**（下一提交）在本提交的干净树上逐份生成、每份生成后挪出树，两份 Lua 档带退役前的 RG1 处置原文，六门 `stage` 翻回 `Scored`，读数与退役前逐份比对。**读数**（062d2d62 的干净树，2026-10-03）：十一份与退役前的 blob 逐字节同，只差 `generated_from`（ab16e390 → 062d2d62，dirty 仍 false）；两份 Lua 档的处置文本原样带回；六门 `stage` 翻回 `Scored`。

## 算法轨 v2.33 W2a 之后：十一份精度册第五次退役并重生成（2026-10-03）

算法轨 v2.33 W2a 把 Python / Lua / Go / C·C++ 四个阶梯的查找搬进核的 `resolve/1`（设计册 `docs/reference/algorithm-track.md` §11 第 16–27 条）：`cli/src/graph/ladder/mod.rs` 的分派改为把这四个语言的站点一次问核，`ladder/c.rs` / `c_search.rs` / `c_index.rs` / `lua.rs` 删除，查找住进 `core/app/CE/Resolve(.hs|/)`，降住进 `cli/src/graph/resolve/`；考题表的 C 与 Lua 阶梯清单随之列入这两处。`ladder/mod.rs` 在每份精度册的 `ANSWERED_BY` 里，门按路径拒了十一份。差分门（核对 a8db74a9 冻结的 Rust 阶梯，真树 7,934 个站点 + 每个语言两颗种子各 12,000 个随机站点，不一致 0）与切换门（十个对拍语料、自仓与六棵真树的五个面新 / 旧逐字节同）都说答案没有理由移动。处置与前四次同形：

- **退役**（本提交）：十一份精度册删档、六门考题的 `stage` 翻回 `Audited`；册 06 §9 引的十一处 summary 行在本提交上按构造缺目标，下一提交回绿。
- **重生成**（下一提交）：在本提交的干净树上逐份生成、每份生成后挪出树再生成下一份，两份 Lua 档带退役前那两段 RG1 处置原文，十一份齐了放回，六门 `stage` 翻回 `Scored`；读数与退役前逐份比对，记在下一行。
- **读数**（6227189b 的干净树，2026-10-03）：十一份与退役前的 blob 逐字节同，只差 `generated_from.commit`（062d2d62 → 6227189b，dirty 仍 false）；两份 Lua 档的处置文本原样带回；六门 `stage` 翻回 `Scored`。

## 算法轨 v2.33 W2-text 之后：十一份精度册第六次退役并重生成（2026-10-04）

算法轨 v2.33 W2-text 阶段 A 让 `resolve/1` 以文本过线（proto 9.0.0），并把 go.mod、`pyproject.toml` 的键、编译数据库的命令行 / 旗标 / 响应文件读法搬进核（设计册 `docs/reference/algorithm-track.md` §11 第 43–49 条）：`cli/src/graph/resolve/` 的降删掉、改为文本请求，`core/app/CE/Resolve(.hs|/)` 多出这些读法——两处都在 C 与 Lua 考题的 `ANSWERED_BY` 里（子仓 `it/eval_lang_parts/exams.rs`；C 的清单去掉已删的 `cli/src/graph/cmdline.rs`，读法如今在 `core/app/CE/Resolve/`），而十一份精度册共读的出处门要求它们的 `generated_from` 晚于每个被引路径的最后一次改动。差分门（阶梯与配置读法两组）全同，判决字节不应移动；重生成按 W2a 那次的两提交走。

- **退役**（本提交）：十一份精度册删档、六门考题的 `stage` 翻回 `Audited`；册 06 §9 引的十一处 summary 行在本提交上按构造缺目标，下一提交回绿。
- **重生成**（下一提交）：在本提交的干净树上逐份生成、每份生成后挪出树再生成下一份，两份 Lua 档带退役前那两段 RG1 处置原文，十一份齐了放回，六门 `stage` 翻回 `Scored`。
- **读数**：十一份在 c4eac446 的干净树上逐份生成、`dirty = false`，与退役前那一代逐份比对只差 `generated_from.commit` 一行（6227189b → c4eac446），判决、真值、宇宙台账与两段 RG1 处置逐字节同。

## 算法轨 v2.33 W2-text 阶段 B 之后：十一份精度册第七次退役并重生成（2026-10-04）

阶段 B 把 R 的两级阶梯、`DESCRIPTION` 读法与包代码展开搬进核（设计册 `docs/reference/algorithm-track.md` §11 第 50–53 条）：`cli/src/graph/ladder/r/` 删掉，`cli/src/graph/resolve/` 读出 `DESCRIPTION` 原文送核，`core/app/CE/Resolve/` 多出 `R.hs` 与 `Description.hs`、`World.hs` 的共用函数改写了 C 与 Lua 的两处同义写法——三处都在 C、Lua 与 R 考题的 `ANSWERED_BY` 里（子仓 `it/eval_lang_parts/exams.rs`；R 的清单从已删的 `cli/src/graph/ladder/r/` 改为 `cli/src/graph/resolve/` 与 `core/app/CE/Resolve(.hs|/)`）。差分门（阶梯与配置读法两组）全同，判决字节不应移动；重生成按前一次的两提交走。

- **退役**（本提交）：十一份精度册删档、六门考题的 `stage` 翻回 `Audited`；册 06 §9 引的十一处 summary 行在本提交上按构造缺目标，下一提交回绿。
- **重生成**（下一提交）：在本提交的干净树上逐份生成、每份生成后挪出树再生成下一份，两份 Lua 档带退役前那两段 RG1 处置原文，十一份齐了放回，六门 `stage` 翻回 `Scored`。
- **读数**：十一份在 1278a18c 的干净树上逐份生成、`dirty = false`，与退役前那一代逐份比对只差 `generated_from.commit` 一行（c4eac446 → 1278a18c），判决、真值、宇宙台账与两段 RG1 处置逐字节同（stringr、covid19model 两份 R 档亦然）。

## 算法轨 v2.33 W2-text 阶段 C 之后：十一份精度册第八次退役并重生成（2026-10-05）

阶段 C 把 Java 的四级阶梯与类型注解的读法搬进核（设计册 `docs/reference/algorithm-track.md` §11 第 54–57 条）：`cli/src/graph/ladder/java.rs` / `java_inherit.rs` / `java_pick.rs` / `java_sets.rs` 删掉，`java_header.rs` 只剩走查用的头部读法，`cli/src/graph/resolve/` 把每个 Java 文件头装进请求，`core/app/CE/Resolve/` 多出 `Java*.hs` 六个模块、`Request` / `Contract` / `Resolve` 的分派带上 Java——后两处在 C、Lua、R 与 Java 考题的 `ANSWERED_BY` 里（子仓 `it/eval_lang_parts/exams.rs`；Java 的清单从 `cli/src/graph/ladder/java*` 扩为它加 `cli/src/graph/resolve/` 与 `core/app/CE/Resolve(.hs|/)`）。差分门全同，判决字节不应移动；重生成按前一次的两提交走。

- **退役**（本提交）：十一份精度册删档、六门考题的 `stage` 翻回 `Audited`；册 06 §9 引的十一处 summary 行在本提交上按构造缺目标，下一提交回绿。
- **重生成**（下一提交）：在本提交的干净树上逐份生成、每份生成后挪出树再生成下一份，两份 Lua 档带退役前那两段 RG1 处置原文，十一份齐了放回，六门 `stage` 翻回 `Scored`。
- **读数**：十一份在 cc11c60f 的干净树上逐份生成、`dirty = false`，与退役前那一代逐份比对只差 `generated_from.commit` 一行（1278a18c → cc11c60f），判决、真值、宇宙台账与两段 RG1 处置逐字节同（gson、jsoup 两份 Java 档亦然）。

## 算法轨 v2.33 W2-text 阶段 D 之后：十一份精度册第九次退役并重生成（2026-10-05）

阶段 D 把 Haskell 的三级阶梯、`.cabal` 读法、可执行与测试入口和包内私有的判定搬进核（设计册 `docs/reference/algorithm-track.md` §11 第 58–62 条）：`cli/src/graph/ladder/hs.rs`、`ladder/paths.rs`、`graph/cabal.rs`、`graph/cabal_parse.rs` 与 `roots.rs` 的 `beside` 删掉，`ladder/mod.rs` 的分派与 `cli/src/graph/resolve/` 的请求带上 Haskell，`core/app/CE/Resolve/` 多出 `Hs` / `Cabal` / `CabalWalk` 三个模块——`ladder/mod.rs` 与 `roots.rs` 在每份精度册共读的 `ANSWERED_BY` 里，后两处在 C、Lua、R 与 Java 考题的阶梯清单里（子仓 `it/lang_provenance.rs` 的 `ANSWERED_BY` 去掉已删的 `cli/src/graph/ladder/paths.rs`，它最后一个函数 `one_of` 在核里早有同义的 `CE.Resolve.Answer.oneOf`）。没有 Haskell 考题；差分门全同，判决字节不应移动；重生成按前一次的两提交走。

- **退役**（本提交）：十一份精度册删档、六门考题的 `stage` 翻回 `Audited`；册 06 §9 引的十一处 summary 行在本提交上按构造缺目标，下一提交回绿。
- **重生成**（下一提交）：在本提交的干净树上逐份生成、每份生成后挪出树再生成下一份，两份 Lua 档带退役前那两段 RG1 处置原文，十一份齐了放回，六门 `stage` 翻回 `Scored`。
- **读数**：十一份在 9c1f7668 的干净树上逐份生成、`dirty = false`，与退役前那一代逐份比对只差 `generated_from.commit` 一行（cc11c60f → 9c1f7668），判决、真值、宇宙台账与两段 RG1 处置逐字节同。
