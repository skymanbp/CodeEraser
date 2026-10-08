-- | The seeded groups behind ReferenceMerge: T1/T2 groups of two to
-- four members over one shape — leaves drawn per member from value
-- pools (some pools spelled under two kinds, one pool whose members
-- differ only in whitespace), internal nodes whose own tokens differ
-- per member, a parent's position class now and then not every
-- member's — and T3 pairs, a tree and a mutation of it (relabels,
-- leaf and own-token changes, subtrees dropped and inserted, now and
-- then a second tree with nothing in common). Each node is written as
-- the measuring side describes it (§6.2): kind, leaf hash, position
-- class, own-token hash, token text; the request is the postorder
-- encoding of those trees, one group per request.
module ReferenceMergeGen (MGroup (..), Node (..), exactGroups, flatten, groupRequest, nearGroups, textOf) where

import Data.Aeson (Value)
import MergeCases (mergeRequest, treeValue)
import ReferenceFlowGen (G, S (..), rand, runG)

-- | A node: kind, leaf hash (0 inside), token text of a leaf, own
-- anonymous tokens' hash (0 on a leaf), position class, children.
data Node = Node {nLab :: Int, nLeaf, nTok, nOwn :: Integer, nSlot :: Int, nKids :: [Node]}

-- | A group: family (0 T1/T2, 1 T3), helper lines, and per member
-- its tree, its lines and its file in-degree.
data MGroup = MGroup {gFam, gHelper :: Integer, gMembers :: [(Node, Integer, Integer)]}

-- | A subtree's token text: a leaf's own, an internal node's a hash of
-- its own tokens and its children's texts in order.
textOf :: Node -> Integer
textOf n = case nKids n of
  [] -> nTok n
  ks -> foldl (\h x -> (h * 1000003 + x + 7) `mod` 2305843009213693951) (nOwn n + 1000000000000) (map textOf ks)

-- | Postorder, each node with its leftmost leaf's index.
flatten :: Node -> [(Node, Int)]
flatten = go 0
 where
  go start n = forest start (nKids n) <> [(n, start)]
  forest _ [] = []
  forest s (k : ks) = let xs = go s k in xs <> forest (s + length xs) ks

groupRequest :: MGroup -> Value
groupRequest grp =
  mergeRequest [[0, gFam grp, gHelper grp]] [[0, m, m, l, d] | (m, (_, l, d)) <- zip [0 ..] (gMembers grp)] (map (tree . fst3) (gMembers grp))
 where
  fst3 (t, _, _) = t
  tree t =
    let cells = flatten t
        col f = Just [f n | (n, _) <- cells]
     in treeValue [toInteger (nLab n) | (n, _) <- cells] [toInteger l | (_, l) <- cells] [col nLeaf, col (toInteger . nSlot), col nOwn, col textOf]

-- | A random shape of n nodes: node k hangs under a random earlier
-- node (preorder), children in arrival order; kinds 0..3, classes
-- drawn with expressions dominating, a leaf its hash 1..3.
shape :: Int -> G Node
shape n = do
  parents <- mapM rand [1 .. n - 1]
  decor <- mapM (const ((,,) <$> rand 4 <*> ((\r -> [1, 1, 1, 1, 3, 0, 2, 4] !! r) <$> rand 8) <*> rand 3)) [0 .. n - 1]
  let build v =
        let ks = [build k | (k, p) <- zip [1 ..] parents, p == v]
            (lab, slot, leaf) = decor !! v
            leafHash = if null ks then 1 + toInteger leaf else 0
         in Node lab leafHash leafHash 0 slot ks
  pure (build 0)

-- | Two hundred and fifty T1/T2 groups; every tenth is RICH — larger,
-- every node an expression, no own-token difference, twelve pools —
-- the groups that carry more parameters than a merge may take.
exactGroups :: [MGroup]
exactGroups = [runG (exact (k `mod` 10 == 0)) (S (k * 9001 + 5) 0 0 0) | k <- [1 .. 250]]

