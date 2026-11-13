# Dead-file exam registry, generation 2 (2026-11-13)

> The frozen record of the dead-file exam's second generation (lane v235_licence, part 4B): which states of an
> unreferenced file may fail the gate by default, read from what first-parent history did with each file the
> product newly called dead, on ten repositories the product had never been run on, under bars written before the
> instrument existed. Generation 1, the blind audit of the standing dead files of thirteen corpora:
> [EVAL-SET-DEADFILE.md](EVAL-SET-DEADFILE.md). The page joins the frozen set (`frozen_set.rs`: no chip, no
> generated block, opted out of the citation gate); the record it reads is pinned byte for byte by sha256 in
> `it/eval_deadfile_gen2_record.rs`, which also holds this page to carrying each rendered table verbatim and every
> quoted line to the bars file it is cut from. Nothing here gates the product: the exam licenses no deletion and
> changes no verdict.

## Instruments, records and gates

- **The bars and the design**
  ([deadfile-gen2-bars-v1.md](../contracts/eval/deadfile-gen2-bars-v1.md),
  [deadfile-gen2-design-v1.md](../contracts/eval/deadfile-gen2-design-v1.md)): the main session's bytes, frozen on
  2026-11-11 before the instrument existed, pinned in `eval_deadfile_gen2_parts/mod.rs`.
- **The registry** (`contracts/eval/deadfile-gen2-registry-v1.json`): the ten primary corpora, their tips and the
  product commit, committed before any replay printed a count.
- **The instrument** (`it/eval_deadfile_gen2*.rs`, `it/eval_deadfile_gen2_parts/`): section 2 of the bars as code -
  the window, the two events, the four fates, the two readings, the bars, the audit's selection and reading C.
  Three `#[ignore]` legs run on the registered clones (`deadfile_gen2_tips`, `deadfile_gen2_replay`,
  `deadfile_gen2_unjudged`) and one renders the report from the replay's docs (`deadfile_gen2_report`). No line of
  it changed after its first printed count.
- **The replay's docs** (`contracts/eval/deadfile-gen2-replay-v1.<corpus>.json`,
  `ce.eval-deadfile-gen2-replay/2.0.0`): per corpus the replayed chain, every event with its state, kind, fate,
  deciding commit and the file's name there, the unjudged and retried trees and the side numbers; each carries the
  tip and the product record (commit, both binaries' sha256, the instrument's commit).
- **The audit of the undetermined** (`contracts/eval/deadfile-gen2-audit-v1.json`,
  `ce.eval-deadfile-gen2-audit/1.0.0`): the rows section 3 draws, both readings and the settled answer; the main
  session's bytes.
- **The acceptance**
  ([deadfile-gen2-acceptance-v1.md](../contracts/eval/deadfile-gen2-acceptance-v1.md)): section 6, item 3 - the
  coordinator's check of the outcome sentences against the printed integers and of twenty fates by hand, written
  before any table was accepted; the main session's bytes.
- **The gate** `it/eval_deadfile_gen2_record.rs`: every record file's bytes; the docs against the registry
  (corpus, tip, product commit); the audit's rows against section 3's selection and its settled answers against
  the readings; this page's tables and quotations.

## The record

