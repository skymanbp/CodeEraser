-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | The `ce flow` document (plan v2.32 step 3; design booklet
-- docs/reference/authority-track.md §5), transcribed from the face
-- that assembled it on the measuring side (cli/src/flow_report/face.rs
-- and mod.rs): every placed finding with its kind by name and whether
-- it is judged or advisory, the units left unjudged with their
-- reasons, the counts. The measuring side sends the findings it could
-- place through their units' legends (with the lines and the variable
-- index), the refusals (its own lowering's, which stand beside a
-- `degraded` reason, and the core's), each file's language, each
-- file's place in path order, and the kinds `--kind` showed.
module CE.Flow.Document (doc) where

import CE.Document.Contract
import Data.Aeson (Value (..), object, (.=))
import Data.Foldable (asum)
import qualified Data.IntMap.Strict as IM
import Data.List (sortOn)

doc :: DocFamily
doc = docFamily "flow" schemaId statement known assemble ["kinds" .= kinds, "judged" .= judgedLangs]

schemaId :: String
schemaId = "ce.flow-report/0.1.0"

-- | A finding row is [file, unit nth, kind, line, lineEnd, variable]
-- (variable −1: none); a refusal [file, nth, reason]; `shown` the kind
-- codes listed (none = every kind).
statement :: String
statement =
  "range files\nrange why\n\
  \fact units kept\nfact stmts kept\nfact vars kept\nfact uses kept\n\
  \fact dynamicUnits judged\n\
  \rows langs 2 kept files -\nrows rankFiles 2 kept files -\nrows shown 1 kept -\n\
  \rows unlowered 3 kept files - why\n\
  \rows findings 6 judged files - - - - -\nrows refused 3 judged files - why\n\
  \ref path files\nref unit files -\nref var files - -\nref why why\n"

-- | The finding kinds by code (CE.Flow.Cost): the one spelling every
-- face, feed and `--kind` uses.
kinds :: [String]
kinds = ["unreachable", "dead_store", "unused_local", "unused_param"]

-- | The kind that is advisory in every language: an unused parameter
-- is often an interface's.
advisory :: Integer
advisory = 3

-- | The languages whose findings are judged, by wire code (Python,
-- TypeScript, TSX, Rust, Go, C, C++, Lua, Java, R): each entered when
-- its precision doc passed the gate (design booklet
-- docs/reference/analysis-track.md §5.5); all ten since v2.31 step 4
-- commit G.
judgedLangs :: [Integer]
judgedLangs = [0, 1, 2, 3, 4, 15, 16, 17, 18, 20]

-- | The tables read by file hold one row per file; every kind is one
-- of the four.
known :: DocReq -> Maybe String
known req =
  asum
    [ dense req "langs" (range req "files")
    , dense req "rankFiles" (range req "files")
    , codes req "findings" 2 0 top
    , codes req "shown" 0 0 top
    ]
 where
  top = toInteger (length kinds) - 1

assemble :: DocReq -> Value
assemble req =
  object
    [ "schema" .= schemaId
    , "counts" .= counted countKeys tallies
    , "findings" .= map face listed
    , "refused" .= map refusal refused
    , "degraded" .= whyRef req
    ]
 where
  langOf = IM.fromList [(fromInteger f, l) | [f, l] <- rows req "langs"]
  rankOf = IM.fromList [(fromInteger f, r) | [f, r] <- rows req "rankFiles"]
  rank f = IM.findWithDefault (-1) (fromInteger f) rankOf
  judgedAt [f, _, k, _, _, _] = k /= advisory && IM.findWithDefault (-1) (fromInteger f) langOf `elem` judgedLangs
  judgedAt _ = False
  found = sortOn (\r -> (rank (head' r), take 3 (drop 1 r))) (rows req "findings")
  shown = [k | [k] <- rows req "shown"]
  listed = [r | r <- found, null shown || take 1 (drop 2 r) `elemOf` shown]
  refused = sortOn (\r -> (rank (head' r), take 1 (drop 1 r))) (rows req "unlowered" <> rows req "refused")
  face r = case r of
    [f, nth, k, l, le, v] ->
      object
        [ "path" .= ref "path" [f]
        , "unit" .= ref "unit" [f, nth]
        , "kind" .= concat (take 1 (drop (fromInteger k) kinds))
        , "line" .= l
        , "lineEnd" .= le
        , "var" .= (if v >= 0 then ref "var" [f, nth, v] else Null)
        , "judged" .= judgedAt r
        ]
    _ -> Null
  refusal r = case r of
    [f, nth, why] -> object ["path" .= ref "path" [f], "unit" .= ref "unit" [f, nth], "reason" .= ref "why" [why]]
    _ -> Null
  countKeys = words "units stmts vars uses findings dynamicUnits refused judged shown"
  tallies =
    map (fact req) ["units", "stmts", "vars", "uses"]
      <> map (toInteger . length) [found]
      <> [fact req "dynamicUnits", toInteger (length refused), toInteger (length (filter judgedAt found)), toInteger (length listed)]
  head' r = case r of
    f : _ -> f
    [] -> -1
  elemOf ks set = any (`elem` set) ks
