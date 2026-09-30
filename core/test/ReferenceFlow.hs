-- | An independently written reference for the flow family (design
-- booklet §5.5): no control-flow graph at all — a tree-walking
-- abstract interpreter that enumerates every execution TRACE of a
-- structured unit (every branch, every case, each loop head visited
-- up to twice, every statement inside a try body allowed to hand
-- control to a catch), and reads the findings off the traces: a
-- statement no trace visits is unreachable, a store no trace reads
-- before the next store is dead — every store of a statement on its
-- own, so a store the same statement overwrites is dead, and the
-- statement's (seq, v) row is reported when any of its stores is.
-- The shipped judgment must agree
-- finding for finding on every program of the seeded generator
-- (ReferenceFlowGen). The only shared piece is the wire: the
-- generator writes the request rows, the reference reads its own
-- tree, and the two meet on the reply.
module ReferenceFlow (referenceFindings) where

import CE.Flow.Cost
import Data.Bits (testBit)
import qualified Data.IntMap.Strict as IM
import qualified Data.IntSet as IS
import Data.List (nub, sort)
import ReferenceFlowGen (Ctx (..), Node (..), Prog (..), walk)

-- | How an execution of a statement list ends. `Stuck` is a trace
-- cut inside an infinite loop that ran out of budget: it ends the
-- execution where it stands and runs no finally.
data Comp = Normal | Brk Int | Cont Int | Ret | Thr | Stuck deriving (Eq, Ord)

-- | Every execution of a statement list: the seqs visited, the
-- completion.
runList :: Ctx -> [Node] -> [([Int], Comp)]
runList _ [] = [([], Normal)]
runList ctx (n : rest) =
  concat
    [ if c == Normal then [(tr <> tr', c') | (tr', c') <- runList ctx rest] else [(tr, c)]
    | (tr, c) <- runNode ctx n
    ]

-- | Every execution of one statement. Inside a try body, any
-- statement may hand control to a catch right after its own
-- accesses (the conservative rule), so each yields a throwing
-- alternative too.
runNode :: Ctx -> Node -> [([Int], Comp)]
runNode ctx n = throwAt ctx s <> byKind
 where
  s = nSeq n
  k = nKind n
  byKind
    | k `elem` [kindReturn, kindThrow, kindNoreturn] = [([s], Ret)]
    | k == kindBreak = [([s], Brk (nAux n))]
    | k == kindContinue = [([s], Cont (nAux n))]
    | k == kindIf = visits (runList ctx (take 1 (nKids n))) <> (if testBit (nFlags n) flagElse then visits (runList ctx (drop 1 (nKids n))) else [([s], Normal)])
    | k == kindLoop = runLoop ctx n (2 :: Int)
    | k == kindSwitch = runSwitch ctx n
    | k == kindTry = runTry ctx n
    | otherwise = visits (runList ctx (nKids n))
  visits = map (\(tr, c) -> (s : tr, c))

throwAt :: Ctx -> Int -> [([Int], Comp)]
throwAt ctx s = [([s], Thr) | inTry ctx]

-- | A loop head: leave (a finite loop), or run the body and come
-- back — up to the budget; an infinite loop out of budget is a
-- truncated trace.
runLoop :: Ctx -> Node -> Int -> [([Int], Comp)]
runLoop ctx n k =
  [([s], Normal) | not infinite]
    <> throwAt ctx s
    <> (if k == 0 then [([s], Stuck) | infinite] else [(s : tr <> tr', c') | (tr, c) <- runList ctx (nKids n), (tr', c') <- after c])
 where
  s = nSeq n
  infinite = testBit (nFlags n) flagInfinite
  after c
    | c == Normal || c == Cont s = runLoop ctx n (k - 1)
    | c == Brk s = [([], Normal)]
    | otherwise = [([], c)]

-- | A switch: one of its arms, chained on fallthrough, or none
-- without a default; a break aimed at it completes it.
runSwitch :: Ctx -> Node -> [([Int], Comp)]
runSwitch ctx n =
  [([s], Normal) | not (testBit (nFlags n) flagElse)]
    <> [(s : tr, out c) | i <- [0 .. length arms - 1], (tr, c) <- arm i]
 where
  s = nSeq n
  arms = nKids n
  out c = if c == Brk s then Normal else c
  arm i =
    [ (tr <> tr', c')
    | (tr, c) <- runNode ctx (arms !! i)
    , (tr', c') <- if c == Normal && testBit (nFlags (arms !! i)) flagFallthrough && i + 1 < length arms then arm (i + 1) else [([], c)]
    ]

