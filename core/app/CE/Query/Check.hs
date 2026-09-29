-- | The checker (design booklet §4.1 / §4.5): a parsed program passes
-- arity, sorts, safety, predicate existence, prelude reservation and
-- stratification before anything is evaluated, and the first phase
-- that finds fault answers its errors — every one at a token. What
-- comes out is the evaluator's program: the rules with their strata,
-- the goals with their projections and sorts.
module CE.Query.Check (Checked (..), Goal (..), RuleC (..), check) where

import CE.Query.Check.Safety (outerVars, safetyErrors)
import CE.Query.Check.Sorts (Sorts, inferSorts, variableSort)
import CE.Query.Cost (errArity, errPrelude, errUnknownPred, errUnstratified, sortInt, sortSym, sortSet)
import CE.Query.Schema (arityOf)
import CE.Query.Syntax
import qualified Data.Graph as G
import qualified Data.IntMap.Strict as IM
import qualified Data.IntSet as IS
import Data.List (nub)
import qualified Data.Map.Strict as M
import Data.Tree (flatten)

-- | A rule as the evaluator runs it: its clause index, head, body.
data RuleC = RuleC {ruleClause :: Int, ruleHead :: Atom, ruleBody :: [Lit]}

-- | A goal: its clause index, kind (0 query / 1 assert), the
-- predicate it stands for (an assert's head; −1 for a query), the
-- terms it answers with (a query's outer variables, an assert's
-- head), the body, the sort of each answered column.
data Goal = Goal {goalClause :: Int, goalKind :: Integer, goalPred :: Int, goalTerms :: [TermV], goalBody :: [Lit], goalSorts :: [Integer]}

data Checked = Checked
  { chkRules :: [RuleC]
  , chkStrata :: [[Int]]
  -- ^ rule indices (into chkRules) per stratum, lowest first
  , chkGoals :: [Goal]
  }

-- | Phases in order; the first faulty phase answers.
check :: Int -> [Clause] -> Either [(Int, Integer)] Checked
check prelude clauses = do
  phase (arityErrors clauses)
  sorts <- either (Left . (: [])) Right (inferSorts clauses)
  phase (concatMap safetyErrors clauses)
  phase (unknownErrors clauses)
  phase (preludeErrors prelude clauses)
  strata <- stratify rules
  Right (Checked rules strata (goals sorts))
 where
  rules = [RuleC i h b | (i, Rule h b _) <- zip [0 ..] clauses]
  goals sorts = [goal sorts i cl | (i, cl) <- zip [0 ..] clauses, isGoal cl]
  isGoal cl = case cl of Rule {} -> False; _ -> True
  phase errs = if null errs then Right () else Left errs

-- | Every atom's arity against the schema, or against the first
-- appearance of a program predicate.
arityErrors :: [Clause] -> [(Int, Integer)]
arityErrors clauses = go IM.empty (concatMap atomsOf clauses)
 where
  go _ [] = []
  go seen (Atom p args at : rest) = case arity of
    Just n | n /= length args -> (at, errArity) : go seen rest
    Just _ -> go seen rest
    Nothing -> go (IM.insert p (length args) seen) rest
   where
    arity = if isIdb p then IM.lookup p seen else arityOf p

atomsOf :: Clause -> [Atom]
atomsOf cl = maybe [] (: []) (clauseHead cl) <> [a | l <- clauseBody cl, (_, a) <- litAtoms l]

-- | A program predicate read anywhere but defined by no rule.
unknownErrors :: [Clause] -> [(Int, Integer)]
unknownErrors clauses =
  [ (at, errUnknownPred)
  | cl <- clauses
  , l <- clauseBody cl
  , (_, Atom p _ at) <- litAtoms l
  , isIdb p
  , not (IS.member p defined)
  ]
 where
  defined = IS.fromList [atomPred h | Rule h _ _ <- clauses]

-- | The first `prelude` clauses reserve the predicates they define.
preludeErrors :: Int -> [Clause] -> [(Int, Integer)]
preludeErrors prelude clauses =
  [(atomAt h, errPrelude) | Rule h _ _ <- drop prelude clauses, IS.member (atomPred h) reserved]
 where
  reserved = IS.fromList [atomPred h | Rule h _ _ <- take prelude clauses]

-- | Strata by the level rule: a head sits at least as high as every
-- positive body predicate and strictly above every negated or
-- aggregated one; a negative edge inside a strongly connected
-- component is the unstratifiable rule, named at its head.
stratify :: [RuleC] -> Either [(Int, Integer)] [[Int]]
stratify rules
  | (r : _) <- bad = Left [(atomAt (ruleHead r), errUnstratified)]
  | otherwise = Right (M.elems (M.fromListWith (flip (<>)) [(level (atomPred (ruleHead r)), [i]) | (i, r) <- zip [0 ..] rules]))
 where
  preds = nub [atomPred (ruleHead r) | r <- rules]
  index = IM.fromList (zip preds [0 ..])
  edges = [(atomPred a, atomPred (ruleHead r), pos) | r <- rules, l <- ruleBody r, (pos, a) <- litAtoms l, isIdb (atomPred a)]
  graph = G.buildG (0, max 0 (length preds - 1)) [(index IM.! p, index IM.! h) | (p, h, _) <- edges]
  comps = IM.fromList [(v, c) | (c, t) <- zip [0 :: Int ..] (G.scc graph), v <- flatten t]
  sameComp p h = IM.lookup (index IM.! p) comps == IM.lookup (index IM.! h) comps
  bad = [r | r <- rules, l <- ruleBody r, (False, a) <- litAtoms l, isIdb (atomPred a), sameComp (atomPred a) (atomPred (ruleHead r))]
  levels = fixLevels edges (IM.fromList [(p, 0 :: Int) | p <- preds])
  level p = IM.findWithDefault 0 p levels

-- | Raise heads until nothing moves (bounded by the predicate count,
-- which a stratifiable program never exceeds).
fixLevels :: [(Int, Int, Bool)] -> IM.IntMap Int -> IM.IntMap Int
fixLevels edges = go (IM.size (IM.fromList [(h, ()) | (_, h, _) <- edges]) + 2)
 where
  go :: Int -> IM.IntMap Int -> IM.IntMap Int
  go 0 m = m
  go n m = if m' == m then m else go (n - 1) m'
   where
    m' = foldl raise m edges
  raise m (p, h, pos) = IM.insertWith max h (IM.findWithDefault 0 p m + (if pos then 0 else 1)) m

-- | A goal's answered terms and their sorts.
goal :: Sorts -> Int -> Clause -> Goal
goal sorts i cl = case cl of
  Query body _ -> Goal i 0 (-1) (map TVar vars) body (map (variableSort sorts i) vars)
   where
    vars = outerVars body
  Assert h body _ -> Goal i 1 (atomPred h) (map termOf (atomArgs h)) body (map (termSort sorts i) (atomArgs h))
  Rule {} -> Goal i 0 (-1) [] [] []

termSort :: Sorts -> Int -> Term -> Integer
termSort sorts c (Term _ v) = case v of
  TVar x -> variableSort sorts c x
  TInt _ -> sortInt
  TSym _ -> sortSym
  TSet _ -> sortSet
  TAnon -> sortInt
