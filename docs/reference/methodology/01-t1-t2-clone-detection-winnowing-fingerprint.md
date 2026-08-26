# T1/T2 clone detection — winnowing fingerprint index

[index](../methodology.md) · [→ 02 T3 near-miss clones — Tree Edit Distance (TSED)](02-t3-near-miss-clones-tree-edit-distance-tsed.md)

The hot path is fixed by ADR-005: normalized token stream → winnowing/Rabin-Karp fingerprints over a SQLite inverted index, with the Schleimer et al. SIGMOD'03 no-miss lower bound as the correctness contract; the cold path (AST structural fingerprint → TSED, T3) is separate ([DEVELOPMENT_PLAN.md:199](../../DEVELOPMENT_PLAN.md#L199)–[203](../../DEVELOPMENT_PLAN.md#L203)). The whole computation is deterministic: the same bytes and the same `Params` produce the same fingerprint rows, the same anchor set, and the same blocks in the same order.

Stages: parse → normalize to tokens → hash tokens → k-gram rolling hash → per-window minimum selection → index → anchor pairing → exact bidirectional extension → diversity floor.

### 1. Token normalization alphabet

Tokenization walks the tree-sitter tree and emits one `Token { hash, start_line, end_line }` per **leaf** ([tokens.rs:23](../../../cli/src/dedup/tokens.rs#L23)–[28](../../../cli/src/dedup/tokens.rs#L28), [tokens.rs:57](../../../cli/src/dedup/tokens.rs#L57)–[86](../../../cli/src/dedup/tokens.rs#L86)). The walk uses `ast::children`, which enumerates `child_count()` and therefore includes **anonymous** nodes ([ast.rs:35](../../../cli/src/scan/ast.rs#L35)–[40](../../../cli/src/scan/ast.rs#L40)) — punctuation and keywords are part of the alphabet, not discarded. Comment nodes are skipped as whole subtrees ([tokens.rs:62](../../../cli/src/dedup/tokens.rs#L62)–[64](../../../cli/src/dedup/tokens.rs#L64)).

Each leaf is classified into exactly three classes ([tokens.rs:107](../../../cli/src/dedup/tokens.rs#L107)–[115](../../../cli/src/dedup/tokens.rs#L115)):

| Class | Predicate | Bytes hashed |
|---|---|---|
| `Id` | `kind.ends_with("identifier")` ([tokens.rs:108](../../../cli/src/dedup/tokens.rs#L108)) | `ID_MARK = b"\x01ID"` ([tokens.rs:30](../../../cli/src/dedup/tokens.rs#L30)) |
| `Lit` | `is_literal(kind, spec)` ([tokens.rs:111](../../../cli/src/dedup/tokens.rs#L111)) | `LIT_MARK = b"\x02LIT"` ([tokens.rs:31](../../../cli/src/dedup/tokens.rs#L31)) |
| `Text` | otherwise ([tokens.rs:114](../../../cli/src/dedup/tokens.rs#L114)) | the node's **kind text**, `kind.as_bytes()` ([tokens.rs:92](../../../cli/src/dedup/tokens.rs#L92)) |

Collapsing identifiers and literals to two fixed marks while keeping every other node's kind text verbatim is exactly the T2 equivalence: renamed variables and changed constants are clones, changed syntax is not. The `\x01` / `\x02` prefixes keep the marks outside the space of real grammar kind strings, so no kind text can alias them.

`is_literal` is leaf-kind only (composite kinds such as Go `composite_literal` never reach it, because `tokenize` recurses on any node with children first — [tokens.rs:65](../../../cli/src/dedup/tokens.rs#L65)–[68](../../../cli/src/dedup/tokens.rs#L68), [tokens.rs:120](../../../cli/src/dedup/tokens.rs#L120)–[121](../../../cli/src/dedup/tokens.rs#L121)). It accepts ([tokens.rs:122](../../../cli/src/dedup/tokens.rs#L122)–[131](../../../cli/src/dedup/tokens.rs#L131)):

- `kind.ends_with("literal")`;
- `kind ∈ {"integer", "float", "number", "escape_sequence"}`;
- `kind.contains("string")` **and** `kind` ends with one of `content` / `fragment` / `start` / `end`;
- `kind ∈ spec.literal_delims` — the per-language anonymous delimiter tokens.

`literal_delims` is per-language because a delimiter in one grammar is an operator in another: TypeScript `["\"", "'", "`"]` ([spec.rs:174](../../../cli/src/scan/spec.rs#L174)), Go `["\"", "`"]` — `'` is the rune delimiter but `rune_literal` lexes as one token ([spec.rs:255](../../../cli/src/scan/spec.rs#L255)–[256](../../../cli/src/scan/spec.rs#L256)), Rust `["\""]` only, because `'` is the lifetime/label tick and classifying it as `LIT` made every `&'a str` signature a false clone driver ([spec.rs:213](../../../cli/src/scan/spec.rs#L213)–[215](../../../cli/src/scan/spec.rs#L215)), Python `[]` because quotes surface as named `string_start` / `string_end` kinds ([spec.rs:124](../../../cli/src/scan/spec.rs#L124)–[125](../../../cli/src/scan/spec.rs#L125)), Haskell `[]` because a string lexes as one `string` leaf with the quotes inside ([spec_hs.rs:69](../../../cli/src/scan/spec_hs.rs#L69)–[71](../../../cli/src/scan/spec_hs.rs#L71)).

Two deliberate stances, documented at the module head ([tokens.rs:6](../../../cli/src/dedup/tokens.rs#L6)–[8](../../../cli/src/dedup/tokens.rs#L8)): booleans and `None` are **not** collapsed to `LIT` (their identity is usually semantic, unlike numbers and strings), and Go `blank_identifier` normalizes to `ID` like any identifier.

Multi-piece literals collapse to one token, but **only within the same parent literal node** — `lit_parent` is the parent node id, and a piece whose parent matches merely extends the previous token's `end_line` ([tokens.rs:70](../../../cli/src/dedup/tokens.rs#L70)–[78](../../../cli/src/dedup/tokens.rs#L78), [tokens.rs:101](../../../cli/src/dedup/tokens.rs#L101)–[105](../../../cli/src/dedup/tokens.rs#L105)). Merging by lexical adjacency instead swallowed whole statements (a Python attribute docstring after a string assignment vanished into the previous `LIT`'s span).

Token hashing is FNV-1a over those bytes: `h = 0xcbf29ce484222325`, then per byte `h ^= b; h = h * 0x100000001b3` in wrapping u64 arithmetic ([tokens.rs:134](../../../cli/src/dedup/tokens.rs#L134)–[141](../../../cli/src/dedup/tokens.rs#L141)). It is dependency-free and stable across runs, which is required because the index persists.

Normalization semantics are versioned: `TOKENIZER_REV = 2` ([tokens.rs:20](../../../cli/src/dedup/tokens.rs#L20)) is stored in the index meta table and a mismatch wipes the database, so fingerprints from an older tokenizer can never mix with new ones ([schema.rs:84-94](../../../cli/src/dedup/schema.rs#L84), [schema.rs:146](../../../cli/src/dedup/schema.rs#L146)–[163](../../../cli/src/dedup/schema.rs#L163)).

Languages without a grammar (`Lang::grammar() == None` — Markdown and the scan-only arm, [lang.rs:116](../../../cli/src/scan/lang.rs#L116)–[114](../../../cli/src/scan/lang.rs#L114)) produce an empty token vector and therefore **zero** fingerprint rows ([index.rs:125](../../../cli/src/dedup/index.rs#L125)–[134](../../../cli/src/dedup/index.rs#L134)). Fingerprints exist for Python, TypeScript, TSX, Rust, Go, and Haskell.

### 2. k-gram rolling hash

Winnowing consumes only the `hash` field of the token stream ([index.rs:133](../../../cli/src/dedup/index.rs#L133)–[134](../../../cli/src/dedup/index.rs#L134)). Let `t[0..n]` be that sequence and `k = p.kgram`.

- `BASE = 1_000_003` ([winnow.rs:17](../../../cli/src/dedup/winnow.rs#L17)), all arithmetic wrapping u64.
- If `n < k`, the hash list is empty and the file contributes no fingerprints ([winnow.rs:29](../../../cli/src/dedup/winnow.rs#L29)–[31](../../../cli/src/dedup/winnow.rs#L31)).
- `top = BASE^(k-1)` (wrapping) ([winnow.rs:32](../../../cli/src/dedup/winnow.rs#L32)).
- Seed: `h_0 = ((t[0]*BASE + t[1])*BASE + … )*BASE + t[k-1]`, i.e. `h = h*BASE + t[j]` for `j` in `[0, k)` ([winnow.rs:34](../../../cli/src/dedup/winnow.rs#L34)–[38](../../../cli/src/dedup/winnow.rs#L38)).
- Roll, for `i` in `[k, n)`: `h = (h - t[i-k]*top)*BASE + t[i]` ([winnow.rs:39](../../../cli/src/dedup/winnow.rs#L39)–[45](../../../cli/src/dedup/winnow.rs#L45)).

This yields exactly `n - k + 1` k-gram hashes ([winnow.rs:33](../../../cli/src/dedup/winnow.rs#L33)). It is the standard Rabin-Karp update; the same function is `pub(crate)` and reused by docdup's word shingles so the offline oracle and the product filter cannot fork into two Rabin-Karps ([winnow.rs:19](../../../cli/src/dedup/winnow.rs#L19)–[24](../../../cli/src/dedup/winnow.rs#L24)).

### 3. Window minimum selection and the no-miss guarantee

Over the k-gram hash array `g[0..m]` with `w = p.window`:

- Window count: `windows = max(m - (w - 1), 1)` ([winnow.rs:64](../../../cli/src/dedup/winnow.rs#L64)). The `.max(1)` means a stream with fewer than `w` k-grams still gets one window, truncated by `end = min(start + w, m)` ([winnow.rs:66](../../../cli/src/dedup/winnow.rs#L66)).
- Within window `[w_i, end)` take the minimum, scanning left to right with `<=` so **ties resolve to the rightmost** occurrence ([winnow.rs:67](../../../cli/src/dedup/winnow.rs#L67)–[72](../../../cli/src/dedup/winnow.rs#L72)).
- A position is recorded **once**: if the chosen `min_idx` equals `last_recorded`, nothing is pushed ([winnow.rs:63](../../../cli/src/dedup/winnow.rs#L63), [winnow.rs:73](../../../cli/src/dedup/winnow.rs#L73)–[79](../../../cli/src/dedup/winnow.rs#L79)). Consecutive windows sharing a minimum therefore contribute one fingerprint, not `w`.

A selected `Fingerprint { hash, start }` covers tokens `[start, start + kgram)` ([winnow.rs:10](../../../cli/src/dedup/winnow.rs#L10)–[15](../../../cli/src/dedup/winnow.rs#L15)).

The two thresholds ([mod.rs:350](../../../cli/src/dedup/mod.rs#L350)–[334](../../../cli/src/dedup/mod.rs#L334), [winnow.rs:4](../../../cli/src/dedup/winnow.rs#L4)–[6](../../../cli/src/dedup/winnow.rs#L6)):

- **Guarantee threshold** `t = window + kgram - 1` ([mod.rs:361](../../../cli/src/dedup/mod.rs#L361)–[365](../../../cli/src/dedup/mod.rs#L365)). Any common substring of at least `t` normalized tokens contains at least `t - k + 1 = w` consecutive k-grams — one complete window — and since selection depends only on the contents of that window, both copies select the same minimum. Hence **≥ 1 shared fingerprint, always**. The positional dedup does not weaken this: it suppresses only a re-record of a position already emitted.
- **Noise threshold** `k = kgram`: no match shorter than `kgram` tokens can ever be reported, because a fingerprint is a whole k-gram.

Defaults: `kgram = 25`, `window = 26`, so `t = 26 + 25 - 1 = 50` tokens — chosen to align with the jscpd min-tokens default ([mod.rs:368](../../../cli/src/dedup/mod.rs#L368)–[377](../../../cli/src/dedup/mod.rs#L377)). The report filter defaults to exactly `p.guarantee()` ([mod.rs:148](../../../cli/src/dedup/mod.rs#L148)–[146](../../../cli/src/dedup/mod.rs#L146)); lowering it with `--min-tokens` is a calibration mode, and detection below `t` is opportunistic rather than guaranteed ([mod.rs:53](../../../cli/src/dedup/mod.rs#L53)–[56](../../../cli/src/dedup/mod.rs#L56)).

### 4. The inverted index

Fingerprints land in `fingerprints(hash, file_id, start_tok, start_line, end_line)` with `idx_fp_hash` and `idx_fp_file`, cascade-deleted from `files` ([schema.rs:56](../../../cli/src/dedup/schema.rs#L56)–[58](../../../cli/src/dedup/schema.rs#L58)). Line mapping at insert time is `start_line = toks[f.start].start_line` and `end_line = toks[f.start + p.kgram - 1].end_line` ([index.rs:331](../../../cli/src/dedup/index.rs#L331)–[332](../../../cli/src/dedup/index.rs#L332)) — the span of the k-gram, not of one token.

Invalidation is content-hash gated per file: `content_hash = fnv1a(src)`, and a match short-circuits the refresh entirely ([index.rs:112](../../../cli/src/dedup/index.rs#L112), [index.rs:122](../../../cli/src/dedup/index.rs#L122)–[125](../../../cli/src/dedup/index.rs#L125)); a change deletes and reinserts only that file's rows in one transaction ([index.rs:135](../../../cli/src/dedup/index.rs#L135)–[158](../../../cli/src/dedup/index.rs#L158)). The whole database is keyed by `SCHEMA_VERSION = 12` ([schema.rs:35](../../../cli/src/dedup/schema.rs#L35)) plus the meta tuple `(kgram, window, tokenizer_rev, graph_rev, struct_rev, docdup_rev)` ([schema.rs:146](../../../cli/src/dedup/schema.rs#L146)–[147](../../../cli/src/dedup/schema.rs#L147)); any mismatch wipes and rebuilds, so a parameter change cannot silently reuse stale fingerprints.

Instance queries sort their rows before returning ([index.rs:266](../../../cli/src/dedup/index.rs#L266), [index.rs:280](../../../cli/src/dedup/index.rs#L280)), so downstream pairing sees a fixed order regardless of SQLite's row order.

### 5. Anchor pairing

Shared fingerprints are **candidate anchors only** — nothing is reported on a hash match alone ([pairs.rs:1](../../../cli/src/dedup/pairs.rs#L1)–[7](../../../cli/src/dedup/pairs.rs#L7)). Instances are grouped by `hash` in a `BTreeMap` and only groups of size > 1 are visited ([pairs.rs:41](../../../cli/src/dedup/pairs.rs#L41)–[45](../../../cli/src/dedup/pairs.rs#L45)):

- Group size `n <= HOT_CAP` → full pairwise, `C(n,2)` pairs ([pairs.rs:52](../../../cli/src/dedup/pairs.rs#L52)–[58](../../../cli/src/dedup/pairs.rs#L58)).
- Group size `n > HOT_CAP` → sort by `(file, start_tok)` and emit the `n-1` adjacent pairs, counting one `Chained` event ([pairs.rs:46](../../../cli/src/dedup/pairs.rs#L46)–[51](../../../cli/src/dedup/pairs.rs#L51)).

`HOT_CAP = 64` ([pairs.rs:27](../../../cli/src/dedup/pairs.rs#L27)). Chaining rather than skipping is load-bearing: skipping hot groups made detection fall to **zero as duplication rose** — 65 identical files produced 0 blocks — while the chain keeps every instance in at least one verified pair at linear cost ([pairs.rs:20](../../../cli/src/dedup/pairs.rs#L20)–[26](../../../cli/src/dedup/pairs.rs#L26)). One grouping walk serves both the T1/T2 extension pass and the S3 candidate source, so the two cannot disagree about which anchors exist ([pairs.rs:36](../../../cli/src/dedup/pairs.rs#L36)–[39](../../../cli/src/dedup/pairs.rs#L39)).

### 6. Anchor extension (verification)

Each pair is normalized to `(a, b)` ordered by `(file, start_tok)` ([pairs.rs:231](../../../cli/src/dedup/pairs.rs#L231)–[223](../../../cli/src/dedup/pairs.rs#L223)), then verified against the two live token streams. A stored offset past the end of the live stream means the file changed after the index refresh: the anchor is skipped and counted in `stale_skipped`, never allowed to index out of bounds ([pairs.rs:242](../../../cli/src/dedup/pairs.rs#L242)–[235](../../../cli/src/dedup/pairs.rs#L235), [pairs.rs:88](../../../cli/src/dedup/pairs.rs#L88)–[91](../../../cli/src/dedup/pairs.rs#L91)).

`extend` computes the maximal **exact** common run around the anchor on token hashes ([pairs.rs:262](../../../cli/src/dedup/pairs.rs#L262)–[272](../../../cli/src/dedup/pairs.rs#L272)):

1. Backward: while `a0 > 0 && b0 > 0 && sa[a0-1].hash == sb[b0-1].hash`, decrement both ([pairs.rs:270](../../../cli/src/dedup/pairs.rs#L270)–[261](../../../cli/src/dedup/pairs.rs#L261)).
2. Forward: `cap = b0 - a0` when both sides are the same stream, else `usize::MAX` ([pairs.rs:275](../../../cli/src/dedup/pairs.rs#L275)); grow `len` while `a0+len < |sa|`, `b0+len < |sb|`, `len < cap`, and `sa[a0+len].hash == sb[b0+len].hash` ([pairs.rs:276](../../../cli/src/dedup/pairs.rs#L276)–[270](../../../cli/src/dedup/pairs.rs#L270)).
3. Return `(a0, b0, len)` when `len > 0` ([pairs.rs:283](../../../cli/src/dedup/pairs.rs#L283)).

The same-stream cap keeps the two ranges disjoint — periodic code reports adjacent segments instead of one self-overlapping range ([pairs.rs:257](../../../cli/src/dedup/pairs.rs#L257)–[249](../../../cli/src/dedup/pairs.rs#L249)).

Runs are then sorted into two sinks ([pairs.rs:248](../../../cli/src/dedup/pairs.rs#L248)–[242](../../../cli/src/dedup/pairs.rs#L242)): `len >= t` is a reportable run; `near_floor <= len < t` goes to the near-miss sink, which with the floor at `kgram` is exactly `25 <= len < 50` and is the T3 candidate source S1 ([pairs.rs:153](../../../cli/src/dedup/pairs.rs#L153)–[152](../../../cli/src/dedup/pairs.rs#L152)). `near_floor = usize::MAX` disables it ([pairs.rs:136](../../../cli/src/dedup/pairs.rs#L136)–[129](../../../cli/src/dedup/pairs.rs#L129)).

Reportable runs are mapped to lines via the run's endpoint tokens: `a_start = sa[a0].start_line`, `a_end = sa[a0+len-1].end_line` ([pairs.rs:286](../../../cli/src/dedup/pairs.rs#L286)–[285](../../../cli/src/dedup/pairs.rs#L285)).

Because periodic content yields one maximal run per offset, `dominant` drops any block whose **both** ranges sit inside a longer block of the same file pair: sort by descending `tokens`, keep a block only if no kept block contains it on both sides, then re-sort by `(a_file, a_start, b_file, b_start)` for a stable report order ([pairs.rs:207](../../../cli/src/dedup/pairs.rs#L207)–[216](../../../cli/src/dedup/pairs.rs#L216)).

### 7. The `min_distinct` low-diversity floor

`distinct` is the cardinality of the set of token hashes inside the verified run: `|{ sa[i].hash : i ∈ [a0, a0+len) }|` ([pairs.rs:289](../../../cli/src/dedup/pairs.rs#L289)–[281](../../../cli/src/dedup/pairs.rs#L281)). It is the literal-degeneracy signal: a data-row match such as a `LIT: (LIT, ...),` table has a tiny alphabet, while a real code clone is diverse ([pairs.rs:73](../../../cli/src/dedup/pairs.rs#L73)–[78](../../../cli/src/dedup/pairs.rs#L78)).

`DEFAULT_MIN_DISTINCT = 7` ([pairs.rs:118](../../../cli/src/dedup/pairs.rs#L118)). The calibration: across the fixture corpus plus cobra and requests, the arbitrated data-row false positives (status_codes rows, locale key sections, pygments style dicts) measured `distinct <= 6`, while arbitrated true clones measured `distinct >= 7` ([pairs.rs:102](../../../cli/src/dedup/pairs.rs#L102)–[102](../../../cli/src/dedup/pairs.rs#L102)). The same comment records that one 16-outlier false positive survives the floor — it buys precision, not purity.

The floor is applied **after** `dominant`, and the number of suppressed blocks is reported as `low_diversity_suppressed` rather than discarded silently ([pairs.rs:180](../../../cli/src/dedup/pairs.rs#L180)–[182](../../../cli/src/dedup/pairs.rs#L182), [pairs.rs:92](../../../cli/src/dedup/pairs.rs#L92)–[94](../../../cli/src/dedup/pairs.rs#L94)); `--min-distinct` overrides it ([mod.rs:44](../../../cli/src/dedup/mod.rs#L44), [mod.rs:150](../../../cli/src/dedup/mod.rs#L150)). Note that the floor filters blocks only — the near-miss sink keeps its own honest `distinct` but is not filtered by it ([pairs.rs:159](../../../cli/src/dedup/pairs.rs#L159)–[152](../../../cli/src/dedup/pairs.rs#L152)).

### 8. What the report carries

`ce.dedup-report/0.5.0` ([mod.rs:36](../../../cli/src/dedup/mod.rs#L36)) emits the blocks, the k-way groups aggregated from them, and a summary that restates the operating point — `kgram`, `window`, `min_tokens`, `min_distinct` — beside the transparency counters `hot_chained`, `stale_skipped`, and `low_diversity_suppressed` ([mod.rs:212](../../../cli/src/dedup/mod.rs#L212)–[149](../../../cli/src/dedup/mod.rs#L149), [mod.rs:270](../../../cli/src/dedup/mod.rs#L270)–[265](../../../cli/src/dedup/mod.rs#L265)). Every approximation the pipeline makes is therefore a number in the output, not an assumption in the reader's head.

**Not found in the source this run:** the `min_distinct` calibration numbers are cited from the comment at [pairs.rs:102](../../../cli/src/dedup/pairs.rs#L102)–[102](../../../cli/src/dedup/pairs.rs#L102), which attributes them to `DEDUP-CALIBRATION.md`; that document was not read for this section, so the per-corpus breakdown behind `distinct <= 6` vs `>= 7` is not reproduced here.
