# Dead-file exam registry, generation 1 (2026-11-11)

> The frozen record of the dead-file exam (lane v235_licence): the question whether a file the graph calls dead
> is unused, asked of independent auditors before any rule of the product reads the answer. Design and the
> erase rule it bears on: [reference/erase.md](reference/erase.md) and booklet
> [reference/methodology/12-deterministic-erase-the-safety-predicate.md](reference/methodology/12-deterministic-erase-the-safety-predicate.md).
> Mother chain: [EVAL-SET.md](EVAL-SET.md) -> [EVAL-SET-LANGS.md](EVAL-SET-LANGS.md) -> this page. The page joins the
> frozen set (`frozen_set.rs`: no chip, no generated block, opted out of the citation gate); the record it reads is
> pinned byte for byte by sha256 in `audit/record.rs`. Every table below is rendered from the committed record and the frozen docs by
> `eval_deadfile_parts::audit::page`, and the gate `it/eval_deadfile_review.rs` holds this page to carrying each one
> verbatim; every quoted line is held to the record file it is cut from. Nothing here gates the product: the exam
> licenses no deletion and changes no verdict.

## Instruments, records and gates

- **The universe** (`contracts/eval/deadfile-slice-<corpus>-v1.json`, `ce.eval-deadfile-slice`): every file-tier dead row
  of the product's deadcode family at each corpus's pinned tip, with the measured features (top-level node kinds,
  kept out-edges, the basename and stem spelled elsewhere, convention path segments, document class) and the text's
  sha256. Generator `eval_deadfile_parts::generate::deadfile_slice` (`#[ignore]`, needs the clone).
- **The sample** (`contracts/eval/deadfile-sample-<language>-v1.json`): per language, the primary rows the language
  exams' quota rule draws over the corpora and a reserve; a language smaller than the whole-below constant is taken
  whole. The gate `it/eval_deadfile.rs` redraws it from the frozen universe.
- **The classes** (`contracts/eval/deadfile-class-<language>-v1.json`, `ce.eval-deadfile-class`): every sample row's K1
  and K1-strict class (BARS section 2 with addenda 2 and 3) with the witness statement, and the lane's first
  classifier K0 beside it. Frozen and committed before any answer file was opened; the gate
  `it/eval_deadfile_class.rs` re-derives everything but K1, `classes_rederive` (`#[ignore]`) re-derives K1 from the
  clones.
- **The record**, copied byte for byte from the main session's review directory and pinned by sha256 in
  `audit/record.rs` (the gate `the_record_is_the_main_sessions_bytes` holds every byte; the files are listed under
  "The record" below). Source names, in that table's order: `BARS.md`, `BARS-ADDENDUM-1.md` … `-4.md`,
  `settlement.json`, `convention_docs.json`, the files of `answers/` and `answers2/` of the main session's review
  directory, and the lane's `AUDITOR-BRIEF.md`.
- **The review docs** (`contracts/eval/deadfile-review-<language>-v1.json`, `ce.eval-deadfile-review`): per sample row
  its first and second reading, the BARS section 1 check and the addendum 1 B tag of each reading read off the pinned
  tree, the second-reading reasons, the final answer and its tag. Generator `audit::derived::deadfile_review_docs`
  (`#[ignore]`); the gate `every_review_row_follows_from_the_record` rebuilds every field from the committed answers,
  the settlement and the recorded check and tag, and refuses an answer that fails the check;
  `deadfile_reviews_rederive` (`#[ignore]`) reads the check and the tag off the clones again.
- **Population B** (`contracts/eval/deadfile-population-b-v1.json`, `ce.eval-deadfile-population-b`): per corpus, a copy
  of the clone judged by this build's `ce deadcode` twice, with the corpus's own `ce.toml` and with every file a
  passing section 3 position rule matches added to `[graph] entry_globs` as a literal; the run stops unless the rows
  that drop are the matched dead rows and what they reach along the kept arcs. Generator
  `audit::derived::deadfile_population_doc` (`#[ignore]`); the gate
  `population_b_stands_on_the_slices_and_the_passing_set` holds it to the frozen universe and to section 3's
  passing set.

## The record

