-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | The `ce graph --sites` document (plan v2.32 step 4; design booklet
-- docs/reference/authority-track.md §5), transcribed from the face
-- that assembled it on the measuring side (cli/src/graph/mod.rs
-- `sites_json` over `analyze`): every reference site the detectors
-- found, file by file in path order and in detection order within a
-- file, with its language and kind by name. No judgment rides this
-- document; the measuring side sends each file's language and place
-- in path order and each site's integers, and the site's spec and
-- owner stay its own text. A walk that fails prints no document (the
-- face exits 2), so the family takes no reason. A site's kind is
-- named from the definition package's `store` table (CE.Lang).
module CE.Graph.Sites (doc) where

import CE.Document.Contract
import CE.Lang (siteKinds)
import Data.Aeson (Value (Null), object, (.=))
import Data.List (sortOn)
import qualified Data.Map.Strict as M
import Data.Maybe (catMaybes, listToMaybe)

doc :: DocFamily
doc = docFamily "sites" schemaId statement checked assemble []

schemaId :: String
schemaId = "ce.sites-report/0.1.0"

-- | The request: a site row is [i, file, kind, line, nth, owned] — `i`
-- its place in detection order, `owned` whether it has an owner.
statement :: String
statement =
  "range files\nrange sites\n\
  \rows rankFiles 2 judged files -\nrows langs 2 judged files -\n\
  \rows sites 6 judged sites files - - - -\n\
  \ref path files\nref site_spec sites\nref site_owner sites\n"

checked :: DocReq -> Maybe String
checked req =
  listToMaybe . catMaybes $
    [ dense req "rankFiles" (range req "files")
    , dense req "langs" (range req "files")
    , dense req "sites" (range req "sites")
    , codes req "sites" 2 0 (toInteger (length siteKinds) - 1)
    , codes req "sites" 5 0 1
    , listToMaybe ["langs " <> show f <> ": no language " <> show l | [f, l] <- rows req "langs", null (langName l)]
    ]

assemble :: DocReq -> Value
assemble req = object ["schema" .= schemaId, "sites" .= map site (sortOn placed (rows req "sites"))]
 where
  at table = M.fromList [(k, v) | [k, v] <- rows req table]
  (rank, lang) = (at "rankFiles", at "langs")
  look m f = M.findWithDefault (-1) f m
  placed r = case r of
    _ : f : _ -> look rank f
    _ -> -1
  site r = case r of
    [i, f, k, line, nth, owned] ->
      object
        [ "path" .= ref "path" [f]
        , "lang" .= langName (look lang f)
        , "kind" .= spelled siteKinds k
        , "line" .= line
        , "nth" .= nth
        , "spec" .= ref "site_spec" [i]
        , "owner" .= (if owned /= 0 then ref "site_owner" [i] else Null)
        ]
    _ -> Null
