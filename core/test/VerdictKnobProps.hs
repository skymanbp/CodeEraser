-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | The verdict family's KNOB-probe battery, split from
-- VerdictWireProps when the 2.14.0 soft-zone probes pushed it to
-- the 300-line wall: every check here perturbs one knob table row
-- through the REAL respond and watches the score/echo move —
-- ceilings (with the v0.6 zone codes), thresholds/tolerance, the
-- dedup budget pair, and the soft-line establish/carry round trip.
module VerdictKnobProps (battery) where

import Data.Aeson
import qualified Data.Aeson.Key as Key
import qualified Data.Aeson.KeyMap as KM
import VerdictWireProps (replyObj, wireReq)
import WireHarness (runChecks, setKey)

battery :: IO Bool
battery =
  runChecks
    [ ("ceilings override moves the score and echoes in knobs", ceilingKnob)
    , ("thresholds/tolerance rows move the verdict and echo", thresholdKnob)
    , ("the dedup pair fails over budget and passes at it", dedupBudget)
    , ("the soft line derives at establish, carries after, and judges", softLineWire)
    ]

-- | ADR-008 first step, driven through the REAL respond: a size row
-- at 550 charges the v0.6 zone floor(10·(250/450)²)=3 under the
-- default soft fallback 300 and floor(10·(150/350)²)=1 under a
-- requested 400 — the score must move — and the reply echoes the
-- EFFECTIVE knobs both times (the round trip the Rust client
-- asserts; an absent ceilings table echoes the defaults). The old
-- 310 probe died with the binary axis: its graded penalty rounds
-- to zero on both sides.
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
  sized = setKey "continuous" (toJSON [[0, 0, 550 :: Integer]]) (wireReq [] [] [] [])

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

-- | ADR-008 P2 through the REAL respond: the [blocks, budget] pair
-- answers through the fail bit — one over budget fails, at budget
-- passes; the absent case is every OTHER check in this battery
-- (none sends the pair, none fails on it).
dedupBudget :: Bool
dedupBudget = failAt [146, 145] == Just True && failAt [145, 145] == Just False
 where
  failAt :: [Integer] -> Maybe Bool
  failAt pair = do
    o <- replyObj (setKey "dedup" (toJSON pair) (wireReq [] [] [] []))
    Object rat <- KM.lookup "ratchet" o
    Bool f <- KM.lookup "fail" rat
    pure f

-- | plan v2.6 §B through the REAL respond: at establish the
-- judgedLoc multiset {240,300,375} derives softLine 468 (the
-- VerdictProps hand computation) and a 550-line file charges
-- floor(10·(82/282)²)=0 under S=468 vs 3 under the fallback 300;
-- feeding the newBaseline back WITHOUT judgedLoc keeps S=468 — the
-- carry, not a re-derivation.
softLineWire :: Bool
softLineWire = case (bare, established) of
  (Just (s0, Null), Just (s1, Number 468)) ->
    s0 /= s1 && (carried >>= soft) == Just (Number 468) && atTheCap
  _ -> False
 where
  sized = setKey "continuous" (toJSON [[0, 0, 550 :: Integer]]) (wireReq [] [] [] [])
  locReq = setKey "judgedLoc" (toJSON [240, 300, 375 :: Integer]) sized
  run req = do
    o <- replyObj req
    s <- KM.lookup "score" o
    sl <- soft o
    pure (s, sl)
  bare = run sized
  established = run locReq
  carried = do
    o <- replyObj locReq
    nb <- KM.lookup "newBaseline" o
    replyObj (setKey "baseline" nb sized)
  soft o = do
    Object nb <- KM.lookup "newBaseline" o
    KM.lookup "softLine" nb
  -- the live side of the code-4 allocation fence (VerdictWireProps
  -- refuses 1e9): AT the cap the exponent must still set the line —
  -- (5/4)^1024 saturates the [200,500] clamp — so a cap trimmed too
  -- far shows up here as a refusal, not as a quietly narrowed knob
  atTheCap =
    (replyObj (setKey "ceilings" (toJSON [[4, 1024 :: Integer]]) locReq) >>= soft)
      == Just (Number 500)

