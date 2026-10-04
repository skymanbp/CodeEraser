-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | The candidates family (plan v2.33 W3): the two bounds sit exactly
-- on the 85/100 boundary in both directions and each owns its tally
-- (the cases the measuring side's own battery held before the bounds
-- moved here), S2 pairs one key across files within a language and
-- merges its bit into a sent pair, S5 windows, cuts and never
-- duplicates, the floor-aware intersection agrees with the bound, two
-- hundred seeded requests agree with ReferenceCandidates, the
-- contract's refusals agree with the reference's predicates, and both
-- caps hold.
module CandidatesProps (battery) where

import CE.Candidates (respond)
import CE.Candidates.Contract (CandReq (..), overCap)
import CE.Candidates.Cost (candidatePairCap, candidateUnitCap)
import CE.Candidates.Units (columns, interUpTo)
import CE.Clone.Prefilter (reachFloor, sizeBelow)
import Data.Aeson (Value (..), object, toJSON, (.=))
import qualified Data.Aeson.KeyMap as KM
import ReferenceCandidates (expected, requests)
import ReferenceContract (answers, cells)
import WireHarness (fieldsOf, runLegs)

battery :: IO Bool
battery =
  runLegs
    [ "the bounds sit on the 85/100 boundary and each owns its tally"
    , "S2 pairs one key across files within a language and merges its bit into a sent pair"
    , "S5 windows by size, cuts by label, never duplicates, and its pairs carry bit 4 alone"
    , "reachFloor is the size bound's threshold and the floor-aware walk decides the label bound alike"
    , "two hundred seeded requests agree with the reference on pairs, S2 counts, counts and degradation"
    , "the seeded requests are not vacuous: both bounds cut, S2 crosses languages, S5 finds, cuts and repeats"
    , "the contract's refusals agree with the reference's predicates"
    , "the unit cap holds on both sides and the pair cap is counted"
    ]
    [ all boundary boundaries
    , s2Case
    , s5Case
    , floorLaw && floorWalk
    , all agrees requests
    , nonVacuous
    , all agrees refusals
    , unitCap && pairCapCounted
    ]

request :: Bool -> [[Integer]] -> [[Integer]] -> Value
request ex units pairs =
  object
    [ "proto" .= ("7.0.0" :: String)
    , "type" .= ("candidates.request" :: String)
    , "id" .= (1 :: Int)
    , "units" .= units
    , "pairs" .= pairs
    , "exhaustive" .= ex
    ]

agrees :: (Bool, [[Integer]], [[Integer]]) -> Bool
agrees (ex, units, pairs) =
  answers respond ["pairs", "keyPairs", "counts", "degraded", "reason"] (request ex units pairs) (expected ex units pairs)

-- | The eleven counts in reply order.
countsOf :: [Integer] -> Value
countsOf ns = object (zipWith (.=) names ns)
 where
  names = ["units", "pairs", "union", "crossLanguage", "prunedSize", "prunedLabel", "survivors", "s5Windowed", "s5PrunedLabel", "s5Already", "s5New"]

-- | The reply's pairs, S2 counts and counts for one request.
replied :: Bool -> [[Integer]] -> [[Integer]] -> Maybe [Maybe Value]
replied ex units pairs = fieldsOf respond (request ex units pairs) ["pairs", "keyPairs", "counts"]

rows :: [[Integer]] -> Maybe Value
rows = Just . toJSON

-- | Rows 1–2: the size bound at 85 vs 84 against 100; rows 3–4: equal
-- sizes, the label multisets sharing 84 (cut) vs 85 (kept) at 100.
-- Each tally is [prunedSize, prunedLabel, survivors]; two files, two
-- keys, so S2 adds nothing.
boundaries :: [([[Integer]], [Integer])]
boundaries =
  [ ([[0, 0, 0, 85, 1, 85], [0, 1, 1, 100, 1, 100]], [0, 0, 1])
  , ([[0, 0, 0, 84, 1, 84], [0, 1, 1, 100, 1, 100]], [1, 0, 0])
  , ([[0, 0, 0, 100, 1, 84, 2, 16], [0, 1, 1, 100, 1, 100]], [0, 1, 0])
  , ([[0, 0, 0, 100, 1, 85, 2, 15], [0, 1, 1, 100, 1, 100]], [0, 0, 1])
  ]

boundary :: ([[Integer]], [Integer]) -> Bool
boundary (units, t) = fmap (drop 2) (replied False units [[0, 1, 1]]) == Just [Just (countsOf ([2, 1, 1, 0] <> t <> [0, 0, 0, 0]))]

