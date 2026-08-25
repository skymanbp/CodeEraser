-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | The verdict family's WIRE-driven battery, split from
-- VerdictProps at the 300-line law (ADR-008 P4): everything here
-- drives the REAL CE.Verdict.respond — ablation, idempotence,
-- refusal-by-name, and the knob-table probes (ceilings from the
-- first ADR-008 step; thresholds/tolerance from P4). The pure
-- score/ratchet checks stay in VerdictProps; the scaffold lives in
-- WireHarness (the P3 tenth-bite repayment).
module VerdictWireProps (battery, wireReq, replyObj) where

import CE.Verdict (respond)
import CE.Verdict.Cost (verdictNodeCap)
import Data.Aeson
import qualified Data.Aeson.Key as Key
import qualified Data.Aeson.KeyMap as KM
import WireHarness (refusedBy, replyObjWith, runChecks, setKey)

battery :: IO Bool
battery =
  runChecks
    [ ("ablating any join leg moves the candidate census", ablation)
    , ("ratchet idempotence: newBaseline fed back judges nothing", idempotent)
    , ("refusals name the offender with code and message", refusals)
    , ("an over-cap request degrades to a reply that FAILS", degradedFails)
    ]

-- | A wire request whose candidates cover merge, delete and hotspot;
-- the ablations empty one leg each and the census must move.
wireReq :: [[Integer]] -> [[Integer]] -> [[Integer]] -> [[Integer]] -> Value
wireReq sim pos churn coch =
  object
    [ "proto" .= ("5.1.0" :: String)
    , "type" .= ("verdict.request" :: String)
    , "id" .= (1 :: Int)
    , "sim" .= sim
    , "pos" .= pos
    , "tier" .= [[u, 0] | u <- [0 .. 5 :: Int]]
    , "churn" .= churn
    , "cochange" .= coch
    , "continuous" .= ([] :: [Value])
    , "discrete" .= ([] :: [Integer])
    , "baseline" .= Null
    , "weights" .= ([] :: [Value])
    , "floor" .= Null
    ]

wSim, wPos, wChurn, wCoch :: [[Integer]]
wSim = [[0, 1, 1, 90, 100], [2, 3, 1, 90, 100], [4, 5, 2, 85, 100]]
wPos =
  [ [0, 1, 0, 0, 1, 1]
  , [1, 2, 0, 1, 1, 1]
  , [2, 0, 0, 2, 1, 0]
  , [3, 1, 0, 3, 1, 1]
  , [4, 1, 0, 4, 1, 1]
  , [5, 1, 0, 4, 1, 1]
  ]
wChurn = [[4, 30, 10], [5, 30, 10]]
wCoch = [[4, 5, 3]]

replyObj :: Value -> Maybe Object
replyObj = replyObjWith respond

candidateCensus :: Value -> Maybe [Int]
candidateCensus req = do
  o <- replyObj req
  Array cands <- KM.lookup "candidates" o
  rows <- mapM verdictOfRow (foldr (:) [] cands)
  pure [length [() | v <- rows, v == c] | c <- [0 .. 3]]
 where
  verdictOfRow v = case fromJSON v :: Result [Integer] of
    Success (_ : _ : code : _) -> Just code
    _ -> Nothing

ablation :: Bool
ablation = case candidateCensus (wireReq wSim wPos wChurn wCoch) of
  Nothing -> False
  Just full ->
    drop 1 full == [1, 1, 1] -- one merge, one delete, one hotspot
      && all
        (moved full)
        [ wireReq [[u, v, k, 0, 100] | [u, v, k, _, _] <- wSim] wPos wChurn wCoch
        , wireReq wSim [] wChurn wCoch
        , wireReq wSim wPos [] []
        ]
 where
  moved full v = case candidateCensus v of
    Just c -> c /= full
    Nothing -> False

-- | Round trip: judge with no baseline, feed the returned
-- newBaseline into the SAME facts — the ratchet must find nothing.
idempotent :: Bool
idempotent = case reply (req Null) >>= KM.lookup "newBaseline" of
  Nothing -> False
  Just nb -> case reply (req nb) of
    Just o2 ->
      field o2 "added" == Just (Array mempty)
        && field o2 "over" == Just (Array mempty)
        && field o2 "toleranceDrawn" == Just (Array mempty)
        && field o2 "fail" == Just (Bool False)
    Nothing -> False
 where
  req base =
    setKey "id" (toJSON (2 :: Int)) $
      setKey "continuous" (toJSON [[0, 0, 310 :: Integer], [1, 1, 20], [2, 0, 50]]) $
        setKey "discrete" (toJSON [3, 7 :: Integer]) $
          setKey "baseline" base (wireReq [] [] [] [])
  reply r = replyObj (setKey "tier" (toJSON ([[0, 0], [1, 0], [2, 0]] :: [[Integer]])) r)
  field o sub = do
    Object inner <- KM.lookup "ratchet" o
    KM.lookup (Key.fromString sub) inner

