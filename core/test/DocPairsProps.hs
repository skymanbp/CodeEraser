-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | The docdup coarse filter (docpairs/1) and the sequence shape of
-- docdup/1 (plan v2.33 W3): the run walk measures maximal runs in words
-- on the cases the measuring side's own battery held before it moved
-- here, two hundred seeded coarse requests and two hundred seeded
-- sequence requests agree with ReferenceDocPairs, the seeded requests
-- are not vacuous, the refusals name their offender, and the caps hold.
module DocPairsProps (battery) where

import CE.Docdup (respond)
import CE.Docdup.Cost (docCorpusCap, docSeqCap, docSetCap)
import qualified CE.Docdup.Pairs as Pairs
import CE.Docdup.Runs (indexed, runWords)
import CE.Docdup.Cost (shingleK)
import Data.Aeson (Value (..))
import qualified Data.Aeson.KeyMap as KM
import ReferenceDocPairs (coarseExpected, coarseRequests, runRef, seqExpected, seqRequests)
import WireHarness (fieldsOf, refusedBy, runLegs, tabledRequest)

battery :: IO Bool
battery =
  runLegs
    [ "the run walk measures maximal runs in words (disjoint, whole, interior, repeated)"
    , "the run walk agrees with the dynamic program on every seeded pair"
    , "two hundred seeded coarse requests agree with the reference"
    , "two hundred seeded sequence requests agree with the reference"
    , "the seeded requests are not vacuous: LSH and seeds both find, a seed group chains, runs and dups occur"
    , "the refusals name their offender"
    , "the set, corpus and sequence caps degrade, never truncate"
    ]
    [ runCases
    , runAgrees
    , all coarseAgrees coarseRequests
    , all seqAgrees seqRequests
    , nonVacuous
    , all refused refusalCases
    , caps
    ]

coarseRequest :: [[Integer]] -> Value
coarseRequest ss = tabledRequest "7.0.0" "docpairs.request" [("sets", ss)]

seqRequest :: [[Integer]] -> [[Integer]] -> Value
seqRequest qs ps = tabledRequest "7.0.0" "docdup.request" [("seqs", qs), ("pairs", ps)]

run :: [Integer] -> [Integer] -> Integer
run a b = runWords shingleK (indexed a) (indexed b)

runCases :: Bool
runCases =
  run [1, 2, 3] [4, 5, 6] == 0
    && run [1, 2, 3] [1, 2, 3] == 3 + shingleK - 1
    && run [9, 1, 2, 8] [7, 1, 2, 6] == 2 + shingleK - 1
    && run [5, 5, 5] [5, 5] == 2 + shingleK - 1

runAgrees :: Bool
runAgrees = and [run (qs !! fromInteger i) (qs !! fromInteger j) == runRef (qs !! fromInteger i) (qs !! fromInteger j) | (qs, ps) <- seqRequests, [i, j] <- ps]

coarseAgrees :: [[Integer]] -> Bool
coarseAgrees ss = fieldsOf Pairs.respond (coarseRequest ss) ["pairs", "counts"] == Just (coarseExpected ss)

seqAgrees :: ([[Integer]], [[Integer]]) -> Bool
seqAgrees (qs, ps) = fieldsOf respond (seqRequest qs ps) ["scores", "verdicts", "runs", "counts"] == Just (seqExpected qs ps)

nonVacuous :: Bool
nonVacuous =
  any (positive "lshPairs" . coarseExpected) coarseRequests
    && any (positive "seedPairs" . coarseExpected) coarseRequests
    && any (positive "hotShingles" . coarseExpected) coarseRequests
    && any (\(qs, ps) -> any (/= Number 0) (runsOf (seqExpected qs ps))) seqRequests
    && any (\(qs, ps) -> positive "jaccardDups" (seqExpected qs ps)) seqRequests
 where
  positive k fs = case reverse fs of
    (Just (Object c) : _) -> maybe False (/= Number 0) (KM.lookup k c)
    _ -> False
  runsOf fs = case drop 2 fs of
    (Just (Array xs) : _) -> foldr (:) [] xs
    _ -> []

-- | Each refusal: the family, the request, the offender's stem.
refusalCases :: [(Bool, Value, String)]
refusalCases =
  [ (True, coarseRequest [[]], "set 0: empty set")
  , (True, coarseRequest [[2, 1]], "set 0: not strictly ascending")
  , (True, coarseRequest [[1], [-1]], "set 1: negative element")
  , (False, seqRequest [[]] [], "seq 0: empty sequence")
  , (False, seqRequest [[1], [2 ^ (64 :: Int)]] [], "seq 1: element outside u64")
  , (False, seqRequest [[1], [2]] [[0, 1, 5]], "pair 0: malformed row (need [i,j] beside seqs)")
  , (False, seqRequest [[1], [2]] [[0, 2]], "pair 0: endpoint out of range")
  , (False, seqRequest [[1], [2]] [[1, 1]], "pair 0: self pair")
  , (False, seqRequest [[1], [2], [3]] [[0, 2], [0, 1]], "pair 1: not strictly ascending")
  , (False, tabledRequest "7.0.0" "docdup.request" [("sets", [[1]]), ("seqs", [[1]]), ("pairs", [])], "sets: sent beside seqs")
  ]

refused :: (Bool, Value, String) -> Bool
refused (coarse, r, stem) = refusedBy (if coarse then Pairs.respond else respond) r stem

-- | One past each ceiling answers a degraded reply; at each the family
-- judges.
caps :: Bool
caps =
  degraded (coarseRequest [[1 .. docSetCap + 1]]) Pairs.respond
    && not (degraded (coarseRequest [[1 .. docSetCap]]) Pairs.respond)
    && not (Pairs.overCap (corpus 0)) && Pairs.overCap (corpus 1)
    && degraded (seqRequest [replicate (fromInteger docSeqCap + 1) 1, [1]] [[0, 1]]) respond
    && not (degraded (seqRequest [replicate (fromInteger docSeqCap) 1, [1]] [[0, 1]]) respond)
 where
  degraded r f = fieldsOf f r ["degraded"] == Just [Just (Bool True)]
  -- the corpus ceiling counted on the contract, not over a sixteen-
  -- million-element request: full-size sets sharing one list
  corpus extra = Pairs.PairsReq Null (replicate (fromInteger (docCorpusCap `div` docSetCap) + extra) (replicate (fromInteger docSetCap) 0))
