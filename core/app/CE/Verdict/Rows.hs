-- | Row-shape validators the wire boundary refuses with. Split from
-- CE.Verdict.Wire at the repo's own core-size gate, plan v2.12; these
-- checkers keep the refusal vocabulary at the boundary.
module CE.Verdict.Rows
  ( tierRow
  , pairRow
  , simRow
  , nodeRow
  , posRow
  , contRow
  , discEntry
  , docFilesOffence
  , floorOffence
  , judgedLocOffence
  , presentOffence
  , selfLoopsOffence
  ) where

import CE.Verdict.Cost (classIdPastFence, scoreScale)
import CE.Verdict.Table (ascendingBy, label)
import Control.Applicative ((<|>))
import Data.Foldable (asum)
import qualified Data.IntSet as IS

tierRow :: Int -> [Integer] -> Maybe String
tierRow i row = case row of
  [u, code]
    | u /= toInteger i -> Just ("tier " <> show i <> ": index mismatch")
    | code < 0 || code > 1 -> Just ("tier " <> show i <> ": unknown tier code")
    | otherwise -> Nothing
  _ -> Just ("tier " <> show i <> ": malformed row (need [u,tier])")

pairRow :: Integer -> Int -> String -> Int -> [Integer] -> Maybe String
pairRow n arity name i row = case row of
  (u : v : _)
    | length row /= arity -> Just (label name i <> "malformed row")
    | any (< 0) row -> Just (label name i <> "negative field")
    | u >= n || v >= n -> Just (label name i <> "endpoint out of range")
    | u >= v -> Just (label name i <> "pair not ascending")
    | otherwise -> Nothing
  _ -> Just (label name i <> "malformed row")

-- | sim rows carry the ONE enum + ratio the module used to leave
-- unchecked (review MED: Join routed an out-of-enum kind to the
-- clone bar while Score scored it zero, and den = 0 made the
-- cross-multiplication vacuously true 鈥?a certain clone from 0/0).
simRow :: Integer -> String -> Int -> [Integer] -> Maybe String
simRow n name i row =
  pairRow n 5 name i row <|> case row of
    [_, _, kind, _, den]
      | kind > 2 -> Just (label name i <> "unknown sim kind")
      | den == 0 -> Just (label name i <> "zero denominator")
      | otherwise -> Nothing
    _ -> Nothing

nodeRow :: Integer -> Int -> String -> Int -> [Integer] -> Maybe String
nodeRow n arity name i row = case row of
  (u : _)
    | length row /= arity -> Just (label name i <> "malformed row")
    | any (< 0) row -> Just (label name i <> "negative field")
    | u >= n -> Just (label name i <> "node out of range")
    | otherwise -> Nothing
  _ -> Just (label name i <> "malformed row")

posRow :: IS.IntSet -> Integer -> Int -> [Integer] -> Maybe String
posRow unitTier n i row = case row of
  [u, indeg, outdeg, sccId, sccSize, reachIn]
    | any (< 0) [u, indeg, outdeg, sccId, sccSize, reachIn] ->
        Just (label "pos" i <> "negative field")
    | u >= n -> Just (label "pos" i <> "node out of range")
    -- range-checked above and the tier table is dense by the time
    -- this runs, so set membership IS "tier code /= 0"
    | IS.member (fromInteger u) unitTier -> Just (label "pos" i <> "unit-tier node")
    | otherwise -> Nothing
  _ -> Just (label "pos" i <> "malformed row (need 6 fields)")

-- | continuous entities are FINGERPRINTS (u64), not tier indexes:
-- the ratchet joins current-vs-baseline on (u, code) across runs,
-- and a tier index shifts whenever a file lands 鈥?so u here is
-- range-checked against u64, never against the node universe.
-- Three columns, or four with the rulepack class (3.1.0, plan v2.13
-- ①): the class is bounded by the fence here, and the TABLE keeps
-- one arity (Table.uniformArity at the boundary — the graph/1
-- node-row precedent), so a half-classed table refuses instead of
-- being read two ways at once.
contRow :: String -> Int -> [Integer] -> Maybe String
contRow name i row = case row of
  (u : code : _ : rest)
    | length rest > 1 -> Just (label name i <> "malformed row")
    | any (< 0) row -> Just (label name i <> "negative field")
    | u >= 18446744073709551616 -> Just (label name i <> "outside u64")
    | code > 6 -> Just (label name i <> "unknown metric code")
    | any classIdPastFence rest -> Just (label name i <> "class beyond the fence")
    | otherwise -> Nothing
  _ -> Just (label name i <> "malformed row")

