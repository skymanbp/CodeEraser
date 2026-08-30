| | 不带 CodeEraser | 带 CodeEraser |
|---|---|---|
| 种子树，同样六道门实测：克隆块 · 文档孪生 · 死文件 | 0 · 0 · 0 | 0 · 0 · 0 |
| 落地的写入 | 7 / 7 | 5 / 7 |
| PreToolUse 当场拒绝 | 0 | 2 |
| Stop 审计 | 不在环内 | **拦停** — `本会话的编辑留下 2 个触及改动文件的重复块（净 +105 行）` |
| 审计点名的那处修复 | — | 写下之后，审计转为沉默 |
| `ce erase --apply` | — | 移除 1 行：逐字文档孪生 |
| `ce check` 分数（棘轮） | 952/1000 — **FAIL**: ratchet_over, discrete_added | 979/1000 — **FAIL**: ratchet_over |
| T1/T2 克隆块（`ce dedup --check`，预算 0） | 4 (**FAIL**) | 0 (**pass**) |
| 近似克隆对（`ce clone`） | 4 | 0 |
| 重复文档段（`ce docdup --check`） | 1 (**FAIL**) | 0 (**pass**) |
| 死文件（`ce deadcode --check`） | 3 (**FAIL**) | 2 (**FAIL**) |
| 仍待执行的可证安全删除（`ce erase --check`） | 1 (**FAIL**) | 0 (**pass**) |
