{-# LANGUAGE OverloadedStrings #-}

-- | merge/1 held to ReferenceMerge, a second spelling that walks the
-- members' trees top-down: two hundred and fifty generated T1/T2
-- groups and two hundred and fifty generated T3 pairs, each sent
-- through the REAL respond, must answer the reference's suggestion row
-- and hole rows field for field — the parameter count and numbering,
-- the member kept, the lines saved, the feasibility and its reason,
-- each hole's first and last root per member; and the generated
-- groups must reach every reason and every hole shape the rules name.
module MergeReferenceProps (battery) where

import CE.Merge (respond)
import Data.Aeson (toJSON)
import Data.List (nub)
import qualified Data.Set as S
import ReferenceMerge (expected, unmappedRoot)
import ReferenceMergeGen
import WireHarness (fieldsOf, generated, runChecks)

battery :: IO Bool
battery =
  runChecks
    ( generated ("v2.33: 250 generated T1/T2 groups answer the reference's suggestion and hole rows", "v2.33: the T1/T2 groups reach every reason and hole shape") judged reached exactShapes exactGroups
        <> generated ("v2.33: 250 generated T3 pairs answer the reference's suggestion and hole rows", "v2.33: the T3 pairs reach every reason and an unmapped root") judged reached nearShapes nearGroups
    )

judged :: MGroup -> Maybe String
judged grp
  | got == Just [Just (toJSON [row]), Just (toJSON holes)] = Nothing
  | otherwise = Just (take 300 (show got <> " /= " <> show (row, holes)))
 where
  (row, holes) = expected 0 grp
  got = fieldsOf respond (groupRequest grp) ["suggestions", "holes"]

exactShapes, nearShapes :: [String]
exactShapes = ["reason 0", "reason 1", "reason 2", "reason 4", "reason 5", "a shared parameter", "several holes"]
nearShapes = ["reason 0", "reason 1", "reason 3", "reason 5", "several holes", "an unmapped root"]

-- | What a group's answer reaches: its reason, a parameter two holes
-- share, more than one hole, a T3 pair with no kept pair.
reached :: MGroup -> S.Set String
reached grp =
  S.fromList
    ( ["reason " <> show (row !! 5)]
        <> ["a shared parameter" | length params > length (nub params)]
        <> ["several holes" | length params > 1]
        <> ["an unmapped root" | unmappedRoot grp]
    )
 where
  (row, holes) = expected 0 grp
  params = [p | [_, _, p, 0, _, _] <- holes]
