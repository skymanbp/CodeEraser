# 跨文件搬迁 / 堆叠判定 FPR 回放记录（计划 v2.29 步 10，C-R-L2-4）

> 2026-09-06 首测已冻结：821 个完整事件，零跳过、零拦截；严格 0/125、宽读法 0/820。
> 唯一 copy 正例 ripgrep `1035f6b1` 未被拦截，召回 0/1；无默认档位变更。

> 门（计划 §4.2）：500 次真实正常编辑误报 ≤ 1%；未达即停在 observe。
> **本册只立证据基础，不翻档位**——Stop 审计的四分类汇总仍是 informational；
> `ce precommit` / `ce commitmsg` 不读取这份汇总。`[guard] mode` 未声明时审计路线默认
> 是 `observe`（`config::tier_of(&loaded, "observe")`），冻结档的 `promoted`
> 字段写 `false`，翻档另裁。
>
> 方法：`cli/tests/it/l2_fpr_replay.rs`（`#[ignore]`，release 跑，常设）——
> 把三份冻结切片的 first-parent 提交当真实编辑流：**每提交 = 一个多文件改动集**
> （`session::commit_pairs` 配对，改名按 `R` 一对、复制按 `C` 只留新侧，
> `texts::load` 一次 `cat-file --batch` 取父/子两侧 blob），经
> `Judge::judge_changeset` 走审计**同一条**判决路（`classify_batch` + 同一个
> `fourclass/2` 核链），判决常数在 `CE.FourClass.Verdict` 一处。
>
> **事件口径 = 提交**（Stop 腿的单位就是一次改动集）。改动集不完整就不算事件：
> `no_parent` / `no_pairs` / `no_texts` / `no_judged_files` / `partial`
> （`texts::load` 的 `unread > 0`：超 `PAIR_CAP` 256、二进制、超 `READ_CAP`）/
> `degraded`（无核链、核无 `fourclass/2`、链路错、核自己的桶上限 `reason`——
> 这几种情形下 L2 那一趟根本没跑）逐类计数印出，不折进「干净」。
>
> **拦截判据**（反事实 deny）：`[guard] mode == "deny"` ∧ 核回执 `suspicions`
> 非空。M4 堆叠合取 = 落在**后侧新出现重复键的顶层具名单元跨度内**的 novel 行
> ≥ 20 ∧ 删除 × 10 < novel。跨文件 relocations **不进判据**——一次干净重构就
> 造几百行（L2 达标线 547/547），拿它定罪等于给重构定罪；它作为证据列印在
> 拦截行旁边，这正是「跨文件搬迁报告得以有 deny 路径的证据基础」的含义。
>
> 本册是**三份语料上的校准证据**，不是野外误报率的上界。

## 语料 = 三份冻结切片本身

口径与 L2 达标线、零虚报门**同一批提交**（按 sha 读冻结档，不重新划窗口）：

| 语料 | 冻结档 | 窗口 / tip | 入册提交 | 已审（labels 行） |
|---|---|---|---|---|
| CodeEraser 自仓 | `contracts/eval/commit-slice-v1.json` | `2f40f22b`（冻结历史，逐 SHA 验 first-parent 链） | 47 | 22 |
| requests | `commit-slice-requests-v1.json` | `00fd4c8e..8068356` | 341 | 47 |
| ripgrep | `commit-slice-ripgrep-v1.json` | `14860b0f..3fce3b5` | 433 | 57 |
| 合计 | — | — | **821** | **126** |

外部语料的检出走 `.ce-eval/corpora/<name>`，tip 由 `PINNED_CORPORA` 断言
（与 `eval_mention` / `eval_dedup_distinct` 同一条解析路），不用手给路径。
自仓冻结 tip 与当前 HEAD 属于不同根的历史；两边对象均在本地。仪器逐条断言
入册 SHA 属于冻结 tip 的 first-parent 链，保留原 47 条与标签，不改用当前 HEAD 划窗。

