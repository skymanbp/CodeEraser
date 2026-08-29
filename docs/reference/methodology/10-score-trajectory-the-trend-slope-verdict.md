# Score trajectory — the trend slope verdict

[index](../methodology.md) · [← 09 Edit four-classification (update supervision)](09-edit-four-classification-update-supervision.md) · [→ 11 FPR discipline and the guard tier ladder](11-fpr-discipline-and-the-guard-tier-ladder.md)

The eighth judgment family answers one question about a repository's history: is the check score going up, flat, or down, and by how much per day — and since trend/2, where it fell hardest and for how long. Rust measures the trajectory; Haskell judges it. Rust computes no policy — sign, floor, the fail bit and both shape facts all come back on the wire ([judge.rs:5](../../../cli/src/trend/judge.rs#L5)).

### Measurement — what becomes a row

The window is the newest `n` **first-parent** commits of `HEAD` ([mod.rs:113](../../../cli/src/trend/mod.rs#L113)). Each uncached commit is scored in a detached temp worktree against the committed soft line rather than one re-derived per point ([mod.rs:134](../../../cli/src/trend/mod.rs#L134), the soft pin from [mod.rs:76](../../../cli/src/trend/mod.rs#L76)). Since plan v2.18 step #14 the point also judges against a pinned identity baseline — every ceiling equal to its own measurement, its own member set, the run's digest ([pinned.rs:21-40](../../../cli/src/score/pinned.rs#L21)) — so the ratchet can neither draw nor fail on the point's own facts, and a verdict that still fails is a tripwire the row never reaches ([mod.rs:147-158](../../../cli/src/trend/mod.rs#L147)). Rows are cached in `.ce/index.db` stamped with the measuring toolchain ([mod.rs:75](../../../cli/src/trend/mod.rs#L75)) and reversed to oldest-first for chart order before judgment ([mod.rs:102](../../../cli/src/trend/mod.rs#L102)) — a presentation choice only; the judged view below sorts for itself. Only `[ts, score, scale]` triples cross the wire — no commit hashes, no paths ([judge.rs:57](../../../cli/src/trend/judge.rs#L57)).

### Boundary contract

Before any arithmetic, the first offender in request order is named and the request refused ([Trend.hs:44](../../../core/app/CE/Trend.hs#L44)):

- row must be exactly `[ts, score, scale]`, with `ts >= 0`, `scale > 0`, and `0 <= score <= scale` ([Trend.hs:52](../../../core/app/CE/Trend.hs#L52));
- knob must be `[code, value]` with `code ∈ {0, 1}`, `value >= 0`, and — for `code == 0` — `value >= 2` ([Trend.hs:63](../../../core/app/CE/Trend.hs#L63));
- knob codes must be strictly ascending ([Trend.hs:48](../../../core/app/CE/Trend.hs#L48)).

Row **order is deliberately unconstrained**: the judged view sorts by timestamp, and first-parent order is topological rather than chronological, so rebased or backdated commits are legal input. The property `orderFree` pins that a shuffled window states the same slope, verdict and fail — and the same cliff FACT: the request index moves with the request, the timestamp it points at must not ([TrendProps.hs:246](../../../core/test/TrendProps.hs#L246)).

Cap: `length rows + length knobs > 4096` produces a complete **degraded** reply rather than a truncated one, with `reason = "trend_too_large"` ([Cost.hs:36](../../../core/app/CE/Trend/Cost.hs#L36), [Trend.hs:141](../../../core/app/CE/Trend.hs#L141)). One row per mainline commit; 4096 covers roughly a decade of daily commits.

### The slope — exact Rational Theil-Sen

No floating point ever crosses a verdict ([Cost.hs:4](../../../core/app/CE/Trend/Cost.hs#L4)). Each row maps to an exact `Rational` point:

```
x_i = ts_i % 86400              -- seconds → days, exact ratio
y_i = (score_i * 1000000) % scale_i
```

([Cost.hs:60](../../../core/app/CE/Trend/Cost.hs#L60)). `y` renormalizes every commit onto a fixed `10^6` full-scale grid, so rows measured under different `scoreScale` values are commensurable. With the usual `scoreScale = 1000`, one per-mille point of score equals 1000 y-units — a decline of 1‰ per day reads as slope `-1000`.

The slope is the **Theil-Sen estimator** — the median of pairwise slopes over timestamp-distinct pairs:

```
slope = median{ (y_j − y_i) / (x_j − x_i) : x_i ≠ x_j }
```

([Cost.hs:72](../../../core/app/CE/Trend/Cost.hs#L72), the median a plain order statistic with even counts averaged, [Cost.hs:86](../../../core/app/CE/Trend/Cost.hs#L86)). Units: **micro-per-mille per day**. The estimator change (2.31.0, the capability renamed `trend/2` to say so) is a recorded behavior change bought for robustness: one wild point — a broken commit that still measured — drags a least-squares mean anywhere; it cannot move a median past its neighbors. `TrendProps` pins the counterfactual where the two estimators disagree on the SIGN: five points falling 1‰/day plus one wild high outlier — the retired mean, kept in the test as the reference, says `+14000`; the median says `-1000`, through the real `respond` ([TrendProps.hs:91](../../../core/test/TrendProps.hs#L91)). On an exact line every pairwise slope is the line's, so the median is it ([TrendProps.hs:79](../../../core/test/TrendProps.hs#L79)); the grid property checks the shipped walk against an independent derivation — index-pair enumeration plus insertion-sort median ([TrendProps.hs:43](../../../core/test/TrendProps.hs#L43)).

It is `Nothing` — underdetermined, not zero — when no pair has distinct timestamps: same-second commits are a legal, common shape (rebases, scripted pushes), so ties ride the wire, and only the *all-tied* window has no RATE to state — the falls it contains still report as shape facts ([Cost.hs:73](../../../core/app/CE/Trend/Cost.hs#L73)).

### The judged window — tsWindow

Theil-Sen prices `n(n−1)/2` pairwise slopes, so the judgment window is bounded at the judgment, not by the wire cap: the view keeps the `tsWindow = 512` most recent points — 130,816 pairs, measured ~150 ms request-to-reply on the dev machine, median of 5 with process spawn included ([Cost.hs:46](../../../core/app/CE/Trend/Cost.hs#L46)). Rows are stable-sorted by timestamp — ties keep request order — with request indices preserved, and ONE decomposition feeds slope, cliff and decline run alike ([Cost.hs:55](../../../core/app/CE/Trend/Cost.hs#L55)). Older rows still cross and are counted: `counts.rows` is the request, `counts.judged` names the cut ([Trend.hs:138](../../../core/app/CE/Trend.hs#L138)). The `windowed` property sends 513 rows whose OLDEST is a wild outlier: the kept 512 are an exact line and the slope is exactly the line's — the outlier left no trace ([TrendProps.hs:199](../../../core/test/TrendProps.hs#L199)).

### Shape facts — the cliff and the decline run

Two facts about the sorted walk ride beside the slope, each naming a commit by **request index** — the row order the client sent; hashes never cross (§5.9.2):

- `cliff = [i, drop]` — the steepest single-step fall between consecutive points: the request index of the LATER point and the drop in micro units. The first occurrence wins a tie; a monotone rise has no cliff (`null`). A fall between same-second commits counts — the drop is a fact about scores, not about time ([Cost.hs:102](../../../core/app/CE/Trend/Cost.hs#L102)).
- `declineRun = [i, k]` — the longest run of consecutive strictly-falling steps: the request index of the run's FIRST point and the number of points in the run. The first run wins a length tie; no falling step, no run ([Cost.hs:121](../../../core/app/CE/Trend/Cost.hs#L121)).

Both are facts, not verdicts: no knob arms them, and they cannot fail a gate. On the Rust side the indices are fenced against the rows actually sent — an index past the request is core drift, refused before anything renders a commit name from it ([judge.rs:75](../../../cli/src/trend/judge.rs#L75)) — and the console names the commit behind each one ([report.rs:115](../../../cli/src/trend/report.rs#L115)).

### minPoints — absence, never a fabricated flat

`minPoints` is knob code `0`, default `3`. Below it nothing is judged:

```haskell
enough = toInteger (length (rowsOf req)) >= minPoints
slope = if enough then slopeMicroPerDay view else Nothing
verdict = verdictOf floorMicro <$> slope
```

([Trend.hs:96](../../../core/app/CE/Trend.hs#L96)). EVERY judgment field serializes as JSON `null` — slope, verdict, cliff, declineRun — and the fail bit stays `false`: nothing was judged, and an unjudged trend must not gate. This is the one case distinguished from a degraded reply, where judgment was *denied* by the cap and `fail = true` says so. The `absence` property pins all four fields to this posture ([TrendProps.hs:220](../../../core/test/TrendProps.hs#L220)).

Validation forbids `minPoints < 2` ([Trend.hs:67](../../../core/app/CE/Trend.hs#L67)), so the knob can never demand a slope from a single point.

### Verdict codes and the decline floor

`declineFloorMicro` is knob code `1`, default `0`. It defines a symmetric dead band around zero:

```
slope < −band  → 2  (degrading)
slope >  band  → 0  (improving)
otherwise      → 1  (flat)        where band = floorMicro
```

([Cost.hs:139](../../../core/app/CE/Trend/Cost.hs#L139)). The band is **inclusive**: at slope `-1000`, floor `999` says degrading but floor `1000` says flat ([TrendProps.hs:112](../../../core/test/TrendProps.hs#L112)). With the default floor `0` the band collapses to the single point zero, so the raw sign is the report. Console words for the three codes plus the unjudged case are rendering only ([judge.rs:127](../../../cli/src/trend/judge.rs#L127)).

**Fail arming** is a separate decision made at the reply, not in the classifier:

```haskell
"fail" .= (jDegraded j || (jVerdict j == Just 2 && lookup 1 (jEffective j) > Just 0))
```

([Trend.hs:136](../../../core/app/CE/Trend.hs#L136)). Two conjuncts on the judged path: the verdict must be `2` (degrading) **and** a floor must have been declared strictly greater than `0`. Floor `0` — the default every family launches in — is a report-only posture: it can report degrading and cannot fail. The `floorLever` property runs the same falling-1‰/day rows through the real `respond` three times: floor absent → verdict `2`, fail `false`; floor `500` → verdict `2`, fail `true`; floor `5000` → verdict `1`, fail `false` ([TrendProps.hs:139](../../../core/test/TrendProps.hs#L139)).

The degraded path fails unconditionally, and echoes the **default** knob table rather than the request's unvalidated override.

### Serialization and the round-trip pin

The slope and the cliff's drop are judged exactly and only *displayed* rounded: `round` is round-half-even, and the verdict compared the exact values — no client re-derives them ([Trend.hs:134](../../../core/app/CE/Trend.hs#L134)).

On the Rust side, knobs ride the wire only when `ce.toml` declares them — `[trend] min_points` → code `0`, `decline_floor_micro` → code `1`, both `Option` ([judge.rs:49](../../../cli/src/trend/judge.rs#L49), [config.rs:138-140](../../../cli/src/config.rs#L138)). The effective-knob echo is verified, not trusted: exactly two rows must come back, and every knob the request sent must echo the value it sent, or the report errors out ([judge.rs:97](../../../cli/src/trend/judge.rs#L97)). A missing `trend/2` capability or a non-`trend.result` reply is an error, never a silently unjudged report ([judge.rs:45](../../../cli/src/trend/judge.rs#L45)). Report schema id: `ce.trend-report/0.3.0` ([report.rs:17](../../../cli/src/trend/report.rs#L17)).

### Not found in source

Since trend/2 the judgment window IS bounded — `tsWindow = 512` — but the prose should not conflate it with the `4096`-row wire cap: rows past the window still cross, count, and refuse offenders; they are simply outside the judged view. The window's *default breadth* is a separate constant again: `DEFAULT_COMMITS = 30`, shared by every face ([mod.rs:30](../../../cli/src/trend/mod.rs#L30)) — before it existed the clap face carried a `30` literal and the MCP adapter a `10`.
