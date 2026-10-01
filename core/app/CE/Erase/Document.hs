-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | The `ce erase` plan document (plan v2.32 step 5; design booklet
-- docs/reference/authority-track.md §5), transcribed from the face that
-- assembled it on the measuring side (cli/src/erase/mod.rs `plan`,
-- cli/src/erase/render.rs `report_json`, the shapes in
-- cli/src/erase/model.rs): the rows the erase/1 target closure kept,
-- each with its class, the predicate's verdict and reason, its target,
-- provenance, the unresolved sites of its language and the file's
-- content hash; the counts; and the family command behind every
-- out-of-class kind. The class and reason names are CE.Erase.Cost's;
-- the out-of-class kinds and their commands are this module's table.
-- The measuring side sends every judged candidate in plan order with
-- the reply's [eraseable, reason] and `kept` bit, and the out-of-class
-- counts by kind code; paths and provenance are references. The
-- trail `ce erase --log` reads is this module's second family
-- (`trailDoc`, below).
module CE.Erase.Document (doc, logRel, outOfClass, planRows, trailDoc) where

import CE.Document.Contract
import CE.Erase.Cost (classNames, reasonNames)
import Data.Aeson (Value (..), object, toJSON, (.=))
import Data.Aeson.Key (fromString)
import Data.Foldable (asum)
import Data.List (sortOn)
import qualified Data.Map.Strict as M

doc :: DocFamily
doc =
  docFamily "erase" "ce.erase-plan/0.3.0" statement checked assemble $
    ["classes" .= classNames, "reasons" .= reasonNames, "kinds" .= map fst outOfClass, "diffContext" .= diffContext]

-- | The out-of-class kinds the plan counts, by code, each with the
-- family command that owns its findings (O24): a T1/T2 clone block
-- that covers no whole unit has no deterministic-safe erase, and
-- `ce dedup` judged it.
outOfClass :: [(String, String)]
outOfClass = [("t1t2_block_no_whole_unit", "ce dedup")]

-- | The context lines around each hunk of the console's unified diff
-- (cli/src/erase/render.rs `CONTEXT`); the measuring side renders the
-- diff, this is the one source of the number.
diffContext :: Integer
diffContext = 3

-- | A candidate row is [class, file, spanned (0 the whole file), start,
-- end, sites, content hash, eraseable, reason, kept], in plan order; an
-- out-of-class row [kind, count]; `check` the console's `--check`,
-- `apply` whether the apply line prints and `applied` the rows it
-- erased (0 is a count).
statement :: String
statement =
  "range paths\nrange cands\nfact check judged\nfact apply judged\nfact applied judged\n\
  \rows cands 10 judged - paths - - - - - - - -\nrows outOfClass 2 judged - -\n\
  \ref path paths\nref provenance cands\nref diff cands cands\n"

checked :: DocReq -> Maybe String
checked req =
  asum
    [ if toInteger (length (rows req "cands")) == range req "cands" then Nothing else Just "cands: not one row per candidate"
    , codes req "cands" 0 1 3
    , asum [codes req "cands" c 0 1 | c <- [2, 7, 9]]
    , codes req "cands" 8 0 (toInteger (length reasonNames) - 1)
    , codes req "outOfClass" 0 0 (toInteger (length outOfClass) - 1)
    , bits req ["check", "apply"]
    , if fact req "applied" > 0 && not (flag req "apply") then Just "facts: applied without apply" else Nothing
    ]

-- | The rows the closure kept, with their place in the request.
planRows :: DocReq -> [(Integer, [Integer])]
planRows req = [(i, r) | (i, r) <- zip [0 ..] (rows req "cands"), drop 9 r == [1]]

