# The three-signal join

[index](../methodology.md) · [← 06 Graph liveness and dead-code verdicts](06-graph-liveness-and-dead-code-verdicts.md) · [→ 08 Split-ROI seam pricing (four legs)](08-split-roi-seam-pricing-four-legs.md)

Similarity says two entities look alike. Graph position says whether anything points at them. Churn says whether they are being maintained twice. None of the three is a verdict on its own; the join is the deterministic rule that turns a triple of legs into one of four codes — and then declines to act on it.

The computation lives on two roads that share no code:

- **Leg assembly (Rust, report only).** `ce join` aggregates the three legs into file-tier and unit-tier rows and prints them. Its module header states the stance outright: each file pair is "judged by the SAME verdict/1 lattice `ce check` gates with (2.33.0, H4: one judgment, two faces)", and the command is "still report-only at the EXIT: candidates inform, the fail bit never reads them, and nothing here thresholds" ([mod.rs:1-8](../../../cli/src/join/mod.rs#L1)). Output schema <!--ce:report:join#schemaver-->`ce.join-report/0.3.0`<!--/ce-->, whose file rows carry the core's join verdict and whose unit rows carry the R6 caveat as a code rather than a sentence ([mod.rs:30-35](../../../cli/src/join/mod.rs#L30)).
- **The lattice (Haskell, pure).** `CE.Verdict.Join` takes `Knobs -> Legs` and returns `(verdict, legsMask, reasonBits)` ([Join.hs:147-149](../../../core/app/CE/Verdict/Join.hs#L147)). It is reached over the `verdict/1` wire by `ce check`, which builds one `candidates` row per sim row ([Candidates.hs:21-31](../../../core/app/CE/Verdict/Candidates.hs#L21)).

### The join key

At **tier F** the key is the unordered file pair, normalized by lexicographic order before aggregation:

```
key = if a_file <= b_file { (a_file, b_file) } else { (b_file, a_file) }
```

([mod.rs:151-155](../../../cli/src/join/mod.rs#L151)). Clone blocks are folded into that key as `(block count, token sum)` ([mod.rs:156-158](../../../cli/src/join/mod.rs#L156)).

At **tier U** the key is the triple `(path, key, nth)` — the unit identity ([churn_unit.rs:37-43](../../../cli/src/join/churn_unit.rs#L37)). A clone-block side is attributed to the **innermost unit containing the whole span**, chosen by minimum extent:

```
hit = argmin over units with (start_u <= span_start and span_end <= end_u) of (end_u - start_u)
```

([churn_unit.rs:93-97](../../../cli/src/join/churn_unit.rs#L93)). A span no single unit contains is not split between neighbours; it falls to the file top level, `key = ""`, `nth = 0` ([churn_unit.rs:103-107](../../../cli/src/join/churn_unit.rs#L103)) — a refusal to guess, pinned by the test `crossing.key == ""` ([unit/join/churn_unit.rs:20-21](../../../cli/tests/unit/join/churn_unit.rs#L20)).

On the wire the key is a pair of **dense file indices** `u < v` in the same index space the graph judgment uses ([wire.rs:19-23](../../../cli/src/score/wire.rs#L19)). The core rejects a non-ascending or out-of-range pair rather than reordering it (`"pair not ascending"`, `"endpoint out of range"` — [Rows.hs:38-39](../../../core/app/CE/Verdict/Rows.hs#L38)).

### Leg 1 — similarity

The leg travels as `(simKind, num, den)`, where kind `0 = t1t2`, `1 = t3`, `2 = docdup` ([Join.hs:57-59](../../../core/app/CE/Verdict/Join.hs#L57)). It is judged against the **owning family's** threshold by integer cross-multiplication, never division:

```
kind 2   : num * jaccardDen >= den * jaccardNum      -- 80/100
kind 0,1 : num * tsedDen    >= den * tsedNum         -- 85/100
```

([Join.hs:160-162](../../../core/app/CE/Verdict/Join.hs#L160)), with `tsedNum = 85` [Clone/Cost.hs:22](../../../core/app/CE/Clone/Cost.hs#L22), `tsedDen = 100` [Clone/Cost.hs:25](../../../core/app/CE/Clone/Cost.hs#L25), `jaccardNum = 80` [Docdup/Cost.hs:29-30](../../../core/app/CE/Docdup/Cost.hs#L29), `jaccardDen = 100` [Docdup/Cost.hs:32-33](../../../core/app/CE/Docdup/Cost.hs#L32). Those four constants are *reused* from the clone and docdup families rather than re-declared here — one authority per fact ([Join.hs:80-95](../../../core/app/CE/Verdict/Join.hs#L80), and the same rule restated in [Verdict/Cost.hs:7-10](../../../core/app/CE/Verdict/Cost.hs#L7)). Two wire-level offences protect the comparison: `kind > 2` is `"unknown sim kind"` and `den == 0` is `"zero denominator"` — the latter because `0/0` cross-multiplies to a vacuously certain clone ([Rows.hs:47-53](../../../core/app/CE/Verdict/Rows.hs#L47)).

One measurement caveat: today's only producer of sim rows emits `[u, v, 0, 100, 100]` — kind `t1t2` at ratio `100/100`, because the pair reached the table only by carrying a verified block ([score/mod.rs:269](../../../cli/src/score/mod.rs#L269), fn doc at [score/mod.rs:249-251](../../../cli/src/score/mod.rs#L249)). So on both live roads — `ce check` and, since 2.33.0, `ce join`, which calls the same `score::sim_rows` — the similarity bit holds for every candidate row by construction; kinds `1` and `2` are exercised only by the lattice's own battery ([JoinProps.hs:49-50](../../../core/test/JoinProps.hs#L49), [JoinProps.hs:65-66](../../../core/test/JoinProps.hs#L65)).

### Leg 2 — graph position

Each side's position is `Pos { pIndeg, pReach, pFlags, pScc }` ([Join.hs:43-48](../../../core/app/CE/Verdict/Join.hs#L43)), decoded from the graph reply's `pos` rows `[indeg, outdeg, sccId, sccSize, reachIn]` ([mod.rs:37-41](../../../cli/src/join/mod.rs#L37), [mod.rs:129-143](../../../cli/src/join/mod.rs#L129)). Both sides are `Maybe`, and the pair is taken applicatively — either side missing kills the leg ([Join.hs:163-166](../../../core/app/CE/Verdict/Join.hs#L163)). Three predicates read it:

```
bothRef     = indeg a >= 1 && indeg b >= 1
sccDistinct = scc a /= scc b
deadV x y   = indeg x == 0 && reach x == 0 && (flags x .&. entryMask) == 0 && indeg y >= 1
deadFlank   = deadV a b || deadV b a
publicGuard = (deadV x y) && testBit (flags x) 0, for either orientation
```

([Join.hs:167-174](../../../core/app/CE/Verdict/Join.hs#L167)), with `entryMask = 126` reused from the graph family ([Graph/Cost.hs:95-96](../../../core/app/CE/Graph/Cost.hs#L95)) — bits 1..6 (main, test, entry-glob, dyn-referenced, doc-entry, `ce:allow(deadcode)`), bit 0 (exported) deliberately excluded ([Graph/Cost.hs:85-94](../../../core/app/CE/Graph/Cost.hs#L85)).

Note that "partner still alive" (`indeg y >= 1`) is inside the definition of a dead flank, so at most one side of a pair can be dead. Bit 0 of `flags` is exported-ness, and it is only ever a *guard* (RG10), never an argument for a verdict ([Verdict/Cost.hs:20-23](../../../core/app/CE/Verdict/Cost.hs#L20)). `pFlags` carries the export axis and nothing else: entry-ness rides `reachIn` (an entry seeds the reach set, so it is never a dead flank), which is why the `pos` row has no flags column and needs none ([Join.hs:38-42](../../../core/app/CE/Verdict/Join.hs#L38)). Exported-ness has had a producer since 4.1.0 — the graph request's `symbols` table ORs flag bit 0 in — but until 6.1.0 that bit reached the graph face ALONE, so this lattice synthesized `0` and `publicGuard` was inert in production while `delete` could be proposed for an exported flank. `verdict/1` now carries the same table re-keyed to the tier universe, and the guard reads the bit the graph family's own `exportVisBit` decides ([Candidates.hs:29-42](../../../core/app/CE/Verdict/Candidates.hs#L29), [symwire.rs:76-86](../../../cli/src/graph/symwire.rs#L76)). It guards the flank being proposed for deletion and only that one: exporting the LIVE partner changes nothing, which is what separates a firewall from a mute ([VerdictWireProps.hs:117-136](../../../core/test/VerdictWireProps.hs#L117)).

At **tier U** this leg is `null` by design, not by omission: import-granularity edges give units a constant indegree of 0, so any number would be fabricated. A caveat CODE rides every unit row instead — `GRAPH_NULL_IMPORT_GRANULARITY`, naming R6 (an independent 100-callsite audit at ≥ 0.90) as the unlock condition ([churn_unit.rs:35](../../../cli/src/join/churn_unit.rs#L35), emitted at [report.rs:28](../../../cli/src/join/report.rs#L28)). It was an English sentence until plan v2.15: prose on the machine face is prose no lookup switch can reach, so the console rendered the same fact from its own bilingual template while the GUI showed 200 characters of English.

### Leg 3 — churn

Per side the leg is `(appended, rewrote)` line counts over the window ([Join.hs:62-63](../../../core/app/CE/Verdict/Join.hs#L62), [churn_unit.rs:46-50](../../../cli/src/join/churn_unit.rs#L46)). At tier F they are summed from the per-unit ledger so the report totals and the join legs come from one bookkeeping ([mod.rs:184-194](../../../cli/src/join/mod.rs#L184)); the wire row is `[u, rewrote, appended]` — three columns since proto 3.0.0, when the constant fourth (`rewrote + appended`) and the never-measured fifth (`survived`, always 0) were cut ([score/mod.rs:349-352](../../../cli/src/score/mod.rs#L349)), decoded back as `(appended, rewrote)` at [Candidates.hs:51](../../../core/app/CE/Verdict/Candidates.hs#L51). A pair's co-change count is a separate table, `[u, v, count]` ([score/mod.rs:362-368](../../../cli/src/score/mod.rs#L362)).

```
total      = appended_a + rewrote_a + appended_b + rewrote_b
rewriteHot = total > 0 && (rewrote_a + rewrote_b) * rewriteDen >= total * rewriteNum   -- >= 50%
cochangeHot = cochange >= cochangeFloor                                                 -- >= 2
```

([Join.hs:175-179](../../../core/app/CE/Verdict/Join.hs#L175)), with `rewriteNum = 50` [Verdict/Cost.hs:102-103](../../../core/app/CE/Verdict/Cost.hs#L102), `rewriteDen = 100` [Verdict/Cost.hs:105-106](../../../core/app/CE/Verdict/Cost.hs#L105), `cochangeFloor = 2` [Verdict/Cost.hs:94-95](../../../core/app/CE/Verdict/Cost.hs#L94). Both are configurable per request: `rewriteNum`/`rewriteDen` are thresholds codes 1/2 and `cochangeFloor` is code 3, all echoed back in the effective-knob table ([Knobs.hs:72-78](../../../core/app/CE/Verdict/Knobs.hs#L72), [Knobs.hs:51-52](../../../core/app/CE/Verdict/Knobs.hs#L51), [Knobs.hs:99-101](../../../core/app/CE/Verdict/Knobs.hs#L99)).

`cochangeFloor = 2` is not an independent choice — it is the churn table's own admission floor, and since batch-7 slice 12 the Rust side follows the CONFIGURED `cochange_floor` when one is set (the hardcoded `>= 2` used to withhold count-1 pairs from a core configured to judge them) and ships the table WHOLE — the rank cut `truncate(20)` is gone (it ran before the judge and before the relevance filter, spending most of the evidence budget on rows the score path discarded; measured on this repository at the time of the change: 20 kept of 1020, only 5 of the 20 in the judged language set — a run-time observation, not a constant, and nothing in the source records it) ([churn/mod.rs:88-98](../../../cli/src/churn/mod.rs#L88) for the configured floor, [churn/mod.rs:259-277](../../../cli/src/churn/mod.rs#L259) for the whole table); the console keeps a 20-row display cut with the remainder counted out loud. The numerically-coincident `COCHANGE_FILE_CAP = 20` ([report.rs:14](../../../cli/src/churn/report.rs#L14)) is a different guard, skipping pair-counting for commits that touch more files than it, so the lattice can never claim heat the report would not even list ([Verdict/Cost.hs:92-95](../../../core/app/CE/Verdict/Cost.hs#L92)). Correspondingly `cochange` is `Option`: `None` means the pair sits below that floor — unknown-small, never a fabricated zero ([mod.rs:54-56](../../../cli/src/join/mod.rs#L54), [Join.hs:53-55](../../../core/app/CE/Verdict/Join.hs#L53)), and `maybe False (>= floor)` makes an unknown never fire ([Join.hs:179](../../../core/app/CE/Verdict/Join.hs#L179)). Churn *zeros*, by contrast, are real zeros: an absent ledger row means the unit genuinely saw no window edits ([churn_unit.rs:124-126](../../../cli/src/join/churn_unit.rs#L124), default at [churn_unit.rs:139-144](../../../cli/src/join/churn_unit.rs#L139)).

### The verdict table

Priority is data, not guard order — an ordered list of `(code, severity, requiredBits, forbiddenBits)`; the first row whose required bits all hold and whose forbidden bits all stay clear wins, else `0`:

```
(1, sev 2, [1,2,3,4], [])    -- merge_candidate:  sim + graph + both referenced + distinct SCCs
(2, sev 3, [1,2,5],   [6])   -- delete_candidate: sim + graph + dead flank, RG10 guard clear
(3, sev 1, [1,2,7,8], [])    -- churn_hotspot:    sim + graph + cochange + rewrite
```

([Join.hs:120-125](../../../core/app/CE/Verdict/Join.hs#L120)); codes are `0 report_only / 1 merge_candidate / 2 delete_candidate / 3 churn_hotspot` ([Join.hs:10-13](../../../core/app/CE/Verdict/Join.hs#L10)). Selection is the literal first match:

```haskell
code = case [c | (c, _, req, forb) <- table, all (testBit reasons) req, not (any (testBit reasons) forb)] of
  (c : _) -> c
  []      -> 0
```

([Join.hs:180-182](../../../core/app/CE/Verdict/Join.hs#L180)). Making the order data is what lets the battery falsify it: the `reorder` probe judges a crafted row with a rotated table and requires the answer to flip from merge to `3` ([JoinProps.hs:24](../../../core/test/JoinProps.hs#L24), [JoinProps.hs:201-206](../../../core/test/JoinProps.hs#L201)).

**Reason bits** — the ledger of which conditions held, shipped alongside the code so a two-leg firing cannot hide ([Join.hs:184-198](../../../core/app/CE/Verdict/Join.hs#L184)):

| bit | name | source |
|---|---|---|
| 0 | *deliberately unused* — exported-ness never argues *for* a verdict | [Verdict/Cost.hs:20-23](../../../core/app/CE/Verdict/Cost.hs#L20) |
| 1 | `simOver` | [Join.hs:188](../../../core/app/CE/Verdict/Join.hs#L188) |
| 2 | `graphBoth` | [Join.hs:189](../../../core/app/CE/Verdict/Join.hs#L189) |
| 3 | `bothRef` | [Join.hs:190](../../../core/app/CE/Verdict/Join.hs#L190) |
| 4 | `sccDistinct` | [Join.hs:191](../../../core/app/CE/Verdict/Join.hs#L191) |
| 5 | `deadFlank` | [Join.hs:192](../../../core/app/CE/Verdict/Join.hs#L192) |
| 6 | `publicGuard` | [Join.hs:193](../../../core/app/CE/Verdict/Join.hs#L193) |
| 7 | `cochangeHot` | [Join.hs:194](../../../core/app/CE/Verdict/Join.hs#L194) |
| 8 | `rewriteHot` | [Join.hs:195](../../../core/app/CE/Verdict/Join.hs#L195) |

Bit 0 is asserted silent by the battery (`"reason bit 0 never fires (deliberately absent)"` — [JoinProps.hs:22](../../../core/test/JoinProps.hs#L22)); RG10 stays inside the delete *condition* as a forbidden bit rather than as a post-filter ([Join.hs:110-114](../../../core/app/CE/Verdict/Join.hs#L110)), with a counterfactual probe flipping only the dead flank's exported bit ([JoinProps.hs:18](../../../core/test/JoinProps.hs#L18)).

**legsMask** records which signals were actually present — `legSim = 1`, `legGraph = 2`, `legChurn = 4` ([Join.hs:97-101](../../../core/app/CE/Verdict/Join.hs#L97)):

```
legsMask = legSim .|. (if graphBoth then legGraph else 0) .|. legChurn
```

([Join.hs:183](../../../core/app/CE/Verdict/Join.hs#L183)) — i.e. `7` when both graph rows answered, `5` when they did not. Because every gating row requires bit 2, a mask of `5` can only carry code `0`: a missing graph leg refuses to gate rather than pretending indegree 0 ([Join.hs:13-16](../../../core/app/CE/Verdict/Join.hs#L13)), asserted as `"legsMask honest: gated => 3 legs; graph-absent never gates"` ([JoinProps.hs:19](../../../core/test/JoinProps.hs#L19)).

### The report-only stance

The join produces *candidates*, and nothing in the pipeline converts a candidate into a failure.

- Each candidate is the 6-tuple `[u, v, code, reasonBits, legsMask, confidence]` (2.33.0), one per sim row ([Candidates.hs:27-28](../../../core/app/CE/Verdict/Candidates.hs#L27)), typed Rust-side as `Vec<[i64; 6]>` ([wire.rs:116](../../../cli/src/score/wire.rs#L116)). The **confidence** is the leg-agreement count — of the legs present, how many contributed at least one held condition, judged through the attribution table `legBits` (sim = bit 1, graph = bits 2..6, churn = bits 7..8; [Join.hs:136](../../../core/app/CE/Verdict/Join.hs#L136), [Join.hs:142](../../../core/app/CE/Verdict/Join.hs#L142)) and pinned by the two-leg/three-leg probes ([JoinProps.hs:192](../../../core/test/JoinProps.hs#L192)). The **severity** column of the verdict table (delete 3 > merge 2 > hotspot 1 — data the battery pins beside the permutable table, [JoinProps.hs:186](../../../core/test/JoinProps.hs#L186)) ships once per reply as `joinSeverity` ([Verdict.hs:104](../../../core/app/CE/Verdict.hs#L104)); the report ranks with the core's numbers, never its own.
- The fail bit is a disjunction over six *named* conditions — `ratchet_over`, `discrete_added`, `floor`, `dedup_budget`, `knobs_digest`, `rows_dropped` (6.4.0) ([Faces.hs:23-31](../../../core/app/CE/Verdict/Faces.hs#L23), folded at [Verdict.hs:170](../../../core/app/CE/Verdict.hs#L170) and disjoined at [Faces.hs:46](../../../core/app/CE/Verdict/Faces.hs#L46)). No verdict code appears in that list.
- `ce check` consequently prints only the candidate *count* on the console ([report.rs:85-89](../../../cli/src/score/report.rs#L85)) and passes the rows through verbatim in JSON ([report.rs:55](../../../cli/src/score/report.rs#L55)).
- Since 2.33.0 `ce join` judges its pairs over the SAME verdict/1 road the check gate uses — one judgment, two faces: its own single measurement builds the request (score-side tables it has no stake in ride empty), and each file row renders the core's verdict, severity and confidence ([verdicts.rs:39](../../../cli/src/join/verdicts.rs#L39), consumed at [mod.rs:106-114](../../../cli/src/join/mod.rs#L106)). The EXIT stays report-only: the command runs through `family_cmd`'s no-veto closure and always exits `SUCCESS`, and the summary line says which half is which: `"verdicts by the check lattice; exit stays report-only"` ([report.rs:114](../../../cli/src/join/report.rs#L114)).
- Degradation is visible, not silent — and it is the reply's own `degraded` boolean that says so, with `reason` carried as its text only when that bit is set ([mod.rs:94-97](../../../cli/src/join/mod.rs#L94)); it prints as `"join graph leg degraded: {}"` ([report.rs:105](../../../cli/src/join/report.rs#L105)). On the scoring road a degraded graph reply is refused outright rather than scored on an empty `pos` table ([score/mod.rs:213-229](../../../cli/src/score/mod.rs#L213)).

**Not found in source.** The `Join.hs` header refers to a "3h token-count floor" as the pre-wire approximation ([Join.hs:6-8](../../../core/app/CE/Verdict/Join.hs#L6)); no such constant exists in `Join.hs`, `Verdict/Cost.hs`, or `cli/src/join/` as read this run — the similarity leg is judged solely by the cross-multiplied family ratio above. Likewise the `blocks` and `tokens` fields on a Tier F row ([mod.rs:48-49](../../../cli/src/join/mod.rs#L48)) are reported but never thresholded HERE: `cli/src/join/mod.rs` declares no constant other than `SCHEMA_ID`, `cli/src/join/churn_unit.rs` none other than `GRAPH_NULL_IMPORT_GRANULARITY`, and `cli/src/join/verdicts.rs` — the third file, added at 2.33.0 — none other than the rendering table `VERDICT_NAMES`. "Here" is load-bearing: every block that becomes a Tier F row already cleared dedup's own floor upstream, `min_tokens = Params::guarantee() = window + kgram - 1 = 50` ([mod.rs:154](../../../cli/src/dedup/mod.rs#L154), applied at [probe.rs:113](../../../cli/src/dedup/probe.rs#L113)), and the join neither re-applies nor relaxes it. The clone family's other admission floor is not a token floor at all — `minUnitNodes = 24` counts AST nodes, on the ground that below it a "clone" is a signature rather than an implementation ([Clone/Cost.hs:38-39](../../../core/app/CE/Clone/Cost.hs#L38)).
