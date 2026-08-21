-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | graph.request handler: decode, enforce the node/edge caps (the
-- real oversize guard — the envelope byte precheck is relaxed for
-- the trusted same-machine child, 2026-08-12 decision),
-- machine-check the boundary contract (node rows are
-- [lang,kind,flags] and edge rows [src,dst,kind,rung], endpoints and
-- pos indices in range, edges strictly ascending hence
-- duplicate-free) — then judge. The M5-2a stub refused here; M5-2g
-- replaced exactly that refusal with the computation, which lives
-- behind the exhaustive reference harness (core/test/) and takes its
-- knobs from CE.Graph.Cost — the only ablation targets.
module CE.Graph (respond) where

import CE.Graph.Build (Built (..), build)
import CE.Graph.Cost (assetKind, edgeCap, entryMask, granFile, minRung, nodeCap, sccFloor)
import qualified CE.Graph.Cycles as Cycles
import qualified CE.Graph.Dead as Dead
import qualified CE.Graph.Position as Position
import CE.Wire (Family (..), ascendingOn, respondWith)
import Data.Aeson
import qualified Data.ByteString.Char8 as B8
import qualified Data.ByteString.Lazy as BL
import Data.Foldable (asum)
import qualified Data.IntSet as IS
import Data.List (partition)

-- | Wire shape (design brief §2): index = node identity, nothing
-- text-shaped crosses. @unresolved@ is part of the family shape but
-- carries no validation obligation — it is the honest ledger, not an
-- input to judgment (§1 unknown-field rule). Absent @pos@ = counts
-- only.
data GraphReq = GraphReq
  { reqId :: Value
  , reqNodes :: [[Integer]]
  , reqEdges :: [[Integer]]
  , reqPos :: [Integer]
  }

instance FromJSON GraphReq where
  parseJSON = withObject "GraphReq" $ \o ->
    GraphReq
      <$> o .: "id"
      <*> o .: "nodes"
      <*> o .: "edges"
      <*> o .:? "pos" .!= []

-- | The shared cascade with this family's bindings (CE.Wire —
-- decode error prefix, caps, offence, replies all byte-identical to
-- the pre-skeleton cascade; the goldens are the proof).
respond :: String -> B8.ByteString -> Either (Maybe Value, String, String) B8.ByteString
respond proto =
  respondWith
    Family
      { famName = "graph"
      , famId = reqId
      , famOverCap = \req ->
          toInteger (length (reqNodes req)) > nodeCap
            || toInteger (length (reqEdges req)) > edgeCap
      , famOffence = violation
      , famDegraded = tooLarge proto
      , famJudged = result proto
      }

-- | First boundary-contract offender, if any — checked in request
-- order so the message is deterministic. Shape errors surface before
-- ordering errors, so the ascending pass only ever compares
-- well-formed four-tuples (list Ord is lexicographic).
violation :: GraphReq -> Maybe String
violation req =
  asum
    [ asum (zipWith nodeRow [0 :: Int ..] (reqNodes req))
    , asum (zipWith (edgeRow n) [0 :: Int ..] es)
    , ascendingOn "edge" id es
    , asum (zipWith (posRow n) [0 :: Int ..] ps)
    , -- ascending pos is also the reply BOUND (M5-close review MED:
      -- pos escaped the declared caps — a repeated-index list made
      -- the reply larger than the request without limit; strictly
      -- ascending indices in [0, n) cannot exceed nodeCap rows)
      ascendingOn "pos" id ps
    ]
 where
  n = fromIntegral (length (reqNodes req))
  es = reqEdges req
  ps = reqPos req

nodeRow :: Int -> [Integer] -> Maybe String
nodeRow i row = case row of
  [lang, kind, flags]
    | any (< 0) [lang, kind, flags] -> Just (label <> "negative field")
    | otherwise -> Nothing
  _ -> Just (label <> "malformed row (need [lang,kind,flags])")
 where
  label = "node " <> show i <> ": "

