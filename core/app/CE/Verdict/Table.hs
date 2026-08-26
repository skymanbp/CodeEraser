-- | The shared row grammar and the knob-table judges (ADR-008),
-- split from CE.Verdict.Wire at the 300-line law: `table` walks
-- rows with a per-row checker then demands strictly-ascending
-- identities, and `knobTable` is its [code, value] instantiation —
-- the four knob families (weights, ceilings, thresholds, tolerance)
-- differ only in DATA: the code bound, its refusal text, and the
-- per-code value judgment.
module CE.Verdict.Table
  ( label
  , ascendingBy
  , table
  , weightsOffence
  , ceilingsOffence
  , thresholdsOffence
  , toleranceOffence
  , dedupOffence
  , dedupDistinctOffence
  , uniformArity
  , classKnobsOffence
  ) where

import CE.Verdict.Cost (classCap, classTolCode)
import Control.Applicative ((<|>))
import Data.Foldable (asum)
import Data.List (nub)

label :: String -> Int -> String
label name i = name <> " " <> show i <> ": "

-- | Strictly ascending on the first `k` fields — the row's IDENTITY;
-- duplicate identities are refused by implication, whatever the
-- payload says.
ascendingBy :: String -> Int -> [[Integer]] -> Maybe String
ascendingBy name k rows =
  asum
    [ if take k prev < take k cur
        then Nothing
        else Just (label name i <> "not strictly ascending")
    | (i, (prev, cur)) <- zip [1 ..] (zip rows (drop 1 rows))
    ]

table :: String -> (String -> Int -> [Integer] -> Maybe String) -> Int -> [[Integer]] -> Maybe String
table name rowCheck idWidth rows =
  asum (zipWith (rowCheck name) [0 :: Int ..] rows) <|> ascendingBy name idWidth rows

-- | One arity per table (3.1.0): a row set mixing widths would be
-- read two ways at once — refused by name, the graph/1 precedent.
uniformArity :: String -> [[Integer]] -> Maybe String
uniformArity name rows = case nub (map length rows) of
  (_ : _ : _) -> Just (name <> " rows: mixed arity")
  _ -> Nothing

-- | The rulepack's knob rows [classId, code, value] (3.1.0, plan
-- v2.13 ①): a class from 1 below the fence (class 0 IS the global
-- table, which has the ceilings channel already), a code in the
-- ceilings' own 0..2 — joined at 5.1.0 by classTolCode = 3, the
-- class's own ratchet allowance — the value floored per code
-- below, (classId, code) strictly ascending. The ceilingsOffence
-- reading, one class dimension wider.
classKnobsOffence :: [[Integer]] -> Maybe String
classKnobsOffence = table "classKnobs" one 2
 where
  one nm i row = case row of
    [c, code, v]
      | c < 1 -> Just (label nm i <> "class 0 has no override channel")
      | c >= classCap -> Just (label nm i <> "class beyond the fence")
      | code < 0 || code > classTolCode -> Just (label nm i <> "unknown class knob code")
      | v < floorFor code -> Just (label nm i <> "knob below " <> show (floorFor code))
      | otherwise -> Nothing
    _ -> Just (label nm i <> "malformed row (need [class,code,value])")
  -- codes 0/1/2 are LINES and a line of zero is nonsense; the
  -- tolerance (5.1.0) is an allowance, and zero allowance is its
  -- whole point — a frozen fixture tree that may not grow by one
  -- line. One bound for all of them would have to be the loosest,
  -- which is how a nonsense ceiling gets in.
  floorFor :: Integer -> Integer
  floorFor code = if code == classTolCode then 0 else 1

-- | [code, value] rows, code-bounded, judged per code — generalized
-- for P4 (the thresholds table judges denominators and divisor
-- knobs by CODE). The dedup ratchet caught the second family
-- cloning the first's scaffolding; this shared grammar is the
-- throat.
knobTable :: String -> Integer -> String -> (Integer -> Integer -> Maybe String) -> [[Integer]] -> Maybe String
knobTable name axisMax axisWhy judgeV = table name one 1
 where
  one nm i row = case row of
    [code, v]
      | code < 0 || code > axisMax -> Just (label nm i <> axisWhy)
      | Just why <- judgeV code v -> Just (label nm i <> why)
      | otherwise -> Nothing
    _ -> Just (label nm i <> "malformed row (need [axis,value])")

-- | A request may zero SOME axes (unlisted ones default to 1);
-- zeroing all seven leaves the score with no divisor and is refused.
weightsOffence :: [[Integer]] -> Maybe String
weightsOffence rows =
  knobTable "weights" 6 "unknown axis code" negW rows
    <|> if length rows == 7 && all zeroed rows
      then Just "weights: every axis zeroed"
      else Nothing
 where
  negW _ w = if w < 0 then Just "negative field" else Nothing
  zeroed row = case row of
    [_, w] -> w == 0
    _ -> False

