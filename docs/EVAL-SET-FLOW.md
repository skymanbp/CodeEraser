# v2.31 函数内死代码精度考题册（第五次拆册，2026-09-30）

> 计划 v2.31 设计册 [reference/analysis-track.md](reference/analysis-track.md) §5.5「精度考题」段与 §13 第 16 条的冻结登记。
> 每个语言的 `flow/1` 精度照 v2.30 语言考题的仪器走，顺序由提交先后证明，只是抽样之前多一步：**降表与语言表先落（提交 A）→
> 单元宇宙 + 候选池 + 抽样冻结并提交（提交 B，本册第一节）→ 没看过判决的独立代理逐题盲判并提交 → 精度册提交**（顺序门
> `cli/tests/it/flow_provenance.rs`；题是从降出的四表里按类按层抽的，降表不先落地就没有池可抽，所以是 C / C++ 阶梯先于考题的那种
> `ladder_first` 形——盲判的独立性靠题不带答案、盲窗内 `cli/src/flow`（`mod.rs` 除外）零提交、精度册钉在回答它的代码上）。母册链：
> [EVAL-SET.md](EVAL-SET.md) → [EVAL-SET-M5-3.md](EVAL-SET-M5-3.md) → [EVAL-SET-M5-CLOSE.md](EVAL-SET-M5-CLOSE.md) →
> [EVAL-SET-SIMILAR.md](EVAL-SET-SIMILAR.md) → [EVAL-SET-LANGS.md](EVAL-SET-LANGS.md) → 本册。本册与它的第一代归档册 [EVAL-SET-FLOW-GEN1.md](EVAL-SET-FLOW-GEN1.md)、前五册同入冻结集
> （`frozen_set.rs`：不扫芯片、不生成、退出引文门），行号引文一律不写。一个语言在它自己的步里加一节，三步各记一段；
> 重冻结 = 该语言考题的代数加一（考题表的 `generation`：降表多读了一种写法、或换 tip），新一代用新文件名，旧一代的宇宙 /
> 样本 / 审阅档留在盘上作那一代的记录（门只读考题表指向的代），本册在该语言下具名加一节「第二代」，该门第一代的各节逐字节搬进归档册、主册该门只留一行指向（F′）；只有一代的门，退役的第一代精度节同样搬进归档册、留一行指向（G）；生成器拒绝覆写已冻结的档。

## 仪器与门

- **单元宇宙**（`cli/tests/it/eval_flow_parts/generate.rs` 的 `flow_slice`，`#[ignore]`，读 `.ce-eval/corpora/<名>` 的钉住克隆，
  别的树按名拒绝）：按考题表 `eval_flow_parts::EXAMS` 的扩展名走一遍钉住的树，只取产品自己的走查读的文件（`scan::walk::Scope`，
  语料自己的 ignore 文件与内建排除照读；走查拒读的被追踪文件只计 `walk_refused`，不出题），逐文件把 `scan::functions::extract`
  抽出的每个单元经 `flow::lower::lower_file` 降成四表，记文本 sha256、每个单元的序号 / 名 / 行段 / 形参数 / `dynamic` 位 /
  三张表的行数 / 每个（类，层）格的候选池大小，降不出合法树形的单元按名记 `unlowered` 与原因。池只读降出的表与图例，不问核，
  所以能先于任何判决冻结。档 `contracts/eval/flow-slice-<语言>-<语料>-v<代>.json`（zod 在 TypeScript 与 TSX 两门考题里各出一份，
  同一个克隆目录，只有扩展名表不同；`CE_FLOW_OUT=<目录>` 改写出目录——降表改动后在仓外重导一份、与冻结档逐格比池）。
- **候选池**（同目录 `pools.rs`，对一个单元的纯函数）：类 0 不可达 = 有前一兄弟（同父、seq 最大且更小）的源语句，前一兄弟本身是
  终结叶（return / throw / break / continue / goto / noreturn 调用）或带 `infinite` 位的循环为层 A、是子树含终结叶的 if / loop /
  switch / try 为层 B、是其余这四种结构语句为层 C，合成语句不作题；类 1 死存储 = 有读的非豁免变量（含形参）上的每次写
  （mode 1 / 2），同一变量按访问序的下一次访问是纯写（mode 1；readwrite 先读再写，不算）为层 A、它是最后一次访问为层 B、其余为
  层 C；类 2 未用局部量 = 非豁免局部量，零读（mode 0 / 2 都算读）为层 A、有读为层 B；类 3 未用形参同类 2（顾问，只记不门）。豁免 =
  `captured` / `ignored` / `address_taken` 任一位。每道题的锚：类 0 = 语句所在行与该行上按列序第几个源语句；类 1 = 写语句所在行、
  该行上对该名的第几次写、变量名；类 2 / 3 = 声明位置的行、该行第几个同名声明、名。dynamic 单元（核整体不判）的项不入池：它的每道题按构造只能答 unjudged，问了等于没问；dynamic 是降表侧的源码事实，与层同一性质，抽样仍对判决盲。每份宇宙仍记该语料的 dynamic 单元数。
- **抽样**（同文件 `flow_sample`）：一个语言的全部冻结宇宙合成一个池，逐文件先用同一份降表复现它的冻结行再取池项——池等于
  冻结宇宙靠核对、不靠信任。秩 = `sha256(域|corpus|commit|path|unit|kind|stratum|line|nth|name)`（名居末保单射：同一行的两个
  声明只差名字）；每格取 min(15, 该格的池) 个秩最小者，无备用题；样本按审阅域哈希排列（审阅者看不到秩序）。层留在样本档里
  （门用它重算秩；层是源码事实，不是答案），交给审阅代理的那一批不带层、不带产品的答案。档 `contracts/eval/flow-sample-<语言>-v<代>.json`。
- **CI 门**（`cli/tests/it/eval_flow.rs`，不跑 git、不要语料克隆）：冻结宇宙的集合恰为考题表的 `<语言>-<语料>` 键（按考题表第五列 `generation` 的代读：每个键在它考题的代上恰一份，更早一代的档是那一代的记录，更晚一代、不属任何键的档与缺的一份都点名；样本同读）、
  每份过共用信封（摘要从文件行重算、常量、行序、钉 tip）、语料与范围是考题的、排除只有「其他扩展名 / 走查拒读」两键、每个单元的
  三张表行数非负且池键合法；考题扩展名 = 产品路径表对该语言的扩展名行（`Lang::extensions`）；阶梯常量指向盘上的降表目录与
  它唯一的具名例外（`mod.rs`：模块表与判决掩码是政策不是答案）；样本的每格取数从冻结宇宙的摘要重算、每行两个哈希从自身字段重导、
  身份是考题的（语料在表内、commit = 该语料 tip）、（类，层）合法、(路径，单元) 在宇宙里并回显单元名与行段、行号落在行段内、
  同一单元同一格被抽中的数不超过它冻结行的池、不落在 dynamic 单元上；每份宇宙里 dynamic 单元的池全 0；篡改（伪路径、伪层、伪秩、伪审阅哈希、
  缺一行、伪单元、伪类、调换两行、行数超池、抽中 dynamic 单元）一律拒绝。
- **第二代**（提交 F，2026-09-30；设计册 §13 第 28 条）：降表改动挪了一门考题的候选池（逐（单元，格）比池计数，见「步 4 提交 E」一节的宇宙漂移表），这门的第一代样本就不再是新降表下的同一次抽样，于是这一门出第二代：考题表该行第五列 `generation` 1 → 2、stage 回 `sampled`，宇宙与样本在改动之后的树上按同一套仪器、同一组预登记常量重导（`CE_FLOW_OUT` 写到仓外、九份齐了一次拷入，样本读同一目录下的宇宙），全部题交新一批独立判官重判（F′），不复用第一代真值；池一格没动的门保留第一代。题的身份是 `audit` 哈希（它哈希的是题的全部身份字段），两代的共有 / 新增 / 消失按它数，不按秩。
- **第二代的读法**：第二代的批次提示在 `cannot_tell` 段与答案格式之间多一节 `## Language readings`（英文原文在 `prompt.rs` 的 `READINGS_TEXT`），把降表自提交 E 起读的隐式读写成题义，判官与产品按同一读法读同一段源码——此后留下的分歧只能是降表缺陷或真值争议，不再是没写明的约定。四组规则的中文复述：**TypeScript / TSX**，类型位置（注解、类型别名、泛型实参、`keyof typeof X`）的 `typeof X` 是对变量 X 的读；嵌套函数、箭头函数、类方法与对象字面量的 getter / setter / 方法提到 X 就读 X，不论离 X 的声明多远、是否被调用；带 `public` / `private` / `protected` / `readonly` 的构造器形参声明同名字段并从形参赋值，该形参被读。**Rust**，格式类宏（`format!`、`println!`、`write!`、`panic!`、`assert!` 系的消息、`format_args!` 等）的字符串字面量（raw 字符串也算）在花括号里点名的变量被读——`{name}`、`{name:?}`、`{name:>8}`，以及作宽度或精度的 `{:width$}`、`{:.prec$}`；`{{` 与 `}}` 是字面花括号、什么也不读。**C++**，构造器的成员初始化列表（`: m(x), n(y)`）读 x 与 y；带括号初始化的声明（`T v(x, y);`）像调用读实参那样读 x 与 y。**R**，`UseMethod`、`NextMethod`、`standardGeneric`、`callNextMethod` 调用读它所在函数的全部形参。第一代没写明这些，判官凭常识判；C2 的读数说明六个池未动的语言与这四组规则相容（类 0–2 fp 0，fn 逐条有归因），它们不重判。
- **盲判**与**精度册**：另两个提交（B′ / C）各记一段；门 = 每语言每种非顾问发现（0 / 1 / 2）读四态——`fail`（fp ≥ 1）、
  `pass`（fp = 0 ∧ tp ≥ 1）、`vacuous`（fp = tp = fn = 0：样本里没有正例可找，准入靠负例上的零误报，各节照抄「0 / n 个负例」；零行也归此态）、
  `silent`（fp = tp = 0 ∧ fn ≥ 1：有正例而一个没报，不准入）；三门各 ∈ {pass, vacuous} 的语言进 `flow::judged_mask()`，其余只 observe
  （设计册 §13 第 25 条：原判据让无正例可找的类 fail、却让零行判 vacuous，证据更多反判更差）。
