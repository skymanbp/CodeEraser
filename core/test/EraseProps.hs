-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | The erase family's battery (M9 batch 3): the safety predicate
-- against an enumerated truth table (exact tuples — a wrong REASON
-- is as red as a wrong verdict), the mixed request through the REAL
-- respond with count conservation, refusal-by-name for every
-- malformed fact, the knobless stance, and the degraded-fails
-- posture with an EMPTY verdict table (a refused plan licenses
-- nothing). Scaffolding lives in WireHarness.
module EraseProps (battery) where

import CE.Erase (respond)
import CE.Erase.Cost (eraseRowCap, judgeRow)
import Data.Aeson
import WireHarness (degradedFace, field, refusedBy, replyObjWith, rowsRequest, runChecks, setKey)

battery :: IO Bool
battery =
  runChecks
    [ ("the predicate matches the enumerated truth table, reasons included", truthTable)
    , ("a mixed request judges every row in order and conserves counts", mixed)
    , ("erase refusals name the offender", refusals)
    , ("the family is knobless: any knob row refuses by name", knobless)
    , ("an over-cap erase request degrades to an EMPTY table that FAILS", degradedFails)
    ]

-- | Exact expected (eraseable, reason) per fact combination — the
-- independent second derivation is this literal table, written from
-- the contract (erase.md), not from the code. Facts and expectations
-- ride as two PARALLEL data lists: a per-case assertion line was a
-- T2 clone chain by this repo's own measure (the length conjunct
-- guards zipWith's silent truncation).
truthTable :: Bool
truthTable =
  length facts == length wants
    && and (zipWith (\r v -> judgeRow r == v) facts wants)
 where
  facts =
    -- class 0 is absent by construction (4.0.0): the row shape
    -- refuses it upstream, so no judgeRow case can be written for
    -- it -- `refusals` below is where its retirement is asserted.
    [ [1, 60, 60, 60, 1], [1, 59, 60, 60, 1]
    , [1, 60, 60, 61, 1], [1, 60, 60, 60, 0]
    , [2, 1, 1, 1, 0], [2, 0, 1, 1, 0], [2, 1, 0, 1, 0]
    , [2, 1, 1, 0, 0], [2, 1, 1, 1, 3]
    , [3, 1, 0, 0, 0], [3, 2, 1, 0, 0], [3, 4, 2, 0, 0]
    ]
  wants =
    [ (True, 0), (False, 2), (False, 2), (False, 3)
    , (True, 0), (False, 5), (False, 3), (False, 4), (False, 1)
    , (False, 1), (True, 0), (True, 0)
    ]

req :: [[Integer]] -> Value
req = rowsRequest "5.1.0" "erase.request"

mixed :: Bool
mixed = case replyObjWith respond (req rows) of
  Just o ->
    field o "rows" == Just (toJSON [[1, 0 :: Integer], [0, 1], [1, 0], [0, 3]])
      && field o "degraded" == Just (Bool False)
      && field o "fail" == Just (Bool False)
      && (counts o "eraseable" == Just 2)
      && (counts o "advisory" == Just 2)
      && (counts o "rows" == Just 4)
  Nothing -> False
 where
  -- the two dead_file rows ride class 3 since 4.0.0 retired class
  -- 0; confidence 2 (vouched) and 0 (unvouched) reproduce exactly
  -- the verdicts the local-count road gave them
  rows =
    [ [3, 3, 2, 0, 0]
    , [3, 1, 0, 0, 0]
    , [1, 80, 80, 80, 1]
    , [2, 1, 0, 1, 0]
    ]
  counts o k = do
    Object c <- field o "counts"
    n <- field c k
    case n of Number v -> Just v; _ -> Nothing

refusals :: Bool
refusals =
  and
    [ refusedBy respond (req [[4, 0, 0, 0, 0]]) "row 0: unknown class"
    , -- K2: the retired class is refused BY NAME, not folded into
      -- "unknown class" -- a client still sending it learns which
      -- road replaced it (2.32.0's class 3), and no class-0 row can
      -- ever be judged again
      refusedBy respond (req [[0, 1, 0, 0, 0]]) "row 0: retired class 0"
    , refusedBy respond (req [[3, 0, 0, 0, 0]]) "row 0: dead verdict outside 1..4"
    , refusedBy respond (req [[3, 1, 3, 0, 0]]) "row 0: confidence outside 0..2"
    , refusedBy respond (req [[3, 1, -1, 0, 0]]) "row 0: negative fact"
    , refusedBy respond (req [[3, 5, 0, 0, 0]]) "row 0: dead verdict outside 1..4"
    , refusedBy respond (req [[1, 60, 60, 60, 2]]) "row 0: bytesEqual not a boolean"
    , refusedBy respond (req [[2, 2, 1, 1, 0]]) "row 0: coverage/equality/death not booleans"
    , refusedBy respond (req [[3, 1, 0, 0]]) "row 0: malformed row"
    ]

knobless :: Bool
knobless =
  refusedBy
    respond
    (setKey "knobs" (toJSON [[0 :: Integer, 1]]) (req [[3, 1, 0, 0, 0]]))
    "knob 0: erase/1 declares no knob codes"

degradedFails :: Bool
degradedFails = degradedFace respond (req big) "rows" "erase_too_large"
 where
  big = replicate (fromInteger eraseRowCap + 1) [3, 1, 0, 0, 0]
