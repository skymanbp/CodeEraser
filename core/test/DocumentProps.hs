-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | The document family's battery for step 3's five families (plan
-- v2.32 step 3; design booklet docs/reference/authority-track.md §5;
-- step 4's seven are DocumentProps4's): each family's empty document
-- is the catalogue's and has the fields its statement table below
-- names; over two hundred seeded requests per family every reference
-- names a stated class inside its ranges, every count is its rows'
-- measure, what sorts by a path sorts by its rank, and the same
-- request assembles to the same bytes; the contract refuses by name;
-- past the cap the empty document is answered with its reason; the
-- definition package carries the catalogue and its digest moves with
-- it.
module DocumentProps (battery) where

import CE.Document (catalogue, emptyOf, families, respond)
import CE.Document.Contract (DocFamily (..), docRowCap)
import CE.Lang (digestOf, pack)
import qualified CE.Resolve.Vocab as Vocab
import CE.Limits (limits)
import CE.Tables (package, tablesDigest)
import Data.Aeson
import qualified Data.Aeson.Key as Key
import qualified Data.Aeson.KeyMap as KM
import qualified Data.ByteString.Lazy as BL
import Data.Foldable (toList)
import Data.List (zip4)
import Data.Maybe (fromMaybe)
import qualified Data.Set as S
import DocumentGen (docRequest, requests)
import DocumentHarness (at, emptiesHeld, familiesNamed, fieldsHeld, int, ints, items, judgedBy, path, rankOf, refsHeld, refsIn, sameBytes)
import WireHarness (field, refusedBy, replyObjWith, runLegs, setKey)

battery :: IO Bool
battery =
  runLegs
    [ "every family's empty document is the catalogue's"
    , "every empty document has the fields its statement names"
    , "every reference in 200 documents per family is a stated class inside its ranges"
    , "every count in 200 documents per family is its rows' measure"
    , "folded references and listed findings sort by their path's rank"
    , "the same request assembles to the same bytes"
    , "the contract refuses by name"
    , "past the row cap the empty document, its reason named"
    , "the package carries the catalogue and the digest moves with it"
    , "the seeded documents are not vacuous: every road and every sorted list occurs"
    ]
    [emptiesHeld five, fieldsHeld fieldTable five, referencesHeld, countsMeasured, ranked, sameBytes (take 50 judged), refusals, capped, catalogued, seeded]

-- | Step 3's families.
five :: [DocFamily]
five = familiesNamed (words "arch query rules flow merge")

-- | Each family's fields, `family key type` (a `ref` is the reason, a
-- reference to the measuring side's text): today's report shape.
fieldTable :: String
fieldTable =
  "arch schema string\narch counts object\narch layers array\narch cuts array\narch clusters array\n\
  \arch misplaced array\narch impact array\narch metrics array\narch degraded ref\n\
  \query schema string\nquery program object\nquery goals array\nquery answers array\nquery proof array\n\
  \query errors array\nquery counts object\nquery degraded ref\n\
  \rules schema string\nrules program object\nrules goals array\nrules answers array\nrules proof array\n\
  \rules errors array\nrules counts object\nrules degraded ref\n\
  \flow schema string\nflow counts object\nflow findings array\nflow refused array\nflow degraded ref\n\
  \merge schema string\nmerge counts object\nmerge unsendable object\nmerge groups array\nmerge degraded ref\n"

-- | Every request with its document, per family.
judged :: [(DocFamily, Value, Value)]
judged = judgedBy requests five

referencesHeld :: Bool
referencesHeld = length judged == 1000 && refsHeld judged

