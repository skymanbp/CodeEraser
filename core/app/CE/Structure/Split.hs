-- | The split-ROI advisory (plan v2.6 §C): given a candidate file's
-- top-level unit spans and its intra-file mention edges, enumerate
-- every seam (the gap after each unit), price it — benefit = the
-- soft-zone penalty the split recovers (the SAME curve the verdict
-- family judges with, CE.Verdict.Soft), cost = crossing references
-- × roiRefMilli + the flat roiPhiMilli per new file — and answer
-- either the best viable seam (ROI >= 1) or an exemption row whose
-- numbers let the Rust side write the why. Everything is exact
-- integer/Rational; milli is the one published scale.
--
-- Both faces group the five tables by file id ONCE (the Declared.hs
-- / Axes.hs Data.Map idiom) and charge each seam by prefix sum. The
-- pre-fix form rescanned instead: seamFiles per span row, seamUnits
-- per file, and the whole refs/clones/churn table per SEAM.
-- structNodeCap admits 524288 seam rows, so a contract-VALID request
-- cost ~1e11 list steps — not slow, unanswerable (review 2026-08-20
-- findings #1 and #2).
module CE.Structure.Split
  ( splitOffence
  , splitRows
  ) where

import CE.Structure.Axes (Knobs (..))
import CE.Verdict.Soft (zonePenalty)
import CE.Wire (tableOffence)
import Data.Foldable (asum)
import Data.List (maximumBy)
import qualified Data.Map.Strict as M
import Data.Ord (comparing)
import Data.Ratio ((%))

-- | (units, refs, clones, churn) — the four unit-and-edge tables
-- ride as ONE bundle so every consumer names the same shape.
type SeamTables = ([[Integer]], [[Integer]], [[Integer]], [[Integer]])

-- | One file's slice of that bundle, same order: unit (id, end)
-- pairs in ascending span order, ref pairs, clone (start, end)
-- spans, churn pairs.
type Slice =
  ([(Integer, Integer)], [(Integer, Integer)], [(Integer, Integer)], [(Integer, Integer)])

-- | Boundary contract for the five seam tables, in request order.
-- Files are dense (id == index); units are dense PER FILE, spans
-- inside the file's total, strictly ordered and non-overlapping;
-- refs and churn pairs name in-range distinct units of their file;
-- clone spans sit inside their file's total (v2.7 ② tables).
splitOffence :: [[Integer]] -> SeamTables -> Maybe String
splitOffence files (units, refs, clones, churn) =
  asum
    [ asum (zipWith fileRow [0 :: Int ..] files)
    , asum (zipWith (unitRow totals) [0 :: Int ..] units)
    , unitsOrdered units
    , tableOffence "seamRefs" id (refRow counts) refs
    , tableOffence "seamClones" id (cloneRow totals) clones
    , tableOffence "seamChurn" id (churnRow counts) churn
    ]
 where
  -- asum is lazy, so both maps are demanded only once every
  -- seamFiles row has passed fileRow — malformed rows never reach
  -- them, and no row-level check rescans a table again
  totals = M.fromList [(f, t) | [f, t] <- files]
  counts = unitCounts files units
  fileRow i row = case row of
    [fid, total]
      | fid /= toInteger i -> Just ("seamFiles " <> show i <> ": index mismatch")
      | total < 1 -> Just ("seamFiles " <> show i <> ": total below 1")
      | otherwise -> Nothing
    _ -> Just ("seamFiles " <> show i <> ": malformed row (need [file,total])")

-- | The shared span contract: in-range file, 1-based ordered span
-- inside the file's total — seamUnits and seamClones both speak it
-- (ONE checker, so the two tables cannot drift on what a span is).
-- seamFiles is dense by fileRow, so an ABSENT total is exactly an
-- out-of-range file id: one lookup where the old fileTotal summed
-- the whole seamFiles table per span row.
spanOffence :: M.Map Integer Integer -> String -> (Integer, Integer, Integer) -> Maybe String
spanOffence totals lbl (f, s, e) = case M.lookup f totals of
  Nothing -> Just (lbl <> "file out of range")
  Just total
    | s < 1 || e < s -> Just (lbl <> "bad span")
    | e > total -> Just (lbl <> "span past the file total")
    | otherwise -> Nothing

-- | One unit row: the span contract on [file, unit, start, end].
-- Density and overlap are the table-level pass.
unitRow :: M.Map Integer Integer -> Int -> [Integer] -> Maybe String
unitRow totals i row = case row of
  [f, _, s, e] -> spanOffence totals lbl (f, s, e)
  _ -> Just (lbl <> "malformed row (need [file,unit,start,end])")
 where
  lbl = "seamUnits " <> show i <> ": "

-- | One clone-block span (v2.7 ②): the T1/T2 block a seam must not
-- cut unpriced — the same span contract on [file, start, end].
cloneRow :: M.Map Integer Integer -> Int -> [Integer] -> Maybe String
cloneRow totals i row = case row of
  [f, s, e] -> spanOffence totals lbl (f, s, e)
  _ -> Just (lbl <> "malformed row (need [file,start,end])")
 where
  lbl = "seamClones " <> show i <> ": "

-- | The shared pair contract: distinct in-range units of one file,
-- plus each table's own relation — refs refuse self edges, churn
-- pairs must ascend (an unordered pair has one spelling).
pairRow ::
  M.Map Integer Integer -> String -> (Integer -> Integer -> Maybe String) -> Int -> [Integer] -> Maybe String
pairRow counts name rel i row = case row of
  [f, a, b]
    | Just n <- M.lookup f counts ->
        if any (< 0) [a, b] || any (>= n) [a, b]
          then Just (lbl <> "unit out of range")
          else (lbl <>) <$> rel a b
    | otherwise -> Just (lbl <> "file out of range")
  _ -> Just (lbl <> "malformed row (need [file,a,b])")
 where
  lbl = name <> " " <> show i <> ": "

churnRow :: M.Map Integer Integer -> Int -> [Integer] -> Maybe String
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

refRow :: M.Map Integer Integer -> Int -> [Integer] -> Maybe String
refRow counts =
  pairRow counts "seamRefs" (\a b -> if a == b then Just "self reference" else Nothing)

-- | Units per file id. Every declared file gets an entry, zero
-- included, so a file with no unit row refuses its edges by "unit
-- out of range" — the old list-of-pairs form said the same thing at
-- files × units cost, and every pairRow lookup then walked it.
unitCounts :: [[Integer]] -> [[Integer]] -> M.Map Integer Integer
unitCounts files units =
  M.unionWith (+) (M.fromList [(f, 0) | [f, _] <- files]) perFile
 where
  perFile = M.fromListWith (+) [(f, 1) | [f, _, _, _] <- units]

-- | The judged rows: (splitCandidates, sizeExempt). One best seam
-- per file when viable — argmax ROI by cross-multiplied compare
-- (cost is never 0: phi >= 1 by the knob rule) — else an exemption
-- row carrying the best seam's numbers (0/0 when the file has no
-- seam at all: a single unit cannot be split). Rows CONS and the
-- pair reverses once: the old `cands <> [row]` re-walked the answer
-- built so far on every file, F² over the files the cap admits.
splitRows :: Knobs -> [[Integer]] -> SeamTables -> ([[Integer]], [[Integer]])
splitRows k files (units, refs, clones, churn) = (reverse cands, reverse exempts)
 where
  (cands, exempts) = foldl' step ([], []) files
  byUnit = byFile [(f, (u, e)) | [f, u, _, e] <- units]
  byRef = byFile [(f, (a, b)) | [f, a, b] <- refs]
  byClone = byFile [(f, (s, e)) | [f, s, e] <- clones]
  byChurn = byFile [(f, (a, b)) | [f, a, b] <- churn]
  slice f = (grab byUnit, grab byRef, grab byClone, grab byChurn)
   where
    grab = M.findWithDefault [] f
  step (cs, es) [fid, total] = case seams k (slice fid) total of
    [] -> (cs, [fid, 0, 0] : es)
    priced ->
      let (u, b, c) = maximumBy (comparing (\(_, b', c') -> b' % c')) priced
       in if b >= c
            then ([fid, u, b, c] : cs, es)
            else (cs, [fid, b, c] : es)
  step acc _ = acc

