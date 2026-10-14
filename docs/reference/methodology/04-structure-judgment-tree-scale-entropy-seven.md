# Structure judgment — tree-scale entropy, eight axes

[index](../methodology.md) · [← 03 Documentation duplication — shingling + MinHash/LSH](03-documentation-duplication-shingling-minhash.md) · [→ 05 Scoring and the ADR-006 ratchet](05-scoring-and-the-adr-006-ratchet.md)

The `structure/1` family judges the *tree*, not the file: directory geometry, naming
distributions, reference locality, documentation coverage, and — when the deep tables ride
the wire — doc staleness and redundancy rollups. Since plan v2.33 W1 item 3 Rust sends the
walked judged paths, the graph's file paths and arcs and the layout by path; the core builds
the directory tree and every dir-keyed table (CE.Structure.Raw), judges, and lays the document
out; the integer form survives only as the test seam ([Request.hs:14-21](../../../core/app/CE/Structure/Request.hs#L14)). Only the *judged*
language set enters the tree; the size-gate-only language arm is excluded, or the file
population would shift S0 geometry, S1 naming, S4 documentation and both entropy rows —
NOT because "S2 would call a front-end directory language-mixed", which blames an axis that
has no language term at all ([judge.rs:95-100](../../../cli/src/structure/judge.rs#L95)) ([structure-axes.md:29-32](../structure-axes.md#L29)).
This file keeps its original `-seven` name because that anchor is linked from outside the
repository; the axis roster itself has been eight since proto 7.1.0 added modularity (S7).

### 1. The entropy primitive: exact rationals, no logarithms

Shannon entropy and KL divergence require logarithms — irrational, therefore not exactly
decidable, and banned along with every float by the core's integer discipline. The family
therefore publishes the rational-closed members of the same families
([Entropy.hs:1-15](../../../core/app/CE/Structure/Entropy.hs#L1)): Tsallis-2 entropy (= Gini–Simpson
diversity) and the χ² f-divergence. All arithmetic runs over `Data.Ratio`
([Entropy.hs:24](../../../core/app/CE/Structure/Entropy.hs#L24)).

**Tsallis-2 diversity** over a count vector, `N = Σc`
([Entropy.hs:29-35](../../../core/app/CE/Structure/Entropy.hs#L29)):

```
tsallis2 cs = 1 - Σ (c/N)^2        -- and = 0 when N == 0
```

**Normalized** against the `n`-bin maximum `1 - 1/n` (attained exactly on the uniform
distribution), where `n` = the number of *nonzero* bins
([Entropy.hs:40-45](../../../core/app/CE/Structure/Entropy.hs#L40)):

```
tsallis2Norm cs = tsallis2 cs / (1 - 1/n)     -- n > 1
tsallis2Norm cs = 0                           -- n <= 1  (nothing to normalize)
```

**χ² divergence** of observed counts against reference counts, paired by bin position, with
`p = o/Σo` and `q = r/Σr` ([Entropy.hs:55-66](../../../core/app/CE/Structure/Entropy.hs#L55)):

```
chi2 pairs = Σ_{r > 0} (p - q)^2 / q
```

with four boundary arms, in evaluation order
([Entropy.hs:56-60](../../../core/app/CE/Structure/Entropy.hs#L56)):

| condition | result | reading |
|---|---|---|
| `Σo == 0 && Σr == 0` | `Just 0` | nothing diverges from nothing |
| `Σr == 0` | `Nothing` | no reference to judge against |
| any bin with `r == 0 && o > 0` | `Nothing` | observed mass on a zero-reference bin — divergence is infinite there |
| `Σo == 0` | `Just 1` | equals `Σ q` over the support |

`Nothing` is a refusal, not a zero. On the DECLARED road it is also unreachable — weights are
validated `>= 1`, so no reference bin can be zero — and what actually withholds the number
there is `Declared.hs`'s own `unowned` list, which drops `divergence` to `[]` and names the
directories instead (§3 below states the mechanism correctly; the stance is the same either
way, the trigger is not) ([Entropy.hs:49-53](../../../core/app/CE/Structure/Entropy.hs#L49),
[Declared.hs:40-42](../../../core/app/CE/Structure/Declared.hs#L40)).

**Publishing scale.** Every exact rational reaches the wire through one deterministic
truncation ([Entropy.hs:71-72](../../../core/app/CE/Structure/Entropy.hs#L71)):

```
perMille r = floor (r * 1000)
```

The algebra is proved against pair *enumeration* by an exhaustive reference battery — e.g.
`tsallis2Norm (replicate n 2) == 1`, `tsallis2Norm [5] == 0`, `chi2 [(1,0),(0,1)] == Nothing`,
`chi2 [(0,1),(0,3)] == Just 1`, `chi2 [(3,2),(1,2)] == Just (1/4)`
([EntropyProps.hs:85-116](../../../core/test/EntropyProps.hs#L85)).

**Headline entropy rows.** The reply carries exactly two, both normalized Tsallis-2 in
per-mille ([Axes.hs:246-254](../../../core/app/CE/Structure/Axes.hs#L246)):

- row `0` — the *global* naming distribution: pattern counts summed by pattern code over every
  directory;
- row `1` — the file-count distribution across directories, zero-file directories filtered out.

### 2. The self-referential floor

The phrase names the mode in which the family runs with **no external template**: the eight
axes and the two entropy rows read nothing but the tree's own fact tables, so the tree is
judged against itself. The only in-tree definition of the term is the config comment at
[config.rs:102-108](../../../cli/src/config.rs#L102) — "the OPTIONAL layout template the χ² divergence
judges against. Absent = the self-referential floor alone (row C)". *Caveat for the reader:*
the design booklet's row A/B/C taxonomy that the phrase indexes is **not** in the tree — the
booklet was distilled into `structure-axes.md` and its full text lives only in git history
([structure-axes.md:3-6](../structure-axes.md#L3)). No other definition of "row C"
or of "self-referential floor" exists in the repository (verified by grep this run: the only other hit in tracked source is
[Boot2.hs:46](../../../core/app/CE/Lang/Common/Boot2.hs#L46), a Haskell module list where
"Arrow Control.Category" spells the substring by accident).

Mechanically the floor is a shape guarantee: the A-layer keys `divergence` and `deviations`
exist **only** when the request declares a layout; an undeclared request answers the S2 shape
byte for byte ([Structure.hs:180-184](../../../core/app/CE/Structure.hs#L180),
[Declared.hs:16-23](../../../core/app/CE/Structure/Declared.hs#L16)), and the battery asserts both keys
absent on the undeclared fixture ([StructureProps.hs:158-166](../../../core/test/StructureProps.hs#L158)).

### 3. Declaration coverage — the χ² overlay (S3a A-layer)

`ce.toml`'s `[structure.layout]` maps directory paths to relative weights `>= 1`; `"."` is the
root and, under deepest-owner semantics, the catch-all bin
([config.rs:102-113](../../../cli/src/config.rs#L102), [ce.toml:9-23](../../../ce.toml#L9)). Rust sends
it as `(path, weight)` pairs; the core resolves each path to its directory id (refusing a path
that names no walked directory), sorts, then checks arity 2, non-negativity, `dirId < |nodes|`,
`weight >= 1`, and strict ascent by `dirId`
([Structure.hs:77](../../../core/app/CE/Structure.hs#L77),
[Structure.hs:120-123](../../../core/app/CE/Structure.hs#L120),
[Structure.hs:156-169](../../../core/app/CE/Structure.hs#L156)).

Ownership resolves by depth, never by guess. Ids are dense and parents precede children, so one
left fold suffices: a directory owns itself when declared, otherwise it inherits its parent's
owner; the root has no fallback ([Declared.hs:49-56](../../../core/app/CE/Structure/Declared.hs#L49)).
Observed mass is the file count folded onto owners
([Declared.hs:27-30](../../../core/app/CE/Structure/Declared.hs#L27)):

```
ownedBy[o]  = Σ { files(i) : owner(i) == o }
bins        = [ (ownedBy[d], w) | (d, w) <- declared ]     -- observed vs reference
divergence  = [ perMille (chi2 bins) ]                     -- one row, or []
```

Two deviation kinds are emitted instead of, or alongside, the number
([Declared.hs:31-35](../../../core/app/CE/Structure/Declared.hs#L31)):

| row | kind | meaning |
|---|---|---|
| `[dirId, 0]` | unowned | directory holds files (`files > 0`) but no declared ancestor owns it |
| `[dirId, 1]` | empty | declared bin that owns zero files — an expectation with no reality |

If **any** unowned directory exists, `divergence` is `[]` — the whole number is withheld and the
kind-0 rows say where the undeclared mass sits
([Declared.hs:36-39](../../../core/app/CE/Structure/Declared.hs#L36)). When every file is owned, the
weights are already validated `>= 1`, so `chi2`'s zero-reference arm is unreachable and the
`maybe` merely keeps the call total ([Declared.hs:40-42](../../../core/app/CE/Structure/Declared.hs#L40)).
Deviation rows are emitted sorted ([Declared.hs:24](../../../core/app/CE/Structure/Declared.hs#L24)).

Worked example on the battery fixture, every digit hand-checked
([StructureProps.hs:146-176](../../../core/test/StructureProps.hs#L146)): declared `[(0,1),(1,2),(2,3)]`,
owned file counts `(3, 9, 6)` of 18 → `p = (1/6, 1/2, 1/3)`, `q = (1/6, 1/3, 1/2)`,
`χ² = (1/6)²/(1/3) + (1/6)²/(1/2) = 5/36` → `138‰`. Weights `(1,3,2)` match the tree exactly →
`0`. Declaring dir 1 alone leaves root and dir 2 unowned → `divergence = []`, deviations
`[[0,0],[2,0]]`.

### 4. The eight axes

Each axis is one named predicate over the validated fact tables, owning its knob(s), so the
perturbation battery has a lever per row ([Axes.hs:1-7](../../../core/app/CE/Structure/Axes.hs#L1)).
S0–S4 are always judged; 5, 6 and 7 are judged **exactly when** their fact table rode the wire —
absent table (`Nothing`) means unjudged, empty table (`Just []`) means judged clean
([Axes.hs:38-50](../../../core/app/CE/Structure/Axes.hs#L38),
[Axes.hs:122-124](../../../core/app/CE/Structure/Axes.hs#L122)).

Every axis penalty is a count of **directories**, axis 3 included — booklet amendment ①
(user decision 2026-08-19, effective v0.5.0). Before it, S3 counted files, and one junk drawer could drive the whole structure score to 0
and drown every other axis (the illustration's "500 files" is a doc-side figure with no
fixture behind it; under today's density law a charge is bounded by `scale`, so no single
axis can zero the score at all); files remain the *measured* unit, the directory is the *judged* unit
([Axes.hs:128-134](../../../core/app/CE/Structure/Axes.hs#L128),
[structure-axes.md:34-39](../structure-axes.md#L34)). The migration moved this repo's own score 990 → 992 under the THEN-current raw-mass fold;
2.26.0's density fold re-migrated every structure score again, so neither figure describes
the score today ([structure-axes.md:49-50](../structure-axes.md#L49)).

| axis | fact row | predicate | knobs (code, default) |
|---|---|---|---|
| **S0** geometry | `[id,parent,depth,subdirs,files]` | `depth > depthCeil \|\| subdirs + files > fanoutCeil` | 0 `depthCeil=8`, 1 `fanoutCeil=30` |
| **S1** naming | `[dirId,patternCode,count]` | `Σcs >= namingMin && perMille (tsallis2Norm cs) > namingCeil` | 2 `namingMin=5`, 3 `namingCeil=600` |
| **S2** mixing | `[dirId,inside,outside,count]` | `ins + outs >= mixRefFloor && outs > ins` | 4 `mixRefFloor=5` |
| **S3** misplacement | `[dirId,inside,outside,count]` | per file `outside >= misplaceMin && outside > 2*inside`, then dedup to dirs | 5 `misplaceMin=3` |
| **S4** documentation | `[id,…,files]` + `[dirId,bits]` | `(files >= bigDirFloor && even bits) \|\| (id == 0 && bits < 2)` | 6 `bigDirFloor=8` |
| **S5** staleness | wire `staleDocs` by path, core-built into `[dirId,docTs]` + `[docIdx,targetTs]` and derived into `[dirId,stale,total]` | `stale >= staleMin` | 11 `staleMin=1` |
| **S6** redundancy | wire `clonePairs` + `deadPaths` by path, core-built into `[dirId,dupBlocks,deadUnits]` | `dupBlocks >= dupMin \|\| deadUnits >= deadMin` | 9 `dupMin=1`, 10 `deadMin=1` |
| **S7** modularity | `[fromDir,toDir,count]` + `fileRefs`' `inside` sums, both core-built from the one arc list | `mu >= modMassFloor && mu < m && 1000*(e*m − o*i) < modFloor*mu*(m−mu)` | 19 `modFloor=1`, 20 `modMassFloor=4` |

Sources: predicates [Axes.hs:163-174](../../../core/app/CE/Structure/Axes.hs#L163),
[Axes.hs:176-181](../../../core/app/CE/Structure/Axes.hs#L176),
[Axes.hs:185-194](../../../core/app/CE/Structure/Axes.hs#L185),
[Axes.hs:200-212](../../../core/app/CE/Structure/Axes.hs#L200),
[Axes.hs:216-228](../../../core/app/CE/Structure/Axes.hs#L216),
[Axes.hs:232-240](../../../core/app/CE/Structure/Axes.hs#L232); constants
[Cost.hs:35](../../../core/app/CE/Structure/Cost.hs#L35), [Cost.hs:40](../../../core/app/CE/Structure/Cost.hs#L40),
[Cost.hs:46](../../../core/app/CE/Structure/Cost.hs#L46), [Cost.hs:52](../../../core/app/CE/Structure/Cost.hs#L52),
[Cost.hs:58](../../../core/app/CE/Structure/Cost.hs#L58), [Cost.hs:65](../../../core/app/CE/Structure/Cost.hs#L65),
[Cost.hs:70](../../../core/app/CE/Structure/Cost.hs#L70), [Cost.hs:104-105](../../../core/app/CE/Structure/Cost.hs#L104),
[Cost.hs:92-93](../../../core/app/CE/Structure/Cost.hs#L92), [Cost.hs:98-99](../../../core/app/CE/Structure/Cost.hs#L98);
knob codes [Knobs.hs:32-56](../../../core/app/CE/Structure/Knobs.hs#L32).

Notes on the non-obvious ones:

- **S1** builds its per-directory count vector by grouping the pattern rows on `dirId`
  ([Axes.hs:193-194](../../../core/app/CE/Structure/Axes.hs#L193)); only pattern *codes* are judged —
  names never enter the judgment, and since 7.2.0 (plan v2.30 step 7b) neither does the producer's
  classification: since 9.0.0 the core computes each stem's seven character-class bits itself
  (CE.Structure.Tree `shapeBits`) from the paths — a `patternShapes` table survives only on the
  integer test seam — and folds them to codes through its STYLE table
  ([Shape.hs:31-44](../../../core/app/CE/Structure/Shape.hs#L31),
  [Request.hs:184-187](../../../core/app/CE/Structure/Request.hs#L184)). Bits are bounded `0..127`
  by the boundary contract ([Structure.hs:105-111](../../../core/app/CE/Structure.hs#L105)); the
  producer-classified `patterns` road (codes `0..6`) was retired at 8.0.0 and is refused by
  name ([Structure.hs:72-79](../../../core/app/CE/Structure.hs#L72)). `600‰` "tolerates one odd name in a
  convention-following set and flags a genuine style mix"
  ([Cost.hs:49-52](../../../core/app/CE/Structure/Cost.hs#L49)).
- **S2** uses *one basis on both sides* — per-file touch counts, never edges-vs-touches. Each
  `fileRefs` row contributes `(inside * count, outside * count)`, summed per directory
  ([Modularity.hs:44-48](../../../core/app/CE/Structure/Modularity.hs#L44)).
- **S3**'s `> 2*inside` ratio is part of the predicate's definition in v1, not a separate knob
  ([Cost.hs:62-65](../../../core/app/CE/Structure/Cost.hs#L62),
  [Axes.hs:220-228](../../../core/app/CE/Structure/Axes.hs#L220)). Directories are deduped via a map's
  key set ([Axes.hs:217](../../../core/app/CE/Structure/Axes.hs#L217)).
- **S4** convention bits are a mask: `1 = README`, `2 = config`; `even bits` means the README bit
  is clear, and the root additionally owes a recognized config
  ([Axes.hs:230-240](../../../core/app/CE/Structure/Axes.hs#L230)). Bits are constrained to `1..3` at the
  boundary ([Structure.hs:112-115](../../../core/app/CE/Structure.hs#L112)).
- **S6** never re-derives duplication or dead code: it convolves the per-file families' verdicts
  to the tree scale ([Axes.hs:167-168](../../../core/app/CE/Structure/Axes.hs#L167)).
- **S7** (7.1.0, plan v2.29 step 10, O54) is the only axis that reads *edges*. The core builds
  `fileRefs` and `dirEdges` from the one arc list the request carries (CE.Structure.Raw); the
  `dirEdges` table carries the **crossing** mass — `[fromDir, toDir, count]`, `from != to`,
  ascending — and the intra mass rides nowhere: an intra edge adds 1 to `inside` at *both* of its
  ends, so a directory's `fileRefs` inside sum is exactly twice its internal edge count, and a
  number with two owners is a number that can disagree with itself. What licenses the derivation
  is a boundary law over the pair, enforced whenever `dirEdges` is present — on the path form
always: every directory's inside
  sum is even, and its outside sum equals the `dirEdges` mass incident to it in either direction
  ([Modularity.hs](../../../core/app/CE/Structure/Modularity.hs)).

  With `e` internal edges, out-mass `o`, in-mass `i`, incident mass `mu = e + crossOut + crossIn`
  and `m` edges in the tree, Newman's contribution of a directory is `q = e/m − o*i/m^2`, and the
  most a directory of mass `mu` could earn — every incident edge internal — is
  `qMax = mu(m−mu)/m^2`. The judged quantity is the **ratio**

  ```
  rho = q / qMax = (e*m − o*i) / (mu*(m − mu))
  ```

  `1000‰` for a directory that never reaches out, `0` exactly at the null model's expectation,
  negative below it. The ratio, not `q` itself, is what the floor bites on, because `Σ_c q_c = Q ≤ 1`:
  a floor on `q` alone would flag a fixed *fraction* of any large tree however modular it is — the
  same raw-mass failure the 2.26.0 density law retired one section below. Under `modMassFloor`
  incident edges, and at `mu == m` (a directory incident to every edge, where `qMax` is 0 and there
  is no complement to be separate from), the directory is **absent from the axis**: never clean,
  never flagged.

  S7 is not S2 restated. S2's `outs > ins` on touch counts *is* `inside share < 1/3`, a fixed
  locality line with no size correction; S7 sits at the null model, so what it adds is the
  **monolith** — a directory holding most of the tree's edges internally passes S2 loudly and still
  fails S7, because most of the degree mass is its own. One phenomenon, one axis
  ([structure-axes.md:45-47](../structure-axes.md#L45)).

**Drill-down.** The same predicate lists feed the sparse `[dirId, axis]` finding rows the GUI
tree colours by — one list per axis, in axis order
([Axes.hs:135-144](../../../core/app/CE/Structure/Axes.hs#L135)).

### 5. The fold: charges to one score

Since contract 2.26.0 (M9 batch 9 P9, user ruling — a VERSIONING entry, not a proto
version; the protocol is far past it) the structure family runs the **same density
law as the verdict family**: each axis pairs its flagged-directory count `v` with the one
opportunity every structure axis shares — the directory total `N` — and maps the odds
through `chargeAt`, imported from [Score.hs:161](../../../core/app/CE/Verdict/Score.hs#L161)
(one law, two families; [Structure.hs:228-239](../../../core/app/CE/Structure.hs#L228)):

```
charge_i = floor(scale * v_i / (v_i + N))
raw      = Σ_axes (charge_i * violCost)
score    = max 0 (scale - raw `div` (structViolCostNeutral * judgedAxisCount))
```

with `violCost = 10` (knob 7, the family's strictness dial) and its structural neutral
`structViolCostNeutral = 10` ([Cost.hs:76-77](../../../core/app/CE/Structure/Cost.hs#L76),
[Cost.hs:85-86](../../../core/app/CE/Structure/Cost.hs#L85)): at the
neutral default the fold is the scale minus the FLOOR of the mean of the bounded charges (each
charge itself floored once), at most one point above the scale minus the exact mean — lenient by
less than one — and it cannot saturate.
`scale = 1000` (knob 8). All arithmetic is exact `Rational` then integer `div` — no float
anywhere on the path. `judgedAxisCount` is 5 to 8 depending on which optional tables
rode. The axis rows the reply carries are the **charges** (‰ of scale), the verdict-family
grammar; the flagged directories themselves remain in `findings`, unchanged.

The raw-mass fold this replaced (`scale - Σ(count*violCost) div axes`) was the exact shape
the M9 batch-6 field test retired in verdict/1: a mean flagged count of 100 directories
pinned the score at 0, and the same violation *rate* on a 10× repo cost 10× the penalty.
Density is scale-free: same rate, same score.

Worked end-to-end on the battery fixture — root (2 subdirs, 3 files, README+config), dir 1
(9 files, 5 snake + 4 pascal, two files with 4 outside refs each, no README), dir 2 (6 files,
uniform names), dir 3 (empty, depth 2), so `N = 4`
([StructureProps.hs:41-51](../../../core/test/StructureProps.hs#L41)):

```
axes    = [[0,0],[1,200],[2,200],[3,200],[4,200]]  -- v=1 at N=4: floor(1000/5)=200
raw     = 800 * 10
score   = 1000 - 8000 `div` (10*5) = 840
entropy = [[0,782],[1,916]]                        -- patterns [11,4]; dir files [3,9,6]
findings= [[1,1],[1,2],[1,3],[1,4]]
```

asserted against the real `respond` at
[StructureProps.hs:83-91](../../../core/test/StructureProps.hs#L83). Adding an empty `redundancy` table
makes it six judged axes: `1000 - 8000 div 60 = 867`; both optional tables plus flagged rows
give seven axes, charges `800+200+333`, `1000 - 13330 div 70 = 810`
([StructureProps.hs:191-228](../../../core/test/StructureProps.hs#L191)).

**Knob echo.** All 21 knobs (codes `0..20`) live in one authority table of
`(code, getter, setter)` triples, so the effective fold and the reply's echo read the same rows
and a knob cannot exist in one direction only
([Knobs.hs:25-56](../../../core/app/CE/Structure/Knobs.hs#L25)); rows outside `0..20` or with value `< 1`
are refused by name ([Knobs.hs:18-23](../../../core/app/CE/Structure/Knobs.hs#L18)). `ce.toml` is the
source, `Cost.hs` the defaults ([Cost.hs:1-6](../../../core/app/CE/Structure/Cost.hs#L1)), and the reply
echoes the full effective set ([Structure.hs:199](../../../core/app/CE/Structure.hs#L199)). Codes `12..18`
(`seamSoft=300`, `seamHard=750`, `seamPMax=10`, `roiRefMilli=250`, `roiPhiMilli=500`,
`roiCloneMilli=500`, `roiChurnMilli=150` —
[Cost.hs:133-178](../../../core/app/CE/Structure/Cost.hs#L133)) belong to the split-ROI advisory, not to
any axis, and never enter this fold.

### 6. Preconditions the fold assumes

The score above is only meaningful because the boundary contract runs first, in request order,
and returns the *first* offender by name ([Structure.hs:71-90](../../../core/app/CE/Structure.hs#L71)):

- node rows are dense and tree-shaped: `id == index`, no negative fields, root self-loops at
  depth 0, `parent < id` for every non-root row
  ([Structure.hs:125-138](../../../core/app/CE/Structure.hs#L125));
- `depth == parent.depth + 1` is *checked*, not assumed. It was previously only claimed in a
  docstring, and a forged row `[1,0,999,0,1]` rode straight into the geometry axis and moved the
  score (review 2026-08-20 #6) ([Structure.hs:140-154](../../../core/app/CE/Structure.hs#L140),
  probe at [StructureProps.hs:124-128](../../../core/test/StructureProps.hs#L124));
- the six `dirTables` share one checker — arity, non-negativity, `dirId < |nodes|`, a
  per-table extra rule, and strict ascent
  ([Structure.hs:77-79,96-103](../../../core/app/CE/Structure.hs#L77),
  [Structure.hs:156-169](../../../core/app/CE/Structure.hs#L156)); `staleDocRows` shares the
  ROW checker but orders **non-descending**, because one directory holds many docs
  ([Stale.hs:26-29](../../../core/app/CE/Structure/Stale.hs#L26)). Extra rules: (7.2.0) shape bits
  `<= 127` with count `>= 1` — the pre-classified `patterns` table is refused by name since 8.0.0;
  convention bits in `1..3`; `fileRefs` count `>= 1`; declared weight `>= 1`
  (the pre-judged staleDocs rules retired with their table at 2.29.0 — the raw
  `staleDocRows`/`staleEdgeRows` validators own staleness now)
  ([Structure.hs:96-123](../../../core/app/CE/Structure.hs#L96)).

**Over-cap.** `structNodeCap = 524288` ([Cost.hs:180-183](../../../core/app/CE/Structure/Cost.hs#L180)).
Node rows, the seam tables *and* (7.1.0) the dir-edge table count against the same cap — a declared
cap that misses a request dimension walks that dimension uncapped
([Structure.hs:57-62](../../../core/app/CE/Structure.hs#L57)). Since plan v2.33 W1 item 3 the core
prices the tables it built from the paths, and a path-form reply that degrades carries the rows it
priced, which the measuring side prints in its refusal
([wire.rs:172-182](../../../cli/src/structure/wire.rs#L172)). Over-cap answers a **complete degraded
reply that fails**: facts are emptied, the A-layer and split keys drop, `fail` and `degraded`
are both true and `reason` is `structure_too_large`
([Structure.hs:180-200](../../../core/app/CE/Structure.hs#L180),
[StructureProps.hs:246-255](../../../core/test/StructureProps.hs#L246)). Note the consequence of the
empty-facts path: five axes at penalty 0, hence `score = 1000` with `fail = true` — the score is
not evidence of health in a degraded reply. No `ce structure` user ever sees that 1000: the CLI
turns a degraded reply into an error before rendering ([wire.rs:181](../../../cli/src/structure/wire.rs#L181)),
so the number matters to a second client of the protocol, not to this one's console. In the non-degraded case `fail` equals `degraded`,
i.e. always false: S2 is report-only, and the CLI gates nothing on this score
([Structure.hs:178-180](../../../core/app/CE/Structure.hs#L178)).
