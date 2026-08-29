# demo — the same task, run twice

```sh
node demo/run.js            # run both, write demo/out/*
node demo/run.js --check    # run both, fail if any committed output would change
node demo/run.js --keep     # also leave both scratch trees on disk to poke at
```

Needs `ce` on PATH (or `CE_BIN=/path/to/ce`) with a reachable `ce-core`
(`CE_CORE_BIN` or a sibling), git and node. No packages.

One coding task — *add discounts, a compact report, CSV and JSON output, and
money formatting in the API* — run twice against identical copies of
[`seed/`](seed/README.md), a small cent-exact invoicing service in Python and
TypeScript. The only variable is whether CodeEraser's PreToolUse guard and
Stop audit sit in the loop (`seed/ce.toml` says `[guard] mode = "deny"`).
Both trees are then measured by the same six commands — the CI face.

<!-- demo:begin -->
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
<!-- demo:end -->

![Without CodeEraser: seven writes land, and the measured tree carries four
exact clones, a pasted paragraph and three dead files](out/without-codeeraser.svg)

![With CodeEraser: two writes are denied at PreToolUse with the region they
duplicate named, the Stop audit refuses to end the turn over the two blocks
that slipped past, and the gates name the rest](out/with-codeeraser.svg)

Transcripts as text: [without](out/without-codeeraser.txt) ·
[with](out/with-codeeraser.txt) · the numbers as [JSON](out/summary.json) and
as the table above ([en](out/summary.md) / [zh](out/summary.zh.md)).

## What is real and what is scripted

- **Real** — every verdict. Each PreToolUse decision is the verbatim stdout of
  `ce probe --hook` fed the envelope Claude Code sends for a `Write`; the Stop
  line is `ce audit --hook`'s; every gate line is the command's own output.
  Scratch paths are replaced by `<work>`; nothing else is edited.
- **Scripted** — the agent's seven moves ([`steps.js`](steps.js)). No model is
  in the loop. Each write is built from the seed alone, so no move depends on
  an earlier one having landed, and a refusal in one run cannot change what
  the remaining moves do in the other. Scripting them is what makes the two
  runs identical in everything except the hooks.
- **Gated** — the replay test in the test suite re-runs this driver and
  compares `out/` and the three embedded tables byte for byte, so a change in
  any verdict's wording fails CI rather than leaving a stale picture here.

## The seven moves, and what each one is

| # | write | the drift it stands for | write-time verdict |
|---|---|---|---|
| 1 | `invoicer/discount.py` | copies `to_cents` and `scale_cents` out of `money.py` "to stay self-contained" | **denied** — an exact T1 clone of an indexed region, named by file and lines |
| 2 | `invoicer/report.py` | a "compact" renderer: the old rows and footer, renamed and reordered | lands — the file already carried those blocks, so the write introduces nothing *novel*; the Stop audit convicts it |
| 3 | `docs/DISCOUNTS.md` | opens by pasting the pricing paragraph | lands — doc duplication is judged by `ce docdup`, not at write time (no false-positive record yet) |
| 4 | `invoicer/invoice.py` | CSV export appended to the busiest module | lands — growth inside the hard line is the ratchet's business (`ce check`: `ratchet_over`) |
| 5 | `invoicer/report_json.py` | a JSON renderer, written fresh | lands — genuinely new |
| 6 | `invoicer/cli.py` | switches the CLI to JSON; `report.py` is left behind | lands — `ce deadcode` names the orphan |
| 7 | `web/api.ts` | a local copy of `format.ts`'s `formatCents` | **denied** — the TypeScript twin of move 1 |

Move 2 is the honest boundary on purpose: the duplicate-write rule charges
only duplication a write *introduces* (the 2,761-event replay in
[FPR-REPLAY.md](../docs/FPR-REPLAY.md) is why), and a full-file rewrite that
copies its own blocks introduces none — so the next layer, the Stop audit,
refuses to end the turn, and the CI face's clone budget refuses the commit.
