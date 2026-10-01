-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | The merge.request shape and its boundary contract (design booklet
-- §3's row and §6.2; plan v2.31 step 6, merge generation 2 ruling R1):
-- three tables — the groups with their family and helper lines, the
-- members with their unit, lines and file in-degree, one tree per
-- member in member order — each absent read as empty; which dimensions
-- the caps price; and the first offender in request order, by name:
-- the group rows, the member rows, the members against their groups,
-- each group's member count, the tree count, each tree (clone/1's own
-- shape contract, shared, then the four columns merge/1 adds) and last
-- a T1/T2 group whose members are not isomorphic (the measuring side
-- promised one token run).
module CE.Merge.Contract (MergeReq (..), MergeTree (..), groupsOf, offence, overCap) where

import CE.Clone (WireTree (..), treeShape)
import CE.Merge.Align (isomorphic)
import CE.Merge.Cost
import CE.Merge.Tree (Group (..), mtree)
import CE.Wire (rowCheck, tableOffence)
import Data.Aeson
import Data.Aeson.Types (Parser)
import Data.Foldable (asum)
import qualified Data.Map.Strict as M
import Data.Maybe (fromMaybe)

-- | One member's tree: clone/1's wire tree (lab, lld, leaf), the slot
-- column, and the own and text columns (ruling R1: each node's own
-- anonymous tokens and its subtree's token stream, fnv1a64 hashes).
data MergeTree = MergeTree {tWire :: WireTree, tSlot :: Maybe [Int], tOwn :: Maybe [Integer], tText :: Maybe [Integer]}

instance FromJSON MergeTree where
  parseJSON v = MergeTree <$> parseJSON v <*> column "slot" <*> column "own" <*> column "text"
   where
    column :: (FromJSON a) => Key -> Parser (Maybe a)
    column k = withObject "tree" (.:? k) v

data MergeReq = MergeReq
  { reqId :: Value
  , groupRows :: [[Integer]]
  , memberRows :: [[Integer]]
  , treeRows :: [MergeTree]
  }

instance FromJSON MergeReq where
  parseJSON = withObject "MergeReq" $ \o ->
    MergeReq <$> o .: "id" <*> o .:? "groups" .!= [] <*> o .:? "members" .!= [] <*> o .:? "trees" .!= []

-- | Groups and nodes, each against its own ceiling.
overCap :: MergeReq -> Bool
overCap req =
  toInteger (length (groupRows req)) > groupCap
    || toInteger (sum (map (length . wLab . tWire) (treeRows req))) > treeNodeCap

offence :: MergeReq -> Maybe String
offence req =
  asum
    [ tableOffence "group" (take 1) groupShape (groupRows req)
    , tableOffence "member" (take 2) memberShape members
    , asum (zipWith3 (memberLink families) [0 ..] (Nothing : map Just members) members)
    , asum (zipWith (groupSize (M.fromListWith (+) [(g, 1 :: Int) | g : _ <- members])) [0 ..] (groupRows req))
    , treeCount
    , asum (zipWith treeOffence [0 ..] (treeRows req))
    , asum [Just ("group " <> show i <> ": members are not isomorphic") | (i, g) <- zip [0 :: Int ..] (groupsOf req), gFamily g == familyExact, not (isomorphic (gTrees g))]
    ]
 where
  members = memberRows req
  families = M.fromList [(g, fam) | [g, fam, _] <- groupRows req]
  treeCount
    | length (treeRows req) == length members = Nothing
    | otherwise = Just ("trees: " <> show (length (treeRows req)) <> " trees for " <> show (length members) <> " members")

-- | [g, family, helper]: non-negative, a known family.
groupShape :: Int -> [Integer] -> Maybe String
groupShape = rowCheck "group" "malformed group (need [g,family,helper])" 3 checks
 where
  checks row = case row of
    [_, fam, _]
      | any (< 0) row -> Just "negative group value"
      | fam > familyNear -> Just "unknown family"
    _ -> Nothing

-- | [g, m, unit, lines, fileIndeg]: non-negative.
memberShape :: Int -> [Integer] -> Maybe String
memberShape = rowCheck "member" "malformed member (need [g,m,unit,lines,fileIndeg])" 5 checks
 where
  checks row
    | any (< 0) row = Just "negative member value"
    | otherwise = Nothing

-- | A member names a group, and its m continues the previous row's
-- group (0 when it opens one).
memberLink :: M.Map Integer Integer -> Int -> Maybe [Integer] -> [Integer] -> Maybe String
memberLink families i prev row = case row of
  g : m : _
    | not (M.member g families) -> label "unknown group"
    | m /= next g -> label "m not consecutive"
  _ -> Nothing
 where
  label why = Just ("member " <> show i <> ": " <> why)
  next g = case prev of
    Just (pg : pm : _) | pg == g -> pm + 1
    _ -> 0

-- | Two members at least; a T3 pair exactly two.
groupSize :: M.Map Integer Int -> Int -> [Integer] -> Maybe String
groupSize counts i row = case row of
  [g, fam, _] -> case M.findWithDefault 0 g counts of
    0 -> label "no members"
    1 -> label "one member"
    n | fam == familyNear && n /= 2 -> label "a T3 pair needs exactly two members"
    _ -> Nothing
  _ -> Nothing
 where
  label why = Just ("group " <> show i <> ": " <> why)

-- | clone/1's shape contract, then the four added columns present in
-- wire order, each as long as the tree, every slot a known class,
-- every own and text hash non-negative.
treeOffence :: Int -> MergeTree -> Maybe String
treeOffence t tree = asum [treeShape t (tWire tree), leaf, slot, hashes "own" (tOwn tree), hashes "text" (tText tree)]
 where
  label why = Just ("tree " <> show t <> ": " <> why)
  size = length (wLab (tWire tree))
  leaf = maybe (label "leaf missing") (const Nothing) (wLeaf (tWire tree))
  slot = case tSlot tree of
    Nothing -> label "slot missing"
    Just slots
      | length slots /= size -> label "slot length mismatch"
      | otherwise -> asum [label ("node " <> show n <> ": unknown slot") | (n, s) <- zip [0 :: Int ..] slots, s < 0 || s > slotCeil]
  hashes what column = case column of
    Nothing -> label (what <> " missing")
    Just hs
      | length hs /= size -> label (what <> " length mismatch")
      | otherwise -> asum [label ("node " <> show n <> ": negative " <> what) | (n, h) <- zip [0 :: Int ..] hs, h < 0]

-- | The groups in request order, each with its member rows and their
-- trees (members and trees zip in member order).
groupsOf :: MergeReq -> [Group]
groupsOf req = [Group g fam helper (map fst own) (map snd own) | [g, fam, helper] <- groupRows req, let own = M.findWithDefault [] g byGroup]
 where
  byGroup = M.fromListWith (flip (<>)) [(g, [(row, tree)]) | (row@(g : _), tree) <- zip (memberRows req) trees]
  trees = [mtree (tWire t) (fromMaybe [] (tSlot t)) (fromMaybe [] (tOwn t)) (fromMaybe [] (tText t)) | t <- treeRows req]
