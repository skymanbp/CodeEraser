-- | A second spelling of the `[rules.pack]` items (CE.Query.Pack) for
-- the pack battery: seeded fact tables, and each item's violations
-- computed straight off those tables as plain set comprehensions — no
-- Datalog, no prelude, no evaluator. The battery holds the core's
-- witness rows (the rendered program, lexed and judged through the real
-- respond) to these, table for table. A glob's members are drawn per
-- glob; the battery seats them under the set ids the core's lex reply
-- answers, the way the measuring side assembles the `set` table.
module ReferencePack (Tables (..), tables, cycles, fanIn, fanOut, layered, forbidden) where

import Data.Bits ((.&.))
import qualified Data.Map.Strict as M
import qualified Data.Set as S
import ReferenceFlowGen (lcgStep)

-- | One seeded table set: the nodes, which of them are files, each
-- node's directory, the reference arcs (with their kind and rung, two
-- kinds of one pair now and then), and each glob's members.
data Tables = Tables
  { tNodes :: [Integer]
  , tFiles :: S.Set Integer
  , tDir :: M.Map Integer Integer
  , tRefs :: S.Set [Integer]
  , tMembers :: M.Map String (S.Set Integer)
  }

-- | The batteries' one LCG from a seed: a stream of draws (its high
-- bits), each taken `mod` m below.
draws :: Integer -> [Integer]
draws seed = map (`div` 65536) (drop 1 (iterate lcgStep (seed * 2654435761 + 1)))

-- | The tables for a seed over the globs an item spells.
tables :: Integer -> [String] -> Tables
tables seed globs = Tables nodes files dirOf refs members
 where
  rs = draws seed
  pick i m = (rs !! i) `mod` m
  n = 5 + pick 0 5
  nodes = [0 .. n - 1]
  dirs = 2 + pick 1 3
  files = S.fromList [v | v <- nodes, pick (fromInteger (10 + v)) 6 /= 0]
  dirOf = M.fromList [(v, pick (fromInteger (30 + v)) dirs) | v <- nodes]
  pairs = [(a, b) | a <- nodes, b <- nodes]
  refs =
    S.fromList $
      concat
        [ [[a, b, 7, 1] | k .&. 3 == 0] <> [[a, b, 11, 2] | k .&. 15 == 1]
        | (i, (a, b)) <- zip [50 :: Int ..] pairs
        , let k = pick i 64
        , a /= b || k .&. 7 == 0
        ]
  members = M.fromList [(g, S.fromList [v | v <- nodes, pick (200 + 16 * gi + fromInteger v) 2 == 0]) | (gi, g) <- zip [0 ..] globs]

-- | The distinct (from, to) pairs any reference joins.
arcs :: Tables -> S.Set (Integer, Integer)
arcs t = S.fromList [(a, b) | a : b : _ <- S.toList (tRefs t)]

-- | `no_dir_cycles`: the directory pairs that reach each other through
-- the directory graph of the arcs, once per unordered pair.
cycles :: Tables -> S.Set [Integer]
cycles t = S.fromList [[0, d, e] | (d, e) <- S.toList reach, S.member (e, d) reach, d < e]
 where
  dirOf v = tDir t M.! v
  succs = M.fromListWith S.union [(dirOf a, S.singleton (dirOf b)) | (a, b) <- S.toList (arcs t), dirOf a /= dirOf b]
  next d = M.findWithDefault S.empty d succs
  -- breadth-first from each directory: everything it reaches
  from = walk S.empty . next
  walk seen frontier
    | S.null frontier = seen
    | otherwise = let seen' = S.union seen frontier in walk seen' (S.unions (map next (S.toList frontier)) S.\\ seen')
  reach = S.fromList [(d, e) | d <- M.keys succs, e <- S.toList (from d)]

-- | `fan_in_max n`: each file and the other files referencing it, when
-- more than n.
fanIn :: Integer -> Tables -> S.Set [Integer]
fanIn n t = S.fromList [[0, g, c] | g <- S.toList (tFiles t), let c = toInteger (S.size (S.fromList [f | (f, g') <- S.toList (arcs t), g' == g, f /= g, S.member f (tFiles t)])), c > n]

-- | `fan_out_max n`: each file and the other files it references, when
-- more than n — fan-in over the arcs turned around.
fanOut :: Integer -> Tables -> S.Set [Integer]
fanOut n t = fanIn n t {tRefs = S.map (\r -> case r of a : b : rest -> b : a : rest; _ -> r) (tRefs t)}

-- | `layers gs`: goal k is the k-th ordered pair (lower i, upper j),
-- i > j, in the order the item renders them; a row per arc from a
-- member of the lower glob to a member of the upper one.
layered :: [String] -> Tables -> S.Set [Integer]
layered gs t = S.unions [crossing k lower upper t | (k, (lower, upper)) <- zip [0 ..] ordered]
 where
  ordered = [(lo, up) | (i, lo) <- zip [0 :: Int ..] gs, (j, up) <- zip [0 ..] gs, i > j]

-- | `forbid` pairs: goal k is the k-th pair.
forbidden :: [(String, String)] -> Tables -> S.Set [Integer]
forbidden ps t = S.unions [crossing k from to t | (k, (from, to)) <- zip [0 ..] ps]

-- | Goal k's rows: an arc from a FILE in the first glob to a FILE in
-- the second (a glob also holds nodes that are no file — the tables
-- seat every node in a glob's members, the way the measuring side seats
-- assets and prose pages).
crossing :: Integer -> String -> String -> Tables -> S.Set [Integer]
crossing k from to t = S.fromList [[k, f, g] | (f, g) <- S.toList (arcs t), S.member f (mem from), S.member g (mem to), S.member f (tFiles t), S.member g (tFiles t)]
 where
  mem g = M.findWithDefault S.empty g (tMembers t)
