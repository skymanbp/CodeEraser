-- | The plan-v2.6 soft zone (§A) and relative soft line (§B), pure
-- and log-free. The contract writes S = clamp(median + k·MAD, [lo,
-- hi]) in log-LOC space; log is monotone, so the median of log x is
-- the log of the middle LOC — and of the GEOMETRIC middle √(x_p·x_q)
-- when the count is even, since the median of an even log sample is
-- the mean of its two middle logs — and MAD(log x) = median of
-- |log x − log m| = the log of the (again geometric) middle of
-- ρ = max(x/m, m/x). Exponentiating, S = m·r^k with both middles
-- geometric. No logarithm is ever taken (the Entropy.hs discipline:
-- Shannon/KL fell to Tsallis-2/χ² for the same reason) and no square
-- root of a rational either: S is computed in FOURTH powers, where
-- both geometric middles are plain products — m² = x_p·x_q and
-- r⁴ = ρ_p²·ρ_q² (an odd count takes its one middle twice) — and
-- S = ⌊(m⁴·r^{4k})^{1/4}⌋ is an exact integer fourth root. The identity
-- holds for every count; before 9.0.0 an even count took the
-- ARITHMETIC mean of the two middles (for m, and again for r around
-- that m), which missed the definition in either direction.
module CE.Verdict.Soft
  ( median
  , softLine
  , zonePenalty
  , classedZone
  ) where

import Data.Bits (shiftL)
import Data.List (sort)
import qualified Data.Map.Strict as M
import Data.Ratio (denominator, numerator, (%))
import GHC.Num (integerLog2)

-- | Exact order-statistic median of a non-empty list: the middle
-- element of its sort, or the mean of the two middles (even length) —
-- Rational either way, deterministic by sort. A LINEAR-space median:
-- trend/2's Theil-Sen slope (CE.Trend.Cost) is its one reader. The
-- soft line's middles are log-space and live in `middles` below.
median :: [Rational] -> Rational
median xs
  | odd n = sorted !! mid
  | otherwise = (sorted !! (mid - 1) + sorted !! mid) / 2
 where
  sorted = sort xs
  n = length sorted
  mid = n `div` 2

-- | The two middle order statistics of a non-empty list (the one
-- middle twice when the count is odd): their PRODUCT is the square of
-- the log-space median, which is how the soft line reads a middle
-- without ever taking a root.
middles :: Ord a => [a] -> (a, a)
middles xs = (sorted !! ((n - 1) `div` 2), sorted !! (n `div` 2))
 where
  sorted = sort xs
  n = length sorted

-- | S = clamp(⌊m·r^k⌋, [lo, hi]) over the judged-language LOC
-- multiset; Nothing when no positive LOC exists (an empty or
-- all-empty-file set has no distribution to speak of — absence,
-- never a fabricated line). floor is the conservative side: a lower
-- S opens the graded zone earlier, and ties stay closed (the
-- ratchet's "ties don't open" stance). The caller passes hi = min
-- softMax H (9.0.0), so a derived S never sits above the hard line.
--
-- Cost: the fourth powers square what `m * spread ^ k` used to raise.
-- With k bounded by the ceilings table's softKMax (1024) and every
-- LOC under 2^64, the rational m⁴·r^{4k} has a numerator and a
-- denominator of at most ~4× the bits the old intermediate reached —
-- tens of kilobytes at the fence, not gigabytes — and the root is
-- two integer square roots of its floor (⌊q^{1/4}⌋ = ⌊⌊q⌋^{1/4}⌋ for
-- the integer answer s, since s⁴ ≤ q ⇔ s⁴ ≤ ⌊q⌋).
softLine :: Integer -> Integer -> Integer -> [Integer] -> Maybe Integer
softLine kExp lo hi locs = case [x | x <- locs, x > 0] of
  [] -> Nothing
  xs -> Just (min hi (max lo (iroot4 (numerator q4 `div` denominator q4))))
   where
    squares = [x * x | x <- xs]
    m2 = uncurry (*) (middles xs)
    -- ρ²: each square's ratio to m², folded to >= 1 (|log ρ| = the log
    -- of the larger ratio); ρ² orders exactly as ρ does
    rho2 = [max (sq % m2) (m2 % sq) | sq <- squares]
    r4 = uncurry (*) (middles rho2)
    q4 = (m2 * m2 % 1) * r4 ^ kExp

-- | ⌊n^{1/4}⌋ for n >= 0: ⌊√⌊√n⌋⌋ (s² ≤ √n ⇔ s² ≤ ⌊√n⌋ for an integer s).
iroot4 :: Integer -> Integer
iroot4 = isqrt . isqrt

-- | ⌊√n⌋ for n >= 0 by Newton's method from a start above the root
-- (2^(⌊log₂ n⌋ `div` 2 + 1) > √n), where every step decreases until
-- the floor is reached; the first non-decreasing step ends it.
isqrt :: Integer -> Integer
isqrt n
  | n < 2 = n
  | otherwise = go (1 `shiftL` (fromIntegral (integerLog2 n `div` 2) + 1))
 where
  go x = let y = (x + n `div` x) `div` 2 in if y >= x then x else go y

-- | p(x) for one file: 0 at or under the soft line; the convex
-- curve pMax·((x−s)/(h−s))² on the zone (s,h]; past the hard line
-- the C¹ LINEAR continuation pMax·(1 + 2(x−h)/(h−s)) — the slope at
-- h⁻ is 2·pMax/(h−s) and the linear arm keeps it exactly, so the
-- curve is monotone with no kink and never stops charging (a score
-- that stopped at the wall would reward growth), but no longer
-- grows quadratically outside the contracted domain: the M9 batch-6
-- field test measured two ordinary real repositories at 0/1000
-- because the extrapolated square summed past ten times the whole
-- scale (cc-memory axis 0 = 10176‰, one 4802-line file charging
-- like 95 hard-line files). size-advisory.md §A prices only (S,H];
-- the deny AT the wall is the guard's job.
--
-- The curve opens at s' = min s h (9.0.0): the relative line widens
-- the comfort zone, never past the wall. The establish side already
-- derives S under H (softLine's hi), but a baseline committed before
-- 9.0.0 may carry S above H, and a class's own hard line may sit
-- below the global S — either way a file past H used to charge 0
-- here. Degenerate h <= s' (only h == s' now) falls back to the
-- pre-v0.6 binary (x > s' costs pMax flat): a misdeclared hard line
-- must not divide by zero or flip the sign.
zonePenalty :: Integer -> Integer -> Integer -> Integer -> Rational
zonePenalty s0 h pMax x
  | x <= s = 0
  | h <= s = pMax % 1
  | x <= h = (pMax % 1) * step * step
  | otherwise = (pMax % 1) * (1 + 2 * ((x - h) % (h - s)))
 where
  s = min s0 h
  step = (x - s) % (h - s)

-- | zonePenalty for a file of class `cls` (0 = none): the class's own
-- opening edge (class code 0) and hard line (class code 2) where its
-- rows declare them, the global (s, h) otherwise; P_max has no class
-- channel. The ONE resolution the verdict's size axis (Score.sizeMass)
-- and the split advisory (Structure.Split, 9.0.0) price with, so the
-- two families read one curve per file.
classedZone :: M.Map (Integer, Integer) Integer -> Integer -> (Integer, Integer, Integer) -> Integer -> Rational
classedZone knobs cls (s, h, pMax) = zonePenalty (line 0 s) (line 2 h) pMax
 where
  line code g = M.findWithDefault g (cls, code) knobs
