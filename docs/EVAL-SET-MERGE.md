# v2.31 clone-merge audit registry (2026-09-30)

> The frozen record of the clone-merge family's blind audit: plan v2.31 steps 6-7, design booklet
> [reference/analysis-track.md](reference/analysis-track.md) section 6.5 and section 13 items 31-36, the method in booklet 18
> [reference/methodology/18-clone-merge-suggestions-anti-unification.md](reference/methodology/18-clone-merge-suggestions-anti-unification.md)
> section 6. Mother chain: [EVAL-SET.md](EVAL-SET.md) -> [EVAL-SET-M5-3.md](EVAL-SET-M5-3.md) ->
> [EVAL-SET-M5-CLOSE.md](EVAL-SET-M5-CLOSE.md) -> [EVAL-SET-SIMILAR.md](EVAL-SET-SIMILAR.md) ->
> [EVAL-SET-LANGS.md](EVAL-SET-LANGS.md) -> [EVAL-SET-FLOW.md](EVAL-SET-FLOW.md) -> this page. The page joins the frozen
> set (`frozen_set.rs`: no chip, no generated block, opted out of the citation gate); line citations are never written.
> Its numbers were computed from the four docs below and the batch manifest, rendered onto the page and read back
> before filing. A merge suggestion is advice (booklet section 6.4): the readings are recorded and gate nothing.

## Instruments and gates

- **The frozen suggestion set** (`contracts/eval/merge-suggestions-v1.json`, `ce.eval-merge-suggestions/1.0.0`; generator
  `eval_merge_suggestions::regenerate`, `#[ignore]`): the `ce merge` document over this repository at one pinned commit,
  exported with `git archive` and `.gitmodules` removed, and over the four pinned external checkouts - per corpus the
  counts, the groups not sent, and one row per suggestion with its members (`at`, `run`), family, fragment, parameters,
  member kept, savings, feasibility, reason and hole count. The CI leg measures the pinned self tree again and holds every
  row; no two rows of a corpus share a member set.
- **The sample** (`contracts/eval/merge-sample-v1.json`, `ce.eval-merge-sample/1.0.0`): the feasible and the infeasible half,
  50 rows each, every half apportioned over the corpora by their share of it (largest remainder), each corpus's seats the
  lowest identity hashes under the domain `merge-sample-v1`. A sampled row carries the members' identities, runs and
  source and, per parameter the core found, every member's text (`param_texts`); it withholds the feasibility, the reason,
  the savings and the member kept. The gate redraws the sample from the frozen set and refuses a sampled row that
  carries a feasibility, a reason, a parameter count or savings.
- **The batches** (`eval_merge_batches::merge_batches`, `#[ignore]`, written outside the tree): the sample reordered by a
  second hash of each row's id under `merge-batches-v1` and cut into 4 batches of 25, so no batch's place
  says which half a row came from; the plan is a pure function of the sample, the manifest a convenience. A CI leg holds
  every row in exactly one batch and no rendered batch spelling the savings or the member kept as a field.
- **The review doc** (`contracts/eval/merge-review-v1.json`, `ce.eval-merge-review/1.0.0`; `eval_merge_review::assemble`,
  `#[ignore]`, `CE_BLESS=1 CE_MERGE_BATCH_DIR=<dir> CE_MERGE_AUDITOR=<sentence>`): the manifest checked against the batch
  plan, each answer file read line by line - as many lines as questions, the batch's ids in its order, exactly the five
  fields `id`, `feasible`, `reason`, `params`, `note`, `reason` one of the six names and `ok` exactly when feasible,
  `params` a non-negative integer, `note` a non-empty sentence of at most 200 characters - every refusal named by batch and
  line; the rows written verbatim in the sample's order under the auditor sentence, the batch count and the provenance
  stamp. The gate `the_review_scores_when_filed` re-runs the same row check on the filed doc; the tamper battery
  `the_review_refuses_tampering` holds a reason outside the six, a parameter count that is no integer, a note past its
  bound, a dropped row, an extra row, a swapped pair, a feasibility against its reason, a forged batch count and an empty
  auditor each refused, and the filed doc byte for byte what it was after the battery.
