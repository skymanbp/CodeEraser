-- | The control-flow graph of one unit (design booklet §5.2):
-- structured control flow unfolded from the statement kinds, a
-- try's body handed to its catches conservatively (every statement
-- inside the body may throw), a finally on every edge that leaves
-- its try, jumps routed to their targets, exits to the unit's exit.
--
-- Nodes are statement seqs, plus the entry, the exit and one hub
-- per finally: the hub is where the finally's body completes, and
-- its successors are the union of every target an edge into the
-- finally was carrying (a return through a finally still returns,
-- a fall-through still falls through). The union is a deliberate
-- over-approximation — a finally is not duplicated per target — and
-- it only ever ADDS paths, which can hide a finding, never invent
-- one (the 99 % precision gate's side of every tie).
module CE.Flow.Cfg (Cfg (..), cfg) where

import CE.Flow.Cost
import CE.Flow.Tree (Stmt (..), Unit (..), ancestors, hasFlag, kidsOf)
import qualified Data.IntMap.Strict as IM
import qualified Data.IntSet as IS
import qualified Data.Set as S

data Cfg = Cfg
  { entry :: Int
  , exit :: Int
  , nodeCount :: Int
  , succs :: IM.IntMap [Int]
  -- ^ every node's successors, ascending and distinct
  }

-- | Which part of a try a statement sits in.
data Part = Body | Catch | Finally deriving (Eq)

-- | One edge to resolve: its source node, the try chain the source
-- sits in (innermost first, each with the part) and the target node
-- the edge means to reach. Routing sends it through the innermost
-- finally that lies between the two, or straight to the target.
data Hop = Hop Int [(Int, Part)] Int

cfg :: Unit -> Cfg
cfg u = Cfg entryN exitN (n + 2 + IM.size hubs) (IM.map IS.toList (IM.fromListWith IS.union (map single edges)))
 where
  n = IM.size (stmts u)
  entryN = n
  exitN = n + 1
  hubs = IM.fromList (zip [sSeq s | s <- IM.elems (stmts u), sKind s == kindFinally] [n + 2 ..])
  hubOf f = IM.findWithDefault exitN f hubs
  edges = (entryN, firstChild u (-1) exitN) : resolve u hubs (concatMap (hops u hubOf) (IM.elems (stmts u)))
  single (a, b) = (a, IS.singleton b)

-- | Resolve every hop, then the hubs' hops as their pending targets
-- arrive, until no new (finally, target) pair appears.
resolve :: Unit -> IM.IntMap Int -> [Hop] -> [(Int, Int)]
resolve u hubs = go S.empty
 where
  go _ [] = []
  go seen (Hop from chain target : rest) = (from, to) : go seen' (rest <> new)
   where
    (to, pending) = route u hubs chain target
    fresh = filter (`S.notMember` seen) pending
    seen' = foldr S.insert seen fresh
    new = [Hop (IM.findWithDefault (-1) f hubs) (chainOf u f) t | (f, t) <- fresh]

-- | Where a hop lands: the innermost finally of a try the hop leaves
-- from inside (body or catch part) without the target being inside
-- it — and that finally then owes the target — or the target itself.
route :: Unit -> IM.IntMap Int -> [(Int, Part)] -> Int -> (Int, [(Int, Int)])
route u hubs chain target =
  case [f | (t, part) <- chain, part /= Finally, Just f <- [finallyOf u t], not (inside u hubs target t)] of
    (f : _) -> (f, [(f, target)])
    [] -> (target, [])

-- | A node is inside a try when the try is an ancestor (or the node
-- itself); a hub sits where its finally sits; entry and exit are
-- inside nothing.
inside :: Unit -> IM.IntMap Int -> Int -> Int -> Bool
inside u hubs node t = case [f | (f, h) <- IM.toList hubs, h == node] of
  (f : _) -> t `elem` (f : ancestors u f)
  [] | IM.member node (stmts u) -> t `elem` (node : ancestors u node)
  [] -> False

finallyOf :: Unit -> Int -> Maybe Int
finallyOf u t = case [c | c <- kidsOf u t, kindOf u c == kindFinally] of
  (f : _) -> Just f
  [] -> Nothing

catchesOf :: Unit -> Int -> [Int]
catchesOf u t = [c | c <- kidsOf u t, kindOf u c == kindCatch]

kindOf :: Unit -> Int -> Int
kindOf u s = maybe (-1) sKind (IM.lookup s (stmts u))

-- | The first child of a statement (or of the unit body at −1), or
-- the fallback when it has none.
firstChild :: Unit -> Int -> Int -> Int
firstChild u p fallback = case kidsOf u p of
  (c : _) -> c
  [] -> fallback

-- | The try chain of a statement: every try ancestor, innermost
-- first, with the part of it the statement sits in (told by the
-- child of the try on the path down).
chainOf :: Unit -> Int -> [(Int, Part)]
chainOf u s = [(a, partOf child) | (child, a) <- zip (s : anc) anc, kindOf u a == kindTry]
 where
  anc = ancestors u s
  partOf c
    | kindOf u c == kindCatch = Catch
    | kindOf u c == kindFinally = Finally
    | otherwise = Body

-- | A statement's outgoing hops: its structural successors, and — for
-- every try it sits in the body of — one hop to each catch.
hops :: Unit -> (Int -> Int) -> Stmt -> [Hop]
hops u hubOf s = [Hop me c t | (c, t) <- successors u hubOf s] <> [Hop me chain c | (t, Body) <- chain, c <- catchesOf u t]
 where
  me = sSeq s
  chain = chainOf u me

-- | Where a statement's structure sends control, each target under
-- the try chain the hop is routed by: the handlers' own rules, a
-- branch's alternatives, an exit to the unit's exit, a break past its
-- target, a continue or goto onto it, anything else into its first
-- child or on to what follows.
successors :: Unit -> (Int -> Int) -> Stmt -> [([(Int, Part)], Int)]
successors u hubOf s
  | k `elem` [kindTry, kindCatch, kindFinally] = handling u hubOf s
  | k `elem` [kindIf, kindLoop, kindSwitch] = under (branching u hubOf s)
  | k `elem` [kindReturn, kindThrow, kindNoreturn] = under [IM.size (stmts u) + 1]
  | k == kindBreak = under [next u hubOf (sAux s)]
  | k `elem` [kindContinue, kindGoto] = under [sAux s]
  | otherwise = under [firstChild u me (next u hubOf me)]
 where
  me = sSeq s
  k = sKind s
  under = map ((,) (chainOf u me))

-- | An if runs one branch (the else, or what follows); a loop runs
-- its body and, unless constant-true, leaves; a switch runs an arm
-- and, without a default, may run none.
branching :: Unit -> (Int -> Int) -> Stmt -> [Int]
branching u hubOf s
  | k == kindIf = take 1 children <> (if hasFlag flagElse s then drop 1 children else [after])
  | k == kindLoop = firstChild u me me : [after | not (hasFlag flagInfinite s)]
  | otherwise = children <> [after | not (hasFlag flagElse s)]
 where
  me = sSeq s
  k = sKind s
  children = kidsOf u me
  after = next u hubOf me

-- | A try enters its body, a catch its body, a finally its body or
-- its hub; an empty try body or catch completes as if from inside
-- the try, so routing still sees the try's finally.
handling :: Unit -> (Int -> Int) -> Stmt -> [([(Int, Part)], Int)]
handling u hubOf s
  | k == kindTry = case [c | c <- kidsOf u me, kindOf u c `notElem` [kindCatch, kindFinally]] of
      (c : _) -> [(chainOf u me, c)]
      [] -> [((me, Body) : chainOf u me, next u hubOf me)]
  | k == kindCatch = case kidsOf u me of
      (c : _) -> [(chainOf u me, c)]
      [] -> [((sParent s, Catch) : chainOf u (sParent s), next u hubOf (sParent s))]
  | otherwise = [(chainOf u me, firstChild u me (hubOf me))]
 where
  me = sSeq s
  k = sKind s

-- | The node control reaches when a statement completes normally:
-- the next sibling, or what the parent's completion means — after an
-- if, back to a loop's head, into the next case on fallthrough, out
-- of a switch, out of a try (its finally is routing's business), to
-- a finally's hub, or out of the unit.
next :: Unit -> (Int -> Int) -> Int -> Int
next u hubOf s = case IM.lookup s (stmts u) of
  Nothing -> exit'
  Just st -> case (sParent st, sibling st) of
    (-1, Nothing) -> exit'
    (_, Just sib) -> sib
    (p, Nothing)
      | kindOf u p == kindLoop -> p
      | kindOf u p == kindFinally -> hubOf p
      | otherwise -> next u hubOf p
 where
  exit' = IM.size (stmts u) + 1
  sibling st = case dropWhile (/= sSeq st) (peers st) of
    (_ : after : _) -> Just after
    _ -> Nothing
  -- the peers a completion may run into: a try's body children stop
  -- before its handlers; a case's next peer counts only on fallthrough
  peers st
    | kindOf u (sParent st) == kindTry = [c | c <- kidsOf u (sParent st), kindOf u c `notElem` [kindCatch, kindFinally]]
    | sKind st == kindCase && not (hasFlag flagFallthrough st) = []
    | sKind st == kindCase = kidsOf u (sParent st)
    | kindOf u (sParent st) == kindIf = []
    | otherwise = kidsOf u (sParent st)
