# Deterministic erase — the safety predicate

[index](../methodology.md) · [← 11 FPR discipline and the guard tier ladder](11-fpr-discipline-and-the-guard-tier-ladder.md) · [→ 13 Unmentioned-declaration advisory — the mention veto](13-unmentioned-declaration-advisory.md)

The erase family judges rows in an erase plan. Rust measures candidate facts and
the Haskell core decides whether a row is deterministic-safe; the boundary is
explicit in the predicate module's ADR-008 comment
([Cost.hs:1-5](../../../core/app/CE/Erase/Cost.hs#L1)). The Rust side calls the
three source families in their normal order, then sends those facts to
`erase/1`; it does not select a winner or infer safety
([gather.rs:24-69](../../../cli/src/erase/gather.rs#L24)). The wire carries dense
integer rows, with row order serving as identity and paths kept on the client
([VERSIONING-ARCHIVE-2.15-2.33.md:135-136](../../../contracts/VERSIONING-ARCHIVE-2.15-2.33.md#L135)).

### 1. The row in, the verdict out

The request shape is `rows=[[class,w,x,y,z]]`. The class is a frozen position;
the remaining four cells are facts whose meaning depends on that class
([VERSIONING-ARCHIVE-2.15-2.33.md:135-139](../../../contracts/VERSIONING-ARCHIVE-2.15-2.33.md#L135)). The client
prepends the class to those four facts, sends the resulting five-integer row,
and reads one `[eraseable, reason]` pair back for every candidate
([wire.rs:35-55](../../../cli/src/erase/wire.rs#L35)). Thus the measurement side
can assemble evidence, but the boolean and reason are produced by `judgeRow`.

Since 7.2.0 (plan v2.30 step 7b) the request also carries the rows' targets —
one `[pathId, start, end]` per row, path ids dense in request order and `0/0`
for a whole file — and the reply carries `kept`, one bit per row saying which
row STANDS for its target ([wire.rs:97-113](../../../cli/src/erase/wire.rs#L97),
[Erase.hs:55-61](../../../core/app/CE/Erase.hs#L55)). The client reads both
tables length-locked and refuses a reply without `kept` as a pre-7.2.0 core
([wire.rs:116-128](../../../cli/src/erase/wire.rs#L116)); the closure itself is
§2's class-2 paragraph and `keptRows`.

Rust's measurement leg preserves raw bytes for the final equality test: the
candidate finder may use masked or normalized family equivalence, but deletion
requires equality of the raw line slices
([gather.rs:1-7](../../../cli/src/erase/gather.rs#L1)). Dead-file candidates
come from the graph report and carry a verdict code plus the graph family's own
per-row confidence (class 3 since 2.32.0)
([gather.rs:111-137](../../../cli/src/erase/gather.rs#L111)). Document candidates
carry both segment word counts and the raw-slice equality bit
([doc_rows.rs:13-85](../../../cli/src/erase/doc_rows.rs#L13)). T1-twin candidates
carry whole-unit coverage, byte equality, copy-file liveness, and the locally
folded per-language unresolved count
([gather.rs:143-206](../../../cli/src/erase/gather.rs#L143)).

### 2. Four frozen class codes, three judged families, one that erases

The class positions are part of the source contract: position `0` is
retired, `1` is `verbatim_doc`, `2` is `t1_twin`, and `3` is `dead_file`
([Cost.hs:5-26](../../../core/app/CE/Erase/Cost.hs#L5), [Cost.hs:81](../../../core/app/CE/Erase/Cost.hs#L81)). Each class below is
written as facts → predicate → guard, so the row's evidence and its refusal
path stay visible together.

Since 9.1.0 (lane v235_licence) only class 1 has an eraseable outcome.
Classes 2 and 3 would delete a whole file, and their evidence is the graph's
dead verdict, which answers whether any reference site of the corpus
lands on the file. A file is also used with no such site — run by its
name, found by a tool's convention, served or read by its path — so a
whole-file row past every other bar is refused as `reachable_by_name`
and stays in the plan as an advisory row
([Cost.hs:27-37](../../../core/app/CE/Erase/Cost.hs#L27)). The released
1.9.2 offered for deletion two Python scripts that import nothing and a
Java tree's `package-info.java` and `module-info.java`; the standing
rule that a class acts only with its own measured record has no record
to read for these two classes, because the precision docs measure where
a site lands, not whether an unreferenced file is unused.

#### Class 0 — retired at 4.0.0 (superseded at 2.32.0 by class 3)

Class 0 was `dead_file` on the local-count road: its row carried the graph
verdict plus that language's own unresolved-site count, and the predicate
accepted it exactly when the count was zero. 2.32.0 moved the trust judgment
to the family that owns the site ledger — the graph family's per-row
confidence — and shipped it as class 3 below; Rust stopped minting class-0
rows in that same minor, and 4.0.0 removed the predicate clause once the
grace window closed. The per-language fold survives only for class-2 twin
rows ([gather.rs:89-109](../../../cli/src/erase/gather.rs#L89)).

The position stays frozen and is refused **by name** rather than folded into
`unknown class`, so a client still sending it learns which road replaced it
([Erase.hs:40](../../../core/app/CE/Erase.hs#L40), the text owned by [Retired.hs:30-31](../../../core/app/CE/Wire/Retired.hs#L30),
[Cost.hs:81](../../../core/app/CE/Erase/Cost.hs#L81)). Renumbering the
survivors would have moved three other frozen codes to reclaim one array
slot, so the name array keeps a `(retired)` placeholder in that position.
The Rust graph leg still refuses a degraded deadcode report by name before
any dead row is minted, so an incomplete graph judgment cannot become proof
([gather.rs:74-87](../../../cli/src/erase/gather.rs#L74)).

#### Class 1 — `verbatim_doc`

**Facts.** A class-1 row is
`[1, verbatim, wordsA, wordsB, bytesEqual]`. The first three values are the
reported verbatim length and the two segment word counts; `bytesEqual` is the
raw-slice equality bit ([Cost.hs:94-97](../../../core/app/CE/Erase/Cost.hs#L94)).
The client counts, for each side, the link sites whose fragment names the
heading (or HTML id) of the section holding the segment — the nearest heading
at or above its first line, slugged by the ladder's own rule; a segment above
the first heading has no section and counts 0 — and chooses the side with FEWER
such links as the candidate, the path-lexicographically later one on equal
counts; it then computes equality from the two inclusive line slices
([doc_rows.rs:13-85](../../../cli/src/erase/doc_rows.rs#L13)). Since release
1.10.0 the candidate's count rides beside the rows as the `linked` table
(`[row, links]` for each linked row, strictly ascending; [Erase.hs:101](../../../core/app/CE/Erase.hs#L101)).

**Predicate.** The full-segment test is integer-only: `verbatim` must be at
least both segment word counts, and the raw bytes must compare equal
([Cost.hs:95-96](../../../core/app/CE/Erase/Cost.hs#L95)). Passing both tests
returns `(True, 0)` ([Cost.hs:97](../../../core/app/CE/Erase/Cost.hs#L97)).

**Guard.** A short verbatim run refuses with reason `2`; a byte mismatch refuses
with reason `3` ([Cost.hs:95-96](../../../core/app/CE/Erase/Cost.hs#L95)). The
guard therefore licenses only a complete segment whose bytes are identical,
not merely a high similarity score. A row that passes and whose section is
linked is refused with reason `8` (`judgeLinked`, [Cost.hs:125](../../../core/app/CE/Erase/Cost.hs#L125)): a section
a link lands on is referenced, and emptying it is not provably safe. An erased
copy leaves its heading in place, so remove the heading or put a link to the
surviving copy.

#### Class 2 — `t1_twin`

**Facts.** A class-2 row is
`[2, unitCovered, bytesEqual, copyDead, langUnresolved]`
([Cost.hs:98-104](../../../core/app/CE/Erase/Cost.hs#L98)). Rust first finds a
dedup block that covers at least one complete cached unit, then records the
coverage bit, raw equality, the target file's dead VERDICT code (`0` when the
graph does not call it dead, else 1–4 in the deadcode family's order), and its
language unresolved count ([gather.rs:151-206](../../../cli/src/erase/gather.rs#L151)).
Until proto 7.0.0 the third fact was a death bit, so the RG10 bar never reached
this road: the acceptance fixture's public `copy.py` twin was refused only
because the `dead_file` row for the same path won the plan's closure by
class-name order (plan v2.29 step 8, O51). Since 7.2.0 the closure is
the core's: an eraseable row stands over an advisory one for its target, the
dead file's refusal stands over the twin's among advisory ones (its reason is
about the file), and a tie inside one class breaks to the earliest row
([Cost.hs:150-180](../../../core/app/CE/Erase/Cost.hs#L150)). Two rules of
7.2.0 left with 9.1.0, when no input could reach them any more: an eraseable
whole-file row owning its path, and the `licence` ranking that put an
eraseable twin over an eraseable dead file.

**Predicate.** `judgeRow` evaluates those facts in source order: coverage must
be `1`, bytes must be equal, the copy must be dead and not a public surface
(`copyDead` outside `publicDeadVerdicts`), and unresolved sites must be `0`;
a row past all four is `(False, 7)`, never `(True, 0)`
([Cost.hs:104](../../../core/app/CE/Erase/Cost.hs#L104)).

**Guard.** The first failed fact wins. Missing whole-unit coverage emits `5`, a
byte mismatch emits `3`, a live copy emits `4`, a public copy `6`, unresolved
sites emit `1`, and `7` closes the list
([Cost.hs:99-104](../../../core/app/CE/Erase/Cost.hs#L99)). The coverage fact
is explicit and checked before the other evidence, so the reason a reader sees
for a partial-unit block is the coverage, not the liveness.

#### Class 3 — `dead_file`, the confidence road (2.32.0)

The same candidate family as class 0 with the trust judgment moved to its owner: fact 1 is no longer a locally folded per-language unresolved count but the graph family's OWN per-row confidence (book 06 §8 — 0 unvouched / 1 vacuous / 2 vouched), and the predicate refuses at 0 ([Cost.hs:114](../../../core/app/CE/Erase/Cost.hs#L114)) and as `reachable_by_name` past it (9.1.0, [Cost.hs:115](../../../core/app/CE/Erase/Cost.hs#L115)): the trust fact is named first because it says the graph could not even show the file unreferenced, which tells the reader more than the residual rule that holds for every whole-file row. Since 6.1.0 the verdict code is read as well, and read FIRST: `unref_public` (2) and `unreach_public` (4) are refused whatever their confidence ([Cost.hs:65-75](../../../core/app/CE/Erase/Cost.hs#L65)) — CE.Graph.Dead splits dead along public/private precisely so an exported API cannot be treated as plain dead, and a face that reads past the code turns that firewall into a deletion proposal. The bar is categorical, so it is named before the strength question is asked. Shape: the dead verdict stays bounded 1..4 and the confidence 0..2 ([Erase.hs:47-51](../../../core/app/CE/Erase.hs#L47)). The Rust planner refuses a dead row that carries no confidence — a reply whose request never shipped the ledger is refused by name ([gather.rs:126](../../../cli/src/erase/gather.rs#L126)). Class 0 kept judging through its grace window and RETIRED at 4.0.0, its position frozen and refused by name ([Cost.hs:6-10](../../../core/app/CE/Erase/Cost.hs#L6)); the class-2 twin row deliberately keeps its local count — a twin row is not a graph dead row, so no core confidence exists for it ([Cost.hs:19-21](../../../core/app/CE/Erase/Cost.hs#L19)).

### 3. The nine reason codes

The advisory vocabulary is frozen at nine positions in the core's table
([Cost.hs:82](../../../core/app/CE/Erase/Cost.hs#L82)); since release 1.10.0 the
measuring side reads the names from the definition package and keeps no copy.
The table records the meaning and the exact condition that emits each code.
Reason `0` is the successful verdict; the other eight are refusals. The domain
only ever grows — position 6 arrived at 6.1.0, position 7 at 9.1.0 and
position 8 at 10.0.0, and nothing renumbered, because a frozen position that
moves silently rewrites every plan a reader has already read.

| position/name | meaning | condition in `judgeRow` |
|---|---|---|
| `0 eraseable` | every class-specific safety test passed | class 1 alone, after the full-segment and byte tests ([Cost.hs:94-97](../../../core/app/CE/Erase/Cost.hs#L94)) |
| `1 language_unresolved` | the owning language still has unresolved graph sites | class 2: the final guard sees `langUnresolved /= 0`; class 3: the graph family's confidence is `0` ([Cost.hs:103](../../../core/app/CE/Erase/Cost.hs#L103), [Cost.hs:112-114](../../../core/app/CE/Erase/Cost.hs#L112)) |
| `2 not_full_segment` | the reported verbatim run does not cover either segment | class 1: `verbatim < wordsA || verbatim < wordsB` ([Cost.hs:94-95](../../../core/app/CE/Erase/Cost.hs#L94)) |
| `3 bytes_differ` | the candidate and its survivor are not byte-identical | class 1 or class 2: `bytesEqual /= 1` ([Cost.hs:94-100](../../../core/app/CE/Erase/Cost.hs#L94)) |
| `6 public_surface` | the dead file is an EXPORT surface, and RG10 forbids acting on one | class 3: `verdict` is 2 `unref_public` or 4 `unreach_public`, tested before the confidence ([Cost.hs:112-113](../../../core/app/CE/Erase/Cost.hs#L112)); class 2: `copyDead` in the same table, tested after the death itself ([Cost.hs:102](../../../core/app/CE/Erase/Cost.hs#L102), table at [Cost.hs:74-75](../../../core/app/CE/Erase/Cost.hs#L74)) |
| `4 copy_not_dead` | the T1 twin's target file is not graph-dead | class 2: `copyDead == 0`; the catch-all malformed-row arm also refuses with this code ([Cost.hs:98-101](../../../core/app/CE/Erase/Cost.hs#L98), [Cost.hs:116](../../../core/app/CE/Erase/Cost.hs#L116)) |
| `5 unit_not_covered` | the duplicate block does not cover a whole unit | class 2: `unitCovered /= 1` ([Cost.hs:98-99](../../../core/app/CE/Erase/Cost.hs#L98)) |
| `7 reachable_by_name` | a whole-file row whose every other fact holds: nothing in the tree references the file, which is not evidence that nothing uses it | class 2 after all five checks, class 3 after a non-public verdict and a non-zero confidence ([Cost.hs:98-104](../../../core/app/CE/Erase/Cost.hs#L98), [Cost.hs:112-115](../../../core/app/CE/Erase/Cost.hs#L112)) |
| `8 linked_section` | a byte-identical doc copy whose section a link's fragment lands on: the section is referenced | class 1: a row `judgeRow` would make eraseable, with a `linked` count above 0 (`judgeLinked`, [Cost.hs:125-128](../../../core/app/CE/Erase/Cost.hs#L125)) |

The order in the table is operational, not descriptive. For a class-2 row with
several bad facts, the first failing guard determines the one reason returned;
there is no second pass that chooses a more convenient explanation
([Cost.hs:90-92](../../../core/app/CE/Erase/Cost.hs#L90)). A row whose shape does
not match any class is refused by the final catch-all rather than reaching an
eraseable result ([Cost.hs:116](../../../core/app/CE/Erase/Cost.hs#L116)).

### 4. Capacity and degraded replies

`eraseRowCap` is `4096` ([Cost.hs:84-88](../../../core/app/CE/Erase/Cost.hs#L84)).
The wire contract repeats that ceiling: an over-cap request returns a complete
degraded reply with `fail:true` and an empty judgment table, so no row can be
authorized from an over-cap computation
([VERSIONING-ARCHIVE-2.15-2.33.md:144-146](../../../contracts/VERSIONING-ARCHIVE-2.15-2.33.md#L144)).

The erase client never sends a request over the ceiling the package states
(`limits.caps.erase_rows`): a plan with more candidates goes in several
requests over the one core link, split by path the way scan/1 splits by file —
the closure chooses among the rows of one path and nothing across paths, so a
path's rows never straddle a request and the split plan is the one-request
plan byte for byte; each request carries its own dense path ids, and the
verdicts concatenate in request order (lane v235_graphtrust, item T5). One path
with more candidates than a request holds is refused by name before anything
is sent ([wire.rs:82-106](../../../cli/src/erase/wire.rs#L82)). A degraded reply,
should one still arrive, is the plan's refusal, naming the request, rather than
an empty table read as “nothing to erase”, and each request's decoded count
must equal its candidate count ([wire.rs:22-56](../../../cli/src/erase/wire.rs#L22), [wire.rs:118-128](../../../cli/src/erase/wire.rs#L118)).

### 5. `--apply`: predicates before writes

Applying a plan enters the executor only after it has collected the rows marked
eraseable, and refuses by name any whole-file row among them before a
precondition runs: no plan holds one since 9.1.0, so every write is a line
splice ([apply.rs:19-32](../../../cli/src/erase/apply.rs#L19)). The
executor's precondition function is deliberately ordered
([apply.rs:47-53](../../../cli/src/erase/apply.rs#L47)).

1. **Repository identity.** `git rev-parse --show-toplevel` must succeed, and
   its canonical path must equal the supplied erase root
   ([apply.rs:55-67](../../../cli/src/erase/apply.rs#L55)).
2. **Worktree cleanliness.** `git status --porcelain` must succeed; after the
   `.ce` state directory is removed from the comparison, no user dirt may
   remain ([apply.rs:68-87](../../../cli/src/erase/apply.rs#L68)).
3. **Inside this repository.** No target may sit at or below a submodule; such
   a row is refused by name and stays planned, its removal made in that
   repository by hand ([apply.rs:102-121](../../../cli/src/erase/apply.rs#L102)).
4. **Target identity.** Every target is read again and its FNV-1a content hash
   must equal the hash captured in the plan
   ([apply.rs:89-98](../../../cli/src/erase/apply.rs#L89)).

Only after all four checks does the executor write targets and append the
audit records ([apply.rs:38-43](../../../cli/src/erase/apply.rs#L38)). The apply
entry then re-plans the tree and fails if any applied eraseable verdict
survives; convergence is part of the operation's result
([mod.rs:92-124](../../../cli/src/erase/mod.rs#L92)).

### 6. No tuning surface

There is no knob echo and no client-selectable threshold for this family. The
core's own comment freezes the reason: “a knob that loosens "safe" would be a
licence to guess” ([Cost.hs:49-50](../../../core/app/CE/Erase/Cost.hs#L49)). The
versioned wire contract makes the same boundary explicit by rejecting knob
rows as `error/contract` ([VERSIONING-ARCHIVE-2.15-2.33.md:140](../../../contracts/VERSIONING-ARCHIVE-2.15-2.33.md#L140)).

### 7. Not found in source

This booklet uses the index's arrow phrase **facts → predicate → verdict** as a
reading aid. The three words are source vocabulary in the predicate module's
comments, where facts are measured, the predicate chooses safety, and
`judgeRow` returns the verdict pair
([Cost.hs:1-5](../../../core/app/CE/Erase/Cost.hs#L1), [Cost.hs:90-93](../../../core/app/CE/Erase/Cost.hs#L90)).
“Guard” is the only explanatory label that is not a named wire field or a
Haskell identifier here; it means the `|` conditions shown beside each class,
not an additional rule ([Cost.hs:94-100](../../../core/app/CE/Erase/Cost.hs#L94),
[VERSIONING-ARCHIVE-2.15-2.33.md:132-146](../../../contracts/VERSIONING-ARCHIVE-2.15-2.33.md#L132)). No other
constant, class, reason, degraded behavior, or apply condition in this booklet
is an inferred term: each is named in the source links above.