exact :: Bool -> G MGroup
exact rich = do
  t <- (if rich then expressive else id) <$> (shape . ((if rich then 16 else 4) +) =<< rand 15)
  k <- (2 +) <$> rand 3
  plan <- decorate rich t
  rows <- mapM (const ((,) <$> ((2 +) . toInteger <$> rand 30) <*> (toInteger <$> rand 3))) [1 .. k]
  helper <- toInteger <$> rand 3
  pure (MGroup 0 helper [(instance' plan m, l, d) | (m, (l, d)) <- zip [0 ..] rows])

-- | Per node, how members differ: a leaf drawn from pool p (spelled
-- under one kind or two), the whitespace pool, or fixed; an internal
-- node whose own tokens differ, whose class differs on member 1.
data Plan = Plan Node (Maybe (Int, Bool)) Bool Bool [Plan]

decorate :: Bool -> Node -> G Plan
decorate rich n = do
  draw <- rand 9
  pool <- rand (if rich then 12 else 8)
  twoKinds <- (== 0) <$> rand 3
  ownDiff <- (&& not rich) . (== 0) <$> rand 5
  slotDiff <- (&& not rich) . (== 0) <$> rand 12
  ks <- mapM (decorate rich) (nKids n)
  let leafDraw
        | not (null (nKids n)) || draw < 4 = Nothing
        | draw == 8 = Just (-1, False)
        | otherwise = Just (pool, twoKinds)
  pure (Plan n leafDraw (ownDiff && not (null ks)) (slotDiff && not (null ks)) ks)

-- | Every node an expression.
expressive :: Node -> Node
expressive n = n {nSlot = 1, nKids = map expressive (nKids n)}

instance' :: Plan -> Int -> Node
instance' (Plan n leafDraw ownDiff slotDiff ks) m = case leafDraw of
  Just (-1, _) -> n {nLeaf = 5000 + mi, nTok = 4242}
  Just (p, twoKinds) ->
    let v = 1000 + 37 * toInteger p + mi
     in n {nLab = if twoKinds && odd m then 3 - nLab n else nLab n, nLeaf = v, nTok = v}
  Nothing ->
    n
      { nOwn = if ownDiff then 900 + mi else if null ks then 0 else 77
      , nSlot = if slotDiff && m == 1 then 0 else nSlot n
      , nKids = map (`instance'` m) ks
      }
 where
  mi = toInteger m

-- | Two hundred and fifty T3 pairs.
nearGroups :: [MGroup]
nearGroups = [runG near (S (k * 7717 + 13) 0 0 0) | k <- [1 .. 250]]

near :: G MGroup
near = do
  a <- owned <$> (shape . (5 +) =<< rand 16)
  fresh <- (== 0) <$> rand 10
  b <- if fresh then owned <$> (shape . (3 +) =<< rand 10) else mutate a
  rows <- mapM (const ((,) <$> ((2 +) . toInteger <$> rand 30) <*> (toInteger <$> rand 2))) [a, b]
  helper <- toInteger <$> rand 3
  pure (MGroup 1 helper [(t, l, d) | (t, (l, d)) <- zip [a, b] rows])

-- | Every internal node's own tokens one hash, a leaf none.
owned :: Node -> Node
owned t = t {nOwn = if null (nKids t) then 0 else 77, nKids = map owned (nKids t)}

-- | A mutation: each node now and then relabelled, its leaf, class or
-- own tokens changed, a child dropped, a fresh subtree inserted among
-- its children.
mutate :: Node -> G Node
mutate n = do
  r <- rand 14
  ks <- mapM mutate (nKids n)
  extra <- owned <$> (shape . (1 +) =<< rand 3)
  at <- rand (length ks + 1)
  let n' = case r of
        0 -> n {nLab = (nLab n + 1) `mod` 4}
        1 | null ks -> n {nLeaf = nLeaf n + 10, nTok = nTok n + 10}
        2 -> n {nSlot = 4}
        5 -> n {nOwn = 78}
        _ -> n
      ks'
        | r == 3 && not (null ks) = take at ks <> drop (at + 1) ks
        | r == 4 = take at ks <> [extra] <> drop at ks
        | otherwise = ks
      -- a node that lost its last child becomes a leaf of hash 1, one
      -- that gained a child an internal node of hash 0
      leafy v
        | not (null ks') = 0
        | null (nKids n) = v
        | otherwise = 1
  pure n' {nKids = ks', nLeaf = leafy (nLeaf n'), nTok = leafy (nTok n'), nOwn = if null ks' then 0 else max 77 (nOwn n')}
