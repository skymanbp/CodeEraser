-- | Roots and dependents of the dead file set (plan v2.35, the
-- dead-file amendment, phase 5A). U is the set of nodes the entry set
-- does not reach along kept arcs (the complement of CE.Graph's
-- `reach`), D the file nodes of U (the `dead` rows). A dead file g
-- SUPPORTS a dead file f when a kept path leads from g to f; every node
-- on such a path is in U, since a reached node on it would reach f. A
-- dead file f is a ROOT when every dead file that supports it is also
-- supported by it; every other dead file is a DEPENDENT, and its roots
-- are the roots that support it (at least one: follow supporters
-- upward through the finite condensation).
--
-- The component algorithm is the one CE.Graph.Build already answers,
-- `bScc` (Data.Graph.scc: every SCC of the whole graph, in reverse
-- topological order). An SCC is wholly inside U or wholly outside it
-- (its members reach each other), and every predecessor of a node in U
-- is in U, so the condensation of U is the dead SCCs with the arcs
-- between them. A file f is a root iff no STRICT ancestor component of
-- f's component holds a dead file: a supporter in the same component is
-- supported by f, one in an ancestor component is not.
--
-- Cost. The root set is one pass over the condensation in topological
-- order (a bit per component: does a strict ancestor hold a file),
-- O(arcs), and it is never dropped. The dependent -> root pairs carry,
-- per component, the set of roots above it, built along the same arcs;
-- the cap counts WORK, not output alone: every root id copied across a
-- condensation arc (one per element of the source's contribution, the
-- arc counted once per kept arc between the two components) plus every
-- pair written. Each unit is one IntSet insert or one list cell, so
-- with the count checked before each step the pass does at most
-- `cap` inserts and writes beyond its O(arcs) frame, and the final sort
-- is over at most `cap` pairs: O(arcs + cap log cap). Past the cap the
-- pairs are dropped whole (Nothing), never a prefix.
module CE.Graph.Roots (Grouping (..), group) where

import CE.Graph.Build (Built (..))
import qualified Data.IntMap.Strict as IM
import qualified Data.IntSet as IS
import Data.List (sort)
import qualified Data.Set as S

-- | The answer: the roots ascending, and the (dependent, root) pairs
-- ascending, or Nothing when they were dropped over the cap.
data Grouping = Grouping
  { gRoots :: [Int]
  , gUnder :: Maybe [(Int, Int)]
  }

-- | One dead component in topological order: its id, its members, the
-- dead files among them.
data Comp = Comp {cId :: Int, cFiles :: [Int]}

group :: Integer -> Built -> IS.IntSet -> [Int] -> Grouping
group cap b reach dead = Grouping (sort [f | c <- topo, rootComp c, f <- cFiles c]) (pairs cap topo preds rootComp)
 where
  files = IS.fromList dead
  comps = [(k, ms) | (k, ms@(m : _)) <- zip [0 ..] (bScc b), not (IS.member m reach)]
  topo = reverse [Comp k (filter (`IS.member` files) ms) | (k, ms) <- comps]
  compOf = IM.fromList [(m, k) | (k, ms) <- comps, m <- ms]
  -- condensation arcs into each dead component, one per kept arc
  preds =
    IM.fromListWith
      (<>)
      [ (cd, [cs])
      | (s, d) <- S.toList (bArcs b)
      , Just cd <- [IM.lookup d compOf]
      , Just cs <- [IM.lookup s compOf]
      , cs /= cd
      ]
  fileMap = IM.fromList [(cId c, cFiles c) | c <- topo]
  -- does a strict ancestor component hold a dead file (topological fold)
  above = foldl' step IM.empty topo
  step acc c = IM.insert (cId c) (or [held acc p | p <- IM.findWithDefault [] (cId c) preds]) acc
  held acc p = IM.findWithDefault False p acc || not (null (IM.findWithDefault [] p fileMap))
  rootComp c = not (null (cFiles c)) && not (IM.findWithDefault False (cId c) above)

-- | The roots above each component and the pairs, under the work cap.
pairs :: Integer -> [Comp] -> IM.IntMap [Int] -> (Comp -> Bool) -> Maybe [(Int, Int)]
pairs cap topo preds rootComp = go 0 IM.empty [] topo
 where
  go _ _ out [] = Just (sort out)
  go w acc out (c : rest)
    | w1 > cap || w2 > cap = Nothing
    | otherwise = go w2 (IM.insert (cId c) given acc) (written <> out) rest
   where
    contribs = [IM.findWithDefault (0, []) p acc | p <- IM.findWithDefault [] (cId c) preds]
    w1 = w + sum (map (toInteger . fst) contribs)
    set = foldr (\(_, xs) s -> foldr IS.insert s xs) IS.empty contribs
    n = IS.size set
    root = rootComp c
    w2 = w1 + if root then 0 else toInteger (length (cFiles c)) * toInteger n
    written = if root then [] else [(f, r) | f <- cFiles c, r <- IS.toAscList set]
    -- what this component hands down: its own files when it is a root
    -- component, else the roots above it
    given = if root then (length (cFiles c), cFiles c) else (n, IS.toList set)
