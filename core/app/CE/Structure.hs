-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | structure.request handler (M6 S2, design booklet §4): decode
-- the tree-scale fact tables — dense directory nodes, name-pattern
-- distributions, convention bits, per-file reference splits —
-- enforce the node cap (over-cap = a complete degraded reply that
-- FAILS, the P1 posture), machine-check the boundary contract in
-- request order, then judge the axes — five S2 axes always, plus
-- staleness (5), redundancy (6) and modularity (7) when their fact
-- tables ride the wire — and the headline entropy rows. Names and
-- paths never cross (§5.9.2): the report's
-- vocabulary is dense ids, codes and counts, re-labelled by the
-- Rust side that kept the names. Knob rows ride the established
-- [code, value] grammar; ce.toml is the source, Cost.hs the
-- defaults, and the reply echoes the effective set whole. The
-- request record and its decoder live in CE.Structure.Request
-- (split at the 300-line wall, plan v2.30 step 7b).
module CE.Structure (respond) where

import CE.Structure.Axes (Facts (..), Knobs (kScale, kViolCost), axes, entropyRows, findings)
import CE.Structure.Cost (structNodeCap, structViolCostNeutral)
import qualified CE.Structure.Modularity as Mod
import CE.Verdict.Score (chargeAt)
import CE.Structure.Declared (declaredRows)
import CE.Structure.Knobs (effective, knobTable, knobsOffence)
import CE.Structure.Request (StructReq (..), pathFault, pathKeys, pathOffence, patternsOf, seamTables)
import CE.Structure.Shape (shapeBitsCap)
import CE.Structure.Split (splitOffence, splitRows)
import qualified CE.Structure.Stale as Stale
import CE.Wire (Family (..), respondWith, tableOffence)
import CE.Wire.Retired (retired)
import Data.Aeson
import qualified Data.ByteString.Char8 as B8
import qualified Data.ByteString.Lazy as BL
import Data.Foldable (asum)
import qualified Data.IntMap.Strict as IM
import Data.Maybe (fromMaybe)

-- | The shared cascade with this family's bindings (CE.Wire).
respond :: String -> B8.ByteString -> Either (Maybe Value, String, String) B8.ByteString
respond proto =
  respondWith
    Family
      { famName = "structure"
      , famId = reqId
      , famOverCap = \req -> priced req > structNodeCap
      , famOffence = violation
      , famDegraded = \req -> reply proto req (effective []) True
      , famJudged = \req -> maybe (reply proto req (effective (reqKnobs req)) False) (faultReply proto req) (pathFault req)
      }

-- | The rows the cap prices: the nodes, the seam tables and the dir-edge
-- table together (C15: a declared cap that misses a request dimension
-- walks it uncapped).
priced :: StructReq -> Integer
priced req = toInteger (length (reqNodes req) + seamRows + edgeRows)
 where
  (u, r, c, h) = seamTables req
  seamRows = maybe 0 length (reqSeamFiles req) + sum (map length [u, r, c, h])
  edgeRows = maybe 0 length (reqDirEdges req)

-- | A path-form request whose path the tree cannot place: the fault in
-- place of a judgment (the measuring side stops on it, as it stopped on
-- its own message before the core built the tree).
faultReply :: String -> StructReq -> String -> B8.ByteString
faultReply proto req fault =
  BL.toStrict . encode . object $
    ["proto" .= proto, "type" .= ("structure.result" :: String), "id" .= reqId req, "fault" .= fault, "degraded" .= False]

