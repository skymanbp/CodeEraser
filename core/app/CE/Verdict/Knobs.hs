-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | The effective-knob assembly and its echo (split from CE.Verdict
-- when the 2.14.0 zone knobs pushed it past the 300-line core
-- wall): ce.toml's four [code, value] tables land on the Cost.hs
-- defaults here, and knobsEcho is the round trip the Rust client
-- asserts. One module owns both directions, so a knob cannot exist
-- in one direction only.
module CE.Verdict.Knobs
  ( effectiveKnobs
  , effectiveRatchet
  , effectiveJoin
  , knobsEcho
  ) where

import CE.Verdict.Join (Knobs (..), bound)
import CE.Verdict.Ratchet (RatchetKnobs (..), ratchetBound)
import CE.Verdict.Score (ScoreKnobs (..), scoreBound)
import CE.Wire (applyRows, pick)
import Data.Aeson

-- | scoreBound with the request's knob rows applied: the ceilings
-- table drives axes 0/1, the thresholds table its Score-owned codes
-- (3 = cochangeFloor is Join-owned and falls through to
-- effectiveJoin). One (code, setter) row per knob; absent rows keep
-- the Cost.hs DEFAULTS. Two tables, not four: the batch's first
-- ratchet bite was the join/tolerance sibling tables, repaid by
-- effectiveJoin/effectiveRatchet constructing records directly.
effectiveKnobs :: [[Integer]] -> [[Integer]] -> ScoreKnobs
effectiveKnobs ceils thrs =
  -- ceilings as a record from lookups, not a sibling lambda table
  -- (the effectiveRatchet precedent — the 2.14.0 zone codes 2..4
  -- made the setter-table twin a literal clone of thresholdSetters,
  -- and the dedup ratchet bit)
  k0
    { sSizeCeil = leg 0 sSizeCeil
    , sCocCeil = leg 1 sCocCeil
    , sSizeHard = leg 2 sSizeHard
    , sSizePMax = leg 3 sSizePMax
    , sSoftK = leg 4 sSoftK
    }
 where
  k0 = applyRows thresholdSetters thrs scoreBound
  leg c f = pick ceils c (f k0)

thresholdSetters :: [(Integer, Integer -> ScoreKnobs -> ScoreKnobs)]
thresholdSetters =
  [ (0, \v k -> k {sDeadIndegCeil = v})
  , (1, \v k -> k {sRewriteNum = v})
  , (2, \v k -> k {sRewriteDen = v})
  , (4, \v k -> k {sViolCost = v})
  , (5, \v k -> k {sDefaultWeight = v})
  , (6, \v k -> k {sScoreScale = v})
  ]

-- | The tolerance legs by position (0 num / 1 den / 2 abs): a
-- record from lookups, not a sibling lambda table.
effectiveRatchet :: [[Integer]] -> RatchetKnobs
effectiveRatchet rows =
  RatchetKnobs (leg 0 rTolNum) (leg 1 rTolDen) (leg 2 rTolAbs)
 where
  leg c f = pick rows c (f ratchetBound)

-- | The join lattice reads the SAME effective rewrite ratio the
-- score judged with (one authority, two readers) plus the cochange
-- floor row (code 3) the score does not own.
effectiveJoin :: ScoreKnobs -> [[Integer]] -> Knobs
effectiveJoin k thrs =
  bound
    { kRewriteNum = sRewriteNum k
    , kRewriteDen = sRewriteDen k
    , kCochangeFloor = pick thrs 3 (kCochangeFloor bound)
    }

-- pick/applyRows live in CE.Wire since the twelfth bite (the
-- seventh family recloned both) — the fold grammar is family
-- infrastructure, not verdict-family property.

-- | Every configurable knob the judgment ran with, echoed so the
-- client can assert the FULL round trip and the empty-table default
-- gate can kill every remaining mirror. The dedup floor (batch-7
-- slice 1, 2.19.0) arrives as its own parameter: it is effective
-- per REQUEST (CLI --min-distinct override or CE.Dedup.Cost
-- default), not a member of the three knob sets.
knobsEcho :: ScoreKnobs -> RatchetKnobs -> Knobs -> Integer -> Value
knobsEcho k rk jk dedupFloor =
  object
    [ "sizeCeil" .= sSizeCeil k
    , "sizeHard" .= sSizeHard k
    , "sizePMax" .= sSizePMax k
    , "softLineK" .= sSoftK k
    , "cocCeil" .= sCocCeil k
    , "deadIndegCeil" .= sDeadIndegCeil k
    , "rewriteNum" .= sRewriteNum k
    , "rewriteDen" .= sRewriteDen k
    , "cochangeFloor" .= kCochangeFloor jk
    , "violCost" .= sViolCost k
    , "defaultWeight" .= sDefaultWeight k
    , "scoreScale" .= sScoreScale k
    , "tolNum" .= rTolNum rk
    , "tolDen" .= rTolDen rk
    , "tolAbs" .= rTolAbs rk
    , "minDistinct" .= dedupFloor
    ]

