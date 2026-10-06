-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | The core spells the report strings (plan v2.33 W7): the binder
-- (CE.Document.Bind) and the family rules (CE.Document.Spell) on hand
-- cases whose answers are worked out here, each the spelling the frozen
-- 1324c927 face gave (cli/tests/unit/document/frozen/; the seeded
-- differential against those copies rides the Rust suite,
-- unit/document/spelled/). A product request without `strings` is
-- answered as before: its lines keep their holes and references.
module SpellProps (battery) where

import CE.Document (respond)
import CE.Document.Bind (bindDocument, bindLine)
import CE.Document.Contract (DocReq (..))
import CE.Document.Spell (derived, resolverFor)
import CE.Text (Line (..), Piece (..))
import Data.Aeson
import Data.List (isInfixOf)
import qualified Data.ByteString.Char8 as B8
import qualified Data.ByteString.Lazy as BL
import CE.Protocol.Version (proto)
import qualified Data.Map.Strict as M
import WireHarness (runLegs)

battery :: IO Bool
battery =
  runLegs
    (lines legNames)
    [plainLists, rootSlashed, nodeNames, flowUnits, flowVars, clipped, mergeHoles, eraseDiff, trendShort, segNames, ranked, widths, holes, documents, unspelled]

legNames :: String
legNames =
  "a class's list answers by index, past it and below it nothing\n\
  \arch spells the root directory ./ and any other dir/\n\
  \deadcode spells a section path#unit and a file its path\n\
  \flow's unit is the first lowered at the place, else the first left out, else empty\n\
  \flow's variable is the first lowered unit's at the place\n\
  \merge clips on Rust's whitespace, counting characters\n\
  \merge's text is the hole row's naming that member and those nodes\n\
  \erase's diff is keyed by its first integer\n\
  \trend's short commit is the commit's first twelve\n\
  \docdup spells path:start-end kind, kind? past the vocabulary\n\
  \the path order is the strings below, ties sharing\n\
  \arch measures a directory in UTF-8 bytes and in characters\n\
  \a line fills its holes left to right and refuses a count that differs\n\
  \a document spells every reference and leaves look-alikes alone\n\
  \a request without strings is answered with its holes and references"

-- | A request of `family` over `strings` and `rows`.
req :: String -> Value -> Value -> DocReq
req family strings rowsV = case fromJSON (object ["id" .= (1 :: Int), "family" .= family, "rows" .= rowsV, "strings" .= strings]) of
  Success r -> r
  Error e -> error e

-- | `family`'s spelling of `[class, ints…]` over `strings` and `rows`.
spell :: String -> Value -> Value -> String -> [Integer] -> Maybe String
spell family strings rowsV = resolverFor family (req family strings rowsV)

noRows :: Value
noRows = object []

plainLists :: Bool
plainLists =
  map (spell "dedup" (object ["path" .= ["a.rs", "b.rs" :: String]]) noRows "path") [[0], [1], [2], [-1], [0, 0], []]
    == [Just "a.rs", Just "b.rs", Nothing, Nothing, Nothing, Nothing]

rootSlashed :: Bool
rootSlashed = map (spell "arch" (object ["dir" .= ["", "src" :: String]]) noRows "slashed") [[0], [1], [2]] == [Just "./", Just "src/", Nothing]

nodeNames :: Bool
nodeNames =
  map (spell "deadcode" (object ["path" .= ["a.rs", "a.rs" :: String], "node_unit" .= ["", "Intro" :: String]]) noRows "node_name") [[0], [1], [2]]
    == [Just "a.rs", Just "a.rs#Intro", Nothing]

-- | Two files: the first lowers `f` at 2 twice (the first counts) and
-- leaves `g` out at 3; the second holds nothing.
flowStrings :: Value
flowStrings =
  object
    [ "units" .= [[named 2 "f", named 2 "f2"], []]
    , "unlowered" .= [[named 3 "g", named 2 "h"], []]
    , "vars" .= [[listed 2 ["x", "y"], listed 2 ["z"]], []]
    ]
 where
  named :: Int -> String -> Value
  named line v = toJSON [toJSON line, toJSON v]
  listed :: Int -> [String] -> Value
  listed line vs = toJSON [toJSON line, toJSON vs]

flowUnits :: Bool
flowUnits = map (spell "flow" flowStrings noRows "unit") [[0, 2], [0, 3], [0, 9], [1, 0], [2, 0], [0, -1]] == [Just "f", Just "g", Just "", Just "", Nothing, Nothing]

flowVars :: Bool
flowVars = map (spell "flow" flowStrings noRows "var") [[0, 2, 1], [0, 2, 2], [0, 3, 0], [0, 2, -1]] == [Just "y", Nothing, Nothing, Nothing]

-- | Member 0 of group 0 holds hole rows at nodes (0, 1); the second
-- member none.
mergeRows :: Value
mergeRows = object ["members" .= [[0, 0, 0, 1, 1, 1, 1, 1], [1, 0, 1, 1, 1, 1, 1, 1 :: Int]], "holes" .= [[0, 0, 0, 0, 0, 1 :: Int]]]

