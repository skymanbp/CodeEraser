-- | Cycle report (design §2): the SCCs at or above the floor, id =
-- position in the deterministic full SCC list (so cycle ids and
-- Position's sccId agree by construction). RG9 stance: cycles are
-- REPORTED, never judged — the verdict pass does not read this list.
module CE.Graph.Cycles (cycles) where

import CE.Graph.Build (Built (..))
import qualified Data.Set as S

-- | sccFloor is a parameter (Cost.sccFloor at the boundary, the
-- request's `sccFloor` since 6.4.0) so the dead-knob test can move
-- it. A SINGLETON counts only through its own arc (O59): every
-- isolated vertex is a one-node SCC too, so a floor of 1 must
-- report the self-loop and never the edgeless.
cycles :: Integer -> Built -> [(Int, [Int])]
cycles sccFloor b =
  [ (i, members)
  | (i, members) <- zip [0 ..] (bScc b)
  , toInteger (length members) >= sccFloor
  , cyclic members
  ]
 where
  cyclic [v] = S.member (v, v) (bArcs b)
  cyclic _ = True
