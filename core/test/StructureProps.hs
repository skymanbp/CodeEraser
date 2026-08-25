-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | The structure family's battery (M6 S2): one hand-computed tree
-- fixture through the REAL respond — axes, score, entropy rows and
-- findings all checked to the digit — plus a lever per knob (twelve
-- rows total, F16: 0..8 levered in knobLevers below, 9/10 in the
-- redundancy leg and 11 in the staleness leg), the refusals by
-- name, and the degraded-fails posture.
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
    , ("the redundancy axis judges present, absent and clean apart", redundancyAxis)
    , ("the staleness axis judges by hand-computed digit", staleAxis)
    ]

-- | Fixture: root (2 subdirs, 3 files, README+config) / dir 1
-- (9 files, 5 snake + 4 pascal names, 2 files with 4 outside refs
-- each, no README) / dir 2 (6 files, uniform names) / dir 3 (an
-- empty dir under dir 2 at depth 2 — the depthCeil lever's seat).
-- Hand-computed: S1 dir1 naming 987‰ > 600; S2 dir1 outs 8 > ins 0
-- at traffic 8; S3 dir1 holds misplaced files (two at outside 4 ≥ 3
-- and > 2×0 — ONE dir, dir-counted per amendment ①); S4 dir1 big
-- and README-less; S0 clean. Density fold (2.26.0): N = 4 dirs, so
-- each v=1 axis charges floor(1000·1/5) = 200; score = 1000 −
-- (0+200·4)·10 div (10·5) = 840. Entropy: global patterns [11,4]
-- → 782‰; dir files [3,9,6] (zero-file dir 3 filtered) → 916‰.
wireReq :: Value
wireReq =
  object
    [ "proto" .= ("5.0.0" :: String)
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

-- | The fixture's always-on findings (axes 1..4 flag dir 1) — ONE
-- definition, because the optional-axis probes below assert the same
-- prefix plus their own gain.
baseFindings :: [[Integer]]
baseFindings = [[1, 1], [1, 2], [1, 3], [1, 4]]

fixtureJudged :: Bool
fixtureJudged = case replyObj wireReq of
  Nothing -> False
  Just o ->
    field o "axes" == Just (toJSON [[0, 0], [1, 200], [2, 200], [3, 200], [4, 200 :: Integer]])
      && field o "score" == Just (Number 840)
      && field o "entropy" == Just (toJSON [[0, 782], [1, 916 :: Integer]])
      && field o "findings" == Just (toJSON baseFindings)
      && field o "fail" == Just (Bool False)

-- | Each knob row against its hand-computed flip: the same fixture,
-- one override, one designated number moves (F16 for knobs 0..8;
-- 9/10/11 get their levers in the redundancy/staleness legs).
-- Density base (amendment ① counts + 2.26.0 fold): charges at
-- N = 4 are v=1 → 200, v=2 → 333, v=3 → 428; base sum 800 → 840.
knobLevers :: Bool
knobLevers =
  and
    [ scoreAt [[0, 1]] == want 800 -- dir 3 depth 2 > ceil 1: S0 v=1, sum 1000
    , scoreAt [[1, 2]] == want 755 -- fanouts 5, 9, 7 > 2: S0 v=3 → 428, sum 1228
    , scoreAt [[2, 10]] == want 880 -- naming set 9 < min 10: S1 off, sum 600
    , scoreAt [[3, 990]] == want 880 -- ceil above 987‰: S1 off
    , scoreAt [[4, 9]] == want 880 -- traffic 8 < floor 9: S2 off
    , scoreAt [[5, 5]] == want 880 -- outside 4 < min 5: S3 off
    , scoreAt [[6, 10]] == want 880 -- dir1 9 < floor 10: S4 off
    , scoreAt [[7, 50]] == want 200 -- violCost 50: 1000 − 800·50 div 50
    , scoreAt [[8, 500]] == want 420 -- scale 500: charges 100·4, 500 − 80
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
    , -- the forged-depth probe (review 2026-08-20 #6): nodeRow's old
      -- docstring claimed depth was checked by position; this row
      -- rode a depth of 999 into the geometry axes and moved the
      -- score before depthChain existed
      refused (nodes [[0, 0, 0, 2, 3], [1, 0, 999, 0, 9]]) "depth is not parent depth + 1"
    , refused (setKey "patterns" (toJSON [[0, 7, 1 :: Integer]]) wireReq) "unknown pattern code"
    , refused (setKey "conventions" (toJSON [[0, 0 :: Integer]]) wireReq) "bits outside 1..3"
    , refused (setKey "fileRefs" (toJSON [[9, 0, 1, 1 :: Integer]]) wireReq) "dir out of range"
    , -- max+1: the boundary-exact unknown code, moved WITH the knob
      -- face by this battery's own F16 discipline (the golden's
      -- unknown-knob pair probes a stably-unknown 99 instead — the
      -- second face-growth made its probe legal and taught us not
      -- to freeze a moving boundary in a fixture; 17/18 = the v2.7
      -- price knobs, so max+1 is 19 now)
      refused (setKey "knobs" (toJSON [[19, 1 :: Integer]]) wireReq) "unknown structure knob"
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

-- | S3b, absence vs zero vs mass — all by hand (density base 800).
-- Absent table: five axes, score 840 (fixtureJudged). Empty table:
-- six axes with [6,0], score 1000 − 8000 div 60 = 867. Two flagged
-- dirs (dir 1 one clone block, dir 2 one dead unit): axis 6 v=2 →
-- 333, sum 1133, score 812, findings gain [1,6],[2,6]. Knob levers:
-- dupMin 2 releases dir 1, deadMin 2 releases dir 2 — either way
-- one dir stays, axis 6 → 200, sum 1000, score 834 (F16, both
-- knobs live). Refusals ride the shared dir-table loop.
redundancyAxis :: Bool
redundancyAxis =
  and
    [ probeAt [] "score" == Just (toJSON (867 :: Integer))
    , probeAt [] "axes"
        == Just (toJSON [[0, 0], [1, 200], [2, 200], [3, 200], [4, 200], [6, 0 :: Integer]])
    , probeAt flagged "score" == Just (toJSON (812 :: Integer))
    , probeAt flagged "findings" == Just (toJSON (baseFindings <> [[1, 6], [2, 6]]))
    , fmap (field' "score") (replyObj (setKey "knobs" (toJSON [[9, 2 :: Integer]]) redReq))
        == Just (Just (toJSON (834 :: Integer)))
    , fmap (field' "score") (replyObj (setKey "knobs" (toJSON [[10, 2 :: Integer]]) redReq))
        == Just (Just (toJSON (834 :: Integer)))
    , refusedBy respond (redAt [[9, 0, 0]]) "redundancy 0: dir out of range"
    , refusedBy respond (redAt [[2, 1, 0], [1, 0, 1]]) "redundancy 1: not strictly ascending"
    ]
 where
  flagged = [[1, 1, 0], [2, 0, 1 :: Integer]]
  redReq = redAt flagged
  redAt rows = setKey "redundancy" (toJSON (rows :: [[Integer]])) wireReq
  probeAt rows key = replyObj (redAt rows) >>= \o -> field o key
  field' k o = field o k

-- | S3c on the RAW road (2.29.0 — the pre-judged staleDocs arm
-- retired): empty raw table = six axes with [5,0], 1000 − 8000
-- div 60 = 867; dir 1 with two docs and ONE changed-later target
-- (doc ts 5 < target ts 7; the ts-9 doc has no changed target)
-- derives [1,1,2] = axis 5 charge 200, sum 1000, score 834,
-- findings gain [1,5]; staleMin 2 releases it back to 867 (F16);
-- both optional tables together = seven axes, score 810. The
-- LEGACY staleDocs key alone judges nothing (§1 unknown-field
-- rule) — the retirement's own probe. Refusals ride the raw-table
-- validators.
staleAxis :: Bool
staleAxis =
  and
    [ staleAt ([], []) "score" == Just (toJSON (867 :: Integer))
    , staleAt flaggedRaw "score" == Just (toJSON (834 :: Integer))
    , staleAt flaggedRaw "findings" == Just (toJSON (baseFindings <> [[1, 5]]))
    , fmap (`field` "score") (replyObj (setKey "knobs" (toJSON [[11, 2 :: Integer]]) staleReq))
        == Just (Just (toJSON (867 :: Integer)))
    , fmap (`field` "score") (replyObj bothReq) == Just (Just (toJSON (810 :: Integer)))
    , -- ignored ≡ absent: the legacy key must change NOTHING
      (replyObj (setKey "staleDocs" (toJSON [[1, 1, 2 :: Integer]]) wireReq) >>= \o -> field o "score")
        == (replyObj wireReq >>= \o -> field o "score")
    , refusedBy respond (staleReqAt ([[9, 5]], [])) "dir out of range"
    , refusedBy respond (staleReqAt ([[1, 5]], [[0, 0]])) "targetTs below 1"
    , refusedBy respond (staleReqAt ([[1, 5]], [[7, 3]])) "docIdx out of range"
    ]
 where
  flaggedRaw = ([[1, 5], [1, 9]], [[0, 7]])
  staleReq = staleReqAt flaggedRaw
  staleReqAt (docs, edges) =
    setKey "staleEdgeRows" (toJSON (edges :: [[Integer]])) $
      setKey "staleDocRows" (toJSON (docs :: [[Integer]])) wireReq
  staleAt rows key = replyObj (staleReqAt rows) >>= \o -> field o key
  bothReq =
    setKey "redundancy" (toJSON [[1, 1, 0], [2, 0, 1 :: Integer]]) staleReq

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
