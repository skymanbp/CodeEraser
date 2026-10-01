-- | From a group's holes to its suggestion (plan v2.31 step 6, design
-- booklet §6.3; step-6 rulings 5–8, merge generation 2): the holes in
-- hole order, the quiet ones (texts every member's alike) left out,
-- their parameters (one per distinct text vector, numbered by first
-- appearance — ruling R7), the feasibility and its reason (the first
-- infeasible hole's, else too many parameters, else no line saved), the
-- member kept, the lines a merge would save (a fragment's merged
-- function paying its helper's head and closing lines — ruling R6);
-- and the rebuild a skeleton and a member's values give back — the
-- law the battery holds every suggestion to.
module CE.Merge.Holes (instantiate, liveHoles, paramsOf, skeletonOf, suggest) where

import CE.Merge.Align (align)
import CE.Merge.Cost
import CE.Merge.Mapped (mapped)
import CE.Merge.Tree
import Data.List (sortOn, unzip4)
import qualified Data.Map.Strict as M

-- | A group's skeleton, every hole in hole order (ascending key —
-- member 0's postorder, a gap empty on member 0's side just before the
-- kept child it precedes; the quiet holes included, for the rebuild)
-- and, for a T3 pair, the kept pair count.
skeletonOf :: Group -> (Skel, [Hole], Int)
skeletonOf g = case gTrees g of
  [a, b] | gFamily g == familyNear -> ordered (mapped a b)
  ts -> let (skel, holes) = align ts in ordered (skel, holes, 0)
 where
  ordered (skel, holes, n) = (skel, sortOn hKey holes, n)

-- | The holes a suggestion reads: every one but the quiet.
liveHoles :: [Hole] -> [Hole]
liveHoles = filter (not . quiet)

-- | Each hole's parameter: the index of its text vector among the
-- distinct vectors in order of first appearance — one parameter per
-- distinct combination of the members' texts, whatever the places'
-- kinds.
paramsOf :: [Hole] -> [Int]
paramsOf holes = reverse (snd (foldl' step (M.empty, []) (map hTexts holes)))
 where
  step (seen, acc) vec = case M.lookup vec seen of
    Just p -> (seen, p : acc)
    Nothing -> (M.insert vec (M.size seen) seen, M.size seen : acc)

-- | The suggestion row [g,params,kept,savings,feasible,reason] and
-- the hole rows [g,hole,param,m,post,postEnd] of the live holes: the
-- member's first and last root of the hole (one node twice for a node
-- hole or a widened one).
suggest :: Group -> ([Integer], [[Integer]])
suggest g = ([gId g, toInteger count, toInteger kept, savings, toInteger (fromEnum (reason == reasonNone)), reason], holeRows)
 where
  (_, all', pairs) = skeletonOf g
  holes = liveHoles all'
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

-- | One hole's reason (step-6 ruling 6, gen-2 ruling R4): a gap
-- holding a statement first, then the hole's position class — a gap
-- and an own-token hole are of class other.
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

-- | The merged function's lines (step-6 ruling 7, gen-2 ruling R6): a
-- T1/T2 skeleton is the kept member itself; a T3 skeleton is the kept
-- member's lines in the share of its nodes the kept pairs cover,
-- rounded up — the wire carries no per-node line, so this is the
-- honest integer estimate, never a measured line count; either way a
-- fragment's merged function adds its helper's head and closing lines
-- (the group's `helper` column, 0 for whole units).
skeletonLines :: Group -> Int -> Int -> Integer
skeletonLines g kept pairs = gHelper g + body
 where
  body
    | gFamily g == familyNear = (whole * toInteger pairs + nodes - 1) `div` nodes
    | otherwise = whole
  whole = linesOf (gRows g !! kept)
  nodes = toInteger (sizeOf (gTrees g !! kept))

-- | A member rebuilt from the skeleton and its values: postorder
-- (lab, lld, leaf, own) columns. A node's leftmost leaf is the first
-- node emitted for it — its own index when no child emitted any (a gap
-- empty on this side).
instantiate :: Skel -> [Hole] -> Int -> ([Int], [Int], [Integer], [Integer])
instantiate skel holes m = unzip4 (reverse (snd (go skel (0, []))))
 where
  values = M.fromList [(hKey h, v) | h <- holes, v <- take 1 (drop m (hValues h))]
  value k = M.findWithDefault [] k values
  go (SGap k) (n, acc) = (n + length vs, reverse [(lab, n + rel, leaf, own) | (lab, leaf, own, rel) <- vs] <> acc)
   where
    vs = value k
  go (SNode key kids) (n, acc) = (n' + 1, (lab, n, leaf, own) : acc')
   where
    (n', acc') = foldl' (flip go) (n, acc) kids
    (lab, leaf, own) = either (\k -> case value k of (l, f, o, _) : _ -> (l, f, o); [] -> (-1, -1, -1)) id key
