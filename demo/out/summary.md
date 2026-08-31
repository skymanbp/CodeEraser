| | Without CodeEraser | With CodeEraser |
|---|---|---|
| The seed, by the same six gates: clone blocks · doc twins · dead files | 0 · 0 · 0 | 0 · 0 · 0 |
| Writes that landed | 7 of 7 | 5 of 7 |
| Denied at PreToolUse | 0 | 2 |
| Stop audit | not in the loop | **blocked** — `this session's edits leave 2 duplicate block(s) touching changed files (net +105 LOC)…` |
| The repair the audit named | — | written, and the audit goes silent |
| `ce erase --apply` | — | 1 row removed: the verbatim doc twin |
| `ce check` score (ratchet) | 952/1000 — **FAIL**: ratchet_over, discrete_added | 979/1000 — **FAIL**: ratchet_over |
| T1/T2 clone blocks (`ce dedup --check`, budget 0) | 4 (**FAIL**) | 0 (**pass**) |
| near-miss clone pairs (`ce clone`) | 4 | 0 |
| duplicated doc segments (`ce docdup --check`) | 1 (**FAIL**) | 0 (**pass**) |
| dead files (`ce deadcode --check`) | 3 (**FAIL**) | 2 (**FAIL**) |
| provably-safe removals still planned (`ce erase --check`) | 1 (**FAIL**) | 0 (**pass**) |
