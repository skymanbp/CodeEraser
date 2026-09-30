-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | The seeded generator behind ReferenceFlow (design booklet §5.5):
-- two hundred small structured units — ifs, loops, switches, one try
-- with catches and a finally, breaks and continues aimed at what
-- encloses them, returns, throws, exits, three variables with random
-- exemptions, zero to two accesses per statement — and the request
-- rows each one sends. Split from the interpreter so both stand
-- below the core's file wall (the Reference.hs posture: deterministic,
-- no RNG inside a byte-determinism contract).
module ReferenceFlowGen (Ctx (..), Node (..), Prog (..), programs, request, walk) where

import CE.Flow.Cost
import Data.Aeson (Value)
import Data.Bits (testBit, (.|.))
import WireHarness (tabledRequest)

-- | A statement with its accesses `(variable, mode)` in order.
data Node = Node {nSeq, nKind, nFlags, nAux :: Int, nAcc :: [(Int, Int)], nKids :: [Node]}

-- | A unit: its body, its variables `(declSeq, flags)` by id, its
-- statement count.
data Prog = Prog {pRoots :: [Node], pVars :: [(Int, Int)], pSize :: Int}

-- | The position context both the generator and the interpreter
-- read: enclosing loops and switches, whether the position sits
-- inside a try body, the depth.
data Ctx = Ctx {loops, switches :: [Int], inTry :: Bool, depth :: Int}

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

-- | A number in [0, n).
rand :: Int -> G Int
rand n = G (\s -> let x = (seed s * 1103515245 + 12345) `mod` 2147483648 in (x `div` 65536 `mod` n, s {seed = x}))

fresh :: G Int
fresh = G (\s -> (counter s, s {counter = counter s + 1}))

spend :: (S -> Int) -> (S -> S) -> G Bool
spend quota pay = G (\s -> if quota s > 0 then (True, pay s) else (False, s))

-- | Two hundred seeded programs.
programs :: [Prog]
programs = [runG (program n) (S (n * 7919 + 17) 0 2 1) | n <- [1 .. 200]]

program :: Int -> G Prog
program n = do
  count <- rand 4
  roots <- stmts (Ctx [] [] False 0) (count + n `mod` 3)
  size <- G (\s -> (counter s, s))
  locals <- mapM (const (localVar size)) [() | size > 0, _ <- [1, 2 :: Int]]
  paramFlags <- exemptions
  pure (Prog roots ((-1, paramFlags .|. 1) : locals) size)

localVar :: Int -> G (Int, Int)
localVar size = (,) <$> rand size <*> exemptions

-- | Captured, ignored or address-taken, each one time in five.
exemptions :: G Int
exemptions = foldr (.|.) 0 <$> mapM (\bit -> (\r -> if r == 0 then 2 ^ bit else 0) <$> rand 5) [varCaptured, varIgnored, varAddress]

stmts :: Ctx -> Int -> G [Node]
stmts ctx n = mapM (const (stmt ctx)) [1 .. n]

-- | One statement: the slot a twenty-sided die lands on names its
-- kind — seven plain statements, three ifs, two loops, then a
-- switch, a try, a return, a throw, an exit, a break, a continue and
-- a block. A kind the context or the quotas do not admit, and any
-- kind at depth three, is a plain statement instead.
stmt :: Ctx -> G Node
stmt ctx = do
  r <- rand 20
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
         , jump kindBreak (loops ctx <> switches ctx)
         , jump kindContinue (loops ctx)
         , compound kindBlock
         ]
  quota q pay make = spend q pay >>= \ok -> if ok then make else leaf kindStmt 0
  jump k targets = case targets of
    (t : _) -> leaf k t
    [] -> leaf kindStmt 0
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
      kids <- rand 3 >>= \n -> stmts inner {loops = s : loops ctx} (n + 1)
      pure (if inf == 0 then 2 else 0, kids)
  | k == kindSwitch = do
      hasElse <- rand 2
      n <- rand 3
      arms <- mapM (const (arm inner {switches = s : switches ctx})) [0 .. n]
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

-- | The request the generated program sends: unit 0, lang 3, the
-- parameter count, the rows in pre-order.
request :: Prog -> Value
request p =
  tabledRequest
    "7.0.0"
    "flow.request"
    [ ("units", [[0, 3, toInteger (length [() | (_, f) <- pVars p, testBit f varParam])]])
    , ("stmts", [map toInteger [0, nSeq n, parent, nKind n, nFlags n, nAux n] | (parent, n) <- walk (-1) (pRoots p)])
    , ("vars", [map toInteger [0, v, decl, flags] | (v, (decl, flags)) <- zip [0 ..] (pVars p)])
    , ("uses", [map toInteger [0, nSeq n, v, m] | (_, n) <- walk (-1) (pRoots p), (v, m) <- nAcc n])
    ]

-- | Every node with its parent, in pre-order (ascending seq).
walk :: Int -> [Node] -> [(Int, Node)]
walk parent = concatMap (\n -> (parent, n) : walk (nSeq n) (nKids n))
