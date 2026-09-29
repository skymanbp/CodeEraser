-- | Proof trees (design booklet §4.5): for an answer, the rule that
-- produced it and, under each positive body atom it read, the tree
-- of that fact — down to the fact rows the measuring side sent
-- (rule −1). One tree per answer, nodes numbered in pre-order, and a
-- global node budget: a tree that would not fit whole is left out
-- and counted, never cut in the middle, so every emitted tree is a
-- complete derivation.
module CE.Query.Proof (Node (..), Root (..), proofRows) where

import CE.Query.Eval (Prov)
import CE.Query.Eval.Join (Fact)
import CE.Query.Syntax (isIdb)
import qualified Data.Map.Strict as M

-- | A derived tuple as a proof node: the predicate and rule it stands
-- for, its values, and the facts its body read.
data Node = Node
  { nodePred :: Integer
  , nodeRule :: Integer
  , nodeTuple :: [Integer]
  , nodeFacts :: [Fact]
  }

-- | An answer's root: the goal it answers, its index among that
-- goal's answers, and the node itself.
data Root = Root
  { rootGoal :: Int
  , rootAnswer :: Int
  , rootNode :: Node
  }

-- | A tree's rows, and how many nodes it holds.
data Tree = Tree {treeRows :: [[Integer]], treeNodes :: Integer}

-- | `[goal, answer, node, parent, rule, pred, args…]` rows for the
-- roots that fit the budget, the count of roots left out, the nodes
-- emitted.
proofRows :: Prov -> Integer -> [Root] -> ([[Integer]], Integer, Integer)
proofRows prov budget = go budget [] 0 0
 where
  go _ acc skipped used [] = (concat (reverse acc), skipped, used)
  go left acc skipped used (r : rs)
    | treeNodes t <= left = go (left - treeNodes t) (treeRows t : acc) skipped (used + treeNodes t) rs
    | otherwise = go left acc (skipped + 1) used rs
   where
    t = tree prov r

-- | One answer's tree, numbered in pre-order.
tree :: Prov -> Root -> Tree
tree prov r = Tree rows (toInteger (length rows))
 where
  (rows, _) = expand prov [toInteger (rootGoal r), toInteger (rootAnswer r)] 0 (-1) (rootNode r)

-- | A node and its subtree; returns the rows and the next free node
-- number.
expand :: Prov -> [Integer] -> Integer -> Integer -> Node -> ([[Integer]], Integer)
expand prov prefix node parent (Node p rule args facts) = (row : concat children, next)
 where
  row = prefix <> [node, parent, rule, p] <> args
  (children, next) = foldl child ([], node + 1) facts
  child (acc, n) fact = (acc <> [rows], n')
   where
    (rows, n') = factTree prov prefix n node fact

-- | A read fact: derived (its own provenance expands) or sent
-- (a leaf with rule −1).
factTree :: Prov -> [Integer] -> Integer -> Integer -> Fact -> ([[Integer]], Integer)
factTree prov prefix node parent (p, t) = case (isIdb p, M.lookup (p, t) prov) of
  (True, Just (rule, facts)) -> expand prov prefix node parent (Node (toInteger p) (toInteger rule) t facts)
  _ -> ([prefix <> [node, parent, -1, toInteger p] <> t], node + 1)
