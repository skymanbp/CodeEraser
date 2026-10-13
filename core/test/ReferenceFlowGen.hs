-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | The seeded generator behind ReferenceFlow (design booklet §5.5):
-- two hundred small requests of one to three structured units each —
-- ifs, loops, switches, up to two trys (nested or apart) with catches
-- and a finally, breaks aimed at ANY enclosing loop or switch (a
-- switch inside a loop, an outer loop past an inner one) and
-- continues at any enclosing loop, labels in statement lists and gotos
-- to a label of a list that encloses them (forward, backward, out of a
-- try through its finally), returns, throws, exits, three variables
-- with random exemptions, zero to two accesses per statement — each
-- unit of a language drawn from the ones §5.5 lists plus one unjudged
-- code, now and then one unit carrying the dynamic flag — and the
-- request rows each one sends. Split from the interpreter so both
-- stand below the core's file wall (the Reference.hs posture:
-- deterministic, no RNG inside a byte-determinism contract).
module ReferenceFlowGen (Node (..), Prog (..), G, S (..), batches, langCodes, lcgStep, rand, request, runG, walk) where

import CE.Flow.Cost
import Data.Aeson (Value)
import Data.Bits (testBit, (.|.))
import WireHarness (tabledRequest)

-- | A statement with its accesses `(variable, mode)` in order.
data Node = Node {nSeq, nKind, nFlags, nAux :: Int, nAcc :: [(Int, Int)], nKids :: [Node]}

-- | A unit: its body, its variables `(declSeq, flags)` by id, its
-- statement count, its language code.
data Prog = Prog {pRoots :: [Node], pVars :: [(Int, Int)], pSize :: Int, pLang :: Integer}

-- | The position context the generator reads: the enclosing loops and
-- switches innermost first (seq, is a loop), whether the position
-- sits inside a try body, the depth.
data Ctx = Ctx {targets :: [(Int, Bool)], inTry :: Bool, depth :: Int}

-- | The generator's state: the LCG seed, the next seq, and the
-- program's remaining loop and try quotas (the trace count is what
-- they bound).
data S = S {seed, counter, loopQuota, tryQuota :: Int}

newtype G a = G (S -> (a, S))

instance Functor G where
  fmap f (G g) = G (\s -> let (a, s') = g s in (f a, s'))

instance Applicative G where
  pure a = G (\s -> (a, s))
  G f <*> G g = G (\s -> let (h, s1) = f s; (a, s2) = g s1 in (h a, s2))

instance Monad G where
  G g >>= k = G (\s -> let (a, s1) = g s; G h = k a in h s1)

runG :: G a -> S -> a
runG (G g) s = fst (g s)

-- | One step of the seeded batteries' one LCG (audit C row 21): every
-- battery that draws, through `G` or through a plain stream, steps it;
-- its high bits are the draw (`div` 65536).
lcgStep :: Integer -> Integer
lcgStep x = (x * 1103515245 + 12345) `mod` 2147483648

-- | A number in [0, n).
rand :: Int -> G Int
rand n = G (\s -> let x = fromInteger (lcgStep (toInteger (seed s))) in (x `div` 65536 `mod` n, s {seed = x}))

fresh :: G Int
fresh = G (\s -> (counter s, s {counter = counter s + 1}))

spend :: (S -> Int) -> (S -> S) -> G Bool
spend quota pay = G (\s -> if quota s > 0 then (True, pay s) else (False, s))

-- | The language codes a unit is drawn from: the ten §5.5 judges and
-- Haskell's 6, a code the flow family does not judge.
langCodes :: [Integer]
langCodes = [0, 1, 2, 3, 4, 15, 16, 17, 18, 20, 6]

-- | Two hundred seeded requests.
batches :: [[Prog]]
batches = [runG (batch n) (S (n * 7919 + 17) 0 2 2) | n <- [1 .. 200]]

batch :: Int -> G [Prog]
batch n = do
  units <- (1 +) <$> rand 3
  mapM (\u -> G (\s -> ((), s {counter = 0, loopQuota = 2, tryQuota = 2})) >> program (n + u)) [1 .. units]

program :: Int -> G Prog
program n = do
  count <- rand 4
  raw <- stmts (Ctx [] False 0) (count + n `mod` 3)
  aimed <- aimList [] raw
  roots <- (aimed <>) <$> (rand 10 >>= scaffold)
  size <- G (\s -> (counter s, s))
  locals <- mapM (const (localVar size)) [() | size > 0, _ <- [1, 2 :: Int]]
  paramFlags <- exemptions
  lang <- (langCodes !!) <$> rand (length langCodes)
  dyn <- rand (5 * max 1 size)
  pure (Prog (map (flagAt dyn) roots) ((-1, paramFlags .|. 1) : locals) size lang)
 where
  -- one unit in five carries the dynamic flag on one statement
  flagAt s x = x {nFlags = nFlags x .|. (if nSeq x == s then 2 ^ flagDynamic else 0), nKids = map (flagAt s) (nKids x)}

