-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | The `ce query` and `ce rules` documents (plan v2.32 step 3; design
-- booklet docs/reference/authority-track.md §5), transcribed from the
-- face that assembled them on the measuring side
-- (cli/src/query/face.rs): the program block, every goal with its
-- kind, columns and sorts, every answer, every proof row, every error,
-- the core's counts. Three roads: the measuring side's own faults (a
-- lexical fault or a glob it could not read: `faults`, each a
-- position text and a message text), the core's program errors
-- (`errors`), or the judgment. The two documents differ in their
-- schema id alone.
module CE.Query.Document (queryDoc, rulesDoc) where

import CE.Document.Contract
import CE.Query.Schema (sortsOf)
import Data.Aeson (Value (..), object, toJSON, (.=))
import Data.Foldable (asum)
import qualified Data.IntMap.Strict as IM
import Data.Maybe (fromMaybe)

queryDoc, rulesDoc :: DocFamily
queryDoc = family "query" "ce.query-report/0.1.0"
rulesDoc = family "rules" "ce.rules-report/0.1.0"

family :: String -> String -> DocFamily
family name schemaId = docFamily name schemaId statement aligned (assemble schemaId) []

-- | The request. `at` is the token positions an error may name (the
-- stream's length plus one: the end of input), `heads` the program's
-- goals as spelled ([goal, has a name, columns]), `askWhy` / `rulesFile`
-- / `query` the program block's flags, and the ten counts the core's.
statement :: String
statement =
  "range at\nrange why\n\
  \fact tokens kept\nfact clauses kept\nfact prelude kept\nfact askWhy kept\n\
  \fact rulesFile kept\nfact query kept\n\
  \fact rules kept\nfact queries kept\nfact asserts kept\nfact strata kept\n\
  \fact facts kept\nfact derived kept\nfact answers kept\nfact violations kept\n\
  \fact proofNodes kept\nfact proofTruncated kept\n\
  \rows faults 2 kept why why\nrows heads 3 kept - - -\n\
  \rows goals 2+ judged - -\nrows preds 1+ judged -\nrows answers 1+ judged -\n\
  \rows proof 6+ judged - - - - - -\nrows errors 2 judged at -\n\
  \ref at at\nref goal_name -\nref column - -\nref value - -\nref pred -\n\
  \ref rules_file\nref query\nref why why\n"

counts :: [String]
counts = words "rules queries asserts strata facts derived answers violations proofNodes proofTruncated"

-- | The core's program errors by code; 0 is the measuring side's own
-- (a fault, whose message is its text).
errorNames :: [String]
errorNames =
  [ "lexical"
  , "syntax error"
  , "unknown predicate"
  , "arity mismatch"
  , "sort mismatch"
  , "unsafe variable"
  , "unstratifiable negation"
  , "prelude predicate redefined"
  , "aggregate shape"
  , "anonymous head"
  ]

-- | A sort's name by code (CE.Query.Cost); any other is `open`.
sortName :: Integer -> String
sortName s = case drop (fromInteger s) (words "node dir unit int sym set") of
  (n : _) | s >= 0 -> n
  _ -> "open"

-- | A fault answers no goal; an error code is the core's; a judged
-- program's heads are its goals, one each, in order.
aligned :: DocReq -> Maybe String
aligned req =
  asum
    [ if null (rows req "faults") || all (null . rows req) verdict then Nothing else Just "faults: beside the core's rows"
    , codes req "errors" 1 1 9
    , if judged req then asum [dense req "heads" goals, dense req "goals" goals, headsMatch] else Nothing
    ]
 where
  verdict = ["goals", "preds", "answers", "proof", "errors"]
  goals = toInteger (length (rows req "goals"))
  headsMatch = if length (rows req "heads") == length (rows req "goals") then Nothing else Just "heads: one per goal"

-- | The goals are labelled only when nothing failed.
judged :: DocReq -> Bool
judged req = null (rows req "faults") && null (rows req "errors") && dDegraded req == Nothing

assemble :: String -> DocReq -> Value
assemble schemaId req =
  object
    [ "schema" .= schemaId
    , "program" .= program req
    , "goals" .= if judged req then zipWith goal (rows req "heads") (rows req "goals") else []
    , "answers" .= if judged req then map (answer ofGoal) (rows req "answers") else []
    , "proof" .= if judged req then map (proofRow ofGoal ofPred) (rows req "proof") else []
    , "errors" .= errors req
    , "counts" .= counted counts (map (fact req) counts)
    , "degraded" .= whyRef req
    ]
 where
  ofGoal = sortsBy (rows req "goals") (drop 1)
  ofPred = sortsBy (rows req "preds") id

program :: DocReq -> Value
program req =
  object
    [ "rules_file" .= flagged "rulesFile" "rules_file"
    , "query" .= flagged "query" "query"
    , "why" .= (fact req "askWhy" /= 0)
    , "tokens" .= fact req "tokens"
    , "clauses" .= fact req "clauses"
    , "prelude" .= fact req "prelude"
    ]
 where
  flagged k cls = if fact req k /= 0 then ref cls [] else Null

-- | A fault's position and message are texts; a core error is a token
-- and a code.
errors :: DocReq -> [Value]
errors req
  | faults@(_ : _) <- rows req "faults" = [face (ref "why" [p]) Null 0 (ref "why" [m]) | [p, m] <- faults]
  | otherwise = [face (ref "at" [t]) (toJSON t) c (named c) | [t, c] <- rows req "errors"]
 where
  face :: Value -> Value -> Integer -> Value -> Value
  face at tok code what = object ["at" .= at, "token" .= tok, "code" .= code, "what" .= what]
  named c = toJSON (fromMaybe "?" (lookup c (zip [0 ..] errorNames)))

goal :: [Integer] -> [Integer] -> Value
goal head' row = case (head', row) of
  ([g, named, cols], _ : kind : sorts) ->
    object
      [ "goal" .= g
      , "kind" .= (if kind == 1 then "assert" else "query" :: String)
      , "name" .= (if named /= 0 then ref "goal_name" [g] else Null)
      , "columns" .= [ref "column" [g, i] | i <- [0 .. cols - 1]]
      , "sorts" .= map sortName sorts
      ]
  _ -> Null

