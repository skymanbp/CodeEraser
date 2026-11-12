# Pre-registered bars - the dead-file gate, generation 2 (which states of an unreferenced file may block)

Written by the main session on 2026-11-11 before the replay instrument exists and before any event was counted.
Nothing below may be changed after the instrument's first run prints a count; a rule that turns out wrong is
replaced only through a new generation on fresh corpora. The design this measures is `DESIGN-GEN2.md` (parts A and
B); generation 1's `BARS.md` section 5 stands (no classifier is judged on the 424 audited rows again).

## 1. Corpora

Primary (decide everything): ten repositories, one per judged code language, none of them among the audited
thirteen and none ever run through the product in this project. First choice, then the replacement:

| language | first | replacement |
|---|---|---|
| Python | pallets/flask | psf/black |
| TypeScript | pmndrs/zustand | sindresorhus/ky |
| Rust | sharkdp/fd | clap-rs/clap |
| Go | spf13/viper | gin-gonic/gin |
| Java | junit-team/junit4 | apache/commons-cli |
| C | jqlang/jq | redis/hiredis |
| C++ | gabime/spdlog | nlohmann/json |
| Lua | lunarmodules/Penlight | leafo/lapis |
| R | tidyverse/purrr | r-lib/cli |
| Haskell | ndmitchell/hlint | haskell/aeson |

- Tip = the default branch's head at the first clone; the ten shas are committed in the instrument's registry
  before any replay runs.
- A first choice is replaced only for a reason that can be read without counting an event: fewer than 500
  first-parent commits, or the product refuses to judge the tip's tree (`ce deadcode` exits 2). The replacement and
  the reason are recorded. A language whose two choices both fail is left out and named.
- Secondary (printed, decides nothing): the thirteen audited corpora under the same instrument.

## 2. The instrument (frozen definitions)

Product: one `ce` / `ce-core` pair built from the licence lane's tip after part 3A has landed (the eight tool
conventions are entries); its commit is recorded. A later change of the graph ladder or of the entry table is a new
generation.

Window: per corpus the 600 first-parent commits that end 200 commits before the tip (W = tip~799 .. tip~200); a
corpus with fewer than 800 first-parent commits uses every commit up to tip~200. Fates are read up to the tip, so
every event has at least 200 commits of future.

For each commit c of W with first parent p, the product's own verdict on both trees gives the dead set and, in it,
the ROOTS (dead, no reference from any file of the tree). A pair where the product refuses either tree yields no
event and is counted as unjudged.

Paths across p -> c follow git's rename detection at its default similarity (`git diff -M --name-status p c`): the
destination of a rename is the same file as its source. Without this a renamed dead file would read as born.

- event **orphaned** at c: a root at c; the file exists at p; it is not dead at p.
- event **born** at c: a root at c; the file does not exist at p.
- not an event: a root at c that was dead at p (standing: root or dependent there); a dead file at c that is not
  a root (a dependent; counted in a side table, part A attaches it to its root).

Kind of an event: `document` when the product classes the file as a documentation file, else `code`.

Fate of an event, read forward along the first-parent chain from c to the tip, renames followed; the FIRST of
these that happens decides:

1. **re-referred**: at some later commit the file exists and is not dead -> the block was false.
2. **kept up**: while still dead, a later commit changes the file's content (a rename alone is no change).
   `focused` when that commit changes at most 20 files, `sweep` otherwise -> see the readings.
3. **removed untouched**: a later commit deletes it, with no content change since c -> the block was right.
4. none by the tip -> **undetermined**.

## 3. The two readings

- Reading A (upper): false = re-referred + kept up (focused and sweep) + undetermined; denominator = all events.
- Reading B (lower): false = re-referred + kept up focused; denominator = those + removed untouched; kept up by a
  sweep alone and undetermined are left out.

Reported per state (orphaned, born), per kind, per corpus and pooled over the primary corpora: events, each fate,
the false share under A and under B with its Clopper-Pearson 95 % upper bound, and the pooled share with each
corpus left out in turn. The bound and the leave-one-out table are printed; the bars read the shares.

Audit of the undetermined (a third number for the owner, it decides nothing by itself): per state, the 60
undetermined events lowest by `sha256("ce-deadfile-gen2-v1|" + corpus + "|" + commit + "|" + path)` (all of them
when fewer), read blind by independent auditors under generation 1's brief (they see corpus, commit, path),
every fifth row and every `unsure` read twice. Reading C = reading B with the undetermined added at the audited
`in_use` share.

## 4. The bars

A state BLOCKS BY DEFAULT under a reading iff, on the primary corpora:

1. it has at least 100 events;
2. its pooled false share is at most 1.0 % (the bar every blocking class of this product answers to);
3. no single corpus with at least 30 events of the state has a false share above 5.0 %;
4. conditions 2 and 3 also hold for each kind (`code`, `document`) that has at least 100 events of its own.

What follows:

- BLOCKS under A and under B -> the state fails the gate by default.
- misses under A and under B -> the state is listed, never fails the gate by default, and its line carries the
  measured share; a project may switch it to blocking for itself.
- A and B disagree -> the owner is asked, in plain words, with the two shares and reading C.
- fewer than 100 events -> not judged: it does not block (a class with no false-block record of its own does not
  block), and the count is reported.
- **standing** rows never block, whatever the numbers: they are the inventory, their remedy is to declare an entry
  or to delete.

1.0 % is the product's standing bar. 100, 30, 5.0 %, 20 files, 600 / 200 commits and 60 audit rows are judgment
constants fixed here, not derived.

## 5. What this generation cannot do

- It licenses no deletion (generation 1, section 5).
- It does not judge part C (manifest files, resource roots): each family there is its own rule set with its own
  fresh-material exam.
- It cannot be rerun with another truth rule on these ten corpora: a second attempt needs ten other repositories.
- Whatever it finds, the product changes only after the plan amendment (ADR-006 member set, the gate's meaning)
  and the owner's word with the numbers.

## 6. Order

1. This file is frozen; its sha256 is written into `LANES.md`.
2. The lane writes the instrument (tests side, no product change) and commits the registry (ten tips, product
   commit) and the definitions of section 2 as code BEFORE its first full run.
3. The main session re-derives the fate of 20 events by hand from `git log` before it accepts any table.
