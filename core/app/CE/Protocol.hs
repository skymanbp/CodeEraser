-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | Envelope layer (contracts/VERSIONING.md §1): owns the protocol
-- version, the byte-length precheck, dispatch on @type@, and the
-- @error@ reply. Every failure becomes a well-formed error line —
-- never a hello-shaped rejection (the M0 defect this module fixes),
-- never a crash.
module CE.Protocol (internalError, proto, respond) where

import qualified CE.Clone as Clone
import qualified CE.Docdup as Docdup
import qualified CE.Erase as Erase
import qualified CE.FourClass as FourClass
import qualified CE.Graph as Graph
import qualified CE.Handshake as Handshake
import qualified CE.Scan as Scan
import qualified CE.Structure as Structure
import qualified CE.Trend as Trend
import qualified CE.Verdict as Verdict
import Control.Applicative ((<|>))
import Data.Aeson
import qualified Data.Aeson.KeyMap as KM
import qualified Data.ByteString.Char8 as B8
import qualified Data.ByteString.Lazy as BL

-- | Protocol version spoken by this server (single source together
-- with cli/src/corelink.rs::PROTO — contracts/VERSIONING.md §1).
-- 2.17.0 = the density-scoring minor (M9 batch 6): verdict/1 axes
-- rows become bounded per-axis CHARGES — floor(scale·v/(v+n)) over
-- each axis's violation mass and opportunity count — and the score
-- their weighted mean under the violCost dial (neutral at 10); the
-- zone curve turns C¹ linear past H. Row SHAPES are unchanged
-- (values-only migration, the 2.14.0 axis-0 precedent): two real
-- repositories field-measured at 0/1000 because raw mass summed
-- onto the bounded scale saturated the clamp.
-- 2.16.0 = the erase minor (M9 batch 3, plan v2.8 ②): the erase/1
-- family — the deterministic eraser's safety predicate. Dense
-- [class, w, x, y, z] fact rows in, [eraseable, reason] verdicts
-- out in request order; knobless by design (safety is not tunable),
-- paths never on the wire, degraded = fail with an empty table.
-- 2.15.0 = the ROI-pricing minor (plan v2.7 ②, v0.7): the split
-- advisory's two priced legs — structure.request gains the OPTIONAL
-- `seamClones` [file, start, end] block spans and `seamChurn`
-- [file, a, b] co-change pairs (both legal only beside seamFiles,
-- absent = that leg unpriced), knobs 17/18 (roiCloneMilli /
-- roiChurnMilli); the reply shape is unchanged — both legs fold
-- into costMilli.
-- 2.14.0 = the size-advisory minor (plan v2.6, v0.6): ONE additive
-- minor for both faces the release ships together. verdict/1 —
-- graded axis 0 (the soft-zone curve; a declared score migration),
-- the `judgedLoc` multiset, ceilings codes 2..4 (H / P_max / k),
-- and `newBaseline.softLine` (derived at establish, carried after).
-- structure/1 — the split-ROI advisory: optional seamFiles /
-- seamUnits / seamRefs tables in, `splitCandidates` + `sizeExempt`
-- rows out exactly when seamFiles rode, knobs 12..16 (the zone
-- triple + the two milli prices).
-- 2.13.0 = the trend minor (M7.5b): the trend/1 family — the score
-- trajectory's [ts, score, scale] rows in (any ts order since the
-- 2026-08-20 #9 loosening — least squares is order-free), the exact-
-- Rational least-squares slope verdict out with the two-knob echo
-- (minPoints / declineFloorMicro); below minPoints answers null,
-- and the fail bit trips only under a DECLARED floor.
-- 2.12.0 = the staleness-axis minor (M6 S3c, closing the seven-axis
-- face): structure.request gains the OPTIONAL `staleDocs`
-- [dirId, stale, total] join (same absence semantics) and knob 11
-- (staleMin); axis-5 rows appear exactly when the table rode.
-- 2.11.0 = the redundancy-axis minor (M6 S3b): structure.request
-- gains the OPTIONAL `redundancy` [dirId, dupBlocks, deadUnits]
-- rollup (absent = axis 6 not judged, empty = judged clean — the
-- churn-table honesty) and knobs 9/10 (dupMin/deadMin); the reply's
-- axes/findings carry code-6 rows only when the table rode.
-- 2.10.0 = the declared-layout minor (M6 S3a): structure.request
-- gains the additive `declared` [dirId, weight] rows (ce.toml's
-- [structure] layout, owner = deepest declared ancestor) and the
-- reply — ONLY when a layout is declared — the `divergence` χ² row
-- and the named `deviations` [dirId, kind] rows (0 = undeclared
-- territory holding files, 1 = a declared bin owning none).
-- 2.9.0 = the structure minor (M6 S2): the structure/1 family —
-- tree-scale fact tables in (dense nodes, pattern distributions,
-- convention bits, file reference splits), five judged axes, the
-- headline entropy rows and sparse findings out. 2.8.0 = the
-- review-repair minor (ADR-008 反审批): verdict replies
-- gain the effective `weights` table and the ratchet's held-name
-- `failed` list; floor validates against the effective scale;
-- self-pairs and same-pair duplicates refuse at the boundary; caps
-- cover the knob/grade tables. 2.7.0 = the scan-grading minor
-- (ADR-008 P3): the scan/1 family — measurement rows in, graded
-- levels and the fail bit out, the grade table echoed whole.
-- 2.6.0 = the ratchet-unification minor (ADR-008 P2):
-- verdict.request gains the additive `dedup` [blocks, budget] pair
-- and the fail table its dedup_budget row — the second ratchet's
-- comparison is the core's. 2.5.0 = the verdict-repatriation minor
-- (ADR-008 P1): clone/docdup replies gain per-row `verdicts` bits,
-- the docdup echo gains `verbatimFloor`, and the degraded verdict
-- reply carries ratchet.fail=true itself. 2.4.0 = verdict.request
-- `thresholds` + `tolerance` knob rows and the reply's FULL
-- effective-knob echo (ADR-008 P4): additive, absent fields parse
-- as no override. 2.3.0 = `ceilings` rows + the two-knob echo
-- (ADR-008 first step, M5 close); 2.2.0 = clone/1 + docdup/1 +
-- verdict/1 in ONE additive minor (M5-3a); 2.1.0 = graph/1
-- (M5-2a); 2.0.0 = the M5-1c-iii anchor shape.
proto :: String
proto = "2.17.0"

