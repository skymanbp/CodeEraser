{-# LANGUAGE OverloadedStrings #-}

-- | What the resolve family reads of the definition package (plan v2.33
-- W2-text; replaces the W2a vocabulary): the External tables
-- (`ladder.py.stdlib`, `ladder.lua.stdlib`, `ladder.go.std`, and since
-- stage C the JDK's `ladder.java` packages and `java.lang` types, since
-- stage D the global package database's `ladder.hs.boot`), the
-- two compile-flag spelling lists (`compdb.gnu`, `compdb.skip`), and the
-- storage codes of the site kinds the Lua, R and Java ladders branch on.
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
  gnuFlags,
  skipFlags,
  kindRequire,
  kindLoad,
  kindSource,
  kindLibrary,
  kindImport,
  kindImportStar,
  kindTypeRef,
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
pyStdlib = Set.fromList (packList ["ladder", "py", "stdlib"])
luaStdlib = Set.fromList (packList ["ladder", "lua", "stdlib"])
goStd = Set.fromList (packList ["ladder", "go", "std"])

-- | The packages the JDK's runtime image exports and `java.lang`'s
-- public top-level types.
javaPackages, javaLang :: Set.Set String
javaPackages = Set.fromList (packList ["ladder", "java", "packages"])
javaLang = Set.fromList (packList ["ladder", "java", "lang"])

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

-- | The storage codes of the two Lua site kinds, the two R ones and the
-- three Java ones (`store.site_kinds`).
kindRequire, kindLoad, kindSource, kindLibrary, kindImport, kindImportStar, kindTypeRef :: Integer
kindRequire = kindCode "require"
kindLoad = kindCode "load"
kindSource = kindCode "source"
kindLibrary = kindCode "library"
kindImport = kindCode "import"
kindImportStar = kindCode "import_star"
kindTypeRef = kindCode "type_ref"

kindCode :: String -> Integer
kindCode k = maybe (error ("no site kind " <> k)) toInteger (elemIndex k siteKinds)

-- | The walk's config basenames whose text a request carries (`go.mod`
-- under `go.mods`, `DESCRIPTION` under `r.descriptions`, every `*.cabal`
-- under `hs.cabals` — a name opening with `*` is a basename suffix; the
-- root `pyproject.toml`, the compile databases and their response files
-- travel by the measuring side's own finders).
configNames :: [String]
configNames = ["go.mod", "DESCRIPTION", "*.cabal"]

-- | The `resolve` key of the package.
table :: Value
table = Object (KM.fromList [("configs", toJSON configNames)])
