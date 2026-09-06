-- | S7 modularity — the directed dir-edge table's one judgment (plan
-- v2.29 step 10 batch C3, O54). The tree's directories ARE the
-- partition; a directory that keeps less cohesion than its own traffic
-- could buy is not a module, it is a place files happen to sit.
--
-- The intra-directory mass rides NOWHERE. fileRefs already carries it:
-- an intra edge adds 1 to `inside` at BOTH of its ends, so a
-- directory's inside sum is exactly twice its internal edge count. A
-- number that rides twice is a number that can disagree with itself
-- (the seamSoft lesson at CE.Structure.Cost). What makes the
-- derivation a READING of the request rather than a guess is
-- 'crossTableOffence' below, which holds the two tables to ONE graph
-- before anything here counts.
module CE.Structure.Modularity
  ( edgeRowSpec
  , crossTableOffence
  , endpointsByDir
  , unmodular
  ) where

import Data.Foldable (asum)
import qualified Data.Map.Strict as M

-- | The dirEdges spec tuple for CE.Structure's shared dirRow checker:
-- arity 3, no self edge, count at least 1. The FROM endpoint's range
-- and the negativity pass ride the generic checks; the TO endpoint
-- needs the node count, which the caller holds.
edgeRowSpec :: Integer -> (Int, String, [Integer] -> Maybe String)
edgeRowSpec n = (3, "dirEdges", ok)
 where
  ok row = case row of
    [a, b, c]
      | a == b -> Just "self edge (from == to)"
      | b >= n -> Just "to dir out of range"
      | c < 1 -> Just "count below 1"
      | otherwise -> Nothing
    _ -> Nothing

-- | Per-directory (inside sum, outside sum) over the fileRefs rows —
-- ONE projection, two readers: S2 mixing compares the two halves, S7
-- reads `inside` as twice the internal edge count and `outside` as the
-- crossing endpoints the dir-edge table must account for. One basis on
-- both sides, the Axes.hs discipline, now across two axes.
endpointsByDir :: [[Integer]] -> M.Map Integer (Integer, Integer)
endpointsByDir refs =
  M.fromListWith add [(d, (i * n, o * n)) | [d, i, o, n] <- refs]
 where
  add (a, b) (c, e) = (a + c, b + e)

-- | The two tables must describe ONE graph, checked before the axis
-- derives anything from either: a directory's fileRefs `inside` sum is
-- twice an edge count (hence even), and its `outside` sum — the
-- crossing endpoints it owns — equals the dirEdges mass incident to it
-- in either direction. Enforced only when dirEdges rides, so every
-- request shape that predates it is untouched.
crossTableOffence :: [[Integer]] -> [[Integer]] -> Maybe String
crossTableOffence refs edges = asum (map bad keys)
 where
  keys = M.keys (M.union (() <$ ends) (() <$ incident))
  ends = endpointsByDir refs
  incident =
    M.fromListWith
      (+)
      ([(a, n) | [a, _, n] <- edges] <> [(b, n) | [_, b, n] <- edges])
  bad d
    | odd ins =
        Just (at d <> "fileRefs inside sum " <> show ins <> " is odd (not twice an edge count)")
    | outs /= mass =
        Just (at d <> "rides " <> show outs <> " crossing endpoints in fileRefs but " <> show mass <> " in dirEdges")
    | otherwise = Nothing
   where
    (ins, outs) = M.findWithDefault (0, 0) d ends
    mass = M.findWithDefault 0 d incident
  at d = "dirEdges: directory " <> show d <> " "

-- | S7: the directories whose modularity contribution falls below the
-- floor. Write e = internal edges, o = out-degree mass, i = in-degree
-- mass, mu = incident edges, m = every edge in the tree. Newman's
-- contribution is q = e/m − o*i/m^2, and the most a directory of mass
-- mu could earn (every incident edge internal) is qMax = mu(m−mu)/m^2.
-- The judged quantity is the RATIO
--
--     rho = q / qMax = (e*m − o*i) / (mu*(m − mu))
--
-- which is 1 for a directory that never reaches out, 0 exactly at the
-- null model's expectation, negative below it — and which, unlike q
-- itself, does not shrink as the tree grows (Sum_c q_c <= 1, so a
-- floor on q alone flags a fixed FRACTION of any large tree; that is
-- the raw-mass failure the 2.26.0 density law retired). The comparison
-- is one integer inequality against the per-mille floor: nothing
-- divides, and mu(m−mu) > 0 holds on every judged row by the guards.
--
-- mu == m (a directory incident to every edge) is DECLINED, not
-- flagged: qMax is 0 there — no complement to be separate from. m == 0
-- declines everything, so an empty table judges clean.
unmodular :: (Integer, Integer) -> [[Integer]] -> [[Integer]] -> [Integer] -> [Integer]
unmodular (floorPerMille, massFloor) refs edges dirs =
  [ d
  | d <- dirs
  , let e = intraOf d
  , let out = e + at fromMass d
  , let inn = e + at toMass d
  , let mu = e + at fromMass d + at toMass d
  , mu >= massFloor
  , mu < m
  , 1000 * (e * m - out * inn) < floorPerMille * mu * (m - mu)
  ]
 where
  ends = endpointsByDir refs
  intraOf d = fst (M.findWithDefault (0, 0) d ends) `div` 2
  fromMass = M.fromListWith (+) [(a, n) | [a, _, n] <- edges]
  toMass = M.fromListWith (+) [(b, n) | [_, b, n] <- edges]
  at t d = M.findWithDefault 0 d t
  m = sum (map intraOf dirs) + sum [n | [_, _, n] <- edges]
