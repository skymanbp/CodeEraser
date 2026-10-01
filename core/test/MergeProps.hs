-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | The merge family's battery (plan v2.31 step 6, design booklet
-- §6.5): the hand-written cases of MergeCases and MergeRefusals (one
-- leg each), then the laws — every suggestion's skeleton and each
-- member's values rebuild that member (instantiation), and no
-- parameter can be dropped (minimality) — over those cases and two
-- hundred generated T1/T2 groups; tedMapping against ted over the
-- exhaustive small-tree family (n ≤ 4, CE_DEEP_TED=1 → 5) and two
-- hundred generated pairs (same distance, a valid Tai mapping, cost
-- equal to the distance); the generated groups' parameter counts
-- and feasibility; the equal-gap fold; each hole's first and last
-- root per member; the caps, the empty request and the counts.
module MergeProps (battery) where

import CE.Clone (WireTree (..), decodeTree)
import CE.Clone.Ted (ted, tedMapping)
import CE.Merge (respond)
import CE.Merge.Contract (MergeReq (..), MergeTree (..), groupsOf, overCap)
import CE.Merge.Cost (familyNear, groupCap, treeNodeCap)
import CE.Merge.Holes (instantiate, paramsOf, skeletonOf)
import CE.Merge.Mapped (mappedWith)
import CE.Merge.Tree
import Data.Aeson
import Data.List (nub, tails)
import qualified Data.Set as S
import MergeCases (Case (..), judgments, mergeRequest, treeValue)
import MergeRefusals (refusals)
import ReferenceTed (family, taiConsistent)
import ReferenceTreeGen (Synth (..), synthGroups, treePairs)
import System.Environment (lookupEnv)
import WireHarness (fieldsOf, refusedBy, runLegs)

battery :: IO Bool
battery = do
  deep <- lookupEnv "CE_DEEP_TED"
  let fam = family (if deep == Just "1" then 5 else 4)
      exhaustive = [(a, b) | a <- fam, b <- fam]
  putStrLn ("     merge mapping: " <> show (length exhaustive) <> " exhaustive pairs, " <> show (length treePairs) <> " generated")
  runLegs (map caseName table <> names) (map holds table <> probes exhaustive)
 where
  table = judgments <> refusals

names :: [String]
names =
  [ "every judgment and every generated group rebuilds each member from the skeleton and its values"
  , "every parameter is needed: made member 0's constant, some member no longer rebuilds"
  , "the exhaustive small-tree family: tedMapping's distance is ted's, its mapping a valid Tai mapping costing the distance"
  , "two hundred generated pairs of 8 to 40 nodes: the same three"
  , "generated T1/T2 groups: the parameters are the distinct vectors, feasible iff every hole is an expression or a name and at most six"
  , "every judgment's and generated group's suggestion: feasible iff reason 0, reason 0 only with a line saved, reason 5 only without; some answers reason 5"
  , "a valid mapping that leaves two equal forests unmapped folds them into the skeleton, no hole"
  , "every hole's roots per member: -1 -1 on an empty side, else post <= postEnd, one node or two siblings under one parent; some gap spans two roots"
  , "an over-cap request degrades with no rows and its counts; the caps sit at 4096 groups and 1,048,576 nodes"
  , "an empty request answers no rows, not degraded"
  , "the counts name groups, members, nodes, suggestions, holes and feasible"
  ]

probes :: [(([Int], [Int]), ([Int], [Int]))] -> [Bool]
probes exhaustive =
  [ all rebuilds lawGroups
  , all minimal lawGroups
  , all (uncurry mappingHolds) exhaustive
  , all (uncurry mappingHolds) treePairs
  , all synthAgrees synthGroups && any synthFeasible synthGroups && not (all synthFeasible synthGroups)
  , reasonsCohere
  , equalGapFolds
  , all postsHold (lawGroups <> nearGroups) && or [p < e | g <- nearGroups, let (_, hs, _) = skeletonOf g, h <- hs, (p, e) <- hPosts h]
  , capped
  , emptyRequest
  , countsNamed
  ]

-- | A case holds when the core answers its suggestion and hole rows,
-- or refuses by its message.
holds :: Case -> Bool
holds c = case caseRefusal c of
  Just message -> refusedBy respond (caseRequest c) message
  Nothing -> fieldsOf respond (caseRequest c) ["suggestions", "holes"] == Just [rows (fst (caseRows c)), rows (snd (caseRows c))]

rows :: [[Integer]] -> Maybe Value
rows = Just . toJSON

groupsIn :: Value -> [Group]
groupsIn v = case fromJSON v of
  Success req -> groupsOf req
  Error _ -> []

synthRequest :: Synth -> Value
synthRequest s =
  mergeRequest
    [[0, 0]]
    [[0, m, m, 10, 0] | m <- [0 .. toInteger (length (synthTrees s)) - 1]]
    [treeValue (ints lab) (ints lld) (Just leaf) (Just (ints slot)) | (lab, lld, leaf, slot) <- synthTrees s]
 where
  ints = map toInteger

lawGroups :: [Group]
lawGroups = concatMap (groupsIn . caseRequest) judgments <> concatMap (groupsIn . synthRequest) synthGroups

-- | A member's own (lab, lld, leaf) columns.
columns :: MTree -> ([Int], [Int], [Integer])
columns t = unzip3 [(lab, lldAt t i, leaf) | i <- [0 .. rootOf t], let (lab, leaf) = keyOf t i]

rebuilds :: Group -> Bool
rebuilds g = and [instantiate skel holes m == columns t | (m, t) <- zip [0 ..] (gTrees g)]
 where
  (skel, holes, _) = skeletonOf g

