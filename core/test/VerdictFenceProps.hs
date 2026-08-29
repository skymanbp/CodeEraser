-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | The 6.4.0 fence batch on the verdict road (plan v2.18 step #14,
-- piece (b)), split from VerdictWireProps at the 300-line core gate:
-- the provenance table answering `dropped` (K49) and the cycle floor
-- riding as threshold code 7 with its self-loop table (K50). The
-- request scaffold and the reply reader are VerdictWireProps' own.
module VerdictFenceProps (battery) where

import CE.Verdict (respond)
import CE.Verdict.Cost (verdictNodeCap)
import Data.Aeson
import qualified Data.Aeson.Key as Key
import qualified Data.Aeson.KeyMap as KM
import VerdictWireProps (replyObj, wPos, wireReq)
import WireHarness (refusedBy, runChecks, setKey)

battery :: IO Bool
battery =
  runChecks
    [ ("K49: present answers dropped — the rows an exclusion explains, by name", presentDropped)
    , ("K50: cycleFloor rides as code 7 with the self-loop table, required at 1 and refused elsewhere", cycleFloorRoad)
    ]

-- | K49 (6.4.0, O40): a committed baseline holds three rows over two
-- entities; this run measured entity 7 only. With entity 8 in the
-- `present` table its two rows are DROPPED (an exclusion hid a file
-- that still exists) and `rows_dropped` holds; with an empty table
-- the rows simply vanished (a deletion) and nothing holds; without
-- the table there is no `dropped` key at all — a pre-6.4.0 reply,
-- byte for byte. The table's own grammar refuses by name, and the
-- degraded face still answers the key (an empty table) so a capped
-- reply cannot be mistaken for an old core's.
presentDropped :: Bool
presentDropped =
  ratchetOf (present [8]) "dropped" == Just (toJSON ([[8, 0, 200], [8, 1, 5]] :: [[Integer]]))
    && ratchetOf (present [8]) "failed" == Just (toJSON (["rows_dropped"] :: [String]))
    && ratchetOf (present []) "dropped" == Just (toJSON ([] :: [Value]))
    && ratchetOf (present []) "failed" == Just (toJSON ([] :: [String]))
    && ratchetOf committed "dropped" == Nothing
    && ratchetOf committed "failed" == Just (toJSON ([] :: [String]))
    && refused (present [8, 7]) "present"
    && refused (setKey "present" (toJSON [-1 :: Integer]) committed) "present"
    && ratchetOf (setKey "present" (toJSON ([] :: [Integer])) overCap) "dropped" == Just (toJSON ([] :: [Value]))
 where
  committed =
    setKey "baseline" (object ["continuous" .= ([[7, 0, 300], [8, 0, 200], [8, 1, 5]] :: [[Integer]]), "discrete" .= ([] :: [Integer])]) $
      setKey "continuous" (toJSON ([[7, 0, 300]] :: [[Integer]])) $
        wireReq [] [] [] []
  present us = setKey "present" (toJSON (us :: [Integer])) committed
  overCap = setKey "tier" (toJSON [[u, 0] | u <- [0 .. verdictNodeCap]]) (wireReq [] [] [] [])
  refused = refusedBy respond

-- | One key of the reply's ratchet object.
ratchetOf :: Value -> String -> Maybe Value
ratchetOf req k = case replyObj req of
  Just o -> case KM.lookup "ratchet" o of
    Just (Object rt) -> KM.lookup (Key.fromString k) rt
    _ -> Nothing
  Nothing -> Nothing

-- | K50 (6.4.0, O59): threshold code 7 echoes as `cycleFloor` exactly
-- when it rode; at floor 1 the self-loop table is required (refused
-- absent) and elsewhere refused present; an index outside the file
-- universe refuses; and a singleton SCC with a self-arc is charged on
-- the cycle axis at floor 1 while the same request without the loop
-- is not — the graph's cycle table and this axis read ONE floor.
cycleFloorRoad :: Bool
cycleFloorRoad =
  knob (floorOne (loops [0])) "cycleFloor" == Just (toJSON (1 :: Integer))
    && knob base "cycleFloor" == Nothing
    && refused (floorOne base) "cycleSelfLoops: required"
    && refused (loops [0]) "cycleSelfLoops: only meaningful"
    && refused (floorOne (loops [9])) "cycleSelfLoops"
    && refused (setKey "thresholds" (toJSON ([[7, 0]] :: [[Integer]])) base) "knob below 1"
    && axis 6 (floorOne (loops [0])) > 0
    && axis 6 (floorOne (loops [])) == 0
 where
  base = wireReq [] wPos [] []
  floorOne = setKey "thresholds" (toJSON ([[7, 1]] :: [[Integer]]))
  loops us = setKey "cycleSelfLoops" (toJSON (us :: [Integer])) base
  refused = refusedBy respond
  knob req k = case replyObj req of
    Just o -> case KM.lookup "knobs" o of
      Just (Object ks) -> KM.lookup (Key.fromString k) ks
      _ -> Nothing
    Nothing -> Nothing
  axis code req = case replyObj req of
    Just o -> case KM.lookup "axes" o of
      Just v -> case fromJSON v :: Result [[Integer]] of
        Success rows -> maybe (-1) id (lookup code [(c, p) | [c, p] <- rows])
        _ -> -1
      Nothing -> -1
    Nothing -> -1
