# v2.31 函数内死代码精度考题册·第一代归档（F′，2026-09-30）

> 本册是 [EVAL-SET-FLOW.md](EVAL-SET-FLOW.md) 的第一代记录。提交 E 的降表回修挪了 TypeScript、Rust、C++、R 四门考题的候选池，
> 这四门出了第二代（提交 F 重导宇宙与样本、F′ 第二代盲判）；它们第一代的「宇宙与抽样」「盲判」「精度（第一代，提交 E 退役）」三节
> 于 F′ 从主册逐字节搬到这里，只搬不改——节内「见『仪器与门』」「见『候选池』一段」「见『步 4 提交 E』一节」指的都是主册的节。
> 第一代的宇宙 `flow-slice-<键>-v1.json`、样本 `flow-sample-<语言>-v1.json` 与审阅档 `flow-review-<语言>-v1.json` 仍在盘上，
> 是那一代的记录（门只读考题表指向的代）；第一代的十份精度册自提交 E 起退役（删出树、历史里仍在）。池一格没动的六门（Python、
> TSX、Go、C、Java、Lua）只有一代，它们的「宇宙与抽样」「盲判」两节留在主册，退役的「精度（第一代，提交 E 退役）」一节于 G（2026-09-30）
> 逐字节搬到这里（只搬不改）；第二代各节、两代共有题的比对与 G 起重生成的精度册都在主册。本册与主册同入冻结集
> （`frozen_set.rs`：不扫芯片、不生成、退出引文门），行号引文一律不写。

## Python（步 4 提交 B）

### 精度（第一代，2026-09-30；提交 E 退役）

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

### 精度（第一代，2026-09-30；提交 E 退役）

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

### 精度（第一代，2026-09-30；提交 E 退役）

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

### 精度（第一代，2026-09-30；提交 E 退役）

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

### 精度（第一代，2026-09-30；提交 E 退役）

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

### 精度（第一代，2026-09-30；提交 E 退役）

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

### 精度（第一代，2026-09-30；提交 E 退役）

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

### 精度（第一代，2026-09-30；提交 E 退役）

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

### 精度（第一代，2026-09-30；提交 E 退役）

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

### 精度（第一代，2026-09-30；提交 E 退役）

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
