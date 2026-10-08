-- | A second spelling of merge/1's judgment (plan v2.33, reference
-- evaluators) from design booklet docs/reference/analysis-track.md
-- §6.3: a group's holes, their parameters, the member kept, the
-- lines saved, the feasibility and its reason, and each hole's first
-- and last root per member — computed by walking the members' trees
-- TOP-DOWN as rose trees (ReferenceMergeGen), where the shipped
-- modules index postorder arrays bottom-up. A T1/T2 group is walked
-- node for node; a T3 pair through CE.Clone.Ted.tedMapping, whose
-- optimality and Tai validity MergeProps holds against ReferenceTed.
--
-- The rules as read here: a node whose key — kind, and a leaf's hash
-- — differs among the members is a relabel hole of member 0's class;
-- one whose key agrees but own tokens differ, a hole of class other.
-- A pair of T3 parents keeps only the mapped pairs among its direct
-- children; between them (and before the first, after the last) the
-- unmapped children of each side form a gap — none when both sides
-- are empty or cell for cell equal (kind, leaf, relative leftmost
-- leaf), else a hole of class other, a statement anywhere in it making
-- it `spans_statements`. A parent that is an expression on every
-- member WIDENS when a child is a non-quiet relabelled leaf of class
-- other on member 0, or (T3) a gap has subtrees on one side only: it
-- becomes one expression hole over its whole subtree, everything
-- inside absorbed — met top-down, so the outermost such parent wins.
-- Holes are ordered by member 0's postorder (anchor, rank, tie); a
-- hole whose text vectors are all alike is quiet; the live holes'
-- distinct text vectors are the parameters, numbered as they first
-- appear.
--
-- Where the booklet is silent the reading taken is stated: a quiet
-- relabel widens nothing (a hole that is no parameter decides no
-- class), and a relabel's class is member 0's node's.
module ReferenceMerge (expected, unmappedRoot) where

import CE.Clone (WireTree (..), decodeTree)
import CE.Clone.Ted (tedMapping)
import Data.List (elemIndex, nub, sortOn, transpose)
import qualified Data.Set as S
import ReferenceMergeGen

-- | A node with its postorder index and leftmost leaf.
data At = At {aIx, aLld :: Int, aNode :: Node, aKids :: [At]}

-- | A hole: its order key, each member's (first root, last root), each
-- member's text vector, the reason it alone would give (0 = none).
data H = H {hKey :: (Int, Int, Int), hPosts :: [(Int, Int)], hTexts :: [[Integer]], hReason :: Integer}

number :: Node -> At
number = fst . go 0
 where
  go start n =
    let step (acc, s) k = let (a, s') = go s k in (acc <> [a], s')
        (ks, next) = foldl step ([], start) (nKids n)
     in (At next start n ks, next + 1)

slot :: At -> Int
slot = nSlot . aNode

text :: At -> Integer
text = textOf . aNode

key :: At -> (Int, Integer)
key a = (nLab (aNode a), nLeaf (aNode a))

-- | Member 0's node of an aligned set (a group has members).
lead :: [At] -> At
lead (a : _) = a
lead [] = error "an aligned set has a node per member"

isLeaf :: At -> Bool
isLeaf = null . aKids

-- | The reason a hole of class c gives: none for an expression or a
-- name, 2 for a type, 1 for a statement or other.
classReason :: Int -> Integer
classReason c
  | c == 1 || c == 3 = 0
  | c == 2 = 2
  | otherwise = 1

varies :: (Eq b) => (At -> b) -> [At] -> Bool
varies f ats = length (nub (map f ats)) > 1

-- | The hole a set of aligned nodes opens by itself, if any.
nodeH :: [At] -> [H]
nodeH ats
  | varies key ats = [H here posts [[text a] | a <- ats] (classReason (slot (lead ats)))]
  | varies (nOwn . aNode) ats = [H here posts [[nOwn (aNode a)] | a <- ats] 1]
  | otherwise = []
 where
  here = (aIx (lead ats), 1, 0)
  posts = [(aIx a, aIx a) | a <- ats]

-- | One expression hole over the aligned subtrees.
widenedH :: [At] -> H
widenedH ats = H (aIx (lead ats), 1, 0) [(aIx a, aIx a) | a <- ats] [[text a] | a <- ats] 0

-- | A child column that widens its parent: leaves everywhere, keys
-- and texts not all alike, member 0's class other.
triggers :: [At] -> Bool
triggers col = all isLeaf col && varies key col && varies text col && slot (lead col) == 4

expressions :: [At] -> Bool
expressions = all ((== 1) . slot)

-- | T1/T2: the aligned members, top-down.
exactH :: [At] -> [H]
exactH ats
  | expressions ats && any triggers cols = [widenedH ats]
  | otherwise = nodeH ats <> concatMap exactH cols
 where
  cols = transpose (map aKids ats)

-- | T3: the kept pairs reached from a kept pair, the parent counted.
keptPairs :: S.Set (Int, Int) -> At -> At -> Int
keptPairs m a b = 1 + sum [keptPairs m c d | (c, d) <- childPairs m a b]

childPairs :: S.Set (Int, Int) -> At -> At -> [(At, At)]
childPairs m a b = [(c, d) | c <- aKids a, d <- aKids b, S.member (aIx c, aIx d) m]

