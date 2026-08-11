-- | The cost model — all of it. Two per-line and two per-site
-- integer constants (floats tie-break differently across platforms;
-- the output contract is byte determinism). Every downstream rule is
-- a consequence of these four numbers, so the ablation and the plan
-- §7.4 sensitivity test have exactly one target and a dead knob
-- cannot hide.
module CE.FourClass.Cost
  ( movedCost
  , plainCost
  , siteCostWithin
  , siteCostCross
  , siteOpens
  , destFloor
  ) where

-- | Cost of explaining a line as moved.
movedCost :: Int
movedCost = 1

-- | Cost of leaving a line novel / deleted.
plainCost :: Int
plainCost = 3

-- | Opening a relocation site inside one file pair is free — which
-- derives L1's unfloored within-file rule (any single matching line).
siteCostWithin :: Int
siteCostWithin = 0

-- | Opening a site across two file pairs costs 2 — from which the
-- two-line cross-file evidence floor is a theorem, not a threshold:
-- a single cross line is 1*1 + 2 = 3 = 1*3, a tie, and ties do not
-- open. That tie IS the coincidence rejection.
siteCostCross :: Int
siteCostCross = 2

-- | A site opens iff explaining its lines as moved beats leaving
-- them plain: n*movedCost + s < n*plainCost. Ties resolve to not
-- opening (conservative).
siteOpens :: Int -> Int -> Bool
siteOpens s n = n * movedCost + s < n * plainCost

-- | Minimal block length that opens a cross-pair site (derived).
destFloor :: Int
destFloor = go 1
 where
  go n = if siteOpens siteCostCross n then n else go (n + 1)