countsMeasured :: Bool
countsMeasured = all measured judged
 where
  count d k = int (path ["counts", k] d)
  size r t = toInteger (length (ints (path ["rows", t] r)))
  factOf r k = int (path ["facts", k] r)
  measured (f, r, d) = case dfName f of
    "arch" ->
      and [count d t == size r t | t <- words "files dirs edges pkgEdges focus cuts misplaced impact"]
        && count d "clusters" == toInteger (S.size (S.fromList [c | [_, c] <- ints (path ["rows", "clusters"] r)]))
    "flow" ->
      count d "findings" == size r "findings"
        && count d "refused" == size r "unlowered" + size r "refused"
        && count d "shown" == toInteger (maybe 0 length (path ["findings"] d >>= asArray))
        && and [count d k == factOf r k | k <- words "units stmts vars uses dynamicUnits"]
        && count d "judged" == toInteger (length (filter (judgedFinding r) (ints (path ["rows", "findings"] r))))
    "merge" ->
      and [count d k == size r k | k <- words "groups members holes"]
        && count d "suggestions" == size r "groups"
        && count d "feasible" == toInteger (length [() | g <- ints (path ["rows", "groups"] r), take 1 (drop 4 g) == [1]])
        && and [count d k == factOf r k | k <- words "nodes merged_duplicates"]
        && and [int (path ["unsendable", k] d) == factOf r k | k <- words "not_isomorphic no_slot_table unbuilt over_cap"]
    _ -> and [count d k == factOf r k | k <- words "rules queries asserts strata facts derived answers violations proofNodes proofTruncated"]
  asArray v = case v of Array xs -> Just (toList xs); _ -> Nothing
  judgedFinding r row = case row of
    [file, _, k, _, _, _] -> k /= 3 && maybe False (`elem` flowJudged) (lookup file [(g, l) | [g, l] <- ints (path ["rows", "langs"] r)])
    _ -> False
  flowJudged = fromMaybe [] (path ["flow", "judged"] catalogue >>= asInts)
  asInts v = case fromJSON v of Success xs -> Just (xs :: [Integer]); _ -> Nothing

-- | A path reference's rank: a file's off `rankFiles`, a slashed
-- directory's off `rankDirs`.
placeOf :: Value -> Value -> Integer
placeOf r ref = case refsIn ref of
  [("slashed", _)] -> rankOf "rankDirs" r ref
  _ -> rankOf "rankFiles" r ref

ranked :: Bool
ranked = all sorted judged
 where
  sorted (f, r, d) = case dfName f of
    "arch" -> all (\c -> ascending (map (pairOf r) (list c "files"))) (list d "cuts")
    "flow" -> ascending (map (\x -> placeOf r (orNull (at x "path"))) (list d "findings")) && ascending (map (\x -> placeOf r (orNull (at x "path"))) (list d "refused"))
    _ -> True
  pairOf r x = (placeOf r (orNull (at x "from")), placeOf r (orNull (at x "to")))
  list = items
  orNull = fromMaybe Null
  ascending xs = and (zipWith (<=) xs (drop 1 xs))

-- | Each family's documents reach what the legs above read: folded
-- cuts with two references or more, two findings in one listing, all
-- three query roads and a degraded one, holes in a merge group.
seeded :: Bool
seeded = all (\(fam, test) -> any (\(f, _, d) -> dfName f == fam && test d) judged) cases
 where
  len v k = case at v k of Just (Array xs) -> length xs; _ -> 0
  cases =
    [ ("arch", \d -> any (\c -> len c "files" >= 2) (items d "cuts"))
    , ("flow", \d -> len d "findings" >= 2 && len d "refused" >= 2)
    , ("flow", \d -> at d "degraded" /= Just Null)
    , ("query", \d -> len d "goals" >= 1 && len d "proof" >= 1)
    , ("rules", \d -> any (\e -> at e "code" == Just (Number 0)) (items d "errors"))
    , ("rules", \d -> any (\e -> at e "code" /= Just (Number 0)) (items d "errors"))
    , ("query", \d -> at d "degraded" /= Just Null)
    , ("merge", \d -> any (\g -> len g "holes" >= 2) (items d "groups"))
    ]

