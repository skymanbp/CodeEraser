{-# LANGUAGE OverloadedStrings #-}

-- | What the resolve family reads of the definition package (plan v2.33
-- W2-text; replaces the W2a vocabulary): the External tables
-- (`ladder.py.stdlib`, `ladder.lua.stdlib`, `ladder.go.std`, and since
-- stage C the JDK's `ladder.java` packages and `java.lang` types, since
-- stage D the global package database's `ladder.hs.boot`, since stage E
-- Node's builtin modules, `ladder.ts`, since stage F the toolchain's
-- crates, `ladder.rs`), the
-- two compile-flag spelling lists (`compdb.gnu`, `compdb.skip`), and the
-- storage codes of the site kinds the Lua, R, Java, Rust and Markdown
-- ladders branch on.
-- The package's `resolve` key states which configuration files the
-- measuring side sends as text — the basenames among the walk's
-- configs (`configs`): a rule of this family, so not a second list
-- there.
module CE.Resolve.Tables (
  pyStdlib,
  luaStdlib,
  goStd,
  javaPackages,
  javaLang,
  hsBoot,
  nodeBuiltins,
  nodePrefixOnly,
  rsBuiltin,
  gnuFlags,
  skipFlags,
  kindRequire,
  kindLoad,
  kindSource,
  kindLibrary,
  kindImport,
  kindImportStar,
  kindTypeRef,
  kindModDecl,
  kindUse,
  kindLink,
  kindImage,
  kindRefLink,
  kindRefDef,
  kindUrl,
  kindHref,
  kindAction,
  configNames,
  table,
) where

import CE.Lang (pack, siteKinds)
import Data.Aeson (FromJSON, Value (..), toJSON)
import qualified Data.Aeson.Key as K
import qualified Data.Aeson.KeyMap as KM
import Data.Aeson.Types (parseEither, parseJSON)
import Data.List (elemIndex)
import qualified Data.Set as Set

pyStdlib, luaStdlib, goStd :: Set.Set String
pyStdlib = ladderSet "py" "stdlib"
luaStdlib = ladderSet "lua" "stdlib"
goStd = ladderSet "go" "std"

-- | One name set of a language's ladder in the package.
ladderSet :: String -> String -> Set.Set String
ladderSet lang key = Set.fromList (packList ["ladder", lang, key])

-- | Node's builtin modules: the names each importable bare and under
-- `node:`, and the ones only `node:` reaches.
nodeBuiltins, nodePrefixOnly :: Set.Set String
nodeBuiltins = ladderSet "ts" "builtins"
nodePrefixOnly = ladderSet "ts" "prefix_only"

-- | The crates the Rust toolchain provides without a declaration.
rsBuiltin :: Set.Set String
rsBuiltin = ladderSet "rs" "builtin"

-- | The packages the JDK's runtime image exports and `java.lang`'s
-- public top-level types.
javaPackages, javaLang :: Set.Set String
javaPackages = ladderSet "java" "packages"
javaLang = ladderSet "java" "lang"

-- | The global package database's packages, each with its modules.
hsBoot :: [(String, [String])]
hsBoot = packAt ["ladder", "hs", "boot"]

-- | The GNU joined spellings (longest conflicting spelling first) and
-- the separate operands that cannot open an include option, in the
-- package's order.
gnuFlags, skipFlags :: [String]
gnuFlags = packList ["compdb", "gnu"]
skipFlags = packList ["compdb", "skip"]

-- | One string list of the package by its key path.
packList :: [String] -> [String]
packList = packAt

-- | One value of the package by its key path.
packAt :: (FromJSON a) => [String] -> a
packAt path = either refuse id (parseEither parseJSON =<< walk path pack)
 where
  walk [] v = Right v
  walk (k : ks) (Object o) | Just v <- KM.lookup (K.fromString k) o = walk ks v
  walk _ _ = Left ("no " <> show path)
  refuse e = error ("resolve tables do not read: " <> e)

-- | The storage codes of the two Lua site kinds, the two R ones, the
-- three Java ones, the two Rust ones, the five Markdown ones and the two
-- HTML ones whose empty specifier lands the page (`store.site_kinds`).
kindRequire, kindLoad, kindSource, kindLibrary, kindImport, kindImportStar, kindTypeRef, kindModDecl, kindUse, kindLink, kindImage, kindRefLink, kindRefDef, kindUrl, kindHref, kindAction :: Integer
kindRequire = kindCode "require"
kindLoad = kindCode "load"
kindSource = kindCode "source"
kindLibrary = kindCode "library"
kindImport = kindCode "import"
kindImportStar = kindCode "import_star"
kindTypeRef = kindCode "type_ref"
kindModDecl = kindCode "mod_decl"
kindUse = kindCode "use"
kindLink = kindCode "link"
kindImage = kindCode "image"
kindRefLink = kindCode "ref_link"
kindRefDef = kindCode "ref_def"
kindUrl = kindCode "url"
kindHref = kindCode "href"
kindAction = kindCode "action"

kindCode :: String -> Integer
kindCode k = maybe (error ("no site kind " <> k)) toInteger (elemIndex k siteKinds)

-- | The walk's config basenames whose text a request carries (`go.mod`
-- under `go.mods`, `DESCRIPTION` under `r.descriptions`, every `*.cabal`
-- under `hs.cabals`, `package.json` and `tsconfig.json` as `ts.facts`,
-- `Cargo.toml` as `ts.facts` decoded — a name opening with `*` is a
-- basename suffix; the
-- root `pyproject.toml`, the compile databases and their response files
-- travel by the measuring side's own finders).
configNames :: [String]
configNames = ["go.mod", "DESCRIPTION", "*.cabal", "package.json", "tsconfig.json", "Cargo.toml"]

-- | The `resolve` key of the package.
table :: Value
table = Object (KM.fromList [("configs", toJSON configNames)])
