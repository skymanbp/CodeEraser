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

### 真值与判分

尚未进行：真值在下一个提交里由独立代理判定并冻结，判分随 Java 的阶梯一起提交，两段届时补在这里。
