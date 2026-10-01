-- | The seeded tree generator behind the merge battery (plan v2.31
-- step 6, design booklet §6.5): two hundred pairs of random trees of
-- 8 to 40 nodes over labels 0..3 for the mapping-TED legs, and two
-- hundred synthetic T1/T2 groups — one random tree, its leaves turned
-- into value vectors drawn from a small pool — whose parameter count
-- and feasibility are known by construction; and the battery's model
-- of the text column a tree carries when a case does not spell it.
-- Deterministic: the LCG and its monad are ReferenceFlowGen's, the
-- shape-to-lld walk is ReferenceTed's (no RNG inside a
-- byte-determinism contract).
module ReferenceTreeGen (Synth (..), synthGroups, textColumn, treePairs) where

import qualified Data.IntMap.Strict as IM
import qualified Data.Set as S
import ReferenceFlowGen (G, S (..), rand, runG)
import ReferenceTed (Rose (..), lldOf)

-- | A tree as its wire arrays (the ReferenceTed form).
type T = ([Int], [Int])

-- | One synthetic group: each member's (lab, lld, leaf, slot) columns,
-- the parameter count and the feasibility its construction implies.
data Synth = Synth {synthTrees :: [([Int], [Int], [Integer], [Int])], synthParams :: Int, synthFeasible :: Bool}

-- | A random shape of n nodes: node k (preorder) hangs under a random
-- earlier node, children in the order they arrive.
shape :: Int -> G Rose
shape n = do
  parents <- mapM rand [1 .. n - 1]
  let kids = IM.fromListWith (flip (<>)) [(p, [k]) | (k, p) <- zip [1 ..] parents]
      rose v = Rose (map rose (IM.findWithDefault [] v kids))
  pure (rose 0)

-- | A random tree of n nodes, labels below the bound.
tree :: Int -> Int -> G T
tree n labels = do
  s <- shape n
  labs <- mapM (const (rand labels)) [1 .. n]
  pure (labs, lldOf s)

-- | Two hundred seeded pairs, each tree 8 to 40 nodes, labels 0..3.
treePairs :: [(T, T)]
treePairs = [runG pair (S (k * 104729 + 3) 0 0 0) | k <- [1 .. 200]]
 where
  pair = do
    n1 <- (8 +) <$> rand 33
    n2 <- (8 +) <$> rand 33
    (,) <$> tree n1 4 <*> tree n2 4

-- | Two hundred seeded T1/T2 groups.
synthGroups :: [Synth]
synthGroups = [runG synth (S (k * 7907 + 11) 0 0 0) | k <- [1 .. 200]]

-- | The text column a case leaves unspelled: a leaf's text is its leaf
-- hash (one token, no whitespace), an internal node's a hash of its
-- subtree's (lab, leaf, relative lld) cells, offset past any leaf hash
-- the battery writes — two subtrees share a text exactly when they
-- share their cells, a model of a token stream that holds no
-- whitespace-only difference. A malformed tree (the refusal cases'
-- column lengths or llds) takes zeros: the contract refuses it first.
textColumn :: [Int] -> [Int] -> [Integer] -> [Integer]
textColumn labs llds leaves
  | length llds /= n || length leaves /= n || or [l < 0 || l > i | (i, l) <- zip [0 ..] llds] = map (const 0) labs
  | otherwise = map text [0 .. n - 1]
 where
  n = length labs
  text i
    | llds !! i == i = leaves !! i
    | otherwise = 1000000000000 + foldl' mix 7 (concat [[toInteger (labs !! j), leaves !! j, toInteger (llds !! j - llds !! i)] | j <- [llds !! i .. i]])
  mix h x = (h * 1000003 + x + 1) `mod` 2305843009213693951

-- | One tree of 5 to 20 nodes and 2 to 4 members; each leaf keeps one
-- hash for every member or takes a pool vector p (member m's hash
-- 100 (p + 1) + m, so no pool vector is constant). A drawn leaf of
-- class other under an expression parent widens to the parent (gen-2
-- ruling R3), the outermost widened parents absorbing every hole
-- inside them; a hole's text vector is its text per member (a leaf's
-- its hash, a widened parent's its subtree text), so the parameters
-- are the distinct text vectors of the holes left; feasible ⇔ every
-- hole at an expression or name slot (a widened one is an expression)
-- and at most six parameters.
synth :: G Synth
synth = do
  n <- (5 +) <$> rand 16
  (labs, llds) <- tree n 4
  slots <- mapM (const (slotOf <$> rand 8)) labs
  k <- (2 +) <$> rand 3
  pool <- (1 +) <$> rand 8
  draws <- mapM (\i -> if llds !! i == i then drawn pool else pure Nothing) [0 .. n - 1]
  base <- mapM (const ((1 +) . toInteger <$> rand 3)) labs
  let leafOf m i = case draws !! i of
        Just p -> toInteger (100 * (p + 1) + m)
        Nothing -> if llds !! i == i then base !! i else 0
      texts = [textColumn labs llds (map (leafOf m) [0 .. n - 1]) | m <- [0 .. k - 1]]
      vector i = [t !! i | t <- texts]
      parent i = take 1 [j | j <- [i + 1 .. n - 1], llds !! j <= i]
      drawnAt = [i | (i, Just _) <- zip [0 ..] draws]
      wide = [p | i <- drawnAt, slots !! i == 4, p <- parent i, slots !! p == 1]
      outer = [p | p <- wide, not (any (\q -> q /= p && llds !! q <= p && p < q) wide)]
      inside i = any (\p -> llds !! p <= i && i <= p) outer
      kept = [i | i <- drawnAt, not (inside i)]
      params = length (S.fromList (map vector (outer <> kept)))
      feasible = all (\i -> slots !! i == 1 || slots !! i == 3) kept && params <= 6
  pure (Synth [(labs, llds, map (leafOf m) [0 .. n - 1], slots) | m <- [0 .. k - 1]] params feasible)
 where
  drawn pool = do
    c <- rand 3
    if c == 0 then pure Nothing else Just <$> rand pool
  -- expression positions dominate real trees
  slotOf r = [1, 1, 1, 1, 3, 0, 2, 4] !! r
