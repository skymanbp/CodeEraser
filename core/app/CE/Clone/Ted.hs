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
--
-- Since 7.5.0 (plan v2.31 step 6, merge/1) the same tables also give
-- back an optimal mapping (`tedMapping`): its distance is `ted`'s
-- because both read one cell recurrence (`candidates` + `choose`),
-- and the traceback only re-walks it (MergeProps holds the mapping a
-- valid Tai mapping whose cost is the distance).
module CE.Clone.Ted (Tree (..), ted, tedMapping) where

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
      td <- distances a b
      readArray td (rootCell a b)

-- | The distance and one optimal mapping — (node of a, node of b)
-- pairs, ascending — whose cost n1 + n2 − 2|M| + relabels is that
-- distance. The empty tree maps nothing.
tedMapping :: Tree -> Tree -> (Integer, [(Int, Int)])
tedMapping a b
  | tSize a == 0 || tSize b == 0 = (ted a b, [])
  | otherwise = runST $ do
      td <- distances a b
      d <- readArray td (rootCell a b)
      m <- trace a b td (tSize a - 1) (tSize b - 1)
      pure (fromIntegral d, sort m)

-- | Every keyroot pass; the tree-distance table they fill.
distances :: Tree -> Tree -> ST s (STUArray s Int Int)
distances a b = do
  td <- newIntArray (tSize a * tSize b)
  forM_ [(i, j) | i <- keyroots a, j <- keyroots b] $ \(i, j) ->
    forestPass a b i j td
  pure td

rootCell :: Tree -> Tree -> Int
rootCell a b = (tSize a - 1) * tSize b + (tSize b - 1)

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

-- | A forest pass's rectangle for spans [lld i1 .. i1] × [lld j1 ..
-- j1]: the two trees, the two first nodes and the (w1+1)×(w2+1)
-- table's widths. Strict: every cell of the hot loop reads it.
data Span = Span {sA :: !Tree, sB :: !Tree, sL1 :: !Int, sL2 :: !Int, sW1 :: !Int, sW2 :: !Int}

spanOf :: Tree -> Tree -> Int -> Int -> Span
spanOf a b i1 j1 = Span a b (tLld a ! i1) (tLld b ! j1) (i1 - tLld a ! i1 + 1) (j1 - tLld b ! j1 + 1)

{-# INLINE fkey #-}
fkey :: Span -> Int -> Int -> Int
fkey sp di dj = di * (sW2 sp + 1) + dj

-- | Both nodes sit on their span's leftmost leaf: a tree-distance
-- cell.
{-# INLINE aligned #-}
aligned :: Span -> Int -> Int -> Bool
aligned sp i j = tLld (sA sp) ! i == sL1 sp && tLld (sB sp) ! j == sL2 sp

-- | One forest pass: fills the forest-distance table for the span
-- pair and harvests tree distances at left-aligned cells (the
-- Zhang-Shasha recurrence). The forest table is a fresh rectangle
-- per pass, handed back for the traceback; `distances` drops it.
forestPass :: Tree -> Tree -> Int -> Int -> STUArray s Int Int -> ST s (STUArray s Int Int)
forestPass a b i1 j1 td = do
  let sp = spanOf a b i1 j1
  fd <- newIntArray ((sW1 sp + 1) * (sW2 sp + 1))
  forM_ [1 .. sW1 sp] $ \di -> writeArray fd (fkey sp di 0) di
  forM_ [1 .. sW2 sp] $ \dj -> writeArray fd (fkey sp 0 dj) dj
  forM_ [(i, j) | i <- [sL1 sp .. i1], j <- [sL2 sp .. j1]] $ \(i, j) -> do
    v <- fst . choose <$> candidates sp td fd i j
    writeArray fd (fkey sp (i - sL1 sp + 1) (j - sL2 sp + 1)) v
    when (aligned sp i j) $ writeArray td (i * tSize b + j) v
  pure fd

-- | One cell's three candidate costs: delete node i, insert node j,
-- and either match the two nodes (aligned: the diagonal plus the
-- relabel) or take the two subtrees whole (the forests left of them
-- plus their tree distance) — the one recurrence the distance and
-- the traceback both read. Inlined into both, so the distance's hot
-- loop keeps no call and no tuple per cell (the 3e budget's path).
{-# INLINE candidates #-}
candidates :: Span -> STUArray s Int Int -> STUArray s Int Int -> Int -> Int -> ST s (Int, Int, Int)
candidates sp td fd i j = do
  let (a, b) = (sA sp, sB sp)
      (di, dj) = (i - sL1 sp + 1, j - sL2 sp + 1)
  del <- (+ 1) <$> readArray fd (fkey sp (di - 1) dj)
  ins <- (+ 1) <$> readArray fd (fkey sp di (dj - 1))
  third <-
    if aligned sp i j
      then (+ fromEnum (tLab a ! i /= tLab b ! j)) <$> readArray fd (fkey sp (di - 1) (dj - 1))
      else (+) <$> readArray fd (fkey sp (tLld a ! i - sL1 sp) (tLld b ! j - sL2 sp)) <*> readArray td (i * tSize b + j)
  pure (del, ins, third)

-- | What an optimal cell did.
data Move = Delete | Insert | Take

-- | The cell's value — the least of the three — and the move that
-- reached it; a tie prefers taking the pair, then deleting.
{-# INLINE choose #-}
choose :: (Int, Int, Int) -> (Int, Move)
choose (del, ins, third)
  | third <= del && third <= ins = (third, Take)
  | del <= ins = (del, Delete)
  | otherwise = (ins, Insert)

-- | The mapping of subtrees i1 × j1: re-run their forest pass (the
-- tree-distance table is complete, so any pair's rectangle can be
-- rebuilt, and its aligned cells rewrite the values they already
-- hold) and walk back from the corner. A taken aligned cell maps the
-- two nodes; a taken subtree pair recurses into its own rectangle
-- and continues left of both subtrees; a border cell is all deletes
-- or all inserts, which map nothing.
trace :: Tree -> Tree -> STUArray s Int Int -> Int -> Int -> ST s [(Int, Int)]
trace a b td i1 j1 = do
  fd <- forestPass a b i1 j1 td
  let sp = spanOf a b i1 j1
      walk i j
        | i < sL1 sp || j < sL2 sp = pure []
        | otherwise = do
            (_, move) <- choose <$> candidates sp td fd i j
            case move of
              Delete -> walk (i - 1) j
              Insert -> walk i (j - 1)
              Take
                | aligned sp i j -> ((i, j) :) <$> walk (i - 1) (j - 1)
                | otherwise -> (<>) <$> trace a b td i j <*> walk (tLld a ! i - 1) (tLld b ! j - 1)
  walk i1 j1