| record file | sha256 |
|---|---|
| `contracts/eval/deadfile-gen2-bars-v1.md` | `8444a90834fcf8f430dbff0aa0cb76213d4b10a5934cfdb8753a991035d53a3e` |
| `contracts/eval/deadfile-gen2-design-v1.md` | `224437fec7086f6cf990087dd8331ea9310ee0d52f7605f641d1ef47eacc2514` |
| `contracts/eval/deadfile-gen2-replay-v1.Penlight.json` | `35d909b1abe57741ab7ff3cd61ca4a22f29437145fe435a2c578435fef18907b` |
| `contracts/eval/deadfile-gen2-replay-v1.fd.json` | `ffb6622af2ec682792702db5674f1a972caa0402c072788551305280f6c1fe78` |
| `contracts/eval/deadfile-gen2-replay-v1.flask.json` | `a6e342220735278e0a371e345dddf711f92d321a35d8b5bb111b0c486b473286` |
| `contracts/eval/deadfile-gen2-replay-v1.hiredis.json` | `aa9b3993a45fd98640a90afdac0c304a635dcf85bb071dc6579eae47bcc40d2c` |
| `contracts/eval/deadfile-gen2-replay-v1.hlint.json` | `145d4b048ba52ab5d3846e56d4f8e273bc45f3919210d2e13228a74307e5d0ce` |
| `contracts/eval/deadfile-gen2-replay-v1.junit4.json` | `a237151a8e6a183f63ae6da571392ba0ded4d4f3983f6ee1859c5848cf592d7d` |
| `contracts/eval/deadfile-gen2-replay-v1.purrr.json` | `091c2bae35dce6e8984339ae60f3d1f3933c9a48103c6562b01c669ffae7f37d` |
| `contracts/eval/deadfile-gen2-replay-v1.spdlog.json` | `9640b89bc2f76a31b14ea05f79a068d3d370dc238972a66001d0b2308fe65fc0` |
| `contracts/eval/deadfile-gen2-replay-v1.viper.json` | `bb30c78c84ec4e1d10c272094e86a0dbb3b0dde5fbb236fe25b6bbd040fe84f9` |
| `contracts/eval/deadfile-gen2-replay-v1.zustand.json` | `8076e1832d26d0fc83a168bef1e2f5c30a5e0c5508a41a164664dfe01d6bf029` |
| `contracts/eval/deadfile-gen2-audit-v1.json` | `53a972cb1982981727c905ce5a347b3ed6a078d4d4a1a830ee27a8d101985acc` |
| `contracts/eval/deadfile-gen2-acceptance-v1.md` | `25412e628c9409512592299870585b8d9957b6658a10e4e9ead445bd4b805948` |

## The product and the corpora

product "5a3f959380b339031f06b7cd01c452b3e1244729" ("release"), ce sha256 "ad7f28abd9a7de206946d8ae201a77077a02db8d6d21dd61349de231dd542ec9", ce-core sha256 "54dbf46e47aa55d90fff199ba463c6c143c4337b7ae8fbc0ea9b24a1fcec8be3", instrument "244a0dd3ce82e3493e2f21c2b118ac866c669ab0"
corpus "Penlight" at "99b7542a047123bb484510d34ee567bca40b119e"
corpus "fd" at "24d90b28d25438af2a8f2e7f7e46d03d6828da94"
corpus "flask" at "49943b5a94413a530761350f04e896f76e8d620e"
corpus "hiredis" at "5bb7f030f6631b243438cf0fdd3be84352b905bb"
corpus "hlint" at "1cc1908555ab0ff3570188f70d3eb78cddb86218"
corpus "junit4" at "890f3c972647de378f25e7271d8fbbd9d3456b79"
corpus "purrr" at "b213ceab2756e56b8ac8d26b1d0dabec0cbdb77f"
corpus "spdlog" at "6d9805df3f588402b17073dcd13eb569ceda864d"
corpus "viper" at "528f7416c4b56a4948673984b190bf8713f0c3c4"
corpus "zustand" at "9fed2eb2f3af672da9f7bf694b996df3831fc207"

Nine first choices were judged at their tips. C's first choice was refused at its tip and replaced by the
pre-registered replacement, section 1's second reason; the registry's row holds the product's own words:

C: jqlang/jq -> redis/hiredis: the product refuses the first choice's tip (jqlang/jq 5c001b18d7237cce1fdba63608732163f978dd23, 1781 first-parent commits): exit 2: ce deadcode: mentions: submodule vendor/oniguruma is not checked out under . — `git submodule update --init` first

## The bars, as frozen

Sections 2 to 4 of `deadfile-gen2-bars-v1.md`, the lines that define an event, a fate, the two readings and the
bars:

> - event **orphaned** at c: a root at c; the file exists at p; it is not dead at p.
> - event **born** at c: a root at c; the file does not exist at p.
> - not an event: a root at c that was dead at p (standing: root or dependent there); a dead file at c that is not
>   a root (a dependent; counted in a side table, part A attaches it to its root).
>
> Fate of an event, read forward along the first-parent chain from c to the tip, renames followed; the FIRST of
> these that happens decides:
>
> 1. **re-referred**: at some later commit the file exists and is not dead -> the block was false.
> 2. **kept up**: while still dead, a later commit changes the file's content (a rename alone is no change).
>    `focused` when that commit changes at most 20 files, `sweep` otherwise -> see the readings.
> 3. **removed untouched**: a later commit deletes it, with no content change since c -> the block was right.
> 4. none by the tip -> **undetermined**.
>
> - Reading A (upper): false = re-referred + kept up (focused and sweep) + undetermined; denominator = all events.
> - Reading B (lower): false = re-referred + kept up focused; denominator = those + removed untouched; kept up by a
>   sweep alone and undetermined are left out.
>
> A state BLOCKS BY DEFAULT under a reading iff, on the primary corpora:
>
> 1. it has at least 100 events;
> 2. its pooled false share is at most 1.0 % (the bar every blocking class of this product answers to);
> 3. no single corpus with at least 30 events of the state has a false share above 5.0 %;
> 4. conditions 2 and 3 also hold for each kind (`code`, `document`) that has at least 100 events of its own.
>
> - BLOCKS under A and under B -> the state fails the gate by default.
> - misses under A and under B -> the state is listed, never fails the gate by default, and its line carries the
>   measured share; a project may switch it to blocking for itself.
> - A and B disagree -> the owner is asked, in plain words, with the two shares and reading C.
> - fewer than 100 events -> not judged: it does not block (a class with no false-block record of its own does not
>   block), and the count is reported.
> - **standing** rows never block, whatever the numbers: they are the inventory, their remedy is to declare an entry
>   or to delete.
> <!-- ce:allow(docdup) -- quoted verbatim from the frozen record under contracts/eval -->

