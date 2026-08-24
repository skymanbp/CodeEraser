# Graph liveness and dead-code verdicts

[index](../methodology.md) · [← 05 Scoring and the ADR-006 ratchet](05-scoring-and-the-adr-006-ratchet.md) · [→ 07 The three-signal join](07-the-three-signal-join.md)

The `deadcode` family answers one question — *which files does nothing live reach?* — by
building a **reference graph** over dense node indices in Rust and handing the whole graph to
the Haskell core for judgment. No text crosses the wire: node identity **is** the row index
([Graph.hs:28-32](../../../core/app/CE/Graph.hs#L28)). Every number below is a constant in
`CE.Graph.Cost` or a frozen storage code, so the computation is a pure function of the edge
set and the flag column.

### 1. Pipeline

```
walk → sites (grammar tables)  →  ladder (per-language rungs)  →  edge rows (SQLite)
     →  graph_rows  →  dense node ids + containment arcs  →  graph.request
     →  CE.Graph.Build/Dead/Cycles/Position  →  graph.result  →  named verdicts
```

Phase 1 detection is **resolution-free by construction**: which tree-sitter node kinds open a
site, and where the specifier lives, is a frozen table per language
([spec.rs:53-102](../../../cli/src/graph/spec.rs#L53)), so the site universe (the precision denominator)
freezes before any resolver exists ([spec.rs:8-11](../../../cli/src/graph/spec.rs#L8)). Markdown has no
grammar and scans line-wise ([spec.rs:97-99](../../../cli/src/graph/spec.rs#L97)). The ten frozen site
kinds are `import, import_from, export_from, use, mod_decl, link, image, ref_link, ref_def, url`
([store.rs:85-96](../../../cli/src/graph/store.rs#L85)) — positions, not names, so reordering is a
`GRAPH_REV` bump ([store.rs:48](../../../cli/src/graph/store.rs#L48), currently `7`).

### 2. The resolution ladder

A site walks its language's rungs **in order**; the first rung producing *exactly one* in-scope
candidate resolves it, and more than one candidate at a rung is `Unresolved(ambiguous_*)` —
picking a "best" would invent a path ([ladder/mod.rs:1-8](../../../cli/src/graph/ladder/mod.rs#L1)).
`External` (stdlib, registry, `node_modules`) is a **correct terminal answer, not a miss**
(same lines). Every resolved edge stores the rung that answered it
([ladder/mod.rs:41](../../../cli/src/graph/ladder/mod.rs#L41)), which is what makes per-level precision
attributable. The refusal vocabulary is frozen: `Dynamic, AmbiguousPaths, AmbiguousRoot,
AmbiguousWorkspace, AmbiguousExports, Macro, ConfigDepth, OutOfScope, Unsupported`
([ladder/mod.rs:47-58](../../../cli/src/graph/ladder/mod.rs#L47)); a language without rungs must return
`Unsupported`, never a silent skip ([ladder/mod.rs:176-179](../../../cli/src/graph/ladder/mod.rs#L176)).

| Lang | R1 | R2 | R3 | R4 | R5 |
|---|---|---|---|---|---|
| TS/TSX | relative + extension order `ts, tsx, d.ts, mts, cts` ([ts.rs:23](../../../cli/src/graph/ladder/ts.rs#L23), [ts.rs:29](../../../cli/src/graph/ladder/ts.rs#L29)) | ESM `.js`→`.ts` rewrite, only if the TS twin is in scope and the JS twin is absent on disk ([ts.rs:76-84](../../../cli/src/graph/ladder/ts.rs#L76)) | nearest tsconfig `paths`, then `baseUrl` join ([ts.rs:104-118](../../../cli/src/graph/ladder/ts.rs#L104)) | workspace member by `name` + `exports` subpath ([ts.rs:151-189](../../../cli/src/graph/ladder/ts.rs#L151)) | bare specifier in deps or under `node_modules/` ⇒ External ([ts.rs:223-238](../../../cli/src/graph/ladder/ts.rs#L223)) |
| Python | leading-dot relative; *n* dots climb *n−1* levels ([py.rs:37-48](../../../cli/src/graph/ladder/py.rs#L37)) | absolute dotted path over source roots ([py.rs:60-78](../../../cli/src/graph/ladder/py.rs#L60)) | `__init__.py` longest-prefix degradation ([py.rs:109-120](../../../cli/src/graph/ladder/py.rs#L109)) | stdlib table or pyproject dep ⇒ External ([py.rs:123-130](../../../cli/src/graph/ladder/py.rs#L123)) | — (structurally empty: the detector never opens dynamic imports, [py.rs:14-16](../../../cli/src/graph/ladder/py.rs#L14)) |
| Rust | `mod foo;` child lookup, `#[path]` remap wins outright ([rs.rs:76-90](../../../cli/src/graph/ladder/rs.rs#L76)) | `use crate::…` from covering crate roots ([rs_use.rs:69-73](../../../cli/src/graph/ladder/rs_use.rs#L69)) | `self::`/`super::`, inline-`mod` depth consumed before any file climb ([rs_use.rs:74-88](../../../cli/src/graph/ladder/rs_use.rs#L74), [rs_use.rs:100-117](../../../cli/src/graph/ladder/rs_use.rs#L100)) | builtin crates `std, core, alloc, proc_macro, test` ⇒ External; in-scope package descends its tree ([rs_use.rs:14](../../../cli/src/graph/ladder/rs_use.rs#L14), [rs_use.rs:159-192](../../../cli/src/graph/ladder/rs_use.rs#L159)) | single unambiguous top-level `pub use` binds **≤1 hop** to the definition file ([rs_use.rs:124-151](../../../cli/src/graph/ladder/rs_use.rs#L124)) |
| Go | longest in-scope `go.mod` module prefix ([go.rs:44-71](../../../cli/src/graph/ladder/go.rs#L44)) | importer's module `replace` directives ([go.rs:85-110](../../../cli/src/graph/ladder/go.rs#L85)) | stdlib table, or a dotted first segment with no local match ⇒ External ([go.rs:150-156](../../../cli/src/graph/ladder/go.rs#L150)) | — | — |
| Markdown | relative join; a directory holding in-scope files is a package ([md.rs:66-81](../../../cli/src/graph/ladder/md.rs#L66), [md.rs:101-115](../../../cli/src/graph/ladder/md.rs#L101)) | anchor validated against the target's ATX slug set ([md.rs:119-133](../../../cli/src/graph/ladder/md.rs#L119)) | reference-link definition substituted, chain rerun relabeled ([md.rs:137-160](../../../cli/src/graph/ladder/md.rs#L137)) | bare fragment = in-file section claim, taken as written ([md.rs:85-97](../../../cli/src/graph/ladder/md.rs#L85)) | any URI scheme, `/x` or `//x` ⇒ External ([md.rs:45](../../../cli/src/graph/ladder/md.rs#L45), [md.rs:53-58](../../../cli/src/graph/ladder/md.rs#L53)) |
| Haskell | module name dots→slashes under the owning cabal's stanza source roots ([hs.rs:61-77](../../../cli/src/graph/ladder/hs.rs#L61)) | global-package-db table, gated by the owner cabal's `build-depends` ⇒ External ([hs.rs:137-147](../../../cli/src/graph/ladder/hs.rs#L137)) | — | — | — |

Numeric details that are policy, not taste:

- The tsconfig `extends` chain is bounded at **8** hops with a cycle check; exceeding it is
  `config_depth`, never a guess ([roots.rs:29-30](../../../cli/src/graph/roots.rs#L29),
  [roots.rs:49](../../../cli/src/graph/roots.rs#L49)).
- Python source roots are `{repo root, "src"}` plus pyproject-declared dirs
  ([py.rs:134-141](../../../cli/src/graph/ladder/py.rs#L134)); within one root, package-before-module is
  CPython's own finder order and therefore **not** ambiguity — only cross-root disagreement is
  ([py.rs:11-13](../../../cli/src/graph/ladder/py.rs#L11), [py.rs:70-77](../../../cli/src/graph/ladder/py.rs#L70)).
- A Go directory counts as an importable package only while it *directly* holds an in-scope
  non-`_test.go` file ([go.rs:122-140](../../../cli/src/graph/ladder/go.rs#L122)).
- GitHub slugging: lowercase, keep alphanumerics/`_`/`-`, spaces→hyphens, everything else
  dropped ([md.rs:266-276](../../../cli/src/graph/ladder/md.rs#L266)); duplicates take `-N` suffixes in
  document order ([md.rs:240-244](../../../cli/src/graph/ladder/md.rs#L240)). Anything but exactly one
  slug match degrades to a file-level edge, never invents a section
  ([md.rs:126-132](../../../cli/src/graph/ladder/md.rs#L126)).
- The external tables are machine-generated, never hand-typed: CPython 3.13
  `sys.stdlib_module_names` ([py.rs:143-147](../../../cli/src/graph/ladder/py.rs#L143)), Go 1.26.4
  `go list std` minus `internal/`/`vendor/` ([go.rs:158-163](../../../cli/src/graph/ladder/go.rs#L158)),
  and GHC 9.14.1's global db — **43 packages, 1371 modules**
  ([hs_boot.rs:14-15](../../../cli/src/graph/ladder/hs_boot.rs#L14)). A missing name degrades to
  `Unresolved` (precision-safe), visible in the ledger.
- Rust's `ResolvedVia` keeps the **original walk's rung** and records the hop as a separate
  `via_reexport` column, not as a new rung
  ([rs_use.rs:145-148](../../../cli/src/graph/ladder/rs_use.rs#L145),
  [wire.rs:59](../../../cli/src/graph/wire.rs#L59)).

Cross-file staleness is closed at the resolve key rather than by re-sweeping: the only
target-content facts a ladder consults are the Markdown slug set and the Rust `pub use`
surface, and each is folded into a hash that is a resolve-key input
([md.rs:219-226](../../../cli/src/graph/ladder/md.rs#L219),
[rs_reexport.rs:162-177](../../../cli/src/graph/ladder/rs_reexport.rs#L162)). Both hashes are pinned by
a coupling battery asserting `hash(a)==hash(b) ⟺ projection(a)==projection(b)`
([md_tests.rs:12-31](../../../cli/src/graph/ladder/md_tests.rs#L12),
[rs_reexport.rs:189-219](../../../cli/src/graph/ladder/rs_reexport.rs#L189)).

### 3. Edge extraction and node identity

Only in-corpus outcomes become stored rows; `External` and `Unresolved` sites stay
ledger-visible as sites *without* edges ([wire.rs:55-72](../../../cli/src/graph/wire.rs#L55)). Edge kinds
are frozen positions: `EDGE_IMPORT = 0`, `EDGE_DOC_LINK = 1`, `EDGE_DOC_REF = 2`,
`EDGE_ASSET = 3`, `EDGE_CONTAIN = 4` ([wire.rs:23-27](../../../cli/src/graph/wire.rs#L23)); granularity
codes are `GRAN_FILE = 0`, `GRAN_PACKAGE = 1`, `GRAN_SECTION = 2`
([wire.rs:30-32](../../../cli/src/graph/wire.rs#L30)).

Node identity is the pair `(path, unit)` over a `BTreeSet` of every walked file plus every edge
target, so the id assignment is a function of the graph and the wire bytes are shuffle-proof
([nodes.rs:24-50](../../../cli/src/graph/nodes.rs#L24), asserted at
[nodes.rs:111-127](../../../cli/src/graph/nodes.rs#L111)). Package-ness is read from the edge's *stored*
granularity, never inferred from a target's absence — the old absence rule minted image assets
and dangling doc refs as packages ([nodes.rs:20-23](../../../cli/src/graph/nodes.rs#L20),
[nodes.rs:133-148](../../../cli/src/graph/nodes.rs#L133)).

Two transformations happen on the way to the wire:

1. **Asset edges are dropped by kind** — an image reference is not a reference for liveness
   purposes ([deadcode.rs:132](../../../cli/src/graph/deadcode.rs#L139)). An endpoint that is not a node
   is a *named error*, never a panic ([deadcode.rs:134-138](../../../cli/src/graph/deadcode.rs#L141)).
2. **Synthetic containment arcs** are added from each package node to every file under its
   directory, at `rung 1` because containment is a fact, not a resolution mechanism, and must
   survive every rung ceiling ([nodes.rs:61-83](../../../cli/src/graph/nodes.rs#L61)). A repo-root
   package has path `""`, and the naive `format!("{}/", "")` prefix `"/"` matched nothing —
   measured: a root `lib.go` imported by `cmd/main.go` was reported dead
   ([nodes.rs:66-73](../../../cli/src/graph/nodes.rs#L66)).

The whole read runs in **one snapshot transaction**: as three autocommit statements a
convergent writer landing between them could hand the edge query a source file the files query
never saw ([load.rs:70-75](../../../cli/src/graph/load.rs#L70)). `unresolved_sites` is the count of sites
with no edge row ([load.rs:97-102](../../../cli/src/graph/load.rs#L97)) and travels with the report so
the reader sees what the graph refuses to know
([deadcode.rs:22-24](../../../cli/src/graph/deadcode.rs#L23)).

### 4. Boundary contract and caps

`graph.request` carries `nodes: [[lang, kind, flags]]`, `edges: [[src, dst, kind, rung]]`, and
an optional `pos: [idx]`. The core machine-checks, in request order so the message is
deterministic ([Graph.hs:69-85](../../../core/app/CE/Graph.hs#L69)):

- node rows are exactly 3 fields, all `≥ 0` ([Graph.hs:87-94](../../../core/app/CE/Graph.hs#L88));
- edge rows are exactly 4 fields, all `≥ 0`, with `src < n` and `dst < n`
  ([Graph.hs:96-104](../../../core/app/CE/Graph.hs#L110));
- the edge table is **strictly ascending** lexicographically, hence duplicate-free
  ([Graph.hs:74](../../../core/app/CE/Graph.hs#L74), [Wire.hs:44-47](../../../core/app/CE/Wire.hs#L44));
- `pos` indices lie in `[0, n)` and are strictly ascending — which is also the reply *bound*,
  since a repeated-index list would make the reply larger than the request without limit
  ([Graph.hs:76-80](../../../core/app/CE/Graph.hs#L77),
  [Graph.hs:109-112](../../../core/app/CE/Graph.hs#L123)).

Oversize protection is by row count, not bytes (the envelope precheck is relaxed for the
trusted same-machine child): `nodeCap = 131072` and `edgeCap = 524288`
([Cost.hs:22-26](../../../core/app/CE/Graph/Cost.hs#L22)). The sizing anchor is ~20k nodes / ~60k edges
per 100k LOC, so the caps carry roughly 6× headroom
([Cost.hs:15-21](../../../core/app/CE/Graph/Cost.hs#L15)). Over cap the core returns a **well-formed
degraded result** with `dead = []`, `reported = []`, `kept = 0`, `degraded = true`,
`reason = "graph_too_large"` and `fail = true` — a gate that could not judge never passes, said
by the core itself since 2.18.0, and never a truncated graph
([Graph.hs:57-59](../../../core/app/CE/Graph.hs#L57), [Graph.hs:161-186](../../../core/app/CE/Graph.hs#L161)).
The CLI treats a degraded reply as an event, not silence: it lands in the observe feed
([deadcode.rs:265-271](../../../cli/src/graph/deadcode.rs#L277)) and `ce deadcode --check` relays the
core's fail bit ([main_cmds.rs:70-90](../../../cli/src/main_cmds.rs#L70)).

### 5. Kept arcs and liveness

The kept arc set is the rung-filtered, `(src,dst)`-deduplicated edge list — kind multiplicity
between one pair is *not* extra evidence of reference, so indegree counts distinct arcs
([Build.hs:1-7](../../../core/app/CE/Graph/Build.hs#L1), [Build.hs:29-38](../../../core/app/CE/Graph/Build.hs#L29)):

```
arcs  = { (s,d) | [s,d,_kind,rung] ∈ edges,  rung ≤ minRung }
kept  = |arcs|
G     = buildG (0, n-1) arcs        -- all n vertices; isolated ones are exactly the unreferenced
```

`minRung = 5` ([Cost.hs:34-35](../../../core/app/CE/Graph/Cost.hs#L34)). Because the ladder never
guesses — ambiguity becomes `Unresolved` and never crosses the wire — every rung the Rust side
emits (`1..5`) is admitted by default; lowering the constant is the ablation lever that trades
recall for certainty ([Cost.hs:28-33](../../../core/app/CE/Graph/Cost.hs#L28)). At the current value the
filter is a no-op ceiling: the highest rung any ladder emits is 5 (TS R5, Markdown R5), and the
per-ceiling trade is published instead by the `cut` table of the precision instrument
([eval_graph_precision_parts/mod.rs](../../../cli/tests/eval_graph_precision_parts/mod.rs)).

**Entry roots.** A node seeds reachability iff `flags .&. entryMask ≠ 0`
([Dead.hs:17-18](../../../core/app/CE/Graph/Dead.hs#L17)), with `entryMask = 126` = bits 1–6
([Cost.hs:47-48](../../../core/app/CE/Graph/Cost.hs#L47)). Declared bits: 1 main, 2 test, 3 entry-glob,
4 dyn-referenced, 5 doc-entry, 6 `ce:allow(deadcode)`
([Cost.hs:38-42](../../../core/app/CE/Graph/Cost.hs#L38)). Bit 0 (exported) is **deliberately absent** —
exported-ness is the public/private *verdict* axis, so a library's unreferenced API surfaces as
`unref_public` rather than as plain dead or as silence
([Cost.hs:43-46](../../../core/app/CE/Graph/Cost.hs#L43)).

Only file nodes carry entry facts; section and package rows get `0`
([deadcode.rs:177-197](../../../cli/src/graph/deadcode.rs#L177)). Since proto **2.28.0**
(batch-7 slice 3 main body) the node row carries a 4th column of **role facts**, and the
category membership Rust used to fuse into the flags column is decided by the core's
**role table** `roleBits` ([Graph/Cost.hs:89-90](../../../core/app/CE/Graph/Cost.hs#L89)):
a 4-column row's entry bits derive through `deriveFlags`
([Dead.hs:53-55](../../../core/app/CE/Graph/Dead.hs#L53), applied at
[Graph.hs:182-185](../../../core/app/CE/Graph.hs#L182)), the legacy flags column yields, and a
table mixing the two arities refuses by name. The Rust producer measures:

```
role 0  base ∈ {main.rs, main.go, __main__.py, build.rs, Main.hs}   [flags.rs:49-54]
role 1  path starts with src/bin/, examples/, benches/, cmd/        [flags.rs:55-60]
role 2  base ends _test.go | .test.ts | (test_*.py) | == Spec.hs,
        or path starts tests/ or contains /tests/ | /__tests__/     [flags.rs:115-124]
role 3  ce.toml [graph] entry_globs hit: exact path, dir/ prefix,
        or *.ext basename (full glob syntax is not promised)        [flags.rs:128-133]
role 4  base ∈ {README.md, CLAUDE.md}, or docs/**/{index.md, README.md}
role 5  inline `ce:allow(deadcode) -- <why>` claim (a BARE marker
        claims nothing — the docdup exemption discipline)           [flags.rs:102-113]
role 6  a manifest-declared build target: Cargo [lib]/[[bin]] paths
        and conventional targets via crate_roots, cabal main-is
        through each stanza's source roots                          [targets.rs:21-70, 75-101]
```

([flags.rs:19-25](../../../cli/src/graph/deadcode/flags.rs#L19),
[flags.rs:46-85](../../../cli/src/graph/deadcode/flags.rs#L46)). The role→bit landing is the
core's data: roles 0, 1 and 6 all land on bit 1, roles 2/3/4/5 on bits 2/3/5/6. **Role 6 closes
a ledgered defect**: a declared `[[bin]] path` or cabal `main-is` target is a root, where
before only the name conventions were — the discovery is nearest-manifest per walked directory
([targets.rs:75-101](../../../cli/src/graph/deadcode/targets.rs#L75),
[targets.rs:52-66](../../../cli/src/graph/deadcode/targets.rs#L52)). The legacy flags column is still
produced bit-identically to the pre-2.28 semantics
([flags.rs:33-43](../../../cli/src/graph/deadcode/flags.rs#L33)) and retires next minor.
**Honest gaps that remain:** bit 4 (dyn-referenced) has no producer, and bit 0 is never set at
file granularity ([flags.rs:1-11](../../../cli/src/graph/deadcode/flags.rs#L1)). The consequence
of the bit-0 gap is stated in the ladder itself: RG10's `unref_public` class cannot fire until
symbol-level flags land
([hs.rs:26-31](../../../cli/src/graph/ladder/hs.rs#L26)).

Reachability is plain forward closure from the seeds over kept arcs
([Build.hs:41-42](../../../core/app/CE/Graph/Build.hs#L41)):

```
reach = ⋃ { reachable(G, s) | s ∈ entries(entryMask, flags) }
```

`Data.Graph.reachable` includes the seed itself, so an entry node is never judged
([Dead.hs:6-7](../../../core/app/CE/Graph/Dead.hs#L6)).

### 6. SCC handling and the position surface

Every SCC in `Data.Graph` order, members ascending, is a deterministic function of the sorted
arc set; **singletons are included** so the id space covers every vertex
([Build.hs:44-49](../../../core/app/CE/Graph/Build.hs#L44)). The cycle *report* applies the floor
downstream: an SCC is reported iff `|members| ≥ sccFloor`
([Cycles.hs:11-16](../../../core/app/CE/Graph/Cycles.hs#L11)) with `sccFloor = 2`, i.e. only true
multi-node cycles — a self-loop singleton stays unreported, and widening to self-loops is a
knob change the dead-knob test can see, not a code change
([Cost.hs:50-55](../../../core/app/CE/Graph/Cost.hs#L50)). Cycle ids are positions in the full SCC list,
so they agree with `Position`'s `sccId` by construction
([Cycles.hs:1-4](../../../core/app/CE/Graph/Cycles.hs#L1)).

**Cycles are reported, never judged** — the verdict pass does not read this list
([Cycles.hs:3-4](../../../core/app/CE/Graph/Cycles.hs#L3)). A cyclic island with no entry seed is
therefore dead by reachability alone, without a special case.

The per-node join surface, computed only for the requested `pos` indices, is
`[idx, indeg, outdeg, sccId, sccSize, reachIn]`
([Position.hs:1-3](../../../core/app/CE/Graph/Position.hs#L1),
[Position.hs:13-32](../../../core/app/CE/Graph/Position.hs#L13)); degrees count distinct kept arcs, and
`reachIn` is `fromEnum (i ∈ reach)`. A non-degraded reply **must** answer every requested index
— a short `pos` table would silently starve the M5-3 join, so the CLI refuses it
([deadcode.rs:183-186](../../../cli/src/graph/deadcode.rs#L183)).

### 7. The four-way verdict

Dead splits along **two independent axes** — indegree × reachability — with public structurally
separated so an exported-but-unreferenced API can never collapse into plain dead
([Dead.hs:1-7](../../../core/app/CE/Graph/Dead.hs#L1)):

```
public     = testBit flags 0
referenced = i ∈ { d | (_,d) ∈ arcs }        -- indegree ≥ 1 over kept arcs
judged     = i ∉ reach
```

The code assignment is a **total lookup table**, not arithmetic — the ADR-008 lattice-table
form, so a reordered row is a data diff a brute-force property test disagrees with on every
fixture it touches ([Dead.hs:20-31](../../../core/app/CE/Graph/Dead.hs#L20)):

| public | referenced | code | name |
|---|---|---|---|
| false | false | 1 | `unref_private` |
| true | false | 2 | `unref_public` |
| false | true | 3 | `unreach_private` |
| true | true | 4 | `unreach_public` |

Equivalent to `1 + public + 2*referenced` ([Dead.hs:5-6](../../../core/app/CE/Graph/Dead.hs#L5)); the
table is the authority and the arithmetic is the mnemonic. The result is `[(i, code)]` ascending
over every node outside `reach` ([Dead.hs:33-39](../../../core/app/CE/Graph/Dead.hs#L33)).

Naming back on the Rust side is by position — `VERDICT_NAMES[code - 1]`
([deadcode.rs:39-44](../../../cli/src/graph/deadcode.rs#L39),
[deadcode.rs:220-226](../../../cli/src/graph/deadcode.rs#L220)) — and a code past the four this side
knows is treated as wire-version skew, not a panic (same lines). The `why` string is a two-way
split on the same axis: codes 1–2 read *"no kept in-edge and no entry flag"*, codes 3–4 read
*"referenced only from dead code; no entry flag"*
([deadcode.rs:230-234](../../../cli/src/graph/deadcode.rs#L230)).

**The reporting firewall.** Only file nodes enter `dead`; section and package verdicts go to a
separate `reported` table and are never called dead — aggregates are not code entities. Since
proto 2.18.0 (batch-7 slice 4) the split is the CORE's: the reply partitions its verdicts on
the node kind column it always received ([Graph.hs:127-156](../../../core/app/CE/Graph.hs#L127),
`granFile` at [Cost.hs:55-66](../../../core/app/CE/Graph/Cost.hs#L55)), and carries the additive
`fail` bit naming the zero-tolerance gate. The Rust side keeps the split as a boundary
contract, because the failing table is what licenses `ce erase`'s dead-file rows: an aggregate
arriving in `dead` refuses as wire skew, never a directory erase
([deadcode.rs:217-225](../../../cli/src/graph/deadcode.rs#L217)); against a pre-2.18 core the
absent bit falls back to the client's old conjunction, byte-identical
([deadcode.rs:249-253](../../../cli/src/graph/deadcode.rs#L249)). Both lists, the counts, and
`unresolved_sites` ship in the JSON document
([report.rs:75-89](../../../cli/src/report.rs#L75)). The design's *"no entry rule ⇒ every doc trivially
dies"* stance is deliberate: an unlinked doc **is** reported
([deadcode.rs:13-15](../../../cli/src/graph/deadcode.rs#L13)).

### 8. Acceptance

The M5-2 row sets four criteria: import-edge precision **≥ 0.90** on a 100-site manual audit
spanning the five launch languages; `unreferenced_public` as its own report class, not folded
into dead; every finding in this repository dispositioned; and the core's judgment-invariant
property battery in CI
([DEVELOPMENT_PLAN.md:268](../../DEVELOPMENT_PLAN.md#L268)). The gate is coded at `0.90`,
applied overall and per corpus **where the in-corpus ground-truth denominator reaches 5**
([eval_graph_precision.rs:85](../../../cli/tests/eval_graph_precision.rs#L85),
[eval_graph_precision.rs:88-96](../../../cli/tests/eval_graph_precision.rs#L88),
[precision.rs:38-48](../../../cli/tests/eval_support/precision.rs#L38)); precision is
`correct / (correct + wrong)` over answered rows only
([precision.rs:43](../../../cli/tests/eval_support/precision.rs#L43),
[precision.rs:65](../../../cli/tests/eval_support/precision.rs#L65)).

Frozen results across the five pinned corpora (100 judged sites total):

| corpus | correct | wrong | in-corpus GT | precision | gated? |
|---|---|---|---|---|---|
| zod | 22 | 0 | 22 | 1.0 ([zod:509-513](../../../contracts/eval/graph-precision-zod-v1.json#L509)) | yes |
| ripgrep | 9 | 1 | 10 | 0.9 ([ripgrep:434-440](../../../contracts/eval/graph-precision-ripgrep-v1.json#L434)) | yes |
| self | 4 | 0 | 4 | 1.0 ([self:333-337](../../../contracts/eval/graph-precision-v1.json#L333)) | no (denom < 5) |
| requests | 2 | 1 | 3 | 0.667 ([requests:284-291](../../../contracts/eval/graph-precision-requests-v1.json#L284)) | no (denom < 5) |
| cobra | 1 | 0 | 1 | 1.0 ([cobra:237-241](../../../contracts/eval/graph-precision-cobra-v1.json#L237)) | no (denom < 5) |

Aggregating by the same formula gives **38 / 40 = 0.95** overall, above the 0.90 contract. The
sample universe and the audit ground truth were both frozen *before any resolver existed*, so
the resolver cannot choose its own denominator
([eval_graph_precision.rs:1-6](../../../cli/tests/eval_graph_precision.rs#L1)); the sample is 100 sites
with a per-language floor of 15 ([eval_graph_precision.rs:93-94](../../../cli/tests/eval_graph_precision.rs#L93)).
The "all findings dispositioned" criterion, honored by discipline at M5-2, is now a gate:
`ce deadcode --check` exits non-zero on any dead file
([main_cmds.rs:87-98](../../../cli/src/main_cmds.rs#L87)).
