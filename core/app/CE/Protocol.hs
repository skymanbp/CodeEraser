-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | Envelope layer (contracts/VERSIONING.md §1): owns the protocol
-- version, the byte-length precheck, dispatch on @type@, and the
-- @error@ reply. Every failure becomes a well-formed error line —
-- never a hello-shaped rejection (the M0 defect this module fixes),
-- never a crash.
module CE.Protocol (proto, respond) where

import qualified CE.Clone as Clone
import qualified CE.Docdup as Docdup
import qualified CE.FourClass as FourClass
import qualified CE.Graph as Graph
import qualified CE.Handshake as Handshake
import qualified CE.Verdict as Verdict
import Control.Applicative ((<|>))
import Data.Aeson
import qualified Data.ByteString.Char8 as B8
import qualified Data.ByteString.Lazy as BL

-- | Protocol version spoken by this server (single source together
-- with cli/src/corelink.rs::PROTO — contracts/VERSIONING.md §1).
-- 2.2.0 = clone/1 + docdup/1 + verdict/1 declared in ONE additive
-- minor (M5-3a): stub handlers answer error/contract until each
-- family's judgment batch lands — the declare-then-implement path
-- graph/1 walked (2a → 2g). 2.1.0 = graph/1 (M5-2a); 2.0.0 was the
-- M5-1c-iii anchor-width shape (rem/add entries carry alnum width).
proto :: String
proto = "2.2.0"

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
      Left e -> errReply Nothing "bad_request" ("parse error: " <> e)
      Right env -> dispatch version env line

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
