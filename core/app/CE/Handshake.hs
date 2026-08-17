-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | Hello semantics: SemVer negotiation (major must match) plus
-- capability discovery. Envelope concerns — the proto constant,
-- dispatch, error replies — live in "CE.Protocol", which passes
-- @proto@ in to keep the dependency one-directional.
module CE.Handshake (respond) where

import Data.Aeson
import qualified Data.ByteString.Char8 as B8
import qualified Data.ByteString.Lazy as BL

-- | Client hello; unknown extra fields are deliberately ignored
-- (forward compatibility within one major version). The @type@ field
-- was already verified by the dispatcher.
newtype Hello = Hello {helloProto :: String}

instance FromJSON Hello where
  parseJSON = withObject "Hello" $ \o -> Hello <$> o .: "proto"

-- | Informational discovery only — SemVer stays the sole authority
-- for accept/reject (contracts/VERSIONING.md §1).
-- | fourclass/2 = the anchor-width request shape (proto 2.0.0). An
-- old client probing fourclass/1 sees absence and degrades to L1
-- loudly instead of sending the un-parseable two-element shape.
-- graph/1 = the M5-2 graph family (proto 2.1.0, additive).
-- clone/1 + docdup/1 + verdict/1 = the M5-3 families, declared in
-- one additive minor (proto 2.2.0) with stub handlers answering
-- error/contract until each judgment batch lands (M5-3a).
-- scan/1 = the graded-verdict-table family (ADR-008 P3, proto
-- 2.7.0, additive), declared WITH its judgment in one batch.
capabilities :: [String]
capabilities = ["hello", "fourclass/2", "graph/1", "clone/1", "docdup/1", "verdict/1", "scan/1"]

data Reply = Reply
  { replyProto :: String
  , replyAccept :: Bool
  , replyReason :: Maybe String
  , replyVersion :: String
  }

instance ToJSON Reply where
  toJSON r =
    object $
      [ "proto" .= replyProto r
      , "type" .= ("hello" :: String)
      , "server" .= ("ce-core" :: String)
      , "version" .= replyVersion r
      , "accept" .= replyAccept r
      , "capabilities" .= capabilities
      ]
        <> maybe [] (\why -> ["reason" .= why]) (replyReason r)

-- | Answer one hello line with one response line.
respond :: String -> String -> B8.ByteString -> B8.ByteString
respond proto version line = BL.toStrict . encode $ case eitherDecodeStrict line of
  Left err -> Reply proto False (Just ("parse error: " <> err)) version
  Right h
    | major (helloProto h) /= major proto ->
        Reply proto False (Just ("incompatible proto: " <> helloProto h)) version
    | otherwise -> Reply proto True Nothing version

major :: String -> String
major = takeWhile (/= '.')