-- | One table bucketed by file id, table order preserved (the
-- fromListWith accumulator prepends, hence the single reverse).
byFile :: [(Integer, a)] -> M.Map Integer [a]
byFile rows = M.map reverse (M.fromListWith (++) [(f, [v]) | (f, v) <- rows])

-- | Every seam of one file as (afterUnit, benefitMilli, costMilli).
-- Cost (v2.7 ② full pricing): severed references × ref price, cut
-- clone blocks (span straddling the seam line) × clone price,
-- crossing co-change pairs × churn price, plus the flat φ. All three
-- crossing tables are built once per FILE and read per seam.
seams :: Knobs -> Slice -> Integer -> [(Integer, Integer, Integer)]
seams k (mine, refs, clones, churn) total =
  [(u, benefit end, cost u) | (u, end) <- open]
 where
  -- the seams are the gaps after every unit BUT the last
  open = zipWith const mine (drop 1 mine)
  p = zonePenalty (kSeamSoft k) (kSeamHard k) (kSeamPMax k)
  benefit end = max 0 (floor (1000 * (p total - p end - p (total - end))))
  refX = crossings refs
  churnX = crossings churn
  cutX = cuts open clones
  cost u =
    charge refX u * kRoiRefMilli k
      + charge cutX u * kRoiCloneMilli k
      + charge churnX u * kRoiChurnMilli k
      + kRoiPhiMilli k

-- | Severed-edge count per seam. A pair (a, b) crosses exactly the
-- seams after units min(a,b) .. max(a,b)−1 — ONE contiguous run —
-- so a +1/−1 delta per edge plus a running sum answers every seam of
-- the file in one pass, where the old form rescanned the whole edge
-- table per seam.
crossings :: [(Integer, Integer)] -> M.Map Integer Integer
crossings pairs = running (M.fromListWith (+) (concatMap ends pairs))
 where
  ends (a, b) = [(min a b, 1), (max a b, -1)]

-- | The same delta trick for clone blocks: [s, e) straddles exactly
-- the seams whose LINE falls in [s, e), and a file's unit spans
-- ascend, so those seams are again a contiguous run — bounded here
-- by two ordered lookups instead of a rescan of seamClones.
cuts :: [(Integer, Integer)] -> [(Integer, Integer)] -> M.Map Integer Integer
cuts open clones = running (M.fromListWith (+) (concatMap straddle clones))
 where
  lineIx = M.fromList [(e, u) | (u, e) <- open]
  straddle (s, e) = case (M.lookupGE s lineIx, M.lookupLT e lineIx) of
    (Just (_, lo), Just (_, hi)) | lo <= hi -> [(lo, 1), (hi + 1, -1)]
    _ -> []

-- | Running totals in ascending key order: the value at a key holds
-- for every seam from it up to the next key.
running :: M.Map Integer Integer -> M.Map Integer Integer
running = snd . M.mapAccum (\acc d -> let s = acc + d in (s, s)) 0

charge :: M.Map Integer Integer -> Integer -> Integer
charge m u = maybe 0 snd (M.lookupLE u m)
