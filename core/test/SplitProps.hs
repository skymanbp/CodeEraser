-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | The split-ROI advisory battery (plan v2.6 §C): the fixture is
-- hand-computed to the digit — one two-unit file whose seam clears
-- ROI 1, one single-unit file that exempts with 0/0 — then a lever
-- per new knob (F16), the named refusals for each seam table, and
-- the exactly-when-sent reply-key contract.
module SplitProps (battery) where

import CE.Structure (respond)
import Data.Aeson
import qualified Data.Aeson.KeyMap as KM
import WireHarness (field, refusedBy, replyObjWith, runChecks, setKey)

battery :: IO Bool
battery =
  runChecks
    [ ("the fixture seam prices to the hand-computed digit", fixtureJudged)
    , ("the v1.1 legs price to the hand-computed digit", pricedJudged)
    , ("a cap-shaped seam request still answers", scaledJudged)
    , ("every split knob is a live lever", knobLevers)
    , ("split refusals name the offender", refusals)
    , ("the reply keys exist exactly when seamFiles rode", exactlyWhenSent)
    ]

-- | nodes = one root dir; seam tables as documented in the module
-- header. Hand arithmetic (defaults S=300 H=750 P=10, ref 250 φ
-- 500): p(550)=10·(250/450)² = 250/81; both halves (270, 280) are
-- under S so the whole penalty comes back — benefit
-- floor(1000·250/81) = 3086; one crossing ref -> cost 750; the
-- single-unit file has no seam at all.
wireReq :: Value
wireReq =
  setKey "seamFiles" (toJSON [[0, 550], [1, 400 :: Integer]]) $
    setKey
      "seamUnits"
      (toJSON [[0, 0, 10, 270], [0, 1, 280, 540], [1, 0, 1, 390 :: Integer]])
      $ setKey "seamRefs" (toJSON [[0, 0, 1 :: Integer]]) base

base :: Value
base =
  object
    [ "proto" .= ("6.1.0" :: String)
    , "type" .= ("structure.request" :: String)
    , "id" .= (1 :: Int)
    , "nodes" .= [[0, 0, 0, 0, 2 :: Integer]]
    ]

replyObj :: Value -> Maybe Object
replyObj = replyObjWith respond

-- | ONE reply-shape judge both fixtures (and the priced levers)
-- read through: the two advisory arrays equal the hand-computed
-- rows, or the check fails.
judgedAs :: Value -> [[Integer]] -> [[Integer]] -> Bool
judgedAs req cands exempts = case replyObj req of
  Nothing -> False
  Just o ->
    field o "splitCandidates" == Just (toJSON cands)
      && field o "sizeExempt" == Just (toJSON exempts)

-- | Also the 2.14.0-compat regression: with the clone/churn tables
-- ABSENT, both v1.1 legs price at zero and the old cost stands.
fixtureJudged :: Bool
fixtureJudged = judgedAs wireReq [[0, 0, 3086, 750]] [[1, 0, 0]]

-- | v2.7 ② full pricing, hand-computed: one clone span [260, 300]
-- straddles the seam line 270 (260 <= 270 < 300) and one co-change
-- pair (0, 1) crosses the seam — cost = 250 ref + 500 clone + 150
-- churn + 500 φ = 1400; benefit 3086 unchanged.
pricedReq :: Value
pricedReq =
  setKey "seamClones" (toJSON [[0, 260, 300 :: Integer]]) $
    setKey "seamChurn" (toJSON [[0, 0, 1 :: Integer]]) wireReq

pricedJudged :: Bool
pricedJudged = judgedAs pricedReq [[0, 0, 3086, 1400]] [[1, 0, 0]]