-- | Three units of one key: two in different files of one language
-- (an S2 pair), the third in another language (two crossings); then
-- the same S2 pair also sent by S3: one union pair carrying both bits.
s2Case :: Bool
s2Case =
  replied False three [] == Just [rows [[0, 1, 2]], rows [[0, 1]], Just (countsOf [3, 0, 1, 2, 0, 0, 1, 0, 0, 0, 0])]
    && replied False three [[0, 1, 4]] == Just [rows [[0, 1, 6]], rows [[0, 1]], Just (countsOf [3, 1, 1, 2, 0, 0, 1, 0, 0, 0, 0])]
 where
  three = [[0, 0, 5, 100, 1, 100], [0, 1, 5, 100, 1, 100], [1, 2, 5, 100, 1, 100]]

-- | Four units: three of 100 nodes (two sharing every kind, one
-- sharing none), one of 84 outside every window; the sent union holds
-- (0,1), which S5 counts as already kept and never repeats. Then two
-- units of 100 and 90 sharing everything and nothing sent: the new
-- pair lands with bit 4 alone.
s5Case :: Bool
s5Case =
  replied True four [[0, 1, 1]] == Just [rows [[0, 1, 1]], rows [], Just (countsOf [4, 1, 1, 0, 0, 0, 1, 3, 2, 1, 0])]
    && replied True two [] == Just [rows [[0, 1, 16]], rows [], Just (countsOf [2, 0, 0, 0, 0, 0, 0, 1, 0, 0, 1])]
 where
  four = [[0, 0, 0, 100, 1, 100], [0, 1, 1, 100, 1, 100], [0, 2, 2, 100, 2, 100], [0, 3, 3, 84, 1, 84]]
  two = [[0, 0, 0, 100, 1, 100], [0, 1, 1, 90, 1, 90]]

-- | sizeBelow q mx ⇔ q < reachFloor mx over every q, mx in 0..300.
floorLaw :: Bool
floorLaw = and [sizeBelow q mx == (q < reachFloor mx) | mx <- [0 .. 300 :: Int], q <- [0 .. 300]]

-- | Over every pair of the seeded requests' units and every floor up to
-- the larger size: the floor-aware walk is below the floor exactly when
-- the full intersection is (the full one is the walk with floor 0).
floorWalk :: Bool
floorWalk =
  and
    [ (interUpTo us f a b < f) == (interUpTo us 0 a b < f)
    | (_, units, _) <- take 60 requests
    , let us = columns units
    , let n = length units
    , a <- [0 .. n - 1]
    , b <- [0 .. n - 1]
    , f <- [0 .. 70]
    ]

nonVacuous :: Bool
nonVacuous = all (\k -> any (positive k) requests) ["prunedSize", "prunedLabel", "crossLanguage", "s5New", "s5PrunedLabel", "s5Already"]
 where
  positive k (ex, us, ps) = case expected ex us ps of
    Right [_, _, Just (Object c), _, _] -> maybe False (/= Number 0) (KM.lookup k c)
    _ -> False

refusals :: [(Bool, [[Integer]], [[Integer]])]
refusals = [(False, read u, read p) | [u, p] <- cells (unlines table)]
 where
  two = "[[0,0,0,1,0,1],[0,1,1,1,0,1]]"
  table =
    [ "[[-1,0,0,1,0,1]] ; []"
    , "[[0,-1,0,1,0,1]] ; []"
    , "[[0,0,-1,1,0,1]] ; []"
    , "[[0,0,0,0]] ; []"
    , "[[0,0,0,2,0]] ; []"
    , "[[0,0,0,2,-1,2]] ; []"
    , "[[0,0,0,2,0,0,1,2]] ; []"
    , "[[0,0,0,2,1,1,0,1]] ; []"
    , "[[0,0,0,3,0,1,1,1]] ; []"
    , "[[0,0,0]] ; []"
    , two <> " ; [[0,2,1]]"
    , two <> " ; [[1,1,1]]"
    , "[[0,0,0,1,0,1],[1,1,1,1,0,1]] ; [[0,1,1]]"
    , two <> " ; [[0,1,0]]"
    , two <> " ; [[0,1,2]]"
    , two <> " ; [[0,1]]"
    , "[[0,0,0,1,0,1],[0,1,1,1,0,1],[0,2,2,1,0,1]] ; [[0,2,1],[0,1,1]]"
    ]

-- | At the unit cap exactly the core judges; one past it the reply is
-- the reference's degraded one. One key per unit, so S2 pairs nothing
-- (the reference's comprehensions are quadratic: it answers only the
-- degraded side, which it decides by length alone).
unitCap :: Bool
unitCap =
  fieldsOf respond (request False (atCap 0) []) ["degraded"] == Just [Just (Bool False)]
    && agrees (False, atCap 1, [])
 where
  atCap extra = [[0, 0, k, 1, 0, 1] | k <- [1 .. candidateUnitCap + extra]]

-- | The pair cap, counted without a four-million-row request on the
-- wire: the contract's own overCap at the cap and one past it.
pairCapCounted :: Bool
pairCapCounted =
  not (overCap (at candidatePairCap)) && overCap (at (candidatePairCap + 1))
 where
  at n = CandReq Null [] (replicate (fromInteger n) []) False
