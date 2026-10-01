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
module CE.Flow.Document (doc, kindTable) where

import CE.Document.Contract
import CE.Lang (languages)
import CE.Lang.Spec (Language (..))
import Data.Aeson (Value (..), object, (.=))
import Data.Foldable (asum)
import qualified Data.IntMap.Strict as IM
import Data.List (intercalate, sortOn)

doc :: DocFamily
doc = docFamily "flow" schemaId statement known assemble ["kinds" .= kindTable, "judged" .= judgedLangs]

schemaId :: String
schemaId = "ce.flow-report/0.1.0"

-- | A finding row is [file, unit nth, kind, line, lineEnd, variable]
-- (variable −1: none); a refusal [file, nth, reason]; `shown` the kind
-- codes listed (none = every kind); `check` / `deny` the face's
-- `--check` and the flow tier's deny, read by the veto (step 5).
-- References: path [file], unit [file, nth], var [file, nth, variable], why [text].
statement :: String
statement =
  "range files\nrange why\n\
  \fact units kept\nfact stmts kept\nfact vars kept\nfact uses kept\n\
  \fact dynamicUnits judged\n\
  \fact check kept\nfact deny kept\noptional check deny\n\
  \rows langs 2 kept files -\nrows rankFiles 2 kept files -\nrows shown 1 kept -\n\
  \rows unlowered 3 kept files - why\n\
  \rows findings 6 judged files - - - - -\nrows refused 3 judged files - why\n\
  \ref path files\nref unit files -\nref var files - -\nref why why\n"

-- | The finding kinds by code (CE.Flow.Cost), each with whether it is
-- advisory in every language (an unused parameter is often an
-- interface's): the one table every face, feed and `--kind` reads,
-- answered as the catalogue's `kinds` rows `[name, advisory]`.
kindTable :: [(String, Bool)]
kindTable = [("unreachable", False), ("dead_store", False), ("unused_local", False), ("unused_param", True)]

kinds :: [String]
kinds = map fst kindTable

-- | The advisory kinds' codes, read off the table.
advisoryKinds :: [Integer]
advisoryKinds = [code | (code, (_, True)) <- zip [0 ..] kindTable]

-- | The languages whose findings are judged, by wire code: the
-- language table's `flow_judged` column (CE.Lang.Common), where each
-- entered when its precision doc passed the gate (design booklet
-- docs/reference/analysis-track.md §5.5). The catalogue lists the
-- same codes.
judgedLangs :: [Integer]
judgedLangs = [toInteger (lgCode l) | l <- languages, lgFlowJudged l]

-- | The tables read by file hold one row per file; every kind is one
-- of the four. A `shown` row of −1 is a `--kind` name the measuring
-- side found nowhere in this catalogue: refused, the first such row
-- named, with the names the catalogue does list.
known :: DocReq -> Maybe String
known req =
  asum
    [ dense req "langs" (range req "files")
    , dense req "rankFiles" (range req "files")
    , codes req "findings" 2 0 top
    , unnamed
    , codes req "shown" 0 0 top
    ]
 where
  top = toInteger (length kinds) - 1
  unnamed = case [i | (i, [-1]) <- zip [0 :: Int ..] (rows req "shown")] of
    i : _ -> Just ("shown " <> show i <> ": unknown kind; the catalogue lists " <> intercalate ", " kinds)
    [] -> Nothing

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
  judgedAt [f, _, k, _, _, _] = k `notElem` advisoryKinds && IM.findWithDefault (-1) (fromInteger f) langOf `elem` judgedLangs
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
