# T3 near-miss clones — Tree Edit Distance (TSED)

[index](../methodology.md) · [← 01 T1/T2 clone detection — winnowing fingerprint index](01-t1-t2-clone-detection-winnowing-fingerprint.md) · [→ 03 Documentation duplication — shingling + MinHash/LSH](03-documentation-duplication-shingling-minhash.md)

T1/T2 clone detection reports *exact* and *parameterized* duplicate token runs. T3 covers the near-miss case: two units whose ASTs are structurally almost the same but whose token streams are not. The judgment is an exact tree edit distance under a fixed normalization, computed in the Haskell core, with the Rust side restricted to parsing, candidate selection, and transport.

The plan row that scopes this: `clone` covers "跨文件 T1/T2（热路径）；T3 near-miss（冷路径）；**不承诺 T4**", with the threshold given as "T3 TSED 0.85（定义与阈值仓内自定义并文档化）" — the repo owns the definition rather than citing one ([DEVELOPMENT_PLAN.md:63](../../DEVELOPMENT_PLAN.md#L63), restated at [Cost.hs:13-20](../../../core/app/CE/Clone/Cost.hs#L13)).

### The judged object

A *unit* is an admitted function-scale span carrying a cached node count. Units below the admission floor never enter T3 at all:

```
T3_MIN_NODES = 24        // named nodes; below this a "clone" is a signature, not an implementation
```
([candidates.rs:25](../../../cli/src/dedup/candidates.rs#L25), applied at [candidates.rs:129](../../../cli/src/dedup/candidates.rs#L129))

Each admitted unit is rebuilt into a postorder tree from a single parse per file ([t3/mod.rs:155-194](../../../cli/src/dedup/t3/mod.rs#L155)). The wire form is two parallel arrays: `lab[i]` = node kind code, `lld[i]` = postorder index of node `i`'s leftmost leaf descendant ([tree.rs:15-20](../../../cli/src/dedup/t3/tree.rs#L15), [Ted.hs:1-3](../../../core/app/CE/Clone/Ted.hs#L1)).

Node selection is the *maximal named nodes* inside the unit's 1-based inclusive line span, using the same predicate as `struct_fp::unit_seq` ([tree.rs:58-78](../../../cli/src/dedup/t3/tree.rs#L58), [tree.rs:82-84](../../../cli/src/dedup/t3/tree.rs#L82)). Anonymous intermediates are looked through, so the selected set matches the fingerprint spine by construction ([tree.rs:108-123](../../../cli/src/dedup/t3/tree.rs#L108)).

Two structural outcomes are possible, and neither is guessed:

- **Tree** — exactly one maximal node. Emitted postorder; `lld` is derived at ENTER time as `min(base, idx)` ([tree.rs:93-106](../../../cli/src/dedup/t3/tree.rs#L93)).
- **Forest** — the maximal count is `!= 1`. Ledgered as a drop, never rooted by fiat ([tree.rs:42-56](../../../cli/src/dedup/t3/tree.rs#L42), `forest_units` / `pairs_dropped_forest` at [t3/mod.rs:47-56](../../../cli/src/dedup/t3/mod.rs#L47)).

A per-unit equality assertion ties the two independent walks together: the built tree's `lab.len()` must equal the cached `unitsig.nodes`, otherwise the run dies rather than judging a drifted tree ([t3/mod.rs:177-185](../../../cli/src/dedup/t3/mod.rs#L177)).

Kind codes are raw FNV-1a hashes locally; the wire re-encodes them as **request-local dense labels** in first-seen order across the chunk's trees, since the judge only ever compares labels for equality ([wire.rs:28-45](../../../cli/src/dedup/t3/wire.rs#L28)).

### The distance

`ted` is Zhang-Shasha with unit costs — delete = insert = 1, relabel = 0 if the kind codes match, 1 otherwise ([Ted.hs:1-4](../../../core/app/CE/Clone/Ted.hs#L1), relabel at [Ted.hs:87](../../../core/app/CE/Clone/Ted.hs#L87)).

Keyroots are, per distinct `lld` value, the highest postorder index carrying it, sorted **ascending by node index** — IntMap key order is not postorder, and the accumulation requires subtree distances to exist before larger spans read them ([Ted.hs:57-65](../../../core/app/CE/Clone/Ted.hs#L57)).

One forest pass per keyroot pair fills a fresh `(w1+1) × (w2+1)` rectangle over spans `[lld i1 .. i1] × [lld j1 .. j1]`, harvesting permanent tree distances at left-aligned cells ([Ted.hs:66-90](../../../core/app/CE/Clone/Ted.hs#L66)). The recurrence, verbatim:

- `del = fd[di-1][dj] + 1` ([Ted.hs:82](../../../core/app/CE/Clone/Ted.hs#L82))
- `ins = fd[di][dj-1] + 1` ([Ted.hs:83](../../../core/app/CE/Clone/Ted.hs#L83))
- aligned (`lld a i == l1 && lld b j == l2`): `min(del, ins, fd[di-1][dj-1] + rel)`, then written through to the tree table ([Ted.hs:81-89](../../../core/app/CE/Clone/Ted.hs#L81), [Ted.hs:95](../../../core/app/CE/Clone/Ted.hs#L95))
- otherwise: `min(del, ins, fd[lld a i - l1][lld b j - l2] + td[i][j])` ([Ted.hs:90-93](../../../core/app/CE/Clone/Ted.hs#L90))

The answer is `td[(n1-1)][(n2-1)]` ([Ted.hs:47](../../../core/app/CE/Clone/Ted.hs#L47)). The empty tree is total-function territory: `ted = max(n1, n2)` ([Ted.hs:42](../../../core/app/CE/Clone/Ted.hs#L42)).

Tables are unboxed ST arrays, not `IntMap` — the 3e measured exit found per-cell IntMap inserts pushing ripgrep's cold path an order of magnitude past budget (self: 524 pairs ≈ 20 s) ([Ted.hs:5-9](../../../core/app/CE/Clone/Ted.hs#L5)).

Correctness is not argued in the implementation; it is asserted. `CloneProps.battery` holds `ted` equal to the mapping-definition brute force over the **exhaustive** small-tree family, plus identity, symmetry, and triangle inequality ([CloneProps.hs:28](../../../core/test/CloneProps.hs#L28), [CloneProps.hs:34-35](../../../core/test/CloneProps.hs#L34), [CloneProps.hs:97-103](../../../core/test/CloneProps.hs#L97)). CI walks `n ≤ 4`; `CE_DEEP_TED=1` extends to `n = 5` ([CloneProps.hs:8-9](../../../core/test/CloneProps.hs#L8), [CloneProps.hs:22-23](../../../core/test/CloneProps.hs#L22)).

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

No floats appear anywhere in core, because floats tie-break differently across platforms ([Cost.hs:16-18](../../../core/app/CE/Clone/Cost.hs#L16)). Since `ted` is always integral, the comparison is exact and the boundary is decidable in both directions: at `max = 100`, `ted 15` is a clone and `ted 16` is not ([t3/mod.rs:255-262](../../../cli/src/dedup/t3/mod.rs#L255), same pair asserted through the shipped Haskell binding at [CloneProps.hs:46-52](../../../core/test/CloneProps.hs#L46)).

**Ownership (ADR-008 P1).** The verdict bit is computed by the threshold's owner — the core — and rides each score row over the wire; raw `ted`, `n1`, `n2` also cross so the instruments can recompute cut tables from one run ([Clone.hs:11-15](../../../core/app/CE/Clone.hs#L11), [Clone.hs:135-140](../../../core/app/CE/Clone.hs#L135), reply fields at [Clone.hs:159-160](../../../core/app/CE/Clone.hs#L159)). Rust's `is_clone` is a **mirror**, not an authority ([t3/mod.rs:138-147](../../../cli/src/dedup/t3/mod.rs#L138)), and every reported row is checked against it — disagreement kills the run by name:

> `core clone verdict ({v}) disagrees with the pinned mirror at ted {ted} nodes {n1}/{n2} — formula drift (Clone/Cost.hs vs t3/mod.rs)`
> ([t3/mod.rs:121-125](../../../cli/src/dedup/t3/mod.rs#L121))

The threshold constants are additionally pinned by a knobs echo: the reply must carry `tsedNum`/`tsedDen` matching the Rust constants or the parse fails ([Clone.hs:168-170](../../../core/app/CE/Clone.hs#L168), [wire.rs:81-87](../../../cli/src/dedup/t3/wire.rs#L81); drift-refusal test at [wire.rs:135-137](../../../cli/src/dedup/t3/wire.rs#L135)).

The threshold is proven *live* rather than merely present: over the exhaustive family, 85/100 admits a nonempty clone set and 75/100 admits strictly more ([CloneProps.hs:110-117](../../../core/test/CloneProps.hs#L110)).

### Two provably admissible prefilters

Both bounds follow from the Tai-mapping cost identity. For a mapping `M` with `r` label-mismatched pairs, `cost = n1 + n2 − 2|M| + r`; zero-cost pairs number at most `I = Σ_label min(c1, c2)`, so `|M| − r ≤ I`, giving `cost ≥ n1 + n2 − |M| − I ≥ max(n1, n2) − I`, and since `I ≤ min(n1, n2)`, also `ted ≥ |n1 − n2|` ([Prefilter.hs:1-15](../../../core/app/CE/Clone/Prefilter.hs#L1)).

Hence for any bound quantity `q ∈ {min(n1,n2), I}`, `ted ≥ max − q` implies `TSED ≤ q / max`, so the O(1) test

```
q · tsedDen < tsedNum · max      ⇒  provably below threshold
```

decides "below", never "probably below" ([Prefilter.hs:33-41](../../../core/app/CE/Clone/Prefilter.hs#L33); Rust twin at [candidates.rs:165-180](../../../cli/src/dedup/candidates.rs#L165)). The size bound is evaluated first — its tally owns pairs both bounds would cut ([candidates.rs:176-186](../../../cli/src/dedup/candidates.rs#L176)).

`I` is a multiset label-histogram intersection: `Σ_label min(c1, c2)` ([Prefilter.hs:26-31](../../../core/app/CE/Clone/Prefilter.hs#L26), sorted-merge Rust form at [struct_fp.rs:99-113](../../../cli/src/dedup/struct_fp.rs#L99)).

The filter is applied on **both** sides — Rust prunes before shipping ([candidates.rs:182-193](../../../cli/src/dedup/candidates.rs#L182)), the judge prunes before TED ([Clone.hs:134](../../../core/app/CE/Clone.hs#L134)). A prefiltered pair produces no score row and no verdict bit at all; only a `prefiltered` counter ([Clone.hs:123-141](../../../core/app/CE/Clone.hs#L123), reported at [Clone.hs:165-166](../../../core/app/CE/Clone.hs#L165)). Admissibility is executed coverage, not a transcription: the shipped `provablyBelow` is asserted against real `ted` through the shipped `cloneDecides` ([CloneProps.hs:41](../../../core/test/CloneProps.hs#L41), [CloneProps.hs:78-80](../../../core/test/CloneProps.hs#L78)).

The prunes use the *same* 85/100 the judgment will — that identity is precisely what makes them admissible ([candidates.rs:27-32](../../../cli/src/dedup/candidates.rs#L27), [Cost.hs:18-20](../../../core/app/CE/Clone/Cost.hs#L18)).

### The four candidate sources

Candidate generation consults neither TSED nor TED, so the candidate universe freezes before the judge exists — "no judge picks its own denominator" ([candidates.rs:1-6](../../../cli/src/dedup/candidates.rs#L1)). The four generator walks are merged into one `(pair → source bits)` map, where **array order is the bit assignment** ([sources.rs:37-56](../../../cli/src/dedup/sources.rs#L37)):

| bit | id | source | definition |
|---|---|---|---|
| 0 | `s1` | `near_pairs` | Verified near-miss token runs the T1/T2 *report* threshold drops, reclaimed from `pairs.rs`'s second sink ([sources.rs:139-164](../../../cli/src/dedup/sources.rs#L139)) |
| 1 | `s2` | `same_key` | Same unit key across **different** files, exhaustive within each key group ([sources.rs:96-112](../../../cli/src/dedup/sources.rs#L96)) |
| 2 | `s3` | `fingerprint_pairs` | Raw fingerprint co-occurrence, **no extension** — deliberately wider than any reasonable candidate pass ([sources.rs:117-131](../../../cli/src/dedup/sources.rs#L117)) |
| 3 | `s4` | `structural_pairs` | MinHash/LSH over structural shingle sets; supplementary only ([sources.rs:170-188](../../../cli/src/dedup/sources.rs#L170)) |

Constants that parameterize these:

- **S1 band.** Runs land in the near sink when `len ∈ [near_floor, t)`. `near_floor = kgram = 25` and `t = guarantee() = window + kgram − 1 = 26 + 25 − 1 = 50` ([pairs.rs:248-253](../../../cli/src/dedup/pairs.rs#L248), [sources.rs:147-151](../../../cli/src/dedup/sources.rs#L147), [dedup/mod.rs:290-294](../../../cli/src/dedup/mod.rs#L290), defaults `kgram = 25, window = 26` at [dedup/mod.rs:297-306](../../../cli/src/dedup/mod.rs#L297)). `min_distinct = 0` for this pass ([sources.rs:149](../../../cli/src/dedup/sources.rs#L149)). S1 is **read-only** on the index — its writing predecessor silently re-hashed mid-run edits and orphaned their cascade-dropped edges ([sources.rs:136-138](../../../cli/src/dedup/sources.rs#L136)).
- **S4 LSH shape.** `LSH_SHAPE = (128, 32, 4)` — (permutations, bands, rows), `128 = 32 × 4`, and `band_keys` asserts the product covers the signature ([candidates.rs:34-37](../../../cli/src/dedup/candidates.rs#L34), consumed at [sources.rs:178-180](../../../cli/src/dedup/sources.rs#L178)). Band-group size distribution is published as `s4_band_groups` so its discriminative power is a number, not a hope ([sources.rs:166-169](../../../cli/src/dedup/sources.rs#L166), [candidates.rs:86](../../../cli/src/dedup/candidates.rs#L86)).
- **Hot-group cap.** `HOT_GROUP_CAP = pairs::HOT_CAP = 64` ([candidates.rs:42](../../../cli/src/dedup/candidates.rs#L42), [pairs.rs:27](../../../cli/src/dedup/pairs.rs#L27)). Above the cap a group pairs as an adjacent chain rather than exhaustively, and the chaining is counted — skipping hot groups entirely had zeroed detection ([sources.rs:191-209](../../../cli/src/dedup/sources.rs#L191), [candidates.rs:39-42](../../../cli/src/dedup/candidates.rs#L39)).

S1 and S3 are line-anchored: both endpoints must land inside an admitted unit, resolved to the **innermost** containing unit, or the pair is ledgered as `unowned_dropped` rather than guessed ([sources.rs:59-66](../../../cli/src/dedup/sources.rs#L59), [sources.rs:87-92](../../../cli/src/dedup/sources.rs#L87)).

**As-built addition — S5.** `SOURCES` is a five-element table; bit 4 is `s5`, an exhaustive in-domain source added at M5 close and **product-only** ([candidates.rs:44-51](../../../cli/src/dedup/candidates.rs#L44), [candidates.rs:195-232](../../../cli/src/dedup/candidates.rs#L195)). It runs at the `ce clone` seam via `extend_exhaustive`, after `collect()` ([t3/mod.rs:73-77](../../../cli/src/dedup/t3/mod.rs#L73)), so the frozen four-source universe whose digest CI re-derives stays untouched. S5 walks same-language units sorted by node count and **generates** only pairs already inside the §4.3 size window — the identical predicate the prune applies, evaluated at generation so the pair space stays near-linear ([candidates.rs:245-249](../../../cli/src/dedup/candidates.rs#L245)); appended rows are re-sorted because the clone wire refuses non-ascending pair rows ([candidates.rs:241-244](../../../cli/src/dedup/candidates.rs#L241)).

### The cross-language drop

Every source funnels through one `push` throat that canonicalizes and gates ([sources.rs:69-83](../../../cli/src/dedup/sources.rs#L69)):

1. `x == y` → `self_pair_dropped` ([sources.rs:70-73](../../../cli/src/dedup/sources.rs#L70)). The judge independently refuses self pairs at the boundary contract, since `[0,0]` would otherwise pass and be judged `ted 0` = certain clone ([Clone.hs:116-118](../../../core/app/CE/Clone.hs#L116)).
2. Canonicalize to `(min, max)` ([sources.rs:74](../../../cli/src/dedup/sources.rs#L74)).
3. **`units[a].lang != units[b].lang` → `cross_lang_dropped`, unconditionally** ([sources.rs:75-78](../../../cli/src/dedup/sources.rs#L75)).

`lang` is the **grammar name** — `Lang::from_path(...).name()`, with the raw extension only as a totality fallback for a path no grammar claims ([candidates.rs:140-142](../../../cli/src/dedup/candidates.rs#L140)) — so `.ts`, `.tsx` and `.mts` share one bucket; partitioning by literal extension was the defect batch-7 slice 15 removed, when a byte-identical `a.ts` → `b.mts` copy scored TED 0 and died at the cross-language gate. The drop is total: there is no cross-language T3 path anywhere in the pipeline. S5 inherits it structurally by bucketing on `lang` before pairing ([candidates.rs:211-219](../../../cli/src/dedup/candidates.rs#L211)). Surviving pairs are tallied per `source/lang` key ([sources.rs:79-82](../../../cli/src/dedup/sources.rs#L79)).

### Caps and the degraded reply

Two ceilings, owned by `CE.Clone.Cost` and mirrored in Rust:

| constant | value | owner | mirror |
|---|---|---|---|
| `unitNodeCap` | 256 | [Cost.hs:41-42](../../../core/app/CE/Clone/Cost.hs#L41) | `UNIT_NODE_CAP` [wire.rs:17-18](../../../cli/src/dedup/t3/wire.rs#L17) |
| `pairCap` | 4096 | [Cost.hs:50-51](../../../core/app/CE/Clone/Cost.hs#L50) | `PAIR_CAP` [wire.rs:20-21](../../../cli/src/dedup/t3/wire.rs#L20) |

`unitNodeCap = 256` is a sizing anchor decided before any corpus measurement: Zhang-Shasha is `O(n1 · n2 · min(d,l)1 · min(d,l)2)`, so at 256 nodes with a typical `min(depth, leaves) ≈ 16` one pair costs `≈ 256 · 256 · 16 · 16 ≈ 1.7×10⁷` strict map updates ([Cost.hs:27-31](../../../core/app/CE/Clone/Cost.hs#L27)). `pairCap = 4096` applies *after* the two admissible prunes and is backed by the 3e measured exit, with zod's 21,740 survivors as the pressure case ([Cost.hs:44-51](../../../core/app/CE/Clone/Cost.hs#L44)).

Rust never builds an over-cap unit's tree ([t3/mod.rs:159-160](../../../cli/src/dedup/t3/mod.rs#L159)) and chunks requests at `PAIR_CAP` ([wire.rs:64-71](../../../cli/src/dedup/t3/wire.rs#L64)), so in a healthy run the core's over-cap branch is unreachable. If it fires, the core answers a **complete degraded reply, never a truncated one** ([Clone.hs:54-58](../../../core/app/CE/Clone.hs#L54), reason `clone_too_large` at [Clone.hs:173](../../../core/app/CE/Clone.hs#L173)) — and a degraded reply to a client-sized request is precisely the signal that the cap mirrors disagree with `Cost.hs` ([wire.rs:2-6](../../../cli/src/dedup/t3/wire.rs#L2), [wire.rs:76-79](../../../cli/src/dedup/t3/wire.rs#L76)).

The boundary contract is machine-checked in request order, naming the first offender deterministically: empty tree, `lab`/`lld` length mismatch, `lld` out of range (`l < 0 || l > i`), root `lld /= 0` (i.e. a forest, not a single tree), negative label, and postorder reconstructibility — node `i`'s children must tile `[lld i .. i−1]` exactly, walking right to left ([Clone.hs:85-109](../../../core/app/CE/Clone.hs#L85)). Pair rows must be `[i, j]`, in range, non-self, and strictly ascending across the request ([Clone.hs:111-121](../../../core/app/CE/Clone.hs#L111), [Clone.hs:80](../../../core/app/CE/Clone.hs#L80)).

Every pair that never reaches the wire lands in a named ledger, not in silence: `pairs_dropped_over_cap` and `pairs_dropped_forest`, with an over-cap endpoint claiming the pair first ([t3/mod.rs:198-215](../../../cli/src/dedup/t3/mod.rs#L198)), alongside `survivors`, `s5_*`, `sent`, `requests`, `prefiltered`, `judged`, `clones` ([t3/mod.rs:44-61](../../../cli/src/dedup/t3/mod.rs#L44)). Report schema id: `ce.clone-report/0.2.0` ([t3/mod.rs:20](../../../cli/src/dedup/t3/mod.rs#L20)).

### Recall-floor epoch discipline

The M5-3A acceptance row sets the comparator and the rules ([DEVELOPMENT_PLAN.md:274](../../DEVELOPMENT_PLAN.md#L274)):

- **Denominator never shrinks** (`分母永不缩减`). It is the comparator's full default-parameter detection set — mizchi/similarity, with per-language thresholds `similarity-ts 0.87`, `similarity-py 0.85`, `similarity-generic 0.85` ([EVAL-SET-M5-CLOSE.md:37-39](../../EVAL-SET-M5-CLOSE.md#L37)). ripgrep and self are excluded with reasons on record ([EVAL-SET-M5-CLOSE.md:41-42](../../EVAL-SET-M5-CLOSE.md#L41)).
- **Credit is whole-stack.** A comparator hit counts as detected if *any* CE layer reports it — a T1/T2 block or a T3 clone verdict with ≥1 line of span overlap on both sides. A T1/T2 hit is a product true positive, not an exclusion ([DEVELOPMENT_PLAN.md:274](../../DEVELOPMENT_PLAN.md#L274), [EVAL-SET-M5-CLOSE.md:43](../../EVAL-SET-M5-CLOSE.md#L43)).
- **Misses are attributed by a closed vocabulary** into a frozen ledger; growing the vocabulary requires explicit accept ([DEVELOPMENT_PLAN.md:274](../../DEVELOPMENT_PLAN.md#L274)).

The original literal gate was recall ≥ 0.90. The instrument proved it *unreachable by definition* under the in-repo TSED, because 100% of misses are definitional rather than blind spots ([EVAL-SET-M5-CLOSE.md:56-61](../../EVAL-SET-M5-CLOSE.md#L56)):

| bucket | zod / requests / cobra | meaning |
|---|---|---|
| `size_bound_not_clone` | 1 / 135 / 4453 | best-case `min/max < 0.85` — mathematically impossible under registered TSED |
| `below_floor` | 0 / 0 / 2578 | short units, `T3_MIN_NODES = 24` domain boundary |
| `judged_not_clone` | 2 / 223 / 757 | actually sent to TED, rejected at `θ = 85/100` |

Frozen epoch values (`t3-recall-{zod,requests,cobra}-v1.json`, `ce.eval-t3-recall/1.0.0`) — `recall_raw` **zod 3/6 = 0.50**, **requests 67/425 = 0.158**, **cobra 1417/9205 = 0.154**; `recall_incremental` 0.0 / 0.058 / 0.083, with the written-disposition trigger at `< 0.50` discharged in that section ([EVAL-SET-M5-CLOSE.md:51-56](../../EVAL-SET-M5-CLOSE.md#L51)). Conclusion of record: the shortfall is *measure divergence, not blindness* — mizchi's similarity axis is not CE's TSED axis ([EVAL-SET-M5-CLOSE.md:59-61](../../EVAL-SET-M5-CLOSE.md#L59)).

Plan amendment **v1.6** (user decision 2026-08-14) therefore replaced the literal gate with a **monotone-nondecreasing regression floor** — the frozen epoch values above become a floor that may rise and may never fall ([DEVELOPMENT_PLAN.md:274](../../DEVELOPMENT_PLAN.md#L274), [EVAL-SET-M5-CLOSE.md:61](../../EVAL-SET-M5-CLOSE.md#L61)). Candidate blindness itself was root-fixed rather than accepted: before S5, requests produced 128 candidate pairs against a 425-pair denominator and cobra 1,124 against 9,205 — a hard ceiling; after S5 the `not_candidate` bucket is **zero** ([EVAL-SET-M5-CLOSE.md:45-49](../../EVAL-SET-M5-CLOSE.md#L45), motivation restated at [candidates.rs:208-212](../../../cli/src/dedup/candidates.rs#L208)). The cost was published, not hidden: post-S5 cold `ce clone` at 1.8 s (requests) / 3.6 s (cobra) / 47.1 s (zod), against 24.9 s for zod pre-S5 ([EVAL-SET-M5-CLOSE.md:62-63](../../EVAL-SET-M5-CLOSE.md#L62)).

**Epoch semantics.** A frozen family is pinned to one detector version. Sample rows embed unit keys and sampling is in key-hash order, so re-freezing candidates necessarily breaks the `pool_digests` anchor chain and re-freezing the sample invalidates the five auditors' ground truth — *partial re-freeze does not exist*. Regeneration must re-establish the **whole family together with its audit** under a new epoch ([EVAL-SET-M5-3.md:49-56](../../EVAL-SET-M5-3.md#L49)).

The paired precision gate is **≥ 85%**, scored against the frozen four-source candidate universe with independent audited ground truth, over answered rows only, with an output-volume floor ([DEVELOPMENT_PLAN.md:274](../../DEVELOPMENT_PLAN.md#L274)); the audited run recorded `θ` swept 70..100 with per-cell `wrong` identically 0 and the contract grid point 85 on file ([EVAL-SET-M5-3.md:46-47](../../EVAL-SET-M5-3.md#L46)).

**Current status.** The gate `eval_t3_recall` and its frozen artifacts were retired in the v0.5.0 slimming batch; the full record lives in git history ([EVAL-SET-M5-CLOSE.md:52-53](../../EVAL-SET-M5-CLOSE.md#L52), retirement inventory at [EVAL-SET.md:297](../../EVAL-SET.md#L297)). The three-family universe drift nets and precision regression gates remain live, reading the frozen sample/oracle JSON directly ([EVAL-SET.md:297](../../EVAL-SET.md#L297)).
