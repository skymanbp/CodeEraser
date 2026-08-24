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
`total_lines <= soft` are skipped [seams.rs:51-56](../../../cli/src/structure/seams.rs#L51). The
`soft` passed in is the *committed* line — `softLine` from `ce-baseline.json`, falling back
to `thresholds.file_lines_warn`, falling back to `300`
[judge.rs:145-156](../../../cli/src/structure/judge.rs#L145) — so the advisory opens the zone at
exactly the line the hook uses.

The reply keys `splitCandidates` / `sizeExempt` exist **iff** `seamFiles` rode the wire, and
a degraded reply drops them with the rest of the facts
[Structure.hs:246-253](../../../core/app/CE/Structure.hs#L238).

### Seam enumeration and best-seam selection

Units on the wire are the **top-level** spans only: outermost, non-overlapping,
start-ordered, so a nested helper always lands on its holder's side of every seam
[seams.rs:183-197](../../../cli/src/structure/seams.rs#L183). Each unit's `end` is clamped to the
file total [seams.rs:202-205](../../../cli/src/structure/seams.rs#L202).

A seam is the gap *after* a unit that has a successor — the enumeration zips the file's unit
list against its own tail, so a file with `n` top-level units yields `n − 1` seams and a
single-unit file yields none [Split.hs:196-211](../../../core/app/CE/Structure/Split.hs#L196). Seam
`u` cuts the file at line `end_u` into a prefix of `end_u` lines and a suffix of
`total − end_u` lines.

Each seam is priced to a triple `(u, benefitMilli, costMilli)`. Selection is the exact
rational argmax over ROI, compared by cross-multiplied `b % c` rather than division —
`maximumBy (comparing (\(_, b', c') -> b' % c'))`
[Split.hs:180](../../../core/app/CE/Structure/Split.hs#L180). Cost is never zero because φ ≥ 1 by the
knob rule, so the ratio is always defined
[Split.hs:161-163](../../../core/app/CE/Structure/Split.hs#L161).

### Benefit: soft-zone penalty recovered

Benefit is the graded-zone penalty the split gives back, computed on the **same curve the
verdict family judges with** — `CE.Verdict.Soft.zonePenalty`, imported directly rather than
re-derived [Split.hs:2-5,24,202](../../../core/app/CE/Structure/Split.hs#L2):

```
p(x) = 0                              if x <= S
p(x) = P_max · ((x − S)/(H − S))²      if S < x <= H
p(x) = P_max · (1 + 2(x − H)/(H − S))  if x > H     -- C¹ linear arm (proto 2.17.0)
```

[Soft.hs:59-66](../../../core/app/CE/Verdict/Soft.hs#L59). The curve is convex on the zone, exact
`Rational`, and keeps charging past `H` — since proto 2.17.0 linearly, at exactly the slope the
quadratic reached at the wall (monotone, no kink; the deny at `H` is the guard's job, and a
score that stopped charging past the wall would reward growth — but the quadratic never leaves
its contracted `(S,H]` domain: the M9 batch-6 saturation lesson)
[Soft.hs:44-58](../../../core/app/CE/Verdict/Soft.hs#L44). For seam pricing this means the size
benefit of splitting a past-`H` giant is linear in its overhang, not quadratic. A degenerate
`H <= S` falls back to the pre-v0.6 binary `p = P_max` flat
[Soft.hs:61](../../../core/app/CE/Verdict/Soft.hs#L61).

```
benefitMilli(u) = max 0 (floor (1000 · (p(total) − p(end_u) − p(total − end_u))))
```

[Split.hs:202-203](../../../core/app/CE/Structure/Split.hs#L202). The `1000·` converts penalty units
to milli — milli is the one published scale
[Split.hs:8-9](../../../core/app/CE/Structure/Split.hs#L8). Because `p` is convex with `p(0) = 0`, it
is superadditive, so the bracket is non-negative for any well-formed zone triple; the
`max 0` clamp is what keeps the degenerate `H <= S` fallback (where `p(total) − 2·P_max < 0`)
from producing a negative benefit.

The zone triple is the advisory's own copy of `S/H/P_max`, so the structure family can price
a seam without a `verdict/1` request in flight; Rust sends the same numbers it sends
`verdict/1` [Cost.hs:99-104](../../../core/app/CE/Structure/Cost.hs#L99):

| knob | code | default | source |
|---|---|---|---|
| `seamSoft` (S) | 12 | `300` | [Cost.hs:105-106](../../../core/app/CE/Structure/Cost.hs#L105) |
| `seamHard` (H) | 13 | `750` | [Cost.hs:108-109](../../../core/app/CE/Structure/Cost.hs#L108) |
| `seamPMax` (P_max) | 14 | `10` | [Cost.hs:111-112](../../../core/app/CE/Structure/Cost.hs#L111) |

### Cost: the four priced legs

```
costMilli(u) = crossRefs(u)  · roiRefMilli
             + cutClones(end_u) · roiCloneMilli
             + crossChurn(u) · roiChurnMilli
             + roiPhiMilli
```

[Split.hs:207-211](../../../core/app/CE/Structure/Split.hs#L207). Three counting legs plus one flat
leg. Both crossing legs charge through one helper — `crossings` folds every edge `(a,b)` into a
difference map `[(min a b, +1), (max a b, -1)]` and running-sums it, so a seam `u` is charged
iff `min <= u < max`, exactly "one endpoint at or before `u`"
([Split.hs:218-221](../../../core/app/CE/Structure/Split.hs#L218), read per seam by `charge`'s
`lookupLE` at [Split.hs:240-241](../../../core/app/CE/Structure/Split.hs#L240)). The clone leg uses a line-level
straddle instead: block `[s,e)` is cut iff `s <= line && line < e`, mapped onto the seam-line
index by the `lookupGE s` / `lookupLT e` pair
([Split.hs:227-233](../../../core/app/CE/Structure/Split.hs#L227)).

| leg | knob | code | default (milli) | constant | measurement |
|---|---|---|---|---|---|
| severed reference | `roiRefMilli` | 15 | `250` | [Cost.hs:120-121](../../../core/app/CE/Structure/Cost.hs#L120) | word-bounded mention edges [seams.rs:215-234](../../../cli/src/structure/seams.rs#L215) |
| cut clone block | `roiCloneMilli` | 17 | `500` | [Cost.hs:132-133](../../../core/app/CE/Structure/Cost.hs#L132) | T1/T2 dedup block spans [seams.rs:79-107](../../../cli/src/structure/seams.rs#L79) |
| crossing co-change pair | `roiChurnMilli` | 18 | `150` | [Cost.hs:135-136](../../../core/app/CE/Structure/Cost.hs#L135) | 14-day commit ledger [seams.rs:115-144](../../../cli/src/structure/seams.rs#L115) |
| per-new-file overhead φ | `roiPhiMilli` | 16 | `500` | [Cost.hs:123-124](../../../core/app/CE/Structure/Cost.hs#L123) | flat, no measurement |

All seven knobs (zone triple + four prices) ride the `Knobs` record
[Axes.hs:58-67](../../../core/app/CE/Structure/Axes.hs#L58) and are bound to the `Cost.hs` defaults
[Axes.hs:85-91](../../../core/app/CE/Structure/Axes.hs#L85). The wire carries them as knob codes
12–18; the golden fixture pins `[12,300],[13,750],[14,10],[15,250],[16,500],[17,500],[18,150]`
[golden.ndjson:22](../../../contracts/fixtures/structure/golden.ndjson#L22).

**Leg 1 — severed references.** The honest v1 proxy: true intra-file symbol co-reference
exists in no cache, so an edge `(i → j)` is recorded when unit `j`'s bare name appears
word-bounded inside unit `i`'s body
[seams.rs:212-234](../../../cli/src/structure/seams.rs#L212),
[size-advisory.md:92-95](../size-advisory.md#L93). Word-boundedness is
identifier-char adjacency on both sides
[seams.rs:236-254](../../../cli/src/structure/seams.rs#L236). Names shorter than `NAME_FLOOR = 3` are
dropped as noise — `new`, `run`, `id` would edge every unit to every other
[seams.rs:31-34](../../../cli/src/structure/seams.rs#L31),
[seams.rs:226](../../../cli/src/structure/seams.rs#L226). Documented limitation: short names and
in-string mentions will count an edge; the advisory face is non-binding, so this is tolerated
[size-advisory.md:93-95](../size-advisory.md#L94).

**Leg 2 — cut clone blocks.** Spans come off the *same* index the dedup family judges from
(`crate::dedup::analyze`), both sides of each block, clamped to `[1, total]` and deduplicated
through a `BTreeSet` [seams.rs:89-105](../../../cli/src/structure/seams.rs#L89). A seam through a
block splits one coherent duplicate span across two files — priced dearer than one severed
reference [Cost.hs:126-131](../../../core/app/CE/Structure/Cost.hs#L126).

**Leg 3 — crossing co-change pairs.** Pairs of top-level units that the churn window edits in
the same commit. Commits are narrowed at git (`--since {14} days ago --first-parent
--no-merges`, path-limited to the seam files)
[seams.rs:148-162](../../../cli/src/structure/seams.rs#L148), then each commit's ledger rows are
joined onto the *current* snapshot's units at key level; renamed units drop out honestly, and
a tree without git history prices the leg at zero rather than failing the advisory
[seams.rs:109-144](../../../cli/src/structure/seams.rs#L109). The window constant is
`CHURN_WINDOW_DAYS = 14` — the §4.1 two-week anchor, and deliberately a **measurement
constant, not a wire knob** (the prices are the knobs)
[seams.rs:36-39](../../../cli/src/structure/seams.rs#L36),
[size-advisory.md:119](../size-advisory.md#L119).

**Leg 4 — φ.** The flat per-new-file cost: S0 fanout plus the mental-load overhead the design
booklet names φ [Cost.hs:114-119](../../../core/app/CE/Structure/Cost.hs#L114). Defaults were sized so
a mid-zone file with a clean seam clears ROI 1 and one with 10+ crossing references does not
[Cost.hs:117-119](../../../core/app/CE/Structure/Cost.hs#L117).

### Price calibration (as-built)

The v1.1 legs were calibrated by recompiling the `Cost.hs` price points and replaying
`--split-candidates` over corpora, comparing candidate/exemption flips and best-seam movement
[size-advisory.md:107-110](../size-advisory.md#L107).

- `roiCloneMilli = 500` (= 2 × ref): swept `{250, 500, 1000}` over four external corpora / 86
  candidates. Zero candidate flips across the fourfold price range — the price is a
  *seam-steering* term, not a candidate killer — with exactly one best-seam move per price
  point, each in the correct direction (cobra's `command_test` moves its seam off the clone
  block at the 1000 price point)
  [size-advisory.md:111-114](../size-advisory.md#L111).
- `roiChurnMilli = 150` (= 0.6 × ref, reflecting that historical correlation is weaker
  evidence than in-situ code coupling): swept `{150, 300, 600}` on the self repo only —
  external corpora tips are frozen outside the 14-day window, so their churn leg is honestly
  zero and only a live window can be calibrated. Zero label flips over the fourfold range,
  with cost scaling verified linear (`graph_ladder` 800 → 1100 → 1700)
  [size-advisory.md:115-118](../size-advisory.md#L115).

Self-repo first run at the `S = 294` calibration: `graph_ladder.rs` got a real seam after line
318 (2120‰ benefit against 500‰ cost, ROI 4.2); `DEVELOPMENT_PLAN.md` and `main_cmds.rs` got
machine-written cohesion exemptions (11‰ vs 500‰ and 0‰ vs 2000‰); `cli.md` crossed the line
by one notch (507‰ vs 500‰) and flipped to a candidate
[size-advisory.md:99-103](../size-advisory.md#L99).

### The ROI auto-exemption

Viability is `ROI >= 1`, evaluated without division as `b >= c`
[Split.hs:181](../../../core/app/CE/Structure/Split.hs#L181),
[Split.hs:7](../../../core/app/CE/Structure/Split.hs#L7). The fold produces exactly one row per file,
into one of three shapes [Split.hs:177-184](../../../core/app/CE/Structure/Split.hs#L177):

| condition | row | table |
|---|---|---|
| best seam has `b >= c` | `[fid, u, b, c]` | `splitCandidates` |
| best seam has `b < c` | `[fid, b, c]` | `sizeExempt` |
| no seam at all (< 2 top-level units) | `[fid, 0, 0]` | `sizeExempt` |

The exemption row carries the **best seam's** numbers (best by ROI, not by benefit), which is
what lets the Rust side write the machine-generated *why*
[Split.hs:6-8,180-183](../../../core/app/CE/Structure/Split.hs#L6). This is the advisory's answer to
"big projects have naturally long files": long-and-cohesive is an exemption **with numbers
attached**, long-and-splittable gets a cut line
[size-advisory.md:53-55](../size-advisory.md#L53). Relabelling back to names
happens only in Rust, with every dense id range-checked before it is used as a subscript
[judge.rs:156-181](../../../cli/src/structure/judge.rs#L160); candidates surface as
`(path, afterLine, unitName, benefitMilli, costMilli)` where `afterLine` is the chosen unit's
end line [judge.rs:171-180](../../../cli/src/structure/judge.rs#L175).

### Input validation

The five seam tables are boundary-checked before any pricing, in request order
[Split.hs:47-59](../../../core/app/CE/Structure/Split.hs#L47): file ids dense (`id == index`) with
`total >= 1` [Split.hs:66-71](../../../core/app/CE/Structure/Split.hs#L66); units dense per file from
0, spans strictly ascending and non-overlapping, checked in one fold carrying
`(file, previousEnd, expectedNextUnit)`
[Split.hs:128-143](../../../core/app/CE/Structure/Split.hs#L128); `seamUnits` and `seamClones` share
**one** span checker so the two tables cannot drift on what a span is
[Split.hs:79-85](../../../core/app/CE/Structure/Split.hs#L79); ref edges refuse self-edges
[Split.hs:145-147](../../../core/app/CE/Structure/Split.hs#L145) and churn pairs must ascend, so an
unordered pair has exactly one spelling
[Split.hs:121-123](../../../core/app/CE/Structure/Split.hs#L121); all three edge tables must arrive in
ascending canonical order [Split.hs:54,56,58](../../../core/app/CE/Structure/Split.hs#L54).

Every quantity above is exact integer or `Rational` arithmetic — no floating point enters the
computation at any stage [Split.hs:8-9](../../../core/app/CE/Structure/Split.hs#L8).

### Not found in source

The design contract §C lists two benefit terms and one cost term that the **as-built code does
not implement**: benefit "dedup budget effect" and "hot/cold unit isolation", and cost
"baseline re-key noise" [size-advisory.md:46-48](../size-advisory.md#L46). The
shipped benefit is the soft-zone recovery term alone
[Split.hs:202-203](../../../core/app/CE/Structure/Split.hs#L202) and the shipped cost is exactly the
four legs above [Split.hs:207-211](../../../core/app/CE/Structure/Split.hs#L207). No constant for
either omitted term exists in `Cost.hs`.