## Ground truth（按 sha 读冻结档，不新造标注）

| 标签 | 判据 | 数量 |
|---|---|---|
| `copy`（异常） | 切片行里有 `"copied": true` 的对——整文件复制膨胀形态 | **1**（ripgrep `1035f6b1`，`crates/regex/src/literal.rs` → `literalold.rs`） |
| `normal`（已审正常） | 有 `commit-labels*` 行（逐行对过原始 diff）且不是 copy | **125** |
| `unreviewed`（未审） | 切片入册但未逐条审过重复形态 | **695** |

入册 `normal` 125 < 500，故**严格率是校准数字**；§4.2 的「每 500 次」读的是宽读法。
入册非 copy 共 820；具名跳过后，实际事件与两种分母以下表为准。两种读法都印、都冻结、
都带精确 95 % 区间。

## 折算与门

事件 = 判到底的提交（口径见页眉）。下表逐格取自同次三语料冻结输出；
`l2_fpr_gate` 对照 `contracts/eval/fpr-l2-v1.json` 核对全部 14 列。

| 语料 | 事件 | 已审正常 | 未审 | 异常 | 拦截 | 误报（严格） | 漏 | 严格率 ‰ | 误报（宽） | 宽率 ‰ | 严格 CP 95 % | 宽 CP 95 % | 召回 ‰ |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| self | 47 | 22 | 25 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0.000–15.437 % | 0.000–7.549 % | 不适用 |
| requests | 341 | 47 | 294 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0.000–7.549 % | 0.000–1.076 % | 不适用 |
| ripgrep | 433 | 56 | 376 | 1 | 0 | 0 | 1 | 0 | 0 | 0 | 0.000–6.375 % | 0.000–0.850 % | 0 |
| 合计 | 821 | 125 | 695 | 1 | 0 | 0 | 1 | 0 | 0 | 0 | 0.000–2.908 % | 0.000–0.449 % | 0 |

逐语料打印的 `skipped {}` 均为空，六个具名原因逐格展开如下，未折入正常分母：

| 语料 | no_parent | no_pairs | no_texts | no_judged_files | partial | degraded |
|---|---|---|---|---|---|---|
| self | 0 | 0 | 0 | 0 | 0 | 0 |
| requests | 0 | 0 | 0 | 0 | 0 | 0 |
| ripgrep | 0 | 0 | 0 | 0 | 0 | 0 |

三次 dry 测试体分别耗时 **16.76 / 126.63 / 195.28 s**；首次完整冻结 340.27 s，消重后同次三语料复冻
**328.41 s**（PowerShell 命令总耗时 328.914 s），冻结件与首测逐字节相同，逐语料计数与 dry 一致。

召回按实际判到底的异常事件计；无异常的语料记「不适用」。入册仅有一个正例，
它未被拦截，以下单列原始 diff 的复核，不能据此推断普遍召回能力。

区间算法 = Clopper–Pearson 双侧 95 %，在精确二项尾上二分求解；同一实现由
`l2_fpr_gate` 的对照表钉住——它必须逐格重现 [FPR-TOMBSTONE.md](FPR-TOMBSTONE.md)
已公布的六个区间（400/0、530/3、530/7、537/1、537/6、936/4），所以本册的区间
和墓碑册的区间是同一把尺。

## 逐条仲裁

每次拦截打印一行 `INTERCEPT` JSON，含 `corpus / sha / pairs / file / moved_cross /
novel / deleted / dup_spans / verdict / label`，按行读 diff；`inDup`（落在重复跨度内的 novel 行数）
**留在 Haskell**，Rust 侧不复算——那是判决第三臂，镜像它就是 ADR-008 说的错误。
印出的三个整数（novel / deleted / 新重复跨度数）与文件名足以打开 diff 定案。

