-- | Sort inference for a parsed program (design booklet §4.2): every
-- fact argument has a declared sort, every program predicate's
-- position takes the sort of whatever flows into it, every clause
-- variable the sort of its first use — and any two that meet must
-- agree. A union-find over three kinds of node (a clause variable, a
-- program predicate's position, a constant sort) does the meeting;
-- the first disagreement is the program's sort error, at the token
-- that disagreed.
module CE.Query.Check.Sorts (Sorts, inferSorts, positionSort, variableSort) where

import CE.Query.Cost (errSort, sortInt, sortOpen, sortSet, sortSym, tokCount)
import CE.Query.Schema (sortsOf)
import CE.Query.Syntax
import qualified Data.Map.Strict as M

-- | A node of the union-find.
data Key = KVar Int Int | KPos Int Int | KSort Integer
  deriving (Eq, Ord, Show)

-- | parent links; a root absent from the map is its own root.
type Sorts = M.Map Key Key

find :: Sorts -> Key -> Key
find m k = maybe k (find m) (M.lookup k m)

-- | Union two nodes; two different constant sorts cannot meet.
union :: Key -> Key -> Sorts -> Either () Sorts
union a b m = case (find m a, find m b) of
  (ra, rb)
    | ra == rb -> Right m
    | KSort x <- ra, KSort y <- rb, x /= y -> Left ()
    | KSort _ <- ra -> Right (M.insert rb ra m)
    | otherwise -> Right (M.insert ra rb m)

-- | Unify under a token position: a failure is that token's error.
unifyAt :: Int -> Key -> Key -> Sorts -> Either (Int, Integer) Sorts
unifyAt at a b m = either (const (Left (at, errSort))) Right (union a b m)

-- | Every clause's atoms and literals, in program order.
inferSorts :: [Clause] -> Either (Int, Integer) Sorts
inferSorts = foldEither clauseSorts M.empty . zip [0 ..]

foldEither :: (a -> s -> Either e s) -> s -> [a] -> Either e s
foldEither step = go
 where
  go s [] = Right s
  go s (x : xs) = step x s >>= \s' -> go s' xs

clauseSorts :: (Int, Clause) -> Sorts -> Either (Int, Integer) Sorts
clauseSorts (c, cl) m0 = do
  m1 <- maybe (Right m0) (\h -> atomSorts c h m0) (clauseHead cl)
  foldEither (litSorts c) m1 (clauseBody cl)

-- | An atom's arguments each meet the sort of their position.
atomSorts :: Int -> Atom -> Sorts -> Either (Int, Integer) Sorts
atomSorts c (Atom p args _) m0 = foldEither one m0 (zip [0 ..] args)
 where
  one (i, Term at v) m = case termKey c v of
    Nothing -> Right m
    Just k -> unifyAt at k (posKey p i) m

-- | The union-find node a term stands for; anonymous stands for none.
termKey :: Int -> TermV -> Maybe Key
termKey c v = case v of
  TVar x -> Just (KVar c x)
  TAnon -> Nothing
  TInt _ -> Just (KSort sortInt)
  TSym _ -> Just (KSort sortSym)
  TSet _ -> Just (KSort sortSet)

-- | A fact predicate's position is its declared sort; a program
-- predicate's position is a node of its own.
posKey :: Int -> Int -> Key
posKey p i = case sortsOf p of
  Just sorts | i < length sorts -> KSort (sorts !! i)
  _ -> KPos p i

litSorts :: Int -> Lit -> Sorts -> Either (Int, Integer) Sorts
litSorts c lit m = case lit of
  LPos a -> atomSorts c a m
  LNeg a _ -> atomSorts c a m
  LCmp _ l r at -> exprSorts c l m >>= exprSorts c r >>= sameSort c at l r
  LAssign v e at -> exprSorts c e m >>= \m1 -> maybe (Right m1) (\k -> unifyAt at (KVar c v) k m1) (exprKey c e)
  LAgg v op vars body at -> do
    m1 <- foldEither (litSorts c) m body
    m2 <- unifyAt at (KVar c v) (KSort sortInt) m1
    case vars of
      (x, xat) : _ | op /= tokCount -> unifyAt xat (KVar c x) (KSort sortInt) m2
      _ -> Right m2

-- | A comparison's two sides are of one sort (two ids of a kind
-- order like their numbers, which is what breaks a symmetric pair);
-- arithmetic is integer-only, forced by `exprSorts`.
sameSort :: Int -> Int -> Expr -> Expr -> Sorts -> Either (Int, Integer) Sorts
sameSort c at l r m = case (exprKey c l, exprKey c r) of
  (Just a, Just b) -> unifyAt at a b m
  _ -> Right m

-- | A bare variable or literal has a node; a compound expression is
-- an integer and its operands were already forced to int.
exprKey :: Int -> Expr -> Maybe Key
exprKey c e = case e of
  EVar v _ -> Just (KVar c v)
  EInt _ -> Just (KSort sortInt)
  ESym _ _ -> Just (KSort sortSym)
  EBin {} -> Just (KSort sortInt)

-- | Every operand of an operator is an integer.
exprSorts :: Int -> Expr -> Sorts -> Either (Int, Integer) Sorts
exprSorts c e m = case e of
  EBin _ l r -> operand l m >>= operand r
  _ -> Right m
 where
  operand x m' = case x of
    EVar v at -> unifyAt at (KVar c v) (KSort sortInt) m'
    EInt _ -> Right m'
    ESym _ at -> unifyAt at (KSort sortSym) (KSort sortInt) m'
    EBin {} -> exprSorts c x m'

-- | The sort a program predicate's position resolved to, or open.
positionSort :: Sorts -> Int -> Int -> Integer
positionSort m p i = resolved m (posKey p i)

-- | The sort a clause's variable resolved to, or open.
variableSort :: Sorts -> Int -> Int -> Integer
variableSort m c v = resolved m (KVar c v)

resolved :: Sorts -> Key -> Integer
resolved m k = case find m k of
  KSort s -> s
  _ -> sortOpen
