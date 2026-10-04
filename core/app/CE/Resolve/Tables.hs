{-# LANGUAGE OverloadedStrings #-}

-- | What the resolve family reads of the definition package (plan v2.33
-- W2-text; replaces the W2a vocabulary): the three External tables
-- (`ladder.py.stdlib`, `ladder.lua.stdlib`, `ladder.go.std`), the two
-- compile-flag spelling lists (`compdb.gnu`, `compdb.skip`), and the
-- storage codes of the two Lua site kinds the Lua ladder branches on.
-- The package's `resolve` key states which configuration files the
-- measuring side sends as text — the basenames among the walk's
-- configs (`configs`): a rule of this family, so not a second list
-- there.
module CE.Resolve.Tables (
  pyStdlib,
  luaStdlib,
  goStd,
  gnuFlags,
  skipFlags,
  kindRequire,
  kindLoad,
  configNames,
  table,
) where

import CE.Lang (pack, siteKinds)
import Data.Aeson (Value (..), toJSON)
import qualified Data.Aeson.Key as K
import qualified Data.Aeson.KeyMap as KM
import Data.Aeson.Types (parseEither, parseJSON)
import Data.List (elemIndex)
import qualified Data.Set as Set

pyStdlib, luaStdlib, goStd :: Set.Set String
pyStdlib = Set.fromList (packList ["ladder", "py", "stdlib"])
luaStdlib = Set.fromList (packList ["ladder", "lua", "stdlib"])
goStd = Set.fromList (packList ["ladder", "go", "std"])

-- | The GNU joined spellings (longest conflicting spelling first) and
-- the separate operands that cannot open an include option, in the
-- package's order.
gnuFlags, skipFlags :: [String]
gnuFlags = packList ["compdb", "gnu"]
skipFlags = packList ["compdb", "skip"]

-- | One string list of the package by its key path.
packList :: [String] -> [String]
packList path = either refuse id (parseEither parseJSON =<< walk path pack)
 where
  walk [] v = Right v
  walk (k : ks) (Object o) | Just v <- KM.lookup (K.fromString k) o = walk ks v
  walk _ _ = Left ("no " <> show path)
  refuse e = error ("resolve tables do not read: " <> e)

-- | The storage codes of the two Lua site kinds (`store.site_kinds`).
kindRequire, kindLoad :: Integer
kindRequire = kindCode "require"
kindLoad = kindCode "load"

kindCode :: String -> Integer
kindCode k = maybe (error ("no site kind " <> k)) toInteger (elemIndex k siteKinds)

-- | The walk's config basenames whose text a request carries (`go.mod`;
-- the root `pyproject.toml`, the compile databases and their response
-- files travel by the measuring side's own finders).
configNames :: [String]
configNames = ["go.mod"]

-- | The `resolve` key of the package.
table :: Value
table = Object (KM.fromList [("configs", toJSON configNames)])
