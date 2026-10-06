{-# LANGUAGE OverloadedStrings #-}

-- | The tsconfig chain and the package.json surface the TS rungs read
-- (moved from cli/src/graph/roots_ts.rs and the package.json half of
-- roots.rs in plan v2.33 W2-text stage E — each function below is the
-- Rust function of the same name): the nearest tsconfig from a
-- directory and every config its `extends` reaches, folded to the
-- effective options the third rung reads, and the chain's file list the
-- measuring side's resolve key hashes; one package.json's name, exports
-- and declared dependencies. Documents come from the facts
-- (CE.Resolve.TsFacts), read as JSONC (CE.Resolve.Json).
module CE.Resolve.TsConfig (
  TsOptions (..),
  TsChain (..),
  tsOptions,
  tsExtendsFiles,
  Package (..),
  package,
  nearestPackage,
  field,
  str,
  keysUnder,
) where

import CE.Resolve.Str (joinRel, parentDir)
import CE.Resolve.TsFacts
import Data.Aeson (Value (..))
import qualified Data.Aeson.Key as K
import qualified Data.Aeson.KeyMap as KM
import Data.List (isPrefixOf, isSuffixOf)
import Data.Maybe (isNothing, mapMaybe)
import Data.Foldable (toList)

-- | Effective TS options after the extends walk: each field from the
-- config nearest the importer that defines it (TS semantics: `paths`
-- replaces, never merges), with its anchor — baseUrl resolves against
-- the config that declared it, paths without baseUrl against the config
-- that declared them.
data TsOptions = TsOptions
  { toBaseDir :: Maybe String
  , toPaths :: [(String, [String])]
  , toPathsAnchor :: String
  }
  deriving (Eq, Show)

-- | No config at all, usable options, or a chain the ladder refuses (a
-- cycle, an unreadable or out-of-tree target, a non-string entry) —
-- config_depth, never a guess.
data TsChain = TsNone | TsOk TsOptions | TsBroken
  deriving (Eq, Show)

-- | `Value::get(key)`: an object's field.
field :: String -> Value -> Maybe Value
field k (Object o) = KM.lookup (K.fromString k) o
field _ _ = Nothing

-- | `Value::as_str`.
str :: Maybe Value -> Maybe String
str (Just (String t)) = Just (K.toString (K.fromText t))
str _ = Nothing

-- | The keys of the object under each of the given fields, in field
-- order (a field that holds no object adds none): a manifest's declared
-- dependency names, package.json's and Cargo.toml's alike.
keysUnder :: [String] -> Value -> [String]
keysUnder ks doc = concat [map K.toString (KM.keys m) | k <- ks, Just (Object m) <- [field k doc]]

-- | `ts_options`: the nearest tsconfig walking up from `fromDir`, then
-- every config its extends chain reaches — an `extends` array read last
-- entry first (TS 5.0: a later entry overrides an earlier one), each
-- entry's own chain walked whole before the entry before it, no depth
-- cap, a cycle refused. A bare-package target (node_modules) ends its
-- branch and keeps the fields already collected.
tsOptions :: Facts -> String -> Need TsChain
tsOptions fx fromDir = do
  start <- nearestUp fx fromDir "tsconfig.json"
  case start of
    Nothing -> pure TsNone
    Just s -> do
      (visited, ok) <- extendsWalk fx [] s
      pure $
        if ok
          then TsOk (foldl (\o (path, doc) -> collectTs doc (parentDir path) o) (TsOptions Nothing [] "") visited)
          else TsBroken

-- | `extends_walk`: the configs visited from `current` (each with its
-- document, nearest first) and whether the walk ended without a
-- refusal; `branch` is the path stack of the current branch — a config
-- on it again is a cycle, while a diamond (one base reached by two
-- branches) is not.
extendsWalk :: Facts -> [String] -> String -> Need ([(String, Value)], Bool)
extendsWalk fx branch current
  | current `elem` branch = pure ([], False)
  | otherwise = do
      doc <- textOf fx current
      case doc of
        Nothing -> pure ([], False)
        Just d -> do
          (rest, ok) <- targets (current : branch) (extendsOf d)
          pure ((current, d) : rest, ok)
 where
  dir = parentDir current
  extendsOf d = case field "extends" d of
    Nothing -> []
    Just (Array items) -> reverse (toList items)
    Just one -> [one]
  targets _ [] = pure ([], True)
  targets br (t : ts) = case str (Just t) of
    Just target
      | not ("." `isPrefixOf` target) -> targets br ts
      | otherwise -> case joinRel dir (target <> jsonExt target) of
          Nothing -> pure ([], False)
          Just next -> do
            (seen, ok) <- extendsWalk fx br next
            if ok
              then (\(more, ok') -> (seen <> more, ok')) <$> targets br ts
              else pure (seen, False)
    _ -> pure ([], False)

-- | `ts_extends_files`: every config the extends chain from `start`
-- reaches, `start` first — a broken chain lists what was reached before
-- the break.
tsExtendsFiles :: Facts -> String -> Need [String]
tsExtendsFiles fx start = map fst . fst <$> extendsWalk fx [] start

-- | `collect_ts`: baseUrl and paths from one config, a field the
-- options already hold kept.
collectTs :: Value -> String -> TsOptions -> TsOptions
collectTs doc dir opts = case field "compilerOptions" doc of
  Nothing -> opts
  Just co -> anchored (withPaths co (withBase co opts))
 where
  withBase co o
    | isNothing (toBaseDir o)
    , Just base <- str (field "baseUrl" co)
    , Just joined <- joinRel dir base =
        o {toBaseDir = Just joined}
    | otherwise = o
  withPaths co o
    | null (toPaths o)
    , Just (Object m) <- field "paths" co =
        o {toPaths = [(K.toString k, targetsOf v) | (k, v) <- KM.toList m], toPathsAnchor = dir}
    | otherwise = o
  targetsOf v = case v of
    Array xs -> mapMaybe (str . Just) (toList xs)
    _ -> []
  anchored o = case toBaseDir o of
    Just base | not (null (toPaths o)) -> o {toPathsAnchor = base}
    _ -> o

-- | `json_ext`: an extends target without `.json` implies it.
jsonExt :: String -> String
jsonExt target = if ".json" `isSuffixOf` target then "" else ".json"

-- | One package.json surface, enough for the fourth and fifth rungs:
-- its directory ("" = the tree root), name, exports and the union of
-- the dependency keys of its four dependency maps.
data Package = Package
  { pkDir :: String
  , pkName :: Maybe String
  , pkExports :: Maybe Value
  , pkDeps :: [String]
  }

-- | `package`: one package.json, none when it does not read.
package :: Facts -> String -> Need (Maybe Package)
package fx rel = fmap surface <$> textOf fx rel
 where
  surface doc =
    Package
      { pkDir = parentDir rel
      , pkName = str (field "name" doc)
      , pkExports = field "exports" doc
      , pkDeps = keysUnder ["dependencies", "devDependencies", "peerDependencies", "optionalDependencies"] doc
      }

-- | `nearest_package`: the nearest package.json walking up from
-- `fromDir`, read.
nearestPackage :: Facts -> String -> Need (Maybe Package)
nearestPackage fx fromDir = nearestUp fx fromDir "package.json" >>= maybe (pure Nothing) (package fx)
