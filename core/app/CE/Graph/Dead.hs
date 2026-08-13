-- | The four-way verdict (design §2): dead splits along TWO axes —
-- indegree × reachability — and public is structurally separated so
-- a library's exported-but-unreferenced API can never collapse into
-- plain dead (the RG10 firewall is a verdict code, not a policy).
-- Codes: 1 unref_private, 2 unref_public, 3 unreach_private,
-- 4 unreach_public — 1 + public + 2*referenced. A node whose flags
-- meet the entry mask seeds reachability and is never judged.
module CE.Graph.Dead (entries, verdicts) where

import CE.Graph.Build (Built (..), reachFrom)
import Data.Bits (testBit, (.&.))
import qualified Data.IntSet as IS
import qualified Data.Set as S

-- | Root nodes: flags ∩ entryMask ≠ 0 (Cost.entryMask at the
-- boundary; a parameter here so the dead-knob test can perturb it).
entries :: Integer -> [Integer] -> [Int]
entries mask flagses = [i | (i, f) <- zip [0 ..] flagses, f .&. mask /= 0]

-- | [(idx, verdict)] ascending for every node outside the reach set.
verdicts :: Built -> Integer -> [Integer] -> [(Int, Int)]
verdicts b mask flagses =
  [ (i, 1 + fromEnum (testBit f 0) + 2 * fromEnum (IS.member i referenced))
  | (i, f) <- zip [0 ..] flagses
  , not (IS.member i reach)
  ]
 where
  reach = reachFrom b (entries mask flagses)
  referenced = IS.fromList [d | (_, d) <- S.toList (bArcs b)]
