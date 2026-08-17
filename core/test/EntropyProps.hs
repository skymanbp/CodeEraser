-- | The structure entropy core's reference battery (M6 S1): over an
-- EXHAUSTIVE family of count vectors, the shipped Tsallis-2 algebra
-- must equal the definitional pair ENUMERATION (two independent
-- draws, count the differing ordered pairs), the range/maximum/
-- transfer/permutation laws must hold, and χ² must vanish exactly
-- on identity and refuse unsupported mass — plus the S1
-- counterfactual: a perturbed distribution strictly moves both
-- measures (F16: nothing here may pass vacuously).
module EntropyProps (battery) where

import CE.Structure.Entropy (chi2, perMille, tsallis2, tsallis2Norm)
import Data.List (permutations, sort)
import Data.Ratio (Ratio, (%))
import WireHarness (runChecks)

battery :: IO Bool
battery = do
  ok <-
    runChecks
      [ ("tsallis2 ≡ pair-enumeration reference (whole family)", all algebraAgrees family)
      , ("range: 0 ≤ H2 ≤ 1 − 1/n, max exactly on uniform", all rangeHolds family && uniformMax)
      , ("transfer: evening one unit out strictly raises diversity", all transferHolds family)
      , ("permutation invariance over the family", all permInvariant family)
      , ("normalized H2 ∈ [0,1], exactly 1 on uniform multi-bin", normLaws)
      , ("χ²: zero on identity, positive elsewhere, refusals honest", chi2Laws)
      , ("counterfactual: the S1 levers move both measures", levers)
      ]
  putStrLn ("     entropy family: " <> show (length family) <> " count vectors")
  pure ok

-- | Every count vector over 3 bins with entries 0..4 — 125 members,
-- expansions ≤ 12 items (144 ordered pairs each): exhaustive and
-- cheap, the ReferenceJaccard scale.
family :: [[Integer]]
family = [[a, b, c] | a <- [0 .. 4], b <- [0 .. 4], c <- [0 .. 4]]

-- | Definitional reference: expand the counts to a draw list and
-- COUNT the ordered pairs that differ — no algebra anywhere.
refDiffer :: [Integer] -> Ratio Integer
refDiffer counts
  | total == 0 = 0
  | otherwise = toInteger differing % (total * total)
 where
  expanded = concat [replicate (fromInteger c) b | (b, c) <- zip [0 :: Int ..] counts]
  total = sum counts
  differing = length [() | x <- expanded, y <- expanded, x /= y]

algebraAgrees :: [Integer] -> Bool
algebraAgrees v = tsallis2 v == refDiffer v

rangeHolds :: [Integer] -> Bool
rangeHolds v
  | n == 0 = tsallis2 v == 0
  | otherwise = 0 <= tsallis2 v && tsallis2 v <= 1 - 1 % n
 where
  n = toInteger (length (filter (> 0) v))

uniformMax :: Bool
uniformMax = and [tsallis2 (replicate n c) == 1 - 1 % toInteger n | n <- [2, 3, 4], c <- [1, 3]]

-- | Moving one unit from a strictly larger bin to a strictly
-- smaller one must strictly increase diversity (the transfer
-- principle every honest diversity index obeys).
transferHolds :: [Integer] -> Bool
transferHolds v@[_, _, _] =
  and
    [ tsallis2 (moved i j) > tsallis2 v
    | (i, j) <- [(0, 1), (0, 2), (1, 0), (1, 2), (2, 0), (2, 1)]
    , v !! i >= (v !! j) + 2
    ]
 where
  moved i j = [adjust k x | (k, x) <- zip [0 :: Int ..] v]
   where
    adjust k x
      | k == i = x - 1
      | k == j = x + 1
      | otherwise = x
transferHolds _ = True

permInvariant :: [Integer] -> Bool
permInvariant v = all (\p -> tsallis2 p == tsallis2 v) (permutations v)

normLaws :: Bool
normLaws =
  all (\v -> 0 <= tsallis2Norm v && tsallis2Norm v <= 1) family
    && and [tsallis2Norm (replicate n 2) == 1 | n <- [2, 3, 4]]
    && tsallis2Norm [5] == 0
    && tsallis2Norm [] == 0

chi2Laws :: Bool
chi2Laws =
  all (\v -> sum v == 0 || chi2 (zip v v) == Just 0) family
    && and
      [ maybe False (> 0) (chi2 (zip o r))
      | o <- family
      , r <- family
      , all (> 0) r
      , sum o > 0
      , norm o /= norm r
      ]
    && chi2 [(1, 0), (0, 1)] == Nothing
    && chi2 [(0, 0)] == Just 0
    && chi2 [(0, 1), (0, 3)] == Just 1
 where
  norm v = [x % sum v | x <- v]

-- | The S1 counterfactual levers, hand-computed: concentrating
-- [2,2] into [4,0] collapses H2 from 1/2 to 0 (perMille 500 → 0),
-- and skewing an even split against an even reference raises χ²
-- from 0 to exactly 1/4.
levers :: Bool
levers =
  perMille (tsallis2 [2, 2]) == 500
    && perMille (tsallis2 [4, 0]) == 0
    && chi2 [(2, 2), (2, 2)] == Just 0
    && chi2 [(3, 2), (1, 2)] == Just (1 % 4)
    && sort [perMille (tsallis2 [1, 1, 1]), perMille (tsallis2 [3, 0, 0])] == [0, 666]
