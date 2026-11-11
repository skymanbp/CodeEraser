-- | The third admissible bound before the tree edit distance (plan
-- v2.35 C1): the unit-cost edit distance between the two trees' label
-- SEQUENCES. Derivation (asserted against brute-force ted over the
-- exhaustive small-tree family in core/test/CloneSeqProps.hs):
--
--   A Tai mapping M is one-to-one and keeps ancestry and left-to-right
--   order. In postorder u precedes v iff u is a descendant of v or lies
--   to the left of v; M keeps both relations, so it is strictly
--   increasing in both postorder positions — an alignment of the two
--   postorder label sequences. Its cost as an alignment (unmatched
--   positions of either side plus matched positions whose labels
--   differ) is n1 + n2 − 2|M| + r, the mapping's cost. The sequence
--   distance is the least cost over all alignments, so it is at most
--   the cost of the best mapping: ted ≥ the postorder sequence
--   distance. In preorder u precedes v iff u is an ancestor of v or
--   lies to the left of v — also kept by M — so the same holds there.
--
-- A pair whose sequence distance exceeds the clone ceiling
-- (CE.Clone.Prefilter.cloneCeiling) provably is not a clone, whatever the
-- full computation would answer. The test runs the banded (Ukkonen)
-- recurrence over |i − j| ≤ k with two unboxed rows and stops as soon
-- as a row's least cell passes k: it answers "over k or not", never a
-- distance. The postorder sequence is the tree as sent; the preorder
-- one is rebuilt here from lab and lld and is read only when the
-- postorder test did not decide.
module CE.Clone.Sequence (labelsOver, preorder, sequenceOver) where

import CE.Clone.Ted (Tree (..))
import Control.Monad (forM_)
import Control.Monad.ST (ST, runST)
import Data.Array.Base (unsafeAt, unsafeRead, unsafeWrite)
import Data.Array.ST (STUArray, newArray, readArray, writeArray)
import Data.Array.Unboxed (UArray, bounds, listArray, rangeSize, (!))

-- | True ⇔ the pair provably is not within distance k: the postorder
-- sequence distance exceeds k, or else the preorder one does.
sequenceOver :: Int -> Tree -> Tree -> Bool
sequenceOver k a b = labelsOver k (tLab a) (tLab b) || labelsOver k (tPre a) (tPre b)

-- | Labels in preorder from the postorder (lab, lld) form: node i's
-- children tile [lld i .. i − 1], the last of them i − 1 and each
-- one's left sibling ending just before its leftmost leaf.
preorder :: UArray Int Int -> UArray Int Int -> UArray Int Int
preorder lab lld
  | n == 0 = listArray (0, -1) []
  | otherwise = listArray (0, n - 1) (map (lab !) (visit [n - 1]))
 where
  n = rangeSize (bounds lab)
  visit [] = []
  visit (i : rest) = i : visit (children i <> rest)
  children i = walk (i - 1) []
   where
    walk c acc
      | c < lld ! i = acc
      | otherwise = walk (lld ! c - 1) (c : acc)

-- | True ⇔ the unit-cost edit distance (insert and delete 1,
-- substitute 0 or 1) of the two sequences exceeds k. A cell outside the
-- band or above k is read as k + 1: no alignment through it can come
-- back to k or below, since every step costs at least nothing and the
-- band holds every alignment of cost k or less.
labelsOver :: Int -> UArray Int Int -> UArray Int Int -> Bool
labelsOver k a b
  | k < 0 || abs (n - m) > k = True
  | otherwise = runST $ do
      prev <- newRow k
      cur <- newRow k
      forM_ [0 .. min m k] $ \j -> writeArray prev (j + k) j
      rows 1 prev cur
 where
  (n, m) = (rangeSize (bounds a), rangeSize (bounds b))
  rows i prev cur
    | i > n = (> k) <$> readArray prev (m - n + k)
    | otherwise = do
        low <- bandRow k (a, b) i prev cur
        if low > k then pure True else rows (i + 1) cur prev

-- | One band row, offsets 0 .. 2k, every cell k + 1 (the one
-- allocation throat; it also pins the element type).
newRow :: Int -> ST s (STUArray s Int Int)
newRow k = newArray (0, 2 * k) (k + 1)

-- | Row i of the band into `cur` from row i − 1 in `prev`, cell (i, j)
-- at offset o = j − i + k; answers the row's least cell. Row i spans
-- j ∈ [lo, hi], row i − 1 spans [loP, hiP]; a neighbour outside its
-- row's span is "over k" (k + 1), so no cell is ever read that this or
-- the previous row did not write, and the left neighbour rides in the
-- loop instead of the array. The unchecked reads and writes are in
-- range by that construction: for lo ≤ j ≤ hi the cell's own offset o
-- lies in [0, 2k] (j ≥ i − k and j ≤ i + k); the upper neighbour
-- (i − 1, j) sits at o + 1, read only when j ≤ hiP ≤ i − 1 + k, so
-- o + 1 ≤ 2k; the diagonal (i − 1, j − 1) sits at o, read only when
-- j − 1 ≥ loP; the labels are read at i − 1 < n and j − 1 < m.
bandRow :: Int -> (UArray Int Int, UArray Int Int) -> Int -> STUArray s Int Int -> STUArray s Int Int -> ST s Int
bandRow k (a, b) i prev cur = go (max 0 (i - k)) big big
 where
  big = k + 1
  m = rangeSize (bounds b)
  (hi, loP, hiP) = (min m (i + k), max 0 (i - 1 - k), min m (i - 1 + k))
  ai = unsafeAt a (i - 1)
  go !j !low !left
    | j > hi = pure low
    | otherwise = do
        let o = j - i + k
        del <- if j <= hiP then (+ 1) <$> unsafeRead prev (o + 1) else pure big
        sub <-
          if j >= 1 && j - 1 >= loP
            then (+ fromEnum (ai /= unsafeAt b (j - 1))) <$> unsafeRead prev o
            else pure big
        let v = min big (min del (min (left + 1) sub))
        unsafeWrite cur o v
        go (j + 1) (min low v) v
