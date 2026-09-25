# v2.30 语言精度考题册（第四次拆册，2026-09-24）

> 计划 v2.30 设计册 [reference/language-expansion.md](reference/language-expansion.md) §11 与 §14 第 14 条的冻结登记。
> 每个新语言的图精度照 M5-2 的仪器优先脊柱走三步，顺序由提交先后证明：**站点宇宙 + 抽样冻结并提交 →
> 没看过解析结果的独立代理逐站判真值（GT）并提交 → 该语言的阶梯与精度册提交**（顺序门
> `cli/tests/it/lang_provenance.rs`）。母册链：[EVAL-SET.md](EVAL-SET.md) → [EVAL-SET-M5-3.md](EVAL-SET-M5-3.md) →
> [EVAL-SET-M5-CLOSE.md](EVAL-SET-M5-CLOSE.md) → [EVAL-SET-SIMILAR.md](EVAL-SET-SIMILAR.md) → 本册。本册与前四册同入
> 冻结集（`frozen_set.rs`：不扫芯片、不生成、退出引文门），行号引文一律不写。一个语言在它自己的步里加一节，
> 三步各记一段；重冻结 = 换 tip 并在本册具名记一条，生成器拒绝覆写已冻结的档。

## 仪器与门

- **站点宇宙**（`cli/tests/it/eval_lang_parts/generate.rs` 的 `lang_slice`，`#[ignore]`，读 `.ce-eval/corpora/<名>`
  的钉住克隆，别的树按名拒绝）：按考题表 `eval_lang_parts::EXAMS` 的扩展名走一遍钉住的树，逐文件记检测器看到的
  文本 sha256 与各站点类的计数（`graph::sites`——只读文法 kind 表与文件内事实，不查任何路径，所以能先于阶梯冻结）。
  档 `contracts/eval/lang-slice-<语料>-v1.json`。
- **抽样**（同文件 `lang_sample`）：一个语言的全部冻结宇宙合成一个池，逐文件先复现它的冻结行再取站点——池等于
  冻结宇宙靠核对、不靠信任。秩 = `sha256(域|corpus|commit|path|line|nth|kind|spec)`（M5-2 的载荷顺序，spec 居末
  保单射）；每个站点类先取 min(15, 该类的池)，剩下的座位按各类剩余池的最大余数分满 100；主样本按审阅域哈希排列
  （审阅者看不到秩序）；每类另留 min(20, 池 − 配额) 道备用题，只在同类主样本无法作答时按序顶上（补分母不跨类，
  护住地板）。档 `contracts/eval/lang-sample-<语言>-v1.json`。
- **CI 门**（`cli/tests/it/eval_lang.rs`，不跑 git、不要语料克隆）：冻结集恰为考题语料；每份宇宙过共用信封、钉
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
  阶梯的答案。生成器冻结前先跑 CI 的核对。档 `contracts/eval/lang-precision-<语料>-v1.json`；它是冻结档里唯一依赖产品代码的：
  阶梯的改动挪动了答案就重判——删档、重生成、在本册具名记一条。CI 门（`cli/tests/it/eval_lang_precision.rs`，不跑 git、不要
  克隆）：冻结集 = 已判分考题的语料（考题表的 `scored` 旗标与盘上的档逐语料相符，判分前必须已审阅）；每行按样本顺序回显身份
  与审阅真值、答案只能是三种形状之一、判词从行本身重算；摘要从行重算；台账的比率从两张计数表重算、逐类合计等于冻结宇宙；
  首级占比过触发线须带书面处置；候选漏检与审阅表逐条对应；精度不低于 0.90（整体，与站内真值不少于 5 道的每个语料；M5-2 的
  G2）。篡改（伪路径、伪 spec、伪真值、伪判词、非布尔的 external、缺一行、调换两行、翻转答案、改摘要、改台账计数、改比率、
  挪候选漏检的行号）一律拒绝，首级占比的触发线两个方向各有一腿。三份冻结档（样本、审阅表、精度册）的篡改电池共用一个框架
  （`eval_lang_parts/tamper.rs`）。
- **顺序门**（`cli/tests/it/lang_provenance.rs`，要完整 git 历史）：审阅表未提交时，任何提交都不得碰过该语言的
  阶梯路径，精度册不得存在；审阅表提交后，样本 ≺ 每张审阅表 ≺ 每份精度册，阶梯的首个提交严格晚于每张审阅表，
  且样本到审阅之间的盲窗里没有任何 `cli/src/graph` 文件落地（与 `graph_provenance.rs` 共用两层绊线）。

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