## Before any count

The unjudged side was read before the report ran:

"Penlight": 713 commits, 0 unjudged trees, 0 retried trees
"fd": 801 commits, 0 unjudged trees, 0 retried trees
"flask": 801 commits, 0 unjudged trees, 0 retried trees
"hiredis": 788 commits, 0 unjudged trees, 0 retried trees
"hlint": 801 commits, 0 unjudged trees, 0 retried trees
"junit4": 801 commits, 0 unjudged trees, 0 retried trees
"purrr": 801 commits, 0 unjudged trees, 0 retried trees
"spdlog": 801 commits, 0 unjudged trees, 0 retried trees
"viper": 801 commits, 0 unjudged trees, 0 retried trees
"zustand": 801 commits, 0 unjudged trees, 0 retried trees

A pair in which the product refuses either tree yields no event, so a refused tree is where a count could be wrong
without showing. There is none. The coordinator checked on the registered clones that no commit of any replayed
chain touches `.gitmodules`: the refusal that removed C's first choice does not occur unseen.

## The report

### orphaned

| corpus / kind | events | re-referred | kept up focused | kept up sweep | removed untouched | undetermined | false A | false B |
|---|---|---|---|---|---|---|---|---|
| Penlight | 4 | 1 | 2 | 0 | 1 | 0 | 3/4 (75.0 %, upper 99.4 %) | 3/4 (75.0 %, upper 99.4 %) |
| Penlight code | 2 | 1 | 0 | 0 | 1 | 0 | 1/2 (50.0 %, upper 98.7 %) | 1/2 (50.0 %, upper 98.7 %) |
| Penlight document | 2 | 0 | 2 | 0 | 0 | 0 | 2/2 (100.0 %, upper 100.0 %) | 2/2 (100.0 %, upper 100.0 %) |
| fd | 0 | 0 | 0 | 0 | 0 | 0 | 0/0 (0.0 %, upper 100.0 %) | 0/0 (0.0 %, upper 100.0 %) |
| fd code | 0 | 0 | 0 | 0 | 0 | 0 | 0/0 (0.0 %, upper 100.0 %) | 0/0 (0.0 %, upper 100.0 %) |
| fd document | 0 | 0 | 0 | 0 | 0 | 0 | 0/0 (0.0 %, upper 100.0 %) | 0/0 (0.0 %, upper 100.0 %) |
| flask | 0 | 0 | 0 | 0 | 0 | 0 | 0/0 (0.0 %, upper 100.0 %) | 0/0 (0.0 %, upper 100.0 %) |
| flask code | 0 | 0 | 0 | 0 | 0 | 0 | 0/0 (0.0 %, upper 100.0 %) | 0/0 (0.0 %, upper 100.0 %) |
| flask document | 0 | 0 | 0 | 0 | 0 | 0 | 0/0 (0.0 %, upper 100.0 %) | 0/0 (0.0 %, upper 100.0 %) |
| hiredis | 6 | 2 | 4 | 0 | 0 | 0 | 6/6 (100.0 %, upper 100.0 %) | 6/6 (100.0 %, upper 100.0 %) |
| hiredis code | 6 | 2 | 4 | 0 | 0 | 0 | 6/6 (100.0 %, upper 100.0 %) | 6/6 (100.0 %, upper 100.0 %) |
| hiredis document | 0 | 0 | 0 | 0 | 0 | 0 | 0/0 (0.0 %, upper 100.0 %) | 0/0 (0.0 %, upper 100.0 %) |
| hlint | 0 | 0 | 0 | 0 | 0 | 0 | 0/0 (0.0 %, upper 100.0 %) | 0/0 (0.0 %, upper 100.0 %) |
| hlint code | 0 | 0 | 0 | 0 | 0 | 0 | 0/0 (0.0 %, upper 100.0 %) | 0/0 (0.0 %, upper 100.0 %) |
| hlint document | 0 | 0 | 0 | 0 | 0 | 0 | 0/0 (0.0 %, upper 100.0 %) | 0/0 (0.0 %, upper 100.0 %) |
| junit4 | 5 | 0 | 0 | 1 | 4 | 0 | 1/5 (20.0 %, upper 71.6 %) | 0/4 (0.0 %, upper 60.2 %) |
| junit4 code | 5 | 0 | 0 | 1 | 4 | 0 | 1/5 (20.0 %, upper 71.6 %) | 0/4 (0.0 %, upper 60.2 %) |
| junit4 document | 0 | 0 | 0 | 0 | 0 | 0 | 0/0 (0.0 %, upper 100.0 %) | 0/0 (0.0 %, upper 100.0 %) |
| purrr | 3 | 0 | 1 | 1 | 1 | 0 | 2/3 (66.7 %, upper 99.2 %) | 1/2 (50.0 %, upper 98.7 %) |
| purrr code | 1 | 0 | 1 | 0 | 0 | 0 | 1/1 (100.0 %, upper 100.0 %) | 1/1 (100.0 %, upper 100.0 %) |
| purrr document | 2 | 0 | 0 | 1 | 1 | 0 | 1/2 (50.0 %, upper 98.7 %) | 0/1 (0.0 %, upper 97.5 %) |
| spdlog | 2 | 0 | 0 | 1 | 1 | 0 | 1/2 (50.0 %, upper 98.7 %) | 0/1 (0.0 %, upper 97.5 %) |
| spdlog code | 2 | 0 | 0 | 1 | 1 | 0 | 1/2 (50.0 %, upper 98.7 %) | 0/1 (0.0 %, upper 97.5 %) |
| spdlog document | 0 | 0 | 0 | 0 | 0 | 0 | 0/0 (0.0 %, upper 100.0 %) | 0/0 (0.0 %, upper 100.0 %) |
| viper | 0 | 0 | 0 | 0 | 0 | 0 | 0/0 (0.0 %, upper 100.0 %) | 0/0 (0.0 %, upper 100.0 %) |
| viper code | 0 | 0 | 0 | 0 | 0 | 0 | 0/0 (0.0 %, upper 100.0 %) | 0/0 (0.0 %, upper 100.0 %) |
| viper document | 0 | 0 | 0 | 0 | 0 | 0 | 0/0 (0.0 %, upper 100.0 %) | 0/0 (0.0 %, upper 100.0 %) |
| zustand | 1 | 0 | 0 | 1 | 0 | 0 | 1/1 (100.0 %, upper 100.0 %) | 0/0 (0.0 %, upper 100.0 %) |
| zustand code | 1 | 0 | 0 | 1 | 0 | 0 | 1/1 (100.0 %, upper 100.0 %) | 0/0 (0.0 %, upper 100.0 %) |
| zustand document | 0 | 0 | 0 | 0 | 0 | 0 | 0/0 (0.0 %, upper 100.0 %) | 0/0 (0.0 %, upper 100.0 %) |
| pooled | 21 | 3 | 7 | 4 | 7 | 0 | 14/21 (66.7 %, upper 85.4 %) | 10/17 (58.8 %, upper 81.6 %) |
| pooled code | 17 | 3 | 5 | 3 | 6 | 0 | 11/17 (64.7 %, upper 85.8 %) | 8/14 (57.1 %, upper 82.3 %) |
| pooled document | 4 | 0 | 2 | 1 | 1 | 0 | 3/4 (75.0 %, upper 99.4 %) | 2/3 (66.7 %, upper 99.2 %) |

