-- | The ranking constants of the same-role advisor (plan v2.33 W3;
-- booklet 15 §3–§4): integer BM25 with k1 = 6/5 and b = 3/4, idf in
-- 8-bit fixed point from an integer log2, every per-term contribution
-- floored to 16-bit fixed point, query weights in 1/256, and the PPMI
-- widening's floors and caps. Until W3 they lived in
-- cli/src/similar/bm25.rs and ppmi.rs; the measuring side now reads
-- them from the definition package (CE.Limits) — the frozen eval docs
-- echo them — and computes nothing with them but which rows to fetch.
module CE.Similar.Rank.Cost (
  k1,
  b,
  idfFracBits,
  scoreFracBits,
  wUnit,
  chanWeight,
  wordChannel,
  shapeChannel,
  channels,
  topM,
  minCooc,
  minPpmi,
  ppmiCap,
  ppmiScale,
  scoredDfRatio,
  neighbourDfRatio,
  rankCap,
) where

import Data.Ratio ((%))

-- | Okapi k1 and b (Robertson & Walker 1994's usual settings).
k1, b :: Rational
k1 = 6 % 5
b = 3 % 4

-- | Fraction bits of the idf and the PPMI (the same log2) and of a
-- contribution.
idfFracBits, scoreFracBits :: Int
idfFracBits = 8
scoreFracBits = 16

-- | Query weights ride in 1/wUnit, so a PPMI-scaled expansion keeps a
-- fraction of its parent's weight without a float.
wUnit :: Integer
wUnit = 256

-- | The six evidence channels in wire order [N, P, C, D, S, L].
channels :: Integer
channels = 6

-- | Query weight multiplier per channel: names ×3, callees ×2,
-- everything else ×1.
chanWeight :: Integer -> Integer
chanWeight ch = case ch of
  0 -> 3
  2 -> 2
  _ -> 1

-- | The channels that carry WORDS (name, callee, doc) — the ones a
-- query is widened through; the shape channel is P.
wordChannel :: Integer -> Bool
wordChannel ch = ch `elem` [0, 2, 3]

shapeChannel :: Integer
shapeChannel = 1

-- | Neighbours appended per spelled word term; a neighbour counts only
-- when it co-occurred in at least minCooc units, and only at or above
-- two bits of PPMI.
topM :: Int
topM = 3

minCooc :: Integer
minCooc = 2

minPpmi, ppmiCap, ppmiScale :: Integer
minPpmi = 2 * 2 ^ idfFracBits
ppmiCap = 4 * 2 ^ idfFracBits
ppmiScale = 8 * 2 ^ idfFracBits

-- | What the two zero rules reduce to, stated for the measuring side's
-- fetch: a term scores only when n > scoredDfRatio · df (idf > 0), and
-- a word has a neighbour only when neighbourDfRatio · n_a ≤ n (n_ab ≤
-- n_b bounds PPMI(a, b) by log2 (n / n_a), under minPpmi's two bits as
-- soon as 4 · n_a > n). Derived here, never restated: the battery pins
-- both against the formulas they shortcut.
scoredDfRatio, neighbourDfRatio :: Integer
scoredDfRatio = 2
neighbourDfRatio = 2 ^ (minPpmi `div` 2 ^ idfFracBits)

-- | Table ceiling: every row of every table together. A self query's
-- postings run to tens of thousands of rows.
rankCap :: Integer
rankCap = 4194304
