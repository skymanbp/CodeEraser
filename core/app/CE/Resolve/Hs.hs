-- | The Haskell rungs (moved from cli/src/graph/ladder/hs.rs in plan
-- v2.33 W2-text stage D — each function below is the Rust function of
-- the same name). A module name is dots-to-slashes under a source root;
-- the roots are cabal facts (CE.Resolve.Cabal).
--   R1 source-root walk: the importer's owning cabal (deepest directory
--      prefix — two at one depth is ambiguous_workspace), the stanzas
--      whose roots hold the importer (none holding it ⇒ every stanza),
--      one candidate per root; distinct files from different roots is
--      ambiguous_root, never a pick. No cabal owning the importer
--      anchors at the tree root — bare `ghc Main.hs` search semantics.
--   R2 a module another in-corpus package exposes — the package the
--      import names under PackageImports (`import "pkg" M`: the spec
--      keeps the quoted package), else any package the owner's
--      build-depends declares — under that package's library roots;
--      two such packages refuse as ambiguous_workspace.
--   R3 External: the global package database table (`ladder.hs.boot`),
--      gated by the owner cabal's build-depends — with no cabal every
--      db package is visible.
-- A store-installed dependency outside the global db lands out_of_scope
-- (module → package needs evidence). An `import {-# SOURCE #-} M`
-- resolves to `M.hs` like any other import.
module CE.Resolve.Hs (resolveHs) where

import CE.Resolve.Answer
import CE.Resolve.Cabal
import CE.Resolve.Cost (Reason (..))
import CE.Resolve.Str (joinDir, replaceChar, rustTrim, splitOn, splitOnce)
import CE.Resolve.Tables (hsBoot)
import CE.Resolve.World (World, member)
import Data.Char (isAsciiUpper)
import Data.List (isPrefixOf)
import qualified Data.Map.Strict as M
import Data.Maybe (fromMaybe)
import qualified Data.Set as Set

-- | `resolve`: one site, the cabals the request carried (the walk's
-- configs ending `.cabal`, each read).
resolveHs :: World -> [Cabal] -> String -> String -> Answer
resolveHs w cabals from spec
  | not (moduleShaped modl) = AUnresolved OutOfScope
  | otherwise = case owner cabals from of
      Left why -> AUnresolved why
      Right o -> firstOf [sourceRung o, dependedRung w cabals modl package o rel] (externalRung modl package o)
 where
  (package, modl) = splitPackage spec
  rel = replaceChar '.' '/' modl <> ".hs"
  -- R1 asks the importer's own roots, unless the import names another
  -- package by name
  sourceRung o
    | maybe True (\p -> maybe False ((== p) . cName) o) package = case Set.toList (hits o) of
        [p] -> Just (AFile p 1)
        [] -> Nothing
        _ -> Just (AUnresolved AmbiguousRoot)
    | otherwise = Nothing
  hits o = Set.fromList [p | root <- sourceRoots o from, let p = joinDir root rel, member w p]

-- | `module_shaped`: every dot-separated segment opens with an uppercase
-- letter; anything else is not a module name.
moduleShaped :: String -> Bool
moduleShaped spec = not (null spec) && all opens (splitOn '.' spec)
 where
  opens seg = case seg of
    (c : _) -> isAsciiUpper c
    [] -> False

-- | `split_package`: `"pkg" M` is the module `M` from the package `pkg`;
-- a quote never closed leaves the spec whole.
splitPackage :: String -> (Maybe String, String)
splitPackage spec = case spec of
  ('"' : rest) | Just (p, m) <- splitOnce '"' rest -> (Just p, rustTrim m)
  _ -> (Nothing, spec)

-- | `owner`: the cabal whose directory is the deepest prefix of `from`;
-- none is a legitimate anchor, two at one depth refuse. (The holders'
-- directories are all prefixes of `from`, so equal lengths are equal
-- directories.)
owner :: [Cabal] -> String -> Either Reason (Maybe Cabal)
owner cabals from = case [c | c <- holders, length (cDir c) == deepest] of
  [] -> Right Nothing
  [c] -> Right (Just c)
  _ -> Left AmbiguousWorkspace
 where
  holders = [c | c <- cabals, null (cDir c) || (cDir c <> "/") `isPrefixOf` from]
  deepest = maximum (0 : map (length . cDir) holders)

-- | `source_roots`: the owning stanzas' roots (falling back to every
-- stanza), or the tree root without a cabal; once each, in order.
sourceRoots :: Maybe Cabal -> String -> [String]
sourceRoots Nothing _ = [""]
sourceRoots (Just c) from = Set.toList (Set.fromList (concatMap stRoots picked))
 where
  owning = filter (any holds . stRoots) (cStanzas c)
  picked = if null owning then cStanzas c else owning
  holds r = null r || (r <> "/") `isPrefixOf` from

-- | `depended_rung`: the module under the library roots of another
-- in-corpus package that exposes it — the package the import names,
-- else any package the owner's build-depends declares. One package
-- answers (its one file, two roots ambiguous_root, none out of scope —
-- an exposed module is never External); two refuse; none says nothing.
dependedRung :: World -> [Cabal] -> String -> Maybe String -> Maybe Cabal -> String -> Maybe Answer
dependedRung w cabals modl package o rel = case packages of
  [] -> Nothing
  [one] -> Just (fromMaybe (AUnresolved OutOfScope) (oneOf (found one) 2))
  _ -> Just (AUnresolved AmbiguousWorkspace)
 where
  packages = [c | c <- cabals, not (null (cName c)), maybe True ((/= cDir c) . cDir) o, wanted c, exposes c modl]
  wanted c = case package of
    Just p -> cName c == p
    Nothing -> maybe False ((cName c `elem`) . cDeps) o
  found c = Set.fromList [p | root <- libraryRoots c, let p = joinDir root rel, member w p]

-- | `external_rung`: the global-db table, build-depends-gated under a
-- cabal; a package the import names must be the module's own.
externalRung :: String -> Maybe String -> Maybe Cabal -> Answer
externalRung modl package o
  | any visible (M.findWithDefault [] modl bootPackages) = AExternal 3
  | otherwise = AUnresolved OutOfScope
 where
  visible p = maybe True (== p) package && maybe True ((p `elem`) . cDeps) o

-- | The table read once: each module, the db packages that hold it.
bootPackages :: M.Map String [String]
bootPackages = M.fromListWith (<>) [(m, [p]) | (p, ms) <- hsBoot, m <- ms]
