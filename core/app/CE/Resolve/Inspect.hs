{-# LANGUAGE OverloadedStrings #-}

-- | The readers' answers on their own (plan v2.33 W2-text): a request's
-- `inspect` object asks this family's configuration readers directly,
-- and the reply's `inspected` object answers key by key — the
-- differential gate (tests subrepo unit/graph/resolve_text/) holds every
-- answer against the frozen Rust reader it replaced. No ladder reads it;
-- an absent key is an empty question.
--   split      [[mode, text]] → [[word]] (mode: command, gnuJson, gnu,
--              windows, windowsProgram, shaped → [bool])
--   chain      [[base, dir, [arg]]] → [chain]
--   relativize [[base, dir, path]] → [path | null]
--   goMod      [[rel, text]] → [[dir, module | null, [[old, new]]]]
--   description [[rel, text]] → [[dir, package, [collated]] | null]
--   cabal      [[rel, text]] → [[dir, name, [[[root], main | null,
--              library]], [dep], hasLibrary, [hidden], [exposed]]]
--   pyproject  [doc | null] → [[[dir], [dep]] | null]
--   db         [[base, rows | null, [[rel, text | null]]]] → [[[[unit,
--              dir | null, chain]], [response], [wanted]]]
--   flags      [[base, rel, text]] → [[dir, chain]]
--   text       [text] → [[[line], trimmed, [word]]]
--   chars      true → {white: [[lo, hi]], alnum: [[lo, hi]]} over every
--              scalar value
-- A chain is `[msvc, ownDir, quote, bracket, system, forced]`, a searched
-- place `[0 | 1, dir]` (1 = a framework directory).
module CE.Resolve.Inspect (inspected) where

import CE.Resolve.Cabal (Cabal (..), Stanza (..))
import qualified CE.Resolve.Cabal as Cabal
import CE.Resolve.Chars (isRustAlnum, isRustWhite)
import CE.Resolve.Cmdline
import CE.Resolve.CompDb (Entry (..), Expanded (..), parseDb, parseFlags, relativize)
import CE.Resolve.Description (Description (..), readDescription)
import CE.Resolve.Flags (Chain (..), Search (..), chain)
import CE.Resolve.Go (GoMod (..), parseGoMod)
import CE.Resolve.Py (PyProject (..), pyproject)
import CE.Resolve.Str (parentDir, rustLines, rustTrim, splitWhitespace)
import Data.Aeson (Value (..), object, toJSON, (.=))
import qualified Data.Aeson.Key as K
import qualified Data.Aeson.KeyMap as KM
import Data.Aeson.Types (FromJSON, parseMaybe, parseJSON)
import Data.Char (chr)
import qualified Data.Map.Strict as M
import qualified Data.Set as Set

inspected :: Value -> Value
inspected (Object o) = object (concat [maybe [] (\v -> [K.fromString k .= v]) (answer k =<< KM.lookup (K.fromString k) o) | k <- keys])
 where
  keys = ["split", "chain", "relativize", "goMod", "description", "cabal", "pyproject", "db", "flags", "text", "chars"]
inspected _ = object []

-- | One key's answer, Nothing when its question does not read.
answer :: String -> Value -> Maybe Value
answer k v = case k of
  "split" -> each v (\(mode, text) -> splitBy mode text)
  "chain" -> each v (\(base, dir, argv) -> chainJson (chain argv (relativize base dir)))
  "relativize" -> each v (\(base, dir, path) -> toJSON (relativize base dir path))
  "goMod" -> each v (\(rel, text) -> let m = parseGoMod rel text in toJSON (gmDir m, gmModule m, gmReplaces m))
  "description" -> each v (\(rel, text) -> toJSON ((\d -> (dDir d, dPackage d, dCollate d)) <$> readDescription (parentDir rel) text))
  "cabal" -> each v (\(rel, text) -> cabalJson (Cabal.parse rel text))
  "pyproject" -> each v (\doc -> toJSON ((\p -> (ppSourceDirs p, ppDeps p)) <$> pyproject doc))
  "db" -> each v (\(base, rows, texts) -> dbJson (maybe ([], mempty) (parseDb base (M.fromList texts)) rows))
  "flags" -> each v (\(base, rel, text) -> let (d, c) = parseFlags base rel text in toJSON [toJSON (d :: String), chainJson c])
  "text" -> each v (\t -> toJSON (rustLines t, rustTrim t, splitWhitespace t))
  "chars" -> Just (object ["white" .= ranges isRustWhite, "alnum" .= ranges isRustAlnum])
  _ -> Nothing

-- | Every question of a list, answered in order.
each :: (FromJSON a) => Value -> (a -> Value) -> Maybe Value
each v f = toJSON . map f <$> parseMaybe parseJSON v

splitBy :: String -> String -> Value
splitBy mode text = case mode of
  "command" -> toJSON (splitCommand text)
  "gnuJson" -> toJSON (splitGnuJson text)
  "gnu" -> toJSON (splitGnu text)
  "windows" -> toJSON (splitWindows text False)
  "windowsProgram" -> toJSON (splitWindows text True)
  _ -> toJSON [windowsShaped text, isMsvc text]

chainJson :: Chain -> Value
chainJson c = toJSON [toJSON (chMsvc c), toJSON (chOwnDir c), places (chQuote c), places (chBracket c), places (chSystem c), toJSON (chForced c)]
 where
  places = toJSON . map place
  place (SDir d) = toJSON [toJSON (0 :: Int), toJSON d]
  place (SFramework d) = toJSON [toJSON (1 :: Int), toJSON d]

cabalJson :: Cabal -> Value
cabalJson c = toJSON [toJSON (cDir c), toJSON (cName c), toJSON [toJSON [toJSON (stRoots s), toJSON (stMain s), toJSON (stLibrary s)] | s <- cStanzas c], toJSON (cDeps c), toJSON (cHasLibrary c), toJSON (Set.toList (cHidden c)), toJSON (Set.toList (cExposed c))]

dbJson :: ([Entry], Expanded) -> Value
dbJson (entries, x) = toJSON [toJSON [toJSON [toJSON (eUnit e), toJSON (eDir e), chainJson (eChain e)] | e <- entries], toJSON (Set.toList (xResponses x)), toJSON (Set.toList (xWanted x))]

-- | The scalar values a predicate holds for, as inclusive ranges.
ranges :: (Char -> Bool) -> [[Int]]
ranges p = go [i | i <- [0 .. 0x10FFFF], i < 0xD800 || i > 0xDFFF, p (chr i)]
 where
  go [] = []
  go (x : xs) = let (run, rest) = spanRun x xs in [x, run] : go rest
  spanRun lo (y : ys) | y == lo + 1 || (lo == 0xD7FF && y == 0xE000) = spanRun y ys
  spanRun lo ys = (lo, ys)