- **盲判仪器**（提交 B′，`cli/tests/it/eval_flow_parts/` 下三件 + 门一件）：`batches.rs` 的批次渲染是冻结样本的纯函数（同一样本
  两次渲染逐字节同），先按语料分组、再按审阅序切成每批至多 25 道，提示模板常量 `PROMPT` 逐字取自判官提示模板
  `audit_prompt_template.md`，每批另写 `manifest.json`（批号、语料、题 id；不入库；第二代另带 `"readings": 2`，第一代无此键、读作 1）；提示模板与第二代的读法一节（`PROMPT` / `READINGS_TEXT`）住在同目录 `prompt.rs`，读法一节只在第二代渲染、第一代的批次逐字节不变；`answers.rs` 读每批一个 `answers-<n>.jsonl`，
  id 不在该批、一题多答或无答、truth 不在该类词表、理由长度越界、多余字段或非 JSON 行，每条拒绝按批按题点名；`review.rs` 逐字归档成
  `contracts/eval/flow-review-<语言>-v<代>.json` 并提供 `verify_review`（行与样本按审阅序一一对应、身份字段逐个相等、批号是批次计划的、
  摘要重算；信封的 `readings` 由 manifest 抄入，第二代档要 = 2、第一代缺席或 1）；`cli/tests/it/eval_flow_review.rs` 五腿门（渲染纯度、合成答案全收、每种拒绝点名、档与考题 stage 同真同假、合成档的篡改）
  外加对每份已归档的档跑七形篡改（外来秩、空理由、缺一行、翻一个 truth、调换两行、伪批号、伪 `readings`）。判官协议：53 个独立 Opus 子代理、每批一个，
  只读钉住的克隆与自己的批次文件，任何工具的答案都不在场，每题写一行 JSON；十份档共 1,197 行。十份档的 `auditor` 是同一句（各语言的「盲判」节只指到这里），逐字如下：

  > fifty-three independent Opus subagents, one batch each (at most 25 questions, one corpus per batch) in the sample's audit order, reading only the pinned clone under .ce-eval/corpora at its tip and their own batch file - never the product's lowering, no ce, no core, no tool answer anywhere in reach; each answered the four kinds from the source alone under the batch prompt's reading rules (a call that may throw does not end a path; a read is any use of the value, nested closures included; a member write reads the base) and wrote one JSON line per question in the batch's order; the coordinator assembled verbatim, judgments untouched (booklet analysis-track.md section 5.5; RG15); the batch prompt counts a question's nth from 0 while the sample stores it from 1, so every row echoes the sample's nth

  两处口径：批次提示里的 nth 从 0 数，
  档与样本一律从 1 数（档回显样本的 nth）；同一批只装一个语料。审阅档的 `generated_from` 记 ce 1.8.0、树 `2ea957d`、dirty = true，
  与提交 B 的宇宙与样本同一读法：归档工具与这十份档在同一个提交里落地。
  第二代（F′）：四门 20 批 454 道由 20 个独立子代理各判一批，批次提示带上面「第二代的读法」一节；第二代审阅档带 `readings` = 2（由 manifest 抄入）。四份 `flow-review-<语言>-v2.json` 的 `auditor` 是同一句（原文在档的信封里，不在此重引——它与上面第一代那句大半同文，两句并引即成一对重复段；各语言的「盲判（第二代）」节只指到这里），与第一代同一体例，逐项说的是：二十个独立子代理、每批一个（至多 25 道、每批一个语料）、按样本审阅序；只读钉住的克隆与自己的批次文件，产品的降表、ce、核与任何工具的答案都不在场；按批次提示的读法规则与它的 Language readings 一节作答（括注 readings 2 与四组读法的名目）；每题一行 JSON；零 `cannot_tell`；协调者逐字归档、判词不动（设计册 §5.5 与 §13 第 28 条，RG15）；批次提示的 nth 从 0 数、样本从 1 数，档回显样本的 nth。
- **精度仪器**（提交 C，`cli/tests/it/flow_precision/` 的 `mod.rs` / `flagged.rs` + 门两件；放在 `eval_flow_parts` 之外——那里没有模块
  读回它，不入该目录的导入环）：`flow_precision`
  （`#[ignore]`，`CE_FLOW_LANG=<语言>`，要 `CE_CORE_BIN`）对审阅档的每道题，在钉住的 tip 上取题所在的文件、经 `flow::lower::lower_file`
  降表、整文件经 `flow::wire::judge` 送真核（一条链路、每文件一次请求）；题按它被抽出时的池项回映——`pools.rs` 的池项带它代表的语句 seq
  与变量 v，同一个锚、不另推一遍：类 0 = 有一段不可达覆盖该语句、类 1 = 核点名该（写，变量）、类 2 / 3 = 核在该类点名该变量。单元未降出、
  被核拒或 dynamic 答 `unjudged`，逐单元记原因（`unjudged_reasons`）；抽样单元的名或行段与降表不符即按名停（抽样与降表不是同一棵树）。
  判词由（真值，答案）重算：真值正 = `unreachable` / `dead` / `unread`，`cannot_tell` 不入率；`per_kind` 每类记 tp / fp / tn / fn /
  unjudged / cannot_tell、`positives` / `negatives` 与 precision / recall 两个整数对，`gate` 读类 0 / 1 / 2 的四态，`judged` = 三门各 ∈
  {pass, vacuous}。档 `contracts/eval/flow-precision-<语言>-v<代>.json`（`ce.eval-flow-precision/1.0.0`；`CE_FLOW_OUT=<目录>` 改写出目录；
  拒绝覆写；`CE_FLOW_PRECISION_DRY=1` 只印读数与判为 fp / fn 的行、不写档——干跑读的降表若晚于样本（回修降表之后、下一代之前），池项换了层或变量被新降表豁免的题按锚（类、行、nth、名）回映并逐题印 `re-anchored`，核从不点名豁免变量，所以答案仍是产品的，写档的一路照旧按名停；写档前要求 `generated_from.dirty` = false）。门 `cli/tests/it/eval_flow_precision.rs`
  （不跑 git、不要克隆与核）：档与考题 stage 同真同假（`scored` 才在盘上），已归档的逐行对审阅档重算并跑六形篡改（翻判词、翻答案、伪门、
  伪 `judged`、缺一行、把 silent 读成 vacuous——档里没有 silent 时，把某个有正例的类的真答案全改假、计数全重算、只让门写 vacuous）；
  `flow::judged_mask()` 的每一位 ⇔ 该语言精度册 `judged`（tsx 与 typescript 各一位）；四态在手写计数上钉住；篡改电池另在门自己的合成档
  （python / rust；rust 读 F′ 归档的第二代审阅档，一门的当代尚无审阅档时读审阅门自己的合成审阅档）上先跑。出处门 `cli/tests/it/flow_provenance.rs`（跑 git，浅克隆拒）：三档 `generated_from` 的提交都在本历史上且严格先后；
  降表的首个提交是抽样提交的祖先或就是它，抽样到审阅之间降表零提交；精度册从自己的提交起到 HEAD，降表与 `scan/functions.rs` /
  `scan/walk.rs` / `scan/lang.rs` 无提交、工作树无未提交改动、`cli/Cargo.lock` 按钉版不动；反向探针两条（首个提交在抽样之后的路径、
  盲窗内动过的 `scan/lang.rs`）各按自己的句子红。第一代读数（提交 C2）在归档册各门「精度（第一代，提交 E 退役）」一节；现行读数见各语言「精度（第二代，2026-09-30）」或「精度（第一代复判，2026-09-30）」一节（提交 G，在 F′ 的干净树上生成，十门都 judged；汇总见「步 4 提交 G」一节）。

