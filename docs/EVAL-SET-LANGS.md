# v2.30 语言精度考题册（第四次拆册，2026-09-24）

> 计划 v2.30 设计册 [reference/language-expansion.md](reference/language-expansion.md) §11 与 §14 第 14 条的冻结登记。
> 每个新语言的图精度照 M5-2 的仪器优先脊柱走三步，顺序由提交先后证明：**站点宇宙 + 抽样冻结并提交 →
> 没看过解析结果的独立代理逐站判真值（GT）并提交 → 该语言的阶梯与精度册提交**（顺序门
> `cli/tests/it/lang_provenance.rs`）。母册链：[EVAL-SET.md](EVAL-SET.md) → [EVAL-SET-M5-3.md](EVAL-SET-M5-3.md) →
> [EVAL-SET-M5-CLOSE.md](EVAL-SET-M5-CLOSE.md) → [EVAL-SET-SIMILAR.md](EVAL-SET-SIMILAR.md) → 本册。本册与前四册同入
> 冻结集（`frozen_set.rs`：不扫芯片、不生成、退出引文门），行号引文一律不写。一个语言在它自己的步里加一节，
> 三步各记一段；重冻结 = 该语言考题的代数加一（考题表的 `generation`：检测器多读了一种写法、或换 tip），新一代
> 用新文件名，旧一代的档按名退役并在本册具名记一条——顺序门读一份档的首个提交，原地重写的档会留着旧一代的
> 提交；生成器拒绝覆写已冻结的档。

## 仪器与门

- **站点宇宙**（`cli/tests/it/eval_lang_parts/generate.rs` 的 `lang_slice`，`#[ignore]`，读 `.ce-eval/corpora/<名>`
  的钉住克隆，别的树按名拒绝）：按考题表 `eval_lang_parts::EXAMS` 的扩展名走一遍钉住的树，只取产品自己的走查读的
  文件（`scan::walk::Scope`，语料自己的 ignore 文件与内建排除照读；走查拒读的被追踪文件只计 `walk_refused`，不出题——
  考题只问产品读得到的东西），逐文件记检测器看到的文本 sha256 与各站点类的计数（`graph::sites`——只读文法 kind 表与文件内事实，不查任何路径，所以能先于阶梯冻结）。
  档 `contracts/eval/lang-slice-<语料>-v<代>.json`（代 = 考题表的 `generation`，同一门考题的档同一代）。
- **钉住的树**（`cli/tests/it/eval_lang_parts/tree.rs` 的 `lang_tree`，`#[ignore]`，读钉住克隆）：只给真值够得到整棵树的考题
  （考题表 `reach = Tree`，今为 HTML——页面取的是站点服务的东西，不限于页面）：`git ls-tree` 在钉住 tip 列出的每个路径，按字节序，
  只是 tip 的函数、不含任何产品判断。审阅门用它把真值与 `scope_gaps` 绑到真实文件上而不必有克隆；门把树对着宇宙核：宇宙的文件全在
  树上，其余按宇宙自己的排除计数（另一种扩展名 / 走查拒读）逐类对上；判分的生成器有克隆在手，判分前把树重导一遍。
  档 `contracts/eval/lang-tree-<语料>-v<代>.json`。
- **抽样**（同文件 `lang_sample`）：一个语言的全部冻结宇宙合成一个池，逐文件先复现它的冻结行再取站点——池等于
  冻结宇宙靠核对、不靠信任。秩 = `sha256(域|corpus|commit|path|line|nth|kind|spec)`（M5-2 的载荷顺序，spec 居末
  保单射）；每个站点类先取 min(15, 该类的池)，剩下的座位按各类剩余池的最大余数分满 100；主样本按审阅域哈希排列
  （审阅者看不到秩序）；每类另留 min(20, 池 − 配额) 道备用题，只在同类主样本无法作答时按序顶上（补分母不跨类，
  护住地板）。档 `contracts/eval/lang-sample-<语言>-v<代>.json`。
- **CI 门**（`cli/tests/it/eval_lang.rs`，不跑 git、不要语料克隆）：冻结集恰为考题语料、每份在考题的代数上（退役的
  一代留在树里，就读成那个语料出现了两次），审阅表的冻结集恰为已审阅考题的语料；每份宇宙过共用信封、钉
  tip、语言与范围；考题扩展名 = 产品路径表对该语言的扩展名；样本的配额从冻结宇宙的摘要重算、每行两个哈希从自身
  字段重导、同一文件同一类被抽中的数不超过它冻结行的计数、备用题按（类，审阅哈希）排列且秩排在同类每道主样本
  之后；篡改（伪路径、伪 spec、伪类、伪秩、伪审阅哈希、缺一行、调换两行、把备用题混进主样本）一律拒绝；交叉核对夹具
  （SOURCES.md 里该语言那行）按冻结行复核，每个 spec 都得落在它的语句窗口里。
- **判分**（同文件 `lang_precision`；打分 `eval_lang_parts/score.rs`、核对 `eval_lang_parts/precision.rs`）：逐文件先复现
  冻结行，再用产品的阶梯（`ladder::resolve`；Java 文件头按遍历的同一读法读，树里的解析配置照带，不声明任何根——语料都没有
  ce.toml）解该语料的每道主样本，按 M5-2 的五档判词与冻结真值比：站内答案与真值逐字相等、或阶梯没有声明成员时按文件相等为
  correct，包目录答案对包真值；External 是一种答案，对站内真值答 External 算 wrong；没有答案时，真值是关键词为 unresolved_ok，
  否则 missed。摘要 = M5-2 的 rescore（含按级截断表）加每个站点类一行（`type_ref` 单独可归因）；宇宙台账对冻结宇宙的每个站点
  都解一遍（逐类逐级、逐类逐拒答原因计数），解出率是召回的上限；审阅者记下的候选漏检逐条附上检测器在那一行读到的每个站点与
  阶梯的答案。生成器冻结前先跑 CI 的核对。档 `contracts/eval/lang-precision-<语料>-v<代>.json`；它是冻结档里唯一依赖产品代码的：
  阶梯的改动挪动了答案就重判——删档、在干净的树上重生成、在本册具名记一条。判分同样只问走查读的文件：真值指向走查拒读
  的文件（如 luarocks 的 `vendor/`）按站外判分，审阅者的原话另记在 `audit_truth`；样本行与候选漏检不得落在拒读的文件里；
  档多一节 `walk`（拒读的文件与它们的站点计数，台账不含它们）。真值够得到整棵树的考题（HTML）另记 `walk.unreached`——
  宇宙之外、走查拒读的树内路径（内建排除的 `*.min.js`、构建产物）——真值指向它同样按站外判分；门核它每条都在树上、
  不在宇宙里，并把这样的真值也照改判（塞进一条真值指向的树内路径，冻结行上审阅者的原话就对不上，档拒）。判分对着的文件集
  是走查读到的全部被判决文件（页面可以指向任何语言的文档或代码）加走查读到的资产（`Scope::assets`），不再只是该语言的宇宙。
  CI 门（`cli/tests/it/eval_lang_precision.rs`，不跑 git、不要
  克隆）：冻结集 = 已判分考题的语料（考题表的 `scored` 旗标与盘上的档逐语料相符，判分前必须已审阅）；每行按样本顺序回显身份
  与审阅真值、答案只能是三种形状之一、判词从行本身重算；摘要从行重算；台账的比率从两张计数表重算、逐类合计等于冻结宇宙；
  首级占比过触发线须带书面处置；候选漏检与审阅表逐条对应；精度不低于 0.90（整体，与站内真值不少于 5 道的每个语料；M5-2 的
  G2）。篡改（伪路径、伪 spec、伪真值、伪判词、非布尔的 external、缺一行、调换两行、翻转答案、改摘要、改台账计数、改比率、
  挪候选漏检的行号；走查记录里多一个宇宙外的文件、少一个拒读的文件、改拒读计数、把改判的真值记回站内、给没改判的真值
  配原话）一律拒绝，首级占比的触发线两个方向各有一腿。三份冻结档（样本、审阅表、精度册）的篡改电池共用一个框架
  （`eval_lang_parts/tamper.rs`）；精度册的电池篡改的是神谕档——只用冻结输入（样本、审阅表、宇宙）拼出、每道题都答它的真值的
  档（`oracle_precision`）。核对只把档对照冻结输入、从不对照阶梯，所以神谕档能过核对，篡改门也就不依赖哪门语言此刻判了分：
  阶梯一动，真的精度册就退役到重生成为止。
- **顺序门**（`cli/tests/it/lang_provenance.rs`，要完整 git 历史）：审阅表未提交时，任何提交都不得碰过该语言的
  阶梯路径，精度册不得存在；审阅表提交后，样本 ≺ 每张审阅表 ≺ 每份精度册，阶梯的首个提交严格晚于每张审阅表，
  且样本到审阅之间的盲窗里没有任何 `cli/src/graph` 文件落地（与 `graph_provenance.rs` 共用两层绊线）。最后一腿（步 5）
  把每份精度册钉在回答它的代码上：生成时树是干净的、生成它的提交在本历史里、此后没有提交碰过它的阶梯或答案所依赖的
  代码（`ANSWERED_BY`：共用的阶梯与路径助手、站点检测器、解析配置名、走查、语言表、文法钉版）——阶梯一动，门就红到
  重生成为止。别处的改动若也挪了答案，由发版前的重放找出（`eval_lang_parts::replay`，`#[ignore]`，读全部钉住克隆，
  逐键比对；`docs/RELEASE.md` §0）。考题表的 `ladder_first` 记阶梯先于考题的那两门（C / C++，步 6：阶梯随步 2 提交
  b7e78c7，先于任何考题）：门对它们不读「阶梯晚于审阅」，改核「阶梯的首个提交严格早于抽样」这一事实本身，并要求从抽样到每张
  审阅表（审阅未提交时到 HEAD）阶梯路径零提交，第二层绊线照跑；盲判只能靠流程——代理只读钉住克隆与自己那一批。

