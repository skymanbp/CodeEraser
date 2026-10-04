-- | The advisor's integer arithmetic (plan v2.33 W3; booklet 15 §3–§4):
-- the fixed-point log2 by integer squaring, the idf, one term's BM25
-- contribution and PPMI — exact integers and rationals, no float, so the
-- same corpus ranks the same on every platform and the frozen eval docs
-- compare byte for byte with the Rust road they replace.
module CE.Similar.Rank.Math (log2Fp, idfFp, contribution, ppmiFp) where

import CE.Similar.Rank.Cost (b, idfFracBits, k1, minCooc, scoreFracBits)
import Data.Bits (shiftL, shiftR, (.|.))

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
  narrow n d
    | n >= limit || d >= limit = narrow (n `shiftR` 1) (d `shiftR` 1)
    | otherwise = (n, d)
  limit = 2 ^ (62 :: Int)

-- | idf (t) = log2 ((N − df + ½) / (df + ½)) in fixed point, floored at
-- zero: a term in more than half the units carries no discrimination
-- and no penalty.
idfFp :: Integer -> Integer -> Integer
idfFp n df
  | num <= den = 0
  | otherwise = log2Fp num den
 where
  num = 2 * n + 1 - 2 * df
  den = 2 * df + 1

-- | One term's contribution, floored to scoreFracBits fixed point:
-- w · idf · (k1 + 1) · tf / (tf + k1 · (1 − b + b · len / avg)), the
-- exact rational floored once.
contribution :: Integer -> Integer -> Integer -> Integer -> Integer -> Integer
contribution w idf tf len avg = floor (top / bottom)
 where
  top = fromInteger (w * idf * tf * 2 ^ scoreFracBits) * (k1 + 1)
  bottom = fromInteger tf + k1 * (1 - b + b * fromInteger len / fromInteger avg)

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