-- | Each refusal: the request, edited, and the message it must name.
refusals :: Bool
refusals = and (zipWith (refusedBy respond) cases wanted)
 where
  arch = head'' (requests "arch")
  merge = head'' (requests "merge")
  flow = head'' (requests "flow")
  rowsSet t v r = setKey "rows" (setKey t v (orNull (at r "rows"))) r
  orNull = fromMaybe Null
  head'' = foldr const Null
  cases =
    [ setKey "family" "nope" arch
    , dropKey "ranges" arch
    , setKey "ranges" (object ["files" .= int (path ["ranges", "files"] arch), "dirs" .= int (path ["ranges", "dirs"] arch)]) arch
    , rowsSet "bogus" (toJSON [[0 :: Int]]) arch
    , rowsSet "layers" (toJSON [[0 :: Int]]) arch
    , rowsSet "misplaced" (toJSON [[99 :: Int, 0]]) arch
    , setKey "degraded" (toJSON (5 :: Int)) arch
    , setKey "degraded" (toJSON (0 :: Int)) arch
    , setKey "degraded" (toJSON (0 :: Int)) (rowsSet "groups" (toJSON ([] :: [Value])) (rowsSet "members" (toJSON ([] :: [Value])) (rowsSet "holes" (toJSON ([] :: [Value])) (setKey "ranges" (object ["members" .= (0 :: Int), "why" .= (1 :: Int)]) (setKey "facts" (object ["nodes" .= (1 :: Int), "merged_duplicates" .= (0 :: Int), "not_isomorphic" .= (0 :: Int), "no_slot_table" .= (0 :: Int), "unbuilt" .= (0 :: Int), "over_cap" .= (0 :: Int), "only" .= (0 :: Int)]) merge)))))
    , rowsSet "layers" (toJSON ([] :: [Value])) arch
    , rowsSet "shown" (toJSON [[0 :: Int], [-1]]) flow
    ]
  wanted =
    [ "document: unknown family nope"
    , "document: missing ranges"
    , "ranges: missing why"
    , "rows: unknown key bogus"
    , "layers 0: malformed row (need 2 columns)"
    , "misplaced 0: column 0 out of range files"
    , "degraded: out of range why"
    , "degraded: with rows files"
    , "degraded: with fact nodes"
    , "layers: not one row per slot in order"
    , "shown 1: unknown kind; the catalogue lists unreachable, dead_store, unused_local, unused_param"
    ]
  dropKey k r = case r of
    Object o -> Object (KM.delete (Key.fromString k) o)
    v -> v

capped :: Bool
capped = case replyObjWith respond request of
  Just o ->
    field o "degraded" == Just (Bool True)
      && field o "reason" == Just "document_too_large"
      && fmap Just (field o "document") == Just (fmap emptyOf flow)
      && field o "counts" == Just (object ["rows" .= (docRowCap + 1)])
  Nothing -> False
 where
  flow = lookup "flow" [(dfName f, f) | f <- families]
  request =
    docRequest
      "flow"
      [("files", 0), ("why", 1)]
      [("shown", replicate (fromInteger docRowCap + 1) [0])]
      [(k, 0) | k <- words "units stmts vars uses dynamicUnits"]
      Nothing

catalogued :: Bool
catalogued =
  at package "document" == Just catalogue
    && at package "resolve" == Just Vocab.table
    && at package "limits" == Just limits
    && dropDocument package == pack
    && tablesDigest == digestOf package
    && tablesDigest /= digestOf pack
    && all (\f -> path [dfName f, "schema"] catalogue == Just (toJSON (dfSchema f))) (filter (not . null . dfSchema) families)
    && path ["flow", "kinds"] catalogue == Just (toJSON flowKindRows)
    && path ["flow", "judged"] catalogue == Just (toJSON flowJudgedRows)
    && BL.length (encode catalogue) > 0
 where
  -- name, advisory, and the two display labels (plan v2.32 step 5, R8),
  -- column by column: the core's table is row by row
  flowKindRows :: [(String, Bool, String, String)]
  flowKindRows =
    zip4
      (words "unreachable dead_store unused_local unused_param")
      [False, False, False, True]
      ["unreachable", "dead store", "unused local", "unused parameter"]
      (words "不可达 死存储 未用局部量 未用形参")
  flowJudgedRows = [code | Just (Array rs) <- [path ["languages", "rows"] pack], Object row <- toList rs, KM.lookup "flow_judged" row == Just (Bool True), Just (Number code) <- [KM.lookup "code" row]]
  -- the package is the definition pack plus the three tables the core
  -- adds: the document catalogue, the resolve vocabulary (8.1.0) and
  -- the measuring side's limits (plan v2.33 W3)
  dropDocument v = case v of
    Object o -> Object (KM.delete "limits" (KM.delete "resolve" (KM.delete "document" o)))
    _ -> v
