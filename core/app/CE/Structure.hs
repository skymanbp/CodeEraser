-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | structure.request handler (M6 S2, design booklet §4): decode
-- the tree-scale fact tables — dense directory nodes, name-pattern
-- distributions, convention bits, per-file reference splits —
-- enforce the node cap (over-cap = a complete degraded reply that
-- FAILS, the P1 posture), machine-check the boundary contract in
-- request order, then judge the axes — five S2 axes always, plus
-- staleness (5) and redundancy (6) when their S3 fact tables ride
-- the wire — and the headline entropy rows. Names and paths never
-- cross (§5.9.2): the report's
-- vocabulary is dense ids, codes and counts, re-labelled by the
-- Rust side that kept the names. Knob rows ride the established
-- [code, value] grammar; ce.toml is the source, Cost.hs the
-- defaults, and the reply echoes the effective set whole.
module CE.Structure (respond) where

import CE.Structure.Axes (Facts (..), Knobs (kScale, kViolCost), axes, entropyRows, findings)
import CE.Structure.Cost (structNodeCap)
import CE.Structure.Declared (declaredRows)
import CE.Structure.Knobs (effective, knobTable, knobsOffence)
import CE.Structure.Split (splitOffence, splitRows)
import qualified CE.Structure.Stale as Stale
import CE.Wire (Family (..), ascendingOn, respondWith)
import Data.Aeson
import qualified Data.ByteString.Char8 as B8
import qualified Data.ByteString.Lazy as BL
import Data.Foldable (asum)
import qualified Data.IntMap.Strict as IM

data StructReq = StructReq
  { reqId :: Value
  , reqNodes :: [[Integer]]
  , reqPatterns :: [[Integer]]
  , reqConventions :: [[Integer]]
  , reqFileRefs :: [[Integer]]
  , reqDeclared :: [[Integer]]
  , reqStaleDocs :: Maybe [[Integer]]
  -- ^ The PRE-JUDGED per-dir rows [dirId, stale, total] — legal for
  -- one minor beside the raw tables below (2.23.0, batch-7 slice
  -- 11); when both ride, the raw tables judge.
  , -- the RAW staleness facts (2.23.0, additive): one row per md
    -- doc that HAS reference targets — [dirId, docTs], docTs = the
    -- doc's newest change inside the churn window, 0 = unchanged
    -- (the one sentinel, documented); doc identity = row index
    -- (dense by construction, the graph node discipline). Edges
    -- [docIdx, targetTs] exist only for targets that CHANGED in the
    -- window (targetTs >= 1) — an unchanged target can never make a
    -- doc stale, so shipping it would be dead weight.
    reqStaleDocRows :: Maybe [[Integer]]
  , reqStaleEdges :: [[Integer]]
  , reqRedundancy :: Maybe [[Integer]]
  -- ^ Both Maybe, not defaulted: an absent table means its axis is
  -- not judged, an empty one that it judged clean — the churn-table
  -- honesty (absence is spoken, never zero-filled).
  , -- the split-ROI advisory tables (plan v2.6 §C 2.14.0, clones/
    -- churn v2.7 ② 2.15.0 — all additive): seamFiles is the
    -- presence anchor — the two reply keys exist exactly when it
    -- rides; the four unit/edge tables default empty
    reqSeamFiles :: Maybe [[Integer]]
  , reqSeamUnits :: [[Integer]]
  , reqSeamRefs :: [[Integer]]
  , reqSeamClones :: [[Integer]]
  , reqSeamChurn :: [[Integer]]
  , reqKnobs :: [[Integer]]
  }

instance FromJSON StructReq where
  parseJSON = withObject "StructReq" $ \o ->
    StructReq
      <$> o .: "id"
      <*> o .: "nodes"
      <*> o .:? "patterns" .!= []
      <*> o .:? "conventions" .!= []
      <*> o .:? "fileRefs" .!= []
      <*> o .:? "declared" .!= []
      <*> o .:? "staleDocs"
      <*> o .:? "staleDocRows"
      <*> o .:? "staleEdgeRows" .!= []
      <*> o .:? "redundancy"
      <*> o .:? "seamFiles"
      <*> o .:? "seamUnits" .!= []
      <*> o .:? "seamRefs" .!= []
      <*> o .:? "seamClones" .!= []
      <*> o .:? "seamChurn" .!= []
      <*> o .:? "knobs" .!= []

