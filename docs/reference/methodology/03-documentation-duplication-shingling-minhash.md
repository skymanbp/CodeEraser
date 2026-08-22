# Documentation duplication — shingling + MinHash/LSH

[index](../methodology.md) · [← 02 T3 near-miss clones — Tree Edit Distance (TSED)](02-t3-near-miss-clones-tree-edit-distance-tsed.md) · [→ 04 Structure judgment — tree-scale entropy, seven axes](04-structure-judgment-tree-scale-entropy-seven.md)

The docdup family answers one question: are two blocks of *documentation text* — markdown paragraphs, comment blocks, docstrings — near-duplicates of each other? The computation is split across the language boundary the project's ADRs fix: Rust extracts, shingles, and coarse-filters; Haskell owns every threshold and issues every verdict. The Rust side keeps a pinned mirror of the core's constants purely so drift is an error rather than a silent score.

Pipeline order is fixed and conservation-clean — `extract → skeleton strip → wordize → admission floor → exemption classify → store`, with `raw == below_floor + stored` ([mod.rs:9](../../../cli/src/docdup/mod.rs#L9)).

### 1. Segment extraction

Three document-text kinds are extracted, frozen as position codes `["md_para", "comment_block", "docstring"]` = `0, 1, 2` ([spec.rs:39](../../../cli/src/docdup/spec.rs#L39), [spec.rs:40-42](../../../cli/src/docdup/spec.rs#L40)).

- **Markdown** paragraphs are maximal runs of *adjacent* paragraph lines ([segments.rs:58](../../../cli/src/docdup/segments.rs#L58)); a non-paragraph line breaks adjacency, so no state beyond the last line number is carried ([segments.rs:68-70](../../../cli/src/docdup/segments.rs#L68)). A line is paragraph content iff it is non-empty and does not start with `#` (ATX heading) or `|` (table row), is not a bare list marker, and is not an HTML-block line ([segments.rs:101-107](../../../cli/src/docdup/segments.rs#L101)). `html_line` is the CommonMark HTML-block start condition at line level: `<` followed by an ASCII letter or `/` ([segments.rs:112-118](../../../cli/src/docdup/segments.rs#L112)). Bare markers are `-`, `*`, `+`, or a digit run followed by `.` or `)` ([segments.rs:120-126](../../../cli/src/docdup/segments.rs#L120)).
- **Comments and docstrings** come off one tree-sitter parse of docdup's own ([segments.rs:142-165](../../../cli/src/docdup/segments.rs#L142)). Contiguous same-column comment nodes merge into one block — `mergeable` iff the previous node ended on row `r−1` at the same start column ([segments.rs:183-199](../../../cli/src/docdup/segments.rs#L183)). Only Python has a docstring convention here (`module`, `function_definition`, `class_definition` hosts whose body's first named child is a bare-string expression statement); JSDoc and Rust `///` arrive lexically as comments ([spec.rs:88-90](../../../cli/src/docdup/spec.rs#L88), [segments.rs:170-178](../../../cli/src/docdup/segments.rs#L170)).

Markdown segments may come from nothing but `md::masked_content_lines` ([segments.rs:4-6](../../../cli/src/docdup/segments.rs#L4)) — the fence + HTML-comment + inline-code triple mask ([md.rs:40-47](../../../cli/src/graph/md.rs#L40)). The byte mask rides *into* wordization rather than being re-derived, so the judge can never see text the detector masks. Line classification itself runs on the unmasked characters only ([segments.rs:87-94](../../../cli/src/docdup/segments.rs#L87)).

### 2. Exempt classes and line-level strips

Two levels of exclusion, both ledgered — every shed line or segment lands in a counter, never in silence ([exempt.rs:1-7](../../../cli/src/docdup/exempt.rs#L1), [exempt.rs:27-36](../../../cli/src/docdup/exempt.rs#L27)).

**Segment-level exemption** is a three-valued code: `["live", "license_header", "inline_allow"]` = `0, 1, 2` ([exempt.rs:16-19](../../../cli/src/docdup/exempt.rs#L16)).

- `license_header` requires all three of: the segment is the file's **first** comment block, `start_line <= LICENSE_HEAD_LINES` where `LICENSE_HEAD_LINES = 5` ([spec.rs:14](../../../cli/src/docdup/spec.rs#L14)), and some line contains one of five markers — `SPDX-License-Identifier`, `Licensed under the Apache License`, `Copyright (c)`, `Permission is hereby granted`, `MIT License` ([spec.rs:46-52](../../../cli/src/docdup/spec.rs#L46), applied at [exempt.rs:42-45](../../../cli/src/docdup/exempt.rs#L42)).
- `inline_allow` requires the marker `ce:allow(docdup)` ([spec.rs:78](../../../cli/src/docdup/spec.rs#L78)) *plus* a non-empty `-- <why>` tail; a bare marker exempts nothing and is itself ledgered as `allow_missing_why` while the segment stays live ([exempt.rs:46-54](../../../cli/src/docdup/exempt.rs#L46), predicate at [exempt.rs:64-71](../../../cli/src/docdup/exempt.rs#L64)).
- Two further routes are structurally zero: path exclusion never reaches the extractor, and the baseline exemption stock does not exist until `ce baseline` ([exempt.rs:4-7](../../../cli/src/docdup/exempt.rs#L4)).

**Line-level strips** apply to comment/docstring segments only; `md_para` lines are passed through untouched by all three ([exempt.rs:86-89](../../../cli/src/docdup/exempt.rs#L86)):

1. **Fenced code** — ` ``` ` or `~~~` after stripping comment decoration `['#','/','*','!',' ']`, toggling with XOR so an unclosed fence honestly strips to segment end ([exempt.rs:90-95](../../../cli/src/docdup/exempt.rs#L90), [exempt.rs:109-112](../../../cli/src/docdup/exempt.rs#L109)).
2. **Skeleton rows** — a decoration-stripped line that is all `-` and at least 3 chars, or one starting with any of the 16 `SKELETON_PREFIXES` (`Args:`, `Arguments:`, `Returns:`, `Raises:`, `Yields:`, `Parameters`, `Attributes:`, `Example:`, `Examples:`, `Note:`, `:param `, `:return`, `:rtype`, `@param`, `@returns`, `@throws`) ([spec.rs:57-74](../../../cli/src/docdup/spec.rs#L57), [exempt.rs:116-125](../../../cli/src/docdup/exempt.rs#L116)).
3. **Overlong lines** — trimmed visible char count `> DOC_LINE_CAP`, `DOC_LINE_CAP = 200` ([spec.rs:35](../../../cli/src/docdup/spec.rs#L35), [exempt.rs:98](../../../cli/src/docdup/exempt.rs#L98)). Rationale in-source: hard-wrapped comment prose runs under ~120 chars while the audited false-positive lines (regex literals, inline snapshots) ran 300+/600+ ([spec.rs:30-34](../../../cli/src/docdup/spec.rs#L30)).

### 3. Wordization and shingle construction

A **word** is a maximal run of `char::is_alphanumeric()` characters — a combining mark (`General_Category=Mark`) does not end the run it sits on — lowercased and NFC-composed before hashing, so canonically equivalent spellings ("café" typed NFC or NFD) hash alike instead of yielding disjoint shingle sets ([shingle.rs:25-39](../../../cli/src/docdup/shingle.rs#L25), the one hash throat at [shingle.rs:44-47](../../../cli/src/docdup/shingle.rs#L44); `DOCDUP_REV` 4). A masked byte or any other non-alphanumeric character terminates the current word, as does end of line. Each word is hashed with the repo's one FNV-1a:

```
h = 0xcbf29ce484222325;  for each byte b:  h = (h XOR b) * 0x00000100000001b3   (wrapping u64)
```
([tokens.rs:134-141](../../../cli/src/dedup/tokens.rs#L134))

The **admission floor** is applied to the surviving word sequence: `words.len() < MIN_DOC_TOKENS` sends the segment to `ledger.below_floor` and it is never stored, with `MIN_DOC_TOKENS = 50` ([spec.rs:10](../../../cli/src/docdup/spec.rs#L10), [mod.rs:74-77](../../../cli/src/docdup/mod.rs#L74)).

Shingles are word `k`-grams at `DOC_SHINGLE = 5` ([spec.rs:25](../../../cli/src/docdup/spec.rs#L25)), computed by the same rolling Rabin-Karp the code-dedup path uses — `pub(crate)` precisely so a second implementation cannot fork it ([winnow.rs:19-23](../../../cli/src/dedup/winnow.rs#L19)). With `BASE = 1_000_003` ([winnow.rs:17](../../../cli/src/dedup/winnow.rs#L17)) and all-wrapping `u64` arithmetic:

```
h_0     = sum_{t=0..k-1} w[t] * BASE^(k-1-t)
h_{i-k+1} = (h_{i-k} - w[i-k] * BASE^(k-1)) * BASE + w[i]        for i = k..n-1
```
([winnow.rs:32-45](../../../cli/src/dedup/winnow.rs#L32)). Sequences shorter than `k` yield no shingles at all ([winnow.rs:29-31](../../../cli/src/dedup/winnow.rs#L29)), which the floor of 50 words puts far out of reach.

Two derived objects, deliberately distinct:

- `shingle_set` — `kgram_hashes` then `sort_unstable` + `dedup`: the sorted, deduplicated **Jaccard alphabet**, `|set| <= n` where `n = words.len()` ([shingle.rs:52-64](../../../cli/src/docdup/shingle.rs#L52)).
- `shingle_seq` — the **unsorted** k-gram sequence, of length `n − DOC_SHINGLE + 1`, kept because verbatim runs need order ([shingle.rs:69-71](../../../cli/src/docdup/shingle.rs#L69), length asserted at [shingle.rs:121](../../../cli/src/docdup/shingle.rs#L121)).

Only the set is cached: `docsegs.shingles` is the sorted deduped `u64`s in little-endian ([mod.rs:32-37](../../../cli/src/docdup/mod.rs#L32), encoder at [mod.rs:117](../../../cli/src/docdup/mod.rs#L117)). `DOCDUP_REV = 4` sits in the meta cache key so a semantics change wipes stale rows ([mod.rs:22-28](../../../cli/src/docdup/mod.rs#L22)).

### 4. The MinHash/LSH coarse filter

Only `exempt = 0` rows enter the corpus — exempt segments are structurally outside it, not filtered later ([candidates.rs:31-41](../../../cli/src/docdup/judge/candidates.rs#L31)). Blob decode refuses a non-whole-`u64` row shape by name rather than letting `chunks_exact` drop a truncated tail (fewer shingles would mean silently missed duplication) ([candidates.rs:61-71](../../../cli/src/docdup/judge/candidates.rs#L61)).

Segments with `set.len() > DOC_SET_CAP` are excluded from the candidate pass and tallied as `over_cap_segments`, `DOC_SET_CAP = 8192` ([candidates.rs:100-106](../../../cli/src/docdup/judge/candidates.rs#L100), [wire.rs:20](../../../cli/src/docdup/judge/wire.rs#L20)).

**MinHash.** The signature is deterministic and RNG-free — permutation index `i` *is* the salt ([minhash.rs:1-6](../../../cli/src/dedup/minhash.rs#L1)):

```
sig[i] = min over x in set of fnv1a(x.to_le_bytes() ++ (i as u32).to_le_bytes())     i = 0..perms-1
```
([minhash.rs:14-28](../../../cli/src/dedup/minhash.rs#L14)). An empty set saturates to `u64::MAX` rows ([minhash.rs:25](../../../cli/src/dedup/minhash.rs#L25)) — unreachable downstream of the 50-word floor.

**Banding.** `LSH_SHAPE = (perms, bands, rows) = (128, 32, 4)` — one fact, shared with the T3 structural candidate source so the two estimators cannot drift ([candidates.rs:30-33](../../../cli/src/dedup/candidates.rs#L30), consumed at [candidates.rs:98](../../../cli/src/docdup/judge/candidates.rs#L98)). Band `b`'s key is `(b, fnv1a(sig[b*rows .. (b+1)*rows] little-endian concatenated))`, with `bands * rows == sig.len()` asserted ([minhash.rs:33-48](../../../cli/src/dedup/minhash.rs#L33)). Two segments are LSH candidates iff they collide in some band ([minhash.rs:30-32](../../../cli/src/dedup/minhash.rs#L30)). The standard collision probability for this shape — `1 − (1 − J^4)^32` — is a property of `(bands, rows) = (32, 4)`, *derived here, not a constant present in the source*.

**Seed pairs.** LSH is not the only source: an inverted index over every shingle hash contributes all pairs sharing at least one shingle, tallied separately as `seed_pairs` ([candidates.rs:114-120](../../../cli/src/docdup/judge/candidates.rs#L114)). The candidate set is the union of both sources, deduplicated as an ordered `(min, max)` `BTreeSet` ([candidates.rs:150-154](../../../cli/src/docdup/judge/candidates.rs#L150)).

**Hot groups.** A bucket with `len() <= HOT_GROUP_CAP` pairs all-pairs; above the cap it contributes only the adjacent chain `list.windows(2)`, and the event is counted (`hot_bands` / `hot_shingles`) ([candidates.rs:142-149](../../../cli/src/docdup/judge/candidates.rs#L142)). `HOT_GROUP_CAP = pairs::HOT_CAP = 64` ([candidates.rs:38](../../../cli/src/dedup/candidates.rs#L38), [pairs.rs:27](../../../cli/src/dedup/pairs.rs#L27)). Chaining rather than skipping is the fix for a review finding that skipping hot groups drove detection to zero ([candidates.rs:91-94](../../../cli/src/docdup/judge/candidates.rs#L91)).

### 5. The verbatim token floor

The cache stores only the deduped set, so shingle **sequences** are re-derived per hosting file through the same `doc_facts` throat, for exactly the files hosting candidates ([runs.rs:20-51](../../../cli/src/docdup/judge/candidates/runs.rs#L20)). Two guards run there: the segment must still be found at the same `(kind, start_line, end_line)` or the run aborts with "disk drifted from the docsegs cache" ([runs.rs:34-41](../../../cli/src/docdup/judge/candidates/runs.rs#L34)), and the re-derived set must equal the cached one byte for byte ([runs.rs:44-48](../../../cli/src/docdup/judge/candidates/runs.rs#L44)).

The run is the longest common **contiguous** shingle run, measured by seed-extension: for each `(i, j)` with `a[i] == b[j]`, positions where `a[i-1] == b[j-1]` are skipped as non-starts, so each maximal run is measured exactly once ([runs.rs:63-87](../../../cli/src/docdup/judge/candidates/runs.rs#L63)). The result is converted from shingles to **words**:

```
verbatim_words = 0                       if best == 0
verbatim_words = best + DOC_SHINGLE − 1  otherwise      (a run of R shingles spans R + k − 1 words)
```
([runs.rs:82-86](../../../cli/src/docdup/judge/candidates/runs.rs#L82), same identity stated at [shingle.rs:66-68](../../../cli/src/docdup/shingle.rs#L66)).

The floor is `VERBATIM_FLOOR = 50` words, provenance recorded in-source as plan `:68`, Lee et al. `2107.06499`, verbatim lower bound 50 tokens ([spec.rs:19](../../../cli/src/docdup/spec.rs#L19); core-side owner `verbatimFloor = 50` at [Cost.hs:66](../../../core/app/CE/Docdup/Cost.hs#L66) with the same provenance note at [Cost.hs:59-64](../../../core/app/CE/Docdup/Cost.hs#L59)). Since ADR-008 P1 the floor's *verdict home* is the Haskell core; the Rust constant is a pinned mirror ([wire.rs:6-10](../../../cli/src/docdup/judge/wire.rs#L6)). The run rides each request row so one wire transcript holds the complete verdict inputs, while the texts themselves never cross the wire ([Docdup.hs:12-16](../../../core/app/CE/Docdup.hs#L12), [Cost.hs:61-64](../../../core/app/CE/Docdup/Cost.hs#L61)).

### 6. Wire shape and boundary contract

Candidate pairs are chunked at `DOC_PAIR_CAP = 4096` per request ([wire.rs:23](../../../cli/src/docdup/judge/wire.rs#L23), bound into the lockstep family at [wire.rs:83-90](../../../cli/src/docdup/judge/wire.rs#L83), chunk loop at [lockstep.rs:54](../../../cli/src/lockstep.rs#L54)). Each chunk carries the distinct endpoint sets once, addressed by sorted rank, and rows of `[i, j, verbatimRun]` ([wire.rs:37-48](../../../cli/src/docdup/judge/wire.rs#L37), rank throat at [lockstep.rs:78-88](../../../cli/src/lockstep.rs#L78)). Raw `inter`/`union` cross back, never a ratio — if Rust sent a ratio, "the re-check lives in Haskell" would be an empty claim ([Jaccard.hs:1-9](../../../core/app/CE/Docdup/Jaccard.hs#L1)).

The core's cascade is `decode → cap check → boundary contract → judge` ([Wire.hs:27-40](../../../core/app/CE/Wire.hs#L27)). Over-cap is `any set longer than docSetCap` or `more than docPairCap rows` ([Docdup.hs:58-60](../../../core/app/CE/Docdup.hs#L58)) and answers a **complete degraded reply** with `reason = "docdup_too_large"`, never a truncated one ([Docdup.hs:62](../../../core/app/CE/Docdup.hs#L62), [Docdup.hs:151](../../../core/app/CE/Docdup.hs#L151)). The caps are `docSetCap = 8192` and `docPairCap = 4096` ([Cost.hs:50](../../../core/app/CE/Docdup/Cost.hs#L50), [Cost.hs:57](../../../core/app/CE/Docdup/Cost.hs#L57)); the sizing anchor is recorded in-source as `Data.Set` intersection/union costing `O(n·log n)`, so `2 · 8192 · 13 ≈ 2×10⁵` strict steps per pair and `4096 × 2×10⁵ ≈ 8×10⁸` per worst-case request ([Cost.hs:43-48](../../../core/app/CE/Docdup/Cost.hs#L43), [Cost.hs:52-55](../../../core/app/CE/Docdup/Cost.hs#L52)).

The boundary contract names the first offender in request order ([Docdup.hs:68-83](../../../core/app/CE/Docdup.hs#L68)). Per set: non-empty, no negative element, every element `< 2^64`, strictly ascending ([Docdup.hs:85-93](../../../core/app/CE/Docdup.hs#L85)). Per pair row: exactly `[i, j, run]`, endpoints in `[0, n)`, `i /= j` (a segment is not a duplicate of itself), `run >= 0` ([Docdup.hs:95-106](../../../core/app/CE/Docdup.hs#L95)). Rows must ascend on the `(i, j)` **identity prefix**, not the whole row — `[[0,1,0],[0,1,60]]` is lexicographically ascending yet judges one pair twice with two bits ([Docdup.hs:76-79](../../../core/app/CE/Docdup.hs#L76)).

### 7. Exact Jaccard verification and the verdict

The core computes the counts itself, from the ascending deduped sets, with `Data.Set` ([Jaccard.hs:19-26](../../../core/app/CE/Docdup/Jaccard.hs#L19)):

```
interUnion a b = (|S(a) ∩ S(b)|, |S(a) ∪ S(b)|)
```

`fromDistinctAscList`'s precondition is not assumed — the boundary contract rejects any non-ascending set before this module runs, so a violated precondition is unreachable rather than silently corrupting ([Jaccard.hs:14-18](../../../core/app/CE/Docdup/Jaccard.hs#L14)).

The threshold is an integer ratio, cross-multiplied — no floats in the core ([Cost.hs:17-21](../../../core/app/CE/Docdup/Cost.hs#L17)):

```
dupDecidesWith num den inter union  =  inter * den >= num * union
```
([Cost.hs:79-80](../../../core/app/CE/Docdup/Cost.hs#L79)) with `jaccardNum = 80`, `jaccardDen = 100` ([Cost.hs:27](../../../core/app/CE/Docdup/Cost.hs#L27), [Cost.hs:30](../../../core/app/CE/Docdup/Cost.hs#L30)) — i.e. `J >= 0.80` decided exactly in integers. The full verdict is the disjunction:

```
dupVerdictWith (num, den, vfloor) inter union run
  =  dupDecidesWith num den inter union  ||  run >= vfloor
```
([Cost.hs:91-93](../../../core/app/CE/Docdup/Cost.hs#L91)), bound to `(80, 100, 50)` at [Cost.hs:87](../../../core/app/CE/Docdup/Cost.hs#L87). Both halves are expressed *once*, in knob-parameterized form, so the reference battery's dead-knob probe perturbs the production comparison rather than a re-implementation ([Cost.hs:76-78](../../../core/app/CE/Docdup/Cost.hs#L76), [Cost.hs:89-90](../../../core/app/CE/Docdup/Cost.hs#L89)).

Each scored row goes out as `[i, j, inter, union]` with a parallel per-row verdict bit; the additive `counts.jaccardDups` counts the **Jaccard half only** ([Docdup.hs:112-122](../../../core/app/CE/Docdup.hs#L112), reply shape at [Docdup.hs:127-151](../../../core/app/CE/Docdup.hs#L127)).

### 8. Mirror pinning

Rust's `is_dup` is a **mirror**, not an authority — the reported set is built from the wire's per-row bits ([mod.rs:47-56](../../../cli/src/docdup/judge/mod.rs#L47)):

```
inter * JACCARD_DEN >= JACCARD_NUM * union  ||  verbatim >= VERBATIM_FLOOR
```
with `JACCARD_NUM = 80`, `JACCARD_DEN = 100` ([wire.rs:29-30](../../../cli/src/docdup/judge/wire.rs#L29)). Every reported row is checked against it, and disagreement kills the run naming both owning modules ([mod.rs:104-107](../../../cli/src/docdup/judge/mod.rs#L104)); an echoed pair that was never sent is an error, not a panic ([mod.rs:101-103](../../../cli/src/docdup/judge/mod.rs#L101)). The verdict boundary is pinned by test at `is_dup(4,5,0) == true`, `is_dup(3,4,0) == false`, `is_dup(0,100,50) == true`, `is_dup(0,100,49) == false` ([mod.rs:155-160](../../../cli/src/docdup/judge/mod.rs#L155)).

Every reply's `knobs` block is pinned against the Rust copies — `jaccardNum` 80, `jaccardDen` 100, `shingleK == DOC_SHINGLE` (5), `verbatimFloor == VERBATIM_FLOOR` (50) ([wire.rs:60-71](../../../cli/src/docdup/judge/wire.rs#L60), echoed from [Docdup.hs:142-148](../../../core/app/CE/Docdup.hs#L142)). `shingleK` is echoed, never computed, by the core: sets arrive pre-shingled, and the constant exists on the wire solely as the protocol's record of the alphabet geometry — two sides shingling at different widths would compare incommensurable alphabets and no downstream gate could tell ([Cost.hs:32-41](../../../core/app/CE/Docdup/Cost.hs#L32), drift test at [wire.rs:116-121](../../../cli/src/docdup/judge/wire.rs#L116)).
