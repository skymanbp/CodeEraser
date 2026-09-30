-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | The query family's battery (plan v2.31 step 1): the prelude's
-- dead-file query with its proof, an assertion's violations with
-- theirs, every program error named at its token, proof trees
-- replayed node by node against the naive reference, aggregates and
-- arithmetic by value, the open sort, the schema echo, refusal by
-- name for every contract clause, and the three capped dimensions
-- (tokens through the real respond, fact rows at the predicate,
-- derived tuples at the evaluator's cap parameter — the shipped
-- constant's road is one line). Scaffolding lives in WireHarness and
-- ReferenceQuery; the probes read their reply fields through
-- `fieldsOf` in one projection each.
module QueryProps (battery) where

import CE.Query (respond)
import CE.Query.Check (check)
import CE.Query.Contract (QueryReq (..), overCap)
import CE.Query.Cost
import CE.Query.Eval (evalProgramWith)
import CE.Query.Parse (parseProgram)
import CE.Query.Schema (schemaRows)
import Data.Aeson
import qualified Data.Aeson.Key as Key
import qualified Data.IntMap.Strict as IM
import Data.Maybe (isJust)
import qualified Data.Set as S
import ReferenceQuery (naive, replays, toks)
import WireHarness (field, fieldsOf, refusedBy, replyObjWith, runLegs, setKey)

-- | A request from a spaced program, fact tables by code, and extra
-- keys (prelude / why / schema).
request :: String -> [(Int, [[Integer]])] -> [(String, Value)] -> Value
request prog facts extra =
  object
    ( [ "proto" .= ("7.0.0" :: String)
      , "type" .= ("query.request" :: String)
      , "id" .= (1 :: Int)
      , "program" .= [[k, v] | (k, v) <- toks prog]
      , "facts" .= object [Key.fromString (show c) .= t | (c, t) <- facts]
      ]
        <> [Key.fromString k .= v | (k, v) <- extra]
    )

-- | Four files in two directories, one entry, two import arcs
-- 0→1→2 (3 is dead), line counts 3 / 9 / 12 / 40.
facts4 :: [(Int, [[Integer]])]
facts4 = [(1, [[0], [1], [2], [3]]), (2, [[0, 0], [1, 0], [2, 1], [3, 1]]), (7, [[0, 5]]), (8, [[0, 3], [1, 9], [2, 12], [3, 40]]), (9, [[0, 1, 0, 1], [1, 2, 0, 1]])]

prelude :: String
prelude = "p1000 ( v0 ) :- p7 ( v0 , s5 ) . p1000 ( v1 ) :- p1000 ( v0 ) , p9 ( v0 , v1 , _ , _ ) . p1001 ( v0 ) :- p1 ( v0 ) , not p1000 ( v0 ) ."

-- | Reply fields of a program over facts4.
answersOf :: String -> [(String, Value)] -> [String] -> Maybe [Maybe Value]
answersOf prog extra = fieldsOf respond (request prog facts4 extra)

rows :: [[Integer]] -> Maybe Value
rows = Just . toJSON

battery :: IO Bool
battery = runLegs names probes

names :: [String]
names =
  [ "the prelude's dead-file query answers with its proof and its counts"
  , "an assertion's violations carry their derivation without being asked"
  , "every program error is named at its token, and an erroring program answers nothing"
  , "proof trees replay node by node against the naive reference"
  , "aggregates: count, sum, min and max, the empty count is 0, the empty min fails, groups follow the outer binding"
  , "arithmetic binds, `=` on a bound variable compares, division by zero fails the literal, `<` on ids breaks a symmetric pair"
  , "goals carry their kind and column sorts, preds every program predicate's position sorts; a position nothing constrains echoes the open sort"
  , "the schema rides back only when asked"
  , "query refusals name the offender"
  , "an over-cap token stream degrades to empty tables; fact rows are priced at the cap; the derived cap aborts with the count reached"
  , "proof trees past the node budget are left out whole and counted"
  , "an empty program answers empty tables"
  ]

probes :: [Bool]
probes = [deadFile, assertion, errorsNamed, proofReplay, aggregates, arithmetic, goalsAndSorts, schemaEcho, refusals, capped, proofBudget, emptyProgram]

deadFile :: Bool
deadFile =
  answersOf (prelude <> " ?- p1001 ( v0 ) .") [("prelude", Number 3), ("why", Bool True)] ["answers", "goals", "proof", "errors", "counts", "degraded"]
    == Just
      [ rows [[0, 3]]
      , rows [[0, 0, 0]]
      , rows [[0, 0, 0, -1, 3, -1, 3], [0, 0, 1, 0, 2, 1001, 3], [0, 0, 2, 1, -1, 1, 3]]
      , rows []
      , countRow [3, 1, 0, 2, 15, 4, 1, 0, 3, 0]
      , Just (Bool False)
      ]

-- | The counts object from its ten values, in the wire's key order.
countRow :: [Integer] -> Maybe Value
countRow ns = Just (object (zipWith (.=) ["rules", "queries", "asserts", "strata", "facts", "derived", "answers", "violations", "proofNodes", "proofTruncated"] ns))

assertion :: Bool
assertion =
  answersOf (prelude <> " assert p1002 ( v0 ) :- p1001 ( v0 ) .") [("prelude", Number 3)] ["answers", "goals", "proof"]
    == Just [rows [[0, 3]], rows [[0, 1, 0]], rows [[0, 0, 0, -1, 3, 1002, 3], [0, 0, 1, 0, 2, 1001, 3], [0, 0, 2, 1, -1, 1, 3]]]

-- | One fault per program; the phase that finds it answers it and
-- the answers table stays empty.
errorsNamed :: Bool
errorsNamed =
  and
    [ answersOf prog [("prelude", Number pre)] ["errors", "answers"] == Just [rows [[at, code]], rows []]
    | (prog, pre, at, code) <-
        [ ("?- p1 ( v0 ) , v0 = i1 + .", 0, 10, errSyntax)
        , ("?- p1000 ( v0 ) .", 0, 1, errUnknownPred)
        , ("?- p1 ( v0 , v1 ) .", 0, 1, errArity)
        , ("?- p7 ( v0 , v1 ) , v1 = i5 .", 0, 8, errSort)
        , ("?- p8 ( v0 , v1 ) , p2 ( v1 , _ ) .", 0, 10, errSort)
        , ("?- p1 ( v0 ) , v1 > i2 .", 0, 6, errUnsafe)
        , ("p1000 ( v0 ) :- p1001 ( v0 ) . p1001 ( v0 ) :- p1 ( v0 ) , not p1000 ( v0 ) . ?- p1000 ( v0 ) .", 0, 10, errUnstratified)
        , ("p1000 ( v0 ) :- p1 ( v0 ) . p1000 ( v0 ) :- p7 ( v0 , _ ) .", 1, 10, errPrelude)
        , ("?- p8 ( v0 , v1 ) , v1 = count ( v2 : p1 ( v2 ) ) .", 0, 8, errAggregate)
        , ("p1000 ( _ ) :- p1 ( v0 ) . ?- p1000 ( v0 ) .", 0, 2, errHead)
        ]
    ]

-- | The transitive program's proofs, every node replayed: three
-- answers, the two-hop one a five-node tree, the others three.
proofReplay :: Bool
proofReplay = case (parseProgram (toks prog), replyObjWith respond (request prog facts4 [("why", Bool True)])) of
  (Right clauses, Just o)
    | Right chk <- check 0 clauses
    , Just (Success proof) <- fmap fromJSON (field o "proof") ->
        length (proof :: [[Integer]]) == 11 && replays clauses (naive chk db) proof
  _ -> False
 where
  prog = "p1000 ( v0 , v1 ) :- p9 ( v0 , v1 , _ , _ ) . p1000 ( v0 , v1 ) :- p1000 ( v0 , v2 ) , p9 ( v2 , v1 , _ , _ ) . ?- p1000 ( v0 , v1 ) ."
  db = IM.fromList [(c, S.fromList t) | (c, t) <- facts4]

aggregates :: Bool
aggregates =
  and
    [ answersOf "?- v0 = count ( v1 : p1 ( v1 ) ) ." [] ["answers"] == Just [rows [[0, 4]]]
    , answersOf "?- v0 = sum ( v1 : p8 ( _ , v1 ) ) ." [] ["answers"] == Just [rows [[0, 64]]]
    , answersOf "?- v0 = min ( v1 : p8 ( _ , v1 ) ) , v2 = max ( v1 : p8 ( _ , v1 ) ) ." [] ["answers"] == Just [rows [[0, 3, 40]]]
    , answersOf "?- v0 = count ( v1 : p9 ( v1 , v1 , _ , _ ) ) ." [] ["answers"] == Just [rows [[0, 0]]]
    , answersOf "?- v0 = min ( v1 : p8 ( _ , v1 ) , v1 > i100 ) ." [] ["answers", "errors"] == Just [rows [], rows []]
    , answersOf "?- p1 ( v0 ) , v1 = count ( v2 : p9 ( v0 , v2 , _ , _ ) ) ." [] ["answers"] == Just [rows [[0, 0, 1], [0, 1, 1], [0, 2, 0], [0, 3, 0]]]
    ]

arithmetic :: Bool
arithmetic =
  and
    [ answersOf "?- p8 ( v0 , v1 ) , v2 = v1 * i2 + i1 , v2 > i10 ." [] ["answers"] == Just [rows [[0, 1, 9, 19], [0, 2, 12, 25], [0, 3, 40, 81]]]
    , answersOf "?- p8 ( v0 , v1 ) , v1 = i9 ." [] ["answers"] == Just [rows [[0, 1, 9]]]
    , answersOf "?- p8 ( v0 , v1 ) , v2 = v1 / i0 ." [] ["answers", "errors"] == Just [rows [], rows []]
    , answersOf "?- p2 ( v0 , v2 ) , p2 ( v1 , v2 ) , v0 < v1 ." [] ["answers", "goals"] == Just [rows [[0, 0, 0, 1], [0, 2, 1, 3]], rows [[0, 0, 0, 1, 0]]]
    ]

goalsAndSorts :: Bool
goalsAndSorts =
  and
    [ answersOf "?- p8 ( v0 , v1 ) ." [] ["goals"] == Just [rows [[0, 0, 0, 3]]]
    , answersOf "assert p1000 ( v0 , i7 ) :- p1 ( v0 ) ." [] ["goals", "answers"] == Just [rows [[0, 1, 0, 3]], rows [[0, 0, 7], [0, 1, 7], [0, 2, 7], [0, 3, 7]]]
    , answersOf "p1000 ( v0 ) :- p1001 ( v0 ) . p1001 ( v0 ) :- p1000 ( v0 ) . ?- p1000 ( v0 ) ." [] ["goals", "answers", "preds"] == Just [rows [[0, 0, -1]], rows [], rows [[1000, -1], [1001, -1]]]
    , -- two rules of one predicate resolve ONE row; a fact-sorted
      -- position and an int-sorted one read back by name
      answersOf "p1000 ( v0 , v1 ) :- p8 ( v0 , v1 ) . p1000 ( v0 , i0 ) :- p1 ( v0 ) . ?- p1000 ( v0 , v1 ) ." [] ["preds", "answers"] == Just [rows [[1000, 0, 3]], rows [[0, 0, 0], [0, 0, 3], [0, 1, 0], [0, 1, 9], [0, 2, 0], [0, 2, 12], [0, 3, 0], [0, 3, 40]]]
    , -- an erroring program and an empty one carry no preds
      answersOf "?- p1 ( v0 ) , v1 != v0 ." [] ["preds"] == Just [rows []]
    ]

schemaEcho :: Bool
schemaEcho =
  answersOf "" [("schema", Bool True)] ["schema"] == Just [rows schemaRows]
    && answersOf "" [] ["schema"] == Just [Nothing]

refusals :: Bool
refusals =
  and
    [ refusedBy respond (program [[1]]) "token 0: malformed token"
    , refusedBy respond (program [[7, 0]]) "token 0: unknown token kind 7"
    , refusedBy respond (program [[1, -1]]) "token 0: negative token value"
    , refusedBy respond (program [[0, 500]]) "token 0: unknown predicate code 500"
    , isJust (replyObjWith respond (program [[2, -1]]))
    , refusedBy respond (tables ["99" .= rows []]) "facts 99: unknown fact predicate"
    , refusedBy respond (tables ["007" .= rows []]) "facts 007: unknown fact predicate"
    , refusedBy respond (tables ["1" .= rows [[0, 1]]]) "facts 1 0: malformed fact (need 1 values)"
    , refusedBy respond (tables ["1" .= rows [[-1]]]) "facts 1 0: negative fact value"
    , refusedBy respond (tables ["1" .= rows [[1], [1]]]) "facts 1 1: not strictly ascending"
    , refusedBy respond (request "" [] [("prelude", Number (-1))]) "negative prelude"
    ]
 where
  program toksRaw = setKey "program" (toJSON (toksRaw :: [[Integer]])) (request "" [] [])
  tables kvs = setKey "facts" (object kvs) (request "" [] [])

capped :: Bool
capped =
  fieldsOf respond big ["degraded", "reason", "answers", "errors"] == Just [Just (Bool True), Just "query_too_large", rows [], rows []]
    && overCap (tabled (fromInteger factCap + 1))
    && not (overCap (tabled (fromInteger factCap)))
    && either (>= 4) (const False) (evaluated 3)
    && either (const False) (\(_, _, n) -> n == 4) (evaluated 4)
 where
  big = setKey "program" (toJSON (replicate (fromInteger tokenCap + 1) [tokDot, 0])) (request "" [] [])
  tabled n = QueryReq Null [] [("1", replicate n [0])] 0 False False
  evaluated cap = case parseProgram (toks (prelude <> " ?- p1001 ( v0 ) .")) >>= either (const (Left 0)) Right . check 3 of
    Right chk -> evalProgramWith cap chk (IM.fromList [(c, S.fromList t) | (c, t) <- facts4])
    Left _ -> Left 0

-- | 8193 files asked with `why`: two nodes a tree, the last tree
-- would cross the budget and is left out whole.
proofBudget :: Bool
proofBudget =
  fieldsOf respond (request "?- p1 ( v0 ) ." [(1, [[n] | n <- [0 .. 8192]])] [("why", Bool True)]) ["counts"]
    == Just [countRow [0, 1, 0, 0, 8193, 0, 8193, 0, proofCap, 1]]

emptyProgram :: Bool
emptyProgram =
  answersOf "" [] ["answers", "goals", "proof", "errors", "degraded"] == Just [rows [], rows [], rows [], rows [], Just (Bool False)]
