-- | The three complexity numbers of ONE unit — cognitive complexity
-- per the SonarSource whitepaper v1.7, cyclomatic complexity per the
-- gocyclo definition, and the maximum nesting depth — folded from its
-- structural event stream (plan v2.30 step 7b ③). The Rust side
-- classifies each node by its LangSpec tables and states the tree
-- (`cli/src/scan/metrics/events.rs`); every rule that turns those
-- facts into a number lives here and nowhere else.
--
-- The whitepaper rules, as the events carry them: (1) a structure
-- that nests pays 1 plus its nesting level and raises the level for
-- its body, never its header (user ruling 2026-09-24, register D31);
-- (2) an `else` / `elif` / `else if` is a flat +1 with no penalty —
-- an if hanging off a flat clause, or off its parent if's
-- `alternative` field, is that else-if, and the clause that wraps it
-- yields its own point to it; a plain else hung off the field (a Go
-- block, a Java statement) pays its +1 on the if; (3) a lambda raises
-- the level and pays nothing; (4) a boolean chain pays one point per
-- run of like operators — `a && b && c` one, `a || b && c || d` three;
-- a `let_chain` is one run of anonymous `&&`; (5) a labelled jump is
-- a fundamental +1. Cyclomatic is 1 plus one per decision kind, plus
-- N−1 per chain of N operands, plus one per short-circuit operator
-- node — the chain and operator terms read the same events. What is
-- NOT here: the recursion increment, which needs a call relation and
-- is charged by CE.Scan.Cycles on the rows this module derives.
module CE.Scan.Complexity (Event (..), eventOf, fold, runs) where

import Data.Bits (shiftR, testBit, (.&.))
import Data.List (group)
-- lazy on purpose: a level reads its parent's level through the map
import qualified Data.Map as M

-- | One classified node: its pre-order index, its nearest classified
-- ancestor's (−1 = the unit itself), its position under that
-- ancestor (1 body, 0 header), the flag bits, a chain's operand count
-- and a logic root's operator ids in source order.
data Event = Event
  { evSeq :: Int
  , evParent :: Int
  , evPos :: Int
  , evFlags :: Integer
  , evAux :: Integer
  , evOps :: [Integer]
  }

-- | A validated events row minus its leading row index.
eventOf :: [Integer] -> Event
eventOf (s : p : pos : flags : aux : ops) =
  Event (fromInteger s) (fromInteger p) (fromInteger pos) flags aux ops
eventOf _ = error "event shape enforced by violation"

-- | The flag bits (events.rs names the same ones).
nesting, flat, nestOnly, labelledJump, ifKind, chain, ccKind, ccOp, logicRoot, direct, inAlt :: Int
nesting = 0
flat = 1
nestOnly = 2
labelledJump = 3
ifKind = 4
chain = 5
ccKind = 6
ccOp = 7
logicRoot = 8
direct = 9
inAlt = 10

has :: Int -> Event -> Bool
has b e = testBit (evFlags e) b

-- | Bits 11–12: the class of an if's first `alternative` child — 0
-- none, 1 an if kind, 2 a flat kind, 3 anything else (a plain else).
altClass :: Event -> Integer
altClass e = (evFlags e `shiftR` 11) .&. 3

-- | The class a node is walked as: the table order the measuring
-- walker kept (nesting first, then flat, then nest-only).
data Class = Nesting | Flat | NestOnly | Other deriving (Eq)

classOf :: Event -> Class
classOf e
  | has nesting e = Nesting
  | has flat e = Flat
  | has nestOnly e = NestOnly
  | otherwise = Other

-- | (cyclomatic, cognitive, maximum nesting) of one unit; the empty
-- stream — a straight-line unit — reads (1, 0, 0).
fold :: [Event] -> (Integer, Integer, Integer)
fold evs =
  ( 1 + sum (map cyclomatic evs)
  , sum (map cognitive evs)
  , maximum (0 : map depth evs)
  )
 where
  byId = M.fromList [(evSeq e, e) | e <- evs]
  parentOf e = M.lookup (evParent e) byId
  -- the nesting level each node is visited at: 0 under the unit,
  -- else its parent's level moved by the parent's class and the
  -- node's position — the walker's argument, reconstructed
  levels = M.fromList [(evSeq e, level e) | e <- evs]
  levelOf e = levels M.! evSeq e
  level e = case parentOf e of
    Nothing -> 0
    Just p -> childLevel p (levelOf p) (evPos e)
  childLevel p n pos = case classOf p of
    Nesting
      | elseIf p -> if pos == 1 then n else max 0 (n - 1)
      | otherwise -> if pos == 1 then n + 1 else n
    Flat -> if pos == 1 then n else max 0 (n - 1)
    NestOnly -> n + 1
    Other -> n
  -- rule (2): an if directly under a flat clause, or the first
  -- `alternative` child of its parent if
  elseIf e =
    has ifKind e
      && has direct e
      && maybe False (\p -> has flat p || (has ifKind p && has inAlt e)) (parentOf e)
  cognitive e = operatorRuns e + chainRun e + structural e
  operatorRuns e = if has logicRoot e then runs (evOps e) else 0
  chainRun e = if has chain e then 1 else 0
  structural e = case classOf e of
    Nesting -> elseBonus e + (if elseIf e then 1 else 1 + levelOf e)
    -- a flat clause that wraps the next if yields its point to it
    Flat -> if any (wrapsIf e) evs then 0 else 1
    NestOnly -> 0
    Other -> if has labelledJump e then 1 else 0
  wrapsIf e c = evParent c == evSeq e && has direct c && has ifKind c
  elseBonus e = if has ifKind e && altClass e == 3 then 1 else 0
  depth e
    | classOf e == Nesting && not (elseIf e) = levelOf e + 1
    | otherwise = 0
  cyclomatic e
    | has ccKind e = 1
    | has chain e = max 0 (evAux e - 1)
    | has ccOp e = 1
    | otherwise = 0

-- | Runs of like operators in source order: each maximal run of one
-- operator id counts once.
runs :: [Integer] -> Integer
runs = toInteger . length . group
