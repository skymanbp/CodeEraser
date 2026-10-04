# Documentation duplication — shingling + MinHash/LSH

[index](../methodology.md) · [← 02 T3 near-miss clones — Tree Edit Distance (TSED)](02-t3-near-miss-clones-tree-edit-distance-tsed.md) · [→ 04 Structure judgment — tree-scale entropy, eight axes](04-structure-judgment-tree-scale-entropy-seven.md)

The docdup family answers one question: are two blocks of *documentation text* — markdown paragraphs, comment blocks, docstrings, HTML page text, plain-text paragraphs — near-duplicates of each other? The computation is split across the language boundary the project's ADRs fix: Rust extracts and shingles; since plan v2.33 W3 Haskell coarse-filters, measures the verbatim runs, owns every threshold and issues every verdict. The Rust side reads the core's constants off the definition package and pins the reply's echo, so drift is an error rather than a silent score.

Pipeline order is fixed and conservation-clean — `extract → skeleton strip → wordize → admission floor → exemption classify → store`, with `raw == below_floor + stored` ([mod.rs:9](../../../cli/src/docdup/mod.rs#L9)).

### 1. Segment extraction

Five document-text kinds are extracted, frozen as position codes `["md_para", "comment_block", "docstring", "html_text", "text_para"]` = `0, 1, 2, 3, 4` — `html_text` appended at `DOCDUP_REV` 6 (plan v2.30 step 5), `text_para` at `DOCDUP_REV` 8 (step 5b-8) ([spec.rs:29-32](../../../cli/src/docdup/spec.rs#L29), [spec.rs:29-33](../../../cli/src/docdup/spec.rs#L29)).

- **Markdown** paragraphs are maximal runs of *adjacent* paragraph lines ([segments.rs:69](../../../cli/src/docdup/segments.rs#L69)); a non-paragraph line breaks adjacency, so no state beyond the last line number is carried ([segments.rs:79-81](../../../cli/src/docdup/segments.rs#L79)). A line is paragraph content iff it is non-empty and does not start with `#` (ATX heading) or `|` (table row), is not a bare list marker, and is not an HTML-block line ([segments.rs:133-139](../../../cli/src/docdup/segments.rs#L133)). `html_line` is the CommonMark HTML-block start condition at line level: `<` followed by an ASCII letter or `/` ([segments.rs:144-150](../../../cli/src/docdup/segments.rs#L144)). Bare markers are `-`, `*`, `+`, or a digit run followed by `.` or `)` ([segments.rs:152-158](../../../cli/src/docdup/segments.rs#L152)).
- **Comments and docstrings** come off one tree-sitter parse of docdup's own ([segments.rs:177-200](../../../cli/src/docdup/segments.rs#L177)). Contiguous same-column comment nodes merge into one block — `mergeable` iff the previous node's last *content* row is `r−1` at the same start column ([segments.rs:222-240](../../../cli/src/docdup/segments.rs#L222)); the content row is the end row unless the node ends at column 0 of a later row, which tree-sitter-rust's doc-comment node (`///`, `//!`) does, so a Rust doc block is one segment whose `end_line` is its last line, not one segment per line ending a row late (`DOCDUP_REV` 5, v2.28; [segments.rs:242-249](../../../cli/src/docdup/segments.rs#L242)). Only Python has a docstring convention here (`module`, `function_definition`, `class_definition` hosts whose body's first named child is a bare-string expression statement); JSDoc and Rust `///` arrive lexically as comments ([spec.rs:37-46](../../../cli/src/docdup/spec.rs#L37), [segments.rs:205-213](../../../cli/src/docdup/segments.rs#L205)).
- **HTML** text is a block element's prose — a paragraph, a list item, a heading, a table cell, a caption (`BLOCK`) — one segment per element, the way a Markdown paragraph is ([html.rs:28-29](../../../cli/src/docdup/html.rs#L28), [html.rs:39-65](../../../cli/src/docdup/html.rs#L39)). Its lines are the element's source lines under a byte mask that leaves only the element's own text leaves visible, so markup, attributes, entities and comments never reach the wordizer, and neither does the text of a nested block (a segment of its own), inline `code`, or `pre`, `textarea`, `script` and `style` content ([html.rs:70-97](../../../cli/src/docdup/html.rs#L70)); every code and script element the walk meets is counted on the shed ledger beside the Markdown walk's counters, masked or skipped ([segments.rs:34-50](../../../cli/src/docdup/segments.rs#L34)). Because the page's comments are masked, not segmented, a `ce:allow(docdup) -- <why>` written in an HTML comment inside the element stays on the raw line for the exemption classifier ([html.rs:9-13](../../../cli/src/docdup/html.rs#L9)).
- **Plain text** (`.txt`, the prose-only arm of plan v2.30 step 5b-8) has one separator, the blank line: a paragraph is a maximal run of non-blank lines, no line is structure to this kind and nothing is masked, so every line rides whole ([segments.rs:119-126](../../../cli/src/docdup/segments.rs#L119)). A text file is read by this family alone — indexed as a document, never sized, budgeted or ratcheted, with no unit, no site, no fingerprint and no graph node of its own (a page that links it lands an asset node, as any linked file does) ([lang.rs:205-216](../../../cli/src/scan/lang.rs#L205)); the `.txt` names a specification reserves for a machine format — `CMakeLists.txt`, `compile_flags.txt`, `robots.txt` — are no text files ([Common.hs:84-95](../../../core/app/CE/Lang/Common.hs#L84)).

Markdown segments may come from nothing but `md::masked_content_lines` ([segments.rs:6-8](../../../cli/src/docdup/segments.rs#L6)) — the fence + HTML-comment + inline-code triple mask ([md.rs:51-57](../../../cli/src/graph/md.rs#L51)). The byte mask rides *into* wordization rather than being re-derived, so the judge can never see text the detector masks. Line classification itself runs on the unmasked characters only ([segments.rs:96-103](../../../cli/src/docdup/segments.rs#L96)).

### 2. Exempt classes and line-level strips

Two levels of exclusion, both ledgered — every shed line or segment lands in a counter, never in silence ([exempt.rs:1-7](../../../cli/src/docdup/exempt.rs#L1), [exempt.rs:24-33](../../../cli/src/docdup/exempt.rs#L24)).

**Segment-level exemption** is a three-valued code: `["live", "license_header", "inline_allow"]` = `0, 1, 2` ([exempt.rs:13-16](../../../cli/src/docdup/exempt.rs#L13)).

- `license_header` requires all three of: the segment is the file's **first** comment block, `start_line <= license_head_lines` where `license_head_lines = 5` (the core's `licHeadLines`, [Cost.hs:135-136](../../../core/app/CE/Docdup/Cost.hs#L135)), and some line contains one of five markers — `SPDX-License-Identifier`, `Licensed under the Apache License`, `Copyright (c)`, `Permission is hereby granted`, `MIT License` ([Prose.hs:98-101](../../../core/app/CE/Lang/Common/Prose.hs#L98), applied at [exempt.rs:43-46](../../../cli/src/docdup/exempt.rs#L43)).
- `inline_allow` requires the marker `ce:allow(docdup)` ([Prose.hs:117-119](../../../core/app/CE/Lang/Common/Prose.hs#L117)) *plus* a non-empty `-- <why>` tail; a bare marker exempts nothing and is itself ledgered as `allow_missing_why` while the segment stays live ([exempt.rs:47-55](../../../cli/src/docdup/exempt.rs#L47), predicate at [exempt.rs:68-72](../../../cli/src/docdup/exempt.rs#L68)).
- Two further routes are structurally zero: path exclusion never reaches the extractor, and the baseline exemption stock does not exist until `ce baseline` ([exempt.rs:4-7](../../../cli/src/docdup/exempt.rs#L4)).

**Line-level strips** apply to comment/docstring segments only; `md_para`, `html_text` and `text_para` lines are passed through untouched by all three ([exempt.rs:90-93](../../../cli/src/docdup/exempt.rs#L90)):

1. **Fenced code** — ` ``` ` or `~~~` after stripping comment decoration `['#','/','*','!',' ']`, toggling with XOR so an unclosed fence honestly strips to segment end ([exempt.rs:94-99](../../../cli/src/docdup/exempt.rs#L94), [exempt.rs:113-116](../../../cli/src/docdup/exempt.rs#L113)).
2. **Skeleton rows** — a decoration-stripped line that is all `-` and at least 3 chars, or one starting with any of the 43 `skeleton_prefixes`: the Google/Sphinx/NumPy/JSDoc section vocabulary (`Args:`, `Arguments:`, `Returns:`, `Raises:`, `Yields:`, `Parameters`, `Attributes:`, `Example:`, `Examples:`, `Note:`, `:param `, `:return`, `:rtype`, `@param`) and, since `DOCDUP_REV` 6 (plan v2.30 step 5), the Javadoc / Doxygen / roxygen / LDoc tags — `@return` (which covers `@returns` as a prefix), `@throw`, `@exception`, `@brief`, `@see`, `@since`, `@author`, `@version`, `@tparam`, `@treturn`, `@usage`, `@examples`, `@export`, `@importFrom`, `@rdname`, `@details`, `@inheritParams`, `@describeIn`, and the Doxygen commands again under `\` ([Prose.hs:102-116](../../../core/app/CE/Lang/Common/Prose.hs#L102), [exempt.rs:123-135](../../../cli/src/docdup/exempt.rs#L123)). The decoration is `#`, `/`, `*`, `!` and quotes, which strips roxygen's `#'`; after the all-dash rule a leading `--` run — the Lua and Haskell comment marker — goes too, while a single `-` stays a list bullet, so `- Note: …` is prose.
3. **Overlong lines** — trimmed visible char count `> doc_line_cap`, `doc_line_cap = 200` (the core's `docLineCap`, [Cost.hs:105-106](../../../core/app/CE/Docdup/Cost.hs#L105), applied at [exempt.rs:102](../../../cli/src/docdup/exempt.rs#L102)). Rationale in-source: hard-wrapped comment prose runs under ~120 chars while the audited false-positive lines (regex literals, inline snapshots) ran 300+/600+ ([Cost.hs:98-101](../../../core/app/CE/Docdup/Cost.hs#L98)).

### 3. Wordization and shingle construction

A **word** is a maximal run of `char::is_alphanumeric()` characters — a combining mark (`General_Category=Mark`) does not end the run it sits on — lowercased and NFC-composed before hashing, so canonically equivalent spellings ("café" typed NFC or NFD) hash alike instead of yielding disjoint shingle sets ([shingle.rs:25-39](../../../cli/src/docdup/shingle.rs#L25), the one hash throat at [shingle.rs:44-47](../../../cli/src/docdup/shingle.rs#L44); `DOCDUP_REV` <!--ce:ver:docdup_rev#digits-->8<!--/ce-->). A masked byte or any other non-alphanumeric character terminates the current word, as does end of line. Each word is hashed with the repo's one FNV-1a:

```
h = 0xcbf29ce484222325;  for each byte b:  h = (h XOR b) * 0x00000100000001b3   (wrapping u64)
```
([tokens.rs:188-195](../../../cli/src/dedup/tokens.rs#L188))

The **admission floor** is applied to the surviving word sequence: `words.len() < min_doc_tokens` sends the segment to `ledger.below_floor` and it is never stored, with `min_doc_tokens = 50` (the core's `minDocTokens`, [Cost.hs:89-90](../../../core/app/CE/Docdup/Cost.hs#L89), applied at [mod.rs:88-91](../../../cli/src/docdup/mod.rs#L88)).

Shingles are word `k`-grams at `doc_shingle = 5` (the core's `shingleK`, [Cost.hs:47-48](../../../core/app/CE/Docdup/Cost.hs#L47)), computed by the same rolling Rabin-Karp the code-dedup path uses — `pub(crate)` precisely so a second implementation cannot fork it ([winnow.rs:19-23](../../../cli/src/dedup/winnow.rs#L19)). With `BASE = 1_000_003` ([winnow.rs:17](../../../cli/src/dedup/winnow.rs#L17)) and all-wrapping `u64` arithmetic:

```
h_0     = sum_{t=0..k-1} w[t] * BASE^(k-1-t)
h_{i-k+1} = (h_{i-k} - w[i-k] * BASE^(k-1)) * BASE + w[i]        for i = k..n-1
```
([winnow.rs:32-45](../../../cli/src/dedup/winnow.rs#L32)). Sequences shorter than `k` yield no shingles at all ([winnow.rs:29-31](../../../cli/src/dedup/winnow.rs#L29)), which the floor of 50 words puts far out of reach.

Two derived objects, deliberately distinct:

- `shingle_set` — `kgram_hashes` then `sort_unstable` + `dedup`: the sorted, deduplicated **Jaccard alphabet**, `|set| <= n` where `n = words.len()` ([shingle.rs:52-64](../../../cli/src/docdup/shingle.rs#L52)).
- `shingle_seq` — the **unsorted** k-gram sequence, of length `n − doc_shingle + 1`, kept because verbatim runs need order ([shingle.rs:69-71](../../../cli/src/docdup/shingle.rs#L69), length asserted at [unit/docdup/shingle.rs:47](../../../cli/tests/unit/docdup/shingle.rs#L47)).

Only the set is cached: `docsegs.shingles` is the sorted deduped `u64`s in little-endian ([mod.rs:46-51](../../../cli/src/docdup/mod.rs#L46), encoder at [mod.rs:131](../../../cli/src/docdup/mod.rs#L131)). <!--ce:ver:docdup_rev#digits-->`DOCDUP_REV = 8`<!--/ce--> sits in the meta cache key so a semantics change wipes stale rows ([mod.rs:22-31](../../../cli/src/docdup/mod.rs#L22)).

### 4. The MinHash/LSH coarse filter

Only `exempt = 0` rows enter the corpus — exempt segments are structurally outside it, not filtered later ([candidates.rs:79-88](../../../cli/src/docdup/judge/candidates.rs#L79)). Blob decode refuses a non-whole-`u64` row shape by name rather than letting `chunks_exact` drop a truncated tail (fewer shingles would mean silently missed duplication) ([candidates.rs:111-121](../../../cli/src/docdup/judge/candidates.rs#L111)).

Segments with more shingles than `docSetCap = 8192` ([Cost.hs:56-57](../../../core/app/CE/Docdup/Cost.hs#L56), read off the definition package's `limits` since plan v2.33 W3) are never sent to the coarse filter and are tallied as `over_cap_segments` ([candidates.rs:160-169](../../../cli/src/docdup/judge/candidates.rs#L160)). Since W3 the filter itself runs in the core: the measuring side sends every sendable segment's strictly ascending set in one `docpairs/1` request and maps the kept pairs back ([candidates.rs:170-198](../../../cli/src/docdup/judge/candidates.rs#L170), handler [Pairs.hs:5-10](../../../core/app/CE/Docdup/Pairs.hs#L5)); a request over the per-set or the corpus ceiling (`docCorpusCap = 16777216` shingles) answers a complete degraded reply `docpairs_too_large`, and the measuring side refuses a degraded or skewed reply by name ([Pairs.hs:33-38](../../../core/app/CE/Docdup/Pairs.hs#L33)).

**MinHash.** The signature is deterministic and RNG-free — permutation index `i` *is* the salt; one implementation serves this filter and the T3 structural source S4 ([Lsh.hs:3-15](../../../core/app/CE/Candidates/Lsh.hs#L3)):

```
sig[i] = min over x in set of fnv1a(x.to_le_bytes() ++ (i as u32).to_le_bytes())     i = 0..perms-1
```
([Lsh.hs:59-75](../../../core/app/CE/Candidates/Lsh.hs#L59)). An empty set saturates to `u64::MAX` rows ([Lsh.hs:60](../../../core/app/CE/Candidates/Lsh.hs#L60)) — unreachable downstream of the 50-word floor.

**Banding.** `lshShape = (perms, bands, rows) = (128, 32, 4)` — one fact, shared with the T3 structural candidate source so the two estimators cannot drift ([Cost.hs:43-47](../../../core/app/CE/Candidates/Cost.hs#L43), consumed at [Coarse.hs:33](../../../core/app/CE/Docdup/Coarse.hs#L33)). Band `b`'s key is the fnv1a of its `rows` signature words, little-endian, continued one after another ([Lsh.hs:76-78](../../../core/app/CE/Candidates/Lsh.hs#L76)). Two segments **share a band bucket** iff they collide in some band ([Lsh.hs:80-98](../../../core/app/CE/Candidates/Lsh.hs#L80)); bucket membership becomes a candidate PAIR subject to the hot-group rule below, so in a bucket past the cap two colliding segments need not be candidates. The standard collision probability for this shape — `1 − (1 − J^4)^32` — is a property of `(bands, rows) = (32, 4)`, *derived here, not a constant present in the source*.

**Seed pairs.** LSH is not the only source: an inverted index over every shingle hash contributes all pairs sharing at least one shingle; a seed pair counts as `seed_pairs` only when LSH did not already find it ([Coarse.hs:29-43](../../../core/app/CE/Docdup/Coarse.hs#L29)). The candidate set is the union of both sources, each pair once as `(min, max)`, answered ascending ([Coarse.hs:44-48](../../../core/app/CE/Docdup/Coarse.hs#L44)).

**Hot groups.** A bucket with at most `hotCap` members pairs all-pairs; above the cap it contributes only the adjacent chain, and the event is counted (`hot_bands` / `hot_shingles`) ([Groups.hs:12-20](../../../core/app/CE/Candidates/Groups.hs#L12)). `hotCap = 64` ([Cost.hs:39-40](../../../core/app/CE/Dedup/Cost.hs#L39)), the same cap the T3 sources S3 and S4 pair under. Chaining rather than skipping is the fix for a review finding that skipping hot groups drove detection to zero ([Groups.hs:1-6](../../../core/app/CE/Candidates/Groups.hs#L1)).

### 5. The verbatim token floor

The cache stores only the deduped set, so shingle **sequences** are re-derived per hosting file through the same `doc_facts` throat, for exactly the segments the kept pairs name ([seqs.rs:16-31](../../../cli/src/docdup/judge/candidates/seqs.rs#L16), sequence at [shingle.rs:66-71](../../../cli/src/docdup/shingle.rs#L66)). Two guards run there: the segment must still be found at the same `(kind, start_line, end_line)` or the run aborts with "disk drifted from the docsegs cache" ([seqs.rs:32-43](../../../cli/src/docdup/judge/candidates/seqs.rs#L32)), and the re-derived set must equal the cached one byte for byte ([seqs.rs:44-50](../../../cli/src/docdup/judge/candidates/seqs.rs#L44)). Since plan v2.33 W3 the sequences cross the wire (`docdup/1`'s additive `seqs` shape) and the core measures the runs; the measuring side no longer does.

The run is the longest common **contiguous** shingle run, measured by seed-extension: a run starts only at a position pair whose previous shingles differ (or at either sequence's first shingle), so each maximal run is measured exactly once ([Runs.hs:11-14](../../../core/app/CE/Docdup/Runs.hs#L11), [Runs.hs:48](../../../core/app/CE/Docdup/Runs.hs#L48)). The result is converted from shingles to **words**:

```
verbatim_words = 0                       if best == 0
verbatim_words = best + doc_shingle − 1  otherwise      (a run of R shingles spans R + k − 1 words)
```
([Runs.hs:38-43](../../../core/app/CE/Docdup/Runs.hs#L38), same identity stated at [shingle.rs:66-68](../../../cli/src/docdup/shingle.rs#L66)).

The floor is `verbatimFloor = 50` words in the core, provenance recorded in-source as plan `:68`, Lee et al. `2107.06499`, verbatim lower bound 50 tokens ([Cost.hs:115-116](../../../core/app/CE/Docdup/Cost.hs#L115), the provenance note at [Cost.hs:108-111](../../../core/app/CE/Docdup/Cost.hs#L108)). Since ADR-008 P1 the floor's *verdict home* is the Haskell core; since plan v2.32 step 2 the measuring side's copy is the definition package's `docdup.verbatim_floor`, built from the same constant and pinned by the echo ([wire.rs:4-8](../../../cli/src/docdup/judge/wire.rs#L4), [Lang.hs:162-166](../../../core/app/CE/Lang.hs#L162)). The shingle sequences ride the request (the older shape carried the measured run on each row instead), so one wire transcript holds the complete verdict inputs, while the texts themselves never cross the wire ([Docdup.hs:12-16](../../../core/app/CE/Docdup.hs#L12), [Cost.hs:111-114](../../../core/app/CE/Docdup/Cost.hs#L111)).

### 6. Wire shape and boundary contract

Candidate pairs are chunked at the package's `doc_pair_cap` (`docPairCap = 4096`, [Cost.hs:76-77](../../../core/app/CE/Docdup/Cost.hs#L76)) per request (bound into the lockstep family at [wire.rs:87-90](../../../cli/src/docdup/judge/wire.rs#L87), chunk loop at [lockstep.rs:78](../../../cli/src/lockstep.rs#L78)). Each chunk carries the distinct endpoints' shingle sequences once, addressed by sorted rank, and pair rows `[i, j]` ([wire.rs:25-38](../../../cli/src/docdup/judge/wire.rs#L25), rank throat at [lockstep.rs:157-168](../../../cli/src/lockstep.rs#L157)); the core derives each set and measures each run and answers the runs beside the scores ([Docdup.hs:130-137](../../../core/app/CE/Docdup.hs#L130)). The older `sets` shape with `[i, j, verbatimRun]` rows is still answered, and a request that sends both shapes is refused by name ([Docdup.hs:102-103](../../../core/app/CE/Docdup.hs#L102)). Raw `inter`/`union` cross back, never a ratio — if Rust sent a ratio, "the re-check lives in Haskell" would be an empty claim ([Jaccard.hs:1-9](../../../core/app/CE/Docdup/Jaccard.hs#L1)).

The core's cascade is `decode → cap check → boundary contract → judge` ([Wire.hs:197-210](../../../core/app/CE/Wire.hs#L197)). Over-cap is `any set longer than docSetCap` or `more than docPairCap rows` ([Docdup.hs:78-80](../../../core/app/CE/Docdup.hs#L78)) and answers a **complete degraded reply** with `reason = "docdup_too_large"`, never a truncated one ([Docdup.hs:83](../../../core/app/CE/Docdup.hs#L83), [Docdup.hs:213](../../../core/app/CE/Docdup.hs#L213)). The caps are `docSetCap = 8192` and `docPairCap = 4096` ([Cost.hs:54-55](../../../core/app/CE/Docdup/Cost.hs#L54), [Cost.hs:61-62](../../../core/app/CE/Docdup/Cost.hs#L61)); the sizing anchor is recorded in-source as `Data.Set` intersection/union costing `O(n·log n)`, so `2 · 8192 · 13 ≈ 2×10⁵` strict steps per pair and `4096 × 2×10⁵ ≈ 8×10⁸` per worst-case request ([Cost.hs:50-53](../../../core/app/CE/Docdup/Cost.hs#L50), [Cost.hs:72-75](../../../core/app/CE/Docdup/Cost.hs#L72)).

The boundary contract names the first offender in request order ([Docdup.hs:89-111](../../../core/app/CE/Docdup.hs#L89)); beside `seqs` each sequence must be non-empty u64 elements in any order and each pair row exactly `[i, j]` under the same endpoint checks ([Docdup.hs:113-128](../../../core/app/CE/Docdup.hs#L113)). Per set: non-empty, no negative element, every element `< 2^64`, strictly ascending ([Docdup.hs:141-147](../../../core/app/CE/Docdup.hs#L141)). Per pair row: exactly `[i, j, run]`, endpoints in `[0, n)`, `i /= j` (a segment is not a duplicate of itself), `run >= 0` ([Docdup.hs:151-162](../../../core/app/CE/Docdup.hs#L151)). Rows must ascend on the `(i, j)` **identity prefix**, not the whole row — `[[0,1,0],[0,1,60]]` is lexicographically ascending yet judges one pair twice with two bits ([Docdup.hs:97-100](../../../core/app/CE/Docdup.hs#L97)).

### 7. Exact Jaccard verification and the verdict

The core computes the counts itself, from the ascending deduped sets, with `Data.Set` ([Jaccard.hs:19-26](../../../core/app/CE/Docdup/Jaccard.hs#L19)):

```
interUnion a b = (|S(a) ∩ S(b)|, |S(a) ∪ S(b)|)
```

`fromDistinctAscList`'s precondition is not assumed — the boundary contract rejects any non-ascending set before this module runs, so a violated precondition is unreachable rather than silently corrupting ([Jaccard.hs:14-18](../../../core/app/CE/Docdup/Jaccard.hs#L14)).

The threshold is an integer ratio, cross-multiplied — no floats in the core ([Cost.hs:22-30](../../../core/app/CE/Docdup/Cost.hs#L22)):

```
dupDecidesWith num den inter union  =  inter * den >= num * union
```
([Cost.hs:149-150](../../../core/app/CE/Docdup/Cost.hs#L149)) with `jaccardNum = 80`, `jaccardDen = 100` ([Cost.hs:31-32](../../../core/app/CE/Docdup/Cost.hs#L31), [Cost.hs:34-35](../../../core/app/CE/Docdup/Cost.hs#L34)) — i.e. `J >= 0.80` decided exactly in integers. The full verdict is the disjunction:

```
dupVerdictWith (num, den, vfloor) inter union run
  =  dupDecidesWith num den inter union  ||  run >= vfloor
```
([Cost.hs:161-163](../../../core/app/CE/Docdup/Cost.hs#L161)), bound to `(80, 100, 50)` at [Cost.hs:156-157](../../../core/app/CE/Docdup/Cost.hs#L156). Both halves are expressed *once*, in knob-parameterized form, so the reference battery's dead-knob probe perturbs the production comparison rather than a re-implementation ([Cost.hs:145-148](../../../core/app/CE/Docdup/Cost.hs#L145), [Cost.hs:159-160](../../../core/app/CE/Docdup/Cost.hs#L159)).

Each scored row goes out as `[i, j, inter, union]` with a parallel per-row verdict bit; the additive `counts.jaccardDups` counts the **Jaccard half only** ([Docdup.hs:164-174](../../../core/app/CE/Docdup.hs#L164), reply shape at [Docdup.hs:130-157](../../../core/app/CE/Docdup.hs#L130)).

### 8. Mirror pinning

Since plan v2.33 W3 the measuring side keeps no verdict mirror: the reported set is the rows whose core verdict bit is set, and an echoed pair that was never sent is an error, not a panic ([mod.rs:142-161](../../../cli/src/docdup/judge/mod.rs#L142)). The 80/100 ratio the measuring side reads (for the knob pin below) comes off the definition package's `limits` ([wire.rs:17-23](../../../cli/src/docdup/judge/wire.rs#L17)). The verdict boundary — `(4,5,0)` a duplicate, `(3,4,0)` not, `(0,100,50)` a duplicate on the verbatim half alone, `(0,100,49)` not — is the core's `dupVerdict` ([Cost.hs:152-163](../../../core/app/CE/Docdup/Cost.hs#L152)); the frozen Rust mirror survives only as the differential oracle in the tests subrepo (`unit/w3_oracle/`).

Every reply's `knobs` block is pinned against the measuring side's copies — all seven mirrored numbers, not the four the family is usually described by: `jaccardNum` 80, `jaccardDen` 100, `shingleK == doc_shingle` (5), `verbatimFloor == verbatim_floor` (50), `minDocTokens == min_doc_tokens` (50), `docLineCap == doc_line_cap` (200), `licHeadLines == license_head_lines` (5) ([wire.rs:51-77](../../../cli/src/docdup/judge/wire.rs#L51), echoed from [Docdup.hs:201-210](../../../core/app/CE/Docdup.hs#L201)). The last five are no Rust constants since plan v2.32 step 2: the measuring side reads them from the definition package, which the core builds from these same constants — the core owns them. `shingleK` is echoed, never used to shingle, by the core: sets and sequences arrive pre-shingled (the core reads it only to convert a run of shingles to words), and the constant exists on the wire solely as the protocol's record of the alphabet geometry — two sides shingling at different widths would compare incommensurable alphabets and no downstream gate could tell ([Cost.hs:37-48](../../../core/app/CE/Docdup/Cost.hs#L37), drift test at [unit/docdup/judge/wire.rs:23-31](../../../cli/tests/unit/docdup/judge/wire.rs#L23)).
