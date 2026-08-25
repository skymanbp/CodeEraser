-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | The graph JUDGMENT's knob ablation and fixed-seed property
-- battery (plan §7.3 as amended v1.4). Dead-knob: perturbing each
-- CE.Graph.Cost knob — minRung, entryMask, sccFloor — must move a
-- verdict/report count on the fixture, the fuck-u-code dead-field
-- shape the plan names. Properties: entry-mask and rung monotonicity
-- of the dead set, SCC partition sanity, and the ascending-contract
-- rejection, over 200 graphs from a fixed-seed LCG — a seeded
-- generator samples what enumeration cannot afford (n > 4), and no
-- RNG enters a byte-determinism contract.
-- The request contract it answers under lives in GraphWireProps.
module GraphProps (battery) where

import CE.Graph (respond)
import CE.Graph.Build (Built (..), build, reachFrom)
import CE.Graph.Cost (assetKind, entryMask, exportVisBit, minRung, publicFlagBit, refdefKind, roleBits, sccFloor)
import CE.Graph.Cycles (cycles)
import CE.Graph.Dead (deriveFlags, entries, exportedNodes, verdicts, withExport)
import Data.Aeson (Value (..), encode, object, (.=))
import Data.Bits (xor)
import qualified Data.ByteString.Lazy as BL
import qualified Data.IntSet as IS
import Data.List (isInfixOf, sort)
import ReferenceGraph (reachB)
import WireHarness (runChecks)

battery :: IO Bool
battery =
  runChecks
    [ ("dead knobs all live (minRung / entryMask / sccFloor)", deadKnobs)
    , ("dead set anti-monotone in entryMask (200 seeded graphs)", allGraphs maskMono)
    , ("dead set anti-monotone in minRung (200 seeded graphs)", allGraphs rungMono)
    , ("SCC lists partition the vertex set (200 seeded graphs)", allGraphs sccPartition)
    , ("shuffled edge table is refused (200 seeded graphs)", allGraphs shuffledRefused)
    , ("no inert-kind edge keeps its target alive (asset, unused ref-def)", inertNeverAlive)
    , ("roles derive the entry bits through the table (2.28.0)", rolesDerive)
    , ("the role table is a live knob (a dropped row empties the seeds)", roleKnob)
    , ("both export knobs are live (visibility bit, flag bit)", exportKnobs)
    ]

-- | Fixture: entry 0 -> 1 over a rung-5 edge, an unreachable
-- 2-cycle (2,3), two isolated nodes (4,5). Every knob perturbation
-- must move (dead rows, cycle rows) — production (4,1).
deadKnobs :: Bool
deadKnobs =
  and
    [ judged (minRung - 1) entryMask sccFloor /= base
    , judged minRung (entryMask `xor` 2) sccFloor /= base
    , judged minRung entryMask (sccFloor + 1) /= base
    ]
 where
  flagses = [2, 0, 0, 0, 0, 0]
  rows = [[0, 1, 0, 5], [2, 3, 0, 1], [3, 2, 0, 1]]
  base = judged minRung entryMask sccFloor
  judged r m f =
    let b = build r [assetKind, refdefKind] 6 rows
        reach = reachFrom b (entries m flagses)
     in (length (verdicts b reach flagses), length (cycles f b))

-- | batch-7 slice 13 (2.20.0) generalized at H1 slice 16: the rows
-- travel and the CORE drops them — an entry linking a file ONLY
-- through an inert-kind edge (asset, unused ref-def) leaves it
-- dead, and flipping the kind to import revives it (the
-- counterfactual that proves each constant is a live lever).
inertNeverAlive :: Bool
inertNeverAlive =
  all (\k -> deadWith k == [1]) [assetKind, refdefKind] && deadWith 0 == []
 where
  deadWith kind =
    let b = build minRung [assetKind, refdefKind] 2 [[0, 1, kind, 1]]
     in map fst (verdicts b (reachFrom b (entries entryMask [2, 0])) [2, 0])

-- | 2.28.0 (batch-7 slice 3): every role row lands on its declared
-- bit — the named-main, executable-dir and declared-target roles all
-- on bit 1, test/glob/doc/allow on theirs — and a combined mask ORs.
rolesDerive :: Bool
rolesDerive =
  and
    [ deriveFlags roleBits 1 == 2
    , deriveFlags roleBits 2 == 2
    , deriveFlags roleBits 4 == 4
    , deriveFlags roleBits 8 == 8
    , deriveFlags roleBits 16 == 32
    , deriveFlags roleBits 32 == 64
    , deriveFlags roleBits 64 == 2
    , deriveFlags roleBits (1 + 4 + 64) == 6
    ]

-- | Dropping the named-main row from the table must empty the entry
-- set that role seeded — the dead-knob discipline applied to DATA.
roleKnob :: Bool
roleKnob =
  entries entryMask [deriveFlags roleBits 1] == [0]
    && null (entries entryMask [deriveFlags [(0, 0)] 1])

