-- | The directory graph's feedback arc set (step-8 brief §1.0 ruling
-- 2; booklet §13 ruling 12): the cheapest set of arcs whose removal
-- leaves the graph acyclic, found one strongly connected component
-- at a time — an arc between two components lies on no cycle, and a
-- minimum over the whole graph is the union of the components'
-- minima.
--
-- A component of at most `exactVertexCap` vertices is ordered by a
-- dynamic programme over vertex subsets: a linear order's cost is the
-- triple (weight, count, arcs sorted by (from, to)) of its backward
-- arcs — those whose target comes before their source — compared in
-- that order, and cost(S) is the least over v ∈ S of cost(S ∖ {v})
-- plus the arcs from v back into S ∖ {v}, v placed last. The optimal
-- substructure holds for the whole triple: weight and count add, and
-- merging one sorted arc set into two disjoint sorted sets of equal
-- length keeps their order (the lesser is the one holding the least
-- arc of their symmetric difference, which the merge does not touch).
-- Every minimum-weight cut is the backward set of some order, so the
-- full set's cost is the minimum cut with the tie rule — least
-- weight, then fewest arcs, then the least sorted arc list — and it
-- is `exact` 1. A larger component goes through the Eades–Lin–Smyth
-- order on weights — a sink goes to the front of the tail segment, a
-- source to the end of the head segment, otherwise the vertex with
-- the greatest out-weight minus in-weight (the least id on a tie) to
-- the end of the head — and the arcs pointing backwards in head ++
-- tail are the cut; the redundancy pass then offers each cut arc,
-- ascending by (from, to), back to the kept graph and keeps it when
-- it closes no cycle. Every arc that stays cut closes a cycle on its
-- own, so the cut is minimal, though not proven minimum (`exact` 0).
-- Both roads visit vertices by id and arcs by (from, to), so the
-- answer is a function of the arc table.
module CE.Arch.Fas (Arc, acyclic, closes, exactCut, fas) where

import CE.Arch.Cost (exactNo, exactVertexCap, exactYes)
import CE.Arch.Dirs (Arcs, sccsOf)
import Data.Array (Array, accumArray, listArray, (!))
import Data.Bits (clearBit, testBit)
import qualified Data.Graph as G
import qualified Data.IntMap.Strict as IM
import qualified Data.IntSet as IS
import Data.List (find, insert, maximumBy, sort)
import qualified Data.Map.Strict as M
import Data.Ord (comparing)
import Data.Tree (flatten)

-- | One weighted arc of a component, on local vertex ids.
type Arc = ((Int, Int), Integer)

-- | The cut as `(from, to, weight, exact)` rows, ascending.
fas :: Arcs -> [(Int, Int, Integer, Integer)]
fas arcs = sort (concatMap component (M.elems buckets))
 where
  n = if M.null arcs then 0 else 1 + maximum [max a b | (a, b) <- M.keys arcs]
  comps = [ms | ms@(_ : _ : _) <- sccsOf n arcs]
  compOf = IM.fromList [(v, c) | (c, ms) <- zip [0 :: Int ..] comps, v <- ms]
  buckets =
    M.fromListWith
      (flip (<>))
      [ (c, [arc])
      | arc@((a, b), _) <- M.toList arcs
      , Just c <- [IM.lookup a compOf]
      , IM.lookup b compOf == Just c
      ]

-- | One component's cut, relabelled to 0..k−1 and back. The members
-- ascend, so the relabelling keeps every (from, to) comparison the
-- tie rule makes.
component :: [Arc] -> [(Int, Int, Integer, Integer)]
component inner = [(back IM.! a, back IM.! b, w, tag) | ((a, b), w) <- cut]
 where
  members = IS.toAscList (IS.fromList (concat [[a, b] | ((a, b), _) <- inner]))
  local = IM.fromList (zip members [0 ..])
  back = IM.fromList (zip [0 ..] members)
  arcs = [((local IM.! a, local IM.! b), w) | ((a, b), w) <- inner]
  k = length members
  (cut, tag)
    | k <= exactVertexCap = (exactCut k arcs, exactYes)
    | otherwise = (prune k arcs (greedyCut k arcs), exactNo)

