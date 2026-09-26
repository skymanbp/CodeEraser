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
  逐键比对；`docs/RELEASE.md` §0）。

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
`scored` 翻回 false；等 HTML 的阶梯落地后与 HTML 的精度册一起重生成一次——新的一代读数与每处答案的移动在那时记在本节。

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
