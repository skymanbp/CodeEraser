-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | The rulepack battery (plan v2.13 ①, 3.1.0): the class column on
-- the continuous rows and the classKnobs table, in two halves. The
-- PURE half drives Score.penalties — C2 a class's own line moves its
-- rows' charge, C4 a missing row falls back to the global line, C9
-- the table is a Map (row order never judges). The WIRE half drives
-- the REAL respond — the boundary refusals C5–C7 (mixed arity, a
-- class at the fence, class 0 in the knob table), the knob-row
-- grammar, the three-column baseline law, and the echo that rides
-- exactly when the rows did.
module ClassProps (battery) where

import CE.Verdict (respond)
import CE.Verdict.Cost (classCap)
import CE.Verdict.Score (Facts (..), classKnobsOf, penalties, scoreBound)
import Data.Aeson
import qualified Data.Aeson.Key as Key
import qualified Data.Aeson.KeyMap as KM
import VerdictWireProps (replyObj, wireReq)
import WireHarness (field, refusedBy, runChecks, setKey)

battery :: IO Bool
battery = do
  pureHalf <- runChecks (zip pureNames [classLine, classCoc, classHard, fallback, permutation])
  wireHalf <- runChecks (zip wireNames [mixedArity, beyondFence, classZero, knobShape, threeColumnLaw, echo])
  fence <- runChecks (zip fenceNames [fenceAbsent, fenceDrifts, fenceRecorded, classTolBinds])
  pure (pureHalf && wireHalf && fence)
 where
  pureNames =
    [ "C2: a class's own warn line clears its rows' size charge"
    , "C2: a class's own coc ceiling spares its functions"
    , "C2: a class's own hard line rescales its zone"
    , "C4: a class without a row falls back to the global line"
    , "C9: the class table is a Map — row order never judges"
    ]
  wireNames =
    [ "C5: a continuous table mixing arities refuses"
    , "C6: a class at the fence refuses"
    , "C7: class 0 in the knob table refuses"
    , "the knob table refuses an unknown code, a zero value, a disorder"
    , "the class column never reaches the baseline"
    , "the class rows echo exactly when they rode"
    ]
  fenceNames =
    [ "K11: a default config sends no digest — absent, not null, in the newBaseline"
    , "K12: a changed KNOB fails BY NAME, not by silent relaxation"
    , "K13: establish records the digest, and the baseline stays three columns"
    , "K14: a class tolerance of 0 refuses one line, and the global legs do not save it"
    ]

-- | Size and complexity facts alone: classed rows and the knob rows,
-- folded the way CE.Verdict folds them.
contFacts :: [[Integer]] -> [[Integer]] -> Facts
contFacts rows knobs = Facts [] [] [] rows [] (classKnobsOf knobs)

axis :: Integer -> Facts -> Integer
axis code f = maybe (-1) id (lookup code (penalties scoreBound Nothing f))

-- | One classed row over the global line charges its axis; the same
-- row under its class's own knob charges nothing.
spares :: Integer -> [Integer] -> [Integer] -> Bool
spares code row knob =
  axis code (contFacts [row] [knob]) == 0 && axis code (contFacts [row] []) > 0

-- | 350 lines past the fallback opening edge 300, class line 400.
classLine :: Bool
classLine = spares 0 [0, 0, 350, 1] [1, 0, 400]

-- | CoC 18 over the global 15, class ceiling 20.
classCoc :: Bool
classCoc = spares 1 [0, 1, 18, 1] [1, 1, 20]

-- | A farther hard line flattens the zone: the 600-line file charges
-- less under a class H of 900 than under the global 750.
classHard :: Bool
classHard =
  axis 0 (contFacts [[0, 0, 600, 1]] [[1, 2, 900]])
    < axis 0 (contFacts [[0, 0, 600, 1]] [])

-- | A row for ANOTHER class changes nothing for this one, and a
-- classed row with no override judges exactly like the legacy
-- three-column row it shadows.
fallback :: Bool
fallback =
  axis 0 (contFacts [[0, 0, 350, 1]] [[2, 0, 400]]) == unclassed
    && axis 0 (contFacts [[0, 0, 350, 1]] []) == unclassed
 where
  unclassed = axis 0 (contFacts [[0, 0, 350]] [])

permutation :: Bool
permutation =
  penalties scoreBound Nothing (contFacts rows [[1, 0, 400], [1, 1, 20], [2, 0, 350]])
    == penalties scoreBound Nothing (contFacts rows [[2, 0, 350], [1, 1, 20], [1, 0, 400]])
 where
  rows = [[0, 0, 350, 1], [1, 1, 18, 1], [2, 0, 340, 2]]

base :: Value
base = wireReq [] [] [] []

withCont :: [[Integer]] -> Value
withCont rows = setKey "continuous" (toJSON rows) base

refused :: Value -> String -> Bool
refused = refusedBy respond

mixedArity :: Bool
mixedArity = refused (withCont [[0, 0, 310], [1, 1, 20, 0]]) "mixed arity"

beyondFence :: Bool
beyondFence = refused (withCont [[0, 0, 310, classCap]]) "class beyond the fence"

classZero :: Bool
classZero = refused (setKey "classKnobs" (toJSON [[0, 0, 400 :: Integer]]) base) "class 0"

