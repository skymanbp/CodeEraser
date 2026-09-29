-- | The query language's parser (design booklet §4.1 / §4.4): a
-- recursive descent over the `[kind, value]` token stream the
-- measuring side lexed. The grammar lives here and nowhere else —
-- the lexer knows spellings, this knows shapes — and the first
-- shape it cannot read is the program's one syntax error, reported
-- at the token it stopped on (the token count when the program ended
-- early). Hand-written on purpose: the core's dependency set is the
-- freeze's five packages, and a parser combinator library is not one.
module CE.Query.Parse (parseProgram) where

import CE.Query.Cost
import CE.Query.Syntax
import Data.Array (Array, bounds, listArray, (!))

-- | Tokens as the request sent them, indexed for O(1) lookahead.
type Toks = Array Int (Integer, Integer)

-- | A parser: the token index in, the value and the next index out,
-- or the index of the token it could not read.
newtype P a = P {runP :: Toks -> Int -> Either Int (a, Int)}

instance Functor P where
  fmap f (P p) = P (\ts i -> fmap (\(a, j) -> (f a, j)) (p ts i))

instance Applicative P where
  pure a = P (\_ i -> Right (a, i))
  P pf <*> P pa = P $ \ts i -> case pf ts i of
    Left e -> Left e
    Right (f, j) -> fmap (\(a, k) -> (f a, k)) (pa ts j)

instance Monad P where
  P p >>= f = P $ \ts i -> case p ts i of
    Left e -> Left e
    Right (a, j) -> runP (f a) ts j

-- | Parse a whole program: clauses until the tokens run out.
parseProgram :: [(Integer, Integer)] -> Either Int [Clause]
parseProgram toks = fmap fst (runP clauses arr 0)
 where
  arr = listArray (0, length toks - 1) toks
  clauses = P go
  go ts i
    | i > snd (bounds ts) = Right ([], i)
    | otherwise = case runP clause ts i of
        Left e -> Left e
        Right (c, j) -> fmap (\(cs, k) -> (c : cs, k)) (go ts j)

-- | The token at the cursor, or Nothing past the end.
peek :: P (Maybe (Integer, Integer))
peek = P (\ts i -> Right (if i > snd (bounds ts) then Nothing else Just (ts ! i), i))

here :: P Int
here = P (\_ i -> Right (i, i))

failAt :: P a
failAt = P (\_ i -> Left i)

advance :: P ()
advance = P (\_ i -> Right ((), i + 1))

-- | Is the cursor on this token kind?
at :: Integer -> P Bool
at kind = fmap (maybe False ((== kind) . fst)) peek

-- | Consume one punctuation token by code, or stop there.
expect :: Integer -> P ()
expect kind = do
  hit <- at kind
  if hit then advance else failAt

clause :: P Clause
clause = do
  start <- here
  q <- at tokQuery
  a <- at tokAssert
  if q
    then advance >> body >>= \b -> expect tokDot >> pure (Query b start)
    else
      if a
        then advance >> assertion start
        else rule start

assertion :: Int -> P Clause
assertion start = do
  h <- atom
  expect tokRule
  b <- body
  expect tokDot
  pure (Assert h b start)

rule :: Int -> P Clause
rule start = do
  h <- atom
  hasBody <- at tokRule
  b <- if hasBody then advance >> body else pure []
  expect tokDot
  pure (Rule h b start)

body :: P [Lit]
body = do
  l <- lit
  more <- at tokComma
  if more then advance >> fmap (l :) body else pure [l]

-- | A literal starts with `not`, a variable, a predicate, or an
-- expression (a comparison whose left side is not a bare variable).
lit :: P Lit
lit = do
  start <- here
  t <- peek
  case t of
    Just (k, _) | k == tokNot -> advance >> fmap (`LNeg` start) atom
    Just (k, v) | k == kindVar -> advance >> assignOrCompare start (fromInteger v)
    Just (k, _) | k == kindPred -> fmap LPos atom
    _ -> fmap (compareFrom start) (cmpTail =<< expr)

