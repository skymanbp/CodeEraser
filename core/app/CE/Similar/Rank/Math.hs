-- | The advisor's integer arithmetic (plan v2.33 W3; booklet 15 §3–§4):
-- the fixed-point log2 by integer squaring, the idf, one term's BM25
-- contribution and PPMI — exact integers and rationals, no float, so the
-- same corpus ranks the same on every platform and the frozen eval docs
-- compare byte for byte with the Rust road they replace.
module CE.Similar.Rank.Math (log2Fp, idfFp, contribution, ppmiFp) where

import CE.Similar.Rank.Cost (b, idfFracBits, k1, minCooc, scoreFracBits)
import Data.Bits (shiftL, shiftR, (.|.))
import Data.Ratio (denominator, numerator)
import GHC.Num (integerLog2)

-- | floor (2^idfFracBits · log2 (num / den)) for num ≥ den > 0, by
-- integer squaring only: the integer part by doubling den, each fraction
-- bit by whether the squared ratio passes two. Operands are kept below
-- 2^62 by equal right shifts before each squaring — a truncation the
-- measuring side's u128 road took the same way, kept so every frozen
-- number stays the number it was.
log2Fp :: Integer -> Integer -> Integer
log2Fp num den = (intPart `shiftL` idfFracBits) .|. frac
 where
  (d0, intPart) = whole den 0
  whole d i
    | num >= 2 * d = whole (d `shiftL` 1) (i + 1)
    | otherwise = (d, i)
  frac = bits idfFracBits num d0 0
  bits :: Int -> Integer -> Integer -> Integer -> Integer
  bits 0 _ _ acc = acc
  bits left n d acc =
    let (n', d') = narrow n d
        (sn, sd) = (n' * n', d' * d')
     in if sn >= 2 * sd
          then bits (left - 1) sn (sd `shiftL` 1) (acc `shiftL` 1 .|. 1)
          else bits (left - 1) sn sd (acc `shiftL` 1)
  -- One shift by the width a one-bit-at-a-time loop would reach: that
  -- loop stops at the least k with n >> k < 2^62 and d >> k < 2^62,
  -- i.e. bitLen n - k <= 62 and bitLen d - k <= 62, and right shifts
  -- compose (x >> a >> b = x >> (a + b)). Both operands are positive
  -- here — num >= den > 0 on entry, then squares of positives with
  -- n >= d — so integerLog2 is defined.
  narrow n d = (n `shiftR` k, d `shiftR` k)
   where
    k = max 0 (max (bitLen n) (bitLen d) - 62)
  bitLen x = fromIntegral (integerLog2 x) + 1 :: Int

-- | idf (t) = log2 ((N − df + ½) / (df + ½)) in fixed point, floored at
-- zero: a term in at least half the units (2·df ≥ N) carries no
-- discrimination and no penalty.
idfFp :: Integer -> Integer -> Integer
idfFp n df
  | num <= den = 0
  | otherwise = log2Fp num den
 where
  num = 2 * n + 1 - 2 * df
  den = 2 * df + 1

-- | One term's contribution, floored to scoreFracBits fixed point:
-- w · idf · (k1 + 1) · tf / (tf + k1 · (1 − b + b · len / avg)), with
-- k1 = p/q and b = r/s cleared of denominators (`cleared`). Every
-- operand is non-negative and the divisor is at least `cAvg` · avg ≥ 1,
-- so `div` floors the rational exactly; the battery re-derives it from
-- k1 and b in Rational.
contribution :: Integer -> Integer -> Integer -> Integer -> Integer -> Integer
contribution w idf tf len avg = num `div` den
 where
  (cNum, cTf, cAvg, cLen) = cleared
  num = (w * idf * cNum * tf * avg) `shiftL` scoreFracBits
  den = cTf * tf * avg + cAvg * avg + cLen * len

-- | The closed form's four integers: (k1 + 1) · tf · avg over
-- tf · avg + k1 (1 − b) · avg + k1 b · len, every term times q s and
-- the four divided by their gcd — (22, 10, 3, 9) at k1 = 6/5, b = 3/4.
cleared :: (Integer, Integer, Integer, Integer)
cleared = (cn `div` g, ct `div` g, ca `div` g, cl `div` g)
 where
  (p, q, r, s) = (numerator k1, denominator k1, numerator b, denominator b)
  (cn, ct, ca, cl) = ((p + q) * s, q * s, p * (s - r), p * r)
  g = foldr1 gcd [cn, ct, ca, cl]

-- | PPMI (a, b) = max (0, log2 (n_ab · N / (n_a · n_b))) in the idf's
-- fixed point; zero below minCooc.
ppmiFp :: Integer -> Integer -> Integer -> Integer -> Integer
ppmiFp n nab na nb
  | nab < minCooc = 0
  | num <= den = 0
  | otherwise = log2Fp num den
 where
  num = nab * n
  den = na * nb
