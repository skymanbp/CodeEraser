# Deterministic erase — the safety predicate

[index](../methodology.md) · [← 11 FPR discipline and the guard tier ladder](11-fpr-discipline-and-the-guard-tier-ladder.md)

The erase family judges rows in an erase plan. Rust measures candidate facts and
the Haskell core decides whether a row is deterministic-safe; the boundary is
explicit in the predicate module's ADR-008 comment
([Cost.hs:1-5](../../../core/app/CE/Erase/Cost.hs#L1)). The Rust side calls the
three source families in their normal order, then sends those facts to
`erase/1`; it does not select a winner or infer safety
([gather.rs:25-49](../../../cli/src/erase/gather.rs#L25)). The wire carries dense
integer rows, with row order serving as identity and paths kept on the client
([VERSIONING.md:231-240](../../../contracts/VERSIONING.md#L289)).

### 1. The row in, the verdict out

The request shape is `rows=[[class,w,x,y,z]]`. The class is a frozen position;
the remaining four cells are facts whose meaning depends on that class
([VERSIONING.md:234-238](../../../contracts/VERSIONING.md#L292)). The client
prepends the class to those four facts, sends the resulting five-integer row,
and reads one `[eraseable, reason]` pair back for every candidate
([wire.rs:21-40](../../../cli/src/erase/wire.rs#L21)). Thus the measurement side
can assemble evidence, but the boolean and reason are produced by `judgeRow`.

Rust's measurement leg preserves raw bytes for the final equality test: the
candidate finder may use masked or normalized family equivalence, but deletion
requires equality of the raw line slices
([gather.rs:1-7](../../../cli/src/erase/gather.rs#L1)). Dead-file candidates
come from the graph report and carry a verdict code plus the unresolved-site
count for the file's language
([gather.rs:79-118](../../../cli/src/erase/gather.rs#L79)). Document candidates
carry both segment word counts and the raw-slice equality bit
([gather.rs:122-153](../../../cli/src/erase/gather.rs#L126)). T1-twin candidates
carry whole-unit coverage, byte equality, copy-file liveness, and the same
language unresolved count
([gather.rs:156-211](../../../cli/src/erase/gather.rs#L160)).

### 2. Four frozen class codes, three provable families

The class positions are part of the source contract: position `0` is
`dead_file`, `1` is `verbatim_doc`, and `2` is `t1_twin`
([Cost.hs:5-10](../../../core/app/CE/Erase/Cost.hs#L5)). Each class below is
written as facts → predicate → guard, so the row's evidence and its refusal
path stay visible together.

#### Class 0 — `dead_file` (superseded at 2.32.0 by class 3)

**Facts.** A class-0 row has the shape
`[0, _verdict, langUnresolved, _, _]`: the graph verdict is present and the
language-specific unresolved-site count is the trust-boundary fact
([Cost.hs:31-34](../../../core/app/CE/Erase/Cost.hs#L42)). Rust obtains that
count by folding unresolved paths by language and obtains the verdict code from
the deadcode report
([gather.rs:79-118](../../../cli/src/erase/gather.rs#L79)).

**Predicate.** `judgeRow` accepts the row exactly when `langUnresolved == 0`;
the successful result is `(True, 0)`
([Cost.hs:32-34](../../../core/app/CE/Erase/Cost.hs#L43)).

**Guard.** Any non-zero unresolved count emits reason `1` and keeps the row
`False` ([Cost.hs:32-34](../../../core/app/CE/Erase/Cost.hs#L43)). Before that
row exists, the Rust graph leg itself refuses a degraded deadcode report by
name, so an empty or incomplete graph judgment cannot be treated as proof
([gather.rs:64-76](../../../cli/src/erase/gather.rs#L64)).

#### Class 1 — `verbatim_doc`

**Facts.** A class-1 row is
`[1, verbatim, wordsA, wordsB, bytesEqual]`. The first three values are the
reported verbatim length and the two segment word counts; `bytesEqual` is the
raw-slice equality bit ([Cost.hs:35-38](../../../core/app/CE/Erase/Cost.hs#L46)).
The client chooses the path-lexicographically later segment as the candidate,
then computes equality from the two inclusive line slices
([gather.rs:122-151](../../../cli/src/erase/gather.rs#L126)).

**Predicate.** The full-segment test is integer-only: `verbatim` must be at
least both segment word counts, and the raw bytes must compare equal
([Cost.hs:35-38](../../../core/app/CE/Erase/Cost.hs#L46)). Passing both tests
returns `(True, 0)` ([Cost.hs:35-38](../../../core/app/CE/Erase/Cost.hs#L46)).

**Guard.** A short verbatim run refuses with reason `2`; a byte mismatch refuses
with reason `3` ([Cost.hs:35-38](../../../core/app/CE/Erase/Cost.hs#L46)). The
guard therefore licenses only a complete segment whose bytes are identical,
not merely a high similarity score.

#### Class 2 — `t1_twin`

**Facts.** A class-2 row is
`[2, unitCovered, bytesEqual, copyFileDead, langUnresolved]`
([Cost.hs:39-44](../../../core/app/CE/Erase/Cost.hs#L50)). Rust first finds a
dedup block that covers at least one complete cached unit, then records the
coverage bit, raw equality, whether the target file is graph-dead, and its
language unresolved count ([gather.rs:156-211](../../../cli/src/erase/gather.rs#L160)).

**Predicate.** `judgeRow` evaluates those facts in source order: coverage must
be `1`, bytes must be equal, the copy file must be dead, and unresolved sites
must be `0`; only then is the row `(True, 0)`
([Cost.hs:39-44](../../../core/app/CE/Erase/Cost.hs#L50)).

**Guard.** The first failed fact wins. Missing whole-unit coverage emits `5`, a
byte mismatch emits `3`, a live copy emits `4`, and unresolved sites emit `1`
([Cost.hs:39-44](../../../core/app/CE/Erase/Cost.hs#L50)). This is why a T1
block that crosses only part of a unit cannot become an erase authorization:
the coverage fact is explicit and checked before the other evidence.

#### Class 3 — `dead_file`, the confidence road (2.32.0)

The same candidate family as class 0 with the trust judgment moved to its owner: fact 1 is no longer a locally folded per-language unresolved count but the graph family's OWN per-row confidence (book 06 §8 — 0 unvouched / 1 vacuous / 2 vouched), and the predicate refuses only at 0 ([Cost.hs:56](../../../core/app/CE/Erase/Cost.hs#L56)). Shape: the dead verdict stays bounded 1..4 and the confidence 0..2 ([Erase.hs:37](../../../core/app/CE/Erase.hs#L37)). The Rust planner refuses a dead row that carries no confidence — a reply whose request never shipped the ledger licences nothing ([gather.rs:114](../../../cli/src/erase/gather.rs#L114)). Class 0 keeps judging unchanged for the grace window (the staleDocs discipline) and retires in a later minor; the class-2 twin row deliberately keeps its local count — a twin row is not a graph dead row, so no core confidence exists for it ([Cost.hs:16](../../../core/app/CE/Erase/Cost.hs#L16)).

### 3. The six reason codes

The advisory vocabulary is frozen at six positions in the client model
([model.rs:14-25](../../../cli/src/erase/model.rs#L14)). The table records the
meaning and the exact `judgeRow` condition that emits each code. Reason `0` is
the successful verdict; the other five are refusals.

| position/name | meaning | condition in `judgeRow` |
|---|---|---|
| `0 eraseable` | every class-specific safety test passed | class 0 falls through after `langUnresolved == 0`, class 1 after full-segment and byte tests, or class 2 after all four checks ([Cost.hs:32-44](../../../core/app/CE/Erase/Cost.hs#L43)) |
| `1 language_unresolved` | the owning language still has unresolved graph sites | class 0: `langUnresolved /= 0`; class 2: the final guard sees `langUnresolved /= 0` ([Cost.hs:32-34](../../../core/app/CE/Erase/Cost.hs#L43), [Cost.hs:39-44](../../../core/app/CE/Erase/Cost.hs#L50)) |
| `2 not_full_segment` | the reported verbatim run does not cover either segment | class 1: `verbatim < wordsA || verbatim < wordsB` ([Cost.hs:35-38](../../../core/app/CE/Erase/Cost.hs#L46)) |
| `3 bytes_differ` | the candidate and its survivor are not byte-identical | class 1 or class 2: `bytesEqual /= 1` ([Cost.hs:35-42](../../../core/app/CE/Erase/Cost.hs#L46)) |
| `4 copy_not_dead` | the T1 twin's target file is not graph-dead | class 2: `copyFileDead /= 1`; the catch-all malformed-row arm also refuses with this code ([Cost.hs:39-45](../../../core/app/CE/Erase/Cost.hs#L50)) |
| `5 unit_not_covered` | the duplicate block does not cover a whole unit | class 2: `unitCovered /= 1` ([Cost.hs:39-41](../../../core/app/CE/Erase/Cost.hs#L50)) |

The order in the table is operational, not descriptive. For a class-2 row with
several bad facts, the first failing guard determines the one reason returned;
there is no second pass that chooses a more convenient explanation
([Cost.hs:28-30](../../../core/app/CE/Erase/Cost.hs#L39)). A row whose shape does
not match any class is refused by the final catch-all rather than reaching an
eraseable result ([Cost.hs:28-31](../../../core/app/CE/Erase/Cost.hs#L39)).

### 4. Capacity and degraded replies

`eraseRowCap` is `4096` ([Cost.hs:22-26](../../../core/app/CE/Erase/Cost.hs#L33)).
The wire contract repeats that ceiling: an over-cap request returns a complete
degraded reply with `fail:true` and an empty judgment table, so no row can be
authorized from an over-cap computation
([VERSIONING.md:240-245](../../../contracts/VERSIONING.md#L298)).

The erase client treats degraded as an error rather than interpreting an empty
table as “nothing to erase”: `wire.rs` calls `refuse_degraded` before decoding
rows and checks that the decoded count equals the candidate count
([wire.rs:29-40](../../../cli/src/erase/wire.rs#L29)). The shared refusal helper
requires `degraded == false` and reports cap-mirror drift when it is not
([lockstep.rs:103-109](../../../cli/src/lockstep.rs#L103)).

### 5. `--apply`: predicates before writes

Applying a plan enters the executor only after it has collected the rows marked
eraseable. The executor's precondition function is deliberately ordered
([apply.rs:39-45](../../../cli/src/erase/apply.rs#L39)).

1. **Repository identity.** `git rev-parse --show-toplevel` must succeed, and
   its canonical path must equal the supplied erase root
   ([apply.rs:45-63](../../../cli/src/erase/apply.rs#L45)).
2. **Worktree cleanliness.** `git status --porcelain` must succeed; after the
   `.ce` state directory is removed from the comparison, no user dirt may
   remain ([apply.rs:64-83](../../../cli/src/erase/apply.rs#L64)).
3. **Target identity.** Every target is read again and its FNV-1a content hash
   must equal the hash captured in the plan
   ([apply.rs:84-93](../../../cli/src/erase/apply.rs#L84)).

Only after all three checks does the executor write targets and append the
audit records ([apply.rs:18-36](../../../cli/src/erase/apply.rs#L18)). The apply
entry then re-plans the tree and fails if any applied eraseable verdict
survives; convergence is part of the operation's result
([mod.rs:53-84](../../../cli/src/erase/mod.rs#L53)).

### 6. No tuning surface

There is no knob echo and no client-selectable threshold for this family. The
core's own comment freezes the reason: “a knob that loosens "safe" would be a
licence to guess” ([Cost.hs:11-16](../../../core/app/CE/Erase/Cost.hs#L22)). The
versioned wire contract makes the same boundary explicit by rejecting knob
rows as `error/contract` ([VERSIONING.md:237-240](../../../contracts/VERSIONING.md#L295)).

### 7. Not found in source

This booklet uses the index's arrow phrase **facts → predicate → verdict** as a
reading aid. The three words are source vocabulary in the predicate module's
comments, where facts are measured, the predicate chooses safety, and
`judgeRow` returns the verdict pair
([Cost.hs:1-5](../../../core/app/CE/Erase/Cost.hs#L1), [Cost.hs:28-31](../../../core/app/CE/Erase/Cost.hs#L39)).
“Guard” is the only explanatory label that is not a named wire field or a
Haskell identifier here; it means the `|` conditions shown beside each class,
not an additional rule ([Cost.hs:31-45](../../../core/app/CE/Erase/Cost.hs#L42),
[VERSIONING.md:234-245](../../../contracts/VERSIONING.md#L292)). No other
constant, class, reason, degraded behavior, or apply condition in this booklet
is an inferred term: each is named in the source links above.