-- | `Var …` (the variable already consumed): a positive atom never
-- starts with a variable, so this is `Var = agg`, `Var = expr` or a
-- comparison whose left side begins with the variable.
assignOrCompare :: Int -> Int -> P Lit
assignOrCompare start v = do
  eq <- at tokEq
  if eq
    then do
      advance
      agg <- aggHead
      case agg of
        Just op -> aggregate start v op
        Nothing -> fmap (\e -> LAssign v e start) expr
    else fmap (compareFrom start) (cmpTail =<< exprFrom (EVar v start))

-- | The aggregate keyword at the cursor, if any (the cursor stays).
aggHead :: P (Maybe Integer)
aggHead = fmap pick peek
 where
  pick t = case t of
    Just (k, _) | k `elem` [tokCount, tokMin, tokMax, tokSum] -> Just k
    _ -> Nothing

-- | `op(V1, V2… : body)` after `Var =`.
aggregate :: Int -> Int -> Integer -> P Lit
aggregate start v op = do
  advance
  expect tokOpen
  vars <- aggVars
  expect tokColon
  b <- body
  expect tokClose
  pure (LAgg v op vars b start)

aggVars :: P [(Int, Int)]
aggVars = do
  t <- peek
  i <- here
  case t of
    Just (k, x) | k == kindVar -> do
      advance
      more <- at tokComma
      rest <- if more then advance >> aggVars else pure []
      pure ((fromInteger x, i) : rest)
    _ -> failAt

-- | `expr <cmp> expr`: the operator and right side after a left side.
cmpTail :: Expr -> P (Integer, Expr, Expr)
cmpTail lhs = do
  t <- peek
  case t of
    Just (k, _) | k >= tokEq && k <= tokGe -> advance >> fmap (\r -> (k, lhs, r)) expr
    _ -> failAt

compareFrom :: Int -> (Integer, Expr, Expr) -> Lit
compareFrom start (op, l, r) = LCmp op l r start

atom :: P Atom
atom = do
  start <- here
  t <- peek
  case t of
    Just (k, code) | k == kindPred -> do
      advance
      expect tokOpen
      args <- terms
      expect tokClose
      pure (Atom (fromInteger code) args start)
    _ -> failAt

terms :: P [Term]
terms = do
  x <- term
  more <- at tokComma
  if more then advance >> fmap (x :) terms else pure [x]

term :: P Term
term = do
  i <- here
  t <- peek
  v <- case t of
    Just (k, x)
      | k == kindVar -> pure (TVar (fromInteger x))
      | k == kindAnon -> pure TAnon
      | k == kindInt -> pure (TInt x)
      | k == kindSym -> pure (TSym x)
      | k == kindSet -> pure (TSet x)
    _ -> failAt
  advance
  pure (Term i v)

-- | Additive over multiplicative over primary: the usual precedence,
-- left-associative.
expr :: P Expr
expr = primary >>= exprFrom

-- | The rest of an expression after its first primary.
exprFrom :: Expr -> P Expr
exprFrom p = mulRest p >>= addRest

mulRest :: Expr -> P Expr
mulRest = opsRest [tokMul, tokDiv, tokMod] primary

addRest :: Expr -> P Expr
addRest = opsRest [tokAdd, tokSub] (primary >>= mulRest)

-- | Left-associative operators of one level over an operand parser.
opsRest :: [Integer] -> P Expr -> Expr -> P Expr
opsRest ops operand lhs = do
  t <- peek
  case t of
    Just (k, _) | k `elem` ops -> advance >> operand >>= \r -> opsRest ops operand (EBin k lhs r)
    _ -> pure lhs

primary :: P Expr
primary = do
  i <- here
  t <- peek
  case t of
    Just (k, x)
      | k == kindVar -> advance >> pure (EVar (fromInteger x) i)
      | k == kindInt -> advance >> pure (EInt x)
      | k == kindSym -> advance >> pure (ESym x i)
      | k == tokOpen -> advance >> expr >>= \e -> expect tokClose >> pure e
    _ -> failAt