| record file | sha256 |
|---|---|
| `contracts/eval/deadfile-bars-v1.md` | `2934ab1c7f33f3373065a8fbc37b349a35b816481a882a4e94b7c3145a29fa65` |
| `contracts/eval/deadfile-bars-addendum-1-v1.md` | `d42c6f7020c05dac791901f75d137fd9d6c42f29a94bee643cbc4f80a0024a31` |
| `contracts/eval/deadfile-bars-addendum-2-v1.md` | `e2d5860908eb00ca722d4b83a1c557563e72feba3d1fb59f88e8c63b11baa07b` |
| `contracts/eval/deadfile-bars-addendum-3-v1.md` | `8bc0404efe4e4b5fc32bc41029c9c3c95bb7f55fcfecf4b2111c6668aaf59667` |
| `contracts/eval/deadfile-bars-addendum-4-v1.md` | `8e496a0bfde490828defa81df1980aa303df411e4acc8298859833f5140245f9` |
| `contracts/eval/deadfile-settlement-v1.json` | `b7c9f6aa86224f381e9c0df9f2a19b24fbd49ff75a53cd67187b5ba113e83816` |
| `contracts/eval/deadfile-convention-docs-v1.json` | `9598a85ca1ea28a3b4485048010e1ba2da658935867f559099c17738860cb71e` |
| `contracts/eval/deadfile-answers-v1/cpp-python.jsonl` | `c852aaed2f45e6c0331e23046c133987003c6669ed1b7c4172a129c435f49f8a` |
| `contracts/eval/deadfile-answers-v1/html-1.jsonl` | `95fdca32c5873237e4c8bf2a649638a136c2eeb47f2f836a0eeea4f1dfcb44a9` |
| `contracts/eval/deadfile-answers-v1/html-2.jsonl` | `055f439bb4f960b968ec2ba18f10a87366fcbfeb8c58de7d3de72826b0f5de45` |
| `contracts/eval/deadfile-answers-v1/java-gson.jsonl` | `db3cfc46b003260375d4921ca604ab8f3a08316d9b4d2b18877c0e0905205e3d` |
| `contracts/eval/deadfile-answers-v1/java-jsoup.jsonl` | `e1471ff13f005c32557a1673365d718097d4c6a74e006a88f52bfa7f27d3096c` |
| `contracts/eval/deadfile-answers-v1/lua-koreader-1.jsonl` | `835bf42eb59da726f074834dce9a13e2a5d4277a131d9fc3672710207d761794` |
| `contracts/eval/deadfile-answers-v1/lua-koreader-2.jsonl` | `abac7d91cb8c95cfef52a2beda84772d075d480809d671873c0497f66b662cb9` |
| `contracts/eval/deadfile-answers-v1/lua-lua.jsonl` | `3c43399d22e5f2e20dbb4fc1f9c8f45fbc60831c0be65db9ecb22e907254c225` |
| `contracts/eval/deadfile-answers-v1/lua-luarocks-1.jsonl` | `07c06f0f93d04e10bd1e675bf33edc0a6f412b21cfb02298641b14b21a7bb43a` |
| `contracts/eval/deadfile-answers-v1/lua-luarocks-2.jsonl` | `3931a1cb0ad7919bd01f49639bb647fb665f190760f5e87ee33d02b12a8ef35c` |
| `contracts/eval/deadfile-answers-v1/markdown-1.jsonl` | `4511c9f69f6d321efe7830f08a9df6c93e33c5fe4a2fea60a1845f371c413a3b` |
| `contracts/eval/deadfile-answers-v1/markdown-2.jsonl` | `3af4b9d282dccb4e1acf13cd94a3b0b22795b649eef79f8390d7d09fe5aa8b1e` |
| `contracts/eval/deadfile-answers-v1/r-covid19model-1.jsonl` | `1aa9f9d63e861f755c4839a0036b6d14510dd1f9108133a76fa44d7b5933155d` |
| `contracts/eval/deadfile-answers-v1/r-covid19model-2.jsonl` | `d099d06a2bb1970d7db6c8cd44e4f72e1440dff15942d036e4077d0ca7a728ab` |
| `contracts/eval/deadfile-answers-v1/r-covid19model-3.jsonl` | `1124069d7b1ad4d1f0adc5c50eb49028c2b33d98ff59bfc85db5467da791dabe` |
| `contracts/eval/deadfile-answers-v1/tsx-zod-1.jsonl` | `42dc500ccf6fd6d083dd438d458d1a863327f8040a20f059d1133e142b7a2525` |
| `contracts/eval/deadfile-answers-v1/tsx-zod-2.jsonl` | `8ca83f601972e8c05138b1a6ba92fe0e5d45e0ad70721e92fbd41033ba34f6f3` |
| `contracts/eval/deadfile-answers-v1/typescript-zod-1.jsonl` | `9870eb501bf213610f05937e58af277ad835d84b729eeea2c1a2678b656a23db` |
| `contracts/eval/deadfile-answers-v1/typescript-zod-2.jsonl` | `f141947ff844ade75dda8a045d1954074b3f4547caca376e79734e156a835e30` |
| `contracts/eval/deadfile-answers-v1/typescript-zod-3.jsonl` | `f6911b4c69c54d0b6af664456018d2cbe7ac9d0b1adb58b9b0cfd2020a97502c` |
| `contracts/eval/deadfile-answers-v1/typescript-zod-4.jsonl` | `68df49e1e831e8afb2e430f9aee44259a7a8eb354839a17e47c7d005399d2791` |
| `contracts/eval/deadfile-answers2-v1/second-1.jsonl` | `282e718bbef7e3601719b744ac9289690a8e0c87567e3811ee1f7e349cf976d5` |
| `contracts/eval/deadfile-answers2-v1/second-2.jsonl` | `5fd4532769a5fab7d7cf246ba6b8abe016bcc2d9061f08ab5dd1f988fbe53af8` |
| `contracts/eval/deadfile-answers2-v1/second-3.jsonl` | `223b3eb2ccca6344725d3c08b270d384c814142952c68e1406a55c22c73bc147` |
| `contracts/eval/deadfile-answers2-v1/second-4.jsonl` | `bc77b61dc4de7d5e415b5f913766ddf7580be1376cf8cf2701d4c93b2ed40de1` |
| `contracts/eval/deadfile-auditor-brief-v1.md` | `04d032746a79f787d41a4fa03389b05f297ea42f3277d7e2ba4880563b871372` |

