-- | The banded distance's battery (plan v2.35 C7a): `tedWithin k a b`
-- answers `ted a b` cut at k — Just the distance when it is at most k,
-- Nothing otherwise. On the exhaustive small-tree family for every k
-- from 0 to the sum of the sizes, against the mapping-definition brute
-- force (ReferenceTed, which shares nothing with CE.Clone.Within); on
-- the two hundred generated pairs and two near copies of each (every third
-- or every twelfth label changed: outside and inside the band), and on
-- trees at the node ceiling (a 256-node chain and a 256-node fan,
-- relabelled across the ceiling), against the production ted; and named
-- cases: k = 0
-- admits equal trees only, a pair exactly at k is answered, a pair at
-- k + 1 is not.
module CloneWithinProps (battery) where

import CE.Clone.Cost (unitNodeCap)
import CE.Clone.Prefilter (cloneCeiling)
import CE.Clone.Ted (Tree, ted)
import CE.Clone.Within (tedWithin)
import CloneNamedTrees (fork, one, other)
import ReferenceTed (refTed)
import ReferenceTreeGen (treePairs)

type T = ([Int], [Int])

battery :: (T -> Tree) -> [T] -> IO Bool
battery toTree fam = do
  let within k x y = tedWithin k (toTree x) (toTree y)
      exact x y = ted (toTree x) (toTree y)
      gen = concat [[(x, y), (x, near 3 x), (x, near 12 x)] | (x, y) <- treePairs]
  a <- check "within: tedWithin k ≡ brute-force ted cut at k, every k ≤ n1 + n2 (whole family)" (and [within k x y == cut k (refTed x y) | x <- fam, y <- fam, k <- [0 .. size x + size y]])
  b <- check "within: 600 generated pairs (200 + two near copies each), ≡ production ted cut at k" (all (agrees within exact) gen)
  c <- check "within: trees at the node ceiling (chain, fan; relabelled across the ceiling)" (all (agrees within exact) capped)
  d <- check "within: named cases (k = 0 equal trees only, a pair exactly at k, a pair at k + 1)" (named within)
  e <- check "within: the generated pairs reach both answers" (any (inside exact) gen && any (not . inside exact) gen)
  pure (a && b && c && d && e)

check :: String -> Bool -> IO Bool
check name ok = putStrLn ((if ok then "ok   " else "FAIL ") <> name) >> pure ok

size :: T -> Int
size = length . fst

cut :: Int -> Integer -> Maybe Integer
cut k d = if d <= toInteger k then Just d else Nothing

-- | Every k where the answer can turn: 0, either side of the distance,
-- the pair's clone ceiling, the sum of the sizes.
agrees :: (Int -> T -> T -> Maybe Integer) -> (T -> T -> Integer) -> (T, T) -> Bool
agrees within exact (x, y) = and [within k x y == cut k d | k <- ks]
 where
  d = exact x y
  ks = filter (>= 0) [0, fromInteger d - 1, fromInteger d, fromInteger d + 1, cloneCeiling (max (size x) (size y)), size x + size y]

-- | The pair's distance is inside its clone ceiling.
inside :: (T -> T -> Integer) -> (T, T) -> Bool
inside exact (x, y) = exact x y <= toInteger (cloneCeiling (max (size x) (size y)))

-- | Every m-th label changed (shape kept), so the distance is at most
-- n / m.
near :: Int -> T -> T
near m (labs, llds) = ([if i `mod` m == 0 then l + 7 else l | (i, l) <- zip [0 :: Int ..] labs], llds)

-- | A chain and a fan of unitNodeCap nodes, each against copies with 0, 1,
-- ceiling, ceiling + 1 and ceiling + 2 labels changed, and against each
-- other.
capped :: [(T, T)]
capped = [(t, relabel r t) | t <- [chain, fan], r <- [0, 1, c, c + 1, c + 2]] <> [(chain, fan)]
 where
  n = fromInteger unitNodeCap
  c = cloneCeiling n
  chain = (map (`mod` 4) [0 .. n - 1], replicate n 0)
  fan = (map (`mod` 4) [0 .. n - 1], [0 .. n - 2] <> [0])
  relabel r (labs, llds) = ([if i < r then l + 9 else l | (i, l) <- zip [0 ..] labs], llds)

named :: (Int -> T -> T -> Maybe Integer) -> Bool
named within =
  within 0 one one == Just 0
    && within 0 one other == Nothing
    && within 1 one other == Just 1
    && within 3 fork wide == Just 3
    && within 2 fork wide == Nothing
    && within 0 fork fork == Just 0
 where
  -- one / other: distance 1 (a relabel); fork / wide: distance 3 (three
  -- leaves inserted)
  wide = ([1, 1, 1, 0, 0, 0], [0, 1, 2, 3, 4, 0])
