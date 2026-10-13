-- | An independently written reference for the query family's
-- evaluator (design booklet §4.1 / §4.7): its own strata — the
-- predicate graph's components read off a transitive closure (no
-- Data.Graph), a negative or aggregated edge inside one is the
-- unstratifiable program, a component's level the most negative edges
-- on a path into it — then a naive fixpoint per stratum, every rule
-- over the WHOLE database every round, no indexes, no delta routing,
-- no provenance, the join a plain nested product over the body in
-- list-comprehension form; its own arithmetic, comparisons and
-- aggregates (count and sum of nothing are 0, min and max of nothing
-- have no value), its own goal answers (a query's variables in the
-- order the body first binds them, an assertion's head). Only the
-- parser is shared. The shipped checker and evaluator must agree on
-- every program of the seeded generator (ReferenceQueryGen) over its
-- fact set, refusal for refusal and tuple for tuple
-- (QueryReferenceProps). The same reference replays a proof tree node
-- by node: each rule node's children must be one solution of its body.
module ReferenceQuery (Db, events, goalAnswersRef, naive, replays, strataCount, toks) where

import CE.Query.Cost
import CE.Query.Syntax
import Control.Monad (foldM, guard, join)
import Data.Char (isDigit)
import qualified Data.IntMap.Strict as IM
import Data.List (nub, sort)
import qualified Data.Map as M
import Data.Maybe (fromMaybe, mapMaybe, maybeToList)
import qualified Data.Set as S

type Db = IM.IntMap (S.Set [Integer])
type Subst = IM.IntMap Integer

-- | The predicates a literal reads and whether positively: a
-- negation's atom, and every atom an aggregate's body reads at any
-- depth, are negative edges (§4.1 分层).
readsOf :: Lit -> [(Bool, Int)]
readsOf lit = case lit of
  LPos a -> [(True, atomPred a)]
  LNeg a _ -> [(False, atomPred a)]
  LAgg _ _ _ body _ -> [(False, p) | l <- body, (_, p) <- readsOf l]
  _ -> []

-- | The rules by stratum, lowest first; Nothing = a negative edge
-- inside a component (unstratifiable).
strata :: [Clause] -> Maybe [[(Atom, [Lit])]]
strata clauses
  | or [not pos && together q h | (q, h, pos) <- edges] = Nothing
  | otherwise = Just [[r | r@(h, _) <- rules, level (atomPred h) == l] | l <- nub (sort (map (level . atomPred . fst) rules))]
 where
  rules = [(h, b) | Rule h b _ <- clauses]
  edges = [(q, atomPred h, pos) | (h, b) <- rules, l <- b, (pos, q) <- readsOf l, isIdb q]
  -- the closure as each predicate's reach set, walked depth first
  succs = M.fromListWith (<>) [(q, [h]) | (q, h, _) <- edges]
  next p = M.findWithDefault [] p succs
  walk seen [] = seen
  walk seen (x : xs)
    | S.member x seen = walk seen xs
    | otherwise = walk (S.insert x seen) (next x <> xs)
  reach p = walk S.empty (next p)
  together a b = S.member b (reach a) && S.member a (reach b)
  -- a component by its least member
  rep p = minimum (p : [q | (q, _, _) <- edges, together p q])
  memo = M.fromList [(c, compLevel c) | c <- nub [rep (atomPred h) | (h, _) <- rules]]
  compLevel c = maximum ((0 :: Int) : [M.findWithDefault 0 (rep q) memo + (if pos then 0 else 1) | (q, h, pos) <- edges, rep h == c, rep q /= c])
  level p = M.findWithDefault 0 (rep p) memo

-- | How many strata the program stands in, or Nothing.
strataCount :: [Clause] -> Maybe Int
strataCount = fmap length . strata

-- | The naive evaluator: its own strata in order, each to a fixpoint
-- of "apply every rule to the whole database"; Nothing when the
-- program cannot be stratified.
naive :: [Clause] -> Db -> Maybe Db
naive clauses db0 = foldl (flip fix) db0 <$> strata clauses
 where
  fix rules db = let db' = foldl apply db rules in if db' == db then db else fix rules db'
  apply db (h, b) = IM.insertWith S.union (atomPred h) (S.fromList [map (valueIn s) (atomArgs h) | s <- solve db b IM.empty]) db

-- | Every substitution a body admits, one literal at a time.
solve :: Db -> [Lit] -> Subst -> [Subst]
solve _ [] s = [s]
solve db (lit : rest) s = concatMap (solve db rest) (extend db lit s)

extend :: Db -> Lit -> Subst -> [Subst]
extend db lit s = case lit of
  LPos a -> matches a
  LNeg a _ -> [s | null (matches a)]
  LCmp op l r _ -> [s | Just True <- [cmp op <$> ev s l <*> ev s r]]
  LAssign v e _ -> assigned v (ev s e)
  LAgg v op vars body _ -> assigned v (agg db op vars body s)
 where
  -- every tuple of the relation the atom's arguments admit
  matches a = mapMaybe (bind s (atomArgs a)) (S.toList (rel db (atomPred a)))
  -- a value bound to the variable, or nothing when there is no value
  assigned v value = maybeToList (value >>= bindOne s v)

rel :: Db -> Int -> S.Set [Integer]
rel db p = IM.findWithDefault S.empty p db

-- | The atom's arguments against a tuple, position by position: a
-- fold that threads the substitution through Maybe.
bind :: Subst -> [Term] -> [Integer] -> Maybe Subst
bind s args xs = guard (length args == length xs) >> foldM step s (zip (map termOf args) xs)
 where
  step acc (TVar n, x) = bindOne acc n x
  step acc (TAnon, _) = Just acc
  step acc (v, x) = acc <$ guard (valueIn acc (Term 0 v) == x)

bindOne :: Subst -> Int -> Integer -> Maybe Subst
bindOne s v x = maybe (Just (IM.insert v x s)) (\y -> s <$ guard (x == y)) (IM.lookup v s)

valueIn :: Subst -> Term -> Integer
valueIn s (Term _ v) = case v of
  TVar n -> IM.findWithDefault 0 n s
  TAnon -> 0
  TInt n -> n
  TSym n -> n
  TSet n -> n

ev :: Subst -> Expr -> Maybe Integer
ev s expr = case expr of
  EVar v _ -> IM.lookup v s
  EInt n -> pure n
  ESym n _ -> pure n
  EBin op l r -> join (arithOp op <$> ev s l <*> ev s r)

-- | The operators as a table; the two divisions have no value at zero.
arithOp :: Integer -> Integer -> Integer -> Maybe Integer
arithOp op x y = lookup op table >>= \f -> f x y
 where
  table = [(tokAdd, total (+)), (tokSub, total (-)), (tokMul, total (*)), (tokDiv, safe quot), (tokMod, safe rem)]
  total, safe :: (Integer -> Integer -> Integer) -> Integer -> Integer -> Maybe Integer
  total f a b = Just (f a b)
  safe f a b = if b == 0 then Nothing else Just (f a b)

cmp :: Integer -> Integer -> Integer -> Bool
cmp op x y = case lookup op [(tokEq, (==)), (tokNe, (/=)), (tokLt, (<)), (tokLe, (<=)), (tokGt, (>)), (tokGe, (>=))] of
  Just f -> f x y
  Nothing -> False

-- | The inner body's answers projected onto the aggregated variables,
-- as a set: count and sum of nothing are 0, min and max of nothing
-- have no value (the literal fails).
agg :: Db -> Integer -> [(Int, Int)] -> [Lit] -> Subst -> Maybe Integer
agg db op vars body s
  | op == tokCount = Just (toInteger (S.size rows))
  | op == tokSum = Just (sum firsts)
  | S.null rows = Nothing
  | op == tokMin = Just (minimum firsts)
  | otherwise = Just (maximum firsts)
 where
  rows = S.fromList [[IM.findWithDefault 0 v s' | (v, _) <- vars] | s' <- solve db body s]
  firsts = [x | x : _ <- S.toList rows]

-- | Proof rows `[goal, answer, node, parent, rule, pred, args…]`
-- against the final database and the program: a leaf is a sent
-- fact; a rule node's children, in order, are the positive atoms of
-- its clause's body under one solution whose head (or, for a query,
-- whose outer variables) is the node's tuple.
replays :: [Clause] -> Db -> [[Integer]] -> Bool
replays clauses db rows = all node rows
 where
  node row = case row of
    g : a : n : _ : rule : p : args
      | rule < 0 -> S.member args (rel db (fromInteger p))
      | otherwise -> any (fits (clauses !! fromInteger rule) args (children g a n)) (solve db (clauseBody (clauses !! fromInteger rule)) IM.empty)
    _ -> False
  children g a n = [(fromInteger q, as) | g' : a' : _ : par : _ : q : as <- rows, g' == g, a' == a, par == n]
  fits cl args kids s = headOf cl s == args && length positives == length kids && and (zipWith (readsAs s) positives kids)
   where
    positives = [a | LPos a <- clauseBody cl]
  -- an anonymous argument names no value, so the fact read there is
  -- whatever the tuple carried
  readsAs s a (q, t) = atomPred a == q && length (atomArgs a) == length t && and [termOf x == TAnon || valueIn s x == y | (x, y) <- zip (atomArgs a) t]
  headOf cl s = case clauseHead cl of
    Just h -> map (valueIn s) (atomArgs h)
    Nothing -> [IM.findWithDefault 0 v s | v <- firstBound (clauseBody cl)]

-- | A token stream from a spaced spelling: `pN` predicate, `vN`
-- variable, `iN` integer, `sN` symbol, `eN` set, `_` anonymous, and
-- the punctuation and keywords by their spelling.
toks :: String -> [(Integer, Integer)]
toks = map one . words
 where
  one w = case w of
    'p' : ds | all isDigit ds -> (kindPred, read ds)
    'v' : ds | all isDigit ds -> (kindVar, read ds)
    'i' : ds | all isDigit ds -> (kindInt, read ds)
    's' : ds | all isDigit ds -> (kindSym, read ds)
    'e' : ds | all isDigit ds -> (kindSet, read ds)
    "_" -> (kindAnon, 0)
    _ -> (maybe (error ("toks: " <> w)) id (lookup w punct), 0)
  punct = zip [":-", ",", ".", "(", ")", "not", "?-", "assert", "=", "!=", "<", "<=", ">", ">=", "+", "-", "*", "/", "%", "count", "min", "max", "sum", ":"] [punctFloor ..]

-- | A body's variables in the order it first binds them: an atom's
-- in term order, a binding's and an aggregate's target.
firstBound :: [Lit] -> [Int]
firstBound = foldl (\seen l -> seen <> filter (`notElem` seen) (nub (newly l))) []
 where
  newly l = case l of
    LPos a -> [v | Term _ (TVar v) <- atomArgs a]
    LAssign v _ _ -> [v]
    LAgg v _ _ _ _ -> [v]
    _ -> []

-- | Each goal's distinct answer tuples, ascending, in clause order: a
-- query answers its first-bound variables, an assertion its head
-- (the violations).
goalAnswersRef :: [Clause] -> Db -> [[[Integer]]]
goalAnswersRef clauses db = [S.toAscList (S.fromList (map (answer cl) (solve db (clauseBody cl) IM.empty))) | cl <- clauses, isGoal cl]
 where
  isGoal cl = case cl of Rule {} -> False; _ -> True
  answer cl s = case clauseHead cl of
    Just h -> map (valueIn s) (atomArgs h)
    Nothing -> [IM.findWithDefault 0 v s | v <- firstBound (clauseBody cl)]

-- | What an evaluation met, by name — every body literal read under
-- every substitution the literals before it admit, over the final
-- database: each aggregate folded (and whether over nothing), each
-- operator applied, a division by zero, each comparison, a `=` on a
-- bound variable.
events :: [Clause] -> Db -> [String]
events clauses db = concat [met s l | cl <- clauses, let b = clauseBody cl, (i, l) <- zip [0 ..] b, s <- solve db (take i b) IM.empty]
 where
  met s l = case l of
    LAgg _ op _ body _ -> [spell op <> (if null (solve db body s) then " over nothing" else " over rows")]
    LAssign v e _ -> ["a bound = compares" | IM.member v s] <> applied s e
    LCmp op x y _ -> ["comparison " <> spell op] <> applied s x <> applied s y
    _ -> []
  applied s e = case e of
    EBin op x y -> ["operator " <> spell op] <> ["a division by zero fails its literal" | op `elem` [tokDiv, tokMod], ev s y == Just 0] <> applied s x <> applied s y
    _ -> []
  spell op = fromMaybe "?" (lookup op (zip [tokEq, tokNe, tokLt, tokLe, tokGt, tokGe, tokAdd, tokSub, tokMul, tokDiv, tokMod, tokCount, tokMin, tokMax, tokSum] (words "= != < <= > >= + - * / % count min max sum")))