-- | The four unit/edge tables as ONE bundle — the same tuple
-- CE.Structure.Split consumes on both its faces (offence + rows).
seamTables :: StructReq -> ([[Integer]], [[Integer]], [[Integer]], [[Integer]])
seamTables req =
  (reqSeamUnits req, reqSeamRefs req, reqSeamClones req, reqSeamChurn req)

-- | The shared cascade with this family's bindings (CE.Wire).
respond :: String -> B8.ByteString -> Either (Maybe Value, String, String) B8.ByteString
respond proto =
  respondWith
    Family
      { famName = "structure"
      , famId = reqId
      , -- the seam tables count toward the same cap (C15: a declared
        -- cap that misses a request dimension walks it uncapped)
        famOverCap = \req ->
          let (u, r, c, h) = seamTables req
              seamRows = maybe 0 length (reqSeamFiles req) + sum (map length [u, r, c, h])
           in toInteger (length (reqNodes req) + seamRows) > structNodeCap
      , famOffence = violation
      , famDegraded = \req -> reply proto req (effective []) True
      , famJudged = \req -> reply proto req (effective (reqKnobs req)) False
      }

-- | First boundary-contract offender in request order — the three
-- dir-keyed tables walk ONE loop over their spec rows (the twelfth
-- bite's repayment shape: the per-table asum/ascending pair was the
-- clone).
violation :: StructReq -> Maybe String
violation req =
  asum
    ( asum (zipWith nodeRow [0 :: Int ..] (reqNodes req))
        : depthChain (reqNodes req)
        : [ asum
              [ asum (zipWith (dirRow n spec) [0 :: Int ..] rows)
              , ascendingOn nm proj rows
              ]
          | (spec@(_, nm, _), proj, rows) <- dirTables
          ]
        <> [ splitOffence sf (seamTables req)
           | Just sf <- [reqSeamFiles req]
           ]
        <> [ asum (zipWith (dirRow n Stale.docRowSpec) [0 :: Int ..] docRows)
           , Stale.nonDescDir docRows
           , Stale.edgesOffence (toInteger (length docRows)) (reqStaleEdges req)
           , knobsOffence (reqKnobs req)
           ]
    )
 where
  n = toInteger (length (reqNodes req))
  docRows = concat (reqStaleDocRows req)
  dirTables =
    [ ((3, "pattern", patternOk), take 2, reqPatterns req)
    , ((2, "convention", convOk), take 1, reqConventions req)
    , ((4, "fileRefs", refsOk), take 3, reqFileRefs req)
    , ((2, "declared", declOk), take 1, reqDeclared req)
    , ((3, "staleDocs", staleOk), take 1, concat (reqStaleDocs req))
    , ((3, "redundancy", noExtra), take 1, concat (reqRedundancy req))
    ]
  noExtra _ = Nothing
  staleOk row = case row of
    [_, s, total] | total < 1 -> Just "total below 1"
                  | s > total -> Just "stale above total"
                  | otherwise -> Nothing
    _ -> Nothing
  patternOk row = case row of
    [_, code, count] | code > 6 -> Just "unknown pattern code"
                     | count < 1 -> Just "count below 1"
                     | otherwise -> Nothing
    _ -> Nothing
  convOk row = case row of
    [_, bits] | bits < 1 || bits > 3 -> Just "bits outside 1..3"
              | otherwise -> Nothing
    _ -> Nothing
  refsOk row = case row of
    [_, _, _, count] | count < 1 -> Just "count below 1"
                     | otherwise -> Nothing
    _ -> Nothing
  declOk row = case row of
    [_, w] | w < 1 -> Just "weight below 1"
           | otherwise -> Nothing
    _ -> Nothing

-- | One dense node row: id == index, parent < id (root 0 loops to
-- itself); depth is chained against the parent row by depthChain —
-- the shape that makes the tree a tree by construction.
nodeRow :: Int -> [Integer] -> Maybe String
nodeRow i row = case row of
  [nid, parent, depth, subdirs, files]
    | any (< 0) [nid, parent, depth, subdirs, files] -> Just (label <> "negative field")
    | nid /= toInteger i -> Just (label <> "index mismatch")
    | i == 0 && (parent /= 0 || depth /= 0) -> Just (label <> "root must self-loop at depth 0")
    | i > 0 && parent >= toInteger i -> Just (label <> "parent not before child")
    | otherwise -> Nothing
  _ -> Just (label <> "malformed row (need [id,parent,depth,subdirs,files])")
 where
  label = "node " <> show i <> ": "

