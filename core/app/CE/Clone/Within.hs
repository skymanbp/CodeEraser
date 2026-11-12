-- | The tree edit distance of a pair, computed only inside the band the
-- clone threshold allows (plan v2.35 C7a). After the three admissible
-- bounds almost every pair that reaches a distance is a clone, and a
-- clone's distance is at most k = the clone ceiling of the larger tree
-- (CE.Clone.Prefilter.cloneCeiling). `tedWithin k a b` runs `ted`'s
-- Zhang-Shasha recurrence (the same keyroot passes, the same three
-- options per cell, CE.Clone.Ted) with every forest cell outside
-- |di − dj| <= k read as "more than k" and every tree-distance entry
-- whose two subtrees' sizes differ by more than k — exactly the entries
-- no in-band cell writes — read the same way; "more than k" is held as k + 1
-- and every value is clamped to it. It answers `Just d` when the
-- distance is at most k and `Nothing` otherwise. `ted` and `tedMapping`
-- stay as they are: merge/1 reads their full tables.
--
-- Why it is exact. Let fd be one keyroot pass's forest table and td the
-- tree-distance table, fd' and td' their banded, clamped versions.
-- (1) fd[di][dj] >= |di − dj| and td[i][j] >= |size i − size j|: a
-- mapping leaves at least the size difference unmatched. So a cell or
-- entry whose true value is at most k lies in the band (an aligned
-- cell's widths are its two subtrees' sizes).
-- (2) Removing options from a minimum, or reading an operand as k + 1
-- where its true value is larger, cannot lower the result below
-- min (true value, k + 1): fd' >= min (fd, k + 1) and td' >= min (td,
-- k + 1) everywhere.
-- (3) If fd[di][dj] <= k then fd'[di][dj] = fd[di][dj], by induction in
-- the order the tables are filled (keyroots ascending, cells row by
-- row): the option that attains the minimum reads fd[di−1][dj] + 1,
-- fd[di][dj−1] + 1, fd[di−1][dj−1] + (0 or 1), or
-- fd[lld i − l1][lld j − l2] + td[i][j]; every cost is non-negative, so
-- each operand of the attaining option is itself at most k, hence in
-- the band by (1) and equal to its banded value by the induction
-- hypothesis (the td operand was written by an earlier pass or an
-- earlier cell of this one); the other options are at least
-- min (their true values, k + 1) by (2). The same holds for td, which
-- is read off aligned cells. So the root entry is the true distance
-- when that is at most k, and k + 1 otherwise. ∎
--
-- CloneWithinProps holds this equal to `ted` cut at k on the exhaustive
-- small-tree family for every k from 0 to the sum of the sizes, and on
-- generated trees up to the node cap.
module CE.Clone.Within (tedWithin) where

import CE.Clone.Ted (Span (..), Tree (..), keyroots, spanOf)
import Control.Monad (forM_, when)
import Control.Monad.ST (ST, runST)
import Data.Array.Base (unsafeRead, unsafeWrite)
import Data.Array.ST (STUArray, newArray_)
import Data.Array.Unboxed ((!))

-- | The distance when it is at most k; Nothing when it is more.
tedWithin :: Int -> Tree -> Tree -> Maybe Integer
tedWithin k a b
  | k < 0 = Nothing
  | tSize a == 0 || tSize b == 0 = cut (max (tSize a) (tSize b))
  -- (1) at the root: the size difference alone is more than k
  | abs (tSize a - tSize b) > k = Nothing
  | otherwise = cut (runST (banded k a b))
 where
  cut d = if d <= k then Just (toInteger d) else Nothing

-- | Every keyroot pass, banded; the root entry, clamped to k + 1. The
-- tree-distance table is not initialised: an entry is read only when its
-- two subtrees' sizes are within k (`subtree`), and such an entry was
-- written — its own pass runs earlier (the keyroot order `ted` relies on)
-- and its aligned cell, whose widths are those two sizes, lies in that
-- pass's band. The root entry is in band by the guard above.
banded :: Int -> Tree -> Tree -> ST s Int
banded k a b = do
  td <- table (tSize a * tSize b)
  forM_ [(i, j) | i <- keyroots a, j <- keyroots b] $ \(i, j) ->
    bandPass (Band k (spanOf a b i j) td)
  -- in range: the last cell of the n1 × n2 table
  unsafeRead td ((tSize a - 1) * tSize b + (tSize b - 1))

-- | One pass's band: k, the rectangle, the tree-distance table.
data Band s = Band !Int !Span !(STUArray s Int Int)