leave one out, reading A: without Penlight 11/17 (64.7 %, upper 85.8 %); without fd 14/21 (66.7 %, upper 85.4 %); without flask 14/21 (66.7 %, upper 85.4 %); without hiredis 8/15 (53.3 %, upper 78.7 %); without hlint 14/21 (66.7 %, upper 85.4 %); without junit4 13/16 (81.2 %, upper 96.0 %); without purrr 12/18 (66.7 %, upper 86.7 %); without spdlog 13/19 (68.4 %, upper 87.4 %); without viper 14/21 (66.7 %, upper 85.4 %); without zustand 13/20 (65.0 %, upper 84.6 %)

leave one out code, reading A: without Penlight 10/15 (66.7 %, upper 88.2 %); without fd 11/17 (64.7 %, upper 85.8 %); without flask 11/17 (64.7 %, upper 85.8 %); without hiredis 5/11 (45.5 %, upper 76.6 %); without hlint 11/17 (64.7 %, upper 85.8 %); without junit4 10/12 (83.3 %, upper 97.9 %); without purrr 10/16 (62.5 %, upper 84.8 %); without spdlog 10/15 (66.7 %, upper 88.2 %); without viper 11/17 (64.7 %, upper 85.8 %); without zustand 10/16 (62.5 %, upper 84.8 %)

