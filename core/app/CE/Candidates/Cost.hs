-- | The candidates family's constants (plan v2.33 W3; design booklet
-- docs/reference/algorithm-track.md §6): the request ceilings, the LSH
-- shape, and the source bits. The bounds the family applies are the
-- clone judgment's own (CE.Clone.Prefilter over CE.Clone.Cost's
-- 85/100): a candidate pass that pruned by another ratio would not be
-- admissible, so the ratio has one owner and this module holds none.
-- The hot-group cap is CE.Dedup.Cost's (the T1/T2 extension pairs hash
-- groups by the same one).
module CE.Candidates.Cost
  ( candidateUnitCap
  , candidateSigCap
  , candidatePrintCap
  , candidateNearCap
  , lshShape
  , nearSource
  , keySource
  , printSource
  , bandSource
  , exhaustiveSource
  ) where

-- | Unit-row ceiling. A unit row is one admitted unit (at least
-- CE.Clone.Cost.minUnitNodes named nodes) with its file, key, span and
-- kind histogram; this repository admits about 10,000. Over-cap answers
-- a complete degraded reply, never a truncated one.
candidateUnitCap :: Integer
candidateUnitCap = 262144

-- | Ceiling on the structural shingles of all units together (S4's
-- input; this repository sends about 600,000).
candidateSigCap :: Integer
candidateSigCap = 16777216

-- | Ceiling on the fingerprint instances (S3's input; this repository
-- sends about 66,000).
candidatePrintCap :: Integer
candidatePrintCap = 4194304

-- | Ceiling on the near runs (S1's input).
candidateNearCap :: Integer
candidateNearCap = 1048576

-- | The MinHash/LSH shape as one fact — (permutations, bands, rows):
-- 128 = 32 × 4, the docdup coarse filter's split (§5.3); one shape for
-- both estimators.
lshShape :: (Integer, Integer, Integer)
lshShape = (128, 32, 4)

-- | The source bits a kept pair carries: S1 near runs (bit 0), S2 same
-- key (bit 1), S3 fingerprints (bit 2), S4 bands (bit 3), S5 the
-- exhaustive window (bit 4; a pair it alone found carries nothing else).
nearSource, keySource, printSource, bandSource, exhaustiveSource :: Integer
nearSource = 1
keySource = 2
printSource = 4
bandSource = 8
exhaustiveSource = 16
