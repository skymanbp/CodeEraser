# The three-signal join

[index](../methodology.md) · [← 06 Graph liveness and dead-code verdicts](06-graph-liveness-and-dead-code-verdicts.md) · [→ 08 Split-ROI seam pricing (four legs)](08-split-roi-seam-pricing-four-legs.md)

Similarity says two entities look alike. Graph position says whether anything points at them. Churn says whether they are being maintained twice. None of the three is a verdict on its own; the join is the deterministic rule that turns a triple of legs into one of four codes — and then declines to act on it.

The computation lives on two roads that share no code:

- **Leg assembly (Rust, report only).** `ce join` aggregates the three legs into file-tier and unit-tier rows and prints them. Its module header states the stance outright: "REPORT-ONLY: the verdict lattice ... judges these same legs on the verdict/1 wire via `ce check` (M5-3i); nothing here thresholds or gates" ([mod.rs:6-7](../../../cli/src/join/mod.rs#L6)). Output schema `ce.join-report/0.1.0` ([mod.rs:26](../../../cli/src/join/mod.rs#L26)).
- **The lattice (Haskell, pure).** `CE.Verdict.Join` takes `Knobs -> Legs` and returns `(verdict, legsMask, reasonBits)` ([Join.hs:121-128](../../../core/app/CE/Verdict/Join.hs#L121)). It is reached over the `verdict/1` wire by `ce check`, which builds one `candidates` row per sim row ([Verdict.hs:161-166](../../../core/app/CE/Verdict.hs#L161)).

### The join key

At **tier F** the key is the unordered file pair, normalized by lexicographic order before aggregation:

```
key = if a_file <= b_file { (a_file, b_file) } else { (b_file, a_file) }
```

([mod.rs:104-109](../../../cli/src/join/mod.rs#L104)). Clone blocks are folded into that key as `(block count, token sum)` ([mod.rs:110-113](../../../cli/src/join/mod.rs#L110)).

At **tier U** the key is the triple `(path, key, nth)` — the unit identity ([churn_unit.rs:24-30](../../../cli/src/join/churn_unit.rs#L24)). A clone-block side is attributed to the **innermost unit containing the whole span**, chosen by minimum extent:

```
hit = argmin over units with (start_u <= span_start and span_end <= end_u) of (end_u - start_u)
```

([churn_unit.rs:79-83](../../../cli/src/join/churn_unit.rs#L79)). A span no single unit contains is not split between neighbours; it falls to the file top level, `key = ""`, `nth = 0` ([churn_unit.rs:89-93](../../../cli/src/join/churn_unit.rs#L89)) — a refusal to guess, pinned by the test `crossing.key == ""` ([churn_unit.rs:169-170](../../../cli/src/join/churn_unit.rs#L169)).

On the wire the key is a pair of **dense file indices** `u < v` in the same index space the graph judgment uses ([wire.rs:19-23](../../../cli/src/score/wire.rs#L19)). The core rejects a non-ascending or out-of-range pair rather than reordering it (`"pair not ascending"`, `"endpoint out of range"` — [Wire.hs:143-149](../../../core/app/CE/Verdict/Wire.hs#L143)).

### Leg 1 — similarity

The leg travels as `(simKind, num, den)`, where kind `0 = t1t2`, `1 = t3`, `2 = docdup` ([Join.hs:59-61](../../../core/app/CE/Verdict/Join.hs#L59)). It is judged against the **owning family's** threshold by integer cross-multiplication, never division:

```
kind 2   : num * jaccardDen >= den * jaccardNum      -- 80/100
kind 0,1 : num * tsedDen    >= den * tsedNum         -- 85/100
```

([Join.hs:133-135](../../../core/app/CE/Verdict/Join.hs#L133)), with `tsedNum = 85` [Clone/Cost.hs:22](../../../core/app/CE/Clone/Cost.hs#L22), `tsedDen = 100` [Clone/Cost.hs:25](../../../core/app/CE/Clone/Cost.hs#L25), `jaccardNum = 80` [Docdup/Cost.hs:27](../../../core/app/CE/Docdup/Cost.hs#L27), `jaccardDen = 100` [Docdup/Cost.hs:30](../../../core/app/CE/Docdup/Cost.hs#L30). Those four constants are *reused* from the clone and docdup families rather than re-declared here — one authority per fact ([Join.hs:78-93](../../../core/app/CE/Verdict/Join.hs#L78), and the same rule restated in [Verdict/Cost.hs:7-10](../../../core/app/CE/Verdict/Cost.hs#L7)). Two wire-level offences protect the comparison: `kind > 2` is `"unknown sim kind"` and `den == 0` is `"zero denominator"` — the latter because `0/0` cross-multiplies to a vacuously certain clone ([Wire.hs:152-163](../../../core/app/CE/Verdict/Wire.hs#L152)).

One measurement caveat: today's only producer of sim rows emits `[u, v, 0, 100, 100]` — kind `t1t2` at ratio `100/100`, because the pair reached the table only by carrying a verified block ([score/mod.rs:214](../../../cli/src/score/mod.rs#L214), fn doc at [score/mod.rs:194-196](../../../cli/src/score/mod.rs#L194)). So on the live `ce check` road the similarity bit holds for every candidate row by construction; kinds `1` and `2` are exercised only by the lattice's own battery ([JoinProps.hs:47-48](../../../core/test/JoinProps.hs#L47), [JoinProps.hs:59-60](../../../core/test/JoinProps.hs#L59)).

### Leg 2 — graph position

Each side's position is `Pos { pIndeg, pReach, pFlags, pScc }` ([Join.hs:41-46](../../../core/app/CE/Verdict/Join.hs#L41)), decoded from the graph reply's `pos` rows `[indeg, outdeg, sccId, sccSize, reachIn]` ([mod.rs:28-32](../../../cli/src/join/mod.rs#L28), [mod.rs:86-95](../../../cli/src/join/mod.rs#L86)). Both sides are `Maybe`, and the pair is taken applicatively — either side missing kills the leg ([Join.hs:136-139](../../../core/app/CE/Verdict/Join.hs#L136)). Three predicates read it:

```
bothRef     = indeg a >= 1 && indeg b >= 1
sccDistinct = scc a /= scc b
deadV x y   = indeg x == 0 && reach x == 0 && (flags x .&. entryMask) == 0 && indeg y >= 1
deadFlank   = deadV a b || deadV b a
publicGuard = (deadV x y) && testBit (flags x) 0, for either orientation
```

([Join.hs:140-147](../../../core/app/CE/Verdict/Join.hs#L140)), with `entryMask = 126` reused from the graph family ([Graph/Cost.hs:47-48](../../../core/app/CE/Graph/Cost.hs#L47)) — bits 1..6 (main, test, entry-glob, dyn-referenced, doc-entry, `ce:allow(deadcode)`), bit 0 (exported) deliberately excluded ([Graph/Cost.hs:36-46](../../../core/app/CE/Graph/Cost.hs#L36)).

Note that "partner still alive" (`indeg y >= 1`) is inside the definition of a dead flank, so at most one side of a pair can be dead. Bit 0 of `flags` is exported-ness, and it is only ever a *guard* (RG10), never an argument for a verdict ([Verdict/Cost.hs:20-23](../../../core/app/CE/Verdict/Cost.hs#L20)). On the current file-granularity wire the flags field is structurally `0` — entry-ness rides `reachIn` instead, and exported-ness is a symbol fact — so `publicGuard` is dormant in production and live only in the lattice's battery ([Join.hs:36-40](../../../core/app/CE/Verdict/Join.hs#L36), [Verdict.hs:168-175](../../../core/app/CE/Verdict.hs#L168), [Verdict/Cost.hs:39-43](../../../core/app/CE/Verdict/Cost.hs#L39)).

At **tier U** this leg is `null` by design, not by omission: import-granularity edges give units a constant indegree of 0, so any number would be fabricated. `GRAPH_CAVEAT` — naming R6 (an independent 100-callsite audit at ≥ 0.90) as the unlock condition — is printed on every unit row instead ([churn_unit.rs:20-22](../../../cli/src/join/churn_unit.rs#L20), emitted at [mod.rs:157-158](../../../cli/src/join/mod.rs#L157)).

### Leg 3 — churn

Per side the leg is `(appended, rewrote)` line counts over the window ([Join.hs:60-61](../../../core/app/CE/Verdict/Join.hs#L60), [churn_unit.rs:33-37](../../../cli/src/join/churn_unit.rs#L33)). At tier F they are summed from the per-unit ledger so the report totals and the join legs come from one bookkeeping ([mod.rs:137-145](../../../cli/src/join/mod.rs#L137)); the wire row is `[u, rewrote, appended, rewrote + appended, 0]` ([score/mod.rs:274-277](../../../cli/src/score/mod.rs#L276)), decoded back as `(appended, rewrote)` at [Verdict.hs:176](../../../core/app/CE/Verdict.hs#L176). A pair's co-change count is a separate table, `[u, v, count]` ([score/mod.rs:278-282](../../../cli/src/score/mod.rs#L281)).

```
total      = appended_a + rewrote_a + appended_b + rewrote_b
rewriteHot = total > 0 && (rewrote_a + rewrote_b) * rewriteDen >= total * rewriteNum   -- >= 50%
cochangeHot = cochange >= cochangeFloor                                                 -- >= 2
```

([Join.hs:148-152](../../../core/app/CE/Verdict/Join.hs#L148)), with `rewriteNum = 50` [Verdict/Cost.hs:82](../../../core/app/CE/Verdict/Cost.hs#L82), `rewriteDen = 100` [Verdict/Cost.hs:85](../../../core/app/CE/Verdict/Cost.hs#L85), `cochangeFloor = 2` [Verdict/Cost.hs:74](../../../core/app/CE/Verdict/Cost.hs#L74). Both are configurable per request: `rewriteNum`/`rewriteDen` are thresholds codes 1/2 and `cochangeFloor` is code 3, all echoed back in the effective-knob table ([Knobs.hs:69-75](../../../core/app/CE/Verdict/Knobs.hs#L69), [Knobs.hs:51-52](../../../core/app/CE/Verdict/Knobs.hs#L51), [Knobs.hs:95](../../../core/app/CE/Verdict/Knobs.hs#L95)).

`cochangeFloor = 2` is not an independent choice — it is the churn table's own admission floor, and since batch-7 slice 12 the Rust side follows the CONFIGURED `cochange_floor` when one is set (the hardcoded `>= 2` used to withhold count-1 pairs from a core configured to judge them) and ships the table WHOLE — the rank cut `truncate(20)` is gone (it ran before the judge and before the relevance filter, spending most of the evidence budget on rows the score path discarded; measured on this repository: 20 kept of 1020, only 5 of the 20 in the judged language set) ([churn/mod.rs:218-243](../../../cli/src/churn/mod.rs#L218)); the console keeps a 20-row display cut with the remainder counted out loud. The numerically-coincident `COCHANGE_FILE_CAP = 20` ([churn/mod.rs:54](../../../cli/src/churn/mod.rs#L54)) is a different guard, skipping pair-counting for commits that touch more files than it, so the lattice can never claim heat the report would not even list ([Verdict/Cost.hs:70-74](../../../core/app/CE/Verdict/Cost.hs#L70)). Correspondingly `cochange` is `Option`: `None` means the pair sits below that floor — unknown-small, never a fabricated zero ([mod.rs:45-47](../../../cli/src/join/mod.rs#L45), [Join.hs:51-53](../../../core/app/CE/Verdict/Join.hs#L51)), and `maybe False (>= floor)` makes an unknown never fire ([Join.hs:152](../../../core/app/CE/Verdict/Join.hs#L152)). Churn *zeros*, by contrast, are real zeros: an absent ledger row means the unit genuinely saw no window edits ([churn_unit.rs:110-112](../../../cli/src/join/churn_unit.rs#L110), default at [churn_unit.rs:125-130](../../../cli/src/join/churn_unit.rs#L125)).

### The verdict table

Priority is data, not guard order — an ordered list of `(code, requiredBits, forbiddenBits)`; the first row whose required bits all hold and whose forbidden bits all stay clear wins, else `0`:

```
(1, [1,2,3,4], [])    -- merge_candidate:  sim + graph + both referenced + distinct SCCs
(2, [1,2,5],   [6])   -- delete_candidate: sim + graph + dead flank, RG10 guard clear
(3, [1,2,7,8], [])    -- churn_hotspot:    sim + graph + cochange + rewrite
```

([Join.hs:113-118](../../../core/app/CE/Verdict/Join.hs#L113)); codes are `0 report_only / 1 merge_candidate / 2 delete_candidate / 3 churn_hotspot` ([Join.hs:10-13](../../../core/app/CE/Verdict/Join.hs#L10)). Selection is the literal first match:

```haskell
code = case [c | (c, req, forb) <- table, all (testBit reasons) req, not (any (testBit reasons) forb)] of
  (c : _) -> c
  []      -> 0
```

([Join.hs:153-155](../../../core/app/CE/Verdict/Join.hs#L153)). Making the order data is what lets the battery falsify it: the `reorder` probe judges a crafted row with a rotated table and requires the answer to flip from merge to `3` ([JoinProps.hs:24](../../../core/test/JoinProps.hs#L24), [JoinProps.hs:181-182](../../../core/test/JoinProps.hs#L181)).

**Reason bits** — the ledger of which conditions held, shipped alongside the code so a two-leg firing cannot hide ([Join.hs:157-171](../../../core/app/CE/Verdict/Join.hs#L157)):

| bit | name | source |
|---|---|---|
| 0 | *deliberately unused* — exported-ness never argues *for* a verdict | [Verdict/Cost.hs:20-23](../../../core/app/CE/Verdict/Cost.hs#L20) |
| 1 | `simOver` | [Join.hs:161](../../../core/app/CE/Verdict/Join.hs#L161) |
| 2 | `graphBoth` | [Join.hs:162](../../../core/app/CE/Verdict/Join.hs#L162) |
| 3 | `bothRef` | [Join.hs:163](../../../core/app/CE/Verdict/Join.hs#L163) |
| 4 | `sccDistinct` | [Join.hs:164](../../../core/app/CE/Verdict/Join.hs#L164) |
| 5 | `deadFlank` | [Join.hs:165](../../../core/app/CE/Verdict/Join.hs#L165) |
| 6 | `publicGuard` | [Join.hs:166](../../../core/app/CE/Verdict/Join.hs#L166) |
| 7 | `cochangeHot` | [Join.hs:167](../../../core/app/CE/Verdict/Join.hs#L167) |
| 8 | `rewriteHot` | [Join.hs:168](../../../core/app/CE/Verdict/Join.hs#L168) |

Bit 0 is asserted silent by the battery (`"reason bit 0 never fires (deliberately absent)"` — [JoinProps.hs:22](../../../core/test/JoinProps.hs#L22)); RG10 stays inside the delete *condition* as a forbidden bit rather than as a post-filter ([Join.hs:108-112](../../../core/app/CE/Verdict/Join.hs#L108)), with a counterfactual probe flipping only the dead flank's exported bit ([JoinProps.hs:18](../../../core/test/JoinProps.hs#L18)).

**legsMask** records which signals were actually present — `legSim = 1`, `legGraph = 2`, `legChurn = 4` ([Join.hs:96-99](../../../core/app/CE/Verdict/Join.hs#L96)):

```
legsMask = legSim .|. (if graphBoth then legGraph else 0) .|. legChurn
```

([Join.hs:156](../../../core/app/CE/Verdict/Join.hs#L156)) — i.e. `7` when both graph rows answered, `5` when they did not. Because every gating row requires bit 2, a mask of `5` can only carry code `0`: a missing graph leg refuses to gate rather than pretending indegree 0 ([Join.hs:13-16](../../../core/app/CE/Verdict/Join.hs#L13)), asserted as `"legsMask honest: gated => 3 legs; graph-absent never gates"` ([JoinProps.hs:19](../../../core/test/JoinProps.hs#L19)).

### The report-only stance

The join produces *candidates*, and nothing in the pipeline converts a candidate into a failure.

- Each candidate is the 5-tuple `[u, v, code, reasonBits, legsMask]`, one per sim row ([Verdict.hs:161-166](../../../core/app/CE/Verdict.hs#L161)), carried on the reply at [Verdict.hs:86](../../../core/app/CE/Verdict.hs#L86) and typed Rust-side as `Vec<[i64; 5]>` ([wire.rs:54](../../../cli/src/score/wire.rs#L54)).
- The fail bit is a disjunction over four *named* conditions — `ratchet_over`, `discrete_added`, `floor`, `dedup_budget` ([Verdict.hs:149-155](../../../core/app/CE/Verdict.hs#L149), folded at [Verdict.hs:141-142](../../../core/app/CE/Verdict.hs#L141)). No verdict code appears in that list.
- `ce check` consequently prints only the candidate *count* on the console ([report.rs:76-79](../../../cli/src/score/report.rs#L76)) and passes the rows through verbatim in JSON ([report.rs:50](../../../cli/src/score/report.rs#L50)).
- `ce join` never calls the lattice at all — it assembles legs and prints them ([mod.rs:59-74](../../../cli/src/join/mod.rs#L59)) — and its command runs through `family_cmd`, which passes the no-veto `|_| None` closure into `emit_checked` and therefore always exits `SUCCESS` ([main_judge.rs:175-182](../../../cli/src/main_judge.rs#L175), [main_judge.rs:105-110](../../../cli/src/main_judge.rs#L105), [main_judge.rs:203-214](../../../cli/src/main_judge.rs#L203)). Its console tail says so in the summary line: `"report-only; the verdict lattice judges via ce check"` ([mod.rs:231](../../../cli/src/join/mod.rs#L231)).
- Degradation is visible, not silent — and it is the reply's own `degraded` boolean that says so, with `reason` carried as its text only when that bit is set ([mod.rs:68-69](../../../cli/src/join/mod.rs#L68)); it prints as `"join graph leg degraded: {}"` ([mod.rs:222](../../../cli/src/join/mod.rs#L222)). On the scoring road a degraded graph reply is refused outright rather than scored on an empty `pos` table ([score/mod.rs:168-180](../../../cli/src/score/mod.rs#L168)).

**Not found in source.** The `Join.hs` header refers to a "3h token-count floor" as the pre-wire approximation ([Join.hs:6-8](../../../core/app/CE/Verdict/Join.hs#L6)); no such constant exists in `Join.hs`, `Verdict/Cost.hs`, or `cli/src/join/` as read this run — the similarity leg is judged solely by the cross-multiplied family ratio above. Likewise the `blocks` and `tokens` fields on a Tier F row ([mod.rs:39-40](../../../cli/src/join/mod.rs#L39)) are reported but never thresholded: `cli/src/join/mod.rs` declares no constant other than `SCHEMA_ID`, and `cli/src/join/churn_unit.rs` none other than `GRAPH_CAVEAT`.