mergeText :: Value
mergeText = object ["holeText" .= ["a \x3000 b\tc  d\x180e" :: String]]

clipped :: Bool
clipped =
  map (spell "merge" mergeText mergeRows "clipped") [[0, 0, 1, 99], [0, 0, 1, 4], [0, 0, 1, 0], [0, 0, 1, -1]]
    == [Just "a b c d\x180e", Just "a b …", Just "…", Nothing]

mergeHoles :: Bool
mergeHoles = map (spell "merge" mergeText mergeRows "text") [[0, 0, 1], [1, 0, 1], [0, 1, 1], [2, 0, 1]] == [Just "a \x3000 b\tc  d\x180e", Nothing, Nothing, Nothing]

eraseDiff :: Bool
eraseDiff =
  map (spell "erase" (object ["diff" .= [String "d0", Null, String "d2"]]) noRows "diff") [[0, 1], [1, 1], [2, 0], [2], [3, 0], []]
    == [Just "d0", Nothing, Just "d2", Just "d2", Nothing, Nothing]

trendShort :: Bool
trendShort = spell "trend" (object ["commit" .= ["0123456789abcdef0123" :: String]]) noRows "short" [0] == Just "0123456789ab"

segNames :: Bool
segNames =
  map (spell "docdup" (object ["path" .= ["a.txt" :: String]]) (object ["segs" .= [[0, 0, 3, 7, 0], [1, 0, 1, 2, 999], [2, 0, 1, 2, -1 :: Int]]]) "seg") [[0], [1], [2], [3]]
    == [Just "a.txt:3-7 md_para", Just "a.txt:1-2 kind?", Just "a.txt:1-2 kind?", Nothing]

-- | A table of `derived` for `family` over `strings`.
measured :: String -> Value -> String -> Maybe [[Integer]]
measured family strings table = dRows (derived (req family strings noRows)) >>= M.lookup table

ranked :: Bool
ranked =
  measured "sites" (object ["path" .= ["b", "a/", "a", "b", "é" :: String]]) "rankFiles" == Just [[0, 2], [1, 1], [2, 0], [3, 2], [4, 4]]
    && measured "arch" (object ["path" .= ["src/x.rs" :: String], "dir" .= ["", "src" :: String]]) "rankDirs" == Just [[0, 0], [1, 1]]

widths :: Bool
widths = measured "arch" (object ["path" .= ([] :: [String]), "dir" .= ["", "é😀" :: String]]) "widths" == Just [[0, 0, 0], [1, 6, 2]]

holes :: Bool
holes =
  bindLine r (Line 1 (Piece "{} and {}" [ref [0], ref [1]] [])) == Right (toJSON [toJSON (1 :: Int), "a.rs and {}"])
    && either (const True) (const False) (bindLine r (Line 0 (Piece "{}" [] [])))
    && either (const True) (const False) (bindLine r (Line 0 (Piece "{}" [ref [5]] [])))
 where
  r = spell "dedup" (object ["path" .= ["a.rs", "{}" :: String]]) noRows
  ref is = object ["$" .= (toJSON ("path" :: String) : map toJSON (is :: [Integer]))]

documents :: Bool
documents =
  bindDocument r doc == Right (object ["a" .= [toJSON (1 :: Int), "a.rs"], "b" .= object ["$" .= [toJSON ("path" :: String), toJSON (0 :: Int)], "c" .= Null]])
 where
  r = spell "dedup" (object ["path" .= ["a.rs" :: String]]) noRows
  doc = object ["a" .= [toJSON (1 :: Int), object ["$" .= [toJSON ("path" :: String), toJSON (0 :: Int)]]], "b" .= object ["$" .= [toJSON ("path" :: String), toJSON (0 :: Int)], "c" .= Null]]

-- | The churn family's request with a pair, asked with and without its
-- strings: the line spells the path only when the strings ride.
unspelled :: Bool
unspelled = holed (ask False) && not (holed (ask True)) && "a.rs" `isInfixOf` ask True
 where
  body s = object (["proto" .= proto, "type" .= ("document.request" :: String), "id" .= (1 :: Int), "family" .= ("churn" :: String), "ranges" .= object ["paths" .= (2 :: Int), "submodules" .= (0 :: Int)], "facts" .= object [k .= (0 :: Int) | k <- ["days", "commits", "appended", "rewrote", "surviving", "skipped"]], "rows" .= object ["cochange" .= [[0, 1, 2 :: Int]]], "lang" .= (0 :: Int)] <> ["strings" .= object ["path" .= ["a.rs", "b.rs" :: String], "submodule" .= ([] :: [String])] | s])
  ask s = either (const "") B8.unpack (respond proto (BL.toStrict (encode (body s))))
  holed out = "{\"$\"" `isInfixOf` out
