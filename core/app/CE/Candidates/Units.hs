-- | The admitted units of a candidates.request as flat unboxed columns
-- (plan v2.33 W3): language, file, key, node count and line span per
-- unit, and every kind histogram in one array addressed by offsets —
-- the walks over hundreds of thousands of pairs (CE.Candidates.T3) and
-- the line anchors (CE.Candidates.Sources) read O(1) cells, never a
-- list.
module CE.Candidates.Units (Units (..), columns, endOf, interUpTo, nodesOf, startOf) where

import Control.Monad.ST (ST, runST)
import Data.Array.Base (unsafeAt, unsafeFreeze)
import Data.Array.ST (STUArray, newArray, writeArray)
import Data.Array.Unboxed (UArray)

-- | One request's units, columnar. Unit i's histogram entries are
-- entries [off i, off (i + 1)) of `kinds` / `counts` (kinds ascending);
-- `rest` holds, per entry, the sum of the counts from that entry to the
-- end of its unit's histogram.
data Units = Units
  { count :: !Int
  , lang :: !(UArray Int Int)
  , file :: !(UArray Int Int)
  , key :: !(UArray Int Int)
  , nodes :: !(UArray Int Int)
  , start :: !(UArray Int Int)
  , end :: !(UArray Int Int)
  , off :: !(UArray Int Int)
  , kinds :: !(UArray Int Int)
  , counts :: !(UArray Int Int)
  , rest :: !(UArray Int Int)
  }

-- | The number of leading scalar columns of a unit row.
heads :: Int
heads = 6

-- | The columns of the unit rows `[lang, file, key, nodes, start, end,
-- kind, count, …]` (the contract admitted every row), written in one
-- pass over the rows into arrays sized up front.
columns :: [[Integer]] -> Units
columns rows = runST $ do
  ls <- newArray (0, n - 1) 0
  fs <- newArray (0, n - 1) 0
  ys <- newArray (0, n - 1) 0
  ns <- newArray (0, n - 1) 0
  ss <- newArray (0, n - 1) 0
  ds <- newArray (0, n - 1) 0
  offs <- newArray (0, n) 0
  ks <- newArray (0, total - 1) 0
  cs <- newArray (0, total - 1) 0
  rs <- newArray (0, total - 1) 0
  let fill _ _ [] = pure ()
      fill i e (row : more) = do
        let ints = map fromInteger row :: [Int]
            hist = pairs (drop heads ints)
            e' = e + length hist
        sequence_ [writeArray h i x | (h, x) <- zip [ls, fs, ys, ns, ss, ds] ints]
        writeArray offs (i + 1) e'
        sequence_
          [ writeArray ks j k >> writeArray cs j c >> writeArray rs j r
          | (j, (k, c), r) <- zip3 [e ..] hist (scanr1 (+) (map snd hist))
          ]
        fill (i + 1) e' more
  fill 0 0 rows
  Units n
    <$> freeze' ls
    <*> freeze' fs
    <*> freeze' ys
    <*> freeze' ns
    <*> freeze' ss
    <*> freeze' ds
    <*> freeze' offs
    <*> freeze' ks
    <*> freeze' cs
    <*> freeze' rs
 where
  n = length rows
  total = sum [(length row - heads) `div` 2 | row <- rows]
  pairs xs = case xs of
    (k : c : more) -> (k, c) : pairs more
    _ -> []
  freeze' :: STUArray s Int Int -> ST s (UArray Int Int)
  freeze' = unsafeFreeze

nodesOf, startOf, endOf :: Units -> Int -> Int
nodesOf us i = nodes us `unsafeAt` i
startOf us i = start us `unsafeAt` i
endOf us i = end us `unsafeAt` i

-- | I = Σ_kind min(c1, c2) between units a and b — one merge walk over
-- the two ascending histograms — or, as soon as the counts left on the
-- shorter side cannot lift the running sum to `floor`, that sum plus
-- those counts: a number that is below `floor` exactly when I is. The
-- label bound only asks whether I reaches the floor.
interUpTo :: Units -> Int -> Int -> Int -> Int
interUpTo us floor_ a b = go (off us `unsafeAt` a) (off us `unsafeAt` b) 0
 where
  ea = off us `unsafeAt` (a + 1)
  eb = off us `unsafeAt` (b + 1)
  k = unsafeAt (kinds us)
  c = unsafeAt (counts us)
  r = unsafeAt (rest us)
  go :: Int -> Int -> Int -> Int
  go !i !j !acc
    | i >= ea || j >= eb = acc
    | reach < floor_ = reach
    | k i < k j = go (i + 1) j acc
    | k i > k j = go i (j + 1) acc
    | otherwise = go (i + 1) (j + 1) (acc + min (c i) (c j))
   where
    reach = acc + min (r i) (r j)