edgeRow :: Integer -> Int -> [Integer] -> Maybe String
edgeRow n i row = case row of
  [src, dst, kind, rung]
    | any (< 0) [src, dst, kind, rung] -> Just (label <> "negative field")
    | src >= n || dst >= n -> Just (label <> "endpoint out of range")
    | otherwise -> Nothing
  _ -> Just (label <> "malformed row (need [src,dst,kind,rung])")
 where
  label = "edge " <> show i <> ": "

-- notAscending moved to CE.Wire (its birthplace was here — the
-- tenth ratchet bite promoted it to the shared skeleton).

posRow :: Integer -> Int -> Integer -> Maybe String
posRow n i p
  | p < 0 || p >= n = Just ("pos " <> show i <> ": index out of range")
  | otherwise = Nothing

-- | The judged result. Knobs are the CE.Graph.Cost constants;
-- everything else is a function of the request, and the aeson
-- KeyMap encodes keys sorted — deterministic bytes by construction.
result :: String -> GraphReq -> B8.ByteString
result proto req =
  BL.toStrict . encode $
    object
      [ "proto" .= proto
      , "type" .= ("graph.result" :: String)
      , "id" .= reqId req
      , -- RG9 split, core-owned since 2.18.0 (batch-7 slice 4):
        -- only file-granularity verdicts land in the FAILING dead
        -- table; package/section verdicts are informational
        -- `reported` rows. The kind column always crossed the wire
        -- and was validated, then discarded — an unnamed Rust
        -- branch held the policy instead.
        "dead" .= [[toInteger i, toInteger v] | (i, v) <- deadRows]
      , "reported" .= [[toInteger i, toInteger v] | (i, v) <- reportedRows]
      , -- the zero-tolerance gate, named: any file-tier dead verdict
        -- fails `ce deadcode --check` — the exit was synthesized
        -- client-side before, where no ablation could see it
        "fail" .= not (null deadRows)
      , "pos" .= Position.positions b entryMask flagses (reqPos req)
      , "cycles"
          .= [ toJSON [toJSON (toInteger i), toJSON (map toInteger ms)]
             | (i, ms) <- Cycles.cycles sccFloor b
             ]
      , "counts"
          .= object
            [ "nodes" .= length (reqNodes req)
            , "edges" .= length (reqEdges req)
            , "kept" .= bKept b
            ]
      , "degraded" .= False
      ]
 where
  b = build minRung assetKind (length (reqNodes req)) (reqEdges req)
  flagses = [f | [_, _, f] <- reqNodes req]
  fileIdx = IS.fromList [i | (i, [_, k, _]) <- zip [0 ..] (reqNodes req), k == granFile]
  (deadRows, reportedRows) =
    partition (\(i, _) -> IS.member i fileIdx) (Dead.verdicts b entryMask flagses)

-- | Over-cap refusal: a well-formed degraded result, never a
-- truncated graph. counts echoes what arrived (informational);
-- kept = 0 because nothing was analyzed.
tooLarge :: String -> GraphReq -> B8.ByteString
tooLarge proto req =
  BL.toStrict . encode $
    object
      [ "proto" .= proto
      , "type" .= ("graph.result" :: String)
      , "id" .= reqId req
      , "dead" .= ([] :: [Value])
      , "reported" .= ([] :: [Value])
      , -- a gate that could not judge must never pass (the verdict
        -- family's P1 stance, applied here at 2.18.0): the degraded
        -- reply fails by itself
        "fail" .= True
      , "pos" .= ([] :: [Value])
      , "cycles" .= ([] :: [Value])
      , "counts"
          .= object
            [ "nodes" .= length (reqNodes req)
            , "edges" .= length (reqEdges req)
            , "kept" .= (0 :: Int)
            ]
      , "degraded" .= True
      , "reason" .= ("graph_too_large" :: String)
      ]