## 预登记常量（测量前写进每份档的 `constants`，改一个即换一套仪器）

| 常量 | 值 | 含义 |
|---|---|---|
| `min_per_stratum` | 15 | 每个（类，层）格的取数上限（池不足则整格取尽） |
| 站点域 / 审阅域 | `ce-flow-site-v1` / `ce-flow-audit-v1` | 秩与审阅序用两个互不相干的哈希域 |
| 类与层 | 0 unreachable A/B/C · 1 dead_store A/B/C · 2 unused_local A/B · 3 unused_param A/B | `eval_flow_parts::CLASSES` 一张字面量 |

## Python（步 4 提交 B）

### 宇宙与抽样（2026-09-30 冻结）

范围 = `*.py`；排除计「其他扩展名 / 走查拒读」；层的定义见「候选池」一段：

| 语料 | tip | 许可 | 文件 | 单元 | dynamic | unlowered | 排除 |
|---|---|---|---|---|---|---|---|
| psf/requests | `8068356` | Apache-2.0 | 37 | 711 | 0 | 0 | other_extension 93 |

| 类 | 层 A 池 → 取数 | 层 B 池 → 取数 | 层 C 池 → 取数 |
|---|---|---|---|
| 0 不可达 | 1 → 1 | 137 → 15 | 217 → 15 |
| 1 死存储 | 77 → 15 | 5 → 5 | 1,338 → 15 |
| 2 未用局部量 | 1 → 1 | 1,153 → 15 | — |
| 3 未用形参 | 83 → 15 | 713 → 15 | — |

样本共 112 道，落在 13 个文件、94 个单元上。

### 盲判（2026-09-30）

112 道分 5 批（每批一个语料、至多 25 道，按语料 requests 112），逐字归档自各批答案文件；审阅者句见「仪器与门」（十份档的 `auditor` 是同一句，只在那里逐字引一次）。

| 类 | 发现词 | 反面词 | cannot_tell |
|---|---|---|---|
| 0 不可达 | unreachable 0 | reachable 31 | 0 |
| 1 死存储 | dead 1 | live 34 | 0 |
| 2 未用局部量 | unread 1 | read 15 | 0 |
| 3 未用形参 | unread 15 | read 15 | 0 |

`cannot_tell` 0 道。

notes：无。

第一代的「精度（第一代，提交 E 退役）」一节已于 G（2026-09-30）逐字节搬进归档册 [EVAL-SET-FLOW-GEN1.md](EVAL-SET-FLOW-GEN1.md) 的同名一节；本门只有一代，上面的宇宙与盲判两节就是它的。

### 精度（第一代复判，2026-09-30）

`flow-precision-python-v1.json`：112 道，生成于 F′ 提交 `5023273` 的干净树（dirty = false），读的是提交 E 回修后的降表；判词与四态见「仪器与门」。

| 类 | tp | fp | tn | fn | unjudged | cannot_tell | 正例 | 负例 | precision | recall |
|---|---|---|---|---|---|---|---|---|---|---|
| 0 不可达 | 0 | 0 | 31 | 0 | 0 | 0 | 0 | 31 | — | — |
| 1 死存储 | 1 | 0 | 34 | 0 | 0 | 0 | 1 | 34 | 1 / 1 | 1 / 1 |
| 2 未用局部量 | 1 | 0 | 15 | 0 | 0 | 0 | 1 | 15 | 1 / 1 | 1 / 1 |
| 3 未用形参 | 15 | 0 | 15 | 0 | 0 | 0 | 15 | 15 | 15 / 15 | 15 / 15 |

门：类 0 vacuous（0 / 31 个负例） · 类 1 pass（1 / 1） · 类 2 pass（1 / 1）；`judged` = true（类 3 顾问只记不判）。

误报（类 0–2）0 条、漏报（类 0–2）0 条、顾问误报（类 3）0 条、unjudged 0 道。

## TypeScript（步 4 提交 B）

第一代的「宇宙与抽样」「盲判」「精度（第一代，提交 E 退役）」三节已于 F′（2026-09-30）逐字节搬进归档册 [EVAL-SET-FLOW-GEN1.md](EVAL-SET-FLOW-GEN1.md) 的同名一节；本节只记第二代。

### 第二代（F，2026-09-30）

- 宇宙 v1 → v2（池动的单元 = 逐（单元，格）比池计数）：colinhacks/zod 文件 375 → 375、单元 6,356 → 6,356、dynamic 0 → 0、unlowered 0 → 0、排除 other_extension 208 → 208，池动的单元 52。
- 池 v1 → v2 → 取数（按类按层）：0 不可达 A 0 → 0 → 0 · B 1,000 → 1,000 → 15 · C 467 → 467 → 15；1 死存储 A 59 → 59 → 15 · B 41 → 41 → 15 · C 4,668 → 4,627 → 15；2 未用局部量 A 27 → 0 → 0 · B 4,527 → 4,486 → 15；3 未用形参 A 7 → 0 → 0 · B 2,929 → 2,936 → 15。
- `flow-sample-typescript-v2.json`：105 道（第一代 127），落在 55 个文件、88 个单元上；与第一代共有 105、新增 0、消失 22（2/A 共有 0 / 新增 0 / 消失 15、3/A 共有 0 / 新增 0 / 消失 7）。批次 5 批：25 / 25 / 25 / 25 / 5。

### 盲判（第二代，2026-09-30）

105 道分 5 批（每批一个语料、至多 25 道，按语料 zod 105），逐字归档自各批答案文件，档 `flow-review-typescript-v2.json`（`readings` = 2）；审阅者句见「仪器与门」（第二代四份档的 `auditor` 是同一句，在那里逐项复述）。

| 类 | 发现词 | 反面词 | cannot_tell |
|---|---|---|---|
| 0 不可达 | unreachable 0 | reachable 30 | 0 |
| 1 死存储 | dead 0 | live 45 | 0 |
| 2 未用局部量 | unread 0 | read 15 | 0 |
| 3 未用形参 | unread 0 | read 15 | 0 |

`cannot_tell` 0 道。

notes：无。

与第一代比（同一题 = `audit` 哈希相同）：共有 105 道，真值一致 104 道（104 / 105），不一致 1 道，真值以第二代为准（档不改）：

- 类 1 层 C `zod:packages/zod/src/v3/tests/partials.test.ts:152` `(anonymous)` 变量 `requiredObject`：第一代 `dead`（Only later mention is `type required = z.infer<typeof requiredObject>;` (line 154), a type-level query erased at compile time; the value is never read before the function ends.）→ 第二代 `live`（Line 154 type required = z.infer<typeof requiredObject>; reads requiredObject (typeof in a type position).）。归因：第二代读法规则使然——第二代提示的 TypeScript 读法写明类型位置的 `typeof X` 是对变量 X 的读（设计册 §13 第 27 条 TS-2 的裁定、第 28 条读法即题义），这次写入之后唯一再提到它的 `type required = z.infer<typeof requiredObject>;` 因此读了它；第一代没写这条读法，判官按编译期擦除判 `dead`，正是第一代精度册里记为真值争议的那道漏报。源码在 tip 上核过，两代判官都读对了各自题义下的源码，不算判错。

### 精度（第二代，2026-09-30）

`flow-precision-typescript-v2.json`：105 道，生成于 F′ 提交 `5023273` 的干净树（dirty = false），读的是提交 E 回修后的降表；判词与四态见「仪器与门」。

| 类 | tp | fp | tn | fn | unjudged | cannot_tell | 正例 | 负例 | precision | recall |
|---|---|---|---|---|---|---|---|---|---|---|
| 0 不可达 | 0 | 0 | 30 | 0 | 0 | 0 | 0 | 30 | — | — |
| 1 死存储 | 0 | 0 | 45 | 0 | 0 | 0 | 0 | 45 | — | — |
| 2 未用局部量 | 0 | 0 | 15 | 0 | 0 | 0 | 0 | 15 | — | — |
| 3 未用形参 | 0 | 0 | 15 | 0 | 0 | 0 | 0 | 15 | — | — |

门：类 0 vacuous（0 / 30 个负例） · 类 1 vacuous（0 / 45 个负例） · 类 2 vacuous（0 / 15 个负例）；`judged` = true（类 3 顾问只记不判）。

