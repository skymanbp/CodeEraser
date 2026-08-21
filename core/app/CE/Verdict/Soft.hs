-- | The plan-v2.6 soft zone (§A) and relative soft line (§B), pure
-- and log-free. The contract writes S = clamp(median + k·MAD, [lo,
-- hi]) in log-LOC space; log is monotone, so median(log x) =
-- log(median x) and MAD(log x) = median |log x − log m| =
-- log(median max(x/m, m/x)) — exponentiating gives S = m·r^k
-- EXACTLY, as order statistics over 'Rational'. No logarithm is
-- ever taken (the Entropy.hs discipline: Shannon/KL fell to
-- Tsallis-2/χ² for the same reason).
module CE.Verdict.Soft
  ( softLine
  , zonePenalty
  ) where

import Data.List (sort)
import Data.Ratio ((%))

-- | Exact median of a non-empty sorted list: middle element, or the
-- mean of the two middles (even length) — stays Rational either way.
medianR :: [Rational] -> Rational
medianR xs
  | odd n = xs !! mid
  | otherwise = (xs !! (mid - 1) + xs !! mid) / 2
 where
  n = length xs
  mid = n `div` 2

-- | S = clamp(floor(m·r^k), [lo, hi]) over the judged-language LOC
-- multiset; Nothing when no positive LOC exists (an empty or
-- all-empty-file set has no distribution to speak of — absence,
-- never a fabricated line). floor is the conservative side: a lower
-- S opens the graded zone earlier, and ties stay closed (the
-- ratchet's "ties don't open" stance).
softLine :: Integer -> Integer -> Integer -> [Integer] -> Maybe Integer
softLine kExp lo hi locs = case sort [x | x <- locs, x > 0] of
  [] -> Nothing
  xs -> Just (min hi (max lo (floor (m * spread ^ kExp))))
   where
    sorted = map (% 1) xs
    m = medianR sorted
    -- r: the multiplicative MAD — median of each point's ratio to
    -- the median, folded to >= 1 (|log| = log of the larger ratio)
    spread = medianR (sort [max (x / m) (m / x) | x <- sorted])

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
-- the deny AT the wall is the guard's job. Degenerate h <= s falls
-- back to the pre-v0.6 binary (x > s costs pMax flat): a
-- misdeclared hard line must not divide by zero or flip the sign.
zonePenalty :: Integer -> Integer -> Integer -> Integer -> Rational
zonePenalty s h pMax x
  | x <= s = 0
  | h <= s = pMax % 1
  | x <= h = (pMax % 1) * step * step
  | otherwise = (pMax % 1) * (1 + 2 * ((x - h) % (h - s)))
 where
  step = (x - s) % (h - s)
