-- | An independently written reference for the flow family (design
-- booklet §5.5): no control-flow graph at all — a tree-walking
-- abstract interpreter that enumerates every execution TRACE of a
-- structured unit (every branch, every case, each loop head visited
-- up to twice, every statement inside a try body allowed to hand
-- control to a catch, and inside the body or a catch of a try with a
-- finally to throw on out through it, a goto resuming its label's statement list at
-- the label, up to three times per run of that list), and reads the
-- findings off the traces: a statement no trace visits is unreachable,
-- a store no trace reads before the next store is dead — every store
-- of a statement on its own, so a store the same statement overwrites
-- is dead, and the statement's (seq, v) row is reported when any of
-- its stores is. A unit with a dynamic statement is not judged, only
-- counted. The judged bit of every finding comes off this module's own
-- copy of §5's two tables (the advisory kinds, the judged languages by
-- wire code). The shipped judgment must agree finding for finding on
-- every request of the seeded generator (ReferenceFlowGen). The only
-- shared piece is the wire: the generator writes the request rows, the
-- reference reads its own trees, and the two meet on the reply.
module ReferenceFlow (dynamicUnits, judgedBit, referenceFindings) where

import CE.Flow.Cost
import Data.Bits (testBit)
import qualified Data.IntMap.Strict as IM
import qualified Data.IntSet as IS
import Data.List (nub, sort)
import ReferenceFlowGen (Node (..), Prog (..), walk)

-- | How an execution of a statement list ends. `Stuck` is a trace
-- cut inside an infinite loop, or a goto loop, that ran out of
-- budget: it ends the execution where it stands and runs no finally.
-- `Goto l` travels up until the statement list holding label l
-- resumes there.
data Comp = Normal | Brk Int | Cont Int | Goto Int | Ret | Thr | Stuck deriving (Eq, Ord)

-- | Every label of the unit with the statements of its list from the
-- label on — built once, before any trace runs. The lists are the ones
-- the interpreter runs as lists: a unit body, a try's body, the
-- children of anything but an if or a switch.
type Labels = IM.IntMap [Node]

labelsOf :: [Node] -> Labels
labelsOf roots = IM.fromList (lists roots)
 where
  lists ns = [(nSeq x, drop i ns) | (i, x) <- zip [0 ..] ns, nKind x == kindLabel] <> concatMap inner ns
  inner x
    | nKind x `elem` [kindIf, kindSwitch] = concatMap inner (nKids x)
    | nKind x == kindTry = lists [k | k <- nKids x, not (handler k)] <> concatMap inner (filter handler (nKids x))
    | otherwise = lists (nKids x)
  handler k = nKind k `elem` [kindCatch, kindFinally]

-- | Every execution of a statement list: the seqs visited, the
-- completion. A goto to one of this list's labels resumes the list at
-- the label, three times at most per run of the list (then the trace
-- is cut).
runList :: Labels -> Bool -> [Node] -> [([Int], Comp)]
runList labels inTry nodes = go (3 :: Int) nodes
 where
  go _ [] = [([], Normal)]
  go budget (n : rest) = concat [settle budget rest tr c | (tr, c) <- runNode labels inTry n]
  settle budget rest tr c = case c of
    Normal -> [(tr <> tr', c') | (tr', c') <- go budget rest]
    Goto l
      | Just from@(h : _) <- IM.lookup l labels
      , nSeq h `elem` map nSeq nodes ->
          if budget == 0 then [(tr, Stuck)] else [(tr <> tr', c') | (tr', c') <- go (budget - 1) from]
    _ -> [(tr, c)]

-- | Every execution of one statement. Where a throw has somewhere to
-- go — the body of a try with a catch, or the body or a catch of a try
-- with a finally (this try's or an enclosing one's) — any statement
-- may throw right after its own accesses (§5.2, ruling 2026-10-13), so
-- each yields a throwing alternative too; `inTry` says it has.
runNode :: Labels -> Bool -> Node -> [([Int], Comp)]
runNode labels inTry n = throwAt inTry s <> byKind
 where
  s = nSeq n
  k = nKind n
  byKind
    | k `elem` [kindReturn, kindThrow, kindNoreturn] = [([s], Ret)]
    | k == kindBreak = [([s], Brk (nAux n))]
    | k == kindContinue = [([s], Cont (nAux n))]
    | k == kindGoto = [([s], Goto (nAux n))]
    | k == kindIf = visits (runList labels inTry (take 1 (nKids n))) <> (if testBit (nFlags n) flagElse then visits (runList labels inTry (drop 1 (nKids n))) else [([s], Normal)])
    | k == kindLoop = runLoop labels inTry n (2 :: Int)
    | k == kindSwitch = runSwitch labels inTry n
    | k == kindTry = runTry labels inTry n
    | otherwise = visits (runList labels inTry (nKids n))
  visits = map (\(tr, c) -> (s : tr, c))

throwAt :: Bool -> Int -> [([Int], Comp)]
throwAt inTry s = [([s], Thr) | inTry]

