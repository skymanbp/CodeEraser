-- | The sequence bound's battery (plan v2.35 C1, beside the two
-- older bounds' legs in CloneProps): over the exhaustive small-tree
-- family the postorder and preorder label-sequence distances never
-- exceed the brute-force tree edit distance; the banded test answers
-- what the full-matrix distance compared with k answers, for every k
-- from 0 to the sum of the sizes; the clone ceiling is the clone
-- verdict's threshold; the bound rejects no clone under any threshold
-- the battery probes, and both orders decide something. The same on
-- two hundred generated pairs (ReferenceTreeGen) against the
-- production ted, and four named cases. The references here share
-- nothing with CE.Clone.Sequence: the distance is the textbook full
-- matrix over lists, the preorder a sort by the ancestor / left
-- relation read straight off lld.
module CloneSeqProps (battery) where

import CE.Clone.Cost (cloneDecidesWith, tsedDen, tsedNum)
import CE.Clone.Prefilter (cloneCeiling, cloneCeilingWith, provablyBelow, reachFloor)
import CE.Clone.Sequence (labelsOver, sequenceOver)
import CE.Clone.Ted (Tree (..), ted)
import Data.Array.Unboxed (UArray, elems, listArray)
import Data.List (sortBy)
import ReferenceTed (refTed)
import ReferenceTreeGen (treePairs)

type T = ([Int], [Int])

-- | The thresholds the clone battery already probes.
thresholds :: [(Integer, Integer)]
thresholds = [(tsedNum, tsedDen), (75, 100), (86, 100)]

battery :: (T -> Tree) -> [T] -> IO Bool
battery toTree fam = do
  let refs = [(refTed a b, a, b) | a <- fam, b <- fam]
  a <- check "sequence: preorder ≡ the ancestor/left order (family + generated)" (all (preorderOk toTree) (fam <> concatMap pairList treePairs))
  b <- check "sequence: post- and preorder distance ≤ brute-force ted (whole family)" (all bounded refs)
  c <- check "sequence: banded test ≡ full matrix > k, every k ≤ n1 + n2 (family, both orders)" (and [banded k x y | (_, x, y) <- refs, k <- [0 .. size x + size y]])
  d <- check "sequence: t ≤ cloneCeiling mx ⇔ cloneDecides ⇔ t ≤ mx − reachFloor mx" ceilingOk
  e <- check "sequence: the bound rejects no clone at 85/100, 75/100, 86/100 (family)" (and [admissible toTree th r | th <- thresholds, r <- refs])
  f <- check "sequence: a third bound (rejects pairs size and label bounds leave) and both orders decide" (alive toTree refs)
  g <- check "sequence: 200 generated pairs, bound ≤ ted and banded ≡ full matrix" (all (generated toTree) treePairs)
  h <- check "sequence: named cases (single nodes, relabel-only, insert-only, k = 0)" (named toTree fam)
  i <- check "sequence: the ceiling never falls as the threshold falls (θ grid 100 … 70, mx ≤ 600)" ceilingGrows
  pure (a && b && c && d && e && f && g && h && i)

check :: String -> Bool -> IO Bool
check name ok = putStrLn ((if ok then "ok   " else "FAIL ") <> name) >> pure ok

pairList :: (T, T) -> [T]
pairList (x, y) = [x, y]

size :: T -> Int
size = length . fst

arr :: [Int] -> UArray Int Int
arr xs = listArray (0, length xs - 1) xs

-- | Full-matrix unit-cost edit distance of two label lists.
refSeq :: [Int] -> [Int] -> Int
refSeq xs ys = last (foldl row [0 .. length ys] xs)
 where
  row prev x = scanl (next x) (firstOf prev + 1) (zip3 ys prev (drop 1 prev))
  next x left (y, diag, up) = minimum [left + 1, up + 1, diag + fromEnum (x /= y)]
  firstOf = foldr const 0

-- | Preorder by definition: u before v iff u is an ancestor of v, or
-- neither is the other's ancestor and u comes first in postorder (u
-- lies to the left).
refPre :: T -> [Int]
refPre (lab, lld) = map (lab !!) (sortBy before [0 .. length lab - 1])
 where
  ancestor u v = lld !! u <= v && v < u
  before u v
    | ancestor u v = LT
    | ancestor v u = GT
    | otherwise = compare u v

preorderOk :: (T -> Tree) -> T -> Bool
preorderOk toTree t = elems (tPre (toTree t)) == refPre t

bounded :: (Integer, T, T) -> Bool
bounded (v, x, y) = toInteger (refSeq (fst x) (fst y)) <= v && toInteger (refSeq (refPre x) (refPre y)) <= v

