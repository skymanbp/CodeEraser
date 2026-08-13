-- | Exhaustive brute-force equivalence for the graph judgment
-- (design §6, 2g exit). Pass (a): EVERY directed graph on 4 vertices
-- — 2^16 arc subsets, self-loops included — under 3 fixed entry
-- sets; indegree, outdegree, reachability and the SCC partition must
-- match an independently written fixpoint / mutual-reachability
-- reference. Pass (b): every graph on 3 vertices (2^9) × every entry
-- subset (2^3), driven through the PRODUCTION flags→entryMask path —
-- the four-way verdict as a total function, the only pass covering
-- the entry-set dimension. First 3 mismatches reported (the
-- Reference.hs convention). Lives in core/test/ (RG14: the harness
-- must never cost the app tree its 300-line gate).
module ReferenceGraph (equivalence, arcsOf, edgeRows, reachB) where

import CE.Graph.Build (build, sccList)
import CE.Graph.Dead (verdicts)
import CE.Graph.Position (positions)
import Data.Bits (testBit)
import Data.List (sort)
import qualified Data.IntSet as IS
import qualified Data.Set as S

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
  | rowsOK && sccProd == sccWant = Nothing
  | otherwise =
      Just ("passA code=" <> show code <> " entries=" <> show es)
 where
  arcs = arcsOf 4 code
  b = build 5 4 (edgeRows arcs)
  flagses = [if i `elem` es then 2 else 0 | i <- [0 .. 3]]
  rows = positions b 2 flagses [0 .. 3]
  reach = reachB arcs es
  sccProd = S.fromList (map S.fromList (sccList b))
  sccWant = sccB 4 arcs
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
  b = build 5 3 (edgeRows arcs)
  flagOf i = (if odd i then 1 else 0) + (if testBit emask i then 2 else 0)
  flagses = [flagOf i | i <- [0 .. 2]]
  got = verdicts b 2 flagses
  reach = reachB arcs [i | i <- [0 .. 2], testBit emask i]
  referenced = IS.fromList (map snd arcs)
  want =
    [ (i, 1 + fromEnum (odd i) + 2 * fromEnum (IS.member i referenced))
    | i <- [0 .. 2]
    , not (IS.member i reach)
    ]

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
  case take 3 (passA <> passB) of
    [] -> do
      putStrLn "ok   graph reference equivalence (2^16 x 3 entry sets; 2^9 x 2^3 verdicts)"
      pure True
    bad -> do
      mapM_ (putStrLn . ("FAIL graph reference: " <>)) bad
      pure False
