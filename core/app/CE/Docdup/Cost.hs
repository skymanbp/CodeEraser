-- | The docdup knobs — every constant the doc-duplication judgment
-- reads, nothing else (the CE.Clone.Cost posture: ablation targets
-- live in ONE module, so a dead knob has nowhere to hide).
module CE.Docdup.Cost
  ( jaccardNum
  , jaccardDen
  , shingleK
  , docLineCap
  , licHeadLines
  , minDocTokens
  , verbatimFloor
  , docSetCap
  , docPairCap
  , dupDecides
  , dupDecidesWith
  , dupVerdict
  , dupVerdictWith
  ) where

-- | The Jaccard report threshold as an integer ratio (0.80). Defined
-- in-repo and documented here (the 2026-08-13 decision ② posture,
-- same as tsedNum): plan §4.1 gives the number without a checkable
-- source, so this repo owns the definition. dup ⇔ inter·jaccardDen ≥
-- jaccardNum·union — integer cross-multiplication, no floats in core.
-- The pre-registered instrument copy (eval_support/docseg.rs
-- JACCARD_REPORT_FLOOR, frozen with the sample census BEFORE this
-- module existed) and the Rust verdict mirror are both held to these
-- two integers by the knobs echo.
jaccardNum :: Integer
jaccardNum = 80

jaccardDen :: Integer
jaccardDen = 100

-- | The estimator's shingle width, ECHOED not computed: sets arrive
-- pre-shingled, so this module never windows anything — the constant
-- is the protocol's record of the alphabet geometry. Since plan
-- v2.32 step 2 the measuring side shingles at the width it reads from
-- the definition package (`docdup.doc_shingle`, this number), and the
-- reply echo still pins the two equal at every parse (D13/F29): two
-- sides shingling at different widths would compare incommensurable
-- alphabets and no gate downstream could tell. Decided, not derived
-- (instruments §9.6): the measured k-window and its counterfactual
-- live in the frozen docdup-segments doc's method line.
shingleK :: Integer
shingleK = 5

-- | Per-set element ceiling. Sizing anchor (Clone.Cost posture):
-- Data.Set intersection/union on two n-element sets is O(n·log n)
-- strict tree steps; at 8192 elements a pair costs ≈ 2·8192·13 ≈
-- 2×10⁵ steps — bounded work per pair, decided before any corpus
-- measurement. 8192 distinct 5-word shingles ≈ an 8000-word document
-- segment; the oracle's F31 segcap sits at the same scale.
docSetCap :: Integer
docSetCap = 8192

-- | Per-request pair ceiling, the clone pairCap anchor carried over:
-- over-cap answers a complete degraded reply, never a truncated one.
-- Worst-case request ≈ 4096 pairs × 2×10⁵ steps ≈ 8×10⁸ bounded
-- strict steps; real doc segments run tens-to-hundreds of shingles.
docPairCap :: Integer
docPairCap = 4096

-- | Admission floor for a documentation segment: below this many
-- words the segment never enters the corpus (batch-7 slice 10 —
-- the other half of the provenance verbatimFloor carries: plan :68,
-- Lee et al. 2107.06499, verbatim lower bound 50 tokens). The
-- EXECUTION stays in Rust before persistence (sub-floor segments
-- have no row and never cross the wire — shipping them was priced
-- and declined); this constant is the AUTHORITY — the measuring
-- side reads it from the definition package (`docdup.min_doc_tokens`)
-- and the echo pins the two equal, so the recall floor is
-- core-visible, ablatable and drift-detectable.
minDocTokens :: Integer
minDocTokens = 50

-- | Per-line word cap inside comment/docstring segments (batch-7
-- defect sweep): an overlong line is masked as data/generated
-- before shingling (md_para is exempt — prose legitimately writes
-- one long line per paragraph). Execution stays in Rust at
-- segmentation; this constant is the authority (the package's
-- `docdup.doc_line_cap`, echo-pinned). Decided, not derived
-- (2026-08-14 attainment-line-B amendment, ccm #842): hard-wrapped
-- comment prose runs under ~120 chars by convention, while the
-- audited false-positive lines — regex literals and inline snapshots
-- — ran 300+/600+. The skeleton-prefix table (CE.Lang.Common.Prose
-- `skeleton_prefixes`) stays unpinned by decision: the echo grammar
-- is numeric, and the table's drift guard is DOCDUP_REV (a semantic
-- change there re-freezes the evaluation set by protocol).
docLineCap :: Integer
docLineCap = 200

-- | Verbatim hard-hit floor in words (plan :68, Lee et al.
-- 2107.06499, verbatim lower bound 50 tokens): a common word run this
-- long is a duplicate regardless of what Jaccard says about the rest
-- of the segment. ADR-008 P1 moved the floor's VERDICT home here: the
-- run lengths already ride each request row ([i,j,run] — F26), the
-- texts never cross the wire (§5.9.2), and the knobs echo pins the
-- measuring side's copy (`docdup.verbatim_floor`) to this number.
verbatimFloor :: Integer
verbatimFloor = 50

-- | License-header window in lines (batch-7 slice 9): only a
-- file's FIRST comment block starting at or above this line can
-- be a license header, and only then do the license markers
-- excuse it from the corpus. Execution stays in Rust at
-- segmentation (exemption is decided before persistence — an
-- exempt segment has no row and never crosses the wire, the
-- minDocTokens stance); this constant is the AUTHORITY the echo
-- pins the package's `docdup.license_head_lines` to. The marker
-- STRING tables (CE.Lang.Common.Prose `license_markers` and
-- `allow_marker`) stay unpinned by the same decision as
-- `skeleton_prefixes`: the echo grammar is
-- numeric and their drift guard is DOCDUP_REV. The bare-marker
-- rule — an allow WITHOUT a ` -- <why>` tail exempts NOTHING
-- and is ledgered as a violation (plan :79-80) — is part of
-- the same ruling: it has no number to echo, so its authority is
-- this written record plus the allow_missing_why count the
-- report surfaces (2.22.0).
licHeadLines :: Integer
licHeadLines = 5

-- | The Jaccard half of the duplication verdict, stated by the
-- threshold's OWNER: dup ⇔ inter·jaccardDen ≥ jaccardNum·union.
-- Applied per scored row into counts.jaccardDups, and mirrored by
-- the Rust is_dup binding that the knobs echo pins.
dupDecides :: Integer -> Integer -> Bool
dupDecides = dupDecidesWith jaccardNum jaccardDen

-- | The threshold-parameterized form — ONE formula, so the reference
-- battery's dead-knob probe (perturb the ratio, the family's decision
-- count must move) exercises the production comparison, never a
-- re-implementation.
dupDecidesWith :: Integer -> Integer -> Integer -> Integer -> Bool
dupDecidesWith num den inter union = inter * den >= num * union

-- | The FULL duplication verdict (ADR-008 P1): Jaccard half ∨
-- verbatim half, both owned here now that the wire carries every
-- verdict input — the reported set is the core's decision, riding
-- each reply as a per-row bit.
dupVerdict :: Integer -> Integer -> Integer -> Bool
dupVerdict = dupVerdictWith (jaccardNum, jaccardDen, verbatimFloor)

-- | The knob-parameterized disjunction — ONE formula, so the
-- battery's probes perturb the production comparison on BOTH halves.
dupVerdictWith :: (Integer, Integer, Integer) -> Integer -> Integer -> Integer -> Bool
dupVerdictWith (num, den, vfloor) inter union run =
  dupDecidesWith num den inter union || run >= vfloor
