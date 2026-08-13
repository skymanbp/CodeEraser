-- | Per-node join surface, computed only for the requested pos
-- indices (design §2): [idx, indeg, outdeg, sccId, sccSize, reachIn].
-- Degrees count distinct kept arcs (Build's dedup); sccId indexes
-- the same deterministic SCC list the cycle report uses.
module CE.Graph.Position (positions) where

import CE.Graph.Build (Built (..), reachFrom, sccList)
import CE.Graph.Dead (entries)
import qualified Data.IntMap.Strict as IM
import qualified Data.IntSet as IS
import qualified Data.Set as S

positions :: Built -> Integer -> [Integer] -> [Integer] -> [[Integer]]
positions b mask flagses pos =
  [ [ p
    , toInteger (IM.findWithDefault 0 i indeg)
    , toInteger (IM.findWithDefault 0 i outdeg)
    , toInteger (IM.findWithDefault 0 i sccOf)
    , toInteger (IM.findWithDefault 0 i sizeOf)
    , toInteger (fromEnum (IS.member i reach))
    ]
  | p <- pos
  , let i = fromInteger p
  ]
 where
  arcs = S.toList (bArcs b)
  indeg = IM.fromListWith (+) [(d, 1 :: Int) | (_, d) <- arcs]
  outdeg = IM.fromListWith (+) [(s, 1 :: Int) | (s, _) <- arcs]
  sccs = sccList b
  sccOf = IM.fromList [(v, i) | (i, ms) <- zip [0 :: Int ..] sccs, v <- ms]
  sizeOf = IM.fromList [(v, length ms) | ms <- sccs, v <- ms]
  reach = reachFrom b (entries mask flagses)