-- | One unit in two closes on a scaffold: a shape the die reaches too
-- rarely, built whole in pre-order with its jumps aimed and its
-- accesses drawn — a loop around a switch whose case breaks the loop;
-- two loops, the inner one breaking or continuing the outer; a return
-- two trys deep, each try with a finally; a goto out of a try with a
-- finally, forward to a label after it; a catchless try whose body
-- writes a local twice and whose finally reads it (the first store
-- lives on the throw through the finally, ruling 2026-10-13).
scaffold :: Int -> G [Node]
scaffold r = case r of
  0 -> single (mk kindLoop 0 (\l -> [mk kindSwitch 0 (const [mk kindCase 0 (const [plain, leafAt kindBreak l])])]))
  1 -> single (mk kindLoop 0 (\o -> [mk kindLoop 0 (const [mk kindIf (2 ^ flagElse) (const [leafAt kindBreak o, leafAt kindContinue o])])]))
  2 -> single (mk kindTry 0 (const [mk kindTry 0 (const [plain, leafAt kindReturn 0, fin]), fin]))
  3 -> do
    t <- mk kindTry 0 (const [leafAt kindGoto (-1), fin])
    l <- mk kindLabel 0 (const [plain])
    pure [t {nKids = [k {nAux = if nKind k == kindGoto then nSeq l else nAux k} | k <- nKids t]}, l]
  4 -> single (mk kindTry 0 (const [at [(1, modeWrite)], at [(1, modeWrite)], mk kindFinally 0 (const [at [(1, modeRead)]])]))
  _ -> pure []
 where
  single = fmap (: [])
  fin = mk kindFinally 0 (const [plain])
  plain = leafAt kindStmt 0
  at acc = (\n -> n {nAcc = acc}) <$> mk kindStmt 0 (const [])
  leafAt k aux = (\n -> n {nAux = aux}) <$> mk k 0 (const [])
  mk k flags kids = do
    s <- fresh
    acc <- accesses
    ks <- sequence (kids s)
    pure (Node s k flags 0 acc ks)

localVar :: Int -> G (Int, Int)
localVar size = (,) <$> rand size <*> exemptions

-- | Captured, ignored or address-taken, each one time in five.
exemptions :: G Int
exemptions = foldr (.|.) 0 <$> mapM (\bit -> (\r -> if r == 0 then 2 ^ bit else 0) <$> rand 5) [varCaptured, varIgnored, varAddress]

-- | A statement list: each place one time in seven a label (zero or
-- one statement under it) below depth three, else a statement.
stmts :: Ctx -> Int -> G [Node]
stmts ctx n = mapM (const item) [1 .. n]
 where
  item = rand 7 >>= \r -> if r == 0 && depth ctx < 3 then labelled else stmt ctx
  labelled = do
    s <- fresh
    acc <- accesses
    kids <- rand 2 >>= stmts ctx {depth = depth ctx + 1}
    pure (Node s kindLabel 0 0 acc kids)

-- | One statement: the slot a twenty-two-sided die lands on names its
-- kind — seven plain statements, three ifs, two loops, then a
-- switch, a try, a return, a throw, an exit, two breaks, a continue,
-- a goto and a block. A kind the context or the quotas do not admit,
-- and any kind at depth three, is a plain statement instead. A goto
-- leaves with no target; `aimList` aims it once the unit's labels
-- exist.
stmt :: Ctx -> G Node
stmt ctx = do
  r <- rand 22
  if depth ctx >= 3 then leaf kindStmt 0 else choices !! r
 where
  choices =
    replicate 7 (leaf kindStmt 0)
      <> replicate 3 (compound kindIf)
      <> replicate 2 (quota loopQuota (\s -> s {loopQuota = loopQuota s - 1}) (compound kindLoop))
      <> [ compound kindSwitch
         , quota tryQuota (\s -> s {tryQuota = tryQuota s - 1}) (compound kindTry)
         , leaf kindReturn 0
         , leaf kindThrow 0
         , leaf kindNoreturn 0
         , jump kindBreak (map fst (targets ctx))
         , jump kindBreak (map fst (targets ctx))
         , jump kindContinue [t | (t, True) <- targets ctx]
         , leaf kindGoto (-1)
         , compound kindBlock
         ]
  quota q pay make = spend q pay >>= \ok -> if ok then make else leaf kindStmt 0
  -- one of the three innermost targets
  jump k ts = if null ts then leaf kindStmt 0 else rand (min 3 (length ts)) >>= \i -> leaf k (ts !! i)
  leaf k aux = do
    s <- fresh
    acc <- accesses
    pure (Node s k 0 aux acc [])
  compound k = do
    s <- fresh
    acc <- accesses
    (flags, kids) <- body ctx k s
    pure (Node s k flags 0 acc kids)