-- | The runs of unmapped children around the kept child pairs, each
-- with the kept pair that follows it.
runsOf :: [At] -> [At] -> [(At, At)] -> [([At], [At], Maybe At)]
runsOf xs ys [] = [(xs, ys, Nothing)]
runsOf xs ys ((c, d) : rest) = (before c xs, before d ys, Just c) : runsOf (after c xs) (after d ys) rest
 where
  before k = takeWhile ((/= aIx k) . aIx)
  after k = drop 1 . dropWhile ((/= aIx k) . aIx)

nearH :: S.Set (Int, Int) -> At -> At -> [H]
nearH m a b
  | expressions [a, b] && (any (triggers . pairList) pairs || any oneSided runs) = [widenedH [a, b]]
  | otherwise = nodeH [a, b] <> concatMap (gapH a) runs <> concat [nearH m c d | (c, d) <- pairs]
 where
  pairs = childPairs m a b
  runs = runsOf (aKids a) (aKids b) pairs
  pairList (c, d) = [c, d]
  oneSided (fa, fb, _) = null fa /= null fb

-- | A run's gap hole, unless both sides are empty or alike.
gapH :: At -> ([At], [At], Maybe At) -> [H]
gapH parent (fa, fb, next)
  | null fa && null fb = []
  | cells fa == cells fb = []
  | otherwise = [forestH k fa fb]
 where
  k = case (fa, fb) of
    (r : _, _) -> (aLld r, 1, 0)
    (_, r : _) -> (maybe (aIx parent) aLld next, 0, aIx r)
    _ -> (aIx parent, 0, -1)

-- | Two forests as one hole of class other, `spans_statements` when
-- either holds a statement.
forestH :: (Int, Int, Int) -> [At] -> [At] -> H
forestH k fa fb = H k [ends fa, ends fb] [map text fa, map text fb] (if any ((== 0) . slot) (concatMap subtree (fa <> fb)) then 3 else 1)
 where
  ends rs = case rs of
    r : _ -> (aIx r, aIx (last rs))
    [] -> (-1, -1)

subtree :: At -> [At]
subtree a = concatMap subtree (aKids a) <> [a]

-- | A forest's nodes as (kind, leaf, leftmost leaf relative to the
-- forest's first node).
cells :: [At] -> [(Int, Integer, Int)]
cells rs = [(nLab (aNode x), nLeaf (aNode x), aLld x - base) | x <- concatMap subtree rs]
 where
  base = case rs of
    r : _ -> aLld r
    [] -> 0

-- | The suggestion row [g,params,kept,savings,feasible,reason] and the
-- hole rows [g,hole,param,m,post,postEnd] merge/1 answers group g.
expected :: Integer -> MGroup -> ([Integer], [[Integer]])
expected g grp = ([g, toInteger (length vectors), toInteger kept, savings, if reason == 0 then 1 else 0, reason], rows)
 where
  trees = [number t | (t, _, _) <- gMembers grp]
  (holes, pairs) = alignment grp
  live = [h | h <- sortOn hKey holes, length (nub (hTexts h)) > 1]
  vectors = nub (map hTexts live)
  param h = maybe (-1) toInteger (elemIndex (hTexts h) vectors)
  degrees = [d | (_, _, d) <- gMembers grp]
  kept = length (takeWhile (/= maximum degrees) degrees)
  lineCount = [l | (_, l, _) <- gMembers grp]
  keptLines = lineCount !! kept
  keptNodes = toInteger (aIx (trees !! kept) + 1)
  skeleton
    | gFam grp == 1 = negate (negate (keptLines * toInteger pairs) `div` keptNodes)
    | otherwise = keptLines
  savings = sum lineCount - (gHelper grp + skeleton + toInteger (length lineCount))
  reason = case filter (/= 0) (map hReason live) of
    r : _ -> r
    []
      | length vectors > 6 -> 4
      | savings <= 0 -> 5
      | otherwise -> 0
  rows = [[g, i, param h, m, toInteger p, toInteger e] | (i, h) <- zip [0 ..] live, (m, (p, e)) <- zip [0 ..] (hPosts h)]

-- | A group's holes and, for a T3 pair, its kept pair count: a T1/T2
-- group node for node; a T3 pair through the optimal mapping, the
-- whole pair one gap when the roots are not mapped to each other.
alignment :: MGroup -> ([H], Int)
alignment grp = case trees of
  [a, b] | gFam grp == 1 -> near a b
  _ -> (exactH trees, 0)
 where
  trees = [number t | (t, _, _) <- gMembers grp]
  near a b
    | S.member (aIx a, aIx b) mapping = (nearH mapping a b, keptPairs mapping a b)
    | otherwise = ([forestH (aLld a, 1, 0) [a] [b]], 0)
   where
    mapping = S.fromList (snd (tedMapping (wire a) (wire b)))
    wire t = decodeTree (WireTree [nLab (aNode x) | x <- subtree t] [aLld x | x <- subtree t] Nothing)

-- | A T3 pair whose roots the mapping leaves apart.
unmappedRoot :: MGroup -> Bool
unmappedRoot grp = gFam grp == 1 && snd (alignment grp) == 0
