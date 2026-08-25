-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | The audit family's battery (M9 batch 7 slice 7): the conviction
-- disjunction against its enumerated truth table, the zero-tolerance
-- boundary at exactly ONE conviction, refusal-by-name for smuggled
-- counts and malformed rows, the knobless stance, and the
-- degraded-fails posture (a refused index licenses nothing — and the
-- CLI turns that fail into a VISIBLE degraded skip, never a block).
-- Scaffolding lives in WireHarness.
module AuditProps (battery) where

import CE.Audit (respond)
import CE.Audit.Cost (auditBlockCap, dupTolerance, judgeRow)
import Data.Aeson
import WireHarness (degradedFace, field, refusedBy, replyObjWith, rowsRequest, runChecks, setKey)

battery :: IO Bool
battery =
  runChecks
    [ ("the conviction disjunction matches its truth table", truthTable)
    , ("a mixed request convicts by index and fails past zero tolerance", mixed)
    , ("a clean index passes: no convictions, no fail", clean)
    , ("audit refusals name the offender", refusals)
    , ("the family is knobless: any knob row refuses by name", knobless)
    , ("an over-cap audit request degrades to an EMPTY table that FAILS", degradedFails)
    ]

-- | Either touched side convicts — the deliberate asymmetry (the
-- pre-existing pair whose other side was merely brushed) is the
-- contract, so all four combinations are pinned.
truthTable :: Bool
truthTable =
  dupTolerance == 0
    && map judgeRow [[0, 0], [1, 0], [0, 1], [1, 1]] == [False, True, True, True]

req :: [[Integer]] -> Value
req = rowsRequest "6.0.0" "audit.request"

mixed :: Bool
mixed = case replyObjWith respond (req [[0, 0], [1, 0], [0, 1], [1, 1], [0, 0]]) of
  Just o ->
    field o "dups" == Just (toJSON [1, 2, 3 :: Integer])
      && field o "fail" == Just (Bool True)
      && field o "degraded" == Just (Bool False)
      && (counts o "rows" == Just (Number 5))
      && (counts o "dups" == Just (Number 3))
  Nothing -> False

clean :: Bool
clean = case replyObjWith respond (req [[0, 0], [0, 0]]) of
  Just o ->
    field o "dups" == Just (toJSON ([] :: [Integer]))
      && field o "fail" == Just (Bool False)
      && (counts o "dups" == Just (Number 0))
  Nothing -> False

counts :: Object -> String -> Maybe Value
counts o k = do
  Object c <- field o "counts"
  field c k

refusals :: Bool
refusals =
  and
    [ refusedBy respond (req [[2, 0]]) "row 0: touched bit not a boolean"
    , refusedBy respond (req [[0, -1]]) "row 0: touched bit not a boolean"
    , refusedBy respond (req [[0]]) "row 0: malformed row"
    , refusedBy respond (req [[0, 0, 0]]) "row 0: malformed row"
    ]

knobless :: Bool
knobless =
  refusedBy
    respond
    (setKey "knobs" (toJSON [[0 :: Integer, 1]]) (req [[0, 0]]))
    "knob 0: audit/1 declares no knob codes"

degradedFails :: Bool
degradedFails = degradedFace respond (req big) "dups" "audit_too_large"
 where
  big = replicate (fromInteger auditBlockCap + 1) [0, 0]
