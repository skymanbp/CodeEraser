-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | The `ce similar` document (plan v2.32 step 5; design booklet
-- docs/reference/authority-track.md §5), transcribed from the face that
-- assembled it on the measuring side (cli/src/similar/face.rs `Row`,
-- `report_json`): the index revision the bags were built at, the query
-- as resolved, every candidate in the order the core answered — where
-- it is, what it is, its BM25 score's integer part, the six channel
-- hits, the shape bit, the arm that reached it and the core's
-- same-role bit — the counts, and the degraded reason. A degraded
-- document keeps its candidates (the measured order, no role bit:
-- each arm the core did not judge) beside the measuring side's reason.
-- The measuring side sends the candidates in display order as
-- integers; the query's label, a candidate's place and key are
-- references.
module CE.Similar.Document (doc) where

import CE.Document.Contract
import Data.Aeson (Value (..), object, (.=))
import Data.Foldable (asum)

doc :: DocFamily
doc = docFamily "similar" "ce.similar-report/0.1.0" statement checked assemble []

-- | A candidate row is [seat, nth, score, six channel hits, shape
-- equal, widened, role (0 not, 1 same-role, 2 unjudged)], in display
-- order; `terms` the query's terms, `widen` the switch, `similarRev`
-- the bags' revision. Everything rides beside a reason.
statement :: String
statement =
  "range seats\nrange why\n\
  \fact terms kept\nfact widen kept\nfact similarRev kept\n\
  \rows candidates 12 kept seats - - - - - - - - - - -\n\
  \ref at seats\nref key seats\nref label\nref why why\n"

checked :: DocReq -> Maybe String
checked req =
  asum
    [ asum [codes req "candidates" c 0 1 | c <- [9, 10]]
    , codes req "candidates" 11 0 2
    , bits req ["widen"]
    ]

assemble :: DocReq -> Value
assemble req =
  object
    [ "schema" .= dfSchema doc
    , "similar_rev" .= fact req "similarRev"
    , "query" .= object ["label" .= ref "label" [], "terms" .= fact req "terms", "widen" .= flag req "widen"]
    , "candidates" .= map candidate cands
    , "counts" .= counted (words "candidates role widened") (map count [const True, \r -> role r == 1, \r -> widened r == 1])
    , "degraded" .= whyRef req
    ]
 where
  cands = rows req "candidates"
  count p = toInteger (length (filter p cands))
  role = column 11
  widened = column 10
  column c r = sum (take 1 (drop c r))
  candidate r = case r of
    [seat, nth, score, n, p, c, d, s, l, shape, wide, rl] ->
      object
        [ "at" .= ref "at" [seat]
        , "key" .= ref "key" [seat]
        , "nth" .= nth
        , "role" .= (if rl == 2 then Null else Bool (rl == 1))
        , "score" .= score
        , "hits" .= [n, p, c, d, s, l]
        , "shape_equal" .= (shape == 1)
        , "widened" .= (wide == 1)
        ]
    _ -> object []