leave one out document, reading A: without Penlight 1/2 (50.0 %, upper 98.7 %); without fd 3/4 (75.0 %, upper 99.4 %); without flask 3/4 (75.0 %, upper 99.4 %); without hiredis 3/4 (75.0 %, upper 99.4 %); without hlint 3/4 (75.0 %, upper 99.4 %); without junit4 3/4 (75.0 %, upper 99.4 %); without purrr 2/2 (100.0 %, upper 100.0 %); without spdlog 3/4 (75.0 %, upper 99.4 %); without viper 3/4 (75.0 %, upper 99.4 %); without zustand 3/4 (75.0 %, upper 99.4 %)

leave one out, reading B: without Penlight 7/13 (53.8 %, upper 80.8 %); without fd 10/17 (58.8 %, upper 81.6 %); without flask 10/17 (58.8 %, upper 81.6 %); without hiredis 4/11 (36.4 %, upper 69.2 %); without hlint 10/17 (58.8 %, upper 81.6 %); without junit4 10/13 (76.9 %, upper 95.0 %); without purrr 9/15 (60.0 %, upper 83.7 %); without spdlog 10/16 (62.5 %, upper 84.8 %); without viper 10/17 (58.8 %, upper 81.6 %); without zustand 10/17 (58.8 %, upper 81.6 %)

leave one out code, reading B: without Penlight 7/12 (58.3 %, upper 84.8 %); without fd 8/14 (57.1 %, upper 82.3 %); without flask 8/14 (57.1 %, upper 82.3 %); without hiredis 2/8 (25.0 %, upper 65.1 %); without hlint 8/14 (57.1 %, upper 82.3 %); without junit4 8/10 (80.0 %, upper 97.5 %); without purrr 7/13 (53.8 %, upper 80.8 %); without spdlog 8/13 (61.5 %, upper 86.1 %); without viper 8/14 (57.1 %, upper 82.3 %); without zustand 8/14 (57.1 %, upper 82.3 %)

leave one out document, reading B: without Penlight 0/1 (0.0 %, upper 97.5 %); without fd 2/3 (66.7 %, upper 99.2 %); without flask 2/3 (66.7 %, upper 99.2 %); without hiredis 2/3 (66.7 %, upper 99.2 %); without hlint 2/3 (66.7 %, upper 99.2 %); without junit4 2/3 (66.7 %, upper 99.2 %); without purrr 2/2 (100.0 %, upper 100.0 %); without spdlog 2/3 (66.7 %, upper 99.2 %); without viper 2/3 (66.7 %, upper 99.2 %); without zustand 2/3 (66.7 %, upper 99.2 %)

bars, orphaned:
(1) 21 events, at least 100: fails

orphaned has fewer than 100 events: not judged, it does not block.

### born

