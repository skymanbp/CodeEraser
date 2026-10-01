-- | The arch battery's feedback-arc-set legs (plan v2.31 step 8, the
-- exact road widened to every component of at most fourteen
-- vertices): three graph families, each judged once by the product
-- and once by a reference written another way. The subset programme
-- against the arc-subset reference on every directed graph of four
-- vertices and two hundred seeded ones of five or six vertices and at
-- most twelve arcs; against the permutation reference on two hundred
-- of five to seven vertices and thirteen to twenty-four arcs, past
-- the subset reference's reach; the greedy road on two hundred
-- strongly connected graphs of fifteen or sixteen vertices, its cut
-- acyclic and minimal and its weight set against the programme's
-- minimum — the ratio printed, never asserted (Eades–Lin–Smyth is a
-- heuristic with no constant bound). Then the layers against the
-- cuts over all three, the fourteen-vertex line, and the merge rule
-- the programme's optimal substructure rests on. The programme's own
-- wall time on two fourteen-vertex graphs is printed once.
module ArchFasProps (draw, fasLegs, ordered, stream) where

import CE.Arch.Fas (exactCut, fas)
import CE.Arch.Layers (levels)
import Control.Exception (evaluate)
import Data.List (maximumBy, sort)
import qualified Data.Map.Strict as M
import Data.Ord (comparing)
import qualified Data.Set as S
import ReferenceArch (acyclicRef, fasByPermutation, fasBySubsets)
import System.CPUTime (getCPUTime)

type Weighted = (Int, [((Int, Int), Integer)])

-- | A graph, the product's cut rows, and the reference's cut.
data Judged = Judged {graphOf :: Weighted, cutRows :: [(Int, Int, Integer, Integer)], refCut :: [((Int, Int), Integer)]}

judgeBy :: (Weighted -> [((Int, Int), Integer)]) -> Weighted -> Judged
judgeBy ref g@(_, arcs) = Judged g (fas (M.fromList arcs)) (ref g)

bySubsets, byPermutation, greedyRoad :: [Judged]
bySubsets = map (judgeBy (fasBySubsets . snd)) smallGraphs
byPermutation = map (judgeBy (uncurry fasByPermutation)) mediumGraphs
greedyRoad = map (judgeBy (uncurry exactCut)) strongGraphs

fasLegs :: IO ([String], [Bool])
fasLegs = do
  timed "a 14-vertex tournament (91 arcs, weight 1)" (tournament 14)
  timed "the 14-vertex complete digraph (182 arcs, weight 1)" (14, [(p, 1) | p <- ordered 14])
  mapM_ ratio [("arc-subset family", bySubsets), ("permutation family", byPermutation), ("greedy family", greedyRoad)]
  pure (names, probes)

names :: [String]
names =
  [ "the subset programme's cut equals the arc-subset reference's, tie rule included: all 4,096 four-vertex graphs and 200 seeded ones"
  , "the subset programme's cut equals the permutation reference's on 200 seeded graphs of 5 to 7 vertices and 13 to 24 arcs"
  , "the greedy cut on 200 strongly connected graphs of 15 or 16 vertices is acyclic and minimal (its ratio to the minimum is printed)"
  , "with the cuts removed every kept arc runs from a higher level to a lower one"
  , "a component of at most 14 vertices is cut exactly, one of 15 or more greedily"
  , "merging one sorted arc set into two disjoint sorted sets of equal length keeps their order (200 seeded)"
  ]

probes :: [Bool]
probes =
  [ all agrees bySubsets
  , all agrees byPermutation
  , all (\j -> flagged 0 j && validCut j) greedyRoad
  , all layered (bySubsets <> byPermutation <> greedyRoad)
  , and [flagsOf k == [k <= 14] | k <- [2 .. 16]]
  , all mergeKeepsOrder [1 .. 200]
  ]

-- | The product's cut is the reference's, flagged exact, and valid.
agrees :: Judged -> Bool
agrees j = cutRows j == [(a, b, w, 1) | ((a, b), w) <- refCut j] && validCut j

-- | The cut is arcs of the graph, removing it leaves no cycle, and
-- each cut arc put back alone closes one.
validCut :: Judged -> Bool
validCut j = all (`elem` arcs) cut && acyclicRef kept && all (\(e, _) -> not (acyclicRef (e : kept))) cut
 where
  arcs = snd (graphOf j)
  cut = [((a, b), w) | (a, b, w, _) <- cutRows j]
  kept = [e | (e, _) <- arcs, e `notElem` map fst cut]

flagged :: Integer -> Judged -> Bool
flagged e j = all (\(_, _, _, x) -> x == e) (cutRows j)

layered :: Judged -> Bool
layered j = and [level a > level b | ((a, b), _) <- arcs, not (S.member (a, b) cut)]
 where
  (k, arcs) = graphOf j
  cut = S.fromList [(a, b) | (a, b, _, _) <- cutRows j]
  table = M.fromList [(d, l) | [d, l] <- levels k (M.fromList arcs) cut]
  level v = table M.! toInteger v

-- | The distinct exact flags on the ring of k vertices, as Bools.
flagsOf :: Int -> [Bool]
flagsOf k = S.toList (S.fromList [e == 1 | (_, _, _, e) <- fas (M.fromList [((v, (v + 1) `mod` k), 1) | v <- [0 .. k - 1]])])

