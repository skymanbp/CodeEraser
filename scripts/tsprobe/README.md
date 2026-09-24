# tsprobe — 语言扩展设计册的 AST 探针

设计册 [`docs/reference/language-expansion.md`](../../docs/reference/language-expansion.md) §4–§10 引用的每个结点 kind 与字段名都来自这个探针的转录：把一个样本交给七套钉版文法之一，逐结点打印 `字段: kind` 与叶文本。它不是产品面——`cli/` 与 `core/` 都不读它，测试子仓不挂载它，`Cargo.lock` 不入库（`Cargo.toml` 里的精确版本号才是复现契约）。

```sh
cd scripts/tsprobe
cargo run --release -- c    snippets/probe.c
cargo run --release -- cpp  snippets/round2.h     # C 头文件在 C++ 文法下
cargo run --release -- ruby snippets/probe.rb > /tmp/ruby.txt
```

首行是 `== <文法>: abi=<n> has_error=<bool> node_kinds=<n>`；abi 必须落在 tree-sitter 0.27 接受的 13–15 之内，`has_error=true` 或行尾 `!!ERROR` 说明样本没写对或文法不认。

| 轮 | 样本 | 文法 | 覆盖 |
|---|---|---|---|
| 1 | `probe.c` `probe.cpp` `probe.lua` `Probe.java` `probe.rb` `probe.R` `probe.html` | 各语言自己的 | 函数/方法/匿名函数、分支与循环、跳转与标签、注释、字面量、调用与成员访问、导入/包含、类型与可见性声明 |
| 2 | `round2.h`（C 头，走 cpp）`round2.c` `round2.cpp` `round2.lua` `round2.java` `round2.rb` `round2.R` | 同上 | 访问节默认值与类外定义、嵌套命名空间、函数指针与拼接串、Lua 表字段函数与长括号、Java 嵌套/局部/匿名类与带标签 continue、Ruby 访问节三形与 `class << self`、R 的 `x$y$z <-` 与默认实参 |

2026-09-24 两轮十四个样本全部 `has_error=false`、零 `!!ERROR`。加一门语言 = 加一个 `grammar()` 臂、一条 Cargo 依赖、一份样本；改样本后重跑并核对设计册 §10 的事实行。

**自食提示**：`snippets/` 里的七种扩展名今天不在判决集，`ce` 不走它们（`.html` 只进尺寸臂）；等这些语言落地后，请把 `scripts/tsprobe/snippets/**` 加进根 `ce.toml` 的 `exclude`（与 `contracts/fixtures/crosscheck/**` 同理），否则样本会以真实文件的身份进指纹索引与引用图。改 `ce.toml` 会动旋钮指纹，走 `CE_ACCEPT_FENCE=1` 重钉。
