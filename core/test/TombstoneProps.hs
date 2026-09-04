-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | The tombstone family's battery (plan v2.27 step 4): the residue
-- conjunction against its truth table, the budget boundary at exactly
-- the declared number, no condition without a budget, refusal by name
-- for a foreign kind / a negative count / a malformed row / a foreign
-- knob, and the degraded posture (an empty site table, the condition
-- unevaluated). Scaffolding lives in WireHarness; the probes read
-- their reply fields through `fieldsOf` in one projection each.
module TombstoneProps (battery) where

import CE.Tombstone (respond)
import CE.Tombstone.Cost (isSite, minMarks, minName, overBudget, tombstoneRowCap)
import Data.Aeson
import WireHarness (fieldsOf, refusedBy, rowsRequest, runChecks, setKey)

-- | The request every probe edits: rows under the anchor proto, knobs
-- added by `withKnobs`.
req :: [[Integer]] -> Value
req = rowsRequest "6.6.0" "tombstone.request"

withKnobs :: [[Integer]] -> Value -> Value
withKnobs ks = setKey "knobs" (toJSON ks)

battery :: IO Bool
battery = runChecks (zip names probes)
 where
  names =
    [ "the conjunction matches its truth table"
    , "a mixed request names its sites, splits them and judges the budget"
    , "without a budget there is no condition"
    , "the budget boundary is strict"
    , "tombstone refusals name the offender"
    , "an over-cap request degrades to an empty table, condition unevaluated"
    ]
  probes = [truthTable, mixed, unbudgeted, boundary, refusals, degradedFace]

-- | A label binds a name; prose needs the mark AND the name; a
-- changeset is over its budget strictly, and never without one.
truthTable :: Bool
truthTable =
  (minName, minMarks) == (1, 1)
    && map isSite [[0, 0, 1], [1, 0, 1], [0, 0, 0], [2, 1, 1], [2, 1, 0], [2, 0, 1], [2, 0, 0]]
      == [True, True, False, True, False, False, False]
    && not (overBudget Nothing 5)
    && not (overBudget (Just 2) 2)
    && overBudget (Just 2) 3

mixed :: Bool
mixed =
  fieldsOf respond (withKnobs [[0, 1]] (req rows)) ["sites", "counts", "over", "knobs", "degraded"]
    == Just
      [ Just (toJSON [0, 1, 2 :: Integer])
      , Just (object ["rows" .= (6 :: Int), "label" .= (2 :: Int), "prose" .= (1 :: Int)])
      , Just (Bool True)
      , Just (toJSON [[0, 1 :: Integer]])
      , Just (Bool False)
      ]
 where
  rows = [[0, 0, 1], [1, 0, 1], [2, 1, 1], [2, 1, 0], [2, 0, 1], [2, 0, 0]]

unbudgeted :: Bool
unbudgeted =
  fieldsOf respond (req [[2, 1, 1], [0, 0, 1]]) ["sites", "over", "knobs"]
    == Just [Just (toJSON [0, 1 :: Integer]), Just (Bool False), Just (toJSON ([] :: [[Integer]]))]

boundary :: Bool
boundary = over 2 == Just [Just (Bool False)] && over 1 == Just [Just (Bool True)]
 where
  over b = fieldsOf respond (withKnobs [[0, b]] (req [[0, 0, 1], [2, 1, 1]])) ["over"]

refusals :: Bool
refusals =
  and
    [ refusedBy respond (req [[3, 0, 1]]) "row 0: kind outside 0..2"
    , refusedBy respond (req [[2, -1, 1]]) "row 0: negative count"
    , refusedBy respond (req [[0, 1]]) "row 0: malformed row"
    , refusedBy respond (withKnobs [[1, 5]] (req [[0, 0, 1]])) "knob 0: unknown knob code"
    , refusedBy respond (withKnobs [[0, -1]] (req [[0, 0, 1]])) "knob 0: negative knob value"
    , refusedBy respond (withKnobs [[0]] (req [[0, 0, 1]])) "knob 0: malformed knob"
    ]

degradedFace :: Bool
degradedFace =
  fieldsOf respond (req big) ["degraded", "sites", "over", "reason"]
    == Just [Just (Bool True), Just (toJSON ([] :: [Integer])), Just (Bool False), Just "tombstone_too_large"]
 where
  big = replicate (fromInteger tombstoneRowCap + 1) [2, 1, 1]
