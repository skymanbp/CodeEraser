# Split-ROI seam pricing (four legs)

[index](../methodology.md) · [← 07 The three-signal join](07-the-three-signal-join.md) · [→ 09 Edit four-classification (update supervision)](09-edit-four-classification-update-supervision.md)

The split-ROI advisory answers one question per oversized file with a number instead of a
slogan: *is this file worth splitting, and where?* It is an **advisory**, not a gate — it
never contributes to the structure score or the fail bit. Measurement is Rust
(`cli/src/structure/seams.rs`), pricing and judgment are Haskell
(`core/app/CE/Structure/Split.hs`); the module header states the split explicitly —
"NOTHING in this module judges" [seams.rs:1-7](../../../cli/src/structure/seams.rs#L1).

### Scope: which files get priced

A file enters the seam tables only if it is in the **judged** language set and strictly
exceeds the committed soft line: `Lang::judged_path(...)` must resolve, and files with
`total_lines <=` its opening line are skipped [seams.rs:76-81](../../../cli/src/structure/seams.rs#L76).
The opening line is the *committed* soft line — `softLine` from `ce-baseline.json`, falling back
to the *global* `thresholds.file_lines_warn`, falling back to `300`
[judge.rs:82-93](../../../cli/src/structure/judge.rs#L82) — or, for a file whose
`[[rules.class]]` declares `file_lines_warn`, that class's warn line (proto 9.0.0,
[seams.rs:40-62](../../../cli/src/structure/seams.rs#L40)); the class rides as `seamFiles`' third
column, so the advisory opens a classed file at the same class line the hook reads, and wherever a
baseline is committed every other file at exactly the line the hook uses.

The reply keys `splitCandidates` / `sizeExempt` exist **iff** `seamFiles` rode the wire, and
a degraded reply drops them with the rest of the facts
[Structure.hs:220-227](../../../core/app/CE/Structure.hs#L220).

### Seam enumeration and best-seam selection

Units on the wire are the **top-level** spans only: outermost, non-overlapping,
start-ordered, so a nested helper always lands on its holder's side of every seam
[seams.rs:212-226](../../../cli/src/structure/seams.rs#L212). Each unit's `end` is clamped to the
file total [seams.rs:231](../../../cli/src/structure/seams.rs#L231).

A seam is the gap *after* a unit that has a successor — the enumeration zips the file's unit
list against its own tail, so a file with `n` top-level units yields `n − 1` seams and a
single-unit file yields none [Split.hs:208-222](../../../core/app/CE/Structure/Split.hs#L208). Seam
`u` cuts the file at line `end_u` into a prefix of `end_u` lines and a suffix of
`total − end_u` lines.

Each seam is priced to a triple `(u, benefitMilli, costMilli)`. Selection is the exact
rational argmax over ROI, compared by cross-multiplied `b % c` rather than division —
`maximumBy (comparing (\(_, b', c') -> b' % c'))`
[Split.hs:190](../../../core/app/CE/Structure/Split.hs#L190). Equal ROIs go to the **last** seam in
file order: `maximumBy` keeps the later of equals (the reference battery `StructureEquivProps`
pins it). Cost is never zero because φ ≥ 1 by the
knob rule, so the ratio is always defined
[Split.hs:168-170](../../../core/app/CE/Structure/Split.hs#L168).

### Benefit: soft-zone penalty recovered

Benefit is the graded-zone penalty the split gives back, computed on the **same curve the
verdict family judges with** — `CE.Verdict.Soft.zonePenalty`, reached through the verdict's one
class-line resolver `Soft.classedZone` rather than re-derived
[Split.hs:2-5,25,186](../../../core/app/CE/Structure/Split.hs#L2), [Soft.hs:134-137](../../../core/app/CE/Verdict/Soft.hs#L134):

```
p(x) = 0                              if x <= S     -- S read as min S H (proto 9.0.0)
p(x) = P_max · ((x − S)/(H − S))²      if S < x <= H
p(x) = P_max · (1 + 2(x − H)/(H − S))  if x > H     -- C¹ linear arm (proto 2.17.0)
```

[Soft.hs:119-126](../../../core/app/CE/Verdict/Soft.hs#L119). The curve is convex on the zone, exact
`Rational`, and keeps charging past `H` — since proto 2.17.0 linearly, at exactly the slope the
quadratic reached at the wall (monotone, no kink; the deny at `H` is the guard's job, and a
score that stopped charging past the wall would reward growth — but the quadratic never leaves
its contracted `(S,H]` domain: the M9 batch-6 saturation lesson)
[Soft.hs:97-111](../../../core/app/CE/Verdict/Soft.hs#L97). For seam pricing this means the size
benefit of splitting a past-`H` giant is linear in its overhang, not quadratic. A degenerate
`H <= S` (only `H == S` since the curve opens at `min S H`) falls back to the pre-v0.6 binary `p = P_max` flat
[Soft.hs:121](../../../core/app/CE/Verdict/Soft.hs#L121).

```
benefitMilli(u) = max 0 (floor (1000 · (p(total) − p(end_u) − p(total − end_u))))
```

[Split.hs:214](../../../core/app/CE/Structure/Split.hs#L214). The `1000·` converts penalty units
to milli — milli is the one published scale
[Split.hs:8-9](../../../core/app/CE/Structure/Split.hs#L8). Because `p` is convex with `p(0) = 0`, it
is superadditive, so the bracket is non-negative for any well-formed zone triple; the
`max 0` clamp is what keeps the degenerate `H <= S` fallback (where `p(total) − 2·P_max < 0`)
from producing a negative benefit.

The zone triple is the advisory's own copy of `S/H/P_max`, so the structure family can price
a seam without a `verdict/1` request in flight; Rust sends the same numbers it sends
`verdict/1`: knobs 12, 13 and 14 ride once, tree-wide, off the committed soft line, the global
`file_lines_fail` (left out when it is `0` = no hard line, so the core's `750` judges, as the
score's does) and — since 6.1.0 — `score.size_penalty_max` when one is declared. Since proto
9.0.0 the request also carries the verdict's own `classKnobs` rows and each seam file's class,
and a classed file is priced on its class's opening edge and hard line through the one resolver
the score's size axis reads (`Soft.classedZone`); until then the advisory was class-blind, while
`verdict/1` (since 3.1.0) measured a classed row against its `[[rules.class]]` lines. Both gaps
had the same shape — before 6.1.0 only 12 and 13 rode, so a repo declaring `size_penalty_max`
got the declared curve in its score and the core's built-in `P_max = 10` in its advisory — with
both halves internally consistent and nothing anywhere disagreeing out loud
([judge.rs:189-198](../../../cli/src/structure/judge.rs#L189),
[Cost.hs:133-142](../../../core/app/CE/Structure/Cost.hs#L133),
counterfactual at [structure_knobs.rs:77-88](../../../cli/tests/it/structure_knobs.rs#L77)):

| knob | code | default | source |
|---|---|---|---|
| `seamSoft` (S) | 12 | `300` | [Cost.hs:147-148](../../../core/app/CE/Structure/Cost.hs#L147) |
| `seamHard` (H) | 13 | `750` | [Cost.hs:150-151](../../../core/app/CE/Structure/Cost.hs#L150) |
| `seamPMax` (P_max) | 14 | `10` | [Cost.hs:153-154](../../../core/app/CE/Structure/Cost.hs#L153) |

### Cost: the four priced legs

```
costMilli(u) = crossRefs(u)  · roiRefMilli
             + cutClones(end_u) · roiCloneMilli
             + crossChurn(u) · roiChurnMilli
             + roiPhiMilli
```

[Split.hs:218-222](../../../core/app/CE/Structure/Split.hs#L218). Three counting legs plus one flat
leg. Both crossing legs charge through one helper — `crossings` folds every edge `(a,b)` into a
difference map `[(min a b, +1), (max a b, -1)]` and running-sums it, so a seam `u` is charged
iff `min <= u < max`, exactly "one endpoint at or before `u`"
([Split.hs:229-232](../../../core/app/CE/Structure/Split.hs#L229), read per seam by `charge`'s
`lookupLE` at [Split.hs:251-252](../../../core/app/CE/Structure/Split.hs#L251)). The clone leg uses a line-level
straddle instead: block `[s,e)` is cut iff `s <= line && line < e`, mapped onto the seam-line
index by the `lookupGE s` / `lookupLT e` pair
([Split.hs:242-244](../../../core/app/CE/Structure/Split.hs#L242)).

| leg | knob | code | default (milli) | constant | measurement |
|---|---|---|---|---|---|
| severed reference | `roiRefMilli` | 15 | `250` | [Cost.hs:162-163](../../../core/app/CE/Structure/Cost.hs#L162) | word-bounded mention edges [seams.rs:248-269](../../../cli/src/structure/seams.rs#L248) |
| cut clone block | `roiCloneMilli` | 17 | `500` | [Cost.hs:174-175](../../../core/app/CE/Structure/Cost.hs#L174) | T1/T2 dedup block spans [seams.rs:105-133](../../../cli/src/structure/seams.rs#L105) |
| crossing co-change pair | `roiChurnMilli` | 18 | `150` | [Cost.hs:177-178](../../../core/app/CE/Structure/Cost.hs#L177) | 14-day commit ledger [seams.rs:141-170](../../../cli/src/structure/seams.rs#L141) |
| per-new-file overhead φ | `roiPhiMilli` | 16 | `500` | [Cost.hs:165-166](../../../core/app/CE/Structure/Cost.hs#L165) | flat, no measurement |

All seven knobs (zone triple + four prices) ride the `Knobs` record
[Axes.hs:71-80](../../../core/app/CE/Structure/Axes.hs#L71) and are bound to the `Cost.hs` defaults
[Axes.hs:100-106](../../../core/app/CE/Structure/Axes.hs#L100). The wire carries them as knob codes
12–18; the golden fixture pins `[12,300],[13,750],[14,10],[15,250],[16,500],[17,500],[18,150]`
[golden.ndjson:22](../../../contracts/fixtures/structure/golden.ndjson#L22).

**Leg 1 — severed references.** The honest v1 proxy: true intra-file symbol co-reference
exists in no cache, so an edge `(i → j)` is recorded when unit `j`'s bare name appears
word-bounded inside unit `i`'s body
[seams.rs:241-269](../../../cli/src/structure/seams.rs#L241),
[size-advisory.md:96-99](../size-advisory.md#L96). The body is searched with every literal and
comment blanked to spaces first — the exact tree-sitter spans, through the one masker the
tombstone's literal blanking shares ([opaque.rs:1-14](../../../cli/src/scan/opaque.rs#L1),
[opaque.rs:27-38](../../../cli/src/scan/opaque.rs#L27)) — so a name inside a string or a comment is no
reference (plan v2.30 step 5b item 30; it counted one until then). Word-boundedness is
identifier-char adjacency on both sides
[seams.rs:271-289](../../../cli/src/structure/seams.rs#L271). Names shorter than `NAME_FLOOR = 3` are
dropped as noise — `new`, `run`, `id` would edge every unit to every other
[seams.rs:31-34](../../../cli/src/structure/seams.rs#L31),
[seams.rs:261](../../../cli/src/structure/seams.rs#L261). What the proxy measures is therefore a
word-bounded spelling of the unit's name in the neighbour's code — a same-named local or field
counts as one, since no symbol table stands behind it; the advisory face is non-binding
[size-advisory.md:100-105](../size-advisory.md#L100).

**Leg 2 — cut clone blocks.** Spans come off the *same* index the dedup family judges from
(`crate::dedup::snapshot`, the one command-boundary measurement, which itself calls `dedup::analyze`), both sides of each block, clamped to `[1, total]` and deduplicated
through a `BTreeSet` [seams.rs:115-131](../../../cli/src/structure/seams.rs#L115). A seam through a
block splits one coherent duplicate span across two files — priced dearer than one severed
reference [Cost.hs:168-173](../../../core/app/CE/Structure/Cost.hs#L168).

**Leg 3 — crossing co-change pairs.** Pairs of top-level units that the churn window edits in
the same commit. Commits are narrowed at git (`--since {14} days ago --first-parent
--no-merges`, path-limited to the seam files)
[seams.rs:177-191](../../../cli/src/structure/seams.rs#L177), then each commit's ledger rows are
joined onto the *current* snapshot's units at key level; renamed units drop out honestly, and
a tree without git history prices the leg at zero rather than failing the advisory
[seams.rs:135-170](../../../cli/src/structure/seams.rs#L135). The window constant
`CHURN_WINDOW_DAYS` reads the churn family's own default `DEFAULT_DAYS = 14` — the §4.1 two-week
anchor, and deliberately a **measurement constant, not a wire knob** (the prices are the knobs)
[seams.rs:36-38](../../../cli/src/structure/seams.rs#L36), [churn/mod.rs:51-52](../../../cli/src/churn/mod.rs#L51),
[size-advisory.md:129](../size-advisory.md#L129).

**Leg 4 — φ.** The flat per-new-file cost: S0 fanout plus the mental-load overhead the design
booklet names φ [Cost.hs:156-166](../../../core/app/CE/Structure/Cost.hs#L156). Defaults were sized so
a mid-zone file with a clean seam clears ROI 1 and one with 10+ crossing references does not
[Cost.hs:159-161](../../../core/app/CE/Structure/Cost.hs#L159).

### Price calibration (as-built)

The v1.1 legs were calibrated by recompiling the `Cost.hs` price points and replaying
`--split-candidates` over corpora, comparing candidate/exemption flips and best-seam movement
[size-advisory.md:117-120](../size-advisory.md#L117).

- `roiCloneMilli = 500` (= 2 × ref): swept `{250, 500, 1000}` over four external corpora / 86
  candidates. Zero candidate flips across the fourfold price range — the price is a
  *seam-steering* term, not a candidate killer — with exactly one best-seam move per price
  point, each in the correct direction (cobra's `command_test` moves its seam off the clone
  block at the 1000 price point)
  [size-advisory.md:121-124](../size-advisory.md#L121).
- `roiChurnMilli = 150` (= 0.6 × ref, reflecting that historical correlation is weaker
  evidence than in-situ code coupling): swept `{150, 300, 600}` on the self repo only —
  external corpora tips are frozen outside the 14-day window, so their churn leg is honestly
  zero and only a live window can be calibrated. Zero label flips over the fourfold range,
  with cost scaling verified linear (`graph_ladder` 800 → 1100 → 1700)
  [size-advisory.md:125-128](../size-advisory.md#L125).

Self-repo first run at the `S = 294` calibration: `graph_ladder.rs` got a real seam after line
318 (2120‰ benefit against 500‰ cost, ROI 4.2); `DEVELOPMENT_PLAN.md` and `main_cmds.rs` got
machine-written cohesion exemptions (11‰ vs 500‰ and 0‰ vs 2000‰); `cli.md` crossed the line
by one notch (507‰ vs 500‰) and flipped to a candidate
[size-advisory.md:109-113](../size-advisory.md#L109).

### The ROI auto-exemption

Viability is `ROI >= 1`, evaluated without division as `b >= c`
[Split.hs:191](../../../core/app/CE/Structure/Split.hs#L191),
[Split.hs:7](../../../core/app/CE/Structure/Split.hs#L7). The fold produces exactly one row per file,
into one of three shapes [Split.hs:186-194](../../../core/app/CE/Structure/Split.hs#L186):

| condition | row | table |
|---|---|---|
| best seam has `b >= c` | `[fid, u, b, c]` | `splitCandidates` |
| best seam has `b < c` | `[fid, b, c]` | `sizeExempt` |
| no seam at all (< 2 top-level units) | `[fid, 0, 0]` | `sizeExempt` |

The exemption row carries the **best seam's** numbers (best by ROI, not by benefit), which is
what lets the Rust side write the machine-generated *why*
[Split.hs:6-8,166-170](../../../core/app/CE/Structure/Split.hs#L6). This is the advisory's answer to
"big projects have naturally long files": long-and-cohesive is an exemption **with numbers
attached**, long-and-splittable gets a cut line
[size-advisory.md:62-64](../size-advisory.md#L62). Since plan v2.32 step 4 the core lays the
structure document out (`CE.Structure.Document`): its statement names the universe every
column points into, so each dense id is range-checked by the document contract before
anything reads it [Document.hs:38-47](../../../core/app/CE/Structure/Document.hs#L38), and the
names are spelled in the core, which resolves the references the document carries against the
names Rust sends beside the request [Spell.hs:25-38](../../../core/app/CE/Document/Spell.hs#L25); candidates surface as
`(path, afterLine, unitName, benefitMilli, costMilli, roi)` (`roi` the one-decimal ratio the
console prints, null without a priced seam; an exemption row carries it too), where `afterLine` is the chosen unit's
end line, which Rust sends beside the reply's row and refuses for a unit the file does not hold
[document.rs:84-96](../../../cli/src/structure/document.rs#L84).

### Input validation

The five seam tables are boundary-checked before any pricing, in request order
[Split.hs:50-59](../../../core/app/CE/Structure/Split.hs#L50): file ids dense (`id == index`) with
`total >= 1` [Split.hs:66-78](../../../core/app/CE/Structure/Split.hs#L66); units dense per file from
0, spans strictly ascending and non-overlapping, checked in one fold carrying
`(file, previousEnd, expectedNextUnit)`
[Split.hs:135-150](../../../core/app/CE/Structure/Split.hs#L135); `seamUnits` and `seamClones` share
**one** span checker so the two tables cannot drift on what a span is
[Split.hs:86-92](../../../core/app/CE/Structure/Split.hs#L86); ref edges refuse self-edges
[Split.hs:152-154](../../../core/app/CE/Structure/Split.hs#L152) and churn pairs must ascend, so an
unordered pair has exactly one spelling
[Split.hs:128-130](../../../core/app/CE/Structure/Split.hs#L128); all three edge tables must arrive in
ascending canonical order [Split.hs:56-58](../../../core/app/CE/Structure/Split.hs#L56).

Every quantity above is exact integer or `Rational` arithmetic — no floating point enters the
computation at any stage [Split.hs:8-9](../../../core/app/CE/Structure/Split.hs#L8).

### Not found in source

The design contract §C once listed two benefit terms and one cost term beyond the four legs —
benefit "dedup budget effect" and "hot/cold unit isolation", cost "baseline re-key noise" — and
plan v2.30 step 5b retired them from the model with reasons, in place of a standing "not
implemented" ([size-advisory.md:53-54](../size-advisory.md#L53), [size-advisory.md:57](../size-advisory.md#L57)):
a clone block is judged on content fingerprints, and a seam either cuts it (priced by
`roiCloneMilli`) or leaves it whole on one side, so no split lowers the dedup ledger by a line;
a seam between hot and cold units is a seam severing no co-change pair, which the
`roiChurnMilli` leg already prices at zero, so a benefit term would count the same churn fact
twice (the model stays on the conservative side); and re-keying the moved members' baseline
identity is paid once per new file, which is what φ is. The shipped benefit is the soft-zone
recovery term alone [Split.hs:214](../../../core/app/CE/Structure/Split.hs#L214) and the
shipped cost is exactly the four legs above
[Split.hs:218-222](../../../core/app/CE/Structure/Split.hs#L218). No constant for a retired
term exists in `Cost.hs`, and none is owed.
