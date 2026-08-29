# Structure judgment — tree-scale entropy, seven axes

[index](../methodology.md) · [← 03 Documentation duplication — shingling + MinHash/LSH](03-documentation-duplication-shingling-minhash.md) · [→ 05 Scoring and the ADR-006 ratchet](05-scoring-and-the-adr-006-ratchet.md)

The `structure/1` family judges the *tree*, not the file: directory geometry, naming
distributions, reference locality, documentation coverage, and — when the deep tables ride
the wire — doc staleness and redundancy rollups. Rust measures and re-labels; Haskell
decides. Names never cross the wire — the request's vocabulary is dense directory ids,
codes and counts ([Structure.hs:12-15](../../../core/app/CE/Structure.hs#L12)). Only the *judged*
language set enters the tree; the size-gate-only language arm is excluded, or the file
population would shift S0 geometry, S1 naming, S4 documentation and both entropy rows —
NOT because "S2 would call a front-end directory language-mixed", which blames an axis that
has no language term at all ([judge.rs:206-211](../../../cli/src/structure/judge.rs#L206)) ([structure-axes.md:26-27](../structure-axes.md#L26)).

### 1. The entropy primitive: exact rationals, no logarithms

Shannon entropy and KL divergence require logarithms — irrational, therefore not exactly
decidable, and banned along with every float by the core's integer discipline. The family
therefore publishes the rational-closed members of the same families
([Entropy.hs:1-15](../../../core/app/CE/Structure/Entropy.hs#L1)): Tsallis-2 entropy (= Gini–Simpson
diversity) and the χ² f-divergence. All arithmetic runs over `Data.Ratio`
([Entropy.hs:23](../../../core/app/CE/Structure/Entropy.hs#L23)).

**Tsallis-2 diversity** over a count vector, `N = Σc`
([Entropy.hs:28-34](../../../core/app/CE/Structure/Entropy.hs#L28)):

```
tsallis2 cs = 1 - Σ (c/N)^2        -- and = 0 when N == 0
```

**Normalized** against the `n`-bin maximum `1 - 1/n` (attained exactly on the uniform
distribution), where `n` = the number of *nonzero* bins
([Entropy.hs:39-44](../../../core/app/CE/Structure/Entropy.hs#L39)):

```
tsallis2Norm cs = tsallis2 cs / (1 - 1/n)     -- n > 1
tsallis2Norm cs = 0                           -- n <= 1  (nothing to normalize)
```

**χ² divergence** of observed counts against reference counts, paired by bin position, with
`p = o/Σo` and `q = r/Σr` ([Entropy.hs:54-65](../../../core/app/CE/Structure/Entropy.hs#L54)):

```
chi2 pairs = Σ_{r > 0} (p - q)^2 / q
```

with four boundary arms, in evaluation order
([Entropy.hs:55-59](../../../core/app/CE/Structure/Entropy.hs#L55)):

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
way, the trigger is not) ([Entropy.hs:48-52](../../../core/app/CE/Structure/Entropy.hs#L48),
[Declared.hs:40-42](../../../core/app/CE/Structure/Declared.hs#L40)).

**Publishing scale.** Every exact rational reaches the wire through one deterministic
truncation ([Entropy.hs:69-70](../../../core/app/CE/Structure/Entropy.hs#L69)):

```
perMille r = floor (r * 1000)
```

The algebra is proved against pair *enumeration* by an exhaustive reference battery — e.g.
`tsallis2Norm (replicate n 2) == 1`, `tsallis2Norm [5] == 0`, `chi2 [(1,0),(0,1)] == Nothing`,
`chi2 [(0,1),(0,3)] == Just 1`, `chi2 [(3,2),(1,2)] == Just (1/4)`
([EntropyProps.hs:85-116](../../../core/test/EntropyProps.hs#L85)).

**Headline entropy rows.** The reply carries exactly two, both normalized Tsallis-2 in
per-mille ([Axes.hs:217-225](../../../core/app/CE/Structure/Axes.hs#L217)):

- row `0` — the *global* naming distribution: pattern counts summed by pattern code over every
  directory;
- row `1` — the file-count distribution across directories, zero-file directories filtered out.

### 2. The self-referential floor

The phrase names the mode in which the family runs with **no external template**: the seven
axes and the two entropy rows read nothing but the tree's own fact tables, so the tree is
judged against itself. The only in-tree definition of the term is the config comment at
[config.rs:150-156](../../../cli/src/config.rs#L150) — "the OPTIONAL layout template the χ² divergence
judges against. Absent = the self-referential floor alone (row C)". *Caveat for the reader:*
the design booklet's row A/B/C taxonomy that the phrase indexes is **not** in the tree — the
booklet was distilled into `structure-axes.md` and its full text lives only in git history
([structure-axes.md:3-6](../structure-axes.md#L3)). No other definition of "row C"
or of "self-referential floor" exists in the repository (verified by grep this run: the only other hit in tracked source is
[hs_boot.rs:237](../../../cli/src/graph/ladder/hs_boot.rs#L237), a Haskell module list where
"Arrow Control.Category" spells the substring by accident).

Mechanically the floor is a shape guarantee: the A-layer keys `divergence` and `deviations`
exist **only** when the request declares a layout; an undeclared request answers the S2 shape
byte for byte ([Structure.hs:218-222](../../../core/app/CE/Structure.hs#L218),
[Declared.hs:16-23](../../../core/app/CE/Structure/Declared.hs#L16)), and the battery asserts both keys
absent on the undeclared fixture ([StructureProps.hs:143-151](../../../core/test/StructureProps.hs#L143)).

### 3. Declaration coverage — the χ² overlay (S3a A-layer)

`ce.toml`'s `[structure.layout]` maps directory paths to relative weights `>= 1`; `"."` is the
root and, under deepest-owner semantics, the catch-all bin
([config.rs:150-161](../../../cli/src/config.rs#L150), [ce.toml:9-21](../../../ce.toml#L9)). Rust validates and
sends it as `[dirId, weight]` rows; the core re-checks arity 2, non-negativity, `dirId < |nodes|`,
`weight >= 1`, and strict ascent by `dirId`
([Structure.hs:122](../../../core/app/CE/Structure.hs#L122),
[Structure.hs:158-161](../../../core/app/CE/Structure.hs#L158),
[Structure.hs:194-207](../../../core/app/CE/Structure.hs#L194)).

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
([StructureProps.hs:131-161](../../../core/test/StructureProps.hs#L131)): declared `[(0,1),(1,2),(2,3)]`,
owned file counts `(3, 9, 6)` of 18 → `p = (1/6, 1/2, 1/3)`, `q = (1/6, 1/3, 1/2)`,
`χ² = (1/6)²/(1/3) + (1/6)²/(1/2) = 5/36` → `138‰`. Weights `(1,3,2)` match the tree exactly →
`0`. Declaring dir 1 alone leaves root and dir 2 unowned → `divergence = []`, deviations
`[[0,0],[2,0]]`.

### 4. The seven axes

Each axis is one named predicate over the validated fact tables, owning its knob(s), so the
perturbation battery has a lever per row ([Axes.hs:1-7](../../../core/app/CE/Structure/Axes.hs#L1)).
S0–S4 are always judged; 5 and 6 are judged **exactly when** their fact table rode the wire —
absent table (`Nothing`) means unjudged, empty table (`Just []`) means judged clean
([Axes.hs:36-42](../../../core/app/CE/Structure/Axes.hs#L36),
[Axes.hs:107-108](../../../core/app/CE/Structure/Axes.hs#L107)).

Every axis penalty is a count of **directories**, axis 3 included — booklet amendment ①
(user decision 2026-08-19, effective v0.5.0). Before it, S3 counted files, and one junk drawer could drive the whole structure score to 0
and drown the other six axes (the illustration's "500 files" is a doc-side figure with no
fixture behind it; under today's density law a charge is bounded by `scale`, so no single
axis can zero the score at all); files remain the *measured* unit, the directory is the *judged* unit
([Axes.hs:112-118](../../../core/app/CE/Structure/Axes.hs#L112),
[structure-axes.md:29-38](../structure-axes.md#L29)). The migration moved this repo's own score 990 → 992 under the THEN-current raw-mass fold;
2.26.0's density fold re-migrated every structure score again, so neither figure describes
the score today ([structure-axes.md:40-42](../structure-axes.md#L40)).

| axis | fact row | predicate | knobs (code, default) |
|---|---|---|---|
| **S0** geometry | `[id,parent,depth,subdirs,files]` | `depth > depthCeil \|\| subdirs + files > fanoutCeil` | 0 `depthCeil=8`, 1 `fanoutCeil=30` |
| **S1** naming | `[dirId,patternCode,count]` | `Σcs >= namingMin && perMille (tsallis2Norm cs) > namingCeil` | 2 `namingMin=5`, 3 `namingCeil=600` |
| **S2** mixing | `[dirId,inside,outside,count]` | `ins + outs >= mixRefFloor && outs > ins` | 4 `mixRefFloor=5` |
| **S3** misplacement | `[dirId,inside,outside,count]` | per file `outside >= misplaceMin && outside > 2*inside`, then dedup to dirs | 5 `misplaceMin=3` |
| **S4** documentation | `[id,…,files]` + `[dirId,bits]` | `(files >= bigDirFloor && even bits) \|\| (id == 0 && bits < 2)` | 6 `bigDirFloor=8` |
| **S5** staleness | wire `[dirId,docTs]` + `[docIdx,targetTs]`, core-derived into `[dirId,stale,total]` | `stale >= staleMin` | 11 `staleMin=1` |
| **S6** redundancy | `[dirId,dupBlocks,deadUnits]` | `dupBlocks >= dupMin \|\| deadUnits >= deadMin` | 9 `dupMin=1`, 10 `deadMin=1` |

Sources: predicates [Axes.hs:134-145](../../../core/app/CE/Structure/Axes.hs#L134),
[Axes.hs:147-152](../../../core/app/CE/Structure/Axes.hs#L147),
[Axes.hs:156-165](../../../core/app/CE/Structure/Axes.hs#L156),
[Axes.hs:171-183](../../../core/app/CE/Structure/Axes.hs#L171),
[Axes.hs:187-199](../../../core/app/CE/Structure/Axes.hs#L187),
[Axes.hs:203-211](../../../core/app/CE/Structure/Axes.hs#L203); constants
[Cost.hs:34](../../../core/app/CE/Structure/Cost.hs#L34), [Cost.hs:39](../../../core/app/CE/Structure/Cost.hs#L39),
[Cost.hs:45](../../../core/app/CE/Structure/Cost.hs#L45), [Cost.hs:51](../../../core/app/CE/Structure/Cost.hs#L51),
[Cost.hs:57](../../../core/app/CE/Structure/Cost.hs#L57), [Cost.hs:64](../../../core/app/CE/Structure/Cost.hs#L64),
[Cost.hs:69](../../../core/app/CE/Structure/Cost.hs#L69), [Cost.hs:107-108](../../../core/app/CE/Structure/Cost.hs#L107),
[Cost.hs:95-96](../../../core/app/CE/Structure/Cost.hs#L95), [Cost.hs:101-102](../../../core/app/CE/Structure/Cost.hs#L101);
knob codes [Knobs.hs:30-51](../../../core/app/CE/Structure/Knobs.hs#L30).

Notes on the non-obvious ones:

- **S1** builds its per-directory count vector by grouping the pattern rows on `dirId`
  ([Axes.hs:164-165](../../../core/app/CE/Structure/Axes.hs#L164)); only pattern *codes* are judged —
  names never enter. Codes are bounded `0..6` by the boundary contract
  ([Structure.hs:145-149](../../../core/app/CE/Structure.hs#L145)). `600‰` "tolerates one odd name in a
  convention-following set and flags a genuine style mix"
  ([Cost.hs:48-51](../../../core/app/CE/Structure/Cost.hs#L48)).
- **S2** uses *one basis on both sides* — per-file touch counts, never edges-vs-touches. Each
  `fileRefs` row contributes `(inside * count, outside * count)`, summed per directory
  ([Axes.hs:179-183](../../../core/app/CE/Structure/Axes.hs#L179)).
- **S3**'s `> 2*inside` ratio is part of the predicate's definition in v1, not a separate knob
  ([Cost.hs:61-64](../../../core/app/CE/Structure/Cost.hs#L61),
  [Axes.hs:191-199](../../../core/app/CE/Structure/Axes.hs#L191)). Directories are deduped via a map's
  key set ([Axes.hs:188](../../../core/app/CE/Structure/Axes.hs#L188)).
- **S4** convention bits are a mask: `1 = README`, `2 = config`; `even bits` means the README bit
  is clear, and the root additionally owes a recognized config
  ([Axes.hs:201-211](../../../core/app/CE/Structure/Axes.hs#L201)). Bits are constrained to `1..3` at the
  boundary ([Structure.hs:150-153](../../../core/app/CE/Structure.hs#L150)).
- **S6** never re-derives duplication or dead code: it convolves the per-file families' verdicts
  to the tree scale ([Axes.hs:138-139](../../../core/app/CE/Structure/Axes.hs#L138)).

**Drill-down.** The same predicate lists feed the sparse `[dirId, axis]` finding rows the GUI
tree colours by — one list per axis, in axis order
([Axes.hs:119-128](../../../core/app/CE/Structure/Axes.hs#L119)).

### 5. The fold: charges to one score

Since contract 2.26.0 (M9 batch 9 P9, user ruling — a VERSIONING entry, not a proto
version; the protocol is far past it) the structure family runs the **same density
law as the verdict family**: each axis pairs its flagged-directory count `v` with the one
opportunity every structure axis shares — the directory total `N` — and maps the odds
through `chargeAt`, imported from [Score.hs:152](../../../core/app/CE/Verdict/Score.hs#L152)
(one law, two families; [Structure.hs:265-276](../../../core/app/CE/Structure.hs#L265)):

```
charge_i = floor(scale * v_i / (v_i + N))
raw      = Σ_axes (charge_i * violCost)
score    = max 0 (scale - raw `div` (structViolCostNeutral * judgedAxisCount))
```

with `violCost = 10` (knob 7, the family's strictness dial) and its structural neutral
`structViolCostNeutral = 10` ([Cost.hs:75-76](../../../core/app/CE/Structure/Cost.hs#L75),
[Cost.hs:84-85](../../../core/app/CE/Structure/Cost.hs#L84)): at the
neutral default the fold is the plain mean of the bounded charges and cannot saturate.
`scale = 1000` (knob 8). All arithmetic is exact `Rational` then integer `div` — no float
anywhere on the path. `judgedAxisCount` is 5, 6, or 7 depending on which optional tables
rode. The axis rows the reply carries are the **charges** (‰ of scale), the verdict-family
grammar; the flagged directories themselves remain in `findings`, unchanged.

The raw-mass fold this replaced (`scale - Σ(count*violCost) div axes`) was the exact shape
the M9 batch-6 field test retired in verdict/1: a mean flagged count of 100 directories
pinned the score at 0, and the same violation *rate* on a 10× repo cost 10× the penalty.
Density is scale-free: same rate, same score.

Worked end-to-end on the battery fixture — root (2 subdirs, 3 files, README+config), dir 1
(9 files, 5 snake + 4 pascal, two files with 4 outside refs each, no README), dir 2 (6 files,
uniform names), dir 3 (empty, depth 2), so `N = 4`
([StructureProps.hs:30-40](../../../core/test/StructureProps.hs#L30)):

```
axes    = [[0,0],[1,200],[2,200],[3,200],[4,200]]  -- v=1 at N=4: floor(1000/5)=200
raw     = 800 * 10
score   = 1000 - 8000 `div` (10*5) = 840
entropy = [[0,782],[1,916]]                        -- patterns [11,4]; dir files [3,9,6]
findings= [[1,1],[1,2],[1,3],[1,4]]
```

asserted against the real `respond` at
[StructureProps.hs:67-75](../../../core/test/StructureProps.hs#L67). Adding an empty `redundancy` table
makes it six judged axes: `1000 - 8000 div 60 = 867`; both optional tables plus flagged rows
give seven axes, charges `800+200+333`, `1000 - 13330 div 70 = 810`
([StructureProps.hs:176-213](../../../core/test/StructureProps.hs#L176)).

**Knob echo.** All 19 knobs (codes `0..18`) live in one authority table of
`(code, getter, setter)` triples, so the effective fold and the reply's echo read the same rows
and a knob cannot exist in one direction only
([Knobs.hs:28-51](../../../core/app/CE/Structure/Knobs.hs#L28)); rows outside `0..18` or with value `< 1`
are refused by name ([Knobs.hs:19-24](../../../core/app/CE/Structure/Knobs.hs#L19)). `ce.toml` is the
source, `Cost.hs` the defaults ([Cost.hs:1-6](../../../core/app/CE/Structure/Cost.hs#L1)), and the reply
echoes the full effective set ([Structure.hs:237](../../../core/app/CE/Structure.hs#L237)). Codes `12..18`
(`seamSoft=300`, `seamHard=750`, `seamPMax=10`, `roiRefMilli=250`, `roiPhiMilli=500`,
`roiCloneMilli=500`, `roiChurnMilli=150` —
[Cost.hs:110-147](../../../core/app/CE/Structure/Cost.hs#L110)) belong to the split-ROI advisory, not to
any axis, and never enter this fold.

### 6. Preconditions the fold assumes

The score above is only meaningful because the boundary contract runs first, in request order,
and returns the *first* offender by name ([Structure.hs:100-115](../../../core/app/CE/Structure.hs#L100)):

- node rows are dense and tree-shaped: `id == index`, no negative fields, root self-loops at
  depth 0, `parent < id` for every non-root row
  ([Structure.hs:163-176](../../../core/app/CE/Structure.hs#L163));
- `depth == parent.depth + 1` is *checked*, not assumed. It was previously only claimed in a
  docstring, and a forged row `[1,0,999,0,1]` rode straight into the geometry axis and moved the
  score (review 2026-08-20 #6) ([Structure.hs:178-192](../../../core/app/CE/Structure.hs#L178),
  probe at [StructureProps.hs:108-112](../../../core/test/StructureProps.hs#L108));
- the five `dirTables` share one checker — arity, non-negativity, `dirId < |nodes|`, a
  per-table extra rule, and strict ascent
  ([Structure.hs:118-125](../../../core/app/CE/Structure.hs#L118),
  [Structure.hs:194-207](../../../core/app/CE/Structure.hs#L194)); `staleDocRows` shares the
  ROW checker but orders **non-descending**, because one directory holds many docs
  ([Stale.hs:26-29](../../../core/app/CE/Structure/Stale.hs#L26)). Extra rules: pattern code `<= 6` and
  count `>= 1`; convention bits in `1..3`; `fileRefs` count `>= 1`; declared weight `>= 1`
  (the pre-judged staleDocs rules retired with their table at 2.29.0 — the raw
  `staleDocRows`/`staleEdgeRows` validators own staleness now)
  ([Structure.hs:137-161](../../../core/app/CE/Structure.hs#L137)).

**Over-cap.** `structNodeCap = 524288` ([Cost.hs:153-156](../../../core/app/CE/Structure/Cost.hs#L153)).
Node rows *and* the seam tables count against the same cap — a declared cap that misses a
request dimension walks that dimension uncapped
([Structure.hs:102-107](../../../core/app/CE/Structure.hs#L102)). Over-cap answers a **complete degraded
reply that fails**: facts are emptied, the A-layer and split keys drop, `fail` and `degraded`
are both true and `reason` is `structure_too_large`
([Structure.hs:218-238](../../../core/app/CE/Structure.hs#L218),
[StructureProps.hs:231-240](../../../core/test/StructureProps.hs#L231)). Note the consequence of the
empty-facts path: five axes at penalty 0, hence `score = 1000` with `fail = true` — the score is
not evidence of health in a degraded reply. No `ce structure` user ever sees that 1000: the CLI
turns a degraded reply into an error before rendering ([wire.rs:148](../../../cli/src/structure/wire.rs#L148)),
so the number matters to a second client of the protocol, not to this one's console. In the non-degraded case `fail` equals `degraded`,
i.e. always false: S2 is report-only, and the CLI gates nothing on this score
([Structure.hs:216-218](../../../core/app/CE/Structure.hs#L216)).
