-- | From a group's holes to its suggestion (plan v2.31 step 6, design
-- booklet §6.3; step-6 rulings 5–8): the holes in hole order, their
-- parameters (one per distinct value vector, numbered by first
-- appearance), the feasibility and its reason (the first infeasible
-- hole's, else too many parameters, else no line saved), the member
-- kept, the lines a
-- merge would save; and the rebuild a skeleton and a member's
-- values give back — the law the battery holds every suggestion to.
module CE.Merge.Holes (instantiate, paramsOf, skeletonOf, suggest) where

import CE.Merge.Align (align)
import CE.Merge.Cost
import CE.Merge.Mapped (mapped)
import CE.Merge.Tree
import Data.List (sortOn)
import qualified Data.Map.Strict as M

-- | A group's skeleton, its holes in hole order (ascending key —
-- member 0's anchors first, then the gaps empty on member 0's side)
-- and, for a T3 pair, the kept pair count.
skeletonOf :: Group -> (Skel, [Hole], Int)
skeletonOf g = case gTrees g of
  [a, b] | gFamily g == familyNear -> ordered (mapped a b)
  ts -> let (skel, holes) = align ts in ordered (skel, holes, 0)
 where
  ordered (skel, holes, n) = (skel, sortOn hKey holes, n)

-- | Each hole's parameter: the index of its value vector among the
-- distinct vectors in order of first appearance.
paramsOf :: [Hole] -> [Int]
paramsOf holes = reverse (snd (foldl' step (M.empty, []) (map hValues holes)))
 where
  step (seen, acc) vec = case M.lookup vec seen of
    Just p -> (seen, p : acc)
    Nothing -> (M.insert vec (M.size seen) seen, M.size seen : acc)

-- | The suggestion row [g,params,kept,savings,feasible,reason] and
-- the hole rows [g,hole,param,m,post,postEnd]: the member's first and
-- last root of the hole (one node twice for a leaf hole).
suggest :: Group -> ([Integer], [[Integer]])
suggest g = ([gId g, toInteger count, toInteger kept, savings, toInteger (fromEnum (reason == reasonNone)), reason], holeRows)
 where
  (_, holes, pairs) = skeletonOf g
  params = paramsOf holes
  count = length (M.fromList [(p, ()) | p <- params])
  reason = case filter (/= reasonNone) (map holeReason holes) of
    (r : _) -> r
    []
      | count > paramCap -> reasonParams
      | savings <= 0 -> reasonNoSavings
      | otherwise -> reasonNone
  kept = keeper (gRows g)
  savings = sum (map linesOf (gRows g)) - (skeletonLines g kept pairs + toInteger (length (gRows g)) * callLines)
  holeRows =
    [ [gId g, h, toInteger p, m, toInteger post, toInteger postEnd]
    | (h, hole, p) <- zip3 [0 ..] holes params
    , (m, (post, postEnd)) <- zip [0 ..] (hPosts hole)
    ]

-- | One hole's reason (step-6 ruling 6): a gap holding a statement
-- first, then the anchor's position class.
holeReason :: Hole -> Integer
holeReason h
  | hAcross h = reasonAcross
  | hSlot h == slotExpression || hSlot h == slotName = reasonNone
  | hSlot h == slotType = reasonType
  | otherwise = reasonPosition

-- | The member kept (step-6 ruling 8): the greatest file in-degree,
-- a tie to the least m.
keeper :: [[Integer]] -> Int
keeper rows = negate (snd (maximum [(sum (take 1 (drop 4 row)), negate m) | (m, row) <- zip [0 ..] rows]))

linesOf :: [Integer] -> Integer
linesOf row = sum (take 1 (drop 3 row))

-- | The merged function's lines (step-6 ruling 7): a T1/T2 skeleton
-- is the kept member itself; a T3 skeleton is the kept member's lines
-- in the share of its nodes the kept pairs cover, rounded up — the
-- wire carries no per-node line, so this is the honest integer
-- estimate, never a measured line count.
skeletonLines :: Group -> Int -> Int -> Integer
skeletonLines g kept pairs
  | gFamily g == familyNear = (whole * toInteger pairs + nodes - 1) `div` nodes
  | otherwise = whole
 where
  whole = linesOf (gRows g !! kept)
  nodes = toInteger (sizeOf (gTrees g !! kept))

-- | A member rebuilt from the skeleton and its values: postorder
-- (lab, lld, leaf) columns. A node's leftmost leaf is the first node
-- emitted for it — its own index when no child emitted any (a gap
-- empty on this side).
instantiate :: Skel -> [Hole] -> Int -> ([Int], [Int], [Integer])
instantiate skel holes m = unzip3 (reverse (snd (go skel (0, []))))
 where
  values = M.fromList [(hKey h, v) | h <- holes, v <- take 1 (drop m (hValues h))]
  value k = M.findWithDefault [] k values
  go (SGap k) (n, acc) = (n + length vs, reverse [(lab, n + rel, leaf) | (lab, leaf, rel) <- vs] <> acc)
   where
    vs = value k
  go (SNode key kids) (n, acc) = (n' + 1, (lab, n, leaf) : acc')
   where
    (n', acc') = foldl' (flip go) (n, acc) kids
    (lab, leaf) = either (\k -> case value k of (l, f, _) : _ -> (l, f); [] -> (-1, -1)) id key