误报（类 0–2）0 条、漏报（类 0–2）0 条、顾问误报（类 3）0 条、unjudged 0 道。

## TSX（步 4 提交 B）

### 宇宙与抽样（2026-09-30 冻结）

范围 = `*.tsx`；排除计「其他扩展名 / 走查拒读」；层的定义见「候选池」一段：

| 语料 | tip | 许可 | 文件 | 单元 | dynamic | unlowered | 排除 |
|---|---|---|---|---|---|---|---|
| colinhacks/zod | `912f0f5` | MIT | 29 | 71 | 0 | 0 | other_extension 554 |

| 类 | 层 A 池 → 取数 | 层 B 池 → 取数 | 层 C 池 → 取数 |
|---|---|---|---|
| 0 不可达 | 0 → 0 | 3 → 3 | 4 → 4 |
| 1 死存储 | 0 → 0 | 0 → 0 | 65 → 15 |
| 2 未用局部量 | 0 → 0 | 65 → 15 | — |
| 3 未用形参 | 0 → 0 | 58 → 15 | — |

样本共 52 道，落在 18 个文件、29 个单元上。

### 盲判（2026-09-30）

52 道分 3 批（每批一个语料、至多 25 道，按语料 zod 52），逐字归档自各批答案文件；审阅者句见「仪器与门」（十份档的 `auditor` 是同一句，只在那里逐字引一次）。

| 类 | 发现词 | 反面词 | cannot_tell |
|---|---|---|---|
| 0 不可达 | unreachable 0 | reachable 7 | 0 |
| 1 死存储 | dead 0 | live 15 | 0 |
| 2 未用局部量 | unread 0 | read 15 | 0 |
| 3 未用形参 | unread 0 | read 15 | 0 |

`cannot_tell` 0 道。

notes：无。

第一代的「精度（第一代，提交 E 退役）」一节已于 G（2026-09-30）逐字节搬进归档册 [EVAL-SET-FLOW-GEN1.md](EVAL-SET-FLOW-GEN1.md) 的同名一节；本门只有一代，上面的宇宙与盲判两节就是它的。

### 精度（第一代复判，2026-09-30）

`flow-precision-tsx-v1.json`：52 道，生成于 F′ 提交 `5023273` 的干净树（dirty = false），读的是提交 E 回修后的降表；判词与四态见「仪器与门」。

| 类 | tp | fp | tn | fn | unjudged | cannot_tell | 正例 | 负例 | precision | recall |
|---|---|---|---|---|---|---|---|---|---|---|
| 0 不可达 | 0 | 0 | 7 | 0 | 0 | 0 | 0 | 7 | — | — |
| 1 死存储 | 0 | 0 | 15 | 0 | 0 | 0 | 0 | 15 | — | — |
| 2 未用局部量 | 0 | 0 | 15 | 0 | 0 | 0 | 0 | 15 | — | — |
| 3 未用形参 | 0 | 0 | 15 | 0 | 0 | 0 | 0 | 15 | — | — |

门：类 0 vacuous（0 / 7 个负例） · 类 1 vacuous（0 / 15 个负例） · 类 2 vacuous（0 / 15 个负例）；`judged` = true（类 3 顾问只记不判）。

误报（类 0–2）0 条、漏报（类 0–2）0 条、顾问误报（类 3）0 条、unjudged 0 道。

## Rust（步 4 提交 B）

第一代的「宇宙与抽样」「盲判」「精度（第一代，提交 E 退役）」三节已于 F′（2026-09-30）逐字节搬进归档册 [EVAL-SET-FLOW-GEN1.md](EVAL-SET-FLOW-GEN1.md) 的同名一节；本节只记第二代。

### 第二代（F，2026-09-30）

- 宇宙 v1 → v2（池动的单元 = 逐（单元，格）比池计数）：BurntSushi/ripgrep 文件 110 → 110、单元 3,345 → 3,345、dynamic 0 → 0、unlowered 0 → 0、排除 other_extension 127 → 127，池动的单元 6；skymanbp/CodeEraser（本仓，克隆名 `codeeraser-flow`） 文件 361 → 361、单元 4,619 → 4,619、dynamic 0 → 0、unlowered 0 → 0、排除 other_extension 548 → 548 / walk_refused 6 → 6，池动的单元 3。
- 池 v1 → v2 → 取数（按类按层）：0 不可达 A 7 → 7 → 7 · B 1,432 → 1,432 → 15 · C 1,034 → 1,034 → 15；1 死存储 A 36 → 36 → 15 · B 61 → 61 → 15 · C 8,617 → 8,632 → 15；2 未用局部量 A 15 → 0 → 0 · B 8,414 → 8,429 → 15；3 未用形参 A 9 → 7 → 7 · B 7,615 → 7,617 → 15。
- `flow-sample-rust-v2.json`：119 道（第一代 136），落在 87 个文件、109 个单元上，按语料分是 ripgrep 45 / codeeraser-flow 74；与第一代共有 119、新增 0、消失 17（2/A 共有 0 / 新增 0 / 消失 15、3/A 共有 7 / 新增 0 / 消失 2）。批次 5 批：25 / 20 / 25 / 25 / 24。

### 盲判（第二代，2026-09-30）

119 道分 5 批（每批一个语料、至多 25 道，按语料 ripgrep 45 / codeeraser-flow 74），逐字归档自各批答案文件，档 `flow-review-rust-v2.json`（`readings` = 2）；审阅者句见「仪器与门」（第二代四份档的 `auditor` 是同一句，在那里逐项复述）。

| 类 | 发现词 | 反面词 | cannot_tell |
|---|---|---|---|
| 0 不可达 | unreachable 0 | reachable 37 | 0 |
| 1 死存储 | dead 0 | live 45 | 0 |
| 2 未用局部量 | unread 0 | read 15 | 0 |
| 3 未用形参 | unread 7 | read 15 | 0 |

`cannot_tell` 0 道。

notes：无。

与第一代比（同一题 = `audit` 哈希相同）：共有 119 道，真值一致 119 道（119 / 119），不一致 0 道。

### 精度（第二代，2026-09-30）

`flow-precision-rust-v2.json`：119 道，生成于 F′ 提交 `5023273` 的干净树（dirty = false），读的是提交 E 回修后的降表；判词与四态见「仪器与门」。

| 类 | tp | fp | tn | fn | unjudged | cannot_tell | 正例 | 负例 | precision | recall |
|---|---|---|---|---|---|---|---|---|---|---|
| 0 不可达 | 0 | 0 | 37 | 0 | 0 | 0 | 0 | 37 | — | — |
| 1 死存储 | 0 | 0 | 45 | 0 | 0 | 0 | 0 | 45 | — | — |
| 2 未用局部量 | 0 | 0 | 15 | 0 | 0 | 0 | 0 | 15 | — | — |
| 3 未用形参 | 7 | 0 | 15 | 0 | 0 | 0 | 7 | 15 | 7 / 7 | 7 / 7 |

门：类 0 vacuous（0 / 37 个负例） · 类 1 vacuous（0 / 45 个负例） · 类 2 vacuous（0 / 15 个负例）；`judged` = true（类 3 顾问只记不判）。

误报（类 0–2）0 条、漏报（类 0–2）0 条、顾问误报（类 3）0 条、unjudged 0 道。

## Go（步 4 提交 B）

### 宇宙与抽样（2026-09-30 冻结）

范围 = `*.go`；排除计「其他扩展名 / 走查拒读」；层的定义见「候选池」一段：

| 语料 | tip | 许可 | 文件 | 单元 | dynamic | unlowered | 排除 |
|---|---|---|---|---|---|---|---|
| spf13/cobra | `adbc881` | Apache-2.0 | 36 | 595 | 0 | 0 | other_extension 30 |

| 类 | 层 A 池 → 取数 | 层 B 池 → 取数 | 层 C 池 → 取数 |
|---|---|---|---|
| 0 不可达 | 0 → 0 | 212 → 15 | 608 → 15 |
| 1 死存储 | 37 → 15 | 18 → 15 | 1,880 → 15 |
| 2 未用局部量 | 0 → 0 | 1,465 → 15 | — |
| 3 未用形参 | 19 → 15 | 644 → 15 | — |

样本共 120 道，落在 22 个文件、93 个单元上。

### 盲判（2026-09-30）

120 道分 5 批（每批一个语料、至多 25 道，按语料 cobra 120），逐字归档自各批答案文件；审阅者句见「仪器与门」（十份档的 `auditor` 是同一句，只在那里逐字引一次）。

| 类 | 发现词 | 反面词 | cannot_tell |
|---|---|---|---|
| 0 不可达 | unreachable 0 | reachable 30 | 0 |
| 1 死存储 | dead 0 | live 45 | 0 |
| 2 未用局部量 | unread 0 | read 15 | 0 |
| 3 未用形参 | unread 15 | read 15 | 0 |

