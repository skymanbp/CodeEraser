-- | The four-way verdict (design §2): dead splits along TWO axes —
-- indegree × reachability — and public is structurally separated so
-- a library's exported-but-unreferenced API can never collapse into
-- plain dead (the RG10 firewall is a verdict code, not a policy).
-- Codes: 1 unref_private, 2 unref_public, 3 unreach_private,
-- 4 unreach_public — 1 + public + 2*referenced. A node whose flags
-- meet the entry mask seeds reachability and is never judged.
module CE.Graph.Dead (entries, verdicts) where

import CE.Graph.Build (Built (..))
import Data.Bits (testBit, (.&.))
import qualified Data.IntSet as IS
import qualified Data.Set as S

-- | Root nodes: flags ∩ entryMask ≠ 0 (Cost.entryMask at the
-- boundary; a parameter here so the dead-knob test can perturb it).
entries :: Integer -> [Integer] -> [Int]
entries mask flagses = [i | (i, f) <- zip [0 ..] flagses, f .&. mask /= 0]

-- | The four-way TABLE (ADR-008 lattice-table form): (public,
-- referenced) -> code, a total lookup replacing the 1 + p + 2r
-- arithmetic so the code assignment is DATA. The GraphProps
-- brute-force equality is its reorder counterfactual: permute a row
-- and the reference disagrees on every fixture that hits it.
deadTable :: [((Bool, Bool), Int)]
deadTable =
  [ ((False, False), 1) -- unref_private
  , ((True, False), 2) -- unref_public
  , ((False, True), 3) -- unreach_private
  , ((True, True), 4) -- unreach_public
  ]

-- | [(idx, verdict)] ascending for every node outside the reach set.
-- reach is computed once at the CE.Graph boundary and handed down —
-- Position consumes the SAME set (batch 9 P2).
verdicts :: Built -> IS.IntSet -> [Integer] -> [(Int, Int)]
verdicts b reach flagses =
  [ (i, code (testBit f 0) (IS.member i referenced))
  | (i, f) <- zip [0 ..] flagses
  , not (IS.member i reach)
  ]
 where
  referenced = IS.fromList [d | (_, d) <- S.toList (bArcs b)]
  code p r = case lookup (p, r) deadTable of
    Just c -> c
    Nothing -> error "deadTable is total over (Bool, Bool) by construction"
