-- | One body against one database state (design booklet §4.1): the
-- literals in written order, each narrowing the substitutions the
-- ones before it produced. A positive atom joins through the index
-- of whichever relation the caller routes that position to (the
-- semi-naive delta or the full relation), a negation tests absence,
-- a comparison filters, a binding extends, an aggregate folds an
-- inner body's answers under the outer substitution. Every answer
-- carries the positive atoms it read, instantiated — the provenance
-- a proof tree is built from.
module CE.Query.Eval.Join (Fact, Look, Subst, evalExpr, instantiate, joinBody, pattern) where

import CE.Query.Cost (tokAdd, tokCount, tokDiv, tokEq, tokGe, tokGt, tokLe, tokLt, tokMax, tokMin, tokMod, tokMul, tokNe, tokSub)
import CE.Query.Eval.Index (Indexed, Tuple, lookupBound)
import CE.Query.Syntax
import qualified Data.IntMap.Strict as IM
import qualified Data.Set as S

type Subst = IM.IntMap Integer

-- | A positive atom instantiated: its predicate and tuple.
type Fact = (Int, Tuple)

-- | Where a body position's atom reads from (the caller's routing) —
-- the position of the literal in the body and its predicate.
type Look = Int -> Int -> Indexed

-- | The answers of a body: each substitution with the facts it read.
joinBody :: Look -> (Int -> Indexed) -> [Lit] -> Subst -> [(Subst, [Fact])]
joinBody look full body s0 = go 0 body [(s0, [])]
 where
  go _ [] acc = map (\(s, fs) -> (s, reverse fs)) acc
  go i (lit : rest) acc = go (i + 1) rest (concatMap (step i lit) acc)
  step i lit (s, fs) = case lit of
    LPos a -> [(s', (atomPred a, t) : fs) | (s', t) <- matches (look i (atomPred a)) a s]
    LNeg a _ -> [(s, fs) | null (matches (full (atomPred a)) a s)]
    LCmp op l r _ -> [(s, fs) | Just x <- [evalExpr s l], Just y <- [evalExpr s r], compareBy op x y]
    LAssign v e _ -> [(s', fs) | Just x <- [evalExpr s e], Just s' <- [bindVar s v x]]
    LAgg v op vars inner _ -> [(s', fs) | Just x <- [aggregate full op vars inner s], Just s' <- [bindVar s v x]]

-- | The tuples an atom matches under a substitution, each with the
-- substitution it extends to.
matches :: Indexed -> Atom -> Subst -> [(Subst, Tuple)]
matches rel (Atom _ args _) s = [(s', t) | t <- lookupBound rel (pattern s args), Just s' <- [unify s args t]]

-- | An atom's bound positions under a substitution.
pattern :: Subst -> [Term] -> [Maybe Integer]
pattern s = map (valueOf s . termOf)

valueOf :: Subst -> TermV -> Maybe Integer
valueOf s v = case v of
  TVar x -> IM.lookup x s
  TAnon -> Nothing
  TInt n -> Just n
  TSym n -> Just n
  TSet n -> Just n

-- | Bind the atom's unbound variables to a tuple; a variable that
-- appears twice must agree with itself, a constant with the tuple.
unify :: Subst -> [Term] -> Tuple -> Maybe Subst
unify s0 args t = go s0 (zip args t)
 where
  go s [] = Just s
  go s ((Term _ v, x) : rest) = case v of
    TVar y -> bindVar s y x >>= \s' -> go s' rest
    TAnon -> go s rest
    _ -> if valueOf s v == Just x then go s rest else Nothing

-- | Bind, or check an existing binding.
bindVar :: Subst -> Int -> Integer -> Maybe Subst
bindVar s v x = case IM.lookup v s of
  Nothing -> Just (IM.insert v x s)
  Just y -> if x == y then Just s else Nothing

-- | Integer arithmetic; a division by zero has no value, so the
-- literal it sits in simply fails.
evalExpr :: Subst -> Expr -> Maybe Integer
evalExpr s e = case e of
  EVar v _ -> IM.lookup v s
  EInt n -> Just n
  ESym n _ -> Just n
  EBin op l r -> do
    x <- evalExpr s l
    y <- evalExpr s r
    arith op x y

arith :: Integer -> Integer -> Integer -> Maybe Integer
arith op x y
  | op == tokAdd = Just (x + y)
  | op == tokSub = Just (x - y)
  | op == tokMul = Just (x * y)
  | op == tokDiv = if y == 0 then Nothing else Just (x `quot` y)
  | op == tokMod = if y == 0 then Nothing else Just (x `rem` y)
  | otherwise = Nothing

compareBy :: Integer -> Integer -> Integer -> Bool
compareBy op x y
  | op == tokEq = x == y
  | op == tokNe = x /= y
  | op == tokLt = x < y
  | op == tokLe = x <= y
  | op == tokGt = x > y
  | op == tokGe = x >= y
  | otherwise = False

-- | An aggregate under the outer substitution: the inner body's
-- answers projected onto the aggregated variables, as a set, folded.
-- count and sum of nothing are 0; min and max of nothing have no
-- value, so the literal fails.
aggregate :: (Int -> Indexed) -> Integer -> [(Int, Int)] -> [Lit] -> Subst -> Maybe Integer
aggregate full op vars inner s = fold (S.toList rows)
 where
  rows = S.fromList [[IM.findWithDefault 0 v s' | (v, _) <- vars] | (s', _) <- joinBody (const full) full inner s]
  firsts = map head' rows'
  rows' = S.toList rows
  head' r = case r of x : _ -> x; [] -> 0
  fold xs
    | op == tokCount = Just (toInteger (length xs))
    | op == tokMin = if null xs then Nothing else Just (minimum firsts)
    | op == tokMax = if null xs then Nothing else Just (maximum firsts)
    | otherwise = Just (sum firsts)

-- | A head under a substitution: every argument is bound or literal
-- by the safety pass, so a tuple always comes out.
instantiate :: Subst -> [Term] -> Tuple
instantiate s = map (maybe 0 id . valueOf s . termOf)