`cannot_tell` 0 道。

notes：无。

第一代的「精度（第一代，提交 E 退役）」一节已于 G（2026-09-30）逐字节搬进归档册 [EVAL-SET-FLOW-GEN1.md](EVAL-SET-FLOW-GEN1.md) 的同名一节；本门只有一代，上面的宇宙与盲判两节就是它的。

### 精度（第一代复判，2026-09-30）

`flow-precision-go-v1.json`：120 道，生成于 F′ 提交 `5023273` 的干净树（dirty = false），读的是提交 E 回修后的降表；判词与四态见「仪器与门」。

| 类 | tp | fp | tn | fn | unjudged | cannot_tell | 正例 | 负例 | precision | recall |
|---|---|---|---|---|---|---|---|---|---|---|
| 0 不可达 | 0 | 0 | 30 | 0 | 0 | 0 | 0 | 30 | — | — |
| 1 死存储 | 0 | 0 | 45 | 0 | 0 | 0 | 0 | 45 | — | — |
| 2 未用局部量 | 0 | 0 | 15 | 0 | 0 | 0 | 0 | 15 | — | — |
| 3 未用形参 | 15 | 0 | 15 | 0 | 0 | 0 | 15 | 15 | 15 / 15 | 15 / 15 |

门：类 0 vacuous（0 / 30 个负例） · 类 1 vacuous（0 / 45 个负例） · 类 2 vacuous（0 / 15 个负例）；`judged` = true（类 3 顾问只记不判）。

误报（类 0–2）0 条、漏报（类 0–2）0 条、顾问误报（类 3）0 条、unjudged 0 道。

## C（步 4 提交 B）

### 宇宙与抽样（2026-09-30 冻结）

范围 = `*.c`；排除计「其他扩展名 / 走查拒读」；层的定义见「候选池」一段：

| 语料 | tip | 许可 | 文件 | 单元 | dynamic | unlowered | 排除 |
|---|---|---|---|---|---|---|---|
| lua/lua | `0b29f40` | MIT | 40 | 1,307 | 228 | 0 | other_extension 71 |

| 类 | 层 A 池 → 取数 | 层 B 池 → 取数 | 层 C 池 → 取数 |
|---|---|---|---|
| 0 不可达 | 149 → 15 | 310 → 15 | 626 → 15 |
| 1 死存储 | 197 → 15 | 188 → 15 | 1,916 → 15 |
| 2 未用局部量 | 0 → 0 | 1,528 → 15 | — |
| 3 未用形参 | 2 → 2 | 2,248 → 15 | — |

样本共 122 道，落在 31 个文件、110 个单元上。

### 盲判（2026-09-30）

122 道分 5 批（每批一个语料、至多 25 道，按语料 lua 122），逐字归档自各批答案文件；审阅者句见「仪器与门」（十份档的 `auditor` 是同一句，只在那里逐字引一次）。

| 类 | 发现词 | 反面词 | cannot_tell |
|---|---|---|---|
| 0 不可达 | unreachable 0 | reachable 45 | 0 |
| 1 死存储 | dead 0 | live 45 | 0 |
| 2 未用局部量 | unread 0 | read 15 | 0 |
| 3 未用形参 | unread 2 | read 15 | 0 |

`cannot_tell` 0 道。

notes：无。

第一代的「精度（第一代，提交 E 退役）」一节已于 G（2026-09-30）逐字节搬进归档册 [EVAL-SET-FLOW-GEN1.md](EVAL-SET-FLOW-GEN1.md) 的同名一节；本门只有一代，上面的宇宙与盲判两节就是它的。

### 精度（第一代复判，2026-09-30）

`flow-precision-c-v1.json`：122 道，生成于 F′ 提交 `5023273` 的干净树（dirty = false），读的是提交 E 回修后的降表；判词与四态见「仪器与门」。

| 类 | tp | fp | tn | fn | unjudged | cannot_tell | 正例 | 负例 | precision | recall |
|---|---|---|---|---|---|---|---|---|---|---|
| 0 不可达 | 0 | 0 | 45 | 0 | 0 | 0 | 0 | 45 | — | — |
| 1 死存储 | 0 | 0 | 45 | 0 | 0 | 0 | 0 | 45 | — | — |
| 2 未用局部量 | 0 | 0 | 15 | 0 | 0 | 0 | 0 | 15 | — | — |
| 3 未用形参 | 2 | 0 | 15 | 0 | 0 | 0 | 2 | 15 | 2 / 2 | 2 / 2 |

门：类 0 vacuous（0 / 45 个负例） · 类 1 vacuous（0 / 45 个负例） · 类 2 vacuous（0 / 15 个负例）；`judged` = true（类 3 顾问只记不判）。

误报（类 0–2）0 条、漏报（类 0–2）0 条、顾问误报（类 3）0 条、unjudged 0 道。

## C++（步 4 提交 B）

第一代的「宇宙与抽样」「盲判」「精度（第一代，提交 E 退役）」三节已于 F′（2026-09-30）逐字节搬进归档册 [EVAL-SET-FLOW-GEN1.md](EVAL-SET-FLOW-GEN1.md) 的同名一节；本节只记第二代。

### 第二代（F，2026-09-30）

- 宇宙 v1 → v2（池动的单元 = 逐（单元，格）比池计数）：fmtlib/fmt 文件 73 → 73、单元 4,654 → 4,654、dynamic 550 → 550、unlowered 0 → 0、排除 other_extension 72 → 72，池动的单元 188。
- 池 v1 → v2 → 取数（按类按层）：0 不可达 A 11 → 11 → 11 · B 532 → 532 → 15 · C 498 → 498 → 15；1 死存储 A 95 → 95 → 15 · B 229 → 226 → 15 · C 2,344 → 2,349 → 15；2 未用局部量 A 38 → 32 → 15 · B 2,059 → 2,065 → 15；3 未用形参 A 286 → 3 → 3 · B 2,093 → 2,375 → 15。
- `flow-sample-cpp-v2.json`：134 道（第一代 146），落在 26 个文件、121 个单元上；与第一代共有 128、新增 6、消失 18（1/B 共有 14 / 新增 1 / 消失 1、2/A 共有 14 / 新增 1 / 消失 1、3/A 共有 0 / 新增 3 / 消失 15、3/B 共有 14 / 新增 1 / 消失 1）。批次 6 批：25 / 25 / 25 / 25 / 25 / 9。

### 盲判（第二代，2026-09-30）

134 道分 6 批（每批一个语料、至多 25 道，按语料 fmt 134），逐字归档自各批答案文件，档 `flow-review-cpp-v2.json`（`readings` = 2）；审阅者句见「仪器与门」（第二代四份档的 `auditor` 是同一句，在那里逐项复述）。

| 类 | 发现词 | 反面词 | cannot_tell |
|---|---|---|---|
| 0 不可达 | unreachable 1 | reachable 40 | 0 |
| 1 死存储 | dead 4 | live 41 | 0 |
| 2 未用局部量 | unread 15 | read 15 | 0 |
| 3 未用形参 | unread 2 | read 16 | 0 |

`cannot_tell` 0 道。

notes：无。

与第一代比（同一题 = `audit` 哈希相同）：共有 128 道，真值一致 128 道（128 / 128），不一致 0 道；第二代新增的 6 道没有第一代真值可比。

### 精度（第二代，2026-09-30）

`flow-precision-cpp-v2.json`：134 道，生成于 F′ 提交 `5023273` 的干净树（dirty = false），读的是提交 E 回修后的降表；判词与四态见「仪器与门」。

| 类 | tp | fp | tn | fn | unjudged | cannot_tell | 正例 | 负例 | precision | recall |
|---|---|---|---|---|---|---|---|---|---|---|
| 0 不可达 | 1 | 0 | 40 | 0 | 0 | 0 | 1 | 40 | 1 / 1 | 1 / 1 |
| 1 死存储 | 2 | 0 | 41 | 2 | 0 | 0 | 4 | 41 | 2 / 2 | 2 / 4 |
| 2 未用局部量 | 15 | 0 | 15 | 0 | 0 | 0 | 15 | 15 | 15 / 15 | 15 / 15 |
| 3 未用形参 | 2 | 1 | 15 | 0 | 0 | 0 | 2 | 16 | 2 / 3 | 2 / 2 |

门：类 0 pass（1 / 1） · 类 1 pass（2 / 2） · 类 2 pass（15 / 15）；`judged` = true（类 3 顾问只记不判）。

