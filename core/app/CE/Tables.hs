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
import CE.Tombstone (kindNames)
import CE.Wire.Families (FamilyName (..))
import qualified CE.Wire.Families as F
import CE.Wire (Respond, envelope)
import Data.Aeson
import qualified Data.Aeson.KeyMap as KM
import qualified Data.ByteString.Char8 as B8
import qualified Data.ByteString.Lazy as BL

respond :: Respond
respond proto line = case decodeStrict line of
  Just (Object o) -> case offence o of
    Just why -> Left (KM.lookup "id" o, "contract", why)
    Nothing -> Right (reply proto (KM.lookup "id" o))
  _ -> Left (Nothing, "bad_request", nameWord F.tables <> ": the request is not an object")

-- | The definition package: the language and product definitions
-- (CE.Lang), the document catalogue and the families' limits (plan
-- v2.33 W3, CE.Limits). The `tombstone` table also carries the row
-- kinds by code (step 11a: the names a site's `file:line kind` spells,
-- CE.Tombstone.kindNames — the family's own list, not a transcription).
package :: Value
package = case pack of
  Object keys -> Object (KM.mapWithKey withKinds (KM.insert "limits" limits (KM.insert "resolve" ResolveTables.table (KM.insert "document" catalogue keys))))
  v -> v
 where
  withKinds "tombstone" (Object t) = Object (KM.insert "kind_names" (toJSON kindNames) t)
  withKinds _ t = t

-- | The number the hello names as `tablesDigest` and the reply as
-- `digest`: the package's own.
tablesDigest :: Integer
tablesDigest = digestOf package

-- | The package's keys beside the envelope's and the digest.
reply :: String -> Maybe Value -> B8.ByteString
reply proto rid = BL.toStrict (encode (Object (KM.union fixed content)))
 where
  fixed = KM.fromList (envelope proto F.tables (maybe Null id rid) <> ["digest" .= tablesDigest])
  content = case package of
    Object keys -> keys
    _ -> KM.empty