-- | Checked before any JSON parse, so a hostile oversized line is
-- never decoded. Relaxed from 1 MiB at M5-2a (2026-08-12 decision):
-- the only client is the trusted same-machine daemon, and a graph
-- request legitimately carries ~1 MB per 100k LOC — the real
-- per-family guards are CE.Graph.Cost's node/edge caps.
maxLineBytes :: Int
maxLineBytes = 33554432

-- | Just the envelope: @type@ for dispatch, @id@ for echoing into
-- error replies, @proto@ for the per-message major check. Unknown
-- extra fields are ignored (§1).
data Envelope = Envelope
  { envType :: String
  , envId :: Maybe Value
  , envProto :: Maybe String
  }

instance FromJSON Envelope where
  parseJSON = withObject "Envelope" $ \o ->
    Envelope <$> o .: "type" <*> o .:? "id" <*> o .:? "proto"

-- | Answer one request line with one response line.
respond :: String -> B8.ByteString -> B8.ByteString
respond version line
  | B8.length line > maxLineBytes =
      errReply Nothing "too_large" "request line exceeds the byte ceiling"
  | otherwise = case eitherDecodeStrict line of
      Left e -> errReply (lenientId line) "bad_request" ("parse error: " <> e)
      Right env -> dispatch version env line

-- | When the ENVELOPE fails to decode (type missing or mistyped) the
-- line may still be a JSON object carrying an id — echo it, so a
-- shape typo does not read as L2-down client-side (the non-echoed-id
-- desync rule, VERSIONING.md §1; M5-close review LOW).
lenientId :: B8.ByteString -> Maybe Value
lenientId line = case decodeStrict line of
  Just (Object o) -> KM.lookup "id" o
  _ -> Nothing

-- | The Main loop's exception barrier reply ("never a crash",
-- structurally): id stays null — the computation died before any
-- field of it could be trusted.
internalError :: String -> B8.ByteString
internalError = errReply Nothing "internal"

-- | One row per judgment family — clone/1 landed with the T3 batch
-- (M5-3e), docdup/1 with the docdup batch (M5-3g), verdict/1 with
-- the score batch (M5-3i), each walking the declare-then-implement
-- path the goldens pin. Five identical case arms once inflated
-- dispatch to CC 19 (the M5-close warn repayment): the table IS the
-- dispatch, one arm total.
families :: [(String, String -> B8.ByteString -> Either (Maybe Value, String, String) B8.ByteString)]
families =
  [ ("fourclass.request", FourClass.respond)
  , ("graph.request", Graph.respond)
  , ("clone.request", Clone.respond)
  , ("docdup.request", Docdup.respond)
  , ("verdict.request", Verdict.respond)
  , ("scan.request", Scan.respond)
  , ("structure.request", Structure.respond)
  , ("trend.request", Trend.respond)
  , ("erase.request", Erase.respond)
  ]

-- | Every non-hello message must carry a proto with the server's
-- major (§1: 必带信封字段; 1.0.0 attack-review fix — the 0.x
-- implementation negotiated only at hello, so a skewed or bare
-- request was silently answered). hello keeps its own negotiation
-- reply (accept:false), which is richer than an error line.
dispatch :: String -> Envelope -> B8.ByteString -> B8.ByteString
dispatch version env line
  | envType env == "hello" = Handshake.respond proto version line
  | not (majorMatches (envProto env)) =
      errReply (envId env) "bad_request" ("proto missing or major-mismatched (server " <> proto <> ")")
  | Just familyRespond <- lookup (envType env) families =
      familyReply env (familyRespond proto line)
  | otherwise = errReply (envId env) "unknown_type" ("unsupported type: " <> envType env)

-- | A family-level decode failure hands back rid = Nothing, but the
-- ENVELOPE id is already in hand here — falling back to it keeps a
-- single malformed request from desyncing the whole session (the
-- client treats a non-echoed id as L2-down, VERSIONING.md §1; Opus
-- review, one fix for every family).
familyReply :: Envelope -> Either (Maybe Value, String, String) B8.ByteString -> B8.ByteString
familyReply env (Left (rid, code, message)) = errReply (rid <|> envId env) code message
familyReply _ (Right bytes) = bytes

majorMatches :: Maybe String -> Bool
majorMatches Nothing = False
majorMatches (Just v) = takeWhile (/= '.') v == takeWhile (/= '.') proto

errReply :: Maybe Value -> String -> String -> B8.ByteString
errReply requestId code message =
  BL.toStrict . encode $
    object
      [ "proto" .= proto
      , "type" .= ("error" :: String)
      , "id" .= maybe Null id requestId
      , "code" .= code
      , "message" .= message
      ]
