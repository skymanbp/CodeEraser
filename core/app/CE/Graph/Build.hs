-- | Graph construction for the judgment passes (design §2). The
-- kept arc set is the rung-filtered, (src,dst)-deduped edge list:
-- kind multiplicity between one pair of nodes is not extra evidence
-- of reference, so indegree counts distinct arcs. Data.Graph.buildG
-- over dense 0-based indices makes every downstream answer a
-- function of the (contract-sorted) edge set — byte determinism by
-- structure, not assertion.
module CE.Graph.Build (Built (..), build, reachFrom, sccList) where

import qualified Data.Graph as G
import qualified Data.IntSet as IS
import Data.List (sort)
import qualified Data.Set as S
import Data.Tree (flatten)

-- | Kept arcs plus the containers graph over ALL n nodes — isolated
-- vertices stay in the graph because they are exactly the
-- unreferenced ones. kept is the deduped arc count (counts.kept on
-- the wire: what the analysis actually used).
data Built = Built
  { bN :: Int
  , bKept :: Int
  , bArcs :: S.Set (Int, Int)
  , bGraph :: G.Graph
  }

-- | minRung is a parameter (Cost.minRung at the boundary) so the
-- dead-knob test can move it; the asset-kind exclusion (batch-7
-- slice 13) lives in the SAME comprehension as the rung filter —
-- the kind column always crossed and was discarded here while Rust
-- pre-dropped the rows the rule was about.
build :: Integer -> Integer -> Int -> [[Integer]] -> Built
build minR asset n rows = Built n (S.size arcs) arcs g
 where
  arcs =
    S.fromList
      [ (fromInteger s, fromInteger d)
      | [s, d, kind, rung] <- rows
      , rung <= minR
      , kind /= asset
      ]
  g = G.buildG (0, n - 1) (S.toList arcs)

-- | Entry seeds plus everything they reach along kept arcs.
reachFrom :: Built -> [Int] -> IS.IntSet
reachFrom b seeds = IS.fromList (concatMap (G.reachable (bGraph b)) seeds)

-- | Every SCC in Data.Graph order — a deterministic function of the
-- sorted arc set — members ascending. Singleton SCCs are included so
-- the id space covers every vertex (Position reports isolated nodes
-- too); the cycle report applies its floor downstream.
sccList :: Built -> [[Int]]
sccList b = [sort (flatten t) | t <- G.scc (bGraph b)]
