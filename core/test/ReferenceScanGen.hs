-- | The seeded generator behind ReferenceScan: small units written as
-- STRUCTURE trees — if / else / else-if chains, loops (a Python
-- loop-else among them), switches whose cases are or are not
-- decisions, try / catch / finally, labelled and bare jumps, lambdas,
-- ternaries, boolean chains with mixed operators, `??`, let chains,
-- nested named functions that are units of their own — and the scan/1
-- request each program sends, written the way the measuring side
-- states a tree (cli/src/scan/metrics/events.rs): the tree lowered to
-- a concrete node shape under one of three table families — CLike
-- (Rust, C, TypeScript: an `else_clause` flat node), GoLike (Go, Java:
-- the next if or the else block in the if's `alternative` field),
-- PyLike (Python: `elif_clause` siblings) — and `emit` walking it as
-- `Emitter::visit` does: a transparent node passes its frame and
-- position through; a classified node takes the next seq, the nearest
-- classified ancestor as parent, DIRECT / IN_ALT from its tree edge,
-- and gives its children their body / header position. Nothing here
-- computes a number; ReferenceScan reads the TREE.
module ReferenceScanGen (Case (..), Expr (..), Op (..), Program (..), Stmt (..), Style (..), Tail (..), callGraphs, emit, programs, request, units) where

import Data.Aeson (Value, toJSON)
import Data.Bits ((.|.))
import Data.List (nub, sort)
import ReferenceFlowGen (G, S (..), rand, runG)
import WireHarness (rowsRequest, setKey)

data Style = CLike | GoLike | PyLike deriving (Eq, Show)

data Op = And | Or | Coalesce deriving (Eq, Show)

data Expr = Leaf | Bin Op Expr Expr | Paren Expr | LetChain [Expr] | Lambda [Stmt] | Ternary Expr Expr Expr

data Stmt
  = Do Expr
  | If Expr [Stmt] Tail
  | Loop Expr [Stmt] (Maybe [Stmt])
  | Switch Expr [Case]
  | Try [Stmt] [[Stmt]] [Stmt]
  | Jump Bool
  | Block [Stmt]
  | Func [Stmt]

-- | What follows an if: nothing, an else (braced, or one bare
-- statement), or an else-if with its own tail.
data Tail = NoElse | Else Bool [Stmt] | ElseIf Expr [Stmt] Tail

-- | A case: whether the table counts it as a decision, its body.
data Case = Case Bool [Stmt]

-- | A program: its encoding, its top-level units, its calls between
-- units (indices into `units`).
data Program = Program {pStyle :: Style, pTop :: [[Stmt]], pArcs :: [(Int, Int)]}

-- | Every unit of a program, nested functions after their host, in
-- pre-order: the order their row triples take.
units :: Program -> [[Stmt]]
units = concatMap unfold . pTop
 where
  unfold body = body : concatMap inner body
  inner s = case s of
    Func b -> unfold b
    If _ b t -> concatMap inner b <> tailInner t
    Loop _ b e -> concatMap inner (b <> maybe [] id e)
    Switch _ cs -> concat [concatMap inner b | Case _ b <- cs]
    Try b cs f -> concatMap inner (b <> concat cs <> f)
    Block b -> concatMap inner b
    _ -> []
  tailInner t = case t of
    Else _ b -> concatMap inner b
    ElseIf _ b t' -> concatMap inner b <> tailInner t'
    NoElse -> []

-- | A concrete node: class bits (0 = transparent), an if's alt class,
-- a chain's operand count, a logic root's operators, whether it splits
-- its children, each child with (in a body spot, first alternative).
data T = T {tBits, tAlt, tAux :: Integer, tOps :: [Integer], tSplit :: Bool, tKids :: [(Bool, Bool, T)]}

nestB, flatB, nestOnlyB, jumpB, ifB, chainB, ccB, ccOpB, rootB :: Integer
nestB = 1
flatB = 2
nestOnlyB = 4
jumpB = 8
ifB = 16
chainB = 32
ccB = 64
ccOpB = 128
rootB = 256

plain :: [T] -> T
plain ks = T 0 0 0 [] False [(False, False, k) | k <- ks]

classed :: Integer -> [(Bool, Bool, T)] -> T
classed bits = T bits 0 0 [] True

flatT :: T -> T
flatT k = classed flatB [(True, False, k)]

block :: Style -> [Stmt] -> T
block st = plain . map (lowerS st)

lowerS :: Style -> Stmt -> T
lowerS st s = case s of
  Do e -> plain [lowerE st False e]
  Block ss -> block st ss
  Jump labelled -> if labelled then T jumpB 0 0 [] False [] else plain []
  Func _ -> plain []
  If c b t -> lowerIf st c b t
  Loop h b e -> classed (nestB .|. ccB) ([(False, False, lowerE st False h), (True, False, block st b)] <> [(True, True, flatT (block st x)) | Just x <- [e]])
  Switch v cs -> classed nestB ((False, False, lowerE st False v) : cases st cs)
  Try b cs f -> plain ([block st b] <> [classed (nestB .|. ccB) [(False, False, plain []), (True, False, block st c)] | c <- cs] <> [block st f])

-- | Go states each case as a node holding its statements; the other
-- families a body whose case labels are leaves beside them.
cases :: Style -> [Case] -> [(Bool, Bool, T)]
cases GoLike cs = [(True, False, (if d then T ccB 0 0 [] False else \ks -> plain [k | (_, _, k) <- ks]) [(False, False, lowerS GoLike x) | x <- b]) | Case d b <- cs]
cases st cs = [(True, False, plain (concat [[if d then T ccB 0 0 [] False [] else plain []] <> map (lowerS st) b | Case d b <- cs]))]

lowerIf :: Style -> Expr -> [Stmt] -> Tail -> T
lowerIf st c b t = (classed (nestB .|. ifB .|. ccB) (head2 <> rest)) {tAlt = alt}
 where
  head2 = [(False, False, lowerE st False c), (True, False, block st b)]
  (alt, rest) = case st of
    PyLike -> (if null (pyTail True t) then 0 else 2, pyTail True t)
    CLike -> case t of
      NoElse -> (0, [])
      Else braced x -> (2, [(True, True, flatT (elseBody braced x))])
      ElseIf c' b' t' -> (2, [(True, True, flatT (lowerIf st c' b' t'))])
    GoLike -> case t of
      NoElse -> (0, [])
      Else braced x -> (3, [(True, True, elseBody braced x)])
      ElseIf c' b' t' -> (1, [(True, True, lowerIf st c' b' t')])
  -- one bare statement, or a braced body
  elseBody braced x = case x of
    [one] | not braced -> lowerS st one
    _ -> block st x
  pyTail first tl = case tl of
    NoElse -> []
    Else _ x -> [(True, first, flatT (block st x))]
    ElseIf c' b' t' -> (True, first, classed (flatB .|. ccB) [(False, False, lowerE st False c'), (True, False, block st b')]) : pyTail False t'

-- | An expression; the flag says the tree parent is an `&&`/`||`
-- node, which is what keeps an operand from rooting a chain.
lowerE :: Style -> Bool -> Expr -> T
lowerE st underLogic e = case e of
  Leaf -> plain []
  Paren x -> plain [lowerE st False x]
  Bin op l r ->
    let logic = op /= Coalesce
        root = logic && not underLogic
     in T (ccOpB .|. (if root then rootB else 0)) 0 0 (if root then inOrder e else []) False [(False, False, lowerE st logic l), (False, False, lowerE st logic r)]
  LetChain xs -> T chainB 0 (fromIntegral (length xs)) [] False [(False, False, lowerE st False x) | x <- xs]
  Lambda ss -> T nestOnlyB 0 0 [] False [(False, False, block st ss)]
  Ternary c a b -> classed (nestB .|. ccB) [(True, False, lowerE st False x) | x <- [c, a, b]]
 where
  inOrder (Bin o l r) | o /= Coalesce = inOrder l <> [if o == And then 0 else 1] <> inOrder r
  inOrder _ = []

-- | One unit's events, `[seq, parent, pos, flags, aux, op…]`.
emit :: Style -> [Stmt] -> [[Integer]]
emit st body = reverse (fst (visit (-1) 0 512 (block st body) ([], 0)))

-- | `edge` carries the node's relation bits: DIRECT (512) when its tree
-- parent is the frame's node, IN_ALT (1024) when it is that parent's
-- first `alternative` child.
visit :: Integer -> Integer -> Integer -> T -> ([[Integer]], Integer) -> ([[Integer]], Integer)
visit frame pos edge n (acc, next)
  | tBits n == 0 = foldl (\st (_, alt, k) -> visit frame pos (inAlt alt) k st) (acc, next) (tKids n)
  | otherwise = foldl (\st (body, alt, k) -> visit next (if tSplit n && body then 1 else 0) (512 .|. inAlt alt) k st) (row : acc, next + 1) (tKids n)
 where
  inAlt alt = if alt then 1024 else 0
  row = [next, frame, pos, tBits n .|. edge .|. tAlt n * 2048, tAux n] <> tOps n

-- | The scan/1 request of a program: a [3,4,5] row triple per unit,
-- the events keyed at each unit's cognitive row, the calls between
-- units as cognitive-row pairs, ascending.
request :: Program -> Value
request p = setKey "callEdges" (toJSON calls) (setKey "events" (toJSON evs) (rowsRequest "7.0.0" "scan.request" rows))
 where
  us = units p
  rows = concat [[[3, 0], [4, 0], [5, 0]] | _ <- us]
  cog i = 3 * toInteger i + 1
  evs = concat [map (cog i :) (emit (pStyle p) u) | (i, u) <- zip [0 :: Int ..] us]
  calls = [[cog a, cog b] | (a, b) <- sort (nub (pArcs p))] :: [[Integer]]

-- | Twelve hundred seeded programs, the three encodings in turn.
programs :: [Program]
programs = [runG (program n) (S (n * 6151 + 29) 0 0 0) | n <- [1 .. 1200]]

program :: Int -> G Program
program n = do
  let st = [CLike, GoLike, PyLike] !! (n `mod` 3)
  k <- rand 3
  top <- mapM (const (stmts st 3)) [0 .. k]
  let count = length (units (Program st top []))
  arcs <- pairs count (count + 1)
  pure (Program st top arcs)

pairs :: Int -> Int -> G [(Int, Int)]
pairs count m = do
  j <- rand (m + 1)
  mapM (const ((,) <$> rand count <*> rand count)) [1 .. j]

-- | Three hundred call graphs over one to six straight units each, the
-- arcs dense: the recursion increment's own leg.
callGraphs :: [Program]
callGraphs = [runG (graph n) (S (n * 4099 + 3) 0 0 0) | n <- [1 .. 300]]
 where
  graph n = do
    k <- rand 6
    top <- mapM (const (stmts CLike 1)) [0 .. k]
    arcs <- pairs (k + 1) ((k + 1) * (k + 1) `div` 2 + n `mod` 2)
    pure (Program CLike top arcs)

stmts :: Style -> Int -> G [Stmt]
stmts st d = do
  n <- rand (if d <= 0 then 2 else 4)
  mapM (const (stmt st d)) [1 .. n]

stmt :: Style -> Int -> G Stmt
stmt st d
  | d <= 0 = rand 3 >>= \k -> if k == 0 then pure (Jump True) else Do <$> (if k == 1 then chain st 0 else pure Leaf)
  | otherwise = rand 13 >>= \k -> if k < 5 then compound st d k else simple st d k

-- | The structures that hold a body.
compound :: Style -> Int -> Int -> G Stmt
compound st d k = case k of
  0 -> If <$> cond st d <*> body <*> tailOf st d 3
  1 -> If <$> cond st d <*> body <*> tailOf st d 1
  2 -> Loop <$> cond st d <*> body <*> (if st == PyLike then rand 2 >>= \c -> if c == 0 then pure Nothing else Just <$> body else pure Nothing)
  3 -> Switch <$> expr st (d - 1) <*> (rand 4 >>= \c -> mapM (const (Case <$> ((/= 0) <$> rand 4) <*> body)) [0 .. c])
  _ -> Try <$> body <*> (rand 2 >>= \c -> mapM (const body) [0 .. c]) <*> (rand 2 >>= \f -> if f == 0 then pure [] else body)
 where
  body = stmts st (d - 1)

simple :: Style -> Int -> Int -> G Stmt
simple st d k = case k of
  5 -> Jump . (== 0) <$> rand 2
  6 -> Block <$> stmts st (d - 1)
  7 -> Func <$> stmts st (d - 1)
  _ -> Do <$> expr st (d - 1)

-- | An if's tail, at most `len` links long; PyLike's else is always a
-- clause, the bare form only where the table family allows it.
tailOf :: Style -> Int -> Int -> G Tail
tailOf st d len = do
  k <- rand 4
  case k of
    0 -> pure NoElse
    1 | len > 0 -> ElseIf <$> cond st d <*> stmts st (d - 1) <*> tailOf st d (len - 1)
    2 | st /= PyLike -> do
      s <- stmt st (d - 1)
      pure (if isIf s then Else True [s] else Else False [s])
    _ -> Else True <$> stmts st (d - 1)
 where
  isIf (If {}) = True
  isIf _ = False

-- | A condition: mostly a boolean chain, sometimes a let chain (the
-- CLike family's Rust), a ternary or a lambda-bearing expression.
cond :: Style -> Int -> G Expr
cond st d = do
  k <- rand 6
  case k of
    0 | st == CLike -> LetChain <$> (rand 3 >>= \m -> mapM (const operand) [0 .. m + 1])
    1 -> expr st (d - 1)
    _ -> chain st d
 where
  operand = rand 3 >>= \c -> if c == 0 then Paren <$> chain st 0 else pure Leaf

expr :: Style -> Int -> G Expr
expr st d = do
  k <- rand (if d <= 0 then 3 else 6)
  case k of
    0 -> pure Leaf
    1 -> chain st d
    2 -> Bin Coalesce <$> chain st 0 <*> (Paren <$> chain st 0)
    3 -> Lambda <$> stmts st (d - 1)
    4 -> Ternary <$> chain st (d - 1) <*> expr st (d - 1) <*> expr st (d - 1)
    _ -> Paren <$> expr st (d - 1)

-- | A random binary tree of one to five `&&`/`||` operands in source
-- order; an operand is a leaf or, below the top, a parenthesised
-- expression that starts a sequence of its own.
chain :: Style -> Int -> G Expr
chain st d = rand 5 >>= go
 where
  go 0 = rand 4 >>= \c -> if c == 0 && d > 0 then Paren <$> expr st (d - 1) else pure Leaf
  go m = do
    split <- rand m
    op <- (\c -> if c == 0 then And else Or) <$> rand 2
    Bin op <$> go split <*> go (m - 1 - split)
