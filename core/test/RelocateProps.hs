-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | The relocation stage's battery (v2.35 guard lane, GD4): the rule at
-- its boundary (equal counts are a relocation, one more is not), a
-- mixed request naming its relocations and `all` only when every row
-- is one, the empty request never `all`, refusal by name for a
-- negative count / the write's own copy missing / a malformed row /
-- any knob, and the degraded posture past the cap (nothing credited).
module RelocateProps (battery) where

import CE.Relocate (respond)
import CE.Relocate.Cost (relocateRowCap, relocated)
import Data.Aeson
import WireHarness (fieldsOf, refusedBy, rowsRequest, runChecks, setKey)

req :: [[Integer]] -> Value
req = rowsRequest "9.1.0" "relocate.request"

battery :: IO Bool
battery = runChecks (zip names probes)
 where
  names =
    [ "a relocation does not raise the tree's count of its run"
    , "a mixed request names its relocations and is not all"
    , "every row a relocation is all; no row is never all"
    , "relocate refusals name the offender"
    , "an over-cap request degrades with nothing credited"
    ]
  probes = [rule, mixed, allOrNone, refusals, degradedFace]

rule :: Bool
rule =
  map relocated [[2, 2], [2, 1], [2, 3], [1, 1], [0, 1]]
    == [True, True, False, True, False]

mixed :: Bool
mixed =
  fieldsOf respond (req [[2, 2], [1, 2], [3, 1]]) ["relocated", "all", "counts", "degraded"]
    == Just
      [ Just (toJSON [0, 2 :: Integer])
      , Just (Bool False)
      , Just (object ["rows" .= (3 :: Int)])
      , Just (Bool False)
      ]

allOrNone :: Bool
allOrNone =
  fieldsOf respond (req [[2, 2], [5, 1]]) ["all"] == Just [Just (Bool True)]
    && fieldsOf respond (req []) ["all"] == Just [Just (Bool False)]

refusals :: Bool
refusals =
  and
    [ refusedBy respond (req [[-1, 1]]) "row 0: negative count"
    , refusedBy respond (req [[1, 1], [2, 0]]) "row 1: occNow 0: the write's own copy is missing from the count"
    , refusedBy respond (req [[1, 1, 1]]) "row 0: malformed row"
    , refusedBy respond (setKey "knobs" (toJSON [[0, 1 :: Integer]]) (req [[1, 1]])) "knob 0: relocate/1 declares no knob codes"
    ]

degradedFace :: Bool
degradedFace =
  fieldsOf respond (req (replicate (fromInteger relocateRowCap + 1) [1, 1])) ["relocated", "all", "degraded"]
    == Just [Just (toJSON ([] :: [Integer])), Just (Bool False), Just (Bool True)]
