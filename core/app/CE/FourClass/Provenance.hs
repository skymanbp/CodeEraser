-- | Phases 2 and 3 over the anchored sites, then the assembled
-- result. Both phases are consequences of the cost model: adding a
-- line to an already-open site costs movedCost < plainCost with no
-- new site cost, so it is always profitable.
--
-- The asymmetry between them is the product thesis: on the removal
-- side, "the content left its home" is itself provenance (bulk
-- removal of copies is the normal shape of a de-duplication
-- refactor); on the addition side, a fresh line duplicating removed
-- content is DUPLICATION — the signal this product exists to catch —
-- so additions require site or edge evidence.
module CE.FourClass.Provenance (classify) where

import CE.FourClass.Anchor (runsOf, sites)
import CE.FourClass.Wire
import Data.List (sortOn)
import qualified Data.Map.Strict as M
import qualified Data.Set as S

type Mark = (Int, Int) -- (pair, line)

classify :: Request -> Result
classify req = Result (reqId req) moved sortedBlocks reason
 where
  ps = reqPairs req
  (blocks, capped) = sites ps
  sortedBlocks =
    sortOn (\b -> (bFromPair b, bFromLines b, bToPair b, bToLines b)) blocks
  reason = if capped then Just "bucket_cap" else Nothing
  outMarks = anchorMarks bFromPair bFromLines blocks `S.union` phase3 ps blocks
  inMarks = anchorMarks bToPair bToLines blocks `S.union` phase2 ps blocks
  moved =
    [ (pIdx p, outs, ins)
    | p <- ps
    , let outs = [l | (l, _) <- pRem p, (pIdx p, l) `S.member` outMarks]
    , let ins = [l | (l, _) <- pAdd p, (pIdx p, l) `S.member` inMarks]
    , not (null outs && null ins)
    ]

anchorMarks :: (Block -> Int) -> (Block -> [Int]) -> [Block] -> S.Set Mark
anchorMarks pair lns blocks = S.fromList [(pair b, l) | b <- blocks, l <- lns b]

-- | Site extension, run-scoped: an unclaimed added line inside a
-- contiguous added run that already contains an anchored line, whose
-- hash occurs among the leftover removals of a pair with an
-- established edge into this pair. Recovers the one-line tails of
-- proven relocations without licensing arbitrary file-wide claims.
phase2 :: [Pair] -> [Block] -> S.Set Mark
phase2 ps blocks =
  S.fromList
    [ (q, l)
    | p <- ps
    , let q = pIdx p
    , (l, h) <- pAdd p
    , (q, l) `S.notMember` anchored
    , maybe False (\r -> (q, r) `S.member` anchoredRuns) (M.lookup (q, l) runId)
    , any (\src -> h `S.member` remHashesOf src) (sourcesInto q)
    ]
 where
  anchored = anchorMarks bToPair bToLines blocks
  runId =
    M.fromList
      [ ((pIdx p, l), r)
      | p <- ps
      , (r, run) <- zip [0 :: Int ..] (runsOf (pAdd p))
      , (l, _) <- run
      ]
  anchoredRuns =
    S.fromList
      [(q, r) | (q, l) <- S.toList anchored, Just r <- [M.lookup (q, l) runId]]
  edges = S.toList (S.fromList [(bFromPair b, bToPair b) | b <- blocks])
  sourcesInto q = [src | (src, dst) <- edges, dst == q]
  remHashesOf src = M.findWithDefault S.empty src remByPair
  remByPair =
    M.fromListWith S.union [(pIdx p, S.fromList (map snd (pRem p))) | p <- ps]

-- | Asymmetric source attribution: a leftover removed line whose
-- content sits inside an accepted block landing in a DIFFERENT pair
-- is a further copy of relocated content — moved-out.
phase3 :: [Pair] -> [Block] -> S.Set Mark
phase3 ps blocks =
  S.fromList
    [ (pp, l)
    | p <- ps
    , let pp = pIdx p
    , (l, h) <- pRem p
    , any (\(dst, hs) -> dst /= pp && h `S.member` hs) landed
    ]
 where
  addHash = M.fromList [((pIdx p, l), h) | p <- ps, (l, h) <- pAdd p]
  landed =
    M.toList $
      M.fromListWith
        S.union
        [ (bToPair b, S.fromList [addHash M.! (bToPair b, l) | l <- bToLines b])
        | b <- blocks
        ]
