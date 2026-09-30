# v2.31 函数内死代码精度考题册（第五次拆册，2026-09-30）

> 计划 v2.31 设计册 [reference/analysis-track.md](reference/analysis-track.md) §5.5「精度考题」段与 §13 第 16 条的冻结登记。
> 每个语言的 `flow/1` 精度照 v2.30 语言考题的仪器走，顺序由提交先后证明，只是抽样之前多一步：**降表与语言表先落（提交 A）→
> 单元宇宙 + 候选池 + 抽样冻结并提交（提交 B，本册第一节）→ 没看过判决的独立代理逐题盲判并提交 → 精度册提交**（顺序门
> `cli/tests/it/flow_provenance.rs`；题是从降出的四表里按类按层抽的，降表不先落地就没有池可抽，所以是 C / C++ 阶梯先于考题的那种
> `ladder_first` 形——盲判的独立性靠题不带答案、盲窗内 `cli/src/flow`（`mod.rs` 除外）零提交、精度册钉在回答它的代码上）。母册链：
> [EVAL-SET.md](EVAL-SET.md) → [EVAL-SET-M5-3.md](EVAL-SET-M5-3.md) → [EVAL-SET-M5-CLOSE.md](EVAL-SET-M5-CLOSE.md) →
> [EVAL-SET-SIMILAR.md](EVAL-SET-SIMILAR.md) → [EVAL-SET-LANGS.md](EVAL-SET-LANGS.md) → 本册。本册与前五册同入冻结集
> （`frozen_set.rs`：不扫芯片、不生成、退出引文门），行号引文一律不写。一个语言在它自己的步里加一节，三步各记一段；
> 重冻结 = 该语言考题的代数加一（考题表的 `generation`：降表多读了一种写法、或换 tip），新一代用新文件名，旧一代的档按名
> 退役并在本册具名记一条；生成器拒绝覆写已冻结的档。

## 仪器与门

- **单元宇宙**（`cli/tests/it/eval_flow_parts/generate.rs` 的 `flow_slice`，`#[ignore]`，读 `.ce-eval/corpora/<名>` 的钉住克隆，
  别的树按名拒绝）：按考题表 `eval_flow_parts::EXAMS` 的扩展名走一遍钉住的树，只取产品自己的走查读的文件（`scan::walk::Scope`，
  语料自己的 ignore 文件与内建排除照读；走查拒读的被追踪文件只计 `walk_refused`，不出题），逐文件把 `scan::functions::extract`
  抽出的每个单元经 `flow::lower::lower_file` 降成四表，记文本 sha256、每个单元的序号 / 名 / 行段 / 形参数 / `dynamic` 位 /
  三张表的行数 / 每个（类，层）格的候选池大小，降不出合法树形的单元按名记 `unlowered` 与原因。池只读降出的表与图例，不问核，
  所以能先于任何判决冻结。档 `contracts/eval/flow-slice-<语言>-<语料>-v<代>.json`（zod 在 TypeScript 与 TSX 两门考题里各出一份，
  同一个克隆目录，只有扩展名表不同）。
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
- **CI 门**（`cli/tests/it/eval_flow.rs`，不跑 git、不要语料克隆）：冻结宇宙的集合恰为考题表的 `<语言>-<语料>` 键（多一份少一份都点名）、
  每份过共用信封（摘要从文件行重算、常量、行序、钉 tip）、语料与范围是考题的、排除只有「其他扩展名 / 走查拒读」两键、每个单元的
  三张表行数非负且池键合法；考题扩展名 = 产品路径表对该语言的扩展名行（`Lang::extensions`）；阶梯常量指向盘上的降表目录与
  它唯一的具名例外（`mod.rs`：模块表与判决掩码是政策不是答案）；样本的每格取数从冻结宇宙的摘要重算、每行两个哈希从自身字段重导、
  身份是考题的（语料在表内、commit = 该语料 tip）、（类，层）合法、(路径，单元) 在宇宙里并回显单元名与行段、行号落在行段内、
  同一单元同一格被抽中的数不超过它冻结行的池、不落在 dynamic 单元上；每份宇宙里 dynamic 单元的池全 0；篡改（伪路径、伪层、伪秩、伪审阅哈希、
  缺一行、伪单元、伪类、调换两行、行数超池、抽中 dynamic 单元）一律拒绝。
- **盲判**与**精度册**：另两个提交（B′ / C）各记一段；门 = 每语言每种非顾问发现（0 / 1 / 2）读四态——`fail`（fp ≥ 1）、
  `pass`（fp = 0 ∧ tp ≥ 1）、`vacuous`（fp = tp = fn = 0：样本里没有正例可找，准入靠负例上的零误报，各节照抄「0 / n 个负例」；零行也归此态）、
  `silent`（fp = tp = 0 ∧ fn ≥ 1：有正例而一个没报，不准入）；三门各 ∈ {pass, vacuous} 的语言进 `flow::judged_mask()`，其余只 observe
  （设计册 §13 第 25 条：原判据让无正例可找的类 fail、却让零行判 vacuous，证据更多反判更差）。
