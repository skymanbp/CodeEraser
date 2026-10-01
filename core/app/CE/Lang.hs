{-# LANGUAGE OverloadedStrings #-}

-- | The definition package (plan v2.32 step 1; design booklet
-- docs/reference/authority-track.md §4): every judged language's
-- tables and the common ones, as the one JSON object `tables/1`
-- answers, with its digest. Each language is one document read whole
-- (CE.Lang.Spec's `LangTables`), the common tables one more; a data
-- module holds only the chunks of its document, and this module names
-- the order they are read in. The docdup numbers are the ones
-- CE.Docdup.Cost already owns — grafted here, never restated.
module CE.Lang (allTables, digestOf, languages, pack) where

import qualified CE.Docdup.Cost as Doc
import qualified CE.Lang.C as C
import qualified CE.Lang.C.Flow as CF
import qualified CE.Lang.Common as Common
import qualified CE.Lang.Common.Boot1 as Boot1
import qualified CE.Lang.Common.Boot2 as Boot2
import qualified CE.Lang.Common.Boot3 as Boot3
import qualified CE.Lang.Common.Graph as Graph
import qualified CE.Lang.Common.Ladder as Ladder
import qualified CE.Lang.Common.Ladder2 as Ladder2
import qualified CE.Lang.Common.Prose as Prose
import qualified CE.Lang.Common.Protocol as Protocol
import qualified CE.Lang.Cpp as Cpp
import qualified CE.Lang.Go as Go
import qualified CE.Lang.Go.Flow as GoF
import qualified CE.Lang.Haskell as Hs
import qualified CE.Lang.Html as Html
import qualified CE.Lang.Java as Java
import qualified CE.Lang.Java.Flow as JavaF
import qualified CE.Lang.Lua as Lua
import qualified CE.Lang.Lua.Flow as LuaF
import qualified CE.Lang.Markdown as Md
import qualified CE.Lang.Python as Py
import qualified CE.Lang.Python.Flow as PyF
import qualified CE.Lang.R as R
import qualified CE.Lang.R.Flow as RF
import qualified CE.Lang.Rust as Rs
import qualified CE.Lang.Rust.Flow as RsF
import CE.Lang.Spec (LangTables (..), Language)
import CE.Lang.Table (rows)
import CE.Lang.Toml (toml)
import qualified CE.Lang.Tsx as Tsx
import qualified CE.Lang.TypeScript as Ts
import qualified CE.Lang.TypeScript.Flow as TsF
import Data.Aeson
import qualified Data.Aeson.Key as K
import qualified Data.Aeson.KeyMap as KM
import Data.Aeson.Types (parseEither)
import Data.Bits (xor)
import qualified Data.ByteString.Lazy as BL
import Data.Maybe (fromMaybe)
import Data.Word (Word64, Word8)

-- | The judged languages' tables, in wire-code order. TSX reads
-- TypeScript's document with its own slot piece; C++ reads C's with its
-- overloads, three `std::` names before C's noreturn list closes, and
-- its own slot piece — the way the measuring side spliced its texts.
allTables :: [LangTables]
allTables = map (decoded "language") (scripted <> compiled)

-- | Python through HTML.
scripted :: [String]
scripted =
  [ Py.top <> Py.scan <> Py.scan2 <> PyF.flow <> PyF.flow2 <> Py.slot
  , Ts.name <> typescript <> Ts.only
  , Tsx.name <> typescript <> Tsx.only
  , Rs.top <> Rs.scan <> Rs.scan2 <> RsF.flow <> RsF.flow2 <> Rs.slot
  , Go.top <> Go.scan <> Go.scan2 <> GoF.flow <> GoF.flow2 <> Go.slot
  , Md.top <> Md.scan
  , Hs.top <> Hs.scan <> Hs.scan2 <> Hs.scan3 <> Hs.slot <> Hs.slot2
  , Html.top <> Md.scan <> Html.scan
  ]
 where
  typescript = Ts.top <> Ts.scan <> Ts.scan2 <> TsF.flow <> TsF.flow2 <> Ts.shared

-- | C through R.
compiled :: [String]
compiled =
  [ C.name <> cScan <> CF.flow <> CF.flow2 <> CF.noreturn <> CF.close <> C.shared <> C.only
  , Cpp.name <> cScan <> Cpp.overloads <> CF.flow <> CF.flow2 <> CF.noreturn <> Cpp.std
      <> CF.close <> C.shared <> Cpp.only
  , Lua.top <> Lua.scan <> Lua.scan2 <> Lua.scan3 <> LuaF.flow <> Lua.slot
  , Java.top <> Java.scan <> Java.scan2 <> Java.scan3 <> JavaF.flow <> JavaF.flow2 <> Java.slot
  , R.top <> R.scan <> R.scan2 <> RF.flow <> RF.flow2 <> R.slot
  ]
 where
  cScan = C.top <> C.scan <> C.scan2 <> C.scan3

-- | The common document: outputs, the language rows, walk, the
-- four-class, key, flag, compdb and call tables, the prose vocabularies,
-- the protocol names and the ladders' name tables.
common :: Value
common =
  decoded "common" $
    Common.outputs <> Common.languages <> Common.languages2 <> Common.walk
      <> Graph.fourclass <> Graph.fourclass2 <> Graph.keys <> Graph.flags <> Graph.compdb
      <> Graph.calls <> Prose.tombstone <> Prose.tombstone2 <> Prose.docdup
      <> Protocol.protocol <> Protocol.protocol2 <> Protocol.protocol3
      <> Ladder.java <> Ladder.java2 <> Ladder.java3 <> Ladder.ts
      <> Ladder2.go <> Ladder2.py <> Ladder2.lua <> Ladder2.rs

-- | The language rows, every absent arm its blank.
languages :: [Language]
languages = case common of
  Object o | Just v <- KM.lookup "languages" o -> either refuse id (parseEither rowsOf v)
  _ -> refuse "no languages table"
 where
  rowsOf = withObject "languages" (.: "rows")
  refuse e = error ("definition document `common` does not read: " <> e)

-- | The GHC global package table, a row text.
boot :: [(String, [String])]
boot =
  rows $
    Boot1.boot <> Boot1.boot2 <> Boot1.boot3 <> Boot1.boot4 <> Boot1.boot5
      <> Boot2.boot <> Boot2.boot2 <> Boot2.boot3 <> Boot2.boot4
      <> Boot3.boot <> Boot3.boot2 <> Boot3.boot3 <> Boot3.boot4 <> Boot3.boot5 <> Boot3.boot6

-- | A document read into its shape; one that does not read is a defect
-- of this source, named — the battery forces every one.
decoded :: (FromJSON a) => String -> String -> a
decoded what text = either refuse id (toml text >>= parseEither parseJSON)
 where
  refuse e = error ("definition document `" <> what <> "` does not read: " <> e)

-- | One value per judged language, keyed by its report name.
byLang :: (ToJSON a) => (LangTables -> a) -> Value
byLang f = object [K.fromString (ltName t) .= f t | t <- allTables]

-- | The package: the common document with the per-language tables, the
-- typed language rows, the package table and the docdup numbers
-- grafted in at their dotted paths.
pack :: Value
pack = foldl' graft common derived
 where
  derived =
    [ ("scan", byLang ltScan)
    , ("flow", byLang ltFlow)
    , ("slot", byLang ltSlot)
    , ("sites", byLang ltSites)
    , ("calls.calls", byLang ltCalls)
    , ("calls.protected", byLang ltProtected)
    , ("fourclass.extra", byLang ltExtra)
    , ("docdup.doc_spec", byLang (\t -> object ["docstring_hosts" .= ltDocstringHosts t]))
    , ("docdup.min_doc_tokens", toJSON Doc.minDocTokens)
    , ("docdup.license_head_lines", toJSON Doc.licHeadLines)
    , ("docdup.verbatim_floor", toJSON Doc.verbatimFloor)
    , ("docdup.doc_shingle", toJSON Doc.shingleK)
    , ("docdup.doc_line_cap", toJSON Doc.docLineCap)
    , ("languages.rows", toJSON languages)
    , ("ladder.hs.boot", toJSON boot)
    ]

-- | A value set at a dotted path, the tables on the way made.
graft :: Value -> (String, Value) -> Value
graft root (path, v) = go (K.fromString <$> splitDots path) root
 where
  go [k] (Object m) = Object (KM.insert k v m)
  go (k : ks) (Object m) = Object (KM.insert k (go ks (fromMaybe (Object KM.empty) (KM.lookup k m))) m)
  go _ other = other
  splitDots s = case break (== '.') s of
    (a, _ : rest) -> a : splitDots rest
    (a, []) -> [a]

-- | The fnv1a64 of a value's canonical bytes (aeson's encoding: keys
-- in order, no blanks) — a function of the data alone.
digestOf :: Value -> Integer
digestOf v = toInteger (BL.foldl' step 14695981039346656037 (encode v))
 where
  step :: Word64 -> Word8 -> Word64
  step h b = (h `xor` fromIntegral b) * 1099511628211
