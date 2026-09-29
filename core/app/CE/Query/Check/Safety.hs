-- | Range restriction for a parsed program (design booklet §4.1,
-- "安全性"): a variable is read only after a positive atom or a
-- binding literal earlier in the same body bound it, so every
-- negation, comparison, arithmetic operand and head argument names
-- a value the evaluator will actually hold. The pass walks each
-- body in order with the set of bound variables, and records the
-- order the outer level bound them in — the projection a query
-- answers with.
module CE.Query.Check.Safety (outerVars, safetyErrors) where

import CE.Query.Cost (errAggregate, errHead, errUnsafe)
import CE.Query.Syntax
import qualified Data.IntSet as IS

-- | Every safety error of a clause, in body order (empty = safe).
safetyErrors :: Clause -> [(Int, Integer)]
safetyErrors cl = bodyErrs <> headErrs
 where
  (bound, bodyErrs) = walk IS.empty (clauseBody cl)
  headErrs = maybe [] (headCheck bound) (clauseHead cl)

-- | Variables the outer body binds, in first-binding order.
outerVars :: [Lit] -> [Int]
outerVars = reverse . snd . foldl step (IS.empty, [])
 where
  step (bound, acc) lit = case binds bound lit of
    new -> (IS.union bound (IS.fromList new), reverse new <> acc)

-- | The variables a literal binds beyond `bound`, in term order.
binds :: IS.IntSet -> Lit -> [Int]
binds bound lit = case lit of
  LPos (Atom _ args _) -> dedupe [v | Term _ (TVar v) <- args, not (IS.member v bound)]
  LAssign v _ _ | not (IS.member v bound) -> [v]
  LAgg v _ _ _ _ | not (IS.member v bound) -> [v]
  _ -> []

dedupe :: [Int] -> [Int]
dedupe = go IS.empty
 where
  go _ [] = []
  go seen (x : xs)
    | IS.member x seen = go seen xs
    | otherwise = x : go (IS.insert x seen) xs

-- | Walk a body: the bound set after it and the errors it raised.
walk :: IS.IntSet -> [Lit] -> (IS.IntSet, [(Int, Integer)])
walk bound [] = (bound, [])
walk bound (lit : rest) = (bound'', errs <> errs')
 where
  errs = litErrors bound lit
  bound' = IS.union bound (IS.fromList (binds bound lit))
  (bound'', errs') = walk bound' rest

-- | A literal's own reads against what is bound before it.
litErrors :: IS.IntSet -> Lit -> [(Int, Integer)]
litErrors bound lit = case lit of
  LPos _ -> []
  LNeg (Atom _ args _) _ -> [(at, errUnsafe) | Term at (TVar v) <- args, unbound v]
  LCmp _ l r _ -> exprErrors bound l <> exprErrors bound r
  LAssign _ e _ -> exprErrors bound e
  LAgg v _ vars body at ->
    [(at, errAggregate) | IS.member v bound]
      <> snd (walk bound body)
      <> [(vat, errAggregate) | (x, vat) <- vars, not (IS.member x inner) || IS.member x bound]
   where
    inner = fst (walk bound body)
 where
  unbound v = not (IS.member v bound)

-- | Variables of an expression with their tokens.
exprVars :: Expr -> [(Int, Int)]
exprVars e = case e of
  EVar v at -> [(v, at)]
  EBin _ l r -> exprVars l <> exprVars r
  _ -> []

exprErrors :: IS.IntSet -> Expr -> [(Int, Integer)]
exprErrors bound e = [(at, errUnsafe) | (v, at) <- exprVars e, not (IS.member v bound)]

-- | A head argument is a bound variable or a literal; an anonymous
-- head argument names no column.
headCheck :: IS.IntSet -> Atom -> [(Int, Integer)]
headCheck bound (Atom _ args _) = concatMap one args
 where
  one (Term at v) = case v of
    TVar x | not (IS.member x bound) -> [(at, errUnsafe)]
    TAnon -> [(at, errHead)]
    _ -> []
