{-# LANGUAGE OverloadedStrings #-}

-- | The Cargo surface the Rust rungs and the declared-target and mounts
-- passes read (moved from cli/src/graph/cargo.rs and the bin-root facts
-- of cli/src/graph/mounts.rs in plan v2.33 W2-text stage F — each
-- function below is the Rust function of the same name): one
-- Cargo.toml's package name, lib and bin target paths and declared
-- dependency names, its crate roots among the walked files, its bin
-- roots and its lib root. The document is the measuring side's decoded
-- TOML (a TOML table as an object, a TOML string as a string): this
-- family reads the keys.
module CE.Resolve.Cargo (
  Package (..),
  package,
  crateRoots,
  binRoots,
  libRoot,
  keeps,
) where

import CE.Resolve.Str (joinDir, joinRel, parentDir, splitOn)
import CE.Resolve.TsConfig (field, keysUnder, str)
import Data.Aeson (Value (..))
import Data.Foldable (toList)
import Data.List (isSuffixOf, stripPrefix)
import Data.Maybe (isJust, mapMaybe, maybeToList)
import qualified Data.Set as Set

-- | One Cargo.toml surface: its directory ("" = the tree root), the
-- `[package]` name (hyphens allowed; the rungs normalize), the `[lib]`
-- path, the `[[bin]]` paths and the union of the dependencies,
-- dev-dependencies and build-dependencies keys.
data Package = Package
  { cpDir :: String
  , cpName :: Maybe String
  , cpLibPath :: Maybe String
  , cpBinPaths :: [String]
  , cpDeps :: [String]
  }

-- | `package`: one decoded Cargo.toml at a repo-relative path.
package :: String -> Value -> Package
package rel doc =
  Package
    { cpDir = parentDir rel
    , cpName = str (tableAt ["package", "name"])
    , cpLibPath = str (tableAt ["lib", "path"])
    , cpBinPaths = [p | Just (Array bins) <- [field "bin" doc], b <- toList bins, Just p <- [str (field "path" b)]]
    , cpDeps = keysUnder ["dependencies", "dev-dependencies", "build-dependencies"] doc
    }
 where
  -- `roots::table_at`: one dotted key path into the document
  tableAt keys = foldl (\cur k -> cur >>= field k) (Just doc) keys

-- | `crate_roots`: the declared lib and bin paths, the default targets,
-- and both auto-discovery forms under the conventional target
-- directories — those that are walked files.
crateRoots :: Set.Set String -> Package -> Set.Set String
crateRoots files p = Set.unions (declared : map (autoTargets files p) ["src/bin", "tests", "examples", "benches"])
 where
  targets = maybeToList (cpLibPath p) <> cpBinPaths p <> ["src/lib.rs", "src/main.rs", "build.rs"]
  declared = Set.fromList [c | t <- targets, Just c <- [joinRel (cpDir p) t], Set.member c files]

-- | `bin_roots`: the declared bin paths, the default main and the
-- auto-discovered src/bin targets that are walked files.
binRoots :: Set.Set String -> Package -> Set.Set String
binRoots files p = Set.union declared (autoTargets files p "src/bin")
 where
  declared = Set.fromList (filter (`Set.member` files) (mapMaybe (joinRel (cpDir p)) (cpBinPaths p <> ["src/main.rs"])))

-- | `auto_targets`: the walked files directly under one conventional
-- directory named `*.rs`, or a `<name>/main.rs` one level down.
autoTargets :: Set.Set String -> Package -> String -> Set.Set String
autoTargets files p sub = Set.filter auto files
 where
  prefix = joinDir (cpDir p) sub <> "/"
  auto f = case splitOn '/' <$> stripPrefix prefix f of
    Just [leaf] -> ".rs" `isSuffixOf` leaf
    Just [_, "main.rs"] -> True
    _ -> False

-- | `lib_root`: the lib crate root, when it is a walked file.
libRoot :: Set.Set String -> Package -> Maybe String
libRoot files p = case joinRel (cpDir p) (maybe "src/lib.rs" id (cpLibPath p)) of
  Just c | Set.member c files -> Just c
  _ -> Nothing

-- | `RustTargets::keeps`: whether a package keeps a file private — a
-- package (it names itself) without a lib target keeps every file, one
-- with a lib target its bin roots alone; a manifest that does not read,
-- or names no package (a virtual workspace), keeps nothing.
keeps :: Set.Set String -> Maybe Package -> String -> Bool
keeps files pkg path = case pkg of
  Just p -> isJust (cpName p) && (not (isJust (libRoot files p)) || Set.member path (binRoots files p))
  Nothing -> False
