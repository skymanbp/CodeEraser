-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | The verdict family's WIRE-driven battery, split from
-- VerdictProps at the 300-line law (ADR-008 P4): everything here
-- drives the REAL CE.Verdict.respond — ablation, idempotence,
-- refusal-by-name, and the knob-table probes (ceilings from the
-- first ADR-008 step; thresholds/tolerance from P4). The pure
-- score/ratchet checks stay in VerdictProps.
module VerdictWireProps (battery) where

import CE.Verdict (respond)
import CE.Verdict.Cost (verdictNodeCap)
import Data.Aeson
import qualified Data.Aeson.Key as Key
import qualified Data.Aeson.KeyMap as KM
import qualified Data.ByteString.Lazy as BL
import Data.List (isInfixOf)

-- | The checks as a TABLE (also what keeps this module's scaffold
-- from cloning VerdictProps' monadic sequence — the batch's own
-- ratchet bite).
battery :: IO Bool
battery = fmap and (mapM one checks)
 where
  checks =
    [ ("ablating any join leg moves the candidate census", ablation)
    , ("ratchet idempotence: newBaseline fed back judges nothing", idempotent)
    , ("refusals name the offender with code and message", refusals)
    , ("ceilings override moves the score and echoes in knobs", ceilingKnob)
    , ("thresholds/tolerance rows move the verdict and echo", thresholdKnob)
    , ("an over-cap request degrades to a reply that FAILS", degradedFails)
    ]
  one (name, ok) = do
    putStrLn ((if ok then "ok   " else "FAIL ") <> name)
    pure ok

