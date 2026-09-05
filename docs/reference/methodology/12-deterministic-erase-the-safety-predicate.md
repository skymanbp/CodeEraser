# Deterministic erase — the safety predicate

[index](../methodology.md) · [← 11 FPR discipline and the guard tier ladder](11-fpr-discipline-and-the-guard-tier-ladder.md) · [→ 13 Unmentioned-declaration advisory — the mention veto](13-unmentioned-declaration-advisory.md)

The erase family judges rows in an erase plan. Rust measures candidate facts and
the Haskell core decides whether a row is deterministic-safe; the boundary is
explicit in the predicate module's ADR-008 comment
([Cost.hs:1-5](../../../core/app/CE/Erase/Cost.hs#L1)). The Rust side calls the
three source families in their normal order, then sends those facts to
`erase/1`; it does not select a winner or infer safety
([gather.rs:24-48](../../../cli/src/erase/gather.rs#L24)). The wire carries dense
integer rows, with row order serving as identity and paths kept on the client
([VERSIONING.md:451](../../../contracts/VERSIONING.md#L451)).

### 1. The row in, the verdict out

The request shape is `rows=[[class,w,x,y,z]]`. The class is a frozen position;
the remaining four cells are facts whose meaning depends on that class
([VERSIONING.md:451-455](../../../contracts/VERSIONING.md#L451)). The client
prepends the class to those four facts, sends the resulting five-integer row,
and reads one `[eraseable, reason]` pair back for every candidate
([wire.rs:21-40](../../../cli/src/erase/wire.rs#L21)). Thus the measurement side
can assemble evidence, but the boolean and reason are produced by `judgeRow`.

Rust's measurement leg preserves raw bytes for the final equality test: the
candidate finder may use masked or normalized family equivalence, but deletion
requires equality of the raw line slices
([gather.rs:1-7](../../../cli/src/erase/gather.rs#L1)). Dead-file candidates
come from the graph report and carry a verdict code plus the graph family's own
per-row confidence (class 3 since 2.32.0)
([gather.rs:120-140](../../../cli/src/erase/gather.rs#L120)). Document candidates
carry both segment word counts and the raw-slice equality bit
([gather.rs:143-174](../../../cli/src/erase/gather.rs#L143)). T1-twin candidates
carry whole-unit coverage, byte equality, copy-file liveness, and the locally
folded per-language unresolved count
([gather.rs:179-234](../../../cli/src/erase/gather.rs#L179)).

### 2. Four frozen class codes, three provable families

The class positions are part of the source contract: position `0` is
retired, `1` is `verbatim_doc`, `2` is `t1_twin`, and `3` is `dead_file`
([Cost.hs:5-20](../../../core/app/CE/Erase/Cost.hs#L5), [model.rs:16-24](../../../cli/src/erase/model.rs#L16)). Each class below is
written as facts → predicate → guard, so the row's evidence and its refusal
path stay visible together.

#### Class 0 — retired at 4.0.0 (superseded at 2.32.0 by class 3)

Class 0 was `dead_file` on the local-count road: its row carried the graph
verdict plus that language's own unresolved-site count, and the predicate
accepted it exactly when the count was zero. 2.32.0 moved the trust judgment
to the family that owns the site ledger — the graph family's per-row
confidence — and shipped it as class 3 below; Rust stopped minting class-0
rows in that same minor, and 4.0.0 removed the predicate clause once the
grace window closed. The per-language fold survives only for class-2 twin
rows ([gather.rs:86-106](../../../cli/src/erase/gather.rs#L86)).

The position stays frozen and is refused **by name** rather than folded into
`unknown class`, so a client still sending it learns which road replaced it
([Erase.hs:32](../../../core/app/CE/Erase.hs#L32),
[model.rs:16-23](../../../cli/src/erase/model.rs#L16)). Renumbering the
survivors would have moved three other frozen codes to reclaim one array
slot, so the name array keeps a `(retired)` placeholder in that position.
The Rust graph leg still refuses a degraded deadcode report by name before
any dead row is minted, so an incomplete graph judgment cannot become proof
([gather.rs:71-83](../../../cli/src/erase/gather.rs#L71)).

#### Class 1 — `verbatim_doc`

**Facts.** A class-1 row is
`[1, verbatim, wordsA, wordsB, bytesEqual]`. The first three values are the
reported verbatim length and the two segment word counts; `bytesEqual` is the
raw-slice equality bit ([Cost.hs:62-65](../../../core/app/CE/Erase/Cost.hs#L62)).
The client chooses the path-lexicographically later segment as the candidate,
then computes equality from the two inclusive line slices
([gather.rs:143-172](../../../cli/src/erase/gather.rs#L143)).

**Predicate.** The full-segment test is integer-only: `verbatim` must be at
least both segment word counts, and the raw bytes must compare equal
([Cost.hs:63-64](../../../core/app/CE/Erase/Cost.hs#L63)). Passing both tests
returns `(True, 0)` ([Cost.hs:52](../../../core/app/CE/Erase/Cost.hs#L52)).

**Guard.** A short verbatim run refuses with reason `2`; a byte mismatch refuses
with reason `3` ([Cost.hs:63-64](../../../core/app/CE/Erase/Cost.hs#L63)). The
guard therefore licenses only a complete segment whose bytes are identical,
not merely a high similarity score.

#### Class 2 — `t1_twin`

**Facts.** A class-2 row is
`[2, unitCovered, bytesEqual, copyDead, langUnresolved]`
([Cost.hs:66-72](../../../core/app/CE/Erase/Cost.hs#L66)). Rust first finds a
dedup block that covers at least one complete cached unit, then records the
coverage bit, raw equality, the target file's dead VERDICT code (`0` when the
graph does not call it dead, else 1–4 in the deadcode family's order), and its
language unresolved count ([gather.rs:189-242](../../../cli/src/erase/gather.rs#L189)).
Until proto 7.0.0 the third fact was a death bit, so the RG10 bar never reached
this road: the acceptance fixture's public `copy.py` twin was refused only
because the `dead_file` row for the same path won the plan's closure by
class-name order (plan v2.29 step 8, O51). The closure now keeps the richer
eraseable row for a path — a twin names the live unit it duplicates, a dead file
names only its death ([mod.rs:100-130](../../../cli/src/erase/mod.rs#L100)).

**Predicate.** `judgeRow` evaluates those facts in source order: coverage must
be `1`, bytes must be equal, the copy must be dead and not a public surface
(`copyDead` outside `publicDeadVerdicts`), and unresolved sites must be `0`;
only then is the row `(True, 0)`
([Cost.hs:66-72](../../../core/app/CE/Erase/Cost.hs#L66)).

**Guard.** The first failed fact wins. Missing whole-unit coverage emits `5`, a
byte mismatch emits `3`, a live copy emits `4`, and unresolved sites emit `1`
([Cost.hs:66-72](../../../core/app/CE/Erase/Cost.hs#L66)). This is why a T1
block that crosses only part of a unit cannot become an erase authorization:
the coverage fact is explicit and checked before the other evidence.

#### Class 3 — `dead_file`, the confidence road (2.32.0)

The same candidate family as class 0 with the trust judgment moved to its owner: fact 1 is no longer a locally folded per-language unresolved count but the graph family's OWN per-row confidence (book 06 §8 — 0 unvouched / 1 vacuous / 2 vouched), and the predicate refuses at 0 ([Cost.hs:76-79](../../../core/app/CE/Erase/Cost.hs#L76)). Since 6.1.0 the verdict code is read as well, and read FIRST: `unref_public` (2) and `unreach_public` (4) are refused whatever their confidence ([Cost.hs:40-50](../../../core/app/CE/Erase/Cost.hs#L40)) — CE.Graph.Dead splits dead along public/private precisely so an exported API cannot be treated as plain dead, and a face that reads past the code turns that firewall into a deletion proposal. The bar is categorical, so it is named before the strength question is asked. Shape: the dead verdict stays bounded 1..4 and the confidence 0..2 ([Erase.hs:34-37](../../../core/app/CE/Erase.hs#L34)). The Rust planner refuses a dead row that carries no confidence — a reply whose request never shipped the ledger licences nothing ([gather.rs:130](../../../cli/src/erase/gather.rs#L130)). Class 0 kept judging through its grace window and RETIRED at 4.0.0, its position frozen and refused by name ([Cost.hs:6-10](../../../core/app/CE/Erase/Cost.hs#L6)); the class-2 twin row deliberately keeps its local count — a twin row is not a graph dead row, so no core confidence exists for it ([Cost.hs:21](../../../core/app/CE/Erase/Cost.hs#L21)).

### 3. The seven reason codes

The advisory vocabulary is frozen at seven positions in the client model
([model.rs:27-39](../../../cli/src/erase/model.rs#L27)). The table records the
meaning and the exact `judgeRow` condition that emits each code. Reason `0` is
the successful verdict; the other six are refusals. The domain only ever
grows — position 6 arrived at 6.1.0 and nothing renumbered, because a frozen
position that moves silently rewrites every plan a reader has already read.

| position/name | meaning | condition in `judgeRow` |
|---|---|---|
| `0 eraseable` | every class-specific safety test passed | class 1 falls through after the full-segment and byte tests, class 2 after all four checks, class 3 after a non-public verdict and a non-zero confidence ([Cost.hs:62-78](../../../core/app/CE/Erase/Cost.hs#L62)) |
| `1 language_unresolved` | the owning language still has unresolved graph sites | class 2: the final guard sees `langUnresolved /= 0`; class 3: the graph family's confidence is `0` ([Cost.hs:66-72](../../../core/app/CE/Erase/Cost.hs#L66), [Cost.hs:76-79](../../../core/app/CE/Erase/Cost.hs#L76)) |
| `2 not_full_segment` | the reported verbatim run does not cover either segment | class 1: `verbatim < wordsA || verbatim < wordsB` ([Cost.hs:62-63](../../../core/app/CE/Erase/Cost.hs#L62)) |
| `3 bytes_differ` | the candidate and its survivor are not byte-identical | class 1 or class 2: `bytesEqual /= 1` ([Cost.hs:62-68](../../../core/app/CE/Erase/Cost.hs#L62)) |
| `6 public_surface` | the dead file is an EXPORT surface, and RG10 forbids acting on one | class 3: `verdict` is 2 `unref_public` or 4 `unreach_public`, tested before the confidence ([Cost.hs:76-77](../../../core/app/CE/Erase/Cost.hs#L76)); class 2: `copyDead` in the same table, tested after the death itself ([Cost.hs:70](../../../core/app/CE/Erase/Cost.hs#L70), table at [Cost.hs:49-50](../../../core/app/CE/Erase/Cost.hs#L49)) |
| `4 copy_not_dead` | the T1 twin's target file is not graph-dead | class 2: `copyDead == 0`; the catch-all malformed-row arm also refuses with this code ([Cost.hs:66-80](../../../core/app/CE/Erase/Cost.hs#L66)) |
| `5 unit_not_covered` | the duplicate block does not cover a whole unit | class 2: `unitCovered /= 1` ([Cost.hs:66-67](../../../core/app/CE/Erase/Cost.hs#L66)) |

The order in the table is operational, not descriptive. For a class-2 row with
several bad facts, the first failing guard determines the one reason returned;
there is no second pass that chooses a more convenient explanation
([Cost.hs:58-60](../../../core/app/CE/Erase/Cost.hs#L58)). A row whose shape does
not match any class is refused by the final catch-all rather than reaching an
eraseable result ([Cost.hs:80](../../../core/app/CE/Erase/Cost.hs#L80)).

### 4. Capacity and degraded replies

`eraseRowCap` is `4096` ([Cost.hs:52-56](../../../core/app/CE/Erase/Cost.hs#L52)).
The wire contract repeats that ceiling: an over-cap request returns a complete
degraded reply with `fail:true` and an empty judgment table, so no row can be
authorized from an over-cap computation
([VERSIONING.md:460-462](../../../contracts/VERSIONING.md#L460)).

The erase client treats degraded as an error rather than interpreting an empty
table as “nothing to erase”: `wire.rs` calls `refuse_degraded` before decoding
rows and checks that the decoded count equals the candidate count
([wire.rs:29-40](../../../cli/src/erase/wire.rs#L29)). The shared refusal helper
requires `degraded == false` and reports cap-mirror drift when it is not
([lockstep.rs:103-109](../../../cli/src/lockstep.rs#L103)).

### 5. `--apply`: predicates before writes

Applying a plan enters the executor only after it has collected the rows marked
eraseable. The executor's precondition function is deliberately ordered
([apply.rs:40-46](../../../cli/src/erase/apply.rs#L40)).

1. **Repository identity.** `git rev-parse --show-toplevel` must succeed, and
   its canonical path must equal the supplied erase root
   ([apply.rs:46-60](../../../cli/src/erase/apply.rs#L46)).
2. **Worktree cleanliness.** `git status --porcelain` must succeed; after the
   `.ce` state directory is removed from the comparison, no user dirt may
   remain ([apply.rs:61-81](../../../cli/src/erase/apply.rs#L61)).
3. **Target identity.** Every target is read again and its FNV-1a content hash
   must equal the hash captured in the plan
   ([apply.rs:82-91](../../../cli/src/erase/apply.rs#L82)).

Only after all three checks does the executor write targets and append the
audit records ([apply.rs:19-37](../../../cli/src/erase/apply.rs#L19)). The apply
entry then re-plans the tree and fails if any applied eraseable verdict
survives; convergence is part of the operation's result
([mod.rs:54-85](../../../cli/src/erase/mod.rs#L54)).

### 6. No tuning surface

There is no knob echo and no client-selectable threshold for this family. The
core's own comment freezes the reason: “a knob that loosens "safe" would be a
licence to guess” ([Cost.hs:32-33](../../../core/app/CE/Erase/Cost.hs#L32)). The
versioned wire contract makes the same boundary explicit by rejecting knob
rows as `error/contract` ([VERSIONING.md:456](../../../contracts/VERSIONING.md#L456)).

### 7. Not found in source

This booklet uses the index's arrow phrase **facts → predicate → verdict** as a
reading aid. The three words are source vocabulary in the predicate module's
comments, where facts are measured, the predicate chooses safety, and
`judgeRow` returns the verdict pair
([Cost.hs:1-5](../../../core/app/CE/Erase/Cost.hs#L1), [Cost.hs:58-61](../../../core/app/CE/Erase/Cost.hs#L58)).
“Guard” is the only explanatory label that is not a named wire field or a
Haskell identifier here; it means the `|` conditions shown beside each class,
not an additional rule ([Cost.hs:62-75](../../../core/app/CE/Erase/Cost.hs#L62),
[VERSIONING.md:448-462](../../../contracts/VERSIONING.md#L448)). No other
constant, class, reason, degraded behavior, or apply condition in this booklet
is an inferred term: each is named in the source links above.
