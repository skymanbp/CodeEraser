-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | The `[rules.pack]` battery (plan v2.33; CE.Query.Pack): every item
-- renders to a program that lexes and checks clean beside the shipped
-- prelude; a rule that redefines a pack predicate — or any `pack_…`
-- predicate — is program error 7 at the user's own token; each item's
-- witness rows, through the real respond over 200 seeded fact tables,
-- equal the set comprehensions of ReferencePack; the rendered text
-- rides back in the lex reply and its tokens read `pack line:column`;
-- and every refusal names its cause — the two list caps among them.
module PackProps (battery) where

import CE.Query (respond)
import CE.Query.Cost (errPrelude, forbidCap, layerCap)
import Data.Aeson
import qualified Data.Aeson.Key as Key
import Data.List (isPrefixOf)
import qualified Data.Map.Strict as M
import Data.Maybe (fromMaybe)
import qualified Data.Set as S
import ReferencePack
import System.IO (IOMode (ReadMode), hGetContents', hSetEncoding, utf8, withFile)
import WireHarness (field, refusedBy, replyObjWith, runLegs)

battery :: IO Bool
battery = do
  prelude <- withFile "../cli/src/query/prelude.rules" ReadMode (\h -> hSetEncoding h utf8 >> hGetContents' h)
  runLegs names (probes prelude)

names :: [String]
names =
  [ "every item, and all five together, renders to a program that lexes and checks clean"
  , "a rule defining a pack predicate is program error 7 at the rules file's token; reading one is fine"
  , "no_dir_cycles: witness rows equal the reference on 200 seeded tables, 40 or more witnessing"
  , "fan_in_max: witness rows equal the reference on 200 seeded tables, 40 or more witnessing"
  , "fan_out_max: witness rows equal the reference on 200 seeded tables, 40 or more witnessing"
  , "layers: witness rows equal the reference on 200 seeded tables, 40 or more witnessing, a glob member that is no file witnessing nothing"
  , "forbid: witness rows equal the reference on 200 seeded tables, 40 or more witnessing, a glob member that is no file witnessing nothing"
  , "the lex reply carries the rendered text, its tokens read `pack line:column`, the prelude count stays the prelude's"
  , "every pack refusal names its cause, the two caps among them"
  ]

probes :: String -> [Bool]
probes prelude =
  [ all (clean prelude) (every : map (: []) every)
  , reserved prelude
  , agrees prelude (const [["no_dir_cycles"]]) (const cycles)
  , agrees prelude (\s -> [["fan_in_max", toJSON (s `mod` 3)]]) (fanIn . (`mod` 3))
  , agrees prelude (\s -> [["fan_out_max", toJSON (s `mod` 3)]]) (fanOut . (`mod` 3))
  , agrees prelude (\s -> [toJSON ("layers" :: String) : map toJSON (layersOf s)]) (layered . layersOf) && filesOnly prelude (\s -> [toJSON ("layers" :: String) : map toJSON (layersOf s)]) (layered . layersOf)
  , agrees prelude (map (\(a, b) -> ["forbid", toJSON a, toJSON b]) . pairsOf) (forbidden . pairsOf) && filesOnly prelude (map (\(a, b) -> ["forbid", toJSON a, toJSON b]) . pairsOf) (forbidden . pairsOf)
  , lexed prelude
  , refusals prelude
  ]
 where
  every = [["no_dir_cycles"], ["fan_in_max", Number 3], ["fan_out_max", Number 2], ["layers", "a/**", "b/**", "c/**"], ["forbid", "b/**", "a/**"]]

-- | A request: the prelude, a rules text, the pack rows (no `pack`
-- key when there are none), the facts.
request :: String -> Maybe String -> [[Value]] -> [(Int, [[Integer]])] -> [(Key.Key, Value)] -> Value
request prelude rules pack facts extra =
  object
    ( [ "proto" .= ("7.0.0" :: String)
      , "type" .= ("query.request" :: String)
      , "id" .= (1 :: Int)
      , "texts" .= (Just prelude, rules, Nothing :: Maybe String)
      , "facts" .= object [Key.fromString (show c) .= t | (c, t) <- facts]
      ]
        <> ["pack" .= pack | not (null pack)]
        <> extra
    )

-- | The reply object of a request.
reply :: Value -> Maybe Object
reply = replyObjWith respond

-- | The lexed object of a lex request over a pack.
lexOf :: String -> Maybe String -> [[Value]] -> Maybe Object
lexOf prelude rules pack = do
  o <- reply (request prelude rules pack [] ["lex" .= True])
  Object l <- field o "lexed"
  pure l

clean :: String -> [[Value]] -> Bool
clean prelude pack = case reply (request prelude Nothing pack [] []) of
  Just o -> field o "errors" == Just (toJSON ([] :: [Value])) && field o "degraded" == Just (Bool False)
  Nothing -> False

-- | Error 7 at a rules-file token for a redefined pack predicate (one
-- the pack defines, and one no pack defines); none for a read.
reserved :: String -> Bool
reserved prelude =
  sevenAt [["no_dir_cycles"]] "pack_dir_reach(D, E) :- dir(D), dir(E)."
    && sevenAt [] "pack_mine(D) :- dir(D)."
    && fmap (field' "errors") (reply (request prelude (Just "?- pack_dir_reach(D, E).") [["no_dir_cycles"]] [] [])) == Just (Just (toJSON ([] :: [Value])))
 where
  field' = flip field
  sevenAt pack rules = fromMaybe False $ do
    o <- reply (request prelude (Just rules) pack [] [])
    Success [[at, code]] <- fromJSON <$> field o "errors" :: Maybe (Result [[Int]])
    l <- lexOf prelude (Just rules) pack
    Success at' <- fromJSON <$> field l "at" :: Maybe (Result [String])
    pure (code == fromInteger errPrelude && "rules 1:1" == at' !! at)

-- | The seeds: 200 per item.
seeds :: [Integer]
seeds = [1 .. 200]

layersOf :: Integer -> [String]
layersOf s = ["l" <> show i <> "/**" | i <- [0 .. 1 + s `mod` 3]]

pairsOf :: Integer -> [(String, String)]
pairsOf s = take (fromInteger (1 + s `mod` 3)) [(pool (s + i), pool (s * 7 + i * 3)) | i <- [0 ..]]
 where
  pool k = "g" <> show (k `mod` 4) <> "/**"

-- | Every seed's tables judged through the real respond with the item's
-- pack equal the reference's rows, and at least 40 of the 200 seeds
-- witness a violation (an all-empty agreement proves nothing); the set
-- table seats each glob's members under the id the lex reply gives the
-- glob.
agrees :: String -> (Integer -> [[Value]]) -> (Integer -> Tables -> S.Set [Integer]) -> Bool
agrees prelude packOf reference = all (== Just True) [fmap (\(ok, _, _) -> ok) o | o <- outcomes] && length [() | Just (_, True, _) <- outcomes] >= 40
 where
  outcomes = judged prelude packOf reference

-- | Some seed's rows would differ were every glob member a file: an arc
-- with an end that is no file is reached, and the core kept it out.
filesOnly :: String -> (Integer -> [[Value]]) -> (Integer -> Tables -> S.Set [Integer]) -> Bool
filesOnly prelude packOf reference = or [moved | Just (_, _, moved) <- judged prelude packOf reference]

-- | Each seed: the core's rows equal the reference's, the reference
-- witnesses something, and its rows move when every node is a file.
judged :: String -> (Integer -> [[Value]]) -> (Integer -> Tables -> S.Set [Integer]) -> [Maybe (Bool, Bool, Bool)]
judged prelude packOf reference = map one seeds
 where
  one s = do
    let pack = packOf s
    l <- lexOf prelude Nothing pack
    Success globs <- fromJSON <$> field l "sets" :: Maybe (Result [String])
    let t = tables s globs
        sets = [[i, v] | (i, g) <- zip [0 ..] globs, v <- S.toList (M.findWithDefault S.empty g (tMembers t))]
        facts = [(1, [[v] | v <- S.toList (tFiles t)]), (2, [[v, d] | (v, d) <- M.toList (tDir t)]), (9, S.toList (tRefs t)), (26, S.toAscList (S.fromList sets))]
    o <- reply (request prelude Nothing pack facts [])
    Success answers <- fromJSON <$> field o "answers" :: Maybe (Result [[Integer]])
    let want = reference s t
    pure (S.fromList answers == want && length answers == S.size want, not (S.null want), reference s t {tFiles = S.fromList (tNodes t)} /= want)

-- | The lex reply over a pack: the text it rendered, tokens placed in
-- the pack, the prelude count unchanged by the pack's clauses.
lexed :: String -> Bool
lexed prelude = fromMaybe False $ do
  bare <- lexOf prelude Nothing []
  l <- lexOf prelude Nothing [["no_dir_cycles"]]
  Success text <- fromJSON <$> field l "packText" :: Maybe (Result String)
  Success at <- fromJSON <$> field l "at" :: Maybe (Result [String])
  pure
    ( "# [rules.pack] no_dir_cycles = true\npack_dir_reach(D, E) :- dir_ref(D, E).\n" `isPrefixOf` text
        && "pack 2:1" `elem` at
        && field l "prelude" == field bare "prelude"
        && field l "clauses" /= field bare "clauses"
        && field bare "packText" == Nothing
    )

refusals :: String -> Bool
refusals prelude =
  and
    [ refused [["layers", "a/**"]] "pack layers: 1 glob(s), a layering needs at least two"
    , refused [toJSON ("layers" :: String) : [toJSON ("l" <> show i) | i <- [0 .. layerCap]]] ("pack layers: 17 globs, over the cap " <> show layerCap)
    , refused [["forbid", toJSON ("a" <> show i), "b"] | i <- [0 .. forbidCap]] ("pack forbid: 65 pairs, over the cap " <> show forbidCap)
    , refused [["fan_in_max", Number 1], ["fan_in_max", Number 2]] "pack fan_in_max: declared twice"
    , refused [["fan_out_max", Number (-1)]] "pack fan_out_max: -1 is not a count"
    , refused [["fan_out_max", "3"]] "pack fan_out_max: malformed (need one integer)"
    , refused [["forbid", "a\"b", "c"]] "holds a quote or a newline"
    , refused [["forbid", "a"]] "pack forbid: malformed (need two globs)"
    , refused [["no_dir_cycles", Bool True]] "pack no_dir_cycles: malformed (need no parameters)"
    , refused [["dir_cycles"]] "pack: unknown item \"dir_cycles\""
    , refused [[Number 3]] "pack: malformed item"
    , refusedBy respond (object ["proto" .= ("7.0.0" :: String), "type" .= ("query.request" :: String), "id" .= (1 :: Int), "pack" .= [["no_dir_cycles" :: String]]]) "pack without texts"
    ]
 where
  refused pack = refusedBy respond (request prelude Nothing pack [] [])