## What was sampled

| language | universe rows | primary sample | reserve |
|---|---|---|---|
| cpp | 8 | 8 | 0 |
| html | 29 | 29 | 0 |
| java | 43 | 43 | 0 |
| lua | 235 | 100 | 40 |
| markdown | 44 | 44 | 0 |
| python | 11 | 11 | 0 |
| r | 62 | 62 | 0 |
| tsx | 29 | 29 | 0 |
| typescript | 98 | 98 | 0 |
| all | 559 | 424 | 40 |

## The bars, as frozen

Quoted from [deadfile-bars-v1.md](../contracts/eval/deadfile-bars-v1.md), section 1 (reading the answers):

> - Each primary row gets one answer from an independent auditor who sees only (corpus, commit, path): `in_use`,
>   `unused` or `unsure`.
> - Second reading, by a different auditor who does not see the first answer: every row whose
>   `int(sha256("ce-deadfile-doubleread-v1|" + corpus + "|" + commit + "|" + path)[:8], 16) % 5 == 0`, every row
>   answered `unsure`, and every row answered `unused` that carries a convention named in section 3.
>   Agreement = share of twice-read rows where both readings fall on the same side of (in_use / not in_use).
>   Agreement below 0.90 → every row is read twice.
> - A row with two different answers is settled by the evidence, not by a vote: `in_use` stands only when its cited
>   `file:line` exists at the pinned tip and does what the answer says; otherwise the row is `unsure`.
> - A mechanical check runs over every answer before any table is built: an `in_use` answer must cite a path that
>   exists at the pinned tip (and a line inside the file) or a named tool convention; an `unused` answer must say
>   where it searched. An answer that fails goes back to a second auditor; it is never repaired by hand.
> - Shares: `in(C)` = weighted share of rows of class C answered `in_use`; `un(C)` = weighted share answered
>   `unused`; `unsure` belongs to neither. Weight of a row = pool(language, corpus) / primary rows of that
>   (language, corpus); this is 1 everywhere except Lua.
> - The audited rows are every dead-file row of twelve projects (Lua: a sample). The numbers are exact for these
>   projects; no interval is claimed for other projects. Robustness is asked through the per-project floors below,
>   and every share is also printed with each project left out in turn.
> <!-- ce:allow(docdup) -- quoted verbatim from the frozen record under contracts/eval -->

