## Scoring and the ADR-006 ratchet

Everything in this section is computed in `ce-core` (Haskell), from the fact tables that arrive over the `verdict.request` wire. It is pure integer/`Rational` arithmetic — no floating point, no logarithms — so the same request always yields the same verdict.

### The aggregate score

Seven axes, indexed by a fixed code. Since proto 2.17.0 (the density migration, M9 batch 6) each axis pairs a non-negative violation **mass** `v` with its **opportunity** count `n` and charges the bounded density `floor(scale · v/(v+n))` per-mille ([Score.hs:92-121](../../../core/app/CE/Verdict/Score.hs#L92)) — strictly monotone in `v`, never reaching the scale, scale-free across repository sizes, and `0` when `n = 0` (no opportunity table — the honest-absence stance, [Score.hs:115-121](../../../core/app/CE/Verdict/Score.hs#L115)). The migration's cause is recorded in the wire ledger: under the old raw-mass fold two ordinary real repositories both measured 0/1000 ([VERSIONING.md](../../../contracts/VERSIONING.md), 2.17.0 entry).

| code | axis | violation mass `v` | opportunity `n` | knob(s) and value |
|---|---|---|---|---|
| 0 | size | convex soft-zone penalty summed over `metricCode = 0` rows, exact `Rational` ([Score.hs:130-134](../../../core/app/CE/Verdict/Score.hs#L130)) | files (`metricCode = 0` rows) | see soft zone below |
| 1 | complexity | count of `metricCode = 1` rows with `v > cocCeil` ([Score.hs:136-137](../../../core/app/CE/Verdict/Score.hs#L136)) | functions (`metricCode = 1` rows) | `cocCeil = 15` ([Cost.hs:142-143](../../../core/app/CE/Verdict/Cost.hs#L142)) |
| 2 | clone | count of sim rows with `kind <= 1` and `n * cloneDen >= d * cloneNum` ([Score.hs:139-141](../../../core/app/CE/Verdict/Score.hs#L139)) | files | `tsedNum/tsedDen = 85/100` ([Clone/Cost.hs:22](../../../core/app/CE/Clone/Cost.hs#L22), [Clone/Cost.hs:25](../../../core/app/CE/Clone/Cost.hs#L25)) |
| 3 | docdup | count of sim rows with `kind == 2` and `n * dupDen >= d * dupNum` ([Score.hs:154-155](../../../core/app/CE/Verdict/Score.hs#L154)) | files | `jaccardNum/jaccardDen = 80/100` ([Docdup/Cost.hs:27](../../../core/app/CE/Docdup/Cost.hs#L27), [Docdup/Cost.hs:30](../../../core/app/CE/Docdup/Cost.hs#L30)) |
| 4 | deadcode | count of pos rows with `reachIn == 0` and `indeg <= deadIndegCeil` ([Score.hs:146-147](../../../core/app/CE/Verdict/Score.hs#L146)) | graph file nodes (pos rows) | `deadIndegCeil = 0` ([Cost.hs:149-150](../../../core/app/CE/Verdict/Cost.hs#L149)) |
| 5 | churn | count of churn rows with `rw + ap > 0` and `rw * rewriteDen >= (rw + ap) * rewriteNum` ([Score.hs:149-151](../../../core/app/CE/Verdict/Score.hs#L149)) | churned entities (churn rows) | `rewriteNum/rewriteDen = 50/100` ([Cost.hs:81](../../../core/app/CE/Verdict/Cost.hs#L81), [Cost.hs:84](../../../core/app/CE/Verdict/Cost.hs#L84)) |
| 6 | graph_cycle | count of code-file pos rows with `sccSize >= cycleFloor`; `docFiles` indices are excluded ([Score.hs:164-170](../../../core/app/CE/Verdict/Score.hs#L164)) | graph file nodes minus documented files | `sccFloor = 2` ([Graph/Cost.hs:55](../../../core/app/CE/Graph/Cost.hs#L55)) |

All ratio thresholds are cross-multiplied rather than divided, so no rounding enters the predicates ([Score.hs:141](../../../core/app/CE/Verdict/Score.hs#L141), [Score.hs:144](../../../core/app/CE/Verdict/Score.hs#L144), [Score.hs:151](../../../core/app/CE/Verdict/Score.hs#L151)). Note the axis-2 and axis-3 thresholds are *not* owned by the verdict family: they are re-exported from the clone and docdup cost modules so that one authority defines "is a clone" ([Score.hs:20-23](../../../core/app/CE/Verdict/Score.hs#L20), [Cost.hs:7-10](../../../core/app/CE/Verdict/Cost.hs#L7)).

**Weights.** Each axis carries an effective weight: the wire's `weights` table supplies `[axisCode, w]` rows, and the first matching row wins; an unlisted axis takes `defaultWeight` ([Score.hs:161-164](../../../core/app/CE/Verdict/Score.hs#L161)), which is `1` ([Cost.hs:161-162](../../../core/app/CE/Verdict/Cost.hs#L161)) — equal weights are the opening stance. The same lookup that folds the score also builds the echoed table `0..6` returned in the reply, so the echo cannot diverge from the computation ([Score.hs:167-168](../../../core/app/CE/Verdict/Score.hs#L167), [Verdict.hs:116](../../../core/app/CE/Verdict.hs#L116)).

**The fold.** With `p_i` the axis charges (per-mille, bounded) and `w_i` the effective weights ([Score.hs:178-188](../../../core/app/CE/Verdict/Score.hs#L178)):

```
raw     = sum_i (w_i * p_i * violCost)
wTotal  = sum_i w_i                       -- derived, never a literal
score   = max 0 (scoreScale - raw `div` (violCostNeutral * wTotal))
```

`violCost = 10` ([Cost.hs:161-162](../../../core/app/CE/Verdict/Cost.hs#L161)), `violCostNeutral = 10` ([Cost.hs:168-169](../../../core/app/CE/Verdict/Cost.hs#L168)) and `scoreScale = 1000` ([Cost.hs:182-183](../../../core/app/CE/Verdict/Cost.hs#L182)) — i.e. the score is an integer per-mille value opening at 1000, polarity higher-is-better ([DEVELOPMENT_PLAN.md:71](../../DEVELOPMENT_PLAN.md#L71)). At the neutral default the score is exactly the weighted mean of the bounded axis charges, so the structural `max 0` is unreachable; `viol_cost` remains a live ce.toml dial — a repo declaring it above neutral asks for harsher scores and may saturate by that explicit choice ([Score.hs:197-204](../../../core/app/CE/Verdict/Score.hs#L197)). `div` is Haskell floor division. `wTotal` is summed from the effective weights rather than declared, so a weight can never be silently dropped from the divisor; validation refuses an all-zero weight table, making the divisor non-zero — that refusal lives in the boundary contract `CE.Verdict.Wire.violation` ([Verdict.hs:40-41](../../../core/app/CE/Verdict.hs#L40)), asserted there by the source comment.

An over-cap request never gets a partial judgment: it returns a fully-shaped degraded reply with `score = 0`, empty axes, and `fail = true` with reason `verdict_too_large` ([Verdict.hs:192-225](../../../core/app/CE/Verdict.hs#L192)). Caps are `verdictNodeCap = 131072` nodes and `verdictRowCap = 524288` rows ([Cost.hs:191-195](../../../core/app/CE/Verdict/Cost.hs#L191)), the row count summing every fact and knob table plus the baseline's rows ([Verdict.hs:47-72](../../../core/app/CE/Verdict.hs#L47)).

### The size soft zone (axis 0)

Axis 0 is the one axis whose mass is not a count. For a file of `x` LOC, with soft line `S` and hard line `H` ([Soft.hs:59-66](../../../core/app/CE/Verdict/Soft.hs#L59)):

```
p(x) = 0                               if x <= S
     = pMax                            if H <= S         -- degenerate fallback
     = pMax * ((x - S) / (H - S))^2    if S < x <= H
     = pMax * (1 + 2 * (x - H)/(H - S))  if x > H        -- C¹ linear arm
```

The curve is exact `Rational`; past `H` it continues **linearly** at exactly the slope the quadratic reached at the wall (`2·pMax/(H−S)`) — monotone, no kink, still charging every added line, but never quadratic outside the contracted `(S,H]` domain ([Soft.hs:44-58](../../../core/app/CE/Verdict/Soft.hs#L44), [size-advisory.md](../size-advisory.md) §A). The quadratic extrapolation this replaced is what saturated both field-test repositories at 0/1000 (proto 2.17.0 ledger). Denying at `H` is Rust's job (scan fail tier, guard budget); here `H` only scales the curve ([Cost.hs:106-111](../../../core/app/CE/Verdict/Cost.hs#L106)). The degenerate `H <= S` branch reproduces the pre-v0.6 binary behaviour instead of dividing by zero or flipping the curve's sign ([Soft.hs:61-62](../../../core/app/CE/Verdict/Soft.hs#L61)).

Constants: `sizeHard = 750` ([Cost.hs:110-111](../../../core/app/CE/Verdict/Cost.hs#L110)), `sizePMax = 10` ([Cost.hs:113-120](../../../core/app/CE/Verdict/Cost.hs#L113)) — one file at the hard line weighs like ten of any other axis's violations, which is what lets the size mass share the counting axes' odds scale in the density map. The per-file penalties are summed as `Rational` across all `metricCode = 0` rows ([Score.hs:130-134](../../../core/app/CE/Verdict/Score.hs#L130)) and the axis floors exactly once, inside `charge` ([Score.hs:115-121](../../../core/app/CE/Verdict/Score.hs#L115)), so the wire's axes rows stay `Integer`.

**Where `S` comes from.** `S` is relative to the repository, not a constant. Over the multiset of judged-language LOC values (positives only), with `m` the exact median and `r` the multiplicative MAD ([Soft.hs:33-42](../../../core/app/CE/Verdict/Soft.hs#L33)):

```
m = median(x)
r = median( max(x/m, m/x) )            -- >= 1 by construction
S = clamp(floor(m * r^k), [softMin, softMax])
```

This is the identity `S = clamp(median + k·MAD, ...)` in log-LOC space, re-expressed multiplicatively so no logarithm is ever taken; all order statistics are over `Rational` ([Soft.hs:1-8](../../../core/app/CE/Verdict/Soft.hs#L1), [Soft.hs:19-25](../../../core/app/CE/Verdict/Soft.hs#L19)). `k = softLineK = 2`, calibrated over self + requests + ripgrep so that `S` lands within ±6% of the historical 300 on this repo's judged set ([Cost.hs:121-126](../../../core/app/CE/Verdict/Cost.hs#L121)). The clamp fence is `[softMin, softMax] = [200, 500]`, declared structural rather than a knob ([Cost.hs:128-135](../../../core/app/CE/Verdict/Cost.hs#L128)). `floor` is the conservative direction: a lower `S` opens the graded zone earlier ([Soft.hs:27-32](../../../core/app/CE/Verdict/Soft.hs#L27)). An empty or all-empty LOC set yields `Nothing` — absence, never a fabricated line ([Soft.hs:34-35](../../../core/app/CE/Verdict/Soft.hs#L34)).

`S` is derived **only at establish** (no baseline present) and then frozen into the new baseline; every later run judges with the committed `S`, and a pre-v0.6 baseline carrying no `softLine` falls back to the `sizeCeil` knob `300` ([Verdict.hs:130-133](../../../core/app/CE/Verdict.hs#L130), [Score.hs:111](../../../core/app/CE/Verdict/Score.hs#L111), [Cost.hs:99-104](../../../core/app/CE/Verdict/Cost.hs#L99)). Because only the establish path reaches the derivation, re-anchoring the soft line requires `CE_ACCEPT_BASELINE` by construction ([Verdict.hs:101-104](../../../core/app/CE/Verdict.hs#L101)).

### The continuous ratchet

ADR-006 defines per-file/per-function ceilings on continuous metrics (file LOC, function CoC): the ceiling is the baseline value; exceeding it fails, and coming in under it tightens the ceiling automatically. A single edit is allowed `+2%` or `+10` lines, whichever is larger, and consumed tolerance is reported in the `ce check` ratchet line ([DEVELOPMENT_PLAN.md:205-212](../../DEVELOPMENT_PLAN.md#L205)).

As built ([Ratchet.hs:55-56](../../../core/app/CE/Verdict/Ratchet.hs#L55)):

```
tolerated(c) = max (c * tolNum `div` tolDen) (c + tolAbs)
```

with `tolNum/tolDen = 102/100` and `tolAbs = 10` ([Cost.hs:90-97](../../../core/app/CE/Verdict/Cost.hs#L90)). Integer `div` truncates down — the conservative side, the "ties don't open" stance — and the two legs cross at ceiling `500`, with one property assertion pinned on each side ([Ratchet.hs:51-54](../../../core/app/CE/Verdict/Ratchet.hs#L51), [Cost.hs:86-89](../../../core/app/CE/Verdict/Cost.hs#L86)).

For each current row `[u, metricCode, v]` matched against the baseline ceiling `bv` for the same `(entity, metric)` key ([Ratchet.hs:66-94](../../../core/app/CE/Verdict/Ratchet.hs#L66)):

- `v > tolerated(bv)` → **over**, emitted as `[u, c, v, allowed]` ([Ratchet.hs:70-76](../../../core/app/CE/Verdict/Ratchet.hs#L70)). Note the comparison is strict, so `v == tolerated(bv)` is tolerated, not over.
- `bv < v <= tolerated(bv)` → **tolerance drawn**, emitted as `[u, c, v - bv]` for the Stop summary ([Ratchet.hs:77-81](../../../core/app/CE/Verdict/Ratchet.hs#L77), [Ratchet.hs:43-45](../../../core/app/CE/Verdict/Ratchet.hs#L43)).
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

**Establish.** With no baseline, `ratchet` returns all four report lists empty and promotes the current facts wholesale to the new baseline — nothing can fail on the establishing run ([Ratchet.hs:67](../../../core/app/CE/Verdict/Ratchet.hs#L67), [Ratchet.hs:3-6](../../../core/app/CE/Verdict/Ratchet.hs#L3)).

### Composing the fail bit

ADR-006 makes the ratchet the primary gate for a repository that has a baseline and `--fail-under` a floor underneath it; either alone fails ([DEVELOPMENT_PLAN.md:211-212](../../DEVELOPMENT_PLAN.md#L211)). As built, the fail bit is the disjunction of four **named** conditions ([Verdict.hs:149-155](../../../core/app/CE/Verdict.hs#L149)):

| name | holds when |
|---|---|
| `ratchet_over` | the over list is non-empty |
| `discrete_added` | the added set is non-empty |
| `floor` | `score < reqFloor` — the `--fail-under` value, when supplied ([Verdict.hs:135](../../../core/app/CE/Verdict.hs#L135)) |
| `dedup_budget` | the request carried a `[blocks, budget]` pair and `blocks > budget` ([Verdict.hs:138-140](../../../core/app/CE/Verdict.hs#L138)) |

`fail = any` of them, and the reply also carries the list of the names that held, so a consumer attributes the failure by name rather than by reconstructing the conjunction ([Verdict.hs:141-142](../../../core/app/CE/Verdict.hs#L141), [Verdict.hs:96-99](../../../core/app/CE/Verdict.hs#L96)). The `removed` and `toleranceDrawn` lists are reported but never contribute to the fail bit. Note also that `dedup_budget` is only judged when the pair is present — `ce dedup --check` sends it, the `ce check` path does not ([Verdict.hs:146-148](../../../core/app/CE/Verdict.hs#L146)).

### Determinism notes

- Every knob above travels into the pure functions as a parameter; production binds them to the `Cost` constants exactly once, at `scoreBound` ([Score.hs:58-77](../../../core/app/CE/Verdict/Score.hs#L58)) and `ratchetBound` ([Ratchet.hs:36-37](../../../core/app/CE/Verdict/Ratchet.hs#L36)). That is what lets the perturbation batteries move one constant and watch the census move without touching production code ([Cost.hs:1-6](../../../core/app/CE/Verdict/Cost.hs#L1)).
- All arithmetic is `Integer` or `Rational`; the bounded-arithmetic ban is a recorded 2026-08-12 decision ([Cost.hs:12-14](../../../core/app/CE/Verdict/Cost.hs#L12)).
- The baseline crosses the wire verbatim from `ce-baseline.json` and is parsed exactly once, in `respond` ([Verdict.hs:9-11](../../../core/app/CE/Verdict.hs#L9), [Verdict.hs:44-45](../../../core/app/CE/Verdict.hs#L44)); Rust never interprets it.
