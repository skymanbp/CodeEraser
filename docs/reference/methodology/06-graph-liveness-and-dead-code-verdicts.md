# Graph liveness and dead-code verdicts

[index](../methodology.md) · [← 05 Scoring and the ADR-006 ratchet](05-scoring-and-the-adr-006-ratchet.md) · [→ 07 The three-signal join](07-the-three-signal-join.md)

The `deadcode` family answers one question — *which files does nothing live reach?* — by
building a **reference graph** over dense node indices in Rust and handing the whole graph to
the Haskell core for judgment. No text crosses the judgment wire (`graph/1`) — paths travel only
to the layout step `document/1` — and node identity **is** the row index
([Contract.hs:27-33](../../../core/app/CE/Graph/Contract.hs#L27)). Every number below is a constant in
`CE.Graph.Cost` or a frozen storage code, so the computation is a pure function of the edge
set and the flag column. The files it does not reach form the **dead set** (§7); what a face says
about each of them is that it is **unreferenced** — no reference was found, which is not the claim
that the file is unused. Since plan v2.35 the dead set is grouped into roots and dependents, each root
compared against the point `ce-baseline.json` records, and by default an unreferenced file is listed and fails
nothing; a project that wants a gate declares one (§10).

### 1. Pipeline

```
walk → sites (grammar tables)  →  ladder (per-language rungs)  →  edge rows (SQLite)
     →  graph_rows  →  dense node ids + containment arcs  →  graph.request
     →  CE.Graph.Build/Dead/Cycles/Position  →  graph.result  →  named verdicts
```

Phase 1 detection is **resolution-free by construction**: which tree-sitter node kinds open a
site, and where the specifier lives, is a frozen table per language — the core's since plan v2.32 step 2, read by
[spec.rs:100-105](../../../cli/src/graph/spec.rs#L100) (Python's at [Python.hs:17-25](../../../core/app/CE/Lang/Python.hs#L17)), so the site universe (the precision denominator)
freezes before any resolver exists ([spec.rs:10-13](../../../cli/src/graph/spec.rs#L10)). Markdown has no
grammar and scans line-wise ([spec.rs:98-99](../../../cli/src/graph/spec.rs#L98)); HTML's sites are
(element, attribute) pairs read off each tag in a pass of their own — a `<link>` reads its `rel` to tell a
stylesheet, icon or preload (`link_asset`) from a page (`href`), and a `srcset` opens one site per candidate
URL ([sites/html.rs:19-45](../../../cli/src/graph/sites/html.rs#L19),
[sites/html.rs:113-130](../../../cli/src/graph/sites/html.rs#L113)). The twenty-four frozen site
kinds are `import, import_from, export_from, use, mod_decl, link, image, ref_link, ref_def, url, export_star,
include, import_star, type_ref, require, load, source, library, href, src, srcset, action, link_asset,
dynamic` — the
definition package's `store` table since plan v2.32 step 4
([Graph.hs:17-20](../../../core/app/CE/Lang/Common/Graph.hs#L17), read at
[store.rs:229-231](../../../cli/src/graph/store.rs#L229)) — positions, not names, so reordering is a
`GRAPH_REV` bump ([store.rs:189](../../../cli/src/graph/store.rs#L189), currently <!--ce:ver:graph_rev#digits-->`28`<!--/ce-->); `export_star` (a TS
`export *` / `export * as ns` statement) was split out of `export_from` at rev 13 because the mounts table
reads it as a re-export target. Rev 14 (plan v2.17 L round step 8) added no kind: a Python `from
__future__` opens an `import_from` site on the literal module name and a TS `import x = require("…")`
an `import` site off its require clause ([spec.rs:43-49](../../../cli/src/graph/spec.rs#L43),
[Python.hs:23-24](../../../core/app/CE/Lang/Python.hs#L23), [TypeScript.hs:16](../../../core/app/CE/Lang/TypeScript.hs#L16)); the rev paid for the stored-fact and ladder changes.
Rev 16 (plan v2.30, one release for all of it) added C / C++'s `include`, Java's `import_star` and `type_ref`,
Lua's `require` and `load`, R's `source` and `library`, and HTML's `href`, `src`, `srcset`, `action` and
`link_asset` (step 5; files the index never held, so no stored row moved); a Java single-type import keeps the
`import` label. Rev 25 (lane v235_graphtrust) added `dynamic`: a load call whose target argument is no
literal — Lua `require` / `dofile`, R `source` / `library`, Python `importlib.import_module` / `__import__`,
TS `require` / `import()` — opens a `dynamic` site, which the resolve family refuses as `Dynamic` before any
rung ([call_site.rs:9-12](../../../cli/src/scan/call_site.rs#L9), [Resolve.hs:135](../../../core/app/CE/Resolve.hs#L135)), so its
language carries an unresolved site and cannot vouch for its dead verdicts; a literal argument opens the
language's ordinary site (`import` for Python and TS) and walks the ladder. Rev 26 added the site's
ledger column (`sites.ledger`, §3). Rev 28 (lane v235_licence, part 4C) added no kind: stored edges of
Markdown and HTML sources changed, a link into another file's confirmed section drawing one more edge,
to that file (§2, the `also` paragraph).

### 2. The resolution ladder

A site walks its language's rungs **in order**; the first rung producing *exactly one* in-scope
candidate resolves it, and more than one candidate at a rung is `Unresolved(ambiguous_*)` —
picking a "best" would invent a path ([ladder/mod.rs:1-8](../../../cli/src/graph/ladder/mod.rs#L1)).
`External` (stdlib, registry, `node_modules`) is a **correct terminal answer, not a miss**
(same lines). Every resolved edge stores the rung that answered it
([ladder/outcome.rs:8](../../../cli/src/graph/ladder/outcome.rs#L8)), which is what makes per-level precision
attributable. The refusal vocabulary is frozen: `Dynamic, AmbiguousPaths, AmbiguousRoot,
AmbiguousWorkspace, AmbiguousExports, Macro, ConfigDepth, OutOfScope, Unsupported, Empty,
OwnUnit` (`Empty` = a degenerate specifier such as `import ""`, kept as a site and refused by the
dispatcher before any rung could read the empty string as a name — O60, L round step #15;
`OwnUnit` = a Java name declared in the referencing file's own compilation unit, no edge drawn,
plan v2.30 step 5) ([ladder/outcome.rs:14-34](../../../cli/src/graph/ladder/outcome.rs#L14)); a language without rungs must return
`Unsupported`, never a silent skip ([ladder/mod.rs:223-227](../../../cli/src/graph/ladder/mod.rs#L223)).

Every language's rungs below run in the core's `resolve/1` family: the Python, Go, C / C++ and
Lua rungs since plan v2.33 wave W2a, then through W2-text the R rungs (stage B), Java (C),
Haskell (D), TypeScript / TSX (E), Rust (F), Markdown (G) and HTML (H); since W2-text so do the
readers of the configuration they search — the tsconfig chains and package.json files, the
Cargo.toml files, go.mod, R's `DESCRIPTION`, the .cabal files, the root `pyproject.toml`'s keys,
the compile databases with their response and flag files — with every
walked Java file's header as this side's lexer read it
([Resolve.hs:1-24](../../../core/app/CE/Resolve.hs#L1)): this side reads the tree, detects the sites
and reads the files, sends what it read as text, reads any response file or fact the core names as
wanted — one round per resolve level, every fact a level can name asked together, each Rust site's
row and its directory's Cargo.toml probes carried in the first request — over the one core session
the command shares with every other family, and maps each reply row back to the outcome the other
rungs return
([resolve/mod.rs:1-12](../../../cli/src/graph/resolve/mod.rs#L1),
[request.rs:1-10](../../../cli/src/graph/resolve/request.rs#L1)). A sweep whose core cannot answer
stores those languages' sites unresolved, owes their files to the next run, and every face
that reads edges refuses by name until a run settles them
([owed.rs:1-10](../../../cli/src/graph/owed.rs#L1)).

| Lang | R1 | R2 | R3 | R4 | R5 |
|---|---|---|---|---|---|
| TS/TSX | relative + extension order `ts, tsx, d.ts, mts, cts` ([Ts.hs:51-53](../../../core/app/CE/Resolve/Ts.hs#L51), [Ts.hs:77-83](../../../core/app/CE/Resolve/Ts.hs#L77)) | ESM `.js`→`.ts` rewrite, only if the TS twin is in scope and the JS twin is absent on disk ([Ts.hs:89-101](../../../core/app/CE/Resolve/Ts.hs#L89)) | nearest tsconfig `paths`, then `baseUrl` join ([Ts.hs:103-121](../../../core/app/CE/Resolve/Ts.hs#L103)); its `extends` chain read as tsc reads it — an array's last entry is the nearest base, a diamond is no cycle, no hop cap, and a cycle, a missing base or a non-string entry breaks the chain to `config_depth` ([TsConfig.hs:86-93](../../../core/app/CE/Resolve/TsConfig.hs#L86), [TsConfig.hs:46-49](../../../core/app/CE/Resolve/TsConfig.hs#L46)) | workspace member by `name` + `exports` subpath — the exact key, else the single-`*` pattern whose longest prefix encloses it, a `null` entry exporting nothing ([Ts.hs:135-157](../../../core/app/CE/Resolve/Ts.hs#L135), [Ts.hs:159-180](../../../core/app/CE/Resolve/Ts.hs#L159)) | a Node builtin, bare or `node:`-prefixed, ⇒ External before any package is read ([Ts.hs:210-215](../../../core/app/CE/Resolve/Ts.hs#L210), [Ts.hs:69-75](../../../core/app/CE/Resolve/Ts.hs#L69)); a bare specifier in deps or under a `node_modules/` of the importer's directory or any ancestor ⇒ External ([Ts.hs:190-198](../../../core/app/CE/Resolve/Ts.hs#L190)) |
| Python | leading-dot relative; *n* dots climb *n−1* levels ([Py.hs:49-64](../../../core/app/CE/Resolve/Py.hs#L49)) | absolute dotted path over source roots ([Py.hs:71-89](../../../core/app/CE/Resolve/Py.hs#L71)) | `__init__.py` longest-prefix degradation ([Py.hs:91-100](../../../core/app/CE/Resolve/Py.hs#L91)) | stdlib table, `__future__` by name (a real module the public-names table omits, step 8), or pyproject dep ⇒ External ([Py.hs:113-120](../../../core/app/CE/Resolve/Py.hs#L113)) | — (structurally empty: a computed `import_module` / `__import__` opens a `dynamic` site, refused before the ladder, [Py.hs:15-16](../../../core/app/CE/Resolve/Py.hs#L15)) |
| Rust | `mod foo;` child lookup, `#[path]` remap wins outright ([Rs.hs:73-84](../../../core/app/CE/Resolve/Rs.hs#L73)); crate roots include Cargo's `<name>/main.rs` auto-discovery form since step 8 ([Cargo.hs:78-85](../../../core/app/CE/Resolve/Cargo.hs#L78)) | `use crate::…` from covering crate roots ([Rs.hs:129](../../../core/app/CE/Resolve/Rs.hs#L129)) — a spec a hand fold cut mid-path is read whole from the `use` on its line, never guessed shallow ([Rs.hs:98-107](../../../core/app/CE/Resolve/Rs.hs#L98)), and a group with no path before its brace opens one site per entry ([sites.rs:235-238](../../../cli/src/graph/sites.rs#L235)); a lib+bin package's two root terminals are settled by the root whose top level defines or imports the next segment, neither or both still refuse ([Rs.hs:147-155](../../../core/app/CE/Resolve/Rs.hs#L147)) | `self::`/`super::`, inline-`mod` depth consumed before any file climb ([Rs.hs:130-136](../../../core/app/CE/Resolve/Rs.hs#L130), [Rs.hs:178-188](../../../core/app/CE/Resolve/Rs.hs#L178)); a bare head DECLARED as a module in the site's own namespace is read before any crate name — uniform paths, step 8 ([Rs.hs:164-174](../../../core/app/CE/Resolve/Rs.hs#L164)) | builtin crates `std, core, alloc, proc_macro, test` ⇒ External; in-scope package descends its tree ([Ladder2.hs:103-106](../../../core/app/CE/Lang/Common/Ladder2.hs#L103), [Rs.hs:193-207](../../../core/app/CE/Resolve/Rs.hs#L193)) | single unambiguous top-level `pub use` binds **≤1 hop** to the definition file ([Rs.hs:216-227](../../../core/app/CE/Resolve/Rs.hs#L216)); a uniform-path facade `pub use source::Thing` binds too, its hop reading the facade's own `mod source;`; a `pub use p::*` glob is followed when exactly one glob's module exports the name — a pub item of its own or a pub use, never a glob inside it ([Rs.hs:251-261](../../../core/app/CE/Resolve/Rs.hs#L251), [RsSurface.hs:95-98](../../../core/app/CE/Resolve/RsSurface.hs#L95)), and a `pub extern crate x [as y]` binds the crate root under the bound name ([rs_surface.rs:42-45](../../../cli/src/graph/ladder/rs_surface.rs#L42)) |
| Go | longest in-scope `go.mod` module prefix ([Go.hs:71-90](../../../core/app/CE/Resolve/Go.hs#L71)) | importer's module `replace` directives ([Go.hs:95-111](../../../core/app/CE/Resolve/Go.hs#L95)) | stdlib table, or a dotted first segment with no local match ⇒ External ([Go.hs:127-131](../../../core/app/CE/Resolve/Go.hs#L127)) | — | — |
| Markdown | relative join, the path percent-decoded after the `#` split; a directory holding in-scope files is a package ([Md.hs:80-89](../../../core/app/CE/Resolve/Md.hs#L80), [Md.hs:113-119](../../../core/app/CE/Resolve/Md.hs#L113)); a walked asset the index holds no parse of (an image, a data file) is a file-level edge, its node standing as an asset ([Md.hs:97-103](../../../core/app/CE/Resolve/Md.hs#L97)) | anchor, percent-decoded, validated against the target's anchor set — rendered-text ATX and setext slugs plus raw-HTML anchor ids ([Md.hs:121-126](../../../core/app/CE/Resolve/Md.hs#L121), [md_slug.rs:42-57](../../../cli/src/graph/ladder/md_slug.rs#L42)); a confirmed section of another file also references that file (an `also` row, below) | reference-link definition substituted, chain rerun relabeled ([Md.hs:53-57](../../../core/app/CE/Resolve/Md.hs#L53), [Md.hs:128-140](../../../core/app/CE/Resolve/Md.hs#L128)) | bare fragment = in-file section claim, taken as written ([Md.hs:107-111](../../../core/app/CE/Resolve/Md.hs#L107)) | any URI scheme or `//x` ⇒ External, a site-root `/x` ⇒ Unresolved(OutOfScope) ([Md.hs:71](../../../core/app/CE/Resolve/Md.hs#L71), [Md.hs:82-83](../../../core/app/CE/Resolve/Md.hs#L82), [Url.hs:16](../../../core/app/CE/Resolve/Url.hs#L16)) |
| Haskell | module name dots→slashes under the owning cabal's stanza source roots ([Hs.hs:50-56](../../../core/app/CE/Resolve/Hs.hs#L50)) — a stanza's roots include the `common` blocks it `import:`s ([CabalWalk.hs:170-175](../../../core/app/CE/Resolve/CabalWalk.hs#L170)), and an `import {-# SOURCE #-} M` answers `M.hs` like any import ([Hs.hs:20-22](../../../core/app/CE/Resolve/Hs.hs#L20)) | a module an in-corpus package exposes — the package a PackageImports spec names (`import "pkg" M`, the spec keeping the quoted package, [Hs.hs:71](../../../core/app/CE/Resolve/Hs.hs#L71)), else any package the owner's `build-depends` declares, the owner's own package among them (cabal: a component listing its own package in `build-depends` depends on that package's library and sees its exposed modules; release 1.10.0) — under that package's library roots; hidden `other-modules` stay out; two such packages ⇒ ambiguous_workspace ([Hs.hs:11-16](../../../core/app/CE/Resolve/Hs.hs#L11), [Hs.hs:110-120](../../../core/app/CE/Resolve/Hs.hs#L110)) | global-package-db table, gated by the owner cabal's `build-depends` ⇒ External ([Hs.hs:124-129](../../../core/app/CE/Resolve/Hs.hs#L124)) | — | — |
| C / C++ | the including file's own directory, quoted form only — unless every chain compiling the file passes `-I-` ([C.hs:7-8](../../../core/app/CE/Resolve/C.hs#L7), [C.hs:50](../../../core/app/CE/Resolve/C.hs#L50)) | the declared `[graph.search_roots] c` directories (C++ shares the key); two holding two files ⇒ ambiguous_root ([C.hs:9-11](../../../core/app/CE/Resolve/C.hs#L9), [C.hs:40](../../../core/app/CE/Resolve/C.hs#L40)) | the compile databases clangd would find — `compile_commands.json`, `build/compile_commands.json`, `compile_flags.txt`, probed from each C-family file's directory upward whatever the ignore rules say ([compdb_find.rs:1-14](../../../cli/src/graph/compdb_find.rs#L1), [compdb_find.rs:25-29](../../../cli/src/graph/compdb_find.rs#L25)); each entry's `command` read as clang reads it ([Cmdline.hs:1-19](../../../core/app/CE/Resolve/Cmdline.hs#L1)) and its flags sorted into the preprocessor's classes — `-iquote` before `-I` before the system class, `-I-` and the prefix families as GCC reads them ([Flags.hs:1-34](../../../core/app/CE/Resolve/Flags.hs#L1), [Flags.hs:48-58](../../../core/app/CE/Resolve/Flags.hs#L48)); a file is searched along every chain it compiles under — its own entries, else the chains of the translation units whose include closure reaches it, else the nearest `compile_flags.txt` ([CIndex.hs:1-15](../../../core/app/CE/Resolve/CIndex.hs#L1), [CIndex.hs:170-175](../../../core/app/CE/Resolve/CIndex.hs#L170)) — first hit per chain in class order ([CIndex.hs:205-218](../../../core/app/CE/Resolve/CIndex.hs#L205)); one file ⇒ Resolved, two ⇒ ambiguous_root ([C.hs:12-15](../../../core/app/CE/Resolve/C.hs#L12), [C.hs:41](../../../core/app/CE/Resolve/C.hs#L41)) | the `include` directory beside the including file's own directory or beside any ancestor of it, the root among them, asked for a file no compile chain reaches — both forms; two such directories holding two files ⇒ ambiguous_root; a directory a build script alone declares is never read off the script ([C.hs:16-18](../../../core/app/CE/Resolve/C.hs#L16), [C.hs:42](../../../core/app/CE/Resolve/C.hs#L42)) | `<x>` with no hit ⇒ External, `"x"` with no hit ⇒ Unresolved(OutOfScope); never a basename search of the tree ([C.hs:19-22](../../../core/app/CE/Resolve/C.hs#L19)) |
| Java | `import a.b.C` ⇒ the file that DECLARES `package a.b` and a top-level type `C` — the index is what each header declares, never where the file sits, and a second top-level type in a file of another name counts ([Java.hs:5-10](../../../core/app/CE/Resolve/Java.hs#L5), [JavaPick.hs:35-58](../../../core/app/CE/Resolve/JavaPick.hs#L35), [java_types.rs:1-6](../../../cli/src/graph/ladder/java_types.rs#L1)); an import a hand fold cut is completed from the header's reading of the line ([Java.hs:74-87](../../../core/app/CE/Resolve/Java.hs#L74)) | a shorter prefix naming such a file (a nested type, a static member), `a.b.*` ⇒ the package's directory, `a.b.C.*` ⇒ `C.java` ([JavaAt.hs:24-32](../../../core/app/CE/Resolve/JavaAt.hs#L24), [JavaAt.hs:34-44](../../../core/app/CE/Resolve/JavaAt.hs#L34)) | `type_ref` in the JLS 6.4.1 order: a member type the class enclosing the site inherits from a supertype another walked file declares — enclosing types innermost first, the nearest supertype level declaring it wins, two supertypes each declaring one ⇒ ambiguous_paths ([JavaInherit.hs:1-15](../../../core/app/CE/Resolve/JavaInherit.hs#L1), [Java.hs:89-96](../../../core/app/CE/Resolve/Java.hs#L89)); then the file's own single-type import, the name's file in its own package (own directory first), then the star-imported packages; a qualified name is whatever its head names, else a fully qualified name ([JavaAt.hs:46-63](../../../core/app/CE/Resolve/JavaAt.hs#L46)) | a name the JDK answers — a package prefix it exports, a `java.lang` type, a simple name under JDK-only star imports ⇒ External ([JavaPick.hs:108-116](../../../core/app/CE/Resolve/JavaPick.hs#L108), [Ladder.hs:9-21](../../../core/app/CE/Lang/Common/Ladder.hs#L9)) | — (four rungs; settling rules across them: a `main` source set sees no test code and a split package answers the importer's own part ([JavaPick.hs:60-74](../../../core/app/CE/Resolve/JavaPick.hs#L60)), and a name the file declares itself ⇒ own_unit ([JavaPick.hs:76-81](../../../core/app/CE/Resolve/JavaPick.hs#L76))) |
| Lua | `require "a.b"` ⇒ `a/b.lua` then `a/b/init.lua` under every search directory — the root, `src`, `lua`, the declared `[graph.search_roots] lua`, the requiring file's own directory and every `package.path` template the tree's own files assign; two directories answering two files ⇒ ambiguous_root ([Lua.hs:52-65](../../../core/app/CE/Resolve/Lua.hs#L52), [lua_path.rs:36-39](../../../cli/src/graph/ladder/lua_path.rs#L36)) | `dofile` / `loadfile` ⇒ the path beside the loading file, then under the root; first hit ([Lua.hs:83-86](../../../core/app/CE/Resolve/Lua.hs#L83)) | a standard-library or LuaJIT built-in name ⇒ External, whatever files the tree holds ([Ladder2.hs:89-98](../../../core/app/CE/Lang/Common/Ladder2.hs#L89), [Lua.hs:47-50](../../../core/app/CE/Resolve/Lua.hs#L47)) | — | — |
| R | `source("x.R")` ⇒ beside the sourcing file, then under the root, then the declared `[graph.search_roots] r` (two answering two files ⇒ ambiguous_root) ([R.hs:34-39](../../../core/app/CE/Resolve/R.hs#L34)) | `library` / `requireNamespace` / `pkg::` ⇒ the directory of the in-scope `DESCRIPTION` whose `Package:` names it — a package node contains its declared code only ([R.hs:43-47](../../../core/app/CE/Resolve/R.hs#L43), [Description.hs:38-43](../../../core/app/CE/Resolve/Description.hs#L38), [Description.hs:73-78](../../../core/app/CE/Resolve/Description.hs#L73)) | a package no in-scope `DESCRIPTION` declares ⇒ External — base R, CRAN and Bioconductor live outside the corpus by construction, so no name table is needed ([R.hs:18-20](../../../core/app/CE/Resolve/R.hs#L18)); a `source` of a URL ⇒ External here too ([R.hs:36](../../../core/app/CE/Resolve/R.hs#L36)) | — | — |
| HTML | a relative reference joined onto the document's directory — its `<base href>` first ([Html.hs:94-98](../../../core/app/CE/Resolve/Html.hs#L94), [Html.hs:104-112](../../../core/app/CE/Resolve/Html.hs#L104)) | a root-relative `/x` under the declared `[graph.search_roots] html` roots (two holding it ⇒ ambiguous_root), else under the deployment root the page's own served URL derives — canonical, `og:url`, its own-language `hreflang` ([Html.hs:119-128](../../../core/app/CE/Resolve/Html.hs#L119), [HtmlHead.hs:49-57](../../../core/app/CE/Resolve/HtmlHead.hs#L49)); a fragment on any found page is validated against that page's `id` set (a Markdown target's slug set): one match ⇒ the section, else the file, reported at this rung ([Html.hs:159-167](../../../core/app/CE/Resolve/Html.hs#L159), [html_head.rs:70-75](../../../cli/src/graph/ladder/html_head.rs#L70)); a confirmed section of another page also references that page (an `also` row, below) | with neither, under the one ancestor directory of the page holding the path; a directory serves `index.html` then `index.htm` ([Html.hs:11-13](../../../core/app/CE/Resolve/Html.hs#L11), [Html.hs:140-145](../../../core/app/CE/Resolve/Html.hs#L140)) | a bare fragment names a section of the page itself, as written — nothing to validate against ([Html.hs:15-16](../../../core/app/CE/Resolve/Html.hs#L15), [Md.hs:108-111](../../../core/app/CE/Resolve/Md.hs#L108)) | a same-host absolute URL is its root-relative path; any other host or scheme ⇒ External; the query drops, character and percent escapes decode, an empty `href` / `action` is the page and an empty `src` / `srcset` / asset `<link>` fetches nothing (ledger `empty`) ([Html.hs:62-70](../../../core/app/CE/Resolve/Html.hs#L62), [HtmlHead.hs:100-103](../../../core/app/CE/Resolve/HtmlHead.hs#L100)) |

Numeric details that are policy, not taste:

- The tsconfig `extends` chain has no hop cap — TypeScript sets none — only a cycle check on the
  branch being walked; a cycle, a missing base or an entry that is no string is `config_depth`,
  never a guess ([TsConfig.hs:86-93](../../../core/app/CE/Resolve/TsConfig.hs#L86),
  [TsConfig.hs:120-124](../../../core/app/CE/Resolve/TsConfig.hs#L120)). The chain's every file, and the
  presence of a `node_modules/` under any ancestor of a TS file, are resolve-key inputs
  ([keys.rs:91-94](../../../cli/src/graph/keys.rs#L91)).
- Python source roots are `{repo root, "src"}` plus pyproject-declared dirs
  ([Py.hs:122-125](../../../core/app/CE/Resolve/Py.hs#L122)); within one root, package-before-module is
  CPython's own finder order and therefore **not** ambiguity — only cross-root disagreement is
  ([Py.hs:12-14](../../../core/app/CE/Resolve/Py.hs#L12), [Py.hs:81-89](../../../core/app/CE/Resolve/Py.hs#L81)).
- A Python site's answer is not all CPython executes for it (proto 9.1.0): beside it ride `also` rows —
  every ancestor package `__init__.py` on the dotted path under the frame the answer was found in, every
  imported name that is a submodule (`from pkg import sub`, package before module, under every frame; two
  roots holding it both draw), and for a relative import the importer's own contiguous package chain; a
  name that misses is an attribute and an absent `__init__.py` a namespace package, both quiet
  ([PyAlso.hs:1-24](../../../core/app/CE/Resolve/PyAlso.hs#L1)). They are edges every graph reader follows,
  stored with `edges.also` = 1 so the unresolved count still reads each site's own answer alone
  ([load.rs:60-69](../../../cli/src/graph/load.rs#L60)). A star import (`from pkg import *`, no names of its
  own) whose answer is a package `__init__.py` draws each name that file's literal `__all__` bindings and
  edits list, as `from pkg import n` would — the walk reads the list (`py.all`,
  [py_all.rs:1-7](../../../cli/src/graph/ladder/py_all.rs#L1)); when any binding or edit of `__all__` there is one
  no literal reads, the literal names still draw and the site carries one more unresolved ledger row
  (`opaque`, [PyAlso.hs:18-24](../../../core/app/CE/Resolve/PyAlso.hs#L18)).
- A link to a part of a file is a link to the file (rev 28): a Markdown or HTML site whose answer is a
  confirmed section of ANOTHER file also references that file — one `also` row at the answer's rung, the
  section staying the answer and keeping its own verdict. None for a section of the linking page itself
  (`[x](#part)`, or `[x](me.md#part)` in `me.md`: a page pointing into itself is referenced by nothing),
  none for an unconfirmed fragment (its answer is already the file), none for an unused definition (its
  answer is inert) ([Answer.hs:65-74](../../../core/app/CE/Resolve/Answer.hs#L65),
  [Resolve.hs:160-173](../../../core/app/CE/Resolve.hs#L160)).
- A Go directory counts as an importable package only while it *directly* holds an in-scope
  non-`_test.go` file ([Go.hs:120-125](../../../core/app/CE/Resolve/Go.hs#L120)).
- GitHub slugging: lowercase, keep alphanumerics/`_`/`-`, spaces→hyphens, everything else
  dropped ([md_slug.rs:197-207](../../../cli/src/graph/ladder/md_slug.rs#L197)), applied to the heading's
  RENDERED text — link and image syntax collapse to their text, code and emphasis delimiters drop,
  inline HTML drops, escapes unescape ([md_slug.rs:78-104](../../../cli/src/graph/ladder/md_slug.rs#L78));
  duplicates take `-N` suffixes in document order ([md_slug.rs:62-66](../../../cli/src/graph/ladder/md_slug.rs#L62)).
  A setext heading — a paragraph under `===` or `---` — slugs like an ATX one, its lines joined, from
  its first row ([md_head.rs:30-61](../../../cli/src/graph/ladder/md_head.rs#L30)), and the section units
  read the same headings ([units.rs:174-193](../../../cli/src/fourclass/units.rs#L174)) — plan v2.30 step 5b.
  Raw-HTML anchors (`<a name=…>`, `<a id=…>`, `<h1..6 id=…>`) enter the set verbatim, the tag read across
  lines and its attribute with or without spaces around `=`, quoted or bare
  ([md_head.rs:126-150](../../../cli/src/graph/ladder/md_head.rs#L126)); a fragment is percent-decoded
  before the lookup ([Url.hs:30-46](../../../core/app/CE/Resolve/Url.hs#L30)); an indented
  code block offers no heading and no site — four columns past the innermost open list item's content
  column, or past the margin outside a list, where no paragraph is open
  ([md_mask.rs:28-38](../../../cli/src/graph/md_mask.rs#L28), [md_mask.rs:127-145](../../../cli/src/graph/md_mask.rs#L127)).
  Anything but exactly one
  match degrades to a file-level edge, never invents a section
  ([Md.hs:121-126](../../../core/app/CE/Resolve/Md.hs#L121)).
- The external tables are machine-generated, never hand-typed: CPython 3.13
  `sys.stdlib_module_names` ([Ladder2.hs:54-58](../../../core/app/CE/Lang/Common/Ladder2.hs#L54)), Go 1.26.4
  `go list std` minus `internal/`/`vendor/` ([Ladder2.hs:10-15](../../../core/app/CE/Lang/Common/Ladder2.hs#L10)),
  and GHC 9.14.1's global db — **43 packages, 1371 modules**
  ([Boot1.hs:20-21](../../../core/app/CE/Lang/Common/Boot1.hs#L20)). A missing name degrades to
  `Unresolved` (precision-safe), visible in the ledger.
- Rust's `ResolvedVia` keeps the **original walk's rung** and records the hop as a separate
  `via_reexport` column, not as a new rung
  ([Rs.hs:224-226](../../../core/app/CE/Resolve/Rs.hs#L224),
  [wire.rs:171-172](../../../cli/src/graph/wire.rs#L171)).

Cross-file staleness is closed at the resolve key rather than by re-sweeping: the
target-content facts a ladder consults are six — the Markdown anchor set, a page's HTML `id`
set, the Rust top-level surface (pub-use bindings, every top-level `use` binding, the item
names — the crate rung's tie-break reads the last two), a Java file's declared package, the
templates a Lua file assigns to `package.path` and a C-family file's include list — and each is
folded into a hash that is a resolve-key input
([walkidx.rs:241-291](../../../cli/src/dedup/walkidx.rs#L241),
[md_slug.rs:24-31](../../../cli/src/graph/ladder/md_slug.rs#L24),
[html_head.rs:77-86](../../../cli/src/graph/ladder/html_head.rs#L77),
[rs_cst.rs:179-214](../../../cli/src/graph/ladder/rs_cst.rs#L179)). The Markdown and Rust hashes are pinned by
a coupling battery asserting `hash(a)==hash(b) ⟺ projection(a)==projection(b)`
([md_tests.rs:14-33](../../../cli/tests/unit/graph/ladder/md_tests.rs#L14),
[unit/graph/ladder/rs_cst.rs:13-75](../../../cli/tests/unit/graph/ladder/rs_cst.rs#L13)).

### 3. Edge extraction and node identity

Only in-corpus outcomes become stored rows; `External` and `Unresolved` sites stay
ledger-visible as sites *without* edges ([wire.rs:155-188](../../../cli/src/graph/wire.rs#L155)), and
an `External` one is marked answered in its ledger word — `sites.ledger` bit 0, so the trust
ledger never counts a stdlib, registry or vendored target as a miss (lane v235_licence,
[wire.rs:98-111](../../../cli/src/graph/wire.rs#L98), [load.rs:60-69](../../../cli/src/graph/load.rs#L60)). An edge's
kind follows its site's: code kinds import; the Markdown link family and HTML's page references (`href`, a
form's `action`) split doc_link / doc_ref by whether the target is a page (Markdown or HTML); an image, a
`src` / `srcset` resource and an asset `<link>` are always assets
([wire.rs:215-232](../../../cli/src/graph/wire.rs#L215), [wire.rs:234-243](../../../cli/src/graph/wire.rs#L234)).
Edge kinds
are frozen positions: `EDGE_IMPORT = 0`, `EDGE_DOC_LINK = 1`, `EDGE_DOC_REF = 2`,
`EDGE_ASSET = 3`, `EDGE_CONTAIN = 4`, and since 2.29.0 `EDGE_REFDEF_UNUSED = 5` — an unused
reference definition's in-scope target, which resolves and travels as an edge while the core
excludes it from liveness beside the asset kind
([wire.rs:27-36](../../../cli/src/graph/wire.rs#L27), [Cost.hs:199-206](../../../core/app/CE/Graph/Cost.hs#L199)); granularity
codes are `GRAN_FILE = 0`, `GRAN_PACKAGE = 1`, `GRAN_SECTION = 2`
([wire.rs:44-47](../../../cli/src/graph/wire.rs#L44)).

Node identity is the pair `(path, unit)` over a `BTreeSet` of every walked file plus every edge
target, so the id assignment is a function of the graph and the wire bytes are shuffle-proof
([nodes.rs:32-67](../../../cli/src/graph/nodes.rs#L32), asserted at
[unit/graph/nodes.rs:70-86](../../../cli/tests/unit/graph/nodes.rs#L70)). Package-ness is read from the edge's *stored*
granularity, never inferred from a target's absence — the old absence rule minted image assets
and dangling doc refs as packages ([nodes.rs:35-38](../../../cli/src/graph/nodes.rs#L35),
[unit/graph/nodes.rs:88-111](../../../cli/tests/unit/graph/nodes.rs#L88)).

Two transformations happen on the way to the wire:

1. **Two edge kinds are liveness-inert, in the core** — an image reference is not a reference
   for liveness purposes, and neither is an unused reference definition, which renders nothing and
   must not keep its target alive (user decision D3); since 2.20.0 every edge kind travels and the
   exclusion is the core's own rule — `assetKind`
   ([Cost.hs:143-151](../../../core/app/CE/Graph/Cost.hs#L143)) since 2.20.0, `refdefKind`
   ([Cost.hs:199-206](../../../core/app/CE/Graph/Cost.hs#L199)) since 2.29.0 — the two riding one
   inert list into the same comprehension as the rung filter
   ([Graph.hs:159](../../../core/app/CE/Graph.hs#L159), [Build.hs:43-49](../../../core/app/CE/Graph/Build.hs#L43)) — Rust no longer pre-drops rows
   ([deadcode.rs:327-338](../../../cli/src/graph/deadcode.rs#L327)). An endpoint that is not a node
   is a *named error*, never a panic ([deadcode.rs:331-334](../../../cli/src/graph/deadcode.rs#L331), [deadcode.rs:345](../../../cli/src/graph/deadcode.rs#L345)).
2. **Synthetic containment arcs** are added from each package node to every file under its
   directory — or, for a package whose code its manifest declares (an R package's `DESCRIPTION`,
   plan v2.30 step 4: the `Collate` files, else the files directly in `R/`), to that code alone,
   since `library(pkg)` runs the package's `R/` files and not its NEWS.md, and a package at the
   tree root would otherwise reach every file of the tree — at `rung 1` because containment is a
   fact, not a resolution mechanism, and must survive every rung ceiling ([nodes.rs:104-127](../../../cli/src/graph/nodes.rs#L104)). A repo-root
   package has path `""`, and the naive `format!("{}/", "")` prefix `"/"` matched nothing —
   measured: a root `lib.go` imported by `cmd/main.go` was reported dead
   ([nodes.rs:120-128](../../../cli/src/graph/nodes.rs#L120)).

The whole read runs in **one snapshot transaction**: as three autocommit statements a
convergent writer landing between them could hand the edge query a source file the files query
never saw ([load.rs:130-136](../../../cli/src/graph/load.rs#L130)). `unresolved_sites` is the count of sites
whose own answer drew no edge and was not External (`sites.ledger` bit 0, rev 27), plus one for each star-import site whose `__all__` is opaque (`sites.ledger` bit 1,
rev 26; [load.rs:60-69](../../../cli/src/graph/load.rs#L60)) ([load.rs:161-165](../../../cli/src/graph/load.rs#L161)) and travels with the report so
the reader sees what the graph refuses to know
([deadcode.rs:27-29](../../../cli/src/graph/deadcode.rs#L27)).

### 4. Boundary contract and caps

`graph.request` carries `nodes: [[lang, kind, roles]]`, `edges: [[src, dst, kind, rung]]`, and
an optional `pos: [idx]`. The core machine-checks, in request order so the message is
deterministic ([Contract.hs:88-110](../../../core/app/CE/Graph/Contract.hs#L88)):

- node rows are exactly 3 fields — `[lang, kind, roles]`, all `≥ 0`; ONE arity since 5.0.0 retired the pre-2.28 legacy flags column, so a wrong-width row is malformed and says which row ([Contract.hs:216-231](../../../core/app/CE/Graph/Contract.hs#L216));
- edge rows are exactly 4 fields, all `≥ 0`, with `src < n` and `dst < n`
  ([Contract.hs:223-229](../../../core/app/CE/Graph/Contract.hs#L223));
- the edge table is **strictly ascending** lexicographically, hence duplicate-free
  ([Contract.hs:106](../../../core/app/CE/Graph/Contract.hs#L106), [Rows.hs:49-59](../../../core/app/CE/Wire/Rows.hs#L49));
- `pos` indices lie in `[0, n)` and are strictly ascending — which is also the reply *bound*,
  since a repeated-index list would make the reply larger than the request without limit
  ([Contract.hs:107-111](../../../core/app/CE/Graph/Contract.hs#L107),
  [Contract.hs:235-238](../../../core/app/CE/Graph/Contract.hs#L235)).

Oversize protection is by row count, not bytes (the envelope precheck is relaxed for the
trusted same-machine child): `nodeCap = 131072` and `edgeCap = 524288`
([Cost.hs:25-29](../../../core/app/CE/Graph/Cost.hs#L25)). The sizing anchor is ~20k nodes / ~60k edges
per 100k LOC, so the caps carry ~6× headroom on nodes and ~8× on edges; a request with every
table at its cap — the 6.2.0 advisory tables included — stays under the 128 MiB envelope
([Cost.hs:16-24](../../../core/app/CE/Graph/Cost.hs#L16)). Over cap the core returns a **well-formed
degraded result** with `dead = []`, `reported = []`, `kept = 0`, `degraded = true`,
`reason = "graph_too_large"` and `fail = true` — a gate that could not judge never passes, said
by the core itself since 2.18.0, and never a truncated graph
([Graph.hs:65-85](../../../core/app/CE/Graph.hs#L65)).
The CLI treats a degraded reply as an event, not silence: it lands in the observe feed
([deadcode.rs:468-482](../../../cli/src/graph/deadcode.rs#L468)) and `ce deadcode --check` relays the
core's fail bit — degraded, or a blocking file (§10) ([Lines.hs:166-167](../../../core/app/CE/Graph/Lines.hs#L166), [main_prelude.rs:29-44](../../../cli/src/main_prelude.rs#L29)).

### 5. Kept arcs and liveness

The kept arc set is the rung-filtered, kind-filtered, `(src,dst)`-deduplicated edge list — the
two liveness-inert kinds are dropped in the same comprehension as the rung filter, and kind
multiplicity between one pair is *not* extra evidence of reference, so indegree counts distinct
arcs ([Build.hs:1-7](../../../core/app/CE/Graph/Build.hs#L1), [Build.hs:40-51](../../../core/app/CE/Graph/Build.hs#L40),
the inert list supplied at [Graph.hs:159](../../../core/app/CE/Graph.hs#L159)):

```
inert = { assetKind = 3, refdefKind = 5 }
arcs  = { (s,d) | [s,d,kind,rung] ∈ edges,  rung ≤ minRung,  kind ∉ inert }
kept  = |arcs|
G     = buildG (0, n-1) arcs        -- all n vertices; isolated ones are exactly the unreferenced
```

`minRung = 5` ([Cost.hs:108-109](../../../core/app/CE/Graph/Cost.hs#L108)). Because the ladder never
guesses — ambiguity becomes `Unresolved` and never crosses the wire — every rung `resolve/1`
answers (`1..5`) is admitted by default; lowering the constant is the ablation lever that trades
recall for certainty ([Cost.hs:102-107](../../../core/app/CE/Graph/Cost.hs#L102)). At the current value the
filter is a no-op ceiling: the highest rung any ladder emits is 5 (TS R5, Markdown R5), and the
per-ceiling trade is published instead by the `cut` table of the precision instrument
([eval_graph_precision_parts/mod.rs](../../../cli/tests/it/eval_graph_precision_parts/mod.rs)).

A fragment link reaches the file. No arc relates a section node to the file that holds it, so an arc
into a section alone never made its file reachable: until rev 28 a page a live page links to by one of
its headings was reported unreferenced, while a link with a broken fragment kept it alive. A site whose answer
is a confirmed section of another file now also stores a file-granular `also` edge to that file (§2),
so the file has an in-arc of its own: alive when the linking page is, a dependent (`unreach_*`) of it
when the linking page is unreferenced.

**Entry roots.** A node seeds reachability iff `flags .&. entryMask ≠ 0`
([Dead.hs:17-18](../../../core/app/CE/Graph/Dead.hs#L17)), with `entryMask = 126` = bits 1–6
([Cost.hs:122-123](../../../core/app/CE/Graph/Cost.hs#L122)). Declared bits: 1 main, 2 test, 3 entry-glob,
4 dyn-referenced, 5 doc-entry, 6 `ce:allow(deadcode)`
([Cost.hs:112-116](../../../core/app/CE/Graph/Cost.hs#L112)). Bit 0 (exported) is **deliberately absent** —
exported-ness is the public/private *verdict* axis, so a library's unreferenced API surfaces as
`unref_public` rather than as a private unreferenced file or as silence
([Cost.hs:118-121](../../../core/app/CE/Graph/Cost.hs#L118)).

Only file nodes carry entry facts; section and package rows get `0`
([deadcode.rs:357-390](../../../cli/src/graph/deadcode.rs#L357)). Since proto **2.28.0**
(batch-7 slice 3 main body) the node row's last column carries **role facts** — the third and
last since 5.0.0, `[lang, kind, roles]` ([deadcode.rs:389](../../../cli/src/graph/deadcode.rs#L389)) — and the
category membership Rust used to fuse into the flags column is decided by the core's
**role table** `roleBits` ([Graph/Cost.hs:180-181](../../../core/app/CE/Graph/Cost.hs#L180)):
the row's entry bits derive through `deriveFlags`
([Dead.hs:76-78](../../../core/app/CE/Graph/Dead.hs#L76), applied at
[Graph.hs:182-184](../../../core/app/CE/Graph.hs#L182)). Until 5.0.0 a legacy flags
column sat between `kind` and `roles` and yielded to them; it is gone, and a
wrong-width row now refuses by row index rather than as a mixed table. The Rust producer measures:

```
role 0  base ∈ {main.rs, build.rs, main.go, __main__.py, Main.hs, main.c,
        main.cc, main.cpp, Main.java, main.lua, conf.lua, app.R, ui.R,
        server.R, global.R, index.html, 404.html}                   [flags.rs:71-84, 91-93]
role 1  path starts with an entry directory: src/bin/ examples/ benches/
        cmd/ (every language); Neovim's plugin/ ftplugin/ indent/ syntax/
        colors/ compiler/ ftdetect/ lsp/ after/ (Lua); an R package's
        inst/ vignettes/ data-raw/ exec/ demo/ (R)                  [flags.rs:124-149]
role 2  base is a name a test runner discovers, the core's
        flags.runner_tests, each form as its runner documents it: Go's
        *_test.go, pytest's test_*.py / *_test.py / conftest.py, Jest's
        and Vitest's *.test.* / *.spec.* on .ts .tsx .mts .cts (and Jest's
        bare test.* / spec.*), hspec-discover's *Spec.hs and the driver
        Spec.hs, the C family's _test, Surefire's .java forms, busted's
        _spec.lua / _test.lua, testthat's test[-_]*.[Rr]; or a path
        component in flags.test_dirs_role (tests, __tests__) — the
        convention word reads the same runner table and its own wider
        directory row                               [flags.rs:171-180; name.rs:95-106; PathForms.hs:11-60]
role 3  ce.toml [graph] entry_globs hit through the ONE ce.toml glob
        dialect (exclude / class / entry share it): exact path, bare
        basename, dir/ (= dir/**), *.ext, and every pattern as written [globs.rs:33-89]
role 4  base ∈ flags.doc_names {README.md, CLAUDE.md}, or a flags.doc_dirs
        row: docs/**/{index.md, README.md}         [flags.rs:159-169; PathForms.hs:65-71]
role 5  inline `ce:allow(deadcode) -- <why>` claim (a BARE marker
        claims nothing — the docdup exemption discipline)           [flags.rs:151-159]
role 6  a manifest-declared target: Cargo [lib]/[[bin]]/[[example]]/
        [[test]]/[[bench]] paths and conventional targets via
        crate_roots, cabal main-is through each stanza's source
        roots, package.json main/module/types/bin/exports, the root
        pyproject's scripts / gui-scripts / entry-points           [targets.rs:44-80; Cabal.hs:72-84; cabal_find.rs:10-26; Entries.hs:3-27]
role 7  a declared submodule's node (index `files.owner` = 1; a
        package or section under a foreign file's path), sent ALONE
        — no other role is measured on a reader                     [flags.rs:31-35; deadcode.rs:325-327, 347-348; nodes.rs:32-93]
role 8  a C-family compilation unit (.c / .cc / .cpp / .cxx)        [flags.rs:36-41, 97-100]
role 9  a walked asset: a file the index holds no parse of, named
        by a page's src / href / link (a stylesheet, a script, an
        image) — the node table marks it by construction and it is
        sent ALONE, no other role measured on it                  [nodes.rs:20-29, 69-77; flags.rs:42-49; deadcode.rs:328-330, 349-350]
role 10 a tool convention: a file a tool finds by where it sits, one
        row per convention in the core's flags.entry_conventions, as
        its tool's documentation states it — basename package-info.java,
        vitest.config.{ts,mts,cts,js,mjs,cjs}, setup.py, conf.py
        (Sphinx); under the FIRST app/ segment with no later _ segment,
        page.tsx / layout.tsx / route.ts (Next.js); exact paths
        {,docs/,.github/}CONTRIBUTING.md (without case),
        SECURITY.md and CODE_OF_CONDUCT.md; and Neovim's init.lua at
        the root (part 4A: the eight conventions generation 1 of the
        dead-file exam passed, docs/EVAL-SET-DEADFILE.md §3)         [flags.rs:50-53, 94-96; convention.rs:11-76; Graph.hs entry_conventions]
```

([flags.rs:24-53](../../../cli/src/graph/deadcode/flags.rs#L24),
[flags.rs:86-122](../../../cli/src/graph/deadcode/flags.rs#L86)). The role→bit landing is the
core's data: roles 0, 1, 6, 8 (7.2.0) and 10 (9.1.0) all land on bit 1, roles 2/3/4/5 on bits 2/3/5/6, role 7
(6.3.0) on bit 2 beside the test convention — a foreign reader's references seed
reachability and it is never judged, the same standing a test file has — and role 9 (7.2.0,
plan v2.30 step 5) alone on bit 4, the dyn-referenced bit, which had no producer before: the
references the graph cannot read (a stylesheet's `url()`, a script's fetch, a manifest's icons)
keep an asset alive, so it is never a candidate and never a measured node
([deadcode.rs:249-251](../../../cli/src/graph/deadcode.rs#L249)). **Role 6 closes
a ledgered defect**: a declared `[[bin]] path` or cabal `main-is` target is a root, where
before only the name conventions were — and, since lane v235_graphtrust, so is every file a manifest
names as what a runtime or a tool starts: a package.json's `main` / `module` / `types` / `bin` and its
`exports` leaves (a `*` pattern names every walked file it matches; a `.js` target no walked file answers
names its TS twin), the root `pyproject.toml`'s `[project.scripts]` / `[project.gui-scripts]` /
`[project.entry-points.*]` modules, found by the core (resolve/1 `entries`,
[Entries.hs:3-27](../../../core/app/CE/Resolve/Entries.hs#L3)) — the discovery is nearest-manifest per walked directory
([targets.rs:53-98](../../../cli/src/graph/deadcode/targets.rs#L53),
[cabal_find.rs:10-26](../../../cli/src/graph/cabal_find.rs#L10)). A tree whose manifest lives
elsewhere — the test-suite submodule is a slice of the `cli` package, its binaries cargo
targets only in the superproject's Cargo.toml — declares its roots in `ce.toml [graph]
crate_roots` (plan v2.18 step #12, zero wire): a declared root is a target for this role
([targets.rs:82](../../../cli/src/graph/deadcode/targets.rs#L82)) and a crate root for the
Rust ladder's `mod` and `crate::` rungs alike
([Rs.hs:62](../../../core/app/CE/Resolve/Rs.hs#L62)), one normalizer serving both readers
([graph.rs:77](../../../cli/src/config/graph.rs#L77)); a declared path the walk does not hold, or that
is no Rust file, is refused by name ([walkidx.rs:195](../../../cli/src/dedup/walkidx.rs#L195)). The legacy flags column this
module also produced — bit-identical to the pre-2.28 semantics, and read by no core since
2.28.0 — retired at 5.0.0, once 4.1.0's symbols table gave visibility the producer whose
absence had blocked the subtraction.
**Honest gaps that remain:** bit 4 (dyn-referenced) has one producer, role 9 — a walked asset, kept
alive by the references the graph cannot read (7.2.0, above) — and no other: dynamic reference
construction in code is an open set that grows with each language version, so no enumeration of it
can be called complete (plan v2.14 K8).

Bit 0 was the other one, and it closed at 4.1.0. Nothing at file granularity ever measured it,
because public-ness is not a file fact ([flags.rs:1-11](../../../cli/src/graph/deadcode/flags.rs#L1));
it now reaches the core through the `symbols` export surface
([symwire.rs:1-27](../../../cli/src/graph/symwire.rs#L1)), so `unref_public` and `unreach_public`
fire for the first time — including for Haskell, whose export list the visibility slice reads
where it lives ([hs.rs:27](../../../cli/src/fourclass/visibility/hs.rs#L27)).

Reachability is plain forward closure from the seeds over kept arcs
([Build.hs:53-58](../../../core/app/CE/Graph/Build.hs#L53)):

```
reach = ⋃ { reachable(G, s) | s ∈ entries(entryMask, flags) }
```

The union is computed as one multi-source `Data.Graph.dfs` pass from all the seeds (one shared
visited set, O(V+E); the per-seed `reachable` form retired in batch 9 P2), and the forest holds
the seeds themselves, so an entry node is never judged
([Build.hs:53-58](../../../core/app/CE/Graph/Build.hs#L53), [Dead.hs:6-7](../../../core/app/CE/Graph/Dead.hs#L6)).

### 6. SCC handling and the position surface

Every SCC in `Data.Graph` order, members ascending, is a deterministic function of the sorted
arc set; **singletons are included** so the id space covers every vertex
([Build.hs:51](../../../core/app/CE/Graph/Build.hs#L51)). The cycle *report* applies the floor
downstream: an SCC is reported iff `|members| ≥ sccFloor`
([Cycles.hs:15-21](../../../core/app/CE/Graph/Cycles.hs#L15)) with `sccFloor = 2` shipped, i.e. only true
multi-node cycles; since 6.4.0 the request's `sccFloor` (ce.toml `[graph] scc_floor`, refused below 1)
overrides it, and at floor 1 a singleton is reported exactly when it carries a self-arc
([Cycles.hs:23-24](../../../core/app/CE/Graph/Cycles.hs#L23)) — the verdict's cycle axis reads the SAME number as
threshold code 7, so a file is a cycle on both faces or on neither; widening the shipped floor is
a knob change the dead-knob test can see, not a code change
([Cost.hs:125-130](../../../core/app/CE/Graph/Cost.hs#L125)). Cycle ids are positions in the full SCC list,
so they agree with `Position`'s `sccId` by construction
([Cycles.hs:1-4](../../../core/app/CE/Graph/Cycles.hs#L1)).

**Cycles are reported, never judged** — the verdict pass does not read this list
([Cycles.hs:3-4](../../../core/app/CE/Graph/Cycles.hs#L3)). A cyclic island with no entry seed is
therefore in the dead set by reachability alone, without a special case (its files are roots, §10).

The per-node join surface, computed only for the requested `pos` indices, is
`[idx, indeg, outdeg, sccId, sccSize, reachIn]`
([Position.hs:1-3](../../../core/app/CE/Graph/Position.hs#L1),
[Position.hs:14-32](../../../core/app/CE/Graph/Position.hs#L14)); degrees count distinct kept arcs, and
`reachIn` is `fromEnum (i ∈ reach)`. A non-degraded reply **must** answer every requested index
— a short `pos` table would silently starve the M5-3 join, so the CLI refuses it
([deadcode.rs:406-409](../../../cli/src/graph/deadcode.rs#L406)).

### 7. The four-way verdict

The dead set splits along **two independent axes** — indegree × reachability — with public structurally
separated so an exported-but-unreferenced API can never collapse into a private verdict
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

The names are the core's too since plan v2.32 step 4: the deadcode document
(`CE.Graph.Document`, laid out over `document/1`) spells a verdict by position — `verdictNames`
read at `code - 1` ([Document.hs:69-74](../../../core/app/CE/Graph/Document.hs#L69)) — and the
Rust side, which sends the judgment's rows back as codes, refuses a code past the four as
wire-version skew, not a panic ([reply.rs:69-75](../../../cli/src/graph/deadcode/reply.rs#L69)).
The `why` string is a two-way split on the same axis: codes 1–2 read *"no kept in-edge and no
entry flag"*, codes 3–4 read *"referenced only from unreferenced files; no entry flag"*
([Document.hs:76-79](../../../core/app/CE/Graph/Document.hs#L76), placed at
[Document.hs:253-258](../../../core/app/CE/Graph/Document.hs#L253)).

**The reporting firewall.** Only file nodes enter `dead`; section and package verdicts go to a
separate `reported` table and are never in the dead set — aggregates are not code entities. Since
proto 2.18.0 (batch-7 slice 4) the split is the CORE's: the reply partitions its verdicts on
the node kind column it always received ([Graph.hs:176-178](../../../core/app/CE/Graph.hs#L176),
`granFile` at [Cost.hs:132-141](../../../core/app/CE/Graph/Cost.hs#L132)), and carries the additive
`fail` bit, since plan v2.35 the gate's own answer — a degraded run or a blocking file (§10). The Rust
side keeps the split as a boundary
contract, because `ce erase` builds its dead-file rows from the `dead` table (rows, not deletions —
book 12 §2, `reachable_by_name`): an aggregate arriving in `dead` refuses as wire skew, never a directory row
([reply.rs:89-96](../../../cli/src/graph/deadcode/reply.rs#L89)); an absent `fail` bit or
`reported` table refuses as wire skew by name too — the handshake already turns a pre-2.18
core away, so the client's old fallback conjunction was unreachable and was retired (L round
step #15, O62; [reply.rs:43-46](../../../cli/src/graph/deadcode/reply.rs#L43)). Both lists, the counts, and
`unresolved_sites` ship in the JSON document the core lays out
([Document.hs:172-188](../../../core/app/CE/Graph/Document.hs#L172)). The design's *"no entry rule ⇒ every doc trivially
dies"* stance is deliberate: an unlinked doc **is** listed
([deadcode.rs:13-15](../../../cli/src/graph/deadcode.rs#L13)).

### 8. The dead-row confidence (2.32.0)

Since 2.32.0 the request carries a per-language site ledger (optional until 9.0.0, required since) — `"unres": [[lang, unresolvedSites, totalSites], ...]`, langs judged-set-bounded, counts coherent (`unresolved <= total`), strictly ascending hence duplicate-free ([Contract.hs:203-210](../../../core/app/CE/Graph/Contract.hs#L203)). Unlike the old scalar count (an unvalidated honest ledger), this table is an INPUT to judgment: every dead row carries a third column, the confidence the dead node's OWN language can lend its verdict ([Graph.hs:122](../../../core/app/CE/Graph.hs#L122)):

```
0  unvouched — the language still carries unresolved sites: "nothing
   references this" assumed none of those sites lands in-corpus
1  vacuous   — no site of that language ever existed (an absent
   ledger row reads (0, 0))
2  vouched   — a fully resolved reference population
```

`vouched` says every reference site of the language is answered — in-corpus or External —
which is a statement about sites, not about use: a file run by its name or found by a tool's
convention leaves no site, so a vouched unreferenced file can still be in use, and the erase family
never deletes one (book 12 §2, `reachable_by_name`).

([Cost.hs:182-197](../../../core/app/CE/Graph/Cost.hs#L182)). This is the erase family's trust boundary — *a language with unresolved sites cannot vouch for its dead verdicts* — executed by the family that owns the ledger; the erase predicate consumes the column as a fact (book 12 §class 3). Until 9.0.0 a request without the key kept two-column dead rows; since 9.0.0, which made the measuring side's standing practice the contract, such a request is refused by name (`unres: required at 9.0.0`). The Rust side folds per-path site counts to the per-language rows inside the same snapshot that produced the edges ([load.rs:166-176](../../../cli/src/graph/load.rs#L166), [deadcode.rs:303](../../../cli/src/graph/deadcode.rs#L303), [deadcode.rs:312-325](../../../cli/src/graph/deadcode.rs#L312)), fences every returned index and bounds the column ([reply.rs:83-100](../../../cli/src/graph/deadcode/reply.rs#L83)), and the core renders the trust word beside each dead file ([Lines.hs:87-92](../../../core/app/CE/Graph/Lines.hs#L87)). The props battery pins all three codes through the real `respond`, the refusal of a request without the ledger beside them, and every ledger refusal by name ([GraphWireProps.hs:135](../../../core/test/GraphWireProps.hs#L135)).

### 9. Acceptance

The M5-2 row sets four criteria: import-edge precision **≥ 0.90** on a 100-site manual audit
spanning the five launch languages; `unreferenced_public` as its own report class, not folded
into dead; every finding in this repository dispositioned; and the core's judgment-invariant
property battery in CI
([DEVELOPMENT_PLAN.md:286](../../DEVELOPMENT_PLAN.md#L286)). The gate is coded at `0.90`,
applied overall and per corpus **where the in-corpus ground-truth denominator reaches 5**
([eval_graph_precision.rs:83](../../../cli/tests/it/eval_graph_precision.rs#L83),
[eval_graph_precision.rs:86-94](../../../cli/tests/it/eval_graph_precision.rs#L86),
[precision.rs:43-53](../../../cli/tests/it/eval_support/precision.rs#L43)); precision is
`correct / (correct + wrong)` over answered rows only
([precision.rs:48](../../../cli/tests/it/eval_support/precision.rs#L48),
[precision.rs:70](../../../cli/tests/it/eval_support/precision.rs#L70)).

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
([eval_graph_precision.rs:1-6](../../../cli/tests/it/eval_graph_precision.rs#L1)); the sample is 100 sites
with a per-language floor of 15 ([eval_graph_precision.rs:91-92](../../../cli/tests/it/eval_graph_precision.rs#L91)).
The "all findings dispositioned" criterion, honored by discipline at M5-2, became a gate, and since
plan v2.35 it is a project's declaration: `ce deadcode --check` exits non-zero when an unreferenced
file's state is one `[deadcode] block` names (§10) — this repository blocks `orphaned` and `born`
([Lines.hs:163-164](../../../core/app/CE/Graph/Lines.hs#L163), [Deadcode.hs:43](../../../core/app/CE/Text/Deadcode.hs#L43)).

The six languages plan v2.30 added sit the same exam, each against a blind audit frozen before any
scoring and registered in [EVAL-SET-LANGS.md](../../EVAL-SET-LANGS.md): the sample is frozen first,
independent auditors resolve every site from the pinned tree alone, and only then does the ladder
answer — for C and C++ the ladder itself was committed before the sample (the audit was blind to its
answers, not to its existence), for the other four after it. Precision is the same
`correct / (correct + wrong)` over answered rows; a truth naming a path the walk refuses is scored
outside the corpus, so the ladder cannot choose its denominator there either
([precision.rs:1-8](../../../cli/tests/it/eval_lang_parts/precision.rs#L1)):

| corpus | correct | wrong | in-corpus GT | precision | gated? |
|---|---|---|---|---|---|
| lua (C) | 73 | 0 | 74 | 1.0 ([lua:1662-1664](../../../contracts/eval/lang-precision-lua-v1.json#L1662)) | yes |
| fmt (C++) | 15 | 0 | 17 | 1.0 ([fmt:1571-1573](../../../contracts/eval/lang-precision-fmt-v1.json#L1571)) | yes |
| gson (Java) | 12 | 0 | 12 | 1.0 ([gson:689-691](../../../contracts/eval/lang-precision-gson-v1.json#L689)) | yes |
| jsoup (Java) | 24 | 0 | 24 | 1.0 ([jsoup:1083-1085](../../../contracts/eval/lang-precision-jsoup-v1.json#L1083)) | yes |
| koreader (Lua) | 76 | 0 | 76 | 1.0 ([koreader:1429-1431](../../../contracts/eval/lang-precision-koreader-v2.json#L1429)) | yes |
| luarocks (Lua) | 10 | 0 | 10 | 1.0 ([luarocks:296-298](../../../contracts/eval/lang-precision-luarocks-v2.json#L296)) | yes |
| covid19model (R) | 18 | 0 | 18 | 1.0 ([covid19model:1608-1610](../../../contracts/eval/lang-precision-covid19model-v1.json#L1608)) | yes |
| stringr (R) | 0 | 0 | 0 | — ([stringr:95-97](../../../contracts/eval/lang-precision-stringr-v1.json#L95)) | no (denom < 5) |
| codeeraser (HTML) | 25 | 0 | 25 | 1.0 ([codeeraser:850-852](../../../contracts/eval/lang-precision-codeeraser-v1.json#L850)) | yes |
| html5-boilerplate (HTML) | 0 | 0 | 0 | — ([html5-boilerplate:51-53](../../../contracts/eval/lang-precision-html5-boilerplate-v1.json#L51)) | no (denom < 5) |
| learning-area (HTML) | 50 | 0 | 51 | 1.0 ([learning-area:1095-1097](../../../contracts/eval/lang-precision-learning-area-v1.json#L1095)) | yes |

Every answered row is correct (303 / 303 across the eleven docs) and 4 truths are
missed, each named in the ledger: a Lua test library's `lauxlib.h` at the root, fmt's two gtest headers
under a directory no `include/` convention or declared root names, and one empty `link_asset` record in
learning-area. Beside precision, `contracts/eval/fpr-lang-v1.json` freezes one duplicate-write replay row
per language — 400 commits of the first exam corpus each, both framings recorded (booklet 11) — and
`fpr_lang_gate.rs` executes the release gate against every bit of `Lang::judged_mask()`: a language is
judged only while its own row admits it, and all six read 0 false intercepts by the standing ledger's
criterion ([fpr_lang_gate.rs:1-9](../../../cli/tests/it/fpr_lang_gate.rs#L1)).

### 10. Roots, dependents, states and the gate (plan v2.35)

§7 answers *which files does nothing live reach*; it does not answer *which of them nobody uses*, and two
generations of blind audit say an unreached file is often in use (below). Since plan v2.35, the dead-file
amendment, such a file is listed as **unreferenced**, and by default it fails no check and charges no
score.

**Roots and dependents** — the core's definitions ([Roots.hs:1-31](../../../core/app/CE/Graph/Roots.hs#L1)).
Write U for the nodes that kept arcs from the entry set never reach, and D for its file nodes — the dead
set. When some kept path runs from one dead file g to another, f, then g **supports** f; such a path stays
inside U, because a reached node on it would make f reached. A **root** is a dead file that supports back
every file supporting it (no strict ancestor of its component in the condensation holds a dead file); the others
are **dependents**, each falling with the roots that support it, of which there is always one or more. A
file nothing references is a root; so is every file of a dead cycle that no
dead file outside the cycle supports. The core reads the root set off the condensation the build already
holds (`bScc`) in one pass and never drops it; the dependent → root pairs cost at most
`underCap = 131072` units of work (root ids copied along condensation arcs plus pairs written) and past it
are dropped whole, the reply saying so with `underDropped`
([Cost.hs:77-85](../../../core/app/CE/Graph/Cost.hs#L77), [Graph.hs:116](../../../core/app/CE/Graph.hs#L116)).

**The comparison point and the states** ([Point.hs:5-26](../../../core/app/CE/Graph/Point.hs#L5)).
`ce-baseline.json` records the graph's own facts on the day it is written: `deadPoint = {files, dead}`,
the entity ids of every file node the graph judged and of those in the dead set — a file's id is its size
row's, `score::baseline::file_entity` of its project-relative path. graph/1 answers today's point
(`deadNow`) and the verdict family copies it into every baseline it writes. Every file of D has a
**state**, root and dependent alike: its id in the point's `dead` → `standing`; in `files` and not in
`dead` → `orphaned`; not in `files` → `born`; no point rode → `untold`
([Cost.hs:87-92](../../../core/app/CE/Graph/Cost.hs#L87),
[Point.hs:121-140](../../../core/app/CE/Graph/Point.hs#L121)). Two corners follow, both by
these words:

- *A file is judged by its own history, wherever it hangs.* With `main.ts` importing `lib.ts` when the
  baseline was written, replace that import and add `n.ts` importing `lib.ts`: `lib.ts` lost its last live
  reference and reads `orphaned`, a dependent under the `born` root `n.ts`. A project that blocks
  `orphaned` alone is failed by `lib.ts` (no file in a blocked state supports it); one that blocks both
  states is failed by `n.ts` alone, `lib.ts` falling with it.
- *The point is keyed by path.* A standing `s.ts` renamed to `t.ts` has an id the point never saw:
  `t.ts` reads `born`.

**The gate.** `ce.toml [deadcode] block` lists the states that fail, out of `"orphaned"` and `"born"` —
`blockStates`, the states after `untold` and `standing`
([Cost.hs:94-100](../../../core/app/CE/Graph/Cost.hs#L94)); the default is empty. `"standing"` is refused
at load by name — the inventory never blocks: declare an entry or delete the file — and any other word
by name ([config/deadcode.rs:33-47](../../../cli/src/config/deadcode.rs#L33)); the list enters
`knobs_digest` (booklet 05). Write M for the files of D whose state the list names. A file is
**blocking** when it is a root *of M*: in M, and supported by every file of M that supports it — the root
law above with M in the place of D, read off the same condensation; the pairs of that second reading are
never built ([Point.hs:133-134](../../../core/app/CE/Graph/Point.hs#L133),
[Graph.hs:119](../../../core/app/CE/Graph.hs#L119)). **The guarantee:** the gate passes iff M is
empty, because a non-empty finite M has a root (follow supporters inside M upward through the finite
condensation). With `block = ["orphaned", "born"]` and a comparison point, the gate passes iff every
unreferenced file is standing. **Where this equals the roots-only reading, and where it differs.** When
M = D — every unreferenced file in a blocked state: no standing file, both states blocked — the blocking
files are D's roots exactly. When M is a proper part of D the two readings differ on the files of M that
are roots of M but dependents in D: a newly unreferenced file under a standing root (`s.ts`, unreferenced
when the baseline was written, comes to import `lib.ts`, which lost its live reference, and a new `n.ts`:
both block, though their one root is standing), and, under `block = ["born"]`, a born file whose only
supporter is an orphaned root (a new file nothing live reaches). Every root of D whose state the list
names is a root of M, so nothing the roots-only reading blocked stops blocking. `ce deadcode --check`
exits 1 when a file blocks — the reply's `fail` bit, which also reads a degraded judgment
([Lines.hs:163-164](../../../core/app/CE/Graph/Lines.hs#L163)), while a degraded judgment exits 2 with and
without `--check`, as every degrading family does (release 1.10.0,
[gate.rs:194](../../../cli/src/graph/deadcode/gate.rs#L194)) — and `ce check` holds its seventh fail
condition, `dead_added`, on the same files ([Faces.hs:33](../../../core/app/CE/Verdict/Faces.hs#L33)). A
declared `block` with no comparison point — no `ce-baseline.json`, or one written before the amendment
(the file on disk is what is read) — cannot be answered: both commands refuse by name with exit 2 and the
remedy, `CE_ACCEPT_BASELINE=1 ce baseline .` from the project root, then commit the file
([main_cmds.rs:142](../../../cli/src/main_cmds.rs#L142),
[score/mod.rs:320](../../../cli/src/score/mod.rs#L320)). With nothing declared an unreferenced file is
listed and fails nothing.

**The document and the faces.** `ce.deadcode-report` 0.5.0 carries, on each `dead` row, `root`,
`state` and `blocking`; on a dependent, `under` (its roots); in `counts`, `roots`, `dependents` and one
count per state over the unreferenced files; at the top, `gate = {block, comparison, blocked, answered}`
and, when the pairs were dropped, `under_dropped`
([Document.hs:213-220](../../../core/app/CE/Graph/Document.hs#L213)). The console prints the
roots first, each with the files that fall with it, then the dependents naming their roots; every file's
state is printed where it is listed, only when a comparison point rode, so a blocking dependent is marked
as a blocking root is. The GUI's graph screen and the MCP tool read the same document.

**The score.** The dead axis keeps its law and weighs 0 unless `[score.weights] deadcode` names it
(booklet 05).

**The evidence.** Generation 1 of the dead-file exam read 424 of the 559 existing dead files of 13
corpora: 392 were in use ([EVAL-SET-DEADFILE.md:243](../../EVAL-SET-DEADFILE.md#L243)). Generation 2
replayed the histories of ten repositories the product had never run on: 21 `orphaned` events and 90
`born` events, each fewer than the 100 a judged class needs, so neither is judged
([EVAL-SET-DEADFILE-GEN2.md:190-193](../../EVAL-SET-DEADFILE-GEN2.md#L190),
[EVAL-SET-DEADFILE-GEN2.md:245-248](../../EVAL-SET-DEADFILE-GEN2.md#L245)). No false-block rate is claimed
for any state, and a class with no record of its own does not block: blocking is a project's declaration.
This repository declares `block = ["orphaned", "born"]` and names the axis at weight 1, so its own gate is
as strict as it was before the amendment.

**What did not change**: which files are unreachable (the entry table, the ladders, the kept arcs);
`ce erase`, for which a verdict here is no deletion licence (book 12 §2); and the advisory readers —
`ce join`, the candidates, `ce structure`, the query prelude's `dead(F)`.
