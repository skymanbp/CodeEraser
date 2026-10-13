{-# LANGUAGE OverloadedStrings #-}

-- | merge/1 held to ReferenceMerge, a second spelling that walks the
-- members' trees top-down, under three base seeds each: two hundred
-- and fifty generated T1/T2 groups and two hundred and fifty generated
-- T3 pairs, each sent through the REAL respond, must answer the
-- reference's suggestion row and hole rows field for field — the
-- parameter count and numbering, the member kept, the lines saved, the
-- feasibility and its reason, each hole's first and last root per
-- member; two hundred T3 pairs aligned through a non-optimal mapping
-- (CE.Merge.Mapped.mappedWith, the door the optimal mapping never
-- opens) must open the reference's live holes and keep its pair count;
-- and under EVERY seed the groups must reach every reason and every
-- rule shape §6.3 states — the widenings R3 and R3b, a quiet relabel
-- and a leaf against an inner node that do not widen, member 0's class
-- deciding a hole and a widening, a widened pair that keeps pairs, the
-- own-only gap and the equal gap.
module MergeReferenceProps (battery) where

import CE.Clone (WireTree (..))
import CE.Merge (respond)
import CE.Merge.Holes (liveHoles)
import CE.Merge.Mapped (mappedWith)
import CE.Merge.Tree (Hole (..), MTree, mtree)
import Data.Aeson (toJSON)
import Data.List (nub, sortOn)
import qualified Data.Set as S
import ReferenceMerge (expected, holesWith, shapesOf)
import ReferenceMergeGen
import WireHarness (fieldsOf, generated, runChecks)

battery :: IO Bool
battery = runChecks (concatMap seeded [0, 1, 2])

seeded :: Int -> [(String, Bool)]
seeded base =
  generated (named "250 generated T1/T2 groups answer the reference's suggestion and hole rows", named "the T1/T2 groups reach every reason and rule shape") judged (reached Nothing) exactShapes (exactGroups base)
    <> generated (named "250 generated T3 pairs answer the reference's suggestion and hole rows", named "the T3 pairs reach every reason and rule shape") judged (reached Nothing) nearShapes (nearGroups base)
    <> generated (named "200 T3 pairs under a non-optimal mapping open the reference's live holes", named "the mapped pairs reach the own-only gap and the equal gap") mappedJudged mappedReached gapShapes (ownGapGroups base)
 where
  named s = "v2.33 seed " <> show base <> ": " <> s

judged :: MGroup -> Maybe String
judged grp
  | got == Just [Just (toJSON [row]), Just (toJSON holes)] = Nothing
  | otherwise = Just (take 300 (show got <> " /= " <> show (row, holes)))
 where
  (row, holes) = expected 0 grp
  got = fieldsOf respond (groupRequest grp) ["suggestions", "holes"]

-- | The shipped alignment under the given mapping against the
-- reference's: the live holes in hole order and the kept pair count.
mappedJudged :: (MGroup, S.Set (Int, Int)) -> Maybe String
mappedJudged (grp, m) = case [tree t | (t, _, _) <- gMembers grp] of
  [a, b] ->
    let (_, holes, kept) = mappedWith m a b
        got = ([(hKey h, hPosts h, hTexts h) | h <- liveHoles (sortOn hKey holes)], kept)
     in if got == want then Nothing else Just (take 300 (show got <> " /= " <> show want))
  _ -> Just "a mapped group is a pair"
 where
  want = holesWith m grp

-- | A generated member as the shipped modules read it.
tree :: Node -> MTree
tree t = mtree (WireTree [nLab n | (n, _) <- cells] [l | (_, l) <- cells] (Just [nLeaf n | (n, _) <- cells])) [nSlot n | (n, _) <- cells] [nOwn n | (n, _) <- cells] [textOf n | (n, _) <- cells]
 where
  cells = flatten t

exactShapes, nearShapes, gapShapes :: [String]
exactShapes =
  reasons [0, 1, 2, 4, 5]
    <> ["a shared parameter", "several holes", "widen R3", "a quiet relabel does not widen", "member 0's class decides a hole", "member 0's class decides a widening"]
nearShapes =
  reasons [0, 1, 2, 3, 4, 5]
    <> ["several holes", "an unmapped root", "widen R3", "widen R3b", "a quiet relabel does not widen", "a leaf against an inner node does not widen", "member 0's class decides a hole", "member 0's class decides a widening", "a widened pair keeps pairs"]
gapShapes = ["an own-only gap", "an equal gap joins the skeleton"]

reasons :: [Int] -> [String]
reasons = map (("reason " <>) . show)

-- | What a group reaches: its reason, a parameter two holes share,
-- more than one hole, and every rule shape its walk met.
reached :: Maybe (S.Set (Int, Int)) -> MGroup -> S.Set String
reached m grp =
  S.fromList
    ( ["reason " <> show (row !! 5)]
        <> ["a shared parameter" | length params > length (nub params)]
        <> ["several holes" | length params > 1]
        <> shapesOf m grp
    )
 where
  (row, holes) = expected 0 grp
  params = [p | [_, _, p, 0, _, _] <- holes]

mappedReached :: (MGroup, S.Set (Int, Int)) -> S.Set String
mappedReached (grp, m) = S.fromList (shapesOf (Just m) grp)