-- | depth == parent.depth + 1 for every non-root row. nodeRow's
-- docstring CLAIMED this held by position, but nothing checked it —
-- a forged depth (node row [1,0,999,0,1]) rode straight into the
-- geometry axes and moved the score (review 2026-08-20 #6,
-- reproduced by driving the core directly). Runs after the per-row
-- pass in the asum, so every row here is already well-formed.
depthChain :: [[Integer]] -> Maybe String
depthChain rows = asum (zipWith step [1 :: Int ..] (drop 1 rows))
 where
  table = IM.fromList [(fromInteger nid, d) | [nid, _, d, _, _] <- rows]
  step i row = case row of
    [_, parent, depth, _, _]
      | IM.lookup (fromInteger parent) table /= Just (depth - 1) ->
          Just ("node " <> show i <> ": depth is not parent depth + 1")
    _ -> Nothing

-- | Shared shape for the dir-keyed tables — the table's identity
-- travels as ONE spec tuple (arity, name, extra rule), which also
-- keeps the checker under the repo's own param gate.
dirRow :: Integer -> (Int, String, [Integer] -> Maybe String) -> Int -> [Integer] -> Maybe String
dirRow n (arity, name, extra) i row = case row of
  (d : _)
    | length row /= arity -> Just (label <> "malformed row")
    | any (< 0) row -> Just (label <> "negative field")
    | d >= n -> Just (label <> "dir out of range")
    | Just why <- extra row -> Just (label <> why)
    | otherwise -> Nothing
  [] -> Just (label <> "malformed row")
 where
  label = name <> " " <> show i <> ": "

-- knobsOffence / knobTable / effective live in CE.Structure.Knobs
-- (E01 split at the 300-line wall, the CE.Verdict.Knobs precedent).

-- | The judged reply: five to seven axis rows (the two conditional
-- axes join when their tables rode the wire), the Score.hs fold at
-- equal weight over the judged axis count, the headline entropy rows
-- and the sparse findings — plus the FULL effective knob echo.
-- fail = degraded alone in S2 (the report-only stance: the CLI
-- gates nothing until a score floor lands with S3+); a degraded
-- reply carries fail=true (P1) and echoes the defaults. The S3
-- A-layer keys (divergence + deviations) exist ONLY when the
-- request declares a layout — an undeclared request answers the
-- S2 shape byte for byte, and a degraded reply drops the
-- declaration with the rest of the facts.
reply :: String -> StructReq -> Knobs -> Bool -> B8.ByteString
reply proto req k degraded =
  BL.toStrict . encode . object $
    [ "proto" .= proto
    , "type" .= ("structure.result" :: String)
    , "id" .= reqId req
    , "axes" .= [[c, p] | (c, p) <- pens]
    , "score" .= score
    , "entropy" .= entropyRows facts
    , "findings" .= findings k facts
    ]
      <> declaredKeys
      <> splitKeys
      <> [ "fail" .= degraded
         , "knobs" .= [[c, g k] | (c, g, _) <- knobTable]
         , "degraded" .= degraded
         ]
      <> ["reason" .= ("structure_too_large" :: String) | degraded]
 where
  facts =
    if degraded
      then Facts [] [] [] [] Nothing Nothing
      else
        Facts
          (reqNodes req)
          (reqPatterns req)
          (reqConventions req)
          (reqFileRefs req)
          (Stale.effectiveStale (reqStaleDocs req) (reqStaleDocRows req) (reqStaleEdges req))
          (reqRedundancy req)
  declaredKeys = case declaredRows (fNodes facts) (if degraded then [] else reqDeclared req) of
    Nothing -> []
    Just (divergence, deviations) ->
      ["divergence" .= divergence, "deviations" .= deviations]
  -- the split-ROI keys exist exactly when seamFiles rode the wire
  -- (the divergence precedent); a degraded reply drops them with
  -- the rest of the facts
  splitKeys = case (if degraded then Nothing else reqSeamFiles req) of
    Nothing -> []
    Just sf ->
      let (cands, exempts) = splitRows k sf (seamTables req)
       in ["splitCandidates" .= cands, "sizeExempt" .= exempts]
  pens = axes k facts
  raw = sum [p * kViolCost k | (_, p) <- pens]
  score = max 0 (kScale k - raw `div` toInteger (length pens))