- **盲判仪器**（提交 B′，`cli/tests/it/eval_flow_parts/` 下三件 + 门一件）：`batches.rs` 的批次渲染是冻结样本的纯函数（同一样本
  两次渲染逐字节同），先按语料分组、再按审阅序切成每批至多 25 道，提示模板常量 `PROMPT` 逐字取自判官提示模板
  `audit_prompt_template.md`，每批另写 `manifest.json`（批号、语料、题 id；不入库）；`answers.rs` 读每批一个 `answers-<n>.jsonl`，
  id 不在该批、一题多答或无答、truth 不在该类词表、理由长度越界、多余字段或非 JSON 行，每条拒绝按批按题点名；`review.rs` 逐字归档成
  `contracts/eval/flow-review-<语言>-v<代>.json` 并提供 `verify_review`（行与样本按审阅序一一对应、身份字段逐个相等、批号是批次计划的、
  摘要重算）；`cli/tests/it/eval_flow_review.rs` 五腿门（渲染纯度、合成答案全收、每种拒绝点名、档与考题 stage 同真同假、合成档的篡改）
  外加对每份已归档的档跑六形篡改（外来秩、空理由、缺一行、翻一个 truth、调换两行、伪批号）。判官协议：53 个独立 Opus 子代理、每批一个，
  只读钉住的克隆与自己的批次文件，任何工具的答案都不在场，每题写一行 JSON；十份档共 1,197 行。十份档的 `auditor` 是同一句（各语言的「盲判」节只指到这里），逐字如下：

  > fifty-three independent Opus subagents, one batch each (at most 25 questions, one corpus per batch) in the sample's audit order, reading only the pinned clone under .ce-eval/corpora at its tip and their own batch file - never the product's lowering, no ce, no core, no tool answer anywhere in reach; each answered the four kinds from the source alone under the batch prompt's reading rules (a call that may throw does not end a path; a read is any use of the value, nested closures included; a member write reads the base) and wrote one JSON line per question in the batch's order; the coordinator assembled verbatim, judgments untouched (booklet analysis-track.md section 5.5; RG15); the batch prompt counts a question's nth from 0 while the sample stores it from 1, so every row echoes the sample's nth

  两处口径：批次提示里的 nth 从 0 数，
  档与样本一律从 1 数（档回显样本的 nth）；同一批只装一个语料。审阅档的 `generated_from` 记 ce 1.8.0、树 `2ea957d`、dirty = true，
  与提交 B 的宇宙与样本同一读法：归档工具与这十份档在同一个提交里落地。
- **精度仪器**（提交 C，`cli/tests/it/flow_precision/` 的 `mod.rs` / `flagged.rs` + 门两件；放在 `eval_flow_parts` 之外——那里没有模块
  读回它，不入该目录的导入环）：`flow_precision`
  （`#[ignore]`，`CE_FLOW_LANG=<语言>`，要 `CE_CORE_BIN`）对审阅档的每道题，在钉住的 tip 上取题所在的文件、经 `flow::lower::lower_file`
  降表、整文件经 `flow::wire::judge` 送真核（一条链路、每文件一次请求）；题按它被抽出时的池项回映——`pools.rs` 的池项带它代表的语句 seq
  与变量 v，同一个锚、不另推一遍：类 0 = 有一段不可达覆盖该语句、类 1 = 核点名该（写，变量）、类 2 / 3 = 核在该类点名该变量。单元未降出、
  被核拒或 dynamic 答 `unjudged`，逐单元记原因（`unjudged_reasons`）；抽样单元的名或行段与降表不符即按名停（抽样与降表不是同一棵树）。
  判词由（真值，答案）重算：真值正 = `unreachable` / `dead` / `unread`，`cannot_tell` 不入率；`per_kind` 每类记 tp / fp / tn / fn /
  unjudged / cannot_tell、`positives` / `negatives` 与 precision / recall 两个整数对，`gate` 读类 0 / 1 / 2 的四态，`judged` = 三门各 ∈
  {pass, vacuous}。档 `contracts/eval/flow-precision-<语言>-v<代>.json`（`ce.eval-flow-precision/1.0.0`；`CE_FLOW_OUT=<目录>` 改写出目录；
  拒绝覆写；`CE_FLOW_PRECISION_DRY=1` 只印读数不写档；写档前要求 `generated_from.dirty` = false）。门 `cli/tests/it/eval_flow_precision.rs`
  （不跑 git、不要克隆与核）：档与考题 stage 同真同假（`scored` 才在盘上），已归档的逐行对审阅档重算并跑六形篡改（翻判词、翻答案、伪门、
  伪 `judged`、缺一行、把 silent 读成 vacuous——档里没有 silent 时，把某个有正例的类的真答案全改假、计数全重算、只让门写 vacuous）；
  `flow::judged_mask()` 的每一位 ⇔ 该语言精度册 `judged`（tsx 与 typescript 各一位）；四态在手写计数上钉住；篡改电池另在门自己的合成档
  （python / rust）上先跑。出处门 `cli/tests/it/flow_provenance.rs`（跑 git，浅克隆拒）：三档 `generated_from` 的提交都在本历史上且严格先后；
  降表的首个提交是抽样提交的祖先或就是它，抽样到审阅之间降表零提交；精度册从自己的提交起到 HEAD，降表与 `scan/functions.rs` /
  `scan/walk.rs` / `scan/lang.rs` 无提交、工作树无未提交改动、`cli/Cargo.lock` 按钉版不动；反向探针两条（首个提交在抽样之后的路径、
  盲窗内动过的 `scan/lang.rs`）各按自己的句子红。读数见各语言「精度（2026-09-30）」一节（提交 C2）。

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

### 精度（2026-09-30）

`flow-precision-python-v1.json`：112 道，生成于 C1 提交 `bb9bdc8` 的干净树（dirty = false）；判词与四态见「仪器与门」。

| 类 | tp | fp | tn | fn | unjudged | cannot_tell | 正例 | 负例 | precision | recall |
|---|---|---|---|---|---|---|---|---|---|---|
| 0 不可达 | 0 | 0 | 31 | 0 | 0 | 0 | 0 | 31 | — | — |
| 1 死存储 | 1 | 0 | 34 | 0 | 0 | 0 | 1 | 34 | 1 / 1 | 1 / 1 |
| 2 未用局部量 | 1 | 0 | 15 | 0 | 0 | 0 | 1 | 15 | 1 / 1 | 1 / 1 |
| 3 未用形参 | 15 | 0 | 15 | 0 | 0 | 0 | 15 | 15 | 15 / 15 | 15 / 15 |

