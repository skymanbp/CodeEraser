# Unmentioned-declaration advisory — the mention veto

[index](../methodology.md) · [← 12 Deterministic erase — the safety predicate](12-deterministic-erase-the-safety-predicate.md)

The graph family (booklet 6) judges files. This family reads one level below it and
issues no verdict at all: for every judged declaration it asks whether any OTHER file in
the tree spells the declaration's name, and reports the ones nothing spells. It is a
**negative instrument** — the only thing it can say is "no static reference was seen",
so every rule leans toward counting a mention, and the one dangerous direction (a
referenced name read as unmentioned) is the direction every rule is built against
([mod.rs:1-13](../../../cli/src/mention/mod.rs#L1)). The output is an *advisory* beside
the four-way verdicts: it never turns a gate red, never enters `ce erase`, and is
rendered with that word on every surface
([Cost.hs:50-52](../../../core/app/CE/Graph/Cost.hs#L50),
[report.rs:53-58](../../../cli/src/graph/deadcode/report.rs#L53)). The plan calls it ADR-008
step 3b ([DEVELOPMENT_PLAN.md:290](../../DEVELOPMENT_PLAN.md#L290)); the split follows
ADR-008 as everywhere else — Rust measures (walks, tokenizes, stores hashes, extracts the
declaration's name and category), Haskell decides which rows come out and with which code.

### 1. The universe U — a second walk

The corpus that may spell a name is not the scan's file set: the scan's exclusions serve
"what is measured", the veto needs "what could reference a name", so U is a walk of its
own with every parameter frozen as a `MENTION_REV` input
([walk.rs:1-47](../../../cli/src/mention/walk.rs#L1), [mod.rs:63-85](../../../cli/src/mention/mod.rs#L63)):
hidden files enter, `.git`/`.ce` are cut by name, and a nested repository is read off ONE
owner predicate shared with the measurement walk and the guard's scope
([gitmodules.rs:89-132](../../../cli/src/gitmodules.rs#L89)): a path the root's
`.gitmodules` declares is **foreign** — in U, since it can spell a name, and never measured
(plan v2.18 step #12: the suite rides at `cli/tests` that way, a reader of this tree and no
part of its score); an undeclared nested repository is **cut** whole. One reader parses the
declaration with git's own config grammar
([gitmodules.rs:1-23](../../../cli/src/gitmodules.rs#L1)), and a declared checkout that is not
seated refuses by name rather than letting U shrink ([walk.rs:84-87](../../../cli/src/mention/walk.rs#L84)),
`.gitignore` and `.ceignore` are honoured and nothing else — not the walker's `.ignore`,
not global or parent ignore files, and `.git` is not required, so one commit yields one
U on any machine ([walk.rs:90-101](../../../cli/src/mention/walk.rs#L90)); the cut is one published predicate,
`cut`, that the walk's entry filter, the census and the formula below all read
([walk.rs:134-173](../../../cli/src/mention/walk.rs#L134)). Directory symlinks are not
followed; a file symlink is read through when its target is a regular file inside the
root, identity being the canonical relative path, so a link and its target enter once
([walk.rs:175-185](../../../cli/src/mention/walk.rs#L175),
[walk.rs:187-235](../../../cli/src/mention/walk.rs#L187)). Files over 4 MiB are skipped and
counted ([walk.rs:59](../../../cli/src/mention/walk.rs#L59)); the exclusion table is the
scan's secret globs plus four omni-mentioners (`*.map`, `tags`, `TAGS`, `*.po`) — files
whose purpose is to name every symbol ([walk.rs:64](../../../cli/src/mention/walk.rs#L64),
[walk.rs:241-258](../../../cli/src/mention/walk.rs#L241)). Generated and vendored trees are
NOT excluded: they are in U and outside the judged domain, which is the safe side.

The binary rule is git's: a UTF-16 BOM decodes, otherwise a NUL in the first 8000 bytes
skips the file; a later NUL keeps it, decoded lossily so one stray byte cannot lose a
file's mentions ([walk.rs:254-271](../../../cli/src/mention/walk.rs#L254)). The consequence
is stated, not hidden: PDF-disguised `.ai` assets whose first NUL falls past byte 8000
are in U (zod holds eight, requests one), and §8 prices what they add.

U is **pinned to a formula, never a literal**: git's listing under `.gitignore` alone
(`--cached --others --exclude-per-directory=.gitignore` — the walk turns `info/exclude`
and `core.excludesFile` off, so one machine's own exclude file must not move U) minus one
term per rule of the walk: the by-name cut, the nested-repository cut (git lists such a
tree as one `sub/` entry), the tracked files a `.gitignore` pattern matches (the walk
reads patterns and never the index, so such a file is outside U — zod has one), the
exclusion table, the entries no regular file backs (deleted unstaged, a link to a
directory), the size cap and the binary rule — each computed with the walk's own
published predicate (`cut`, `excluded`, `FILE_CAP`, `decode`), never a second reading of
it ([mention_universe.rs:33-63](../../../cli/tests/it/mention_universe.rs#L33),
[mention_universe.rs:99-146](../../../cli/tests/it/mention_universe.rs#L99)). Every term is witnessed once on a scratch
repository where the walk's count and the formula agree
([mention_universe.rs:209-245](../../../cli/tests/it/mention_universe.rs#L209)); the self corpus is pinned in CI
([mention_universe.rs:195](../../../cli/tests/it/mention_universe.rs#L195)); the four external corpora are pinned by the same
formula in the `--ignored` instrument leg, whose printed line carries every term so
`listed − Σ terms = U` closes inside it (§8).

### 2. Tokens — one run, three emitters

A run opens on a Unicode letter, `_` or `$` and continues over Unicode alphanumerics, `_`
and `$` ([token.rs:29-35](../../../cli/src/mention/token.rs#L29)). Three emitters read each
run and none feeds another: (i) the whole run; (ii) the script split — the run's maximal
ASCII-identifier pieces, so `调用$graph函数` yields `$graph` and never a bare `graph`;
(iii) the `$` arm — the run's maximal `$`-free pieces, for every extension OUTSIDE the
JS family ([token.rs:72-80](../../../cli/src/mention/token.rs#L72),
[token.rs:97-103](../../../cli/src/mention/token.rs#L97)). A piece that does not open a run
(digit-led) is dropped; a piece equal to the run is the run. The arm table is
`MENTION_WHOLE_RUN_EXTS`, looked up lower-cased, and no extension is the union arm
([token.rs:92-96](../../../cli/src/mention/token.rs#L92)). The reason the arm exists is a
cost the spec measured before sealing: in a `.ts` file `$ZodString` must stay whole (a
bare `ZodString` would mention its twin), while `exec $ce_entry_main` in a shell script
must keep `ce_entry_main`.

Two hashes are stored per distinct token: the fnv1a64 of the token, and — for tokens
of at least seven literal characters — the fnv1a64 of its fold key (`_`, `-` and `$`
filtered, lower-cased), a second chance for a Rust `zod_string` spelled `$ZodString`
elsewhere ([token.rs:109-120](../../../cli/src/mention/token.rs#L109),
[mod.rs:281-288](../../../cli/src/mention/mod.rs#L281)). No plaintext token enters the
database ([store.rs:32](../../../cli/src/mention/store.rs#L32)); the pass has its own
version row and any change to a frozen input re-derives every row
([mod.rs:85](../../../cli/src/mention/mod.rs#L85)). Two caps bound the store — 65,536
distinct tokens per file (a function of the bytes: the clip is final and the file's hash
is stored) and 4,194,304 rows per table (a function of the whole store: a starved file
gets neither rows nor hash and is retried every run) — and both are counted in the
header the operator sees ([mod.rs:87-92](../../../cli/src/mention/mod.rs#L87),
[mod.rs:250-278](../../../cli/src/mention/mod.rs#L250)).

### 3. The domain and the veto

The domain is every judged declaration whose key yields a **single-token** mention
name — the name the veto can look for at all. Arity suffixes are stripped, Python dunders
and multi-token keys (`foo'`, `(<+>)`, `r#type`, `"zod 3"`) are out of the domain on the
safe side ([name.rs:30-44](../../../cli/src/mention/name.rs#L30),
[name.rs:63](../../../cli/src/mention/name.rs#L63)); the unit is `(file, name)`, so a file
declaring one name twice is one candidate
([candidates.rs:163-191](../../../cli/src/mention/candidates.rs#L163)).

The veto asks three questions in a fixed order, cheapest first, and stops at the first
yes ([candidates.rs:89-111](../../../cli/src/mention/candidates.rs#L89)):

1. **another file spells it** — the identity hash occurs in a file other than the
   declaring one ([store.rs:225-227](../../../cli/src/mention/store.rs#L225));
2. **the fold second chance** — Rust only, and only for names with ≥ 2 segments
   (`_`/camel boundaries, an all-caps run one segment) and ≥ 7 characters
   ([token.rs:127-144](../../../cli/src/mention/token.rs#L127),
   [store.rs:231-233](../../../cli/src/mention/store.rs#L231));
3. **the file's own exception regions spell it** — Go template actions, TS string and
   template literals, Python doctests, Rust macro definitions and fenced doc blocks,
   Haskell haddock fences: text inside the declaring file that a loader or a reader
   treats as a reference ([selfref.rs:71-109](../../../cli/src/mention/selfref.rs#L71),
   [selfref.rs:206-264](../../../cli/src/mention/selfref.rs#L206)).

A survivor becomes one row keyed `[node, vis, conv]` with its names kept beside the key
on the Rust side — the wire carries integers only, never a name
([candidates.rs:46](../../../cli/src/mention/candidates.rs#L46),
[candidates.rs:115-147](../../../cli/src/mention/candidates.rs#L115)). The table is cut at
131,072 keys in wire order, the same rows every run, and the cut is reported as a fact
of its own ([candidates.rs:52-63](../../../cli/src/mention/candidates.rs#L52),
[candidates.rs:153-159](../../../cli/src/mention/candidates.rs#L153)).

**Blindness named.** A name declared in two files is counted mentioned by the second
declaration alone; the census reports how often that was the only speller
(`collision_saved`, [rates.rs:95-102](../../../cli/src/mention/rates.rs#L95)). Consistency
is per file, never per run: a file read before a name was added and the declaring file
read after it yields one false unmentioned that the next run converges away
([mod.rs:22-25](../../../cli/src/mention/mod.rs#L22)).

### 4. The category word — why a survivor may still be silent

Every survivor carries a twelve-bit category word; bits 0–10 are *exemptions* (a reason
the name is reached without being spelled), bit 11 is rendered and never exempts
([conv/mod.rs:35-67](../../../cli/src/mention/conv/mod.rs#L35)). The AST half is stored at
index time (`Ffi` for Rust export attributes and `extern`, Haskell `foreign export`, Go
`//export`; `Registration` for a decorator; `Member`; `DefaultExport`; `Ambient`; Rust
`cfg(test)` and `allow(dead_code)`) ([conv/mod.rs:95-104](../../../cli/src/mention/conv/mod.rs#L95)).
The name-table half is computed at wire time from the path, the name and the key: a test
file by path component, a `benches`/`examples` component only under a Cargo package
root, `conftest.py`/`Spec.hs`/`build.rs` and the `*_test.go` / `test_*.py` / `.test.` /
`.spec.` patterns ([conv/name.rs:29](../../../cli/src/mention/conv/name.rs#L29),
[conv/name.rs:116-135](../../../cli/src/mention/conv/name.rs#L116)); Python/Haskell `main`;
the framework `Protocol` names a loader spells for the author — Python unittest/xunit/
pluggy/Django hooks, TS file-form × export-name rows, Haskell `Paths_*` and hspec
([conv/name.rs:33-63](../../../cli/src/mention/conv/name.rs#L33),
[conv/name.rs:153-207](../../../cli/src/mention/conv/name.rs#L153)); a Go method's receiver
exportedness ([conv/name.rs:217-229](../../../cli/src/mention/conv/name.rs#L217)); and a
file-level `ce:allow(unmentioned) -- <why>` claim
([conv/name.rs:169-175](../../../cli/src/mention/conv/name.rs#L169)). Every bit is silence,
the safe direction.

### 5. Visibility, mounts and the core's code

The core reads two more integer facts per row. The visibility word is three bits: bit 0
is "exported" ([visibility/mod.rs:71](../../../cli/src/fourclass/visibility/mod.rs#L71)), bit 1 that the enclosing scopes let the
name out too ([visibility/mod.rs:73](../../../cli/src/fourclass/visibility/mod.rs#L73)), and bit 2 marks a restricted export
(`pub(crate)` and kin) ([visibility/mod.rs:75](../../../cli/src/fourclass/visibility/mod.rs#L75)). The **mounts** table is one row per node —
`[node, private, total, bits]` — computed for every node without exception: how many of
the file's `mod` mounts are private, how many mounts it has, whether a façade re-exports
it (a Rust `via_reexport` edge or a TS `export *` target — bit 0) and whether its own
package keeps it private (Go `package main` / `internal/`, a Cargo package with no lib
target, a cabal package with no library or a module only in `other-modules`, a Python
module whose path carries an underscore-led segment, dunders excepted — bit 1)
([mounts.rs:37-43](../../../cli/src/graph/mounts.rs#L37), [mounts.rs:61-90](../../../cli/src/graph/mounts.rs#L61),
[mounts.rs:117-123](../../../cli/src/graph/mounts.rs#L117), [mounts.rs:220-239](../../../cli/src/graph/mounts.rs#L220)).

Both tables ride `graph.request` as optional keys that live and die together — one
without the other is refused by name, first in the violation chain
([Contract.hs:65-66](../../../core/app/CE/Graph/Contract.hs#L65),
[Contract.hs:112-119](../../../core/app/CE/Graph/Contract.hs#L112)); each row is validated
for width, sign, bound and `private ≤ total`
([Advisory.hs:29-48](../../../core/app/CE/Graph/Advisory.hs#L29)). The core then emits
`exportUnmentioned = [[node, vis, conv, code]]` for every row whose visibility carries
both bits of `unmentionedVisMask = 3` — exported AND scope-exported: a `pub fn` inside a
private `mod` is unreachable from outside and not a public-surface question — and whose
category word has none of the exempt bits 0..10 (bit 11, a Rust `cfg` naming no `test`,
is rendered but never exempts) — the mask reads the
visibility word, the exemptions read the category word, and the two never cross
([Advisory.hs:61-70](../../../core/app/CE/Graph/Advisory.hs#L61),
[Cost.hs:194](../../../core/app/CE/Graph/Cost.hs#L194), [Cost.hs:208](../../../core/app/CE/Graph/Cost.hs#L208)).
The code is a frozen total order `1 > 2 > 3 > 0`: **1 private** when the file has at
least one mount, every mount is a private `mod` and no façade re-exports it, or the
package keeps it private; **2 restricted** on visibility bit 2; **3 reexported** on mounts
bit 0; else **0 public** — zero mounts is not private (a lib root, a Go or TS file), and a
missing mounts row reads `[0,0,0]`
([Advisory.hs:80-107](../../../core/app/CE/Graph/Advisory.hs#L80)).

Caps are priced like every other table: `mountCap` 131,072 (one row per node is the
structural ceiling), `unmentionedCap` 131,072 is a **soft** cap — past it the core still
judges the graph but drops the table and says so (`unmentionedDropped`), and the
producer cuts at the same number so the two can never disagree — and `unmentionedHardCap`
524,288 is the outer bound only a defective client reaches
([Cost.hs:41-74](../../../core/app/CE/Graph/Cost.hs#L41),
[Graph.hs:115-122](../../../core/app/CE/Graph.hs#L115)). The iron rule is two byte-level
facts: a request without the tables gets the ten-key reply unchanged, and the dead set is
the same with or without them ([VERSIONING.md:173-190](../../../contracts/VERSIONING.md#L173)).

### 6. Rendering — one home, three faces

Only `ce deadcode` and the GUI/MCP deadcode faces ask for the advisory; the five other
consumers of the graph wire (`erase`, `join`, `score`/`check`, `structure`, the canvas)
pass `Advisory::No`, each with its reason at the call site
([deadcode.rs:81-84](../../../cli/src/graph/deadcode.rs#L81),
[deadcode.rs:182-227](../../../cli/src/graph/deadcode.rs#L182)). The reply is consumed once:
each core row is looked up in the producer's own table (a key the producer never offered,
or a key without names, is a named wire-skew refusal), and a non-degraded reply without
the key is refused as a pre-6.2.0 core rather than read as "asked and clean"
([advisory.rs:83-146](../../../cli/src/graph/deadcode/advisory.rs#L83)). The report gains
three keys, present only when the road was asked: `unmentioned` rows of five scalars
`{name, symbol, line, code, why}`, `unmentioned_dropped`, and `unmentioned_cut` — the
producer's cut, which the core cannot see ([advisory.rs:31-50](../../../cli/src/graph/deadcode/advisory.rs#L31),
[report.rs:114-145](../../../cli/src/report.rs#L114)). The console prints one line per row, a
census line by code and, on either degradation, one local line saying which
([report.rs:53-109](../../../cli/src/graph/deadcode/report.rs#L53)); the MCP `deadcode`
tool returns the same document ([tools.rs:78-82](../../../cli/src/mcp/tools.rs#L78)); the GUI
graph screen loads that document as a second judgment beside the canvas one, joins the
two by file path (a rendering join on a shared string, never a verdict, and best-effort
by path since they are separate runs) and lists a selected file's rows with the
root-level census and the notices — the two the document carries, and a third when the
advisory road failed while the canvas drew (a pre-6.2.0 core), so "no advisory" and "not
judged" never look alike ([graph.js:34-69](../../../gui/ui/graph.js#L34),
[graph.js:202-275](../../../gui/ui/graph.js#L202), [i18n.js:81-91](../../../gui/ui/i18n.js#L81)).
A projection gate pins that the symbol column survives the hub's generic table
([hub_projection.js](../../../cli/tests/gui/hub_projection.js)).

### 7. Residual risks, stated

- **Twin blindness is by design.** A `$ZodString` in docs mentions `ZodString`; the union
  arm exists for that. §8 ① counts the declarations whose advisory the arm changes — 0 on
  all five trees, where every bare name the arm would rescue is spelled by its own file's
  string literals already; the arm's cost is row count alone.
- **File granularity.** A `pub(crate)` accessor called only inside its own file is a true
  row (nothing else spells it) and an invitation to narrow it, not a dead symbol; the
  ledger in §8 shows 31 such rows across four corpora and no dead-code claim is made.
- **Python module privacy became a mount fact in step 8** (plan v2.17 L round, ruling ⑤
  2026-08-28). The mounts table's Python arm reads underscore path segments
  ([mounts.rs:236-245](../../../cli/src/graph/mounts.rs#L236)); a literal `__all__` (`=` / `+=`
  of string lists or tuples, any non-literal form ⇒ the convention) narrows bit 0 to the
  names it lists ([visibility/py.rs:27-37](../../../cli/src/fourclass/visibility/py.rs#L27)) — the
  Haskell export-list precedent, and the same narrowing the underscore convention already
  is: a helper the module's own export list omits is not public API, so the erase refusal
  `public_surface` no longer holds it; a body under `if TYPE_CHECKING:` carries conv
  `Ambient` ([conv/py.rs:57-70](../../../cli/src/mention/conv/py.rs#L57)). requests `_types.py:157`
  now reads private and exempt and leaves the advisory; `__init__.py:60 check_compatibility`,
  public by convention but absent from the module's `__all__`, leaves it too (exported
  645 → <!--ce:restate:survival:requests-8068356:declared-exported#paren-->644<!--/ce-->, unmentioned-exported 432 → <!--ce:restate:survival:requests-8068356:unmentioned-exported#paren-->431<!--/ce-->); the requests population is 16 → 14.
- **Late-NUL binaries are in U** and lossily decoded (§1); their token cost is measured
  as its own column (§8), and the rule is not changed because git's own binary rule is
  the same 8,000 bytes.
- **The census is the veto's own survival**, before the core's mask and exemptions;
  the advisory rows are after them. The two numbers differ by construction
  ([rates.rs:1-15](../../../cli/src/mention/rates.rs#L1)).

### 8. Acceptance

**Universe and census (K23), measured 2026-08-28 with one instrument in one run** on the
pinned tips of [EVAL-SET-M5-3.md](../../EVAL-SET-M5-3.md) — every number below is copied
from the leg's own JSON line, which carries every term of the formula
([eval_mention.rs:48-90](../../../cli/tests/it/eval_mention.rs#L48),
[eval_support/mention.rs:146-188](../../../cli/tests/it/eval_support/mention.rs#L146)); the formula pin holds on all five trees. The
self row is the one corpus this booklet is a member of: it is re-taken on the commit that
ships the text (digits only, so the fixed point holds) and moves with the tree by design —
the pin is the formula, the row is the reading.

| corpus | U (listed − terms) | language | declared (exported) | unmentioned (exported) | survival | collision-saved / unmentioned | of by-other |
|---|---|---|---|---|---|---|---|
| self @ this commit | 764 (777 − 13 early-NUL) | rust | 2021 (1102) | 297 (0) | 14.7 % | 17 / 297 = 5.7 % | 17 / 1702 |
| | | haskell | 1333 (309) | 307 (2) | 23.0 % | 14 / 307 = 4.6 % | 14 / 1026 |
| | | python | 17 (17) | 0 (0) | 0.0 % | 0 / 0 | 0 / 17 |
| | | typescript | 5 (5) | 0 (0) | 0.0 % | 0 / 0 | 0 / 5 |
| cobra adbc881 | 65 (66 − 1 early-NUL) | go | 613 (481) | 403 (313) | 65.7 % | 4 / 403 = 1.0 % | 4 / 200 |
| requests 8068356 | 118 (130 − 7 excluded − 5 early-NUL) | python | 666 (644) | 450 (431) | 67.6 % | 18 / 450 = 4.0 % | 18 / 214 |
| ripgrep 3fce3b5 | 230 (237 − 7 early-NUL) | rust | 2501 (886) | 885 (47) | 35.4 % | 120 / 885 = 13.6 % | 120 / 1546 |
| zod 912f0f5 | 536 (583 − 45 early-NUL − 1 excluded − 1 pattern-ignored) | typescript | 1944 (1127) | 353 (197) | 18.2 % | 107 / 353 = 30.3 % | 107 / 1530 |
| | | tsx | 56 (37) | 12 (4) | 21.4 % | 8 / 12 = 66.7 % | 8 / 44 |

Survival = unmentioned / declared at the veto layer, and the collision-saved rate — of the
survivors' population, the share that only a same-name declaration in another file kept
out of the table — is the second number the criterion asked for (§0 clause 3: 存活/域,
碰撞得救/未提及); the last column restates the same count over the by-other vetoes, the
layer it is a partition of. The exported-only survival on the same rows is the extra the
operator reads for the public surface: self rust <!--ce:restate:survival:self-this-commit:unmentioned-exported#paren-->0<!--/ce--> / <!--ce:restate:survival:self-this-commit:declared-exported#paren-->1102<!--/ce--> = <!--ce:restate:survival:self-this-commit:unmentioned-exported/declared-exported#paren-pct1-->0.0<!--/ce--> % (the suite is a reader of
this tree since plan v2.18 step #12, so its declarations sit in its own domain, not here), zod typescript
<!--ce:restate:survival:zod-912f0f5:unmentioned-exported#paren-->197<!--/ce--> / <!--ce:restate:survival:zod-912f0f5:declared-exported#paren-->1127<!--/ce--> = <!--ce:restate:survival:zod-912f0f5:unmentioned-exported/declared-exported#paren-pct1-->17.5<!--/ce--> %, cobra <!--ce:restate:survival:cobra-adbc881:unmentioned-exported#paren-->313<!--/ce--> / <!--ce:restate:survival:cobra-adbc881:declared-exported#paren-->481<!--/ce--> = <!--ce:restate:survival:cobra-adbc881:unmentioned-exported/declared-exported#paren-pct1-->65.1<!--/ce--> %. The spread across languages — two thirds
of Go's exported surface is unspoken inside its own tree at this layer, most of
TypeScript's is spoken — is why the census is reported per language and never as one
number ([rates.rs:1-12](../../../cli/src/mention/rates.rs#L1)).

**The `$` arm's two costs and the JS arm's collateral**, same run, two universes (full U /
U without the `.ai`/`.eps` late-NUL PDFs):

| corpus | ② rows emitter (iii) alone adds | ① advisories the arm changes | JS-arm pieces silenced / with no other source / in the domain |
|---|---|---|---|
| self | 62 / 62 | 0 | 8 / 5 / **0** |
| cobra | 24 / 24 | 0 | 0 / 0 / 0 |
| requests | 449 / 3 | 0 | 0 / 0 / 0 |
| ripgrep | 61 / 61 | 0 | 0 / 0 / 0 |
| zod | 438 / 146 | 0 | 624 / 444 / **0** |

The pre-registration said ① = 0 everywhere, and it holds on all five trees once the
instrument asks the veto's three channels on both arms — identity, the Rust fold, the
declaring file's own exception regions — as the producer asks them. An identity-only
reading of the same run had shown two zod rows, `ZodBase64URL` (`v4/classic/schemas.ts:939`)
and `ZodExactOptional` (`:2148`), v4 classic interfaces whose bare name no other file
spells while their `$`-twin is spelled in `core.mdx` and `wiki/optionality.md`; both are
vetoed with the arm silent too, by their own file's string literal
(`$constructor("ZodBase64URL", …)`, `:943` / `:2155`), so the arm changes no advisory
anywhere and its cost is the row count above. The leg prints the rows beside the zero, so
a non-zero can be read ([eval_mention.rs:71-85](../../../cli/tests/it/eval_mention.rs#L71)); the domain-name
collateral of the JS arm is 0 on every tree, as pre-registered. requests' 449 → 3 is one
file, `ext/requests-logo.ai`. `$`-run shapes in zod's JS-family files: 2392 bare, 5416
leading, 119 trailing, 10 inner. The TEST rule's two pins hold: no external corpus has a
`test` (singular) component, and ripgrep's package-root rule fires on exactly
`crates/globset/benches`, `crates/{grep,ignore,searcher}/examples`
([eval_mention.rs:36-41](../../../cli/tests/it/eval_mention.rs#L36)). Protocol-table hits:
requests `setup`, `clean_proxy_environ`; zod `GET` ×3, `generateMetadata` ×2,
`generateStaticParams` ×2. The FFI/macro rows have no corpus witness and are pinned by
synthetic fixture instead ([conv/tests.rs:49-56](../../../cli/tests/unit/mention/conv/tests.rs#L49)).

**Every external advisory row dispositioned** (258 rows at the audit: cobra 4, requests 16,
ripgrep 41, zod 197 — the self rows are a plan step of their own): ten disposing agents each
followed by an independent refuter re-running the search; 8 dispositions corrected, **0 rows
where another file spelled the name** — the instrument's claim held on 258 / 258. Reading of
the rows: 218 public API surface, 2 loader-spelled (cobra's `Gt`/`Eq` reached through a
`text/template` FuncMap keyed `gt`/`eq` in another file), 5 test-only, 31 restricted or
private declarations used only in their own file or nowhere, 2 domain readings named in
§7 (requests `_types.py:157`, resolved by step 8; ripgrep `matcher.rs:548`, a same-file-only
`pub(crate)`). After step 8 the same instrument reads **257**: cobra 5 (`doc/man_docs.go:84
GenManTreeOptions`, a type form that entered the symbols domain only then), requests 14 (§7),
ripgrep 41 (7 of them now `reexported_unmentioned` through facades the widened ladder binds),
zod 197.

**Cost (K45)** — the mention pass is its own entry and not in `dedup::analyze`; the
consumers that never ask must not pay. A/B medians, n = 9 interleaved on two identical
HEAD trees with their own `.ce/`, old client (1f493df) vs this batch, quiet window:
`ce audit --hook` 1.186 s → 0.954 s, `ce erase` (plan) 1.526 s → 1.493 s, `ce check`
1.786 s → 1.802 s (spread 1.729–1.928) — no consumer slower
([PERF-BUDGET.md:188-197](../../PERF-BUDGET.md#L188)). The pass itself: cold ≈ 1.95 s, warm ≈
0.54 s on the self corpus ([PERF-BUDGET.md:54](../../PERF-BUDGET.md#L54)).

**Gates in CI**: the self-U formula pin and every term of the formula witnessed on a
scratch repository ([mention_universe.rs:209-245](../../../cli/tests/it/mention_universe.rs#L209)); the self
pre-registered zeros; the mentions face schema <!--ce:report:mentions#schemaver-->`ce.mentions-report/0.2.0`<!--/ce--> with its
`rates` key ([face.rs:15](../../../cli/src/mention/face.rs#L15)) and the face run as a
reader would — field names, the fold channel on a fixture, the console's nine holes in
both languages ([mentions_face.rs](../../../cli/tests/it/mentions_face.rs)); the census
counted on a synthetic tree with a collision told apart from a reference
([unit/mention/rates.rs:12-53](../../../cli/tests/unit/mention/rates.rs#L12)); the producer's cut flag; the 0.3.0 keys absent on every
pass-No road and the row-key order on the console and in the GUI projection
([deadcode_e2e.rs](../../../cli/tests/it/deadcode_e2e.rs),
[hub_projection.js](../../../cli/tests/gui/hub_projection.js)); and the soft cap pinned equal
source-to-source between `candidates.rs` and `Cost.hs`.
