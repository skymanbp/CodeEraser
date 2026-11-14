# `ce erase` — the deterministic two-phase eraser (contract)

> Status: implemented (M9 batch 3, plan v2.8 ruling ②; wire face
> `erase/1`, introduced at proto 2.16.0, RG10 firewall at 6.1.0, no
> whole-file deletion since 9.1.0; predicate in CE.Erase.Cost). The
> implementation answers to this file; divergence is a defect in one
> of the two. Acceptance is pinned by `cli/tests/it/erase_e2e.rs` and
> the CI Dogfood `erase .. --check` self-gate.

## The ruling this implements

User ruling 2026-08-21 (plan v2.8): the GUI must be able to *erase*,
not only diagnose — via a deterministic two-phase `ce erase` that acts
ONLY on classes whose removal is provably safe, with plan/apply
separation, dry-run default, a clean-worktree precondition, and the
eraseability judgment in the Haskell core. **Never an LLM rewrite**:
this product's thesis is deterministic computation auditing
non-deterministic output, and an eraser that guesses would forfeit it.

## What the plan judges, and what may be erased

| class | source verdict | the erase | why |
|---|---|---|---|
| verbatim doc duplicate | `docdup` pair with **verbatim = full segment** (byte-identical after the family's own masking) | delete every occurrence after the first, in path-lexicographic order | prose has no call sites; byte-identity means zero information loss whoever reads the file; the survivor is chosen deterministically, never judged "better" |
| dead file | `deadcode` file tier: a file in the dead set, listed as unreferenced (no kept in-edge from a reached file, no entry flag) | **none** — an advisory row with its reason | the graph verdict answers "does any reference site of the corpus land on this file?", and a file is also used with no such site: run by its name (a script, a command in a Makefile, a CI file or a package script), found by a tool's convention the product does not list (a test runner's data directory; the eight the dead-file exam passed, among them `setup.py`, `docs/conf.py` and `package-info.java`, are entries, booklet 06 role 10), served or opened by its path. So the row is refused by name in this order: the PUBLIC half of the dead domain — 2 `unref_public`, 4 `unreach_public` — as `public_surface` (6.1.0, RG10: a library's exported API is unreferenced in-corpus by construction); an unvouched language (the graph family's confidence 0) as `language_unresolved`; every other row as `reachable_by_name` (9.1.0) — a `[vouched]` unreferenced file is one whose language has every reference site answered (in-corpus or External), a statement about sites, not about use |
| whole-unit T1 twin | `dedup` T1 block spanning an ENTIRE unit whose twin is byte-identical, and whose copy's file is graph-dead (the row carries the copy's dead verdict code, proto 7.0.0) | **none** — an advisory row with its reason | the copy is a whole file, so the dead file's question stands: refused as `unit_not_covered`, `bytes_differ`, `copy_not_dead`, `public_surface`, `language_unresolved` or, past all of them, `reachable_by_name` — the first that holds |

README's permanent stance applies: a class acts only with its own
measured record, and the two file-deleting classes have none — the
precision docs measure where a reference site lands, not whether an
unreferenced file is unused. The plan says what it will not touch and
why. The blind audit that asked the second question of the unreferenced files
of twelve corpora and this repository is recorded in
[EVAL-SET-DEADFILE.md](../EVAL-SET-DEADFILE.md).
Its second generation replays first-parent history and asks what
became of each file that turned unreferenced; its bars and design are
[deadfile-gen2-bars-v1.md](../../contracts/eval/deadfile-gen2-bars-v1.md)
and [deadfile-gen2-design-v1.md](../../contracts/eval/deadfile-gen2-design-v1.md),
and its record is [EVAL-SET-DEADFILE-GEN2.md](../EVAL-SET-DEADFILE-GEN2.md):
on ten fresh repositories neither kind of newly unreferenced file
reached the 100 events the bars ask before it may fail the gate by
default, and most of those files turned out to be wanted. That record
is why, since plan v2.35, an unreferenced file is listed and fails no
gate unless the project declares `[deadcode] block` (booklet 06 §10);
the erase rows are unchanged by it.

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
2. sends the fact tables — and, since proto 7.2.0, the rows' targets
   beside them — to the core's `erase/1`, which answers the
   eraseable-row set and which row stands for each target — the
   PREDICATE and the closure are Haskell's (ADR-008: which rows are
   safe is judgment; the bytes are measurement; see *The target
   closure* below);
3. renders a unified diff (`--format json`: machine rows with
   file/span/class/provenance), each hunk carrying its verdict
   provenance (family, member/segment id, evidence `file:line`) and a
   content hash of the target file;
4. prints the advisory rows (what it will NOT touch, and why);
5. exits 0 without changing user files; the `.ce/` cache may have
   been created or refreshed.

**`ce erase --apply`** additionally requires, in order:

0. no whole-file row among the rows it would act on — the planner
   never makes one eraseable (9.1.0), and a hand-built plan that asks
   for a file deletion is refused by name before anything else runs;
   every write is a line splice;
1. a git repository (revert must be one `git checkout` away);
2. a CLEAN worktree (`git status --porcelain` empty — an erase must
   never be entangled with uncommitted work; the tool's OWN state
   under `.ce/` is exempt from the check, because running the
   planner is what creates it — ce's index is never "uncommitted
   work", batch-7 defect sweep: the carve-out existed in code,
   undocumented);
3. every target inside THIS repository's own subtree — a target below
   a declared submodule (a gitlink in HEAD) refuses by name: the
   worktree the command verified is the superproject's, and a file
   under `cli/tests/` belongs to another repository's index and
   history (plan v2.18 follow-up). Since a declared submodule is a
   reader of this tree and never a measured part (plan v2.18 step #12),
   the superproject's plan holds no row under it and this refusal is
   defence in depth against a hand-built plan; an unreferenced file in the suite
   is reported and erased by the CI leg that runs `ce erase tests --check`
   inside the suite's own repository;
4. every target file's content hash equal to the plan's (a file that
   moved since planning refuses by name — plans are not portable
   across edits);
5. after writing: the source family re-runs and the erased verdicts
   must be GONE — a survivor fails the command loudly (convergence is
   part of apply, not a suggestion);
6. an append-only record in `.ce/erase-log.ndjson`: ts, class,
   file/span, provenance, plan hash — an audit file for human review
   alongside git's recovery path, read back by `ce erase --log`, the
   MCP tool `erase_log` and the GUI erase screen's audit-log section
   (one document, `ce.erase-trail-report`; a line the reader cannot parse
   is named by number and fails `--log`'s exit code).

## The target closure (proto 7.2.0)

A target is one (path, span): the whole file, or one line span in it.
Several candidate rows may name one target — an unreferenced file that is also
a byte-identical twin of a live unit names its path twice, once per
class, and two doc pairs can name one later segment — and the plan
holds ONE row per target. Which row stands is
judgment and lives in the core (`CE.Erase.Cost.keptRows`) since plan
v2.30 step 7b; until then the planner closed the set for itself
(`close_targets`, 7.0.0 O51). The request carries
`targets=[[pathId,start,end]]` aligned with `rows` (dense path ids —
names never cross; `0/0` = the whole file; a `verbatim_doc` row names
a span, the other two classes a whole file; the table in key order),
and the reply carries `kept`, one bit per row. The closure never looks
across paths, so a plan past the core's row ceiling
(`limits.caps.erase_rows`, 4096) goes in several requests split by
path — a path's rows never straddle a request, each request numbers
its own paths from 0 — and plans exactly as one request would; one
path with more rows than a request holds is refused by name:

1. within one target an eraseable row stands over an advisory one —
   since 9.1.0 only a `verbatim_doc` span can be eraseable, so this
   rule fires when two doc pairs name one segment and only one of them
   is a full byte-identical match;
2. with no eraseable row the `dead_file` row stands over the `t1_twin`
   row (`advisoryFirst`): its reason is about the file — the
   categorical `public_surface` bar, the trust fact or
   `reachable_by_name` — where the twin's is about the copy;
3. ties inside one class break to the EARLIEST row, which is why the
   contract asks for key order: the planner sends its rows sorted by
   (path, span, class name), so two producers sending the same rows in
   the same order close the same way.

Two rules of 7.2.0 went with 9.1.0 because no input can reach them:
"an eraseable whole-file row owns its path" (there is none, so a span
row on an unreferenced file's path always stays) and the `licence` ranking
among eraseable rows of one target (every eraseable row is a
`verbatim_doc` row).

A reply without `kept` is refused by name as a pre-7.2.0 core: a plan
closed on this side would hold two rows per target.

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

- plan-then-apply on a fixture tree erases one verbatim-doc span and
  nothing else: the private unreferenced file `orphan.md` and the
  private unreferenced twin `spare.py` (whose `_spare_total` duplicates the live unit
  in `__main__.py`; its `dead_file` row stands for the target) are
  refused by name as `reachable_by_name`, and the public unreferenced file
  `copy.py` (which declares an exported `def`) as `public_surface` —
  all three survive the apply, not only the plan — and the re-run
  proves zero surviving source verdicts;
- a dirty worktree, a drifted file hash, a non-repo root and a
  hand-built whole-file row each refuse BY NAME without touching
  anything;
- an advisory row (live T2 clone) is never planned;
- `ce erase` twice = byte-identical plans (determinism pinned by test);
- the self repo: `ce erase` at HEAD plans ZERO rows (this repository
  keeps itself clean — a non-empty self-plan is a red gate).
