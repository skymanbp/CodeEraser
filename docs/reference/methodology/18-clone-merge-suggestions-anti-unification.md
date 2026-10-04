# Clone merge suggestions — anti-unification over the clone families' trees

[index](../methodology.md) · [← 17 Intra-function dead code — the flow graph, reachability and liveness](17-intra-function-dead-code-flow-graph-reachability-and-liveness.md) · [→ 19 Architecture analysis — layers, cuts, clusters, impact](19-architecture-analysis-layers-cuts-clusters-impact.md)

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
`clone/1`'s postorder encoding with four more integer columns, and labels the answers back with
the members' source text; the alignment, the holes, the parameters, the feasibility, the member
kept and the savings are all Haskell's, over the fifteenth wire family, `merge/1`, since proto
7.5.0 ([wire.rs:1-12](../../../cli/src/merge/wire.rs#L1), [Merge.hs:34](../../../core/app/CE/Merge.hs#L34)).
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
  marks the group `fragment` ([tree.rs:103](../../../cli/src/dedup/t3/tree.rs#L103)).
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
one's last, as its `run` ([tree.rs:64](../../../cli/src/dedup/t3/tree.rs#L64),
[groups.rs:271](../../../cli/src/merge/groups.rs#L271)). The request's `lines` column is the run's,
so the core prices the lines a merge would fold, never a boundary statement the trim cut off
([wire.rs:122](../../../cli/src/merge/wire.rs#L122)); a whole unit's run is its span. The report
prints both spans, and the texts on it come off the trimmed trees.

A group the core would refuse, or that no request can carry, is never sent; it is counted by why
under `unsendable`, in this order: a member whose language has no position-class table
(`no_slot_table`), a member whose tree was not built — over `clone/1`'s per-tree cap, a forest
where one tree was needed, nothing selected (`unbuilt`) —, a fragment family with no run of tops
in common (`not_isomorphic`), a group whose nodes alone pass the request cap (`over_cap`), and
last, for T1/T2 alone, members whose trees are not one shape (`not_isomorphic`)
([groups.rs:216](../../../cli/src/merge/groups.rs#L216)). The last check is the core's own
definition asked early, so the core's refusal never lands on the live road
([groups_trim.rs:95](../../../cli/src/merge/groups_trim.rs#L95), [Align.hs:26](../../../core/app/CE/Merge/Align.hs#L26)).

**One member set, one suggestion.** The two families can name the same members: a T1/T2 family
whose every member one unit fills, and `ce clone` judging those two units a near-miss pair. A group
is keyed by its members' identities, sorted — each unit, or a fragment's clone-family span — and a
T1/T2 group covers a T3 group over the same set, the stand `ce check`'s similarity table takes (one
row per pair, the stronger kind kept): an exact isomorphism is the stronger claim than a near miss.
The drop comes after the local pre-checks and before any request, so a T3 pair whose T1/T2 twin was
not sent stays; the groups dropped are counted apart as `merged_duplicates`, never under
`unsendable` — they are the same suggestion twice, not a group no request can carry
([groups.rs:78](../../../cli/src/merge/groups.rs#L78)).

### 2. The trees — `clone/1`'s encoding and four columns

A member's tree is the one `clone/1` already sends: its named nodes in postorder, `lab` the
node's kind hash (made request-local dense), `lld` the postorder index of its leftmost leaf. The
same walk now fills five more columns at the same node — `leaf`, `slot`, `own`, `text` and the
node's byte span — so the two families can never read two different trees of one unit
([tree.rs:172](../../../cli/src/dedup/t3/tree.rs#L172), [tree.rs:232](../../../cli/src/dedup/t3/tree.rs#L232)).
`clone/1`'s request still sends `lab` and `lld` alone, so its judgment bytes do not move
([wire.rs:22](../../../cli/src/dedup/t3/wire.rs#L22)).

**The merge family's trees leave the grammar's extras — its comments — out.** The token runs
behind a T1/T2 family hold no comment, so a comment on one member only must not make the members
two shapes: without extras an extra is never a top node, and an extra child's subtree is never
emitted. `clone/1`'s judgment keeps them: its trees are the `unit_seq` selection node for node, and
the node-count check against the cached unit signature holds exactly there; a merge tree holds that
count less its comments, never more ([tree.rs:36-58](../../../cli/src/dedup/t3/tree.rs#L36),
[tree.rs:279](../../../cli/src/dedup/t3/tree.rs#L279)).

- **`leaf`** — for a leaf (a node whose `lld` is its own index) the fnv1a64 of the leaf's source
  bytes, the same hash the token layer uses; 0 on every internal node.
- **`slot`** — the node's position class: 0 a statement, 1 an expression, 2 a type, 3 a declared
  name or a local assignment's bare target, 4 anything else ([Cost.hs:65-71](../../../core/app/CE/Merge/Cost.hs#L65)).
- **`own`** — the fnv1a64 of the node's own anonymous tokens — its operator, keyword and
  punctuation: the unnamed children the tree's child walk steps over, looked through to their
  tokens and never into a named child — joined in source order with one 0x00 between two tokens;
  0 when it has none. `a < b` and `a > b` are one shape and differ here
  ([tree.rs:259](../../../cli/src/dedup/t3/tree.rs#L259), [tree_text.rs:72](../../../cli/src/dedup/t3/tree_text.rs#L72)).
- **`text`** — the fnv1a64 of the subtree's whole token stream, named leaves and anonymous tokens
  alike, joined the same way. Whitespace is no token, so `()` and `( )` share their text while
  their `leaf` hashes differ ([tree_text.rs:64](../../../cli/src/dedup/t3/tree_text.rs#L64)).
- the byte span — kept on this side only, to read a hole's text back.

The position class is a syntactic fact read off one table per language, read beside the flow
family's tables because the statement class is theirs and is never restated
([slot.rs:1-20](../../../cli/src/merge/slot.rs#L1)). A node is in a statement position when its
kind is one of the flow table's statement forms — the conditionals, the loops, the switches and
their cases, the tries and their catches and finally clauses, the `with` forms, the declarations,
`return`, `throw`, `break`, `continue`, `goto`, the labels, the statement wrappers, the `else` and
`elif` arms — or when its parent is one of the flow table's block or splice kinds
([slot.rs:166](../../../cli/src/merge/slot.rs#L166)). What the flow tables do not say, the slot
table says: its expression kinds, its type kinds, the (declaring kind, field) pairs where a
declared name sits, and — named, so a decision is told apart from an omission — the kinds that
are "other" ([slot.rs:45](../../../cli/src/merge/slot.rs#L45)). Three more lists place what an
expression is made of: the (kind, field) pairs where an assignment's target sits — a bare
identifier there is a name, 3, a local rename, any other target 4, since a parameter cannot take a
write — the (kind, field) pairs where a member, method or field name sits inside an expression, and
the kinds of a string literal's content; those last two are 4, left for the core to widen to the
enclosing expression (§3). The rule reads the sets in a fixed order: statement, then declared name,
then type, then assignment target, then part name or literal content, then expression, else 4 —
the declared name before the type, so a type's own name is a name; an unknown position holds no
parameter, the safe side ([slot.rs:209](../../../cli/src/merge/slot.rs#L209)). Every table also
names its language's helper lines — the head and closing lines a merged function wraps around a
fragment: 1 in Python and Haskell, 2 elsewhere ([slot.rs:66](../../../cli/src/merge/slot.rs#L66)). A table is
read in pieces: TSX reads TypeScript's shared piece and its JSX kinds while TypeScript adds the one
kind only its grammar has, and C and C++ share a piece the same way — so a shared piece never names
a kind the other grammar lacks ([slot.rs:24-28](../../../cli/src/merge/slot.rs#L24)).

Haskell has no flow table, and it is this repository's core language, so its slot table carries
the two lists a flow table would give: `stmt_kinds`, its statement forms (the do block's `bind`,
`exp`, `let` and `rec`, and the declarations — signatures, data and newtype declarations, classes,
instances, imports and the rest), and `container_kinds`, the kinds whose children stand in a
statement position (`do`, `declarations`, `local_binds`, the class and instance bodies). A
language with a flow table never fills either list — the unit leg holds both ways
([Haskell.hs:126-141](../../../core/app/CE/Lang/Haskell.hs#L126), [slot.rs:166](../../../cli/src/merge/slot.rs#L166)).
The Haskell grammar spells an expression, a pattern and a type with the same kinds (`apply`,
`variable`, `tuple`…), so its type class holds only the kinds that are types alone — a type
constructor's `name`, the arrow, `forall`, `context` and the like — and a shared kind reads as an
expression. Markdown and HTML have no function to fold into and no table: their groups are counted
`no_slot_table` and never sent.

A fragment's synthetic root carries kind `ce:fragment`, `leaf` 0 and `slot` 4, its `lld` the first
subtree's — the kept run hangs under it as siblings, so a fragment is one tree like any other
([tree.rs:136](../../../cli/src/dedup/t3/tree.rs#L136)).

### 3. The judgment

**Family 0 — a T1/T2 group** ([Align.hs:1-9](../../../core/app/CE/Merge/Align.hs#L1)). The
members are one token run, so their trees must be isomorphic: every member's `lld` column equal to
member 0's and every internal node's label and leaf hash equal. The leaves alone may differ — T2
normalises identifiers and literals to one token — and the holes are exactly the leaf positions
whose keys are not every member's, in postorder. The skeleton is member 0's tree with those
leaves left open ([Align.hs:35](../../../core/app/CE/Merge/Align.hs#L35)); an internal node whose
own tokens are not every member's is a hole too, at that node. A request whose T1/T2
group is not isomorphic is refused by name — `group <g>: members are not isomorphic` — since the
measuring side promised one token run ([Contract.hs:66](../../../core/app/CE/Merge/Contract.hs#L66)).

**Family 1 — a T3 pair** ([Mapped.hs:1-12](../../../core/app/CE/Merge/Mapped.hs#L1)). The two
trees are aligned by an optimal tree-edit mapping. `CE.Clone.Ted` gives it back beside the
distance: `tedMapping` reads the same cell recurrence as `ted` and only re-walks it, so its
distance is `ted`'s by construction ([Ted.hs:14-18](../../../core/app/CE/Clone/Ted.hs#L14),
[Ted.hs:56](../../../core/app/CE/Clone/Ted.hs#L56)). The Tai mapping is then narrowed top-down: a
mapped pair is kept only when its parents are a kept pair, starting from the two roots; roots not
mapped to each other leave no skeleton and the whole pair is one hole
([Mapped.hs:37](../../../core/app/CE/Merge/Mapped.hs#L37)). A kept pair whose keys differ is a leaf
hole — the mapping's relabel, its value the node alone ([Mapped.hs:48](../../../core/app/CE/Merge/Mapped.hs#L48));
a kept pair whose keys agree and whose own tokens differ — an operator, a keyword, a punctuation
mark — is a hole at that node in the other class, so it is never feasible
([Widen.hs:27](../../../core/app/CE/Merge/Widen.hs#L27)).
Between two neighbouring kept child pairs, and before the first and after the last, the unkept
subtrees on each side form a gap: two gaps node for node the same join the skeleton, any other is a
gap hole whose value on each side is that side's forest, an empty side an empty value
([Mapped.hs:72](../../../core/app/CE/Merge/Mapped.hs#L72)). A gap is a structural difference:
its class is "other" whichever side is present — an argument, an element or a statement one member
has and the other lacks is not a value a parameter can stand for — and two roots not mapped to each
other are one such gap over the whole pair. The one exception is a gap with subtrees on one side
and none on the other directly under a kept pair that stands in an expression position on both
sides — `b""` against `b"\n"`, the escape sequence only one string literal holds: it widens that
pair as the next paragraph widens a relabelled leaf
([Widen.hs:65](../../../core/app/CE/Merge/Widen.hs#L65)). An argument one member lacks sits under
an argument list, which is no expression, and stays a gap.

**Widening one level.** A member name, a method name, a field name or a string literal's content
is never a parameter by itself. A relabel hole on a leaf of class 4 — or, in a T3 pair, a gap
filled on one side only — whose parent pair stands in an expression position on both sides becomes
a hole over the parent's whole subtree: the skeleton
leaves the parent open as a gap, the value is the subtree, the class is expression, and every other
hole inside the parent — the parent's own-token hole, a gap among its children — is absorbed. One
level only, class-4 leaves and one-sided gaps only; a parent that is not an expression on both sides — the
target of an assignment — leaves the hole where it is, a `position` hole. Both families widen the
same way ([Widen.hs:43](../../../core/app/CE/Merge/Widen.hs#L43), [Widen.hs:55](../../../core/app/CE/Merge/Widen.hs#L55)).

**Holes to parameters.** The holes are ordered by member 0's postorder: a hole at a node keyed by
the node, a gap present on member 0's side by its first root's leftmost leaf, and a gap empty on
member 0's side just before the next kept child (its leftmost leaf, or the parent itself when none
follows), an outer one before an inner one — a place inside another first, siblings left to right,
a node's own tokens after everything inside it. Each hole has a text vector per member — a
relabelled leaf's text, an own-token hole's own tokens, a widened hole's parent text, a gap's
roots' texts, an empty side none — and holes with the same vectors are one parameter, numbered by
first appearance, so the parameter count is the number of distinct texts whatever the places'
kinds ([Holes.hs:39](../../../core/app/CE/Merge/Holes.hs#L39)). A hole whose texts are every
member's alike — a difference in whitespace alone — stays in the skeleton for the rebuild and is
no parameter, no reason and no hole row ([Holes.hs:32](../../../core/app/CE/Merge/Holes.hs#L32)).

**Feasibility and its reason.** A merge is feasible when every hole stands where a parameter can:
an expression position or a declared name. Otherwise the first infeasible hole in hole order
names the reason — 3 a gap hole whose forests hold a statement, 1 any other gap, an own-token
hole or a hole at a statement or other position, 2 a type position — a group whose every hole is
feasible but
whose parameters number more than six answers 4, and one whose holes and parameters pass but whose
savings (below) are not positive answers 5, `no_savings`: a merge that saves no line is not a
suggestion anyone would take, the stance of 4 ([Holes.hs:57-62](../../../core/app/CE/Merge/Holes.hs#L57),
[Holes.hs:74](../../../core/app/CE/Merge/Holes.hs#L74),
[Cost.hs:48-51](../../../core/app/CE/Merge/Cost.hs#L48), [Cost.hs:82-95](../../../core/app/CE/Merge/Cost.hs#L82)).

**The member kept and the savings.** The member kept is the one whose file has the greatest
in-degree in the reference graph — the file most others already reach — a tie to the least member
index ([Holes.hs:81-84](../../../core/app/CE/Merge/Holes.hs#L81)). The savings are the members'
lines less the merged function's lines and one call line per member left behind
([Holes.hs:64](../../../core/app/CE/Merge/Holes.hs#L64), [Cost.hs:53-56](../../../core/app/CE/Merge/Cost.hs#L53)).
A T1/T2 skeleton is the kept member itself; a T3 skeleton is the kept member's lines in the share
of its nodes the kept pairs cover, rounded up — the wire carries no per-node line, so this is an
honest integer estimate, never a measured count ([Holes.hs:89-95](../../../core/app/CE/Merge/Holes.hs#L89)).
A fragment group's merged function wraps the run in a helper, so either family adds the group's
`helper` lines to its skeleton; a whole unit's group sends 0 ([Holes.hs:96](../../../core/app/CE/Merge/Holes.hs#L96)).

**The wire.** The request is three tables — `groups=[[g,family,helper]]`, `members=[[g,m,unit,lines,fileIndeg]]`,
`trees=[{lab,lld,leaf,slot,own,text}]` in member order — and the reply one suggestion row per group
`[g,params,kept,savings,feasible,reason]`, its hole rows `[g,hole,param,m,post,postEnd]` and the
counts of groups, members, nodes, suggestions, holes and feasible suggestions
([Merge.hs:37-61](../../../core/app/CE/Merge.hs#L37)). `post` and `postEnd` are the member's first
and last root of the hole: one node twice for a leaf or relabel hole, the first and last root of a
gap's forest — siblings under one parent — and −1 −1 where that side is empty
([Tree.hs:87-98](../../../core/app/CE/Merge/Tree.hs#L87), [Holes.hs:46-67](../../../core/app/CE/Merge/Holes.hs#L46)).
The contract refuses, by name and in request order, a malformed group or member row, a member
whose group is unknown, a group of the wrong size for its family, a tree count unequal to the
member count, a tree failing `clone/1`'s own shape contract, a leaf, slot, own or text column
missing or of the wrong length, a slot out of range, a negative own or text, and a non-isomorphic
T1/T2 group ([Contract.hs:57](../../../core/app/CE/Merge/Contract.hs#L57),
[Contract.hs:122](../../../core/app/CE/Merge/Contract.hs#L122)). More than 4,096 groups or
131,072 nodes (`treeNodeCap`) answers a complete degraded reply with no rows — the node cap is
the one that keeps the widest request at both caps inside the protocol's 32 MiB line, the six
columns and a member row per one-node tree at their widest
([Cost.hs:29-37](../../../core/app/CE/Merge/Cost.hs#L29)); the measuring side chunks by both caps
and never splits a group across two requests ([wire.rs:75](../../../cli/src/merge/wire.rs#L75)).
So a reply the core degraded to a request this side priced within both caps is a drift between the
two sides' cap mirrors — an error naming both owners, never a document; a core without the family,
or one that stops answering mid-run, gives a document with `degraded`, and a core that cannot be
started is refused by name, since the document is the core's to lay out
([wire.rs:149](../../../cli/src/merge/wire.rs#L149), [face.rs:95](../../../cli/src/merge/face.rs#L95)).

### 4. The measuring side's rulings

The design booklet fixes the core's side; eight rulings of step 7 fix the measuring side
(recorded in the design booklet's decision log on landing):

1. The position-class tables live in the merge module in their own files and structure; the
   statement class is read off the flow tables' existing fields, never a second statement table —
   a language without a flow table names its own statement forms and containers in its slot table.
2. Each table is held by three legs: every kind and field exists in its grammar; every named node
   of the language's probe sample that answers 4 is a kind the table names other, a part name, a
   literal's content or an assignment target's root — the table's completeness evidence, naming
   whatever it missed; the kind sets are pairwise disjoint and none of them a statement form, a
   table's own statement and container lists are filled exactly when its language has no flow
   table, every table names its helper lines, and per table a probe source pins the classes the
   second generation moved ([slot.rs:58](../../../cli/tests/unit/merge/slot.rs#L58),
   [slot.rs:109](../../../cli/tests/unit/merge/slot.rs#L109), [slot.rs:147](../../../cli/tests/unit/merge/slot.rs#L147)).
3. `leaf`, `slot`, `own` and `text` come out of the one walk that emits `lab` and `lld`; `own`
   reads the very tokens that walk steps over to reach a node's named children.
4. The groups and the local pre-checks of §1; a member row's `unit` is the request-local member
   number (the core echoes it, never reads it), its `lines` the unit's line count or the
   fragment's kept run's (§1), its `fileIndeg` the reference graph's edges landing on the
   member's file node — the same graph `ce deadcode` judges, from the same index ([face.rs:185](../../../cli/src/merge/face.rs#L185)).
5. One report, `ce.merge-report/0.1.0`, for all three faces, laid out by the core over every
   chunk's rows joined, its counts tallied there, `merged_duplicates` (§1) beside them; each parameter labelled with every member's text at the parameter's first hole
   ([Document.hs:95](../../../core/app/CE/Merge/Document.hs#L95)). The text runs from the hole's first root
   to its last on that member — a leaf or relabel hole's node, a gap hole's whole forest — and an
   empty side reads `""` ([face.rs:257](../../../cli/src/merge/face.rs#L257)).
6. The T3 trees are built once for both families; `clone/1` never sends the new columns.
7. The console prints the counts and the groups not sent, then per group a head line, its
   members — a trimmed member's run beside its span — and one line per parameter with every
   member's text cut at 40 characters; `--group n`
   prints one group and leaves the JSON face whole ([Lines.hs:25-26](../../../core/app/CE/Merge/Lines.hs#L25)).
8. Exit codes: a judged document 0; a degraded one 2; an argument error — `--group` past the last
   group included — 2 ([main_merge.rs:27](../../../cli/src/main_merge.rs#L27)).

### 5. The faces

`ce merge [--group <n>] [--format json]`, the MCP tool `merge_suggestions` and the GUI's merge
family in the Reports hub all read the one document, which the core lays out (`document/1`) from
every chunk's answer joined on this side ([Document.hs:76](../../../core/app/CE/Merge/Document.hs#L76), [face.rs:119](../../../cli/src/merge/face.rs#L119)).
The document holds the counts (the core's, and `merged_duplicates`), `unsendable`, and per group its family (`t1t2` / `t3`), whether it
is a fragment, its members (`path`, `unit` as `path:key#nth` or null for a fragment, `lines` the
clone-family span, `run` the lines sent and priced — `lines` again for a whole unit), the
parameter count, the member kept, the savings, `feasible`, the reason by name (`ok`, `position`,
`type`, `spans_statements`, `too_many_params`, `no_savings` — `position` for a hole at a statement
or any other position no parameter can stand for) and the parameters, each with every member's `text`
([Document.hs:52](../../../core/app/CE/Merge/Document.hs#L52)). A core without the family,
or one that stops answering, gives a document with `degraded` naming why and no group — a request
the core did not judge licenses nothing; a core that answers degraded to a request this side priced
within both caps is a cap-mirror drift and an error, never a document. The GUI card leads with the
counts, then per group its members (the member kept starred, a trimmed member's run beside its
span) and a parameter table, one row per
parameter and one column per member.

### 6. Gates

The core's battery holds the anti-unification laws over its hand-written cases and two hundred
generated T1/T2 groups: instantiating the skeleton with each member's values rebuilds that member
node for node, and no parameter can be dropped — set to member 0's constant, some member no longer
rebuilds ([MergeProps.hs:54-67](../../../core/test/MergeProps.hs#L54), [Holes.hs:105-120](../../../core/app/CE/Merge/Holes.hs#L105)).
`tedMapping` is held to `ted` over the exhaustive small-tree family and two hundred generated pairs:
the same distance, a valid Tai mapping, a cost equal to the distance. Every hole's first and last
root per member is held over those groups and the two hundred pairs as T3 groups: −1 −1 on an empty
side, else `post ≤ postEnd`, one node or two children of one parent ([MergeProps.hs:186](../../../core/test/MergeProps.hs#L186)).
Every reported hole is live and two holes share a parameter exactly when they share their text
vectors ([MergeProps.hs:132](../../../core/test/MergeProps.hs#L132)); the second generation's
rulings each have a case and its reverse probe — an operator that differs, a member name widened to
its expression and a target root left alone, a one-sided gap widened to its literal and left alone
under a class-other parent, whitespace alone, an argument one member lacks, a gap
in member 0's postorder, a fragment's helper lines, one parameter per text
([MergeRulings.hs:1-12](../../../core/test/MergeRulings.hs#L1)). The generated groups' parameter
counts and feasibility, the reasons' coherence over every judged
and generated suggestion — feasible exactly at 0, 0 only with a line saved, 5 only without one
([MergeProps.hs:169](../../../core/test/MergeProps.hs#L169)) —, the equal-gap fold, both caps, the
empty request and the counts each have their leg, and seven request–reply pairs are golden
([golden.ndjson](../../../contracts/fixtures/merge/golden.ndjson)).

On the measuring side the unit legs hold the slot tables (§4 ruling 2), the nine-tenths rule, the
fragment's synthetic root, the trim — a boundary statement on one side trimmed off and the family
sent, a statement inside one side's only top still not one shape, a member whose boundary statement
was trimmed priced by its run ([groups.rs:158](../../../cli/tests/unit/merge/groups.rs#L158)) — the
isomorphism pre-check
leaving leaves open, every unsendable group counted by why, one member set giving one suggestion
with T1/T2 over T3 ([groups.rs:226](../../../cli/tests/unit/merge/groups.rs#L226)), a chunk never
splitting a group, the
request body's tables and `consume`'s reading of a healthy, a degraded (a cap-mirror drift) and a
skewed reply ([groups.rs:29](../../../cli/tests/unit/merge/groups.rs#L29),
[groups.rs:144](../../../cli/tests/unit/merge/groups.rs#L144), [groups.rs:184](../../../cli/tests/unit/merge/groups.rs#L184),
[wire.rs:47](../../../cli/tests/unit/merge/wire.rs#L47), [wire.rs:94](../../../cli/tests/unit/merge/wire.rs#L94));
the tree legs hold a leaf's hash to its own text and every column to the node count, and a tree
without extras to the tree with them less exactly its comments, every other column in step
([tree.rs:47](../../../cli/tests/unit/dedup/t3/tree.rs#L47), [tree.rs:76](../../../cli/tests/unit/dedup/t3/tree.rs#L76)).
The integration legs seed three renamed copies of one function, a near-miss pair with one extra
statement and a statement run two functions share, and hold the CLI's JSON to the library's byte
for byte, the whole-unit group feasible with one parameter per rename and the renamed names as its
texts, the T3 pair infeasible across statements with the extra statement as its gap's text, the
fragment group marked, `--group` and its refusal, `ce clone`'s report on the same fixture unmoved,
the MCP relay, and a core that cannot be reached refused by name
([merge_face.rs:118](../../../cli/tests/it/merge_face.rs#L118),
[merge_face.rs:183](../../../cli/tests/it/merge_face.rs#L183), [merge_face.rs:215](../../../cli/tests/it/merge_face.rs#L215)).

**The frozen suggestion set.** `contracts/eval/merge-suggestions-v<n>.json` freezes, for this
repository at one pinned commit (exported with `git archive`, `.gitmodules` removed) and for the four
pinned external corpora, the counts (`merged_duplicates` among them, §1), the groups not sent and
one row per suggestion — its members
(`at` the identity, `run` the lines priced), family, fragment, parameters, member kept, savings,
feasibility, reason and hole count; a row's sample id hashes the identities alone, and no two rows
of a corpus share their identities — one member set, one suggestion — so the id is unique by
construction ([eval_merge_suggestions.rs:50](../../../cli/tests/it/eval_merge_suggestions.rs#L50)).
It is kept whole, every corpus row by row (9.4 MB), because two gates read the external
corpora's rows: `the_sample_is_the_draw_of_the_frozen_set` redraws the sample from every row's
identity and feasibility, and `readings` reads each drawn row's feasibility, reason and parameter
count. The CI leg measures the pinned self tree again and holds every row; re-freezing re-measures
all five ([eval_merge_suggestions.rs:24](../../../cli/tests/it/eval_merge_suggestions.rs#L24),
[eval_merge_review.rs:66](../../../cli/tests/it/eval_merge_review.rs#L66), [eval_merge_review.rs:65](../../../cli/tests/it/eval_merge_review.rs#L65)).

**The audit.** A hundred suggestions are drawn from the frozen set by the shared identity hash — the
feasible and the infeasible half each, every half apportioned over the corpora by their share of
it. The audit is blind to the verdict — feasibility, reason, savings and the member kept are
withheld — and sees the core's parameterisation, which it may reject: each sampled row carries the
members' identities, runs and source and, per parameter the core found, every member's text.
Independent agents who read the sample alone give each row a feasibility, a reason (one of the six
names) and a parameter
count; the readings, overall and per corpus, are how often the core's feasibility and reason agree
with theirs, and `params_agree` — how often the count given after seeing the core's
parameterisation equals the core's ([eval_merge_review.rs:93](../../../cli/tests/it/eval_merge_review.rs#L93)).

**The precision.** Four judges, one batch of twenty-five each, answered the hundred rows; their answer files are
filed verbatim, in the sample's order, as the review doc, every line held to the same row check the gate re-runs
([answers.rs:119](../../../cli/tests/it/eval_merge_review/answers.rs#L119)). The precision doc reads no product: it is
the three frozen docs' readings by definition, so it carries no tree and lands in the review's commit
([eval_merge_review.rs:229](../../../cli/tests/it/eval_merge_review.rs#L229)). The core's feasibility agrees with the
audit on 83 of the 100 rows, its reason on 77 and its parameter count on 45 (zod 60 / 56 / 26 of 68, ripgrep 17 / 15 /
11 of 22, cobra 4 / 4 / 7 of 8, requests 1 / 1 / 1 of 1, this repository 1 / 1 / 0 of 1). Of the 23 reason
disagreements, 11 are rows the core answers `position` and the judges find feasible, and 4 more are `position` rows
saving no line that the judges answer `no_savings`; the core's parameter count exceeds the judges' on 50 rows and falls
short on 5. The readings gate nothing — the family is advice — and the registry
[EVAL-SET-MERGE.md](../../EVAL-SET-MERGE.md) lists every disagreement with the judge's note, for the next generation's
parameterisation and reason order.

**The second generation.** Every one of those disagreements was checked against the source; nine
rulings came of it (design booklet §13 items 42–50) — the own and text columns, an own-token
hole, one-level widening, every gap structural, member 0's postorder, a fragment's helper lines,
parameters by text, the three table fixes, and the batch prompt's reading rules, which spell the
core's rulings in the judge's words. A tenth came of the review of that commit (items 51–52): a
gap filled on one side only under an expression on both sides widens like a relabelled leaf, and the
prompt's structural rule names the same exception. The core and the tables changed; the frozen set, the sample and
the batches were drawn again as the second generation (`-v2`), the first generation's four docs kept
on disk as the record and still checked as one; the gates read the newest generation.

**The second generation's precision.** Four new judges, one batch of twenty-five each, answered the second sample
under the batch prompt's reading rules, and their files are filed verbatim as `merge-review-v2.json`; the precision doc
`merge-precision-v2.json` is again the three frozen docs' readings and lands with the review (design booklet §13 items
36 and 53). The core's feasibility agrees with the audit on 96 of the 100 rows, its reason on 92 and its parameter
count on 87 (zod 74 / 72 / 71 of 74, ripgrep 17 / 16 / 12 of 20, cobra 4 / 4 / 3 of 5, this repository 1 / 0 / 1 of 1).
Read as a classifier of feasibility, the core calls 50 rows feasible and the judges 46, all among the 50: precision
46 / 50, recall 46 / 46. The four rows the core alone calls feasible are T1/T2 groups whose difference the
second-generation rulings do not reach (an element or a parameter one member lacks, a match pattern, a const's type);
the parameter count is higher than the judges' on 9 rows and lower on 4. On the 73 questions both samples share (the
same member pairs under the same ids) the two generations' judges agree on feasibility 67 times and on the reason 66
times; against them the core agreed on feasibility 60 times and on the reason 54 times in the first generation, 69 and
66 times in the second. The readings still gate nothing; the registry
[EVAL-SET-MERGE.md](../../EVAL-SET-MERGE.md) lists the eight reason disagreements and the seven questions whose judges
changed, each with both verdicts.

### 7. Design boundaries

- A T3 skeleton's line count is an integer estimate from the kept pairs' share of nodes, not a
  measured count; its savings inherit that.
- Markdown and HTML groups have no function to fold into; they are counted `no_slot_table`.
- A fragment has no floor on how many top nodes it keeps: one top can be a whole `match` or `if`,
  so a run is sized by its lines and nodes, and a run too short to save a line answers `no_savings`.
- A parameter is a place where the members differ; whether the differing values share a type is not
  asked, so a feasible suggestion is a shape that folds, not a guarantee that the folded function
  type-checks.