-- | One forest pass over the band of its rectangle. The forest table
-- holds a row's band only — 2k + 1 cells, or the whole row when that is
-- narrower; it is not initialised either: every cell a read reaches is in
-- the band (reads outside it answer k + 1 without touching the table), and
-- every in-band cell is written before it is read — the borders up to k
-- first, then the rows in order, each row left to right, and every operand
-- of a cell lies in an earlier row or earlier in the same row.
bandPass :: Band s -> ST s ()
bandPass bd@(Band k sp td) = do
  let (w1, w2, n2) = (sW1 sp, sW2 sp, tSize (sB sp))
      lay = layout k w2
  fd <- table ((w1 + 1) * fst lay)
  -- in range: (0, dj) and (di, 0) with dj, di <= k are band cells of rows
  -- 0 .. w1 (`at` maps a band cell of row di into that row's width)
  forM_ [0 .. min k w2] $ \dj -> unsafeWrite fd (at lay 0 dj) dj
  forM_ [1 .. min k w1] $ \di -> unsafeWrite fd (at lay di 0) di
  forM_ [1 .. w1] $ \di -> do
    let i = sL1 sp + di - 1
        onPath = tLld (sA sp) ! i == sL1 sp
    forM_ [max 1 (di - k) .. min w2 (di + k)] $ \dj -> do
      let j = sL2 sp + dj - 1
          aligned = onPath && tLld (sB sp) ! j == sL2 sp
      v <- cell bd fd lay (i, j, aligned) (di, dj)
      -- in range: (di, dj) is a band cell of row di <= w1
      unsafeWrite fd (at lay di dj) v
      -- in range: i < n1 and j < n2, nodes of the two trees
      when aligned $ unsafeWrite td (i * n2 + j) v

-- | One in-band cell: `ted`'s three options (CE.Clone.Ted.candidates),
-- their least clamped to k + 1.
{-# INLINE cell #-}
cell :: Band s -> STUArray s Int Int -> (Int, Bool) -> (Int, Int, Bool) -> (Int, Int) -> ST s Int
cell (Band k sp td) fd lay (i, j, aligned) (di, dj) = do
  let (a, b) = (sA sp, sB sp)
      rd = band k fd lay
  del <- (+ 1) <$> rd (di - 1) dj
  ins <- (+ 1) <$> rd di (dj - 1)
  third <-
    if aligned
      then (+ fromEnum (tLab a ! i /= tLab b ! j)) <$> rd (di - 1) (dj - 1)
      else do
        f <- rd (tLld a ! i - sL1 sp) (tLld b ! j - sL2 sp)
        t <- subtree k td (tSize b) (i - tLld a ! i + 1, j - tLld b ! j + 1) (i, j)
        pure (f + t)
  pure (min (k + 1) (min del (min ins third)))

-- | A forest cell: k + 1 outside the band, the table's value inside it.
{-# INLINE band #-}
band :: Int -> STUArray s Int Int -> (Int, Bool) -> Int -> Int -> ST s Int
band k fd lay di dj
  | abs (di - dj) > k = pure (k + 1)
  -- in range: a band cell (di, dj >= 0 here: every caller's operands are
  -- at least 0, and dj = di ± k + 1 is outside the band) of a row <= w1
  | otherwise = unsafeRead fd (at lay di dj)

-- | A tree-distance entry: k + 1 when the two subtrees' sizes differ by
-- more than k (its true value is at least that, by (1)), else the entry.
{-# INLINE subtree #-}
subtree :: Int -> STUArray s Int Int -> Int -> (Int, Int) -> (Int, Int) -> ST s Int
subtree k td n2 (si, sj) (i, j)
  | abs (si - sj) > k = pure (k + 1)
  -- in range: i < n1 and j < n2; written, as `banded` says
  | otherwise = unsafeRead td (i * n2 + j)

-- | A forest table's row width and whether it holds whole rows.
layout :: Int -> Int -> (Int, Bool)
layout k w2 = if 2 * k + 1 >= w2 + 1 then (w2 + 1, True) else (2 * k + 1, False)

-- | A band cell's slot: whole rows by column, else by its offset in the
-- band (dj − di + k, 0 .. 2k).
{-# INLINE at #-}
at :: (Int, Bool) -> Int -> Int -> Int
at (width, full) di dj = di * width + (if full then dj else dj - di + (width - 1) `div` 2)

-- | An uninitialised table of n cells (also pins the element type
-- `newArray_` alone leaves open); see `banded` and `bandPass` for why no
-- cell is read before it is written.
table :: Int -> ST s (STUArray s Int Int)
table n = newArray_ (0, max 0 (n - 1))