门：类 0 vacuous（0 / 31 个负例） · 类 1 pass（1 / 1） · 类 2 pass（1 / 1）；`judged` = true（类 3 顾问只记不判）。

误报（类 0–2）0 条、漏报（类 0–2）0 条、顾问误报（类 3）0 条、unjudged 0 道。

## TypeScript（步 4 提交 B）

### 宇宙与抽样（2026-09-30 冻结）

范围 = `*.ts *.mts *.cts`；排除计「其他扩展名 / 走查拒读」；层的定义见「候选池」一段：

| 语料 | tip | 许可 | 文件 | 单元 | dynamic | unlowered | 排除 |
|---|---|---|---|---|---|---|---|
| colinhacks/zod | `912f0f5` | MIT | 375 | 6,356 | 0 | 0 | other_extension 208 |

| 类 | 层 A 池 → 取数 | 层 B 池 → 取数 | 层 C 池 → 取数 |
|---|---|---|---|
| 0 不可达 | 0 → 0 | 1,000 → 15 | 467 → 15 |
| 1 死存储 | 59 → 15 | 41 → 15 | 4,668 → 15 |
| 2 未用局部量 | 27 → 15 | 4,527 → 15 | — |
| 3 未用形参 | 7 → 7 | 2,929 → 15 | — |

样本共 127 道，落在 61 个文件、99 个单元上。

### 盲判（2026-09-30）

127 道分 6 批（每批一个语料、至多 25 道，按语料 zod 127），逐字归档自各批答案文件；审阅者句见「仪器与门」（十份档的 `auditor` 是同一句，只在那里逐字引一次）。

| 类 | 发现词 | 反面词 | cannot_tell |
|---|---|---|---|
| 0 不可达 | unreachable 0 | reachable 30 | 0 |
| 1 死存储 | dead 1 | live 44 | 0 |
| 2 未用局部量 | unread 0 | read 30 | 0 |
| 3 未用形参 | unread 0 | read 22 | 0 |

`cannot_tell` 0 道。

notes：无。

### 精度（2026-09-30）

`flow-precision-typescript-v1.json`：127 道，生成于 C1 提交 `bb9bdc8` 的干净树（dirty = false）；判词与四态见「仪器与门」。

| 类 | tp | fp | tn | fn | unjudged | cannot_tell | 正例 | 负例 | precision | recall |
|---|---|---|---|---|---|---|---|---|---|---|
| 0 不可达 | 0 | 0 | 30 | 0 | 0 | 0 | 0 | 30 | — | — |
| 1 死存储 | 0 | 0 | 44 | 1 | 0 | 0 | 1 | 44 | — | 0 / 1 |
| 2 未用局部量 | 0 | 15 | 15 | 0 | 0 | 0 | 0 | 30 | 0 / 15 | — |
| 3 未用形参 | 0 | 7 | 15 | 0 | 0 | 0 | 0 | 22 | 0 / 7 | — |

门：类 0 vacuous（0 / 30 个负例） · 类 1 silent（fn 1） · 类 2 fail（fp 15）；`judged` = false（类 3 顾问只记不判）。

误报（类 0–2）15 条，归因：降表缺陷（猜：后绑定只给 Python / R / TS `var`，见设计册拍板记录的 A2 裁定）——`const` / `let` 的名字在它自己的初始化式里、或在更早声明的闭包 / 对象字面量 getter 里被读，读先于声明降表、解析不到这个变量，既没记读也没标 `captured`：
- 类 2 层 A `zod:packages/zod/src/v4/mini/tests/recursive-types.test.ts:246` `(anonymous)` 变量 `I`：I is mentioned by the nested getter on line 248 `return z.optional(I);`, a closure that reads it.
- 类 2 层 A `zod:packages/zod/src/v4/mini/tests/recursive-types.test.ts:258` `(anonymous)` 变量 `L`：L is mentioned by the nested getter on line 260 `return z.optional(L);`, a closure that reads it.
- 类 2 层 A `zod:packages/zod/src/v4/classic/tests/recursive-types.test.ts:435` `(anonymous)` 变量 `A`：The getters nested in the object literal mention A, e.g. 'get array() { return A.array(); }' on lines 436-437.
- 类 2 层 A `zod:packages/zod/src/v4/mini/tests/recursive-types.test.ts:216` `(anonymous)` 变量 `D`：The nested getter reads D: 'return z.union([D, z.string()]);' on line 218.
- 类 2 层 A `zod:packages/zod/src/v4/classic/tests/lazy.test.ts:212` `(anonymous)` 变量 `c`：The closure in its own initializer reads c: 'const c: any = z.lazy(() => c).default({} as any);'.
- 类 2 层 A `zod:packages/zod/src/v4/classic/tests/lazy.test.ts:216` `(anonymous)` 变量 `g`：The closure reads g via a shorthand property: 'const g: any = z.lazy(() => z.object({ g })).readonly();'.
- 类 2 层 A `zod:packages/zod/src/v4/classic/tests/lazy.test.ts:211` `(anonymous)` 变量 `b`：`const b: any = z.lazy(() => b).nullable();` - the closure `() => b` mentions b, which counts as a read.
- 类 2 层 A `zod:packages/zod/src/v4/mini/tests/recursive-types.test.ts:234` `(anonymous)` 变量 `G`：G is mentioned in the nested getter on line 236, return z.map(z.string(), G);, and a closure mentioning G counts as a read.
- 类 2 层 A `zod:packages/zod/src/v4/classic/tests/recursive-types.test.ts:405` `(anonymous)` 变量 `Node`：Node is mentioned in nested getters of the same function, e.g. line 356 return z.array(Node).optional(); and line 385 return Node.optional();
- 类 2 层 A `zod:packages/zod/src/v4/mini/tests/recursive-types.test.ts:252` `(anonymous)` 变量 `J`：J is mentioned in the nested getter on line 254, return z.nullable(J);, and a closure mentioning J counts as a read.
- 类 2 层 A `zod:packages/zod/src/v4/classic/tests/lazy.test.ts:210` `(anonymous)` 变量 `a`：a is mentioned in the closure on line 210, const a: any = z.lazy(() => a).optional();, and a closure mentioning a counts as a read.
- 类 2 层 A `zod:packages/zod/src/v4/classic/tests/lazy.test.ts:224` `(anonymous)` 变量 `categorySchema`：categorySchema is mentioned in the closure on line 225, z.lazy(() => categorySchema.array()), and a closure mentioning it counts as a read.
- 类 2 层 A `zod:packages/zod/src/v4/classic/tests/to-json-schema.test.ts:1794` `(anonymous)` 变量 `FileSchema`：FileSchema is read by the nested getter of FolderSchema on line 1790: `return z.array(FileSchema);`.
- 类 2 层 A `zod:packages/zod/src/v4/classic/tests/to-json-schema.test.ts:3104` `(anonymous)` 变量 `B`：B is read by A's nested getter on line 3100: `return z.array(B);`.
- 类 2 层 A `zod:packages/zod/src/v4/classic/tests/lazy.test.ts:215` `(anonymous)` 变量 `f`：f is mentioned by the closure on its own line: `const f: any = z.lazy(() => f).catch({} as any);`, which reads it.

