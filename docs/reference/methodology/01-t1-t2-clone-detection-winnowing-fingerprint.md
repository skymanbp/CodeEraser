# T1/T2 clone detection — winnowing fingerprint index

[index](../methodology.md) · [→ 02 T3 near-miss clones — Tree Edit Distance (TSED)](02-t3-near-miss-clones-tree-edit-distance-tsed.md)

The hot path is fixed by ADR-005: normalized token stream → winnowing/Rabin-Karp fingerprints over a SQLite inverted index, with the Schleimer et al. SIGMOD'03 no-miss lower bound as the correctness contract; the cold path (AST structural fingerprint → TSED, T3) is separate ([DEVELOPMENT_PLAN.md:199-203](../../DEVELOPMENT_PLAN.md#L199)). The whole computation is deterministic: the same bytes and the same `Params` produce the same fingerprint rows, the same anchor set, and the same blocks in the same order.

Stages: parse → normalize to tokens → hash tokens → k-gram rolling hash → per-window minimum selection → index → anchor pairing → exact bidirectional extension → diversity floor.

### 1. Token normalization alphabet

Tokenization walks the tree-sitter tree and emits one `Token { hash, start_line, end_line }` per **leaf** ([tokens.rs:31-36](../../../cli/src/dedup/tokens.rs#L31), [tokens.rs:65-98](../../../cli/src/dedup/tokens.rs#L65)). The walk uses `ast::children`, which enumerates `child_count()` and therefore includes **anonymous** nodes ([ast.rs:43-45](../../../cli/src/scan/ast.rs#L43)) — punctuation and keywords are part of the alphabet, not discarded. Comment nodes are skipped as whole subtrees ([tokens.rs:70-72](../../../cli/src/dedup/tokens.rs#L70)).

Each leaf is classified into exactly three classes ([tokens.rs:150-158](../../../cli/src/dedup/tokens.rs#L150)):

| Class | Predicate | Bytes hashed |
|---|---|---|
| `Id` | `kind.ends_with("identifier")` ([tokens.rs:151](../../../cli/src/dedup/tokens.rs#L151)) | `ID_MARK = b"\x01ID"` ([tokens.rs:38](../../../cli/src/dedup/tokens.rs#L38)) |
| `Lit` | `is_literal(kind, spec)` ([tokens.rs:154](../../../cli/src/dedup/tokens.rs#L154)) | `LIT_MARK = b"\x02LIT"` ([tokens.rs:39](../../../cli/src/dedup/tokens.rs#L39)) |
| `Text` | otherwise ([tokens.rs:157](../../../cli/src/dedup/tokens.rs#L157)) | the node's **kind text**, `kind.as_bytes()` ([tokens.rs:105](../../../cli/src/dedup/tokens.rs#L105)) |

Collapsing identifiers and literals to two fixed marks while keeping every other node's kind text verbatim is exactly the T2 equivalence: renamed variables and changed constants are clones, changed syntax is not. The `\x01` / `\x02` prefixes keep the marks outside the space of real grammar kind strings, so no kind text can alias them.

`is_literal` sees leaf kinds and the named descendants of a string-shaped literal (composite kinds such as Go `composite_literal` never reach it, because `tokenize` recurses on any node with children first — [tokens.rs:78-81](../../../cli/src/dedup/tokens.rs#L78), [tokens.rs:164-167](../../../cli/src/dedup/tokens.rs#L164)). It accepts ([tokens.rs:168-185](../../../cli/src/dedup/tokens.rs#L168)):

- `kind.ends_with("literal")`;
- `kind ∈ {"integer", "float", "number", "escape_sequence", "raw_string_delimiter", "literal_suffix"}`;
- `kind.contains("string")` **and** `kind` ends with one of `content` / `fragment` / `start` / `end`;
- `kind ∈ spec.literal_delims` — the per-language anonymous delimiter tokens.

`literal_delims` is per-language because a delimiter in one grammar is an operator in another: TypeScript `["\"", "'", "`"]` ([spec_launch.rs:111](../../../cli/src/scan/spec_launch.rs#L111)), Go `["\"", "`"]` — `'` is the rune delimiter but `rune_literal` lexes as one token ([spec_launch.rs:217-218](../../../cli/src/scan/spec_launch.rs#L217)), Rust `["\""]` only, because `'` is the lifetime/label tick and classifying it as `LIT` made every `&'a str` signature a false clone driver ([spec_launch.rs:161-163](../../../cli/src/scan/spec_launch.rs#L161)), Python `[]` because quotes surface as named `string_start` / `string_end` kinds ([spec_launch.rs:50-51](../../../cli/src/scan/spec_launch.rs#L50)), Haskell `[]` because a string lexes as one `string` leaf with the quotes inside ([spec_hs.rs:81-83](../../../cli/src/scan/spec_hs.rs#L81)). The v2.30 grammars follow the same reading: C and C++ share one table — `"`, `'`, `character` and the raw-string opener `R"` — with a string, char or user-defined literal node read whole as one `LIT` ([spec_c.rs:122](../../../cli/src/scan/spec_c.rs#L122), [tokens.rs:132-148](../../../cli/src/dedup/tokens.rs#L132)); Java `"` and the text-block `"""` ([spec_java.rs:88](../../../cli/src/scan/spec_java.rs#L88)); Lua `"`, `'`, `[[` and `]]` ([spec_lua.rs:72](../../../cli/src/scan/spec_lua.rs#L72)); R the named `string_open` / `string_close` kinds ([spec_r.rs:71](../../../cli/src/scan/spec_r.rs#L71)); HTML takes Markdown's empty table and is never tokenized ([spec.rs:236](../../../cli/src/scan/spec.rs#L236), [spec.rs:253-256](../../../cli/src/scan/spec.rs#L253)).

Two deliberate stances, documented at the module head ([tokens.rs:7-9](../../../cli/src/dedup/tokens.rs#L7)): booleans and `None` are **not** collapsed to `LIT` (their identity is usually semantic, unlike numbers and strings), and Go `blank_identifier` normalizes to `ID` like any identifier.

Multi-piece literals collapse to one token, but **only within the same parent literal node** — `lit_parent` is the parent node id, and a piece whose parent matches merely extends the previous token's `end_line` ([tokens.rs:83-91](../../../cli/src/dedup/tokens.rs#L83), [tokens.rs:114-118](../../../cli/src/dedup/tokens.rs#L114)). Merging by lexical adjacency instead swallowed whole statements (a Python attribute docstring after a string assignment vanished into the previous `LIT`'s span). A string-shaped literal node — a string, char or user-defined literal none of whose named descendants is anything but a literal piece — is one token before any piece is visited ([tokens.rs:73-81](../../../cli/src/dedup/tokens.rs#L73), [tokens.rs:132-148](../../../cli/src/dedup/tokens.rs#L132)): a C++ raw, delimited or prefixed string and a user-defined literal weigh what a plain string weighs (plan v2.30 step 5b; a raw string used to weigh five), while a literal holding code — a Java `\{x}` interpolation — fails the test and keeps the piece rule.

Token hashing is FNV-1a over those bytes: `h = 0xcbf29ce484222325`, then per byte `h ^= b; h = h * 0x100000001b3` in wrapping u64 arithmetic ([tokens.rs:188-195](../../../cli/src/dedup/tokens.rs#L188)). It is dependency-free and stable across runs, which is required because the index persists.

Normalization semantics are versioned: <!--ce:ver:tokenizer_rev#digits-->`TOKENIZER_REV = 4`<!--/ce--> ([tokens.rs:28](../../../cli/src/dedup/tokens.rs#L28)) is stored in the index meta table and a mismatch clears every parser-derived table while the trend history stays — stamped with the revision that measured it and remeasured before reuse ([parser.rs:26](../../../cli/src/dedup/schema/parser.rs#L26), [mod.rs:4-8](../../../cli/src/trend/mod.rs#L4)) — so fingerprints from an older tokenizer can never mix with new ones ([schema.rs:112-122](../../../cli/src/dedup/schema.rs#L112), [schema.rs:182-204](../../../cli/src/dedup/schema.rs#L182)).

Languages that do not fingerprint (`Lang::fingerprints() == false` — a grammar-less language such as Markdown or plain text — the scan-only arm never reaches the index at all ([lang.rs:195-197](../../../cli/src/scan/lang.rs#L195)) — and HTML, whose grammar feeds its sections, sites and page text but whose leaves carry no identifiers, so any two pages would hash into one clone pair; [lang.rs:268-270](../../../cli/src/scan/lang.rs#L268)) get no token stream and therefore **zero** fingerprint rows ([index.rs:145-162](../../../cli/src/dedup/index.rs#L145)). Fingerprints exist for Python, TypeScript, TSX, Rust, Go, Haskell, C, C++, Java, Lua and R.

### 2. k-gram rolling hash

Winnowing consumes only the `hash` field of the token stream ([index.rs:150-151](../../../cli/src/dedup/index.rs#L150)). Let `t[0..n]` be that sequence and `k = p.kgram`.

- `BASE = 1_000_003` ([winnow.rs:17](../../../cli/src/dedup/winnow.rs#L17)), all arithmetic wrapping u64.
- If `n < k`, the hash list is empty and the file contributes no fingerprints ([winnow.rs:29-31](../../../cli/src/dedup/winnow.rs#L29)).
- `top = BASE^(k-1)` (wrapping) ([winnow.rs:32](../../../cli/src/dedup/winnow.rs#L32)).
- Seed: `h_0 = ((t[0]*BASE + t[1])*BASE + … )*BASE + t[k-1]`, i.e. `h = h*BASE + t[j]` for `j` in `[0, k)` ([winnow.rs:34-38](../../../cli/src/dedup/winnow.rs#L34)).
- Roll, for `i` in `[k, n)`: `h = (h - t[i-k]*top)*BASE + t[i]` ([winnow.rs:39-45](../../../cli/src/dedup/winnow.rs#L39)).

This yields exactly `n - k + 1` k-gram hashes ([winnow.rs:33](../../../cli/src/dedup/winnow.rs#L33)). It is the standard Rabin-Karp update; the same function is `pub(crate)` and reused by docdup's word shingles so the offline oracle and the product filter cannot fork into two Rabin-Karps ([winnow.rs:19-24](../../../cli/src/dedup/winnow.rs#L19)).

### 3. Window minimum selection and the no-miss guarantee

Over the k-gram hash array `g[0..m]` with `w = p.window`:

- Window count: `windows = max(m - (w - 1), 1)` ([winnow.rs:64](../../../cli/src/dedup/winnow.rs#L64)). The `.max(1)` means a stream with fewer than `w` k-grams still gets one window, truncated by `end = min(start + w, m)` ([winnow.rs:66](../../../cli/src/dedup/winnow.rs#L66)).
- Within window `[w_i, end)` take the minimum, scanning left to right with `<=` so **ties resolve to the rightmost** occurrence ([winnow.rs:67-72](../../../cli/src/dedup/winnow.rs#L67)).
- A position is recorded **once**: if the chosen `min_idx` equals `last_recorded`, nothing is pushed ([winnow.rs:63](../../../cli/src/dedup/winnow.rs#L63), [winnow.rs:73-79](../../../cli/src/dedup/winnow.rs#L73)). Consecutive windows sharing a minimum therefore contribute one fingerprint, not `w`.

A selected `Fingerprint { hash, start }` covers tokens `[start, start + kgram)` ([winnow.rs:10-15](../../../cli/src/dedup/winnow.rs#L10)).

The two thresholds ([mod.rs:291-294](../../../cli/src/dedup/mod.rs#L291), [winnow.rs:4-6](../../../cli/src/dedup/winnow.rs#L4)):

- **Guarantee threshold** `t = window + kgram - 1` ([mod.rs:301-309](../../../cli/src/dedup/mod.rs#L301)). Any common substring of at least `t` normalized tokens contains at least `t - k + 1 = w` consecutive k-grams — one complete window — and since selection depends only on the contents of that window, both copies select the same minimum. Hence **≥ 1 shared fingerprint, always**. The positional dedup does not weaken this: it suppresses only a re-record of a position already emitted.
- **Noise threshold** `k = kgram`: no match shorter than `kgram` tokens can ever be reported, because a fingerprint is a whole k-gram.

Defaults: `kgram = 25`, `window = 26`, so `t = 26 + 25 - 1 = 50` tokens — chosen to align with the jscpd min-tokens default ([mod.rs:309-314](../../../cli/src/dedup/mod.rs#L309)). The report filter defaults to exactly `p.guarantee()` ([mod.rs:155-158](../../../cli/src/dedup/mod.rs#L155)); lowering it with `--min-tokens` is a calibration mode, and detection below `t` is opportunistic rather than guaranteed ([mod.rs:54-57](../../../cli/src/dedup/mod.rs#L54)).

### 4. The inverted index

Fingerprints land in `fingerprints(hash, file_id, start_tok, start_line, end_line)` with `idx_fp_hash` and `idx_fp_file`, cascade-deleted from `files` ([schema.rs:89-97](../../../cli/src/dedup/schema.rs#L89)). Line mapping at insert time is `start_line = toks[f.start].start_line` and `end_line = toks[f.start + p.kgram - 1].end_line` ([index.rs:394-395](../../../cli/src/dedup/index.rs#L394)) — the span of the k-gram, not of one token.

Invalidation is content-hash gated per file: `content_hash = fnv1a(src)`, and a match short-circuits the refresh entirely ([index.rs:121](../../../cli/src/dedup/index.rs#L121), [index.rs:137-139](../../../cli/src/dedup/index.rs#L137)); a change deletes and reinserts only that file's rows in one transaction ([index.rs:152-185](../../../cli/src/dedup/index.rs#L152)). The whole database is keyed by <!--ce:ver:schema.index#digits-->`SCHEMA_VERSION = 17`<!--/ce--> ([schema.rs:62](../../../cli/src/dedup/schema.rs#L62)) plus the meta tuple `(kgram, window, tokenizer_rev, graph_rev, struct_rev, docdup_rev, similar_rev)` ([schema.rs:191-201](../../../cli/src/dedup/schema.rs#L191)); any mismatch wipes and rebuilds, so a parameter change cannot silently reuse stale fingerprints.

Instance queries sort their rows before returning ([index.rs:324](../../../cli/src/dedup/index.rs#L324), [index.rs:342](../../../cli/src/dedup/index.rs#L342)), so downstream pairing sees a fixed order regardless of SQLite's row order.

### 5. Anchor pairing

Shared fingerprints are **candidate anchors only** — nothing is reported on a hash match alone ([pairs.rs:1-7](../../../cli/src/dedup/pairs.rs#L1)). Instances are grouped by `hash` in a `BTreeMap` and only groups of size > 1 are visited ([pairs.rs:41-45](../../../cli/src/dedup/pairs.rs#L41)):

- Group size `n <= HOT_CAP` → full pairwise, `C(n,2)` pairs ([pairs.rs:52-58](../../../cli/src/dedup/pairs.rs#L52)).
- Group size `n > HOT_CAP` → sort by `(file, start_tok)` and emit the `n-1` adjacent pairs, counting one `Chained` event ([pairs.rs:46-51](../../../cli/src/dedup/pairs.rs#L46)).

`HOT_CAP = 64` ([pairs.rs:27](../../../cli/src/dedup/pairs.rs#L27)). Chaining rather than skipping is load-bearing: skipping hot groups made detection fall to **zero as duplication rose** — 65 identical files produced 0 blocks — while the chain keeps every instance in at least one verified pair at linear cost ([pairs.rs:20-26](../../../cli/src/dedup/pairs.rs#L20)). One grouping walk serves both the T1/T2 extension pass and the S3 candidate source, so the two cannot disagree about which anchors exist ([pairs.rs:36-39](../../../cli/src/dedup/pairs.rs#L36)).

### 6. Anchor extension (verification)

Each pair is normalized to `(a, b)` ordered by `(file, start_tok)` ([pairs.rs:231-235](../../../cli/src/dedup/pairs.rs#L231)), then verified against the two live token streams. A stored offset past the end of the live stream means the file changed after the index refresh: the anchor is skipped and counted in `stale_skipped`, never allowed to index out of bounds ([pairs.rs:242-247](../../../cli/src/dedup/pairs.rs#L242), [pairs.rs:88-91](../../../cli/src/dedup/pairs.rs#L88)).

`extend` computes the maximal **exact** common run around the anchor on token hashes ([pairs.rs:262-272](../../../cli/src/dedup/pairs.rs#L262)):

1. Backward: while `a0 > 0 && b0 > 0 && sa[a0-1].hash == sb[b0-1].hash`, decrement both ([pairs.rs:270-273](../../../cli/src/dedup/pairs.rs#L270)).
2. Forward: `cap = b0 - a0` when both sides are the same stream, else `usize::MAX` ([pairs.rs:275](../../../cli/src/dedup/pairs.rs#L275)); grow `len` while `a0+len < |sa|`, `b0+len < |sb|`, `len < cap`, and `sa[a0+len].hash == sb[b0+len].hash` ([pairs.rs:276-282](../../../cli/src/dedup/pairs.rs#L276)).
3. Return `(a0, b0, len)` when `len > 0` ([pairs.rs:283](../../../cli/src/dedup/pairs.rs#L283)).

The same-stream cap keeps the two ranges disjoint — periodic code reports adjacent segments instead of one self-overlapping range ([pairs.rs:257-261](../../../cli/src/dedup/pairs.rs#L257)).

Runs are then sorted into two sinks ([pairs.rs:248-254](../../../cli/src/dedup/pairs.rs#L248)): `len >= t` is a reportable run; `near_floor <= len < t` goes to the near-miss sink, which with the floor at `kgram` is exactly `25 <= len < 50` and is the T3 candidate source S1 ([pairs.rs:153-162](../../../cli/src/dedup/pairs.rs#L153)). `near_floor = usize::MAX` disables it ([pairs.rs:136-138](../../../cli/src/dedup/pairs.rs#L136)).

Reportable runs are mapped to lines via the run's endpoint tokens: `a_start = sa[a0].start_line`, `a_end = sa[a0+len-1].end_line` ([pairs.rs:294-300](../../../cli/src/dedup/pairs.rs#L294)).

Because periodic content yields one maximal run per offset, `dominant` drops any block whose **both** ranges sit inside a longer block of the same file pair: sort by descending `tokens`, keep a block only if no kept block contains it on both sides, then re-sort by `(a_file, a_start, b_file, b_start)` for a stable report order ([pairs.rs:207-228](../../../cli/src/dedup/pairs.rs#L207)).

### 7. The `min_distinct` low-diversity floor

`distinct` is the cardinality of the set of token hashes inside the verified run: `|{ sa[i].hash : i ∈ [a0, a0+len) }|` ([pairs.rs:289-293](../../../cli/src/dedup/pairs.rs#L289)). It is the literal-degeneracy signal: a data-row match such as a `LIT: (LIT, ...),` table has a tiny alphabet, while a real code clone is diverse ([pairs.rs:73-78](../../../cli/src/dedup/pairs.rs#L73)).

`DEFAULT_MIN_DISTINCT = 7` ([pairs.rs:118](../../../cli/src/dedup/pairs.rs#L118)). The calibration: across the fixture corpus plus cobra and requests, the arbitrated data-row false positives (status_codes rows, locale key sections, pygments style dicts) measured `distinct <= 6`, while arbitrated true clones measured `distinct >= 7` ([pairs.rs:102-107](../../../cli/src/dedup/pairs.rs#L102)). The same comment records that one 16-outlier false positive survives the floor — it buys precision, not purity.

The floor is applied **after** `dominant`, and the number of suppressed blocks is reported as `low_diversity_suppressed` rather than discarded silently ([pairs.rs:180-182](../../../cli/src/dedup/pairs.rs#L180), [pairs.rs:92-94](../../../cli/src/dedup/pairs.rs#L92)); `--min-distinct` overrides it ([mod.rs:45](../../../cli/src/dedup/mod.rs#L45), [mod.rs:157](../../../cli/src/dedup/mod.rs#L157)) — on the `--check` road only in the tightening direction: `--min-tokens` above 50, `--min-distinct` above 7, or `0`, are refused by name before any measurement or core contact ([budget.rs:21](../../../cli/src/dedup/budget.rs#L21)), so the ratchet always judges at the calibrated operating point or a stricter one. Note that the floor filters blocks only — the near-miss sink keeps its own honest `distinct` but is not filtered by it ([pairs.rs:159-162](../../../cli/src/dedup/pairs.rs#L159)).

### 8. What the report carries

<!--ce:report:dedup#schemaver-->`ce.dedup-report/0.5.0`<!--/ce--> ([mod.rs:219](../../../cli/src/dedup/mod.rs#L219)) emits the blocks, the k-way groups aggregated from them, and a summary that restates the operating point — `kgram`, `window`, `min_tokens`, `min_distinct` — beside the transparency counters `hot_chained`, `stale_skipped`, and `low_diversity_suppressed` ([mod.rs:219-234](../../../cli/src/dedup/mod.rs#L219), [mod.rs:275-289](../../../cli/src/dedup/mod.rs#L275), [report.rs:12-18](../../../cli/src/dedup/report.rs#L12)). Every approximation the pipeline makes is therefore a number in the output, not an assumption in the reader's head.

**Reproduced by the product (plan v2.29 step 9, O45):** the per-corpus breakdown behind `distinct <= 6` vs `>= 7` is re-measured rather than quoted — `cli/tests/it/eval_dedup_distinct.rs` runs `dedup::analyze` with the floor off over the tracked crosscheck fixtures on every CI run, and over the four pinned corpora under `CE_BLESS=1 cargo test --release --test it -- --ignored eval_dedup_distinct::regenerate --nocapture`; the histogram, the count the shipped floor suppresses and the suppressed blocks themselves are frozen in `contracts/eval/dedup-distinct-v1.json` and rendered as the table in [DEDUP-CALIBRATION.md § Reproduction](../../../contracts/fixtures/crosscheck/DEDUP-CALIBRATION.md). The instrument reproduces the numbers; which suppressed block was a data row remains the 2026-08-07 arbitration's reading.
