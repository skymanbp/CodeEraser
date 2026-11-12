-- | The verdict family's second implementation of the soft line (plan
-- v2.32 step 7, authority-track §7): booklet 05 read as definitions.
-- Its two log-space middles are ORDER STATISTICS found by counting,
-- never by sorting, and its fourth root a bisection over the integers.
-- The density charges this module also restated retired with the
-- verdict's density law (10.0.0; the extent law's own reference is
-- ReferenceExtent), and the zone curve moved to ReferenceSplit with
-- the split advisory, its one reader.
module ReferenceScore (refSoftLine) where

-- | S = clamp(floor(m·r^k), [lo, hi]) over the positive LOC values in
-- log space (9.0.0): m the log-space median of the values and r the
-- log-space median of every value's ratio to m folded to at least one
-- — an even count's middle is the geometric mean of its two middles —
-- read in fourth powers, m⁴ = (x_a·x_b)² and r⁴ = ρ_a²·ρ_b², so S is
-- the largest integer whose fourth power stays at or under m⁴·r^{4k};
-- Nothing when no value is positive.
refSoftLine :: Integer -> Integer -> Integer -> [Integer] -> Maybe Integer
refSoftLine kExp lo hi locs = case [fromInteger x | x <- locs, x > 0] of
  [] -> Nothing
  vs ->
    let mSq = logMiddleSquared vs
        r4 = logMiddleSquared [max (v * v / mSq) (mSq / (v * v)) | v <- vs]
     in Just (max lo (min hi (fourthRootFloor (mSq * mSq * r4 ^ kExp))))

-- | The square of a log-space median: the product of the two middle
-- order statistics (the one middle squared when the count is odd).
logMiddleSquared :: [Rational] -> Rational
logMiddleSquared vs = rankAt ((n - 1) `div` 2) vs * rankAt (n `div` 2) vs
 where
  n = length vs

-- | The largest integer s >= 0 with s⁴ <= q, by doubling then halving
-- the bracket [s, s + step).
fourthRootFloor :: Rational -> Integer
fourthRootFloor q = narrow 0 (until (\b -> fromInteger (b ^ (4 :: Int)) > q) (* 2) 1)
 where
  narrow s step
    | step <= 1 = s
    | fromInteger ((s + half) ^ (4 :: Int)) <= q = narrow (s + half) (step - half)
    | otherwise = narrow s half
   where
    half = step `div` 2

-- | The order statistic of rank j: the value with at most j values
-- strictly below it and more than j at or below it.
rankAt :: Int -> [Rational] -> Rational
rankAt j xs = case [x | x <- xs, length (filter (< x) xs) <= j, length (filter (<= x) xs) > j] of
  (x : _) -> x
  [] -> 0
