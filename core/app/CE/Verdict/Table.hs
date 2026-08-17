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

-- | Only the size (0) and coc (1) axes are configurable here, and a
-- ceiling below 1 would violate every unit by fiat.
ceilingsOffence :: [[Integer]] -> Maybe String
ceilingsOffence = knobTable "ceilings" 1 "unknown ceiling axis" low
 where
  low _ v = if v < 1 then Just "ceiling below 1" else Nothing

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
