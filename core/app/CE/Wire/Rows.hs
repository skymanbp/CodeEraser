-- | The row contract's vocabulary, one owner for every family (split
-- from CE.Wire at the 290-line wall, plan v2.33 step 11a): the
-- "<table> <i>: " label every row refusal starts with, the u64 range,
-- the row / table / element / knob checks the families compose. The
-- refusal TEXTS stay each family's: these take them as arguments, so
-- a fold never rewords a contract. CE.Wire re-exports all of it.
module CE.Wire.Rows (label, u64, outsideU64, required, rowCheck, tableOffence, ascendingOn, elements, knobTable, topCode) where

import Data.Foldable (asum)

-- | The label a row refusal starts with: "<table> <i>: ".
label :: String -> Int -> String
label what i = what <> " " <> show i <> ": "

-- | The refusal of a request that leaves out a key the 9.0.0 major made
-- required (plan v2.33 step 11a: the absent-key roads went with it),
-- one text per key: "<key>: required at 9.0.0".
required :: String -> Maybe String
required key = Just (key <> ": required at 9.0.0")

-- | Inside the u64 range: 0 <= x < 2^64.
u64 :: Integer -> Bool
u64 x = x >= 0 && x < 2 ^ (64 :: Int)

-- | The one out-of-range class most element tables refuse with.
outsideU64 :: [(Integer -> Bool, String)]
outsideU64 = [(not . u64, "element outside u64")]

-- | One row's whole contract: the right width, then the checks a
-- well-formed row must pass, both wearing the same label. The
-- malformed message is the CALLER's whole string, not a template:
-- families say "malformed row (need [..])" and "malformed knob (need
-- [..])", and the wording is each family's contract, not this
-- skeleton's to normalize.
rowCheck ::
  String -> String -> Int -> ([Integer] -> Maybe String) -> Int -> [Integer] -> Maybe String
rowCheck what malformed width checks i row
  | length row /= width = Just (label what i <> malformed)
  | otherwise = fmap (label what i <>) (checks row)

-- | One table's whole contract: every row well shaped in request
-- order, then the table strictly ascending. Order is deliberate:
-- shape errors surface before ordering errors, so the ascending pass
-- only ever compares rows it can compare.
tableOffence :: (Ord b) => String -> (a -> b) -> (Int -> a -> Maybe String) -> [a] -> Maybe String
tableOffence what proj row rows =
  asum [asum (zipWith row [0 :: Int ..] rows), ascendingOn what proj rows]

-- | Whole-table strictly-ascending pass on a PROJECTION of each row —
-- the identity prefix for docdup pairs (take 2), the code for scan
-- grades (take 1), the whole row for clone pairs and graph edges (id);
-- list Ord is lexicographic. The offender is named by the later row.
ascendingOn :: (Ord b) => String -> (a -> b) -> [a] -> Maybe String
ascendingOn what proj rows =
  asum
    [ Just (label what i <> "not strictly ascending")
    | (i, (p, c)) <- zip [1 :: Int ..] (zip rows (drop 1 rows))
    , proj p >= proj c
    ]

-- | A non-empty run of elements — a set or a sequence: "empty <noun>",
-- then each out-of-range class (a predicate over the whole run and its
-- refusal, in order), then, when the table is a set, strictly
-- ascending. A sequence keeps its order and its repeats.
elements :: String -> String -> [(Integer -> Bool, String)] -> Bool -> Int -> [Integer] -> Maybe String
elements what noun bad ascending i xs
  | null xs = refuse ("empty " <> noun)
  | (why : _) <- [w | (p, w) <- bad, any p xs] = refuse why
  | ascending && or (zipWith (>=) xs (drop 1 xs)) = refuse "not strictly ascending"
  | otherwise = Nothing
 where
  refuse why = Just (label what i <> why)

-- | A [code, value] knob table: each row two wide (else the family's
-- own malformed text), the code inside 0..top (else the family's
-- unknown-code text), then the family's judgment of the value; the
-- codes strictly ascending. `top` is the last code of the family's
-- knob table, read off that table, never a second literal.
knobTable :: String -> String -> Integer -> String -> (Integer -> Integer -> Maybe String) -> [[Integer]] -> Maybe String
knobTable what malformed top codeWhy judgeV =
  tableOffence what (take 1) (rowCheck what malformed 2 one)
 where
  one row = case row of
    [code, v]
      | code < 0 || code > top -> Just codeWhy
      | otherwise -> judgeV code v
    _ -> Nothing

-- | The last code of a code-indexed table: a code bound read off the
-- table it indexes, so a row added to the table moves the bound.
topCode :: [a] -> Integer
topCode xs = toInteger (length xs) - 1
