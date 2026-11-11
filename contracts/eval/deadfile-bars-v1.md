# Pre-registered bars — the dead-file exam, generation 1 (lane v235_licence, phase 2)

Written by the main session on 2026-11-11 before any auditor was dispatched and before any answer existed.
Nothing below may be changed after the first answer file is written; a rule that turns out wrong is replaced
only through a new generation of the exam on fresh material.

Frozen inputs: the 9 sample files `contracts/eval/deadfile-sample-<lang>-v1.json` of lane commit c51f75d9
(424 primary rows; Lua 100 of a pool of 235, every other language whole), the auditor brief
`v235_licence_scratch/AUDITOR-BRIEF.md`, and the lane's first classifier `universe_counts.py`
(sha256 549337f8b658574a1181ee79836b00a1293dbc3f4a5279ca5a088822dd5dd0f4), called K0 below.

## 1. Reading the answers

- Each primary row gets one answer from an independent auditor who sees only (corpus, commit, path): `in_use`,
  `unused` or `unsure`.
- Second reading, by a different auditor who does not see the first answer: every row whose
  `int(sha256("ce-deadfile-doubleread-v1|" + corpus + "|" + commit + "|" + path)[:8], 16) % 5 == 0`, every row
  answered `unsure`, and every row answered `unused` that carries a convention named in section 3.
  Agreement = share of twice-read rows where both readings fall on the same side of (in_use / not in_use).
  Agreement below 0.90 → every row is read twice.
- A row with two different answers is settled by the evidence, not by a vote: `in_use` stands only when its cited
  `file:line` exists at the pinned tip and does what the answer says; otherwise the row is `unsure`.
- A mechanical check runs over every answer before any table is built: an `in_use` answer must cite a path that
  exists at the pinned tip (and a line inside the file) or a named tool convention; an `unused` answer must say
  where it searched. An answer that fails goes back to a second auditor; it is never repaired by hand.
- Shares: `in(C)` = weighted share of rows of class C answered `in_use`; `un(C)` = weighted share answered
  `unused`; `unsure` belongs to neither. Weight of a row = pool(language, corpus) / primary rows of that
  (language, corpus); this is 1 everywhere except Lua.
- The audited rows are every dead-file row of twelve projects (Lua: a sample). The numbers are exact for these
  projects; no interval is claimed for other projects. Robustness is asked through the per-project floors below,
  and every share is also printed with each project left out in turn.

## 2. The classifier that is judged: K1 (K0 is printed beside it, it decides nothing)

K0 reads the distinct top-level node kinds only. Reviewed before the audit, it has two logic faults: every R file
is "runs code" (R binds a function with `<-`, an operator node), and every Python module with a docstring or a
constant is "runs code" (both are expression statements). So the judged classifier is K1:

A top-level statement is INERT when it is one of
1. a comment;
2. an import / include / package / module header of the language;
3. a declaration of a function, method, class, type, interface, enum, record, trait, impl, macro or namespace
   (a decorated or annotated declaration included);
4. a bare string literal statement (a docstring, a `"use strict"` / `"use client"` directive);
5. a binding (assignment, variable / lexical declaration, R `<-` `=` `<<-`, Lua local or global assignment)
   whose every right-hand side is CONSTANT-LIKE: a number / string / boolean / null literal, a negative number,
   a function literal (lambda, arrow function, `function` expression, R `function`), a class expression, a name
   or dotted name, or a list / tuple / set / dict / table / array / object literal whose every element, key and
   value is constant-like;
6. an export statement around any of 1-5, an `export { … }` / `export * from …` list, and a top-level Lua
   `return` of a constant-like value.

Every other top-level statement is ACTIVE (a call, a binding whose right-hand side holds a call / `new` / `await` /
an operator over non-literals / a template with a substitution, any control statement, an `if __name__ == …`
block, an R `source(...)` / `library(...)` call).

Class of a row: `document` when its `doc_class` is not null; else `declares` when every top-level statement is
inert (an empty file is `declares`); else `runs-alone`.

K1 is implemented in the exam instrument from the parser the product uses, and the class of every sample row
(with the first active statement's kind and line as witness) is frozen and committed BEFORE the lane opens any
answer file. The main session holds the answer files until that commit exists.

## 3. Bar (ii) — a convention becomes an entry name

Decided first; rows of a convention that passes leave the population of section 4.

A file name or path pattern enters a language's entry table only when all three hold:
1. It is a tool's convention, not a project's: the name or position is fixed by a compiler, build tool, package
   manager, test runner, documentation generator, framework or forge, and that tool's own documentation says it
   reads the file by that name. The table row carries the citation, and encodes the position rule the tool uses
   (a name the tool reads only at one place is not entered as a bare basename).
2. Every audited row that carries it is `in_use`, none `unused`, none `unsure`, and at least one answer names
   that tool as the mechanism without having been told the hypothesis.
3. At least one audited row carries it (the tool's documentation is the evidence for the rule; the audit can only
   refute it).

A project's own convention (a plugin loader's file name, a project's test directory, a package script's
directory) never enters a language table. Its road is the project's own `[graph] entry_globs`; its rows stay in
section 4.

Candidates named before the audit (DESIGN.md (ii)): `package-info.java`, `module-info.java`, `vitest.config.ts`,
Next.js `page.tsx` / `layout.tsx` / `route.ts`, `setup.py`, Sphinx `conf.py`, `tlconfig.lua`, busted `spec/`,
Maven `src/test/resources`, `CONTRIBUTING.md`, `SECURITY.md`, `CODE_OF_CONDUCT.md`. Project conventions that stay
out by rule 1 whatever the audit says: koreader `plugins/*/_meta.lua`, lua `testes/`, zod `packages/bench/`.

## 4. Bar (i) — which unreferenced files may fail `ce deadcode --check`

Population: audited rows left after section 3, by K1 class. A class with fewer than 30 rows left is not judged
and keeps today's behaviour.

For each class C:
- ADVISORY when `in(C) >= 0.80`, and every project with at least 10 rows of C left has `in >= 0.60` there, and
  at least two projects have 10 rows of C left. The rows of an advisory class stop failing `--check`; they stay
  in the report under their own word.
- otherwise FAILING (today's behaviour).

What ships:
- `runs-alone` ADVISORY and `in(declares) <= in(runs-alone) - 0.25` → the split ships: `runs-alone` rows are
  reported and no longer fail, `declares` rows keep failing.
- `runs-alone` ADVISORY and `declares` ADVISORY → no split: one advisory word for every unreferenced code file.
- `runs-alone` ADVISORY and `declares` neither advisory nor 0.25 below → nothing ships from this audit; the
  numbers go to the owner as a question about the gate's default.
- `runs-alone` FAILING → nothing ships.
- `document` is judged by the same ADVISORY rule on its own.
- Any class left FAILING with `in(C) >= 0.50` (the failing verdict is wrong more often than right) is reported
  to the owner as a question; this audit does not decide it.

0.80, 0.60, 0.25 and 0.90 are judgment constants fixed here, not derived: at 0.80 at most one in five files
that stop failing was really unused, and that file is still printed in the report.

## 5. What this audit cannot do

- It licenses no deletion, whatever it finds. `ce erase` deletes no whole file (L1); a population that could be
  deleted would need its own exam and its own false-refusal record.
- It cannot rescue a classifier: if K1 does not pass, a third classifier is not tried on these rows.
- The overall share `un(all)` per language is printed as the precision of today's "dead" word; it is a reading,
  not a bar.