-- | Sets A and B of equal size and X disjoint from both: A's sorted
-- list is below B's exactly when A ∪ X's is below B ∪ X's.
mergeKeepsOrder :: Int -> Bool
mergeKeepsOrder i = (sort a <= sort b) == (sort (a <> x) <= sort (b <> x))
 where
  (x, rest) = splitAt 3 (shuffled (ordered 6) (stream i))
  m = 1 + fromInteger (draw (toInteger i) `mod` 5)
  a = take m (shuffled rest (stream (i + 401)))
  b = take m (shuffled rest (stream (i + 809)))

-- | The largest weight ratio of a family's product cut to its
-- reference cut, with its graph.
ratio :: (String, [Judged]) -> IO ()
ratio (family, js) = putStrLn ("     " <> family <> " largest cut / minimum: " <> show top <> " / " <> show low <> " on " <> show (graphOf j))
 where
  pair k = (sum [w | (_, _, w, _) <- cutRows k], sum (map snd (refCut k)))
  j = maximumBy (comparing (\k -> let (p, q) = pair k in fromInteger p / (fromInteger q :: Rational))) [k | k <- js, snd (pair k) > 0]
  (top, low) = pair j

-- | The programme's CPU time on one graph, printed once.
timed :: String -> Weighted -> IO ()
timed label (k, arcs) = do
  start <- getCPUTime
  cutWeight <- evaluate (sum (map snd (exactCut k arcs)))
  end <- getCPUTime
  putStrLn ("     subset programme on " <> label <> ": cut " <> show cutWeight <> " in " <> show ((end - start) `div` 1000000000) <> " ms CPU")

-- | i → j when j follows i by one to six steps round the circle, and
-- the seven-step pairs from the lower id: one arc per pair, strongly
-- connected.
tournament :: Int -> Weighted
tournament k = (k, [((a, b), 1) | (a, b) <- ordered k, let d = (b - a) `mod` k, d < 7 || (d == 7 && a < b)])

-- | Every directed graph on four vertices, weight 1, then 200 seeded
-- graphs on five or six vertices with at most twelve arcs of weight
-- 1 to 3 — within the arc-subset reference's reach.
smallGraphs :: [Weighted]
smallGraphs = [(4, [(p, 1) | (i, p) <- zip [0 :: Int ..] (ordered 4), odd (code `div` (2 ^ i))]) | code <- [0 .. 4095 :: Integer]] <> map seeded [1 .. 200]
 where
  seeded i = (k, take 12 [(p, 1 + draw s `mod` 3) | (p, s) <- zip (ordered k) (stream i), draw (s + 1) `mod` 3 == 0])
   where
    k = 5 + fromInteger (draw (toInteger i) `mod` 2)

-- | 200 seeded graphs on five to seven vertices with 13 to 24 arcs
-- (as many as five vertices hold), weights 1 to 3.
mediumGraphs :: [Weighted]
mediumGraphs = map medium [1 .. 200]
 where
  medium i = (k, drawn k total i)
   where
    k = 5 + fromInteger (draw (toInteger i * 13) `mod` 3)
    total = min (k * (k - 1)) (13 + fromInteger (draw (toInteger i * 17) `mod` 12))

-- | 200 strongly connected graphs on 15 or 16 vertices: the cycle
-- 0 → 1 → … → 0 first, then arcs drawn from the rest up to a total
-- of 20 to 48, weights 1 to 3.
strongGraphs :: [Weighted]
strongGraphs = map strong [1 .. 200]
 where
  strong i = (k, [(p, 1 + draw s `mod` 3) | (p, s) <- zip (ring <> extra) (stream (i + 7))])
   where
    k = 15 + fromInteger (draw (toInteger i) `mod` 2)
    ring = [(v, (v + 1) `mod` k) | v <- [0 .. k - 1]]
    total = 20 + fromInteger (draw (toInteger i * 31) `mod` 29)
    extra = take (total - k) (shuffled (filter (`notElem` ring) (ordered k)) (stream (i + 3)))

-- | `total` distinct arcs on k vertices, weights 1 to 3.
drawn :: Int -> Int -> Int -> [((Int, Int), Integer)]
drawn k total i = [(p, 1 + draw s `mod` 3) | (p, s) <- zip (take total (shuffled (ordered k) (stream (i + 5)))) (stream (i + 9))]

-- | The ordered pairs of distinct vertices below k.
ordered :: Int -> [(Int, Int)]
ordered k = [(a, b) | a <- [0 .. k - 1], b <- [0 .. k - 1], a /= b]

-- | Draw without replacement, one pick per seed.
shuffled :: [a] -> [Integer] -> [a]
shuffled [] _ = []
shuffled _ [] = []
shuffled xs (s : ss) = case splitAt (fromInteger (draw s `mod` toInteger (length xs))) xs of
  (before, x : after) -> x : shuffled (before <> after) ss
  (before, []) -> before

-- | The seeded stream and its high bits.
stream :: Int -> [Integer]
stream i = drop 1 (iterate lcg (toInteger i * 7919))

lcg :: Integer -> Integer
lcg s = (s * 6364136223846793005 + 1442695040888963407) `mod` 18446744073709551616

draw :: Integer -> Integer
draw s = lcg s `div` 8589934592