-- | Zero to two accesses of three variables.
accesses :: G [(Int, Int)]
accesses = rand 3 >>= \n -> mapM (const ((,) <$> rand 3 <*> rand 3)) [1 .. n]

-- | A compound statement's flags and children under its own context.
body :: Ctx -> Int -> Int -> G (Int, [Node])
body ctx k s
  | k == kindIf = do
      hasElse <- rand 2
      thenB <- branch
      elseB <- if hasElse == 1 then (: []) <$> branch else pure []
      pure (hasElse, thenB : elseB)
  | k == kindLoop = do
      inf <- rand 4
      kids <- rand 3 >>= \n -> stmts inner {targets = (s, True) : targets ctx} (n + 1)
      pure (if inf == 0 then 2 else 0, kids)
  | k == kindSwitch = do
      hasElse <- rand 2
      n <- rand 3
      arms <- mapM (const (arm inner {targets = (s, False) : targets ctx})) [0 .. n]
      pure (hasElse, arms)
  | k == kindTry = do
      bodyKids <- rand 2 >>= \n -> stmts inner {inTry = True} (n + 1)
      catches <- rand 3 >>= \n -> mapM (const (handler inner kindCatch)) [1 .. n]
      fin <- rand 2 >>= \n -> mapM (const (handler inner kindFinally)) [1 .. n]
      pure (0, bodyKids <> catches <> fin)
  | otherwise = (,) 0 <$> (rand 2 >>= \n -> stmts inner (n + 1))
 where
  inner = ctx {depth = depth ctx + 1}
  branch = rand 2 >>= \single -> if single == 1 then stmt inner else compoundKids kindBlock 0 inner
  arm c = rand 3 >>= \ft -> compoundKids kindCase (if ft == 0 then 4 else 0) c
  handler c k' = compoundKids k' 0 c

-- | A block-like node (block, case, catch, finally) of zero to two
-- statements; a case may carry the fallthrough flag.
compoundKids :: Int -> Int -> Ctx -> G Node
compoundKids k flags ctx = do
  s <- fresh
  acc <- accesses
  kids <- rand 3 >>= stmts ctx
  pure (Node s k flags 0 acc kids)

-- | Aim every goto at a label of a statement list that encloses it —
-- the lists a unit body, a block, a loop, a case, a catch, a finally,
-- a label and a try's body hold (an if's branches and a switch's arms
-- are no such list, and a catch or finally is not in its try's body
-- list) — chosen at random; a goto with none becomes a plain
-- statement. `outer` are the labels the enclosing lists hold.
aimList :: [Int] -> [Node] -> G [Node]
aimList outer nodes = mapM (aimNode (here <> outer)) nodes
 where
  here = [nSeq n | n <- nodes, nKind n == kindLabel]

aimNode :: [Int] -> Node -> G Node
aimNode labels n
  | nKind n == kindGoto && null labels = pure n {nKind = kindStmt, nAux = 0}
  | nKind n == kindGoto = (\i -> n {nAux = labels !! i}) <$> rand (length labels)
  | nKind n `elem` [kindIf, kindSwitch] = kidsBy (aimNode labels)
  | nKind n == kindTry = do
      bodyKids <- aimList labels [k | k <- nKids n, not (handler k)]
      handlers <- mapM (aimNode labels) [k | k <- nKids n, handler k]
      pure n {nKids = bodyKids <> handlers}
  | otherwise = (\ks -> n {nKids = ks}) <$> aimList labels (nKids n)
 where
  kidsBy f = (\ks -> n {nKids = ks}) <$> mapM f (nKids n)
  handler k = nKind k `elem` [kindCatch, kindFinally]

-- | The request a generated batch sends: a units row per unit (its
-- language, its parameter count), the rows in pre-order, unit by unit.
request :: [Prog] -> Value
request ps =
  tabledRequest
    "7.0.0"
    "flow.request"
    [ ("units", [[u, pLang p, toInteger (length [() | (_, f) <- pVars p, testBit f varParam])] | (u, p) <- units])
    , ("stmts", [[u, toInteger (nSeq n), toInteger parent, toInteger (nKind n), toInteger (nFlags n), toInteger (nAux n)] | (u, p) <- units, (parent, n) <- walk (-1) (pRoots p)])
    , ("vars", [[u, v, toInteger decl, toInteger flags] | (u, p) <- units, (v, (decl, flags)) <- zip [0 ..] (pVars p)])
    , ("uses", [[u, toInteger (nSeq n), toInteger v, toInteger m] | (u, p) <- units, (_, n) <- walk (-1) (pRoots p), (v, m) <- nAcc n])
    ]
 where
  units = zip [0 ..] ps

-- | Every node with its parent, in pre-order (ascending seq).
walk :: Int -> [Node] -> [(Int, Node)]
walk parent = concatMap (\n -> (parent, n) : walk (nSeq n) (nKids n))
