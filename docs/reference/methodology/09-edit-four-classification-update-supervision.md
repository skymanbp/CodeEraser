# Edit four-classification (update supervision)

[index](../methodology.md) · [← 08 Split-ROI seam pricing (four legs)](08-split-roi-seam-pricing-four-legs.md) · [→ 10 Score trajectory — the trend slope verdict](10-score-trajectory-the-trend-slope-verdict.md)

Every edit CodeEraser supervises is reduced to four integer counts per file pair: **matched** (unchanged — never enumerated, it is the diff's complement), **novel** (added, no provenance), **moved** (added or removed with provenance on the other side), **deleted** (removed, no destination). The split is `FourClass { added_novel, added_moved, removed_deleted, removed_moved }` ([model.rs:14-19](../../../cli/src/fourclass/model.rs#L14)) — matched lines are exactly the lines the diff did not report, so the four-class ledger is closed by construction over the changed set.

The design intent is recorded in plan §4.3 ([DEVELOPMENT_PLAN.md:107](../../DEVELOPMENT_PLAN.md#L107)): a difftastic-inspired but self-implemented **integer** cost model, from which a cross-file evidence floor of ≥2 lines is *derived* rather than tuned, plus a decided anchor requirement of one ≥19-alnum evidence line ([DEVELOPMENT_PLAN.md:109-112](../../DEVELOPMENT_PLAN.md#L109)).

### Language split

Judgment (which lines correspond) is Haskell; alignment, diff, and symbol attribution are Rust. Nothing text-shaped crosses the wire — only pair indices, line numbers, hashes, and widths ([Wire.hs:9-10](../../../core/app/CE/FourClass/Wire.hs#L9)).

### L1: within-file classification

`classify` hashes each line with `DefaultHasher` ([model.rs:101](../../../cli/src/fourclass/model.rs#L101)), runs a self-contained Myers line diff, then classifies each changed line by content lookup against the opposite side:

```
removed line i is MOVED  iff significant(a[i]) && a[i].trim() ∈ added_sig
added   line j is MOVED  iff significant(b[j]) && b[j].trim() ∈ removed_sig
otherwise removed → deleted, added → novel
```

([model.rs:69](../../../cli/src/fourclass/model.rs#L69); the opposite-side sets are built over trimmed content at [model.rs:130](../../../cli/src/fourclass/model.rs#L130)).

Two properties are load-bearing:

- **Whitespace-insensitive matching.** Comparison is on `.trim()`, matching git's `--color-moved-ws=allow-indentation-change` ([mod.rs:5-8](../../../cli/src/fourclass/mod.rs#L5)).
- **Significance.** `significant(line) = line.chars().any(char::is_alphanumeric)` ([model.rs:118](../../../cli/src/fourclass/model.rs#L118)). Blank and pure-punctuation lines carry no move identity and can never be classified moved — they land in novel/deleted. This is the ground-truth convention (labels-v1), and the same function is the single source for eval tooling.

Sides are marked independently, so `added_moved` and `removed_moved` need not balance ([mod.rs:8-10](../../../cli/src/fourclass/mod.rs#L8)).

Each moved line is attributed to the innermost unit containing it — `owner` picks the minimum-span unit ([units.rs:216-221](../../../cli/src/fourclass/units.rs#L216)). Unit keys are `name/arity` for code ([units.rs:69-74](../../../cli/src/fourclass/units.rs#L69)) and heading text for Markdown ([units.rs:165-187](../../../cli/src/fourclass/units.rs#L165)). Non-function named units (Rust `const_item`, `static_item`, `struct_item`, `enum_item`, `trait_item`, `mod_item`; Python `class_definition`; TS/TSX class/interface/enum declarations; since plan v2.17 L round step 8 also Go `type_spec`/`type_alias` and Haskell `data_type`/`newtype`/`type_synomym`/`class`/`type_family`/`data_family` — the register is the symbols domain too, so a named type form must be a unit to carry a visibility word) are registered only for relocation reporting, not for the M1 function metrics ([kinds.rs:39-61](../../../cli/src/fourclass/kinds.rs#L39)). Rust `impl_item` gets an impl key `impl Foo` / `impl Advisor for Foo` ([units.rs:148-161](../../../cli/src/fourclass/units.rs#L148)) so that methods of different impls are not seen as top-level.

A unit is reported **relocated intact** when it exists on both sides and *every* changed line inside either span is a move: `rm + ad > 0 && moved_of(true, key) == rm && moved_of(false, key) == ad` ([model.rs:173](../../../cli/src/fourclass/model.rs#L173)).

**Diff degradation.** The Myers search is bounded by `MAX_D = 3000` ([diff.rs:20](../../../cli/src/fourclass/diff.rs#L20)); beyond it the trimmed window is reported wholesale changed, which over-counts novel/deleted but never invents a move, and sets `degraded` ([diff.rs:38-45](../../../cli/src/fourclass/diff.rs#L38)). The one-side-empty case (pure creation/deletion) bypasses the bound entirely because its minimal script is exact by construction ([diff.rs:36-40](../../../cli/src/fourclass/diff.rs#L36)).

### The integer cost model

All of it is four constants ([Cost.hs:1-6](../../../core/app/CE/FourClass/Cost.hs#L1) — integers because floats tie-break differently across platforms and the output contract is byte determinism):

| constant | value | source |
|---|---|---|
| `movedCost` (m) — explain a line as moved | `1` | [Cost.hs:21](../../../core/app/CE/FourClass/Cost.hs#L21) |
| `plainCost` (v) — leave a line novel/deleted | `3` | [Cost.hs:25](../../../core/app/CE/FourClass/Cost.hs#L25) |
| `siteCostWithin` — open a relocation site inside one pair | `0` | [Cost.hs:30](../../../core/app/CE/FourClass/Cost.hs#L30) |
| `siteCostCross` — open a site across two pairs | `2` | [Cost.hs:37](../../../core/app/CE/FourClass/Cost.hs#L37) |

A site opens iff explaining its lines as moved strictly beats leaving them plain:

```
siteOpens s n  =  n * movedCost + s  <  n * plainCost
```

([Cost.hs:46-49](../../../core/app/CE/FourClass/Cost.hs#L46)). Ties resolve to *not* opening. The arithmetic rides `Integer`, not machine `Int` ([Cost.hs:48-49](../../../core/app/CE/FourClass/Cost.hs#L48)).

Two consequences, both theorems rather than thresholds:

- `siteCostWithin = 0` ⇒ `1*1 + 0 < 1*3`, so **any single matching line opens a within-file site** — which is exactly L1's unfloored rule ([Cost.hs:27-28](../../../core/app/CE/FourClass/Cost.hs#L27)).
- `siteCostCross = 2` ⇒ a single cross line gives `1*1 + 2 = 3 = 1*3`, a tie, which does not open. So `destFloor`, defined as the least `n` with `siteOpens siteCostCross n` ([Cost.hs:52-55](../../../core/app/CE/FourClass/Cost.hs#L52)), evaluates to **2**. That tie *is* the coincidence rejection ([Cost.hs:32-35](../../../core/app/CE/FourClass/Cost.hs#L32)).

The sensitivity test pins the knob as live: `destFloor == 2` and `not (siteOpens 2 1)` ([Spec.hs:152-153](../../../core/test/Spec.hs#L152)), and perturbing the site cost moves the floor — `s ∈ {0,2,4,6}` ⇒ floor `{1,2,3,4}` ([Spec.hs:154-157](../../../core/test/Spec.hs#L154)).

### Line-evidence floor plus the anchor-line requirement

`destFloor` alone is insufficient in two measured ways, so acceptance of a cross-pair block requires **three** conditions ([Anchor.hs:108-127](../../../core/app/CE/FourClass/Anchor.hs#L108)):

1. **Block start.** `isStart` — position 0 on either side, or unequal predecessor hashes ([Anchor.hs:119-122](../../../core/app/CE/FourClass/Anchor.hs#L119)). Each maximal block is therefore discovered exactly once; interior positions fail the test. Block length `n` is the common prefix of the two tails ([Anchor.hs:123](../../../core/app/CE/FourClass/Anchor.hs#L123)).
2. **Distinct-content floor.** `distinctEvidence = |{ hash(line) : line ∈ evidence }| >= destFloor` ([Anchor.hs:111](../../../core/app/CE/FourClass/Anchor.hs#L111), [Anchor.hs:125](../../../core/app/CE/FourClass/Anchor.hs#L125)). Counting *distinct content values*, not lines: one common line repeated twice is a single piece of evidence, and length alone let `[x,x]` matched against `[x,x]` clear the floor ([Anchor.hs:100-103](../../../core/app/CE/FourClass/Anchor.hs#L100)).
3. **Anchor line.** `anchored = any (\(_,_,w) -> w >= anchorFloor) evidence` with `anchorFloor = 19` ([Anchor.hs:126](../../../core/app/CE/FourClass/Anchor.hs#L126), [Cost.hs:65-66](../../../core/app/CE/FourClass/Cost.hs#L65)). `w` is the line's **alnum width**: alphanumeric characters of the trimmed content, measured by the Rust aligner and shipped as a line fact ([model.rs:126](../../../cli/src/fourclass/model.rs#L126), [Wire.hs:41-44](../../../core/app/CE/FourClass/Wire.hs#L41)).

`anchorFloor` is the one constant that is **decided, not derived**. Its recorded basis ([Cost.hs:58-64](../../../core/app/CE/FourClass/Cost.hs#L58)): in the dual-corpus shadow ablation the invented station's widest anchor measured 16 and the thinnest real anchor 19, so every threshold in 17..19 kills all measured coincidences while keeping every measured real site; 19 is the top of that window. The aggregate form was rejected — `7+16=23` would re-admit the invention. The failing shape it exists to reject: two short distinct lines (`Timeout,` + `TooManyRedirects,`) cleared the old floor on a pure-reformat commit and invented a station ([Anchor.hs:104-107](../../../core/app/CE/FourClass/Anchor.hs#L104)).

**Run structure** is alignment data and is produced in Rust ([Wire.hs:38-42](../../../core/app/CE/FourClass/Wire.hs#L38)). Two leftovers are adjacent iff every line between them is also changed and none of those in-between changed lines is significant ([batch.rs:131-134](../../../cli/src/fourclass/batch.rs#L131)); an unchanged gap breaks a run ([batch.rs:178-180](../../../cli/src/fourclass/batch.rs#L178)), a within-moved line breaks it ([batch.rs:187-190](../../../cli/src/fourclass/batch.rs#L187)), blank/punctuation changed lines bridge it ([batch.rs:183-185](../../../cli/src/fourclass/batch.rs#L183)). Bridging is bounded by `MAX_BRIDGE = 7` ([batch.rs:169](../../../cli/src/fourclass/batch.rs#L169)) — unbounded bridging let two significant lines 1000 punctuation lines apart compress into adjacency. The bound is the maximum observed on the frozen slice; the bridge-width histogram over every leftover run of all 47 commits is `{0:7037, 1:663, 2:411, 3:90, 4:19, 5:17, 6:2, 7:1}` ([batch.rs:164-168](../../../cli/src/fourclass/batch.rs#L164)).

**Work budget.** A hash whose removed × added occurrence product exceeds `bucketCap^2` with `bucketCap = 64` degrades the whole request, all-or-nothing ([Anchor.hs:38-39](../../../core/app/CE/FourClass/Anchor.hs#L38), [Anchor.hs:77-86](../../../core/app/CE/FourClass/Anchor.hs#L77)). The product is computed in `Integer` because machine `Int` is 32-bit on some GHC targets, where two ~50k-occurrence sides would overflow and bypass the budget ([Anchor.hs:73-76](../../../core/app/CE/FourClass/Anchor.hs#L73)). Recorded headroom: largest measured self-slice bucket 9 ([Anchor.hs:37](../../../core/app/CE/FourClass/Anchor.hs#L37)).

**Determinism** is structural, not enforced: no exclusivity, no greedy claiming, no tie-break, because de-duplication commits are many-to-one and accepted blocks are a union of independently derived sets with no dependence on iteration order ([Anchor.hs:6-11](../../../core/app/CE/FourClass/Anchor.hs#L6)). Blocks are sorted on `(bFromPair, bFromLines, bToPair, bToLines)` before emission ([Provenance.hs:38-39](../../../core/app/CE/FourClass/Provenance.hs#L38)).

### Asymmetric extension: phases 2 and 3

Both phases follow from the cost model: adding a line to an already-open site costs `movedCost < plainCost` with no new site cost, so it is always profitable ([Provenance.hs:2-4](../../../core/app/CE/FourClass/Provenance.hs#L2)).

- **Phase 2 (addition side, run-scoped).** An unclaimed added line is marked moved-in iff it sits in a contiguous added run that already contains an anchored line, *and* its hash occurs among the leftover removals of a pair with an established block edge into this pair ([Provenance.hs:59-88](../../../core/app/CE/FourClass/Provenance.hs#L59)). This recovers one-line tails of proven relocations without licensing file-wide claims.
- **Phase 3 (removal side, asymmetric).** A leftover removed line whose content landed at any marked-in line of a *different* pair is moved-out ([Provenance.hs:96-104](../../../core/app/CE/FourClass/Provenance.hs#L96)) — no run scoping, no site membership required.

The asymmetry is the product thesis, stated at [Provenance.hs:5-11](../../../core/app/CE/FourClass/Provenance.hs#L5): on the removal side, "the content left its home" is itself provenance (bulk removal of copies is the normal shape of a de-duplication refactor); on the addition side, a fresh line duplicating removed content is **duplication** — the signal the product exists to catch — so additions require site or edge evidence.

Only anchored block lines appear in `blocks`; lines admitted by extension or source attribution appear in `moved` but not in `blocks` — they are a relocation's tail, not its evidence ([Wire.hs:73-76](../../../core/app/CE/FourClass/Wire.hs#L73)).

### Declaration-level relocation (proto 7.1.0)

Rust measures declaration keys that occur exactly once on one side of a file pair and not on the other. It sends paired optional `declRem` / `declAdd` rows `[fnv1a(key), kind, start, end]`; names remain local ([decls.rs:76](../../../cli/src/fourclass/decls.rs#L76)). The existing capability stays `fourclass/2`.

Haskell accepts `p → q` for a vanished key of the same kind appearing in exactly one other pair, when the leftover content inside the two declaration spans shares at least `declFloor` distinct hashes ([Decl.hs:71](../../../core/app/CE/FourClass/Decl.hs#L71)). Multiple sources may feed that destination; two destinations refuse the key. Every measured pair is sent, including pairs without leftover lines, because they may contain a competing destination.

The declaration key supplies provenance identity and pays the cross-site cost: `declCredit = siteCostCross`. Thus `declFloor = least n with siteOpens (siteCostCross - declCredit) n = 1` ([Cost.hs:76](../../../core/app/CE/FourClass/Cost.hs#L76)). One shared line opens an edge; zero does not. Setting the credit to zero restores `destFloor = 2`.

The key replaces the line anchor for this stage. The four identical body lines of `CLASSES` have a widest alphanumeric width of 14, below `anchorFloor = 19`; they can identify content once the declaration identifies its destination. The known requests reformat coincidence has no vanished declaration and gains no declaration edge.

The stage only appends relocations with `lines = 0`; line classifications, blocks, suspicions and scores do not change. A line relocation that already names either end wins, so the same edge is reported once ([edges.rs:40](../../../cli/src/fourclass/batch/edges.rs#L40)). The frozen L2 documents and their seven line gates remain unchanged.

Boundary: tables must arrive together and cover every pair ([Decl.hs:49](../../../core/app/CE/FourClass/Decl.hs#L49)). Absent tables mean unmeasured; empty tables mean measured with no candidates. Above `declCap = 65536` declaration rows across both tables, the core refuses the entire table with `unitEdges: []` and `unitEdgesDropped: true` ([Decl.hs:27](../../../core/app/CE/FourClass/Decl.hs#L27)). Older replies without `unitEdges` and over-cap replies add no declaration rows.

The [measured continuation ledger](../../EVAL-SET-M5-3.md#声明级搬迁o48) records each of the six previously uncovered pairs. The adapted `~out_dir` retains its key but no identical significant content; name/kind plus content, T2 equality and structural-skeleton equality do not identify it. A similarity rule would need its own definition and evaluation.

### Stacking suspicion

One M4 judgment rule ships, intent-free by design ([Verdict.hs:1-14](../../../core/app/CE/FourClass/Verdict.hs#L1)). It fires only on a conjunction of three signals, two of them one line since proto 7.0.0 ([Verdict.hs:42-48](../../../core/app/CE/FourClass/Verdict.hs#L42)):

```
inDup   >= stackingNovelFloor          -- ≥ 20 novel lines INSIDE a newly duplicated unit's span
&& deleted * stackingRatio < novel     -- deletions under novel/10
```

with `stackingNovelFloor = 20` ([Verdict.hs:25-26](../../../core/app/CE/FourClass/Verdict.hs#L25)) and `stackingRatio = 10` ([Verdict.hs:30-31](../../../core/app/CE/FourClass/Verdict.hs#L30)). Rationale as recorded: below the floor even a true duplicate is a nit; editing-in-place removes roughly what it adds, while stacking removes almost nothing ([Verdict.hs:23-31](../../../core/app/CE/FourClass/Verdict.hs#L23)). `inDup` is the count of the pair's novel lines that fall inside any shipped span ([Verdict.hs:46](../../../core/app/CE/FourClass/Verdict.hs#L46)): a fresh copy written beside the old one puts its lines there, while twenty novel lines elsewhere in a file that happens to gain a duplicate key are ordinary editing — the 6.x rule joined the two signals and fired on that shape (plan v2.29 step 8, O47; [Verdict.hs:34-41](../../../core/app/CE/FourClass/Verdict.hs#L34)). The removal ratio still reads the whole edit.

`novel` and `deleted` here are **post-reclassification** — the novel LINES and the deleted count — supplied by the caller, not the raw leftover list lengths, which would overcount ([Verdict.hs:33-36](../../../core/app/CE/FourClass/Verdict.hs#L33)). They are the sent-leftover lines not present in the phase marks, `leftLines side marks p`, kept as lines on the addition side so the rule can place them and counted on the removal side ([Provenance.hs:30](../../../core/app/CE/FourClass/Provenance.hs#L30), [Provenance.hs:34-35](../../../core/app/CE/FourClass/Provenance.hs#L34)).

**The duplication evidence** (`dupSpans`) is one `[fnv1a(key), start, end]` row per after-side occurrence of a newly duplicated unit key, computed in Rust and shipped as hashes and line numbers only, since symbol knowledge stays on the Rust side per ADR-002 ([stacking.rs:1-10](../../../cli/src/fourclass/stacking.rs#L1), [Wire.hs:49](../../../core/app/CE/FourClass/Wire.hs#L49), [Wire.hs:66](../../../core/app/CE/FourClass/Wire.hs#L66)). A key qualifies iff its after-side count rises to ≥2 *and* strictly exceeds its before-side count: `spans.len() >= 2 && spans.len() > before.get(k)` ([stacking.rs:33](../../../cli/src/fourclass/stacking.rs#L33)); a span with `start < 1` or `end < start` is refused by name at the envelope, pair index named ([FourClass.hs:41-42](../../../core/app/CE/FourClass.hs#L41)). Three scoping exclusions, each a measured false-positive source ([stacking.rs:58-60](../../../cli/src/fourclass/stacking.rs#L58)):

- **top-level only** — a unit strictly span-contained in another is excluded ([stacking.rs:52-56](../../../cli/src/fourclass/stacking.rs#L52)), because a method nested in two different classes shares its flat key legitimately;
- **`(anonymous)` keys excluded** — an anonymous closure has no stacking identity;
- **`impl ` keys excluded** — impl blocks are containers so methods are not top-level, never stacking identities themselves; a type's inherent and trait impls, or split inherent impls, coexist in normal Rust.

Recorded FPR effect of this scoping on the real-edit corpus: `contracts/eval/fpr-fourclass-v1.json` flagged 8/600 before, 0/600 after ([stacking.rs:19-24](../../../cli/src/fourclass/stacking.rs#L19); corroborated at [EVAL-SET.md:138](../../EVAL-SET.md#L138)).

The output is `(pair index, "stacking")` ([Verdict.hs:44](../../../core/app/CE/FourClass/Verdict.hs#L44)); the report renders it as `{"file": …, "kind": …}` ([session.rs:153-157](../../../cli/src/fourclass/session.rs#L153)).

The other §4.3 rules — novel-vs-repository similarity as duplicate-implementation suspicion, and MinHash paragraph similarity as restatement suspicion ([DEVELOPMENT_PLAN.md:121-125](../../DEVELOPMENT_PLAN.md#L121)) — are not implemented in this module; `CE.FourClass.Verdict` exports exactly one rule ([Verdict.hs:1-2](../../../core/app/CE/FourClass/Verdict.hs#L1)).

### The L0 / L1 / L2 fallback ladder

Plan §4.3 B3c defines three rungs, each the control group for the next ([DEVELOPMENT_PLAN.md:114-117](../../DEVELOPMENT_PLAN.md#L114)):

| rung | definition | measured on the eval corpus |
|---|---|---|
| **L0** | `git diff --numstat -M -C --find-copies-harder`, zero self-implementation | moved recall **0/62** ([EVAL-SET.md:56](../../EVAL-SET.md#L56)); the `--color-moved=plain` reference reaches 62/62 recall but 62/125 precision, 63 blank-line artifacts ([EVAL-SET.md:57](../../EVAL-SET.md#L57)) |
| **L1** | L0 + function-boundary alignment (tree-sitter symbol table) | moved recall **62/62**, precision **100%**, 195/200 sample-exact ([EVAL-SET.md:66](../../EVAL-SET.md#L66)); on the whole-commit slice, cross-file recall **0/547** — a structural blind spot ([EVAL-SET.md:91-92](../../EVAL-SET.md#L91)) |
| **L2** | cross-file provenance judgment (the integer cost model above); AST units used for attribution and the relocation register, with a declaration stage for units below the line floor | cross-file recall **547/547** (366 out + 181 in), misses = 0; zero false cross-predictions on commits with no cross-move ground truth ([EVAL-SET.md:112](../../EVAL-SET.md#L112)) |

L2 must prove incremental gain over L1 or the ladder falls back to L1 ([DEVELOPMENT_PLAN.md:117](../../DEVELOPMENT_PLAN.md#L117)).

**L1 is the IR producer, not a modified engine.** L2 runs L1 per pair unchanged, ships only the leftovers (significant lines L1 called novel/deleted) as `[line, fnv1a(trim), alnum_width]` grouped into runs, and applies a monotone delta ([batch.rs:1-6](../../../cli/src/fourclass/batch.rs#L1), [batch.rs:142-160](../../../cli/src/fourclass/batch.rs#L142)). Single-pair batches with no link are bitwise L1 ([batch.rs:5-6](../../../cli/src/fourclass/batch.rs#L5)).

The delta is monotone in one direction only: `removed_deleted → removed_moved`, `added_novel → added_moved` ([Wire.hs:84-86](../../../core/app/CE/FourClass/Wire.hs#L84), [delta.rs:61-66](../../../cli/src/fourclass/batch/delta.rs#L61)). L2 can therefore only reclassify plain lines as moved, never the reverse.

**Every fallback returns the pure-L1 result with a named reason** ([batch.rs:8-10](../../../cli/src/fourclass/batch.rs#L8)):

| condition | `degraded` reason | `link_failed` |
|---|---|---|
| no core link | `"no_link"` ([batch.rs:58](../../../cli/src/fourclass/batch.rs#L58)) | false |
| link alive, capability `fourclass/2` absent | `"no_capability"` ([batch.rs:60-62](../../../cli/src/fourclass/batch.rs#L60)) | false |
| no leftovers to ask about | `None` — the pass ran vacuously ([batch.rs:64-69](../../../cli/src/fourclass/batch.rs#L64)) | false |
| transport error | the error string ([batch.rs:71](../../../cli/src/fourclass/batch.rs#L71)) | **true** |
| core answered with a `reason` (e.g. `bucket_cap`) | that reason ([batch.rs:100-102](../../../cli/src/fourclass/batch.rs#L100)) | false |
| core answered, delta failed validation | the merge error ([batch.rs:73](../../../cli/src/fourclass/batch.rs#L73)) | false |

`link_failed` is stated rather than inferred because the restart budget keys on it ([batch.rs:49](../../../cli/src/fourclass/batch.rs#L49)). A degraded reply may carry partial blocks; the reason is checked **before** merge on purpose, since applying them would be partial L2 behind a flag ([batch.rs:98-102](../../../cli/src/fourclass/batch.rs#L98)). Capability `fourclass/2` is the anchor-width request shape (proto 2.0.0); a client probing `fourclass/1` sees absence and degrades to L1 loudly rather than sending the un-parseable two-element shape ([Handshake.hs:30-32](../../../core/app/CE/Handshake.hs#L30), [Handshake.hs:30-32](../../../core/app/CE/Handshake.hs#L30)).

### Boundary checks

The reply is an answer, not an authority ([delta.rs:4-5](../../../cli/src/fourclass/batch/delta.rs#L4)):

- **Merge is all-or-nothing.** `merge` works on a copy; an in-place form leaked a half-merged result through the error path as the claimed "pure L1 fallback" ([delta.rs:18-20](../../../cli/src/fourclass/batch/delta.rs#L18), [batch.rs:103-105](../../../cli/src/fourclass/batch.rs#L103)).
- **Each returned line is consumed once** from a per-side unconsumed set; a double-listed line is a named error, not a `usize` underflow that produced ~18e18 "deleted" lines ([delta.rs:13-16](../../../cli/src/fourclass/batch/delta.rs#L13), [delta.rs:58-59](../../../cli/src/fourclass/batch/delta.rs#L58)).
- **Wire indices are bounds-checked**, not used as slice subscripts, in both the merge and the report — the report path runs inside the daemon, which has no `catch_unwind` ([delta.rs:100-102](../../../cli/src/fourclass/batch/delta.rs#L100), [session.rs:131-141](../../../cli/src/fourclass/session.rs#L131)).
- **The core checks pair uniqueness and within-first**, alongside declaration and duplication-span validation, at its boundary ([FourClass.hs:35-48](../../../core/app/CE/FourClass.hs#L35)): no duplicate pair index (Anchor's run maps key on `(pair, run)`, so `M.fromList` would silently drop an earlier duplicate's runs), and **within-first** — no leftover added hash of a pair may occur among that same pair's leftover removed hashes. Within-first is L1's within-file consumption rule seen from the judgment side; verifying its consequence turns a cross-language assumption into a checked contract ([FourClass.hs:3-6](../../../core/app/CE/FourClass.hs#L3)).
- **Blocks are unit-attributed line by line**, not head-line only: one block can span several units, and head-line attribution left 7 of 35 registered units unnamed ([delta.rs:89-94](../../../cli/src/fourclass/batch/delta.rs#L89)).

### Session scope

The session's file pairs come from `git diff --name-status -z -M -C HEAD` ([session.rs:22-24](../../../cli/src/fourclass/session.rs#L22)). A copy record (`C`) carries two paths and maps to an **added** file — the source survives and the destination is new material, which is precisely the duplication signal ([session.rs:101-104](../../../cli/src/fourclass/session.rs#L101)); consuming only one of its two tokens desynchronizes the whole stream. Pairs are filtered by `Lang::judged_path` — the scan-only arm is size-gated and never four-classified ([session.rs:111-115](../../../cli/src/fourclass/session.rs#L111)).

Cross-file relocations remain **informational**: no multi-file FPR instrument exists (R-L2-4 — ruled not built at the v2.22 close-out), and claiming a move where there is duplication would hide duplication inside a health signal, so no deny path leans on this report ([session.rs:3-6](../../../cli/src/fourclass/session.rs#L3)).

### Constants not found

Plan §4.3 refers to a similarity threshold for the duplicate-implementation rule and to MinHash paragraph similarity ([DEVELOPMENT_PLAN.md:121-125](../../DEVELOPMENT_PLAN.md#L121)); no such constant exists anywhere in `core/app/CE/FourClass/` or `cli/src/fourclass/`. Those rules live outside this module (the `clone`/`docdup` families), and their thresholds are not documented here.
