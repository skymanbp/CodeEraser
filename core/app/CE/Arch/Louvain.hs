-- | The file graph's clusters and the files sitting outside their
-- cluster's directory (step-8 brief §1.0 rulings 5 and 6; booklet
-- §7.2). A deterministic Louvain: every file starts alone; the local
-- move visits files by id, takes the file out of its community, and
-- weighs each neighbouring community C and the one it left by the
-- modularity gain in integer form, 2m · k_{i,C} − Σtot(C) · k_i (the
-- real gain times 2m², the same order); the greatest gain wins, the
-- least community id on a tie, and the file moves only when that is
-- strictly better than going back. A pass without a move converges.
-- The communities are then folded once into super-vertices (each
-- keeping its members' total degree) and the local move runs to
-- convergence again on them; then it stops — two levels, no more.
-- Cluster ids are renumbered 0, 1, 2… by each cluster's least file.
-- Nothing divides: modularity only ever compares, so the core stays
-- in integers (the CE.Clone.Cost stance).
module CE.Arch.Louvain (clusters, misplaced) where

import qualified Data.IntMap.Strict as IM
import Data.List (maximumBy, nub)
import Data.Ord (comparing)

-- | A weighted undirected graph on 0..k−1: neighbours without self
-- arcs, and each vertex's degree (for a super-vertex, its members'
-- degrees together — the internal weight it swallowed included).
data Graph = Graph {vertices :: Int, adjacent :: IM.IntMap (IM.IntMap Integer), degree :: IM.IntMap Integer}

-- | Each of n files' cluster, renumbered by least member.
clusters :: Int -> IM.IntMap (IM.IntMap Integer) -> IM.IntMap Int
clusters n adj = renumber n (\f -> second IM.! (super IM.! f))
 where
  g = Graph n adj (IM.fromList [(v, sum (IM.elems (IM.findWithDefault IM.empty v adj))) | v <- [0 .. n - 1]])
  twoM = sum (IM.elems (degree g))
  first = settle twoM g
  (super, g2) = fold g first
  second = settle twoM g2

-- | `n` labels renumbered 0, 1, 2… in order of each label's least
-- member (the members are visited ascending, so first seen = least).
renumber :: Int -> (Int -> Int) -> IM.IntMap Int
renumber n label = IM.fromList [(f, ids IM.! label f) | f <- [0 .. n - 1]]
 where
  ids = foldl' see IM.empty (map label [0 .. n - 1])
  see m l = if IM.member l m then m else IM.insert l (IM.size m) m

-- | Local moves until a pass moves nothing; the vertex → community
-- map (communities named by a member id).
settle :: Integer -> Graph -> IM.IntMap Int
settle twoM g = go start (degree g)
 where
  start = IM.fromList [(v, v) | v <- [0 .. vertices g - 1]]
  go comm tot = case foldl' (move twoM g) (comm, tot, False) [0 .. vertices g - 1] of
    (comm', tot', True) -> go comm' tot'
    (comm', _, False) -> comm'

-- | One vertex's move: out of its community, into the best candidate.
move :: Integer -> Graph -> (IM.IntMap Int, IM.IntMap Integer, Bool) -> Int -> (IM.IntMap Int, IM.IntMap Integer, Bool)
move twoM g (comm, tot, moved) v = (IM.insert v target comm, IM.insertWith (+) target k tot', moved || target /= home)
 where
  home = comm IM.! v
  k = IM.findWithDefault 0 v (degree g)
  tot' = IM.adjust (subtract k) home tot
  links = IM.fromListWith (+) [(comm IM.! u, w) | (u, w) <- IM.toList (IM.findWithDefault IM.empty v (adjacent g))]
  gain c = twoM * IM.findWithDefault 0 c links - IM.findWithDefault 0 c tot' * k
  best = maximumBy (comparing (\c -> (gain c, negate c))) (nub (home : IM.keys links))
  target = if gain best > gain home then best else home

-- | The communities as super-vertices, numbered by least member: the
-- vertex → super-vertex map and the folded graph, whose weights add
-- up every arc between two communities and whose degrees add up the
-- members' degrees.
fold :: Graph -> IM.IntMap Int -> (IM.IntMap Int, Graph)
fold g comm = (super, Graph (IM.size (IM.fromList [(s, ()) | s <- IM.elems super])) adj deg)
 where
  super = renumber (vertices g) (comm IM.!)
  adj =
    IM.fromListWith
      (IM.unionWith (+))
      [ (super IM.! v, IM.singleton (super IM.! u) w)
      | (v, ns) <- IM.toList (adjacent g)
      , (u, w) <- IM.toList ns
      , super IM.! v /= super IM.! u
      ]
  deg = IM.fromListWith (+) [(super IM.! v, d) | (v, d) <- IM.toList (degree g)]

-- | Ruling 6 as tightened in the step-8 lane: `[[F, M]]` for every
-- file whose own directory holds strictly fewer of its cluster's files
-- than M, the directory holding most of them (the least id on a tie).
-- A tie between M and the file's own directory is no evidence of where
-- the file belongs, so it is no finding: a ring of one-file
-- directories names nobody, and a file alone in its cluster never
-- does. Ascending by F.
misplaced :: IM.IntMap Int -> IM.IntMap Int -> [[Integer]]
misplaced dirOf cluster =
  [ [toInteger f, toInteger home]
  | (f, c) <- IM.toList cluster
  , let home = majority IM.! c
  , held c home > held c (dirOf IM.! f)
  ]
 where
  held c d = IM.findWithDefault 0 d (IM.findWithDefault IM.empty c counts)
  counts = IM.fromListWith (IM.unionWith (+)) [(c, IM.singleton (dirOf IM.! f) (1 :: Int)) | (f, c) <- IM.toList cluster]
  majority = IM.map (fst . maximumBy (comparing (\(d, m) -> (m, negate d))) . IM.toList) counts
