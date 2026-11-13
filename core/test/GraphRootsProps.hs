-- | Roots and dependents of the dead file set (plan v2.35, the dead-file
-- amendment, phase 5A): CE.Graph.Roots against an independent reference,
-- exhaustively over a small family. The family: every arc set on four
-- nodes without self-loops (2^12 = 4096), the entry set empty or one
-- node (5), the file nodes any subset (16) -- 327,680 graphs. It holds a
-- chain, a dead cycle nothing outside supports, a dependent with two
-- roots and a path through a non-file node (the coverage leg names each).
--
-- The reference is spelled apart from the product on purpose: no
-- component algorithm, the support relation is the transitive closure of
-- the arc relation as a boolean matrix (Warshall's triple loop), and the
-- definitions are read off it word for word -- f is a root when every
-- dead file that supports it is supported by it; a dependent's roots are
-- the roots that support it.
module GraphRootsProps (battery) where

import CE.Graph.Build (build, reachFrom)
import CE.Graph.Cost (underCap)
import qualified CE.Graph.Roots as Roots
import Data.Bits (testBit)
import Data.Maybe (fromMaybe)
import WireHarness (runLegs)

-- | One graph of the family: its arcs, its entries, its file nodes.
data Case = Case {kArcs :: [(Int, Int)], kEntries :: [Int], kFiles :: [Int]}

battery :: IO Bool
battery =
  runLegs
    [ "roots equal the reference's (327,680 graphs)"
    , "pairs equal the reference's"
    , "every dependent has a root"
    , "no dead file iff no root"
    , "a dead file with no kept in-arc from any other node is a root"
    , "under a forced cap of 0 the pairs drop whole and the roots stay"
    , "rows strictly ascending"
    , "the family holds a chain, a dead cycle, two roots, a non-file hop"
    , "a self-arc on every node changes nothing"
    ]
    [ every (\c -> product' c == reference c)
    , every (\c -> Roots.gUnder (grouped underCap c) == Just (snd (reference c)))
    , every (\c -> let (rs, ps) = reference c in all (\f -> any ((== f) . fst) ps) [f | f <- dead c, f `notElem` rs])
    , every (\c -> null (dead c) == null (fst (reference c)))
    , every (\c -> all (`elem` fst (reference c)) [f | f <- dead c, not (any (\(s, d) -> d == f && s /= f) (kArcs c))])
    , every capped
    , every (\c -> let g = grouped underCap c in ascending (Roots.gRoots g) && maybe False ascending (Roots.gUnder g))
    , all (\shape -> any shape cases) [chain, deadCycle, twoRoots, nonFileHop]
    , every (\c -> same (grouped underCap c) (grouped underCap c {kArcs = kArcs c <> [(v, v) | v <- [0 .. 3]]}))
    ]
 where
  every p = all p cases
  same g h = Roots.gRoots g == Roots.gRoots h && Roots.gUnder g == Roots.gUnder h
  product' c = let g = grouped underCap c in (Roots.gRoots g, fromMaybe [] (Roots.gUnder g))
  capped c =
    let g = grouped 0 c
        (rs, ps) = reference c
     in Roots.gRoots g == rs && (if null ps then Roots.gUnder g == Just [] || Roots.gUnder g == Nothing else Roots.gUnder g == Nothing)

cases :: [Case]
cases =
  [ Case [a | (i, a) <- zip [0 ..] arcPool, testBit m i] e [v | v <- [0 .. 3], testBit fs v]
  | m <- [0 .. 4095 :: Int]
  , e <- [] : [[v] | v <- [0 .. 3]]
  , fs <- [0 .. 15 :: Int]
  ]
 where
  arcPool = [(s, d) | s <- [0 .. 3], d <- [0 .. 3], s /= d]

-- | The product: CE.Graph.Build's graph and reach, then Roots.
grouped :: Integer -> Case -> Roots.Grouping
grouped cap c = Roots.group cap b reach (dead c)
 where
  b = build 5 [] 4 [[toInteger s, toInteger d, 0, 1] | (s, d) <- kArcs c]
  reach = reachFrom b (kEntries c)

-- | The dead files: not reached, by the matrix.
dead :: Case -> [Int]
dead c = [f | f <- kFiles c, not (any (\e -> e == f || closure c !! e !! f) (kEntries c))]

-- | The transitive closure of the arc relation (paths of length >= 1).
closure :: Case -> [[Bool]]
closure c = foldl through start [0 .. 3]
 where
  start = [[(i, j) `elem` kArcs c | j <- [0 .. 3]] | i <- [0 .. 3 :: Int]]
  through m k = [[m !! i !! j || (m !! i !! k && m !! k !! j) | j <- [0 .. 3]] | i <- [0 .. 3]]

-- | The reference's roots and (dependent, root) pairs, ascending.
reference :: Case -> ([Int], [(Int, Int)])
reference c = (roots, [(f, r) | f <- ds, f `notElem` roots, r <- roots, supports r f])
 where
  ds = dead c
  m = closure c
  supports g f = g /= f && m !! g !! f
  roots = [f | f <- ds, all (\g -> not (supports g f) || supports f g) ds]

ascending :: Ord a => [a] -> Bool
ascending xs = and (zipWith (<) xs (drop 1 xs))

-- | The shapes the family must hold (read off the reference).
chain, deadCycle, twoRoots, nonFileHop :: Case -> Bool
chain c = let (rs, ps) = reference c in length (dead c) == 3 && length rs == 1 && length ps == 2 && length (kArcs c) == 2
deadCycle c = let (rs, _) = reference c in length rs >= 2 && all (\r -> any (\g -> g /= r && closure c !! r !! g && closure c !! g !! r) rs) rs
twoRoots c = any (\f -> length [() | (d, _) <- snd (reference c), d == f] >= 2) (dead c)
nonFileHop c =
  any
    (\(f, r) -> not ((r, f) `elem` kArcs c) && any (\v -> v `notElem` kFiles c && (r, v) `elem` kArcs c && (v, f) `elem` kArcs c) [0 .. 3])
    (snd (reference c))
