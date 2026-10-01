-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | document.request handler (plan v2.32 step 3; design booklet
-- docs/reference/authority-track.md §5): the report documents of the
-- query, rules, flow, merge and architecture families, assembled here
-- — the fields, their order, the counts, the schema id and the
-- degraded bit are the judge's statement, and every face prints the
-- one document. Not a judgment family: it judges nothing, it lays out
-- what a judgment already answered. Its own family rather than a key
-- on each family's reply, because two of the five judge in batches the
-- measuring side joins (flow by `rowCap`, merge by its two caps) and a
-- document folded over the batches on that side would leave the
-- assembly there. Every repository string is a reference
-- (CE.Document.Contract); the catalogue — each family's schema id and
-- empty document, and flow's kind names — rides in the definition
-- package (CE.Tables), the one document a face can print when the
-- judgment never happened.
module CE.Document (catalogue, emptyOf, families, respond) where

import qualified CE.Arch.Document as Arch
import CE.Document.Contract
import qualified CE.Flow.Document as Flow
import qualified CE.Merge.Document as Merge
import qualified CE.Query.Document as Query
import qualified CE.Wire as Wire
import Data.Aeson (Value, encode, object, (.=))
import Data.Aeson.Key (fromString)
import qualified Data.ByteString.Char8 as B8
import qualified Data.ByteString.Lazy as BL
import qualified Data.Map.Strict as M

-- | The five documents, by the name a request gives.
families :: [DocFamily]
families = [Arch.doc, Query.queryDoc, Query.rulesDoc, Flow.doc, Merge.doc]

familyOf :: DocReq -> Maybe DocFamily
familyOf req = do
  name <- dFamily req
  lookup name [(dfName f, f) | f <- families]

-- | A family's document when the judgment did not happen: every range
-- and fact zero, no row, the reason the measuring side's first text.
emptyOf :: DocFamily -> Value
emptyOf fam = dfAssemble fam blank
 where
  sp = dfSpec fam
  blank =
    DocReq
      { dId = "empty"
      , dFamily = Just (dfName fam)
      , dRanges = Just (M.fromList [(r, 0) | r <- spRanges sp])
      , dRows = Just (M.fromList [(tName t, []) | t <- spTables sp])
      , dFacts = Just (M.fromList [(n, 0) | (n, _) <- spFacts sp])
      , dDegraded = Just 0
      }

-- | The `document` key of the definition package: per family its
-- schema id, its empty document, and what else it lists.
catalogue :: Value
catalogue =
  object
    [ fromString (dfName f) .= object (["schema" .= dfSchema f, "empty" .= emptyOf f] <> dfCatalogue f)
    | f <- families
    ]

-- | A family name first, then that family's statement.
offences :: DocReq -> Maybe String
offences req = case (dFamily req, familyOf req) of
  (Nothing, _) -> Just "document: missing family"
  (Just name, Nothing) -> Just ("document: unknown family " <> name)
  (_, Just fam) -> offence fam req

-- | Over the cap only once the family is known: a request naming no
-- family is refused by name first.
overCap :: DocReq -> Bool
overCap req = maybe False (const (totalRows req > docRowCap)) (familyOf req)

-- | The document.result object: the family's document and the rows it
-- was assembled from; past the cap the family's empty document, the
-- reason named.
answer :: String -> Bool -> DocReq -> B8.ByteString
answer proto degraded req = BL.toStrict (encode (object (fields <> ["reason" .= ("document_too_large" :: String) | degraded])))
 where
  document = maybe (object []) (\f -> if degraded then emptyOf f else dfAssemble f req) (familyOf req)
  fields =
    [ "proto" .= proto
    , "type" .= ("document.result" :: String)
    , "id" .= dId req
    , "document" .= document
    , "counts" .= object ["rows" .= totalRows req]
    , "degraded" .= degraded
    ]

-- | decode → cap → contract → assemble.
respond :: String -> B8.ByteString -> Either (Maybe Value, String, String) B8.ByteString
respond proto = Wire.family "document" dId overCap offences (answer proto True) (answer proto False)
