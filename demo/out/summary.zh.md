| | 不带 CodeEraser | 带 CodeEraser |
|---|---|---|
| 落地的写入 | 7 / 7 | 5 / 7 |
| PreToolUse 当场拒绝 | 0 | 2 |
| Stop 审计 | 不在环内 | **拦停** — `本会话的编辑留下 2 个触及改动文件的重复块（净 +105 行）` |
| `ce check` 分数（棘轮） | 952/1000 (**FAIL**) | 979/1000 (**FAIL**) |
| T1/T2 克隆块（`ce dedup --check`，预算 0） | 4 (**FAIL**) | 2 (**FAIL**) |
| 近似克隆对（`ce clone`） | 4 | 1 |
| 重复文档段（`ce docdup --check`） | 1 (**FAIL**) | 1 (**FAIL**) |
| 死文件（`ce deadcode --check`） | 3 (**FAIL**) | 2 (**FAIL**) |
| 计划中的可证安全删除（`ce erase --check`） | 1 | 1 |
