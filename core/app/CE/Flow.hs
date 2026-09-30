-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | flow.request handler (plan v2.31 step 3; ADR-008 seventh
-- instalment, design booklet docs/reference/analysis-track.md §5):
-- the fourteenth judgment family — the dead code INSIDE a function,
-- behind `ce flow`, the guard's write-time leg, the MCP tool and the
-- GUI screen. The measuring side lowers every unit of a language its
-- FlowSpec table knows into four integer tables — statements as a
-- pre-order tree with kinds and flags, variables with their
-- declaring statement and exemption flags, accesses in evaluation
-- order (§5.9.2: no name, path or source text crosses) — and this
-- side builds the control-flow graph, walks reachability and
-- backward liveness, and answers the findings: unreachable runs,
-- dead stores, unused locals, and unused parameters as advice. A
-- unit that calls an evaluator by name is skipped whole and counted.
-- One cap over the four tables; no knob, no condition bit — the face
-- and the guard read the findings under `[flow] tier`.
module CE.Flow (respond, unitFindings) where

import CE.Flow.Cfg (cfg)
import CE.Flow.Contract (FlowReq (..), offence, overCap)
import CE.Flow.Cost (findDeadStore, findUnreachable, flagDynamic)
import CE.Flow.Live (deadStores, unused)
import CE.Flow.Reach (reachable, runs)
import CE.Flow.Tree (Unit (..), hasFlag)
import CE.Wire (family)
import Data.Aeson (Value, encode, object, (.=))
import Data.ByteString (ByteString)
import Data.ByteString.Lazy (toStrict)
import qualified Data.IntMap.Strict as IM
import Data.List (partition, sort)

-- | decode → cap (the four tables together) → contract → judge; the
-- degraded reply is the judged reply's shape with nothing judged.
respond :: String -> ByteString -> Either (Maybe Value, String, String) ByteString
respond proto = family "flow" reqId overCap offence (answer proto True) (answer proto False)

-- | The flow.result object. Judged: every unit not skipped answers
-- its findings, ascending as rows, and the skipped ones are counted.
-- Degraded: no findings — a request the core refused to judge
-- licenses nothing — and the reason. The counts name the four tables
-- either way.
answer :: String -> Bool -> FlowReq -> ByteString
answer proto degraded req = toStrict (encode (object (fields <> ["reason" .= ("flow_too_large" :: String) | degraded])))
 where
  units = if degraded then [] else either (const []) IM.elems (unitsOf req)
  (skipped, judged) = partition dynamic units
  findings = sort (concatMap unitFindings judged)
  tallies = map (toInteger . length) [unitRows req, stmtRows req, varRows req, useRows req, findings] <> [toInteger (length skipped)]
  fields =
    [ "proto" .= proto
    , "type" .= ("flow.result" :: String)
    , "id" .= reqId req
    , "findings" .= findings
    , "counts" .= object (zipWith (.=) ["units", "stmts", "vars", "uses", "findings", "dynamicUnits"] tallies)
    , "degraded" .= degraded
    ]

-- | A unit any of whose statements carries the dynamic flag.
dynamic :: Unit -> Bool
dynamic u = any (hasFlag flagDynamic) (IM.elems (stmts u))

-- | One unit's findings as `[u, kind, seq, v, seqEnd]` rows: the
-- unreachable runs (v −1), the dead stores, the unused variables.
unitFindings :: Unit -> [[Integer]]
unitFindings u =
  [[me, findUnreachable, toInteger a, -1, toInteger b] | (a, b) <- runs (IM.size (stmts u)) seen]
    <> [[me, findDeadStore, toInteger s, toInteger v, toInteger s] | (s, v) <- deadStores u g seen]
    <> [[me, kind, toInteger decl, toInteger v, toInteger decl] | (kind, v, decl) <- unused u]
 where
  me = toInteger (unitId u)
  g = cfg u
  seen = reachable g
