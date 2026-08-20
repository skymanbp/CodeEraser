-- | The split-ROI advisory (plan v2.6 §C): given a candidate file's
-- top-level unit spans and its intra-file mention edges, enumerate
-- every seam (the gap after each unit), price it — benefit = the
-- soft-zone penalty the split recovers (the SAME curve the verdict
-- family judges with, CE.Verdict.Soft), cost = crossing references
-- × roiRefMilli + the flat roiPhiMilli per new file — and answer
-- either the best viable seam (ROI >= 1) or an exemption row whose
-- numbers let the Rust side write the why. Everything is exact
-- integer/Rational; milli is the one published scale.
module CE.Structure.Split
  ( splitOffence
  , splitRows
  ) where

import CE.Structure.Axes (Knobs (..))
import CE.Verdict.Soft (zonePenalty)
import CE.Wire (ascendingOn)
import Data.Foldable (asum)
import Data.List (maximumBy)
import Data.Ord (comparing)
import Data.Ratio ((%))

-- | Boundary contract for the five seam tables, in request order.
-- Files are dense (id == index); units are dense PER FILE, spans
-- inside the file's total, strictly ordered and non-overlapping;
-- refs and churn pairs name in-range distinct units of their file;
-- clone spans sit inside their file's total (v2.7 ② tables).
splitOffence :: [[Integer]] -> SeamTables -> Maybe String
splitOffence files (units, refs, clones, churn) =
  asum
    [ asum (zipWith fileRow [0 :: Int ..] files)
    , asum (zipWith (unitRow files) [0 :: Int ..] units)
    , unitsOrdered units
    , asum (zipWith (refRow counts) [0 :: Int ..] refs)
    , ascendingOn "seamRefs" id refs
    , asum (zipWith (cloneRow files) [0 :: Int ..] clones)
    , ascendingOn "seamClones" id clones
    , asum (zipWith (churnRow counts) [0 :: Int ..] churn)
    , ascendingOn "seamChurn" id churn
    ]
 where
  counts = unitCounts files units
  fileRow i row = case row of
    [fid, total]
      | fid /= toInteger i -> Just ("seamFiles " <> show i <> ": index mismatch")
      | total < 1 -> Just ("seamFiles " <> show i <> ": total below 1")
      | otherwise -> Nothing
    _ -> Just ("seamFiles " <> show i <> ": malformed row (need [file,total])")

-- | (units, refs, clones, churn) — the four unit-and-edge tables
-- ride as ONE bundle so every consumer names the same shape.
type SeamTables = ([[Integer]], [[Integer]], [[Integer]], [[Integer]])

