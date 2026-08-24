# Scoring and the ADR-006 ratchet

[index](../methodology.md) · [← 04 Structure judgment — tree-scale entropy, seven axes](04-structure-judgment-tree-scale-entropy-seven.md) · [→ 06 Graph liveness and dead-code verdicts](06-graph-liveness-and-dead-code-verdicts.md)

Everything in this section is computed in `ce-core` (Haskell), from the fact tables that arrive over the `verdict.request` wire. It is pure integer/`Rational` arithmetic — no floating point, no logarithms — so the same request always yields the same verdict.

### The aggregate score

Seven axes, indexed by a fixed code. Since proto 2.17.0 (the density migration, M9 batch 6) each axis pairs a non-negative violation **mass** `v` with its **opportunity** count `n` and charges the bounded density `floor(scale · v/(v+n))` per-mille ([Score.hs:116-145](../../../core/app/CE/Verdict/Score.hs#L116)) — strictly monotone in `v`, never reaching the scale, scale-free across repository sizes, and `0` when `n = 0` (no opportunity table — the honest-absence stance, [Score.hs:141-147](../../../core/app/CE/Verdict/Score.hs#L141)). The migration's cause is recorded in the wire ledger: under the old raw-mass fold two ordinary real repositories both measured 0/1000 ([VERSIONING.md](../../../contracts/VERSIONING.md), 2.17.0 entry).

| code | axis | violation mass `v` | opportunity `n` | knob(s) and value |
|---|---|---|---|---|
| 0 | size | convex soft-zone penalty summed over `metricCode = 0` rows, exact `Rational` ([Score.hs:156-160](../../../core/app/CE/Verdict/Score.hs#L156)) | files (`metricCode = 0` rows) | see soft zone below |
| 1 | complexity | count of `metricCode = 1` rows with `v > cocCeil` ([Score.hs:162-163](../../../core/app/CE/Verdict/Score.hs#L162)) | functions (`metricCode = 1` rows) | `cocCeil = 15` ([Cost.hs:149-150](../../../core/app/CE/Verdict/Cost.hs#L149)) |
| 2 | clone | count of sim rows with `kind <= 1` and `n * cloneDen >= d * cloneNum` ([Score.hs:165-167](../../../core/app/CE/Verdict/Score.hs#L165)) | files | `tsedNum/tsedDen = 85/100` ([Clone/Cost.hs:22](../../../core/app/CE/Clone/Cost.hs#L22), [Clone/Cost.hs:25](../../../core/app/CE/Clone/Cost.hs#L25)) |
| 3 | docdup | count of sim rows with `kind == 2` and `n * dupDen >= d * dupNum` ([Score.hs:192-193](../../../core/app/CE/Verdict/Score.hs#L192)) | files | `jaccardNum/jaccardDen = 80/100` ([Docdup/Cost.hs:27](../../../core/app/CE/Docdup/Cost.hs#L27), [Docdup/Cost.hs:30](../../../core/app/CE/Docdup/Cost.hs#L30)) |
| 4 | deadcode | count of pos rows with `reachIn == 0` and `indeg <= deadIndegCeil` ([Score.hs:179-180](../../../core/app/CE/Verdict/Score.hs#L179)) | graph file nodes (pos rows) | `deadIndegCeil = 0` ([Cost.hs:156-157](../../../core/app/CE/Verdict/Cost.hs#L156)) |
| 5 | churn | count of churn rows with `rw + ap > 0` and `rw * rewriteDen >= (rw + ap) * rewriteNum` ([Score.hs:187-189](../../../core/app/CE/Verdict/Score.hs#L187)) | churned entities (churn rows) | `rewriteNum/rewriteDen = 50/100` ([Cost.hs:88](../../../core/app/CE/Verdict/Cost.hs#L88), [Cost.hs:91](../../../core/app/CE/Verdict/Cost.hs#L91)) |
| 6 | graph_cycle | count of code-file pos rows with `sccSize >= cycleFloor`; `docFiles` indices are excluded ([Score.hs:202-208](../../../core/app/CE/Verdict/Score.hs#L202)) | graph file nodes minus documented files | `sccFloor = 2` ([Graph/Cost.hs:55](../../../core/app/CE/Graph/Cost.hs#L55)) |

All ratio thresholds are cross-multiplied rather than divided, so no rounding enters the predicates ([Score.hs:170](../../../core/app/CE/Verdict/Score.hs#L170), [Score.hs:177](../../../core/app/CE/Verdict/Score.hs#L177), [Score.hs:189](../../../core/app/CE/Verdict/Score.hs#L189)). Note the axis-2 and axis-3 thresholds are *not* owned by the verdict family: they are re-exported from the clone and docdup cost modules so that one authority defines "is a clone" ([Score.hs:22-25](../../../core/app/CE/Verdict/Score.hs#L22), [Cost.hs:7-10](../../../core/app/CE/Verdict/Cost.hs#L7)).

**Weights.** Each axis carries an effective weight: the wire's `weights` table supplies `[axisCode, w]` rows, and the first matching row wins; an unlisted axis takes `defaultWeight` ([Score.hs:199-202](../../../core/app/CE/Verdict/Score.hs#L199)), which is `1` ([Cost.hs:168-169](../../../core/app/CE/Verdict/Cost.hs#L168)) — equal weights are the opening stance. The same lookup that folds the score also builds the echoed table `0..6` returned in the reply, so the echo cannot diverge from the computation ([Score.hs:205-206](../../../core/app/CE/Verdict/Score.hs#L205), [Verdict.hs:116](../../../core/app/CE/Verdict.hs#L119)).

**The fold.** With `p_i` the axis charges (per-mille, bounded) and `w_i` the effective weights ([Score.hs:216-226](../../../core/app/CE/Verdict/Score.hs#L216)):

```
raw     = sum_i (w_i * p_i * violCost)
wTotal  = sum_i w_i                       -- derived, never a literal
score   = max 0 (scoreScale - raw `div` (violCostNeutral * wTotal))
```

`violCost = 10` ([Cost.hs:168-169](../../../core/app/CE/Verdict/Cost.hs#L168)), `violCostNeutral = 10` ([Cost.hs:175-176](../../../core/app/CE/Verdict/Cost.hs#L175)) and `scoreScale = 1000` ([Cost.hs:189-190](../../../core/app/CE/Verdict/Cost.hs#L189)) — i.e. the score is an integer per-mille value opening at 1000, polarity higher-is-better ([DEVELOPMENT_PLAN.md:71](../../DEVELOPMENT_PLAN.md#L71)). At the neutral default the score is exactly the weighted mean of the bounded axis charges, so the structural `max 0` is unreachable; `viol_cost` remains a live ce.toml dial — a repo declaring it above neutral asks for harsher scores and may saturate by that explicit choice ([Score.hs:235-242](../../../core/app/CE/Verdict/Score.hs#L235)). `div` is Haskell floor division. `wTotal` is summed from the effective weights rather than declared, so a weight can never be silently dropped from the divisor; validation refuses an all-zero weight table, making the divisor non-zero — that refusal lives in the boundary contract `CE.Verdict.Wire.violation` ([Verdict.hs:40-41](../../../core/app/CE/Verdict.hs#L40)), asserted there by the source comment.

An over-cap request never gets a partial judgment: it returns a fully-shaped degraded reply with `score = 0`, empty axes, and `fail = true` with reason `verdict_too_large` ([Verdict.hs:192-225](../../../core/app/CE/Verdict.hs#L207)). Caps are `verdictNodeCap = 131072` nodes and `verdictRowCap = 524288` rows ([Cost.hs:198-202](../../../core/app/CE/Verdict/Cost.hs#L198)), the row count summing every fact and knob table plus the baseline's rows ([Verdict.hs:47-72](../../../core/app/CE/Verdict.hs#L47)).

### The size soft zone (axis 0)

Axis 0 is the one axis whose mass is not a count. For a file of `x` LOC, with soft line `S` and hard line `H` ([Soft.hs:59-66](../../../core/app/CE/Verdict/Soft.hs#L59)):

```
p(x) = 0                               if x <= S
     = pMax                            if H <= S         -- degenerate fallback
     = pMax * ((x - S) / (H - S))^2    if S < x <= H
     = pMax * (1 + 2 * (x - H)/(H - S))  if x > H        -- C¹ linear arm
```

The curve is exact `Rational`; past `H` it continues **linearly** at exactly the slope the quadratic reached at the wall (`2·pMax/(H−S)`) — monotone, no kink, still charging every added line, but never quadratic outside the contracted `(S,H]` domain ([Soft.hs:44-58](../../../core/app/CE/Verdict/Soft.hs#L44), [size-advisory.md](../size-advisory.md) §A). The quadratic extrapolation this replaced is what saturated both field-test repositories at 0/1000 (proto 2.17.0 ledger). Denying at `H` is Rust's job (scan fail tier, guard budget); here `H` only scales the curve ([Cost.hs:113-118](../../../core/app/CE/Verdict/Cost.hs#L113)). The degenerate `H <= S` branch reproduces the pre-v0.6 binary behaviour instead of dividing by zero or flipping the curve's sign ([Soft.hs:61-62](../../../core/app/CE/Verdict/Soft.hs#L61)).

Constants: `sizeHard = 750` ([Cost.hs:117-118](../../../core/app/CE/Verdict/Cost.hs#L117)), `sizePMax = 10` ([Cost.hs:120-127](../../../core/app/CE/Verdict/Cost.hs#L120)) — one file at the hard line weighs like ten of any other axis's violations, which is what lets the size mass share the counting axes' odds scale in the density map. The per-file penalties are summed as `Rational` across all `metricCode = 0` rows ([Score.hs:156-160](../../../core/app/CE/Verdict/Score.hs#L156)) and the axis floors exactly once, inside `charge` ([Score.hs:141-147](../../../core/app/CE/Verdict/Score.hs#L141)), so the wire's axes rows stay `Integer`.

**Where `S` comes from.** `S` is relative to the repository, not a constant. Over the multiset of judged-language LOC values (positives only), with `m` the exact median and `r` the multiplicative MAD ([Soft.hs:33-42](../../../core/app/CE/Verdict/Soft.hs#L33)):

```
m = median(x)
r = median( max(x/m, m/x) )            -- >= 1 by construction
S = clamp(floor(m * r^k), [softMin, softMax])
```

This is the identity `S = clamp(median + k·MAD, ...)` in log-LOC space, re-expressed multiplicatively so no logarithm is ever taken; all order statistics are over `Rational` ([Soft.hs:1-8](../../../core/app/CE/Verdict/Soft.hs#L1), [Soft.hs:19-25](../../../core/app/CE/Verdict/Soft.hs#L19)). `k = softLineK = 2`, calibrated over self + requests + ripgrep so that `S` lands within ±6% of the historical 300 on this repo's judged set ([Cost.hs:128-133](../../../core/app/CE/Verdict/Cost.hs#L128)). The clamp fence is `[softMin, softMax] = [200, 500]`, declared structural rather than a knob ([Cost.hs:135-142](../../../core/app/CE/Verdict/Cost.hs#L135)). `floor` is the conservative direction: a lower `S` opens the graded zone earlier ([Soft.hs:27-32](../../../core/app/CE/Verdict/Soft.hs#L27)). An empty or all-empty LOC set yields `Nothing` — absence, never a fabricated line ([Soft.hs:34-35](../../../core/app/CE/Verdict/Soft.hs#L34)).

`S` is derived **only at establish** (no baseline present) and then frozen into the new baseline; every later run judges with the committed `S`, and a pre-v0.6 baseline carrying no `softLine` falls back to the `sizeCeil` knob `300` ([Verdict.hs:130-133](../../../core/app/CE/Verdict.hs#L120), [Score.hs:137](../../../core/app/CE/Verdict/Score.hs#L137), [Cost.hs:106-111](../../../core/app/CE/Verdict/Cost.hs#L106)). Because only the establish path reaches the derivation, re-anchoring the soft line requires `CE_ACCEPT_BASELINE` by construction ([Verdict.hs:101-104](../../../core/app/CE/Verdict.hs#L104)).

**Per-class lines (proto 3.1.0, plan v2.13 ①).** A continuous row may carry a fourth column, the file's *path class* — the 1-based index of the first `[[rules.class]]` whose globs match it, `0` for none — and the request may carry a `classKnobs` table `[classId, code, value]` whose codes are the ceilings' own `0 / 1 / 2` (`sizeCeil` / `cocCeil` / `sizeHard`) under a class; no new code was minted. The rows fold into one `Map` per judgment ([Score.hs:56-59](../../../core/app/CE/Verdict/Score.hs#L56), built once in [Verdict.hs:134-135](../../../core/app/CE/Verdict.hs#L134)) and a row's class is its fourth column or `0` ([Score.hs:63-66](../../../core/app/CE/Verdict/Score.hs#L63)); `sizeMass` measures a classed row against its class's own opening edge and hard line where declared, falling back to the global `S` and `H` ([Score.hs:170-178](../../../core/app/CE/Verdict/Score.hs#L170)), and `cocOver` likewise against the class's ceiling ([Score.hs:180-186](../../../core/app/CE/Verdict/Score.hs#L180)). The charge law is untouched — only the two lines a row is measured against move — so an unclassed repository judges byte-for-byte as before. The ratchet reads the three-column prefix alone ([Verdict.hs:150](../../../core/app/CE/Verdict.hs#L150)): a class is a charging parameter, never a baseline fact, and the baseline stays three columns. At the boundary a table mixing three- and four-column rows refuses ([Table.hs:48-52](../../../core/app/CE/Verdict/Table.hs#L48)), a class at or past `classCap = 64` refuses ([Cost.hs:76-77](../../../core/app/CE/Verdict/Cost.hs#L76), [Rows.hs:84-96](../../../core/app/CE/Verdict/Rows.hs#L84)), and the knob rows obey the ceilings grammar one class dimension wider — class `0` has no override channel, `(classId, code)` strictly ascending ([Table.hs:59-69](../../../core/app/CE/Verdict/Table.hs#L59)); that ordering is a validation fact, never a judgment fact (`ClassProps` pins the permutation). Names and globs never cross the wire; the index does.

### The continuous ratchet

ADR-006 defines per-file/per-function ceilings on continuous metrics (file LOC, function CoC): the ceiling is the baseline value; exceeding it fails, and coming in under it tightens the ceiling automatically. A single edit is allowed `+2%` or `+10` lines, whichever is larger, and consumed tolerance is reported in the `ce check` ratchet line ([DEVELOPMENT_PLAN.md:205-212](../../DEVELOPMENT_PLAN.md#L205)).

As built ([Ratchet.hs:55-56](../../../core/app/CE/Verdict/Ratchet.hs#L55)):

```
tolerated(c) = max (c * tolNum `div` tolDen) (c + tolAbs)
```

with `tolNum/tolDen = 102/100` and `tolAbs = 10` ([Cost.hs:90-97](../../../core/app/CE/Verdict/Cost.hs#L90)). Integer `div` truncates down — the conservative side, the "ties don't open" stance — and the two legs cross at ceiling `500`, with one property assertion pinned on each side ([Ratchet.hs:93-96](../../../core/app/CE/Verdict/Ratchet.hs#L93), [Cost.hs:86-89](../../../core/app/CE/Verdict/Cost.hs#L86)).

For each current row `[u, metricCode, v]` matched against the baseline ceiling `bv` for the same `(entity, metric)` key ([Ratchet.hs:62-90](../../../core/app/CE/Verdict/Ratchet.hs#L62)):

- `v > tolerated(bv)` → **over**, emitted as `[u, c, v, allowed]` ([Ratchet.hs:67-73](../../../core/app/CE/Verdict/Ratchet.hs#L67)). Note the comparison is strict, so `v == tolerated(bv)` is tolerated, not over.
- `bv < v <= tolerated(bv)` → **tolerance drawn**, emitted as `[u, c, v - bv]` for the Stop summary ([Ratchet.hs:3-7](../../../core/app/CE/Verdict/Ratchet.hs#L3), [Ratchet.hs:43-45](../../../core/app/CE/Verdict/Ratchet.hs#L43)).
- new ceiling = `min(v, bv)` — auto-tighten; an entity the baseline never saw adopts its current value as its ceiling (bootstrap, not a violation) ([Ratchet.hs:84](../../../core/app/CE/Verdict/Ratchet.hs#L84), [Ratchet.hs:58-62](../../../core/app/CE/Verdict/Ratchet.hs#L58)).

Tolerance is drawn per run against the *baseline* ceiling, and the new ceiling is `min(v, bv)`, so a drawn edit never raises the committed ceiling: the allowance does not accumulate across runs.

### The discrete ratchet

For discrete violations (clone instances, deadcode symbols) the baseline is a **set** of violation fingerprints; a new member fails, a removed member shrinks the baseline ([DEVELOPMENT_PLAN.md:209-210](../../DEVELOPMENT_PLAN.md#L209)). The implementation is plain set difference both ways over `Data.Set`, with the results returned in ascending order ([Ratchet.hs:82-83](../../../core/app/CE/Verdict/Ratchet.hs#L82), [Ratchet.hs:93-94](../../../core/app/CE/Verdict/Ratchet.hs#L93)):

```
added   = current \ baseline        -- non-empty => fail
removed = baseline \ current        -- informational; drives the shrink
newDisc = current                   -- verbatim, not intersected
```

`newDisc` is the current set verbatim ([Ratchet.hs:85](../../../core/app/CE/Verdict/Ratchet.hs#L85)); the "only shrink" invariant (`new ⊆ old`) is enforced by the *caller's* acceptance gate, not faked inside this function ([Ratchet.hs:62-65](../../../core/app/CE/Verdict/Ratchet.hs#L62)).

**Establish.** With no baseline, `ratchet` returns all four report lists empty and promotes the current facts wholesale to the new baseline — nothing can fail on the establishing run ([Ratchet.hs:36](../../../core/app/CE/Verdict/Ratchet.hs#L36), [Ratchet.hs:3-6](../../../core/app/CE/Verdict/Ratchet.hs#L3)).

### Composing the fail bit

ADR-006 makes the ratchet the primary gate for a repository that has a baseline and `--fail-under` a floor underneath it; either alone fails ([DEVELOPMENT_PLAN.md:211-212](../../DEVELOPMENT_PLAN.md#L211)). As built, the fail bit is the disjunction of four **named** conditions ([Verdict.hs:149-155](../../../core/app/CE/Verdict.hs#L9)):

| name | holds when |
|---|---|
| `ratchet_over` | the over list is non-empty |
| `discrete_added` | the added set is non-empty |
| `floor` | `score < reqFloor` — the `--fail-under` value, when supplied ([Verdict.hs:135](../../../core/app/CE/Verdict.hs#L44)) |
| `dedup_budget` | the request carried a `[blocks, budget]` pair and the judged count exceeds `budget` — since proto 2.19.0 that count is the core's own derivation from the shipped `distinct` rows, falling back to the client's `blocks` only when they are absent ([Verdict.hs:165-174](../../../core/app/CE/Verdict.hs#L155)) |

`fail = any` of them, and the reply also carries the list of the names that held, so a consumer attributes the failure by name rather than by reconstructing the conjunction ([Verdict.hs:141-142](../../../core/app/CE/Verdict.hs#L131), [Verdict.hs:96-99](../../../core/app/CE/Verdict.hs#L99)). The `removed` and `toleranceDrawn` lists are reported but never contribute to the fail bit. Note also that `dedup_budget` is only judged when the pair is present — `ce dedup --check` sends it, the `ce check` path does not ([Verdict.hs:146-148](../../../core/app/CE/Verdict.hs#L136)).

### Determinism notes

- Every knob above travels into the pure functions as a parameter; production binds them to the `Cost` constants exactly once, at `scoreBound` ([Score.hs:58-77](../../../core/app/CE/Verdict/Score.hs#L58)) and `ratchetBound` ([Ratchet.hs:36-37](../../../core/app/CE/Verdict/Ratchet.hs#L36)). That is what lets the perturbation batteries move one constant and watch the census move without touching production code ([Cost.hs:1-6](../../../core/app/CE/Verdict/Cost.hs#L1)).
- All arithmetic is `Integer` or `Rational`; the bounded-arithmetic ban is a recorded 2026-08-12 decision ([Cost.hs:12-14](../../../core/app/CE/Verdict/Cost.hs#L12)).
- The baseline crosses the wire verbatim from `ce-baseline.json` and is parsed exactly once, in `respond` ([Verdict.hs:9-11](../../../core/app/CE/Verdict.hs#L9), [Verdict.hs:44-45](../../../core/app/CE/Verdict.hs#L44)); Rust never interprets it.