-- | Sorts by a row's first column: a goal's (past its kind) or a
-- program predicate's, as the core resolved them.
sortsBy :: [[Integer]] -> ([Integer] -> [Integer]) -> Integer -> Maybe [Integer]
sortsBy table past = \x -> IM.lookup (fromInteger x) m
 where
  m = IM.fromList [(fromInteger k, past rest) | k : rest <- table]

-- | Values as the measuring side's labels render them under their
-- sorts (an unknown position reads sort −1).
values :: [Integer] -> [Integer] -> [Value]
values sorts = zipWith (\s v -> ref "value" [s, v]) (sorts <> repeat (-1))

answer :: (Integer -> Maybe [Integer]) -> [Integer] -> Value
answer ofGoal row = case row of
  g : vs -> object ["goal" .= g, "values" .= values (fromMaybe [] (ofGoal g)) vs, "raw" .= vs]
  _ -> Null

-- | A proof row; its argument sorts are the goal's at a query root,
-- the core's resolved ones for a program predicate, the schema's for a
-- fact.
proofRow :: (Integer -> Maybe [Integer]) -> (Integer -> Maybe [Integer]) -> [Integer] -> Value
proofRow ofGoal ofPred row = case row of
  g : a : node : parent : rule : p : args ->
    object
      [ "goal" .= g
      , "answer" .= a
      , "node" .= node
      , "parent" .= parent
      , "rule" .= rule
      , "pred" .= (if p < 0 then String "?-" else ref "pred" [p])
      , "args" .= values (sortsAt g p) args
      ]
  _ -> Null
 where
  sortsAt g p
    | p < 0 = fromMaybe [] (ofGoal g)
    | otherwise = fromMaybe (fromMaybe [] (sortsOf (fromInteger p))) (ofPred p)