-- | One 3-column row beside a 4-column row refuses by name — a
-- | Both export knobs move the answer. Reading a visibility bit the
-- rows do not carry names no surface; setting a FLAG bit inside
-- entryMask does something else entirely — node 0 becomes an entry
-- and leaves the judged set — which is exactly why bit 0, outside
-- the mask, is the only bit an export surface may set (RG10).
exportKnobs :: Bool
exportKnobs =
  and
    [ codesWith exportVisBit publicFlagBit == [(0, 2), (1, 1)]
    , codesWith 1 publicFlagBit == [(0, 1), (1, 1)]
    , codesWith exportVisBit 2 == [(1, 1)]
    ]
 where
  b = build minRung [assetKind, refdefKind] 2 []
  codesWith visBit flagBit = verdicts b (reachFrom b (entries entryMask fs)) fs
   where
    fs = [withExport flagBit (exportedNodes visBit [[0, 1]]) i 0 | i <- [0, 1]]

allGraphs :: ((Int, [(Int, Int, Integer)], [Integer]) -> Bool) -> Bool
allGraphs prop = all (prop . graphAt) [1 .. 200 :: Int]

graphAt :: Int -> (Int, [(Int, Int, Integer)], [Integer])
graphAt i = (n, arcs, flagses)
 where
  s0 = lcg (toInteger i * 7919)
  n = 3 + fromInteger (s0 `mod` 6)
  picks = drops (lcg s0) [(a, b) | a <- [0 .. n - 1], b <- [0 .. n - 1]]
  arcs = [(a, b, 1 + lcg (toInteger (a * n + b)) `mod` 5) | (a, b) <- picks]
  flagses = [lcg (s0 + toInteger v) `mod` 8 | v <- [1 .. n]]

-- | Keep each element on a 1-in-4 draw from the seed stream.
drops :: Integer -> [(Int, Int)] -> [(Int, Int)]
drops _ [] = []
drops s (x : xs)
  | s `mod` 4 == 0 = x : drops (lcg s) xs
  | otherwise = drops (lcg s) xs

lcg :: Integer -> Integer
lcg s = (s * 6364136223846793005 + 1442695040888963407) `mod` 18446744073709551616

rowsOf :: [(Int, Int, Integer)] -> [[Integer]]
rowsOf arcs = sort [[toInteger a, toInteger b, 0, r] | (a, b, r) <- arcs]

deadIdx :: Integer -> Integer -> Int -> [(Int, Int, Integer)] -> [Integer] -> IS.IntSet
deadIdx r m n arcs flagses =
  IS.fromList (map fst (verdicts b (reachFrom b (entries m flagses)) flagses))
 where
  b = build r [assetKind, refdefKind] n (rowsOf arcs)

-- | More entry bits => more roots => the dead set can only shrink.
maskMono :: (Int, [(Int, Int, Integer)], [Integer]) -> Bool
maskMono (n, arcs, flagses) =
  deadIdx minRung 126 n arcs flagses `IS.isSubsetOf` deadIdx minRung 2 n arcs flagses

-- | A higher rung ceiling keeps more edges => reach grows => the
-- dead set can only shrink.
rungMono :: (Int, [(Int, Int, Integer)], [Integer]) -> Bool
rungMono (n, arcs, flagses) =
  deadIdx 5 entryMask n arcs flagses `IS.isSubsetOf` deadIdx 2 entryMask n arcs flagses

-- | Every vertex in exactly one SCC, members mutually reachable
-- (checked against the independent fixpoint).
sccPartition :: (Int, [(Int, Int, Integer)], [Integer]) -> Bool
sccPartition (n, arcs, _) =
  sort (concat sccs) == [0 .. n - 1] && all mutual sccs
 where
  sccs = bScc (build 5 [assetKind, refdefKind] n (rowsOf arcs))
  plain = [(a, b) | (a, b, _) <- arcs]
  mutual ms = and [IS.member j (reachB plain [i]) | i <- ms, j <- ms]

-- | Reversing a >=2-row edge table must trip the ascending contract.
shuffledRefused :: (Int, [(Int, Int, Integer)], [Integer]) -> Bool
shuffledRefused (n, arcs, _) =
  case sortedRows of
    rows@(_ : _ : _) -> refused (reverse rows)
    _ -> True
 where
  sortedRows = rowsOf arcs
  refused rows = case respond "5.1.0" (req rows) of
    Left (_, code, msg) -> code == "contract" && "not strictly ascending" `isInfixOf` msg
    Right _ -> False
  req rows =
    BL.toStrict . encode $
      object
        [ "id" .= (1 :: Int)
        , "nodes" .= replicate n ([0, 0, 0] :: [Integer])
        , "edges" .= rows
        , "pos" .= ([] :: [Value])
        ]