| corpus / kind | events | re-referred | kept up focused | kept up sweep | removed untouched | undetermined | false A | false B |
|---|---|---|---|---|---|---|---|---|
| Penlight | 20 | 4 | 7 | 4 | 5 | 0 | 15/20 (75.0 %, upper 91.3 %) | 11/16 (68.8 %, upper 89.0 %) |
| Penlight code | 8 | 3 | 1 | 1 | 3 | 0 | 5/8 (62.5 %, upper 91.5 %) | 4/7 (57.1 %, upper 90.1 %) |
| Penlight document | 12 | 1 | 6 | 3 | 2 | 0 | 10/12 (83.3 %, upper 97.9 %) | 7/9 (77.8 %, upper 97.2 %) |
| fd | 2 | 0 | 2 | 0 | 0 | 0 | 2/2 (100.0 %, upper 100.0 %) | 2/2 (100.0 %, upper 100.0 %) |
| fd code | 0 | 0 | 0 | 0 | 0 | 0 | 0/0 (0.0 %, upper 100.0 %) | 0/0 (0.0 %, upper 100.0 %) |
| fd document | 2 | 0 | 2 | 0 | 0 | 0 | 2/2 (100.0 %, upper 100.0 %) | 2/2 (100.0 %, upper 100.0 %) |
| flask | 0 | 0 | 0 | 0 | 0 | 0 | 0/0 (0.0 %, upper 100.0 %) | 0/0 (0.0 %, upper 100.0 %) |
| flask code | 0 | 0 | 0 | 0 | 0 | 0 | 0/0 (0.0 %, upper 100.0 %) | 0/0 (0.0 %, upper 100.0 %) |
| flask document | 0 | 0 | 0 | 0 | 0 | 0 | 0/0 (0.0 %, upper 100.0 %) | 0/0 (0.0 %, upper 100.0 %) |
| hiredis | 8 | 1 | 6 | 1 | 0 | 0 | 8/8 (100.0 %, upper 100.0 %) | 7/7 (100.0 %, upper 100.0 %) |
| hiredis code | 7 | 1 | 5 | 1 | 0 | 0 | 7/7 (100.0 %, upper 100.0 %) | 6/6 (100.0 %, upper 100.0 %) |
| hiredis document | 1 | 0 | 1 | 0 | 0 | 0 | 1/1 (100.0 %, upper 100.0 %) | 1/1 (100.0 %, upper 100.0 %) |
| hlint | 1 | 1 | 0 | 0 | 0 | 0 | 1/1 (100.0 %, upper 100.0 %) | 1/1 (100.0 %, upper 100.0 %) |
| hlint code | 0 | 0 | 0 | 0 | 0 | 0 | 0/0 (0.0 %, upper 100.0 %) | 0/0 (0.0 %, upper 100.0 %) |
| hlint document | 1 | 1 | 0 | 0 | 0 | 0 | 1/1 (100.0 %, upper 100.0 %) | 1/1 (100.0 %, upper 100.0 %) |
| junit4 | 27 | 0 | 15 | 1 | 3 | 8 | 24/27 (88.9 %, upper 97.6 %) | 15/18 (83.3 %, upper 96.4 %) |
| junit4 code | 1 | 0 | 0 | 0 | 1 | 0 | 0/1 (0.0 %, upper 97.5 %) | 0/1 (0.0 %, upper 97.5 %) |
| junit4 document | 26 | 0 | 15 | 1 | 2 | 8 | 24/26 (92.3 %, upper 99.1 %) | 15/17 (88.2 %, upper 98.5 %) |
| purrr | 10 | 1 | 5 | 1 | 3 | 0 | 7/10 (70.0 %, upper 93.3 %) | 6/9 (66.7 %, upper 92.5 %) |
| purrr code | 0 | 0 | 0 | 0 | 0 | 0 | 0/0 (0.0 %, upper 100.0 %) | 0/0 (0.0 %, upper 100.0 %) |
| purrr document | 10 | 1 | 5 | 1 | 3 | 0 | 7/10 (70.0 %, upper 93.3 %) | 6/9 (66.7 %, upper 92.5 %) |
| spdlog | 6 | 0 | 5 | 1 | 0 | 0 | 6/6 (100.0 %, upper 100.0 %) | 5/5 (100.0 %, upper 100.0 %) |
| spdlog code | 6 | 0 | 5 | 1 | 0 | 0 | 6/6 (100.0 %, upper 100.0 %) | 5/5 (100.0 %, upper 100.0 %) |
| spdlog document | 0 | 0 | 0 | 0 | 0 | 0 | 0/0 (0.0 %, upper 100.0 %) | 0/0 (0.0 %, upper 100.0 %) |
| viper | 2 | 0 | 2 | 0 | 0 | 0 | 2/2 (100.0 %, upper 100.0 %) | 2/2 (100.0 %, upper 100.0 %) |
| viper code | 2 | 0 | 2 | 0 | 0 | 0 | 2/2 (100.0 %, upper 100.0 %) | 2/2 (100.0 %, upper 100.0 %) |
| viper document | 0 | 0 | 0 | 0 | 0 | 0 | 0/0 (0.0 %, upper 100.0 %) | 0/0 (0.0 %, upper 100.0 %) |
| zustand | 14 | 2 | 11 | 1 | 0 | 0 | 14/14 (100.0 %, upper 100.0 %) | 13/13 (100.0 %, upper 100.0 %) |
| zustand code | 1 | 0 | 0 | 1 | 0 | 0 | 1/1 (100.0 %, upper 100.0 %) | 0/0 (0.0 %, upper 100.0 %) |
| zustand document | 13 | 2 | 11 | 0 | 0 | 0 | 13/13 (100.0 %, upper 100.0 %) | 13/13 (100.0 %, upper 100.0 %) |
| pooled | 90 | 9 | 53 | 9 | 11 | 8 | 79/90 (87.8 %, upper 93.7 %) | 62/73 (84.9 %, upper 92.2 %) |
| pooled code | 25 | 4 | 13 | 4 | 4 | 0 | 21/25 (84.0 %, upper 95.5 %) | 17/21 (81.0 %, upper 94.6 %) |
| pooled document | 65 | 5 | 40 | 5 | 7 | 8 | 58/65 (89.2 %, upper 95.6 %) | 45/52 (86.5 %, upper 94.4 %) |

