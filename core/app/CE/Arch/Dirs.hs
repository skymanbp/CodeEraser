-- | The two graphs the arch judgment reads (step-8 brief §1.0 rulings
-- 1 and 5): the directory graph — every file-to-file reference folded
-- onto its two directories, every file-to-directory reference onto
-- the file's directory and the named one, references inside one
-- directory dropped, one arc per ordered pair with the weights added
-- — and the undirected file graph the clustering walks, where a pair
-- referencing each other both ways is one neighbour with both
-- weights. The strongly connected components of the directory graph
-- come from CE.Graph.Build, the same decomposition the graph family
-- answers its cycles with: one Tarjan in the core, not two.
module CE.Arch.Dirs (Arcs, arcsOf, dirOfFile, fileNeighbours, sccsOf) where

import CE.Graph.Build (Built (..), build)
import qualified Data.IntMap.Strict as IM
import qualified Data.Map.Strict as M

-- | The directory graph: (from, to) ↦ weight, from ≠ to.
type Arcs = M.Map (Int, Int) Integer

-- | Each file's directory, from the `files` table.
dirOfFile :: [[Integer]] -> IM.IntMap Int
dirOfFile rows = IM.fromList [(fromInteger f, fromInteger d) | [f, d, _] <- rows]

-- | Ruling 1: dir(F) → dir(G) per file edge and dir(F) → D per
-- package edge, same-directory references uncounted, an ordered pair
-- met twice one arc with its weights added.
arcsOf :: IM.IntMap Int -> [[Integer]] -> [[Integer]] -> Arcs
arcsOf dirOf edges pkgs =
  M.fromListWith (+) [(pair, w) | (pair@(a, b), w) <- fileArcs <> pkgArcs, a /= b]
 where
  at f = IM.findWithDefault 0 (fromInteger f) dirOf
  fileArcs = [((at f, at g), w) | [f, g, w] <- edges]
  pkgArcs = [((at f, fromInteger d), w) | [f, d, w] <- pkgs]

-- | Ruling 5: the file graph without direction — f's neighbour g
-- weighs every reference between them, either way. No file is its
-- own neighbour (the contract refuses a self edge).
fileNeighbours :: [[Integer]] -> IM.IntMap (IM.IntMap Integer)
fileNeighbours edges =
  IM.fromListWith (IM.unionWith (+)) (concat [[one f g w, one g f w] | [f, g, w] <- edges])
 where
  one a b w = (fromInteger a, IM.singleton (fromInteger b) w)

-- | The directory graph's strongly connected components over n
-- directories, members ascending: the graph family's builder over
-- rows of its own shape (kind 0, rung 1, nothing inert), so the
-- components are a function of the sorted arc set.
sccsOf :: Int -> Arcs -> [[Int]]
sccsOf n arcs = bScc (build 1 [] n [[toInteger a, toInteger b, 0, 1] | (a, b) <- M.keys arcs])