Section 3 (a convention becomes an entry name):

> A file name or path pattern enters a language's entry table only when all three hold:
> 1. It is a tool's convention, not a project's: the name or position is fixed by a compiler, build tool, package
>    manager, test runner, documentation generator, framework or forge, and that tool's own documentation says it
>    reads the file by that name. The table row carries the citation, and encodes the position rule the tool uses
>    (a name the tool reads only at one place is not entered as a bare basename).
> 2. Every audited row that carries it is `in_use`, none `unused`, none `unsure`, and at least one answer names
>    that tool as the mechanism without having been told the hypothesis.
> 3. At least one audited row carries it (the tool's documentation is the evidence for the rule; the audit can only
>    refute it).
>
> A project's own convention (a plugin loader's file name, a project's test directory, a package script's
> directory) never enters a language table. Its road is the project's own `[graph] entry_globs`; its rows stay in
> section 4.
> <!-- ce:allow(docdup) -- quoted verbatim from the frozen record under contracts/eval -->

Section 4 (which unreferenced files may fail `ce deadcode --check`):

> Population: audited rows left after section 3, by K1 class. A class with fewer than 30 rows left is not judged
> and keeps today's behaviour.
>
> For each class C:
> - ADVISORY when `in(C) >= 0.80`, and every project with at least 10 rows of C left has `in >= 0.60` there, and
>   at least two projects have 10 rows of C left. The rows of an advisory class stop failing `--check`; they stay
>   in the report under their own word.
> - otherwise FAILING (today's behaviour).
>
> What ships:
> - `runs-alone` ADVISORY and `in(declares) <= in(runs-alone) - 0.25` → the split ships: `runs-alone` rows are
>   reported and no longer fail, `declares` rows keep failing.
> - `runs-alone` ADVISORY and `declares` ADVISORY → no split: one advisory word for every unreferenced code file.
> - `runs-alone` ADVISORY and `declares` neither advisory nor 0.25 below → nothing ships from this audit; the
>   numbers go to the owner as a question about the gate's default.
> - `runs-alone` FAILING → nothing ships.
> - `document` is judged by the same ADVISORY rule on its own.
> - Any class left FAILING with `in(C) >= 0.50` (the failing verdict is wrong more often than right) is reported
>   to the owner as a question; this audit does not decide it.
>
> 0.80, 0.60, 0.25 and 0.90 are judgment constants fixed here, not derived: at 0.80 at most one in five files
> that stop failing was really unused, and that file is still printed in the report.
> <!-- ce:allow(docdup) -- quoted verbatim from the frozen record under contracts/eval -->

Section 5 (what this audit cannot do), first line:

> - It licenses no deletion, whatever it finds. `ce erase` deletes no whole file (L1); a population that could be
>   deleted would need its own exam and its own false-refusal record.
> <!-- ce:allow(docdup) -- quoted verbatim from the frozen record under contracts/eval -->

Addendum 1 ([deadfile-bars-addendum-1-v1.md](../contracts/eval/deadfile-bars-addendum-1-v1.md)), the population of
section 4 and the mechanical tag:

> Both readings are computed and printed:
> - reading A (as registered): population = audited rows minus the rows carrying a passing convention;
> - reading B: population = audited rows the product still reports dead when every passing convention is declared
>   an entry — computed by the product itself (`[graph] entry_globs` in a scratch `ce.toml` of the corpus clone
>   copy, `ce deadcode`), never by hand.
>
> Decision: a §4 outcome ships only when readings A and B give the same outcome for that class. If they differ,
> nothing ships for that class from this audit and the numbers go to the owner.
> <!-- ce:allow(docdup) -- quoted verbatim from the frozen record under contracts/eval -->

> For every `in_use` answer the review table also carries a mechanical tag, computed from the cited evidence only:
> - `named` — some cited `path:line` exists at the pinned tip and that line, or one of the two lines on either
>   side, contains the row's basename or its stem as a word;
> - `generic` — every existing citation fails that test (a glob, a directory rule, a generic runner), or the
>   answer names a convention with no citation.
>
> Every share of BARS §4 is printed twice: with all `in_use` answers, and with only `named` ones (`generic`
> counted as not in use). The bars of §4 are evaluated on the first, as registered. If a class is ADVISORY on the
> first and would not be on the second, that is stated next to the outcome and reported to the owner with it.
> <!-- ce:allow(docdup) -- quoted verbatim from the frozen record under contracts/eval -->

Addendum 2 ([deadfile-bars-addendum-2-v1.md](../contracts/eval/deadfile-bars-addendum-2-v1.md)), the two classifiers:

> 3. The class of every row is frozen twice: `k1` (section 2 with the rulings below) and `k1_strict` (section 2
>    read with no ruling: every construct the lane's inventory reported as undecided is ACTIVE / not
>    constant-like). Every table of section 4 is printed under both. An outcome of section 4 ships without a
>    question only when it is the same under `k1` and `k1_strict` (and, per addendum 1, under readings A and B).
>    If they differ, nothing ships for that class from this audit and both sets of numbers go to the owner.
> <!-- ce:allow(docdup) -- quoted verbatim from the frozen record under contracts/eval -->

Addendum 3 ([deadfile-bars-addendum-3-v1.md](../contracts/eval/deadfile-bars-addendum-3-v1.md)), a fragment the
parser could not place:

> - (W) Witness. The class `runs-alone` rests on a witness: a statement the parser read and section 2 calls
>   active. A fragment the parser could not place is no witness of anything — the classifier does not know what
>   it is.
>
> So under `k1` a top-level `ERROR` fragment is not a witness: it does not make the row `runs-alone`. The row
> records ruling 16 and the fragment's line. Under `k1_strict` it is active, like every construct the inventory
> reported as undecided. A `MISSING` node belongs to the parsed statement that holds it and needs no ruling. An
> `ERROR` node inside a binding's right-hand side is not on rule 5's list, so that binding is active under both
> classes (the letter decides).
> <!-- ce:allow(docdup) -- quoted verbatim from the frozen record under contracts/eval -->

Addendum 4 ([deadfile-bars-addendum-4-v1.md](../contracts/eval/deadfile-bars-addendum-4-v1.md)), how a position rule is
derived from a tool's documentation:

> - (F) relative to the repository or to a named directory — then the rule is exactly those places;
> - (I) relative to the invocation (the directory the tool runs in, or a directory given on its command line) —
>   then the documentation fixes no place inside a repository: any directory may be that project root, and the
>   rule is "the name, in any directory". Rule 2 (every audited carrying row `in_use`) is the audit's check on it.
>
> Where the documentation is silent or contradicts itself, the narrower reading is taken and the gap is named.
> <!-- ce:allow(docdup) -- quoted verbatim from the frozen record under contracts/eval -->

## Who answered

Every primary row was answered by independent auditors who saw only the corpus, the commit and the path, never the
product's answer, the classes or the hypothesis, under the brief
[deadfile-auditor-brief-v1.md](../contracts/eval/deadfile-auditor-brief-v1.md). The second reading was done by other
auditors who did not see the first answer, on the rows BARS section 1 names. Two different answers on one row were
settled by the main session from the evidence, never by a vote
([deadfile-settlement-v1.json](../contracts/eval/deadfile-settlement-v1.json)); no answer failed the mechanical check.

| quantity | value |
|---|---|
| rows answered (first reading) | 424 |
| in_use / unused / unsure | 392 / 28 / 4 |
| in_use tagged named / generic | 278 / 114 |
| rows read twice | 98 |
| same side of in_use, all rows read twice | 94 / 98 (0.959) |
| same side, first answer in_use and generic | 29 / 33 (0.879) |
| same side, first answer in_use and named | 52 / 52 (1.000) |
| same side, first answer not in_use | 13 / 13 (1.000) |
| final in_use / unused / unsure | 392 / 28 / 4 |

The rows whose two readings gave two different answers, and the settled word:

| corpus | path | first | second | settled |
|---|---|---|---|---|
| zod | `packages/docs-v3/CONTRIBUTING.md` | unsure | unused | unsure |
| zod | `packages/docs-v3/MIGRATION.md` | unsure | unused | unsure |
| zod | `packages/docs/pages/api/_og.tsx` | unsure | unused | unsure |
| zod | `packages/treeshake/example.ts` | in_use | unused | in_use |
| zod | `packages/treeshake/valibot-boolean.ts` | in_use | unsure | in_use |
| zod | `packages/treeshake/zod3-boolean.ts` | in_use | unsure | in_use |
| zod | `packages/treeshake/zod3-object.ts` | in_use | unsure | in_use |

## Section 3: which conventions pass

Position rules and citations: [deadfile-convention-docs-v1.json](../contracts/eval/deadfile-convention-docs-v1.json)
(addendum 4). The `beside` line is the bare-basename reading of one candidate, printed for comparison; only the
documented rule is evaluated.

| candidate | rows | in_use / unused / unsure | an answer names the tool | documented | position rule | outcome |
|---|---|---|---|---|---|---|
| `package-info.java` | 13 | 13 / 0 / 0 | yes | yes | yes | passes |
| `module-info.java` | 4 | 4 / 0 / 0 | no | yes | yes | refused by rule 2 |
| `vitest.config` | 4 | 4 / 0 / 0 | yes | yes | yes | passes |
| `next-app-entry` | 9 | 9 / 0 / 0 | yes | yes | yes | passes |
| `setup.py` | 1 | 1 / 0 / 0 | yes | yes | yes | passes |
| `sphinx-conf.py` | 1 | 1 / 0 / 0 | yes | yes | yes | passes |
| `tlconfig.lua` | 0 | 0 / 0 / 0 | no | yes | yes | refused by rule 3 |
| `busted-spec` | 0 | 0 / 0 / 0 | no | yes | yes | refused by rule 3 |
| `maven-test-resources` | 27 | 26 / 1 / 0 | no | yes | yes | refused by rule 2 |
| `CONTRIBUTING.md` | 3 | 3 / 0 / 0 | yes | yes | yes | passes |
| - beside (printed for comparison, not evaluated) | 4 | 3 / 0 / 1 | yes | yes | yes | refused by rule 2 |
| `SECURITY.md` | 3 | 3 / 0 / 0 | yes | yes | yes | passes |
| `CODE_OF_CONDUCT.md` | 2 | 2 / 0 / 0 | yes | yes | yes | passes |
| `koreader-plugin-meta` | 0 | 0 / 0 / 0 | no | no | no | stays out by rule 1 (a project's convention) |
| `lua-testes` | 0 | 0 / 0 / 0 | no | no | no | stays out by rule 1 (a project's convention) |
| `zod-bench` | 0 | 0 / 0 / 0 | no | no | no | stays out by rule 1 (a project's convention) |

## Population B

| corpus | dead before -> after | matched files | reached beyond matched |
|---|---|---|---|
| cobra | 0 -> 0 | 2 (0 dead) | 0 |
| codeeraser | 0 -> 0 | 1 (0 dead) | 0 |
| covid19model | 64 -> 64 | 1 (0 dead) | 0 |
| fmt | 12 -> 11 | 1 (1 dead) | 0 |
| gson | 21 -> 16 | 5 (5 dead) | 0 |
| jsoup | 53 -> 44 | 9 (9 dead) | 0 |
| koreader | 150 -> 150 | 0 (0 dead) | 0 |
| lua | 34 -> 34 | 0 (0 dead) | 0 |
| luarocks | 69 -> 67 | 2 (2 dead) | 0 |
| requests | 5 -> 3 | 2 (2 dead) | 0 |
| ripgrep | 4 -> 3 | 1 (1 dead) | 0 |
| stringr | 5 -> 5 | 0 (0 dead) | 0 |
| zod | 142 -> 114 | 16 (16 dead) | 12 |

## Section 4: the eight readings

Classifier `k1` / `k1_strict` (addendum 2), population A / B (addendum 1 A), `in_use` counted with every answer or with
named ones only (addendum 1 B). Shares are weighted (BARS section 1); the bars are evaluated on every answer.

| reading | in_use counted | runs-alone | declares | document | code outcome (all answers) |
|---|---|---|---|---|---|
| k1/A | all | 237 rows, in 0.948, ADVISORY | 86 rows, in 0.905, ADVISORY | 65 rows, in 0.846, FAILING | no split: one advisory word for every unreferenced code file |
| k1/A | named | 237 rows, in 0.701, FAILING | 86 rows, in 0.737, FAILING | 65 rows, in 0.769, FAILING | no split: one advisory word for every unreferenced code file |
| k1/B | all | 233 rows, in 0.948, ADVISORY | 78 rows, in 0.897, ADVISORY | 65 rows, in 0.846, FAILING | no split: one advisory word for every unreferenced code file |
| k1/B | named | 233 rows, in 0.698, FAILING | 78 rows, in 0.716, FAILING | 65 rows, in 0.769, FAILING | no split: one advisory word for every unreferenced code file |
| k1_strict/A | all | 243 rows, in 0.949, ADVISORY | 80 rows, in 0.900, ADVISORY | 65 rows, in 0.846, FAILING | no split: one advisory word for every unreferenced code file |
| k1_strict/A | named | 243 rows, in 0.695, FAILING | 80 rows, in 0.761, FAILING | 65 rows, in 0.769, FAILING | no split: one advisory word for every unreferenced code file |
| k1_strict/B | all | 239 rows, in 0.949, ADVISORY | 72 rows, in 0.891, FAILING | 65 rows, in 0.846, FAILING | nothing ships: owner question about the gate's default |
| k1_strict/B | named | 239 rows, in 0.692, FAILING | 72 rows, in 0.741, FAILING | 65 rows, in 0.769, FAILING | nothing ships: owner question about the gate's default |

Leave one project out — every line where taking one project's rows out flips a class's verdict:

| reading | in_use counted | class | project left out | verdict | verdict without it |
|---|---|---|---|---|---|
| k1/A | named | runs-alone | zod | FAILING | ADVISORY (in 0.830) |
| k1/B | named | runs-alone | zod | FAILING | ADVISORY (in 0.830) |
| k1_strict/A | named | runs-alone | zod | FAILING | ADVISORY (in 0.818) |
| k1_strict/A | named | declares | jsoup | FAILING | ADVISORY (in 0.813) |
| k1_strict/B | named | runs-alone | zod | FAILING | ADVISORY (in 0.818) |
| k1_strict/B | all | declares | zod | FAILING | ADVISORY (in 0.958) |

Same under all readings?

| class | same under all readings? |
|---|---|
| runs-alone | no: k1/A/all "ADVISORY" (in 0.948); differs: k1/A/named "FAILING" (in -0.247); k1/B/named "FAILING" (in -0.250); k1_strict/A/named "FAILING" (in -0.253); k1_strict/B/named "FAILING" (in -0.257) |
| declares | no: k1/A/all "ADVISORY" (in 0.905); differs: k1/A/named "FAILING" (in -0.168); k1/B/named "FAILING" (in -0.189); k1_strict/A/named "FAILING" (in -0.144); k1_strict/B/all "FAILING" (in -0.014); k1_strict/B/named "FAILING" (in -0.164) |
| document | yes ("FAILING", rows 65, in 0.846) |

The agreed outcomes (an outcome ships only when `k1` / `k1_strict` and population A / B agree):

- code: owner question: the readings differ (readings agree: false)
- document: FAILING (readings agree: true)

## Outcome

Generation 1 ships no split of unreferenced files: the code outcome is an owner question (the readings differ) and the document outcome is FAILING under every reading and, failing with an in-share of at least 0.50, an owner question. Eight tool conventions pass BARS section 3: `package-info.java`, `vitest.config`, `next-app-entry`, `setup.py`, `sphinx-conf.py`, `CONTRIBUTING.md`, `SECURITY.md`, `CODE_OF_CONDUCT.md`.

The owner was asked how an unreferenced file should be treated by default and answered 「想办法降低误拦」 (find a way
to lower the false blocks). Generation 2 is designed by the main session and is recorded with its own exam:
[EVAL-SET-DEADFILE-GEN2.md](EVAL-SET-DEADFILE-GEN2.md).