-- | The least backward set over every order of the k vertices, by
-- the subset programme. A state is keyed by its bitmask; its value is
-- the (weight, count, sorted arcs) triple of the best order of those
-- vertices, so the full mask's arcs are the cut itself.
exactCut :: Int -> [Arc] -> [Arc]
exactCut k arcs = cutArcs (memo ! full)
 where
  full = 2 ^ k - 1 :: Int
  outOf :: Array Int [(Int, Integer)]
  outOf = accumArray (flip (:)) [] (0, k - 1) [(a, (b, w)) | ((a, b), w) <- reverse arcs]
  memo :: Array Int Placed
  memo = listArray (0, full) (map best [0 .. full])
  best 0 = Placed 0 0 []
  best s = minimum [lastly v (clearBit s v) | v <- [0 .. k - 1], testBit s v]
  lastly v rest = foldr add (memo ! rest) [((v, u), w) | (u, w) <- outOf ! v, testBit rest u]
  add arc (Placed w c l) = Placed (w + snd arc) (c + 1) (insert arc l)

-- | An order's backward arcs as its cost: the derived Ord compares the
-- weight, then the count, then the sorted arc list.
data Placed = Placed {_weight :: Integer, _count :: Int, cutArcs :: [Arc]}
  deriving (Eq, Ord)

-- | The arcs pointing backwards in the Eades–Lin–Smyth order.
greedyCut :: Int -> [Arc] -> [Arc]
greedyCut k arcs = [arc | arc@((a, b), _) <- arcs, pos IM.! a > pos IM.! b]
 where
  pos = IM.fromList (zip (elsOrder k arcs) [0 :: Int ..])

-- | Head ++ tail: sinks peel onto the tail's front, sources onto the
-- head's end, otherwise the heaviest net out-weight onto the head.
elsOrder :: Int -> [Arc] -> [Int]
elsOrder k arcs = go (IS.fromList [0 .. k - 1]) [] []
 where
  outs = IM.fromListWith (<>) [(a, [(b, w)]) | ((a, b), w) <- arcs]
  ins = IM.fromListWith (<>) [(b, [(a, w)]) | ((a, b), w) <- arcs]
  live left adj v = [(u, w) | (u, w) <- IM.findWithDefault [] v adj, IS.member u left]
  go left front tailSeg
    | IS.null left = reverse front <> tailSeg
    | Just v <- find (null . live left outs) vs = go (IS.delete v left) front (v : tailSeg)
    | Just v <- find (null . live left ins) vs = go (IS.delete v left) (v : front) tailSeg
    | otherwise = go (IS.delete best left) (best : front) tailSeg
   where
    vs = IS.toAscList left
    net v = sum (map snd (live left outs v)) - sum (map snd (live left ins v))
    best = maximumBy (comparing (\v -> (net v, negate v))) vs

-- | The redundancy pass: each cut arc, ascending, goes back into the
-- kept graph unless it closes a cycle there. The kept graph only
-- grows, so an arc that closed a cycle when offered still closes one
-- at the end.
prune :: Int -> [Arc] -> [Arc] -> [Arc]
prune k arcs cut = reverse (snd (foldl step (kept0, []) cut))
 where
  kept0 = [arc | arc <- arcs, arc `notElem` cut]
  step (kept, stays) arc@(a, _)
    | closes k (map fst kept) a = (kept, arc : stays)
    | otherwise = (arc : kept, stays)

-- | Adding (a, b) to an acyclic graph closes a cycle exactly when b
-- already reaches a.
closes :: Int -> [(Int, Int)] -> (Int, Int) -> Bool
closes k arcs (a, b) = G.path (G.buildG (0, k - 1) arcs) b a

-- | No component of more than one vertex, and no self arc.
acyclic :: Int -> [(Int, Int)] -> Bool
acyclic k arcs = all single (G.scc (G.buildG (0, k - 1) arcs)) && all (uncurry (/=)) arcs
 where
  single t = null (drop 1 (flatten t))
