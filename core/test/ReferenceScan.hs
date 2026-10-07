-- | A second spelling of scan/1's complexity derivation (plan v2.33,
-- reference evaluators): cyclomatic complexity, cognitive complexity
-- and the maximum nesting depth computed on a unit's STRUCTURE TREE
-- (ReferenceScanGen) by the SonarSource whitepaper v1.7 rules as
-- CE.Scan.Complexity's header states them, with the nesting level an
-- explicit argument of the walk — where the shipped fold rebuilds
-- each level from its events' parent links, positions and class bits.
-- The recursion increment comes from its own closure of the call
-- relation (pairs squared to a fixpoint), not from Data.Graph and
-- not from ReferenceGraph's seeded reachability.
--
-- The rules, read off the tree: an if, a loop, a switch, a catch and
-- a ternary each pay one plus the level they stand at and nest what
-- they hold; their header (a condition, a loop clause, a switch
-- value) stands at their own level, and a ternary nests all three of
-- its operands, the table naming no body spot for it. An else, an
-- else-if, an elif and a loop's else pay one flat point; an else-if's
-- condition stands at the chain's level and its body one deeper. A
-- lambda pays nothing and nests its body. A labelled jump pays one. A
-- boolean expression pays one point each time its operator sequence,
-- read left to right through `&&` and `||` nodes, changes operator —
-- a parenthesis, a `??` or anything else ends the sequence; a let
-- chain is one sequence. try, finally and blocks are transparent.
-- Cyclomatic is one plus every decision: each if, else-if, loop,
-- decision case, catch and ternary, each `&&`, `||` and `??`, and N−1
-- per let chain of N operands. The depth is the deepest level any
-- nesting structure other than an else-if stands at, plus one.
module ReferenceScan (cyclic, measure, settled) where

import qualified Data.Set as S
import ReferenceScanGen

-- | What a subtree contributes: decisions, cognitive points, deepest
-- nesting reached.
data Tally = Tally Integer Integer Integer

instance Semigroup Tally where
  Tally a b c <> Tally a' b' c' = Tally (a + a') (b + b') (max c c')

instance Monoid Tally where
  mempty = Tally 0 0 0

-- | (cyclomatic, cognitive, maximum nesting) of one unit.
measure :: [Stmt] -> (Integer, Integer, Integer)
measure body = let Tally d c m = block 0 body in (1 + d, c, m)

block :: Integer -> [Stmt] -> Tally
block lvl = foldMap (statement lvl)

-- | A structure standing at `lvl` that is a decision, pays one plus
-- its level and reaches depth `lvl + 1`.
nests :: Integer -> Tally
nests lvl = Tally 1 (1 + lvl) (lvl + 1)

statement :: Integer -> Stmt -> Tally
statement lvl s = case s of
  Do e -> expression lvl e
  Block ss -> block lvl ss
  Jump labelled -> Tally 0 (if labelled then 1 else 0) 0
  Func _ -> mempty
  If c body t -> nests lvl <> expression lvl c <> block (lvl + 1) body <> tailOf lvl t
  Loop h body e -> nests lvl <> expression lvl h <> block (lvl + 1) body <> maybe mempty (elseBranch lvl) e
  Switch v cs -> Tally 0 (1 + lvl) (lvl + 1) <> expression lvl v <> mconcat [Tally (if d then 1 else 0) 0 0 <> block (lvl + 1) b | Case d b <- cs]
  Try body catches fin -> block lvl body <> mconcat [nests lvl <> block (lvl + 1) c | c <- catches] <> block lvl fin

-- | An else's flat point and its body one level in.
elseBranch :: Integer -> [Stmt] -> Tally
elseBranch lvl body = Tally 0 1 0 <> block (lvl + 1) body

tailOf :: Integer -> Tail -> Tally
tailOf lvl t = case t of
  NoElse -> mempty
  Else _ body -> elseBranch lvl body
  ElseIf c body t' -> Tally 1 1 0 <> expression lvl c <> block (lvl + 1) body <> tailOf lvl t'

expression :: Integer -> Expr -> Tally
expression lvl e = case e of
  Leaf -> mempty
  Paren x -> expression lvl x
  LetChain xs -> Tally (fromIntegral (length xs) - 1) 1 0 <> foldMap (expression lvl) xs
  Lambda body -> block (lvl + 1) body
  Ternary c a b -> nests lvl <> foldMap (expression (lvl + 1)) [c, a, b]
  Bin Coalesce l r -> Tally 1 0 0 <> expression lvl l <> expression lvl r
  Bin {} ->
    let (ops, rest) = sequenceOf e
     in Tally (fromIntegral (length ops)) (changes ops) 0 <> foldMap (expression lvl) rest

-- | The operators of a boolean sequence left to right, and the
-- operands that end it.
sequenceOf :: Expr -> ([Op], [Expr])
sequenceOf (Bin o l r)
  | o /= Coalesce =
      let (a, xs) = sequenceOf l
          (b, ys) = sequenceOf r
       in (a <> (o : b), xs <> ys)
sequenceOf x = ([], [x])

-- | One point for the first operator and one each time it changes.
changes :: [Op] -> Integer
changes [] = 0
changes (o : os) = 1 + fromIntegral (length (filter id (zipWith (/=) (o : os) os)))

-- | The units that reach themselves through one call or more: (u, u)
-- in the transitive closure of the call relation.
cyclic :: [(Int, Int)] -> S.Set Int
cyclic arcs = S.fromList [u | (u, v) <- S.toList (close (S.fromList arcs)), u == v]
 where
  close r =
    let r' = S.union r (S.fromList [(a, d) | (a, b) <- S.toList r, (c, d) <- S.toList r, b == c])
     in if S.size r' == S.size r then r else close r'

-- | What scan/1 must echo for a program: `derived` — [row, value] for
-- every row, cognitive values after the recursion increment — and
-- `cocBumped`, the raised cognitive rows.
settled :: Program -> ([[Integer]], [[Integer]])
settled p = (concat [triple i u | (i, u) <- zip [0 ..] us], [[3 * i + 1, coc u + 1] | (i, u) <- zip [0 ..] us, S.member (fromInteger i) looped])
 where
  us = units p
  looped = cyclic (pArcs p)
  coc u = let (_, c, _) = measure u in c
  triple i u =
    let (cc, c, depth) = measure u
        bump = if S.member (fromInteger i) looped then 1 else 0
     in [[3 * i, cc], [3 * i + 1, c + bump], [3 * i + 2, depth]]
