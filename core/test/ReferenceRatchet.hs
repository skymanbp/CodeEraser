-- | The verdict family's ratchet and join lattice, second
-- implementation (plan v2.32 step 7, authority-track §7). The ratchet
-- is booklet 05's ADR-006 law over LISTS — baseline lookups by key,
-- set differences as filters over a sorted nub — and the single-edit
-- allowance spelled as the ceiling plus the larger of its own two
-- percent (cleared by hand) and the absolute leg. The join is booklet
-- 07's eight conditions as named booleans with ratios compared as
-- Rationals, the verdict a guard chain in the booklet's priority
-- (merge, delete, hotspot) and the confidence a count over the three
-- legs. Only the entries' record types and the graph family's entry
-- mask are read.
module ReferenceRatchet (RatchetOut, refConfidence, refJudge, refRatchet, refTolerated) where

import CE.Graph.Cost (entryMask)
import CE.Verdict.Join (Knobs (..), Legs (..), Pos (..))
import CE.Verdict.Ratchet (RatchetKnobs (..))
import Data.Bits (testBit, (.&.))
import Data.List (nub, sort)
import Data.Maybe (isJust)
import Data.Ratio ((%))

-- | over, drawn, added, removed, dropped, new continuous, new discrete.
type RatchetOut = ([[Integer]], [[Integer]], [Integer], [Integer], [[Integer]], [[Integer]], [Integer])

-- | The ceiling plus its allowance: the class's own absolute
-- allowance where declared, else c + max(⌊2 % of c⌋, tolAbs) — the
-- +2 % leg written as the excess over c.
refTolerated :: RatchetKnobs -> Maybe Integer -> Integer -> Integer
refTolerated k declared c = case declared of
  Just t -> c + t
  Nothing -> c + max ((c * (rTolNum k - rTolDen k)) `div` rTolDen k) (rTolAbs k)

-- | The ratchet over a baseline (Nothing = establish), the class
-- allowance rows [class, code, value] (code 4 answers metric 1 where
-- declared, code 3 otherwise), the present set, and the current
-- continuous and discrete facts.
refRatchet :: RatchetKnobs -> [[Integer]] -> Maybe [Integer] -> Maybe ([[Integer]], [Integer]) -> ([[Integer]], [Integer]) -> RatchetOut
refRatchet _ _ _ Nothing (cont, disc) = ([], [], [], [], [], map (take 3) cont, disc)
refRatchet k classRows present (Just (base, baseDisc)) (cont, disc) =
  ( [[u, c, v, a] | (u, c, v, Just _, a) <- judged, v > a]
  , [[u, c, v - b] | (u, c, v, Just b, a) <- judged, b < v, v <= a]
  , setMinus disc baseDisc
  , setMinus baseDisc disc
  , [[u, c, v] | Just ps <- [present], [u, c, v] <- base, u `elem` ps, (u, c) `notElem` keys]
  , [[u, c, maybe v (min v) b] | (u, c, v, b, _) <- judged]
  , disc
  )
 where
  keys = [(u, c) | (u : c : _) <- cont]
  judged =
    [ (u, c, v, b, maybe 0 (refTolerated k (allowance cls c)) b)
    | (u : c : v : rest) <- cont
    , let cls = sum (take 1 rest)
    , let b = lookup (u, c) [((bu, bc), bv) | [bu, bc, bv] <- reverse base]
    ]
  allowance cls c = case [v | [cl, code, v] <- classRows, cl == cls, code == 4, c == 1] of
    (v : _) -> Just v
    [] -> case [v | [cl, 3, v] <- classRows, cl == cls] of
      (v : _) -> Just v
      [] -> Nothing
  setMinus xs ys = sort (nub [x | x <- xs, x `notElem` ys])

-- | The eight held conditions as (bit, holds).
conditions :: Knobs -> Legs -> [(Int, Bool)]
conditions k l =
  [ (1, simOver)
  , (2, isJust graph)
  , (3, maybe False (\(a, b) -> pIndeg a > 0 && pIndeg b > 0) graph)
  , (4, maybe False (\(a, b) -> pScc a /= pScc b) graph)
  , (5, any (uncurry dead) flanks)
  , (6, any (\(x, y) -> dead x y && testBit (pFlags x) 0) flanks)
  , (7, any (>= kCochangeFloor k) (lCochange l))
  , (8, total > 0 && rewrites % total >= kRewriteNum k % kRewriteDen k)
  ]
 where
  (kind, num, den) = lSim l
  simOver
    | kind == 2 = num % den >= kDupNum k % kDupDen k
    | otherwise = num % den >= kCloneNum k % kCloneDen k
  graph = (,) <$> lGraphA l <*> lGraphB l
  flanks = maybe [] (\(a, b) -> [(a, b), (b, a)]) graph
  dead x y = pIndeg x == 0 && pReach x == 0 && pFlags x .&. entryMask == 0 && pIndeg y > 0
  (apA, rwA) = lChurnA l
  (apB, rwB) = lChurnB l
  total = apA + rwA + apB + rwB
  rewrites = rwA + rwB

-- | (verdict, legsMask, reasonBits): the first verdict in priority
-- order whose conditions hold.
refJudge :: Knobs -> Legs -> (Integer, Integer, Integer)
refJudge k l = (code, 1 + (if held 2 then 2 else 0) + 4, sum [2 ^ b | (b, True) <- conds])
 where
  conds = conditions k l
  held b = lookup b conds == Just True
  code
    | all held [1, 2, 3, 4] = 1
    | all held [1, 2, 5] && not (held 6) = 2
    | all held [1, 2, 7, 8] = 3
    | otherwise = 0

-- | Of the legs present, how many hold at least one condition: the
-- similarity leg (bit 1), the graph leg (bits 2..6, present when both
-- positions are), the churn leg (bits 7..8).
refConfidence :: Knobs -> Legs -> Integer
refConfidence k l = toInteger (length (filter id [held 1, isJust graph && any held [2 .. 6], any held [7, 8]]))
 where
  conds = conditions k l
  held b = lookup b conds == Just True
  graph = (,) <$> lGraphA l <*> lGraphB l