## 预登记常量（测量前写进每份档的 `constants`，改一个即换一套仪器）

| 常量 | 值 | 含义 |
|---|---|---|
| `total` | 100 | 每语言的主样本数 |
| `min_per_kind` | 15 | 每个站点类的地板（池不足则整类取尽） |
| `backup_per_kind` | 20 | 每类备用题上限 |
| 站点域 / 审阅域 | `ce-lang-site-v1` / `ce-lang-audit-v1` | 秩与审阅序用两个互不相干的哈希域 |
| `r0_share_trigger` | 0.80 | 首级（R1）解析占比过此即须书面处置——M5-2 风险 RG1「怯懦式精度」的预注册触发器，不是红灯 |

## Java（步 3）

### 站点宇宙与抽样（2026-09-24 冻结）

两个语料各用自己的钉住 tip，范围 = `*.java`，排除的只有其他扩展名：

| 语料 | tip | 许可 | 文件 | 站点 | import | import_star | type_ref | 排除（其他扩展名） |
|---|---|---|---|---|---|---|---|---|
| google/gson | `854c825` | Apache-2.0 | 264 | 22,698 | 2,674 | 0 | 20,024 | 49 |
| jhy/jsoup | `093e2f5` | MIT | 204 | 24,210 | 1,887 | 80 | 22,243 | 114 |

jsoup 是用户 2026-09-24 裁定加的（设计册 §14 第 15 条）：gson 一处通配导入都没有，只用它，`import_star` 这一类
与「经通配导入找到类型」那条路都考不到。gson 同时是交叉核对语料（SOURCES.md 的 java 行，五个文件，与宇宙同一 tip）。

样本：主 100 道——import 20 / import_star 15 / type_ref 65（import_star 取到地板 15，全部来自 jsoup；其余 55 座按
剩余池的最大余数分）；备用 60 道（三类各 20）；主样本落在 74 个文件上，按语料分是 gson import 8 + type_ref 30、
jsoup import 12 + import_star 15 + type_ref 35。

三份档的 `generated_from` 记 ce 1.7.4、树 `bac6169`、dirty = true：冻结时 Java 的检测代码本身还没提交（它和这三份
档在同一个提交里落地），与 EVAL-SET-SIMILAR.md 记 `143cfe0` 同理。

`type_ref` 是新站点类（设计册 D17：Java 同包引用不经 import，只能靠类型名站点补上），精度册里单独出一行。

### 真值（2026-09-24 冻结）

档 `contracts/eval/lang-review-gson-v1.json` 与 `lang-review-jsoup-v1.json`。100 道主样本按审阅序切成四批，每批
25 道交给一个独立的 Opus 代理；代理只读两个语料在钉住 tip 的干净克隆和自己那一批，不读仓库里别的东西、不跑 `ce`，
照 javac 的名字解析（JLS §6.4、§6.5.5、§7.5；跨模块可见性读各自的 `pom.xml`）逐站判目标。词表沿用 M5-2：声明目标
类型的语料文件（目标是文件里的成员时带 `#名`）、按需导入一个包时装着该包文件的目录（包拆在几个源根里时取导入者
自己源根下的那个）、`external` / `ambiguous` / `dynamic` / `none`；每条判词写明机制。装配逐字照录，判决不动。
100 道的 spec 全在记录的行上，零失配，没有动用备用题。

| 语料 | 主样本 | external | 文件 | `#成员` | 包目录 | none |
|---|---|---|---|---|---|---|
| gson | 38 | 26 | 11 | 1 | 0 | 0 |
| jsoup | 62 | 37 | 22 | 0 | 2 | 1 |

两处约定照判词记下。jsoup 的 `HtmlTreeBuilderState.java` 用 `import static org.jsoup.parser.HtmlTreeBuilderState.Constants.*`
导入本文件自己的嵌套类，按词表「本文件声明的类型」判 `none`，判词另记了按 `…/HtmlTreeBuilderState.java#Constants`
判的答案。两条 `import org.jsoup.nodes.*` 都在测试根里，而 `org.jsoup.nodes` 在 `src/main/java` 与 `src/test/java`
各有文件，按拆分包约定判 `src/test/java/org/jsoup/nodes`。代理另记了 5 条候选漏检（gson 2、jsoup 3），判分时逐条核实。

门（`cli/tests/it/eval_lang.rs`，读 `eval_lang_parts/review.rs`）：每张表逐行对应该语料的主样本、按样本顺序、身份
字段逐字回显；真值只能是四个关键词、该语料冻结宇宙里的文件（可带 `#成员`），或——只对按需导入——直接装着冻结
文件的目录；判词不短于 40 字符；摘要从真值重算；候选漏检只落在冻结文件上；篡改一律拒绝。考题表的 `audited` 旗标
（本提交对 Java 翻为 true）必须与盘上的表逐语料相符：表消失会被点名，不会被读成「尚未审阅」；顺序门读同一个判定。

### 判分（2026-09-25）

档 `contracts/eval/lang-precision-gson-v1.json` 与 `lang-precision-jsoup-v1.json`，与 Java 阶梯同一个提交落地（顺序门：
审阅表 ≺ 阶梯的首个提交 ≺ 两份档的 `generated_from`）。两份档的 `generated_from` 记树 `2090e57`、dirty = true：判分时阶梯
本身还没提交，与抽样那三份档同理。

| 语料 | 主样本 | correct | wrong | missed | external_ok | unresolved_ok | 精度 | 召回 |
|---|---|---|---|---|---|---|---|---|
| gson | 38 | 12 | 0 | 0 | 17 | 9 | 12/12 | 12/12 |
| jsoup | 62 | 22 | 1 | 2 | 19 | 18 | 22/23 = 0.957 | 22/24 |
| 合计 | 100 | 34 | 1 | 2 | 36 | 27 | 34/35 = 0.971 | 34/36 |

按站点类：`type_ref` 65 道 correct 26、wrong 0，站内真值 26 道全中；`import` 20 道 correct 7、wrong 0；`import_star` 15 道
（全在 jsoup）correct 1、wrong 1、missed 2。按级截断：只收 R1 的答案是 7 对 0 错，收到 R2 是 8 对 1 错，收到 R3 起 34 对
1 错。门是 M5-2 的 G2：整体与站内真值不少于 5 道的每个语料都不低于 0.90——两个语料与整体都过。

- 唯一的 wrong：jsoup 的 `HtmlTreeBuilderState.java` 第 22 行 `import static org.jsoup.parser.HtmlTreeBuilderState.Constants.*`
  导入的是本文件自己的嵌套类 `Constants` 的静态成员。阶梯在 R2 答了这个文件本身，真值按词表「本文件声明的类型」是
  `none`；审阅者在备注里另记了按 `路径#名` 读时的答案 `…/HtmlTreeBuilderState.java#Constants`，那样会在文件级相符。照冻结
  真值判错，阶梯不为这一道改。它在图上是一条自己指向自己的边：不给任何文件添可达性，只会让一个本就不可达的文件的死代码
  判词从「无人引用」变成「有引用但不可达」。
- 两道 missed：测试根里的两条 `import org.jsoup.nodes.*`。这个包在 `src/main/java` 与 `src/test/java` 各有文件，阶梯拒绝在
  两个目录里挑一个（ambiguous_root）；审阅按拆分包约定取导入者自己源根下的目录。这个包的两个目录里没有同名文件（main
  22 个、test 18 个），所以这两个文件经通配导入用到的类名各有自己的 `type_ref` 站点、逐名解到唯一声明它的文件——缺的只是
  包这一级的边。`[graph.search_roots] java` 解不了这种拆分：声明哪一个根，另一个根里的导入者就会指向这个根。
- 27 道 unresolved_ok 的真值都是 `external`，阶梯答 out_of_scope：JUnit 4 / 5、Truth、protobuf、jspecify、Jackson 这些第三方
  库，JDK 名表答不了；拒答不算错，也不进召回的分母。

宇宙台账（对冻结宇宙的每个站点都解一遍，解出率是召回的上限）：

| 语料 | 站点 | 解出 | R1 | R2 | R3 | R4（External） | 拒答 |
|---|---|---|---|---|---|---|---|
| gson | 22,698 | 87.0 % | 987 | 98 | 7,034 | 11,634 | 2,945，全是 out_of_scope |
| jsoup | 24,210 | 86.7 % | 712 | 146 | 11,659 | 8,467 | 3,226，其中 3 个 ambiguous_root、其余 out_of_scope |

首级（R1）在解出里的占比 gson 5.0 %、jsoup 3.4 %，远低于 0.80 的触发线，不需要书面处置。

候选漏检 5 条逐条核实（每条附检测器在那一行读到的站点与阶梯的答案），没有一条是真漏：gson 的
`ReflectionAccessFilterTest.java` 那一行读到 `JsonReader`（R3 解到 `gson/src/main/java/com/google/gson/stream/JsonReader.java`）
与 `IOException`（External），没读的是本文件声明的 `ClassWithoutNoArgsConstructor`——`type_ref` 按设计不收本文件自己声明
的类型；gson `JavaTimeTypeAdapters.java` 的 `TypeAdapters.FactorySupplier` 读到并解到 `TypeAdapters.java`；jsoup
`HttpConnection.java` 的两行里两处 `Connection.Base`、两处 `Connection.Request` 都读到并解到 `Connection.java`（没读的
`HttpConnection.Base` 以本文件的类开头，真值本就是 `none`）；jsoup `TokeniserState.java` 那条走两层嵌套类的 static 导入
读到，R2 解到 `Document.java`。

