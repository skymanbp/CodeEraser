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
import CE.Verdict.Score (Facts (..), ScoreKnobs, classKnobsOf, penalties, score, scoreBound)
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
    , "past the row cap the reply degrades; a descending sim pair is refused by name"
    ]
    [ all charges factCases
    , and [softLine kk lo hi locs == refSoftLine kk lo hi locs | locs <- locCases, (kk, lo, hi) <- [(softLineK, softMin, softMax), (1, 1, 100000)]]
    , and [zonePenalty s h p x == refZone s h p x | s <- [0 .. 6], h <- [0 .. 6], p <- [1, 10], x <- [0 .. 12]]
    , and [tolerated ratchetBound d c == refTolerated ratchetBound d c | c <- [0 .. 1200], d <- [Nothing, Just 0, Just 7]]
    , all ratchets ratchetCases
    , and [judge bound l == refJudge bound l && confidence mask bits == refConfidence bound l | l <- joinFamily, let (_, mask, bits) = judge bound l]
    , all wired wireCases
    , capAndRefusal
    ]

charges :: (ScoreKnobs, Maybe Integer, Inputs) -> Bool
charges (k, soft, inp) = pens == refCharges k soft inp && and [score k w pens == refScore k violCostNeutral w pens | w <- [[], [[0, 2], [3, 0], [6, 5]]]]
 where
  pens = penalties k soft (Facts (iSim inp) (iPos inp) (iChurn inp) (iCont inp) (iDocs inp) (classKnobsOf (iClassRows inp)) (iLoops inp))

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
  pens = refCharges scoreBound (refSoftLine softLineK softMin softMax locs) (Inputs sim pos churn cont docs classes [])
  (perMille, _) = refScore scoreBound violCostNeutral weights pens

-- | The reference's own reading of the two boundaries: one discrete
-- entry past verdictRowCap degrades, and a sim pair whose endpoints
-- descend is the contract's to refuse.
capAndRefusal :: Bool
capAndRefusal =
  fieldsOf respond big ["degraded", "reason", "score"] == Just [Just (Bool True), Just (String "verdict_too_large"), Just (toJSON (0 :: Int))]
    && and [refusedBy respond (request (swapped sim, p, ch, co, cl, l, d, w, n)) "sim 0: pair not ascending" | (sim@(_ : _), p, ch, co, cl, l, d, w, n) <- take 40 wireCases]
 where
  big = setKey "discrete" (toJSON [0 .. verdictRowCap]) (request ([], [], [], [], [], [], [], [], 1))
  swapped rows = case rows of
    ((u : v : rest) : more) -> (v : u : rest) : more
    _ -> rows
