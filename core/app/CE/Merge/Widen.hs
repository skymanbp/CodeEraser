-- | The holes both alignments open at a node (plan v2.31 step 6,
-- design booklet §6.3; merge generation 2, rulings R2 and R3): a
-- node whose key differs among the members is a relabel, a node whose
-- key is every member's but whose own anonymous tokens differ (an
-- operator, a keyword, a punctuation mark) is an own-token hole of
-- class other; and the one-level widening — a relabel at a leaf of
-- class other (a member name, a field, a string's content) whose
-- parent is an expression on every member opens one expression hole
-- over the parent's whole subtree instead, which absorbs every hole
-- inside it. CE.Merge.Align (T1/T2, many members, one shape) and
-- CE.Merge.Mapped (a T3 pair, the kept pairs of a mapping) read the
-- same four functions over (tree, node) per member.
module CE.Merge.Widen (nodeHole, trigger, widenable, widened) where

import CE.Merge.Cost (slotExpression, slotOther)
import CE.Merge.Tree

-- | The hole a node opens, if any: a relabel when some member's key
-- differs (its value the node alone, its text the subtree's token
-- text, its class member 0's), else an own-token hole when some
-- member's own tokens differ (class other: no parameter stands for an
-- operator).
nodeHole :: HoleKey -> [(MTree, Int)] -> Maybe Hole
nodeHole key at
  | differs keyOf = Just (hole (\t n -> [textAt t n]) (slotOf at))
  | differs ownAt = Just (hole (\t n -> [ownAt t n]) slotOther)
  | otherwise = Nothing
 where
  differs f = case [f t n | (t, n) <- at] of
    v : vs -> any (/= v) vs
    [] -> False
  hole texts slot = Hole key [(n, n) | (_, n) <- at] [[nodeCell t n] | (t, n) <- at] [texts t n | (t, n) <- at] slot False
  slotOf ((t, n) : _) = slotAt t n
  slotOf [] = slotOther

-- | A node that widens its parent: a leaf on every member, its key not
-- every member's, its token text not every member's (a whitespace-only
-- difference is no hole at all), its class (member 0's) other.
trigger :: [(MTree, Int)] -> Bool
trigger at = case at of
  (t0, n0) : rest ->
    all (\(t, n) -> isLeafAt t n) at
      && any (\(t, n) -> keyOf t n /= keyOf t0 n0) rest
      && any (\(t, n) -> textAt t n /= textAt t0 n0) rest
      && slotAt t0 n0 == slotOther
  [] -> False

-- | A parent the widening may cover: an expression on every member.
widenable :: [(MTree, Int)] -> Bool
widenable = all (\(t, n) -> n >= 0 && slotAt t n == slotExpression)

-- | The widened hole over each member's node's whole subtree: the
-- subtree as its value and its token text as its text, class
-- expression.
widened :: HoleKey -> [(MTree, Int)] -> Hole
widened key at = Hole key [(n, n) | (_, n) <- at] [cellsOf t [n] | (t, n) <- at] [[textAt t n] | (t, n) <- at] slotExpression False