## Lua（步 4）

### 站点宇宙与抽样（2026-09-25 冻结）

两个语料各用自己的钉住 tip，范围 = `*.lua`，排除的只有其他扩展名：

| 语料 | tip | 许可 | 文件 | 站点 | require | load | 排除（其他扩展名） |
|---|---|---|---|---|---|---|---|
| luarocks/luarocks | `2d2cc8e` | MIT | 162 | 607 | 607 | 0 | 347 |
| koreader/koreader | `d9cd278` | AGPL-3.0 | 594 | 5,025 | 4,938 | 87 | 370 |

Lua 与 R 从一开始就各取两个语料，一个包、一个应用（设计册 §14 第 17 条）：包按模块名找到自己的文件（`require`），
应用还按路径（`dofile` / `loadfile`）——luarocks 的站点宇宙里 `load` 一类为零，只有 koreader 考得到。luarocks 同时是
交叉核对语料（SOURCES.md 的 lua 行，五个文件，与宇宙同一 tip）。koreader 是 AGPL-3.0：冻结档只记它的路径、哈希、
计数与站点的 spec 片段，交叉核对夹具只取 MIT 的 luarocks。

样本：主 100 道——require 84 / load 16（load 先取地板 15，余下 70 座按最大余数再得 1 座）；备用 40 道（两类各
20）；主样本落在 87 个文件上，按语料分是 luarocks require 10、koreader require 74 + load 16。

三份档的 `generated_from` 记 ce 1.7.4、树 `ced7ea8`、dirty = true：冻结时 Lua 的检测代码本身还没提交（它和这三份
档在同一个提交里落地），与 Java 那三份同理。

### 真值（2026-09-25 冻结）

档 `contracts/eval/lang-review-luarocks-v1.json` 与 `lang-review-koreader-v1.json`。切批与读法同 Java 一节：100 道主样本
按审阅序切成四批，每批 25 道交给一个独立的 Opus 代理，只读两个语料在钉住 tip 的干净克隆和自己那一批，不读仓库里别的
东西、不跑 `ce`（派卷前清掉了 luarocks 副本里插件钩子留下的 `.ce/` 索引——那是解析结果）。判词照 Lua 自己的装载规则：
`require` 走 `package.searchers`，用项目实际运行时的 `package.path`（入口脚本、启动器、测试配置、安装布局设的那一个），
答第一个命中的模板找到的语料内文件；`dofile` / `loadfile` 的路径相对进程的工作目录，按项目从哪个目录运行来判。词表：
语料内文件、`external` / `ambiguous` / `dynamic` / `none`。装配逐字照录，判决不动。100 道的 spec 全在记录的行上，零
失配，没有动用备用题；没有 `ambiguous` / `dynamic` / `none`。

| 语料 | 主样本 | external | 文件 |
|---|---|---|---|
| luarocks | 10 | 0 | 10 |
| koreader | 90 | 12 | 78 |

koreader 的 `require` 多落在 `frontend/`：`setupkoenv.lua` 把 `common/?.lua;frontend/?.lua;plugins/exporter.koplugin/?.lua;`
放在搜索路径最前，各启动器先进入安装目录（Makefile 把仓库的 `frontend/` 与 `plugins/` 链接进去），16 道 `load` 也在那个
目录下按路径打开。12 道 external 是 LuaJIT 内建（`ffi`、`bit`）、没检出的子模块 koreader-base 提供的模块（`ffi/*`、
`libs/libkoreader-lfs`）与 LuaSocket（`socket.url`）。luarocks 的 10 道落在 `src/luarocks/` 与 `spec/util/`：源码运行的包装
脚本、安装后的启动器、单文件版与测试四种运行方式到的是同一个文件。

两条约定照判词记下。搜索路径的第一个模板 `common/?.lua` 指向 koreader-base 的构建产物，克隆里没有：代理按「那里没有与
koreader 自己的模块同名的文件」判（若有，那些行会变成 `external`，KOReader 自己也会坏）。测试的运行器在同一个子模块里，
测试的工作目录按 `make/emulator.mk` 推定。候选漏检 16 条（luarocks 12、koreader 4）：`pcall(require, "…")` 传字面模块名
8 条（其中 7 条是 luarocks 各文件首行的 `compat53.module` 兼容前言），`loader.lua` 里 `require` 的局部别名 4 条，拼出来
的 `dofile` 路径 3 条，可能的伪站点 1 条（`spore_spec.lua:67` 的 `require` 取回的是预先塞进 `package.loaded` 的桩）；判分时
逐条核实。另有 7 条落在 luarocks 的命令行启动脚本 `src/bin/luarocks`（没有扩展名的 Lua 脚本，两批各自记下）：按扩展名走的
冻结宇宙看不到它，所以不算本册的候选漏检，照录在审阅表新的 `scope_gaps` 栏。

### 第二代：受保护的加载（2026-09-25 重冻结）

用户裁「现在支持」（2026-09-25）：`pcall(require, "x")` 调用 `require("x")`、把错误交回而不抛出，是可选模块的惯用
写法；检测器从此把受保护的调用读成它保护的那次调用（`graph/spec.rs` 的 `LUA_PROTECTED`：`pcall` 的目标实参跟在
函数之后，`xpcall` 的跟在消息处理函数之后——Lua 5.2 起与 LuaJIT 把其余实参传下去，5.1 的 `xpcall` 不传）。第一代的
站点宇宙少记了这类加载，按上面的重冻结规则整门考题升为第二代。第一代五份档按名退役，删出树、历史里仍在：
`lang-slice-luarocks-v1.json`、`lang-slice-koreader-v1.json`、`lang-sample-lua-v1.json`（冻结于 `9d28d6b`）与
`lang-review-luarocks-v1.json`、`lang-review-koreader-v1.json`（冻结于 `8f823c0`）。

| 语料 | tip | 文件 | 站点 | require | load | 冻结行变了的文件 |
|---|---|---|---|---|---|---|
| luarocks/luarocks | `2d2cc8e` | 162 | 702 | 702 | 0 | 77 |
| koreader/koreader | `d9cd278` | 594 | 5,058 | 4,967 | 91 | 16 |

多出的 95 + 33 个站点逐文件对过受保护加载的字面写法：luarocks 的 95 个都是 `pcall(require, "…")`，其中 73 个是
Teal 编译产物首行的 `compat53.module` 兼容前言；koreader 的 33 个是 29 个 `pcall(require, "…")` 与 4 个
`pcall(dofile, "…")`；两个语料都没有这样用 `xpcall`。字面写法里没读成站点的都该如此：拼出来的模块名 3 处
（`"luarocks.build." .. btype` 一类）、写在字符串里的生成代码 2 处（luarocks 为 Unix 与 Windows 拼的包装脚本）、块注释
里的 1 处（koreader `frontend/device/kindle/device.lua` 注释掉的 `isWifiUp`）。同一个检测器重生成 Java 与 R 的四份站点
宇宙，除 `generated_from` 外与冻结档逐字相同（gson 22,698、jsoup 24,210、stringr 55、covid19model 1,697 个站点）：
`LUA_PROTECTED` 只在 Lua 这一臂。

样本：配额不变（require 84 / load 16，备用两类各 20）；主样本 97 道与第一代相同——秩只由站点自身的字段定，新进的
站点只挤掉排在它们之后的——3 道新进（luarocks 两处 `compat53.module`，koreader `frontend/userpatch.lua` 的
`pcall(require, "android")`），3 道被挤出（koreader 的 `ffi`、`ui/event`、`ui/widget/inputtext`）；按语料分是 luarocks
require 12、koreader require 72 + load 16，落在 87 个文件上。三份档的 `generated_from` 记 ce 1.7.4、树 `8f823c0`、
dirty = true：受保护调用的读法与这三份档在同一个提交里落地。

### 第二代真值（2026-09-25 冻结）

档 `contracts/eval/lang-review-luarocks-v2.json` 与 `lang-review-koreader-v2.json`。切批与读法同第一代：四个新起的独立
Opus 代理（没看过第一代的表）各判一批 25 道，只读两个干净克隆和自己那一批，不跑 `ce`（派卷前查过两个副本都没有 `.ce/`）；
简报只多一句「受保护的调用装载的就是不受保护时装载的那个文件，程序容忍模块缺席不改变装的是哪个文件」。装配逐字照录，判决
不动。100 道的 spec 全在记录的行上，零失配，没有动用备用题；没有 `ambiguous` / `dynamic` / `none`。

| 语料 | 主样本 | external | 文件 |
|---|---|---|---|
| luarocks | 12 | 0 | 12 |
| koreader | 88 | 12 | 76 |

**与第一代的一致程度（盲评本身的噪声读数）**：97 道两代共有的题（秩是站点自身的哈希，同秩即同站点），两批互不相识的代理
判词 97/97 相同，判到的文件逐题相同。3 道新进的题：luarocks 两处 `compat53.module` 前言（`src/luarocks/build/cmake.lua:1`、
`src/luarocks/fetch/cvs.lua:1`）由两个不同的代理各自判 `vendor/compat53/module.lua`——只在 Lua < 5.3 上运行；文档写明的
构建（`GNUmakefile` 为 5.1 / 5.2 打包 `vendor/compat53/`、装到 `$(luadir)/luarocks/vendor/` 并放进搜索路径）、源码树里的
包装脚本（`LUA_PATH=src/?.lua;vendor/?.lua`）与单文件版都装它；两个代理都记下同一条保留意见：`make bootstrap`、
`--with-system-rocks` 与 busted 测试环境装的是另装的 compat53 rock，若要求每种运行方式（含测试）一致，这一题没有单一的
语料内答案。koreader `frontend/userpatch.lua:7` 的 `pcall(require, "android")` 判 `external`：`android` 由 Android 启动器
子模块（`platform/android/luajit-launcher`，未检出）提供，语料里没有 `android.lua`、preload 项或搜索器。

