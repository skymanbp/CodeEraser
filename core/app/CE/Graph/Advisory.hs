-- | The `export_unmentioned` advisory (graph/1 6.2.0, plan v2.17 L
-- round piece (6); sealed criterion §4 folds and §5.2 reply): which
-- declarations nothing outside their file spells, and how far each
-- one's own package lets it out. Its OWN class beside CE.Graph.Dead,
-- never a verdict — a row here can neither add a node to the dead
-- set nor flip the gate (the §0 iron rule the reply keeps by
-- construction: `Graph.result` appends these keys and reads nothing
-- back), and every predicate below is a named, ablatable fold the
-- battery can move one at a time (K33).
--
-- Two request tables feed it: `unmentioned` = [[node, vis, conv]], the
-- declarations no other file mentions (the producer's negative
-- instrument, cli/src/mention), and `mounts` = [[node, private, total,
-- bits]], every node's mount facts. The row validators live here with
-- the fold that consumes the rows, so CE.Graph.Contract stays the
-- table of what arrives and this module the meaning of one row.
module CE.Graph.Advisory (code, judge, mountRow, mountedPrivate, pkgPrivate, reexported, unmentionedRow) where

import CE.Graph.Cost (pkgPrivateMountBit, reexportMountBit, restrictedVisBit)
import CE.Wire (rowCheck)
import Data.Bits (testBit, (.&.))
import qualified Data.Map.Strict as M
import Data.Maybe (fromMaybe)

-- | One mounts row: [node, private, total, bits]. `total = 0` is legal
-- (zero mounts, the row every non-Rust file sends); the coverage of
-- the table — one row per node — is the producer's contract, not a
-- refusal here (§4: a missing row reads [0,0,0] below).
mountRow :: Integer -> Int -> [Integer] -> Maybe String
mountRow n = rowCheck "mount" "malformed row (need [node,private,total,bits])" 4 fields
 where
  fields row@[node, private, total, _]
    | any (< 0) row = Just "negative field"
    | private > total = Just "private above total"
    | node >= n = Just "node out of range"
  fields _ = Nothing

-- | One unmentioned row: [node, vis, conv]. The two words are open
-- bit sets like `symbols`' visibility (only their sign is checked):
-- which bits judge is CE.Graph.Cost's business, so a bit a later core
-- wants arrives without loosening this row.
unmentionedRow :: Integer -> Int -> [Integer] -> Maybe String
unmentionedRow n = rowCheck "unmentioned" "malformed row (need [node,vis,conv])" 3 fields
 where
  fields row@[node, _, _]
    | any (< 0) row = Just "negative field"
    | node >= n = Just "node out of range"
  fields _ = Nothing

-- | The advisory table of one request: [node, vis, conv, code] for
-- every unmentioned row whose visibility carries the whole mask and
-- whose category word names no exempt category. Knobs are PARAMETERS
-- (visMask, the exempt list) bound at the CE.Graph boundary, so the
-- dead-knob battery can perturb each and watch rows appear (K19,
-- K36). One `Data.Map.Strict` over the mounts table per call: a
-- request may carry 131072 rows of each table, and a per-row rescan
-- would be ~1.7e10 comparisons (§4, W9-F6) — the `confidence` rescan
-- in Cost.hs is fine only because validation bounds `unres` to seven
-- rows. A node without a mounts row reads [0,0,0] (§4: the producer
-- covers every node; the core never refuses on coverage).
judge :: Integer -> [Integer] -> [[Integer]] -> [[Integer]] -> [[Integer]]
judge visMask exempt mounts rows =
  [ [node, vis, conv, code (mountOf node) vis]
  | [node, vis, conv] <- rows
  , vis .&. visMask == visMask
  , not (any (testBit conv . fromInteger) exempt)
  ]
 where
  byNode = M.fromList [(node, [private, total, bits]) | [node, private, total, bits] <- mounts]
  mountOf node = fromMaybe [0, 0, 0] (M.lookup node byNode)

-- | The code of one row — a frozen total order 1 > 2 > 3 > 0 (§5.2):
-- 1 private_unmentioned, nothing outside the file can reach the name
-- at all; 2 restricted_unmentioned, the crate can but no crate
-- outside it; 3 reexported_unmentioned, a façade lets it out and
-- still nobody spells it; 0 public_unmentioned, an open surface with
-- no taker. The order runs from the safest deletion to the loudest
-- API question, so a row that qualifies twice reports the narrower
-- reach.
code :: [Integer] -> Integer -> Integer
code mount vis
  | mountedPrivate mount || pkgPrivate mount = 1
  | testBit vis restrictedVisBit = 2
  | reexported mount = 3
  | otherwise = 0

-- | Every mount of the file is a private `mod` and no façade
-- re-exports it: the file is reachable only through its own package
-- tree. Zero mounts is NOT private (a lib root or a Go/TS file), the
-- empty-truth line §4 draws with `total > 0`.
mountedPrivate :: [Integer] -> Bool
mountedPrivate [private, total, bits] =
  total > 0 && private == total && not (testBit bits reexportMountBit)
mountedPrivate _ = False

-- | The file's own package keeps it private: Go `package main` or
-- `internal/`, a lib-less Cargo package or its bin roots, a cabal
-- without a library stanza or a module only in other-modules.
pkgPrivate :: [Integer] -> Bool
pkgPrivate [_, _, bits] = testBit bits pkgPrivateMountBit
pkgPrivate _ = False

-- | A façade re-exports the file: a `pub use` crossed it, or a TS
-- `export *` names it.
reexported :: [Integer] -> Bool
reexported [_, _, bits] = testBit bits reexportMountBit
reexported _ = False