-- | The banded test against the full matrix at k, in both orders.
banded :: Int -> T -> T -> Bool
banded k x y =
  labelsOver k (arr (fst x)) (arr (fst y)) == (refSeq (fst x) (fst y) > k)
    && labelsOver k (arr (refPre x)) (arr (refPre y)) == (refSeq (refPre x) (refPre y) > k)

-- | Every size up to 600 against every distance up to past twice it,
-- under each probed threshold: the ceiling reads the verdict's own
-- comparison, and at the shipped threshold it is mx − reachFloor mx.
ceilingOk :: Bool
ceilingOk =
  and
    [ (t <= cloneCeilingWith th mx) == cloneDecidesWith th t mx n2
    | th <- thresholds
    , mx <- [1 .. 600]
    , n2 <- [1, mx]
    , t <- [0 .. 2 * mx + 1]
    ]
    && all (\mx -> cloneCeiling mx == mx - reachFloor mx) [0 .. 600 :: Integer]
    && all (\mx -> toInteger (cloneCeiling (fromInteger mx :: Int)) == cloneCeiling mx) [0 .. 600]

-- | The T3 precision instrument's θ grid (eval_t3_precision_parts
-- THETA_GRID), falling: a core built at the grid's lowest threshold
-- keeps the distance of every pair that is a clone anywhere on the grid,
-- because the ceiling at a lower threshold is never below the ceiling at
-- a higher one.
ceilingGrows :: Bool
ceilingGrows =
  and
    [ cloneCeilingWith (hi, 100) mx <= cloneCeilingWith (lo, 100) mx
    | mx <- [1 .. 600 :: Integer]
    , (hi, lo) <- zip grid (drop 1 grid)
    ]
 where
  grid = [100, 95, 90, 85, 80, 75, 70]

-- | The pair is either not rejected at the threshold's ceiling or not
-- a clone under that threshold.
admissible :: (T -> Tree) -> (Integer, Integer) -> (Integer, T, T) -> Bool
admissible toTree th (v, x, y) =
  not (sequenceOver (cloneCeilingWith th (max (size x) (size y))) (toTree x) (toTree y))
    || not (cloneDecidesWith th v (toInteger (size x)) (toInteger (size y)))

-- | Non-vacuity: at the shipped ceiling the bound rejects a pair the
-- size and label bounds leave; and over some k each order rejects a
-- pair the other order does not.
alive :: (T -> Tree) -> [(Integer, T, T)] -> Bool
alive toTree refs =
  or [not (provablyBelow (fst x) (fst y)) && sequenceOver (cloneCeiling (max (size x) (size y))) (toTree x) (toTree y) | (_, x, y) <- refs]
    && or [post k x y && not (pre k x y) | (_, x, y) <- refs, k <- [0 .. 3]]
    && or [pre k x y && not (post k x y) | (_, x, y) <- refs, k <- [0 .. 3]]
 where
  post k x y = labelsOver k (tLab (toTree x)) (tLab (toTree y))
  pre k x y = labelsOver k (tPre (toTree x)) (tPre (toTree y))

-- | One generated pair: both bounds at most the production ted, and the
-- banded test equal to the full matrix for every k up to three past
-- the larger distance, and at the sum of the sizes.
generated :: (T -> Tree) -> (T, T) -> Bool
generated toTree (x, y) =
  toInteger (max dPost dPre) <= ted (toTree x) (toTree y)
    && and [banded k x y | k <- [0 .. max dPost dPre + 3] <> [size x + size y]]
 where
  dPost = refSeq (fst x) (fst y)
  dPre = refSeq (refPre x) (refPre y)

-- | Two single-node trees (equal labels within 0, different ones only
-- within 1); a relabel-only pair and an insert-only pair, each at
-- distance 1 (over at k = 0, within at k = 1); k = 0 admits every
-- family tree against itself.
named :: (T -> Tree) -> [T] -> Bool
named toTree fam =
  not (over 0 one one)
    && over 0 one other
    && not (over 1 one other)
    && all (exactlyOne . pairOf) [(fork, relabelled), (stick, fork)]
    && all (\t -> not (over 0 t t)) fam
 where
  over k x y = sequenceOver k (toTree x) (toTree y)
  pairOf (x, y) = (x, y, ted (toTree x) (toTree y))
  exactlyOne (x, y, d) = d == 1 && over 0 x y && not (over 1 x y)
  (one, other) = (([0], [0]), ([1], [0]))
  fork = ([0, 0, 0], [0, 1, 0])
  relabelled = ([0, 1, 0], [0, 1, 0])
  stick = ([0, 0], [0, 0])
