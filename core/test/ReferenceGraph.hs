-- | Exhaustive brute-force equivalence for the graph judgment
-- (design §6, 2g exit). Pass (a): EVERY directed graph on 4 vertices
-- — 2^16 arc subsets, self-loops included — under 3 fixed entry
-- sets; indegree, outdegree, reachability and the SCC partition must
-- match an independently written fixpoint / mutual-reachability
-- reference. Pass (b): every graph on 3 vertices (2^9) × every entry
-- subset (2^3), driven through the PRODUCTION flags→entryMask path —
-- the four-way verdict as a total function, the only pass covering
-- the entry-set dimension. Pass (c): every graph on 3 vertices with
-- each arc's rows drawn — a kind of 0 to 6 and a rung of 1 to 6, now
-- and then a second row of another kind on the same pair — built with
-- the shipped inert kinds and floor rung; the kept arc set and the
-- positions must match this module's own reading of booklet 06 (the
-- asset and unused ref-def kinds, 3 and 5, are inert; a rung past 5 is
-- dropped; a pair counts once however many rows carry it), and the
-- pass must meet every one of those drops. First 3 mismatches
-- reported (the Reference.hs convention). Lives in core/test/ (RG14:
-- the harness must never cost the app tree its 300-line gate).
module ReferenceGraph (equivalence, arcsOf, edgeRows, reachB) where

import CE.Graph.Build (Built (..), build, reachFrom)
import CE.Graph.Cost (assetKind, minRung, refdefKind)
import CE.Graph.Dead (entries, verdicts)
import CE.Graph.Position (positions)
import Data.Bits (testBit)
import Data.List (nub, sort)
import qualified Data.IntSet as IS
import qualified Data.Set as S
import ReferenceFlowGen (lcgStep)

-- | The graph coded by a bitmask over the n*n ordered pairs.
arcsOf :: Int -> Int -> [(Int, Int)]
arcsOf n code =
  [(i, j) | i <- [0 .. n - 1], j <- [0 .. n - 1], testBit code (i * n + j)]

-- | Wire rows, sorted per the boundary contract (kind 0, rung 1).
edgeRows :: [(Int, Int)] -> [[Integer]]
edgeRows arcs = [[toInteger s, toInteger d, 0, 1] | (s, d) <- sort arcs]

-- | Independent reachability: naive fixpoint over the raw arc list.
reachB :: [(Int, Int)] -> [Int] -> IS.IntSet
reachB arcs seeds = go (IS.fromList seeds)
 where
  go s =
    let s' = IS.union s (IS.fromList [d | (a, d) <- arcs, IS.member a s])
     in if s' == s then s else go s'

-- | Independent SCC partition: mutual-reachability classes.
sccB :: Int -> [(Int, Int)] -> S.Set (S.Set Int)
sccB n arcs =
  S.fromList
    [ S.fromList [j | j <- [0 .. n - 1], mutual i j]
    | i <- [0 .. n - 1]
    ]
 where
  mutual i j =
    IS.member j (reachB arcs [i]) && IS.member i (reachB arcs [j])

-- | Pass (a): positions + SCC partition on every 4-vertex graph
-- under entry sets {}, {0}, {0,2}. Entry flags travel the production
-- path (bit 1 vs mask 2).
checkA :: Int -> [Int] -> Maybe String
checkA code es
  | placed 4 (build 5 [3, 5] 4 (edgeRows arcs)) arcs es = Nothing
  | otherwise =
      Just ("passA code=" <> show code <> " entries=" <> show es)
 where
  arcs = arcsOf 4 code

-- | The built graph's positions and SCC partition against the
-- reference's over the arcs it should have kept.
placed :: Int -> Built -> [(Int, Int)] -> [Int] -> Bool
placed n b arcs es = rowsOK && sccProd == sccWant
 where
  flagses = [if i `elem` es then 2 else 0 | i <- [0 .. n - 1]]
  rows = positions b (reachFrom b (entries 2 flagses)) [0 .. toInteger n - 1]
  reach = reachB arcs es
  sccProd = S.fromList (map S.fromList (bScc b))
  sccWant = sccB n arcs
  rowsOK = and (zipWith rowOK [0 ..] rows)
  rowOK i [p, indeg, outdeg, _sccId, sccSize, reachIn] =
    p == toInteger i
      && indeg == count (\(_, d) -> d == i)
      && outdeg == count (\(s, _) -> s == i)
      && sccSize == sizeOfB i
      && reachIn == toInteger (fromEnum (IS.member i reach))
  rowOK _ _ = False
  count f = toInteger (length (filter f arcs))
  -- exactly one class contains i, so the sum IS that class's size
  sizeOfB i = toInteger (sum [S.size c | c <- S.toList sccWant, S.member i c])