- **The precision doc** (`contracts/eval/merge-precision-v1.json`, `ce.eval-merge-precision/1.0.0`;
  `eval_merge_review::regenerate`, `#[ignore]`): per corpus and overall, `feasible_agree` and `reason_agree` - the rows
  whose audited feasibility and reason equal the core's - and `params_agree` - the rows whose audited parameter count,
  given after seeing the core's parameterisation, equals the core's - over `rows`. It is a pure function of the three
  docs it names under `from`; the gate recomputes it and holds the filed doc equal.

## The frozen set and the sample

Per corpus: the suggestions frozen and how many the core judged feasible, and the rows the sample drew - by
the core's feasibility (the two halves) and by family.

| corpus | suggestions | feasible | sampled | sampled feasible | sampled infeasible | t1t2 | t3 |
|---|---|---|---|---|---|---|---|
| zod | 22985 | 3660 | 68 | 30 | 38 | 6 | 62 |
| ripgrep | 6385 | 1433 | 22 | 12 | 10 | 7 | 15 |
| cobra | 1593 | 861 | 8 | 7 | 1 | 4 | 4 |
| requests | 138 | 89 | 1 | 1 | 0 | 0 | 1 |
| codeeraser | 576 | 51 | 1 | 0 | 1 | 0 | 1 |
| all | 31677 | 6094 | 100 | 50 | 50 | 17 | 83 |

17 of the 100 sampled rows are fragment groups (T1/T2 families whose members are not whole
units); every T3 row is a pair of whole units.

## The blind review

Four independent judges, one batch of 25 questions each, answered all 100 sampled rows. The
auditor sentence the review doc carries, verbatim:

> four independent subagents, one batch each (25 questions, in the batches' rehashed order), each reading only its own batch-<n>.md - the members' identities, runs and source and the tool's parameterisation, never the feasibility, the reason, the savings or the member kept - and writing only its answers-<n>.jsonl; no git, no search, no product report, no ce and no core in reach (the first batch's judge also kept a draft and a self-check script that reads its own batch file alone, in its own scratchpad); each answered feasible, reason, params and a note from the source as shown, under the batch prompt's reason order (the first that applies: spans_statements, position, type, too_many_params, no_savings, else ok); the coordinator assembled verbatim, judgments untouched (booklet analysis-track.md sections 6.5 and 13 item 36)

Each judge was asked, one by one, after its batch was written: it had read its own batch file and written its own
answer file, and had used no git, no search and no report of any kind. The first batch's judge said it had also kept
a draft and a short self-check script in its own scratchpad, the script reading nothing but that batch's questions.
No judge saw another's batch or answers, and no question shows a verdict field.

The answers: feasible 55, infeasible 45; by reason:

| ok | position | type | spans_statements | too_many_params | no_savings |
|---|---|---|---|---|---|
| 55 | 6 | 34 | 0 | 0 | 5 |

By batch:

| batch | ok | position | type | spans_statements | too_many_params | no_savings |
|---|---|---|---|---|---|---|
| 1 | 6 | 2 | 14 | 0 | 0 | 3 |
| 2 | 16 | 1 | 6 | 0 | 0 | 2 |
| 3 | 17 | 2 | 6 | 0 | 0 | 0 |
| 4 | 16 | 1 | 8 | 0 | 0 | 0 |

The audited parameter counts:

| params | 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 |
|---|---|---|---|---|---|---|---|---|---|
| rows | 10 | 9 | 12 | 24 | 27 | 9 | 4 | 3 | 2 |

Five rows give more than six parameters, and none of them answers `too_many_params`:

- `0015619b6dc1` (batch 1, line 4): 7 parameters, `position`.
- `004959d655bc` (batch 1, line 9): 7 parameters, `type`.
- `0093f6ecedfb` (batch 1, line 15): 8 parameters, `type`.
- `008cec664c1f` (batch 1, line 22): 8 parameters, `type`.
- `0026e501fbaa` (batch 3, line 2): 7 parameters, `type`.

That is the prompt's order and not a slip: a judge answers the first reason that applies, and
`too_many_params` is the fourth - it applies only when no differing place is a statement run, another position or
a type. Each of these rows names a type or position difference first, so its count stands beside an earlier reason.

## Precision

The precision doc's readings, per corpus and overall (agreeing rows / rows):