-- | The knob families' by-CODE refusals — ceilings (the ADR-008
-- first step) and the P4 threshold/tolerance tables. Every probe is
-- ONE motion: override the key with a single [[code, value]] row and
-- expect a named refusal, so the rows are data — spelled out as
-- calls they were three pasted stanzas the dedup gate counted as
-- clones. Two rows carry their own reason: 99 is the stably unknown
-- ceiling axis (the StructureProps lesson — 2.14.0 made the old
-- max+1 code legal, and a moving boundary must not be frozen here),
-- and code 4 is an EXPONENT reaching `spread ^ k` in CE.Verdict.Soft
-- before the clamp, where an unbounded value kills the process on
-- allocation instead of answering.
knobProbes :: [(String, Integer, Integer, String)]
knobProbes =
  [ ("ceilings", 99, 300, "unknown ceiling axis")
  , ("ceilings", 0, 0, "ceiling below 1")
  , ("ceilings", 4, 1000000000, "soft-line exponent above")
  , ("thresholds", 7, 1, "unknown threshold knob")
  , ("thresholds", 2, 0, "zero denominator")
  , ("thresholds", 5, 0, "knob below 1")
  , ("tolerance", 3, 1, "unknown tolerance leg")
  , ("tolerance", 1, 0, "knob below 1")
  ]

refusals :: Bool
refusals = and (map knob knobProbes <> shaped)
 where
  knob (k, code, v, want) = refused (setKey k (toJSON [[code, v]]) base) want
  -- the probes whose payload is not one knob row: a shape each
  shaped =
    [ refused (setKey "discrete" (toJSON [9, 7 :: Integer]) base) "not strictly ascending"
    , refused (setKey "tier" (toJSON [[1, 0 :: Integer]]) base) "index mismatch"
    , refused posReq "unit-tier node"
    , refused (setKey "weights" (toJSON [[c, 0] | c <- [0 .. 6 :: Integer]]) base) "every axis zeroed"
    , -- the sim domain pair (M5-close review): an out-of-enum kind
      -- and a zero denominator each refuse by name
      refused (simReq [[0, 1, 3, 50, 100]]) "unknown sim kind"
    , refused (simReq [[0, 1, 0, 50, 0]]) "zero denominator"
    , -- the ascending law is over the ceilings TABLE, not over one row
      refused (setKey "ceilings" (toJSON [[1, 15], [0, 300 :: Integer]]) base) "not strictly ascending"
    , -- the 2.14.0 judgedLoc multiset: non-descending, u64 values
      refused (setKey "judgedLoc" (toJSON [300, 200 :: Integer]) base) "not non-descending"
    , -- the P2 dedup pair refuses by name too — a malformed pair
      -- must never read as "under budget"
      refused (setKey "dedup" (toJSON [1 :: Integer]) base) "malformed pair"
    , refused (setKey "dedup" (toJSON [-1, 5 :: Integer]) base) "negative field"
    , refused (setKey "judgedLoc" (toJSON [-1 :: Integer]) base) "outside u64"
    , -- a stored softLine of 0 is no line at all — refused by name
      refused (setKey "baseline" storedZero base) "outside 1..u64"
    ]
  storedZero =
    object
      [ "continuous" .= ([] :: [Value])
      , "discrete" .= ([] :: [Integer])
      , "softLine" .= (0 :: Integer)
      ]
  simReq rows =
    setKey
      "tier"
      (toJSON ([[0, 0], [1, 0]] :: [[Integer]]))
      (setKey "sim" (toJSON (rows :: [[Integer]])) base)
  base = wireReq [] [] [] []
  posReq =
    setKey
      "tier"
      (toJSON ([[0, 0], [1, 1], [2, 0], [3, 0], [4, 0], [5, 0]] :: [[Integer]]))
      (setKey "pos" (toJSON [[1, 0, 0, 0, 1, 0 :: Integer]]) base)
  refused = refusedBy respond

-- | ADR-008 P1 through the REAL respond: one node past the cap
-- degrades to a complete reply whose ratchet.fail is TRUE — a gate
-- that could not judge must never pass, said by the CORE; the Rust
-- side relays the bit, never re-derives the rule.
degradedFails :: Bool
degradedFails = case replyObj overCap of
  Nothing -> False
  Just o ->
    KM.lookup "degraded" o == Just (Bool True)
      && ( do
             Object rat <- KM.lookup "ratchet" o
             KM.lookup "fail" rat
         )
        == Just (Bool True)
 where
  overCap =
    setKey "tier" (toJSON [[u, 0] | u <- [0 .. verdictNodeCap]]) (wireReq [] [] [] [])