漏报（类 0–2）1 条，归因：真值与产品读法之差：类型位置的 `typeof requiredObject` 被降成一次读（偏安全侧、少报），判官按编译期擦除判 dead；记录，不改真值：
- 类 1 层 C `zod:packages/zod/src/v3/tests/partials.test.ts:152` `(anonymous)` 变量 `requiredObject`：Only later mention is `type required = z.infer<typeof requiredObject>;` (line 154), a type-level query erased at compile time; the value is never read before the function ends.

顾问误报（类 3）7 条，归因：降表缺陷（猜）：构造器的参数属性（`public x: T`）隐含 `this.x = x`，这次读没降：
- 类 3 层 A `zod:packages/bench/instanceof.ts:10` `constructor` 变量 `value`：'constructor(public value: string) {}' is a parameter property: the language assigns this.value = value, reading the parameter.
- 类 3 层 A `zod:packages/zod/src/v4/classic/tests/instanceof.test.ts:8` `constructor` 变量 `val`：`constructor(public val: string) {}` is a parameter property: TS emits `this.val = val`, so val's value is stored and later read via `bar.val` (line 22).
- 类 3 层 A `zod:packages/bench/metabench.ts:69` `constructor` 变量 `name`：`public name: string` is a parameter property: TS emits `this.name = name`, and this.name is read elsewhere (e.g. `this.name` in run()).
- 类 3 层 A `zod:packages/bench/metabench.ts:70` `constructor` 变量 `benchmarks`：public benchmarks: Benchmarks<D> (line 70) is a TypeScript parameter property, so the constructor implicitly runs this.benchmarks = benchmarks, reading its value.
- 类 3 层 A `zod:packages/bench/object-creation.ts:5` `constructor` 变量 `value`：`constructor(public value: string) {}` declares a parameter property, which implicitly assigns this.value = value, reading the parameter.
- 类 3 层 A `zod:packages/bench/safe.ts:6` `constructor` 变量 `value`：`constructor(public value: string) {` declares a parameter property, which implicitly assigns this.value = value, reading the parameter.
- 类 3 层 A `zod:packages/zod/src/v3/tests/instanceof.test.ts:11` `constructor` 变量 `val`：`constructor(public val: string) {}` is a parameter property that assigns this.val = val, read back by `expect(bar.val).toEqual("asdf")` on line 25.

unjudged 0 道。

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

### 精度（2026-09-30）

`flow-precision-tsx-v1.json`：52 道，生成于 C1 提交 `bb9bdc8` 的干净树（dirty = false）；判词与四态见「仪器与门」。

| 类 | tp | fp | tn | fn | unjudged | cannot_tell | 正例 | 负例 | precision | recall |
|---|---|---|---|---|---|---|---|---|---|---|
| 0 不可达 | 0 | 0 | 7 | 0 | 0 | 0 | 0 | 7 | — | — |
| 1 死存储 | 0 | 0 | 15 | 0 | 0 | 0 | 0 | 15 | — | — |
| 2 未用局部量 | 0 | 0 | 15 | 0 | 0 | 0 | 0 | 15 | — | — |
| 3 未用形参 | 0 | 0 | 15 | 0 | 0 | 0 | 0 | 15 | — | — |

门：类 0 vacuous（0 / 7 个负例） · 类 1 vacuous（0 / 15 个负例） · 类 2 vacuous（0 / 15 个负例）；`judged` = true（类 3 顾问只记不判）。

误报（类 0–2）0 条、漏报（类 0–2）0 条、顾问误报（类 3）0 条、unjudged 0 道。

## Rust（步 4 提交 B）

### 宇宙与抽样（2026-09-30 冻结）

范围 = `*.rs`；排除计「其他扩展名 / 走查拒读」；层的定义见「候选池」一段：

| 语料 | tip | 许可 | 文件 | 单元 | dynamic | unlowered | 排除 |
|---|---|---|---|---|---|---|---|
| BurntSushi/ripgrep | `3fce3b5` | Unlicense OR MIT | 110 | 3,345 | 0 | 0 | other_extension 127 |
| skymanbp/CodeEraser（本仓，克隆名 `codeeraser-flow`） | `5278e74` | Apache-2.0 | 361 | 4,619 | 0 | 0 | other_extension 548 / walk_refused 6 |

