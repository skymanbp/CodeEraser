{-# LANGUAGE BangPatterns #-}

-- | Verbatim runs over shingle sequences (design vol.2 §5.3; plan v2.33
-- W3): a segment arrives as its UNSORTED shingle sequence (the order a
-- run needs); its Jaccard set is that sequence sorted and deduplicated,
-- and a pair's verbatim run is the longest common contiguous shingle
-- run, in words — R shingles span R + shingleK − 1 words. Until W3 the
-- measuring side measured the runs and sent them with each pair; it now
-- sends the sequences and this module measures.
--
-- The walk is seed-extension: a run starts only at a position pair
-- whose previous shingles differ (or at either sequence's first
-- shingle), then extends right while the shingles agree — each maximal
-- run measured once, from its start.
module CE.Docdup.Runs (Seq, indexed, runWords, setOf) where

import Data.Array.Base (numElements, unsafeAt)
import Data.Array.Unboxed (UArray, listArray)
import qualified Data.IntMap.Strict as IM
import qualified Data.Set as S

-- | One sequence and, per shingle value, its positions ascending.
-- Values are the u64 bit patterns as Int (a bijection), so equality is
-- the hashes' equality.
data Seq = Seq !(UArray Int Int) !(IM.IntMap [Int])

indexed :: [Integer] -> Seq
indexed xs = Seq arr (IM.fromListWith (++) [(v, [j]) | (j, v) <- reverse (zip [0 ..] vals)])
 where
  vals = map fromInteger xs :: [Int]
  arr = listArray (0, length vals - 1) vals

-- | The Jaccard set of a sequence: its distinct values ascending (as
-- the u64 values, not their Int patterns).
setOf :: [Integer] -> [Integer]
setOf = S.toAscList . S.fromList

-- | The longest common contiguous run of two sequences in words: zero
-- when they share no shingle, else the run's shingles plus k − 1.
runWords :: Integer -> Seq -> Seq -> Integer
runWords k (Seq a _) (Seq b pos)
  | best == 0 = 0
  | otherwise = toInteger best + k - 1
 where
  la = numElements a
  lb = numElements b
  best = foldl' max 0 [extend i j 0 | i <- [0 .. la - 1], j <- IM.findWithDefault [] (a `unsafeAt` i) pos, starts i j]
  starts i j = i == 0 || j == 0 || a `unsafeAt` (i - 1) /= b `unsafeAt` (j - 1)
  extend !i !j !len
    | i < la && j < lb && a `unsafeAt` i == b `unsafeAt` j = extend (i + 1) (j + 1) (len + 1)
    | otherwise = len :: Int
