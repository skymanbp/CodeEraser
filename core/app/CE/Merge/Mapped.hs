-- | Family 1, a T3 pair (plan v2.31 step 6, design booklet §6.3;
-- step-6 ruling 4): the two trees aligned by an optimal tree-edit
-- mapping (CE.Clone.Ted.tedMapping), narrowed top-down — a mapped
-- pair is kept only when its parents are a kept pair, starting from
-- the two roots (roots not mapped to each other: no skeleton, the
-- whole pair is one hole). A kept pair whose keys differ is a leaf
-- hole (the mapping's relabel). Between two neighbouring kept child
-- pairs of a kept pair — and before the first, and after the last —
-- the unkept child subtrees on each side form a gap; a gap whose two
-- forests are node for node the same (key and relative lld) joins
-- the skeleton, any other is a gap hole whose value on each side is
-- that side's forest (an empty side, an empty value).
module CE.Merge.Mapped (mapped, mappedWith) where

import CE.Clone.Ted (tedMapping)
import CE.Merge.Cost (slotStatement)
import CE.Merge.Tree
import qualified Data.Set as S

-- | The skeleton, the holes and the kept pair count of a T3 pair.
mapped :: MTree -> MTree -> (Skel, [Hole], Int)
mapped a b = mappedWith (S.fromList (snd (tedMapping (mBase a) (mBase b)))) a b

-- | The same over any valid Tai mapping (the battery feeds it a
-- non-optimal one: the optimal mapping never leaves two equal
-- forests unmapped side by side, so the join-the-skeleton branch is
-- probed through this door).
mappedWith :: S.Set (Int, Int) -> MTree -> MTree -> (Skel, [Hole], Int)
mappedWith m a b
  | S.member (rootOf a, rootOf b) m = kept m a b (rootOf a) (rootOf b)
  | otherwise = case gap a b [rootOf a] [rootOf b] of
      ([skel], holes) -> (skel, holes, 0)
      (_, holes) -> (SGap (0, 0), holes, 0)

-- | A kept pair: its own key (fixed or a leaf hole), then its
-- children — the gaps and the kept child pairs, interleaved in
-- order.
kept :: S.Set (Int, Int) -> MTree -> MTree -> Int -> Int -> (Skel, [Hole], Int)
kept m a b x y = (SNode self (concat (weave (map fst gaps) [[s] | (s, _, _) <- subs])), holes, 1 + sum [n | (_, _, n) <- subs])
 where
  same = keyOf a x == keyOf b y
  self = if same then Right (keyOf a x) else Left (0, x)
  -- a relabelled node's value is the node alone, not its subtree
  own = [Hole (0, x) [(x, x), (y, y)] [[nodeCell a x], [nodeCell b y]] (slotAt a x) False | not same]
  pairs = [(c, d) | c <- kidsOf a x, d <- kidsOf b y, S.member (c, d) m]
  gaps = zipWith (gap a b) (segments (kidsOf a x) (map fst pairs)) (segments (kidsOf b y) (map snd pairs))
  subs = [kept m a b c d | (c, d) <- pairs]
  holes = own <> concatMap snd gaps <> concat [h | (_, h, _) <- subs]

-- | Two runs of sibling subtrees between the same kept pairs: nothing,
-- the skeleton's own copy, or one gap hole.
gap :: MTree -> MTree -> [Int] -> [Int] -> ([Skel], [Hole])
gap a b fa fb
  | null fa && null fb = ([], [])
  | cellsOf a fa == cellsOf b fb = (map (plain a) fa, [])
  | otherwise = ([SGap key], [Hole key [ends fa, ends fb] [cellsOf a fa, cellsOf b fb] slot across])
 where
  (nodesA, nodesB) = (forestNodes a fa, forestNodes b fb)
  key = case (nodesA, nodesB) of
    (n : _, _) -> (0, n)
    (_, n : _) -> (1, n)
    _ -> (1, -1)
  -- a side's first and last root; an empty side (−1, −1)
  ends roots = case roots of
    (r : _) -> (r, last roots)
    [] -> (-1, -1)
  slot = case (fa, fb) of
    (r : _, _) -> slotAt a r
    (_, r : _) -> slotAt b r
    _ -> slotStatement
  across = any ((== slotStatement) . slotAt a) nodesA || any ((== slotStatement) . slotAt b) nodesB

-- | The runs between cut points: k cuts make k + 1 runs (possibly
-- empty), the cuts themselves left out.
segments :: [Int] -> [Int] -> [[Int]]
segments xs [] = [xs]
segments xs (c : cs) = before : segments (drop 1 rest) cs
 where
  (before, rest) = break (== c) xs

-- | g0, s1, g1, s2, … : the gaps around the kept child pairs.
weave :: [a] -> [a] -> [a]
weave (g : gs) (s : ss) = g : s : weave gs ss
weave gs [] = gs
weave [] ss = ss