-- | Codes 0..4 since 2.14.0: size soft-fallback (0), coc (1), and
-- the v0.6 zone knobs — hard line H (2), P_max (3), soft-line
-- exponent k (4). A value below 1 would violate every unit by fiat
-- (and a zero H or P_max would kill the curve).
--
-- Code 4 is the one ceiling that is an EXPONENT, not a line: it
-- reaches `m * spread ^ k` in CE.Verdict.Soft BEFORE the
-- [softMin, softMax] clamp, so an unbounded value never returns a
-- contract error — the process dies allocating the intermediate
-- Rational (review 2026-08-20 finding #3: a wire-supplied 1e9 asks
-- for a multi-gigabyte numerator). judgedLoc is validated under
-- 2^64, so k <= softKMax keeps that intermediate under 2^(64·1024),
-- kilobytes rather than gigabytes, while leaving the knob ~500x its
-- calibrated default of 2 — far past where any spread the clamp
-- cares about has already saturated it.
ceilingsOffence :: [[Integer]] -> Maybe String
ceilingsOffence = knobTable "ceilings" 4 "unknown ceiling axis" judgeC
 where
  judgeC code v
    | v < 1 = Just "ceiling below 1"
    | code == 4 && v > softKMax = Just ("soft-line exponent above " <> show softKMax)
    | otherwise = Nothing

-- | The allocation fence on ceiling code 4 — see ceilingsOffence.
softKMax :: Integer
softKMax = 1024

-- | Thresholds rows [knob, value], codes 0..6 (the wire-doc order).
-- A zero rewrite denominator (code 2) makes the churn
-- cross-multiplication vacuously true (the sim den==0 lesson); a
-- zero default weight (5) could zero the score divisor under an
-- empty weights table; a scale (6) below 1 leaves no score at all.
thresholdsOffence :: [[Integer]] -> Maybe String
thresholdsOffence = knobTable "thresholds" 6 "unknown threshold knob" judgeT
 where
  judgeT code v
    | v < 0 = Just "negative field"
    | code == 2 && v < 1 = Just "zero denominator"
    | code >= 5 && v < 1 = Just "knob below 1"
    | otherwise = Nothing

-- | Tolerance rows [leg, value]: 0 tolNum / 1 tolDen / 2 tolAbs.
-- Both ratio legs stay >= 1 (a zero denominator is vacuous; a num
-- below den just collapses that leg — tolerated already floors at
-- the ceiling through the max with the +abs leg).
toleranceOffence :: [[Integer]] -> Maybe String
toleranceOffence = knobTable "tolerance" 2 "unknown tolerance leg" judgeT
 where
  judgeT code v
    | v < 0 = Just "negative field"
    | code <= 1 && v < 1 = Just "knob below 1"
    | otherwise = Nothing

-- | ADR-008 P2: the dedup budget pair is two in-range counts or
-- nothing — a malformed pair must never read as "under budget".
dedupOffence :: Maybe [Integer] -> Maybe String
dedupOffence Nothing = Nothing
dedupOffence (Just row) = case row of
  [blocks, budget]
    | blocks < 0 || budget < 0 -> Just "dedup: negative field"
    | blocks >= u64 || budget >= u64 -> Just "dedup: outside u64"
    | otherwise -> Nothing
  _ -> Just "dedup: malformed pair (need [blocks,budget])"
 where
  u64 = 18446744073709551616

-- | batch-7 slice 1: the distinct rows are meaningless without the
-- pair they re-derive, the override floor is meaningless without
-- the rows it filters, and every count is a u64 — each mismatch
-- refuses by name, never a silent half-judgment.
dedupDistinctOffence :: Maybe [Integer] -> [Integer] -> Maybe Integer -> Maybe String
dedupDistinctOffence pair rows floor' =
  asum
    [ if null rows || pair /= Nothing
        then Nothing
        else Just "dedupDistinct without the dedup pair"
    , if floor' == Nothing || not (null rows)
        then Nothing
        else Just "dedupMinDistinct without dedupDistinct rows"
    , case floor' of
        Just f | f < 1 -> Just "dedupMinDistinct below 1"
        _ -> Nothing
    , asum
        [ if d >= 0 && d < 18446744073709551616
            then Nothing
            else Just (label "dedupDistinct" i <> "outside u64")
        | (i, d) <- zip [0 :: Int ..] rows
        ]
    ]