leave one out, reading A: without Penlight 64/70 (91.4 %, upper 96.8 %); without fd 77/88 (87.5 %, upper 93.6 %); without flask 79/90 (87.8 %, upper 93.7 %); without hiredis 71/82 (86.6 %, upper 93.1 %); without hlint 78/89 (87.6 %, upper 93.7 %); without junit4 55/63 (87.3 %, upper 94.4 %); without purrr 72/80 (90.0 %, upper 95.6 %); without spdlog 73/84 (86.9 %, upper 93.3 %); without viper 77/88 (87.5 %, upper 93.6 %); without zustand 65/76 (85.5 %, upper 92.5 %)

leave one out code, reading A: without Penlight 16/17 (94.1 %, upper 99.9 %); without fd 21/25 (84.0 %, upper 95.5 %); without flask 21/25 (84.0 %, upper 95.5 %); without hiredis 14/18 (77.8 %, upper 93.6 %); without hlint 21/25 (84.0 %, upper 95.5 %); without junit4 21/24 (87.5 %, upper 97.3 %); without purrr 21/25 (84.0 %, upper 95.5 %); without spdlog 15/19 (78.9 %, upper 93.9 %); without viper 19/23 (82.6 %, upper 95.0 %); without zustand 20/24 (83.3 %, upper 95.3 %)

leave one out document, reading A: without Penlight 48/53 (90.6 %, upper 96.9 %); without fd 56/63 (88.9 %, upper 95.4 %); without flask 58/65 (89.2 %, upper 95.6 %); without hiredis 57/64 (89.1 %, upper 95.5 %); without hlint 57/64 (89.1 %, upper 95.5 %); without junit4 34/39 (87.2 %, upper 95.7 %); without purrr 51/55 (92.7 %, upper 98.0 %); without spdlog 58/65 (89.2 %, upper 95.6 %); without viper 58/65 (89.2 %, upper 95.6 %); without zustand 45/52 (86.5 %, upper 94.4 %)

leave one out, reading B: without Penlight 51/57 (89.5 %, upper 96.0 %); without fd 60/71 (84.5 %, upper 92.0 %); without flask 62/73 (84.9 %, upper 92.2 %); without hiredis 55/66 (83.3 %, upper 91.4 %); without hlint 61/72 (84.7 %, upper 92.1 %); without junit4 47/55 (85.5 %, upper 93.5 %); without purrr 56/64 (87.5 %, upper 94.4 %); without spdlog 57/68 (83.8 %, upper 91.6 %); without viper 60/71 (84.5 %, upper 92.0 %); without zustand 49/60 (81.7 %, upper 90.5 %)

leave one out code, reading B: without Penlight 13/14 (92.9 %, upper 99.8 %); without fd 17/21 (81.0 %, upper 94.6 %); without flask 17/21 (81.0 %, upper 94.6 %); without hiredis 11/15 (73.3 %, upper 92.2 %); without hlint 17/21 (81.0 %, upper 94.6 %); without junit4 17/20 (85.0 %, upper 96.8 %); without purrr 17/21 (81.0 %, upper 94.6 %); without spdlog 12/16 (75.0 %, upper 92.7 %); without viper 15/19 (78.9 %, upper 93.9 %); without zustand 17/21 (81.0 %, upper 94.6 %)

leave one out document, reading B: without Penlight 38/43 (88.4 %, upper 96.1 %); without fd 43/50 (86.0 %, upper 94.2 %); without flask 45/52 (86.5 %, upper 94.4 %); without hiredis 44/51 (86.3 %, upper 94.3 %); without hlint 44/51 (86.3 %, upper 94.3 %); without junit4 30/35 (85.7 %, upper 95.2 %); without purrr 39/43 (90.7 %, upper 97.4 %); without spdlog 45/52 (86.5 %, upper 94.4 %); without viper 45/52 (86.5 %, upper 94.4 %); without zustand 32/39 (82.1 %, upper 92.5 %)