12 道 external：LuaJIT 内建（`bit`）、未检出的 koreader-base 提供的模块（`ffi/util`、`ffi/SDL3`、`ffi/drawcontext`、
`ffi/input_pocketbook`、`ffi/linux_input_h`、`ffi/posix_h`、`libs/libkoreader-lfs`）、LuaSocket（`socket.url`）与上面的
`android`。约定同第一代：`common/?.lua` 排在搜索路径最前而指向 koreader-base 的构建产物，四个代理都按「那里没有与
koreader 自己的模块同名的文件」判；测试运行器在同一个子模块里，工作目录按 `make/emulator.mk`、`.luacov` 与安装布局推定。

候选漏检 5 条（luarocks 3、koreader 2）：luarocks 三条是别的文件首行的 `compat53.module` 前言（`cmd/show.lua`、`cmd/list.lua`、
`fetch/hg_http.lua`，第一代七条里的三条；第二代宇宙已经读到它们，代理「只怕检测器漏掉受保护的写法」才记下，判分时按宇宙
核实即销）；koreader 两条与第一代相同——拼出来的 `dofile` 路径（`pluginloader.lua:244` 装每个插件的 `main.lua` / `_meta.lua`，
`llapp_main.lua:32` 拼 `android.dir .. "/reader.lua"`），不在字面实参的站点定义之内。`scope_gaps` 4 条全在 luarocks 的
命令行启动脚本 `src/bin/luarocks`（没有扩展名）：第 4 / 6 / 7 行的三处 `require` 与第 15 行起用纯字符串命名的命令模块表。
第一代另记的 `loader.lua` 局部别名 4 条、`cmd.lua:37` 的构建期模块 1 条与 `spore_spec.lua:67` 的疑似伪站点这次没有代理
再记（批次切法相同、看到的文件不同）；判分时两代的记录一并核。

### 判分（2026-09-25）

档 `contracts/eval/lang-precision-luarocks-v2.json` 与 `lang-precision-koreader-v2.json`，在 Lua 阶梯的提交 `642e919` 之上生成、
随下一个提交落地：顺序门要每份档的 `generated_from` 严格晚于审阅表的首个提交，而第二代的两张审阅表就是 `f473c39`，与阶梯
同一棵树上生成的档过不了它。两份档的 `generated_from` 记树 `642e919`、dirty = false。

| 语料 | 主样本 | correct | wrong | missed | external_ok | unresolved_ok | 精度 | 召回 |
|---|---|---|---|---|---|---|---|---|
| luarocks | 12 | 10 | 0 | 2 | 0 | 0 | 10/10 | 10/12 |
| koreader | 88 | 75 | 0 | 1 | 1 | 11 | 75/75 | 75/76 |
| 合计 | 100 | 85 | 0 | 3 | 1 | 11 | 85/85 = 1.000 | 85/88 |

按站点类：`require` 84 道 correct 69、wrong 0、missed 3；`load` 16 道 correct 16（全在 koreader，都答在 R2）。按级截断：只收
R1 的答案是 69 对 0 错，收到 R2 起 85 对 0 错。门是 M5-2 的 G2：整体与站内真值不少于 5 道的每个语料都不低于 0.90——两个语料
与整体都过。

- 三道 missed 都是搜索目录不在文本里的情形，阶梯按设计不猜：luarocks 两处 `compat53.module` 前言（`src/luarocks/build/cmake.lua:1`、
  `src/luarocks/fetch/cvs.lua:1`），真值 `vendor/compat53/module.lua`——`vendor/` 只由 `GNUmakefile` 写进包装脚本的 `LUA_PATH`
  （`src/?.lua;vendor/?.lua`），语料里没有哪个 Lua 文件把它写进 `package.path`；声明 `[graph.search_roots] lua = ["vendor"]`
  即答对。koreader `spec/unit/readersearch_spec.lua:7` 的 `require("commonrequire")`，真值 `spec/unit/commonrequire.lua`——
  `spec/unit/` 由测试运行器的配置加进路径，那份配置在未检出的 koreader-base 子模块里。
- 11 道 unresolved_ok 的真值都是 `external`，阶梯答 out_of_scope：koreader-base 提供的 `ffi/*` 与 `libs/libkoreader-lfs`、
  LuaSocket 的 `socket.url`、Android 启动器的 `android`；拒答不算错，也不进召回的分母。1 道 external_ok：`bit`（LuaJIT 内建，R3）。
- 候选漏检：luarocks 三条 compat53 前言（`cmd/show.lua`、`cmd/list.lua`、`fetch/hg_http.lua` 各第 1 行）检测器都读到了，阶梯同样
  答 out_of_scope——不是漏检，是上面两道 missed 的同类；koreader 两条（`pluginloader.lua:244`、`llapp_main.lua:32`）那一行没有
  字面实参的站点，检测器按设计不读。

宇宙台账（对冻结宇宙的每个站点都解一遍，解出率是召回的上限）：

| 语料 | 站点 | 解出 | R1 | R2 | R3（External） | 拒答 |
|---|---|---|---|---|---|---|
| luarocks | 702 | 79.8 % | 559 | 0 | 1 | 142，全是 out_of_scope |
| koreader | 5,058 | 84.1 % | 4,105 | 91 | 59 | 803，其中 2 个 ambiguous_root、其余 out_of_scope |

首级（R1）在解出里的占比 luarocks 99.8 %、koreader 96.5 %，过了 0.80 的触发线（RG1），两份档各带一条书面处置
（`r0_disposition`）：Lua 的第一级就是 `require` 的整个搜索（每个搜索目录与树里文件写的每条模板），而 `require` 是 luarocks
的全部站点、koreader 的 4,967 / 5,058，占比复述的是站点构成而不是某一级偏窄；拒答照计（20.2 % / 15.9 %），召回不怯
（10/12、75/76）。koreader 的模板读自 `setupkoenv.lua`（`common/?.lua;frontend/?.lua;plugins/exporter.koplugin/?.lua;`），
插件加载器用 `string.format` 拼的路径不算模板。

## R（步 4）

### 站点宇宙与抽样（2026-09-25 冻结）

范围 = `*.R` 与 `*.r`（产品路径表给 R 的两个扩展名），排除的只有其他扩展名：

| 语料 | tip | 许可 | 文件 | 站点 | library | source | 排除（其他扩展名） |
|---|---|---|---|---|---|---|---|
| tidyverse/stringr | `ae054b1` | MIT | 67 | 55 | 55 | 0 | 114 |
| ImperialCollegeLondon/covid19model | `fcc30e2` | MIT | 177 | 1,697 | 1,647 | 50 | 984 |

stringr 是包：包内的 R 文件之间不经任何站点互相引用（装载器把 `R/` 整体读入，设计册 D18），它的 `library` 站点
几乎都指向别的包（`pkg::name` 运算符也读作 `library`）；`source` 只有应用 covid19model 考得到。stringr 同时是交叉
核对语料（SOURCES.md 的 r 行）。

样本：主 100 道——library 84 / source 16；备用 40 道（两类各 20，都在 covid19model）；主样本落在 62 个文件上，按语料
分是 stringr library 2、covid19model library 82 + source 16。stringr 的 55 个站点只占 library 池的 55 / 1,702，按哈希秩
取到 2 道（期望 2.7）：抽样按站点人口、不设语料地板，这是预登记的规则；包的读法另由精度册的宇宙台账检验——冻结
宇宙里的每个站点都解一遍。

三份档的 `generated_from` 同 Lua。

### 真值（2026-09-25 冻结）

档 `contracts/eval/lang-review-stringr-v1.json` 与 `lang-review-covid19model-v1.json`，切批与读法同 Lua 一节。判词：
`source` 的路径相对工作目录——covid19model 的说明（README、Docker 说明、CI）都从仓库根运行脚本，`covid19AgeModel/` 之外
没有 `setwd` / `chdir`；`library` / `pkg::` 在语料里有 `DESCRIPTION` 声明同名包时答包目录（语料根是包时写 `.`），否则
`external`。100 道的 spec 全在记录的行上，零失配，没有动用备用题；没有 `ambiguous` / `dynamic` / `none`。

| 语料 | 主样本 | external | 文件 | 包目录 |
|---|---|---|---|---|
| stringr | 2 | 2 | 0 | 0 |
| covid19model | 98 | 80 | 15 | 3 |

covid19model 唯一的 `DESCRIPTION` 在 `covid19AgeModel/`（`Package: covid19AgeModel`），三道 `library(covid19AgeModel)`
答这个目录；80 道 external 里 79 道是 CRAN 包与 R 自带的 `parallel`，1 道是
`source("usa/code/utils/read-data-usa-2.r")`——语料里没有这个文件（调用在 `covid19AgeModel/inst/deprecated/` 的旧脚本
里），按词表「语料外的文件」判 `external`。两道 `source` 只在 `if (FULL)` 分支里执行，README 要求完整模式运行、分支
决定读不读而不决定读哪个，按文件判。stringr 两道是 `cli::` 与 `vctrs::`。候选漏检 6 条：样本没有抽到的字面 `source` 1 条与
`library(covid19AgeModel)` 2 条，经 `system("Rscript …")` 另起进程运行语料内脚本 3 条（不在两类站点之内，只作信息）。一个代理另用 R 4.6.1 自己的解析器（`parse` + `getParseData`）核对了它那批站点，照判词记下；
另一个代理曾在共用的批文件目录里写过两个辅助脚本（含它的判词），交卷前自行删除——每个代理的说明都只许读自己那一批。