漏报（类 0–2）2 条，归因：do-while 少报（设计册 §13 第 20 / 27 条的既有登记）：do-while 的条件挂在循环结点上，体可跳过的那条多出的路径让循环前的写读成活；本提交不修降表：
- 类 1 层 A `fmt:test/scan.h:401` `read` 变量 `prev_digit`：The do-while body always runs `prev_digit = c;` (line 405) before the only read, `unsigned(prev_digit - '0')` on line 418.
- 类 1 层 A `fmt:include/fmt/core.h:1311` `parse_nonnegative_int` 变量 `prev`：`prev = 0` is overwritten by `prev = value;` at the top of the do-while body, which always runs before prev is ever read (line 1325).

顾问误报（类 3）1 条，归因：降表缺陷（新登记 CPP-4，设计册 §13 第 30 条）：C++ 文法把成员指针调用式 `(w.*cb)(args...)` 的 `w.*cb` 读成 `field_expression`（operator `.*`、field 是 `field_identifier`），降表只读 `field_expression` 的 argument 一侧（成员基底 `w`），field 一侧从不解析成变量，于是形参 `cb` 零读；顾问类，只记不门，留给下一代降表：
- 类 3 层 A `fmt:include/fmt/chrono.h:1721` `duration_formatter::format_tm` 变量 `cb`：Line 1724 `(w.*cb)(args...);` calls through the member pointer cb, reading it.

误报（类 0–2）0 条、unjudged 0 道。

## Java（步 4 提交 B）

### 宇宙与抽样（2026-09-30 冻结）

范围 = `*.java`；排除计「其他扩展名 / 走查拒读」；层的定义见「候选池」一段：

| 语料 | tip | 许可 | 文件 | 单元 | dynamic | unlowered | 排除 |
|---|---|---|---|---|---|---|---|
| google/gson | `854c825` | Apache-2.0 | 264 | 3,439 | 0 | 0 | other_extension 49 |

| 类 | 层 A 池 → 取数 | 层 B 池 → 取数 | 层 C 池 → 取数 |
|---|---|---|---|
| 0 不可达 | 5 → 5 | 516 → 15 | 317 → 15 |
| 1 死存储 | 125 → 15 | 143 → 15 | 5,036 → 15 |
| 2 未用局部量 | 105 → 15 | 4,541 → 15 | — |
| 3 未用形参 | 283 → 15 | 1,715 → 15 | — |

样本共 140 道，落在 67 个文件、126 个单元上。

### 盲判（2026-09-30）

140 道分 6 批（每批一个语料、至多 25 道，按语料 gson 140），逐字归档自各批答案文件；审阅者句见「仪器与门」（十份档的 `auditor` 是同一句，只在那里逐字引一次）。

| 类 | 发现词 | 反面词 | cannot_tell |
|---|---|---|---|
| 0 不可达 | unreachable 0 | reachable 35 | 0 |
| 1 死存储 | dead 0 | live 45 | 0 |
| 2 未用局部量 | unread 15 | read 15 | 0 |
| 3 未用形参 | unread 15 | read 15 | 0 |

`cannot_tell` 0 道。

notes：无。

第一代的「精度（第一代，提交 E 退役）」一节已于 G（2026-09-30）逐字节搬进归档册 [EVAL-SET-FLOW-GEN1.md](EVAL-SET-FLOW-GEN1.md) 的同名一节；本门只有一代，上面的宇宙与盲判两节就是它的。

### 精度（第一代复判，2026-09-30）

`flow-precision-java-v1.json`：140 道，生成于 F′ 提交 `5023273` 的干净树（dirty = false），读的是提交 E 回修后的降表；判词与四态见「仪器与门」。

| 类 | tp | fp | tn | fn | unjudged | cannot_tell | 正例 | 负例 | precision | recall |
|---|---|---|---|---|---|---|---|---|---|---|
| 0 不可达 | 0 | 0 | 35 | 0 | 0 | 0 | 0 | 35 | — | — |
| 1 死存储 | 0 | 0 | 45 | 0 | 0 | 0 | 0 | 45 | — | — |
| 2 未用局部量 | 15 | 0 | 15 | 0 | 0 | 0 | 15 | 15 | 15 / 15 | 15 / 15 |
| 3 未用形参 | 15 | 0 | 15 | 0 | 0 | 0 | 15 | 15 | 15 / 15 | 15 / 15 |

门：类 0 vacuous（0 / 35 个负例） · 类 1 vacuous（0 / 45 个负例） · 类 2 pass（15 / 15）；`judged` = true（类 3 顾问只记不判）。

误报（类 0–2）0 条、漏报（类 0–2）0 条、顾问误报（类 3）0 条、unjudged 0 道。

## Lua（步 4 提交 B）

### 宇宙与抽样（2026-09-30 冻结）

范围 = `*.lua`；排除计「其他扩展名 / 走查拒读」；层的定义见「候选池」一段：

| 语料 | tip | 许可 | 文件 | 单元 | dynamic | unlowered | 排除 |
|---|---|---|---|---|---|---|---|
| luarocks/luarocks | `2d2cc8e` | MIT | 157 | 1,821 | 9 | 0 | other_extension 347 / walk_refused 5 |

| 类 | 层 A 池 → 取数 | 层 B 池 → 取数 | 层 C 池 → 取数 |
|---|---|---|---|
| 0 不可达 | 7 → 7 | 861 → 15 | 848 → 15 |
| 1 死存储 | 323 → 15 | 37 → 15 | 3,981 → 15 |
| 2 未用局部量 | 30 → 15 | 3,249 → 15 | — |
| 3 未用形参 | 22 → 15 | 1,556 → 15 | — |

样本共 142 道，落在 62 个文件、120 个单元上。

### 盲判（2026-09-30）

142 道分 6 批（每批一个语料、至多 25 道，按语料 luarocks 142），逐字归档自各批答案文件；审阅者句见「仪器与门」（十份档的 `auditor` 是同一句，只在那里逐字引一次）。

| 类 | 发现词 | 反面词 | cannot_tell |
|---|---|---|---|
| 0 不可达 | unreachable 1 | reachable 36 | 0 |
| 1 死存储 | dead 4 | live 41 | 0 |
| 2 未用局部量 | unread 15 | read 15 | 0 |
| 3 未用形参 | unread 15 | read 15 | 0 |

`cannot_tell` 0 道。

notes：无。

第一代的「精度（第一代，提交 E 退役）」一节已于 G（2026-09-30）逐字节搬进归档册 [EVAL-SET-FLOW-GEN1.md](EVAL-SET-FLOW-GEN1.md) 的同名一节；本门只有一代，上面的宇宙与盲判两节就是它的。

### 精度（第一代复判，2026-09-30）

`flow-precision-lua-v1.json`：142 道，生成于 F′ 提交 `5023273` 的干净树（dirty = false），读的是提交 E 回修后的降表；判词与四态见「仪器与门」。

| 类 | tp | fp | tn | fn | unjudged | cannot_tell | 正例 | 负例 | precision | recall |
|---|---|---|---|---|---|---|---|---|---|---|
| 0 不可达 | 1 | 0 | 36 | 0 | 0 | 0 | 1 | 36 | 1 / 1 | 1 / 1 |
| 1 死存储 | 3 | 0 | 41 | 1 | 0 | 0 | 4 | 41 | 3 / 3 | 3 / 4 |
| 2 未用局部量 | 15 | 0 | 15 | 0 | 0 | 0 | 15 | 15 | 15 / 15 | 15 / 15 |
| 3 未用形参 | 15 | 0 | 15 | 0 | 0 | 0 | 15 | 15 | 15 / 15 | 15 / 15 |

门：类 0 pass（1 / 1） · 类 1 pass（3 / 3） · 类 2 pass（15 / 15）；`judged` = true（类 3 顾问只记不判）。

漏报（类 0–2）1 条，归因：路径不相关（设计册 §13 第 27 条 LUA-1）：两处 `if not file` 的条件相关性不进控制流图，`errcode` 沿不可行的路径读成活；判决本就路径不敏感，记录：
- 类 1 层 A `luarocks:src/luarocks/fetch.lua:248` `fetch.fetch_url_at_temp_dir` 变量 `errcode`：If cachefile is set, file is set and the function returns file, temp_dir without reading errcode; otherwise line 257 `file, err, errcode = fetch.fetch_url(...)` overwrites it.

误报（类 0–2）0 条、顾问误报（类 3）0 条、unjudged 0 道。

## R（步 4 提交 B）

第一代的「宇宙与抽样」「盲判」「精度（第一代，提交 E 退役）」三节已于 F′（2026-09-30）逐字节搬进归档册 [EVAL-SET-FLOW-GEN1.md](EVAL-SET-FLOW-GEN1.md) 的同名一节；本节只记第二代。

### 第二代（F，2026-09-30）

