-- | A package's .cabal file (moved from cli/src/graph/cabal.rs and
-- cabal_parse.rs in plan v2.33 W2-text stage D — each function below is
-- the Rust function of the same name), read for the facts the Haskell
-- rungs, the declared-target pass and the mounts table consume:
-- per-stanza `hs-source-dirs` (the R1 root set), the union of
-- `build-depends` package names (the external gate), the package `name`
-- and which stanza is the public library (the R2 cross-package rung
-- reads a depended package's exposed modules under its library roots),
-- each stanza's `main-is` (a declared executable or test main), and the
-- two package-privacy facts the sealed criterion §4 reads — whether a
-- `library` stanza exists at all, and which modules are listed only
-- under other-modules. A `common` stanza's roots and module lists reach
-- a component through its `import:`. Stated boundaries, degrading to
-- refusals never guesses: cabal.project is not consulted (owner
-- anchoring is directory-prefix), and conditional blocks (`if os(..)`)
-- contribute their fields unconditionally — tag evaluation needs a
-- build configuration there is none of. The measuring side reads the
-- file as UTF-8 (a file that is not is no cabal) and sends the text;
-- the layout walk is CE.Resolve.CabalWalk.
module CE.Resolve.Cabal (
  Cabal (..),
  Stanza (..),
  parse,
  exposes,
  libraryRoots,
  keepsPrivate,
  mainTargets,
) where

import CE.Resolve.CabalWalk (Cabal (..), Stanza (..), walk)
import CE.Resolve.Str (joinRel, parentDir, replaceChar, rustLines, stripSuffix)
import CE.Resolve.World (World, member)
import Data.List (stripPrefix)
import Data.Maybe (mapMaybe)
import qualified Data.Set as Set

-- | `parse`: one cabal's text, the file at `rel`.
parse :: String -> String -> Cabal
parse rel text = walk (parentDir rel) (rustLines text)

-- | `exposes`: whether the package's public library exposes `module` —
-- a library stanza exists and the module is under exposed-modules, what
-- another package may import from it.
exposes :: Cabal -> String -> Bool
exposes c m = cHasLibrary c && Set.member m (cExposed c)

-- | `library_roots`: the public library's source roots, where its
-- exposed modules live.
libraryRoots :: Cabal -> [String]
libraryRoots c = concat [stRoots s | s <- cStanzas c, stLibrary s]

-- | `keeps_private`: whether the package keeps `path` private (the
-- mounts table's bit 1) — no library stanza at all (then every file of
-- the package), or, with a library, the module the file spells under ANY
-- stanza root is a hidden one. Every root is asked: a stanza with no
-- hs-source-dirs roots at the package directory, an ancestor of every
-- other root, and a shallower root can only spell a name with a
-- lowercase segment, never a module name.
keepsPrivate :: Cabal -> String -> Bool
keepsPrivate c path =
  not (cHasLibrary c)
    || any (`Set.member` cHidden c) (mapMaybe (`moduleUnder` path) (concatMap stRoots (cStanzas c)))

-- | `module_under`: the module name `path` spells under one source root
-- — the path below the root, `/` → `.`, `.hs` dropped; Nothing when the
-- file is not below that root.
moduleUnder :: String -> String -> Maybe String
moduleUnder root path = do
  below <- if null root then Just path else stripPrefix (root <> "/") path
  replaceChar '/' '.' <$> stripSuffix ".hs" below

-- | `main_targets` (deadcode/targets.rs): each stanza's main-is joined
-- onto each of its source roots, kept only where the file is walked — a
-- main-is naming a missing file declares nothing.
mainTargets :: World -> Cabal -> Set.Set String
mainTargets w c =
  Set.fromList
    [ cand
    | s <- cStanzas c
    , Just m <- [stMain s]
    , r <- stRoots s
    , Just cand <- [joinRel r m]
    , member w cand
    ]
