-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | The verdict family against its second implementation (plan v2.32
-- step 7): the seven axis charges and the fold over two hundred fact
-- sets under perturbed knobs, the soft line over two hundred LOC
-- distributions, the zone curve and the single-edit allowance over
-- exhaustive grids, the ratchet over two hundred baseline cases, the
-- join lattice and its confidence over every leg combination, and the
-- score, axes and weight echo of two hundred real verdict/1 replies —
-- plus the row cap and a contract refusal through the real respond.
module VerdictEquivProps (battery) where

import CE.Verdict (respond)
import CE.Verdict.Cost (defaultWeight, softLineK, softMax, softMin, verdictRowCap, violCostNeutral)
import CE.Verdict.Join (bound, confidence, judge)
import CE.Verdict.Ratchet (Baseline (..), Ratcheted (..), ratchet, ratchetBound, tolerated)
import CE.Verdict.Score (Facts (..), ScoreKnobs (..), classKnobsOf, penalties, score, scoreBound)
import CE.Verdict.Soft (softLine, zonePenalty)
import Data.Aeson (Value (..), object, toJSON, (.=))
import qualified Data.Map.Strict as M
import ReferenceRatchet (refConfidence, refJudge, refRatchet, refTolerated)
import ReferenceScore (Inputs (..), refCharges, refScore, refSoftLine, refZone)
import VerdictGen (WireCase, factCases, joinFamily, locCases, ratchetCases, wireCases)
import WireHarness (fieldsOf, refusedBy, runLegs, setKey)

battery :: IO Bool
battery =
  runLegs
    [ "two hundred fact sets: the seven charges and the fold agree under perturbed knobs and weights"
    , "two hundred LOC distributions: the soft line agrees, clamped and unclamped"
    , "the zone curve agrees on every (s, h, pMax, x) of the grid, the degenerate wall included"
    , "the single-edit allowance agrees on every ceiling to 1200, global and declared"
    , "two hundred ratchet cases agree on over, drawn, added, removed, dropped and the new baseline"
    , "the join lattice agrees on every leg combination, verdict, legs, reasons and confidence"
    , "two hundred verdict/1 replies agree on the score, the axes and the weight echo"
    , "past the row cap the reply degrades; a descending sim pair, a repeated (u, v, kind) and an endpoint outside its family's set are refused by name; one pair under two families is judged"
    , "booklet 05's bounds: axes 1, 4, 5 and 6 (and 2, 3 when the docdup universe rides) charge at most half the scale; mass inside opportunity; the equal-weight neutral score keeps its floor"
    ]
    [ all charges factCases
    , and [softLine kk lo hi locs == refSoftLine kk lo hi locs | locs <- locCases, (kk, lo, hi) <- [(softLineK, softMin, softMax), (1, 1, 100000)]]
    , and [zonePenalty s h p x == refZone s h p x | s <- [0 .. 6], h <- [0 .. 6], p <- [1, 10], x <- [0 .. 12]]
    , and [tolerated ratchetBound m d c == refTolerated ratchetBound m d c | c <- [0 .. 1200], d <- [Nothing, Just 0, Just 7], m <- [0, 1]]
    , all ratchets ratchetCases
    , and [judge bound l == refJudge bound l && confidence mask bits == refConfidence bound l | l <- joinFamily, let (_, mask, bits) = judge bound l]
    , all wired wireCases
    , capAndRefusal
    , boundsHold
    ]

charges :: (ScoreKnobs, Maybe Integer, Inputs) -> Bool
charges c@(k, soft, inp) = pens == refCharges k soft inp && and [score k w pens == refScore k violCostNeutral w pens | w <- [[], [[0, 2], [3, 0], [6, 5]]]]
 where
  pens = pensOf c

pensOf :: (ScoreKnobs, Maybe Integer, Inputs) -> [(Integer, Integer)]
pensOf (k, soft, inp) = penalties k soft (Facts (iSim inp) (iPos inp) (iChurn inp) (iCont inp) (iDocs inp) (classKnobsOf (iClassRows inp)) (iLoops inp) (iOwners inp))

-- | Booklet 05: axes 1, 4, 5 and 6 count rows of their own opportunity
-- list, so v <= n and the charge is at most half the scale. A request
-- naming the docdup universe (9.1.0) holds axes 2 and 3 there too, by
-- contract: every clone endpoint is a code file and every docdup
-- endpoint a segment owner (CE.Verdict.Rows.simSplitOffence; the
-- generator keeps only the rows it admits), so each mass is a subset of
-- its opportunity. With k axes unbounded (axis 0 always; 2 and 3 on a
-- request naming no universe) the equal-weight neutral floor is
-- scale - floor((k (scale - 1) + (7 - k) floor(scale / 2)) / 7): 287
-- at 1000 with k = 3, 429 with k = 1.
subsets :: (ScoreKnobs, Maybe Integer, Inputs) -> Bool
subsets c@(k, _, inp) =
  and [2 * p <= s | (a, p) <- pens, a `elem` bounded]
    && massesInside inp
    && (sViolCost k /= violCostNeutral || fst (score k [] pens) >= s - (open * (s - 1) + (7 - open) * (s `div` 2)) `div` 7)
 where
  s = sScoreScale k
  pens = pensOf c
  bounded = subsetAxes (iOwners inp)
  open = 7 - toInteger (length bounded)

