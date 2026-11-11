-- | A second spelling of the coverage rule (v2.35 imports lane, R2),
-- written apart from CE.Dedup.Cover.Rule on purpose: a cover is a SET
-- OF TOKEN POSITIONS keyed by (file, position) — so a same-file pair
-- shares its cover without a branch — the order is a repeated pick of
-- the greatest remaining run, and a run's own tokens are counted one
-- position at a time. No interval, no map of spans, no sort.
module ReferenceCover (Row, refClasses, keptInOrder, uncoveredWhenKept) where

import Data.List (maximumBy, (\\))
import qualified Data.Set as S

-- | (fileA, fileB, a0, b0, len, distinct)
type Row = (Integer, Integer, Integer, Integer, Integer, Integer)

-- | 0 reported, 1 suppressed, 2 dropped — every row's code, in row order.
refClasses :: Integer -> Integer -> [Row] -> [Int]
refClasses t d rows = [maybe 2 id (lookup i decided) | i <- [0 .. length rows - 1]]
 where
  decided = concatMap (decidePair t d) (pairsOf (zip [0 ..] rows))

-- | The rows of each pair, pairs in first-seen order.
pairsOf :: [(Int, Row)] -> [[(Int, Row)]]
pairsOf [] = []
pairsOf xs@((_, (fa, fb, _, _, _, _)) : _) = mine : pairsOf rest
 where
  mine = [x | x@(_, (a, b, _, _, _, _)) <- xs, a == fa, b == fb]
  rest = [x | x@(_, (a, b, _, _, _, _)) <- xs, a /= fa || b /= fb]

-- | One pair: pick the greatest remaining run (diverse before not, then
-- longer, then the smaller a0, then the smaller b0), test it against the
-- positions taken so far, and go on.
decidePair :: Integer -> Integer -> [(Int, Row)] -> [(Int, Int)]
decidePair t d = walk S.empty
 where
  walk _ [] = []
  walk taken left =
    let pick@(i, r) = maximumBy (\x y -> compare (rank x) (rank y)) left
        mine = own taken r
        keep = max (fst mine) (snd mine) >= t
        code = if not keep then 2 else if diverse r then 0 else 1
        taken' = if keep then S.union taken (positions r) else taken
     in (i, code) : walk taken' (left \\ [pick])
  diverse (_, _, _, _, _, k) = k >= d
  rank (_, r@(_, _, a0, b0, n, _)) = (diverse r, n, negate a0, negate b0)

-- | The positions a run covers on both sides.
positions :: Row -> S.Set (Integer, Integer)
positions (fa, fb, a0, b0, n, _) =
  S.fromList ([(fa, p) | p <- [a0 .. a0 + n - 1]] <> [(fb, p) | p <- [b0 .. b0 + n - 1]])

-- | The run's tokens outside `taken`, side a and side b.
own :: S.Set (Integer, Integer) -> Row -> (Integer, Integer)
own taken (fa, fb, a0, b0, n, _) = (free fa a0, free fb b0)
 where
  free f s = toInteger (length [p | p <- [s .. s + n - 1], S.notMember (f, p) taken])

-- | The kept runs of one pair in the order the rule met them, each with
-- its own tokens at that moment (the non-redundancy witness).
keptInOrder :: Integer -> Integer -> [Row] -> [(Row, (Integer, Integer))]
keptInOrder t d rows = go S.empty (ordered rows)
 where
  ordered [] = []
  ordered xs = let m = maximumBy (\x y -> compare (key x) (key y)) xs in m : ordered (xs \\ [m])
  key r@(_, _, a0, b0, n, k) = (k >= d, n, negate a0, negate b0, r)
  go _ [] = []
  go taken (r : rs)
    | max (fst o) (snd o) >= t = (r, o) : go (S.union taken (positions r)) rs
    | otherwise = go taken rs
   where
    o = own taken r

-- | Every run's own tokens against the cover of ALL kept runs (the
-- completeness witness: a run not kept leaves fewer than t on each side).
uncoveredWhenKept :: [Row] -> Row -> (Integer, Integer)
uncoveredWhenKept kept = own (S.unions (map positions kept))
