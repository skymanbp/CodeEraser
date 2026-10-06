-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | The bags family (plan v2.33 W6): Porter's worked examples as the
-- measuring side's battery held them before the move, the splitter's
-- named cases, the channel tag, two hundred seeded rows agreeing with
-- ReferenceBags on every bag, the contract's refusals by name, and the
-- cap. The term road's equality with the frozen Rust road is the tests
-- subrepo's differential (unit/similar/bags_diff.rs), over the wire.
module BagsProps (battery) where

import CE.Similar.Bags (BagsReq (..), bagsCap, overCap, respond)
import CE.Similar.Stem (stem)
import CE.Similar.Terms (proseWords, splitIdent, wordTerm)
import Data.Aeson (Value (..), object, toJSON, (.=))
import Data.Aeson.Types (Pair)
import ReferenceBags (BCase (..), cases, encodeCase, expected)
import WireHarness (fieldsOf, refusedBy, runLegs)

battery :: IO Bool
battery =
  runLegs
    [ "Porter 1980's worked examples stem as published"
    , "short and non-ASCII words are left alone; a matched ed with no vowel still tidies (bed → bede)"
    , "identifiers split at case, underscore and digit boundaries"
    , "prose drops stop words but never identifier pieces"
    , "a term is channel-tagged and stemmed"
    , "two hundred seeded rows agree with the reference on every bag"
    , "the seeded rows are not vacuous: anonymous, impl, arity, every return flag, structure, docs"
    , "the contract's refusals name the offender"
    , "the cap counts every item"
    ]
    [ all (\(w, s) -> stem w == s) paper
    , stem "is" == "is" && stem "café" == "café" && stem "x1" == "x1" && stem "bed" == "bede"
    , all (\(i, ws) -> splitIdent i == ws) splits
    , proseWords "Returns the parsed JSON of a file" == ["returns", "parsed", "json", "file"]
        && splitIdent "is_ready" == ["is", "ready"]
    , wordTerm 0 "fetch" /= wordTerm 2 "fetch" && wordTerm 3 "fetching" == wordTerm 3 "fetch"
    , all (\c -> fieldsOf respond (encodeCase c) ["bags", "texts"] == Just (expected c)) cases
    , nonVacuous
    , all refused refusals
    , not (overCap (sized bagsCap)) && overCap (sized (bagsCap + 1))
    ]

paper :: [(String, String)]
paper =
  map (\l -> let (a, b) = break (== ' ') l in (a, drop 1 b)) . lines $
    "caresses caress\nponies poni\ncats cat\nagreed agre\nplastered plaster\nmotoring motor\n\
    \conflated conflat\nhopping hop\nfiling file\nhappy happi\nsky sky\nrelational relat\n\
    \rational ration\ndigitizer digit\nhesitanci hesit\nelectrical electr\nhopeful hope\n\
    \adjustment adjust\nadoption adopt\ncontroll control\ngeneralizations gener"

splits :: [(String, [String])]
splits =
  [ ("parseJSONFile", ["parse", "json", "file"])
  , ("http2_server", ["http", "2", "server"])
  , ("(T) add", ["t", "add"])
  , ("getX", ["get", "x"])
  , ("ABC", ["abc"])
  , ("__init__", ["init"])
  , ("", [])
  ]

nonVacuous :: Bool
nonVacuous =
  any ((== "(anonymous)") . take 11 . bKey) cases
    && any ((== "impl ") . take 5 . bKey) cases
    && any (elem '/' . bKey) cases
    && all (\r -> any ((== r) . bRet) cases) [Nothing, Just 0, Just 1]
    && any (not . null . bStruct) cases
    && any (not . null . bDocs) cases

refused :: (Value, String) -> Bool
refused (r, want) = refusedBy respond r want

refusals :: [(Value, String)]
refusals =
  [ (withUnit [String "k", String "fn", Number 2, empty, empty, empty, empty], "units 0")
  , (withUnit [String "k", String "fn", Null, empty, empty, rows [[5, 1], [3, 1]], empty], "units 0")
  , (withUnit [String "k", String "fn", Null, empty, empty, rows [[5, 0]], empty], "units 0")
  , (withUnit [String "k", String "fn", Null, empty, empty, rows [[2 ^ (64 :: Int), 1]], empty], "units 0")
  , (request ["inspect" .= [-1 :: Integer]], "inspect")
  ]
 where
  empty = Array mempty
  rows = toJSON :: [[Integer]] -> Value
  withUnit u = request ["units" .= [u]]

request :: [Pair] -> Value
request extra = object (["proto" .= ("9.0.0" :: String), "type" .= ("bags.request" :: String), "id" .= (1 :: Int)] <> extra)

sized :: Int -> BagsReq
sized n = BagsReq Null [] [] (replicate n 0)