assemble :: DocReq -> Value
assemble req =
  object
    [ "schema" .= dfSchema doc
    , "rows" .= map row kept
    , "counts" .= object ["candidates" .= length kept, "eraseable" .= length eraseable, "advisory" .= (length kept - length eraseable), "out_of_class" .= object [fromString k .= n | (k, n) <- kinds]]
    , "families" .= object [fromString k .= c | (k, _) <- kinds, Just c <- [lookup k outOfClass]]
    ]
 where
  kept = planRows req
  eraseable = [() | (_, r) <- kept, take 1 (drop 7 r) == [1]]
  kinds = sortOn fst [(nameOf (map fst outOfClass) k, n) | [k, n] <- rows req "outOfClass"]
  row (i, r) = case r of
    [cls, f, spanned, s, e, sites, hash, ok, reason, _] ->
      object
        [ "class" .= spelled classNames cls
        , "eraseable" .= (ok == 1)
        , "reason" .= spelled reasonNames reason
        , "path" .= ref "path" [f]
        , "span" .= (if spanned == 1 then toJSON [s, e] else Null)
        , "provenance" .= ref "provenance" [i]
        , "sites" .= sites
        , "hash" .= hash
        ]
    _ -> object []

-- | The `ce erase --log` document, transcribed from the trail's
-- reader cli/src/erase/log.rs `report_json`: where the trail lives,
-- whether it exists, every record apply.rs wrote (its record schema,
-- stamp, class, target, provenance, hash and plan), every line the
-- reader refused by number with why, and the counts — the records by
-- class among them. No judgment: the measuring side reads the file and
-- sends each record as integers (its class by code into
-- CE.Erase.Cost's names, the writer's vocabulary) and each refused
-- line's number; the target, provenance, hash, plan and the reason a
-- line was refused are references.
trailDoc :: DocFamily
trailDoc = docFamily "erase-trail" "ce.erase-trail-report/0.1.0" trailStatement trailChecked trailAssemble ["log" .= logRel, "record" .= recordSchema]

-- | The record schema apply.rs writes and the reader holds every line
-- to, and where the trail lives (root-relative).
recordSchema, logRel :: String
recordSchema = "ce.erase-log/0.1.0"
logRel = ".ce/erase-log.ndjson"

-- | A record row is [r, ts_ms, class, file, spanned (0 the whole file),
-- start, end], one per record in file order; a refused row [k, line],
-- one per refused line; `present` whether the file exists.
trailStatement :: String
trailStatement =
  "range paths\nrange records\nrange unread\nfact present judged\n\
  \rows records 7 judged records - - paths - - -\nrows unreadable 2 judged unread -\n\
  \ref path paths\nref provenance records\nref hash records\nref plan records\nref unreadable unread\n"

trailChecked :: DocReq -> Maybe String
trailChecked req =
  asum
    [ dense req "records" (range req "records")
    , dense req "unreadable" (range req "unread")
    , codes req "records" 2 0 (toInteger (length classNames) - 1)
    , codes req "records" 4 0 1
    , bits req ["present"]
    , if flag req "present" || range req "records" + range req "unread" == 0 then Nothing else Just "records: a trail that is not present"
    ]

trailAssemble :: DocReq -> Value
trailAssemble req =
  object
    [ "schema" .= dfSchema trailDoc
    , "log" .= logRel
    , "present" .= flag req "present"
    , "rows" .= map record (rows req "records")
    , "unreadable" .= [object ["line" .= n, "why" .= ref "unreadable" [k]] | [k, n] <- rows req "unreadable"]
    , "counts" .= object ["rows" .= range req "records", "unreadable" .= range req "unread", "by_class" .= uncurry counted (unzip (M.toList byClass))]
    ]
 where
  byClass = M.fromListWith (+) [(nameOf classNames cls, 1 :: Integer) | (_ : _ : cls : _) <- rows req "records"]
  record r = case r of
    [i, ts, cls, f, spanned, s, e] ->
      object
        [ "schema" .= recordSchema
        , "ts_ms" .= ts
        , "class" .= spelled classNames cls
        , "path" .= ref "path" [f]
        , "span" .= (if spanned == 1 then toJSON [s, e] else Null)
        , "provenance" .= ref "provenance" [i]
        , "hash" .= ref "hash" [i]
        , "plan" .= ref "plan" [i]
        ]
    _ -> object []
