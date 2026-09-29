-- | An independently written reference for the query family's
-- evaluator (design booklet §4.7): naive fixpoint — every rule over
-- the WHOLE database every round, no indexes, no delta routing, no
-- provenance — with the join written as a plain nested product over
-- the body in list-comprehension form. Only the checker is shared
-- (the strata are its answer, not the evaluator's); everything past
-- it is a second implementation. The shipped evaluator must agree
-- tuple for tuple on every program of the enumerated family over
-- every fact set of the seeded generator (the Reference.hs posture:
-- deterministic, no RNG inside a byte-determinism contract). The
-- same reference replays a proof tree node by node: each rule node's
-- children must be one solution of its body.
module ReferenceQuery (Db, equivalence, factSets, naive, replays, toks) where

import CE.Query.Check (Checked (..), RuleC (..), check)
import CE.Query.Check.Safety (outerVars)
import CE.Query.Cost
import CE.Query.Eval (evalProgram)
import CE.Query.Parse (parseProgram)
import CE.Query.Syntax
import Control.Monad (foldM, guard, join)
import Data.Bits (shiftR, (.&.))
import Data.Char (isDigit)
import qualified Data.IntMap.Strict as IM
import qualified Data.Map.Strict as M
import Data.Maybe (mapMaybe, maybeToList)
import qualified Data.Set as S

type Db = IM.IntMap (S.Set [Integer])
type Subst = IM.IntMap Integer

-- | The naive evaluator: strata in order, each to a fixpoint of
-- "apply every rule to the whole database".
naive :: Checked -> Db -> Db
naive chk = go (chkStrata chk)
 where
  go [] db = db
  go (stratum : rest) db = go rest (fix [chkRules chk !! i | i <- stratum] db)
  fix rules db = let db' = foldl apply db rules in if db' == db then db else fix rules db'
  apply db r = IM.insertWith S.union (atomPred (ruleHead r)) (S.fromList (heads db r)) db
  heads db r = [map (valueIn s) (atomArgs (ruleHead r)) | s <- solve db (ruleBody r) IM.empty]

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

agg :: Db -> Integer -> [(Int, Int)] -> [Lit] -> Subst -> Maybe Integer
agg db op vars body s
  | op == tokCount = Just (toInteger (S.size rows))
  | S.null rows = Nothing
  | op == tokMin = Just (minimum firsts)
  | op == tokMax = Just (maximum firsts)
  | otherwise = Just (sum firsts)
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
    Nothing -> [IM.findWithDefault 0 v s | v <- outerVars (clauseBody cl)]

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

-- | A linear congruential stream from a seed (Numerical Recipes'
-- constants): deterministic, and enough spread for small fact sets.
lcg :: Integer -> [Integer]
lcg seed = drop 1 (iterate (\x -> (1664525 * x + 1013904223) .&. 0xFFFFFFFF) seed)

-- | Fact sets over four nodes (codes 0..3): `file` for every node,
-- `ref` arcs drawn from the stream, `role` entry bits, `lines`
-- counts, `in_dir` placements — one set per seed.
factSets :: [Db]
factSets = [factSet seed | seed <- [1 .. 40]]

factSet :: Integer -> Db
factSet seed =
  IM.fromList
    [ (1, S.fromList [[n] | n <- nodes])
    , (9, S.fromList [[a, b, 0, 1] | (a, b) <- arcs])
    , (7, S.fromList [[n, 5] | n <- nodes, odd (bit n)])
    , (8, S.fromList [[n, 1 + (bit (n + 4) `shiftR` 1)] | n <- nodes])
    , (2, S.fromList [[n, bit (n + 8) .&. 1] | n <- nodes])
    ]
 where
  nodes = [0 .. 3]
  stream = lcg seed
  bit :: Integer -> Integer
  bit i = (stream !! fromInteger i) `shiftR` 16
  arcs = [(a, b) | (i, (a, b)) <- zip [12 ..] [(a, b) | a <- nodes, b <- nodes], bit i .&. 3 == 0]

-- | The programs: the prelude's three rules and its dead-file query;
-- transitive dependence; an outdegree count; arithmetic over line
-- counts; a symmetric pair broken by `<` under two negations.
programs :: [String]
programs =
  [ "p1000 ( v0 ) :- p7 ( v0 , s5 ) . p1000 ( v1 ) :- p1000 ( v0 ) , p9 ( v0 , v1 , _ , _ ) . p1001 ( v0 ) :- p1 ( v0 ) , not p1000 ( v0 ) . ?- p1001 ( v0 ) ."
  , "p1000 ( v0 , v1 ) :- p9 ( v0 , v1 , _ , _ ) . p1000 ( v0 , v1 ) :- p1000 ( v0 , v2 ) , p9 ( v2 , v1 , _ , _ ) . ?- p1000 ( v0 , v1 ) , v0 != v1 ."
  , "p1000 ( v0 , v1 ) :- p1 ( v0 ) , v1 = count ( v2 : p9 ( v0 , v2 , _ , _ ) ) . ?- p1000 ( v0 , v1 ) , v1 > i0 ."
  , "p1000 ( v0 , v1 ) :- p8 ( v0 , v2 ) , v1 = v2 * i2 + i1 , v1 > i3 . ?- p1000 ( v0 , v1 ) ."
  , "p1000 ( v0 , v1 ) :- p2 ( v0 , v2 ) , p2 ( v1 , v2 ) , v0 < v1 . p1001 ( v0 ) :- p1 ( v0 ) , not p1000 ( v0 , _ ) , not p1000 ( _ , v0 ) . ?- p1001 ( v0 ) ."
  ]

-- | Every program over every fact set: the shipped evaluator's
-- database equals the naive one's, relation for relation.
equivalence :: IO Bool
equivalence = do
  let cases = [(pi', fi, prog, db) | (pi', prog) <- zip [0 :: Int ..] programs, (fi, db) <- zip [0 :: Int ..] factSets]
      failures = [(pi', fi) | (pi', fi, prog, db) <- cases, not (agree (toks prog) db)]
      ok = null failures
  putStrLn ((if ok then "ok   " else "FAIL ") <> "semi-naive evaluator agrees with the naive reference on " <> show (length cases) <> " program × fact-set cases" <> (if ok then "" else " — first mismatches " <> show (take 3 failures)))
  pure ok

agree :: [(Integer, Integer)] -> Db -> Bool
agree prog db = case parseProgram prog >>= either (const (Left 0)) Right . check 0 of
  Left _ -> False
  Right chk -> case evalProgram chk db of
    Left _ -> False
    Right (db', _, _) -> idb db' == idb (naive chk db)
 where
  idb = M.fromList . filter (isIdb . fst) . IM.toList
