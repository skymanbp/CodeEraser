-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | Step 4's seven document families (plan v2.32 step 4; design
-- booklet docs/reference/authority-track.md §5): check, structure,
-- join, deadcode, mentions, sites and the graph screen. The legs are
-- step 3's, read through DocumentHarness (see DocumentProps for what
-- each holds), plus the ones only these families need: the canvas
-- edges drop self-loops and package ends, the catalogue lists twelve
-- families, and the definition package holds the site kinds the
-- sites document names.
module DocumentProps4 (battery) where

import CE.Document (catalogue, respond)
import CE.Document.Contract (DocFamily (..))
import CE.Lang (pack)
import Data.Aeson
import Data.List (nub)
import Data.Maybe (fromMaybe)
import DocumentGen (docRequest)
import DocumentGen4 (requests4)
import DocumentHarness (at, emptiesHeld, familiesNamed, fieldsHeld, int, ints, items, judgedBy, path, rankOf, refsHeld, refsIn, sameBytes)
import WireHarness (refusedBy, runLegs, setKey)

battery :: IO Bool
battery =
  runLegs
    [ "every step-4 family's empty document is the catalogue's"
    , "every step-4 empty document has the fields its statement names"
    , "every reference in 200 documents per step-4 family is a stated class inside its ranges"
    , "every count in 200 documents per step-4 family is its rows' measure"
    , "join pairs, sites and canvas edges sort by their rank or index"
    , "the same step-4 request assembles to the same bytes"
    , "the step-4 contracts refuse by name"
    , "the catalogue lists the twelve families; the package holds the site kinds"
    , "the seeded step-4 documents are not vacuous"
    ]
    [emptiesHeld seven, fieldsHeld fieldTable seven, refsHeld judged && length judged == 1400, countsMeasured, ranked, sameBytes (take 70 judged), refusals, catalogued, seeded]

seven :: [DocFamily]
seven = familiesNamed (words "check structure join deadcode mentions sites graphscreen")

judged :: [(DocFamily, Value, Value)]
judged = judgedBy requests4 seven

