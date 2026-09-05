# Same-role advisor — sparse retrieval and in-repo association

[index](../methodology.md) · [← 14 Tombstone residue — the erased-name conjunction](14-tombstone-residue-the-erased-name-conjunction.md)

The three clone families read text: fingerprints (booklet 01), tree edit distance (02) and
shingles (03) all need the two units to *look* alike. This family answers a different
question — `file 1 has a function x; file 2 now has a function y that plays the same part`
— when y and x share no line. It is an **advisor**, on booklet 13's terms: it never reddens a
gate, never enters `ce erase`, and every face prints it as advice
([mod.rs:1-12](../../../cli/src/similar/mod.rs#L1)). Two things make it deterministic and
offline where a "code RAG" usually is neither: the retrieval is sparse — integer BM25 over
term bags read off facts the index already carries, not an embedding — and the only
association it knows is this repository's own, positive pointwise mutual information over the
same bags, opt-in and never evidence ([ppmi.rs:1-15](../../../cli/src/similar/ppmi.rs#L1)).
The split is ADR-008's, sixth instalment: Rust builds the bags, the inverted tables and a
query's top-K with six evidence integers per candidate; Haskell orders the candidates as exact
rationals and decides which of them play the query's role over the twelfth wire family,
`similar/1`. Names, words and paths never cross the wire — hashes and counts only, row index is
identity ([wire.rs:1-7](../../../cli/src/similar/wire.rs#L1),
[Similar.hs:5-17](../../../core/app/CE/Similar.hs#L5)).

### 1. The bag — six channels off facts the tree already carries

Every code unit of the unitsig universe — the T3 universe, `(file, key, nth)`, so Markdown has
no bags — gets one sparse bag ([bag.rs:1-9](../../../cli/src/similar/bag.rs#L1)). Six
channels, each read off a fact the parse already produced, each with a one-letter label that is
mixed into the term hash — a name word and a callee word spelled alike are two terms, so a
shared name is name evidence and a shared callee is callee evidence, and the role rule can read
them apart ([terms.rs:10-23](../../../cli/src/similar/terms.rs#L10)):

| channel | source fact | term |
|---|---|---|
| **N** name | the unit key | identifier pieces, stemmed |
| **P** shape | kind, arity, return presence of the declaration node | a feature spelling |
| **C** callee | the callee spellings in the unit's own body | identifier pieces, stemmed |
| **D** doc | the comment or docstring that belongs to the unit | prose words, stop list dropped, stemmed |
| **S** structure | the unitsig kind histogram | one feature per node kind, count as tf |
| **L** literal | the *kinds* of its literals — never the values | one feature per kind |

Identifier pieces fall at camel, underscore and digit boundaries and are lowercased
(`parseJSONFile` → parse json file, `http2_server` → http 2 server); prose splits through the
same function, so `parseJSON` in a comment meets `parse_json` in a name on the same terms
([terms.rs:76-81](../../../cli/src/similar/terms.rs#L76),
[terms.rs:119-126](../../../cli/src/similar/terms.rs#L119)). The stop list is a fixed table of
48 prose words, never learned from a corpus, and it does not touch identifier pieces — `get`,
`set` and `is` are what a role is made of ([terms.rs:66-69](../../../cli/src/similar/terms.rs#L66)).
Word channels are stemmed by Porter's 1980 algorithm and hashed; feature channels are hashed
as spelled ([stem.rs:12](../../../cli/src/similar/stem.rs#L12),
[terms.rs:129-138](../../../cli/src/similar/terms.rs#L129)). The doc channel takes the
segments docdup already extracts, attributed by position — a leading block ending within
`LEAD_GAP` lines above the unit's first line, or a head block within `HEAD_GAP` lines below it
([docs.rs:13-14](../../../cli/src/similar/docs.rs#L13),
[docs.rs:59](../../../cli/src/similar/docs.rs#L59)). Only channel-tagged fnv1a64 hashes leave
the term module: no word text is stored anywhere downstream, which is the index-privacy clause
the plan writes for every table `.ce/index.db` gains
([terms.rs:1-5](../../../cli/src/similar/terms.rs#L1)).

Query weights are integer multipliers — names ×3, callees ×2, everything else ×1 — so the
score stays exact ([terms.rs:49-56](../../../cli/src/similar/terms.rs#L49)). The whole term
road is declared once as `SIMILAR_REV` and sits in the index cache key: a change to any rule
above wipes the bag tables with the rest of the index rather than ranking old bags against new
queries ([mod.rs:33-43](../../../cli/src/similar/mod.rs#L33)).

### 2. The inverted tables — bags persisted as postings, pairs not stored

Two tables, index schema 16, holding only hashes and counts
([store.rs:1-6](../../../cli/src/similar/store.rs#L1),
[store.rs:45-58](../../../cli/src/similar/store.rs#L45)). `bag(term_hash, unit, tf, channel)`
is keyed by the unit's own `unitsig` row — the bag universe *is* the unitsig universe by
foreign key — and is a `WITHOUT ROWID` table on `(term_hash, unit)`, so the table *is* its
posting list: a term's units are one b-tree range. `df(term_hash, df, marg)` holds, per term,
the units carrying it and — for a word — the units counting it inside the association cap
(§4's marginal), with `CHECK` constraints refusing a negative count or a marginal past its df:
a drift between rows and aggregate fails by name instead of ranking on a wrong idf.

**What is deliberately not stored is the co-occurrence pair table** the spec first drew. On
this repository it held 688k rows, grew the index 5.4× and the cold index 7–10×, for a view
that is opt-in; so the reader derives a word's co-occurrence counts at query time from the bag
rows of the units that carry it — exactly the counts the in-memory table keeps
([store.rs:7-12](../../../cli/src/similar/store.rs#L7),
[reader.rs:1-12](../../../cli/src/similar/reader.rs#L1)). The tables move with the existing
refresh differential: inside `refresh_file`'s content-hash-gated transaction, `retire` tallies
the file's old bags at −1 before the unitsig rows are replaced and `refresh_bags` tallies the
new ones at +1 after, and only the non-zero *net* deltas reach SQL — an edit to one function
costs that unit's terms, never the corpus's; a foreign file (owner 1, measured by nobody)
writes no rows ([store.rs:12-24](../../../cli/src/similar/store.rs#L12),
[store.rs:68](../../../cli/src/similar/store.rs#L68),
[store.rs:79](../../../cli/src/similar/store.rs#L79)). The cost, measured on this tree of 687
files: cold `ce dedup` 5.2 → 8.4 s (0.65 s of a sixth parse, ≈1.5 s of random-key posting
writes that five layouts could not beat), warm unchanged, database 10.7 → 18.0 MB
([PERF-BUDGET.md:247-266](../../PERF-BUDGET.md#L247)).

### 3. Ranking — integer BM25, one road for the instrument and the product

Okapi BM25 with Robertson & Walker's usual `k1 = 6/5`, `b = 3/4` — kept as the rationals they
are and folded into one integer fraction. A term's contribution is

    w · idf · 22 · tf · avg / (10 · tf · avg + 3 · avg + 9 · len)

floored to 16-bit fixed point; the unit test re-derives the fraction from `K1` and `B`
([bm25.rs:21-27](../../../cli/src/similar/bm25.rs#L21),
[bm25.rs:243-248](../../../cli/src/similar/bm25.rs#L243)). `idf = log2((N − df + ½) / (df + ½))`
in 8-bit fixed point from an integer `log2` by squaring only, floored at zero
([bm25.rs:230-237](../../../cli/src/similar/bm25.rs#L230),
[bm25.rs:253](../../../cli/src/similar/bm25.rs#L253)). No float is touched anywhere, so the same
corpus ranks the same on every platform and the frozen evaluation rows compare byte for byte
([bm25.rs:1-7](../../../cli/src/similar/bm25.rs#L1)).

`top_k` returns the `K = 5` best candidates for a query, excluding the query's own seat,
ordered by score then identity. A term in more than half the units — idf 0 — is neither score
nor evidence: sharing what nearly everything shares says nothing, and walking its posting list
would cost the whole corpus per query, so df is asked first and the list is never fetched.
Shape equality and the role bit are read for the K survivors only, and neither orders
([bm25.rs:77-100](../../../cli/src/similar/bm25.rs#L77), [mod.rs:43](../../../cli/src/similar/mod.rs#L43)).
Ranking is written once, against the `Postings` trait: the in-memory `Corpus` the instruments
build and the persisted `Reader` over `.ce/index.db` both feed the same `top_k`, and the replay
asserts they agree on every unit of five corpora — the instrument and the product run one road
([bm25.rs:66](../../../cli/src/similar/bm25.rs#L66),
[reader.rs:7-12](../../../cli/src/similar/reader.rs#L7),
[similar_replay.rs:188](../../../cli/tests/it/similar_replay.rs#L188)). Query weights ride in
`1/W_UNIT` = 1/256, which is what lets a PPMI-scaled expansion keep a *fraction* of its
parent's weight without a float ([bm25.rs:25-27](../../../cli/src/similar/bm25.rs#L25)).

### 4. The associative view — positive PMI over this repository, opt-in

Two word terms co-occurring in one unit's bag are counted once per unit, and

    PPMI(a, b) = max(0, log2(n_ab · N / (n_a · n_b)))

in the same 8-bit fixed point as the idf, from the same integer `log2`
([ppmi.rs:1-6](../../../cli/src/similar/ppmi.rs#L1),
[ppmi.rs:65](../../../cli/src/similar/ppmi.rs#L65)). A neighbour counts only when it co-occurred
in at least `MIN_COOC = 2` units and carries at least `MIN_PPMI` = two bits of association;
each spelled word term of the query appends its `TOP_M = 3` best neighbours at weight
`parent × min(ppmi, PPMI_CAP) / PPMI_SCALE` — at most half the parent's weight — and a term the
query already spells is never appended ([ppmi.rs:21-35](../../../cli/src/similar/ppmi.rs#L21),
[ppmi.rs:79-100](../../../cli/src/similar/ppmi.rs#L79),
[ppmi.rs:102-122](../../../cli/src/similar/ppmi.rs#L102)). A unit past `TERM_CAP = 96` distinct
word terms contributes its first 96 in term order and is ledgered as capped; the cap has one
owner, so the in-memory table and the persisted writer count the same words
([ppmi.rs:32-42](../../../cli/src/similar/ppmi.rs#L32)). One bound does the pruning for free:
`n_ab ≤ n_b` bounds `PPMI(a, b)` by `log2(N / n_a)`, under two bits as soon as `4 · n_a > N`, so
the reader never walks such a word's pair rows ([ppmi.rs:74-83](../../../cli/src/similar/ppmi.rs#L74)).

This is enough to let this repository's `fetch / load / retrieve` meet, and never enough to
outvote what the unit itself spells. No corpus but this one is consulted and no word table is
written. The step-2 tuning verdict made the widened arm **an opt-in association view, never
the default and never evidence**: on the frozen sample it re-ranked the same 84 configurations
the way the bare arm did and crossed no significance line (widened 63/118 against bare 67/118
on the first generation), so the faces show the widened rows as a second page, tagged, and the
role bit is read off the six channels only
([EVAL-SET-SIMILAR.md:222](../../EVAL-SET-SIMILAR.md#L222),
[face.rs:44-45](../../../cli/src/similar/face.rs#L44)).

### 5. The wire — `similar/1`, and what Haskell judges

The request is the query bag as `[termHash, weight]` pairs — strictly ascending, may be empty —
plus one nine-integer row per candidate, `[nHit, pHit, cHit, dHit, sHit, lHit, shapeEqual,
bm25Num, bm25Den]`: the six channel hits, the shape bit, and the fixed-point score as a fraction
over its unit, so the core compares the ratio and never learns the width
([wire.rs:31-42](../../../cli/src/similar/wire.rs#L31),
[Cost.hs:29-31](../../../core/app/CE/Similar/Cost.hs#L29)). The reply is `order` — the candidate
indices by score descending as exact rationals, ties by request index — `roles`, one bit per
row in request order, and `counts{rows, queryTerms, role}`
([Similar.hs:95-101](../../../core/app/CE/Similar.hs#L95),
[Similar.hs:110-127](../../../core/app/CE/Similar.hs#L110)). The role rule is a two-arm
conjunction over one row, and it lives in Haskell:

    role ⇔ (nHit ≥ roleMinName ∧ cHit ≥ roleMinCallee) ∨ (nHit ≥ roleMinNameShape ∧ shapeEqual)

with floors 1, 1 and 2: a unit that is *called* the same and *calls* the same, or two name
words in common with the same signature shape ([Cost.hs:34-56](../../../core/app/CE/Similar/Cost.hs#L34)).
The measuring side's `role` in `bm25.rs` is the instrument's declared mirror of that rule for
the frozen evaluation rows; after the wire landed the measurement never decides alone
([bm25.rs:9-12](../../../cli/src/similar/bm25.rs#L9), [bm25.rs:225-228](../../../cli/src/similar/bm25.rs#L225)).
A request whose query terms plus rows exceed `similarCap` = 65536 gets a complete degraded
reply with empty tables and the reason `similar_too_large` — a query the core refused to judge
has no order and no roles, and the faces name the degradation instead of showing the measuring
side's order ([Cost.hs:26-27](../../../core/app/CE/Similar/Cost.hs#L26),
[Similar.hs:104-107](../../../core/app/CE/Similar.hs#L104)). There is no knob and no fail tier:
a knobless family whose one table is not the shared `RowsReq` — the query bag is its own key —
so it binds the cascade directly. On the Rust side `consume` is strict: the order must be a
permutation of the rows sent, one role bit per row, counts agreeing with the tables; any skew
is a *named* non-judgment, never conflated with "no candidates"
([wire.rs:82-101](../../../cli/src/similar/wire.rs#L82)). The family entered the protocol at
6.7.0, additively ([VERSIONING.md](../../VERSIONING.md)).

### 6. Three faces, one document, one Stop line

Every face renders one document, `ce.similar-report/0.1.0`: the query as
`{label, terms, widen}`, the candidates as rows `{at, key, nth, role, score, hits[6],
shape_equal, widened}` — the first five alphabetical scalars are what the GUI hub's generic
projection shows — the counts, and `degraded` naming why the core did not judge when it did not
([face.rs:26-45](../../../cli/src/similar/face.rs#L26), [face.rs:150](../../../cli/src/similar/face.rs#L150)).
A query is exactly one of three asks: `at` (`file:line`, the innermost unit holding the line),
`unit` (a key, refused by name when ambiguous, naming up to five places) or `text` (free text,
whose words become name and doc evidence — no shape, no callee, so the core's role bit is false
by construction) ([query.rs:16-53](../../../cli/src/similar/query.rs#L16),
[query.rs:61-94](../../../cli/src/similar/query.rs#L61)). `run` refreshes the index over the
same content-hash gate every command uses, resolves the ask, ranks the bare arm — and the widened
arm when asked, its rows not in the bare arm tagged `widened` — and rides one `similar/1`
request per arm over one core link ([face.rs:58-75](../../../cli/src/similar/face.rs#L58)).

- **CLI** `ce similar --at file:line | --text "…" | --unit key [--widen]`: a bilingual head
  line and one line per candidate, `at key  N P C D S L  role`; `--format json` is the document
  ([main_similar.rs:34](../../../cli/src/main_similar.rs#L34), [face.rs:165](../../../cli/src/similar/face.rs#L165)).
- **MCP** `similar_units` — the fifteenth read-only tool, `{at, text, unit, widen}`, relaying
  the same document ([tools.rs:163](../../../cli/src/mcp/tools.rs#L163)).
- **GUI** the eleventh screen, `similar`: an input for `at` or text, the widen switch, the
  candidate table with the six evidence columns ([similar.js](../../../gui/ui/similar.js),
  [commands.rs](../../../gui/src-tauri/src/commands.rs)).
- **Stop audit** — every unit the session *added* (a `(key, nth)` the working tree's file holds
  and `HEAD`'s did not) is asked of the index the way `ce similar` asks, and a row
  `{unit, twin, score}` is written into the feed's `similar` object only when the core's top-1
  carries the role bit: an advisor's line for the evaluation ledger, never a reason to block.
  No new unit, no role hit and nothing degraded = no key at all; the feed schema moved
  additively to `ce.observe/0.10.0` ([audit/similar.rs:1-10](../../../cli/src/audit/similar.rs#L1),
  [hookio.rs:85](../../../cli/src/hookio.rs#L85)). The tombstone leg and this leg read the
  session's changed pairs once, through one git batch (booklet 14 §1).

The write-time hook does **not** run it: a PreToolUse budget does not hold a retrieval, and a
family without a deny tier has nothing to say there (spec §二).

### 7. Evaluation — two oracle generations, two floors

There is no oracle that knows every same-role partner of a unit, so **recall is not reported**;
the ledger reports p@1 — the arm's top-1 arbitrated `same_role` — and hit@5, per arm and per
corpus, plus the confusion of the role bit over every candidate pair
([EVAL-SET-SIMILAR.md:77-80](../../EVAL-SET-SIMILAR.md#L77)). The instrument is
`similar_replay`: five corpora (this repository and the four cross-check fixtures) each become
their own database, every unit is queried against the rest of its corpus on both arms, and the
row identity is the sha256 of the text with CRLF folded to LF — the checkout must not change who
a line is ([similar_replay.rs:1-12](../../../cli/tests/it/similar_replay.rs#L1)). Samples are
drawn by sha256 order, arbitrated candidate by candidate as `same_role / related / unrelated`,
and frozen as oracles.

| generation | queries · pairs | p@1 bare | p@1 widened | p@1 bare, role = 1 | hit@5 bare | role-bit precision | floor |
|---|---|---|---|---|---|---|---|
| v1 (`similar-oracle-v1.json`) | 118 · 700 | 67/118 = 56.8 % | 63/118 = 53.4 % | 39/59 = 66.1 % | 74/118 = 62.7 % | 101/165 = 61.2 % | 60 % |
| v2 holdout (`similar-oracle-v2.json`) | 115 · 668 | 46/115 = 40.0 % | 42/115 = 36.5 % | 30/56 = 53.6 % | 69/115 = 60.0 % | 86/177 = 48.6 % | 40 % |

([EVAL-SET-SIMILAR.md:85](../../EVAL-SET-SIMILAR.md#L85), [EVAL-SET-SIMILAR.md:94](../../EVAL-SET-SIMILAR.md#L94),
[EVAL-SET-SIMILAR.md:272](../../EVAL-SET-SIMILAR.md#L272), [EVAL-SET-SIMILAR.md:281](../../EVAL-SET-SIMILAR.md#L281)).
The second generation is a **holdout by construction** — same instrument, same quotas, same
order, skipping every rank the first oracle arbitrated — and it read one step lower across the
board. That is the finding the tuning had to survive: the three candidates the first sample
favoured (per-channel normalisation, query tf clipped to 1, `spec ∧ 2N ≥ QN`) were retested on
the holdout and none was adopted — the best gained three queries with a 5 : 2 paired split, one
made this repository worse, one lost three true positives — so `SIMILAR_REV` stayed at 1 and the
conjunction entered the core in its spec form ([EVAL-SET-SIMILAR.md:314](../../EVAL-SET-SIMILAR.md#L314)).
The gate `eval_similar_precision` is not ignored: every generation's oracle must be consistent
with the live constants and re-derived from its rows, the four fixture corpora replay byte for
byte, later generations must not overlap earlier ones, and each generation holds the floor its
own ledger set — 60 % for v1, 40 % for v2 — floors that only rise
([eval_similar_precision.rs:37](../../../cli/tests/it/eval_similar_precision.rs#L37),
[eval_similar_precision.rs:65](../../../cli/tests/it/eval_similar_precision.rs#L65),
[eval_similar_precision.rs:165](../../../cli/tests/it/eval_similar_precision.rs#L165)).

### 8. Residual risks, stated

- **The role rule is a precision instrument, not a recall one.** 61 % and 49 % role-bit
  precision on the two generations, against a top-1 that is `same_role` 57 % and 40 % of the
  time: the advisor is right more often than not and wrong often enough that every face prints
  it as advice. Promotion to anything that blocks would need an FPR ledger the family does not
  have and, by the spec, is not built to earn.
- **Free text is weaker than a seat.** A `text` ask has name and doc words only; without shape
  and callee evidence the role bit cannot fire, and the ranking leans on prose the unit may not
  carry. The faces say so where the text form is offered.
- **The stemmer and the stop list are English.** Identifier pieces in other scripts are hashed
  as spelled and still match exactly; they never meet through a stem.
- **Association is only as good as the repository.** PPMI over a small tree finds few pairs past
  two bits; over a large one it finds this repository's synonyms and nobody else's — which is
  the design, and also why the view is opt-in.
- **A sixth parse and random-key writes** are the price of keeping the bags inside the one
  content-hash-gated refresh; the cost sits in PERF-BUDGET and moves only with the tree.

### 9. Acceptance

The three faces agree on one document: the CLI's `--format json` prints the library face byte
for byte and the core judged the fixture pair same-role; the MCP tool relays it and refuses a
request that names two asks; the Stop leg writes `{unit, twin, score}` for a unit the session
added and no key once the unit is committed
([similar_face.rs:48](../../../cli/tests/it/similar_face.rs#L48),
[similar_face.rs:99](../../../cli/tests/it/similar_face.rs#L99)). The wire leg asks the core for
every unit of the go fixture and gets the measurement's order and roles back over one link, and
the family is offered, judged and refuses by name
([similar_wire.rs:18](../../../cli/tests/it/similar_wire.rs#L18),
[similar_wire.rs:42](../../../cli/tests/it/similar_wire.rs#L42)). The replay holds the in-memory
corpus and the SQL reader to one ranking on five corpora
([similar_replay.rs:188](../../../cli/tests/it/similar_replay.rs#L188)); the precision gate
holds both oracle generations to their floors. The advisor is one row of the three-face parity
table — CLI, GUI tab and Tauri command, MCP tool — and the fifteenth tool in the MCP catalogue
([face_parity.rs:38](../../../cli/tests/it/face_parity.rs#L38)). Docs cite implementation lines
(this booklet is under the citations gate), the constants above bind to their source names
under `docs_consts`, and the feed golden carries the `similar` key at `ce.observe/0.10.0`
([feed.golden.json](../../../contracts/fixtures/observe-feed/feed.golden.json)).
