-- | The structure-family knob face (E01 split from CE.Structure at
-- the 300-line wall, the CE.Verdict.Knobs precedent): ONE authority
-- table — (code, getter, setter) — so the effective fold and the
-- reply's echo read the SAME rows and a knob cannot exist in one
-- direction only (the weights lesson, designed in).
module CE.Structure.Knobs
  ( knobsOffence
  , knobTable
  , effective
  ) where

import CE.Structure.Axes (Knobs (..), bound)
import CE.Wire (applyRows, ascendingOn)
import Data.Foldable (asum)

knobsOffence :: [[Integer]] -> Maybe String
knobsOffence rows =
  asum [asum (zipWith one [0 :: Int ..] rows), ascendingOn "knob" (take 1) rows]
 where
  one i row = case row of
    [code, v]
      | code < 0 || code > 18 -> Just (label <> "unknown structure knob")
      | v < 1 -> Just (label <> "knob below 1")
      | otherwise -> Nothing
    _ -> Just (label <> "malformed row (need [code,value])")
   where
    label = "knob " <> show i <> ": "

-- | ONE knob authority: the effective fold and the reply echo both
-- read this table (codes 0..18; 17/18 = the v2.7 ② prices).
knobTable :: [(Integer, Knobs -> Integer, Integer -> Knobs -> Knobs)]
knobTable =
  [ (0, kDepthCeil, \v k -> k {kDepthCeil = v})
  , (1, kFanoutCeil, \v k -> k {kFanoutCeil = v})
  , (2, kNamingMin, \v k -> k {kNamingMin = v})
  , (3, kNamingCeil, \v k -> k {kNamingCeil = v})
  , (4, kMixRefFloor, \v k -> k {kMixRefFloor = v})
  , (5, kMisplaceMin, \v k -> k {kMisplaceMin = v})
  , (6, kBigDirFloor, \v k -> k {kBigDirFloor = v})
  , (7, kViolCost, \v k -> k {kViolCost = v})
  , (8, kScale, \v k -> k {kScale = v})
  , (9, kDupMin, \v k -> k {kDupMin = v})
  , (10, kDeadMin, \v k -> k {kDeadMin = v})
  , (11, kStaleMin, \v k -> k {kStaleMin = v})
  , (12, kSeamSoft, \v k -> k {kSeamSoft = v})
  , (13, kSeamHard, \v k -> k {kSeamHard = v})
  , (14, kSeamPMax, \v k -> k {kSeamPMax = v})
  , (15, kRoiRefMilli, \v k -> k {kRoiRefMilli = v})
  , (16, kRoiPhiMilli, \v k -> k {kRoiPhiMilli = v})
  , (17, kRoiCloneMilli, \v k -> k {kRoiCloneMilli = v})
  , (18, kRoiChurnMilli, \v k -> k {kRoiChurnMilli = v})
  ]

-- | Knob rows over the Cost defaults (the effectiveKnobs pattern):
-- absent rows keep the defaults; the fold grammar is CE.Wire's.
effective :: [[Integer]] -> Knobs
effective rows = applyRows [(c, s) | (c, _, s) <- knobTable] rows bound