门随本节加两处（`cli/tests/it/eval_lang_parts/review.rs`）：包目录真值不再只给按需导入，一张 `PACKAGE_KINDS` 表按站点类说
包的代码在目录下哪里——Java 的按需导入是直接装着该包冻结文件的目录，R 的包装载是包根、其 `R/` 下直接有冻结文件；候选
漏检分两栏，`site_gaps` 必须落在冻结文件上、`scope_gaps` 必须落在冻结宇宙之外（Java 的两张表早于这一栏，没有它）。
考题表 Lua、R 的 `audited` 翻为 true。

### 判分（2026-09-25）

档 `contracts/eval/lang-precision-stringr-v1.json` 与 `lang-precision-covid19model-v1.json`，在 R 阶梯的提交 `642e919` 之上生成、随下一个提交落地（顺序门同
Lua）；`generated_from` 记树 `642e919`、dirty = false。

| 语料 | 主样本 | correct | wrong | missed | external_ok | unresolved_ok | 精度 | 召回 |
|---|---|---|---|---|---|---|---|---|
| stringr | 2 | 0 | 0 | 0 | 2 | 0 | — | — |
| covid19model | 98 | 18 | 0 | 0 | 79 | 1 | 18/18 | 18/18 |
| 合计 | 100 | 18 | 0 | 0 | 81 | 1 | 18/18 = 1.000 | 18/18 |

按站点类：`library` 84 道 correct 3（三道 `library(covid19AgeModel)` 答包目录 `covid19AgeModel`，R2）、external_ok 81；`source`
16 道 correct 15（全在 R1）、unresolved_ok 1。stringr 的两道都是 `cli::` / `vctrs::`，站内真值为零，精度无定义——包的读法由宇宙
台账检验（下表）。1 道 unresolved_ok：`covid19AgeModel/inst/deprecated/R/foursquare_mobility_extend.R:4` 的
`source("usa/code/utils/read-data-usa-2.r")`，语料里没有这个文件，真值 `external`，阶梯答 out_of_scope——拒答不算错。门 G2：
整体 1.000，covid19model 18 道站内真值 1.000；stringr 不足 5 道不单独计。

- 候选漏检 6 条逐条核实：`nature/utils/make-table.r:9` 的 `source('nature/utils/format-data.r')` 检测器读到、阶梯 R1 答
  `nature/utils/format-data.r`；`covid19AgeModel/inst/scripts/post-processing-etas.R:18` 与
  `covid19AgeModel/inst/deprecated/ifr-by-age/ifr-by-age-stan.r:5` 的 `library(covid19AgeModel)` 都读到、答包目录；`base.r:124`、
  `base_general.r:288`、`web-fetch-and-run.r:7` 经 `system("Rscript …")` 另起进程，那一行没有 `source` / `library` 站点，按设计不读。

宇宙台账：

| 语料 | 站点 | 解出 | R1（source） | R2（包目录） | R3（External） | 拒答 |
|---|---|---|---|---|---|---|
| stringr | 55 | 100 % | 0 | 1 | 54 | 0 |
| covid19model | 1,697 | 99.8 % | 47 | 77 | 1,570 | 3，全是 out_of_scope |

stringr 的 R2 一道是 `tests/testthat.R:2` 的 `library(stringr)`，答包根 `.`；54 道 R3 是 CRAN 包与 base R。covid19model 的 77 道
R2 全指 `covid19AgeModel/`（唯一的 `DESCRIPTION`；`library(covid19AgeModel)` 60、`require(covid19AgeModel)` 16、
`covid19AgeModel::` 1）；3 道拒答是 `source` 指向语料里没有的文件。首级占比 stringr 0 %、covid19model 2.8 %，远低于 0.80，不需要
书面处置。

## 步 5：Java、Lua、R 的精度册退役待重判

步 5 的提交 A 挪动了这三门考题的答案所依赖的代码：走查不再按名字排除 `target/ build/ dist/`（只在旁边有产出它的工具的
项目文件时才算产物），Java 的 `main` 源集只看得见 `main` 源集、文件自己声明的名字是 `own_unit`，Lua 的 `require` 多找引用
文件自己的目录；判分也改为只问产品自己的走查读的文件（站点宇宙一节）。六份精度册因此在同一提交里删档、考题表的
`scored` 翻回 false；等 HTML 的阶梯落地后与 HTML 的精度册一起重生成一次——新的一代读数与每处答案的移动记在文末「B′」一节（2026-09-28）。

## HTML（步 5）

### 站点宇宙与抽样（2026-09-26 冻结）

- 语料三份（设计册 §11；第三份是用户的裁定）：本仓 `codeeraser`@`d4b7f1f`（收步 4 的那个提交：官网八页、GUI 页、两张 demo
  记分板，11 页 309 站点；`scripts/tsprobe/snippets/probe.html` 被走查的 `snippets/` 模式拒读，计 `walk_refused` 1）、
  `h5bp/html5-boilerplate`@`b659733`（`src/` 两页 6 站点；`dist/` 的两份构建产物旁边站着 `package.json`，走查拒读，计 2）、
  `mdn/learning-area`@`dbed6bc`（269 页 453 站点——前两份一个表单、一个 `srcset` 候选都没有，这份两样都有）。
- 站点（设计册 §8 HTML 行）：`href` 336、`src` 222、`link_asset` 197、`srcset` 7、`action` 6，共 768。
- 抽样：100 道主样本，`href` 34、`src` 27、`link_asset` 26，`srcset` 7 与 `action` 6 两类池不满地板、整类取尽；按语料
  learning-area 63、codeeraser 37、html5-boilerplate 0（6 个站点没有一个被秩选中）；备用题 60（`href`、`src`、`link_asset`
  各 20，另两类池已空）。档 `contracts/eval/lang-slice-{codeeraser,html5-boilerplate,learning-area}-v1.json`、
  `contracts/eval/lang-sample-html-v1.json`。

### 真值（2026-09-26 冻结）

档 `contracts/eval/lang-review-codeeraser-v1.json`（37 行）、`lang-review-learning-area-v1.json`（63 行）与
`lang-review-html5-boilerplate-v1.json`（0 行：它的 6 个站点没有一个被秩选中，没有代理读过它；表照样立档，因为已审阅的考题每个
语料都要有表〔`Exam::filed`〕，它的宇宙到判分时仍由台账整个解一遍）。切批与读法同 Lua 一节：四个独立 Opus 代理各判一批 25 道
主样本（按样本的审阅序切批），只读两个干净克隆（本仓 `d4b7f1f`、learning-area `dbed6bc`；派卷前查过两个副本都没有 `.ce/`）和
自己那一批，不跑 `ce`、不看产品的任何解析；装配逐字照录，判决不动。100 道的 spec 全在记录的行上，零失配，没有动用备用题。

**词表在 HTML 上的读法**（简报给的定义，代理照此判）：一个站点的真值 = 它的 URL 在该语料**部署出来的站点**上服务的文件——
相对值对文档 URL 解，根相对值对源根解，目录 URL 服务它的 `index.html`，`?query` 不入文件映射，`#frag` 只在目标页真有该 `id`
时带上（裸 `#` 解成本页、不带节）；别的源 = `external`；模板或脚本拼出来的值 = `dynamic`；解出的 URL 上服务不到文件 = `none`。
**部署从仓内证据读出**，不查线上：本仓 = Cloudflare Pages 以 `site/` 为源根（`scripts/deploy_site.js` 的 `pages deploy site`、
每页第 12 行的 `og:url`）；learning-area = GitHub Pages 项目站 `https://mdn.github.io/learning-area/` 一比一映射仓根（仓内三处
绝对 URL 指回语料自己的文件；根相对的 `/my-handling-form-page` 因此出了项目前缀，判 `external`）。

| 语料 | 主样本 | external | 文件（其中宇宙内的页面） | 节（`页#id`） | dynamic | none |
|---|---|---|---|---|---|---|
| codeeraser | 37 | 12 | 20（9） | 5 | 0 | 0 |
| learning-area | 63 | 9 | 51（5） | 1 | 1 | 1 |
| html5-boilerplate | 0 | 0 | 0 | 0 | 0 | 0 |

**真值可指向钉住树里的任何被追踪文件**：页面取的是站点服务的东西——样式表、图片、脚本、另一页——71 道文件真值里 57 道在
站点宇宙之外（`site/style.css`、`site/icon-256.png`，learning-area 的 `style.css` / `main.js` / `.jpg` / `.mp3` / `.svg`）。
审阅门为此多冻一份档 `contracts/eval/lang-tree-<语料>-v1.json`（仪器一节「钉住的树」），考题表的 `reach` 说这门考题的真值
够得到整棵树（Java / Lua / R 只够得到自己的宇宙）；树档与宇宙的排除计数逐类对得上，真值与 `scope_gaps` 必须落在树上。

21 道 external：Google Fonts 两个域 8（`fonts.googleapis.com` 7、`fonts.gstatic.com` 1）、GitHub 的 releases / blob 页 6、
`http://example.com` 表单 2、`/my-handling-form-page` 2、`developer.mozilla.org` / `studio.blender.org` / `thenounproject.com`
各 1。`dynamic` 1 = Flask 模板 `html/forms/sending-form-data/templates/form.html:28` 的 `action="{{ url_for('hello') }}"`
（渲染后是后端路由 `/hello`，静态副本里连花括号都是字面）；`none` 1 = `accessibility/assessment-finished/index.html:99` 的
`<a href="bear.mp3">`（这个目录下没有 `bear.mp3`，音频在 `media/bear.mp3`，同一段的 `<source>` 用的是后者）。6 道节真值全是
本文件里的 `#id`（`site/how/index.html#f04` / `#honesty`、`site/zh/how/index.html#f01` / `#acting` / `#honesty`、learning-area
的 `css/web-fonts/fonts/zantroke-demo.html#layout`），两道 `href="#"` 判成本页不带节。`srcset` 7 道的 `nth` 按物理行内的候选
序位判——多行 `srcset` 属性的续行上第一个候选是 nth 0。

