-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | Hello semantics: SemVer negotiation (major must match) plus
-- capability discovery. Envelope concerns — the proto constant,
-- dispatch, error replies — live in "CE.Protocol", which passes
-- @proto@ AND the capability list in (batch 9 P12: the list is
-- derived from the dispatch table, one table for both hats) to
-- keep the dependency one-directional.
module CE.Handshake (respond) where

import CE.Protocol.Version (majorOf)
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
-- for accept/reject (contracts/VERSIONING.md §1). The list itself
-- is derived in CE.Protocol from the dispatch table and threaded in
-- (P12); when each capability was declared is VERSIONING.md's record.
data Reply = Reply
  { replyProto :: String
  , replyAccept :: Bool
  , replyReason :: Maybe String
  , replyVersion :: String
  , replyCaps :: [String]
  , replyTablesDigest :: Integer
  }

instance ToJSON Reply where
  toJSON r =
    object $
      [ "proto" .= replyProto r
      , "type" .= ("hello" :: String)
      , "server" .= ("ce-core" :: String)
      , "version" .= replyVersion r
      , "accept" .= replyAccept r
      , "capabilities" .= replyCaps r
      , "tablesDigest" .= replyTablesDigest r
      ]
        <> maybe [] (\why -> ["reason" .= why]) (replyReason r)

-- | Answer one hello line with one response line. @digest@ is the
-- definition package's (CE.Tables.tablesDigest): named on every hello, so a
-- client knows the package it would read before it asks for it.
respond :: String -> [String] -> Integer -> String -> B8.ByteString -> B8.ByteString
respond proto caps digest version line = BL.toStrict . encode $ case eitherDecodeStrict line of
  Left err -> Reply proto False (Just ("parse error: " <> err)) version caps digest
  Right h
    | majorOf (helloProto h) /= majorOf proto ->
        Reply proto False (Just ("incompatible proto: " <> helloProto h)) version caps digest
    | otherwise -> Reply proto True Nothing version caps digest

