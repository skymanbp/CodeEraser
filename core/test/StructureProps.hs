-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | The structure family's battery (M6 S2): one hand-computed tree
-- fixture through the REAL respond — axes, score, entropy rows and
-- findings all checked to the digit — plus a lever per knob (every
-- one of the nine rows must flip a designated number, F16), the
-- refusals by name, and the degraded-fails posture.
module StructureProps (battery) where

import CE.Structure (respond)
import CE.Structure.Cost (structNodeCap)
import Data.Aeson
import WireHarness (field, refusedBy, replyObjWith, runChecks, setKey)

battery :: IO Bool
battery =
  runChecks
    [ ("the fixture tree judges to the hand-computed digit", fixtureJudged)
    , ("every structure knob is a live lever", knobLevers)
    , ("structure refusals name the offender", refusals)
    , ("an over-cap structure request degrades and FAILS", degradedFails)
    , ("the declared layout overlays by hand-computed digit", declaredOverlay)
    ]

-- | Fixture: root (2 subdirs, 3 files, README+config) / dir 1
-- (9 files, 5 snake + 4 pascal names, 2 files with 4 outside refs
-- each, no README) / dir 2 (6 files, uniform names) / dir 3 (an
-- empty dir under dir 2 at depth 2 — the depthCeil lever's seat).
-- Hand-computed: S1 dir1 naming 987‰ > 600; S2 dir1 outs 8 > ins 0
-- at traffic 8; S3 two files outside 4 ≥ 3 and > 2×0; S4 dir1 big
-- and README-less; S0 clean. raw = (0+1+1+2+1)·10 = 50, score =
-- 1000 − 50 div 5 = 990. Entropy: global patterns [11,4] → 782‰;
-- dir files [3,9,6] (zero-file dir 3 filtered) → 916‰.
wireReq :: Value
wireReq =
  object
    [ "proto" .= ("2.10.0" :: String)
    , "type" .= ("structure.request" :: String)
    , "id" .= (1 :: Int)
    , "nodes"
        .= [ [0, 0, 0, 2, 3]
           , [1, 0, 1, 0, 9]
           , [2, 0, 1, 1, 6]
           , [3, 2, 2, 0, 0 :: Integer]
           ]
    , "patterns" .= [[1, 0, 5], [1, 3, 4], [2, 0, 6 :: Integer]]
    , "conventions" .= [[0, 3 :: Integer]]
    , "fileRefs" .= [[1, 0, 4, 2 :: Integer]]
    ]

replyObj :: Value -> Maybe Object
replyObj = replyObjWith respond

fixtureJudged :: Bool
fixtureJudged = case replyObj wireReq of
  Nothing -> False
  Just o ->
    field o "axes" == Just (toJSON [[0, 0], [1, 1], [2, 1], [3, 2], [4, 1 :: Integer]])
      && field o "score" == Just (Number 990)
      && field o "entropy" == Just (toJSON [[0, 782], [1, 916 :: Integer]])
      && field o "findings" == Just (toJSON [[1, 1], [1, 2], [1, 4 :: Integer]])
      && field o "fail" == Just (Bool False)

-- | Each knob row against its hand-computed flip: the same fixture,
-- one override, one designated number moves (F16: nine knobs, nine
-- levers, none dead).
knobLevers :: Bool
knobLevers =
  and
    [ scoreAt [[0, 1]] == want 988 -- dir 3 depth 2 > ceil 1: +1 geometry
    , scoreAt [[1, 2]] == want 984 -- fanouts 5, 9, 7 > 2: +3 geometry
    , scoreAt [[2, 10]] == want 992 -- naming set 9 < min 10: S1 off
    , scoreAt [[3, 990]] == want 992 -- ceil above 987‰: S1 off
    , scoreAt [[4, 9]] == want 992 -- traffic 8 < floor 9: S2 off
    , scoreAt [[5, 5]] == want 994 -- outside 4 < min 5: S3 off
    , scoreAt [[6, 10]] == want 992 -- dir1 9 < floor 10: S4 off
    , scoreAt [[7, 50]] == want 950 -- violCost 50: 1000 − 250 div 5
    , scoreAt [[8, 500]] == want 490 -- scale 500: 500 − 10
    ]
 where
  want n = Just (toJSON (n :: Integer))
  scoreAt :: [[Integer]] -> Maybe Value
  scoreAt rows = do
    o <- replyObj (setKey "knobs" (toJSON rows) wireReq)
    field o "score"

refusals :: Bool
refusals =
  and
    [ refused (nodes [[0, 0, 0, 2, 3], [2, 0, 1, 0, 9]]) "index mismatch"
    , refused (nodes [[0, 1, 0, 2, 3]]) "root must self-loop"
    , refused (nodes [[0, 0, 0, 2, 3], [1, 1, 1, 0, 9]]) "parent not before child"
    , refused (setKey "patterns" (toJSON [[0, 7, 1 :: Integer]]) wireReq) "unknown pattern code"
    , refused (setKey "conventions" (toJSON [[0, 0 :: Integer]]) wireReq) "bits outside 1..3"
    , refused (setKey "fileRefs" (toJSON [[9, 0, 1, 1 :: Integer]]) wireReq) "dir out of range"
    , refused (setKey "knobs" (toJSON [[9, 1 :: Integer]]) wireReq) "unknown structure knob"
    , refused (setKey "knobs" (toJSON [[3, 0 :: Integer]]) wireReq) "knob below 1"
    , refused (setKey "patterns" (toJSON [[1, 3, 4], [1, 0, 5 :: Integer]]) wireReq) "not strictly ascending"
    , refused (setKey "declared" (toJSON [[9, 1 :: Integer]]) wireReq) "declared 0: dir out of range"
    , refused (setKey "declared" (toJSON [[1, 0 :: Integer]]) wireReq) "weight below 1"
    , refused (setKey "declared" (toJSON [[2, 1], [1, 1 :: Integer]]) wireReq) "declared 1: not strictly ascending"
    ]
 where
  nodes rows = setKey "nodes" (toJSON (rows :: [[Integer]])) wireReq
  refused = refusedBy respond

-- | The S3 A-layer against the same fixture, every digit by hand.
-- Full coverage — declared (0,w1)(1,w2)(2,w3), owned (3,9,6) of 18:
-- p=(1/6,1/2,1/3) vs q=(1/6,1/3,1/2), χ² = (1/6)²/(1/3) +
-- (1/6)²/(1/2) = 5/36 → 138‰; weights (1,3,2) match the tree
-- exactly → 0 (the weight lever, F16). Partial coverage — declare
-- dir 1 alone: root and dir 2 hold unowned mass → kind-0 rows, no
-- number. Declaring the empty dir 3 names it kind 1 (dir 3's owner
-- chain also proves nesting: its mass would fold into dir 2's bin).
-- No declaration → NO A-layer keys (the S2 shape, byte posture).
declaredOverlay :: Bool
declaredOverlay =
  all probe overlayCases
    && fmap keysOff (replyObj wireReq) == Just True
 where
  probe (rows, dv, dev) = at rows == Just (toJSON dv, toJSON dev)
  at rows = do
    o <- replyObj (setKey "declared" (toJSON (rows :: [[Integer]])) wireReq)
    d <- field o "divergence"
    v <- field o "deviations"
    pure (d, v)
  keysOff o = field o "divergence" == Nothing && field o "deviations" == Nothing

-- | (declared rows, divergence row, deviation rows) — the probe
-- table IS the A-layer contract in miniature.
overlayCases :: [([[Integer]], [Integer], [[Integer]])]
overlayCases =
  [ ([[0, 1], [1, 2], [2, 3]], [138], [])
  , ([[0, 1], [1, 3], [2, 2]], [0], [])
  , ([[1, 1]], [], [[0, 0], [2, 0]])
  , ([[1, 1], [3, 1]], [], [[0, 0], [2, 0], [3, 1]])
  ]

-- | P1 posture on the newest family: one node past the cap
-- degrades to a complete reply whose fail bit is TRUE.
degradedFails :: Bool
degradedFails = case replyObj overCap of
  Nothing -> False
  Just o ->
    field o "degraded" == Just (Bool True)
      && field o "fail" == Just (Bool True)
 where
  overCap = setKey "nodes" (toJSON [[0, 0, 0, 0, 0 :: Integer] | _ <- [0 .. structNodeCap]]) wireReq