| 类 | 层 A 池 → 取数 | 层 B 池 → 取数 | 层 C 池 → 取数 |
|---|---|---|---|
| 0 不可达 | 7 → 7 | 1,432 → 15 | 1,034 → 15 |
| 1 死存储 | 36 → 15 | 61 → 15 | 8,617 → 15 |
| 2 未用局部量 | 15 → 15 | 8,414 → 15 | — |
| 3 未用形参 | 9 → 9 | 7,615 → 15 | — |

样本共 136 道，落在 91 个文件、118 个单元上，按语料分是 ripgrep 59 / codeeraser-flow 77。

### 盲判（2026-09-30）

136 道分 7 批（每批一个语料、至多 25 道，按语料 ripgrep 59 / codeeraser-flow 77），逐字归档自各批答案文件；审阅者句见「仪器与门」（十份档的 `auditor` 是同一句，只在那里逐字引一次）。

| 类 | 发现词 | 反面词 | cannot_tell |
|---|---|---|---|
| 0 不可达 | unreachable 0 | reachable 37 | 0 |
| 1 死存储 | dead 0 | live 45 | 0 |
| 2 未用局部量 | unread 0 | read 30 | 0 |
| 3 未用形参 | unread 7 | read 17 | 0 |

`cannot_tell` 0 道。

notes：无。

### 精度（2026-09-30）

`flow-precision-rust-v1.json`：136 道，生成于 C1 提交 `bb9bdc8` 的干净树（dirty = false）；判词与四态见「仪器与门」。

| 类 | tp | fp | tn | fn | unjudged | cannot_tell | 正例 | 负例 | precision | recall |
|---|---|---|---|---|---|---|---|---|---|---|
| 0 不可达 | 0 | 0 | 37 | 0 | 0 | 0 | 0 | 37 | — | — |
| 1 死存储 | 0 | 0 | 45 | 0 | 0 | 0 | 0 | 45 | — | — |
| 2 未用局部量 | 0 | 15 | 15 | 0 | 0 | 0 | 0 | 30 | 0 / 15 | — |
| 3 未用形参 | 7 | 2 | 15 | 0 | 0 | 0 | 7 | 17 | 7 / 9 | 7 / 7 |

门：类 0 vacuous（0 / 37 个负例） · 类 1 vacuous（0 / 45 个负例） · 类 2 fail（fp 15）；`judged` = false（类 3 顾问只记不判）。

误报（类 0–2）15 条，归因：降表缺陷：格式宏（`write!` / `writeln!` / `format!` / `unreachable!`）格式串里的内联捕获 `{name}` / `{:pad$}` 没降成读（宏实参按记号树读，字符串字面量内部不看）：
- 类 2 层 A `ripgrep:crates/core/flags/doc/help.rs:76` `generate_short_flag` 变量 `name`：`let name = char::from(byte);` is read by the inline format capture on line 77, `write!(col1, r"-{name}");`.
- 类 2 层 A `ripgrep:crates/core/flags/doc/man.rs:56` `generate_flag` 变量 `var`：The binding from `if let Some(var) = flag.doc_variable()` is read by the inline capture on line 57, `write!(out, r" \fI{var}\fP");`.
- 类 2 层 A `ripgrep:crates/core/flags/doc/man.rs:64` `generate_flag` 变量 `var`：The binding from `if let Some(var) = flag.doc_variable()` on line 64 is read by the inline capture on line 65, `write!(out, r"=\fI{var}\fP");`.
- 类 2 层 A `ripgrep:crates/core/flags/doc/help.rs:161` `generate_long_flag` 变量 `name`：`let name = flag.name_long();` is read by the inline capture on line 162, `write!(out, r"--{name}");`.
- 类 2 层 A `ripgrep:crates/core/flags/doc/help.rs:81` `generate_short_flag` 变量 `var`：The var bound by `if let Some(var) = var.as_ref() {` is read by the inline capture on line 82, `write!(col1, r"={var}");`.
- 类 2 层 A `ripgrep:crates/core/flags/doc/help.rs:153` `generate_long_flag` 变量 `var`：Bound by `if let Some(var) = flag.doc_variable()`; the format string of the write! on line 154 captures `{var}`, reading it.
- 类 2 层 A `codeeraser-flow:cli/src/progress.rs:195` `paint` 变量 `pad`：pad is read as the captured named width `{:pad$}` in the format string of `write!(e, ...)` on line 197.
- 类 2 层 A `ripgrep:crates/core/flags/doc/man.rs:62` `generate_flag` 变量 `name`：`let name = flag.name_long().replace(...)` is read by the `{name}` inline capture in the `write!(out, ...)` on line 63.
- 类 2 层 A `ripgrep:crates/core/flags/doc/help.rs:151` `generate_long_flag` 变量 `name`：`let name = char::from(byte);` is read by the `{name}` inline capture in the `write!(out, ...)` on line 152.
- 类 2 层 A `ripgrep:crates/core/flags/doc/man.rs:54` `generate_flag` 变量 `name`：`let name = char::from(byte);` is read by the `{name}` inline capture in the `write!(out, ...)` on line 55.
- 类 2 层 A `ripgrep:crates/core/flags/doc/help.rs:163` `generate_long_flag` 变量 `var`：Bound by `if let Some(var) = flag.doc_variable()`; the format string of the write! on line 164 captures `{var}`, reading it.
- 类 2 层 A `ripgrep:crates/core/flags/doc/man.rs:102` `generate_flag` 变量 `negated`：Bound by `if let Some(negated) = flag.name_negated()`; the `writeln!` on lines 109-112 captures `{negated}` in its format string.
- 类 2 层 A `codeeraser-flow:cli/src/report.rs:169` `render` 变量 `k`：`for (k, val) in v.as_object()...` binds k, interpolated by `out = out.replace(&format!("{{{k}}}"), &s);`.
- 类 2 层 A `ripgrep:crates/core/flags/doc/help.rs:184` `(anonymous)` 变量 `name`：Line 184 binds name via `let Some(name) = flag.name_negated() else {`; line 191 write! uses the format string --{name}, an inline format argument that reads name.
- 类 2 层 A `ripgrep:crates/core/flags/doc/man.rs:90` `(anonymous)` 变量 `name`：Line 90 binds name via `let Some(name) = flag.name_negated() else {`; line 98 write! uses a format string ending in {name}, an inline format argument that reads name.

