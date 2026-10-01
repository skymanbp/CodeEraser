-- | Independent references for the arch judgment (plan v2.31 step 8),
-- each written another way than the product. The product finds the
-- feedback arc set by a programme over vertex subsets; the first
-- reference tries every ARC subset whose complement is acyclic, the
-- second walks every PERMUTATION of the vertices and takes its
-- backward arcs — two other algorithms, both under the product's tie
-- rule (least weight, then fewest arcs, then the least sorted arc
-- list), so the legs compare whole cuts, not only weights. The impact
-- is a whole-set fixpoint, where the product walks a frontier.
-- Modularity is the double sum over vertex pairs, where the product
-- only ever compares gains.
module ReferenceArch (acyclicRef, closure, fasByPermutation, fasBySubsets, modularity4m2) where

import Data.Array (listArray, (!))
import Data.List (permutations, sortOn, subsequences)
import qualified Data.Map.Strict as M
import qualified Data.IntSet as IS
import ReferenceGraph (reachB)

type Weighted = [((Int, Int), Integer)]

-- | The tie rule as a key over a cut listed ascending.
ranked :: Weighted -> (Integer, Int, Weighted)
ranked cut = (sum (map snd cut), length cut, cut)

-- | The least cut among the arc subsets whose complement is acyclic.
fasBySubsets :: Weighted -> Weighted
fasBySubsets arcs = snd (minimum [(ranked cut, cut) | cut <- subsequences sorted, acyclicRef [e | (e, _) <- sorted, e `notElem` map fst cut]])
 where
  sorted = sortOn fst arcs

-- | The least backward set over every order of the k vertices.
fasByPermutation :: Int -> Weighted -> Weighted
fasByPermutation k arcs = snd (minimum [(ranked cut, cut) | order <- permutations [0 .. k - 1], let cut = backward order])
 where
  backward order =
    let at = M.fromList (zip order [0 :: Int ..])
     in sortOn fst [arc | arc@((a, b), _) <- arcs, at M.! a > at M.! b]

-- | No arc (a, b) whose head reaches its tail.
acyclicRef :: [(Int, Int)] -> Bool
acyclicRef arcs = and [not (IS.member a (reachB arcs [b])) | (a, b) <- arcs]

-- | The impact as a fixpoint over the whole reached set: round r adds
-- every file referencing a file already in, and a file's depth is the
-- round it first appears in. `refs` holds (referrer, referenced).
closure :: [(Int, Int)] -> [Int] -> [(Int, Integer)]
closure refs focus = M.toList (go 0 (M.fromList [(f, 0) | f <- focus]))
 where
  go :: Integer -> M.Map Int Integer -> M.Map Int Integer
  go r got
    | M.size grown == M.size got = got
    | otherwise = go (r + 1) grown
   where
    grown = M.union got (M.fromList [(f, r + 1) | (f, g) <- refs, M.member g got])

-- | 4m² · Q for a partition, by the double sum over ordered vertex
-- pairs in one community: Σ (2m · A_ij − k_i · k_j). The rows are
-- the directed file edges; A is their symmetric sum.
modularity4m2 :: Int -> [(Int, Int, Integer)] -> [Int] -> Integer
modularity4m2 n rows part = sum [2 * m * a i j - k i * k j | i <- [0 .. n - 1], j <- [0 .. n - 1], label i == label j]
 where
  a i j = sum [w | (f, g, w) <- rows, (f, g) == (i, j) || (f, g) == (j, i)]
  k i = sum [a i j | j <- [0 .. n - 1]]
  m = sum [w | (_, _, w) <- rows]
  labels = listArray (0, n - 1) part
  label i = labels ! i :: Int