| corpus | rows | feasible_agree | reason_agree | params_agree |
|---|---|---|---|---|
| zod | 68 | 60 / 68 | 56 / 68 | 26 / 68 |
| ripgrep | 22 | 17 / 22 | 15 / 22 | 11 / 22 |
| cobra | 8 | 4 / 8 | 4 / 8 | 7 / 8 |
| requests | 1 | 1 / 1 | 1 / 1 | 1 / 1 |
| codeeraser | 1 | 1 / 1 | 1 / 1 | 0 / 1 |
| all | 100 | 83 / 100 | 77 / 100 | 45 / 100 |

The core's reason (rows) against the audited reason (columns), every sampled row once:

| core \ audit | ok | position | type | spans_statements | too_many_params | no_savings | rows |
|---|---|---|---|---|---|---|---|
| ok | 44 | 4 | 1 | 0 | 0 | 1 | 50 |
| position | 11 | 2 | 2 | 0 | 0 | 4 | 19 |
| type | 0 | 0 | 31 | 0 | 0 | 0 | 31 |
| rows | 55 | 6 | 34 | 0 | 0 | 5 | 100 |

No sampled row has the core's reason `spans_statements`, `too_many_params` or `no_savings`.

Parameter counts: the core's equals the audit's on 45 rows, exceeds it on 50 and falls short of it on
5.

### The 23 reason disagreements

Every row whose audited reason is not the core's (17 of them also disagree on feasibility), grouped by the
pair of reasons, with the core's parameters and savings from the frozen set and the judge's note verbatim. The
audited truth stands and the core is unchanged (design booklet section 13 item 36); the rows are input to the next
generation's parameterisation and reason order.

- `007acd7cec85` cobra t1t2 fragment - core `ok` (params 0, savings 3) / audit `position` (params 2): "The folded run includes the function header line, where member 1's parameter list has an extra formal parameter ctx context.Context, a structural piece."

- `00fe2f72ad8f` cobra t3 - core `ok` (params 6, savings 3) / audit `position` (params 6): "executeCommand receives two string arguments in member 0 and three in member 1, an argument one member lacks, which is a structural difference."

- `0116334e9e04` zod t3 - core `ok` (params 4, savings 8) / audit `position` (params 4): "The comparison operator differs (ch.value < max vs ch.value > min), an operator place that no parameter can stand for."

- `0173d31a603e` cobra t3 - core `ok` (params 5, savings 3) / audit `position` (params 5): "The final call passes output as an extra argument (three arguments vs two), a structural difference in the argument list no parameter can stand for."

- `00e40466c1b4` zod t1t2 fragment - core `ok` (params 1, savings 2) / audit `type` (params 3): "After the declared name pick vs omit, the return-type annotation differs (Pick<...> with an intersection key vs Omit<...> with keyof M), a type place."

- `0164bed5a39c` ripgrep t1t2 fragment - core `ok` (params 3, savings 2) / audit `no_savings` (params 3): "Names ig1/tests_parents, ig2/tests and the foo/tests literal fold, but 8 fragment run lines less a 6-line helper (signature and closing) and 2 calls leave 0."

- `0003206e2776` ripgrep t3 - core `position` (params 2, savings 2) / audit `ok` (params 1): "Only the declared method name and the called builder method differ, both the same dotall/ucp text pair, so one parameter; 4-line runs save 2 lines."

- `001a2c1754a1` zod t3 - core `position` (params 7, savings 10) / audit `ok` (params 5): "Test name, goodData and badData initialisers, schema identifier and toEqual(undefined) vs toEqual(goodData) differ: five expression parameters, 10 lines saved."

- `001badff73a2` zod t3 - core `position` (params 6, savings 17) / audit `ok` (params 4): "Test name, the schema expression z.number().lt(3) vs z.number().positive(), the parseAsync argument 3 vs -1 and the snapshot literal differ; 17 lines saved."

- `001e5f83c730` ripgrep t3 - core `position` (params 4, savings 2) / audit `ok` (params 3): "The fn name, the byte-string literal and the raw-string literal (each pair used twice) differ; 8 run lines less 4 and 2 calls save 2."

- `003064137ab3` zod t3 - core `position` (params 1, savings 2) / audit `ok` (params 1): "Only the value core.$ZodUnion vs core.$ZodTransform differs, one parameter; 4-line runs save 2 lines."

- `003a6bee44ce` zod t1t2 fragment - core `position` (params 5, savings 7) / audit `ok` (params 5): "The runs differ in the test-name literal, the opt/nul name, schema.optional vs schema.nullable, z.ZodOptional vs z.ZodNullable and undefined vs null."

