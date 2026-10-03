# Architecture analysis — layers, cuts, clusters, impact

[index](../methodology.md) · [← 18 Clone merge suggestions — anti-unification over the clone families' trees](18-clone-merge-suggestions-anti-unification.md)

The graph family answers which files nothing reaches; the structure family scores how the tree is
laid out. Neither says how the directories *depend on each other*: which ones sit at the bottom,
which pairs reference each other both ways and so cannot be layered, which arcs would have to go
to break those cycles most cheaply, which files would sit better beside the files they talk to,
and what a change to one file reaches. This family answers those five questions, as advice. No
gate reads it, no baseline holds it and `ce check` never sees it
([mod.rs:1-9](../../../cli/src/arch/mod.rs#L1)). The split is ADR-008's, seventh instalment: Rust
measures the tree it already indexed — the measured files, their directories, the references
between them — as five integer tables and labels the answer back; the directory graph, its
feedback arc set, the layers, the clusters, the misplaced files, the impact walk and the metrics
are Haskell's, over the wire family `arch/1` ([Arch.hs:5-19](../../../core/app/CE/Arch.hs#L5),
[wire.rs:1-12](../../../cli/src/arch/wire.rs#L1)). No path crosses the wire in either direction.

### 1. The tables and where they come from

| table | row | read off |
|---|---|---|
| `files` | `[F, D, lines]` | the measured FILE nodes of the deadcode wire — not foreign, not an asset, not a section, not a package — in path order, dense from 0; `lines` is the file's total line count, the same count `ce scan` takes ([tables.rs:54](../../../cli/src/arch/tables.rs#L54), [face.rs:51](../../../cli/src/arch/face.rs#L51)) |
| `dirs` | `[D, parent]` | the structure family's directory tree over those paths: the root row 0 with parent −1, every other parent an earlier row, since the tree enters every ancestor before its child ([tables.rs:129](../../../cli/src/arch/tables.rs#L129)) |
| `edges` | `[F, G, w]` | every graph arc from one measured file to another — any kind, any rung, through the structure family's own file join — plus an arc to a Markdown section, folded onto the section's file; a self pair is dropped and `w` counts the arcs between the pair ([rows.rs:89](../../../cli/src/structure/rows.rs#L89), [tables.rs:96](../../../cli/src/arch/tables.rs#L96)) |
| `pkgEdges` | `[F, D, w]` | the arcs whose target is a package node — a Go, R or Java package import, a Markdown directory link — onto the package's directory; a package outside the tree has no row, and a reference into the file's own directory is kept for the core to fold |
| `focus` | `[F…]` | the `--impact` paths, root-relative with forward slashes, ascending and distinct; a path that names no measured file is an error by name and no document ([tables.rs:152](../../../cli/src/arch/tables.rs#L152)) |

Two caps, mirrored before the request leaves: 131,072 files, and 524,288 references across the
two reference tables together, since both become arcs of the one directory graph
([Cost.hs:24](../../../core/app/CE/Arch/Cost.hs#L24), [Cost.hs:30](../../../core/app/CE/Arch/Cost.hs#L30),
[wire.rs:29](../../../cli/src/arch/wire.rs#L29)). A request past either is refused on this side as
`arch_too_large` and the document carries that reason; a core that cannot be started or answer,
or one without the family, is named the same way; a core that degrades a request priced inside
the caps is a cap-mirror drift and an error. The core's contract names the first
offending row of any table by its index — width, identity, range, order, the one root, a parent
before its child ([Contract.hs:48](../../../core/app/CE/Arch/Contract.hs#L48)).

### 2. The judgment

**The directory graph.** Every file edge becomes an arc from its source's directory to its
target's, every package edge an arc from the file's directory to the named one; an arc inside
one directory is not counted, and one ordered pair met twice is one arc with its weights added
([Dirs.hs:27](../../../core/app/CE/Arch/Dirs.hs#L27)). Its strongly connected components come
from the graph family's own decomposition — one Tarjan in the core, not two
([Dirs.hs:48](../../../core/app/CE/Arch/Dirs.hs#L48)).

**The cuts.** The cheapest set of arcs whose removal leaves the graph acyclic is found one
component at a time: an arc between two components lies on no cycle, so the minimum over the
graph is the union of the components' minima ([Fas.hs:50](../../../core/app/CE/Arch/Fas.hs#L50)). A
component of at most fourteen vertices is ordered by a dynamic programme over vertex subsets —
cost(S) is the least over v ∈ S of cost(S ∖ {v}) plus the arcs from v back into S ∖ {v}, the cost
a triple (weight, count, the sorted arcs) compared in that order — and every minimum-weight cut
is the backward set of some order, so the answer is the minimum under the tie rule and the row
says `exact` 1 ([Fas.hs:84](../../../core/app/CE/Arch/Fas.hs#L84), [Cost.hs:39](../../../core/app/CE/Arch/Cost.hs#L39)).
A larger component goes through the Eades–Lin–Smyth order on weights — sinks to the front of the
tail, sources to the end of the head, otherwise the vertex with the greatest out-weight minus
in-weight — and a redundancy pass offers every cut arc back, ascending, keeping it whenever it
closes no cycle: every arc still cut closes one on its own, so the set is minimal, not proven
minimum, and the row says `exact` 0 ([Fas.hs:103](../../../core/app/CE/Arch/Fas.hs#L103),
[Fas.hs:110](../../../core/app/CE/Arch/Fas.hs#L110), [Fas.hs:130](../../../core/app/CE/Arch/Fas.hs#L130)).
Both roads visit vertices by id and arcs by (from, to), so the cut is a function of the arc table.

**The layers.** With the cut arcs gone the directory graph is acyclic, and a directory's level is
0 when nothing leaves it and one more than the highest level it points at otherwise: the
foundations sit at 0 and every kept arc runs from a higher level down to a lower one
([Layers.hs:22](../../../core/app/CE/Arch/Layers.hs#L22)).

**The clusters and the misplaced files.** The file graph without direction — a pair referencing
each other both ways is one neighbour with both weights ([Dirs.hs:38](../../../core/app/CE/Arch/Dirs.hs#L38))
— is clustered by a deterministic Louvain: every file starts alone, files are visited by id, a
move is weighed by the modularity gain in integer form 2m·k_{i,C} − Σtot(C)·k_i, the greatest gain
wins and the least community on a tie, and a file moves only when that is strictly better than
going back; the communities are folded once into super-vertices and the local move runs again,
then it stops. Cluster ids are renumbered by each cluster's least file
([Louvain.hs:28](../../../core/app/CE/Arch/Louvain.hs#L28), [Louvain.hs:56](../../../core/app/CE/Arch/Louvain.hs#L56)).
A file is misplaced when its cluster holds strictly more of its files in some other directory M
than in the file's own — the directory holding most, the least id on a tie; a tie between M and
the file's own directory is no evidence of where the file belongs, so a ring of one-file
directories names nobody ([Louvain.hs:85](../../../core/app/CE/Arch/Louvain.hs#L85)).

**The impact.** From the focus, a breadth-first walk over the references read backwards: who
references a focus file, who references those, each file at the depth it is first met and the
focus itself at 0. A package reference F → D reads as F referencing every file directly in D,
since a change to any of them may reach F through D; the subdirectories of D are not in it
([Impact.hs:13](../../../core/app/CE/Arch/Impact.hs#L13)).

**The metrics.** Per directory, over the whole graph before any cut: fan-in counts the distinct
directories pointing at it, fan-out the distinct ones it points at, and the instability is
⌊1000 · out ÷ (in + out)⌋ — Martin's I per mille, an integer — or −1 when no arc touches the
directory ([Layers.hs:35](../../../core/app/CE/Arch/Layers.hs#L35), [Cost.hs:44](../../../core/app/CE/Arch/Cost.hs#L44)).

### 3. The measuring side's six rulings

1. The node universe is the structure family's: the deadcode wire of a refreshed index, its
   measured file nodes in path order — so the two families place a file in one tree
   ([face.rs:24](../../../cli/src/arch/face.rs#L24)).
2. The directories are `structure::tree`'s, which numbers a parent before its child; the table
   checks it by name rather than renumber ([tables.rs:129](../../../cli/src/arch/tables.rs#L129)).
3. `lines` is `ce scan`'s count of the file's bytes, not a second one.
4. A file edge is any graph arc between two measured files, a section target counts as its file,
   a package, asset or foreign target never enters `edges`.
5. A package target enters `pkgEdges` at its directory when the tree holds it; nothing is folded
   or dropped for being in the file's own directory — that is the core's rule, applied where the
   battery sees it.
6. A focus path is spelled root-relative with forward slashes; one that names no measured file
   is refused by name before the core is asked.

The document's two derivations are the core's too (`document/1`, since proto 7.8.0): the file
references under each cut arc (the `edges` and `pkgEdges` rows whose two directories are the
arc's), and each cluster's majority directory, read by the rule the misplaced rows are judged by
— the directory holding most of the cluster's files, the least id on a tie — so a cluster and its
misplaced files never name two majorities. This side sends the tables back with each path's place
in string order and puts the paths back ([Document.hs:96](../../../core/app/CE/Arch/Document.hs#L96), [Document.hs:122](../../../core/app/CE/Arch/Document.hs#L122),
[face.rs:79](../../../cli/src/arch/face.rs#L79)).

### 4. The faces and the document

One document, `ce.arch-report/0.1.0`: the counts, one layer row per directory, the cuts with
their file references (a package target written as its directory with a trailing slash), the
clusters with their files, the misplaced files with their directory and their cluster's majority,
the impact rows and the metrics, instability `null` where the core answered −1
([Document.hs:78](../../../core/app/CE/Arch/Document.hs#L78)). The arch reply is consumed
strictly before the document is asked for: the five request counts echo what was sent, the four answer counts tally the tables,
the layers and the metrics carry one row per directory in order, the clusters one per file, every
id is in range and every focus file has its depth-0 row ([wire.rs:124](../../../cli/src/arch/wire.rs#L124),
[wire.rs:175](../../../cli/src/arch/wire.rs#L175)).

- **`ce arch [--impact <path>…] [--format json]`** prints the counts, the layers from the top
  level down, every cut arc with its references indented under it and `exact` or `greedy`, the
  misplaced files, the impact walk when a focus was named and the metrics table; exit 0 with a
  document, 2 when the core could not judge or a path is not a measured file
  ([main_arch.rs:32](../../../cli/src/main_arch.rs#L32), [Lines.hs:16-17](../../../core/app/CE/Arch/Lines.hs#L16)).
- **The MCP tool `architecture`** takes `impact` as a list of root-relative paths and relays the
  same document ([adapters.rs:201](../../../cli/src/mcp/adapters.rs#L201), [tools.rs:234](../../../cli/src/mcp/tools.rs#L234)).
- **The GUI** renders it in the reports hub rather than a thirteenth tab — the header holds twelve
  tabs in one row at the default window, and the hub is where the report-only families live. The
  document nests, so the family registers its own renderer and an `impact` path box; the counts
  still lead as chips ([hub_arch.js:14](../../../gui/ui/hub_arch.js#L14), [reports.js:21](../../../gui/ui/reports.js#L21),
  [commands_query.rs:39](../../../gui/src-tauri/src/commands_query.rs#L39)).

All three go through one library function ([faces.rs:247](../../../cli/src/faces.rs#L247)).

### 5. Gates

The core's battery runs every hand-written case — three cycles, components, groups and reaches,
each asserting exactly the tables it names — and every contract refusal, twenty-two, one leg each
([ArchCases.hs:1-12](../../../core/test/ArchCases.hs#L1), [ArchRefusals.hs:1-7](../../../core/test/ArchRefusals.hs#L1)).
The feedback arc set is held against references written another way: the subset programme
against a search over every arc subset on every directed graph of four vertices and on seeded
graphs of five or six, against a walk over every vertex permutation on seeded graphs of five to
seven vertices and up to twenty-four arcs, both comparing whole cuts under the tie rule; the
greedy road on seeded strongly connected graphs of fifteen or sixteen vertices is held acyclic
and minimal, its weight against the programme's minimum *printed, not asserted* — the heuristic
has no constant bound ([ArchFasProps.hs:1-16](../../../core/test/ArchFasProps.hs#L1),
[ReferenceArch.hs:1-11](../../../core/test/ReferenceArch.hs#L1)). The layers are held against the
cuts on all three families; the clustering is deterministic and never less modular than every
file alone; the impact equals a whole-set fixpoint closure with its round depths; the caps, the
empty request and the counts each have a leg ([ArchProps.hs:5-9](../../../core/test/ArchProps.hs#L5)).
Seven golden pairs pin the wire bytes, the last a fifteen-directory ring on the greedy road
([golden.ndjson:13](../../../contracts/fixtures/arch/golden.ndjson#L13)).

On the measuring side the unit legs fold a synthetic wire into the five tables — dense ids and
earlier parents, two rungs one pair, the self pair and the section fold, the package in and out
of the tree, the focus and its stranger, both caps by name
([tables.rs:72](../../../cli/tests/unit/arch/tables.rs#L72)). The integration legs seed three
directories in one cycle, fifteen in one ring and a Go package import, and hold the CLI's JSON to
the library's byte for byte, one exact and one greedy cut, every kept arc running downhill, the
console reading the same rows, the impact walk and its refused stranger, the MCP relay, and a
core that cannot lay the document out refused by name
([arch_face.rs:1-8](../../../cli/tests/it/arch_face.rs#L1)).

**The frozen self reading.** `contracts/eval/arch-self-v1.json` freezes this repository's
reading — not the working tree, which moves with every commit, but one pinned commit exported
with `git archive` and its `.gitmodules` removed: the counts, every directory's level and metrics,
every cut with its weight and exactness, every misplaced file, and the clusters as a count and
their sizes. The reading moves only when the measurement or the core does, so a drift is a real
one and the leg prints it field by field with the command that re-freezes it
([eval_arch_self.rs:1-16](../../../cli/tests/it/eval_arch_self.rs#L1)).

### 6. Design boundaries

- A component of at most 14 directories is cut by the exact subset programme and a larger one by
  the greedy order; every cut arc carries its road in `exact` (1 exact, 0 greedy), and a greedy cut
  is minimal, not proven minimum.
- A package reference lands on the package's directory and reaches the files directly in it: a Go
  or Java package is one directory, so its subdirectories are other packages.
- The clusters are a function of the whole file set, and a misplaced row is advice about where a
  file's references lie.
- The arcs are the references the language ladders resolve: an unresolved site is no arc here, as
  it is no edge in the graph family.
