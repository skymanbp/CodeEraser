-- | The S5 staleness machinery, repatriated whole (batch-7 slice
-- 11, 2.23.0) and split out at the 300-line wall (E01, the
-- CE.Structure.Knobs precedent): the raw-table validators and the
-- stale predicate live together because they describe ONE contract
-- — what a legal staleDocRows/staleEdgeRows pair looks like, and
-- what it means.
module CE.Structure.Stale (docRowSpec, nonDescDir, edgesOffence, effectiveStale) where

import Data.Foldable (asum)
import qualified Data.List as L

-- | The staleDocRows spec tuple for the shared dirRow checker:
-- arity 2, and docTs >= 0 (0 = doc unchanged in the window, the one
-- sentinel). dirId range and negativity ride the generic checks.
docRowSpec :: (Int, String, [Integer] -> Maybe String)
docRowSpec = (2, "staleDocRows", ok)
 where
  ok row = case row of
    [_, ts] | ts < 0 -> Just "docTs below 0"
            | otherwise -> Nothing
    _ -> Nothing

-- | Doc identity is the ROW INDEX (dirId repeats — one dir holds
-- many docs); the canonical order is non-descending dirId, so a
-- shuffled table still refuses deterministically.
nonDescDir :: [[Integer]] -> Maybe String
nonDescDir rows = case [() | ([a, _], [b, _]) <- zip rows (drop 1 rows), a > b] of
  [] -> Nothing
  _ -> Just "staleDocRows: dirId not non-descending"

-- | First staleEdgeRows offender: docIdx must land inside the doc
-- table, targetTs >= 1 (an edge exists only for a target that
-- CHANGED in the window — 0 would be the sentinel leaking sides).
edgesOffence :: Integer -> [[Integer]] -> Maybe String
edgesOffence nDocs = asum . zipWith edgeOk [0 :: Int ..]
 where
  edgeOk i row = case row of
    [docIdx, targetTs]
      | docIdx < 0 || docIdx >= nDocs ->
          Just ("staleEdgeRows " <> show i <> ": docIdx out of range")
      | targetTs < 1 ->
          Just ("staleEdgeRows " <> show i <> ": targetTs below 1")
      | otherwise -> Nothing
    _ -> Just ("staleEdgeRows " <> show i <> ": malformed row (need [docIdx,targetTs])")

-- | The axis-5 fact table the judging machinery consumes: the raw
-- tables JUDGE when they ride — the per-dir [dirId, stale, total]
-- rows derive HERE, in the core — and the pre-judged staleDocs
-- table stays legal for one minor, yielding to them.
effectiveStale :: Maybe [[Integer]] -> Maybe [[Integer]] -> [[Integer]] -> Maybe [[Integer]]
effectiveStale preJudged rawDocs edges = case rawDocs of
  Nothing -> preJudged
  Just docs -> Just (deriveStale docs edges)

-- | The S5 predicate, repatriated: a doc is stale iff SOME changed
-- target moved after the doc's own last window change — strict >,
-- so a same-commit update of both is NOT stale, and docTs 0 (doc
-- unchanged in the window) makes any changed target stale. Docs
-- without edges are counted, never stale (no evidence). Output:
-- per-dir [dirId, stale, total], dirId ASCENDING (the BTreeMap
-- order the Rust pre-judged table always shipped).
deriveStale :: [[Integer]] -> [[Integer]] -> [[Integer]]
deriveStale docs edges =
  [ [d, staleIn d, totalIn d]
  | d <- L.nub (L.sort [dirId | [dirId, _] <- docs])
  ]
 where
  staleEdges i = [ts | [j, ts] <- edges, j == i]
  isStale (i, [_, docTs]) = any (\ts -> docTs == 0 || ts > docTs) (staleEdges i)
  isStale _ = False
  indexed = zip [0 :: Integer ..] docs
  staleIn d = toInteger (length [() | row@(_, [d', _]) <- indexed, d' == d, isStale row])
  totalIn d = toInteger (length [() | [d', _] <- docs, d' == d])