knobShape :: Bool
knobShape =
  and
    [ refused (knobs [[1, 4, 5]]) "unknown class knob code"
    , refused (knobs [[1, 0, 0]]) "knob below 1"
    , refused (knobs [[2, 0, 5], [1, 0, 5]]) "not strictly ascending"
    , refused (knobs [[classCap, 0, 5]]) "class beyond the fence"
    , -- code 3 is the ratchet allowance and ZERO is its whole point:
      -- the per-code bound is what lets a line keep its floor of 1
      -- while an allowance may be nothing at all
      not (refused (knobs [[1, 3, 0]]) "knob below")
    ]
 where
  knobs rows = setKey "classKnobs" (toJSON (rows :: [[Integer]])) base

-- | One key out of the reply's newBaseline object.
newBaselineKey :: Value -> String -> Maybe Value
newBaselineKey req k = case replyObj req of
  Just o -> case field o "newBaseline" of
    Just (Object nb) -> KM.lookup (Key.fromString k) nb
    _ -> Nothing
  Nothing -> Nothing

-- | The reply's HELD fail-condition names.
heldConditions :: Value -> Maybe Value
heldConditions req = case replyObj req of
  Just o -> case field o "ratchet" of
    Just (Object rt) -> KM.lookup "failed" rt
    _ -> Nothing
  Nothing -> Nothing

-- | K11: a repo whose config is the shipped default sends no digest,
-- and the newBaseline it gets back does not carry the key at all.
-- Absent, not null: a key holding null is a byte a pre-fence baseline
-- file never had, and this fence is not allowed to cost such a repo
-- one byte.
fenceAbsent :: Bool
fenceAbsent =
  newBaselineKey base "knobsDigest" == Nothing
    && newBaselineKey (setKey "knobsDigest" Null base) "knobsDigest" == Nothing

-- | K12: the ceilings were established under one knob set and this
-- run declares another. The point is not that it fails — it is that
-- it fails BY NAME with an EMPTY over table, so the operator learns
-- which floor to re-name instead of watching every gate quietly move.
-- The scope this pins is the whole config since 6.0.0, not the class
-- table alone: `[score] viol_cost = 0` reached 1000/1000 through the
-- narrow version of this fence and was measured doing it.
fenceDrifts :: Bool
fenceDrifts =
  heldConditions drifted == Just (toJSON (["knobs_digest"] :: [String]))
    && heldConditions agreed == Just (toJSON ([] :: [String]))
 where
  established d = setKey "baseline" (object ["continuous" .= ([] :: [Value]), "discrete" .= ([] :: [Integer]), "knobsDigest" .= (d :: Integer)])
  drifted = established 111 (setKey "knobsDigest" (toJSON (222 :: Integer)) base)
  agreed = established 111 (setKey "knobsDigest" (toJSON (111 :: Integer)) base)

-- | K13: establish (no baseline) records the digest this run declares
-- — the named act — while the ratchet rows it writes stay three
-- columns, because a knob is a charging parameter and never a fact
-- about the tree.
fenceRecorded :: Bool
fenceRecorded =
  newBaselineKey req "knobsDigest" == Just (toJSON (777 :: Integer))
    && newBaselineKey req "continuous" == Just (toJSON [[0, 0, 310 :: Integer]])
 where
  req = setKey "knobsDigest" (toJSON (777 :: Integer)) (withCont [[0, 0, 310, 1]])

-- | K14: class 1 declares a zero allowance and grows by one line —
-- over. Class 2 declares none and grows by the same line — inside the
-- global +10 leg, so merely drawn. The global legs do not rescue
-- class 1, because a declared allowance replaces them rather than
-- joining them.
classTolBinds :: Bool
classTolBinds = over == Just (toJSON [[0, 0, 301, 300 :: Integer]]) && drawn == Just (toJSON ([] :: [Value]))
 where
  req cls =
    setKey "classKnobs" (toJSON [[1, 3, 0 :: Integer]]) $
      setKey "baseline" (object ["continuous" .= [[0, 0, 300 :: Integer]], "discrete" .= ([] :: [Integer])]) $
        withCont [[0, 0, 301, cls]]
  ratchetKey r k = case replyObj r of
    Just o -> case field o "ratchet" of
      Just (Object rt) -> KM.lookup (Key.fromString k) rt
      _ -> Nothing
    Nothing -> Nothing
  over = ratchetKey (req 1) "over"
  drawn = ratchetKey (req 2) "over"

-- | A classed request with no baseline establishes: the newBaseline
-- it hands back carries the three-column prefix alone.
threeColumnLaw :: Bool
threeColumnLaw = case replyObj (withCont [[0, 0, 310, 1], [1, 1, 20, 1]]) of
  Just o -> do
    let cont = do
          Object nb <- field o "newBaseline"
          KM.lookup "continuous" nb
    cont == Just (toJSON [[0, 0, 310], [1, 1, 20 :: Integer]])
  Nothing -> False

echo :: Bool
echo =
  (replyObj classed >>= \o -> field o "classKnobs") == Just (toJSON rows)
    && (replyObj (withCont [[0, 0, 310, 1]]) >>= \o -> field o "classKnobs") == Nothing
 where
  rows = [[1, 0, 400 :: Integer]]
  classed = setKey "classKnobs" (toJSON rows) (withCont [[0, 0, 310, 1]])
