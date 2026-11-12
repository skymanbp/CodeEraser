-- | The boundary contract of the extent law's four request tables
-- (plan v2.35 score amendment). They ride on every request (10.0.0):
--
--   contSpans  aligned with `continuous`, one [file, lo, hi] per row:
--              the row's file in the universe (-1 = outside it, the
--              scan-only arm) and, for a function row, its span; every
--              other row carries [file, 0, 0]
--   dupSpans   [file, lo, hi, kind]: a member span of a verified
--              duplicate, kind as in `sim` (0, 1 clone, 2 docdup)
--   refLines   [src, dst, line]: a reference line the graph judged on
--   proseLines [file, lines]: the line count of a universe file that
--              owns no size row
--
-- The last three strictly ascending on the whole row (on the file for
-- proseLines); integers only. Every universe file has exactly one line
-- count (its size row or its proseLines row), and a duplicate span or
-- a reference line lies inside its file. Function spans are clipped to
-- the file by the law (CE.Verdict.Extent), not refused here.
module CE.Verdict.ExtentRows (extentOffence) where

import CE.Verdict.Table (table)
import CE.Wire (label)
import Data.Foldable (asum)
import qualified Data.IntMap.Strict as IM

-- | The first offender, or Nothing. `cont` is the validated continuous
-- table; the four tables in their wire order.
extentOffence :: Integer -> [[Integer]] -> (Maybe [[Integer]], Maybe [[Integer]], Maybe [[Integer]], Maybe [[Integer]]) -> Maybe String
extentOffence n cont tables = case tables of
  (Just spans, Just dups, Just refs, Just prose) -> checked n cont (spans, dups, refs, prose)
  _ -> Just "extent: contSpans, dupSpans, refLines and proseLines ride on every request"

checked :: Integer -> [[Integer]] -> ([[Integer]], [[Integer]], [[Integer]], [[Integer]]) -> Maybe String
checked n cont (spans, dups, refs, prose) =
  asum
    [ if length spans /= length cont then Just "contSpans: need one row per continuous row" else Nothing
    , asum (zipWith3 (spanRow n) [0 :: Int ..] cont spans)
    , secondSize sized
    , table "proseLines" (proseRow n (IM.fromList sized)) 1 prose
    , asum [Just ("extent: file " <> show u <> " has no line count") | u <- [0 .. n - 1], IM.notMember (fromInteger u) len]
    , table "dupSpans" (dupRow n len) 4 dups
    , table "refLines" (refRow n len) 3 refs
    ]
 where
  -- (file, size-row index) of every placed size row, in row order
  sized = [(fromInteger u, i) | (i, (_ : 0 : _), (u : _)) <- zip3 [0 :: Int ..] cont spans, u >= 0]
  len =
    IM.fromList ([(fromInteger u, v) | ((_ : 0 : v : _), (u : _)) <- zip cont spans, u >= 0] <> [(fromInteger u, l) | [u, l] <- prose])

spanRow :: Integer -> Int -> [Integer] -> [Integer] -> Maybe String
spanRow n i row s = case (row, s) of
  ((_ : code : _), [u, lo, hi])
    | u < -1 || u >= n -> Just (label "contSpans" i <> "unknown file index")
    | code == 1 && (lo < 1 || hi < lo) -> Just (label "contSpans" i <> "a function span needs 1 <= lo <= hi")
    | code /= 1 && (lo, hi) /= (0, 0) -> Just (label "contSpans" i <> "only a function row carries a span")
    | otherwise -> Nothing
  _ -> Just (label "contSpans" i <> "malformed row (need [file,lo,hi])")

-- | One size row per placed file: the second is named by its row.
secondSize :: [(Int, Int)] -> Maybe String
secondSize = go IM.empty
 where
  go seen ((u, i) : rest)
    | IM.member u seen = Just (label "contSpans" i <> "a second size row for one file")
    | otherwise = go (IM.insert u () seen) rest
  go _ [] = Nothing

proseRow :: Integer -> IM.IntMap Int -> String -> Int -> [Integer] -> Maybe String
proseRow n sized name i row = case row of
  [u, l]
    | u < 0 || u >= n -> Just (label name i <> "unknown file index")
    | l < 0 -> Just (label name i <> "negative line count")
    | IM.member (fromInteger u) sized -> Just (label name i <> "the file owns a size row")
    | otherwise -> Nothing
  _ -> Just (label name i <> "malformed row (need [file,lines])")

-- | A duplicate span [file, lo, hi, kind] and a reference line [src,
-- dst, line] (as [src, line, line] plus dst), each placed in its file.
dupRow, refRow :: Integer -> IM.IntMap Integer -> String -> Int -> [Integer] -> Maybe String
dupRow n len name i row = (label name i <>) <$> case row of
  [u, lo, hi, kind]
    | kind < 0 || kind > 2 -> known n [u] (Just "unknown kind")
    | hi < lo -> known n [u] (Just "span hi < lo")
    | otherwise -> known n [u] (placed len "span" u lo hi)
  _ -> Just "malformed row (need [file,lo,hi,kind])"
refRow n len name i row = (label name i <>) <$> case row of
  [src, dst, line] -> known n [src, dst] (placed len "line" src line line)
  _ -> Just "malformed row (need [src,dst,line])"

-- | An unknown file index first, then the row's own offence.
known :: Integer -> [Integer] -> Maybe String -> Maybe String
known n us rest = if any (\u -> u < 0 || u >= n) us then Just "unknown file index" else rest

-- | [lo, hi] inside the file's lines.
placed :: IM.IntMap Integer -> String -> Integer -> Integer -> Integer -> Maybe String
placed len what u lo hi = if lo < 1 || hi > IM.findWithDefault 0 (fromInteger u) len then Just (what <> " outside its file") else Nothing
