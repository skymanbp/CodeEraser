-- | The moves family's constants (plan v2.33 W3, fourclass L1): the
-- request ceilings and the one alignment number L1 owns. The within-file
-- move rule itself is CE.FourClass.Cost's siteCostWithin = 0 — any one
-- matching line opens a site inside a pair — so it has no constant here.
module CE.FourClass.Moves.Cost (movesLineCap, movesUnitCap, maxBridge) where

-- | Ceiling on the line rows of all pairs together (both sides). A
-- changeset over it answers a complete degraded reply, never a
-- truncated one.
movesLineCap :: Integer
movesLineCap = 4194304

-- | Ceiling on the unit rows of all pairs together (both sides).
movesUnitCap :: Integer
movesUnitCap = 262144

-- | Longest insignificant bridge a leftover run may span. Unbounded
-- bridging let two significant lines 1000 punctuation lines apart
-- compress into adjacency and fuse remote coincidences (attack review
-- F6). Bound = the maximum observed on the frozen slice — the bridge
-- widths over every leftover run of all 47 commits were {0:7037, 1:663,
-- 2:411, 3:90, 4:19, 5:17, 6:2, 7:1}; the full-corpus rerun kept recall
-- at 547/547.
maxBridge :: Int
maxBridge = 7
