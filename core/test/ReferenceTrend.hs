-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | The trend family's second implementation (plan v2.32 step 7,
-- authority-track §7): booklet 10's trajectory read off the
-- definitions. The judged view is a hand-written STABLE MERGE SORT of
-- the rows by timestamp, cut to the most recent window; the slope is
-- the Theil-Sen median taken by position from the merge-sorted list
-- of EVERY timestamp-distinct pair's slope; the cliff is a scan for
-- the deepest fall between neighbours; the decline run enumerates
-- every (start, end) window of strictly falling steps; the display
-- rounding is half-to-even spelled out. Only the family's cap and
-- window constants are read.
module ReferenceTrend (expected, roundEven) where

import CE.Trend.Cost (trendRowCap, tsWindow)
import Data.Aeson (Value (..), object, toJSON, (.=))
import Data.Ratio ((%))
import ReferenceContract (knobbedContract)

type Point = (Integer, Rational, Rational)

-- | Stable merge sort on a key: on equal keys the left half's element
-- (the earlier one) goes first.
mergeOn :: (Ord k) => (a -> k) -> [a] -> [a]
mergeOn key xs
  | length xs < 2 = xs
  | otherwise = merge (mergeOn key l) (mergeOn key r)
 where
  (l, r) = splitAt (length xs `div` 2) xs
  merge (a : as) (b : bs)
    | key b < key a = b : merge (a : as) bs
    | otherwise = a : merge as (b : bs)
  merge as [] = as
  merge [] bs = bs

-- | Rows as (request index, days, micro-units), timestamp order, the
-- most recent window kept.
view :: [[Integer]] -> [Point]
view rows = reverse (take tsWindow (reverse sorted))
 where
  sorted = mergeOn (\(_, x, _) -> x) [(i, ts % 86400, (1000000 * s) % scale) | (i, [ts, s, scale]) <- zip [0 ..] rows]

-- | Theil-Sen over the view: the middle of the sorted pair slopes, the
-- two middles averaged on an even count.
slope :: [Point] -> Maybe Rational
slope pts = case mergeOn id [(y2 - y1) / (x2 - x1) | (p, (_, x1, y1)) <- ip, (q, (_, x2, y2)) <- ip, p < q, x1 /= x2] of
  [] -> Nothing
  ss
    | odd (length ss) -> Just (ss !! half)
    | otherwise -> Just ((ss !! (half - 1) + ss !! half) / 2)
   where
    half = length ss `div` 2
 where
  ip = zip [0 :: Int ..] pts

-- | The deepest fall between neighbours, earliest on a tie.
cliff :: [Point] -> Maybe (Integer, Rational)
cliff pts = firstOf [f | f <- falls, snd f == maximum (map snd falls)]
 where
  falls = [(j, y1 - y2) | ((_, _, y1), (j, _, y2)) <- zip pts (drop 1 pts), y2 < y1]

-- | The longest run of strictly falling neighbours, by enumerating
-- every window; its first point's index and its point count, the
-- earliest window winning a tie.
declineRun :: [Point] -> Maybe (Integer, Integer)
declineRun pts = firstOf [r | r <- runs, snd r == maximum (map snd runs)]
 where
  ys = [y | (_, _, y) <- pts]
  idx = [i | (i, _, _) <- pts]
  n = length pts
  falling a b = and [ys !! k > ys !! (k + 1) | k <- [a .. b - 1]]
  runs = [(idx !! a, toInteger (b - a + 1)) | a <- [0 .. n - 1], b <- [a + 1 .. n - 1], falling a b]

firstOf :: [a] -> Maybe a
firstOf xs = case xs of
  (x : _) -> Just x
  [] -> Nothing

-- | Half-to-even, written out: the nearer integer, the even one on a
-- tie.
roundEven :: Rational -> Integer
roundEven x
  | frac < 1 % 2 = f
  | frac > 1 % 2 = f + 1
  | even f = f
  | otherwise = f + 1
 where
  f = floor x
  frac = x - fromInteger f

-- | The contract: each row, then each knob, then the knob codes
-- ascending.
offence :: [[Integer]] -> [[Integer]] -> Maybe String
offence = knobbedContract rowWhy knobWhy
 where
  rowWhy r = case r of
    [ts, s, scale]
      | ts < 0 -> Just "negative timestamp"
      | scale < 1 -> Just "non-positive scale"
      | s < 0 || s > scale -> Just "score outside 0..scale"
      | otherwise -> Nothing
    _ -> Just "malformed row"
  knobWhy k = case k of
    [c, v]
      | c `notElem` [0, 1] -> Just "unknown knob code"
      | v < 0 -> Just "negative knob value"
      | c == 0 && v < 2 -> Just "minPoints below 2"
      | otherwise -> Nothing
    _ -> Just "malformed knob"

-- | The reply fields (slopeMicroPerDay, verdict, cliff, declineRun,
-- fail, knobs, counts, degraded, reason), or Left the refusal's stem.
expected :: [[Integer]] -> [[Integer]] -> Either String [Maybe Value]
expected rows knobs
  | toInteger (length rows + length knobs) > trendRowCap = Right (degraded rows)
  | Just why <- offence rows knobs = Left why
  | otherwise = Right (judged rows (knob 0 3) (knob 1 0))
 where
  knob c d = last (d : [v | [c', v] <- knobs, c' == c])

judged :: [[Integer]] -> Integer -> Integer -> [Maybe Value]
judged rows minPoints floorMicro =
  [ Just (toJSON (roundEven <$> s))
  , Just (toJSON v)
  , Just (toJSON ((\(i, d) -> [i, roundEven d]) <$> (if enough then cliff pts else Nothing)))
  , Just (toJSON ((\(i, n) -> [i, n]) <$> (if enough then declineRun pts else Nothing)))
  , Just (Bool (v == Just 2 && floorMicro > 0))
  , Just (toJSON [[0, minPoints], [1, floorMicro]])
  , Just (object ["rows" .= length rows, "judged" .= length pts])
  , Just (Bool False)
  , Nothing
  ]
 where
  pts = view rows
  enough = toInteger (length rows) >= minPoints
  s = if enough then slope pts else Nothing
  band = fromInteger floorMicro
  v = (\x -> if x < negate band then 2 else if x > band then 0 else 1 :: Integer) <$> s

degraded :: [[Integer]] -> [Maybe Value]
degraded rows =
  [ Just Null
  , Just Null
  , Just Null
  , Just Null
  , Just (Bool True)
  , Just (toJSON [[0, 3], [1, 0 :: Integer]])
  , Just (object ["rows" .= length rows, "judged" .= (0 :: Int)])
  , Just (Bool True)
  , Just (String "trend_too_large")
  ]