discEntry :: Int -> Integer -> Maybe String
discEntry i x
  | x < 0 || x >= 18446744073709551616 = Just (label "discrete" i <> "outside u64")
  | otherwise = Nothing

docFilesOffence :: Integer -> [Integer] -> Maybe String
docFilesOffence = indexSet "docFiles"

-- | An ascending SET of file-universe indices under one label:
-- docFiles (2.27.0) and cycleSelfLoops (6.4.0) are one shape.
indexSet :: String -> Integer -> [Integer] -> Maybe String
indexSet name n files =
  asum
    [ asum (zipWith one [0 :: Int ..] files)
    , ascendingBy name 1 (map pure files)
    ]
 where
  one i u
    | u < 0 || u >= n = Just (label name i <> "node out of range")
    | otherwise = Nothing

-- | The provenance table (6.4.0, O40): ascending u64 file entities
-- that exist under the scope but were not measured. A baseline row
-- of one is DROPPED — a named fail — where a row of an entity that
-- is simply gone stays a removal; the entities are the continuous
-- table's own fingerprints, so the same u64 bound applies.
presentOffence :: [Integer] -> Maybe String
presentOffence rows =
  asum (zipWith one [0 :: Int ..] rows) <|> ascendingBy "present" 1 (map pure rows)
 where
  one i x
    | x < 0 || x >= 18446744073709551616 = Just (label "present" i <> "outside u64")
    | otherwise = Nothing

-- | The self-loop table (6.4.0, O59): the file indices whose node
-- carries an exact self-arc, REQUIRED exactly when the cycle floor
-- (thresholds code 7) is 1 — a singleton SCC is a cycle only through
-- its own arc, and without the table floor 1 would count every
-- isolated file — and refused at any other floor, where it means
-- nothing. Present, it is an ascending index set like docFiles.
selfLoopsOffence :: Integer -> [[Integer]] -> Maybe [Integer] -> Maybe String
selfLoopsOffence n thrs loops = case (floorOne, loops) of
  (True, Nothing) -> Just "cycleSelfLoops: required at cycleFloor 1"
  (False, Just _) -> Just "cycleSelfLoops: only meaningful at cycleFloor 1"
  (_, Just rows) -> indexSet "cycleSelfLoops" n rows
  _ -> Nothing
 where
  floorOne = take 1 [v | [7, v] <- thrs] == [1]

-- | The floor is bounded by the EFFECTIVE score scale (review C7:
-- the 1000 literal survived scoreScale becoming a knob in the same
-- batch 鈥?a floor above the scale can never pass, and one above
-- every reachable score would fail forever undiagnosed; both refuse
-- by name now). The thresholds table is validated before this row
-- of the asum, so the [6, v] scan reads checked rows only.
floorOffence :: [[Integer]] -> Maybe Integer -> Maybe String
floorOffence _ Nothing = Nothing
floorOffence thrs (Just f)
  | f < 0 || f > scale = Just "floor: outside the effective score scale"
  | otherwise = Nothing
 where
  scale = last (scoreScale : [v | [6, v] <- thrs])

-- | plan v2.6 搂B: the judged-LOC multiset is values only,
-- non-descending (duplicates are the POINT of a multiset 鈥?the
-- strict ascendingBy would refuse two same-length files), in u64.
judgedLocOffence :: [Integer] -> Maybe String
judgedLocOffence locs =
  asum
    [ asum (zipWith one [0 :: Int ..] locs)
    , asum
        [ if prev <= cur
            then Nothing
            else Just (label "judgedLoc" i <> "not non-descending")
        | (i, (prev, cur)) <- zip [1 :: Int ..] (zip locs (drop 1 locs))
        ]
    ]
 where
  one i x
    | x < 0 || x >= 18446744073709551616 = Just (label "judgedLoc" i <> "outside u64")
    | otherwise = Nothing
