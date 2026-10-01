-- | Family 1, a T3 pair (plan v2.31 step 6, design booklet §6.3;
-- step-6 ruling 4, merge generation 2): the two trees aligned by an
-- optimal tree-edit mapping (CE.Clone.Ted.tedMapping), narrowed
-- top-down — a mapped pair is kept only when its parents are a kept
-- pair, starting from the two roots (roots not mapped to each other:
-- no skeleton, the whole pair is one hole). A kept pair opens a node
-- hole when its keys differ (a relabel) or its own anonymous tokens do
-- (an operator, a keyword: class other — ruling R2); a kept pair that
-- is an expression on both sides with a kept child pair relabelled at
-- a leaf of class other is one widened expression hole over both
-- subtrees, absorbing every hole inside (ruling R3; CE.Merge.Widen).
-- Between two neighbouring kept child pairs of a kept pair — and
-- before the first, and after the last — the unkept child subtrees on
-- each side form a gap; a gap whose two forests are node for node the
-- same (atom and relative lld) joins the skeleton, any other is a gap
-- hole whose value on each side is that side's forest (an empty side,
-- an empty value), a structural difference on any position (ruling
-- R4), keyed into member 0's postorder (ruling R5).
module CE.Merge.Mapped (mapped, mappedWith) where

import CE.Clone.Ted (tedMapping)
import CE.Merge.Cost (slotOther, slotStatement)
import CE.Merge.Tree
import CE.Merge.Widen (nodeHole, trigger, widenable, widened)
import Data.Maybe (maybeToList)
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
  | otherwise = case gap a b (rootOf a) [rootOf a] [rootOf b] of
      ([skel], holes) -> (skel, holes, 0)
      (_, holes) -> (SGap (0, 1, 0), holes, 0)

-- | A kept pair: widened whole, or its own atom (fixed or a node hole)
-- then its children — the gaps and the kept child pairs, interleaved
-- in order. The kept pair count counts a widened pair's kept pairs
-- too: the widened expression still stands on the skeleton's lines.
kept :: S.Set (Int, Int) -> MTree -> MTree -> Int -> Int -> (Skel, [Hole], Int)
kept m a b x y
  | widenable at && any (\(c, d) -> trigger [(a, c), (b, d)]) pairs = (SGap key, [widened key at], count)
  | otherwise = (SNode self (concat (weave (map fst gaps) [[s] | (s, _, _) <- subs])), holes, count)
 where
  at = [(a, x), (b, y)]
  key = (x, 1, 0)
  own = nodeHole key at
  self = maybe (Right (atomOf a x)) (const (Left key)) own
  pairs = [(c, d) | c <- kidsOf a x, d <- kidsOf b y, S.member (c, d) m]
  -- where a gap empty on member 0's side sits: just before the next
  -- kept child (its leftmost leaf), after the last at x itself
  anchors = map (lldAt a . fst) pairs <> [x]
  gaps = zipWith3 (gap a b) anchors (segments (kidsOf a x) (map fst pairs)) (segments (kidsOf b y) (map snd pairs))
  subs = [kept m a b c d | (c, d) <- pairs]
  count = 1 + sum [n | (_, _, n) <- subs]
  holes = maybeToList own <> concatMap snd gaps <> concat [h | (_, h, _) <- subs]

-- | Two runs of sibling subtrees between the same kept pairs, whose
-- member-0 place is `anchor` when member 0's run is empty (the next
-- kept child's leftmost leaf, or the parent after the last): nothing,
-- the skeleton's own copy, or one gap hole.
gap :: MTree -> MTree -> Int -> [Int] -> [Int] -> ([Skel], [Hole])
gap a b anchor fa fb
  | null fa && null fb = ([], [])
  | cellsOf a fa == cellsOf b fb = (map (plain a) fa, [])
  | otherwise = ([SGap key], [Hole key [ends fa, ends fb] [cellsOf a fa, cellsOf b fb] [map (textAt a) fa, map (textAt b) fb] slotOther across])
 where
  -- member 0's forest anchors at its first node; an empty member-0
  -- side sits at the anchor, before the anchor's own holes, the outer
  -- gap before the inner by member 1's first root
  key = case (fa, fb) of
    (r : _, _) -> (lldAt a r, 1, 0)
    (_, r : _) -> (anchor, 0, r)
    _ -> (anchor, 0, -1)
  -- a side's first and last root; an empty side (−1, −1)
  ends roots = case roots of
    (r : _) -> (r, last roots)
    [] -> (-1, -1)
  across = any ((== slotStatement) . slotAt a) (forestNodes a fa) || any ((== slotStatement) . slotAt b) (forestNodes b fb)

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
