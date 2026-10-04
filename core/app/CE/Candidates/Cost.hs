-- | The candidates family's constants (plan v2.33 W3; design booklet
-- docs/reference/algorithm-track.md §6): the two table ceilings and the
-- source bits: the ones the measuring side may send, the two this side
-- mints. The bounds the family applies are
-- the clone judgment's own (CE.Clone.Prefilter over CE.Clone.Cost's
-- 85/100): a candidate pass that pruned by another ratio would not be
-- admissible, so the ratio has one owner and this module holds none.
module CE.Candidates.Cost (candidateUnitCap, candidatePairCap, exhaustiveSource, keySource, sentSources) where

-- | Unit-row ceiling. A unit row is one admitted unit (at least
-- CE.Clone.Cost.minUnitNodes named nodes) with its file, its key and
-- its kind histogram; this repository admits about 6,400. Over-cap
-- answers a complete degraded reply, never a truncated one.
candidateUnitCap :: Integer
candidateUnitCap = 262144

-- | Sent-pair ceiling: the measuring side's three sources' union.
candidatePairCap :: Integer
candidatePairCap = 4194304

-- | The bits the measuring side's generators may set on a pair it
-- sends: S1, S3 and S4 (bits 0, 2, 3). S2 runs here.
sentSources :: Integer
sentSources = 13

-- | The same-key source's bit (S2, bit 1), set here.
keySource :: Integer
keySource = 2

-- | The exhaustive source's bit (S5, bit 4): a pair it alone found
-- carries this bit and nothing else.
exhaustiveSource :: Integer
exhaustiveSource = 16
