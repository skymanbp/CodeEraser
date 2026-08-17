-- | Exhaustive small-instance equivalence for the docdup judgment
-- (Reference.hs pattern): an independently written reference — the
-- definitional membership counts over an explicit universe — must
-- agree with the shipped Data.Set implementation on EVERY pair of a
-- bounded family. The wire ships SETS precisely so this argument
-- exists (design vol.2 §5.3: pre-computed inter/union would leave
-- the enumeration nothing to prove); the family covers all subsets
-- of an 8-element universe, 256×256 ordered pairs, plus full-u64
-- magnitude probes for the Integer path.
module ReferenceJaccard (equivalence) where

import CE.Docdup.Cost
  ( dupDecides
  , dupDecidesWith
  , dupVerdict
  , dupVerdictWith
  , jaccardDen
  , jaccardNum
  )
import CE.Docdup.Jaccard (interUnion)
import Data.Ratio ((%))

universe :: [Integer]
universe = [0 .. 7]

-- | All subsets of the universe, each already ascending (filtering a
-- sorted list preserves order) — the boundary contract's set shape
-- by construction.
family :: [[Integer]]
family = filterings universe
 where
  filterings [] = [[]]
  filterings (x : xs) = [pick <> rest | rest <- filterings xs, pick <- [[], [x]]]

-- | Definitional reference: count universe members by list
-- membership, no Data.Set anywhere.
refInterUnion :: [Integer] -> [Integer] -> (Integer, Integer)
refInterUnion a b =
  ( count (\x -> x `elem` a && x `elem` b)
  , count (\x -> x `elem` a || x `elem` b)
  )
 where
  count p = toInteger (length (filter p universe))

equivalence :: IO Bool
equivalence = do
  let pairs = [(a, b) | a <- family, b <- family]
  a <-
    check
      ("docdup reference equivalence on " <> show (length pairs) <> " exhaustive pairs")
      (all (\(x, y) -> interUnion x y == refInterUnion x y) pairs)
  b <-
    check
      "u64-magnitude sets survive the Integer path"
      ( interUnion [5, 2 ^ (64 :: Int) - 2, 2 ^ (64 :: Int) - 1] [2 ^ (64 :: Int) - 1]
          == (1, 3)
      )
  c <-
    check
      "integer cross-multiplication == rational comparison on every judged pair"
      ( and
          [ dupDecides i u == ((i % u) >= (jaccardNum % jaccardDen))
          | (x, y) <- pairs
          , let (i, u) = refInterUnion x y
          , u > 0 -- empty∪empty is unreachable: the boundary rejects empty sets
          ]
      )
  d <-
    check
      "threshold sits exactly on 80/100 in both directions"
      (dupDecides 4 5 && not (dupDecides 3 4) && dupDecides 8 10)
  e <- deadKnob
  -- the ADR-008 P1 counterfactual on the verdict's NEW half: the
  -- SHIPPED disjunction sits exactly on the 50-word floor in both
  -- directions, a perturbed floor flips the boundary run, and the
  -- Jaccard half still decides alone — the bit the wire now carries
  -- is this formula's output
  f <-
    check
      "verbatim half: run 50/49 both directions, floor 51 flips it, jaccard alone decides"
      ( dupVerdict 0 100 50
          && not (dupVerdict 0 100 49)
          && not (dupVerdictWith (jaccardNum, jaccardDen, 51) 0 100 50)
          && dupVerdict 4 5 0
      )
  pure (a && b && c && d && e && f)

-- | Dead-knob probe (CloneProps posture): perturbing the ratio must
-- move the family's decision count, and the preconditions are
-- asserted non-vacuous — a count that is zero or total would make
-- the perturbation check pass while proving nothing.
deadKnob :: IO Bool
deadKnob = do
  let ius =
        [ refInterUnion a b
        | a <- family
        , b <- family
        , not (null a)
        , not (null b)
        ]
      decided n d = length [() | (i, u) <- ius, dupDecidesWith n d i u]
      at80 = decided jaccardNum jaccardDen
      at90 = decided 90 100
  a <-
    check
      "decision count is non-vacuous at the shipped ratio"
      (at80 > 0 && at80 < length ius)
  b <- check "perturbing the ratio moves the decision count" (at80 /= at90)
  pure (a && b)

check :: String -> Bool -> IO Bool
check name ok = do
  putStrLn ((if ok then "ok   " else "FAIL ") <> name)
  pure ok
