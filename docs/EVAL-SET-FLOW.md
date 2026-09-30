# v2.31 函数内死代码精度考题册（第五次拆册，2026-09-30）

> 计划 v2.31 设计册 [reference/analysis-track.md](reference/analysis-track.md) §5.5「精度考题」段与 §13 第 16 条的冻结登记。
> 每个语言的 `flow/1` 精度照 v2.30 语言考题的仪器走，顺序由提交先后证明，只是抽样之前多一步：**降表与语言表先落（提交 A）→
> 单元宇宙 + 候选池 + 抽样冻结并提交（提交 B，本册第一节）→ 没看过判决的独立代理逐题盲判并提交 → 精度册提交**（顺序门
> `cli/tests/it/flow_provenance.rs`；题是从降出的四表里按类按层抽的，降表不先落地就没有池可抽，所以是 C / C++ 阶梯先于考题的那种
> `ladder_first` 形——盲判的独立性靠题不带答案、盲窗内 `cli/src/flow` 零提交、精度册钉在回答它的代码上）。母册链：
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
- **盲判**与**精度册**：另两个提交（B′ / C）各记一段；门 = 每语言每种非顾问发现（0 / 1 / 2）fp / (tp + fp) ≤ 1 %（样本内即 0 个 fp），
  且每种至少有一道被产品标出的题，池空的类具名记 vacuous；达门的语言进 `flow::judged_mask()`，未达的只 observe。
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

## 出处

十一份宇宙与十份样本的 `generated_from` 都记 ce 1.8.0、树 `5278e74`、dirty = true：冻结时考题仪器本身还没提交（它和
这二十一份档在同一个提交里落地），与 EVAL-SET-LANGS.md 记 `bac6169` 同理；降表（提交 A）已在 `5278e74` 上，本仓的 Rust 语料
`codeeraser-flow` 钉的正是这个提交。

十份审阅档 `contracts/eval/flow-review-<语言>-v1.json`（提交 B′）的 `generated_from` 记 ce 1.8.0、树 `2ea957d`、dirty = true：归档时审阅仪器本身还没提交；53 批的批次文件、`manifest.json` 与答案文件在仓库外的车道目录里，不入库，答案的每个字经归档逐字进档。
