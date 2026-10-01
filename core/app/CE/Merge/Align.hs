-- | Family 0, a T1/T2 group (plan v2.31 step 6, design booklet §6.3;
-- step-6 rulings 2, 3 and 5, merge generation 2): the measuring side
-- promised the members are one token run, so their trees must be
-- isomorphic — every member's lld column equal to member 0's (one
-- shape, one node count) and every internal node's key equal (T2
-- normalises identifiers and literals to one token, so the LEAVES may
-- differ, and only they). A node opens a hole where its key differs
-- (a leaf relabel) or its own anonymous tokens do (an operator, a
-- keyword: ruling R2); a leaf relabel of class other under an
-- expression parent widens to the parent's subtree (ruling R3), the
-- outermost widened parent absorbing everything inside it. The
-- skeleton is member 0's tree with those places left open.
module CE.Merge.Align (align, isomorphic) where

import CE.Merge.Tree
import CE.Merge.Widen (nodeHole, trigger, widenable, widened)
import qualified Data.IntSet as IS
import Data.Maybe (mapMaybe)

-- | One shape, and every internal node's key member 0's. The internal
-- LEAF hash is held too, not just the label: the measuring side sends
-- 0 there (§6.2), and a member whose internal hash differed would not
-- rebuild from member 0's skeleton — so it is refused as not
-- isomorphic rather than silently merged. The own and text columns
-- are not read: an own-token difference is a hole, not a second shape.
isomorphic :: [MTree] -> Bool
isomorphic [] = True
isomorphic (t0 : ts) = all same ts
 where
  same t = sizeOf t == sizeOf t0 && all (at t) [0 .. rootOf t0]
  at t i = lldAt t i == lldAt t0 i && (isLeafAt t0 i || keyOf t i == keyOf t0 i)

-- | The skeleton and the holes of an isomorphic group, anchored at
-- member 0's nodes (every member's node the same index).
align :: [MTree] -> (Skel, [Hole])
align [] = (SGap (0, 1, 0), [])
align ts@(t0 : _) = (build (rootOf t0), widenedHoles <> filter (not . covered) raw)
 where
  at i = [(t, i) | t <- ts]
  raw = mapMaybe (\i -> nodeHole (i, 1, 0) (at i)) [0 .. rootOf t0]
  wide = IS.fromList [p | i <- [0 .. rootOf t0], isLeafAt t0 i, trigger (at i), let p = parentOf t0 i, p >= 0, widenable (at p)]
  -- the outermost widened parents: none inside another's subtree
  outer = IS.filter (\p -> not (any (\q -> q /= p && lldAt t0 q <= p && p < q) (IS.toList wide))) wide
  inside i = any (\p -> lldAt t0 p <= i && i <= p) (IS.toList outer)
  covered Hole {hKey = (i, _, _)} = inside i
  widenedHoles = [widened (p, 1, 0) (at p) | p <- IS.toList outer]
  open = IS.fromList [i | Hole {hKey = (i, _, _)} <- raw]
  build i
    | i `IS.member` outer = SGap (i, 1, 0)
    | i `IS.member` open = SNode (Left (i, 1, 0)) (map build (kidsOf t0 i))
    | otherwise = SNode (Right (atomOf t0 i)) (map build (kidsOf t0 i))
