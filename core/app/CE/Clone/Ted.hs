-- | Zhang-Shasha tree edit distance over the wire's postorder form:
-- lab[i] = request-local dense label, lld[i] = postorder index of
-- node i's leftmost leaf descendant. Unit costs (delete = insert =
-- 1, relabel = 0/1). The tables are unboxed ST arrays, not IntMap:
-- the 3e measured exit caught per-cell IntMap inserts pushing
-- ripgrep's cold path past its budget by an order of magnitude
-- (self: 524 pairs ≈ 20 s) — the forest tables are dense rectangles,
-- exactly what O(1) unboxed cells exist for. Correctness is not
-- argued here: core/test/CloneProps.hs holds this function equal to
-- the mapping-definition brute force (ReferenceTed) over the
-- exhaustive small-tree family (R2) — the battery that already
-- caught two real defects of the first draft.
module CE.Clone.Ted (Tree (..), ted) where

import Control.Monad (forM_, when)
import Control.Monad.ST (ST, runST)
import Data.Array.ST (STUArray, newArray, readArray, writeArray)
import Data.Array.Unboxed (UArray, (!))
import qualified Data.IntMap.Strict as IM
import Data.List (sort)

-- | One decoded tree: postorder labels and leftmost-leaf indices as
-- O(1) unboxed lookups plus the node count. tHisto is the label
-- histogram — a property of the tree, not of any pair — attached
-- lazily at decode so the prefilter reads it once per OPERAND while
-- a tree whose pairs are all decided by the O(1) size corollary
-- never forces it (batch 9 P11). ted itself never reads it.
data Tree = Tree
  { tLab :: UArray Int Int
  , tLld :: UArray Int Int
  , tSize :: Int
  , tHisto :: IM.IntMap Integer
  }

-- | Tree edit distance. The empty tree is total-function territory
-- (distance = the other tree's size) even though the wire contract
-- refuses empty trees upstream. Distances fit Int (≤ n1 + n2, both
-- capped upstream); the Integer face keeps core float- and
-- overflow-uniform at the caller.
ted :: Tree -> Tree -> Integer
ted a b
  | tSize a == 0 || tSize b == 0 = fromIntegral (max (tSize a) (tSize b))
  | otherwise = fromIntegral $ runST $ do
      td <- newIntArray (tSize a * tSize b)
      forM_ [(i, j) | i <- keyroots a, j <- keyroots b] $ \(i, j) ->
        forestPass a b i j td
      readArray td ((tSize a - 1) * tSize b + (tSize b - 1))

-- | The one array-allocation throat (also pins the STUArray element
-- type, which `newArray` alone leaves ambiguous). Zero-initialized;
-- every cell a pass reads was written by that pass or an earlier one
-- — the ascending keyroot order CloneProps proved on the IntMap
-- version, where a missing fill crashed instead of reading 0.
newIntArray :: Int -> ST s (STUArray s Int Int)
newIntArray n = newArray (0, n - 1) 0

-- | Keyroots: for every distinct lld value, the highest postorder
-- index carrying it (the root of the leftmost-path bundle), sorted
-- ASCENDING BY NODE INDEX — IntMap elems come out in lld-key order,
-- which is not postorder (a two-child root yields [2,1]), and the
-- accumulation requires subtree distances to exist before larger
-- spans read them (the property battery caught exactly this).
keyroots :: Tree -> [Int]
keyroots t =
  sort (IM.elems (IM.fromListWith max [(tLld t ! i, i) | i <- [0 .. tSize t - 1]]))

-- | One keyroot-pair forest pass: fills the forest-distance table
-- for spans [lld i1 .. i1] × [lld j1 .. j1] and harvests tree
-- distances at left-aligned cells (the Zhang-Shasha recurrence).
-- The forest table is a fresh (w1+1)×(w2+1) rectangle per pass.
forestPass :: Tree -> Tree -> Int -> Int -> STUArray s Int Int -> ST s ()
forestPass a b i1 j1 td = do
  let (l1, l2) = (tLld a ! i1, tLld b ! j1)
      (w1, w2) = (i1 - l1 + 1, j1 - l2 + 1)
      fkey di dj = di * (w2 + 1) + dj
  fd <- newIntArray ((w1 + 1) * (w2 + 1))
  forM_ [1 .. w1] $ \di -> writeArray fd (fkey di 0) di
  forM_ [1 .. w2] $ \dj -> writeArray fd (fkey 0 dj) dj
  forM_ [(i, j) | i <- [l1 .. i1], j <- [l2 .. j1]] $ \(i, j) -> do
    let (di, dj) = (i - l1 + 1, j - l2 + 1)
        aligned = tLld a ! i == l1 && tLld b ! j == l2
    del <- (+ 1) <$> readArray fd (fkey (di - 1) dj)
    ins <- (+ 1) <$> readArray fd (fkey di (dj - 1))
    v <-
      if aligned
        then do
          let rel = if tLab a ! i == tLab b ! j then 0 else 1
          diag <- readArray fd (fkey (di - 1) (dj - 1))
          pure (minimum [del, ins, diag + rel])
        else do
          sub <- readArray fd (fkey (tLld a ! i - l1) (tLld b ! j - l2))
          t <- readArray td (i * tSize b + j)
          pure (minimum [del, ins, sub + t])
    writeArray fd (fkey di dj) v
    when aligned $ writeArray td (i * tSize b + j) v
