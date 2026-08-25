# `ce erase` — the deterministic two-phase eraser (contract)

> Status: implemented (M9 batch 3, plan v2.8 ruling ②; wire face
> `erase/1`, introduced at proto 2.16.0, RG10 firewall at 6.1.0;
> predicate in CE.Erase.Cost). The
> implementation answers to this file; divergence is a defect in one
> of the two. Acceptance is pinned by `cli/tests/erase_e2e.rs` and
> the CI Dogfood `erase .. --check` self-gate.

## The ruling this implements

User ruling 2026-08-21 (plan v2.8): the GUI must be able to *erase*,
not only diagnose — via a deterministic two-phase `ce erase` that acts
ONLY on classes whose removal is provably safe, with plan/apply
separation, dry-run default, a clean-worktree precondition, and the
eraseability judgment in the Haskell core. **Never an LLM rewrite**:
this product's thesis is deterministic computation auditing
non-deterministic output, and an eraser that guesses would forfeit it.

## What may be erased (v1 classes, and why each is safe)

| class | source verdict | the erase | why it is deterministic-safe |
|---|---|---|---|
| dead file | `deadcode` file-tier dead (no kept in-edge, no entry flag, `ce deadcode --check`'s own bar) and **private**: verdict 1 `unref_private` or 3 `unreach_private` | delete the file | the graph verdict IS the safety proof: nothing in-corpus references it; the unresolved-site count must be zero for its language, else the row is refused (a verdict that assumed no in-corpus lands is not a deletion licence). The PUBLIC half of the dead domain — 2 `unref_public`, 4 `unreach_public` — is refused by name as `public_surface` since 6.1.0: a library's exported API is unreferenced in-corpus by construction, so "nothing here calls it" is not evidence about its callers, and the four-way dead code exists precisely to keep the two apart (RG10) |
| verbatim doc duplicate | `docdup` pair with **verbatim = full segment** (byte-identical after the family's own masking) | delete every occurrence after the first, in path-lexicographic order | prose has no call sites; byte-identity means zero information loss; the survivor is chosen deterministically, never judged "better" |
| whole-unit T1 twin | `dedup` T1 block spanning an ENTIRE unit whose twin is byte-identical AND whose copy is itself graph-dead | delete the dead copy | the narrow intersection of the clone and liveness verdicts — a cross-function clone with live references has NO deterministic-safe erase, and this contract says so instead of pretending |

The planner runs only `deadcode`, `docdup`, and `dedup`. Candidates
from those families that fail the core predicate remain non-eraseable
plan rows; the sole named aggregate `out_of_class` advisory is
`t1t2_block_no_whole_unit`. T3 and structural findings are outside
the plan surface and never appear.

## Two phases

**`ce erase [root]`** (the plan — default, read-only with respect to
user files; it may create or refresh the `.ce/` cache):

1. runs the source families exactly as their own commands do (same
   caches, same cores, same knobs);
2. sends the fact tables to the core's `erase/1`, which answers the
   eraseable-row set — the PREDICATE is Haskell's (ADR-008: which rows
   are safe is judgment; the bytes are measurement);
3. renders a unified diff (`--format json`: machine rows with
   file/span/class/provenance), each hunk carrying its verdict
   provenance (family, member/segment id, evidence `file:line`) and a
   content hash of the target file;
4. prints the advisory rows (what it will NOT touch, and why);
5. exits 0 without changing user files; the `.ce/` cache may have
   been created or refreshed.

**`ce erase --apply`** additionally requires, in order:

1. a git repository (revert must be one `git checkout` away);
2. a CLEAN worktree (`git status --porcelain` empty — an erase must
   never be entangled with uncommitted work; the tool's OWN state
   under `.ce/` is exempt from the check, because running the
   planner is what creates it — ce's index is never "uncommitted
   work", batch-7 defect sweep: the carve-out existed in code,
   undocumented);
3. every target file's content hash equal to the plan's (a file that
   moved since planning refuses by name — plans are not portable
   across edits);
4. after writing: the source family re-runs and the erased verdicts
   must be GONE — a survivor fails the command loudly (convergence is
   part of apply, not a suggestion);
5. an append-only record in `.ce/erase-log.ndjson`: ts, class,
   file/span, provenance, plan hash — an audit file for human review
   alongside git's recovery path; no CLI or GUI surface renders it today.

## Boundaries

- `.ceignore` is a human ruling and binds here exactly as in the
  guard: an ignored path is never planned, let alone applied.
- The exclusion model (built-ins + ce.toml + .gitignore) bounds the
  plan the same way it bounds every walk.
- `--apply` touches ONLY files the plan named; a plan is the complete
  and closed statement of intent.
- No network, no LLM, no heuristics: every planned hunk must be
  reproducible byte-for-byte from the same tree.
- GUI (batch 4) renders the SAME plan JSON and applies through the
  SAME library entry — one implementation, two faces.

## Acceptance (the gate this feature must pass)

- plan-then-apply on a fixture tree erases two dead files and one
  verbatim-doc span; `t1_twin` is exercised only as a non-eraseable
  plan row, and the re-run proves zero surviving source verdicts;
- a dirty worktree, a drifted file hash, and a non-repo root each
  refuse BY NAME without touching anything;
- an advisory row (live T2 clone) is never planned;
- `ce erase` twice = byte-identical plans (determinism pinned by test);
- the self repo: `ce erase` at HEAD plans ZERO rows (this repository
  keeps itself clean — a non-empty self-plan is a red gate).
