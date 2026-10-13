# Score trajectory — the trend slope verdict

[index](../methodology.md) · [← 09 Edit four-classification (update supervision)](09-edit-four-classification-update-supervision.md) · [→ 11 FPR discipline and the guard tier ladder](11-fpr-discipline-and-the-guard-tier-ladder.md)

The eighth judgment family answers one question about a repository's history: is the check score going up, flat, or down, and by how much per day — and since trend/2, where it fell hardest and for how long. Rust measures the trajectory; Haskell judges it. Rust computes no policy — sign, floor, the fail bit and both shape facts all come back on the wire ([judge.rs:5](../../../cli/src/trend/judge.rs#L5)).

### Measurement — what becomes a row

The window is the newest `n` **first-parent** commits of `HEAD` ([mod.rs:139](../../../cli/src/trend/mod.rs#L139)). Each uncached commit is scored in a detached temp worktree against the committed soft line rather than one re-derived per point ([mod.rs:160](../../../cli/src/trend/mod.rs#L160), the soft pin from [mod.rs:101](../../../cli/src/trend/mod.rs#L101)). Since plan v2.18 step #14 the point also judges against a pinned identity baseline — every ceiling equal to its own measurement, its own member set, the run's digest ([pinned.rs:21-40](../../../cli/src/score/pinned.rs#L21)) — so the ratchet can neither draw nor fail on the point's own facts, and a verdict that still fails is a tripwire the row never reaches ([mod.rs:173-184](../../../cli/src/trend/mod.rs#L173)). Rows are cached in `.ce/index.db` stamped with the measuring toolchain ([mod.rs:100](../../../cli/src/trend/mod.rs#L100)) and reversed to oldest-first for chart order before judgment ([mod.rs:127](../../../cli/src/trend/mod.rs#L127)) — a presentation choice only; the judged view below sorts for itself. Only `[ts, score, scale]` triples cross the wire — no commit hashes, no paths ([judge.rs:51](../../../cli/src/trend/judge.rs#L51)).

### Boundary contract

Before any arithmetic, the first offender in request order is named and the request refused — the knobbed-table cascade every such family runs, rows first, then knobs ([Wire.hs:208](../../../core/app/CE/Wire.hs#L208)), bound to this family's row and knob contracts ([Trend.hs:30](../../../core/app/CE/Trend.hs#L30)):

- row must be exactly `[ts, score, scale]`, with `ts >= 0`, `scale > 0`, and `0 <= score <= scale` ([Trend.hs:39-47](../../../core/app/CE/Trend.hs#L39));
- knob must be `[code, value]` with `code ∈ {0, 1}`, `value >= 0`, and — for `code == 0` — `value >= 2` ([Trend.hs:51-57](../../../core/app/CE/Trend.hs#L51));
- knob codes must be strictly ascending — one knob-table contract every knobbed family reads ([Rows.hs:74-81](../../../core/app/CE/Wire/Rows.hs#L74)).

Row **order is deliberately unconstrained**: the judged view sorts by timestamp, and first-parent order is topological rather than chronological, so rebased or backdated commits are legal input. The property `orderFree` pins that a shuffled window states the same slope, verdict and fail — and the same cliff FACT: the request index moves with the request, the timestamp it points at must not ([TrendProps.hs:246](../../../core/test/TrendProps.hs#L246)).

Cap: `length rows + length knobs > 4096` produces a complete **degraded** reply rather than a truncated one, with `reason = "trend_too_large"` ([Cost.hs:37](../../../core/app/CE/Trend/Cost.hs#L37), [Trend.hs:126](../../../core/app/CE/Trend.hs#L126)). One row per mainline commit; 4096 covers roughly a decade of daily commits. The measuring side never asks for a window that cannot fit: the package states the cap (`limits.caps.trend_rows`), and a `--commits` past it less the declared knob rows is refused by name before a commit is measured ([judge.rs:112-115](../../../cli/src/trend/judge.rs#L112)); a degraded reply is read before any field, as the core's reason ([judge.rs:61](../../../cli/src/trend/judge.rs#L61)).

### The slope — exact Rational Theil-Sen

No floating point ever crosses a verdict ([Cost.hs:4](../../../core/app/CE/Trend/Cost.hs#L4)). Each row maps to an exact `Rational` point:

```
x_i = ts_i % 86400              -- seconds → days, exact ratio
y_i = (score_i * 1000000) % scale_i
```

([Cost.hs:61](../../../core/app/CE/Trend/Cost.hs#L61)). `y` renormalizes every commit onto a fixed `10^6` full-scale grid, so rows measured under different `scoreScale` values are commensurable. With the usual `scoreScale = 1000`, one per-mille point of score equals 1000 y-units — a decline of 1‰ per day reads as slope `-1000`.

The slope is the **Theil-Sen estimator** — the median of pairwise slopes over timestamp-distinct pairs:

```
slope = median{ (y_j − y_i) / (x_j − x_i) : x_i ≠ x_j }
```

([Cost.hs:73](../../../core/app/CE/Trend/Cost.hs#L73), the median a plain order statistic with even counts averaged — the one median, which the soft line reads too, [Soft.hs:31-42](../../../core/app/CE/Verdict/Soft.hs#L31)). Units: **micro-per-mille per day**. The estimator change (2.31.0, the capability renamed `trend/2` to say so) is a recorded behavior change bought for robustness: one wild point — a broken commit that still measured — drags a least-squares mean anywhere; it cannot move a median past its neighbors. `TrendProps` pins the counterfactual where the two estimators disagree on the SIGN: five points falling 1‰/day plus one wild high outlier — the retired mean, kept in the test as the reference, says `+14000`; the median says `-1000`, through the real `respond` ([TrendProps.hs:91](../../../core/test/TrendProps.hs#L91)). On an exact line every pairwise slope is the line's, so the median is it ([TrendProps.hs:79](../../../core/test/TrendProps.hs#L79)); the grid property checks the shipped walk against an independent derivation — index-pair enumeration plus insertion-sort median ([TrendProps.hs:43](../../../core/test/TrendProps.hs#L43)).

It is `Nothing` — underdetermined, not zero — when no pair has distinct timestamps: same-second commits are a legal, common shape (rebases, scripted pushes), so ties ride the wire, and only the *all-tied* window has no RATE to state — the falls it contains still report as shape facts ([Cost.hs:74](../../../core/app/CE/Trend/Cost.hs#L74)).

### The judged window — tsWindow

Theil-Sen prices `n(n−1)/2` pairwise slopes, so the judgment window is bounded at the judgment, not by the wire cap: the view keeps the `tsWindow = 512` most recent points — 130,816 pairs, measured ~150 ms request-to-reply on the dev machine, median of 5 with process spawn included ([Cost.hs:47](../../../core/app/CE/Trend/Cost.hs#L47)). Rows are stable-sorted by timestamp — ties keep request order — with request indices preserved, and ONE decomposition feeds slope, cliff and decline run alike ([Cost.hs:56](../../../core/app/CE/Trend/Cost.hs#L56)). Older rows still cross and are counted: `counts.rows` is the request, `counts.judged` names the cut ([Trend.hs:125](../../../core/app/CE/Trend.hs#L125)). The `windowed` property sends 513 rows whose OLDEST is a wild outlier: the kept 512 are an exact line and the slope is exactly the line's — the outlier left no trace ([TrendProps.hs:199](../../../core/test/TrendProps.hs#L199)).

### Shape facts — the cliff and the decline run

Two facts about the sorted walk ride beside the slope, each naming a commit by **request index** — the row order the client sent; hashes never cross (§5.9.2):

- `cliff = [i, drop]` — the steepest single-step fall between consecutive points: the request index of the LATER point and the drop in micro units. The first occurrence wins a tie; a monotone rise has no cliff (`null`). A fall between same-second commits counts — the drop is a fact about scores, not about time ([Cost.hs:91](../../../core/app/CE/Trend/Cost.hs#L91)).
- `declineRun = [i, k]` — the longest run of consecutive strictly-falling steps: the request index of the run's FIRST point and the number of points in the run. The first run wins a length tie; no falling step, no run ([Cost.hs:110](../../../core/app/CE/Trend/Cost.hs#L110)).

Both are facts, not verdicts: no knob arms them, and they cannot fail a gate. On the Rust side the indices are fenced against the rows actually sent — an index past the request is core drift, refused before anything renders a commit name from it ([judge.rs:75](../../../cli/src/trend/judge.rs#L75)) — and the core's console lines name the commit behind each one ([Lines.hs:42](../../../core/app/CE/Trend/Lines.hs#L42)).

### minPoints — absence, never a fabricated flat

`minPoints` is knob code `0`, default `3`. Below it nothing is judged:

```haskell
enough = toInteger (length (rowsOf req)) >= minPoints
slope = if enough then slopeMicroPerDay view else Nothing
verdict = verdictOf floorMicro <$> slope
```

([Trend.hs:82](../../../core/app/CE/Trend.hs#L82)). EVERY judgment field serializes as JSON `null` — slope, verdict, cliff, declineRun — and the fail bit stays `false`: nothing was judged, and an unjudged trend must not gate. This is the one case distinguished from a degraded reply, where judgment was *denied* by the cap and `fail = true` says so. The `absence` property pins all four fields to this posture ([TrendProps.hs:220](../../../core/test/TrendProps.hs#L220)).

Validation forbids `minPoints < 2` ([Trend.hs:56](../../../core/app/CE/Trend.hs#L56)), so the knob can never demand a slope from a single point.

### Verdict codes and the decline floor

`declineFloorMicro` is knob code `1`, default `0`. It defines a symmetric dead band around zero:

```
slope < −band  → 2  (degrading)
slope >  band  → 0  (improving)
otherwise      → 1  (flat)        where band = floorMicro
```

([Cost.hs:128](../../../core/app/CE/Trend/Cost.hs#L128)). The band is **inclusive**: at slope `-1000`, floor `999` says degrading but floor `1000` says flat ([TrendProps.hs:112](../../../core/test/TrendProps.hs#L112)). With the default floor `0` the band collapses to the single point zero, so the raw sign is the report. Console words for the three codes plus the unjudged case are rendering only, in the core's console lines ([Lines.hs:37](../../../core/app/CE/Trend/Lines.hs#L37)).

**Fail arming** is a separate decision made at the reply, not in the classifier:

```haskell
"fail" .= (jDegraded j || (jVerdict j == Just 2 && lookup 1 (jEffective j) > Just 0))
```

([Trend.hs:122](../../../core/app/CE/Trend.hs#L122)). Two conjuncts on the judged path: the verdict must be `2` (degrading) **and** a floor must have been declared strictly greater than `0`. Floor `0` — the default every family launches in — is a report-only posture: it can report degrading and cannot fail. The `floorLever` property runs the same falling-1‰/day rows through the real `respond` three times: floor absent → verdict `2`, fail `false`; floor `500` → verdict `2`, fail `true`; floor `5000` → verdict `1`, fail `false` ([TrendProps.hs:139](../../../core/test/TrendProps.hs#L139)).

The degraded path fails unconditionally, and echoes the **default** knob table rather than the request's unvalidated override.

### Serialization and the round-trip pin

The slope and the cliff's drop are judged exactly and only *displayed* rounded: `round` is round-half-even, and the verdict compared the exact values — no client re-derives them ([Trend.hs:120](../../../core/app/CE/Trend.hs#L120)).

On the Rust side, knobs ride the wire only when `ce.toml` declares them — `[trend] min_points` → code `0`, `decline_floor_micro` → code `1`, both `Option` ([judge.rs:90](../../../cli/src/trend/judge.rs#L90), [config.rs:122-124](../../../cli/src/config.rs#L122)). The effective-knob echo is verified, not trusted: exactly two rows must come back, and every knob the request sent must echo the value it sent, or the report errors out ([judge.rs:132](../../../cli/src/trend/judge.rs#L132)). A missing `trend/2` capability or a non-`trend.result` reply is an error, never a silently unjudged report ([judge.rs:45](../../../cli/src/trend/judge.rs#L45)). Report schema id: <!--ce:report:trend#schemaver-->`ce.trend-report/0.3.0`<!--/ce--> ([Document.hs:81](../../../core/app/CE/Trend/Document.hs#L81)).

### Not found in source

Since trend/2 the judgment window IS bounded — `tsWindow = 512` — but the prose should not conflate it with the `4096`-row wire cap: rows past the window still cross, count, and refuse offenders; they are simply outside the judged view. The window's *default breadth* is a separate constant again: `DEFAULT_COMMITS = 30`, shared by every face ([mod.rs:31](../../../cli/src/trend/mod.rs#L31)) — before it existed the clap face carried a `30` literal and the MCP adapter a `10`.
