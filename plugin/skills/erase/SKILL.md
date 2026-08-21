---
name: erase
description: Guided cleanup of CodeEraser findings — read the reports, verify every candidate against the actual code, delete safely, and prove convergence with ce check. Invoke when the user asks to clean up duplication, dead code, or act on CodeEraser findings.
---

# erase — from findings to deletions, safely

CodeEraser finds entropy; this skill is the OTHER half — acting on it
without breaking anything. The contract: **never delete what you have
not read, and never claim convergence without re-running the gates.**

## 0. Snapshot first

```sh
ce check .          # the before-score and the current ratchet state
```

Record the score line. Every deletion below must leave `ce check`
passing; the score should not fall.

## 1. The deterministic pass FIRST — `ce erase`

```sh
ce erase .            # dry-run plan: what is PROVABLY safe to remove
ce erase . --apply    # act on it (git repo + clean worktree required)
```

Three classes erase without judgment (dead files, verbatim doc
duplicates, whole-unit byte-identical twins in dead files) — the tool
plans them deterministically, applies behind preconditions, and
proves its own convergence (contract: docs/reference/erase.md). Do
NOT hand-delete anything the plan already covers. Everything it
prints as `advisory` is YOUR half — that is where the judgment below
begins.

## 2. Collect the advisory candidates (three signals, JSON faces)

```sh
ce dedup . --format json      # T1/T2 clone blocks
ce deadcode . --format json   # zero-liveness verdicts (graph-based)
ce join . --days 14 --format json   # similarity × position × churn
```

`ce join` ranks the strongest deletion candidates: high similarity,
weak graph position, low churn. Start from the top of that list.

## 3. Verify each candidate — no exceptions

For every candidate pair or dead unit:

1. **Read both sides in full** (the whole function/section, not the
   matched span). Tool-reported similarity is a lead, not a verdict.
2. Check callers/references: prefer deleting the copy with fewer
   references; re-point the survivors.
3. Respect exemptions: a `.ceignore` entry means a human already
   ruled — skip it. For docdup candidates the same is true of an
   adjacent `ce:allow(docdup) -- <why>` line (the inline marker is
   docdup-only; dedup/deadcode exemptions go through `.ceignore`).
4. Dead code with an `entry_globs` match or exported surface may be a
   public API — confirm before removing.

## 4. Delete in small batches, re-gate each batch

```sh
ce dedup . --check    # the clone budget must not grow
<project test suite>  # the code must still work
```

## 5. Prove convergence

```sh
ce check .            # must pass; compare against the step-0 score
```

If the ratchet reports `removed`, run `ce baseline` to bank the
improvement (the violation set only shrinks without extra flags).
Report to the user: what was deleted, what was kept and why, and the
before/after score lines.
