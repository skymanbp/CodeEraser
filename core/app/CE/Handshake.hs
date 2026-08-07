-- | Handshake protocol: pure request-line -> response-line mapping.
-- Wire format: contracts/VERSIONING.md (NDJSON, SemVer major must match).
module CE.Handshake (proto, respond) where

import Data.Aeson
import Data.Aeson.Types (Parser)
import qualified Data.ByteString.Char8 as B8
import qualified Data.ByteString.Lazy as BL

-- | Protocol version spoken by this server.
proto :: String
proto = "0.1.0"

-- | Client hello; unknown extra fields are deliberately ignored
-- (forward compatibility within one major version).
newtype Hello = Hello {helloProto :: String}

instance FromJSON Hello where
  parseJSON = withObject "Hello" $ \o -> do
    t <- o .: "type" :: Parser String
    if t == "hello"
      then Hello <$> o .: "proto"
      else fail ("unexpected message type: " <> t)

data Reply = Reply
  { replyAccept :: Bool
  , replyReason :: Maybe String
  , replyVersion :: String
  }

instance ToJSON Reply where
  toJSON r =
    object $
      [ "proto" .= proto
      , "type" .= ("hello" :: String)
      , "server" .= ("ce-core" :: String)
      , "version" .= replyVersion r
      , "accept" .= replyAccept r
      ]
        <> maybe [] (\why -> ["reason" .= why]) (replyReason r)

-- | Answer one request line with one response line.
respond :: String -> B8.ByteString -> B8.ByteString
respond version line = BL.toStrict . encode $ case eitherDecodeStrict line of
  Left err -> Reply False (Just ("parse error: " <> err)) version
  Right h
    | major (helloProto h) /= major proto ->
        Reply False (Just ("incompatible proto: " <> helloProto h)) version
    | otherwise -> Reply True Nothing version

major :: String -> String
major = takeWhile (/= '.')
