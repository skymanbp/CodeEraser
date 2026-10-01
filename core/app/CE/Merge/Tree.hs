-- | The merge family's trees (plan v2.31 step 6, design booklet §6.2):
-- clone/1's postorder encoding (CE.Clone.Ted.Tree, decoded by the
-- shared CE.Clone.decodeTree) with the four columns merge/1 adds — a
-- node's source-text hash (`leaf`), its position class (`slot`), the
-- hash of its own anonymous tokens (`own`) and of its subtree's whole
-- token stream (`text`) — plus each node's children and parent; and
-- the two things a group's alignment answers, the skeleton and its
-- holes.
module CE.Merge.Tree (
  Atom,
  Cell,
  Group (..),
  Hole (..),
  HoleKey,
  Key,
  MTree (..),
  Skel (..),
  atomOf,
  cellsOf,
  forestNodes,
  isLeafAt,
  keyOf,
  kidsOf,
  lldAt,
  mtree,
  nodeCell,
  ownAt,
  parentOf,
  plain,
  quiet,
  rootOf,
  sizeOf,
  slotAt,
  textAt,
) where

import CE.Clone (WireTree (..), decodeTree)
import CE.Clone.Ted (Tree (..))
import Data.Array (Array, listArray, (!))
import Data.Array.Unboxed (UArray)
import qualified Data.Array.Unboxed as U
import Data.Maybe (fromMaybe)

-- | A node's identity as a relabel sees it: (lab, leaf).
type Key = (Int, Integer)

-- | A node as a skeleton holds it: (lab, leaf, own) — the key and the
-- node's own anonymous tokens, so a rebuild restores both.
type Atom = (Int, Integer, Integer)

-- | One node of a hole's value: its atom and its lld relative to the
-- value's first node, so a forest rebuilds with its shape.
type Cell = (Int, Integer, Integer, Int)

-- | A hole's place in hole order (step-6 ruling 5, gen-2 ruling R5): (anchor, rank,
-- tie). A hole at member 0's node n is (n, 1, 0); a gap whose member-0
-- side holds a forest is (its first node, 1, 0); a gap empty on member
-- 0's side is (the next kept child's leftmost leaf in member 0, or the
-- parent itself, 0, its member-1 side's first root). Ascending order is
-- hole order: member 0's postorder, a place inside another first.
type HoleKey = (Int, Int, Int)

-- | One member's tree: clone/1's decoded arrays, the leaf, own and
-- text hashes, the position classes, each node's children left to
-- right and its parent (−1 at the root).
data MTree = MTree
  { mBase :: Tree
  , mLeaf :: Array Int Integer
  , mOwn :: Array Int Integer
  , mText :: Array Int Integer
  , mSlot :: UArray Int Int
  , mKids :: Array Int [Int]
  , mParent :: UArray Int Int
  }

-- | One group: its id, its family, the helper lines a fragment's
-- merged function adds (0 for whole units), its member rows
-- [g,m,unit,lines,fileIndeg] and their trees, in member order.
data Group = Group {gId :: Integer, gFamily :: Integer, gHelper :: Integer, gRows :: [[Integer]], gTrees :: [MTree]}

-- | The merged function's shape: a node whose atom is fixed (Right)
-- or a hole (Left: a relabel, or a node whose own tokens differ), or a
-- gap — a run of whole subtrees, or a widened expression, that is one
-- hole.
data Skel = SNode (Either HoleKey Atom) [Skel] | SGap HoleKey

-- | One hole: its order key, each member's first and last root — a
-- node hole's one node twice, a gap's forest from its first root to
-- its last, (−1, −1) an empty side — each member's value (the cells
-- the rebuild restores), each member's text vector (what a parameter
-- is: one per distinct vector, gen-2 ruling R7), the position class
-- feasibility reads and whether a gap's forests hold a statement.
data Hole = Hole
  { hKey :: HoleKey
  , hPosts :: [(Int, Int)]
  , hValues :: [[Cell]]
  , hTexts :: [[Integer]]
  , hSlot :: Int
  , hAcross :: Bool
  }

-- | A hole whose text vectors are every member's alike (only
-- whitespace differs: `leaf` hashes the bytes, `text` the tokens): it
-- stays in the skeleton for the rebuild and is no parameter, no reason,
-- no row (gen-2 ruling R3).
quiet :: Hole -> Bool
quiet h = case hTexts h of
  v : vs -> all (== v) vs
  [] -> True

-- | A member's tree from its wire form and its slot, own and text
-- columns. Children come off the postorder encoding: node i's last
-- child is i − 1, and each child's left sibling ends just before that
-- child's leftmost leaf, down to i's own leftmost leaf.
mtree :: WireTree -> [Int] -> [Integer] -> [Integer] -> MTree
mtree w slots owns texts =
  MTree base (column (wLeaf w)) (column (Just owns)) (column (Just texts)) (U.listArray bounds slots) kids parents
 where
  base = decodeTree w
  n = tSize base
  bounds = (0, n - 1)
  column = listArray bounds . fromMaybe (replicate n 0)
  lld i = tLld base U.! i
  kids = listArray bounds (map children [0 .. n - 1])
  parents = U.accumArray (\_ p -> p) (-1) bounds [(c, i) | i <- [0 .. n - 1], c <- children i]
  children i = walk (i - 1) []
   where
    walk k acc
      | k < lld i = acc
      | otherwise = walk (lld k - 1) (k : acc)

sizeOf :: MTree -> Int
sizeOf = tSize . mBase

rootOf :: MTree -> Int
rootOf t = sizeOf t - 1

lldAt :: MTree -> Int -> Int
lldAt t i = tLld (mBase t) U.! i

keyOf :: MTree -> Int -> Key
keyOf t i = (tLab (mBase t) U.! i, mLeaf t ! i)

atomOf :: MTree -> Int -> Atom
atomOf t i = let (lab, leaf) = keyOf t i in (lab, leaf, ownAt t i)

ownAt :: MTree -> Int -> Integer
ownAt t i = mOwn t ! i

textAt :: MTree -> Int -> Integer
textAt t i = mText t ! i

slotAt :: MTree -> Int -> Int
slotAt t i = mSlot t U.! i

kidsOf :: MTree -> Int -> [Int]
kidsOf t i = mKids t ! i

parentOf :: MTree -> Int -> Int
parentOf t i = mParent t U.! i

isLeafAt :: MTree -> Int -> Bool
isLeafAt t i = lldAt t i == i

-- | Every node of a run of sibling subtrees, in postorder (a
-- subtree's nodes are the range from its leftmost leaf to its root).
forestNodes :: MTree -> [Int] -> [Int]
forestNodes t roots = concat [[lldAt t r .. r] | r <- roots]

-- | A run of sibling subtrees as a value: every node's atom and lld
-- relative to the run's first node.
cellsOf :: MTree -> [Int] -> [Cell]
cellsOf t roots = [(lab, leaf, own, lldAt t i - base) | i <- nodes, let (lab, leaf, own) = atomOf t i]
 where
  nodes = forestNodes t roots
  base = case nodes of
    (first : _) -> first
    [] -> 0

-- | One node alone as a value (a relabel's, an own-token hole's).
nodeCell :: MTree -> Int -> Cell
nodeCell t i = let (lab, leaf, own) = atomOf t i in (lab, leaf, own, 0)

-- | A subtree copied into the skeleton as it stands.
plain :: MTree -> Int -> Skel
plain t i = SNode (Right (atomOf t i)) (map (plain t) (kidsOf t i))
