-- | Layers and per-directory metrics (step-8 brief §1.0 rulings 4 and
-- 8). With the cut arcs gone the directory graph is acyclic, and a
-- directory's level is 0 when nothing leaves it and otherwise one more
-- than the highest level it points at — the foundations sit at 0 and
-- every kept arc runs from a higher level down to a lower one. The
-- metrics read the whole directory graph, before any cut: a
-- directory's fan-in counts the distinct directories pointing at it,
-- its fan-out the distinct ones it points at, and its instability is
-- the per-mille share of fan-out in the two (Martin's I).
module CE.Arch.Layers (levels, metrics) where

import CE.Arch.Cost (instabilityNone, instabilityScale)
import CE.Arch.Dirs (Arcs)
import qualified Data.IntMap.Lazy as IL
import qualified Data.IntMap.Strict as IM
import qualified Data.Map.Strict as M
import qualified Data.Set as S

-- | `[[D, level]]` for directories 0..n−1, ascending. The cut arcs
-- are removed first; the recursion is a lazy table over an acyclic
-- graph, so every level is computed once.
levels :: Int -> Arcs -> S.Set (Int, Int) -> [[Integer]]
levels n arcs cut = [[toInteger d, level d] | d <- [0 .. n - 1]]
 where
  outs = IM.fromListWith (<>) [(a, [b]) | (a, b) <- M.keys arcs, not (S.member (a, b) cut)]
  table = IL.fromList [(d, of' d) | d <- [0 .. n - 1]]
  of' d = case IM.findWithDefault [] d outs of
    [] -> 0
    targets -> 1 + maximum (map level targets)
  level d = IL.findWithDefault 0 d table

-- | `[[D, fanIn, fanOut, instability]]` for directories 0..n−1. The
-- arc table holds one arc per ordered pair, so counting arcs counts
-- distinct directories.
metrics :: Int -> Arcs -> [[Integer]]
metrics n arcs = [row d | d <- [0 .. n - 1]]
 where
  fanOut = IM.fromListWith (+) [(a, 1) | (a, _) <- M.keys arcs]
  fanIn = IM.fromListWith (+) [(b, 1) | (_, b) <- M.keys arcs]
  row d = [toInteger d, i, o, instability i o]
   where
    i = IM.findWithDefault 0 d fanIn
    o = IM.findWithDefault 0 d fanOut

-- | ⌊scale · out ÷ (in + out)⌋, or the none value when nothing
-- touches the directory.
instability :: Integer -> Integer -> Integer
instability i o
  | i + o == 0 = instabilityNone
  | otherwise = instabilityScale * o `div` (i + o)
