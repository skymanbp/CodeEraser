-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | graph.request judgment: take a request the boundary contract has
-- already vouched for (CE.Graph.Contract) and answer it. The M5-2a
-- stub refused here; M5-2g replaced exactly that refusal with the
-- computation, which lives behind the exhaustive reference harness
-- (core/test/) and takes its knobs from CE.Graph.Cost — the only
-- ablation targets. Decode and contract checking moved out at the
-- 300-line dogfood wall when the 4.1.0 symbol table arrived.
module CE.Graph (respond) where

import CE.Graph.Build (Built (..), build, reachFrom)
import CE.Graph.Contract (GraphReq (..), symRows, unresRows, violation)
import CE.Graph.Cost (assetKind, confidence, edgeCap, entryMask, exportVisBit, granFile, minRung, nodeCap, publicFlagBit, refdefKind, roleBits, sccFloor, symCap)
import qualified CE.Graph.Cycles as Cycles
import qualified CE.Graph.Dead as Dead
import qualified CE.Graph.Position as Position
import CE.Wire (Family (..), respondWith)
import Data.Aeson
import qualified Data.ByteString.Char8 as B8
import qualified Data.ByteString.Lazy as BL
import qualified Data.IntSet as IS
import Data.List (partition)

-- | The shared cascade with this family's bindings (CE.Wire —
-- decode error prefix, caps, offence, replies all byte-identical to
-- the pre-skeleton cascade; the goldens are the proof).
respond :: String -> B8.ByteString -> Either (Maybe Value, String, String) B8.ByteString
respond proto =
  respondWith
    Family
      { famName = "graph"
      , famId = reqId
      , -- the unres ledger counts toward a cap too (the scan C15
        -- discipline: every request dimension is priced) — validation
        -- bounds it to seven rows, but the cap must not need that
        famOverCap = \req ->
          toInteger (length (reqNodes req) + length (unresRows req)) > nodeCap
            || toInteger (length (reqEdges req)) > edgeCap
            || toInteger (length (symRows req)) > symCap
      , famOffence = violation
      , famDegraded = tooLarge proto
      , famJudged = result proto
      }

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
        "dead" .= [deadOut i v | (i, v) <- deadRows]
      , "reported" .= [[toInteger i, toInteger v] | (i, v) <- reportedRows]
      , -- the zero-tolerance gate, named: any file-tier dead verdict
        -- fails `ce deadcode --check` — the exit was synthesized
        -- client-side before, where no ablation could see it
        "fail" .= not (null deadRows)
      , "pos" .= Position.positions b reach (reqPos req)
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
  b = build minRung [assetKind, refdefKind] (length (reqNodes req)) (reqEdges req)
  -- the confidence column rides exactly when the ledger rode
  -- (2.32.0): legacy requests keep two-column dead rows,
  -- byte-identical
  deadOut i v = case reqUnres req of
    Nothing -> [toInteger i, toInteger v]
    Just unres -> [toInteger i, toInteger v, confidence unres (langOf i)]
  langOf i
    | ((l : _) : _) <- drop i (reqNodes req) = l
    | otherwise = error "dead index inside the node table by construction"
  -- entry bits derive from the ROLE facts through Cost.roleBits
  -- (2.28.0, batch-7 slice 3); the pre-2.28 legacy flags column it
  -- used to yield to retired at 5.0.0, so there is one road left.
  -- Since 4.1.0 the export surface ORs the public bit in on top.
  -- Absent (or empty) symbols table => empty set => every flag word
  -- is what it was, so such a request answers byte-for-byte as
  -- before (K5).
  flagses =
    [ Dead.withExport publicFlagBit exported i (declaredBits row)
    | (i, row) <- zip [0 ..] (reqNodes req)
    ]
  exported = Dead.exportedNodes exportVisBit (symRows req)
  -- one reach per request (batch 9 P2): the entry knob binds to the
  -- seed set here at the boundary — the posture Cost.hs declares —
  -- and the computed set feeds the verdict AND the join surface.
  reach = reachFrom b (Dead.entries entryMask flagses)
  fileIdx = IS.fromList [i | (i, _ : k : _) <- zip [0 ..] (reqNodes req), k == granFile]
  (deadRows, reportedRows) =
    partition (\(i, _) -> IS.member i fileIdx) (Dead.verdicts b reach flagses)

-- | Entry bits derived from a row's role facts. Other shapes cannot
-- reach here: the contract refused them.
declaredBits :: [Integer] -> Integer
declaredBits [_, _, r] = Dead.deriveFlags roleBits r
declaredBits _ = 0

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
