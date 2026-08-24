# Score trajectory — the trend slope verdict

[index](../methodology.md) · [← 09 Edit four-classification (update supervision)](09-edit-four-classification-update-supervision.md) · [→ 11 FPR discipline and the guard tier ladder](11-fpr-discipline-and-the-guard-tier-ladder.md)

The eighth judgment family answers one question about a repository's history: is the check score going up, flat, or down, and by how much per day. Rust measures the trajectory; Haskell judges it. Rust computes no policy — sign, floor, and the fail bit all come back on the wire ([judge.rs:3-5](../../../cli/src/trend/judge.rs#L3)).

### Measurement — what becomes a row

The window is the newest `n` **first-parent** commits of `HEAD`, taken as `(full sha, author time)` from `git log --first-parent -n <n> --format=%H %ct` ([mod.rs:128-145](../../../cli/src/trend/mod.rs#L129)). Each uncached commit is scored in a detached temp worktree with `establish: true` plus a `pinned_soft` baseline — an EMPTY ratchet (absolute score, no ratchet noise) judged against the committed soft line rather than one re-derived per point — with `days: None` (no churn window) and the historical tree's own `ce.toml` knobs ([mod.rs:150-162](../../../cli/src/trend/mod.rs#L151), the soft pin from [mod.rs:100](../../../cli/src/trend/mod.rs#L100)). A degraded score reply is a measurement failure, not a row ([mod.rs:164-166](../../../cli/src/trend/mod.rs#L165)). The scale stored is the *effective* `scoreScale` from that commit's reply, falling back to `1000` only when the key is absent ([mod.rs:172](../../../cli/src/trend/mod.rs#L173)).

Rows are cached in `.ce/index.db` keyed by `commit_hash` and stamped with the measuring toolchain ([mod.rs:41-57](../../../cli/src/trend/mod.rs#L41)); reads filter on the current stamp ([mod.rs:226-228](../../../cli/src/trend/mod.rs#L226)) and writes are `INSERT OR REPLACE` — a row left by another toolchain is exactly the one this run just re-measured ([mod.rs:259-277](../../../cli/src/trend/mod.rs#L260)). They are reversed to oldest-first for chart order before judgment ([mod.rs:115-117](../../../cli/src/trend/mod.rs#L115)) — a presentation choice only; the math below is order-free.

Only `[ts, score, scale]` triples cross the wire — no commit hashes, no paths ([judge.rs:42](../../../cli/src/trend/judge.rs#L42), [Trend.hs:12-14](../../../core/app/CE/Trend.hs#L12)).

### Boundary contract

Before any arithmetic, the first offender in request order is named and the request refused ([Trend.hs:55-61](../../../core/app/CE/Trend.hs#L55)):

- row must be exactly `[ts, score, scale]`, with `ts >= 0`, `scale > 0`, and `0 <= score <= scale` ([Trend.hs:64-70](../../../core/app/CE/Trend.hs#L64));
- knob must be `[code, value]` with `code ∈ {0, 1}`, `value >= 0`, and — for `code == 0` — `value >= 2` ([Trend.hs:74-81](../../../core/app/CE/Trend.hs#L74));
- knob codes must be strictly ascending ([Trend.hs:60](../../../core/app/CE/Trend.hs#L60), [Wire.hs:54-61](../../../core/app/CE/Wire.hs#L54)).

Row **order is deliberately unconstrained**: least squares is order-free, and first-parent order is topological rather than chronological, so rebased or backdated commits are legal input ([Trend.hs:48-54](../../../core/app/CE/Trend.hs#L48)). The property `orderFree` pins that a shuffled window judges identically to its sorted twin ([TrendProps.hs:126-136](../../../core/test/TrendProps.hs#L126)).

Cap: `length rows + length knobs > 4096` produces a complete **degraded** reply rather than a truncated one ([Trend.hs:41-42](../../../core/app/CE/Trend.hs#L41), [Cost.hs:28-29](../../../core/app/CE/Trend/Cost.hs#L28)). One row per mainline commit; 4096 covers roughly a decade of daily commits ([Cost.hs:25-27](../../../core/app/CE/Trend/Cost.hs#L25)).

### The slope — exact Rational least squares

No floating point ever crosses a verdict ([Cost.hs:1-4](../../../core/app/CE/Trend/Cost.hs#L1)). Each row maps to an exact `Rational` point:

```
x_i = ts_i % 86400              -- seconds → days, exact ratio
y_i = (score_i * 1000000) % scale_i
```

([Cost.hs:47-48](../../../core/app/CE/Trend/Cost.hs#L47)). `y` renormalizes every commit onto a fixed `10^6` full-scale grid, so rows measured under different `scoreScale` values are commensurable. With the usual `scoreScale = 1000`, one per-mille point of score equals 1000 y-units — which is why a decline of 1‰ per day reads as slope `-1000` ([TrendProps.hs:57-59](../../../core/test/TrendProps.hs#L57)).

The slope is the ordinary least-squares product form:

```
slope = (n * Σ(x_i*y_i) − Σx_i * Σy_i) / (n * Σ(x_i²) − (Σx_i)²)
```

([Cost.hs:43](../../../core/app/CE/Trend/Cost.hs#L43), with `sx`, `sy`, `sxy`, `sxx`, `den` at [Cost.hs:49-53](../../../core/app/CE/Trend/Cost.hs#L49)). Units: **micro-per-mille per day**.

It is `Nothing` — underdetermined, not zero — when `n < 2` or `den == 0` ([Cost.hs:42](../../../core/app/CE/Trend/Cost.hs#L42)). `den == 0` is exactly zero timestamp variance: same-second commits are a legal, common shape (rebases, scripted pushes), so ties ride the wire and only the *all-tied* window has no slope to state ([Cost.hs:33-37](../../../core/app/CE/Trend/Cost.hs#L33)).

`TrendProps` checks the product form against an independent centered-moments derivation `slope = Σ(x−x̄)(y−ȳ) / Σ(x−x̄)²` over an enumerated grid of lengths 2..5 across the score alphabet `[0, 250, 500, 1000]` — exact `Rational`, so equality is equality, not tolerance ([TrendProps.hs:31-52](../../../core/test/TrendProps.hs#L31)).

### minPoints — absence, never a fabricated flat

`minPoints` is knob code `0`, default `3` ([Cost.hs:23](../../../core/app/CE/Trend/Cost.hs#L23), resolved at [Trend.hs:94](../../../core/app/CE/Trend.hs#L94) via last-match-wins `pick` at [Wire.hs:77-78](../../../core/app/CE/Wire.hs#L77)). Below it the slope is not computed at all:

```haskell
slope
  | toInteger (length rows) < minPoints = Nothing
  | otherwise = slopeMicroPerDay rows
verdict = verdictOf floorMicro <$> slope
```

([Trend.hs:97-100](../../../core/app/CE/Trend.hs#L97)). Both `slopeMicroPerDay` and `verdict` serialize as JSON `null`, and the fail bit stays `false` — nothing was judged, and an unjudged trend must not gate ([Trend.hs:85-88](../../../core/app/CE/Trend.hs#L85)). This is the one case distinguished from a degraded reply, where judgment was *denied* by the cap and `fail = true` says so. The `absence` property pins both doors to this posture — below minPoints, and the all-tied window ([TrendProps.hs:94-107](../../../core/test/TrendProps.hs#L94)).

Validation forbids `minPoints < 2` ([Trend.hs:79](../../../core/app/CE/Trend.hs#L79)), so the knob can never demand a slope from a single point.

### Verdict codes and the decline floor

`declineFloorMicro` is knob code `1`, default `0` ([Cost.hs:23](../../../core/app/CE/Trend/Cost.hs#L23), [Trend.hs:95](../../../core/app/CE/Trend.hs#L95)). It defines a symmetric dead band around zero:

```
slope < −band  → 2  (degrading)
slope >  band  → 0  (improving)
otherwise      → 1  (flat)        where band = floorMicro
```

([Cost.hs:61-67](../../../core/app/CE/Trend/Cost.hs#L61)). The band is **inclusive**: at slope `-1000`, floor `999` says degrading but floor `1000` says flat ([TrendProps.hs:62](../../../core/test/TrendProps.hs#L62)). With the default floor `0` the band collapses to the single point zero, so the raw sign is the report ([Cost.hs:57-59](../../../core/app/CE/Trend/Cost.hs#L57)). Console words for the three codes plus the unjudged case are rendering only ([judge.rs:92-100](../../../cli/src/trend/judge.rs#L92)).

**Fail arming** is a separate decision made at the reply, not in the classifier:

```haskell
"fail" .= (jDegraded j || (jVerdict j == Just 2 && lookup 1 (jEffective j) > Just 0))
```

([Trend.hs:118](../../../core/app/CE/Trend.hs#L118)). Two conjuncts on the judged path: the verdict must be `2` (degrading) **and** a floor must have been declared strictly greater than `0`. Floor `0` — the default every family launches in — is a report-only posture: it can report degrading and cannot fail ([Cost.hs:7-10](../../../core/app/CE/Trend/Cost.hs#L7)). The `floorLever` property runs the same falling-1‰/day rows through the real `respond` three times: floor absent → verdict `2`, fail `false`; floor `500` → verdict `2`, fail `true`; floor `5000` → verdict `1`, fail `false` ([TrendProps.hs:76-92](../../../core/test/TrendProps.hs#L76)).

The degraded path fails unconditionally via `jDegraded`, and echoes the **default** knob table rather than the request's unvalidated override ([Trend.hs:102-107](../../../core/app/CE/Trend.hs#L102)), with `reason = "trend_too_large"` ([Trend.hs:123](../../../core/app/CE/Trend.hs#L123)).

### Serialization and the round-trip pin

The slope is judged exactly and only *displayed* rounded: `fmap (round :: Rational -> Integer)` ([Trend.hs:116](../../../core/app/CE/Trend.hs#L116)) — Haskell's `round` is round-half-even. The verdict compared the exact value and no client re-derives it ([Trend.hs:120-122](../../../core/app/CE/Trend.hs#L120)).

On the Rust side, knobs ride the wire only when `ce.toml` declares them — `[trend] min_points` → code `0`, `decline_floor_micro` → code `1`, both `Option` ([judge.rs:30-36](../../../cli/src/trend/judge.rs#L30), [config.rs:139-144](../../../cli/src/config.rs#L139)). Absent means the core's own defaults apply ([config.rs:134-138](../../../cli/src/config.rs#L134)). The effective-knob echo is verified, not trusted: exactly two rows must come back, and every knob the request sent must echo the value it sent, or the report errors out ([judge.rs:64-78](../../../cli/src/trend/judge.rs#L64)). A missing `trend/1` capability or a non-`trend.result` reply is an error, never a silently unjudged report ([judge.rs:44-50](../../../cli/src/trend/judge.rs#L44)). Report schema id: `ce.trend-report/0.2.0` ([mod.rs:30](../../../cli/src/trend/mod.rs#L30)).

### Not found in source

The prose should not claim a maximum window size for `ce trend` beyond the `4096`-row wire cap. The window's *default* is a real constant: `DEFAULT_COMMITS = 30`, shared by every face ([mod.rs:32-37](../../../cli/src/trend/mod.rs#L32)) — before it existed the clap face carried a `30` literal and the MCP adapter a `10`.
