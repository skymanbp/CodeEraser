# Edit four-classification (update supervision)

[index](../methodology.md) · [← 08 Split-ROI seam pricing (four legs)](08-split-roi-seam-pricing-four-legs.md) · [→ 10 Score trajectory — the trend slope verdict](10-score-trajectory-the-trend-slope-verdict.md)

Every edit CodeEraser supervises is reduced to four integer counts per file pair: **matched** (unchanged — never enumerated, it is the diff's complement), **novel** (added, no provenance), **moved** (added or removed with provenance on the other side), **deleted** (removed, no destination). The split is `FourClass { added_novel, added_moved, removed_deleted, removed_moved }` ([mod.rs:29](../../../cli/src/fourclass/mod.rs#L29)) — matched lines are exactly the lines the diff did not report, so the four-class ledger is closed by construction over the changed set.

The design intent is recorded in plan §4.3 ([DEVELOPMENT_PLAN.md:107](../../DEVELOPMENT_PLAN.md#L107)): a difftastic-inspired but self-implemented **integer** cost model, from which a cross-file evidence floor of ≥2 lines is *derived* rather than tuned, plus a decided anchor requirement of one ≥19-alnum evidence line ([DEVELOPMENT_PLAN.md:109-112](../../DEVELOPMENT_PLAN.md#L109)).

### Language split

Judgment (which lines correspond) is Haskell; alignment, diff, and symbol attribution are Rust. Nothing text-shaped crosses the wire — only pair indices, line numbers, hashes, and widths ([Wire.hs:9-10](../../../core/app/CE/FourClass/Wire.hs#L9)).

### L1: within-file classification

`classify` hashes each line with `DefaultHasher` ([mod.rs:110-120](../../../cli/src/fourclass/mod.rs#L110)), runs a self-contained Myers line diff, then classifies each changed line by content lookup against the opposite side:

```
removed line i is MOVED  iff significant(a[i]) && a[i].trim() ∈ added_sig
added   line j is MOVED  iff significant(b[j]) && b[j].trim() ∈ removed_sig
otherwise removed → deleted, added → novel
```

([mod.rs:80-95](../../../cli/src/fourclass/mod.rs#L80); the opposite-side sets are built over trimmed content at [mod.rs:139-145](../../../cli/src/fourclass/mod.rs#L139)).

Two properties are load-bearing:

- **Whitespace-insensitive matching.** Comparison is on `.trim()`, matching git's `--color-moved-ws=allow-indentation-change` ([mod.rs:5-8](../../../cli/src/fourclass/mod.rs#L5)).
- **Significance.** `significant(line) = line.chars().any(char::is_alphanumeric)` ([mod.rs:127-129](../../../cli/src/fourclass/mod.rs#L127)). Blank and pure-punctuation lines carry no move identity and can never be classified moved — they land in novel/deleted. This is the ground-truth convention (labels-v1), and the same function is the single source for eval tooling.

Sides are marked independently, so `added_moved` and `removed_moved` need not balance ([mod.rs:8-10](../../../cli/src/fourclass/mod.rs#L8)).

Each moved line is attributed to the innermost unit containing it — `owner` picks the minimum-span unit ([units.rs:162-167](../../../cli/src/fourclass/units.rs#L162)). Unit keys are `name/arity` for code ([units.rs:39-43](../../../cli/src/fourclass/units.rs#L39)) and heading text for Markdown ([units.rs:114-135](../../../cli/src/fourclass/units.rs#L114)). Non-function named units (Rust `const_item`, `static_item`, `struct_item`, `enum_item`, `trait_item`, `mod_item`; Python `class_definition`; TS/TSX class/interface/enum declarations) are registered only for relocation reporting, not for the M1 function metrics ([kinds.rs:14-36](../../../cli/src/fourclass/kinds.rs#L14)). Rust `impl_item` gets a typed key `impl Foo` / `impl Advisor for Foo` ([kinds.rs:46-51](../../../cli/src/fourclass/kinds.rs#L46), [units.rs:95-110](../../../cli/src/fourclass/units.rs#L95)) so that methods of different impls are not seen as top-level.

A unit is reported **relocated intact** when it exists on both sides and *every* changed line inside either span is a move: `rm + ad > 0 && moved_of(true, key) == rm && moved_of(false, key) == ad` ([mod.rs:182](../../../cli/src/fourclass/mod.rs#L182)).

**Diff degradation.** The Myers search is bounded by `MAX_D = 3000` ([diff.rs:20](../../../cli/src/fourclass/diff.rs#L20)); beyond it the trimmed window is reported wholesale changed, which over-counts novel/deleted but never invents a move, and sets `degraded` ([diff.rs:38-45](../../../cli/src/fourclass/diff.rs#L38)). The one-side-empty case (pure creation/deletion) bypasses the bound entirely because its minimal script is exact by construction ([diff.rs:36-40](../../../cli/src/fourclass/diff.rs#L36)).

### The integer cost model

All of it is four constants ([Cost.hs:1-6](../../../core/app/CE/FourClass/Cost.hs#L1) — integers because floats tie-break differently across platforms and the output contract is byte determinism):

| constant | value | source |
|---|---|---|
| `movedCost` (m) — explain a line as moved | `1` | [Cost.hs:19](../../../core/app/CE/FourClass/Cost.hs#L19) |
| `plainCost` (v) — leave a line novel/deleted | `3` | [Cost.hs:22](../../../core/app/CE/FourClass/Cost.hs#L22) |
| `siteCostWithin` — open a relocation site inside one pair | `0` | [Cost.hs:28](../../../core/app/CE/FourClass/Cost.hs#L28) |
| `siteCostCross` — open a site across two pairs | `2` | [Cost.hs:35](../../../core/app/CE/FourClass/Cost.hs#L35) |

A site opens iff explaining its lines as moved strictly beats leaving them plain:

```
siteOpens s n  =  n * movedCost + s  <  n * plainCost
```

([Cost.hs:44-47](../../../core/app/CE/FourClass/Cost.hs#L44)). Ties resolve to *not* opening. The arithmetic rides `Integer`, not machine `Int` ([Cost.hs:46-47](../../../core/app/CE/FourClass/Cost.hs#L46)).

Two consequences, both theorems rather than thresholds:

- `siteCostWithin = 0` ⇒ `1*1 + 0 < 1*3`, so **any single matching line opens a within-file site** — which is exactly L1's unfloored rule ([Cost.hs:25-26](../../../core/app/CE/FourClass/Cost.hs#L25)).
- `siteCostCross = 2` ⇒ a single cross line gives `1*1 + 2 = 3 = 1*3`, a tie, which does not open. So `destFloor`, defined as the least `n` with `siteOpens siteCostCross n` ([Cost.hs:50-53](../../../core/app/CE/FourClass/Cost.hs#L50)), evaluates to **2**. That tie *is* the coincidence rejection ([Cost.hs:30-33](../../../core/app/CE/FourClass/Cost.hs#L30)).

The sensitivity test pins the knob as live: `destFloor == 2` and `not (siteOpens 2 1)` ([Spec.hs:122-123](../../../core/test/Spec.hs#L122)), and perturbing the site cost moves the floor — `s ∈ {0,2,4,6}` ⇒ floor `{1,2,3,4}` ([Spec.hs:127](../../../core/test/Spec.hs#L127)).

### Line-evidence floor plus the anchor-line requirement

`destFloor` alone is insufficient in two measured ways, so acceptance of a cross-pair block requires **three** conditions ([Anchor.hs:108-127](../../../core/app/CE/FourClass/Anchor.hs#L108)):

1. **Block start.** `isStart` — position 0 on either side, or unequal predecessor hashes ([Anchor.hs:119-122](../../../core/app/CE/FourClass/Anchor.hs#L119)). Each maximal block is therefore discovered exactly once; interior positions fail the test. Block length `n` is the common prefix of the two tails ([Anchor.hs:123](../../../core/app/CE/FourClass/Anchor.hs#L123)).
2. **Distinct-content floor.** `distinctEvidence = |{ hash(line) : line ∈ evidence }| >= destFloor` ([Anchor.hs:111](../../../core/app/CE/FourClass/Anchor.hs#L111), [Anchor.hs:125](../../../core/app/CE/FourClass/Anchor.hs#L125)). Counting *distinct content values*, not lines: one common line repeated twice is a single piece of evidence, and length alone let `[x,x]` matched against `[x,x]` clear the floor ([Anchor.hs:100-103](../../../core/app/CE/FourClass/Anchor.hs#L100)).
3. **Anchor line.** `anchored = any (\(_,_,w) -> w >= anchorFloor) evidence` with `anchorFloor = 19` ([Anchor.hs:126](../../../core/app/CE/FourClass/Anchor.hs#L126), [Cost.hs:63-64](../../../core/app/CE/FourClass/Cost.hs#L63)). `w` is the line's **alnum width**: alphanumeric characters of the trimmed content, measured by the Rust aligner and shipped as a line fact ([mod.rs:135-137](../../../cli/src/fourclass/mod.rs#L135), [Wire.hs:36-39](../../../core/app/CE/FourClass/Wire.hs#L36)).

`anchorFloor` is the one constant that is **decided, not derived**. Its recorded basis ([Cost.hs:56-62](../../../core/app/CE/FourClass/Cost.hs#L56)): in the dual-corpus shadow ablation the invented station's widest anchor measured 16 and the thinnest real anchor 19, so every threshold in 17..19 kills all measured coincidences while keeping every measured real site; 19 is the top of that window. The aggregate form was rejected — `7+16=23` would re-admit the invention. The failing shape it exists to reject: two short distinct lines (`Timeout,` + `TooManyRedirects,`) cleared the old floor on a pure-reformat commit and invented a station ([Anchor.hs:104-107](../../../core/app/CE/FourClass/Anchor.hs#L104)).

**Run structure** is alignment data and is produced in Rust ([Wire.hs:33-37](../../../core/app/CE/FourClass/Wire.hs#L33)). Two leftovers are adjacent iff every line between them is also changed and none of those in-between changed lines is significant ([batch.rs:117-120](../../../cli/src/fourclass/batch.rs#L117)); an unchanged gap breaks a run ([batch.rs:164-166](../../../cli/src/fourclass/batch.rs#L164)), a within-moved line breaks it ([batch.rs:173-176](../../../cli/src/fourclass/batch.rs#L173)), blank/punctuation changed lines bridge it ([batch.rs:169-171](../../../cli/src/fourclass/batch.rs#L169)). Bridging is bounded by `MAX_BRIDGE = 7` ([batch.rs:155](../../../cli/src/fourclass/batch.rs#L155)) — unbounded bridging let two significant lines 1000 punctuation lines apart compress into adjacency. The bound is the maximum observed on the frozen slice; the bridge-width histogram over every leftover run of all 47 commits is `{0:7037, 1:663, 2:411, 3:90, 4:19, 5:17, 6:2, 7:1}` ([batch.rs:150-154](../../../cli/src/fourclass/batch.rs#L150)).

**Work budget.** A hash whose removed × added occurrence product exceeds `bucketCap^2` with `bucketCap = 64` degrades the whole request, all-or-nothing ([Anchor.hs:38-39](../../../core/app/CE/FourClass/Anchor.hs#L38), [Anchor.hs:77-86](../../../core/app/CE/FourClass/Anchor.hs#L77)). The product is computed in `Integer` because machine `Int` is 32-bit on some GHC targets, where two ~50k-occurrence sides would overflow and bypass the budget ([Anchor.hs:73-76](../../../core/app/CE/FourClass/Anchor.hs#L73)). Recorded headroom: largest measured self-slice bucket 9 ([Anchor.hs:37](../../../core/app/CE/FourClass/Anchor.hs#L37)).

**Determinism** is structural, not enforced: no exclusivity, no greedy claiming, no tie-break, because de-duplication commits are many-to-one and accepted blocks are a union of independently derived sets with no dependence on iteration order ([Anchor.hs:6-11](../../../core/app/CE/FourClass/Anchor.hs#L6)). Blocks are sorted on `(bFromPair, bFromLines, bToPair, bToLines)` before emission ([Provenance.hs:31-32](../../../core/app/CE/FourClass/Provenance.hs#L31)).

### Asymmetric extension: phases 2 and 3

Both phases follow from the cost model: adding a line to an already-open site costs `movedCost < plainCost` with no new site cost, so it is always profitable ([Provenance.hs:2-4](../../../core/app/CE/FourClass/Provenance.hs#L2)).

- **Phase 2 (addition side, run-scoped).** An unclaimed added line is marked moved-in iff it sits in a contiguous added run that already contains an anchored line, *and* its hash occurs among the leftover removals of a pair with an established block edge into this pair ([Provenance.hs:52-81](../../../core/app/CE/FourClass/Provenance.hs#L52)). This recovers one-line tails of proven relocations without licensing file-wide claims.
- **Phase 3 (removal side, asymmetric).** A leftover removed line whose content landed at any marked-in line of a *different* pair is moved-out ([Provenance.hs:89-97](../../../core/app/CE/FourClass/Provenance.hs#L89)) — no run scoping, no site membership required.

The asymmetry is the product thesis, stated at [Provenance.hs:5-11](../../../core/app/CE/FourClass/Provenance.hs#L5): on the removal side, "the content left its home" is itself provenance (bulk removal of copies is the normal shape of a de-duplication refactor); on the addition side, a fresh line duplicating removed content is **duplication** — the signal the product exists to catch — so additions require site or edge evidence.

Only anchored block lines appear in `blocks`; lines admitted by extension or source attribution appear in `moved` but not in `blocks` — they are a relocation's tail, not its evidence ([Wire.hs:57-60](../../../core/app/CE/FourClass/Wire.hs#L57)).

### Stacking suspicion

One M4 judgment rule ships, intent-free by design ([Verdict.hs:1-12](../../../core/app/CE/FourClass/Verdict.hs#L1)). It fires only on a conjunction of three signals ([Verdict.hs:34-41](../../../core/app/CE/FourClass/Verdict.hs#L34)):

```
not (null (pDup p))                    -- a unit key newly duplicated on the after side
&& novel   >= stackingNovelFloor       -- 20
&& deleted * stackingRatio < novel     -- deletions under novel/10
```

with `stackingNovelFloor = 20` ([Verdict.hs:24](../../../core/app/CE/FourClass/Verdict.hs#L24)) and `stackingRatio = 10` ([Verdict.hs:29](../../../core/app/CE/FourClass/Verdict.hs#L29)). Rationale as recorded: below the floor even a true duplicate is a nit; editing-in-place removes roughly what it adds, while stacking removes almost nothing ([Verdict.hs:21-29](../../../core/app/CE/FourClass/Verdict.hs#L21)).

`novel` and `deleted` here are **post-reclassification** counts supplied by the caller, not the raw leftover list lengths, which would overcount ([Verdict.hs:32-33](../../../core/app/CE/FourClass/Verdict.hs#L32)). They are computed as sent-leftover lines not present in the phase marks: `sigLeft side marks p` ([Provenance.hs:26-27](../../../core/app/CE/FourClass/Provenance.hs#L26)).

**The duplication evidence** (`pDup`) is unit-key hashes, computed in Rust and shipped as `fnv1a` hashes only, since symbol knowledge stays on the Rust side per ADR-002 ([stacking.rs:1-4](../../../cli/src/fourclass/stacking.rs#L1), [Wire.hs:45-47](../../../core/app/CE/FourClass/Wire.hs#L45)). A key qualifies iff its after-side count rises to ≥2 *and* strictly exceeds its before-side count: `*n >= 2 && *n > before.get(k)` ([stacking.rs:43](../../../cli/src/fourclass/stacking.rs#L43)). Three scoping exclusions, each a measured false-positive source ([stacking.rs:32-34](../../../cli/src/fourclass/stacking.rs#L32)):

- **top-level only** — a unit strictly span-contained in another is excluded ([stacking.rs:26-30](../../../cli/src/fourclass/stacking.rs#L26)), because a method nested in two different classes shares its flat key legitimately;
- **`(anonymous)` keys excluded** — an anonymous closure has no stacking identity;
- **`impl ` keys excluded** — impl blocks are containers so methods are not top-level, never stacking identities themselves; a type's inherent and trait impls, or split inherent impls, coexist in normal Rust.

Recorded FPR effect of this scoping on the real-edit corpus: `contracts/eval/fpr-fourclass-v1.json` flagged 8/600 before, 0/600 after ([stacking.rs:16-21](../../../cli/src/fourclass/stacking.rs#L16); corroborated at [EVAL-SET.md:138](../../EVAL-SET.md#L138)).

Note the rule checks only that *some* unit was newly duplicated — it does not verify that the novel mass sits inside that unit. The output is `(pair index, "stacking")` ([Verdict.hs:35](../../../core/app/CE/FourClass/Verdict.hs#L35)); the report renders it as `{"file": …, "kind": …}` ([session.rs:135-139](../../../cli/src/fourclass/session.rs#L135)).

The other §4.3 rules — novel-vs-repository similarity as duplicate-implementation suspicion, and MinHash paragraph similarity as restatement suspicion ([DEVELOPMENT_PLAN.md:121-125](../../DEVELOPMENT_PLAN.md#L121)) — are not implemented in this module; `CE.FourClass.Verdict` exports exactly one rule ([Verdict.hs:1-2](../../../core/app/CE/FourClass/Verdict.hs#L1)).

### The L0 / L1 / L2 fallback ladder

Plan §4.3 B3c defines three rungs, each the control group for the next ([DEVELOPMENT_PLAN.md:114-117](../../DEVELOPMENT_PLAN.md#L114)):

| rung | definition | measured on the eval corpus |
|---|---|---|
| **L0** | `git diff --numstat -M -C --find-copies-harder`, zero self-implementation | moved recall **0/62** ([EVAL-SET.md:56](../../EVAL-SET.md#L56)); the `--color-moved=plain` reference reaches 62/62 recall but 62/125 precision, 63 blank-line artifacts ([EVAL-SET.md:57](../../EVAL-SET.md#L57)) |
| **L1** | L0 + function-boundary alignment (tree-sitter symbol table) | moved recall **62/62**, precision **100%**, 195/200 sample-exact ([EVAL-SET.md:66](../../EVAL-SET.md#L66)); on the whole-commit slice, cross-file recall **0/547** — a structural blind spot ([EVAL-SET.md:91-92](../../EVAL-SET.md#L91)) |
| **L2** | cross-file provenance judgment (the integer cost model above); AST units used for attribution and the relocation register | cross-file recall **547/547** (366 out + 181 in), misses = 0; zero false cross-predictions on commits with no cross-move ground truth ([EVAL-SET.md:112](../../EVAL-SET.md#L112)) |

L2 must prove incremental gain over L1 or the ladder falls back to L1 ([DEVELOPMENT_PLAN.md:117](../../DEVELOPMENT_PLAN.md#L117)).

**L1 is the IR producer, not a modified engine.** L2 runs L1 per pair unchanged, ships only the leftovers (significant lines L1 called novel/deleted) as `[line, fnv1a(trim), alnum_width]` grouped into runs, and applies a monotone delta ([batch.rs:1-6](../../../cli/src/fourclass/batch.rs#L1), [batch.rs:128-146](../../../cli/src/fourclass/batch.rs#L128)). Single-pair batches with no link are bitwise L1 ([batch.rs:5-6](../../../cli/src/fourclass/batch.rs#L5)).

The delta is monotone in one direction only: `removed_deleted → removed_moved`, `added_novel → added_moved` ([Wire.hs:68-70](../../../core/app/CE/FourClass/Wire.hs#L68), [delta.rs:60-66](../../../cli/src/fourclass/batch/delta.rs#L60)). L2 can therefore only reclassify plain lines as moved, never the reverse.

**Every fallback returns the pure-L1 result with a named reason** ([batch.rs:8-10](../../../cli/src/fourclass/batch.rs#L8)):

| condition | `degraded` reason | `link_failed` |
|---|---|---|
| no core link | `"no_link"` ([batch.rs:64](../../../cli/src/fourclass/batch.rs#L64)) | false |
| link alive, capability `fourclass/2` absent | `"no_capability"` ([batch.rs:66-68](../../../cli/src/fourclass/batch.rs#L66)) | false |
| no leftovers to ask about | `None` — the pass ran vacuously ([batch.rs:70-75](../../../cli/src/fourclass/batch.rs#L70)) | false |
| transport error | the error string ([batch.rs:77](../../../cli/src/fourclass/batch.rs#L77)) | **true** |
| core answered with a `reason` (e.g. `bucket_cap`) | that reason ([batch.rs:83-84](../../../cli/src/fourclass/batch.rs#L83)) | false |
| core answered, delta failed validation | the merge error ([batch.rs:89](../../../cli/src/fourclass/batch.rs#L89)) | false |

`link_failed` is stated rather than inferred because the restart budget keys on it ([batch.rs:48](../../../cli/src/fourclass/batch.rs#L48)). A degraded reply may carry partial blocks; the reason is checked **before** merge on purpose, since applying them would be partial L2 behind a flag ([batch.rs:79-84](../../../cli/src/fourclass/batch.rs#L79)). Capability `fourclass/2` is the anchor-width request shape (proto 2.0.0); a client probing `fourclass/1` sees absence and degrades to L1 loudly rather than sending the un-parseable two-element shape ([Handshake.hs:25-27](../../../core/app/CE/Handshake.hs#L25), [Handshake.hs:41](../../../core/app/CE/Handshake.hs#L41)).

### Boundary checks

The reply is an answer, not an authority ([delta.rs:4-5](../../../cli/src/fourclass/batch/delta.rs#L4)):

- **Merge is all-or-nothing.** `merge` works on a copy; an in-place form leaked a half-merged result through the error path as the claimed "pure L1 fallback" ([delta.rs:17-20](../../../cli/src/fourclass/batch/delta.rs#L17), [batch.rs:56](../../../cli/src/fourclass/batch.rs#L56)).
- **Each returned line is consumed once** from a per-side unconsumed set; a double-listed line is a named error, not a `usize` underflow that produced ~18e18 "deleted" lines ([delta.rs:12-16](../../../cli/src/fourclass/batch/delta.rs#L12), [delta.rs:57-59](../../../cli/src/fourclass/batch/delta.rs#L57)).
- **Wire indices are bounds-checked**, not used as slice subscripts, in both the merge and the report — the report path runs inside the daemon, which has no `catch_unwind` ([delta.rs:99-101](../../../cli/src/fourclass/batch/delta.rs#L99), [session.rs:113-123](../../../cli/src/fourclass/session.rs#L113)).
- **The core machine-checks two preconditions** at its boundary ([FourClass.hs:31-44](../../../core/app/CE/FourClass.hs#L31)): no duplicate pair index (Anchor's run maps key on `(pair, run)`, so `M.fromList` would silently drop an earlier duplicate's runs), and **within-first** — no leftover added hash of a pair may occur among that same pair's leftover removed hashes. Within-first is L1's within-file consumption rule seen from the judgment side; verifying its consequence turns a cross-language assumption into a checked contract ([FourClass.hs:3-6](../../../core/app/CE/FourClass.hs#L3)).
- **Blocks are unit-attributed line by line**, not head-line only: one block can span several units, and head-line attribution left 7 of 35 registered units unnamed ([delta.rs:88-93](../../../cli/src/fourclass/batch/delta.rs#L88)).

### Session scope

The session's file pairs come from `git diff --name-status -z -M -C HEAD` ([session.rs:17-19](../../../cli/src/fourclass/session.rs#L17)). A copy record (`C`) carries two paths and maps to an **added** file — the source survives and the destination is new material, which is precisely the duplication signal ([session.rs:83-86](../../../cli/src/fourclass/session.rs#L83)); consuming only one of its two tokens desynchronizes the whole stream. Pairs are filtered by `Lang::judged_path` — the scan-only arm is size-gated and never four-classified ([session.rs:93-97](../../../cli/src/fourclass/session.rs#L93)).

Cross-file relocations remain **informational**: no deny path may lean on this report until a multi-file FPR instrument exists, since claiming a move where there is duplication would hide duplication inside a health signal ([session.rs:3-6](../../../cli/src/fourclass/session.rs#L3)).

### Constants not found

Plan §4.3 refers to a similarity threshold for the duplicate-implementation rule and to MinHash paragraph similarity ([DEVELOPMENT_PLAN.md:121-125](../../DEVELOPMENT_PLAN.md#L121)); no such constant exists anywhere in `core/app/CE/FourClass/` or `cli/src/fourclass/`. Those rules live outside this module (the `clone`/`docdup` families), and their thresholds are not documented here.