-- | Each parameter in turn set to member 0's value in every hole it
-- names: some member no longer rebuilds.
minimal :: Group -> Bool
minimal g = and [not (all rebuilt [0 .. length (gTrees g) - 1]) | p <- nub params, let rebuilt m = instantiate skel (constant p) m == columns (gTrees g !! m)]
 where
  (skel, holes, _) = skeletonOf g
  params = paramsOf holes
  constant p = [if q == p then h {hValues = map (const (concat (take 1 (hValues h)))) (hValues h)} else h | (h, q) <- zip holes params]

mappingHolds :: ([Int], [Int]) -> ([Int], [Int]) -> Bool
mappingHolds (la, da) (lb, db) = d == ted ta tb && valid && cost == d
 where
  (ta, tb) = (decodeTree (WireTree la da Nothing), decodeTree (WireTree lb db Nothing))
  (d, m) = tedMapping ta tb
  valid = nub (map fst m) == map fst m && nub (map snd m) == map snd m && and [taiConsistent da db p q | p : rest <- tails m, q <- rest]
  cost = toInteger (length la + length lb - 2 * length m + length [() | (i, j) <- m, la !! i /= lb !! j])

-- | A synthetic group's suggestion carries the parameter count and
-- feasibility its construction implies.
synthAgrees :: Synth -> Bool
synthAgrees s = case suggestionsOf (synthRequest s) of
  [[_, params, _, _, feasible, _]] -> params == toInteger (synthParams s) && feasible == toInteger (fromEnum (synthFeasible s))
  _ -> False

-- | The suggestion rows the core answers a request (none when it
-- refuses).
suggestionsOf :: Value -> [[Integer]]
suggestionsOf req = case fieldsOf respond req ["suggestions"] of
  Just [Just v] | Success rs <- fromJSON v -> rs
  _ -> []

-- | Every suggestion over the judgments and the generated groups:
-- feasible exactly at reason 0, reason 0 only with a line saved and
-- reason 5 only without one; and the table holds a reason 5.
reasonsCohere :: Bool
reasonsCohere = all coheres answered && any ((== [5]) . drop 5) answered
 where
  answered = concatMap suggestionsOf (map caseRequest judgments <> map synthRequest synthGroups)
  coheres [_, _, _, savings, feasible, reason] = (feasible == 1) == (reason == 0) && (reason /= 0 || savings > 0) && (reason /= 5 || savings <= 0)
  coheres _ = False

-- | The two hundred generated pairs as T3 groups, every node an
-- expression: the gap holes the mapping opens.
nearGroups :: [Group]
nearGroups = [Group 0 familyNear [[0, 0, 0, 1, 0], [0, 1, 1, 1, 0]] [member a, member b] | (a, b) <- treePairs]
 where
  member (lab, lld) = mtree (WireTree lab lld Nothing) (map (const 1) lab)

-- | Each hole's (post, postEnd) on each member: both −1 on an empty
-- side; else both the member's nodes, post ≤ postEnd, and one node or
-- two children of one parent — a gap's forest runs between them.
postsHold :: Group -> Bool
postsHold g = and [rooted t p | let (_, holes, _) = skeletonOf g, h <- holes, (t, p) <- zip (gTrees g) (hPosts h)]
 where
  rooted t (p, e)
    | p == -1 || e == -1 = p == e
    | otherwise = 0 <= p && p <= e && e <= rootOf t && (p == e || any (\q -> all (`elem` kidsOf t q) [p, e]) [0 .. rootOf t])

-- | A pair mapped at the roots and the first leaves only (valid, not
-- optimal): the second leaves are a gap equal on both sides, so the
-- skeleton copies them and no hole opens.
equalGapFolds :: Bool
equalGapFolds = null holes && kept == 2 && all (\m -> instantiate skel holes m == columns t) [0, 1]
 where
  t = mtree (WireTree [1, 2, 9] [0, 1, 0] (Just [11, 12, 0])) [1, 1, 4]
  (skel, holes, kept) = mappedWith (S.fromList [(0, 0), (2, 2)]) t t

-- | 4,097 groups through the real respond; the cap predicate at both
-- boundaries of both dimensions.
capped :: Bool
capped =
  fieldsOf respond big ["degraded", "reason", "suggestions", "holes", "counts"]
    == Just [Just (Bool True), Just "merge_too_large", rows [], rows [], Just (counts [groupCap + 1, 0, 0, 0, 0, 0])]
    && overCap (sized (groupCap + 1) 0)
    && not (overCap (sized groupCap 0))
    && overCap (sized 0 (treeNodeCap + 1))
    && not (overCap (sized 0 treeNodeCap))
 where
  big = mergeRequest [[g, 0] | g <- [0 .. groupCap]] [] []
  sized gs n = MergeReq Null (replicate (fromInteger gs) [0, 0]) [] [MergeTree (WireTree (replicate (fromInteger n) 0) [] Nothing) Nothing]

counts :: [Integer] -> Value
counts = object . zipWith (.=) ["groups", "members", "nodes", "suggestions", "holes", "feasible"]

emptyRequest :: Bool
emptyRequest = fieldsOf respond (mergeRequest [] [] []) ["suggestions", "holes", "degraded", "reason"] == Just [rows [], rows [], Just (Bool False), Nothing]

-- | The judgments table's last case: two groups of two one-node-leaf
-- trees, no hole, one feasible and one saving no line.
countsNamed :: Bool
countsNamed = fieldsOf respond (caseRequest (last judgments)) ["counts"] == Just [Just (counts [2, 4, 8, 2, 0, 1])]