-- | A loop head: leave (a finite loop), or run the body and come
-- back — up to the budget; an infinite loop out of budget is a
-- truncated trace.
runLoop :: Labels -> Bool -> Node -> Int -> [([Int], Comp)]
runLoop labels inTry n k =
  [([s], Normal) | not infinite]
    <> throwAt inTry s
    <> (if k == 0 then [([s], Stuck) | infinite] else [(s : tr <> tr', c') | (tr, c) <- runList labels inTry (nKids n), (tr', c') <- after c])
 where
  s = nSeq n
  infinite = testBit (nFlags n) flagInfinite
  after c
    | c == Normal || c == Cont s = runLoop labels inTry n (k - 1)
    | c == Brk s = [([], Normal)]
    | otherwise = [([], c)]

-- | A switch: one of its arms, chained on fallthrough, or none
-- without a default; a break aimed at it completes it.
runSwitch :: Labels -> Bool -> Node -> [([Int], Comp)]
runSwitch labels inTry n =
  [([s], Normal) | not (testBit (nFlags n) flagElse)]
    <> [(s : tr, out c) | i <- [0 .. length arms - 1], (tr, c) <- arm i]
 where
  s = nSeq n
  arms = nKids n
  out c = if c == Brk s then Normal else c
  arm i =
    [ (tr <> tr', c')
    | (tr, c) <- runNode labels inTry (arms !! i)
    , (tr', c') <- if c == Normal && testBit (nFlags (arms !! i)) flagFallthrough && i + 1 < length arms then arm (i + 1) else [([], c)]
    ]

-- | A try: the body, whose throwing traces may run one of this try's
-- catches; a throw from the body or a catch travels on through this
-- try's finally when it has one (outward to an enclosing try, or out
-- of the unit), and without a finally only when an enclosing try
-- takes it; then the finally, after which a trace may resume with ANY completion the
-- body or the catches produced (the union the graph's hub takes); a
-- truncated trace runs no finally. A try inside a try is one more of
-- these on the way out: a return two trys deep runs the inner
-- finally, resumes as a return, and runs the outer.
runTry :: Labels -> Bool -> Node -> [([Int], Comp)]
runTry labels inTry n = [(s : tr, c) | (tr, c) <- stuck <> finished]
 where
  s = nSeq n
  bodyKids = [c | c <- nKids n, nKind c `notElem` [kindCatch, kindFinally]]
  has k = any ((== k) . nKind) (nKids n)
  outward = inTry || has kindFinally
  bodyRuns = runList labels (outward || has kindCatch) bodyKids
  caught = [(tr <> ct, cc) | (tr, Thr) <- bodyRuns, c <- nKids n, nKind c == kindCatch, (ct, cc) <- runNode labels outward c]
  pend = bodyRuns <> caught
  stuck = [(tr, Stuck) | (tr, Stuck) <- pend]
  live = [(tr, c) | (tr, c) <- pend, c /= Stuck, c /= Thr || outward]
  resumes = nub (map snd live)
  finished = case [f | f <- nKids n, nKind f == kindFinally] of
    (f : _) -> [(tr <> ft, c') | (tr, _) <- live, (ft, fc) <- runNode labels inTry f, c' <- if fc == Normal then resumes else [fc]]
    [] -> live

-- | §5's two tables as this module's own data: the finding kinds that
-- are advice in every language (3, the unused parameter) and the
-- language wire codes whose findings are judged (Python 0, TypeScript
-- 1, TSX 2, Rust 3, Go 4, C 15, C++ 16, Lua 17, Java 18, R 20).
advisory, judgedLanguages :: [Integer]
advisory = [3]
judgedLanguages = [0, 1, 2, 3, 4, 15, 16, 17, 18, 20]

-- | The judged bit of a finding of `kind` in a unit of `lang`.
judgedBit :: Integer -> Integer -> Integer
judgedBit lang kind = if lang `elem` judgedLanguages && kind `notElem` advisory then 1 else 0

dynamic :: Prog -> Bool
dynamic p = any (\(_, n) -> testBit (nFlags n) flagDynamic) (walk (-1) (pRoots p))

dynamicUnits :: [Prog] -> Int
dynamicUnits = length . filter dynamic

-- | The findings of a request's units, as the wire's rows.
referenceFindings :: [Prog] -> [[Integer]]
referenceFindings ps = sort [row <> [judgedBit (pLang p) kind] | (u, p) <- zip [0 ..] ps, not (dynamic p), row@(_ : kind : _) <- unitFindings u p]

-- | One unit's findings `[u, kind, seq, v, seqEnd]`.
unitFindings :: Integer -> Prog -> [[Integer]]
unitFindings u p =
  [[u, findUnreachable, toInteger a, -1, toInteger b] | (a, b) <- groupRuns [s | s <- [0 .. pSize p - 1], IS.notMember s seen]]
    <> [[u, findDeadStore, toInteger s, toInteger v, toInteger s] | (s, v) <- deadStores]
    <> [[u, kind, toInteger decl, toInteger v, toInteger decl] | (v, (decl, flags)) <- zip [0 ..] (pVars p), IS.notMember v readVars, not (any (testBit flags) [varCaptured, varIgnored, varAddress]), let kind = if testBit flags varParam then findUnusedParam else findUnusedLocal]
 where
  traces = map fst (runList (labelsOf (pRoots p)) False (pRoots p))
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