- 宇宙 v1 → v2（池动的单元 = 逐（单元，格）比池计数）：tidyverse/stringr 文件 67 → 67、单元 200 → 200、dynamic 5 → 5、unlowered 0 → 0、排除 other_extension 114 → 114，池动的单元 3。
- 池 v1 → v2 → 取数（按类按层）：0 不可达 A 0 → 0 → 0 · B 69 → 69 → 15 · C 30 → 30 → 15；1 死存储 A 12 → 12 → 12 · B 1 → 1 → 1 · C 196 → 196 → 15；2 未用局部量 A 0 → 0 → 0 · B 143 → 143 → 15；3 未用形参 A 12 → 8 → 8 · B 438 → 442 → 15。
- `flow-sample-r-v2.json`：96 道（第一代 100），落在 20 个文件、55 个单元上；与第一代共有 96、新增 0、消失 4（3/A 共有 8 / 新增 0 / 消失 4）。批次 4 批：25 / 25 / 25 / 21。

### 盲判（第二代，2026-09-30）

96 道分 4 批（每批一个语料、至多 25 道，按语料 stringr 96），逐字归档自各批答案文件，档 `flow-review-r-v2.json`（`readings` = 2）；审阅者句见「仪器与门」（第二代四份档的 `auditor` 是同一句，在那里逐项复述）。

| 类 | 发现词 | 反面词 | cannot_tell |
|---|---|---|---|
| 0 不可达 | unreachable 0 | reachable 30 | 0 |
| 1 死存储 | dead 0 | live 28 | 0 |
| 2 未用局部量 | unread 0 | read 15 | 0 |
| 3 未用形参 | unread 8 | read 15 | 0 |

`cannot_tell` 0 道。

notes：无。

与第一代比（同一题 = `audit` 哈希相同）：共有 96 道，真值一致 96 道（96 / 96），不一致 0 道。

### 精度（第二代，2026-09-30）

`flow-precision-r-v2.json`：96 道，生成于 F′ 提交 `5023273` 的干净树（dirty = false），读的是提交 E 回修后的降表；判词与四态见「仪器与门」。

| 类 | tp | fp | tn | fn | unjudged | cannot_tell | 正例 | 负例 | precision | recall |
|---|---|---|---|---|---|---|---|---|---|---|
| 0 不可达 | 0 | 0 | 30 | 0 | 0 | 0 | 0 | 30 | — | — |
| 1 死存储 | 0 | 0 | 28 | 0 | 0 | 0 | 0 | 28 | — | — |
| 2 未用局部量 | 0 | 0 | 15 | 0 | 0 | 0 | 0 | 15 | — | — |
| 3 未用形参 | 8 | 0 | 15 | 0 | 0 | 0 | 8 | 15 | 8 / 8 | 8 / 8 |

门：类 0 vacuous（0 / 30 个负例） · 类 1 vacuous（0 / 28 个负例） · 类 2 vacuous（0 / 15 个负例）；`judged` = true（类 3 顾问只记不判）。

误报（类 0–2）0 条、漏报（类 0–2）0 条、顾问误报（类 3）0 条、unjudged 0 道。

## 步 4 提交 E：降表回修与第一代精度册退役（2026-09-30）

**退役**：提交 E 改了 `cli/src/flow/` 下的降表文件，它们在精度册的 `ANSWERED_BY` 里，十份第一代精度册
`flow-precision-<语言>-v1.json` 按名退役（删出树、历史里仍在）；十门考题的 `stage` 翻回 `audited`，`flow::judged_mask()` 随门清空
（每一位 ⇔ 一份 judged 的精度册）。各语言「精度（第一代，…）」一节是第一代的记录，都在归档册 [EVAL-SET-FLOW-GEN1.md](EVAL-SET-FLOW-GEN1.md)（池动的四门的这一节连同它们第一代的宇宙与盲判两节 F′ 起在那里，其余六门的这一节 G 起在那里）。宇宙、样本与审阅档不动。

**回修**（设计册 §5.1 第 8–10 条与图例句、§13 第 27 条；每类一条规则、一份探针样本 `scripts/tsprobe/snippets/flow.<语言>`、子仓
`unit/flow/reads.rs` 一块期望表）：RS-1 宏里字符串字面量（含 raw）的占位符按 std 文法读；TS-1 TypeScript 嵌套作用域提到稍后
声明的块级绑定即置 `captured`；TS-2 类型位置的 `typeof x` 是读（注解、别名、声明的类型）；TS-3 参数属性与 CPP-1 成员初始化列表
在单元入口的合成语句里读；CPP-2 被读成原型的声明读它形参表里解析到变量的类型名；R-1 派发调用读全部形参；LEG-1 图例每条语句
记终点行。CPP-3（do-while 少报，§13 第 20 条）与 LUA-1（路径不敏感）核实为既有登记、不改。

**干跑读数**（新降表、第一代审阅档，`CE_FLOW_PRECISION_DRY=1`；tp / fp / tn / fn；「重锚」= 按锚回映的题数）：

| 语言 | 类 0 | 类 1 | 类 2 | 类 3 | 门 0 / 1 / 2 | 重锚 |
|---|---|---|---|---|---|---|
| python | 0 / 0 / 31 / 0 | 1 / 0 / 34 / 0 | 1 / 0 / 15 / 0 | 15 / 0 / 15 / 0 | vacuous / pass / pass | 0 |
| typescript | 0 / 0 / 30 / 0 | 0 / 0 / 44 / 1 | 0 / 0 / 30 / 0 | 0 / 0 / 22 / 0 | vacuous / silent / vacuous | 22 |
| tsx | 0 / 0 / 7 / 0 | 0 / 0 / 15 / 0 | 0 / 0 / 15 / 0 | 0 / 0 / 15 / 0 | vacuous / vacuous / vacuous | 0 |
| rust | 0 / 0 / 37 / 0 | 0 / 0 / 45 / 0 | 0 / 0 / 30 / 0 | 7 / 0 / 17 / 0 | vacuous / vacuous / vacuous | 17 |
| go | 0 / 0 / 30 / 0 | 0 / 0 / 45 / 0 | 0 / 0 / 15 / 0 | 15 / 0 / 15 / 0 | vacuous / vacuous / vacuous | 0 |
| c | 0 / 0 / 45 / 0 | 0 / 0 / 45 / 0 | 0 / 0 / 15 / 0 | 2 / 0 / 15 / 0 | vacuous / vacuous / vacuous | 0 |
| cpp | 1 / 0 / 40 / 0 | 2 / 0 / 41 / 2 | 14 / 0 / 16 / 0 | 0 / 0 / 30 / 0 | pass / pass / pass | 17 |
| java | 0 / 0 / 35 / 0 | 0 / 0 / 45 / 0 | 15 / 0 / 15 / 0 | 15 / 0 / 15 / 0 | vacuous / vacuous / pass | 0 |
| lua | 1 / 0 / 36 / 0 | 3 / 0 / 41 / 1 | 15 / 0 / 15 / 0 | 15 / 0 / 15 / 0 | pass / pass / pass | 0 |
| r | 0 / 0 / 30 / 0 | 0 / 0 / 28 / 0 | 0 / 0 / 15 / 0 | 8 / 0 / 18 / 1 | vacuous / vacuous / vacuous | 4 |

十语言四类 fp 皆 0。余下的 fn 五条：typescript 类 1 `zod:packages/zod/src/v3/tests/partials.test.ts:152` `requiredObject`（真值争议，
§13 第 27 条）；cpp 类 1 `fmt:test/scan.h:401` `prev_digit` 与 `fmt:include/fmt/core.h:1311` `prev`（do-while 少报）；lua 类 1
`luarocks:src/luarocks/fetch.lua:248` `errcode`（路径不敏感）；r 类 3 `stringr:R/modifiers.R:198` `error_call`（R-1 的代价，顾问）。

**宇宙漂移**（新降表下十语料重导到仓外，与冻结宇宙逐（单元，格）比池计数；单元集合与行段两边相同）：

| 宇宙 | 池动的单元 | 动的格（冻结 → 新） |
|---|---|---|
| cpp-fmt | 188 | 1/B 229 → 226 · 1/C 2344 → 2349 · 2/A 38 → 32 · 2/B 2059 → 2065 · 3/A 286 → 3 · 3/B 2093 → 2375 |
| r-stringr | 3 | 3/A 12 → 8 · 3/B 438 → 442 |
| rust-ripgrep | 6 | 1/C 3399 → 3412 · 2/A 13 → 0 · 2/B 3315 → 3328 · 3/A 8 → 7 · 3/B 1853 → 1854 |
| rust-codeeraser-flow | 3 | 1/C 5218 → 5220 · 2/A 2 → 0 · 2/B 5099 → 5101 · 3/A 1 → 0 · 3/B 5762 → 5763 |
| typescript-zod | 52 | 1/C 4668 → 4627 · 2/A 27 → 0 · 2/B 4527 → 4486 · 3/A 7 → 0 · 3/B 2929 → 2936 |
| python-requests、tsx-zod、go-cobra、c-lua、java-gson、lua-luarocks | 0 | — |