顾问误报（类 3）2 条，归因：同上：格式串内联捕获没降成读：
- 类 3 层 A `codeeraser-flow:cli/src/report.rs:86` `(anonymous)` 变量 `k`：In `.filter(|(k, _)| !summary.contains(&format!("{{{k}}}")))` the inner `{k}` is an inline format capture that reads k.
- 类 3 层 A `ripgrep:crates/core/flags/doc/mod.rs:20` `render_custom_markup` 变量 `tag`：tag is read through the inline format capture in `let tag_prefix = format!(r"\{tag}{{");`.

漏报（类 0–2）0 条、unjudged 0 道。

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

### 精度（2026-09-30）

`flow-precision-go-v1.json`：120 道，生成于 C1 提交 `bb9bdc8` 的干净树（dirty = false）；判词与四态见「仪器与门」。

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

### 精度（2026-09-30）

`flow-precision-c-v1.json`：122 道，生成于 C1 提交 `bb9bdc8` 的干净树（dirty = false）；判词与四态见「仪器与门」。

| 类 | tp | fp | tn | fn | unjudged | cannot_tell | 正例 | 负例 | precision | recall |
|---|---|---|---|---|---|---|---|---|---|---|
| 0 不可达 | 0 | 0 | 45 | 0 | 0 | 0 | 0 | 45 | — | — |
| 1 死存储 | 0 | 0 | 45 | 0 | 0 | 0 | 0 | 45 | — | — |
| 2 未用局部量 | 0 | 0 | 15 | 0 | 0 | 0 | 0 | 15 | — | — |
| 3 未用形参 | 2 | 0 | 15 | 0 | 0 | 0 | 2 | 15 | 2 / 2 | 2 / 2 |

门：类 0 vacuous（0 / 45 个负例） · 类 1 vacuous（0 / 45 个负例） · 类 2 vacuous（0 / 15 个负例）；`judged` = true（类 3 顾问只记不判）。

误报（类 0–2）0 条、漏报（类 0–2）0 条、顾问误报（类 3）0 条、unjudged 0 道。

## C++（步 4 提交 B）

### 宇宙与抽样（2026-09-30 冻结）

范围 = `*.cpp *.cc *.cxx *.hpp *.hh *.hxx *.h *.inl`；排除计「其他扩展名 / 走查拒读」；层的定义见「候选池」一段：

| 语料 | tip | 许可 | 文件 | 单元 | dynamic | unlowered | 排除 |
|---|---|---|---|---|---|---|---|
| fmtlib/fmt | `6d71f74` | MIT | 73 | 4,654 | 550 | 0 | other_extension 72 |

| 类 | 层 A 池 → 取数 | 层 B 池 → 取数 | 层 C 池 → 取数 |
|---|---|---|---|
| 0 不可达 | 11 → 11 | 532 → 15 | 498 → 15 |
| 1 死存储 | 95 → 15 | 229 → 15 | 2,344 → 15 |
| 2 未用局部量 | 38 → 15 | 2,059 → 15 | — |
| 3 未用形参 | 286 → 15 | 2,093 → 15 | — |

样本共 146 道，落在 27 个文件、133 个单元上。

### 盲判（2026-09-30）

146 道分 6 批（每批一个语料、至多 25 道，按语料 fmt 146），逐字归档自各批答案文件；审阅者句见「仪器与门」（十份档的 `auditor` 是同一句，只在那里逐字引一次）。

| 类 | 发现词 | 反面词 | cannot_tell |
|---|---|---|---|
| 0 不可达 | unreachable 1 | reachable 40 | 0 |
| 1 死存储 | dead 4 | live 41 | 0 |
| 2 未用局部量 | unread 14 | read 16 | 0 |
| 3 未用形参 | unread 0 | read 30 | 0 |

`cannot_tell` 0 道。

notes：无。

### 精度（2026-09-30）

`flow-precision-cpp-v1.json`：146 道，生成于 C1 提交 `bb9bdc8` 的干净树（dirty = false）；判词与四态见「仪器与门」。

| 类 | tp | fp | tn | fn | unjudged | cannot_tell | 正例 | 负例 | precision | recall |
|---|---|---|---|---|---|---|---|---|---|---|
| 0 不可达 | 1 | 0 | 40 | 0 | 0 | 0 | 1 | 40 | 1 / 1 | 1 / 1 |
| 1 死存储 | 2 | 1 | 40 | 2 | 0 | 0 | 4 | 41 | 2 / 3 | 2 / 4 |
| 2 未用局部量 | 14 | 1 | 15 | 0 | 0 | 0 | 14 | 16 | 14 / 15 | 14 / 14 |
| 3 未用形参 | 0 | 15 | 15 | 0 | 0 | 0 | 0 | 30 | 0 / 15 | — |

门：类 0 pass（1 / 1） · 类 1 fail（fp 1） · 类 2 fail（fp 1）；`judged` = false（类 3 顾问只记不判）。

误报（类 0–2）2 条：

归因：降表缺陷（猜）：`T x(args);` 形的直接初始化声明，实参没降成读（`data_to_string format_str(data, size);`）：
- 类 1 层 B `fmt:test/fuzzing/chrono-timepoint.cc:19` `doit` 变量 `data`：'data += N;' is followed by 'data_to_string format_str(data, size);' on line 21, which passes data as an argument.

