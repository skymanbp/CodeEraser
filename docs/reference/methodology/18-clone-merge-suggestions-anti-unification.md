# Clone merge suggestions — anti-unification over the clone families' trees

[index](../methodology.md) · [← 17 Intra-function dead code — the flow graph, reachability and liveness](17-intra-function-dead-code-flow-graph-reachability-and-liveness.md)

The clone families say *that* code is repeated: `ce dedup` groups the token runs that recur, and
`ce clone` pairs the units whose trees sit within a small edit distance. This family says *how*
the repetition would fold into one function. For every group it lines the members' syntax trees
up, finds the places where they differ — the holes — and turns the holes into parameters: the
count a merged function would take, the member worth keeping (the one the most files already
reach), the lines the merge would save, and whether the merge is feasible at all, with the reason
when it is not. The operation is anti-unification: the least general skeleton every member is an
instance of, each member recovered by filling the skeleton's holes with its own values. It is
advice — no condition bit, no gate reads it, and `ce merge` exits 0 with a document
([Merge.hs:5-19](../../../core/app/CE/Merge.hs#L5), [face.rs:1-16](../../../cli/src/merge/face.rs#L1)).

The split is ADR-008's, seventh instalment. Rust gathers the groups, builds each member's tree in
`clone/1`'s postorder encoding with two more integer columns, and labels the answers back with
the members' source text; the alignment, the holes, the parameters, the feasibility, the member
kept and the savings are all Haskell's, over the fifteenth wire family, `merge/1`, since proto
7.5.0 ([wire.rs:1-11](../../../cli/src/merge/wire.rs#L1), [Merge.hs:33](../../../core/app/CE/Merge.hs#L33)).
No name, path or source text crosses the wire: a leaf goes up as the fnv1a64 hash of its bytes,
and every text on the report is read back on this side from the member's own file.

### 1. The groups

Two families feed the family, each unchanged from its own booklet:

- **T1/T2** — every clone family `ce dedup` verified: a connected component of verified token
  blocks over identical endpoint spans ([01](01-t1-t2-clone-detection-winnowing-fingerprint.md)).
  A member is a line span. It is a *whole unit* when one admitted unit fills it — the longest unit
  lying inside the member's lines, kept when it covers at least nine tenths of them, because a
  member's line span is coarser than its tokens and may carry a comment line or a brace more
  ([groups.rs:159](../../../cli/src/merge/groups.rs#L159)). A family whose every member is a
  whole unit sends those units' trees; any other sends each member as a *fragment*: the maximal
  named nodes inside the member's lines — its top nodes — under one synthetic root, and the report
  marks the group `fragment` ([tree.rs:96](../../../cli/src/dedup/t3/tree.rs#L96)).
- **T3** — every pair `ce clone` judged a near-miss clone: one group of exactly two members, each
  a whole unit ([02](02-t3-near-miss-clones-tree-edit-distance-tsed.md)).

**A fragment is trimmed to the members' common shape.** The token run behind a T1/T2 family starts
and ends mid-line, so a member cut by lines can carry, on its boundary line, half a statement its
twin's does not. Each top node gets a shape key — the fnv1a of its postorder, an internal node's
label or a mark for a leaf (leaves open, as in the core's isomorphism), each `lld` relative to the
top's first node — and the family keeps the longest run of member 0's keys that every member holds
as a contiguous run: the earliest in member 0 among equals, each other member's first occurrence.
Every member sends only that run under its synthetic root, whose span runs from the run's first
top to its last ([groups_trim.rs:1-11](../../../cli/src/merge/groups_trim.rs#L1),
[groups_trim.rs:35](../../../cli/src/merge/groups_trim.rs#L35)). No run in common, and the family
is not one shape. A member keeps its clone-family line span as its identity — the span `ce dedup`
names — and carries beside it the kept run's own lines, the first kept top's first line to the last
one's last, as its `run` ([tree.rs:58](../../../cli/src/dedup/t3/tree.rs#L58),
[groups.rs:271](../../../cli/src/merge/groups.rs#L271)). The request's `lines` column is the run's,
so the core prices the lines a merge would fold, never a boundary statement the trim cut off
([wire.rs:112](../../../cli/src/merge/wire.rs#L112)); a whole unit's run is its span. The report
prints both spans, and the texts on it come off the trimmed trees.

A group the core would refuse, or that no request can carry, is never sent; it is counted by why
under `unsendable`, in this order: a member whose language has no position-class table
(`no_slot_table`), a member whose tree was not built — over `clone/1`'s per-tree cap, a forest
where one tree was needed, nothing selected (`unbuilt`) —, a fragment family with no run of tops
in common (`not_isomorphic`), a group whose nodes alone pass the request cap (`over_cap`), and
last, for T1/T2 alone, members whose trees are not one shape (`not_isomorphic`)
([groups.rs:216](../../../cli/src/merge/groups.rs#L216)). The last check is the core's own
definition asked early, so the core's refusal never lands on the live road
([groups_trim.rs:95](../../../cli/src/merge/groups_trim.rs#L95), [Align.hs:20](../../../core/app/CE/Merge/Align.hs#L20)).

**One member set, one suggestion.** The two families can name the same members: a T1/T2 family
whose every member one unit fills, and `ce clone` judging those two units a near-miss pair. A group
is keyed by its members' identities, sorted — each unit, or a fragment's clone-family span — and a
T1/T2 group covers a T3 group over the same set, the stand `ce check`'s similarity table takes (one
row per pair, the stronger kind kept): an exact isomorphism is the stronger claim than a near miss.
The drop comes after the local pre-checks and before any request, so a T3 pair whose T1/T2 twin was
not sent stays; the groups dropped are counted apart as `merged_duplicates`, never under
`unsendable` — they are the same suggestion twice, not a group no request can carry
([groups.rs:78](../../../cli/src/merge/groups.rs#L78)).

### 2. The trees — `clone/1`'s encoding and two columns

A member's tree is the one `clone/1` already sends: its named nodes in postorder, `lab` the
node's kind hash (made request-local dense), `lld` the postorder index of its leftmost leaf. The
same walk now fills three more columns at the same node — `leaf`, `slot` and the node's byte
span — so the two families can never read two different trees of one unit
([tree.rs:157](../../../cli/src/dedup/t3/tree.rs#L157), [tree.rs:213](../../../cli/src/dedup/t3/tree.rs#L213)).
`clone/1`'s request still sends `lab` and `lld` alone, so its judgment bytes do not move
([wire.rs:28](../../../cli/src/dedup/t3/wire.rs#L28)).

**The merge family's trees leave the grammar's extras — its comments — out.** The token runs
behind a T1/T2 family hold no comment, so a comment on one member only must not make the members
two shapes: without extras an extra is never a top node, and an extra child's subtree is never
emitted. `clone/1`'s judgment keeps them: its trees are the `unit_seq` selection node for node, and
the node-count check against the cached unit signature holds exactly there; a merge tree holds that
count less its comments, never more ([tree.rs:31-53](../../../cli/src/dedup/t3/tree.rs#L31),
[tree.rs:255](../../../cli/src/dedup/t3/tree.rs#L255)).

- **`leaf`** — for a leaf (a node whose `lld` is its own index) the fnv1a64 of the leaf's source
  bytes, the same hash the token layer uses; 0 on every internal node.
- **`slot`** — the node's position class: 0 a statement, 1 an expression, 2 a type, 3 a declared
  name, 4 anything else ([Cost.hs:55-61](../../../core/app/CE/Merge/Cost.hs#L55)).
- the byte span — kept on this side only, to read a hole's text back.

The position class is a syntactic fact read off one table per language, read beside the flow
family's tables because the statement class is theirs and is never restated
([slot.rs:1-20](../../../cli/src/merge/slot.rs#L1)). A node is in a statement position when its
kind is one of the flow table's statement forms — the conditionals, the loops, the switches and
their cases, the tries and their catches and finally clauses, the `with` forms, the declarations,
`return`, `throw`, `break`, `continue`, `goto`, the labels, the statement wrappers, the `else` and
`elif` arms — or when its parent is one of the flow table's block or splice kinds
([slot.rs:125](../../../cli/src/merge/slot.rs#L125)). What the flow tables do not say, the slot
table says: its expression kinds, its type kinds, the (declaring kind, field) pairs where a
declared name sits, and — named, so a decision is told apart from an omission — the kinds that
are "other" ([slot.rs:38](../../../cli/src/merge/slot.rs#L38)). The rule reads the sets in a fixed
order: statement, then type, then name, then expression, else 4 — an unknown position holds no
parameter, the safe side ([slot.rs:201](../../../cli/src/merge/slot.rs#L201)). A table is
read in pieces: TSX reads TypeScript's shared piece and its JSX kinds while TypeScript adds the one
kind only its grammar has, and C and C++ share a piece the same way — so a shared piece never names
a kind the other grammar lacks ([slot.rs:58](../../../cli/src/merge/slot.rs#L58)).

Haskell has no flow table, and it is this repository's core language, so its slot table carries
the two lists a flow table would give: `stmt_kinds`, its statement forms (the do block's `bind`,
`exp`, `let` and `rec`, and the declarations — signatures, data and newtype declarations, classes,
instances, imports and the rest), and `container_kinds`, the kinds whose children stand in a
statement position (`do`, `declarations`, `local_binds`, the class and instance bodies). A
language with a flow table never fills either list — the unit leg holds both ways
([slot_hs.rs:1-14](../../../cli/src/merge/slot_hs.rs#L1), [slot.rs:125](../../../cli/src/merge/slot.rs#L125)).
The Haskell grammar spells an expression, a pattern and a type with the same kinds (`apply`,
`variable`, `tuple`…), so its type class holds only the kinds that are types alone — a type
constructor's `name`, the arrow, `forall`, `context` and the like — and a shared kind reads as an
expression. Markdown and HTML have no function to fold into and no table: their groups are counted
`no_slot_table` and never sent.

A fragment's synthetic root carries kind `ce:fragment`, `leaf` 0 and `slot` 4, its `lld` the first
subtree's — the kept run hangs under it as siblings, so a fragment is one tree like any other
([tree.rs:127](../../../cli/src/dedup/t3/tree.rs#L127)).

### 3. The judgment

**Family 0 — a T1/T2 group** ([Align.hs:1-9](../../../core/app/CE/Merge/Align.hs#L1)). The
members are one token run, so their trees must be isomorphic: every member's `lld` column equal to
member 0's and every internal node's label and leaf hash equal. The leaves alone may differ — T2
normalises identifiers and literals to one token — and the holes are exactly the leaf positions
whose keys are not every member's, in postorder. The skeleton is member 0's tree with those
leaves left open ([Align.hs:30](../../../core/app/CE/Merge/Align.hs#L30)). A request whose T1/T2
group is not isomorphic is refused by name — `group <g>: members are not isomorphic` — since the
measuring side promised one token run ([Contract.hs:60](../../../core/app/CE/Merge/Contract.hs#L60)).

**Family 1 — a T3 pair** ([Mapped.hs:1-12](../../../core/app/CE/Merge/Mapped.hs#L1)). The two
trees are aligned by an optimal tree-edit mapping. `CE.Clone.Ted` gives it back beside the
distance: `tedMapping` reads the same cell recurrence as `ted` and only re-walks it, so its
distance is `ted`'s by construction ([Ted.hs:14-18](../../../core/app/CE/Clone/Ted.hs#L14),
[Ted.hs:56](../../../core/app/CE/Clone/Ted.hs#L56)). The Tai mapping is then narrowed top-down: a
mapped pair is kept only when its parents are a kept pair, starting from the two roots; roots not
mapped to each other leave no skeleton and the whole pair is one hole
([Mapped.hs:28](../../../core/app/CE/Merge/Mapped.hs#L28)). A kept pair whose keys differ is a leaf
hole — the mapping's relabel, its value the node alone ([Mapped.hs:38](../../../core/app/CE/Merge/Mapped.hs#L38)).
Between two neighbouring kept child pairs, and before the first and after the last, the unkept
subtrees on each side form a gap: two gaps node for node the same join the skeleton, any other is a
gap hole whose value on each side is that side's forest, an empty side an empty value
([Mapped.hs:52](../../../core/app/CE/Merge/Mapped.hs#L52)).

**Holes to parameters.** The holes are ordered by their key — member 0's anchors in postorder,
then the gaps empty on member 0's side. Each hole has a value vector, one value per member; holes
with the same vector are one parameter, numbered by first appearance, so the parameter count is
the number of distinct vectors ([Holes.hs:21-35](../../../core/app/CE/Merge/Holes.hs#L21)).

**Feasibility and its reason.** A merge is feasible when every hole stands where a parameter can:
an expression position or a declared name. Otherwise the first infeasible hole in hole order
names the reason — 3 a gap hole whose forests hold a statement (it outranks the anchor's class),
1 a statement or other position, 2 a type position — a group whose every hole is feasible but
whose parameters number more than six answers 4, and one whose holes and parameters pass but whose
savings (below) are not positive answers 5, `no_savings`: a merge that saves no line is not a
suggestion anyone would take, the stance of 4 ([Holes.hs:46-51](../../../core/app/CE/Merge/Holes.hs#L46),
[Holes.hs:62](../../../core/app/CE/Merge/Holes.hs#L62),
[Cost.hs:38-41](../../../core/app/CE/Merge/Cost.hs#L38), [Cost.hs:67-80](../../../core/app/CE/Merge/Cost.hs#L67)).

**The member kept and the savings.** The member kept is the one whose file has the greatest
in-degree in the reference graph — the file most others already reach — a tie to the least member
index ([Holes.hs:69-72](../../../core/app/CE/Merge/Holes.hs#L69)). The savings are the members'
lines less the merged function's lines and one call line per member left behind
([Holes.hs:53](../../../core/app/CE/Merge/Holes.hs#L53), [Cost.hs:43-46](../../../core/app/CE/Merge/Cost.hs#L43)).
A T1/T2 skeleton is the kept member itself; a T3 skeleton is the kept member's lines in the share
of its nodes the kept pairs cover, rounded up — the wire carries no per-node line, so this is an
honest integer estimate, never a measured count ([Holes.hs:77-88](../../../core/app/CE/Merge/Holes.hs#L77)).

**The wire.** The request is three tables — `groups=[[g,family]]`, `members=[[g,m,unit,lines,fileIndeg]]`,
`trees=[{lab,lld,leaf,slot}]` in member order — and the reply one suggestion row per group
`[g,params,kept,savings,feasible,reason]`, its hole rows `[g,hole,param,m,post,postEnd]` and the
counts of groups, members, nodes, suggestions, holes and feasible suggestions
([Merge.hs:36-60](../../../core/app/CE/Merge.hs#L36)). `post` and `postEnd` are the member's first
and last root of the hole: one node twice for a leaf or relabel hole, the first and last root of a
gap's forest — siblings under one parent — and −1 −1 where that side is empty
([Tree.hs:66-77](../../../core/app/CE/Merge/Tree.hs#L66), [Holes.hs:37-58](../../../core/app/CE/Merge/Holes.hs#L37)).
The contract refuses, by name and in request order, a malformed group or member row, a member
whose group is unknown, a group of the wrong size for its family, a tree count unequal to the
member count, a tree failing `clone/1`'s own shape contract, a slot column missing, of the wrong
length or out of range, and a non-isomorphic T1/T2 group ([Contract.hs:51](../../../core/app/CE/Merge/Contract.hs#L51),
[Contract.hs:115](../../../core/app/CE/Merge/Contract.hs#L115)). More than 4,096 groups or
1,048,576 nodes (`treeNodeCap`) answers a complete degraded reply with no rows
([Cost.hs:28-36](../../../core/app/CE/Merge/Cost.hs#L28)); the measuring side chunks by both caps
and never splits a group across two requests ([wire.rs:70](../../../cli/src/merge/wire.rs#L70)).
So a reply the core degraded to a request this side priced within both caps is a drift between the
two sides' cap mirrors — an error naming both owners, never a document; a core without the family,
or one that cannot be started or cannot answer, gives a document with `degraded`
([wire.rs:128](../../../cli/src/merge/wire.rs#L128), [face.rs:120](../../../cli/src/merge/face.rs#L120)).

### 4. The measuring side's rulings

The design booklet fixes the core's side; eight rulings of step 7 fix the measuring side
(recorded in the design booklet's decision log on landing):

1. The position-class tables live in the merge module in their own files and structure; the
   statement class is read off the flow tables' existing fields, never a second statement table —
   a language without a flow table names its own statement forms and containers in its slot table.
2. Each table is held by three legs: every kind and field exists in its grammar; every named node
   of the language's probe sample that answers 4 has its kind in `other_kinds` — the table's
   completeness evidence, naming whatever it missed; the kind sets are pairwise disjoint and none
   of them a statement form, and a table's own statement and container lists are filled exactly
   when its language has no flow table ([slot.rs:39](../../../cli/tests/unit/merge/slot.rs#L39),
   [slot.rs:70](../../../cli/tests/unit/merge/slot.rs#L70), [slot.rs:96](../../../cli/tests/unit/merge/slot.rs#L96)).
3. `leaf` and `slot` come out of the one walk that emits `lab` and `lld`.
4. The groups and the local pre-checks of §1; a member row's `unit` is the request-local member
   number (the core echoes it, never reads it), its `lines` the unit's line count or the
   fragment's kept run's (§1), its `fileIndeg` the reference graph's edges landing on the
   member's file node — the same graph `ce deadcode` judges, from the same index ([face.rs:163](../../../cli/src/merge/face.rs#L163)).
5. One report, `ce.merge-report/0.1.0`, for all three faces; the core's counts summed over the
   chunks, `merged_duplicates` (§1) beside them; each parameter labelled with every member's text at the parameter's first hole
   ([face.rs:182](../../../cli/src/merge/face.rs#L182)). The text runs from the hole's first root
   to its last on that member — a leaf or relabel hole's node, a gap hole's whole forest — and an
   empty side reads `""` ([face.rs:241](../../../cli/src/merge/face.rs#L241)).
6. The T3 trees are built once for both families; `clone/1` never sends the new columns.
7. The console prints the counts and the groups not sent, then per group a head line, its
   members — a trimmed member's run beside its span — and one line per parameter with every
   member's text cut at 40 characters; `--group n`
   prints one group and leaves the JSON face whole ([console.rs:16](../../../cli/src/merge/console.rs#L16)).
8. Exit codes: a judged document 0; a degraded one 2; an argument error — `--group` past the last
   group included — 2 ([main_merge.rs:25](../../../cli/src/main_merge.rs#L25)).

### 5. The faces

`ce merge [--group <n>] [--format json]`, the MCP tool `merge_suggestions` and the GUI's merge
family in the Reports hub all read the one document ([face.rs:29](../../../cli/src/merge/face.rs#L29)).
The document holds the counts (the core's, and `merged_duplicates`), `unsendable`, and per group its family (`t1t2` / `t3`), whether it
is a fragment, its members (`path`, `unit` as `path:key#nth` or null for a fragment, `lines` the
clone-family span, `run` the lines sent and priced — `lines` again for a whole unit), the
parameter count, the member kept, the savings, `feasible`, the reason by name (`ok`, `position`,
`type`, `spans_statements`, `too_many_params`, `no_savings` — `position` for a hole at a statement
or any other position no parameter can stand for) and the parameters, each with every member's `text`
([face.rs:40](../../../cli/src/merge/face.rs#L40)). A core without the family, or one that cannot
be started or cannot answer, gives a document with `degraded` naming why and no group — a request
the core did not judge licenses nothing; a core that answers degraded to a request this side priced
within both caps is a cap-mirror drift and an error, never a document. The GUI card leads with the
counts, then per group its members (the member kept starred, a trimmed member's run beside its
span) and a parameter table, one row per
parameter and one column per member.

### 6. Gates

The core's battery holds the anti-unification laws over its hand-written cases and two hundred
generated T1/T2 groups: instantiating the skeleton with each member's values rebuilds that member
node for node, and no parameter can be dropped — set to member 0's constant, some member no longer
rebuilds ([MergeProps.hs:46-59](../../../core/test/MergeProps.hs#L46), [Holes.hs:90-105](../../../core/app/CE/Merge/Holes.hs#L90)).
`tedMapping` is held to `ted` over the exhaustive small-tree family and two hundred generated pairs:
the same distance, a valid Tai mapping, a cost equal to the distance. Every hole's first and last
root per member is held over those groups and the two hundred pairs as T3 groups: −1 −1 on an empty
side, else `post ≤ postEnd`, one node or two children of one parent ([MergeProps.hs:163](../../../core/test/MergeProps.hs#L163)).
The generated groups' parameter counts and feasibility, the reasons' coherence over every judged
and generated suggestion — feasible exactly at 0, 0 only with a line saved, 5 only without one
([MergeProps.hs:146](../../../core/test/MergeProps.hs#L146)) —, the equal-gap fold, both caps, the
empty request and the counts each have their leg, and six request–reply pairs are golden
([golden.ndjson](../../../contracts/fixtures/merge/golden.ndjson)).

On the measuring side the unit legs hold the slot tables (§4 ruling 2), the nine-tenths rule, the
fragment's synthetic root, the trim — a boundary statement on one side trimmed off and the family
sent, a statement inside one side's only top still not one shape, a member whose boundary statement
was trimmed priced by its run ([groups.rs:156](../../../cli/tests/unit/merge/groups.rs#L156)) — the
isomorphism pre-check
leaving leaves open, every unsendable group counted by why, one member set giving one suggestion
with T1/T2 over T3 ([groups.rs:224](../../../cli/tests/unit/merge/groups.rs#L224)), a chunk never
splitting a group, the
request body's tables and `consume`'s reading of a healthy, a degraded (a cap-mirror drift) and a
skewed reply ([groups.rs:29](../../../cli/tests/unit/merge/groups.rs#L29),
[groups.rs:142](../../../cli/tests/unit/merge/groups.rs#L142), [groups.rs:182](../../../cli/tests/unit/merge/groups.rs#L182),
[wire.rs:44](../../../cli/tests/unit/merge/wire.rs#L44), [wire.rs:74](../../../cli/tests/unit/merge/wire.rs#L74));
the tree legs hold a leaf's hash to its own text and every column to the node count, and a tree
without extras to the tree with them less exactly its comments, every other column in step
([tree.rs:47](../../../cli/tests/unit/dedup/t3/tree.rs#L47), [tree.rs:76](../../../cli/tests/unit/dedup/t3/tree.rs#L76)).
The integration legs seed three renamed copies of one function, a near-miss pair with one extra
statement and a statement run two functions share, and hold the CLI's JSON to the library's byte
for byte, the whole-unit group feasible with one parameter per rename and the renamed names as its
texts, the T3 pair infeasible across statements with the extra statement as its gap's text, the
fragment group marked, `--group` and its refusal, `ce clone`'s report on the same fixture unmoved,
the MCP relay, and a core that cannot be reached as a degraded document
([merge_face.rs:118](../../../cli/tests/it/merge_face.rs#L118),
[merge_face.rs:183](../../../cli/tests/it/merge_face.rs#L183), [merge_face.rs:214](../../../cli/tests/it/merge_face.rs#L214)).

**The frozen suggestion set.** `contracts/eval/merge-suggestions-v1.json` freezes, for this
repository at one pinned commit (exported with `git archive`, `.gitmodules` removed) and for the four
pinned external corpora, the counts (`merged_duplicates` among them, §1), the groups not sent and
one row per suggestion — its members
(`at` the identity, `run` the lines priced), family, fragment, parameters, member kept, savings,
feasibility, reason and hole count; a row's sample id hashes the identities alone, and no two rows
of a corpus share their identities — one member set, one suggestion — so the id is unique by
construction ([eval_merge_suggestions.rs:47](../../../cli/tests/it/eval_merge_suggestions.rs#L47)).
It is kept whole, every corpus row by row (9.4 MB), because two gates read the external
corpora's rows: `the_sample_is_the_draw_of_the_frozen_set` redraws the sample from every row's
identity and feasibility, and `readings` reads each drawn row's feasibility, reason and parameter
count. The CI leg measures the pinned self tree again and holds every row; re-freezing re-measures
all five ([eval_merge_suggestions.rs:21](../../../cli/tests/it/eval_merge_suggestions.rs#L21),
[eval_merge_review.rs:41](../../../cli/tests/it/eval_merge_review.rs#L41), [eval_merge_review.rs:65](../../../cli/tests/it/eval_merge_review.rs#L65)).

**The audit.** A hundred suggestions are drawn from the frozen set by the shared identity hash — the
feasible and the infeasible half each, every half apportioned over the corpora by their share of
it. The audit is blind to the verdict — feasibility, reason, savings and the member kept are
withheld — and sees the core's parameterisation, which it may reject: each sampled row carries the
members' identities, runs and source and, per parameter the core found, every member's text.
Independent agents who read the sample alone give each row a feasibility, a reason (one of the six
names) and a parameter
count; the readings, overall and per corpus, are how often the core's feasibility and reason agree
with theirs, and `params_agree` — how often the count given after seeing the core's
parameterisation equals the core's ([eval_merge_review.rs:110](../../../cli/tests/it/eval_merge_review.rs#L110)).

**The precision.** Four judges, one batch of twenty-five each, answered the hundred rows; their answer files are
filed verbatim, in the sample's order, as the review doc, every line held to the same row check the gate re-runs
([answers.rs:118](../../../cli/tests/it/eval_merge_review/answers.rs#L118)). The precision doc reads no product: it is
the three frozen docs' readings by definition, so it carries no tree and lands in the review's commit
([eval_merge_review.rs:195](../../../cli/tests/it/eval_merge_review.rs#L195)). The core's feasibility agrees with the
audit on 83 of the 100 rows, its reason on 77 and its parameter count on 45 (zod 60 / 56 / 26 of 68, ripgrep 17 / 15 /
11 of 22, cobra 4 / 4 / 7 of 8, requests 1 / 1 / 1 of 1, this repository 1 / 1 / 0 of 1). Of the 23 reason
disagreements, 11 are rows the core answers `position` and the judges find feasible, and 4 more are `position` rows
saving no line that the judges answer `no_savings`; the core's parameter count exceeds the judges' on 50 rows and falls
short on 5. The readings gate nothing — the family is advice — and the registry
[EVAL-SET-MERGE.md](../../EVAL-SET-MERGE.md) lists every disagreement with the judge's note, for the next generation's
parameterisation and reason order.

### 7. Design boundaries

- A T3 skeleton's line count is an integer estimate from the kept pairs' share of nodes, not a
  measured count; its savings inherit that.
- Markdown and HTML groups have no function to fold into; they are counted `no_slot_table`.
- A fragment has no floor on how many top nodes it keeps: one top can be a whole `match` or `if`,
  so a run is sized by its lines and nodes, and a run too short to save a line answers `no_savings`.
- A parameter is a place where the members differ; whether the differing values share a type is not
  asked, so a feasible suggestion is a shape that folds, not a guarantee that the folded function
  type-checks.
