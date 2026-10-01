-- | The merge family's trees (plan v2.31 step 6, design booklet §6.2):
-- clone/1's postorder encoding (CE.Clone.Ted.Tree, decoded by the
-- shared CE.Clone.decodeTree) with the two columns merge/1 adds — a
-- node's source-text hash (`leaf`) and its position class (`slot`) —
-- plus the child lists the alignment walks; and the two things a
-- group's alignment answers, the skeleton and its holes.
module CE.Merge.Tree (
  Cell,
  Group (..),
  Hole (..),
  HoleKey,
  Key,
  MTree (..),
  Skel (..),
  cellsOf,
  forestNodes,
  isLeafAt,
  keyOf,
  kidsOf,
  lldAt,
  mtree,
  nodeCell,
  plain,
  rootOf,
  sizeOf,
  slotAt,
) where

import CE.Clone (WireTree (..), decodeTree)
import CE.Clone.Ted (Tree (..))
import Data.Array (Array, listArray, (!))
import Data.Array.Unboxed (UArray)
import qualified Data.Array.Unboxed as U
import Data.Maybe (fromMaybe)

-- | A node's identity as a parameter sees it: (lab, leaf).
type Key = (Int, Integer)

-- | One node of a hole's value: its key and its lld relative to the
-- value's first node, so a forest rebuilds with its shape.
type Cell = (Int, Integer, Int)

-- | A hole's place in hole order (step-6 ruling 5): (0, n) = anchored
-- at member 0's node n; (1, n) = a gap empty on member 0's side,
-- anchored at member 1's node n. Ascending order is hole order.
type HoleKey = (Int, Int)

-- | One member's tree: clone/1's decoded arrays, the leaf hashes, the
-- position classes and each node's children left to right.
data MTree = MTree
  { mBase :: Tree
  , mLeaf :: Array Int Integer
  , mSlot :: UArray Int Int
  , mKids :: Array Int [Int]
  }

-- | One group: its id, its family, its member rows
-- [g,m,unit,lines,fileIndeg] and their trees, in member order.
data Group = Group {gId :: Integer, gFamily :: Integer, gRows :: [[Integer]], gTrees :: [MTree]}

-- | The merged function's shape: a node whose key is fixed (Right)
-- or a parameter (Left, a leaf hole or a relabel), or a gap — a run
-- of whole subtrees that is a parameter.
data Skel = SNode (Either HoleKey Key) [Skel] | SGap HoleKey

-- | One hole: its order key, each member's first and last root — a
-- leaf or relabel hole's one node twice, a gap's forest from its first
-- root to its last, (−1, −1) an empty side — each member's value, the
-- position class feasibility reads and whether a gap's forests hold a
-- statement.
data Hole = Hole
  { hKey :: HoleKey
  , hPosts :: [(Int, Int)]
  , hValues :: [[Cell]]
  , hSlot :: Int
  , hAcross :: Bool
  }

-- | A member's tree from its wire form and its slot column. Children
-- come off the postorder encoding: node i's last child is i − 1, and
-- each child's left sibling ends just before that child's leftmost
-- leaf, down to i's own leftmost leaf.
mtree :: WireTree -> [Int] -> MTree
mtree w slots = MTree base (listArray bounds leaves) (U.listArray bounds slots) (listArray bounds (map children [0 .. n - 1]))
 where
  base = decodeTree w
  n = tSize base
  bounds = (0, n - 1)
  leaves = fromMaybe (replicate n 0) (wLeaf w)
  lld i = tLld base U.! i
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

slotAt :: MTree -> Int -> Int
slotAt t i = mSlot t U.! i

kidsOf :: MTree -> Int -> [Int]
kidsOf t i = mKids t ! i

isLeafAt :: MTree -> Int -> Bool
isLeafAt t i = lldAt t i == i

-- | Every node of a run of sibling subtrees, in postorder (a
-- subtree's nodes are the range from its leftmost leaf to its root).
forestNodes :: MTree -> [Int] -> [Int]
forestNodes t roots = concat [[lldAt t r .. r] | r <- roots]

-- | A run of sibling subtrees as a value: every node's key and lld
-- relative to the run's first node.
cellsOf :: MTree -> [Int] -> [Cell]
cellsOf t roots = [(lab, leaf, lldAt t i - base) | i <- nodes, let (lab, leaf) = keyOf t i]
 where
  nodes = forestNodes t roots
  base = case nodes of
    (first : _) -> first
    [] -> 0

-- | One node alone as a value (a leaf hole's, a relabel's).
nodeCell :: MTree -> Int -> Cell
nodeCell t i = let (lab, leaf) = keyOf t i in (lab, leaf, 0)

-- | A subtree copied into the skeleton as it stands.
plain :: MTree -> Int -> Skel
plain t i = SNode (Right (keyOf t i)) (map (plain t) (kidsOf t i))