-- | Pass (b): the four-way verdict on every 3-vertex graph × every
-- entry subset; node 1 is public (bit 0), entries via bit 1. The
-- reference recomputes reach and the referenced set from the raw
-- arcs — the verdict formula itself IS the spec on both sides.
checkB :: Int -> Int -> Maybe String
checkB code emask
  | got == want = Nothing
  | otherwise =
      Just ("passB code=" <> show code <> " entryMask=" <> show emask)
 where
  arcs = arcsOf 3 code
  b = build 5 [3, 5] 3 (edgeRows arcs)
  flagOf i = (if odd i then 1 else 0) + (if testBit emask i then 2 else 0)
  flagses = [flagOf i | i <- [0 .. 2]]
  got = verdicts b (reachFrom b (entries 2 flagses)) flagses
  reach = reachB arcs [i | i <- [0 .. 2], testBit emask i]
  referenced = IS.fromList (map snd arcs)
  want =
    [ (i, 1 + fromEnum (odd i) + 2 * fromEnum (IS.member i referenced))
    | i <- [0 .. 2]
    , not (IS.member i reach)
    ]

-- | Booklet 06's liveness filter as this module's data: the asset
-- and unused ref-def kinds are inert, rungs past five are dropped.
inertKinds :: [Integer]
inertKinds = [3, 5]

topRung :: Integer
topRung = 5

-- | A 3-vertex graph's rows, drawn per arc from the batteries' LCG:
-- (kind, rung), and one time in three a second row of another kind.
drawnRows :: Int -> [(Integer, Integer, Integer, Integer)]
drawnRows code = sort (concat (zipWith rowsOf (arcsOf 3 code) (chunks (drop 1 (iterate lcgStep (toInteger code * 7 + 5))))))
 where
  chunks xs = let (a, b) = splitAt 5 xs in map (`div` 65536) a : chunks b
  rowsOf (s, d) [k, r, twice, k2, r2] =
    let first = (toInteger s, toInteger d, k `mod` 7, 1 + r `mod` 6)
        second = (toInteger s, toInteger d, (k + 1 + k2 `mod` 6) `mod` 7, 1 + r2 `mod` 6)
     in first : [second | twice `mod` 3 == 0]
  rowsOf _ _ = []

-- | Pass (c): the kept arcs and the positions over the shipped filter
-- against the reference's, and the drops the row met.
checkC :: Int -> (Maybe String, [String])
checkC code = (if bArcs b == S.fromList live && bKept b == length live && placed 3 b live [0] then Nothing else Just ("passC code=" <> show code), drops)
 where
  rows = drawnRows code
  b = build minRung [assetKind, refdefKind] 3 [[s, d, k, r] | (s, d, k, r) <- rows]
  alive (_, _, k, r) = r <= topRung && k `notElem` inertKinds
  live = nub [(fromInteger s, fromInteger d) | x@(s, d, _, _) <- rows, alive x]
  pairs = [(s, d) | (s, d, _, _) <- rows]
  drops =
    ["an asset arc dropped" | (_, _, 3, r) <- rows, r <= topRung]
      <> ["an unused ref-def arc dropped" | (_, _, 5, r) <- rows, r <= topRung]
      <> ["a rung past the floor dropped" | (_, _, k, r) <- rows, r > topRung, k `notElem` inertKinds]
      <> ["a pair carried twice counts once" | length (nub pairs) < length pairs]

passCShapes :: [String]
passCShapes = ["an asset arc dropped", "an unused ref-def arc dropped", "a rung past the floor dropped", "a pair carried twice counts once"]

equivalence :: IO Bool
equivalence = do
  let passA =
        [ bad
        | code <- [0 .. 65535]
        , es <- [[], [0], [0, 2]]
        , Just bad <- [checkA code es]
        ]
      passB =
        [ bad
        | code <- [0 .. 511]
        , emask <- [0 .. 7]
        , Just bad <- [checkB code emask]
        ]
      passC = [bad | code <- [0 .. 511], Just bad <- [fst (checkC code)]]
      met = S.fromList (concatMap (snd . checkC) [0 .. 511])
      missing = [shape | shape <- passCShapes, S.notMember shape met]
  mapM_ (putStrLn . ("FAIL graph reference never met: " <>)) missing
  case take 3 (passA <> passB <> passC) of
    [] | null missing -> do
      putStrLn "ok   graph reference equivalence (2^16 x 3 entry sets; 2^9 x 2^3 verdicts; 2^9 drawn kinds and rungs)"
      pure True
    [] -> pure False
    bad -> do
      mapM_ (putStrLn . ("FAIL graph reference: " <>)) bad
      pure False