**一次简报错误与它的修正**：简报把 `nth` 定义成「同类站点在行内的序位」，而样本的定义是「该行全部站点（不分类）按文档序的
0 起序位」（`graph/sites.rs`）。发现后四个代理各自按正确定义把 25 行重查一遍：只有一行受影响——`site/zh/how/index.html:31`
同一行有 `<a href="/zh/">` 与 `<img src="/icon-256.png">`，nth 1 是那个 `img`，代理原按同类序位判成 `mismatch`，重查后判
`site/icon-256.png`；其余 99 行每行只有一类站点，两种定义给同一个位置，判词不变。修正记在这里，也记在表的 `auditor` 字段里。

候选漏检：codeeraser 42 条（16 处不同行）全是 `<meta property="og:url" | "og:image" content="https://codeeraser.dev/…">`——
meta 的 `content` 不在站点表里（浏览器既不取也不导航到它，它是给第三方抓取器的元数据），判分时按协议逐条附检测器在该行读到的
站点；learning-area 8 条（6 处不同行）= `<style>` 里的 `url(header.jpg)`（三个批次各记了一次）与四处内联 `<script>` 里写死的
URL 字符串（`fetch()` / `new Request()` / `link.href =`）——内嵌脚本与样式是 `raw_text`、不解析（设计册 §1）；`scope_gaps` 5 条
在宇宙之外的文件上：`javascript/building-blocks/gallery/main.js:26`（脚本拼 `src`）、`javascript/apis/video-audio/finished/style.css:3`
（`@font-face src`）、`javascript/apis/fetching-data/can-store-xhr/can-style.css:24`（`url(icons/…)`）、
`tools-testing/cross-browser-testing/javascript/fetch-broken/script.js:5`（脚本里的 `requestURL`）、
`html/forms/sending-form-data/python-example.py:8`（Flask 路由）。代理另记的两条约定：`</body>` 之后的 `<script>` 仍被解析器
插进 body 取回；`<audio>` / `<video>` 回退内容里的 `<a>` 是真实 DOM 元素。

门随本节加三处（子仓）：`it/eval_lang_parts/tree.rs` = 树档的生成器（`--ignored lang_tree`）与核对（信封、路径严格升序、宇宙文件
全在树上、其余按宇宙自己的排除计数逐类对上），`it/eval_lang.rs` 两腿（每份树档核对；删一个宇宙文件、塞一页、调换两行、改方法句、
换 tip 五种篡改各拒）；`review.rs` 的真值与 `scope_gaps` 改绑 `targets`（够得到树的考题绑树，其余绑宇宙），`it/eval_lang_review.rs`
新腿 `a_truth_beyond_the_tree_is_refused`（宇宙外、树内的真值通过，树外的拒），「包目录真值」的探针改读核对器自己的分类（一条图片
路径不再被当成包）；精度册生成器有克隆时把树重导一遍（`assert_frozen_tree`）。考题表的两个布尔旗（`audited` / `scored`）合成一个
有序的 `Stage`（sampled → audited → scored，「已判分 ⇒ 已审阅」由构造保证），HTML 的 stage 翻为 audited。

### 阶梯（提交 B，2026-09-26）

`cli/src/graph/ladder/html.rs` 与 `html_head.rs`（设计册 §8 HTML 行是权威，这里只记与考题有关的三件事）。**候选集**：一页指向的是
站点服务的东西，所以目标是走查读到的任何文件——被判决的页面 / 文档 / 代码，加走查读到而索引不持有的**资产**（`WalkIndex::assets`，
与每页的 `id` 集一起进 `resolve_key`，资产增删即全量重扫）；判分的 `Scope` 照此拼（`score::tree`）。**部署根**：根相对的 `/x`
先问 `[graph.search_roots] html`，无声明则由页面自己的服务 URL 推出——canonical、`og:url`、本页语言的 hreflang alternate
（本仓每页第 12 行的 `og:url` 正是审阅简报读部署的证据；learning-area 的页面没有这些，靠祖先目录推断：`/x` 在页面的哪个祖先
目录下恰好存在）。**空值**按元素的取回算法分读：`href` / `action` 的空值是本页（空 URL 即文档自身），`src` / `srcset` /
`link_asset` 的空值什么也不取（HTML 的 img / script / link 算法遇空值即返回），留台账行 `empty`。审阅简报按 URL 解析定义
真值，于是 learning-area 有一道 `link_asset` 空值（`tools-testing/cross-browser-testing/javascript/fetch-polyfill-finished.html:9`）
的真值是本页；产品按取回算法答 `empty`，这一道按冻结真值记 `missed`——真值不改（审阅者按简报判得对），分歧记在这里。
**走查拒读的资产**：learning-area 一道 `src` 的真值 `javascript/apis/drawing-graphics/threejs-video-cube/three.min.js`
落在内建排除 `*.min.js` 上（`scan/walk.rs`），产品没有它的结点；判分对这种真值的读法与拒读的宇宙文件相同——按站外判分、原话另记
（`walk.unreached`，本节上文「判分」一条）。精度册与读数随 B′ 在干净的树上生成，记在下一节。

## B′：九份精度册在 5b 收口后的树上生成（2026-09-28）

档 `contracts/eval/lang-precision-{gson,jsoup,stringr,covid19model,codeeraser,html5-boilerplate,learning-area}-v1.json` 与
`lang-precision-{luarocks,koreader}-v2.json`，九份都在 `92ed92e`（步 5b 的最后一个提交，5b-6）之上生成、`dirty = false`：生成器把
`dirty` 读成 `git status --porcelain` 非空，同批先生成的档会让后生成的档读成脏的，所以每份档单独生成在一棵没有别的档的树上、生成完
挪出树、九份齐了再放回；每张审阅表的首个提交都是 `92ed92e` 的严格祖先（顺序门 `lang_provenance`）。Lua 两份档先用临时处置生成一遍
取数、删档、再用引用了新数字的最终处置生成（RG1 处置要引用档自己的数字，而它在生成前不存在）。考题表四门 stage 全翻为 `scored`。

### 读数

| 语料 | 主样本 | correct | wrong | missed | external_ok | unresolved_ok | 精度 | 召回 |
|---|---|---|---|---|---|---|---|---|
| gson | 38 | 12 | 0 | 0 | 17 | 9 | 12/12 | 12/12 |
| jsoup | 62 | 24 | 0 | 0 | 19 | 19 | 24/24 | 24/24 |
| luarocks | 12 | 10 | 0 | 0 | 0 | 2 | 10/10 | 10/10 |
| koreader | 88 | 76 | 0 | 0 | 1 | 11 | 76/76 | 76/76 |
| stringr | 2 | 0 | 0 | 0 | 2 | 0 | — | — |
| covid19model | 98 | 18 | 0 | 0 | 79 | 1 | 18/18 | 18/18 |
| codeeraser | 37 | 25 | 0 | 0 | 12 | 0 | 25/25 | 25/25 |
| html5-boilerplate | 0 | 0 | 0 | 0 | 0 | 0 | — | — |
| learning-area | 63 | 50 | 0 | 1 | 7 | 5 | 50/50 | 50/51 |

门 G2（整体与站内真值不少于 5 道的每个语料都不低于 0.90）：Java 36/36、Lua 86/86、R 18/18、HTML 75/75 = 1.000，全过；召回
Java 36/36、Lua 86/86、R 18/18、HTML 75/76。

**Java / Lua / R 对上一代的移动**（gson、stringr、covid19model 三份的 rows 逐行同上一代；移动全出自步 5 提交 A 的三处根修与「只问走查
读的文件」的判分口径，阶梯本身在 B′ 之前没有为哪一道改过）：

- jsoup 22/23 → 24/24：上一代唯一的 wrong（`HtmlTreeBuilderState.java:22` 的 `import static …HtmlTreeBuilderState.Constants.*`，本文件
  自己的嵌套类）现由阶梯答 `own_unit`（文件自己声明的名字不是别的文件的引用），真值 `none`，按拒答计 unresolved_ok；两道 missed（测试根的
  `TokeniserTest.java:4` / `HtmlParserTest.java:7` 的 `import org.jsoup.nodes.*`）现答 correct——Java 源集规则（`java_sets.rs`）让测试根的
  导入者在自己的源集里先找，拆分包不再 ambiguous_root。宇宙台账随之：解出 86.7 % → 86.8 %（`import` R2 136 → 134、`import_star` R2
  10 → 12、`type_ref` R3 11,659 → 11,682），拒答 3,226 → 3,203（`ambiguous_root` 3 → 0、新增 `own_unit` 3）。
- luarocks 10/12 → 10/10：上一代两道 missed 的真值 `vendor/compat53/module.lua` 落在走查按名排除的 `vendor/`（`scan/walk.rs` 内建排除），
  判分按「只问走查读的文件」把真值记作站外（`audit_truth` 保留审阅原话）、拒答计 unresolved_ok；`vendor/` 的 5 个文件 10 个站点进
  `walk.refused`（上一代里 9 道 out_of_scope 与那一道 R3 标准库名），台账 702 → 692 站点、解出 79.8 % → 80.8 %。上一代写的「声明
  `[graph.search_roots] lua = ["vendor"]` 即答对」在这个口径下不再成立：走查不读的文件没有结点，阶梯答不到它。
