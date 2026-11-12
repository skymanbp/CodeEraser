# Lowering the false blocks of the dead-file verdict

Owner, 2026-11-11, answering "how should an unreferenced file be treated by default" with a line of their own:
「想办法降低误拦」. Not "make it advisory", not "keep blocking": make the verdict right more often.

Status of this note: a DESIGN, written by the main session from a development reading of the 424 audited rows.
BARS.md section 5 stands: no classifier is judged on those rows again. Everything below that claims a rate is a
hypothesis until generation 2 (fresh events, bars written first) has measured it.

## 1. What blocks today

- `ce deadcode --check` exits 1 when ANY file is outside the reach of the entry set (core `CE.Graph.Lines.failed`:
  `not (null dead) || degraded`). No memory, no tolerance: a first run on a real project fails.
- The score's axis 4 charges `deadFiles / nodes`, counting dead files with in-degree 0 (`deadIndegCeil = 0`).
- `ce erase` no longer deletes on a dead verdict (lane phase 1, reason 7 `reachable_by_name`; ships with the next
  regular release by the owner's ruling).
- The ratchet's discrete set holds clone members only; a dead file is not a baseline member.

## 2. What the audited rows say (development reading)

`explore_roots.py`, `explore_root_families.py`, `explore_signals.py` in this directory; 424 rows, 13 corpora.

| the product's word | rows | in use | unused | unsure |
|---|---|---|---|---|
| dependent: referred to, but only from files that are dead themselves (`unreach_*`) | 114 | 114 | 0 | 0 |
| root: nothing refers to it (`unref_*`) | 310 | 278 | 28 | 4 |

- Every one of the 28 unused files is a root. A dependent is accused only because its root is; 114 of the 392 wrong
  accusations (29 %) are this echo.
- No fact the product holds today separates the 28 from the 278: basename spelled nowhere 13 unused of 82 roots
  (16 %); spelled somewhere 7 of 144 (5 %); `confidence` 2 gives 4 of 19.
- How the 278 in-use roots are used, by the auditors' own words (keyword sort, rough): a build or package file names
  or globs it 99 (zod 60); a resource read by path at run time 61; a framework loads it by position 58; a script a
  person or a document runs 35; other 19; a test runner finds it 4; linked or served 2.

Reading: "nothing refers to it" is absence of evidence. The ways a file is used are open-ended (tools, people,
frameworks), so a closed list of reference kinds is wrong about nine times in ten on the STOCK of a mature tree,
where real residue has long been cleaned and what remains unreferenced is mostly used some other way. Adding entry
rules narrows the gap and never closes it.

## 3. The design: three parts

### A. The root carries the verdict (structure, exact)

A dead component = one root (in-degree 0 inside the graph) and what only it reaches. The verdict, the gate line, the
count and the remedy attach to the root; dependents are listed under it ("falls with …"). One declaration of an entry
clears its whole component. No rate is claimed for this part: it is a regrouping of the same facts, and it removes
the echo by construction.

### B. Block on evidence of change, not on absence of reference (the root fix)

Three states of a dead root, told apart by a comparison point:

- **orphaned**: the file existed at the comparison point and was reachable there; now it is not. A reference was
  taken away and the file stayed. This is the residue the product exists to catch.
- **born unreferenced**: the file did not exist at the comparison point and nothing refers to it.
- **standing**: it was already dead at the comparison point.

Comparison points, each a memory the product already keeps:

- the committed baseline (`ce-baseline.json`): the dead set becomes a member set like the clone members, so
  `ce check` and `ce deadcode --check` fail on what is NEW since the baseline and list the standing rows;
- the session (Stop audit / pre-commit): "this change removed the last reference to X and X is still on disk".

Which states block is decided per state by its own measured record (section 4), under the standing rule that a class
with no false-block record of its own does not block. A standing row never blocks; it is an inventory line whose
remedy is one of: declare it an entry, or delete it.

This changes ADR-006's member set and the gate's meaning: a plan amendment and the owner's word with numbers come
before any product code.

### C. Read the largest mechanisms as facts (narrows the stock)

Each as an edge or an entry the graph reads from the tree's own files, each rule taken from the tool's documentation
(BARS section 3 discipline), none from a guess:

1. manifest-declared files: `package.json` (`exports`, `bin`, `main`, `files`, `scripts` naming a file),
   `tsconfig` `include`/`files`, a rockspec's `build.modules` table, `pyproject`/`setup.cfg` entry points;
2. resources read by path: the directories a build tool defines as resource roots (Maven `src/*/resources`),
   and a path literal in code that resolves to a walked file;
3. the eight tool conventions already passed (RESULT-2B section 3) go into the entry table.

Each family is a separate rule set with its own fresh-material exam; the size of each family above is a development
reading and only ranks the order of work.

## 4. Generation 2: the record that decides part B (bars before answers)

Instrument: a history replay (the shape of the standing false-block ledgers: `it/common/history.rs`,
`fpr_lang_replay.rs`). Per corpus, the latest 400 first-parent commits; for each commit c with parent p the product's
own graph gives the dead roots of both trees.

- event (a) orphaned at c: root dead at c, file present at p, not dead at p;
- event (b) born at c: root dead at c, file absent at p.

Truth by the file's fate after c, read from history (written before any count is looked at):

- **re-referred**: it later leaves the dead set while it still exists -> the block was false;
- **kept up**: it is still dead, and a later commit changes its content -> someone maintains it -> false;
- **removed untouched**: a later commit deletes it, no content change in between -> residue, the block was right;
- **standing untouched** at the tip: undetermined -> a blind audit sample (the generation-1 auditor brief), and both
  readings are recorded (counted as false / left out).

Reported per state and per corpus: events, each fate, the false share under both readings with a Clopper-Pearson
95 % upper bound, leave-one-corpus-out. Corpora: the primary evidence is repositories that were NOT among the 13
audited ones; the 13 are a secondary table.

Bars (to be frozen in `BARS-GEN2.md` before the first replay prints a count):

- a state blocks by default iff its false share is at most 1 % under the standing ledger reading (the bar every
  blocking class of this product answers to), on the primary corpora, with at least 100 events;
- a state that misses the bar is listed, never fails the gate by default, and carries its measured share in the
  line; a project may switch it to blocking for itself;
- the same outcome under both readings ships without a question; a split between readings goes to the owner with
  the two numbers.

## 5. Order of work

1. main CI repaired (lane cifix) -> the licence lane rebases and lands phase 1 and the eight conventions (part C.3).
2. Generation 2, measurement only (tests side, no product change): `BARS-GEN2.md` frozen by the main session, the
   replay instrument, the fate table, the two readings.
3. The owner sees the numbers in plain words; plan amendment (ADR-006 member set, gate meaning); then parts A and B
   in the product, three faces alike; part C family by family.

Open point that crosses lanes: the score trial (lane score) taints a file for being "dead" and axis 4 charges it,
on a verdict this note finds wrong nine times in ten on the stock. The trial's numbers are read with that in mind;
a variant without the dead taint is asked of that lane when its first result is in.
