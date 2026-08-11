-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | Envelope layer (contracts/VERSIONING.md §1): owns the protocol
-- version, the byte-length precheck, dispatch on @type@, and the
-- @error@ reply. Every failure becomes a well-formed error line —
-- never a hello-shaped rejection (the M0 defect this module fixes),
-- never a crash.
module CE.Protocol (proto, respond) where

import qualified CE.FourClass as FourClass
import qualified CE.Handshake as Handshake
import Data.Aeson
import qualified Data.ByteString.Char8 as B8
import qualified Data.ByteString.Lazy as BL

-- | Protocol version spoken by this server (single source together
-- with cli/src/corelink.rs::PROTO — contracts/VERSIONING.md §1).
-- 1.0.0 = the M4 content finalization: wire shape identical to
-- 0.2.0, the bump declares the content frozen under §2 major rules.
proto :: String
proto = "1.0.0"

-- | Checked before any JSON parse, so a hostile oversized line is
-- never decoded. Legitimate requests are client-capped well below
-- this (20k lines ≈ 600 KB).
maxLineBytes :: Int
maxLineBytes = 1048576

-- | Just the envelope: @type@ for dispatch, @id@ for echoing into
-- error replies. Unknown extra fields are ignored (§1).
data Envelope = Envelope
  { envType :: String
  , envId :: Maybe Value
  }

instance FromJSON Envelope where
  parseJSON = withObject "Envelope" $ \o ->
    Envelope <$> o .: "type" <*> o .:? "id"

-- | Answer one request line with one response line.
respond :: String -> B8.ByteString -> B8.ByteString
respond version line
  | B8.length line > maxLineBytes =
      errReply Nothing "too_large" "request line exceeds the byte ceiling"
  | otherwise = case eitherDecodeStrict line of
      Left e -> errReply Nothing "bad_request" ("parse error: " <> e)
      Right env -> dispatch version env line

dispatch :: String -> Envelope -> B8.ByteString -> B8.ByteString
dispatch version env line = case envType env of
  "hello" -> Handshake.respond proto version line
  "fourclass.request" -> case FourClass.respond proto line of
    Left (rid, code, message) -> errReply rid code message
    Right bytes -> bytes
  t -> errReply (envId env) "unknown_type" ("unsupported type: " <> t)

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