bars, born:
(1) 90 events, at least 100: fails

born has fewer than 100 events: not judged, it does not block.

| corpus | unjudged_pairs | unjudged_after_window | met_unjudged | dependents | symlink_rows | retried |
|---|---|---|---|---|---|---|
| Penlight | 0 | 0 | 0 | 48 | 0 | 0 |
| fd | 0 | 0 | 0 | 0 | 0 | 0 |
| flask | 0 | 0 | 0 | 0 | 0 | 0 |
| hiredis | 0 | 0 | 0 | 0 | 0 | 0 |
| hlint | 0 | 0 | 0 | 0 | 0 | 0 |
| junit4 | 0 | 0 | 0 | 1322 | 0 | 0 |
| purrr | 0 | 0 | 0 | 440 | 0 | 0 |
| spdlog | 0 | 0 | 0 | 2033 | 0 | 0 |
| viper | 0 | 0 | 0 | 7558 | 0 | 0 |
| zustand | 0 | 0 | 0 | 1230 | 0 | 0 |
| pooled | 0 | 0 | 0 | 12631 | 0 | 0 |

A row with an empty denominator prints `0/0 (0.0 %, upper 100.0 %)`: the renderer's writing of an undefined share.

## The audit of the undetermined (reading C)

Section 3 draws, per state, the undetermined events lowest by key (all of them when fewer than 60); they were read
blind under generation 1's brief at the corpus's tip, every fifth row and every `unsure` a second time by a
different auditor who did not see the first answer. The coordinator read every cited `path:line` at the tip and
searched every `unused` row's basename over the tip's tree before the answers were settled.

| state | undetermined | drawn | read twice | in_use | unused | unsure | reading B | reading C, unsure as in_use | reading C, unsure as not |
|---|---|---|---|---|---|---|---|---|---|
| orphaned | 0 | 0 | 0 | 0 | 0 | 0 | 10/17 (58.8 %) | 10/17 (58.8 %) | 10/17 (58.8 %) |
| born | 8 | 8 | 1 | 5 | 3 | 0 | 62/73 (84.9 %) | 536/648 (82.7 %) | 536/648 (82.7 %) |

| n | state | corpus | commit | path | first | second | settled |
|---|---|---|---|---|---|---|---|
| 1 | born | junit4 | `2f879b9eea` | `doc/ReleaseNotes4.9.1.md` | in_use | - | in_use |
| 2 | born | junit4 | `34a0ecb4ff` | `doc/ReleaseNotes4.8.1.html` | unused | - | unused |
| 3 | born | junit4 | `2cd6f108a0` | `src/site/markdown/cookbook.md` | in_use | - | in_use |
| 4 | born | junit4 | `4dd14be2eb` | `epl-v10.html` | unused | - | unused |
| 5 | born | junit4 | `2f879b9eea` | `doc/ReleaseNotes4.8.1.md` | in_use | in_use | in_use |
| 6 | born | junit4 | `bc0ae224fa` | `doc/ReleaseNotes4.8.2.html` | unused | - | unused |
| 7 | born | junit4 | `2f879b9eea` | `doc/ReleaseNotes4.8.2.md` | in_use | - | in_use |
| 8 | born | junit4 | `2f879b9eea` | `doc/ReleaseNotes4.5.md` | in_use | - | in_use |

Reading C is reading B with the undetermined added at the audited `in_use` share; it decides nothing by itself.

## Not run

The secondary table (the thirteen audited corpora, section 1: printed, decides nothing) was not run: the
instrument reads one registry, the primary one, and no line of it changes after the first printed count.

## Outcome

Neither state is judged. Both have fewer events than the 100 section 4 asks before a state may block by default
(condition (1) in the report), so by the bars neither blocks - a class with no false-block record of its own does
not block - and the counts are reported. The shares that were measured are far above the 1.0 % bar under every
reading: most files the product newly called unreferenced were re-referred or kept up afterwards.

Section 5 stands: this record changes nothing in the product. `ce deadcode --check` still fails on any dead file
and the score still counts a dead file's lines; changing that default needs a plan amendment (ADR-006's member
set, the gate's meaning) and the owner's word with these numbers. The owner was asked on 2026-11-13 and answered
that the line of 2026-11-11 stands: 「想办法降低误拦」 (find a way to lower the false blocks). The plan amendment
follows this record.