-- | A try: the body, whose throwing traces may run a catch or
-- propagate; then the finally, after which a trace may resume with
-- ANY completion the body or the catches produced (the union the
-- graph's hub takes); a truncated trace runs no finally.
runTry :: Ctx -> Node -> [([Int], Comp)]
runTry ctx n = [(s : tr, c) | (tr, c) <- stuck <> finished]
 where
  s = nSeq n
  bodyKids = [c | c <- nKids n, nKind c `notElem` [kindCatch, kindFinally]]
  bodyRuns = runList ctx {inTry = True} bodyKids
  caught = [(tr <> ct, cc) | (tr, Thr) <- bodyRuns, c <- nKids n, nKind c == kindCatch, (ct, cc) <- runNode ctx c]
  pend = bodyRuns <> caught
  stuck = [(tr, Stuck) | (tr, Stuck) <- pend]
  live = [(tr, c) | (tr, c) <- pend, c /= Stuck]
  resumes = nub (map snd live)
  finished = case [f | f <- nKids n, nKind f == kindFinally] of
    (f : _) -> [(tr <> ft, c') | (tr, _) <- live, (ft, fc) <- runNode ctx f, c' <- if fc == Normal then resumes else [fc]]
    [] -> live

-- | The findings the traces yield, as the wire's rows.
referenceFindings :: Prog -> [[Integer]]
referenceFindings p =
  sort
    ( [[0, findUnreachable, toInteger a, -1, toInteger b] | (a, b) <- groupRuns [s | s <- [0 .. pSize p - 1], IS.notMember s seen]]
        <> [[0, findDeadStore, toInteger s, toInteger v, toInteger s] | (s, v) <- deadStores]
        <> [[0, kind, toInteger decl, toInteger v, toInteger decl] | (v, (decl, flags)) <- zip [0 ..] (pVars p), IS.notMember v readVars, not (any (testBit flags) [varCaptured, varIgnored, varAddress]), let kind = if testBit flags varParam then findUnusedParam else findUnusedLocal]
    )
 where
  traces = map fst (runList (Ctx [] [] False 0) (pRoots p))
  accOf = IM.fromList [(nSeq n, nAcc n) | (_, n) <- walk (-1) (pRoots p)]
  seen = IS.fromList (concat traces)
  readVars = IS.fromList [v | acc <- IM.elems accOf, (v, m) <- acc, m /= modeWrite]
  judged v = IS.member v readVars && not (any (testBit (snd (pVars p !! v))) [varCaptured, varAddress])
  events = [[(s, j, v, m) | s <- tr, (j, (v, m)) <- zip [0 :: Int ..] (IM.findWithDefault [] s accOf)] | tr <- traces]
  stores = nub [(s, j, v) | ev <- events, (s, j, v, m) <- ev, m /= modeRead, judged v]
  deadStores = nub [(s, v) | (s, j, v) <- stores, and [not (readLater v rest) | ev <- events, ((s', j', v', m') : rest) <- tails' ev, s' == s, j' == j, v' == v, m' /= modeRead]]

-- | Whether the variable is read before its next plain write.
readLater :: Int -> [(Int, Int, Int, Int)] -> Bool
readLater v ((_, _, v', m) : rest)
  | v' /= v = readLater v rest
  | m == modeWrite = False
  | otherwise = True
readLater _ [] = False

tails' :: [a] -> [[a]]
tails' [] = []
tails' xs@(_ : rest) = xs : tails' rest

-- | Maximal runs of consecutive numbers.
groupRuns :: [Int] -> [(Int, Int)]
groupRuns [] = []
groupRuns (s : rest) = let (run, others) = takeRun s rest in (s, s + length run) : groupRuns others
 where
  takeRun _ [] = ([], [])
  takeRun prev (x : xs)
    | x == prev + 1 = let (a, b) = takeRun x xs in (x : a, b)
    | otherwise = ([], x : xs)
