-- | The four-way verdict (design §2): dead splits along TWO axes —
-- indegree × reachability — and public is structurally separated so
-- a library's exported-but-unreferenced API can never collapse into
-- plain dead (the RG10 firewall is a verdict code, not a policy).
-- Codes: 1 unref_private, 2 unref_public, 3 unreach_private,
-- 4 unreach_public — 1 + public + 2*referenced. A node whose flags
-- meet the entry mask seeds reachability and is never judged.
module CE.Graph.Dead (deriveFlags, entries, exportedNodes, verdicts, withExport) where

import CE.Graph.Build (Built (..))
import Data.Bits (bit, testBit, (.&.), (.|.))
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

-- | The nodes a `symbols` table calls exported (4.1.0): rows are
-- [node, visibility] and the bit that counts is a PARAMETER
-- (Cost.exportVisBit at the CE.Graph boundary), so the knob test can
-- move it and watch verdict codes shift — the same discipline
-- deriveFlags's table follows. Rows the validator refused cannot
-- reach here; a row of another shape contributes nothing rather than
-- reading a number out of a shape it does not understand.
exportedNodes :: Integer -> [[Integer]] -> IS.IntSet
exportedNodes visBit rows =
  IS.fromList
    [fromInteger node | [node, vis] <- rows, testBit vis (fromInteger visBit)]

-- | One node's flags with the export surface folded in. The bit is a
-- parameter for the same reason (Cost.publicFlagBit), and it sits
-- OUTSIDE entryMask on purpose: an export surface is the verdict
-- axis deadTable splits on, never an entry claim (RG10). So this OR
-- can change WHICH CODE a dead node reports and can never change
-- which nodes are dead — the property the battery pins.
withExport :: Integer -> IS.IntSet -> Int -> Integer -> Integer
withExport flagBit exported i f
  | IS.member i exported = f .|. bit (fromInteger flagBit)
  | otherwise = f

-- | Entry bits from role facts through a role table (2.28.0,
-- batch-7 slice 3): the OR of the mapped flag bits for every role
-- bit set. The table is a parameter (Cost.roleBits at the CE.Graph
-- boundary) so the props battery can perturb a row and watch the
-- entry set move — the dead-knob discipline.
deriveFlags :: [(Integer, Integer)] -> Integer -> Integer
deriveFlags table roles =
  foldr (.|.) 0 [bit (fromInteger b) | (r, b) <- table, testBit roles (fromInteger r)]