- `0043c8c0b0ca` zod t3 - core `position` (params 2, savings 2) / audit `ok` (params 2): "Only the values core.$ZodULID vs core.$ZodArray and ZodMiniStringFormat vs ZodMiniType differ, two parameters; 4-line runs save 2 lines."

- `004a32a24156` zod t3 - core `position` (params 2, savings 2) / audit `ok` (params 2): "The arrow bodies differ only in the two init receivers, core.$ZodCIDRv6 vs core.$ZodPromise and ZodMiniStringFormat vs ZodMiniType."

- `004b88c1e095` ripgrep t3 - core `position` (params 9, savings 22) / audit `ok` (params 6): "Six expression or name places differ (fn name, pattern, printer builder, searcher builder chain, sink call, expected text), not more than six, so it folds."

- `00768018a7fc` ripgrep t3 - core `position` (params 1, savings 2) / audit `ok` (params 1): "Only the assigned field differs (args.fixed_strings vs args.no_unicode), one parameter; 4-line runs save 2 lines."

- `008aca0cdd17` cobra t3 - core `position` (params 2, savings 1) / audit `ok` (params 2): "The methods differ only in their declared name and the flag-set call c.Flags() vs c.PersistentFlags()."

- `0022b455f7eb` ripgrep t1t2 fragment - core `position` (params 6, savings 23) / audit `type` (params 6): "Literal, field and struct-name differences could be parameters, but the impl target type for Match<'a> vs for Context<'a> differs, a type."

- `004b98f56894` ripgrep t1t2 fragment - core `position` (params 4, savings 4) / audit `type` (params 5): "Inside the run the const annotation differs, &'static str in member 0 vs &str in member 1, a type; the expected literal and the names before it are parameters."

- `001e93a93275` zod t1t2 fragment - core `position` (params 3, savings -1) / audit `no_savings` (params 3): "multi/schema and the object keys A,B vs a,b could be parameters, but two one-line runs cannot pay for the helper plus two call lines."

- `001fd6b5128e` zod t3 - core `position` (params 1, savings -1) / audit `no_savings` (params 1): "Only symbolProcessor/setProcessor differs, but two one-line runs cannot pay for a merged function plus one call line per member."

- `002e5318cd29` zod t3 - core `position` (params 1, savings -1) / audit `no_savings` (params 1): "Only neverProcessor/prefaultProcessor differs, but two one-line runs cannot pay for a merged function plus one call line per member."

- `003a1ee6cfdc` zod t3 - core `position` (params 1, savings -1) / audit `no_savings` (params 1): "Only nullProcessor/dateProcessor differs, but two one-line runs cannot pay for a merged function plus one call line per member."

## Generation 2

The 23 reason disagreements were checked against the source one by one. Nine rulings came of them - design
booklet section 13 items 42-50 - and changed the core and the position-class tables: every tree node carries
its own anonymous tokens (`own`) and its token text (`text`); a node whose own tokens differ is a hole of the
other class; a leaf of the other class (a member name, a field, a string's content) under an expression on every
member widens to that expression; a gap is always structural; the holes run in member 0's postorder; a fragment's
merged function adds its helper's head and closing lines; parameters are counted by text; the tables place
assignment targets, part names and literal contents; and the batch prompt gains `## Reading rules`, the rulings
in the judge's words. The node cap fell from 1,048,576 to 131,072 so the widest request at both caps stays inside
the protocol's 32 MiB line (the own and text columns put ripgrep's first request at 39,165,017 bytes). The first
generation's four docs stay on disk as the record; the gates read the second.

- **Instruments.** One table names every generation's four docs (`eval_merge_parts::GENERATIONS`); the gates
  read the newest. The sample ids keep the domain `merge-sample-v1`, so a question is the same id in both
  samples; the batches take the domain `merge-batches-v2` and the manifest names `"readings": 2`. Every
  generation's sample is still held to the draw of its set, and the first generation's review and precision
  doc are still checked as one (the review a review of its sample, the precision doc what its three docs read).
- **The frozen set** (`contracts/eval/merge-suggestions-v2.json`), measured again on the same five trees: the
  same 31677 suggestions, every member set in both generations.

