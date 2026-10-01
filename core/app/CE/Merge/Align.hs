-- | Family 0, a T1/T2 group (plan v2.31 step 6, design booklet §6.3;
-- step-6 rulings 2, 3 and 5): the measuring side promised the
-- members are one token run, so their trees must be isomorphic —
-- every member's lld column equal to member 0's (one shape, one node
-- count) and every internal node's key equal (T2 normalises
-- identifiers and literals to one token, so the LEAVES may differ,
-- and only they). The holes are the leaf positions whose keys differ
-- among the members, in postorder; the skeleton is member 0's tree
-- with those leaves left open.
module CE.Merge.Align (align, isomorphic) where

import CE.Merge.Tree
import qualified Data.IntSet as IS

-- | One shape, and every internal node's key member 0's. The internal
-- LEAF hash is held too, not just the label: the measuring side sends
-- 0 there (§6.2), and a member whose internal hash differed would not
-- rebuild from member 0's skeleton — so it is refused as not
-- isomorphic rather than silently merged.
isomorphic :: [MTree] -> Bool
isomorphic [] = True
isomorphic (t0 : ts) = all same ts
 where
  same t = sizeOf t == sizeOf t0 && all (at t) [0 .. rootOf t0]
  at t i = lldAt t i == lldAt t0 i && (isLeafAt t0 i || keyOf t i == keyOf t0 i)

-- | The skeleton and the holes of an isomorphic group: a leaf hole
-- at every leaf whose key is not every member's, anchored at that
-- node in every member, its slot member 0's.
align :: [MTree] -> (Skel, [Hole])
align [] = (SGap (0, 0), [])
align ts@(t0 : _) = (build (rootOf t0), holes)
 where
  holes =
    [ Hole (0, i) (map (const (i, i)) ts) [[nodeCell t i] | t <- ts] (slotAt t0 i) False
    | i <- [0 .. rootOf t0]
    , isLeafAt t0 i
    , any (\t -> keyOf t i /= keyOf t0 i) ts
    ]
  open = IS.fromList [i | Hole {hKey = (_, i)} <- holes]
  build i
    | i `IS.member` open = SNode (Left (0, i)) []
    | otherwise = SNode (Right (keyOf t0 i)) (map build (kidsOf t0 i))
