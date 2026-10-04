# T3 near-miss clones — Tree Edit Distance (TSED)

[index](../methodology.md) · [← 01 T1/T2 clone detection — winnowing fingerprint index](01-t1-t2-clone-detection-winnowing-fingerprint.md) · [→ 03 Documentation duplication — shingling + MinHash/LSH](03-documentation-duplication-shingling-minhash.md)

T1/T2 clone detection reports *exact* and *parameterized* duplicate token runs. T3 covers the near-miss case: two units whose ASTs are structurally almost the same but whose token streams are not. The judgment is an exact tree edit distance under a fixed normalization, computed in the Haskell core, with the Rust side restricted to parsing, candidate selection, and transport.

The plan row that scopes this: `clone` covers "跨文件 T1/T2（热路径）；T3 near-miss（冷路径）；**不承诺 T4**" (cross-file T1/T2 on the hot path, T3 near-miss on the cold path, no T4 promised), with the threshold given as "T3 TSED 0.85（定义与阈值仓内自定义并文档化）" (T3 TSED 0.85, defined and documented in-repo) — the repo owns the definition rather than citing one ([DEVELOPMENT_PLAN.md:64](../../DEVELOPMENT_PLAN.md#L64), restated at [Cost.hs:13-20](../../../core/app/CE/Clone/Cost.hs#L13)).

### The judged object

A *unit* is an admitted function-scale span carrying a cached node count. Units below the admission floor never enter T3 at all:

```
minUnitNodes = 24        // named nodes; below this a "clone" is a signature, not an implementation
```
([Cost.hs:38-39](../../../core/app/CE/Clone/Cost.hs#L38); the measuring side reads it off the definition package and applies it at [candidates.rs:144](../../../cli/src/dedup/candidates.rs#L144))

Each admitted unit is rebuilt into a postorder tree from a single parse per file ([t3/mod.rs:177-178](../../../cli/src/dedup/t3/mod.rs#L177)). The wire form is two parallel arrays: `lab[i]` = node kind code, `lld[i]` = postorder index of node `i`'s leftmost leaf descendant ([tree.rs:16-21](../../../cli/src/dedup/t3/tree.rs#L16), [Ted.hs:1-3](../../../core/app/CE/Clone/Ted.hs#L1)).

Node selection is the *maximal named nodes* inside the unit's 1-based inclusive line span, using the same predicate as `struct_fp::unit_seq` ([tree.rs:192-212](../../../cli/src/dedup/t3/tree.rs#L192), [tree.rs:217-219](../../../cli/src/dedup/t3/tree.rs#L217)). Anonymous intermediates are looked through, so the selected set matches the fingerprint spine by construction ([tree.rs:253-276](../../../cli/src/dedup/t3/tree.rs#L253)).

Two structural outcomes are possible, and neither is guessed:

- **Tree** — exactly one maximal node. Emitted postorder; `lld` is derived at ENTER time as `min(base, idx)` ([tree.rs:232-251](../../../cli/src/dedup/t3/tree.rs#L232)).
- **Forest** — the maximal count is `!= 1`. Ledgered as a drop, never rooted by fiat ([tree.rs:81-94](../../../cli/src/dedup/t3/tree.rs#L81), `forest_units` / `pairs_dropped_forest` at [t3/mod.rs:51-60](../../../cli/src/dedup/t3/mod.rs#L51)).

A per-unit equality assertion ties the two independent walks together: the built tree's `lab.len()` must equal the cached `unitsig.nodes`, otherwise the run dies rather than judging a drifted tree ([t3/mod.rs:211-219](../../../cli/src/dedup/t3/mod.rs#L211)).

Kind codes are raw FNV-1a hashes locally; the wire re-encodes them as **request-local dense labels** in first-seen order across the chunk's trees, since the judge only ever compares labels for equality ([wire.rs:22-39](../../../cli/src/dedup/t3/wire.rs#L22)).

### The distance

`ted` is Zhang-Shasha with unit costs — delete = insert = 1, relabel = 0 if the kind codes match, 1 otherwise ([Ted.hs:1-4](../../../core/app/CE/Clone/Ted.hs#L1), relabel at [Ted.hs:143](../../../core/app/CE/Clone/Ted.hs#L143)).

Keyroots are, per distinct `lld` value, the highest postorder index carrying it, sorted **ascending by node index** — IntMap key order is not postorder, and the accumulation requires subtree distances to exist before larger spans read them ([Ted.hs:84-92](../../../core/app/CE/Clone/Ted.hs#L84)).

One forest pass per keyroot pair fills a fresh `(w1+1) × (w2+1)` rectangle over spans `[lld i1 .. i1] × [lld j1 .. j1]`, harvesting permanent tree distances at left-aligned cells ([Ted.hs:112-126](../../../core/app/CE/Clone/Ted.hs#L112)). The recurrence, verbatim:

- `del = fd[di-1][dj] + 1` ([Ted.hs:139](../../../core/app/CE/Clone/Ted.hs#L139))
- `ins = fd[di][dj-1] + 1` ([Ted.hs:140](../../../core/app/CE/Clone/Ted.hs#L140))
- aligned (`lld a i == l1 && lld b j == l2`): `min(del, ins, fd[di-1][dj-1] + rel)`, then written through to the tree table ([Ted.hs:141-143](../../../core/app/CE/Clone/Ted.hs#L141), [Ted.hs:125](../../../core/app/CE/Clone/Ted.hs#L125))
- otherwise: `min(del, ins, fd[lld a i - l1][lld b j - l2] + td[i][j])` ([Ted.hs:144](../../../core/app/CE/Clone/Ted.hs#L144))

The answer is `td[(n1-1)][(n2-1)]` ([Ted.hs:73-74](../../../core/app/CE/Clone/Ted.hs#L73)). The empty tree is total-function territory: `ted = max(n1, n2)` ([Ted.hs:48](../../../core/app/CE/Clone/Ted.hs#L48)).

Tables are unboxed ST arrays, not `IntMap` — the 3e measured exit found per-cell IntMap inserts pushing ripgrep's cold path an order of magnitude past budget (self: 524 pairs ≈ 20 s) ([Ted.hs:5-9](../../../core/app/CE/Clone/Ted.hs#L5)).

Correctness is not argued in the implementation; it is asserted. `CloneProps.battery` holds `ted` equal to the mapping-definition brute force over the **exhaustive** small-tree family, plus identity, symmetry, and triangle inequality ([CloneProps.hs:37](../../../core/test/CloneProps.hs#L37), [CloneProps.hs:43-44](../../../core/test/CloneProps.hs#L43), [CloneProps.hs:107-113](../../../core/test/CloneProps.hs#L107)). CI walks `n ≤ 4`; `CE_DEEP_TED=1` extends to `n = 5` ([CloneProps.hs:10-11](../../../core/test/CloneProps.hs#L10), [CloneProps.hs:31-32](../../../core/test/CloneProps.hs#L31)).

### Normalization and threshold

TSED normalizes the raw distance by the **larger** of the two node counts:

```
TSED(a, b) = (max(n1, n2) − ted(a, b)) / max(n1, n2)
clone      ⇔ TSED ≥ 0.85
```

It is never evaluated as a float. The verdict is an integer cross-multiplication:

```haskell
cloneDecidesWith (num, den) t n1 n2 = (mx - t) * den >= num * mx  where mx = max n1 n2
```
([Cost.hs:65-68](../../../core/app/CE/Clone/Cost.hs#L65))

with the production binding

```
tsedNum = 85
tsedDen = 100
```
([Cost.hs:21-25](../../../core/app/CE/Clone/Cost.hs#L21))

No floats appear anywhere in core, because floats tie-break differently across platforms ([Cost.hs:16-18](../../../core/app/CE/Clone/Cost.hs#L16)). Since `ted` is always integral, the comparison is exact and the boundary is decidable in both directions: at `max = 100`, `ted 15` is a clone and `ted 16` is not (asserted through the shipped binding at [CloneProps.hs:55-61](../../../core/test/CloneProps.hs#L55)).

**Ownership (ADR-008 P1).** The verdict bit is computed by the threshold's owner — the core — and rides each score row over the wire; raw `ted`, `n1`, `n2` also cross so the instruments can recompute cut tables from one run ([Clone.hs:11-15](../../../core/app/CE/Clone.hs#L11), [Clone.hs:161-166](../../../core/app/CE/Clone.hs#L161), reply fields at [Clone.hs:185-186](../../../core/app/CE/Clone.hs#L185)). Since plan v2.33 W3 Rust holds no copy of the threshold at all: a row the verdict cache (`t3ted` in `.ce/index.db`) replays goes back to the core as a `decide` row `[ted, n1, n2]` ([Clone.hs:139-142](../../../core/app/CE/Clone.hs#L139), answered at [Clone.hs:200-203](../../../core/app/CE/Clone.hs#L200)), and a cached bit the core would not give kills the run by name:

> `the verdict cache's bit ({v}) disagrees with the core's decision at ted {ted} nodes {n1}/{n2} — a corrupt t3ted row (delete .ce/index.db to rebuild)`
> ([judge.rs:55-62](../../../cli/src/dedup/t3/judge.rs#L55))

The threshold constants are additionally pinned by a knobs echo: the reply must carry `tsedNum`/`tsedDen` matching the definition package this run read or the parse fails ([Clone.hs:194-196](../../../core/app/CE/Clone.hs#L194), [wire.rs:77-83](../../../cli/src/dedup/t3/wire.rs#L77); drift-refusal test at [unit/dedup/t3/wire.rs:32-34](../../../cli/tests/unit/dedup/t3/wire.rs#L32)).

The threshold is proven *live* rather than merely present: over the exhaustive family, 85/100 admits a nonempty clone set and 75/100 admits strictly more ([CloneProps.hs:139-146](../../../core/test/CloneProps.hs#L139)).

### Two provably admissible prefilters

Both bounds follow from the Tai-mapping cost identity. For a mapping `M` with `r` label-mismatched pairs, `cost = n1 + n2 − 2|M| + r`; zero-cost pairs number at most `I = Σ_label min(c1, c2)`, so `|M| − r ≤ I`, giving `cost ≥ n1 + n2 − |M| − I ≥ max(n1, n2) − I`, and since `I ≤ min(n1, n2)`, also `ted ≥ |n1 − n2|` ([Prefilter.hs:1-15](../../../core/app/CE/Clone/Prefilter.hs#L1)).

Hence for any bound quantity `q ∈ {min(n1,n2), I}`, `ted ≥ max − q` implies `TSED ≤ q / max`, so the O(1) test

```
q · tsedDen < tsedNum · max      ⇒  provably below threshold
```

decides "below", never "probably below" ([Prefilter.hs:39-43](../../../core/app/CE/Clone/Prefilter.hs#L39); both bounds stated once in `boundOf`, [Prefilter.hs:54-63](../../../core/app/CE/Clone/Prefilter.hs#L54)). The size bound is evaluated first — its tally owns pairs both bounds would cut ([Prefilter.hs:33-37](../../../core/app/CE/Clone/Prefilter.hs#L33)).

`I` is a multiset label-histogram intersection: `Σ_label min(c1, c2)` ([Prefilter.hs:29-31](../../../core/app/CE/Clone/Prefilter.hs#L29)). The candidate pass computes it as one merge walk over the two ascending histograms that stops as soon as the counts left cannot lift it to the floor `⌈85·max/100⌉` — the number it returns is below the floor exactly when `I` is ([Units.hs:89-110](../../../core/app/CE/Candidates/Units.hs#L89), floor at [Prefilter.hs:45-52](../../../core/app/CE/Clone/Prefilter.hs#L45)).

The filter is applied twice, both times in the core — the candidate pass prunes every union pair once before any tree is built ([T3.hs:67-75](../../../core/app/CE/Candidates/T3.hs#L67)), the judge prunes before TED ([Clone.hs:160](../../../core/app/CE/Clone.hs#L160)). A prefiltered pair produces no score row and no verdict bit at all; only a `prefiltered` counter ([Clone.hs:150-167](../../../core/app/CE/Clone.hs#L150), reported at [Clone.hs:191-192](../../../core/app/CE/Clone.hs#L191)). Admissibility is executed coverage, not a transcription: the shipped `provablyBelow` is asserted against real `ted` through the shipped `cloneDecides` ([CloneProps.hs:50](../../../core/test/CloneProps.hs#L50), [CloneProps.hs:88-90](../../../core/test/CloneProps.hs#L88)).

The prunes use the *same* 85/100 the judgment will — that identity is precisely what makes them admissible ([Cost.hs:18-20](../../../core/app/CE/Clone/Cost.hs#L18)); since plan v2.33 W3 there is no second copy to drift.

### The four candidate sources

Candidate generation consults neither TSED nor TED, so the candidate universe freezes before the judge exists — "no judge picks its own denominator" ([candidates.rs:1-6](../../../cli/src/dedup/candidates.rs#L1)). Since plan v2.33 W3 the core generates every source (`candidates/1`, [Sources.hs:1-21](../../../core/app/CE/Candidates/Sources.hs#L1)) from integer facts the measuring side sends — admitted unit rows, shingle sets, fingerprint instances and the S1 near-run anchors — and merges the walks into one `(pair → source bits)` map with one bit per source ([Cost.hs:49-57](../../../core/app/CE/Candidates/Cost.hs#L49), union at [Sources.hs:62-73](../../../core/app/CE/Candidates/Sources.hs#L62)):

| bit | id | source | definition |
|---|---|---|---|
| 0 | `s1` | `near_pairs` | Verified near-miss token runs the T1/T2 *report* threshold drops, reclaimed from `pairs.rs`'s second sink on the measuring side ([sources.rs:27-33](../../../cli/src/dedup/sources.rs#L27)), anchored in the core ([Sources.hs:77-78](../../../core/app/CE/Candidates/Sources.hs#L77)) |
| 1 | `s2` | `same_key` | Same unit key across **different** files, exhaustive within each key group ([T3.hs:105-124](../../../core/app/CE/Candidates/T3.hs#L105)) |
| 2 | `s3` | `fingerprint_pairs` | Raw fingerprint co-occurrence, **no extension** — deliberately wider than any reasonable candidate pass ([Sources.hs:113-124](../../../core/app/CE/Candidates/Sources.hs#L113)) |
| 3 | `s4` | `structural_pairs` | MinHash/LSH over structural shingle sets; supplementary only ([Lsh.hs:80-98](../../../core/app/CE/Candidates/Lsh.hs#L80)) |

Constants that parameterize these:

- **S1 band.** Runs land in the near sink when `len ∈ [near_floor, t)`. `near_floor = kgram = 25` and `t = guarantee() = window + kgram − 1 = 26 + 25 − 1 = 50` ([pairs.rs:254-259](../../../cli/src/dedup/pairs.rs#L254), [sources.rs:34-40](../../../cli/src/dedup/sources.rs#L34), [dedup/mod.rs:282-286](../../../cli/src/dedup/mod.rs#L282), defaults `kgram = 25, window = 26` at [dedup/mod.rs:290-298](../../../cli/src/dedup/mod.rs#L290)). `min_distinct = 0` for this pass ([sources.rs:39](../../../cli/src/dedup/sources.rs#L39)). S1 is **read-only** on the index — its writing predecessor silently re-hashed mid-run edits and orphaned their cascade-dropped edges ([sources.rs:29-32](../../../cli/src/dedup/sources.rs#L29)).
- **S4 LSH shape.** `lshShape = (128, 32, 4)` — (permutations, bands, rows), `128 = 32 × 4` ([Cost.hs:43-47](../../../core/app/CE/Candidates/Cost.hs#L43)). Permutation `i` is the salt: `sig[i]` = the minimum over the set of fnv1a64 of the element's eight little-endian bytes followed by `i`'s four, and a band's key is the fnv1a64 of its four words ([Lsh.hs:59-78](../../../core/app/CE/Candidates/Lsh.hs#L59)). Band-group size distribution is published as `s4_band_groups` so its discriminative power is a number, not a hope ([Sources.hs:72](../../../core/app/CE/Candidates/Sources.hs#L72)).
- **Hot-group cap.** `hotCap = 64` ([Dedup/Cost.hs:33-40](../../../core/app/CE/Dedup/Cost.hs#L33)), read by the measuring side's T1/T2 walk off the package ([pairs.rs:24-30](../../../cli/src/dedup/pairs.rs#L24)). Above the cap a group pairs as an adjacent chain rather than exhaustively, and the chaining is counted — skipping hot groups entirely had zeroed detection ([Groups.hs:1-20](../../../core/app/CE/Candidates/Groups.hs#L1)).

S1 and S3 are line-anchored: both endpoints must land inside an admitted unit, resolved to the **innermost** containing unit, or the pair is ledgered as `unowned_dropped` rather than guessed — the narrowest span, the earliest unit on a tie ([Sources.hs:126-135](../../../core/app/CE/Candidates/Sources.hs#L126), [Sources.hs:96-111](../../../core/app/CE/Candidates/Sources.hs#L96)).

**As-built addition — S5.** `SOURCES` is a five-element table; bit 4 is `s5`, an exhaustive in-domain source added at M5 close and **product-only** ([candidates.rs:46-53](../../../cli/src/dedup/candidates.rs#L46), [T3.hs:83-89](../../../core/app/CE/Candidates/T3.hs#L83)). It runs only when the request asks for it (`exhaustive`, set by `judge_index`) — the one judgment `ce clone` reports and, since plan v2.30 step 5b-9, `ce check` and `ce join` read — so the frozen four-source universe whose digest CI re-derives stays untouched. S5 walks same-language units sorted by node count and **generates** only pairs already inside the §4.3 size window — the identical predicate the prune applies, evaluated at generation so the pair space stays near-linear ([T3.hs:130-148](../../../core/app/CE/Candidates/T3.hs#L130)); a window pair the union already kept is counted, never duplicated, and the kept pairs are answered ascending because the clone wire refuses non-ascending pair rows ([T3.hs:63](../../../core/app/CE/Candidates/T3.hs#L63)).

### The cross-language drop

Every source funnels through one walk that canonicalizes and gates ([Sources.hs:96-111](../../../core/app/CE/Candidates/Sources.hs#L96); S2's own walk applies the same gates, [T3.hs:109-121](../../../core/app/CE/Candidates/T3.hs#L109)):

1. `x == y` → `self_pair_dropped` ([Sources.hs:106](../../../core/app/CE/Candidates/Sources.hs#L106)). The judge independently refuses self pairs at the boundary contract, since `[0,0]` would otherwise pass and be judged `ted 0` = certain clone ([Clone.hs:131-133](../../../core/app/CE/Clone.hs#L131)).
2. Canonicalize to `(min, max)` ([Sources.hs:111](../../../core/app/CE/Candidates/Sources.hs#L111)).
3. **`lang a /= lang b` → `cross_lang_dropped`, unconditionally** ([Sources.hs:107](../../../core/app/CE/Candidates/Sources.hs#L107)).

`lang` is the **grammar name** — `Lang::from_path(...).name()`, with the raw extension only as a totality fallback for a path no grammar claims ([candidates.rs:155-157](../../../cli/src/dedup/candidates.rs#L155)) — so `.ts`, `.mts` and `.cts` share one bucket while `.tsx` is its own grammar and bucket, and a `.c` and a `.h` are two (a header reads as C++); partitioning by literal extension was the defect batch-7 slice 15 removed, when a byte-identical `a.ts` → `b.mts` copy scored TED 0 and died at the cross-language gate. The drop is total: there is no cross-language T3 path anywhere in the pipeline. S5 inherits it structurally by bucketing on `lang` before pairing ([T3.hs:126-128](../../../core/app/CE/Candidates/T3.hs#L126)). Surviving pairs are tallied per `source/lang` key ([Sources.hs:109](../../../core/app/CE/Candidates/Sources.hs#L109)).

### Caps and the degraded reply

Two ceilings, owned by `CE.Clone.Cost`; since plan v2.33 W3 the measuring side reads both off the definition package (`limits.clone`, [wire.rs:1-6](../../../cli/src/dedup/t3/wire.rs#L1)) instead of keeping a mirror:

| constant | value | owner |
|---|---|---|
| `unitNodeCap` | 256 | [Cost.hs:41-42](../../../core/app/CE/Clone/Cost.hs#L41) |
| `pairCap` | 4096 | [Cost.hs:50-51](../../../core/app/CE/Clone/Cost.hs#L50) |

`unitNodeCap = 256` is a sizing anchor decided before any corpus measurement: Zhang-Shasha is `O(n1 · n2 · min(d,l)1 · min(d,l)2)`, so at 256 nodes with a typical `min(depth, leaves) ≈ 16` one pair costs `≈ 256 · 256 · 16 · 16 ≈ 1.7×10⁷` strict map updates ([Cost.hs:27-31](../../../core/app/CE/Clone/Cost.hs#L27)). `pairCap = 4096` applies *after* the two admissible prunes and is backed by the 3e measured exit, with zod's 21,740 survivors as the pressure case ([Cost.hs:44-51](../../../core/app/CE/Clone/Cost.hs#L44)).

Rust never builds an over-cap unit's tree ([t3/mod.rs:185-186](../../../cli/src/dedup/t3/mod.rs#L185)) and chunks requests at the package's `pair_cap` ([wire.rs:66](../../../cli/src/dedup/t3/wire.rs#L66)), so in a healthy run the core's over-cap branch is unreachable. If it fires, the core answers a **complete degraded reply, never a truncated one** ([Clone.hs:64-68](../../../core/app/CE/Clone.hs#L64), reason `clone_too_large` at [Clone.hs:199](../../../core/app/CE/Clone.hs#L199)) — and a degraded reply to a request laid out by the package's own caps is refused by name ([wire.rs:2-6](../../../cli/src/dedup/t3/wire.rs#L2), [wire.rs:98-101](../../../cli/src/dedup/t3/wire.rs#L98)).

The boundary contract is machine-checked in request order, naming the first offender deterministically: empty tree, `lab`/`lld` length mismatch, `lld` out of range (`l < 0 || l > i`), root `lld /= 0` (i.e. a forest, not a single tree), negative label, and postorder reconstructibility — node `i`'s children must tile `[lld i .. i−1]` exactly, walking right to left ([Clone.hs:100-123](../../../core/app/CE/Clone.hs#L100)). Pair rows must be `[i, j]`, in range, non-self, and strictly ascending across the request ([Clone.hs:127-136](../../../core/app/CE/Clone.hs#L127), [Clone.hs:91](../../../core/app/CE/Clone.hs#L91)).

Every pair that never reaches the wire lands in a named ledger, not in silence: `pairs_dropped_over_cap` and `pairs_dropped_forest`, with an over-cap endpoint claiming the pair first ([t3/mod.rs:232-249](../../../cli/src/dedup/t3/mod.rs#L232)), alongside `survivors`, `s5_*`, `sent`, `requests`, `prefiltered`, `judged`, `cached` ([t3/mod.rs:48-67](../../../cli/src/dedup/t3/mod.rs#L48)), with `clones` counted by the core. Report schema id: <!--ce:report:clone#schemaver-->`ce.clone-report/0.3.0`<!--/ce--> ([Clone/Document.hs:57](../../../core/app/CE/Clone/Document.hs#L57)).

### T3 in the gate and the verdict cache (plan v2.30 step 5b-9)

Until 5b-9 the family's verdict reached one face, `ce clone`. The check gate's `sim` table took the T1/T2 blocks (kind 0) and the docdup pairs (kind 2), the join's similarity leg took the blocks alone, and kind 1 — the lattice's name for a T3 pair — was dead on every live road (booklet 07, leg 1). Since 5b-9 both roads judge the family: `score::measure` runs `judge_index` over the same refreshed index snapshot the blocks come from ([t3/mod.rs:119-121](../../../cli/src/dedup/t3/mod.rs#L119), called at [score/mod.rs:106](../../../cli/src/score/mod.rs#L106)), `clone_rows` seats each verified pair's files as a kind-1 row after the kind-0 rows ([score/mod.rs:323-340](../../../cli/src/score/mod.rs#L323)), and `ce join` reads the same `Similar` for its file tier (`near_miss`) and unit tier (`t3` rows with `ted`, `n1`, `n2`) ([join/mod.rs:54-61](../../../cli/src/join/mod.rs#L54)). The core moved nothing: the clone axis already read `kind <= 1` ([Score.hs:210](../../../core/app/CE/Verdict/Score.hs#L210)) and the join lattice already judged kinds 0 and 1 against the clone bar ([Join.hs:162-164](../../../core/app/CE/Verdict/Join.hs#L162)); a kind-1 row is the clone axis's fact exactly as a kind-0 row is, and no docdup fact, pinned by the verdict battery ([VerdictProps.hs:217-229](../../../core/test/VerdictProps.hs#L217)). The discrete ratchet's clone members stay the T1/T2 blocks (booklet 05, § the discrete ratchet).

What made the gate affordable is the **verdict cache**. The core's judgment over the sendable pairs was the one T3 phase that scales with the pair count — on this repository, release, warm, before the cache: candidates 0.6 s, the S5 extension 0.08 s, the trees 0.35 s, the tree edit distance over 2,155 pairs 3.1 s (PERF-BUDGET, 5b-9) — and 5b-9 puts that judgment into every `ce check`, every trend point and every `ce join`. The cache is a table in the index, `t3ted (ka, kb, na, nb, ted, clone)`, keyed by the two trees ([cache.rs:38-47](../../../cli/src/dedup/t3/cache.rs#L38)):

- **Key = the judge's whole input for a tree.** `tree_key` is fnv1a over the postorder kind codes and the `lld` column ([cache.rs:65-76](../../../cli/src/dedup/t3/cache.rs#L65)) — exactly what the wire sends per tree, so two units with equal keys are the same tree to the judge whatever file or language they came from; on this repository 2,158 sendable pairs occupy 1,740 slots. A pair reads one slot in key order, swapped when the reader's `(a, b)` is the slot's `(kb, ka)` — `ted` is symmetric, the sizes are not ([cache.rs:94-106](../../../cli/src/dedup/t3/cache.rs#L94)).
- **Generation = (CACHE_REV, the core's proto, the TSED knobs, `minUnitNodes`)** — the knobs and the floor as the definition package states them. The table carries its generation in `meta`; a mismatch empties it before any row is read, so a core that judges differently is announced by its version and never replayed as this one's ([cache.rs:114-133](../../../cli/src/dedup/t3/cache.rs#L114)).
- **Replay, not policy (ADR-008).** A held `Scored` row becomes this run's row in the reader's orientation, a held `Below` row — the core's prefilter proved the pair below threshold — is a pair with no row, as the core would answer it, and the rest go to the wire ([cache.rs:159-182](../../../cli/src/dedup/t3/cache.rs#L159)); every replayed row's bit is checked against the core's own decision over `clone/1`'s `decide` before anything is reported ([judge.rs:54-65](../../../cli/src/dedup/t3/judge.rs#L54)), so a poisoned bit is refused rather than trusted (`it/t3_cache.rs`), and the reported set is the rows whose bit is set ([t3/mod.rs:166-175](../../../cli/src/dedup/t3/mod.rs#L166)).
- **Bounded by the live tree pairs.** Fresh outcomes are remembered and the rows whose trees left the tree are swept in one transaction ([cache.rs:208-238](../../../cli/src/dedup/t3/cache.rs#L208)); the whole-database wipe and the parser-revision invalidation drop the table with the other derived tables ([schema.rs:60-71](../../../cli/src/dedup/schema.rs#L60), [parser.rs:34](../../../cli/src/dedup/schema/parser.rs#L34)).

The report says what happened: `cached` joins `judged` and `prefiltered`, and the three sum to `sent` on every run ([t3/mod.rs:62-66](../../../cli/src/dedup/t3/mod.rs#L62)). A second `ce clone` over an unchanged tree asks the core nothing and reports the same clones; a tree that changed re-sends its own pairs and no other.

### Recall-floor epoch discipline

The M5-3A acceptance row sets the comparator and the rules ([DEVELOPMENT_PLAN.md:287](../../DEVELOPMENT_PLAN.md#L287)):

- **Denominator never shrinks** (`分母永不缩减`). It is the comparator's full default-parameter detection set — mizchi/similarity, with per-language thresholds `similarity-ts 0.87`, `similarity-py 0.85`, `similarity-generic 0.85` ([EVAL-SET-M5-CLOSE.md:37-39](../../EVAL-SET-M5-CLOSE.md#L37)). ripgrep and self are excluded with reasons on record ([EVAL-SET-M5-CLOSE.md:41-42](../../EVAL-SET-M5-CLOSE.md#L41)).
- **Credit is whole-stack.** A comparator hit counts as detected if *any* CE layer reports it — a T1/T2 block or a T3 clone verdict with ≥1 line of span overlap on both sides. A T1/T2 hit is a product true positive, not an exclusion ([DEVELOPMENT_PLAN.md:287](../../DEVELOPMENT_PLAN.md#L287), [EVAL-SET-M5-CLOSE.md:43](../../EVAL-SET-M5-CLOSE.md#L43)).
- **Misses are attributed by a closed vocabulary** into a frozen ledger; growing the vocabulary requires explicit accept ([DEVELOPMENT_PLAN.md:287](../../DEVELOPMENT_PLAN.md#L287)).

The original literal gate was recall ≥ 0.90. The instrument proved it *unreachable by definition* under the in-repo TSED, because 100% of misses are definitional rather than blind spots ([EVAL-SET-M5-CLOSE.md:56-61](../../EVAL-SET-M5-CLOSE.md#L56)):

| bucket | zod / requests / cobra | meaning |
|---|---|---|
| `size_bound_not_clone` | 1 / 135 / 4453 | best-case `min/max < 0.85` — mathematically impossible under registered TSED |
| `below_floor` | 0 / 0 / 2578 | short units, `minUnitNodes = 24` domain boundary |
| `judged_not_clone` | 2 / 223 / 757 | actually sent to TED, rejected at `θ = 85/100` |

Frozen epoch values (`t3-recall-{zod,requests,cobra}-v1.json`, `ce.eval-t3-recall/1.0.0`) — `recall_raw` **zod 3/6 = 0.50**, **requests 67/425 = 0.158**, **cobra 1417/9205 = 0.154**; `recall_incremental` 0.0 / 0.058 / 0.083, with the written-disposition trigger at `< 0.50` discharged in that section ([EVAL-SET-M5-CLOSE.md:51-56](../../EVAL-SET-M5-CLOSE.md#L51)). Conclusion of record: the shortfall is *measure divergence, not blindness* — mizchi's similarity axis is not CE's TSED axis ([EVAL-SET-M5-CLOSE.md:59-61](../../EVAL-SET-M5-CLOSE.md#L59)).

Plan amendment **v1.6** (user decision 2026-08-14) therefore replaced the literal gate with a **monotone-nondecreasing regression floor** — the frozen epoch values above become a floor that may rise and may never fall ([DEVELOPMENT_PLAN.md:287](../../DEVELOPMENT_PLAN.md#L287), [EVAL-SET-M5-CLOSE.md:61](../../EVAL-SET-M5-CLOSE.md#L61)). Candidate blindness itself was root-fixed rather than accepted: before S5, requests produced 128 candidate pairs against a 425-pair denominator and cobra 1,124 against 9,205 — a hard ceiling; after S5 the `not_candidate` bucket is **zero** ([EVAL-SET-M5-CLOSE.md:45-49](../../EVAL-SET-M5-CLOSE.md#L45)). The cost was published, not hidden: post-S5 cold `ce clone` at 1.8 s (requests) / 3.6 s (cobra) / 47.1 s (zod), against 24.9 s for zod pre-S5 ([EVAL-SET-M5-CLOSE.md:62-63](../../EVAL-SET-M5-CLOSE.md#L62)).

**Epoch semantics.** A frozen family is pinned to one detector version. Sample rows embed unit keys and sampling is in key-hash order, so re-freezing candidates necessarily breaks the `pool_digests` anchor chain and re-freezing the sample invalidates the five auditors' ground truth — *partial re-freeze does not exist*. Regeneration must re-establish the **whole family together with its audit** under a new epoch ([EVAL-SET-M5-3.md:49-56](../../EVAL-SET-M5-3.md#L49)).

The paired precision gate is **≥ 85%**, scored against the frozen four-source candidate universe with independent audited ground truth, over answered rows only, with an output-volume floor ([DEVELOPMENT_PLAN.md:287](../../DEVELOPMENT_PLAN.md#L287)); the audited run recorded `θ` swept 70..100 with per-cell `wrong` identically 0 and the contract grid point 85 on file ([EVAL-SET-M5-3.md:46-47](../../EVAL-SET-M5-3.md#L46)).

**Current status.** The gate `eval_t3_recall` and its frozen artifacts were retired in the v0.5.0 slimming batch; the full record lives in git history ([EVAL-SET-M5-CLOSE.md:52-53](../../EVAL-SET-M5-CLOSE.md#L52), retirement inventory at [EVAL-SET.md:297](../../EVAL-SET.md#L297)). The three-family universe drift nets and precision regression gates remain live, reading the frozen sample/oracle JSON directly ([EVAL-SET.md:297](../../EVAL-SET.md#L297)).
