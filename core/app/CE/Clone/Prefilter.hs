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
module CE.Clone.Prefilter (labelInter, provablyBelow) where

import CE.Clone.Cost (tsedDen, tsedNum)
import qualified Data.IntMap.Strict as IM

-- | I = Σ_label min(c1,c2) over two label lists (request-local dense
-- codes, so IntMap keys fit).
labelInter :: [Int] -> [Int] -> Integer
labelInter a b = sum (IM.elems (IM.intersectionWith min (histo a) (histo b)))
 where
  histo = IM.fromListWith (+) . map (\l -> (l, 1))

-- | True ⇔ the pair is provably below the clone threshold: the O(1)
-- size corollary first, then the O(#labels) intersection bound.
provablyBelow :: [Int] -> [Int] -> Bool
provablyBelow a b =
  below (fromIntegral (min n1 n2)) || below (labelInter a b)
 where
  (n1, n2) = (length a, length b)
  mx = fromIntegral (max n1 n2)
  below q = q * tsedDen < tsedNum * mx