归因：同上：`mock_buffer<char> buffer(data, sizeof(data));` 的实参没降成读：
- 类 2 层 A `fmt:test/core-test.cc:208` `TEST` 变量 `data`：`mock_buffer<char> buffer(data, sizeof(data));` passes data (decayed to a pointer) as a constructor argument.

漏报（类 0–2）2 条，归因：A2 裁定（设计册拍板记录）的已知少报：do-while 的条件挂在循环结点上，体可跳过的多出路径让循环前的写读成活：
- 类 1 层 A `fmt:test/scan.h:401` `read` 变量 `prev_digit`：`char prev_digit = c;` is overwritten by `prev_digit = c;` on line 405 in the do-while body, which always runs first; lines 403-404 do not read prev_digit.
- 类 1 层 A `fmt:include/fmt/core.h:1311` `parse_nonnegative_int` 变量 `prev`：`unsigned value = 0, prev = 0;` is overwritten by `prev = value;` at the top of the do-while body (line 1314) before any read; line 1312 does not read prev.

顾问误报（类 3）15 条，归因：降表缺陷（猜）：构造器的成员初始化列表（`: m_(x)`、基类初始化）与 `T x(args);` 直接初始化的实参没降成读：
- 类 3 层 A `fmt:include/fmt/core.h:855` `parse_context::parse_context` 变量 `fmt`：fmt is read in the member initializer ': fmt_(fmt), next_arg_id_(next_arg_id) {}' on line 857.
- 类 3 层 A `fmt:include/fmt/os.h:350` `ostream_params::ostream_params` 变量 `new_oflag`：The member initializer in `ostream_params(int new_oflag) : oflag(new_oflag) {}` uses new_oflag's value to initialize oflag.
- 类 3 层 A `fmt:test/gtest/gmock/gmock.h:5497` `QuantifierMatcherImpl::QuantifierMatcherImpl` 变量 `inner_matcher`：The member initializer `inner_matcher_(testing::SafeMatcherCast<const Element&>(inner_matcher))` passes inner_matcher as a call argument.
- 类 3 层 A `fmt:test/gtest/gmock/gmock.h:4488` `FloatingEqMatcher::Impl::Impl` 变量 `expected`：expected is read in the member initializer on line 4489: `: expected_(expected),`.
- 类 3 层 A `fmt:test/gtest/gtest/gtest.h:6905` `MatchesRegexMatcher::MatchesRegexMatcher` 变量 `regex`：regex is read in the member initializer on line 6906: `: regex_(regex), full_match_(full_match) {}`.
- 类 3 层 A `fmt:test/gtest/gtest/gtest.h:1974` `GTestMutexLock::GTestMutexLock` 变量 `mutex`：mutex is read in the member initializer on line 1975: `: mutex_(mutex) { mutex_->Lock(); }`.
- 类 3 层 A `fmt:test/gtest/gmock/gmock.h:4939` `PropertyMatcher::PropertyMatcher` 变量 `property`：property is read in the member initializer on line 4940: `: property_(property),`.
- 类 3 层 A `fmt:test/gtest/gmock/gmock.h:5830` `PairMatcher::PairMatcher` 变量 `first_matcher`：first_matcher is read in the member initializer on line 5831: `: first_matcher_(first_matcher), second_matcher_(second_matcher) {}`.
- 类 3 层 A `fmt:include/fmt/compile.h:232` `spec_field::format` 变量 `out`：`basic_format_context<OutputIt, Char> ctx(out, vargs);` passes out as a constructor argument.
- 类 3 层 A `fmt:test/gtest/gmock-gtest-all.cc:8883` `WindowsDeathTest::WindowsDeathTest` 变量 `a_statement`：a_statement is passed to the base initializer `DeathTestImpl(a_statement, std::move(matcher))`.
- 类 3 层 A `fmt:include/fmt/color.h:218` `color_type::color_type` 变量 `term_color`：term_color is read in the member initializer `value_(static_cast<uint32_t>(term_color) | (3 << 24))` on line 219.
- 类 3 层 A `fmt:include/fmt/core.h:1955` `iterator_buffer::iterator_buffer` 变量 `out`：out is read in the member initializer list `out_(out)` on line 1956.
- 类 3 层 A `fmt:include/fmt/chrono.h:1249` `tm_writer::tm_writer` 变量 `out`：out is read in the member initializer `out_(out),` on line 1254.
- 类 3 层 A `fmt:include/fmt/os.h:351` `ostream_params::ostream_params` 变量 `bs`：`ostream_params(detail::buffer_size bs) : buffer_size(bs.value) {}` reads bs through the member access in the initializer.
- 类 3 层 A `fmt:test/gtest/gmock/gmock.h:9118` `TypedExpectation::TypedExpectation` 变量 `a_line`：The mem-initializer `: ExpectationBase(a_file, a_line, a_source_text)` passes a_line to the base-class constructor.

unjudged 0 道。

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

### 精度（2026-09-30）

`flow-precision-java-v1.json`：140 道，生成于 C1 提交 `bb9bdc8` 的干净树（dirty = false）；判词与四态见「仪器与门」。

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

### 精度（2026-09-30）

`flow-precision-lua-v1.json`：142 道，生成于 C1 提交 `bb9bdc8` 的干净树（dirty = false）；判词与四态见「仪器与门」。

| 类 | tp | fp | tn | fn | unjudged | cannot_tell | 正例 | 负例 | precision | recall |
|---|---|---|---|---|---|---|---|---|---|---|
| 0 不可达 | 1 | 0 | 36 | 0 | 0 | 0 | 1 | 36 | 1 / 1 | 1 / 1 |
| 1 死存储 | 3 | 0 | 41 | 1 | 0 | 0 | 4 | 41 | 3 / 3 | 3 / 4 |
| 2 未用局部量 | 15 | 0 | 15 | 0 | 0 | 0 | 15 | 15 | 15 / 15 | 15 / 15 |
| 3 未用形参 | 15 | 0 | 15 | 0 | 0 | 0 | 15 | 15 | 15 / 15 | 15 / 15 |

