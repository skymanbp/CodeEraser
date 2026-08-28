# M3 FPR 重放验收记录（2026-08-07）

> **封册（M7.5 深度瘦身，2026-08-18）**：重放仪器 `fpr_replay.rs` 已随
> 休眠仪器整体退役（EVAL-SET.md 修正案），本册即 FPR 记录的账本；
> 复核走 git 历史复活仪器（退役修正案提交的父 commit）。K 轮复跑
> （2026-08-26，见下节）延账一次并触发探针语义根修，跑毕仪器已复退役。

> 验收门（计划 §6 M3）：500 次真实正常编辑重放误拦 ≤ 1。
> 方法：`cli/tests/fpr_replay.rs`（`#[ignore]`，release 跑）——把 git
> 线性历史当真实编辑流：每提交每改动代码文件 = 一次"把子版本内容写入
> 父状态"的编辑事件；影子目录增量物化父状态，**先探针后应用**；guard
> 默认档（t=50 / min_distinct=7）。拦截逐条列出供仲裁——真实引入了
> 跨文件重复的提交是真阳，不计入误拦。

## 结果

| 语料 | 事件数 | 拦截 | 仲裁后误拦 | 折算 /500 |
|---|---|---|---|---|
| requests 完整历史尾段 400 提交（钉定 1f6589ec 止） | 487 | 0 | **0** | 0.00 |
| CodeEraser 自仓全史（真实 agent 编辑流） | 143 | 35 | **0**（全为真重复，见下） | 0.00 |
| 合计 | 630 | 35 | **0** | **0.00 ≤ 1 ✅** |

## 自仓 35 条拦截仲裁（全真阳）

- 2 条：zod locale fixtures 入库提交——这些文件**按设计**互为 T2 克隆，
  "新增文件与既有文件重复"判定正确。
- 33 条：`cli/tests/*.rs` 之间共享的 `rust_fn(seed)`/`tmp()`/git 助手
  ——**作者本人复制粘贴的真实重复**（8 个测试文件各持一份）。工具当场
  抓住了它要消灭的行为。治理已分四批完成（251→211→209→205→202 块，
  预算同步降档）：`tests/common/mod.rs` 统一数据助手 + hook 运行器 +
  daemon 启动/回收 + observe 尾行读取 + dedup 断言 + tree-sitter 解析
  harness（第 5 份拷贝在 RCA 对拍 harness，切换后 322 单元实跑复验绿）；
  `src/hookio.rs` 统一三 hook 的 stdin 进气与 observe 写入。保留不抽
  =命名边界（ce.toml 注释同步）：用例平行结构（deny/warn/observe 三态
  断言、dedup_index 刷新序列）、三 hook 的信封契约声明（4 块）、
  metrics/divergence/sonar 的文档性测试体（~150 块——逐例白皮书引注
  即内容本体，T2 把 src 字面量折叠后相邻用例连成一个家族）。

## 重放捎带抓获的产品缺陷（已修）

探针候选文件在"索引后、探针前"消失（删除/改名竞态；requests 史上的
目录改名提交实锤触发）曾使探针整体报错——现按陈旧锚点同一哲学降级
跳过（probe.rs `load_candidate_streams`），daemon 探针请求不再因竞态
失败。


## K 轮复跑与探针语义根修（2026-08-26，v1.2.0 前）

> 仲裁出处：用户授权下的**模型仲裁**（授权 2026-08-24，执行 2026-08-25/26），
> 非人工记录；每条首燃判定均以 git 事实与今日检测器复核，方法见文末。

仪器按「复现」节配方复活（两个同代垫片，样张见同节），语料与旋钮同 2026-08-07。

**第一轮（出厂语义=全文探针）**：requests 尾段 487 事件 **0 拦截**（与原记录逐字同）；
自仓全史 2274 事件 **505 拦截**（111.04/500）。按无序文件对分解 = 122 对首燃 +
383 复燃；首燃逐对取**子提交状态**的两文件入临时夹具、以今日 `ce dedup` 终审：
90 对重复真实落地（44 对至今仍在 186 预算台账），32 对仅存在于重放中间态——
**全部是拆叶/并叶提交**（新叶先探、宿主未削；含 2 例并叶=被匹配文件同提交删除）。
真机 hook 实测：暖索引下拆叶写 deny（172 tokens 命中）；本机生产台账 719 探 /
170 degraded / 9 燃、零拆叶误拦。383 复燃 = 编辑携带既接受债块的文件被全文模型
再拦——活 guard 对 Edit 只探 new_string 片段，不受此扰；Write 全文重写则同病，真尖边。

**判定与拍板**：32 条中间态若计误拦 = 1.41% > §4.2 M4 门（≤1%）。用户拍板
（2026-08-26）：**根修探针再复测**，不降档。

**根修（`guard::novel_matches`）**：探针只报**新引入**的匹配——Write 减盘上旧全文、
Edit 减 `old_string` 的自有匹配（基线探针仅首探命中才发；基线 degraded 减零——
放行绝不骑在未答的问题上）；拦截词补搬移安全序（先削源后写目标，探针按当下磁盘
验证候选流，实测放行）。observe feed `matches` 语义随之收窄为新引入数，schema
0.6.0→0.7.0 具名断点。四腿电池 `cli/tests/it/guard_novelty.rs`（携债重写不新 /
加孪生仍拦 / 编辑自替不新 / 拆叶仍拦且教序）——书写该电池时两度被新语义活拦，
按共享咽喉消重后过门：工具第三次当场抓住作者本人。