| corpus | suggestions | feasible v1 | feasible v2 | ok | position | type | spans_statements | too_many_params | no_savings |
|---|---|---|---|---|---|---|---|---|---|
| zod | 22985 | 3660 | 5929 | 5929 | 3386 | 11856 | 198 | 308 | 1308 |
| ripgrep | 6385 | 1433 | 1387 | 1387 | 2505 | 1756 | 133 | 377 | 227 |
| cobra | 1593 | 861 | 584 | 584 | 889 | 9 | 23 | 43 | 45 |
| requests | 138 | 89 | 69 | 69 | 44 | 0 | 11 | 7 | 7 |
| codeeraser | 576 | 51 | 35 | 35 | 238 | 107 | 68 | 31 | 97 |
| all | 31677 | 6094 | 8004 | 8004 | 7062 | 13728 | 433 | 766 | 1684 |

- **The sample** (`contracts/eval/merge-sample-v2.json`), drawn by the same rule from the second set: 72 of
  its 100 ids are in the first sample, 28 are new (zod 24, ripgrep 3, cobra 1) and 28 of the first sample's
  are gone (zod 19, ripgrep 5, cobra 3, requests 1).

| corpus | sampled | sampled feasible | sampled infeasible | t1t2 | t3 | fragments | shared with v1 |
|---|---|---|---|---|---|---|---|
| zod | 73 | 37 | 36 | 5 | 68 | 5 | 49 |
| ripgrep | 20 | 9 | 11 | 6 | 14 | 6 | 17 |
| cobra | 6 | 4 | 2 | 3 | 3 | 3 | 5 |
| requests | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| codeeraser | 1 | 0 | 1 | 0 | 1 | 0 | 1 |
| all | 100 | 50 | 50 | 14 | 86 | 14 | 72 |

- **The 23 disagreements in the second set** (the core's reason and parameters, first generation -> second;
  the first generation's audited reason after the slash): `0116334e9e04` ok 4 -> position 5 / position;
  `00fe2f72ad8f` ok 6 -> position 7 / position; `0173d31a603e` ok 5 -> position 6 / position;
  `0164bed5a39c` ok 3 -> no_savings 3 / no_savings; `00e40466c1b4` ok 1 -> no_savings 1 / type;
  `007acd7cec85` ok 0 -> ok 0 / position; `0003206e2776` position 2 -> ok 2 / ok; `003064137ab3`,
  `0043c8c0b0ca`, `004a32a24156`, `008aca0cdd17` position -> ok at the same count / ok; `003a6bee44ce` position 5
  -> ok 5 / ok; `001a2c1754a1` position 7 -> position 6 / ok (an object literal's elements one member lacks);
  `001badff73a2` position 6 -> position 6 / ok (an argument one member lacks); `004b88c1e095` position 9 ->
  position 10 / ok (arguments one member lacks); `00768018a7fc` position 1 -> position 1 / ok (the differing
  field is an assignment's target); `001e5f83c730` position 4 -> position 4 / ok (an empty byte string has no
  content node, so `b""` against `b"\n"` is a gap); `0022b455f7eb` position 6 -> type 6 / type;
  `004b98f56894` position 4 -> ok 3 / type; `001e93a93275` position 3 -> position 3 / no_savings;
  `001fd6b5128e`, `002e5318cd29`, `003a1ee6cfdc` position 1 -> no_savings 1 / no_savings.
- **The batches** (`eval_merge_batches::merge_batches`, written outside the tree): 4 batches of 25 under
  `merge-batches-v2`, each with the `## Reading rules` section; the second generation's blind review and
  precision doc are filed by the next commit.

## Provenance

- The review doc's `generated_from` is the tree it was assembled on: ce 1.8.0, commit `63041c544060b0e599309e73c8a7e347e9c308e2`,
  dirty true - the assembly leg and the doc land in one commit, the reading design booklet
  section 13 item 24 gave the flow reviews.
- The precision doc has no `generated_from`: it reads no product, only the three frozen docs it names, so it
  computes the same on any tree, and it lands in the review's commit (section 13 item 36).
- The suggestion set and the sample landed in steps 6-7's first landing commit; the manifest the batches were
  rendered beside equals the batch plan of the filed sample, checked when the review was assembled. The batches,
  the manifest and the answer files live outside the tree; the review doc holds the answers verbatim.
