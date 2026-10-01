# Intra-function dead code — the flow graph, reachability and liveness

[index](../methodology.md) · [← 16 Code query and architecture rules — Datalog over the index's facts](16-code-query-and-architecture-rules-datalog-over-the-index.md) · [→ 18 Clone merge suggestions — anti-unification over the clone families' trees](18-clone-merge-suggestions-anti-unification.md)

Booklet 06 judges dead code between files: a file no edge reaches. This family judges it inside
one function: a statement no path from the entry reaches, a store no path reads before the next
store or the exit, a local nothing reads, a parameter nothing reads. The drift it answers is the
one a model leaves when it edits a function instead of rewriting it — the early `return` pasted
above the old tail, the variable computed and then recomputed, the helper value nobody consumes
any more. The split is ADR-008's, seventh instalment: Rust lowers every unit of a language its
`FlowSpec` table knows into four integer tables, and the core builds the control-flow graph,
walks reachability and backward liveness and answers the findings over the fourteenth wire family,
`flow/1`, since proto 7.4.0 ([Flow.hs:5-18](../../../core/app/CE/Flow.hs#L5),
[mod.rs:43-55](../../../cli/src/flow/mod.rs#L43)). No name, path or source text crosses the wire:
a finding comes back as `[unit, kind, seq, var, seqEnd]` and the measuring side labels it again
through the legend it kept ([Flow.hs:67-75](../../../core/app/CE/Flow.hs#L67),
[mod.rs:63](../../../cli/src/flow_report/mod.rs#L63)).

### 1. The lowering — four tables per unit

A unit is the scan's own unit (`scan::functions::extract`: the same kinds, names and order), and
its body is the grammar's body field ([lower.rs:73](../../../cli/src/flow/lower.rs#L73)). The four
tables are the whole of what the core sees (the design booklet's §5.1,
[analysis-track.md](../analysis-track.md)):

- `units` — one row per unit with its parameter count;
- `stmts=[[u, seq, parent, kind, flags, aux]]` — the statements in pre-order, sixteen kinds
  (block, stmt, if, loop, switch, case, try, catch, finally, return, throw, break, continue, goto,
  label, no-return call), five flag bits (has-else, infinite, fallthrough, dynamic, empty) and the
  jump target in `aux`;
- `vars=[[u, v, declSeq, flags]]` — locals and parameters, four flag bits (param, captured,
  ignored, address-taken);
- `uses=[[u, seq, v, mode]]` — the accesses in evaluation order: read, write or read-write.

Eleven lowering rules decide what the integers say, one table per language beside its `LangSpec`
(Python, TypeScript and TSX, Rust, Go, C, C++, Java, Lua, R). Three of them carry the family's
safety argument. The no-return calls are a closed name list matched literally at the top of an
expression statement, and a receiver-dependent name (Go's `t.Fatal`) stays out: a missed exit
hides a finding, an invented one would fabricate it. Control flow in expression position (Rust's
`?`, a `return` inside a `match` arm, Java's switch expression) is not lowered, because both
verdicts negate the existence of a path and a conditional exit only adds paths; what is lost is
the one unconditional exit made of arms that all leave, and its casualty is a missed finding, never
a false one. A unit the measuring side cannot read — an evaluator call (`eval`, `exec`, `load`,
R's `assign` and `get`), a preprocessor conditional inside a C-family body whose two arms would
be lowered as consecutive, a computed `goto`, `asm`, `setjmp` — carries the dynamic flag and is
skipped whole by the core and counted ([Flow.hs:62-63](../../../core/app/CE/Flow.hs#L62)). A unit
whose tree cannot be lowered to the contract's shape is not sent (`unlowered`); a unit the core
still refuses by name is mapped back from the refusal's row, dropped, and the batch asked again,
so one unit's defect never costs the others their verdict
([wire_batch.rs:80](../../../cli/src/flow/wire_batch.rs#L80), [wire_batch.rs:126](../../../cli/src/flow/wire_batch.rs#L126)).

### 2. The graph

`CE.Flow.Cfg` unfolds the structured statements into edges: an if to its then and else, a loop
back to its head (an infinite head has no exit edge), a switch to each case with fallthrough to
the next, a try body's every statement to each of its catches, jumps to their targets, return,
throw and no-return calls to the exit ([Cfg.hs:1-14](../../../core/app/CE/Flow/Cfg.hs#L1),
[Cfg.hs:40](../../../core/app/CE/Flow/Cfg.hs#L40)). A finally is one hub, not a copy per target:
every edge that leaves the try or a catch is routed through it, and the hub's successors are the
union of the targets the incoming edges carried. The union over-approximates, and it only adds
paths — the direction in which a verdict can be lost, never invented.

### 3. Reachability — `unreachable` (kind 0)

A breadth-first walk from the entry ([Reach.hs:13](../../../core/app/CE/Flow/Reach.hs#L13)). The
statements it never visits are reported as maximal runs of consecutive seqs; since the numbering
is pre-order, an unreachable statement's whole subtree is one run and an `if` after a `return`
is one finding, not one per line ([Reach.hs:23](../../../core/app/CE/Flow/Reach.hs#L23)).

### 4. Liveness — `dead_store` (kind 1), `unused_local` (kind 2), `unused_param` (kind 3)

Live-in is solved to a fixpoint of `in = transfer(∪ in of successors)` over the whole graph
([Live.hs:30](../../../core/app/CE/Flow/Live.hs#L30)). Within one statement the accesses are
ordered, so the transfer walks them last first — `x = x + 1` reads before it writes — and a
store's liveness is asked right after that store, inside the statement
([Live.hs:45](../../../core/app/CE/Flow/Live.hs#L45), [Live.hs:54](../../../core/app/CE/Flow/Live.hs#L54)).
A dead store is a write no path reads before the variable is written again or the unit exits.
Four rules keep the verdict to one fact, one finding: a captured or address-taken variable has
readers this graph cannot see and is not judged; a variable nothing reads gets its unused finding
and no dead-store findings; a store in unreachable code is covered by its run; the same
statement's several stores are judged each on its own and reported as one `(seq, v)` row. An
unused local or parameter is one nothing reads, a read-write counting as a read, the three
exemptions (captured, ignored, address-taken) leaving it out
([Live.hs:70-82](../../../core/app/CE/Flow/Live.hs#L70)). Kind 3 is advice by construction: a
parameter an interface, an overload or an override requires is unread for a reason the unit
cannot show, so no tier, mask or gate ever reads it as a verdict
([mod.rs:21-38](../../../cli/src/flow_report/mod.rs#L21)).

### 5. Caps and degradation

The four tables are one dimension against one cap, 524,288 rows; crossing it answers a complete
degraded reply — no findings, the counts, `flow_too_large` — since a request the core refused to
judge licenses nothing ([Cost.hs:53](../../../core/app/CE/Flow/Cost.hs#L53),
[Contract.hs:47](../../../core/app/CE/Flow/Contract.hs#L47), [Flow.hs:45](../../../core/app/CE/Flow.hs#L45)).
The measuring side plans its batches under the same cap, and a single unit heavier than it is
refused by name rather than split ([wire_batch.rs:39](../../../cli/src/flow/wire_batch.rs#L39)).
There is no knob and no condition bit on the wire: what a finding means for a gate is decided on
this side, by the class's tier and the precision mask.

### 6. The faces

`ce flow [--check] [--kind …] [--format json]` walks the tree with the scan's walk, lowers each
file, asks the core and prints `ce.flow-report/0.1.0`: the counts (units, statements, variables,
uses, findings, dynamic units, unjudged units, judged, shown), each finding with its path, unit,
kind, lines, variable and whether it is judged, and each unit the core refused with its reason
([face.rs:21](../../../cli/src/flow_report/face.rs#L21), [face.rs:65](../../../cli/src/flow_report/face.rs#L65),
[face.rs:124](../../../cli/src/flow_report/face.rs#L124)). A finding's `lineEnd` is the line on
which the run's last statement starts: the legend keeps each statement's first line only, so a
last statement written over several lines is not followed to its end; the next lowering
generation carries the end. `--kind` narrows the listing and
never the counts, so a filtered run cannot move the gate. The exit codes are the family's own: 2
when the core is missing or lacks the family (the report names the reason), 1 under `--check`
only when `[flow] tier` is `deny` and a judged finding exists, 0 otherwise
([main_flow.rs:27-53](../../../cli/src/main_flow.rs#L27)). The MCP tool `flow` and the GUI's
reports hub read the same document through `faces::flow`; the hub registers its own renderer for
the kind chips, the judged / advisory mark and the unjudged units.

A finding is **judged** when its language is in `flow::judged_mask()` and its kind is not 3. The
mask holds the languages whose precision doc passed the gate of §8. Step 4's commit C2 admitted
Python, TSX, Go, C, Java, Lua and R; commit E then fixed the lowering those docs answer by and
retired all ten; commit G regenerated the ten docs on the fixed lowering (a second exam
generation for C++, R, Rust and TypeScript) and every language reads judged, so the mask holds
all ten ([mod.rs:45-63](../../../cli/src/flow/mod.rs#L45), [mod.rs:33](../../../cli/src/flow_report/mod.rs#L33)).

### 7. The guard class — novel findings at write time

The PreToolUse leg lowers both sides of one write — the file on disk and the file the edit
would produce — judges each over the daemon's core link (daemon protocol 2.2.0, the additive
`flow` request carrying the four tables exactly as `flow::wire::body` assembles them), places the
findings through each side's legend and subtracts
([flow.rs:1-12](../../../cli/src/guard/flow.rs#L1), [flow.rs:35](../../../cli/src/guard/flow.rs#L35),
[proto.rs:30](../../../cli/src/daemon/proto.rs#L30)). The subtraction is the guard's usual
novelty, applied to findings: a multiset difference keyed on (unit name, kind, variable name),
the line left out so a finding the edit merely moves is not this write's doing
([flow_novel.rs:15-22](../../../cli/src/guard/flow_novel.rs#L15)). The class's condition is a
judged language and a novel finding of a non-advisory kind, and the feed's `novel` counts those
kinds only: an unused parameter the write brings shows in `kinds`, never in `novel`. The class
speaks at its own tier,
`[flow] tier`, shipped at observe, and never on a degraded side — a side the daemon could not
judge is named in the line, never a decision ([flow.rs:114](../../../cli/src/guard/flow.rs#L114),
[flow.rs:20-25](../../../cli/src/config/flow.rs#L20)). Each side is one question to the daemon, so
a side the core refuses, even for one unit, is degraded whole and named in the line: the
drop-and-ask-again of §1 is the batch road `ce flow` walks, and the hook keeps a write's cost to
one question per side. Promotion follows the plan's §4.2 rule: a
class with no FPR record of its own stays at observe. The observe feed (`ce.observe/0.12.0`,
additive) receives a `flow` line when the after side has a finding or a side is degraded: the
before and after counts, the novel count, the kinds and whether the language is judged.

The Stop audit, the git pre-commit and the commit-msg hooks carry a `flow` object on their line —
files with units, units, findings, per-kind counts and the judged count — over the audit's own
core link ([flow.rs:17](../../../cli/src/audit/flow.rs#L17)). That leg never blocks at any tier:
the changeset's findings are recorded, and the site list is `ce flow`'s.

### 8. Gates

The core holds its judgment to a reference that builds no graph at all: a tree-walking
interpreter that enumerates every execution trace of a structured unit — every branch and case,
each loop head at most twice, every statement of a try body free to hand control to a catch,
a finally continuing with the union of the pending completions — and reads the findings off the
traces; the shipped judgment must agree finding for finding on 200 seeded random programs
([ReferenceFlow.hs:1-12](../../../core/test/ReferenceFlow.hs#L1)). The battery adds the
hand-written cases of every kind, forty-two contract refusals pinned by name, the cap and the
skipped unit's counts ([FlowProps.hs:5-12](../../../core/test/FlowProps.hs#L5)), and six golden
pairs pin the wire bytes, their requests lowered from real source since step 4
([Spec.hs:89](../../../core/test/Spec.hs#L89)).

A language's findings become verdicts only through its precision exam, registered in
[EVAL-SET-FLOW.md](../../EVAL-SET-FLOW.md): the unit universe of one pinned corpus per language
(Rust also the repository itself) frozen with the candidate pools read off the lowered tables —
never off the core — a stratified sample drawn by rank, answered blind by independent reviewers
who never saw a verdict, and a precision doc generated on a clean tree after the review. The gate
is per language and per non-advisory kind, in four states: `fail` (a false positive), `pass` (none,
and at least one true positive), `vacuous` (no positive in the sample to find, so admission rests
on zero false positives over the negatives) and `silent` (positives present and none reported,
not admitted); a language is judged when kinds 0, 1 and 2 each pass or are vacuous, kind 3 is
recorded only, and recall is recorded and not gated. The frozen exams hold 1,549 files and
27,118 units across ten languages, 792 of them dynamic, and 1,142 questions in the current
generations (1,197 in the first); the first ten precision docs admitted Python, TSX, Go, C, Java,
Lua and R, and every false positive the other three carried was attributed in the registry to the
lowering or to a disputed truth, none to the core — commit E fixed those lowering classes and
retired the ten docs, and commit G's ten docs on the fixed lowering (four of them second-generation
exams, judged again under the batch prompt's language readings) admitted all ten languages with no
false positive in kinds 0–2; the one advisory false positive left, a C++ pointer-to-member call
`(w.*cb)(args...)` whose `cb` the lowering never reads, is registered for the next lowering. A replay
ledger over each exam corpus's last 400 first-parent commits reads, per language, how often a
finding disappears with its unit's next edit (a true positive) or survives it (a false stop,
strict reading), both readings recorded.

### 9. Known boundaries

- **Dynamic units are skipped whole and counted.** A unit whose behaviour the lowering cannot
  read is never judged in part; the count is in every report and every feed line.
- **`unused_param` is advice forever.** No tier and no mask entry turns it into a verdict.
- **Haskell is not a flow language.** No `FlowSpec` table exists for it (nor for Markdown or
  HTML), so its files lower to no units ([spec.rs:250-253](../../../cli/src/flow/spec.rs#L250)).
- **The judgment is intra-procedural.** A store read only through a callee's side effect the
  graph cannot see is covered by the address-taken and captured exemptions, not by a call model.

### 10. Tests

The unit legs hold the placement of every kind back to its lines and variable, the novelty
subtraction (a moved finding, a rename, a multiset, kind-blind) and the tier's load-time refusal
([flow_novel.rs:17](../../../cli/tests/unit/guard/flow_novel.rs#L17)). The integration legs hold
the CLI to the library face with `--kind` narrowing only the listing, `--check` to the class's
own `deny`, the MCP relay to the same document, the guard's novel count, its sentence at `warn`
and its refusal at `deny` on a judged language's write (Python) and its silence on an unjudged
one (Rust), a moved finding and an unused parameter as not novel, the Stop line's `flow`
object, and the daemon round trip
([flow_face.rs:75](../../../cli/tests/it/flow_face.rs#L75), [flow_guard.rs:64](../../../cli/tests/it/flow_guard.rs#L64),
[flow_audit.rs:46](../../../cli/tests/it/flow_audit.rs#L46), [daemon_flow.rs:18](../../../cli/tests/it/daemon_flow.rs#L18)).
The family is one row of the parity table, one hub renderer of the GUI and one tool of the MCP
catalogue; this booklet is under the citations gate.
