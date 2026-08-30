# demo — the same task, run twice

```sh
node demo/run.js            # run both, write demo/out/*
node demo/run.js --check    # run both, fail if any committed output would change
node demo/run.js --keep     # also leave all three scratch trees on disk to poke at
```

Needs `ce` on PATH (or `CE_BIN=/path/to/ce`) with a reachable `ce-core`
(`CE_CORE_BIN` or a sibling), git and node. No packages.

One coding task — *add discounts, a compact report, CSV and JSON output, and
money formatting in the API* — run twice against identical copies of
[`seed/`](seed/README.md), a small cent-exact invoicing service in Python and
TypeScript. The only variable is whether CodeEraser's PreToolUse guard and
Stop audit sit in the loop (`seed/ce.toml` says `[guard] mode = "deny"`).
The seed is measured first, by those same six commands, so the table can say
whether a finding was already there (it never is) or was written by the task.

Each loop then runs to **its own** end, which is the point rather than a
thumb on the scale: with nothing in the loop nothing refuses anything, so
that run ends at the last write; with the hooks in it the audit refuses to
end the turn, the repair it names is written, and `ce erase --apply` removes
what the plan proves safe. Both trees are then measured — the CI face.

<!-- demo:begin -->
| | Without CodeEraser | With CodeEraser |
|---|---|---|
| The seed, by the same six gates: clone blocks · doc twins · dead files | 0 · 0 · 0 | 0 · 0 · 0 |
| Writes that landed | 7 of 7 | 5 of 7 |
| Denied at PreToolUse | 0 | 2 |
| Stop audit | not in the loop | **blocked** — `this session's edits leave 2 duplicate block(s) touching changed files (net +105 LOC)` |
| The repair the audit named | — | written, and the audit goes silent |
| `ce erase --apply` | — | 1 row removed: the verbatim doc twin |
| `ce check` score (ratchet) | 952/1000 — **FAIL**: ratchet_over, discrete_added | 979/1000 — **FAIL**: ratchet_over |
| T1/T2 clone blocks (`ce dedup --check`, budget 0) | 4 (**FAIL**) | 0 (**pass**) |
| near-miss clone pairs (`ce clone`) | 4 | 0 |
| duplicated doc segments (`ce docdup --check`) | 1 (**FAIL**) | 0 (**pass**) |
| dead files (`ce deadcode --check`) | 3 (**FAIL**) | 2 (**FAIL**) |
| provably-safe removals still planned (`ce erase --check`) | 1 (**FAIL**) | 0 (**pass**) |
<!-- demo:end -->

![Without CodeEraser: seven writes land, and the measured tree carries four
exact clones, a pasted paragraph and three dead files](out/without-codeeraser.svg)

![With CodeEraser: two writes are denied at PreToolUse with the region they
duplicate named, the Stop audit then refuses to end the turn over the two
blocks that slipped past, the repair it names lands, and the erase plan
removes the verbatim doc twin](out/with-codeeraser.svg)

Transcripts as text: [without](out/without-codeeraser.txt) ·
[with](out/with-codeeraser.txt) · the numbers as [JSON](out/summary.json) and
as the table above ([en](out/summary.md) / [zh](out/summary.zh.md)).

## What is real and what is scripted

- **Real** — every verdict. Each PreToolUse decision is the verbatim stdout of
  `ce probe --hook` fed the envelope Claude Code sends for a `Write`; the Stop
  line is `ce audit --hook`'s, asked once per language (`CE_LANG=en` and `zh` —
  the audit is read-only) so each README's table quotes the verdict in its own
  language; every gate line is the command's own output,
  path-normalized (`<work>`), with `advisory` and diff lines dropped and only
  the last 8 lines shown — the unclipped text is in [`out/summary.json`](out/summary.json).
- **Scripted** — the agent's seven moves, and the repair the audit asks for
  ([`steps.js`](steps.js)). No model is in the loop. Each write is built from
  the seed alone, so no move depends on an earlier one having landed, and a
  refusal in one run cannot change what the remaining moves do in the other.
  Scripting them is what makes the two runs identical in everything except
  the hooks. The repair is scripted the same way and reached only through the
  audit's refusal: the run asserts the guard does not refuse it (removing
  duplication never should) and asserts the audit falls silent after it.
- **Gated** — the replay test in the test suite re-runs this driver and
  compares `out/` and the three embedded tables byte for byte, so a change in
  any verdict's wording fails CI rather than leaving a stale picture here. The
  corpus and both READMEs are pinned to LF in `.gitattributes`, so a CRLF
  checkout cannot move a marker.

## The seven moves, and the eighth a gate asked for

| # | write | the drift it stands for | write-time verdict |
|---|---|---|---|
| 1 | `invoicer/discount.py` | copies `to_cents` and `scale_cents` out of `money.py` "to stay self-contained" | **denied** — an exact T1 clone of an indexed region, named by file and lines |
| 2 | `invoicer/report.py` | a "compact" renderer: the old rows and footer, renamed and reordered | lands — the file already carried those blocks, so the write introduces nothing *novel*; the Stop audit convicts it |
| 3 | `docs/DISCOUNTS.md` | opens by pasting the pricing paragraph | lands — doc duplication is judged by `ce docdup`, not at write time (no false-positive record yet) |
| 4 | `invoicer/invoice.py` | CSV export appended to the busiest module | lands — growth inside the hard line is the ratchet's business (`ce check`: `ratchet_over`) |
| 5 | `invoicer/report_json.py` | a JSON renderer, written fresh | lands — genuinely new |
| 6 | `invoicer/cli.py` | switches the CLI to JSON; `report.py` is left behind | lands — `ce deadcode` names the orphan |
| 7 | `web/api.ts` | a local copy of `format.ts`'s `formatCents` | **denied** — the TypeScript twin of move 1 |
| 8 | `invoicer/report.py` | not the task's: the repair the Stop audit named — one renderer, the compact variant differing only where it really differs | lands, and the audit falls silent |

Move 2 is the honest boundary on purpose: the duplicate-write rule charges
only duplication a write *introduces* (the 2,761-event replay in
[FPR-REPLAY.md](../docs/FPR-REPLAY.md) is why), and a full-file rewrite that
copies its own blocks introduces none — so the next layer, the Stop audit,
refuses to end the turn over exactly those two blocks, which is what move 8
answers.

Two gates are still red when the loop converges, and both are asking a person
for a decision rather than reporting a defect: `invoicer/invoice.py` stands at
93 lines against a tolerated ceiling of 61 (move 4), which ADR-006 keeps open
for a named re-establish instead of absorbing, and two files are unreferenced
— `docs/DISCOUNTS.md`, which nothing links, and `invoicer/report.py`, orphaned
by move 6. A demo that scripted those away would be scripting the human out of
a judgement only a human makes.
