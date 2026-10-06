# Code query and architecture rules — Datalog over the index's facts

[index](../methodology.md) · [← 15 Same-role advisor — sparse retrieval and in-repo association](15-same-role-advisor-sparse-retrieval-and-in-repo-association.md) · [→ 17 Intra-function dead code — the flow graph, reachability and liveness](17-intra-function-dead-code-flow-graph-reachability-and-liveness.md)

Every family before this one judges one shape of entropy the tree already carries: a clone, a
dead file, a duplicated paragraph, a unit that plays another's part. This family lets the reader
*ask*. A question in CE Datalog — `dead(F)`, `depends(A, B), not same_dir(A, B)`, `N = count(F :
in(F, "src/**"))` — is answered over facts the index already holds, and every answer comes back
with the derivation that produced it. Written as `assert`, the same question is a rule: its
violations are witness rows, each with its proof, and `ce rules` exits on the first one, so an
architecture constraint ("the measuring side never reads the core", "no page of the site is an
orphan") becomes a gate the way the clone budget is one
([mod.rs:1-13](../../../cli/src/query/mod.rs#L1)). The split is ADR-008's, seventh instalment:
Rust lexes the text, assembles the fact tables the program names from its own index and puts
the names back into the document the core lays out; the grammar, the sorts, the safety and stratification checks, the evaluation
and every proof are Haskell's, over the thirteenth wire family, `query/1`, since proto 7.3.0
([wire.rs:1-8](../../../cli/src/query/wire.rs#L1), [Query.hs:5-18](../../../core/app/CE/Query.hs#L5)).
No name, path or source text crosses the wire in either direction: a program's constants go up
as fnv1a64 hashes and request-local ids, and the answers come back as ids this side labels
again through the request's own tables ([legend.rs:1-8](../../../cli/src/query/legend.rs#L1)).

### 1. The language — CE Datalog v1

```
program  := clause*
clause   := rule | query | assert
rule     := atom ':-' body '.'  |  atom '.'
query    := '?-' body '.'
assert   := 'assert' atom ':-' body '.'
body     := lit (',' lit)*
lit      := atom | 'not' atom | cond | bind
atom     := pred '(' term (',' term)* ')'
term     := Var | '_' | int | "string" | name
cond     := expr ('=' | '!=' | '<' | '<=' | '>' | '>=') expr
bind     := Var '=' expr | Var '=' agg
expr     := term (('+' | '-' | '*' | '/' | '%') term)*
agg      := ('count' | 'min' | 'max' | 'sum') '(' Var (',' Var)* ':' body ')'
```

Predicate names and name constants start lowercase, variables uppercase; `_` (and any word
starting with it) is a fresh variable at every occurrence; `#` comments to the end of the line,
which leaves `%` free to be the remainder operator. The scanner knows spellings and nothing of
shapes: a lowercase word is a predicate when a `(` follows it and a name constant otherwise, a
quoted string is a name except in the one position whose sort is a set, and `in(F, "glob")` is
sugar rewritten on the spot to `set(S, F)` with the glob seated as set `S`
([lexer.rs:1-12](../../../cli/src/query/lexer.rs#L1), [lexer.rs:349](../../../cli/src/query/lexer.rs#L349)).
The grammar lives in the core's hand-written recursive descent and nowhere else — the freeze's
dependency set has no parser library — and the first shape it cannot read is the program's one
syntax error, reported at the token it stopped on
([Parse.hs:1-8](../../../core/app/CE/Query/Parse.hs#L1), [Parse.hs:37](../../../core/app/CE/Query/Parse.hs#L37)).
A body literal that starts with a variable is `Var = agg`, `Var = expr` or a comparison whose
left side begins with the variable; whether `Var = expr` binds or compares is the checker's call
(bound on the left = comparison) ([Parse.hs:120](../../../core/app/CE/Query/Parse.hs#L120),
[Syntax.hs:48-54](../../../core/app/CE/Query/Syntax.hs#L48)). An `assert`'s head names the witness
columns; a violation is one head tuple, and the assertion passes when there is none. A `?-`
answers with the variables its body binds, in the order the safety walk binds them
([Syntax.hs:58-61](../../../core/app/CE/Query/Syntax.hs#L58), [columns.rs:1-8](../../../cli/src/query/columns.rs#L1)).

### 2. The facts — twenty-seven predicates read off the index

The schema is one table with a row per predicate — its code, its name, the sort of each
argument — held in the core and mirrored line for line on the measuring side; the core echoes
it under `schema` when asked, and a unit leg holds the two copies equal
([Schema.hs:18](../../../core/app/CE/Query/Schema.hs#L18), [legend.rs:14](../../../cli/src/query/legend.rs#L14),
[legend.rs:12](../../../cli/tests/unit/query/legend.rs#L12)). Sorts are node 0, dir 1, unit 2, int 3,
sym 4, set 5; a position nothing constrained resolves to the open sort, −1
([Cost.hs:140](../../../core/app/CE/Query/Cost.hs#L140)).

| predicate | sorts | read off |
|---|---|---|
| `node(N, K)` · `file(F)` | node, sym | the graph wire's nodes — files, packages, sections, walked assets, the same dense ids `graph/1` judges — with the prose-only files the docdup family reads appended after them; `K` is `file` / `pkg` / `section` / `asset` / `prose`, and `file` is the view of kind `file` ([graph.rs:30](../../../cli/src/query/facts/graph.rs#L30), [legend.rs:132](../../../cli/src/query/legend.rs#L132)) |
| `in_dir(N, D)` · `dir(D)` · `parent(D, P)` · `dir_name(D, S)` | node, dir, sym | the directory tree the structure family builds over every node path, a package seated at its own directory; root is dir 0 and reads `.`, and `parent` has no row for it ([graph.rs:154](../../../cli/src/query/facts/graph.rs#L154)) |
| `lang(F, L)` | node, sym | `Lang::name` of every file-shaped node, `unknown` where no language claims the path |
| `role(N, R)` | node, sym | one row per set role bit: `entry_named` / `entry_dir` / `test` / `glob` / `doc` / `allow` / `declared` / `foreign` / `unit` / `asset` ([graph.rs:87](../../../cli/src/query/facts/graph.rs#L87), [legend.rs:137](../../../cli/src/query/legend.rs#L137)) |
| `lines(F, N)` | node, int | one read per file-shaped node, only when a program asks ([graph.rs:137](../../../cli/src/query/facts/graph.rs#L137)) |
| `ref(N, M, K, R)` | node, node, sym, int | every resolved edge of the graph wire with its kind (`import` / `doc_link` / `doc_ref` / `asset` / `contain` / `refdef`) and its rung; External and Unresolved sites have no edge, and `unresolved(F, N)` counts the latter per file ([legend.rs:151](../../../cli/src/query/legend.rs#L151)) |
| `unit(U, F)` · `unit_kind(U, K)` · `unit_lines(U, N)` · `unit_at(U, L)` | unit, node, sym, int | the index's symbol rows in one dense order (file, first line, last line, key, nth), each seated in its file node; kinds `fn` / `named` / `impl` / `section` ([units.rs:42](../../../cli/src/query/facts/units.rs#L42)) |
| `named(U, S)` · `exported(U)` · `params(U, N)` | unit, sym, int | the declared name the mention pass would spell, the visibility bit, the arity off the key |
| `coc(U, N)` · `cyclo(U, N)` · `nesting(U, N)` | unit, int | the three complexity numbers the core derives for `ce scan`, seated on the unit sharing the function's file and span — a span two units share goes to the one whose key names the function, and to none when neither does ([units.rs:115](../../../cli/src/query/facts/units.rs#L115)) |
| `clone(U, V, K)` | unit, unit, sym | `t1t2`: a T1/T2 block covering whole units on both sides pairs them in order; `t3`: the pairs the core judged clones, on the units' identities ([pairs.rs:45](../../../cli/src/query/facts/pairs.rs#L45), [pairs.rs:62](../../../cli/src/query/facts/pairs.rs#L62)) |
| `dup(F, G, N)` · `docdup(F, G)` | node, node, int | the T1/T2 blocks between two files with their token count; the file pairs behind the segment pairs the docdup core judged — each pair once, lower id first ([pairs.rs:1-6](../../../cli/src/query/facts/pairs.rs#L1)) |
| `mention(S, F)` | sym, node | the mention pass's own table: the identifier hashes it stores are the hashes a program's `"name"` lexes to ([text.rs:42](../../../cli/src/query/facts/text.rs#L42)) |
| `class(F, S)` | node, sym | the `[[rules.class]]` matcher the scan uses, by the class's name ([text.rs:59](../../../cli/src/query/facts/text.rs#L59)) |
| `set(S, F)` | set, node | each glob the program spelled, expanded over the file-shaped nodes through the exclude list's own dialect ([text.rs:13](../../../cli/src/query/facts/text.rs#L13)) |

Only the tables the program names are built, and the costly ones — a scan for the complexity
numbers, the T3 and docdup judgments, the mention pass, a read of every file for its line count
— only when their predicate is read; a table is a set, sorted and deduplicated before it goes up
([mod.rs:1-7](../../../cli/src/query/facts/mod.rs#L1), [mod.rs:68](../../../cli/src/query/facts/mod.rs#L68),
[mod.rs:123](../../../cli/src/query/facts/mod.rs#L123)). One hash serves every name-shaped
constant: `mention("foo", F)` meets the index's own rows because the lexer and the mention
table key by the same fnv1a64, and the enum words a table spells (`entry`, `import`, `t3`) are
name constants in the program, so `role(N, entry_named)` and `role(N, "entry_named")` are one
token ([legend.rs:125](../../../cli/src/query/legend.rs#L125), [Cost.hs:88](../../../core/app/CE/Query/Cost.hs#L88)).

### 3. The prelude

Eight clauses ship inside the binary as a `.rules` text and go up the same wire as the user's
program, so the core never distinguishes them; their predicates are reserved, and a program that
redefines one is a program error ([mod.rs:27](../../../cli/src/query/mod.rs#L27),
[prelude.rules:8-15](../../../cli/src/query/prelude.rules#L8)):

```
entry(N) :- role(N, _).
reach(N) :- entry(N).
reach(M) :- reach(N), ref(N, M, K, _), K != asset, K != refdef.
dead(F) :- file(F), not reach(F).
depends(N, M) :- ref(N, M, _, _).
depends(N, M) :- depends(N, K), ref(K, M, _, _).
same_dir(F, G) :- in_dir(F, D), in_dir(G, D), F != G.
dir_ref(D, E) :- ref(F, G, _, _), in_dir(F, D), in_dir(G, E), D != E.
```

`dead` mirrors `ce deadcode`'s file-tier verdict by construction: an entry is a node with any
role bit (every role lands in the core's entry mask), and a reference of any rung is followed
except the two liveness-inert kinds the graph judgment drops the same way. The integration leg
holds the two roads to one file ([query_face.rs:87](../../../cli/tests/it/query_face.rs#L87)).
`ce query --prelude` prints the text verbatim.

### 4. The token stream

The request carries the program as `[kind, value]` pairs: 0 a predicate code, 1 a variable, 2
an integer, 3 a set id, 4 a name hash, 6 an anonymous variable, and 10 to 33 the punctuation and
keywords (`:-` `,` `.` `(` `)` `not` `?-` `assert` `=` `!=` `<` `<=` `>` `>=` `+` `-` `*` `/`
`%` `count` `min` `max` `sum` `:`); 5 was an enum kind in the booklet's first draft and is
deliberately unassigned ([lexer.rs:44](../../../cli/src/query/lexer.rs#L44),
[Cost.hs:88](../../../core/app/CE/Query/Cost.hs#L88)). Variables number per clause by first
appearance; program predicates number from 1000 by first appearance across every source —
the prelude first, then the rules file, then the ad hoc query — and codes below 1000 are the
schema's ([lexer.rs:293](../../../cli/src/query/lexer.rs#L293), [program.rs:101](../../../cli/src/query/program.rs#L101),
[program.rs:115](../../../cli/src/query/program.rs#L115), [Cost.hs:81-82](../../../core/app/CE/Query/Cost.hs#L81)).
Every token keeps the line and column it came from, so an error the core reports at a token
index reads back as `where line:column` — the one text-shaped thing about an error, and it
never crosses the wire ([program.rs:131](../../../cli/src/query/program.rs#L131)). A malformed
token or table is the measuring side's fault and is refused by name before any judging —
unknown kind, negative value, unknown predicate code, unknown fact predicate, wrong arity,
negative fact value, a table not strictly ascending, a negative prelude count
([Contract.hs:5-13](../../../core/app/CE/Query/Contract.hs#L5), [Contract.hs:56](../../../core/app/CE/Query/Contract.hs#L56)).

### 5. The checks — arity, sorts, safety, stratification

A parsed program passes six phases before anything is evaluated, and the first phase that finds
fault answers every error it found, each at a token; a program with an error answers nothing
([Check.hs:42](../../../core/app/CE/Query/Check.hs#L42)). The codes: 1 syntax, 2 unknown
predicate, 3 arity, 4 sort, 5 unsafe variable, 6 unstratifiable negation, 7 prelude predicate
redefined, 8 aggregate shape, 9 anonymous head ([Cost.hs:152-153](../../../core/app/CE/Query/Cost.hs#L152),
[Document.hs:61](../../../core/app/CE/Query/Document.hs#L61)).

- **Arity.** A fact predicate's arity is the schema's; a program predicate's is fixed by its
  first appearance.
- **Sorts.** A union-find over three kinds of node — a clause variable, a program predicate's
  position, a constant sort — meets every fact argument with its declared sort, every program
  position with whatever flows into it, every variable with its first use; an integer literal
  is int, a string is sym (set in the set position), a comparison's two sides are one sort, an
  operator's operands and an aggregate's result are int. The first disagreement is the sort
  error, at the token that disagreed ([Sorts.hs:1-8](../../../core/app/CE/Query/Check/Sorts.hs#L1),
  [Sorts.hs:63](../../../core/app/CE/Query/Check/Sorts.hs#L63), [Sorts.hs:94](../../../core/app/CE/Query/Check/Sorts.hs#L94)).
- **Safety** (range restriction). A variable is read only after a positive atom or a binding
  earlier in the same body bound it, so every negation, comparison, arithmetic operand and head
  argument names a value the evaluator will hold; an aggregate's result variable must be new,
  its grouped variables must be bound by the inner body and not by the outer one, and an
  anonymous head argument names no column ([Safety.hs:1-8](../../../core/app/CE/Query/Check/Safety.hs#L1),
  [Safety.hs:55](../../../core/app/CE/Query/Check/Safety.hs#L55), [Safety.hs:82](../../../core/app/CE/Query/Check/Safety.hs#L82)).
  The order the outer level binds its variables in is the projection a query answers with
  ([Safety.hs:23](../../../core/app/CE/Query/Check/Safety.hs#L23), [columns.rs:115](../../../cli/src/query/columns.rs#L115)).
- **Unknown and prelude.** A program predicate read anywhere but defined by no rule; a rule past
  the first `prelude` clauses whose head is a reserved predicate.
- **Stratification.** A head sits at least as high as every positive body predicate and strictly
  above every negated or aggregated one (an aggregate's body counts as negative dependence); the
  levels are raised to a fixpoint, and a negative edge inside a strongly connected component of
  the dependency graph is the unstratifiable rule, named at its head
  ([Check.hs:104](../../../core/app/CE/Query/Check.hs#L104), [Check.hs:121](../../../core/app/CE/Query/Check.hs#L121),
  [Syntax.hs:88](../../../core/app/CE/Query/Syntax.hs#L88)).

What comes out is the evaluator's program: the rules with their strata, the goals with their
projections and sorts, and every program predicate's resolved position sorts — the reply's
`preds` table, which the measuring side labels a proof node's arguments by
([Check.hs:30-39](../../../core/app/CE/Query/Check.hs#L30), [Check.hs:60](../../../core/app/CE/Query/Check.hs#L60),
[Query.hs:82](../../../core/app/CE/Query.hs#L82)).

### 6. Evaluation — semi-naive, one stratum at a time

Set semantics, least model. The strata run lowest first; inside a stratum every rule runs once
over the full database, then each rule whose body reads a predicate of its own stratum re-runs
once per such position with the previous round's new tuples routed to that position, until a
round derives nothing new. Relations below the stratum are indexed once for the whole stratum,
the stratum's own once per round ([Eval.hs:1-11](../../../core/app/CE/Query/Eval.hs#L1),
[Eval.hs:50](../../../core/app/CE/Query/Eval.hs#L50), [Eval.hs:73](../../../core/app/CE/Query/Eval.hs#L73)).
An index is lazy: a join asks for the tuples that agree with the positions already bound, and
each distinct set of bound positions — a bitmask over the arity — gets its map the first time
it is asked for and never again, so a mask no join uses costs nothing; lookups come back in the
relation's own order, which makes every join deterministic
([Index.hs:1-7](../../../core/app/CE/Query/Eval/Index.hs#L1), [Index.hs:36](../../../core/app/CE/Query/Eval/Index.hs#L36)).

A body is joined literal by literal in written order, each narrowing the substitutions before
it: a positive atom joins through the routed index, a negation tests absence, a comparison
filters, a binding extends, an aggregate folds an inner body's answers under the outer
substitution ([Join.hs:1-9](../../../core/app/CE/Query/Eval/Join.hs#L1), [Join.hs:28](../../../core/app/CE/Query/Eval/Join.hs#L28)).
Arithmetic is integer-only; a division by zero has no value, so the literal it sits in simply
fails rather than raising. Two ids of one sort order like their numbers, which is what `F < G`
uses to break a symmetric pair. `count` and `sum` of nothing are 0; `min` and `max` of nothing
have no value, so their literal fails; groups follow the outer binding
([Join.hs:76](../../../core/app/CE/Query/Eval/Join.hs#L76), [Join.hs:95](../../../core/app/CE/Query/Eval/Join.hs#L95),
[Join.hs:109](../../../core/app/CE/Query/Eval/Join.hs#L109)). The first derivation of every tuple
is kept as its provenance — the rule and the facts that rule read — and a goal's answers are
read off the final database in tuple order ([Eval.hs:93](../../../core/app/CE/Query/Eval.hs#L93)).

### 7. Proofs

For an answer, the tree is the rule that produced it and, under each positive body atom it
read, the tree of that fact, down to the rows the measuring side sent. Rows are
`[goal, answer, node, parent, rule, pred, args…]`: nodes numbered in pre-order, the root's
parent −1, `rule` the clause index (−1 for a sent fact), `pred` the node's predicate code (−1
for a query's root) ([Proof.hs:1-7](../../../core/app/CE/Query/Proof.hs#L1),
[Proof.hs:56](../../../core/app/CE/Query/Proof.hs#L56)). A `?-`'s trees are expanded only under
`why`; an assertion's violations carry theirs always — a violation without its derivation is a
bare accusation ([Query.hs:91](../../../core/app/CE/Query.hs#L91)). One global node budget,
`proofCap`, governs a reply: a tree that would not fit whole is left out and counted in
`counts.proofTruncated`, never cut in the middle, so every emitted tree is a complete
derivation and no answer is ever dropped for its proof's sake
([Proof.hs:38](../../../core/app/CE/Query/Proof.hs#L38)). The console prints each node under its
answer, indented by depth, with `(clause N)` or `(fact)` beside it; the GUI does the same in a
row under the answer ([Lines.hs:40](../../../core/app/CE/Query/Lines.hs#L40),
[query.js:85](../../../gui/ui/query.js#L85)).

### 8. Caps and degradation

Four request dimensions, each with its own ceiling. Tokens (65,536) and fact rows (4,194,304)
are counted before judging and answer a complete degraded reply — empty tables, `degraded:
true`, `reason: query_too_large`; a program the core refused to judge has no answers and no
errors. The derived ceiling (2,097,152 tuples) is checked after every round of every stratum,
and crossing it abandons the whole evaluation for the same degraded reply with the count it
reached — never a partial answer. The proof ceiling (16,384 nodes) never degrades: it stops
expanding and counts ([Cost.hs:72-76](../../../core/app/CE/Query/Cost.hs#L72),
[Contract.hs:49](../../../core/app/CE/Query/Contract.hs#L49), [Query.hs:125](../../../core/app/CE/Query.hs#L125)).
On the measuring side a degraded reply is a named non-judgment carried in the document, a core
that offers no `query/1` reads the same way, and a reply whose tables disagree with what was
sent — a goal count, an answer's arity, an error index past the stream — is wire skew, never a
healthy answer ([wire.rs:103](../../../cli/src/query/wire.rs#L103), [rows.rs:17](../../../cli/src/query/rows.rs#L17)).
The family has no knobs, no fail tier and no condition bit: `ce rules` reads
`counts.violations` and nothing else.

### 9. The faces and their exit codes

One document serves the three faces — `ce.query-report/0.1.0` for a question,
`ce.rules-report/0.1.0` for a rules file — the program as lexed, every goal with its columns
and sorts, every answer labelled through the request's own tables (a node as its path, a dir as
its path or `.`, a unit as `path:start-end key`, a set as its glob, a name through the reverse
dictionary), every proof row named, every error at its `where line:column`, the core's counts,
and the named reason when the core did not judge. The core lays the document out (`document/1`,
since proto 7.8.0) from the program's facts, its answered tables and the goals as spelled, which
this side sends back; this side puts every position, name and value back through the request's
own tables, and a core it cannot reach for the layout is refused by name, exit 2
([Document.hs:99](../../../core/app/CE/Query/Document.hs#L99), [face.rs:1-12](../../../cli/src/query/face.rs#L1),
[face.rs:223](../../../cli/src/query/face.rs#L223), [mod.rs:36](../../../cli/src/query/facts/mod.rs#L36)). A question is wrapped once into query form —
`?-` in front, `.` behind — unless written ([face.rs:37](../../../cli/src/query/face.rs#L37)); a glob
the exclude dialect cannot read is a program error at the glob's token before any table is built
([face.rs:163](../../../cli/src/query/face.rs#L163)).

- **`ce query <body> [--why] [--file <rules>] [--prelude]`** answers one question built on the
  rules file's rules; exit 0 when judged, 2 on a program error or a core that could not judge.
  It is a report: an assertion's violations in the file it builds on do not move its exit code
  ([main_query.rs:47](../../../cli/src/main_query.rs#L47)).
- **`ce rules [--file <rules>] [--why]`** judges every `assert` in the file: exit 1 when any
  assertion holds a violation, 2 when the program did not judge, 0 otherwise — and a missing
  default file is zero assertions and 0, said aloud
  ([main_query.rs:1-8](../../../cli/src/main_query.rs#L1), [main_query.rs:61](../../../cli/src/main_query.rs#L61)).
- **The rules file** is the one named on the command line or by the MCP argument (root-relative
  unless absolute, and it must exist), else `[rules] file` from the config (it must exist), else
  `ce.rules` at the project root when it exists — else none, and the program is the prelude
  alone. `[rules] file` is a path the family reads, never a knob: the knob fingerprint drops it,
  so declaring it moves no baseline ([mod.rs:37](../../../cli/src/query/mod.rs#L37),
  [rules.rs:25-29](../../../cli/src/config/rules.rs#L25), [canonical.rs:83-86](../../../cli/src/config/canonical.rs#L83)).
- **The console** prints the errors first, then the degraded reason if any, then each goal —
  `?- F: 1 answer(s)` with its rows, `assert no_dead(F): ok` or `: N violation(s)` with its
  witnesses — every proof node indented under its answer, and one counts line; every sentence
  has its Chinese twin and the program's own words stay as written
  ([Lines.hs:15-19](../../../core/app/CE/Query/Lines.hs#L15), [Query.hs:10-20](../../../core/app/CE/Text/Query.hs#L10),
  [main_lang.rs:76](../../../cli/src/main_lang.rs#L76)).
- **MCP** `query` (`body`, `why`, `file`) and `rules` (`file`, `why`) return the same document;
  the CLI's exit code is its own reading of `counts.violations` and does not exist here
  ([tools.rs:195](../../../cli/src/mcp/tools.rs#L195), [tools.rs:216](../../../cli/src/mcp/tools.rs#L216),
  [adapters.rs:179](../../../cli/src/mcp/adapters.rs#L179)).
- **The GUI's Query screen** — the twelfth tab — takes a question in a box with a `why` switch,
  answers it as one table per goal under its own column names with the derivation rows under an
  answer, and judges the project's rules file with one button; rendering only, a core without the
  family shows the document's degraded posture
  ([commands_query.rs:14](../../../gui/src-tauri/src/commands_query.rs#L14), [query.js:21](../../../gui/ui/query.js#L21),
  [query.js:61](../../../gui/ui/query.js#L61)).

All three go through two library functions, so the document cannot differ by face
([faces.rs:212](../../../cli/src/faces.rs#L212), [faces.rs:225](../../../cli/src/faces.rs#L225)); the
integration leg holds the CLI's JSON to the library's byte for byte
([query_face.rs:87](../../../cli/tests/it/query_face.rs#L87)), and the parity table claims the
capability once across CLI, GUI and MCP ([face_parity_table.rs:35](../../../cli/tests/it/face_parity_table.rs#L35)).

### 10. The repository's own rules

The repository judges itself with nine assertions in `ce.rules` at the root, each written against
the day's facts and green before it entered ([ce.rules:1-6](../../../ce.rules#L1)):

- ADR-008's division: no file under `cli/src` references one under `core`, no file under
  `core/app` references one under `cli`, and the core reads nothing that is not Haskell
  ([ce.rules:10-12](../../../ce.rules#L10));
- the scan stays below the graph: nothing under `cli/src/scan` references `cli/src/graph`
  ([ce.rules:16](../../../ce.rules#L16));
- the product never depends on its integration tests, and the GUI crate reads no test at all
  (the unit tree is mounted from `cli/src` by `#[path]`, the one exception)
  ([ce.rules:21-22](../../../ce.rules#L21));
- no dead file — the deadcode gate restated as a rule ([ce.rules:26](../../../ce.rules#L26));
- every page of the site is referenced by some other page or document
  ([ce.rules:29](../../../ce.rules#L29));
- no two top-level directories reference each other both ways
  ([ce.rules:32](../../../ce.rules#L32)).

The test suite carries three of its own, rooted at `cli/tests`: no dead file, and the integration
tree and the unit tree never read each other ([ce.rules:7-13](../../../cli/tests/ce.rules#L7)).
CI runs `ce rules` as the seventh dogfood gate on both roots, beside scan, the dedup ratchet,
the check floor, deadcode, docdup and erase
([ci.yml:266](../../../.github/workflows/ci.yml#L266), [ci.yml:289](../../../.github/workflows/ci.yml#L289)).

### 11. Gates and goldens

The core's battery has twelve legs: the prelude's dead-file query with its proof and counts; an
assertion's violations carrying their derivation unasked; every program error named at its
token, the erroring program answering nothing; proof trees replayed node by node against the
naive reference; the four aggregates with their empty cases and grouping; arithmetic, `=` on a
bound variable as a comparison, division by zero, `<` on ids; goals with their kinds and sorts
and `preds` with the open sort; the schema echo only when asked; every contract refusal by name;
the three capped dimensions; proof trees past the budget left out whole; the empty program
([QueryProps.hs:5-14](../../../core/test/QueryProps.hs#L5), [QueryProps.hs:65](../../../core/test/QueryProps.hs#L65)).
The reference is an independently written naive evaluator — every rule over the whole database
every round, no indexes, no delta routing, no provenance, the join a plain nested product in
list-comprehension form — that shares only the checker; the shipped evaluator must agree tuple
for tuple on every program of the enumerated family over every seeded fact set (five programs,
forty sets, 200 cases), and the same reference replays a proof tree by checking that each rule
node's children are one solution of its body ([ReferenceQuery.hs:1-12](../../../core/test/ReferenceQuery.hs#L1),
[ReferenceQuery.hs:34](../../../core/test/ReferenceQuery.hs#L34)). Eight golden pairs pin the
wire bytes — the prelude with the schema echo, an assertion's violation, a `why` query negating
a prelude predicate, a syntax error, an unsafe variable, an unstratifiable pair, arithmetic, an
aggregate over a set — and a tests-repo leg regenerates them through the real lexer so every
request line is the prelude followed by its program
([Spec.hs:112](../../../core/test/Spec.hs#L112), [query_golden.rs:25](../../../cli/tests/it/query_golden.rs#L25),
[query_golden.rs:111](../../../cli/tests/it/query_golden.rs#L111)).

On the measuring side the unit legs hold the legend to the core's echo and the vocabulary to the
graph's codes, the lexer's numbering and its faults at their place, a goal's columns in the
safety walk's order, the request body's tables and flags, and `consume`'s reading of a healthy,
a degraded and a skewed reply ([legend.rs:12](../../../cli/tests/unit/query/legend.rs#L12),
[program.rs:19](../../../cli/tests/unit/query/program.rs#L19), [wire.rs:63](../../../cli/tests/unit/query/wire.rs#L63),
[face.rs:142](../../../cli/tests/unit/query/face.rs#L142)). The integration legs seed a Cargo package
and hold `dead(F)` to `ce deadcode`'s own road, run the sugar, the aggregates, the arithmetic and
every program error through the same face, treat `ce rules` as the gate it is — exit 1 on one
violation, the witness and its chain on the console — and name the rules file by flag, config or
default ([query_face.rs:140](../../../cli/tests/it/query_face.rs#L140), [query_face.rs:172](../../../cli/tests/it/query_face.rs#L172),
[query_face.rs:231](../../../cli/tests/it/query_face.rs#L231)). The family is one row of the parity
table, one screen of the GUI roster and two tools of the MCP catalogue; docs cite implementation
lines (this booklet is under the citations gate), and the caps above bind to their source names
under `docs_consts`.
