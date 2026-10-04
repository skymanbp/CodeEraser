-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | The docdup coarse filter's and the sequence judgment's second
-- implementation (plan v2.33 W3): LSH buckets and seed groups as
-- Data.Map keys over the byte-by-byte MinHash of ReferenceCandidates,
-- the pair set a Data.Map, the verbatim run the classic dynamic program
-- over every prefix pair (the longest common suffix table, no seeds, no
-- extension), a set the sorted nub of its sequence, Jaccard by
-- membership counts. It answers the reply fields `docpairs.result` and
-- a sequence `docdup.result` show.
module ReferenceDocPairs (coarseExpected, coarseRequests, runRef, seqExpected, seqRequests) where

import CE.Docdup.Cost (dupDecides, dupVerdict, shingleK)
import Data.Aeson (Value (..), object, toJSON, (.=))
import Data.List (nub, sort)
import qualified Data.Map.Strict as M
import qualified Data.Map.Lazy as Lazy
import ReferenceCandidates (bandKeysOf, hot, pairsIn)
import DocumentGen (seededBy)
import ReferenceFlowGen (G, rand)

-- | pairs, counts for a coarse request over valid sets.
coarseExpected :: [[Integer]] -> [Maybe Value]
coarseExpected ss =
  [ Just (toJSON [[a, b] | (a, b) <- M.keys withSeeds])
  , Just
      ( object
          [ "sets" .= length ss
          , "lshPairs" .= M.size byBands
          , "seedPairs" .= (M.size withSeeds - M.size byBands)
          , "hotBands" .= hotCount buckets
          , "hotShingles" .= hotCount seeds
          ]
      )
  ]
 where
  indexed = zip [0 :: Integer ..] ss
  buckets = M.elems (M.fromListWith (flip (<>)) [(k, [i]) | (i, s) <- indexed, k <- bandKeysOf s])
  seeds = M.elems (M.fromListWith (flip (<>)) [(x, [i]) | (i, s) <- indexed, x <- s])
  collect gs = M.fromList [((min a b, max a b), ()) | g <- gs, (a, b) <- pairsIn g, a /= b]
  byBands = collect buckets
  withSeeds = M.union byBands (collect seeds)
  hotCount gs = length (filter hot gs)

-- | The longest common contiguous run of two sequences in words.
runRef :: [Integer] -> [Integer] -> Integer
runRef xs ys = if best == 0 then 0 else best + shingleK - 1
 where
  -- lazy: each cell reads its diagonal neighbour from the same table
  table = Lazy.fromList [((i, j), cell i j) | i <- [0 .. length xs], j <- [0 .. length ys]]
  cell i j
    | i == 0 || j == 0 = 0
    | xs !! (i - 1) == ys !! (j - 1) = table Lazy.! (i - 1, j - 1) + 1
    | otherwise = 0 :: Integer
  best = maximum (Lazy.elems table)

-- | scores, verdicts, runs, counts for a sequence request over valid
-- sequences and pairs.
seqExpected :: [[Integer]] -> [[Integer]] -> [Maybe Value]
seqExpected qs ps =
  [ Just (toJSON [[i, j, inter, union] | (i, j, inter, union, _) <- rows])
  , Just (toJSON [dupVerdict inter union run | (_, _, inter, union, run) <- rows])
  , Just (toJSON [run | (_, _, _, _, run) <- rows])
  , Just
      ( object
          [ "sets" .= length qs
          , "pairs" .= length ps
          , "judged" .= length rows
          , "jaccardDups" .= length [() | (_, _, inter, union, _) <- rows, dupDecides inter union]
          ]
      )
  ]
 where
  at k = qs !! fromInteger k
  rows =
    [ (i, j, count (\x -> x `elem` sa && x `elem` sb), count (\x -> x `elem` sa || x `elem` sb), runRef (at i) (at j))
    | [i, j] <- ps
    , let sa = sort (nub (at i))
    , let sb = sort (nub (at j))
    , let count p = toInteger (length (filter p (nub (sa <> sb))))
    ]

-- | Two hundred seeded coarse requests: two to twelve sets of one to
-- four values from eight (so bands and seeds collide), and now and then
-- seventy sets sharing one value (a hot seed group).
coarseRequests :: [[[Integer]]]
coarseRequests = seededBy 7001 13 coarseG

coarseG :: G [[Integer]]
coarseG = do
  heat <- rand 5
  size <- if heat == 0 then pure 70 else (+ 2) <$> rand 11
  mapM (const (setG (heat == 0))) [1 .. size]
 where
  setG shared = do
    k <- (+ 1) <$> rand 4
    picks <- mapM (const (rand 8)) [1 .. k]
    let vals = [toInteger p * 2654435761 + 3 | p <- picks] <> [99 | shared]
    pure (sort (nub vals))

-- | Two hundred seeded sequence requests: two to six sequences of one
-- to twelve shingles over four values (so runs repeat and nest), and
-- every ascending pair.
seqRequests :: [([[Integer]], [[Integer]])]
seqRequests = seededBy 3011 7 seqG

seqG :: G ([[Integer]], [[Integer]])
seqG = do
  size <- (+ 2) <$> rand 5
  qs <- mapM (const one) [1 .. size]
  let n = toInteger size
  pure (qs, [[i, j] | i <- [0 .. n - 1], j <- [i + 1 .. n - 1]])
 where
  one = do
    len <- (+ 1) <$> rand 12
    mapM (const ((\v -> toInteger v * 1000000007 + 5) <$> rand 4)) [1 .. len]
