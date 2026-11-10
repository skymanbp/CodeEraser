{-# LANGUAGE OverloadedStrings #-}

-- | The files a manifest names as what a runtime or a tool starts (lane
-- v235_graphtrust, item T2): the declared-target pass gives each the
-- entry role a Cargo `[[bin]]` path or a cabal `main-is` earns, so a
-- program whose entry is a manifest field — never an import — is not
-- read all-dead for want of an entry flag. Read off the manifests this
-- family already reads for its ladders:
--
-- * each package.json the walk holds: `main`, `module`, `types`,
--   `bin` (a string or a map), and every string leaf of `exports` (a
--   string, a condition map, `./sub` keys); a target with a `*` names
--   every walked file it matches, the `*` standing for any text as
--   Node's subpath patterns do. A target is found as the TS ladder's
--   first rung finds a relative import (`firstHit`: the file, then the
--   extension order, then a directory's index); a `.js` / `.mjs` /
--   `.cjs` target no walked file answers names its TS twin when that is
--   walked;
-- * the root `pyproject.toml`: every value of `[project.scripts]`,
--   `[project.gui-scripts]` and each `[project.entry-points.*]` group —
--   `pkg.mod:obj` names the module `pkg.mod`, found under every source
--   root by the Python ladder's absolute rule (`moduleAt`), with the
--   ancestor packages CPython executes before it (`packages`).
--
-- A name no walked file answers names nothing; every root that holds one
-- is an entry (the safe direction: a wrongly live file costs a missed
-- cleanup, a wrongly dead one can be erased).
module CE.Resolve.Entries (entries) where

import CE.Resolve.Py (PyProject, moduleAt, packages, sourceRoots)
import CE.Resolve.Str (joinRel, parentDir, rustTrim, splitOnce, stripSuffix)
import CE.Resolve.Ts (firstHit, stringLeaves)
import CE.Resolve.TsConfig (field)
import CE.Resolve.TsFacts (Facts, Need, textOf)
import CE.Resolve.World (World (..), member)
import Data.Aeson (Value (..))
import qualified Data.Aeson.Key as K
import qualified Data.Aeson.KeyMap as KM
import Data.List (stripPrefix)
import qualified Data.Set as Set

-- | The entries the walk's package.json files and the root
-- pyproject.toml declare (`py` its decoded document's surface, `doc` the
-- document itself).
entries :: World -> Facts -> [String] -> (Maybe PyProject, Maybe Value) -> Need (Set.Set String)
entries w fx pkgs (py, doc) = do
  docs <- mapM (\rel -> (,) rel <$> textOf fx rel) pkgs
  pure (Set.fromList (concat [packageEntries w (parentDir rel) d | (rel, Just d) <- docs] <> pyEntries w py doc))

-- | One package.json's targets, each found under its directory.
packageEntries :: World -> String -> Value -> [String]
packageEntries w dir doc = concatMap (target w dir) (concatMap leaves ["main", "module", "types", "bin", "exports"])
 where
  leaves k = maybe [] stringLeaves (field k doc)

-- | One target string under a package directory: a pattern's every
-- walked match, else the first hit of its path, else its TS twin. A
-- pattern's text before the `*` is a directory (through its last `/`,
-- joined under the package) and a name prefix; a walked file matches
-- when it lies under that directory, the rest of its path starts with
-- the name prefix and ends with the text after the `*` (or a twin of
-- it), the two never overlapping.
target :: World -> String -> String -> [String]
target w dir spec = case splitOnce '*' spec of
  Just (pre, post) -> case joinRel dir (reverse inDir) of
    Just base -> [f | f <- Set.toList (wFiles w), Just rest <- [stripPrefix (slash base <> reverse name) f], any (\p -> stripSuffix p rest /= Nothing) (post : twinsOf post)]
    Nothing -> []
   where
    (name, inDir) = break (== '/') (reverse pre)
  Nothing -> case joinRel dir spec of
    Just base -> case firstHit w base of
      Just hit -> [hit]
      Nothing -> take 1 (filter (member w) (twinsOf base))
    Nothing -> []
 where
  slash b = if null b then "" else b <> "/"

-- | A `.js` / `.mjs` / `.cjs` path's TS twins, in the TS ladder's order.
twinsOf :: String -> [String]
twinsOf p = [stem <> t | (js, tss) <- twins, Just stem <- [stripSuffix js p], t <- tss]
 where
  twins = [(".js", [".ts", ".tsx"]), (".mjs", [".mts"]), (".cjs", [".cts"])] :: [(String, [String])]

-- | The root pyproject's entry-point modules under every source root,
-- each with its ancestor packages.
pyEntries :: World -> Maybe PyProject -> Maybe Value -> [String]
pyEntries w py doc =
  [ hit
  | m <- modules
  , root <- sourceRoots py
  , Just found <- [moduleAt w root m]
  , hit <- found : packages w root m
  ]
 where
  modules = filter (not . null) [rustTrim (takeWhile (/= ':') v) | t <- tables, String s <- KM.elems t, let v = K.toString (K.fromText s)]
  tables = [t | Just (Object t) <- [at ["project", "scripts"], at ["project", "gui-scripts"]]] <> groups
  groups = case at ["project", "entry-points"] of
    Just (Object gs) -> [t | Object t <- KM.elems gs]
    _ -> []
  at ks = doc >>= \d -> foldl (\cur k -> cur >>= field k) (Just d) ks