-- | The one total per dense file id — seamFiles has exactly one row
-- per id, so the comprehension yields one value; sum keeps callers
-- total without a partial head.
fileTotal :: [[Integer]] -> Integer -> Integer
fileTotal files f = sum [t | [f', t] <- files, f' == f]

-- | The shared span contract: in-range file, 1-based ordered span
-- inside the file's total — seamUnits and seamClones both speak it
-- (ONE checker, so the two tables cannot drift on what a span is).
spanOffence :: [[Integer]] -> String -> (Integer, Integer, Integer) -> Maybe String
spanOffence files lbl (f, s, e)
  | f < 0 || f >= toInteger (length files) = Just (lbl <> "file out of range")
  | s < 1 || e < s = Just (lbl <> "bad span")
  | e > fileTotal files f = Just (lbl <> "span past the file total")
  | otherwise = Nothing

-- | One unit row: the span contract on [file, unit, start, end].
-- Density and overlap are the table-level pass.
unitRow :: [[Integer]] -> Int -> [Integer] -> Maybe String
unitRow files i row = case row of
  [f, _, s, e] -> spanOffence files lbl (f, s, e)
  _ -> Just (lbl <> "malformed row (need [file,unit,start,end])")
 where
  lbl = "seamUnits " <> show i <> ": "

-- | One clone-block span (v2.7 ②): the T1/T2 block a seam must not
-- cut unpriced — the same span contract on [file, start, end].
cloneRow :: [[Integer]] -> Int -> [Integer] -> Maybe String
cloneRow files i row = case row of
  [f, s, e] -> spanOffence files lbl (f, s, e)
  _ -> Just (lbl <> "malformed row (need [file,start,end])")
 where
  lbl = "seamClones " <> show i <> ": "

-- | The shared pair contract: distinct in-range units of one file,
-- plus each table's own relation — refs refuse self edges, churn
-- pairs must ascend (an unordered pair has one spelling).
pairRow ::
  [(Integer, Integer)] -> String -> (Integer -> Integer -> Maybe String) -> Int -> [Integer] -> Maybe String
pairRow counts name rel i row = case row of
  [f, a, b]
    | Just n <- lookup f counts ->
        if any (< 0) [a, b] || any (>= n) [a, b]
          then Just (lbl <> "unit out of range")
          else (lbl <>) <$> rel a b
    | otherwise -> Just (lbl <> "file out of range")
  _ -> Just (lbl <> "malformed row (need [file,a,b])")
 where
  lbl = name <> " " <> show i <> ": "

churnRow :: [(Integer, Integer)] -> Int -> [Integer] -> Maybe String
churnRow counts =
  pairRow counts "seamChurn" (\a b -> if a >= b then Just "pair not ascending" else Nothing)

-- | Per-file density (unit == running index within its file) and
-- span order (strictly ascending, non-overlapping) in ONE fold that
-- carries (file, previous end, expected next unit).
unitsOrdered :: [[Integer]] -> Maybe String
unitsOrdered = go (0 :: Int) (-1) (-1) 0
 where
  go _ _ _ _ [] = Nothing
  go i prevF prevEnd nextU ([f, u, s, e] : rest)
    | f < prevF = Just (lbl <> "file ids descending")
    | f > prevF =
        if u /= 0
          then Just (lbl <> "unit not dense from 0")
          else go (i + 1) f e 1 rest
    | u /= nextU = Just (lbl <> "unit not dense")
    | s <= prevEnd = Just (lbl <> "spans overlap or descend")
    | otherwise = go (i + 1) f e (nextU + 1) rest
   where
    lbl = "seamUnits " <> show i <> ": "
  go i _ _ _ _ = Just ("seamUnits " <> show i <> ": malformed row")

refRow :: [(Integer, Integer)] -> Int -> [Integer] -> Maybe String
refRow counts =
  pairRow counts "seamRefs" (\a b -> if a == b then Just "self reference" else Nothing)

unitCounts :: [[Integer]] -> [[Integer]] -> [(Integer, Integer)]
unitCounts files units =
  [ (f, toInteger (length [() | [f', _, _, _] <- units, f' == f]))
  | [f, _] <- files
  ]

-- | The judged rows: (splitCandidates, sizeExempt). One best seam
-- per file when viable — argmax ROI by cross-multiplied compare
-- (cost is never 0: phi >= 1 by the knob rule) — else an exemption
-- row carrying the best seam's numbers (0/0 when the file has no
-- seam at all: a single unit cannot be split).
splitRows :: Knobs -> [[Integer]] -> SeamTables -> ([[Integer]], [[Integer]])
splitRows k files tables = foldl' step ([], []) files
 where
  step (cands, exempts) [fid, total] = case seams k (fid, total) tables of
    [] -> (cands, exempts <> [[fid, 0, 0]])
    priced ->
      let (u, b, c) = maximumBy (comparing (\(_, b', c') -> b' % c')) priced
       in if b >= c
            then (cands <> [[fid, u, b, c]], exempts)
            else (cands, exempts <> [[fid, b, c]])
  step acc _ = acc

-- | Every seam of one file as (afterUnit, benefitMilli, costMilli).
-- Cost (v2.7 ② full pricing): severed references × ref price, cut
-- clone blocks (span straddling the seam line) × clone price,
-- crossing co-change pairs × churn price, plus the flat φ.
seams :: Knobs -> (Integer, Integer) -> SeamTables -> [(Integer, Integer, Integer)]
seams k (fid, total) (units, refs, clones, churn) =
  [ (u, benefit end, cost u end)
  | ([_, u, _, end], _) <- zip mine (drop 1 mine)
  ]
 where
  mine = [row | row@[f, _, _, _] <- units, f == fid]
  p = zonePenalty (kSeamSoft k) (kSeamHard k) (kSeamPMax k)
  benefit end =
    max 0 (floor (1000 * (p total - p end - p (total - end))))
  cross rows u = toInteger (length [() | [f, a, b] <- rows, f == fid, (a <= u) /= (b <= u)])
  cut line = toInteger (length [() | [f, s, e] <- clones, f == fid, s <= line, line < e])
  cost u end =
    cross refs u * kRoiRefMilli k
      + cut end * kRoiCloneMilli k
      + cross churn u * kRoiChurnMilli k
      + kRoiPhiMilli k