-- | The complexity regression (review 2026-08-20 #1/#2), asserted
-- structurally and not by a clock: the priced fixture repeated over
-- 10000 files must answer 10000 rows with the SAME hand-computed
-- 3086/1400 — the per-file numbers cannot depend on how many files
-- rode. 60002 rows is an eighth of structNodeCap, and the pre-fix
-- pricer rescanned seamUnits per file plus the whole
-- refs/clones/churn table per seam (and re-walked the candidate list
-- per append), so this request answered in list steps counted in the
-- billions — i.e. the suite hangs instead of failing an assertion.
scaledJudged :: Bool
scaledJudged = judgedAs scaledReq [[i, 0, 3086, 1400] | i <- ids] []
 where
  ids = [0 .. 9999 :: Integer]
  scaledReq =
    setKey "seamFiles" (toJSON [[i, 550] | i <- ids]) $
      setKey "seamUnits" (toJSON (concat [[[i, 0, 10, 270], [i, 1, 280, 540]] | i <- ids])) $
        setKey "seamRefs" (toJSON [[i, 0, 1] | i <- ids]) $
          setKey "seamClones" (toJSON [[i, 260, 300] | i <- ids]) $
            setKey "seamChurn" (toJSON [[i, 0, 1] | i <- ids]) base

-- | One override per new code, each moving the fixture's verdict:
-- 12 (S=560: the file leaves the zone, benefit 0 -> exempt),
-- 13 (H=600: steeper curve, benefit floor(1000·10·(250/300)²)=6944),
-- 14 (P=20: benefit doubles to 6172), 15 (ref 3000: cost 3500 >
-- benefit -> exempt), 16 (φ 4000: cost 4250 -> exempt); the two
-- v2.7 price knobs lever on the PRICED fixture (base cost 1400):
-- 17 (clone 3000: cost 3900 -> exempt), 18 (churn 2500: cost 3750
-- -> exempt).
knobLevers :: Bool
knobLevers =
  and
    [ probe [[12, 560]] (\c e -> c == Just (toJSON emptyRows) && exemptBest e)
    , probe [[13, 600]] (\c _ -> c == Just (toJSON [[0, 0, 6944, 750 :: Integer]]))
    , probe [[14, 20]] (\c _ -> c == Just (toJSON [[0, 0, 6172, 750 :: Integer]]))
    , probe [[15, 3000]] (\c e -> c == Just (toJSON emptyRows) && e == Just (toJSON [[0, 3086, 3500], [1, 0, 0 :: Integer]]))
    , probe [[16, 4000]] (\c e -> c == Just (toJSON emptyRows) && e == Just (toJSON [[0, 3086, 4250], [1, 0, 0 :: Integer]]))
    , priced [[17, 3000]] [[0, 3086, 3900], [1, 0, 0 :: Integer]]
    , priced [[18, 2500]] [[0, 3086, 3750], [1, 0, 0 :: Integer]]
    ]
 where
  emptyRows = [] :: [[Integer]]
  exemptBest e = e == Just (toJSON [[0, 0, 750], [1, 0, 0 :: Integer]])
  probe rows check = case replyObj (setKey "knobs" (toJSON (rows :: [[Integer]])) wireReq) of
    Nothing -> False
    Just o -> check (field o "splitCandidates") (field o "sizeExempt")
  priced rows = judgedAs (setKey "knobs" (toJSON (rows :: [[Integer]])) pricedReq) []

refusals :: Bool
refusals =
  and
    [ refused (files [[1, 550]]) "index mismatch"
    , refused (files [[0, 0]]) "total below 1"
    , refused (units [[0, 0, 0, 40]]) "bad span"
    , refused (units [[0, 0, 10, 600]]) "span past the file total"
    , refused (units [[0, 1, 10, 40]]) "unit not dense from 0"
    , refused (units [[0, 0, 10, 40], [0, 1, 30, 60]]) "spans overlap or descend"
    , refused (refs [[0, 0, 0]]) "self reference"
    , refused (refs [[0, 0, 9]]) "unit out of range"
    , refused (refs [[2, 0, 1]]) "file out of range"
    , refused (edged "seamClones" [[0, 0, 40]]) "bad span"
    , refused (edged "seamClones" [[0, 10, 600]]) "span past the file total"
    , refused (edged "seamClones" [[2, 10, 40]]) "file out of range"
    , refused (edged "seamChurn" [[0, 1, 1]]) "pair not ascending"
    , refused (edged "seamChurn" [[0, 0, 9]]) "unit out of range"
    ]
 where
  files rows = setKey "seamFiles" (toJSON (rows :: [[Integer]])) base
  units rows =
    setKey "seamUnits" (toJSON (rows :: [[Integer]])) $
      setKey "seamFiles" (toJSON [[0, 550 :: Integer]]) base
  refs rows = edged "seamRefs" rows
  edged key rows =
    setKey key (toJSON (rows :: [[Integer]])) $
      setKey "seamUnits" (toJSON [[0, 0, 10, 270], [0, 1, 280, 540 :: Integer]]) $
        setKey "seamFiles" (toJSON [[0, 550 :: Integer]]) base
  refused = refusedBy respond

-- | Absence semantics (the divergence precedent): no seamFiles = no
-- split keys at all — an absent advisory must not read as "judged
-- clean and empty".
exactlyWhenSent :: Bool
exactlyWhenSent = case replyObj base of
  Nothing -> False
  Just o ->
    not (KM.member "splitCandidates" o) && not (KM.member "sizeExempt" o)