**第二轮（新语义，仪器同镜像）**：

| 语料 | 事件 | 拦截 | 分解 | 折算 /500 |
|---|---|---|---|---|
| requests 尾段（同钉定） | 487 | 0 | — | 0.00 |
| 自仓全史 | 2274 | 139 | 84 真引入 + 32 中间态搬移 + 23 延展复燃 | 见下 |

- **84 真引入**：子状态经今日检测器复核确有跨文件重复的落地提交——工具该拦的。
- **23 延展复燃**：编辑扩张/漂移了既有重复区，父版基线按构造不重叠——判真阳倾向入册。
- **32 中间态搬移**：写先于削的拆叶/并叶。瞬时树内确有重复（探针未说谎），意图是
  搬移——复制与搬移在写入瞬间**信息等价**，瞬时谓词不存在；处置 = 拦截词教安全序
  （先削源即过），终态判决属 Stop 层（搬移完成后树内无重复，Stop 审计对此类天然
  不误报）。
- **门算术（双口径并记，不合成单一数字）**：按重放全文写模型计 32 条误拦 =
  7.03/500（1.41%）；按活流口径（Edit 片段语义 + 安全序协议）本机 719 条生产
  探针 0 条此类误拦 = 0.00/500。两类已晋级规则**维持 deny**（拍板走根修而非降档），
  依据入 CHANGELOG。

**多文件搬移仪器（清单 57 处置）**：无序对首燃/复燃分类 + 同提交 name-status
交叉 + 子状态二文件夹具复核，即本节全部搬移测量的工作法；不另立常驻仪器
（M7.5 休眠仪器退役律），复现法见下节。

## 复现

仪器 `cli/tests/fpr_replay.rs` 已随 M7.5 封册退役（见文首横幅）——按 EVAL-SET.md「再生成」
节的复活律**连同同代支撑**复活、跑毕重退役（其 `common::git_out` 已于 9e05f53 出仓，单取仪器
文件对今日支撑编译不过）：`git show 0c7c936^:cli/tests/fpr_replay.rs > cli/tests/fpr_replay.rs && git archive 0c7c936^ cli/tests/common | tar -x`，
跑下列命令，再 `rm -rf cli/tests/fpr_replay.rs cli/tests/common`
（复活件在两个仓都是未跟踪文件：`cli/tests` 自 9bedcc4 起是 submodule，超仓索引里只有 gitlink 一行，对着
submodule 路径 `git checkout <sha> -- <路径>` 会静默把 gitlink 换成历史 blob——退役必须是纯 `rm`，永不写索引）：

K 轮实测两处同代垫片（复活后按此打上；第一片 product mirror 仅第二轮新语义复测才打、第一轮出厂语义须留白；第二片 remove_missing 补 seen 参两轮都必打——今日签名为二参，缺之编译不过）：

```diff
--- fpr_replay.rs (0c7c936^)
+++ fpr_replay.rs (revived, K round)
@@ -128,6 +128,30 @@
                 lang: Lang::from_path(Path::new(&rel)).expect("lang"),
             };
             let m = probe::probe(&idx, &shadow, target, p, f).expect("probe");
+            // product mirror (K step 11 novel-duplication semantics):
+            // a Write replaces the file, and matches the replaced
+            // content already had are carried, not introduced — for
+            // M events subtract the parent version's own matches
+            let m = if status == 'M' && !m.is_empty() {
+                let old = git_out(&repo, &["show", &format!("{parent}:{rel}")]);
+                let base_t = probe::Target {
+                    rel: &rel,
+                    content: &old,
+                    lang: Lang::from_path(Path::new(&rel)).expect("lang"),
+                };
+                let base = probe::probe(&idx, &shadow, base_t, p, f).expect("probe base");
+                m.into_iter()
+                    .filter(|x| {
+                        !base.iter().any(|b| {
+                            b.file == x.file
+                                && b.start_line <= x.end_line
+                                && x.start_line <= b.end_line
+                        })
+                    })
+                    .collect()
+            } else {
+                m
+            };
             if !m.is_empty() {
                 intercepts.push(format!(
                     "{} {} -> {} ({} tok)",
@@ -140,7 +164,11 @@
             apply(&shadow, &mut idx, &repo, commit, &rel, p);
             live.insert(rel);
         }
-        idx.remove_missing(&live).ok();
+        // same-generation shim: remove_missing grew a `seen` arg after
+        // the instrument retired (walkidx refresh split); the replay's
+        // seen set is simply what the index holds
+        let seen = idx.indexed_paths().expect("indexed_paths");
+        idx.remove_missing(&live, &seen).ok();
     }
     println!(
         "replayed {events} edit events, {} intercepts:",
```

- 自仓：`cargo test --release --test fpr_replay -- --ignored --nocapture`
- 外部仓：`CE_FPR_REPO=<path> cargo test --release --test fpr_replay -- --ignored --nocapture`
  （本记录的 requests 语料：钉定 commit 浅克隆后 `git fetch --deepen 400`）
