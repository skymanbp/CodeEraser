-- | The docdup knobs — every constant the doc-duplication judgment
-- reads, nothing else (the CE.Clone.Cost posture: ablation targets
-- live in ONE module, so a dead knob has nowhere to hide).
module CE.Docdup.Cost
  ( jaccardNum
  , jaccardDen
  , shingleK
  , docSetCap
  , docPairCap
  , dupDecides
  , dupDecidesWith
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
-- is the protocol's record of the alphabet geometry, and the Rust
-- mirror (docdup/spec.rs DOC_SHINGLE, backed by the measured k-window
-- in the frozen docdup-segments method line) is pinned to it at every
-- reply parse (D13/F29). Two sides shingling at different widths
-- would compare incommensurable alphabets and no gate downstream
-- could tell.
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

-- | The Jaccard half of the duplication verdict, stated by the
-- threshold's OWNER: dup ⇔ inter·jaccardDen ≥ jaccardNum·union.
-- The verbatim half of the disjunction is Rust-owned (spec.rs
-- VERBATIM_FLOOR, where the runs are computed); this half is applied
-- here per scored row into counts.jaccardDups, and re-applied by the
-- Rust is_dup mirror that the knobs echo pins.
dupDecides :: Integer -> Integer -> Bool
dupDecides = dupDecidesWith jaccardNum jaccardDen

-- | The threshold-parameterized form — ONE formula, so the reference
-- battery's dead-knob probe (perturb the ratio, the family's decision
-- count must move) exercises the production comparison, never a
-- re-implementation.
dupDecidesWith :: Integer -> Integer -> Integer -> Integer -> Bool
dupDecidesWith num den inter union = inter * den >= num * union
