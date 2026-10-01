-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | The similar family's second implementation (plan v2.32 step 7,
-- authority-track §7): booklet 15's same-role conjunction as a table
-- of ARMS — each arm a list of (column, floor) demands, a row plays
-- the role when every demand of some arm holds — and the judged order
-- as a RANK by pairwise comparison: a candidate's place is the number
-- of candidates that beat it, scores compared by cross-multiplying
-- the integer fractions (never a Rational, never a sort), a tie lost
-- by the later request index. It reads only CE.Similar.Cost's floors
-- and caps and answers the reply fields `similar.result` shows.
module ReferenceSimilar (expected, requests) where

import CE.Similar.Cost (roleMinCallee, roleMinName, roleMinNameShape, rowWidth, similarCap)
import Data.Aeson (Value (..), object, toJSON, (.=))
import ReferenceContract (breaks, firstOffender, labelled)
import ReferenceFlowGen (G, S (..), rand, runG)

-- | The two arms: called the same and calling the same, or two names
-- in common with the signature shape equal. Columns index the row
-- [nHit, pHit, cHit, dHit, sHit, lHit, shapeEqual, bm25Num, bm25Den].
arms :: [[(Int, Integer)]]
arms = [[(0, roleMinName), (2, roleMinCallee)], [(0, roleMinNameShape), (6, 1)]]

plays :: [Integer] -> Bool
plays row = any (all (\(col, least) -> drop col row `startsAtLeast` least)) arms
 where
  startsAtLeast (x : _) least = x >= least
  startsAtLeast [] _ = False

-- | Does candidate j stand ahead of candidate i?
beats :: (Int, [Integer]) -> (Int, [Integer]) -> Bool
beats (j, rj) (i, ri) = case (drop 7 rj, drop 7 ri) of
  ([nj, dj], [ni, di]) -> nj * di > ni * dj || (nj * di == ni * dj && j < i)
  _ -> False

-- | The order by rank: place p holds the candidate exactly p others beat.
order :: [[Integer]] -> [Int]
order rows = [i | p <- [0 .. length indexed - 1], (i, r) <- indexed, rank (i, r) == p]
 where
  indexed = zip [0 ..] rows
  rank c = length (filter (`beats` c) indexed)

-- | The contract as a list of (offender?, message) in request order:
-- the query table first (shape, then ascending hashes), then each row.
offence :: [[Integer]] -> [[Integer]] -> Maybe String
offence query rows =
  firstOffender
    ( labelled "query" termWhy query
        <> breaks "query" "not strictly ascending" (\a b -> take 1 a >= take 1 b) query
        <> labelled "row" rowWhy rows
    )
 where
  termWhy t = case t of
    [h, w]
      | h < 0 -> Just "negative term hash"
      | w < 1 -> Just "non-positive weight"
      | otherwise -> Nothing
    _ -> Just "malformed query term"
  rowWhy r
    | length r /= rowWidth = Just "malformed row"
    | any (< 0) (take 6 r) = Just "negative hit"
    | take 1 (drop 6 r) `notElem` [[0], [1]] = Just "shapeEqual not a boolean"
    | any (< 0) (take 1 (drop 7 r)) = Just "negative score"
    | any (< 1) (drop 8 r) = Just "non-positive denominator"
    | otherwise = Nothing

-- | The reply fields (order, roles, counts, degraded, reason), or Left
-- the refusal's message stem.
expected :: [[Integer]] -> [[Integer]] -> Either String [Maybe Value]
expected query rows
  | toInteger (length query + length rows) > similarCap = Right (answer ([] :: [Int]) [] True)
  | Just why <- offence query rows = Left why
  | otherwise = Right (answer (order rows) (map plays rows) False)
 where
  answer o roles deg =
    [ Just (toJSON o)
    , Just (toJSON roles)
    , Just (object ["rows" .= length rows, "queryTerms" .= length query, "role" .= length (filter id roles)])
    , Just (Bool deg)
    , if deg then Just (String "similar_too_large") else Nothing
    ]

-- | Two hundred seeded requests: a query bag of zero to five ascending
-- hashes, zero to twelve candidates with hits 0..3 and scores drawn
-- from a small fraction grid so equal scores under different
-- spellings (1/2, 2/4, 3/6) and zero numerators are common.
requests :: [([[Integer]], [[Integer]])]
requests = [runG request (S (n * 6007 + 29) 0 0 0) | n <- [1 .. 200 :: Int]]

request :: G ([[Integer]], [[Integer]])
request = do
  terms <- rand 6
  gaps <- mapM (const (rand 4)) [1 .. terms]
  weights <- mapM (const ((+ 1) <$> rand 3)) [1 .. terms]
  count <- rand 13
  rows <- mapM (const candidate) [1 .. count]
  pure ([[toInteger h, toInteger w] | (h, w) <- zip (scanl1 (+) (map (+ 1) gaps)) weights], rows)

candidate :: G [Integer]
candidate = do
  hits <- mapM (const (rand 4)) [1 .. 6 :: Int]
  shape <- rand 2
  den <- (+ 1) <$> rand 4
  num <- (* den) <$> rand 3
  half <- rand 3
  pure (map toInteger (hits <> [shape, if half == 0 then num `div` 2 else num, den]))
