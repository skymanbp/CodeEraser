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
  ) where

import Control.Applicative ((<|>))
import Data.Foldable (asum)

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
