-- | Exact Jaccard re-check (design vol.2 §5.3): the judgment core
-- receives strictly-ascending deduped shingle-hash sets and computes
-- |A∩B| and |A∪B| itself with Data.Set — NOT pre-computed ratios
-- from the estimator side. This is the whole point of the wire shape:
-- if Rust sent inter/union, "the re-check lives in Haskell" would be
-- an empty claim and ReferenceJaccard's exhaustive equivalence would
-- have nothing to hold. Values are Integer because the hashes are
-- full u64 (aeson routes them through Scientific's Integer
-- coefficient — FourClass.Wire precedent).
module CE.Docdup.Jaccard (interUnion) where

import qualified Data.Set as S

-- | (|A∩B|, |A∪B|) of two ascending deduped lists. The ascending
-- precondition of fromDistinctAscList is not assumed: the boundary
-- contract in CE.Docdup rejects any non-ascending set before this
-- module runs, so a violated precondition is unreachable rather than
-- silently corrupting.
interUnion :: [Integer] -> [Integer] -> (Integer, Integer)
interUnion a b =
  ( fromIntegral (S.size (S.intersection sa sb))
  , fromIntegral (S.size (S.union sa sb))
  )
 where
  sa = S.fromDistinctAscList a
  sb = S.fromDistinctAscList b
