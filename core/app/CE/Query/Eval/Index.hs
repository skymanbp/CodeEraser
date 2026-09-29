-- | A relation with lazily built lookup indexes (design booklet
-- §4.1, "每层建索引"): a join asks for the tuples that agree with
-- the argument positions already bound, and each distinct set of
-- bound positions — a bitmask over the arity — gets its own map the
-- first time it is asked for and never again. Laziness is the memo:
-- the index table is a spine of thunks, so a mask no join ever uses
-- costs nothing.
module CE.Query.Eval.Index (Indexed, Rel, Tuple, allTuples, indexed, lookupBound, size) where

import Data.Bits (shiftL, testBit)
import qualified Data.IntMap as IM
import qualified Data.Map.Strict as M
import qualified Data.Set as S

type Tuple = [Integer]
type Rel = S.Set Tuple

-- | The relation and, per bound-position mask, its lookup map.
data Indexed = Indexed Rel (IM.IntMap (M.Map [Integer] [Tuple]))

-- | Index a relation; the arity is read off its first tuple (an
-- empty relation has nothing to index and answers nothing).
indexed :: Rel -> Indexed
indexed rel = Indexed rel (IM.fromList [(m, build m) | m <- [1 .. (1 `shiftL` arity) - 1]])
 where
  arity = maybe 0 length (S.lookupMin rel)
  build m = M.map reverse (M.fromListWith (<>) [(project m t, [t]) | t <- S.toList rel])

-- | The values at the positions the mask marks, in position order.
project :: Int -> Tuple -> [Integer]
project m t = [v | (i, v) <- zip [0 ..] t, testBit m i]

-- | The tuples agreeing with a pattern of bound positions (Just) —
-- the whole relation when nothing is bound. The tuples come back in
-- the relation's own order, so a join is deterministic.
lookupBound :: Indexed -> [Maybe Integer] -> [Tuple]
lookupBound (Indexed rel idx) pat
  | mask == 0 = S.toList rel
  | otherwise = M.findWithDefault [] key (IM.findWithDefault M.empty mask idx)
 where
  mask = sum [1 `shiftL` i | (i, Just _) <- zip [0 :: Int ..] pat]
  key = [v | Just v <- pat]

allTuples :: Indexed -> Rel
allTuples (Indexed rel _) = rel

size :: Indexed -> Int
size = S.size . allTuples
