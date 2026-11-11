# AUDITOR-BRIEF — the dead-file exam, generation 1 (lane v235_licence, item L3)

You are an independent auditor. You did not write the rule under test and you will not see any verdict the product
gives beyond the one fact that defines the row: `ce deadcode` reports the file as file-tier dead (no reference site of
the corpus lands on it, and nothing in the project marks it an entry). Your answer decides whether that report deserves
to call the file "dead" — nothing else.

## What a row is (quoted verbatim from the instrument)

The universe, `contracts/eval/deadfile-slice-<corpus>-v1.json` — `SLICE_METHOD` in
`cli/tests/it/eval_deadfile_parts/generate.rs`:

> file-tier dead universe of one pinned corpus: every dead row of the product's deadcode family (graph wire without
> the symbol advisory, the core's verdict and confidence) at the pinned tip of a clean checkout, one row per file, with
> the measured features of features.rs - top-level node kinds through the product's parser, kept out-edges off the
> same graph wire, other files of the pinned tree spelling the basename (git grep -F) and the stem as a word (git grep
> -F -w), convention path segments, document class - and the sha256 of the text. Frozen before any audit is read and
> before any rule reads these features.

The sample, `contracts/eval/deadfile-sample-<lang>-v1.json` — `SAMPLE_METHOD` in the same file:

> per language over the frozen slices: rank id = sha256(domain|corpus|commit|path); at most `total` primary rows by
> the language exams' quota rule over corpora (each corpus first min(15, pool), the seats left by largest remainder),
> each corpus taking its lowest rank-domain hashes; the reserve is the audit-domain rank over the unpicked rest, at
> most `reserve` rows; a language with fewer than `whole_below` rows is taken whole with no reserve. Rows stored in
> audit-domain order.

Constants frozen in every doc: `total` 100, `reserve` 40, `whole_below` 30, domains `ce-deadfile-rank-v1` /
`ce-deadfile-audit-v1`.

The features (header of `cli/tests/it/eval_deadfile_parts/features.rs`, verbatim):

> (a) the distinct named top-level node kinds of the file — the tree-sitter root's named children through the
> product's parser (`scan::ast::parse_lang`); none for a language without a grammar;
> (b) kept out-edges — the graph wire's edges from the file's node whose kind is not inert (asset 3, unused reference
> definition 5: the two kinds the core excludes from liveness);
> (c) how many OTHER files of the pinned tree spell the file's basename (`git grep -F -l`), and its stem as a word
> (`git grep -F -w -l`) — tracked files at the tip, the file itself not counted (the mention universe's base set,
> without its Terms);
> (d) the convention segments of its path (mod.rs CONVENTION_SEGMENTS, `resources` only below a test segment);
> (e) its document class: markdown / html / text, else null.

You audit the PRIMARY rows (`rows`) of each sample in the order stored. A primary row you cannot answer (the file
cannot be found at the pinned tip) is replaced by the next `reserve` row in stored order, and you say so.

## What you are given per row

`corpus`, `commit` (the pinned tip), `path`. Open the corpus at that commit (the clones the main session names) and
read the file in full. Do NOT read the row's other fields (verdict, confidence, features) before you answer — they are
the instrument's measurements, not evidence, and the point of the audit is to check whether they predict your answer.

## The question (one per row)

"Would deleting this file change anything a maintainer or user of this project does — a build, a test run, a docs
build, packaging, a published page, a documented command?"

Answer exactly one of:

- `in_use` — name the mechanism AND the evidence: the `file:line` of the config, script, CI workflow, build file,
  package manifest, documentation sentence or convention's documentation that makes the file used. Examples of
  mechanisms: run by name (Makefile / CI / package script / README command), found by a tool's convention
  (`setup.py`, `docs/conf.py`, `package-info.java`, `module-info.java`, a test runner's data or fixture directory,
  `CONTRIBUTING.md` / `SECURITY.md` / issue templates the forge reads), served or opened by its path (a page, a data
  file a test opens), packaged (a manifest's file list, a licence a package must carry).
- `unused` — say where you searched (which configs, which grep, which docs) and that nothing names or loads it.
- `unsure` — say what you could not settle and why.

One line of evidence per answer is enough when it is decisive; never answer `in_use` without a `file:line` or a
named convention with its documentation, and never answer `unused` without saying where you looked.

## Output format

One JSON object per row, in the sample's stored order:

```json
{"corpus": "...", "path": "...", "answer": "in_use|unused|unsure", "mechanism": "...", "evidence": "path:line ...", "searched": "..."}
```

The main session collects the answers into the review tables; you do not see other auditors' answers.
