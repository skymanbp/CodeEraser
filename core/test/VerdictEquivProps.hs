-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | The verdict family against its second implementation (plan v2.32
-- step 7): the soft line over two hundred LOC distributions, the
-- single-edit allowance over an exhaustive grid, the
-- ratchet over two hundred baseline cases, the join lattice and its
-- confidence over every leg combination, and the weight echo of two
-- hundred real verdict/1 replies — plus the row cap and the sim
-- contract through the real respond. The seven density charges and
-- their fold retired with that law (10.0.0); the extent law's own
-- reference battery is VerdictExtentProps, and the zone curve went
-- with the split advisory (SplitZoneProps).
module VerdictEquivProps (battery) where

import CE.Verdict (respond)
import CE.Verdict.Cost (defaultWeight, softLineK, softMax, softMin, verdictRowCap)
import CE.Verdict.Join (bound, confidence, judge)
import CE.Verdict.Ratchet (Baseline (..), Ratcheted (..), ratchet, ratchetBound, tolerated)
import CE.Verdict.Soft (softLine)
import Data.Aeson (Value (..), object, toJSON, (.=))
import qualified Data.Map.Strict as M
import ReferenceRatchet (refConfidence, refJudge, refRatchet, refTolerated)
import ReferenceScore (refSoftLine)
import VerdictGen (WireCase, joinFamily, locCases, ratchetCases, wireCases)
import WireHarness (fieldsOf, refusedBy, runLegs, setKey)

battery :: IO Bool
battery =
  runLegs
    [ "two hundred LOC distributions: the soft line agrees, clamped and unclamped"
    , "the single-edit allowance agrees on every ceiling to 1200, global and declared"
    , "two hundred ratchet cases agree on over, drawn, added, removed, dropped and the new baseline"
    , "the join lattice agrees on every leg combination, verdict, legs, reasons and confidence"
    , "two hundred verdict/1 replies echo the weight table and carry seven axis rows"
    , "past the row cap the reply degrades; a descending sim pair and a repeated (u, v, kind) are refused by name; one pair under two families is judged"
    ]
    [ and [softLine kk lo hi locs == refSoftLine kk lo hi locs | locs <- locCases, (kk, lo, hi) <- [(softLineK, softMin, softMax), (1, 1, 100000)]]
    , and [tolerated ratchetBound m d c == refTolerated ratchetBound m d c | c <- [0 .. 1200], d <- [Nothing, Just 0, Just 7], m <- [0, 1]]
    , all ratchets ratchetCases
    , and [judge bound l == refJudge bound l && confidence mask bits == refConfidence bound l | l <- joinFamily, let (_, mask, bits) = judge bound l]
    , all wired wireCases
    , capAndRefusal
    ]

ratchets :: ([[Integer]], Maybe [Integer], Maybe ([[Integer]], [Integer]), [[Integer]], [Integer]) -> Bool
ratchets (classRows, present, base, cont, disc) =
  (rOver r, rDrawn r, rAdded r, rRemoved r, rDropped r, rNewCont r, rNewDisc r) == refRatchet ratchetBound classRows present base (cont, disc)
 where
  r = ratchet ratchetBound classTol present ((\(c, d) -> Baseline c d Nothing Nothing) <$> base) cont disc
  table = M.fromList [((c, code), v) | [c, code, v] <- classRows]
  classTol c metric = case (if metric == 1 then M.lookup (c, 4) table else Nothing) of
    Just v -> Just v
    Nothing -> M.lookup (c, 3) table

request :: WireCase -> Value
request (sim, pos, churn, cont, classes, locs, docs, weights, n) =
  object
    [ "proto" .= ("7.0.0" :: String)
    , "type" .= ("verdict.request" :: String)
    , "id" .= (1 :: Int)
    , "sim" .= sim
    , "pos" .= pos
    , "tier" .= [[u, 0] | u <- [0 .. n - 1]]
    , "churn" .= churn
    , "cochange" .= ([] :: [Value])
    , "continuous" .= cont
    , "discrete" .= ([] :: [Integer])
    , "baseline" .= Null
    , "weights" .= weights
    , "floor" .= Null
    , "classKnobs" .= classes
    , "judgedLoc" .= locs
    , "docFiles" .= docs
      -- the extent law's tables (10.0.0): every continuous row outside
      -- the universe, a function row on one line; every tier file one line
    , "contSpans" .= [if take 1 (drop 1 r) == [1] then [-1, 1, 1] else [-1, 0, 0 :: Integer] | r <- cont]
    , "dupSpans" .= ([] :: [Value])
    , "refLines" .= ([] :: [Value])
    , "proseLines" .= [[u, 1] | u <- [0 .. n - 1]]
    ]

-- | Every reply echoes the effective weight table and seven axis rows:
-- a named axis at its weight, an unnamed one at defaultWeight - except
-- the dead axis (4), which since the dead-file amendment (v2.35) weighs
-- 0 until a request names it.
wired :: WireCase -> Bool
wired c@(_, _, _, _, _, _, _, weights, _) = case fieldsOf respond (request c) ["axes", "weights"] of
  Just [Just (Array axes), w] -> length axes == 7 && w == Just (toJSON [[a, maybe (unnamed a) id (lookup a [(x, w') | [x, w'] <- weights])] | a <- [0 .. 6 :: Integer]])
  _ -> False
 where
  unnamed a = if a == 4 then 0 else defaultWeight

-- | The reference's own reading of the boundaries: one discrete entry
-- past verdictRowCap degrades, a sim pair whose endpoints descend is
-- the contract's to refuse — and since 9.1.0 the sim identity is
-- (u, v, kind): one pair under two families is a request, the same
-- (u, v, kind) twice is not.
capAndRefusal :: Bool
capAndRefusal =
  fieldsOf respond big ["degraded", "reason", "score"] == Just [Just (Bool True), Just (String "verdict_too_large"), Just (toJSON (0 :: Int))]
    && and [refusedBy respond (request (swapped sim, p, ch, co, cl, l, d, w, n)) "sim 0: pair not ascending" | (sim@(_ : _), p, ch, co, cl, l, d, w, n) <- take 40 wireCases]
    && fieldsOf respond (pairs [[0, 1, 0, 100, 100], [0, 1, 2, 100, 100]]) ["score"] == Just [Just (toJSON (1000 :: Int))]
    && refusedBy respond (pairs [[0, 1, 2, 100, 100], [0, 1, 2, 100, 100]]) "sim 1: not strictly ascending"
 where
  -- three files with a position each, the pair (0, 1) under the given
  -- sim rows
  pairs sim = request (sim, [[u, 1, u, 1, 1] | u <- [0 .. 2]], [], [], [], [], [], [], 3)
  big = setKey "discrete" (toJSON [0 .. verdictRowCap]) (request ([], [], [], [], [], [], [], [], 1))
  swapped rows = case rows of
    ((u : v : rest) : more) -> (v : u : rest) : more
    _ -> rows
