-- | The liveness fixpoint's two spellings (plan v2.35 cleanup C3): the
-- product solves `in = transfer (∪ in of successors)` with a worklist;
-- until then it swept the whole graph from the old map each round
-- (Jacobi), which moves liveness one edge per sweep — Θ(n²) on a
-- straight run of n statements. Both start from the empty sets and the
-- transfer is monotone, so both reach the least fixpoint; this battery
-- holds the product to the retired sweep, kept here verbatim, on every
-- unit of the flow generator's seeded requests (ten thousand and more),
-- node for node.
module FlowLiveProps (battery) where

import CE.Flow.Cfg (Cfg (..), cfg)
import CE.Flow.Contract (FlowReq (..))
import CE.Flow.Cost
import CE.Flow.Live (liveIn)
import CE.Flow.Tree (Unit (..), Use (..))
import Data.Aeson (Result (..), fromJSON)
import qualified Data.IntMap.Strict as IM
import qualified Data.IntSet as IS
import ReferenceFlowGen (S (..), batch, request, runG)
import WireHarness (runChecks)

battery :: IO Bool
battery =
  runChecks
    [ ("at least 10,000 generated units", length units >= 10000)
    , ("the worklist fixpoint = the retired Jacobi sweep's, node for node, on every generated unit", all (\(u, g) -> liveIn u g == fst (jacobi u g)) units)
    , ("the units reach fixpoints that took the sweep more than three rounds", any (\(u, g) -> snd (jacobi u g) > 3) units)
    , ("the units hold live sets that are not empty", any (\(u, g) -> any (not . IS.null) (IM.elems (liveIn u g))) units)
    ]

-- | Every unit of the generator's requests, seeded the way `batches`
-- seeds its two hundred, over enough batches for ten thousand units.
units :: [(Unit, Cfg)]
units =
  [ (u, cfg u)
  | n <- [1 .. 5200]
  , Success req <- [fromJSON (request (runG (batch n) (S (n * 7919 + 17) 0 2 2)))]
  , Right us <- [unitsOf req]
  , u <- IM.elems us
  ]

-- | CE.Flow.Live.liveIn before C3, verbatim, with the sweeps it took.
jacobi :: Unit -> Cfg -> (IM.IntMap IS.IntSet, Int)
jacobi u g = go (1 :: Int) (IM.fromList [(node, IS.empty) | node <- [0 .. nodeCount g - 1]])
 where
  go k live
    | live' == live = (live, k)
    | otherwise = go (k + 1) live'
   where
    live' = IM.mapWithKey (\node _ -> transfer (accesses node) (liveOut live node)) live
  liveOut live node = IS.unions [IM.findWithDefault IS.empty s live | s <- IM.findWithDefault [] node (succs g)]
  accesses node = IM.findWithDefault [] node (uses u)
  transfer acc out = foldr step out acc
  step (Use _ v m) live
    | m == modeWrite = IS.delete v live
    | otherwise = IS.insert v live