c-lua 有一个单元的访问行数 0 → 2（`ldo.c` 的 `LUAI_TRY`：C 文法把 C++ 的 `try { f(L, ud); }` 读成原型，CPP-2 读出两个形参），
它是 dynamic 单元，不入池。池动的四门（cpp、r、rust、typescript）出第二代考题；其余六门的池一格没动，第一代宇宙 / 样本 / 审阅档照用，
在新降表的干净树上重生成精度册即可。

## 步 4 提交 G：十份精度册重生成、判决掩码重填（2026-09-30）

十份 `flow-precision-<语言>-v<代>.json` 在 F′ 提交 `5023273` 的干净树上逐语言生成到仓外（`CE_FLOW_OUT`）、十份齐了一次拷入，`generated_from` 皆 dirty = false，读的是提交 E 回修后的降表：cpp / r / rust / typescript 读第二代审阅档（`-v2`），其余六门读第一代（`-v1`）。十门 `stage` 翻 `scored`；`flow::judged_mask()` 按档填满十位（Python / TypeScript / Tsx / Rust / Go / C / Cpp / Java / Lua / R，考题表序）。每格 tp / fp / tn / fn：

| 语言 | 档 | 题 | 类 0 | 类 1 | 类 2 | 类 3（顾问） | 门 0 / 1 / 2 | judged |
|---|---|---|---|---|---|---|---|---|
| python | `flow-precision-python-v1.json` | 112 | 0 / 0 / 31 / 0 | 1 / 0 / 34 / 0 | 1 / 0 / 15 / 0 | 15 / 0 / 15 / 0 | vacuous / pass / pass | true |
| typescript | `flow-precision-typescript-v2.json` | 105 | 0 / 0 / 30 / 0 | 0 / 0 / 45 / 0 | 0 / 0 / 15 / 0 | 0 / 0 / 15 / 0 | vacuous / vacuous / vacuous | true |
| tsx | `flow-precision-tsx-v1.json` | 52 | 0 / 0 / 7 / 0 | 0 / 0 / 15 / 0 | 0 / 0 / 15 / 0 | 0 / 0 / 15 / 0 | vacuous / vacuous / vacuous | true |
| rust | `flow-precision-rust-v2.json` | 119 | 0 / 0 / 37 / 0 | 0 / 0 / 45 / 0 | 0 / 0 / 15 / 0 | 7 / 0 / 15 / 0 | vacuous / vacuous / vacuous | true |
| go | `flow-precision-go-v1.json` | 120 | 0 / 0 / 30 / 0 | 0 / 0 / 45 / 0 | 0 / 0 / 15 / 0 | 15 / 0 / 15 / 0 | vacuous / vacuous / vacuous | true |
| c | `flow-precision-c-v1.json` | 122 | 0 / 0 / 45 / 0 | 0 / 0 / 45 / 0 | 0 / 0 / 15 / 0 | 2 / 0 / 15 / 0 | vacuous / vacuous / vacuous | true |
| cpp | `flow-precision-cpp-v2.json` | 134 | 1 / 0 / 40 / 0 | 2 / 0 / 41 / 2 | 15 / 0 / 15 / 0 | 2 / 1 / 15 / 0 | pass / pass / pass | true |
| java | `flow-precision-java-v1.json` | 140 | 0 / 0 / 35 / 0 | 0 / 0 / 45 / 0 | 15 / 0 / 15 / 0 | 15 / 0 / 15 / 0 | vacuous / vacuous / pass | true |
| lua | `flow-precision-lua-v1.json` | 142 | 1 / 0 / 36 / 0 | 3 / 0 / 41 / 1 | 15 / 0 / 15 / 0 | 15 / 0 / 15 / 0 | pass / pass / pass | true |
| r | `flow-precision-r-v2.json` | 96 | 0 / 0 / 30 / 0 | 0 / 0 / 28 / 0 | 0 / 0 / 15 / 0 | 8 / 0 / 15 / 0 | vacuous / vacuous / vacuous | true |

类 0–2 十门误报皆 0；漏报三条都是既有登记（cpp 类 1 两条 = do-while 少报、lua 类 1 一条 = 路径不敏感，见各节与设计册 §13 第 20 / 27 条）；顾问类 3 误报一条（cpp `fmt:include/fmt/chrono.h:1721` 的 `cb`：成员指针调用式 `(w.*cb)(args...)` 的 field 一侧不解析成变量——新登记 CPP-4，§13 第 30 条，只记不门）。G 是精度册的提交：自此到发版，`cli/src/flow/`（`mod.rs` 除外）、`cli/src/scan/{functions,walk,lang}.rs` 与锁文件的钉版不得再动（出处门要 `touched_between(precision, HEAD, ANSWERED_BY)` 为空）；D2 的回放台账行在 G 之后的干净树上量，与精度册同读一份降表。

## 回放台账（步 4 提交 D1 / D2，2026-09-30）

十个语言的第二读数（语料作者自己的编辑拿每条发现怎么办了：消失 = 真阳、挺过编辑 = 误拦，严格与窄两口径并记）记在 [FPR-REPLAY.md](FPR-REPLAY.md) 的
「`flow/1` 逐语言回放」一节，冻结件 `contracts/eval/fpr-flow-v1.json`（D2 已落：十行 `harness.commit` = 5e02af4a、dirty = false）；读数见那一节的三张表，归因只写在那一节。它不是门：准入只读精度册（设计册 §13 第 6 条）。

## 出处

十一份宇宙与十份样本的 `generated_from` 都记 ce 1.8.0、树 `5278e74`、dirty = true：冻结时考题仪器本身还没提交（它和
这二十一份档在同一个提交里落地），与 EVAL-SET-LANGS.md 记 `bac6169` 同理；降表（提交 A）已在 `5278e74` 上，本仓的 Rust 语料
`codeeraser-flow` 钉的正是这个提交。

十份审阅档 `contracts/eval/flow-review-<语言>-v1.json`（提交 B′）的 `generated_from` 记 ce 1.8.0、树 `2ea957d`、dirty = true：归档时审阅仪器本身还没提交；53 批的批次文件、`manifest.json` 与答案文件在仓库外的车道目录里，不入库，答案的每个字经归档逐字进档。

第二代九份档（提交 F：`flow-slice-<键>-v2.json` 五份——cpp-fmt、r-stringr、rust-ripgrep、rust-codeeraser-flow、typescript-zod——与 `flow-sample-<语言>-v2.json` 四份）的 `generated_from` 记 ce 1.8.0、树 `399291d`（提交 E）、dirty = true：生成时测试子仓的代际列与仪器改动还没提交，与提交 B / B′ 同一读法（设计册 §13 第 24 条）；九份先写到仓外、齐了一次拷入。TSX 与 TypeScript 共用 zod 克隆，但 TSX 的池一格没动：同一棵树上把 `tsx-zod` 宇宙重导到仓外，与冻结的第一代逐文件行相同、摘要相同（`generated_from` 之外整档相同），不出第二代、不拷入。第二代四门 20 批 454 道的批次文件与 `manifest.json` 在仓库外的车道目录里，不入库。

十份精度册 `contracts/eval/flow-precision-<语言>-v1.json`（提交 C2）的 `generated_from` 记 ce 1.8.0、树 `bb9bdc8`、dirty = false：C1 提交了仪器与门之后，在那棵干净树上逐语言生成到车道目录、十份齐了一次拷进 `contracts/eval/`，生成期间仓库树里不建任何文件；judged = Python / TSX / Go / C / Java / Lua / R，掩码 `flow::judged_mask()` 按此填入。提交 E 起十份按名退役（删出树、历史里仍在），掩码随之清空。

十份精度册的第二次生成（提交 G：`flow-precision-{cpp,r,rust,typescript}-v2.json` 读第二代审阅档，其余六门 `-v1.json` 读第一代）的 `generated_from` 记 ce 1.8.0、树 `5023273`（F′）、dirty = false：同 C2 的读法，逐语言经 `CE_FLOW_OUT` 生成到仓外、十份齐了一次拷入，生成期间仓库树里不建任何文件；judged 十门皆真，掩码十位全填。

第二代四份审阅档 `contracts/eval/flow-review-<语言>-v2.json`（F′：cpp、r、rust、typescript）的 `generated_from` 记 ce 1.8.0、树 `dc4f4ec`（提交 F）、dirty = false：归档腿逐份写进仓库、写完即挪到车道目录，四份齐了一次拷入，所以每份生成时树是干净的；20 批的批次文件、`manifest.json` 与答案文件在仓库外的车道目录里，不入库，答案的每个字经归档逐字进档。