-- | The axes whose mass is a subset of their opportunity.
subsetAxes :: Maybe [Integer] -> [Integer]
subsetAxes owners = maybe [1, 4, 5, 6] (const [1, 2, 3, 4, 5, 6]) owners

-- | Mass inside opportunity, set by set, on a request naming the
-- universe: the files the clone rows touch are code files, the files
-- the docdup rows touch are owners.
massesInside :: Inputs -> Bool
massesInside inp = case iOwners inp of
  Nothing -> True
  Just owners -> all (`elem` code) (ends [0, 1]) && all (`elem` owners) (ends [2])
 where
  code = [u | (u : _) <- iPos inp, u `notElem` iDocs inp]
  ends kinds = [x | [u, v, kind, _, _] <- iSim inp, kind `elem` kinds, x <- [u, v]]

-- | The bounds over every fact set, and not vacuously: a subset axis
-- charges, and so does axis 3 on a request naming its universe.
boundsHold :: Bool
boundsHold =
  all subsets factCases
    && any (\c -> any (\(a, p) -> a `elem` [1, 4, 5, 6] && p > 0) (pensOf c)) factCases
    && any (\c@(_, _, inp) -> iOwners inp /= Nothing && any (\(a, p) -> a == 3 && p > 0) (pensOf c)) factCases

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
    ]

-- | The establishing run judges with the soft line its own LOC
-- distribution derives, under the default knobs.
wired :: WireCase -> Bool
wired c@(sim, pos, churn, cont, classes, locs, docs, weights, _) =
  fieldsOf respond (request c) ["score", "axes", "weights"]
    == Just [Just (toJSON perMille), Just (toJSON [[a, p] | (a, p) <- pens]), Just (toJSON [[a, maybe defaultWeight id (lookup a [(x, w) | [x, w] <- weights])] | a <- [0 .. 6 :: Integer]])]
 where
  -- the establishing derivation is fenced under the effective hard
  -- line as well as softMax (9.0.0)
  pens = refCharges scoreBound (refSoftLine softLineK softMin (min softMax (sSizeHard scoreBound)) locs) (Inputs sim pos churn cont docs classes [] Nothing)
  (perMille, _) = refScore scoreBound violCostNeutral weights pens

-- | The reference's own reading of the boundaries: one discrete entry
-- past verdictRowCap degrades, a sim pair whose endpoints descend is
-- the contract's to refuse — and since 9.1.0 the sim identity is
-- (u, v, kind): one pair under two families is a request, the same
-- (u, v, kind) twice is not, and a request naming the docdup universe
-- has each family's endpoints held to its own set.
capAndRefusal :: Bool
capAndRefusal =
  fieldsOf respond big ["degraded", "reason", "score"] == Just [Just (Bool True), Just (String "verdict_too_large"), Just (toJSON (0 :: Int))]
    && and [refusedBy respond (request (swapped sim, p, ch, co, cl, l, d, w, n)) "sim 0: pair not ascending" | (sim@(_ : _), p, ch, co, cl, l, d, w, n) <- take 40 wireCases]
    && fieldsOf respond (split [[0, 1, 0, 100, 100], [0, 1, 2, 100, 100]] [0, 1]) ["axes"] == Just [Just (toJSON [[0, 0 :: Int], [1, 0], [2, 400], [3, 500], [4, 0], [5, 0], [6, 0]])]
    && refusedBy respond (split [[0, 1, 2, 100, 100], [0, 1, 2, 100, 100]] [0, 1]) "sim 1: not strictly ascending"
    && refusedBy respond (split [[0, 2, 2, 100, 100]] [0, 1]) "sim 0: docdup endpoint owns no segment"
    && refusedBy respond (setKey "docFiles" (toJSON [1 :: Int]) (split [[0, 1, 0, 100, 100]] [0, 1])) "sim 0: clone endpoint is not a code file"
 where
  -- three files with a position each, the pair (0, 1) under the given
  -- sim rows, the docdup universe named
  split sim owners = setKey "docOwners" (toJSON (owners :: [Integer])) (request (sim, [[u, 1, 1, u, 1, 1] | u <- [0 .. 2]], [], [], [], [], [], [], 3))
  big = setKey "discrete" (toJSON [0 .. verdictRowCap]) (request ([], [], [], [], [], [], [], [], 1))
  swapped rows = case rows of
    ((u : v : rest) : more) -> (v : u : rest) : more
    _ -> rows
