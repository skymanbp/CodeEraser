# `ce erase` — the deterministic two-phase eraser (contract)

> Status: contract-first (M9 batch 3, plan v2.8 ruling ②). The
> implementation answers to this file; divergence is a defect in one
> of the two. Wire face: `erase/1`, one additive proto minor.

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
| dead file | `deadcode` file-tier dead (no kept in-edge, no entry flag, `ce deadcode --check`'s own bar) | delete the file | the graph verdict IS the safety proof: nothing in-corpus references it; the unresolved-site count must be zero for its language, else the row is refused (a verdict that assumed no in-corpus lands is not a deletion licence) |
| verbatim doc duplicate | `docdup` pair with **verbatim = full segment** (byte-identical after the family's own masking) | delete every occurrence after the first, in path-lexicographic order | prose has no call sites; byte-identity means zero information loss; the survivor is chosen deterministically, never judged "better" |
| whole-unit T1 twin | `dedup` T1 block spanning an ENTIRE unit whose twin is byte-identical AND whose copy is itself graph-dead | delete the dead copy | the narrow intersection of the clone and liveness verdicts — a cross-function clone with live references has NO deterministic-safe erase, and this contract says so instead of pretending |

Everything else — T2 (renamed) clones, T3 near-misses, live
duplicates, structural findings — is *named* by the plan as
`advisory: no deterministic-safe erase` rows. Erasing those requires
judgment about which copy is canonical and how references move; that
is an editor's job (human or agent), and the plan hands them the
evidence instead of acting.

## Two phases

**`ce erase [root]`** (the plan — default, always read-only):

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
5. exits 0 with no filesystem effect of any kind.

**`ce erase --apply`** additionally requires, in order:

1. a git repository (revert must be one `git checkout` away);
2. a CLEAN worktree (`git status --porcelain` empty — an erase must
   never be entangled with uncommitted work);
3. every target file's content hash equal to the plan's (a file that
   moved since planning refuses by name — plans are not portable
   across edits);
4. after writing: the source family re-runs and the erased verdicts
   must be GONE — a survivor fails the command loudly (convergence is
   part of apply, not a suggestion);
5. an append-only record in `.ce/erase-log.ndjson`: ts, class,
   file/span, provenance, plan hash — the audit trail `ce doctor` can
   count and the dashboard can render.

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

- plan-then-apply on a fixture tree erases the three classes and the
  re-run proves zero surviving source verdicts;
- a dirty worktree, a drifted file hash, and a non-repo root each
  refuse BY NAME without touching anything;
- an advisory row (live T2 clone) is never planned;
- `ce erase` twice = byte-identical plans (determinism pinned by test);
- the self repo: `ce erase` at HEAD plans ZERO rows (this repository
  keeps itself clean — a non-empty self-plan is a red gate).
