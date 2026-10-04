-- | The docdup coarse filter over integer facts (design vol.2 §5.3;
-- plan v2.33 W3): MinHash/LSH banding over the segments' shingle sets
-- (CE.Candidates.Lsh, the one shape and implementation the T3 source S4
-- reads) ∪ seed pairs — every two segments that share a shingle — with
-- hot groups chained, never skipped (CE.Candidates.Groups). The LSH
-- pairs come first; a seed pair counts only when LSH did not already
-- find it. Until W3 the measuring side ran this filter; the pairs it
-- keeps go to the docdup judgment (CE.Docdup).
module CE.Docdup.Coarse (Coarse (..), coarse) where

import CE.Candidates.Cost (lshShape)
import CE.Candidates.Groups (Paired (..), paired)
import CE.Candidates.Lsh (Sets (..), bandBuckets)
import CE.Dedup.Cost (hotCap)
import Data.Array.Base (unsafeAt)
import qualified Data.IntMap.Strict as IM
import qualified Data.IntSet as IS

-- | The kept pairs (key a·n + b, a < b) and the tally: the LSH pairs,
-- the seed pairs LSH did not find, and the chained groups of each.
data Coarse = Coarse
  { found :: !IS.IntSet
  , lshPairs :: !Int
  , seedPairs :: !Int
  , hotBands :: !Int
  , hotShingles :: !Int
  }

coarse :: Sets -> Coarse
coarse ss = Coarse withSeeds (IS.size byBands) (IS.size withSeeds - IS.size byBands) hb hs
 where
  n = setCount ss
  shape = let (p, b, r) = lshShape in (fromInteger p, fromInteger b, fromInteger r)
  (byBands, hb) = gather IS.empty (bandBuckets shape (const True) ss)
  (withSeeds, hs) = gather byBands sharing
  sharing = filter ((> 1) . length) (IM.elems inverted)
  inverted =
    IM.fromListWith
      (++)
      [ (fromIntegral (elems ss `unsafeAt` e), [s])
      | s <- [n - 1, n - 2 .. 0]
      , e <- [offsets ss `unsafeAt` s .. offsets ss `unsafeAt` (s + 1) - 1]
      ]
  gather start groups = foldl' add (start, 0) (map (paired (fromInteger hotCap)) groups)
  add (acc, hot) g =
    ( foldl' (\keys (a, b) -> if a == b then keys else IS.insert (min a b * n + max a b) keys) acc (pairsOf g)
    , if chained g then hot + 1 else hot :: Int
    )