-- | A wire request whose candidates cover merge, delete and hotspot;
-- the ablations empty one leg each and the census must move.
wireReq :: [[Integer]] -> [[Integer]] -> [[Integer]] -> [[Integer]] -> Value
wireReq sim pos churn coch =
  object
    [ "proto" .= ("2.2.0" :: String)
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
wChurn = [[4, 30, 10, 0, 0], [5, 30, 10, 0, 0]]
wCoch = [[4, 5, 3]]

replyObj :: Value -> Maybe Object
replyObj r = do
  bytes <- either (const Nothing) Just (respond "0.0.1" (BL.toStrict (encode r)))
  Object o <- decodeStrict bytes
  pure o

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

refusals :: Bool
refusals =
  and
    [ refused (setKey "discrete" (toJSON [9, 7 :: Integer]) base) "not strictly ascending"
    , refused (setKey "tier" (toJSON [[1, 0 :: Integer]]) base) "index mismatch"
    , refused posReq "unit-tier node"
    , refused (setKey "weights" (toJSON [[c, 0] | c <- [0 .. 6 :: Integer]]) base) "every axis zeroed"
    , -- the sim domain pair (M5-close review): an out-of-enum kind
      -- and a zero denominator each refuse by name
      refused (simReq [[0, 1, 3, 50, 100]]) "unknown sim kind"
    , refused (simReq [[0, 1, 0, 50, 0]]) "zero denominator"
    , -- the ceilings table (ADR-008 first step) refuses by name too
      refused (ceilReq [[2, 300]]) "unknown ceiling axis"
    , refused (ceilReq [[0, 0]]) "ceiling below 1"
    , refused (ceilReq [[1, 15], [0, 300]]) "not strictly ascending"
    , -- the P4 tables: each judges by CODE (the generalized grammar)
      refused (thrReq [[7, 1]]) "unknown threshold knob"
    , refused (thrReq [[2, 0]]) "zero denominator"
    , refused (thrReq [[5, 0]]) "knob below 1"
    , refused (tolReq [[3, 1]]) "unknown tolerance leg"
    , refused (tolReq [[1, 0]]) "knob below 1"
    ]
 where
  simReq rows =
    setKey
      "tier"
      (toJSON ([[0, 0], [1, 0]] :: [[Integer]]))
      (setKey "sim" (toJSON (rows :: [[Integer]])) base)
  ceilReq rows = setKey "ceilings" (toJSON (rows :: [[Integer]])) base
  thrReq rows = setKey "thresholds" (toJSON (rows :: [[Integer]])) base
  tolReq rows = setKey "tolerance" (toJSON (rows :: [[Integer]])) base
  base = wireReq [] [] [] []
  posReq =
    setKey
      "tier"
      (toJSON ([[0, 0], [1, 1], [2, 0], [3, 0], [4, 0], [5, 0]] :: [[Integer]]))
      (setKey "pos" (toJSON [[1, 0, 0, 0, 1, 0 :: Integer]]) base)
  refused r want = case respond "0.0.1" (BL.toStrict (encode r)) of
    Left (_, code, msg) -> code == "contract" && want `isInfixOf` msg
    Right _ -> False

setKey :: String -> Value -> Value -> Value
setKey k v (Object o) = Object (KM.insert (Key.fromString k) v o)
setKey _ _ v = v

-- | ADR-008 first step, driven through the REAL respond: a size row
-- at 310 violates the default 300 ceiling and is clean under a
-- requested 400 — the score must move — and the reply echoes the
-- EFFECTIVE knobs both times (the round trip the Rust client
-- asserts; an absent ceilings table echoes the defaults).
ceilingKnob :: Bool
ceilingKnob = case (run Nothing, run (Just [[0, 400]])) of
  (Just (s0, k0), Just (s1, k1)) ->
    s0 /= s1 && k0 == (Number 300, Number 15) && k1 == (Number 400, Number 15)
  _ -> False
 where
  run :: Maybe [[Integer]] -> Maybe (Value, (Value, Value))
  run ceil = do
    o <- replyObj (maybe id (setKey "ceilings" . toJSON) ceil sized)
    s <- KM.lookup "score" o
    (,) s <$> ((,) <$> echo o "sizeCeil" <*> echo o "cocCeil")
  sized = setKey "continuous" (toJSON [[0, 0, 310 :: Integer]]) (wireReq [] [] [] [])

echo :: Object -> String -> Maybe Value
echo o k = do
  Object ks <- KM.lookup "knobs" o
  KM.lookup (Key.fromString k) ks

-- | ADR-008 P4 through the REAL respond: a scoreScale row (code 6)
-- halves the opening value and echoes; a tolerance row zeroing the
-- +abs leg (code 2) turns a within-tolerance ceiling bust (310 over
-- a 300 baseline: tolerated = max(306, 310) = 310) into
-- ratchet.fail (tolerated collapses to 306), with the leg echoed.
thresholdKnob :: Bool
thresholdKnob = scaleMoves && tolFlips
 where
  sized = setKey "continuous" (toJSON [[0, 0, 310 :: Integer]]) (wireReq [] [] [] [])
  scaleMoves = case (run Nothing, run (Just [[6, 500]])) of
    (Just (s0, e0), Just (s1, e1)) ->
      s0 /= s1 && e0 == Number 1000 && e1 == Number 500
    _ -> False
  run :: Maybe [[Integer]] -> Maybe (Value, Value)
  run rows = do
    o <- replyObj (maybe id (setKey "thresholds" . toJSON) rows sized)
    (,) <$> KM.lookup "score" o <*> echo o "scoreScale"
  based =
    setKey
      "baseline"
      (object ["continuous" .= [[0, 0, 300 :: Integer]], "discrete" .= ([] :: [Integer])])
      sized
  tolFlips = case (fails Nothing, fails (Just [[2, 0]])) of
    (Just (False, _), Just (True, e)) -> e == Number 0
    _ -> False
  fails :: Maybe [[Integer]] -> Maybe (Bool, Value)
  fails rows = do
    o <- replyObj (maybe id (setKey "tolerance" . toJSON) rows based)
    Object rat <- KM.lookup "ratchet" o
    Bool f <- KM.lookup "fail" rat
    e <- echo o "tolAbs"
    pure (f, e)

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