- koreader 75/76 → 76/76：上一代唯一的 missed（`spec/unit/readersearch_spec.lua:7` 的 `require("commonrequire")`）现由「引用文件自己的
  目录」答出；宇宙 R1 4,105 → 4,231、out_of_scope 801 → 675、解出 84.1 % → 86.6 %。两份 Lua 档的 `r0_disposition` 按新数字重写（首级
  占比 luarocks 100 %、koreader 96.6 %，仍过 0.80 的触发线）。

### HTML 判分

按站点类：`href` 34 道 correct 24、external_ok 9、unresolved_ok 1（`页#id` 与根相对全答在 R2、裸 `#` 在 R4）；`src` 27 道 correct 25、
external_ok 1、unresolved_ok 1；`link_asset` 26 道 correct 18、external_ok 7、missed 1；`srcset` 7 道 correct 7；`action` 6 道 correct 1、
external_ok 2、unresolved_ok 3。按级截断：codeeraser 只收 R1 是 1 对 0 错、收到 R2 是 20 对、收到 R4 起 25 对；learning-area 只收 R1 是
45 对 0 错、收到 R4 起 50 对。门 G2 两个有站内真值的语料与整体都过。

- 唯一的 missed 是阶梯一节预告的那一道：learning-area `tools-testing/cross-browser-testing/javascript/fetch-polyfill-finished.html:9` 的空
  `link_asset`，真值按简报的 URL 解析定义是本页，产品按取回算法答 `empty`；真值不改、阶梯不为它改。
- 5 道 unresolved_ok：`javascript/apis/drawing-graphics/threejs-video-cube/index.html:9` 的 `src="three.min.js"`（真值是走查内建排除
  `*.min.js` 上的文件，按站外判分，在 `walk.unreached`）、两道 `action="/my-handling-form-page"`（真值 external：GitHub Pages 项目站的
  根相对路径出了项目前缀）、`accessibility/assessment-finished/index.html:99` 的 `href="bear.mp3"`（真值 `none`）、Flask 模板
  `html/forms/sending-form-data/templates/form.html:28` 的 `action="{{ url_for('hello') }}"`（真值 `dynamic`）；阶梯都答 out_of_scope，拒答
  不算错。
- 候选漏检：codeeraser 42 条（16 处不同行）检测器在那些行上一个站点都没读到——全是 `<meta property="og:url" | "og:image">` 的
  `content`，不在站点表里；learning-area 8 条（6 处不同行）同样零站点——内联 `<style>` 与 `<script>` 是 `raw_text`。都不是漏检。

宇宙台账（HTML 的级：R1 相对文档目录、R2 根相对经声明根或页面自己的服务 URL〔`页#id` 也记在 R2〕、R3 根相对经祖先目录、R4 裸片段、
R5 External）：

| 语料 | 站点 | 解出 | R1 | R2 | R3 | R4 | R5（External） | 拒答 |
|---|---|---|---|---|---|---|---|---|
| codeeraser | 309 | 100 % | 19 | 146 | 0 | 46 | 98 | 0 |
| html5-boilerplate | 6 | 83.3 % | 3 | 0 | 2 | 0 | 0 | 1，out_of_scope |
| learning-area | 453 | 94.7 % | 320 | 0 | 0 | 46 | 63 | 24 = out_of_scope 17 + `empty` 7 |

首级占比 codeeraser 6.1 %、html5-boilerplate 60 %、learning-area 74.6 %，都在 0.80 之下，不需要书面处置。走查记录：codeeraser
`unreached` 75 条（`.github/`、`cli/tests` 子模块、锁文件、对拍夹具等树内、宇宙外、走查不读的路径）、html5-boilerplate 51 条、learning-area
12 条（含两份 `three.min.js`、两份 `html5shiv*.min.js`）；三份的 `refused` 都为空。

Java / Lua / R 的宇宙台账（本代）：

| 语料 | 站点 | 解出 | 拒答 |
|---|---|---|---|
| gson | 22,698 | 87.0 % | 2,945，全是 out_of_scope |
| jsoup | 24,210 | 86.8 % | 3,203（out_of_scope 3,200、own_unit 3） |
| luarocks | 692 | 80.8 % | 133，全是 out_of_scope |
| koreader | 5,058 | 86.6 % | 677（out_of_scope 675、ambiguous_root 2） |
| stringr | 55 | 100 % | 0 |
| covid19model | 1,697 | 99.8 % | 3，全是 out_of_scope |

## C 与 C++（步 6）

### 站点宇宙与抽样（2026-09-28 冻结）

两门的三件套在步 6 补做（设计册 §14 第 14 条：阶梯已随步 2 提交 b7e78c7，先于任何考题；顺序门对这两门改核这一反转本身，
见「仪器与门」的顺序门一条；考题表两行的 `ladder_first` 记反转与理由）。每门一个语料，就是交叉核对语料（SOURCES.md 的 c / cpp
行，五个夹具与宇宙同一 tip、逐字节相同）；站点类只有 `include`；`.h` 按产品扩展名表是 C++，故 Lua 解释器的 28 个头文件不在 C 的
宇宙里，fmt 的头文件、源文件与测试同属一个宇宙。阶梯只读 C 族的 `include` 站点（含者同目录 → 声明根 → 编译数据库 → 站外），
两个语料都没有 `compile_commands.json`，也没有 ce.toml。

| 语料 | 语言（范围） | tip | 许可 | 文件 | 站点（`include`） | 排除（其他扩展名） |
|---|---|---|---|---|---|---|
| lua/lua | C（`*.c`） | `0b29f40` | MIT | 40 | 476 | 71（`.lua` 34、`.h` 28、其他 9） |
| fmtlib/fmt | C++（`*.cpp *.cc *.cxx *.hpp *.hh *.hxx *.h *.inl`） | `6d71f74` | MIT | 73 | 740 | 72 |

样本：每门主 100 道（一类，池 476 / 740，`min_per_kind` 不起作用）、备用 20 道。C 的主样本落在 34 个文件上，引号形 75 /
尖括号形 25（`ltests.c` 11 道、`onelua.c` 9 道）；C++ 的落在 37 个文件上，引号形 19 / 尖括号形 81——fmt 的站点宇宙里标准库头
占多数，`test/gtest/gmock-gtest-all.cc` 一个文件 22 道、`test/gtest/gtest/gtest.h` 14 道。spec 按检测器的读法：引号形去引号、
尖括号形留尖括号（阶梯的搜索序由此区分）。

四份档的 `generated_from` 记 ce 1.7.4、树 `37d9772`、dirty = true：冻结时考题表的两行本身还没提交（它和这四份档在同一个
提交里落地），与 Java 那三份同理。

### 真值（2026-09-28 冻结）

档 `contracts/eval/lang-review-lua-v1.json`（C，100 行）与 `lang-review-fmt-v1.json`（C++，100 行）。切批与读法同 Java 一节：
每门 100 道主样本按审阅序切成四批，每批 25 道交给一个独立的 Opus 代理；代理只读一个语料在钉住 tip 的干净克隆（lua/lua `0b29f40`、
fmtlib/fmt `6d71f74`）和自己那一批，不跑 `ce`、不看产品的任何解析；装配逐字照录，判决不动。200 道的 spec 全在记录的行上，
零失配，没有动用备用题。

**词表在 `include` 上的读法**（简报给的定义，代理照此判）：真值 = 该语料**自己的构建**让预处理器找到的那个被追踪文件——引号形先在
引用文件的目录找，再按构建给编译该文件的目标声明的包含目录（Makefile 的 `-I`、CMake 的 `target_include_directories`）依次找，
再走尖括号的搜索；尖括号形只走声明目录再到系统（C11 6.10.2）；路径按 `git ls-files` 的写法；树里没有、是标准库 / 系统 / 未 vendor
的第三方头 = `external`；树里没有、也不是系统头（如构建生成的头）= `none`；两个不同的被追踪文件都能命中 = `ambiguous`；宏作操作数
= `dynamic`。

| 语料 | 主样本 | external | 文件 | none | ambiguous / dynamic |
|---|---|---|---|---|---|
| lua/lua（C） | 100 | 25（全是尖括号形） | 74（`.h` 68、`.c` 6） | 1 | 0 |
| fmtlib/fmt（C++） | 100 | 83（尖括号形 80、引号形 3 = 未 vendor 的 absl 头） | 17（`include/fmt/` 13、`test/gtest/` 3、`test/fuzzing/` 1） | 0 | 0 |

**C 的真值够得到钉住的树**：`.h` 按产品扩展名表是 C++，不在 C 的 `*.c` 宇宙里，而 74 道文件真值里 68 道正是解释器的头文件；
考题表的 C 行因此改 `reach: Tree`（与 HTML 同），多冻一份 `contracts/eval/lang-tree-lua-v1.json`（111 条路径，钉住 tip 的
`git ls-tree`），真值与 `scope_gaps` 必须落在树上。C++ 的八个扩展名把 fmt 的头文件、源文件与测试收进同一个宇宙，17 道文件真值全在
宇宙内，`reach` 仍是宇宙。

**真值对阶梯的含义**（判分前的观察，不是判分）：C 的 74 道文件真值里 73 道由引用文件自己的目录答出（根目录的 `.c` 引根目录的
`.h`；根 makefile 一个 `-I` 也没声明），只有 `testes/libs/lib2.c:2` 的 `lauxlib.h` 靠 `testes/libs/makefile:8` 的
`-I$(LUA_DIR)`（`../../`）——阶梯今天没有这一级（语料无 `compile_commands.json`、无 ce.toml），答 out_of_scope；唯一的 `none`
是 `onelua.c:135` 在 `#ifdef MAKE_LUAC` 下引的 `luac.c`，仓库树里没有这个文件。C++ 的 17 道里只有 4 道由引用文件自己的目录答出，
10 道引号形 `fmt/*.h` 与 1 道尖括号形 `<fmt/chrono.h>` 要 `include/`（`CMakeLists.txt` 的 `setup_target` 给库目标、`test-main` 的
PUBLIC 目录给每个测试目标声明的唯一包含目录），2 道 `gtest/gtest.h` / `gmock/gmock.h` 要 `test/gtest`（`test/gtest/CMakeLists.txt`
的 SYSTEM 包含目录）——这 13 道里 12 道引号形今天答 out_of_scope，那 1 道尖括号形今天答 external、与真值相左。判分前阶梯要不要
按约定读 `include/`，由下一个提交定；本节只记真值。

