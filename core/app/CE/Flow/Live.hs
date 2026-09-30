-- | Backward liveness over a unit's control-flow graph and the three
-- variable findings it yields (design booklet §5.2): a dead store is
-- a write no path reads before the variable is written again or the
-- unit exits; an unused local is a local nothing ever reads; an
-- unused parameter is a parameter nothing ever reads (advisory).
-- Accesses within one statement are ordered — `x = x + 1` reads
-- before it writes — so the transfer walks them backwards, and a
-- store's liveness is asked right after that store, inside the
-- statement, not at the statement's boundary.
--
-- Exemptions are the measuring side's facts: a captured or an
-- address-taken variable has readers this graph cannot see, an
-- ignored variable was named to be ignored. A variable nothing reads
-- gets its unused finding and no dead-store findings — one fact, one
-- finding. A store in unreachable code is covered by its unreachable
-- run and is not reported twice.
module CE.Flow.Live (deadStores, unused) where

import CE.Flow.Cfg (Cfg (..))
import CE.Flow.Cost
import CE.Flow.Tree (Unit (..), Use (..), Var (..))
import Data.Bits (testBit)
import qualified Data.IntMap.Strict as IM
import qualified Data.IntSet as IS
import qualified Data.Set as S

-- | Live-in of every node, to a fixpoint of `in = transfer(∪ in of
-- successors)` over the whole graph; nodes without accesses (entry,
-- exit, hubs) pass their out set through.
liveIn :: Unit -> Cfg -> IM.IntMap IS.IntSet
liveIn u g = go (IM.fromList [(node, IS.empty) | node <- [0 .. nodeCount g - 1]])
 where
  go live
    | live' == live = live
    | otherwise = go live'
   where
    live' = IM.mapWithKey (\node _ -> transfer (accesses u node) (liveOut live node)) live
  liveOut live node = IS.unions [IM.findWithDefault IS.empty s live | s <- IM.findWithDefault [] node (succs g)]

accesses :: Unit -> Int -> [Use]
accesses u node = IM.findWithDefault [] node (uses u)

-- | The live set before a statement's accesses, from the one after
-- them: walked last access first.
transfer :: [Use] -> IS.IntSet -> IS.IntSet
transfer acc out = foldr step out acc
 where
  step (Use _ v m) live
    | m == modeWrite = IS.delete v live
    | otherwise = IS.insert v live

-- | The `(seq, variable)` pairs of dead stores among the reachable
-- statements, ascending and distinct.
deadStores :: Unit -> Cfg -> IS.IntSet -> [(Int, Int)]
deadStores u g seen =
  S.toAscList (S.fromList [(s, v) | s <- IM.keys (uses u), IS.member s seen, v <- deadAt s])
 where
  live = liveIn u g
  judged = IS.fromList [vId v | v <- IM.elems (vars u), not (testBit (vFlags v) varCaptured), not (testBit (vFlags v) varAddress), IS.member (vId v) (readVars u)]
  deadAt s = walk (IS.unions [IM.findWithDefault IS.empty t live | t <- IM.findWithDefault [] s (succs g)]) (reverse (accesses u s))
  walk _ [] = []
  walk after (Use _ v m : earlier)
    | m /= modeRead && IS.notMember v after && IS.member v judged = v : walk (before after v m) earlier
    | otherwise = walk (before after v m) earlier
  before after v m
    | m == modeWrite = IS.delete v after
    | otherwise = IS.insert v after

-- | Every variable some access reads (a read or a read-write).
readVars :: Unit -> IS.IntSet
readVars u = IS.fromList [v | acc <- IM.elems (uses u), Use _ v m <- acc, m /= modeWrite]

-- | `(kind, variable, declSeq)` for every unread variable the
-- exemptions leave: an unused local, or an unused parameter.
unused :: Unit -> [(Integer, Int, Int)]
unused u =
  [ (if param then findUnusedParam else findUnusedLocal, vId v, vDecl v)
  | v <- IM.elems (vars u)
  , IS.notMember (vId v) (readVars u)
  , not (any (testBit (vFlags v)) [varCaptured, varIgnored, varAddress])
  , let param = testBit (vFlags v) varParam
  ]
