-- | Reachability over a unit's control-flow graph (design booklet
-- §5.2): the statements no path from the entry visits, reported as
-- maximal runs of consecutive seqs — a pre-order numbering makes an
-- unreachable statement's whole subtree one run, so an if that
-- follows a return is one finding, not one per line of it.
module CE.Flow.Reach (reachable, runs) where

import CE.Flow.Cfg (Cfg (..))
import qualified Data.IntMap.Strict as IM
import qualified Data.IntSet as IS

-- | Every node a path from the entry visits.
reachable :: Cfg -> IS.IntSet
reachable g = go (IS.singleton (entry g)) [entry g]
 where
  go seen [] = seen
  go seen (node : queue) = go (foldr IS.insert seen fresh) (queue <> fresh)
   where
    fresh = filter (`IS.notMember` seen) (IM.findWithDefault [] node (succs g))

-- | The maximal runs `(first, last)` of statement seqs below `n` that
-- the set does not contain, ascending.
runs :: Int -> IS.IntSet -> [(Int, Int)]
runs n seen = go [s | s <- [0 .. n - 1], IS.notMember s seen]
 where
  go [] = []
  go (s : rest) = let (more, others) = span' s rest in (s, s + length more) : go others
  span' s = spanFrom (s + 1)
  spanFrom _ [] = ([], [])
  spanFrom expect (x : xs)
    | x == expect = let (a, b) = spanFrom (expect + 1) xs in (x : a, b)
    | otherwise = ([], x : xs)
