{-# LANGUAGE OverloadedStrings #-}

-- | tables.request handler (plan v2.32 step 1; design booklet
-- docs/reference/authority-track.md §4): the definition package. Not a
-- judgment family — it judges nothing and reads no repository fact —
-- but the judge's own statement of what it judges with: every language
-- and product definition, one key per table family (CE.Lang), with the
-- digest the hello also names. Every value is a product constant, so
-- the integers-only wire rule (§5.9.2, which keeps the measured
-- repository's names out of the core) is untouched: nothing here came
-- from a repository, and it flows core to measuring side. Since 7.8.0 (plan
-- v2.32 step 3) the package also carries `document`, the document
-- catalogue (CE.Document): each report family's schema id and empty
-- document, the one document a face can print without a judgment.
module CE.Tables (package, respond, tablesDigest) where

import CE.Document (catalogue)
import CE.Lang (digestOf, pack)
import CE.Lang.Contract (offence)
import qualified CE.Resolve.Tables as ResolveTables
import CE.Limits (limits)
import Data.Aeson
import qualified Data.Aeson.KeyMap as KM
import qualified Data.ByteString.Char8 as B8
import qualified Data.ByteString.Lazy as BL

respond :: String -> B8.ByteString -> Either (Maybe Value, String, String) B8.ByteString
respond proto line = case decodeStrict line of
  Just (Object o) -> case offence o of
    Just why -> Left (KM.lookup "id" o, "contract", why)
    Nothing -> Right (reply proto (KM.lookup "id" o))
  _ -> Left (Nothing, "bad_request", "tables: the request is not an object")

-- | The definition package: the language and product definitions
-- (CE.Lang), the document catalogue and the families' limits (plan
-- v2.33 W3, CE.Limits).
package :: Value
package = case pack of
  Object keys -> Object (KM.insert "limits" limits (KM.insert "resolve" ResolveTables.table (KM.insert "document" catalogue keys)))
  v -> v

-- | The number the hello names as `tablesDigest` and the reply as
-- `digest`: the package's own.
tablesDigest :: Integer
tablesDigest = digestOf package

-- | The package's keys beside the envelope's and the digest.
reply :: String -> Maybe Value -> B8.ByteString
reply proto rid = BL.toStrict (encode (Object (KM.union envelope content)))
 where
  envelope =
    KM.fromList
      [ ("proto", toJSON proto)
      , ("type", String "tables.result")
      , ("id", maybe Null id rid)
      , ("digest", toJSON tablesDigest)
      ]
  content = case package of
    Object keys -> keys
    _ -> KM.empty
