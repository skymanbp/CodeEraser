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
-- computes — "below" is a judgment, never a guess.
module CE.Clone.Prefilter (histo, labelInter, provablyBelow, provablyBelowH) where

import CE.Clone.Cost (tsedDen, tsedNum)
import qualified Data.IntMap.Strict as IM

-- | Label histogram — a property of one TREE, not of a pair;
-- exported so decode can attach it to its tree (batch 9 P11).
histo :: [Int] -> IM.IntMap Integer
histo = IM.fromListWith (+) . map (\l -> (l, 1))

-- | I = Σ_label min(c1,c2) over two label lists.
labelInter :: [Int] -> [Int] -> Integer
labelInter a b = interH (histo a) (histo b)

interH :: IM.IntMap Integer -> IM.IntMap Integer -> Integer
interH x y = sum (IM.elems (IM.intersectionWith min x y))

-- | True ⇔ provably below the clone threshold — the O(1) size
-- corollary, then the intersection bound — from each operand's
-- (size, histogram), handed over instead of rebuilt per pair (P11).
provablyBelowH :: (Int, IM.IntMap Integer) -> (Int, IM.IntMap Integer) -> Bool
provablyBelowH (n1, h1) (n2, h2) =
  below (fromIntegral (min n1 n2)) || below (interH h1 h2)
 where
  mx = fromIntegral (max n1 n2)
  below q = q * tsedDen < tsedNum * mx

-- | The list face of the same predicate (the cloneDecidesWith
-- posture: one formula, two faces); CloneProps asserts through this.
provablyBelow :: [Int] -> [Int] -> Bool
provablyBelow a b = provablyBelowH (length a, histo a) (length b, histo b)
