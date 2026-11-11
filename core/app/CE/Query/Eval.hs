-- | Bottom-up evaluation of a checked program (design booklet §4.1,
-- "逐层半朴素求值"): the strata lowest first, and inside a stratum
-- the semi-naive loop — every rule once over the full database, then
-- each rule whose body reads a predicate of its own stratum re-run
-- once per such position with the previous round's new tuples routed
-- to that position, until a round derives nothing new. Relations
-- below the stratum are indexed once for the whole stratum, the
-- stratum's own once per round. The first derivation of every tuple
-- is kept as its provenance; every rule stops reading its answers the
-- moment its new tuples would carry the derived count past the cap
-- (the budget left rides into `derive`, so one round can never build
-- more than the cap and one tuple — until v2.35 V7 the count was
-- compared between rounds only, after a round had built everything),
-- and crossing it abandons the whole evaluation (a degraded reply,
-- never a partial answer). Each relation is indexed on the masks the
-- program's bodies ask with alone (CE.Query.Eval.Index `usedMasks`).
module CE.Query.Eval (Db, Prov, evalProgram, evalProgramWith, goalAnswers) where

import CE.Query.Check (Checked (..), Goal (..), RuleC (..))
import CE.Query.Cost (derivedCap)
import CE.Query.Eval.Index (Indexed, Masks, Rel, Tuple, indexed, usedMasks)
import CE.Query.Eval.Join (Fact, Look, instantiate, joinBody)
import CE.Query.Syntax
import qualified Data.IntMap.Strict as IM
import qualified Data.IntSet as IS
import qualified Data.Map.Strict as M
import qualified Data.Set as S

-- | Predicate code → its tuples.
type Db = IM.IntMap Rel

-- | (predicate, tuple) → the rule that first derived it and the
-- facts that rule read.
type Prov = M.Map Fact (Int, [Fact])

-- | The database, the provenance, the derived count so far.
type State = (Db, Prov, Integer)

-- | Evaluate every stratum under the family's derived cap; Left = the
-- derived count that crossed the cap, Right = the database, the
-- provenance, the derived count.
evalProgram :: Checked -> Db -> Either Integer State
evalProgram = evalProgramWith derivedCap

-- | The same under an explicit cap (the battery exercises the abort
-- at a cap it can afford to reach).
evalProgramWith :: Integer -> Checked -> Db -> Either Integer State
evalProgramWith cap chk db0 = go (chkStrata chk) (db0, M.empty, 0)
 where
  rules = chkRules chk
  masks = usedMasks (map ruleBody rules <> map goalBody (chkGoals chk))
  go [] st = Right st
  go (stratum : rest) st = stratumFix cap masks [rules !! i | i <- stratum] st >>= go rest

-- | One stratum to its fixpoint.
stratumFix :: Integer -> Masks -> [RuleC] -> State -> Either Integer State
stratumFix cap masks rules st0@(db0, _, _) = loop st1 new1
 where
  own = IS.fromList (map (atomPred . ruleHead) rules)
  ix = IM.mapWithKey (indexOn masks)
  below = ix (IM.withoutKeys db0 own)
  snapshot db = IM.union below (ix (IM.restrictKeys db own))
  roundOf st@(db, _, _) = foldl (derive cap (snapshot db)) (st, M.empty)
  (st1, new1) = roundOf st0 [(r, Nothing) | r <- rules]
  loop st@(_, _, n) new
    | n > cap = Left n
    | M.null new = Right st
    | otherwise = uncurry loop (roundOf st (variants (deltas new)))
  deltas new = ix (IM.fromListWith S.union [(p, S.singleton t) | (p, t) <- M.keys new])
  variants ds =
    [ (r, Just (i, IM.findWithDefault empty (atomPred a) ds))
    | r <- rules
    , (i, LPos a) <- zip [0 ..] (ruleBody r)
    , IS.member (atomPred a) own
    ]

-- | A relation on the masks its predicate is asked with.
indexOn :: Masks -> Int -> Rel -> Indexed
indexOn masks p = indexed (IM.findWithDefault IS.empty p masks)

-- | The relation with no tuple.
empty :: Indexed
empty = indexed IS.empty S.empty

-- | Run one rule against the round's snapshot (with a delta routed to
-- one body position, or none) and fold its new tuples in; tuples
-- already in the threaded database are not new. The answers are read
-- until the new tuples pass the budget the cap leaves, and no further.
derive :: Integer -> IM.IntMap Indexed -> (State, Prov) -> (RuleC, Maybe (Int, Indexed)) -> (State, Prov)
derive cap snap ((db, prov, n), new) (r, route) = ((db', prov', n'), new')
 where
  full p = IM.findWithDefault empty p snap
  look :: Look
  look i p = case route of
    Just (k, delta) | k == i -> delta
    _ -> full p
  h = ruleHead r
  answers = [(instantiate s (atomArgs h), (ruleClause r, fs)) | (s, fs) <- joinBody look full (ruleBody r) IM.empty]
  have = IM.findWithDefault S.empty (atomPred h) db
  fresh = firstNew (cap - n) M.empty answers
  -- the first derivation of each tuple not yet held, kept in answer
  -- order until one more than the budget
  firstNew budget acc rest = case rest of
    _ | toInteger (M.size acc) > budget -> acc
    [] -> acc
    (t, d) : more
      | S.member t have || M.member (atomPred h, t) acc -> firstNew budget acc more
      | otherwise -> firstNew budget (M.insert (atomPred h, t) d acc) more
  db' = IM.insertWith S.union (atomPred h) (S.fromList [t | (_, t) <- M.keys fresh]) db
  prov' = M.union prov fresh
  new' = M.union new fresh
  n' = n + toInteger (M.size fresh)

-- | A goal's answers over the final database: each distinct answer
-- tuple with the facts its first answering substitution read, in
-- tuple order.
goalAnswers :: Db -> Goal -> [(Tuple, [Fact])]
goalAnswers db g = M.toAscList (M.fromListWith (\_ old -> old) rows)
 where
  snap = IM.mapWithKey (indexOn (usedMasks [goalBody g])) db
  full p = IM.findWithDefault empty p snap
  rows = [(instantiate s (map (Term 0) (goalTerms g)), fs) | (s, fs) <- joinBody (const full) full (goalBody g) IM.empty]