门：类 0 pass（1 / 1） · 类 1 pass（3 / 3） · 类 2 pass（15 / 15）；`judged` = true（类 3 顾问只记不判）。

漏报（类 0–2）1 条，归因：路径不相关：两处 `if not file` 的条件相关性不进控制流图，`errcode` 沿不可行的路径读成活；判决本就路径不敏感，记录：
- 类 1 层 A `luarocks:src/luarocks/fetch.lua:248` `fetch.fetch_url_at_temp_dir` 变量 `errcode`：If cachefile is set, file is set and the function returns file, temp_dir without reading errcode; otherwise line 257 `file, err, errcode = fetch.fetch_url(...)` overwrites it.

误报（类 0–2）0 条、顾问误报（类 3）0 条、unjudged 0 道。

## R（步 4 提交 B）

### 宇宙与抽样（2026-09-30 冻结）

范围 = `*.R *.r`；排除计「其他扩展名 / 走查拒读」；层的定义见「候选池」一段：

| 语料 | tip | 许可 | 文件 | 单元 | dynamic | unlowered | 排除 |
|---|---|---|---|---|---|---|---|
| tidyverse/stringr | `ae054b1` | MIT | 67 | 200 | 5 | 0 | other_extension 114 |

| 类 | 层 A 池 → 取数 | 层 B 池 → 取数 | 层 C 池 → 取数 |
|---|---|---|---|
| 0 不可达 | 0 → 0 | 69 → 15 | 30 → 15 |
| 1 死存储 | 12 → 12 | 1 → 1 | 196 → 15 |
| 2 未用局部量 | 0 → 0 | 143 → 15 | — |
| 3 未用形参 | 12 → 12 | 438 → 15 | — |

样本共 100 道，落在 20 个文件、58 个单元上。

### 盲判（2026-09-30）

100 道分 4 批（每批一个语料、至多 25 道，按语料 stringr 100），逐字归档自各批答案文件；审阅者句见「仪器与门」（十份档的 `auditor` 是同一句，只在那里逐字引一次）。

| 类 | 发现词 | 反面词 | cannot_tell |
|---|---|---|---|
| 0 不可达 | unreachable 0 | reachable 30 | 0 |
| 1 死存储 | dead 0 | live 28 | 0 |
| 2 未用局部量 | unread 0 | read 15 | 0 |
| 3 未用形参 | unread 9 | read 18 | 0 |

`cannot_tell` 0 道。

notes：无。

### 精度（2026-09-30）

`flow-precision-r-v1.json`：100 道，生成于 C1 提交 `bb9bdc8` 的干净树（dirty = false）；判词与四态见「仪器与门」。

| 类 | tp | fp | tn | fn | unjudged | cannot_tell | 正例 | 负例 | precision | recall |
|---|---|---|---|---|---|---|---|---|---|---|
| 0 不可达 | 0 | 0 | 30 | 0 | 0 | 0 | 0 | 30 | — | — |
| 1 死存储 | 0 | 0 | 28 | 0 | 0 | 0 | 0 | 28 | — | — |
| 2 未用局部量 | 0 | 0 | 15 | 0 | 0 | 0 | 0 | 15 | — | — |
| 3 未用形参 | 9 | 3 | 15 | 0 | 0 | 0 | 9 | 18 | 9 / 12 | 9 / 9 |

门：类 0 vacuous（0 / 30 个负例） · 类 1 vacuous（0 / 28 个负例） · 类 2 vacuous（0 / 15 个负例）；`judged` = true（类 3 顾问只记不判）。

顾问误报（类 3）3 条，归因：降表缺陷（猜）：`NextMethod()` / `UseMethod()` 隐式转发形参，这次读没降：
- 类 3 层 A `stringr:R/modifiers.R:245` ``[.stringr_pattern`` 变量 `i`：Line 247 `NextMethod()` forwards the formals x and i as promises evaluated in this frame to the default `[` method, which uses i as the index.
- 类 3 层 A `stringr:R/modifiers.R:254` ``[[.stringr_pattern`` 变量 `i`：Line 256 `NextMethod()` forwards the formals x and i as promises evaluated in this frame to the default `[[` method, which uses i as the index.
- 类 3 层 A `stringr:R/modifiers.R:198` `type` 变量 `x`：`UseMethod("type")` (line 199) dispatches on the class of the enclosing function's first argument x, so x is evaluated and passed on to the method.

误报（类 0–2）0 条、漏报（类 0–2）0 条、unjudged 0 道。

## 出处

十一份宇宙与十份样本的 `generated_from` 都记 ce 1.8.0、树 `5278e74`、dirty = true：冻结时考题仪器本身还没提交（它和
这二十一份档在同一个提交里落地），与 EVAL-SET-LANGS.md 记 `bac6169` 同理；降表（提交 A）已在 `5278e74` 上，本仓的 Rust 语料
`codeeraser-flow` 钉的正是这个提交。

十份审阅档 `contracts/eval/flow-review-<语言>-v1.json`（提交 B′）的 `generated_from` 记 ce 1.8.0、树 `2ea957d`、dirty = true：归档时审阅仪器本身还没提交；53 批的批次文件、`manifest.json` 与答案文件在仓库外的车道目录里，不入库，答案的每个字经归档逐字进档。

十份精度册 `contracts/eval/flow-precision-<语言>-v1.json`（提交 C2）的 `generated_from` 记 ce 1.8.0、树 `bb9bdc8`、dirty = false：C1 提交了仪器与门之后，在那棵干净树上逐语言生成到车道目录、十份齐了一次拷进 `contracts/eval/`，生成期间仓库树里不建任何文件；judged = Python / TSX / Go / C / Java / Lua / R，掩码 `flow::judged_mask()` 按此填入。
