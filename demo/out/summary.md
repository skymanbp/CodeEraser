| | Without CodeEraser | With CodeEraser |
|---|---|---|
| Writes that landed | 7 of 7 | 5 of 7 |
| Denied at PreToolUse | 0 | 2 |
| Stop audit | not in the loop | **blocked** — `this session's edits leave 2 duplicate block(s) touching changed files (net +105 LOC)` |
| `ce check` score (ratchet) | 952/1000 (**FAIL**) | 979/1000 (**FAIL**) |
| T1/T2 clone blocks (`ce dedup --check`, budget 0) | 4 (**FAIL**) | 2 (**FAIL**) |
| near-miss clone pairs (`ce clone`) | 4 | 1 |
| duplicated doc segments (`ce docdup --check`) | 1 (**FAIL**) | 1 (**FAIL**) |
| dead files (`ce deadcode --check`) | 3 (**FAIL**) | 2 (**FAIL**) |
| provably-safe removals planned (`ce erase --check`) | 1 | 1 |