-- | Each family's top-level fields in its empty document, `family key
-- type` (`ref`: the measuring side's reason; `?`: or null).
fieldTable :: String
fieldTable =
  "check schema string\ncheck score number\ncheck scoreScale number?\ncheck axes array\ncheck floor number?\n\
  \check candidates array\ncheck joinSeverity array\ncheck ratchet object\ncheck counts object\ncheck degraded ref\n\
  \structure schema string\nstructure score number\nstructure scoreScale number\nstructure entropy array\n\
  \structure axes array\nstructure findings array\nstructure dirs number\nstructure divergence number?\n\
  \structure deviations array\nstructure declaredDirs number\nstructure deep bool\nstructure days number?\n\
  \structure tree array\nstructure split bool\n\
  \join schema string\njoin days number\njoin commits number\njoin degraded ref\njoin files array\njoin units array\n\
  \deadcode schema string\ndeadcode dead array\ndeadcode reported array\ndeadcode counts object\n\
  \deadcode unresolved_sites number\ndeadcode degraded ref\n\
  \mentions schema string\nmentions mention_rev number\nmentions universe number\nmentions sources number\n\
  \mentions rows number\nmentions capped number\nmentions dist_js_dedup_runs number\nmentions skipped object\n\
  \mentions run object\nmentions outside object\nmentions rates object\n\
  \sites schema string\nsites sites array\n\
  \graphscreen schema string\ngraphscreen canvas object\ngraphscreen deadcode object\n"

countsMeasured :: Bool
countsMeasured = all measured judged
 where
  size r t = toInteger (length (ints (path ["rows", t] r)))
  factOf r k = int (path ["facts", k] r)
  rangeOf r k = int (path ["ranges", k] r)
  len d k = toInteger (length (items d k))
  measured (f, r, d) = case dfName f of
    "check" ->
      int (path ["counts", "files"] d) == rangeOf r "files"
        && and [int (path ["counts", k] d) == factOf r k | k <- words "simPairs members collapsed skippedSelf"]
        && toInteger (length (items (fromMaybe Null (at d "ratchet")) "failed")) == size r "failed"
        && (path ["ratchet", "dropped"] d /= Nothing) == (factOf r "droppedRode" == 1)
    "structure" ->
      int (at d "dirs") == rangeOf r "dirs"
        && len d "tree" == rangeOf r "dirs"
        && len d "findings" == size r "findings"
        && sum [toInteger (length (items n "axes")) | n <- items d "tree"] == size r "findings"
        && (at d "splitCandidates" /= Nothing) == (factOf r "split" == 1)
    "join" -> len d "files" == size r "files" && len d "units" == size r "units"
    "deadcode" -> deadMeasured r d
    "mentions" -> objectSize (at d "rates") == size r "rates" && int (at d "universe") == factOf r "universe"
    "sites" -> len d "sites" == size r "sites"
    _ ->
      deadMeasured r (fromMaybe Null (at d "deadcode"))
        && int (path ["canvas", "counts", "files"] d) == toInteger (length [() | [_, 0, _] <- ints (path ["rows", "graph"] r)])
        && int (path ["canvas", "counts", "dead"] d) == size r "dead"
        && int (path ["canvas", "counts", "edges"] d) == toInteger (length (items (fromMaybe Null (at d "canvas")) "edges"))
        && all (\e -> case e of [a, b] -> a /= b && max a b < int (path ["canvas", "counts", "files"] d); _ -> False) (ints (path ["canvas", "edges"] d))
  deadMeasured r d =
    int (path ["counts", "nodes"] d) == int (path ["ranges", "nodes"] r)
      && toInteger (length (items d "dead")) == size r "dead"
      && toInteger (length (items d "unmentioned")) == size r "unmentioned"
      && (at d "unmentioned" /= Nothing) == (factOf r "asked" == 1)
  objectSize v = case v of
    Just (Object o) -> toInteger (length o)
    _ -> -1

ranked :: Bool
ranked = all sorted judged
 where
  sorted (f, r, d) = case dfName f of
    "join" -> ascending [(place "rankPaths" r x "a", place "rankPaths" r x "b") | x <- items d "files"]
    "sites" -> ascending [(place "rankFiles" r x "path", second (at x "spec")) | x <- items d "sites"]
    "graphscreen" ->
      let canvas = fromMaybe Null (at d "canvas")
       in strictly (ints (at canvas "edges")) && strictly [second (at x "path") | x <- items canvas "files"]
    _ -> True
  place table r x k = rankOf table r (fromMaybe Null (at x k))
  second v = case refsIn (fromMaybe Null v) of
    [(_, x : _)] -> x
    _ -> -1
  ascending xs = and (zipWith (<=) xs (drop 1 xs))
  strictly xs = and (zipWith (<) xs (drop 1 xs))

-- | Each refusal: the family's first seeded request with no reason,
-- edited, and the message it must name.
refusals :: Bool
refusals = and (zipWith (refusedBy respond) cases wanted)
 where
  base fam = setKey "degraded" Null (foldr const Null (requests4 fam))
  rowsSet t v r = setKey "rows" (setKey t v (fromMaybe Null (at r "rows"))) r
  rowsOf :: [[Integer]] -> Value
  rowsOf = toJSON
  cases =
    [ rowsSet "failed" (rowsOf [[9]]) (base "check")
    , rowsSet "scale" (rowsOf [[1], [2]]) (base "check")
    , setKey "degraded" (toJSON (0 :: Int)) (base "structure")
    , rowsSet "tree" (rowsOf []) (base "structure")
    , scratch "join" [("paths", 1), ("units", 0), ("why", 1)] (("candidates", [[0, 0, 7, 0, 0, 0]]) : ("rankPaths", [[0, 0]]) : blank "files pos cochange joinSeverity units graphReason verdictReason") [("days", 14), ("commits", 0)]
    , rowsSet "dead" (rowsOf [[0, 0]]) (base "deadcode")
    , rowsSet "rates" (rowsOf [[99, 0, 0, 0, 0, 0, 0, 0, 0]]) (base "mentions")
    , scratch "sites" [("files", 1), ("sites", 1)] [("rankFiles", [[0, 0]]), ("langs", [[0, 0]]), ("sites", [[0, 0, 99, 1, 0, 0]])] []
    , scratch "graphscreen" [("nodes", 1), ("advisory", 0), ("why", 1)] (("graph", [[0, 5, -1]]) : blank "dead reported unmentioned edges pos cycles kept reason") (map (\k -> (k, 0)) (words "unresolvedSites asked dropped cut"))
    ]
  scratch fam rs ts fs = docRequest fam rs ts fs Nothing
  blank = map (\t -> (t, [])) . words
  wanted =
    [ "failed 0: column 0 is not a code 0..6"
    , "scale: more than one row"
    , "degraded: out of range why"
    , "tree: not one row per slot in order"
    , "candidates 0: column 2 is not a code 0..3"
    , "dead 0: column 1 is not a code 1..4"
    , "rates 0: no language 99"
    , "sites 0: column 2 is not a code 0..22"
    , "graph 0: column 1 is not a code 0..2"
    ]

catalogued :: Bool
catalogued =
  all (\f -> path [dfName f, "schema"] catalogue == Just (toJSON (dfSchema f))) seven
    && objectKeys catalogue == 12
    && fmap (\ks -> length ks == 23 && nub ks == ks) (path ["store", "site_kinds"] pack >>= asList) == Just True
 where
  objectKeys v = case v of
    Object o -> length o
    _ -> -1
  asList v = case fromJSON v of
    Success xs -> Just (xs :: [String])
    _ -> Nothing

-- | Each family reaches what the legs above read.
seeded :: Bool
seeded = all (\(fam, test) -> any (\(f, _, d) -> dfName f == fam && test d) judged) cases
 where
  cases =
    [ ("check", \d -> path ["ratchet", "dropped"] d /= Nothing && at d "degraded" == Just (String "verdict_too_large"))
    , ("check", \d -> length (items d "candidates") >= 2)
    , ("structure", \d -> length (items d "splitCandidates") >= 1 && any (\n -> length (items n "axes") >= 2) (items d "tree"))
    , ("join", \d -> length (items d "files") >= 2 && any (\x -> at x "verdict" /= Just Null) (items d "files"))
    , ("join", \d -> any (\u -> at u "kind" == Just "t3") (items d "units") && at d "degraded" == Just (String "graph_too_large"))
    , ("deadcode", \d -> length (items d "unmentioned") >= 2 && any (\x -> at x "confidence" /= Just Null) (items d "dead"))
    , ("deadcode", \d -> at d "unmentioned_dropped" == Just (Bool True))
    , ("mentions", \d -> objectSizeOf d "rates" >= 2)
    , ("sites", \d -> length (items d "sites") >= 3 && any (\x -> at x "owner" /= Just Null) (items d "sites"))
    , ("graphscreen", \d -> length (items (canvasOf d) "edges") >= 2 && any (\x -> at x "cycle" == Just (Bool True)) (items (canvasOf d) "files"))
    ]
  canvasOf d = fromMaybe Null (at d "canvas")
  objectSizeOf d k = case at d k of
    Just (Object o) -> length o
    _ -> 0
