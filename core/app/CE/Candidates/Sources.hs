{-# LANGUAGE BangPatterns #-}

-- | The three index-bound T3 candidate sources over integer facts
-- (design vol.2 §4.2; plan v2.33 W3), generated here since W3 — the
-- measuring side sends what its index holds and keeps only the T1/T2
-- extension that finds the near runs:
--
-- * S1 — the verified near-miss runs the T1/T2 report threshold drops,
--   each as two (file, line) anchors;
-- * S3 — raw fingerprint co-occurrence: instances grouped by hash,
--   every two members paired (adjacent ones above the hot cap, sorted
--   by file then token offset), each as two (file, line) anchors;
-- * S4 — MinHash/LSH over each unit's structural shingle set
--   (CE.Candidates.Lsh), every two units sharing a band bucket.
--
-- A line anchor resolves to the innermost admitted unit containing the
-- line (the narrowest span, the earliest unit on a tie) or is counted
-- unowned, never guessed. A pair is canonical (lower unit first); a
-- self pair and a cross-language pair are counted and dropped; each
-- source counts its distinct pairs per language. The union carries each
-- finding source's bit.
module CE.Candidates.Sources (Gen (..), generate) where

import CE.Candidates.Cost (bandSource, lshShape, nearSource, printSource)
import CE.Candidates.Groups (Paired (..), paired)
import CE.Candidates.Lsh (Sets, bandBuckets)
import CE.Candidates.Units (Units, endOf, startOf)
import qualified CE.Candidates.Units as U
import CE.Dedup.Cost (hotCap)
import Data.Array (Array, elems, listArray, (!))
import Data.Array.Base (unsafeAt)
import Data.Bits ((.|.))
import qualified Data.IntMap.Strict as IM
import qualified Data.IntSet as IS
import Data.List (sortOn)

-- | The union (pair key → source bits) and every count the walks shed:
-- each source's distinct pairs per language (keyed by source bit),
-- cross-language and self pairs, unowned anchors, chained hash groups
-- (S3) and bucket groups (S4), and S4's bucket sizes.
data Gen = Gen
  { unioned :: !(IM.IntMap Int)
  , raw :: ![(Int, IM.IntMap Int)]
  , cross :: !Int
  , unowned :: !Int
  , selfPairs :: !Int
  , printHot :: !Int
  , bandHot :: !Int
  , bandGroups :: !(IM.IntMap Int)
  }

-- | One source's walk: its distinct pair keys, pairs per language, and
-- the cross / unowned / self counts.
data Acc = Acc !IS.IntSet !(IM.IntMap Int) !Int !Int !Int

-- | One fingerprint instance: hash, file, line, token offset.
type Print = (Int, Int, Int, Int)

-- | The three sources over the units, their sets, the fingerprint
-- instances `[hash, file, line, tok]` and the near runs `[fileA, lineA,
-- fileB, lineB]` (the contract admitted every row's shape).
generate :: Units -> Sets -> [[Integer]] -> [[Integer]] -> Gen
generate us sigs printRows near =
  Gen
    { unioned = IM.unionsWith (.|.) [IM.fromSet (const bit) ks | (bit, Acc ks _ _ _ _) <- walks]
    , raw = [(bit, perLang) | (bit, Acc _ perLang _ _ _) <- walks]
    , cross = sum [c | (_, Acc _ _ c _ _) <- walks]
    , unowned = sum [o | (_, Acc _ _ _ o _) <- walks]
    , selfPairs = sum [s | (_, Acc _ _ _ _ s) <- walks]
    , printHot = length (filter chained printGroups)
    , bandHot = length (filter chained bandPaired)
    , bandGroups = IM.fromListWith (+) [(length b, 1) | b <- buckets]
    }
 where
  cap = fromInteger hotCap
  owner = ownerOf us
  anchored (fa, la) (fb, lb) = (,) <$> owner fa la <*> owner fb lb
  nearEvents = [anchored (fa, la) (fb, lb) | [fa, la, fb, lb] <- map (map fromInteger) near]
  prints = listArray (0, length printRows - 1) [quad r | r <- printRows] :: Array Int Print
  quad r = case map fromInteger r of
    [h, f, l, t] -> (h, f, l, t)
    _ -> (0, 0, 0, 0) -- unreachable: the contract admitted [hash,file,line,tok]
  lineOf i = let (_, f, l, _) = prints ! i in (f, l)
  printGroups = map (paired cap) (hashGroups prints)
  printEvents = [anchored (lineOf a) (lineOf b) | g <- printGroups, (a, b) <- pairsOf g]
  buckets = bandBuckets (both lshShape) (const True) sigs
  both (p, b, r) = (fromInteger p, fromInteger b, fromInteger r)
  bandPaired = map (paired cap) buckets
  bandEvents = [Just ab | g <- bandPaired, ab <- pairsOf g]
  walks =
    [ (fromInteger nearSource, walk us nearEvents)
    , (fromInteger printSource, walk us printEvents)
    , (fromInteger bandSource, walk us bandEvents)
    ]

-- | One source's events folded: unowned, self, cross-language, or a
-- new distinct pair counted under its language.
walk :: Units -> [Maybe (Int, Int)] -> Acc
walk us = foldl' step (Acc IS.empty IM.empty 0 0 0)
 where
  n = U.count us
  langOf = unsafeAt (U.lang us)
  step acc@(Acc ks perLang c o s) ev = case ev of
    Nothing -> Acc ks perLang c (o + 1) s
    Just (x, y)
      | x == y -> Acc ks perLang c o (s + 1)
      | langOf x /= langOf y -> Acc ks perLang (c + 1) o s
      | IS.member k ks -> acc
      | otherwise -> Acc (IS.insert k ks) (IM.insertWith (+) (langOf x) 1 perLang) c o s
     where
      k = min x y * n + max x y

-- | The fingerprint instances grouped by hash, each group in instance
-- order; a group above the hot cap is walked sorted by file then token
-- offset (a stable sort: ties keep instance order).
hashGroups :: Array Int Print -> [[Int]]
hashGroups prints =
  [ if length members > fromInteger hotCap then sortOn place members else members
  | members <- IM.elems grouped
  , length members > 1
  ]
 where
  grouped = IM.fromListWith (++) [(h, [i]) | (i, (h, _, _, _)) <- reverse (zip [0 ..] (elems prints))]
  place i = let (_, f, _, t) = prints ! i in (f, t)

-- | The innermost admitted unit of a file containing a line: the
-- narrowest span, the earliest unit on a tie.
ownerOf :: Units -> Int -> Int -> Maybe Int
ownerOf us = \f l -> pick [u | u <- IM.findWithDefault [] f byFile, startOf us u <= l, l <= endOf us u]
 where
  byFile = IM.fromListWith (++) [(U.file us `unsafeAt` u, [u]) | u <- [U.count us - 1, U.count us - 2 .. 0]]
  pick members = case members of
    [] -> Nothing
    (u : rest) -> Just (foldl' (\best v -> if width v < width best then v else best) u rest)
  width u = endOf us u - startOf us u
