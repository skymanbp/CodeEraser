-- | The structure family's entropy core (M6 S1, design booklet §2):
-- diversity and divergence in EXACT rational arithmetic. Shannon
-- entropy and KL divergence need logarithms — irrational, hence
-- undecidable in exact terms and banned along with every float by
-- the core's integer discipline — so the family's published
-- 熵/散度 are the rational-closed members of the SAME families:
-- Tsallis-2 entropy (= Gini–Simpson diversity, 1 − Σp², the
-- probability that two independent draws differ — Tsallis q=2) and
-- the χ² divergence (Σ (p−q)²/q, an f-divergence with the same
-- ordering semantics as KL at the tree scale). Both are standard
-- measures, both exactly decidable over Data.Ratio, and the
-- exhaustive reference battery (core/test/EntropyProps.hs) proves
-- the algebra against pair ENUMERATION — the ReferenceJaccard
-- posture: the definition counts, the shipped formula computes,
-- and the two must agree on every member of a bounded family.
module CE.Structure.Entropy
  ( tsallis2
  , tsallis2Norm
  , chi2
  , perMille
  ) where

import Data.Ratio (Ratio, (%))

-- | Tsallis-2 entropy of a count distribution: 1 − Σ (c/N)².
-- Empty and single-bin distributions carry zero diversity; zero
-- counts contribute nothing.
tsallis2 :: [Integer] -> Ratio Integer
tsallis2 counts
  | total == 0 = 0
  | otherwise = 1 - sum [(c % total) * (c % total) | c <- counts]
 where
  total = sum counts

-- | Normalized against the n-bin maximum (1 − 1/n, reached exactly
-- on the uniform distribution), n = nonzero bins — comparable
-- across sibling sets of different sizes. n ≤ 1 has no diversity
-- to normalize: 0 by definition.
tsallis2Norm :: [Integer] -> Ratio Integer
tsallis2Norm counts
  | n <= 1 = 0
  | otherwise = tsallis2 counts / (1 - 1 % n)
 where
  n = toInteger (length (filter (> 0) counts))

-- | χ² divergence of observed against reference counts, paired by
-- bin position: Σ (p − q)² / q over the reference's support.
-- Nothing = the observed places mass on a bin the reference gives
-- zero — the divergence is infinite there, and the S3
-- declared-template road reports those bins BY NAME instead of
-- collapsing them into a number. Both empty = 0 (nothing diverges
-- from nothing); observed empty against a live reference = exactly
-- 1 (Σ q over the support).
chi2 :: [(Integer, Integer)] -> Maybe (Ratio Integer)
chi2 pairs
  | oTotal == 0 && rTotal == 0 = Just 0
  | rTotal == 0 = Nothing
  | any (\(o, r) -> r == 0 && o > 0) pairs = Nothing
  | oTotal == 0 = Just 1
  | otherwise =
      Just (sum [sq (o % oTotal - r % rTotal) / (r % rTotal) | (o, r) <- pairs, r > 0])
 where
  oTotal = sum (map fst pairs)
  rTotal = sum (map snd pairs)
  sq x = x * x

-- | The publishing scale: exact rational → per-mille floor (the
-- deterministic truncation every published structure number rides).
perMille :: Ratio Integer -> Integer
perMille r = floor (r * 1000)
