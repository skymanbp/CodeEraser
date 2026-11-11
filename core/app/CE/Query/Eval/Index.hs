-- | A relation with lazily built lookup indexes (design booklet
-- §4.1, "每层建索引"): a join asks for the tuples that agree with
-- the argument positions already bound, and each distinct set of
-- bound positions — a bitmask over the arity — gets its own map the
-- first time it is asked for and never again. Only the masks a body
-- can ask for are in the table (`usedMasks`, read off the checked
-- program: which positions are bound at a literal is static under the
-- left-to-right join), and each one is a thunk, so a mask no join
-- reaches costs nothing; a mask outside the table — none, by the
-- static reading — is answered by a filter over the relation, never
-- by an empty answer (until v2.35 V7 the table held every mask of the
-- arity, 2^arity − 1 keys a relation a round, and an arity of 64 or
-- more shifted its range to nothing and answered empty).
module CE.Query.Eval.Index (Indexed, Masks, Rel, Tuple, indexed, lookupBound, usedMasks) where

import CE.Query.Syntax
import Data.Bits (shiftL, testBit)
import qualified Data.IntMap as IM
import qualified Data.IntSet as IS
import qualified Data.Map.Strict as M
import qualified Data.Set as S

type Tuple = [Integer]
type Rel = S.Set Tuple

-- | Predicate code → the bound-position masks its literals ask with.
type Masks = IM.IntMap IS.IntSet

-- | The relation and, per bound-position mask, its lookup map.
data Indexed = Indexed Rel (IM.IntMap (M.Map [Integer] [Tuple]))

-- | Index a relation on the masks given (each built on first use).
indexed :: IS.IntSet -> Rel -> Indexed
indexed masks rel = Indexed rel (IM.fromSet build masks)
 where
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
  | Just byKey <- IM.lookup mask idx = M.findWithDefault [] key byKey
  | otherwise = [t | t <- S.toList rel, project mask t == key]
 where
  mask = sum [1 `shiftL` i | (i, Just _) <- zip [0 :: Int ..] pat]
  key = [v | Just v <- pat]

-- | The masks the bodies ask with: walking each body in written order,
-- a positive atom asks with the positions bound before it and then
-- binds its variables, a negation asks and binds nothing, `V = e` and
-- an aggregate bind their variable, and an aggregate's body is walked
-- under the bindings outside it — the join's own order (CE.Query.Eval.Join).
usedMasks :: [[Lit]] -> Masks
usedMasks bodies = IM.fromListWith IS.union [(p, IS.singleton m) | b <- bodies, (p, m) <- walk IS.empty b, m /= 0]
 where
  walk _ [] = []
  walk bound (lit : rest) = case lit of
    LPos a -> asked bound a : walk (IS.union bound (varsOf a)) rest
    LNeg a _ -> asked bound a : walk bound rest
    LAssign v _ _ -> walk (IS.insert v bound) rest
    LAgg v _ _ inner _ -> walk bound inner <> walk (IS.insert v bound) rest
    LCmp {} -> walk bound rest
  asked bound (Atom p args _) = (p, sum [1 `shiftL` i | (i, Term _ v) <- zip [0 :: Int ..] args, fixed bound v])
  fixed bound v = case v of
    TVar x -> IS.member x bound
    TAnon -> False
    _ -> True
  varsOf (Atom _ args _) = IS.fromList [x | Term _ (TVar x) <- args]