本次三份 dry 与完整冻结均打印 **0 条 `INTERCEPT`**，冻结 `rows` 为空；
因此逐拦截仲裁行数为 **0**，没有省略误报，也没有把拦截重标成正常。

唯一正例另立漏报复核，标签保持冻结切片的 `copy`：

| 语料 | 提交 | 文件 | 实测 | 原始 diff 复核 |
|---|---|---|---|---|
| ripgrep | `1035f6b1ff502eb5b1a5fc49a79f45971c772d47` | `crates/regex/src/literal.rs` → `literalold.rs` | 未拦截；漏 1，召回 0/1 | regex 1.9 迁移：原文件大段实现删去并缩为 stub，新文件保留旧实现并适配 API；已通读两文件 diff。副本分处两文件，未形成同一后侧文件内新重复的顶层单元键；M4 堆叠合取未触发。保留 copy 标签与漏报，不据此改阈值。 |

## 本册许可什么（§4.2）

**只有证据基础。** 默认档位不变：四分类汇总仍不进任何 deny 路径，`promoted`
仍是 `false`。要翻档需要用户单独裁定，届时冻结档把 `promoted` 改 `true`，
`l2_fpr_gate` 的蕴含式当场变成硬门（误报率超线即红）。反向探针已钉死这条蕴含
确实咬人。
本次严格率与宽率均为 0，未超过 1 %；严格分母 125 仍低于 500，且唯一正例漏过。
宽区间上界 0.449 % 只属于这份校准语料；零误报不能替代有效召回，也不授权默认翻档。

## 复现

PowerShell 5.1，先按项目构建说明编译 ce-core，再完整跑三份语料并一次冻结：

```powershell
Set-Location 'D:\Projects\CodeEraser\cli'
$env:PATH = 'C:\ghcup\bin;' + $env:PATH
$env:CE_CORE_BIN = 'D:\Projects\CodeEraser\core\dist-newstyle\build\x86_64-windows\ghc-9.14.1\ce-core-1.6.0\x\ce-core\build\ce-core\ce-core.exe'
$env:RUST_TEST_THREADS = '8'
Remove-Item Env:CE_L2FPR_CORPUS,Env:CE_L2FPR_LIMIT -ErrorAction SilentlyContinue
$env:CE_BLESS = '1'
cargo test --release -j 8 --test it -- --ignored l2_fpr_replay::l2_fpr_replay --exact --nocapture
Remove-Item Env:CE_BLESS -ErrorAction SilentlyContinue
```

单份诊断用 `CE_L2FPR_CORPUS=self|requests|ripgrep`，冒烟另加 `CE_L2FPR_LIMIT=40`；
这两种诊断不准冻结。外部检出须先补完整历史：浅克隆在各语料根运行 `git -c http.sslBackend=openssl fetch --unshallow`，
再用 `git rev-parse --is-shallow-repository` 核对 `false` 与钉定 HEAD；本次两份均已核实。
仪器仍按冻结切片逐 sha 取样，不把新抓到的提交扩大成新样本。

无 bless 的快门与合成改动集验收：

```powershell
Set-Location 'D:\Projects\CodeEraser\cli'
$env:PATH = 'C:\ghcup\bin;' + $env:PATH
$env:CE_CORE_BIN = 'D:\Projects\CodeEraser\core\dist-newstyle\build\x86_64-windows\ghc-9.14.1\ce-core-1.6.0\x\ce-core\build\ce-core\ce-core.exe'
$env:RUST_TEST_THREADS = '8'
Remove-Item Env:CE_BLESS,Env:CE_L2FPR_CORPUS,Env:CE_L2FPR_LIMIT -ErrorAction SilentlyContinue
cargo test -j 8 --test it -- l2_fpr
```

拦截按仪器打印的语料、sha 和文件打开原始 diff 仲裁；测量表从打印输出逐格抄入，
冻结件由仪器写入，不能手改。CI 门复算两种率、区间、召回和提交守恒，并核对正文全部表格单元。