**候选漏检**：C 的四个代理里三个各报了同两处 `lvm.c:1211`（`#if LUA_USE_JUMPTABLE` 下、函数体内的 `#include "ljumptab.h"`）与
`lvm.c:1228`（`#if 0` 块里缩进的 `#include "lopnames.h"`），落 `site_gaps`（带批号，6 条）；第三批另报 `lua.h:150` 的
`#include LUA_USER_H`（宏作操作数）——`lua.h` 不在 C 的宇宙里而在树上，按门的两侧规则落 `scope_gaps`。C++ 零漏检。判分时逐条核实：
检测器读不读函数体内与 `#if 0` 下的 include，是判分要回答的问题。

**代理另记的语料约定**（`notes`，逐条带批号）：根 makefile 编译每个根级 `.c`（`CORE_O / LIB_O / LUA_O`），不声明任何包含目录，
引号形一律在引用者自己的目录解出；`testes/libs/makefile` 自己声明 `-I`；fmt 的每个库目标只有 `include/` 一个声明目录
（`setup_target`），每个 `add_fmt_test` 目标经 `test-main` 的 PUBLIC 目录到 `include/`、经 gtest 的 SYSTEM 目录到 `test/gtest`。

### 阶梯（提交 B′，2026-09-28）

`cli/src/graph/ladder/c.rs` 加第四级（设计册 §8 C/C++ 行是权威，这里只记与考题有关的部分）。**为什么加**：上一小节记下 fmt 17 道
文件真值里 11 道要 `include/`，阶梯对其中 12 道引号形答 out_of_scope、对 `<fmt/chrono.h>` 答 external（判分会记一道 `wrong`）。
`include/` 不是 fmt 一家的写法：构建用 `-I include` 让树内的 `#include <fmt/chrono.h>` 与装好之后 `$(includedir)` 下的拼法一致，
头文件才能在树内与装机后用同一句话引用；按约定读它是 join 一个固定的目录名，不是按 basename 搜树。**加的是什么**：R4 = 引用文件自己的
目录及其每级祖先（含仓根）旁的 `include/` 与名字 join，两种形都问，只问没有任何编译链到达的文件——树带了 `compile_commands.json` /
`compile_flags.txt` 就由它说了算，构建找不到的名字就是找不到；两个祖先的 `include/` 各持一个文件 = ambiguous_root（哪个 `-I` 在前是
构建的事，树上没有这个事实）；External 从 R4 挪到 R5；`graph/store.rs` 的 `GRAPH_REV` 21 → 22，已建的索引（考题跑在里面的语料克隆）
把 include 边与级号重算一次。**仍然不读构建脚本**：`test/gtest/CMakeLists.txt` 给 gtest 的 SYSTEM 目录、`testes/libs/makefile` 的
`-I$(LUA_DIR)`，都要经编译数据库（R3）或声明根（R2）到阶梯——读 Makefile / CMake 是第二个构建系统读者，变量与生成器表达式让它永远
只对一半，而编译数据库是每个构建系统都会吐出的同一份机器格式。**按真值表推算**（判分前的推算，读数以提交 C 的精度册为准）：lua 74 道
文件真值 73 道仍由 R1 答出，`testes/libs/lib2.c:2` 仍 out_of_scope（lua 的树没有 `include/`）；fmt 17 道 = R1 4 + R4 11，
`test/no-builtin-types-test.cc:8` 与 `test/core-test.cc:29` 的 gtest 头仍 out_of_scope（`test/gtest` 不叫 `include`，也没人声明它：
在带 ce.toml 的树上 `[graph.search_roots] c = ["test/gtest"]` 答得出，考题判的是钉住的树本身）。**顺序门**：阶梯改动落在两张审阅表的
提交（dea7914f）之后，`lang_provenance` 核的盲窗（抽样 515071b → 审阅表）里阶梯零提交的事实不变；C / C++ 两行仍是 `Audited`，
精度册随提交 C 在这个提交之后的干净树上生成。

### 判分（提交 C，2026-09-28）

档 `contracts/eval/lang-precision-lua-v1.json` 与 `lang-precision-fmt-v1.json`，在阶梯提交 B′（`95521640`）的干净树上逐份生成、随下一个
提交落地：生成器把 `dirty` 读成 `git status --porcelain` 非空，先生成的档会让后生成的读成脏的，所以每份单独生成在没有别的档的树上、
生成完挪出、两份齐了再放回；两份档的 `generated_from` 都记树 `95521640`、dirty = false。顺序门（`lang_provenance`）：两张审阅表的提交
dea7914f 与阶梯提交 95521640 都在它之前或就是它，阶梯在盲窗（抽样 515071b → 审阅表 dea7914f）内零提交的事实不变。

| 语料 | 主样本 | correct | wrong | missed | external_ok | unresolved_ok | 精度 | 召回 |
|---|---|---|---|---|---|---|---|---|
| lua（C） | 100 | 73 | 0 | 1 | 25 | 1 | 73/73 | 73/74 |
| fmt（C++） | 100 | 15 | 0 | 2 | 80 | 3 | 15/15 | 15/17 |
| 合计 | 200 | 88 | 0 | 3 | 105 | 4 | 88/88 = 1.000 | 88/91 |

按级截断：lua 的 73 道全在 R1；fmt 收到 R3 为止 4 对 0 错，收到 R4 起 15 对 0 错——`include/` 级答出的正是真值一节点名的那 11 道，
上一小节按真值表的推算一道不差。门是 M5-2 的 G2：整体与站内真值不少于 5 道的每个语料都不低于 0.90——两个语料与整体都过。

- lua 唯一的 missed：`testes/libs/lib2.c:2` 的 `lauxlib.h`，真值在仓根；引用者在 `testes/libs/`，树里没有 `include/`、没人声明根、也没有
  编译数据库，`testes/libs/makefile` 的 `-I$(LUA_DIR)` 只有经这两条路才到阶梯，阶梯按设计不猜。unresolved_ok：`onelua.c:135` 的 `luac.c`
  （`#ifdef MAKE_LUAC` 之下），真值 none——树里没有这个文件。25 道 external_ok 全是尖括号形的标准库头（R5）。
- fmt 两道 missed：`test/no-builtin-types-test.cc:8` 的 `gtest/gtest.h` 与 `test/core-test.cc:29` 的 `gmock/gmock.h`，真值在 `test/gtest/`——
  `test/gtest/CMakeLists.txt` 把它声明成 gtest 目标的 SYSTEM 目录，一个不叫 `include`、也没人在 ce.toml 里声明的目录（带 ce.toml 的树上
  `[graph.search_roots] c = ["test/gtest"]` 答得出，考题判的是钉住的树本身）。三道 unresolved_ok 是 gtest 源码里引号形的 absl 头
  （`test/gtest/gtest/gtest.h:2527` / `:2585`、`test/gtest/gmock-gtest-all.cc:1622`），真值 external，阶梯答 out_of_scope 不算错、也不进召回的
  分母。80 道 external_ok 是尖括号形的标准库与 absl 头（R5）。
- 候选漏检：C 审阅表的六行（三个代理各报 `lvm.c:1211` 与 `:1228`——函数体内 `#if LUA_USE_JUMPTABLE` 之下的 include 与 `#if 0` 之下缩进的
  include）检测器都读到了并在 R1 答出 `ljumptab.h` / `lopnames.h`，不是漏检；范围外的那一行（`lua.h:150` 的宏操作数 `#include LUA_USER_H`，
  `.h` 不在 `*.c` 宇宙里）只在审阅表的 `scope_gaps` 里；C++ 零。走查：lua 树档 111 条里只有 `.gitignore` 没被走到（点文件），fmt 零。

宇宙台账（对冻结宇宙的每个站点都解一遍，解出率是召回的上限）：

| 语料 | 站点 | 解出 | R1 | R4（`include/`） | R5（External） | 拒答 |
|---|---|---|---|---|---|---|
| lua | 476 | 98.1 % | 333 | 0 | 134 | 9，全是 out_of_scope |
| fmt | 740 | 96.2 % | 60 | 89 | 563 | 28，全是 out_of_scope |

首级（R1）在解出里的占比 lua 71.3 %、fmt 8.4 %，都不过 0.80 的触发线（RG1），两份档不带处置。拒答逐条有名（按站点表与树档推得，与档里的
计数逐格相符）：lua 9 = `testes/libs/` 五个测试库的 `lua.h` × 5 与 `lauxlib.h` × 3（真值都在仓根，与上面那道 missed 同类）+ `onelua.c:135` 的
`luac.c`；fmt 28 = 20 处引号形的 `gtest/gtest.h` / `gmock/gmock.h`（`test/*.cc`、`test/*.h`，以及 gtest 自己的 `gmock/gmock.h:303` 与
`gtest/gtest-spi.h:39`，真值都在 `test/gtest/`，与上面两道 missed 同类）+ 8 处引号形的 absl 头（`gtest.h` 四处、`gmock-gtest-all.cc` 四处，
未内置于树、真值 external）。两类都是构建脚本声明的目录：经编译数据库（R3）或 `[graph.search_roots]`（R2）到阶梯，设计册 §8 的立场不变。