-- | First boundary-contract offender in request order — the three
-- dir-keyed tables walk ONE loop over their spec rows (the twelfth
-- bite's repayment shape: the per-table asum/ascending pair was the
-- clone). The name-pattern distribution rides one road since 8.0.0:
-- a request that still carries the producer-classified `patterns`
-- table is refused by name before anything is read (plan v2.32 step 6;
-- 7.2.0 to 7.10.0 refused it only beside `patternShapes`).
violation :: StructReq -> Maybe String
violation req =
  asum
    ( pathOffence req : retired "patterns" "the core classifies patternShapes" (reqPatternsSent req)
        : asum (zipWith nodeRow [0 :: Int ..] (reqNodes req))
        : depthChain (reqNodes req)
        : [ tableOffence nm proj (dirRow n spec) rows
          | (spec@(_, nm, _), proj, rows) <- dirTables
          ]
        <> [ Mod.crossTableOffence (reqFileRefs req) rows
           | Just rows <- [reqDirEdges req]
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
    [ ((3, "patternShapes", capOk "shape bits outside 0..127" shapeBitsCap), take 2, fromMaybe [] (reqShapes req))
    , ((2, "convention", convOk), take 1, reqConventions req)
    , ((4, "fileRefs", refsOk), take 3, reqFileRefs req)
    , ((2, "declared", declOk), take 1, reqDeclared req)
    , ((3, "redundancy", noExtra), take 1, concat (reqRedundancy req))
    , (Mod.edgeRowSpec n, take 2, concat (reqDirEdges req))
    ]
  noExtra _ = Nothing
  -- the [dir, key, count] reading: the key under its cap, the count
  -- at least one
  capOk why cap row = case row of
    [_, v, count] | v > cap -> Just why
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

-- | The judged reply: five to eight axis rows (the three conditional
-- axes join when their tables rode the wire), the Score.hs fold at
-- equal weight over the judged axis count, the headline entropy rows
-- and the sparse findings — plus the FULL effective knob echo.
-- fail = degraded alone (the report-only stance, by ruling: no
-- score floor — v2.22 close-out, O53; the CLI gates nothing); a degraded
-- reply carries fail=true (P1) and echoes the defaults. The S3
-- A-layer keys (divergence + deviations) exist ONLY when the
-- request declares a layout — an undeclared request answers the
-- S2 shape byte for byte, and a degraded reply drops the
-- declaration with the rest of the facts. The shape road (7.2.0)
-- echoes its row count as `patternShapes` exactly when it rode and
-- was judged, so the producer can pin that a core read the facts —
-- a pre-7.2.0 core would drop the table under the unknown-field rule
-- and judge S1 on nothing.
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
      <> ["fail" .= degraded, "knobs" .= [[c, g k] | (c, g, _) <- knobTable], "degraded" .= degraded]
      <> ["reason" .= ("structure_too_large" :: String) | degraded]
      <> ["patternShapes" .= length rows | not degraded, Just rows <- [reqShapes req]]
      <> pathKeys (if degraded then priced req else 0) req
 where
  facts =
    if degraded
      then Facts [] [] [] [] Nothing Nothing Nothing
      else
        Facts
          (reqNodes req)
          (patternsOf req)
          (reqConventions req)
          (reqFileRefs req)
          (Stale.effectiveStale (reqStaleDocRows req) (reqStaleEdges req))
          (reqRedundancy req)
          (reqDirEdges req)
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
  -- the DENSITY fold (2.26.0, batch 9 P9 — the user's ruling):
  -- every axis penalty is a flagged-directory count, so each pairs
  -- with the SAME opportunity — the directory total — and maps
  -- through the verdict family's charge law. The raw-mass fold this
  -- replaces is the exact shape the batch-6 field test retired
  -- there: mean flagged count ~100 pinned the score at 0 and grew
  -- linearly with repo size. Axis rows now carry CHARGES (‰ of
  -- scale), the verdict-family grammar.
  pens = [(c, chargeAt (kScale k) (fromInteger p) nDirs) | (c, p) <- axes k facts]
  nDirs = toInteger (length (fNodes facts))
  raw = sum [p * kViolCost k | (_, p) <- pens]
  score = max 0 (kScale k - raw `div` (structViolCostNeutral * toInteger (length pens)))
