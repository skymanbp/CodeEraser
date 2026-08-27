# Methodology

CodeEraser applies deterministic computation to the non-deterministic
output of language models. A model writing into a long-lived repository
drifts toward stacking rather than editing — the same function
implemented twice, the same fact restated in a third file, an update
that arrives as an append, a file that only ever grows — and the
temptation is to audit that drift with a second model. This document
records the opposite commitment: every verdict CodeEraser issues is a
computation over facts extracted from the tree — token fingerprints,
tree edit distances, graph in-degrees, git-window counts, integer and
rational arithmetic against thresholds that are written down — and it is
reproducible byte for byte on the same tree, on any machine, at any
hour, with no sampling temperature anywhere between the evidence and the
verdict. Nothing here asks a model what it thinks of your code. A
disagreement about code health is settled by re-running the number and
reading the `file:line` it points at, not by a second opinion; that is
why the judgment layer is a pure function of measured facts and why the
size hard line is a declared line — 750 by default, or the `file_lines_fail` a `[[rules.class]]` declares for the paths it owns — while the soft line is a
statistic of the repository's own frozen distribution
([DEVELOPMENT_PLAN.md:60](../DEVELOPMENT_PLAN.md#L60),
[size-advisory.md:26-30](size-advisory.md#L26)).

## How to read this

Every section below is split the same way, along ADR-002
([DEVELOPMENT_PLAN.md:169-174](../DEVELOPMENT_PLAN.md#L169)): the
**measurement** side is Rust — tree-sitter parsing, symbol and span
extraction, the winnowing index, git history extraction — and emits a
normalized IR; the **judgment** side is Haskell (`ce-core`) — the rule
engine, the four-classification, TSED, graph analysis, scoring and the
ratchet — and turns those facts into verdicts
([DEVELOPMENT_PLAN.md:157-160](../DEVELOPMENT_PLAN.md#L157),
[README.md:172](../../README.md#L172)). The boundary has a one-line
test, from ADR-008: if a rule needs source text or line-level content to
cross the wire, it is measurement and stays in Rust
([DEVELOPMENT_PLAN.md:234](../DEVELOPMENT_PLAN.md#L234)). So read each
section as *facts → predicate → verdict*: the formula and its constants
are the contract, the knobs are echoed back in the report, and each is
cited to the file and line that implements it. No number in this
document is quoted from memory; if a value is not traceable to a source
line, it is not here.

## Contents

One file per judgment family. They are separate files because this
repository's own size gate says so: the assembled booklet ran past the
750-line hard line, and a document that argues for splitting long files
does not get an exemption from the rule it argues for.

| # | Section | What it judges |
|---|---|---|
| 1 | [T1/T2 clone detection — winnowing fingerprint index](methodology/01-t1-t2-clone-detection-winnowing-fingerprint.md) | exact and parameterized duplicate blocks |
| 2 | [T3 near-miss clones — Tree Edit Distance (TSED)](methodology/02-t3-near-miss-clones-tree-edit-distance-tsed.md) | structurally similar implementations |
| 3 | [Documentation duplication — shingling + MinHash/LSH](methodology/03-documentation-duplication-shingling-minhash.md) | repeated paragraphs, comments and docstrings |
| 4 | [Structure judgment — tree-scale entropy, seven axes](methodology/04-structure-judgment-tree-scale-entropy-seven.md) | tree-scale structure, axes S0-S6 |
| 5 | [Scoring and the ADR-006 ratchet](methodology/05-scoring-and-the-adr-006-ratchet.md) | the composite score and the only-tightens baseline |
| 6 | [Graph liveness and dead-code verdicts](methodology/06-graph-liveness-and-dead-code-verdicts.md) | import edges, in-degree, dead symbols |
| 7 | [The three-signal join](methodology/07-the-three-signal-join.md) | similarity x graph position x history |
| 8 | [Split-ROI seam pricing (four legs)](methodology/08-split-roi-seam-pricing-four-legs.md) | whether a long file is worth splitting |
| 9 | [Edit four-classification (update supervision)](methodology/09-edit-four-classification-update-supervision.md) | matched / novel / moved / deleted |
| 10 | [Score trajectory — the trend slope verdict](methodology/10-score-trajectory-the-trend-slope-verdict.md) | the score trajectory's slope |
| 11 | [FPR discipline and the guard tier ladder](methodology/11-fpr-discipline-and-the-guard-tier-ladder.md) | which rule class may deny, and on what record |
| 12 | [Deterministic erase — the safety predicate](methodology/12-deterministic-erase-the-safety-predicate.md) | what is provably safe to delete |
