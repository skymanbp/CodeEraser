-- | Cycle report (design §2): the SCCs at or above the floor, id =
-- position in the deterministic full SCC list (so cycle ids and
-- Position's sccId agree by construction). RG9 stance: cycles are
-- REPORTED, never judged — the verdict pass does not read this list.
module CE.Graph.Cycles (cycles) where

import CE.Graph.Build (Built, sccList)

-- | sccFloor is a parameter (Cost.sccFloor at the boundary) so the
-- dead-knob test can move it.
cycles :: Integer -> Built -> [(Int, [Int])]
cycles sccFloor b =
  [ (i, members)
  | (i, members) <- zip [0 ..] (sccList b)
  , toInteger (length members) >= sccFloor
  ]
