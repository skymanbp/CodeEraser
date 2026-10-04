-- | The two provably admissible pre-TED bounds (design vol.2 §4.3),
-- judge-side half of the Rust/Haskell double application. Derivation
-- (NOT a citation — asserted against brute-force TED as an
-- exhaustive-family property in core/test/CloneProps.hs, R2):
--
--   For any Tai mapping M with r label-mismatched pairs,
--     cost = n1 + n2 − 2|M| + r.
--   Zero-cost pairs number at most I = Σ_label min(c1,c2), so
--     |M| − r ≤ I  ⇒  cost ≥ n1 + n2 − |M| − I ≥ max(n1,n2) − I
--   (using |M| ≤ min(n1,n2)). Hence ted ≥ max − I, and since
--   I ≤ min(n1,n2) also ted ≥ max − min = |n1 − n2|.
--
-- A pair failing `q·tsedDen ≥ tsedNum·max` for q = min(n1,n2) or
-- q = I therefore provably cannot reach the threshold whatever TED
-- computes — "below" is a judgment, never a guess. Since plan v2.33 W3
-- the candidate pass (CE.Candidates.T3) applies the same two bounds
-- through `boundOf`, before any tree is built; this module is the one
-- statement of both.
module CE.Clone.Prefilter (Bound (..), boundOf, histo, provablyBelow, provablyBelowH, reachFloor, sizeBelow) where

import CE.Clone.Cost (tsedDen, tsedNum)
import qualified Data.IntMap.Strict as IM

-- | Label histogram — a property of one TREE, not of a pair;
-- exported so decode can attach it to its tree (batch 9 P11).
histo :: [Int] -> IM.IntMap Integer
histo = IM.fromListWith (+) . map (\l -> (l, 1))

-- | I = Σ_label min(c1,c2) over two label lists.
interH :: IM.IntMap Integer -> IM.IntMap Integer -> Integer
interH x y = sum (IM.elems (IM.intersectionWith min x y))

-- | Which bound decides a pair: the size bound first (its tally owns
-- the pairs both bounds would cut), then the intersection bound, else
-- the pair is within reach of the threshold.
data Bound = SizeBound | LabelBound | Within
  deriving (Eq, Show)

-- | `q·tsedDen < tsedNum·max`: the bound quantity q cannot reach the
-- threshold against the larger operand's size.
sizeBelow :: (Integral a) => a -> a -> Bool
sizeBelow q mx = q * fromInteger tsedDen < fromInteger tsedNum * mx
{-# SPECIALIZE sizeBelow :: Int -> Int -> Bool #-}

-- | The least bound quantity that is not below against `mx`:
-- ceil(tsedNum·mx / tsedDen), so `sizeBelow q mx` ⇔ `q < reachFloor mx`
-- for non-negative q (CloneProps holds the two equal). A walk that only
-- needs to know whether an intersection reaches the threshold may stop
-- as soon as it provably cannot (CE.Candidates.Units.interUpTo).
reachFloor :: (Integral a) => a -> a
reachFloor mx = (fromInteger tsedNum * mx + fromInteger tsedDen - 1) `div` fromInteger tsedDen
{-# SPECIALIZE reachFloor :: Int -> Int #-}

-- | The decision over the two sizes and the label intersection I; I is
-- forced only when the size bound passes.
boundOf :: (Integral a) => a -> a -> a -> Bound
boundOf n1 n2 i
  | sizeBelow (min n1 n2) mx = SizeBound
  | sizeBelow i mx = LabelBound
  | otherwise = Within
 where
  mx = max n1 n2
{-# SPECIALIZE boundOf :: Int -> Int -> Int -> Bound #-}

-- | True ⇔ provably below the clone threshold — the O(1) size
-- corollary, then the intersection bound — from each operand's
-- (size, histogram), handed over instead of rebuilt per pair (P11).
provablyBelowH :: (Int, IM.IntMap Integer) -> (Int, IM.IntMap Integer) -> Bool
provablyBelowH (n1, h1) (n2, h2) =
  boundOf (toInteger n1) (toInteger n2) (interH h1 h2) /= Within

-- | The list face of the same predicate (the cloneDecidesWith
-- posture: one formula, two faces); CloneProps asserts through this.
provablyBelow :: [Int] -> [Int] -> Bool
provablyBelow a b = provablyBelowH (length a, histo a) (length b, histo b)
