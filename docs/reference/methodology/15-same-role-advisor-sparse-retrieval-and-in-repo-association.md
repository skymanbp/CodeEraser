# Same-role advisor — sparse retrieval and in-repo association

[index](../methodology.md) · [← 14 Tombstone residue — the erased-name conjunction](14-tombstone-residue-the-erased-name-conjunction.md) · [→ 16 Code query and architecture rules — Datalog over the index's facts](16-code-query-and-architecture-rules-datalog-over-the-index.md)

The three clone families read text: fingerprints (booklet 01), tree edit distance (02) and
shingles (03) all need the two units to *look* alike. This family answers a different
question — `file 1 has a function x; file 2 now has a function y that plays the same part`
— when y and x share no line. It is an **advisor**, on booklet 13's terms: it never reddens a
gate, never enters `ce erase`, and every face prints it as advice
([mod.rs:1-12](../../../cli/src/similar/mod.rs#L1)). Two things make it deterministic and
offline where a "code RAG" usually is neither: the retrieval is sparse — integer BM25 over
term bags read off facts the index already carries, not an embedding — and the only
association it knows is this repository's own, positive pointwise mutual information over the
same bags, opt-in and never evidence ([ppmi.rs:1-11](../../../cli/src/similar/ppmi.rs#L1)).
The split is ADR-008's, sixth instalment, moved further by plan v2.33 W3: Rust builds the bags
and the inverted tables and fetches what one query's ranking needs; Haskell weights, widens,
scores and cuts the top-K over `rank/1` ([Rank.hs:5-15](../../../core/app/CE/Similar/Rank.hs#L5)), then orders the
candidates as exact rationals and decides which of them play the query's role over the
twelfth wire family, `similar/1`. Names, words and paths never cross the wire — hashes and counts only, row index is
identity ([wire.rs:1-7](../../../cli/src/similar/wire.rs#L1),
[Similar.hs:5-17](../../../core/app/CE/Similar.hs#L5)).

### 1. The bag — six channels off facts the tree already carries

Every code unit of the unitsig universe — the T3 universe, `(file, key, nth)`, so Markdown has
no bags — gets one sparse bag ([bag.rs:1-10](../../../cli/src/similar/bag.rs#L1)). Six
channels, each read off a fact the parse already produced, each with a one-letter label that is
mixed into the term hash — a name word and a callee word spelled alike are two terms, so a
shared name is name evidence and a shared callee is callee evidence, and the role rule can read
them apart ([terms.rs:7-11](../../../cli/src/similar/terms.rs#L7)). The tree stays on the
measuring side, which sends each unit's facts as text over the local pipe — the key, the kind
word and return flag, the callee spellings, the literal kinds, the structure histogram and the
lines of the comments it owns — and the core builds the bag: splitting, the stop list, the
stemmer, the hashes and the channel order are one road in `bags/1`
([bags.rs:1-11](../../../cli/src/similar/bags.rs#L1),
[Bags.hs:5-25](../../../core/app/CE/Similar/Bags.hs#L5)):

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
([Terms.hs:55-60](../../../core/app/CE/Similar/Terms.hs#L55),
[Terms.hs:84-87](../../../core/app/CE/Similar/Terms.hs#L84)). Letter, digit and case classes
are Rust's, held as tables generated from rustc's own, and lowercasing is the full Unicode
mapping, so the core splits a word exactly where the measuring side once did. The stop list is
a fixed table of 48 prose words, never learned from a corpus, and it does not touch identifier
pieces — `get`, `set` and `is` are what a role is made of
([Terms.hs:46-53](../../../core/app/CE/Similar/Terms.hs#L46)). Word channels are stemmed by
Porter's 1980 algorithm and hashed; feature channels are hashed as spelled
([Stem.hs:21](../../../core/app/CE/Similar/Stem.hs#L21),
[Terms.hs:89-97](../../../core/app/CE/Similar/Terms.hs#L89)). The doc channel takes the
segments docdup already extracts, attributed by position — a leading block ending within
`LEAD_GAP` lines above the unit's first line, or a head block within `HEAD_GAP` lines below it
([docs.rs:13-14](../../../cli/src/similar/docs.rs#L13),
[docs.rs:59](../../../cli/src/similar/docs.rs#L59)). Only channel-tagged fnv1a64 hashes come
back from the core: no word text is stored anywhere downstream, which is the index-privacy
clause the plan writes for every table `.ce/index.db` gains
([Terms.hs:1-6](../../../core/app/CE/Similar/Terms.hs#L1)).

Query weights are integer multipliers — names ×3, callees ×2, everything else ×1 — so the
score stays exact ([Cost.hs:51-57](../../../core/app/CE/Similar/Rank/Cost.hs#L51)). The whole term
road is declared once as `SIMILAR_REV` and sits in the index cache key: a change to any rule
above wipes the bag tables with the rest of the index rather than ranking old bags against new
queries ([mod.rs:37-47](../../../cli/src/similar/mod.rs#L37)).

### 2. The inverted tables — bags persisted as postings, pairs not stored

Two tables, added at index schema 16 (17 today), holding only hashes and counts
([store.rs:1-6](../../../cli/src/similar/store.rs#L1),
[store.rs:46-59](../../../cli/src/similar/store.rs#L46)). `bag(term_hash, unit, tf, channel)`
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
[store.rs:69](../../../cli/src/similar/store.rs#L69),
[store.rs:81](../../../cli/src/similar/store.rs#L81)). The cost, measured on this tree of 687
files: cold `ce dedup` 5.2 → 8.4 s (0.65 s of a sixth parse, ≈1.5 s of random-key posting
writes that five layouts could not beat), warm unchanged, database 10.7 → 18.0 MB
([PERF-BUDGET.md:254-273](../../PERF-BUDGET.md#L254)).

### 3. Ranking — integer BM25, one road for the instrument and the product

Okapi BM25 with Robertson & Walker's usual `k1 = 6/5`, `b = 3/4` — kept as the rationals they
are and folded into one integer fraction. A term's contribution is

    w · idf · 22 · tf · avg / (10 · tf · avg + 3 · avg + 9 · len)

floored to 16-bit fixed point; the core's battery re-derives the fraction from `k1` and `b`
([Cost.hs:31-34](../../../core/app/CE/Similar/Rank/Cost.hs#L31),
[Math.hs:49-61](../../../core/app/CE/Similar/Rank/Math.hs#L49)). `idf = log2((N − df + ½) / (df + ½))`
in 8-bit fixed point from an integer `log2` by squaring only, floored at zero
([Math.hs:11-36](../../../core/app/CE/Similar/Rank/Math.hs#L11),
[Math.hs:38-47](../../../core/app/CE/Similar/Rank/Math.hs#L38)). No float is touched anywhere, so the same
corpus ranks the same on every platform and the frozen evaluation rows compare byte for byte
([Math.hs:1-5](../../../core/app/CE/Similar/Rank/Math.hs#L1)).

The core returns the `K = 5` best candidates for a query, excluding the query's own seat,
ordered by score then identity ([Score.hs:62-74](../../../core/app/CE/Similar/Rank/Score.hs#L62)). A term in more than half the units — idf 0 — is neither score
nor evidence: sharing what nearly everything shares says nothing, and walking its posting list
would cost the whole corpus per query, so the measuring side asks df first and fetches the list
only when `N > 2 · df` (`scoredDfRatio`, [Cost.hs:81-89](../../../core/app/CE/Similar/Rank/Cost.hs#L81), [rank.rs:148-176](../../../cli/src/similar/rank.rs#L148)).
Shape equality is read for the K survivors only, by the measuring side, and neither it nor the
role bit orders ([rank.rs:113-121](../../../cli/src/similar/rank.rs#L113)).
The request is written once, against the `Postings` trait: the in-memory `Corpus` the instruments
build and the persisted `Reader` over `.ce/index.db` both feed the same request, and the replay
asserts they send the same bytes on every unit of five corpora — the instrument and the product
run one road ([rank.rs:59-72](../../../cli/src/similar/rank.rs#L59),
[reader.rs:7-10](../../../cli/src/similar/reader.rs#L7),
[similar_replay.rs:247](../../../cli/tests/it/similar_replay.rs#L247)). Query weights ride in
`1/wUnit` = 1/256, which is what lets a PPMI-scaled expansion keep a *fraction* of its
parent's weight without a float ([Cost.hs:42-45](../../../core/app/CE/Similar/Rank/Cost.hs#L42)).

### 4. The associative view — positive PMI over this repository, opt-in

Two word terms co-occurring in one unit's bag are counted once per unit, and

    PPMI(a, b) = max(0, log2(n_ab · N / (n_a · n_b)))

in the same 8-bit fixed point as the idf, from the same integer `log2`
([Math.hs:63-72](../../../core/app/CE/Similar/Rank/Math.hs#L63)). A neighbour counts only when it co-occurred
in at least `minCooc = 2` units and carries at least `minPpmi` = two bits of association;
each spelled word term of the query appends its `topM = 3` best neighbours at weight
`parent × min(ppmi, ppmiCap) / ppmiScale` — at most half the parent's weight — and a term the
query already spells is never appended ([Cost.hs:67-79](../../../core/app/CE/Similar/Rank/Cost.hs#L67),
[Score.hs:30-60](../../../core/app/CE/Similar/Rank/Score.hs#L30)). A unit past `TERM_CAP = 96` distinct
word terms contributes its first 96 in term order and is ledgered as capped; the cap has one
owner, so the in-memory table and the persisted writer count the same words
([ppmi.rs:18-42](../../../cli/src/similar/ppmi.rs#L18)). One bound does the pruning for free:
`n_ab ≤ n_b` bounds `PPMI(a, b)` by `log2(N / n_a)`, under two bits as soon as `4 · n_a > N`, so
the measuring side never fetches such a word's pair rows ([Cost.hs:81-89](../../../core/app/CE/Similar/Rank/Cost.hs#L81), [rank.rs:123-146](../../../cli/src/similar/rank.rs#L123)).

This is enough to let this repository's `fetch / load / retrieve` meet, and never enough to
outvote what the unit itself spells. No corpus but this one is consulted and no word table is
written. The step-2 tuning verdict made the widened arm **an opt-in association view, never
the default and never evidence**: on the frozen sample it re-ranked the same 84 configurations
the way the bare arm did and crossed no significance line (widened 63/118 against bare 67/118
on the first generation as first arbitrated; the Go fixture's re-measurement of 2026-09-26 moved the same oracle to 66/118 against 68/118, and the TypeScript fixture's of 2026-09-27 to 64/118 against 66/118), so the faces show the widened rows as a second page, tagged, and the
role bit is read off the six channels only
([EVAL-SET-SIMILAR.md:228](../../EVAL-SET-SIMILAR.md#L228),
[face.rs:40-41](../../../cli/src/similar/face.rs#L40)).

### 5. The wire — `similar/1`, and what Haskell judges

The request is the query bag as `[termHash, weight]` pairs — strictly ascending, may be empty —
plus one nine-integer row per candidate, `[nHit, pHit, cHit, dHit, sHit, lHit, shapeEqual,
bm25Num, bm25Den]`: the six channel hits, the shape bit, and the fixed-point score as a fraction
over its unit, so the core compares the ratio and never learns the width
([wire.rs:30-41](../../../cli/src/similar/wire.rs#L30),
[Cost.hs:30-32](../../../core/app/CE/Similar/Cost.hs#L30)). The reply is `order` — the candidate
indices by score descending as exact rationals, ties by request index — `roles`, one bit per
row in request order, and `counts{rows, queryTerms, role}`
([Similar.hs:88-94](../../../core/app/CE/Similar.hs#L88),
[Similar.hs:103-120](../../../core/app/CE/Similar.hs#L103)). The role rule is a two-arm
conjunction over one row, and it lives in Haskell:

    role ⇔ (nHit ≥ roleMinName ∧ cHit ≥ roleMinCallee) ∨ (nHit ≥ roleMinNameShape ∧ shapeEqual)

with floors 1, 1 and 2: a unit that is *called* the same and *calls* the same, or two name
words in common with the same signature shape ([Cost.hs:35-57](../../../core/app/CE/Similar/Cost.hs#L35)).
The measuring side keeps no copy of that rule: the frozen evaluation rows take their role bits
from `similar/1` ([similar_replay.rs:158-172](../../../cli/tests/it/similar_replay.rs#L158)), and the only Rust spelling left
is the frozen oracle the differential gate drives against the core ([similar.rs:191-194](../../../cli/tests/unit/w3_oracle/similar.rs#L191)).
The offline tuning instrument that once mirrored it retired with the Rust ranking it tuned (EVAL-SET-SIMILAR.md, re-run section).
A request whose query terms plus rows exceed `similarCap` = 65536 gets a complete degraded
reply with empty tables and the reason `similar_too_large` — a query the core refused to judge
has no order and no roles, and the faces name the degradation instead of showing the measuring
side's order ([Cost.hs:27-28](../../../core/app/CE/Similar/Cost.hs#L27),
[Similar.hs:97-100](../../../core/app/CE/Similar.hs#L97)). There is no knob and no fail tier:
a knobless family whose one table is not the shared `RowsReq` — the query bag is its own key —
so it binds the cascade directly. On the Rust side `consume` is strict: the order must be a
permutation of the rows sent, one role bit per row, counts agreeing with the tables; any skew
is a *named* non-judgment, never conflated with "no candidates"
([wire.rs:78-97](../../../cli/src/similar/wire.rs#L78)). The family entered the protocol at
6.7.0, additively ([VERSIONING.md](../../../contracts/VERSIONING.md)).

### 6. Three faces, one document, one Stop line

Every face renders one document, `ce.similar-report/0.1.0`: the query as
`{label, terms, widen}`, the candidates as rows `{at, key, nth, role, score, hits[6],
shape_equal, widened}` — the first five alphabetical scalars are what the GUI hub's generic
projection shows — the counts, and `degraded` naming why the core did not judge when it did not
— laid out by the core from the measured rows ([face.rs:29-42](../../../cli/src/similar/face.rs#L29), [Document.hs:46](../../../core/app/CE/Similar/Document.hs#L46), [Document.hs:78](../../../core/app/CE/Similar/Document.hs#L78)).
A query is exactly one of three asks: `at` (`file:line`, the innermost unit holding the line),
`unit` (a key, refused by name when ambiguous, naming up to five places) or `text` (free text,
whose words become name and doc evidence — no shape, no callee, so the core's role bit is false
by construction) ([query.rs:16-53](../../../cli/src/similar/query.rs#L16),
[query.rs:61-94](../../../cli/src/similar/query.rs#L61)). `judged` refreshes the index over the
same content-hash gate every command uses, resolves the ask, ranks the bare arm — and the widened
arm when asked, its rows not in the bare arm tagged `widened` — and rides one `similar/1`
request per arm over one core link, the document asked over the same link ([face.rs:55-86](../../../cli/src/similar/face.rs#L55)).

- **CLI** `ce similar --at file:line | --text "…" | --unit key [--widen]`: a bilingual head
  line and one line per candidate, `at key  N P C D S L  role`; `--format json` is the document
  ([main_similar.rs:30](../../../cli/src/main_similar.rs#L30), [Lines.hs:13](../../../core/app/CE/Similar/Lines.hs#L13)).
- **MCP** `similar_units` — the fifteenth read-only tool, `{at, text, unit, widen}`, relaying
  the same document ([tools.rs:178](../../../cli/src/mcp/tools.rs#L178)).
- **GUI** the eleventh screen, `similar`: an input for `at` or text, the widen switch, the
  candidate table with the six evidence columns ([similar.js](../../../gui/ui/similar.js),
  [commands.rs](../../../gui/src-tauri/src/commands.rs)).
- **Stop audit** — every unit the session *added* (a `(key, nth)` the working tree's file holds
  and `HEAD`'s did not) is asked of the index the way `ce similar` asks, and a row
  `{unit, twin, score}` is written into the feed's `similar` object only when the core's top-1
  carries the role bit: an advisor's line for the evaluation ledger, never a reason to block.
  No new unit, no role hit and nothing degraded = no key at all; the feed schema moved
  additively to `ce.observe/0.10.0` ([audit/similar.rs:1-10](../../../cli/src/audit/similar.rs#L1),
  [hookio.rs:104](../../../cli/src/hookio.rs#L104)). The tombstone leg and this leg read the
  session's changed pairs once, through one git batch (booklet 14 §1).

The write-time hook does **not** run it: a PreToolUse budget does not hold a retrieval, and a
family without a deny tier has nothing to say there (spec §2).

### 7. Evaluation — two oracle generations, two floors

There is no oracle that knows every same-role partner of a unit, so **recall is not reported**;
the ledger reports p@1 — the arm's top-1 arbitrated `same_role` — and hit@5, per arm and per
corpus, plus the confusion of the role bit over every candidate pair
([EVAL-SET-SIMILAR.md:83-86](../../EVAL-SET-SIMILAR.md#L83)). The instrument is
`similar_replay`: five corpora (this repository and the four cross-check fixtures) each become
their own database, every unit is queried against the rest of its corpus on both arms, and the
row identity is the sha256 of the text with CRLF folded to LF — the checkout must not change who
a line is ([similar_replay.rs:1-12](../../../cli/tests/it/similar_replay.rs#L1)). Samples are
drawn by sha256 order, arbitrated candidate by candidate as `same_role / related / unrelated`,
and frozen as oracles.

| generation | queries · pairs | p@1 bare | p@1 widened | p@1 bare, role = 1 | hit@5 bare | role-bit precision | floor |
|---|---|---|---|---|---|---|---|
| v1 (`similar-oracle-v1.json`) | 118 · 695 | 66/118 = 55.9 % | 64/118 = 54.2 % | 39/59 = 66.1 % | 75/118 = 63.6 % | 100/165 = 60.6 % | 60 % |
| v2 holdout (`similar-oracle-v2.json`) | 115 · 667 | 44/115 = 38.3 % | 40/115 = 34.8 % | 29/57 = 50.9 % | 69/115 = 60.0 % | 89/179 = 49.7 % | 40 % |

([EVAL-SET-SIMILAR.md:91](../../EVAL-SET-SIMILAR.md#L91), [EVAL-SET-SIMILAR.md:100](../../EVAL-SET-SIMILAR.md#L100),
[EVAL-SET-SIMILAR.md:280](../../EVAL-SET-SIMILAR.md#L280), [EVAL-SET-SIMILAR.md:289](../../EVAL-SET-SIMILAR.md#L289)).
The second generation is a **holdout by construction** — same instrument, same quotas, same
order, skipping every rank the first oracle arbitrated — and it read one step lower across the
board. That is the finding the tuning had to survive: the three candidates the first sample
favoured (per-channel normalisation, query tf clipped to 1, `spec ∧ 2N ≥ QN`) were retested on
the holdout and none was adopted — the best gained three queries with a 5 : 2 paired split, one
made this repository worse, one lost three true positives — so `SIMILAR_REV` stayed at 1 and the
conjunction entered the core in its spec form ([EVAL-SET-SIMILAR.md:322](../../EVAL-SET-SIMILAR.md#L322)).
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
[similar_face.rs:99](../../../cli/tests/it/similar_face.rs#L99)). The wire leg ranks every unit of the go
fixture through rank/1, has similar/1 judge it, and checks every role bit against the spec's
conjunction; the family is offered, judged and refuses by name
([similar_wire.rs:17](../../../cli/tests/it/similar_wire.rs#L17),
[similar_wire.rs:36](../../../cli/tests/it/similar_wire.rs#L36)). The replay holds the in-memory
corpus and the SQL reader to one ranking on five corpora
([similar_replay.rs:247](../../../cli/tests/it/similar_replay.rs#L247)); the precision gate
holds both oracle generations to their floors. The advisor is one row of the three-face parity
table — CLI, GUI tab and Tauri command, MCP tool — and the fifteenth tool in the MCP catalogue
([face_parity_table.rs:34](../../../cli/tests/it/face_parity_table.rs#L34)). Docs cite implementation lines
(this booklet is under the citations gate), the constants above bind to their source names
under `docs_consts`, and the feed golden carries no `similar` object by design: its
staged twin shares one name word where the core's role bit wants two, so `similar_face.rs` seeds the pair
that earns the key ([feed.golden.json](../../../contracts/fixtures/observe-feed/feed.golden.json),
[observe_feed.rs:1-10](../../../cli/tests/it/observe_feed.rs#L1)).
