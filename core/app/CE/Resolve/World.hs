-- | The tree one resolve.request describes (plan v2.33 wave W2a; text
-- since W2-text): the walked paths as a set — the ladders' `scope.files`,
-- the only place a candidate may come from — each site file's path by
-- its index, and the directories that hold an importable Go file; and
-- the steps every ladder takes over it (cli/src/graph/ladder/paths.rs,
-- `scan::lang::Lang::from_path`).
module CE.Resolve.World (
  World (..),
  world,
  member,
  pathOf,
  inScope,
  declaredIn,
  besideOrRoot,
  ofLangs,
) where

import CE.Lang (languages)
import CE.Lang.Spec (Language (..))
import CE.Resolve.Request (ResolveReq (..))
import CE.Resolve.Str (baseName, extension, joinRel, parentDir, rooted)
import Data.Array (Array, listArray, (!))
import Data.List (isSuffixOf)
import qualified Data.Set as Set

data World = World
  { wPaths :: Array Int String
  , wFiles :: Set.Set String
  , wGoDirs :: Set.Set String
  }

-- | The world of a request: files then origins by index; a site's own
-- file the walk did not hold is readable (its directory) but never a
-- target.
world :: ResolveReq -> World
world rq =
  World
    { wPaths = listArray (0, length paths - 1) paths
    , wFiles = Set.fromList (rqFiles rq)
    , wGoDirs = Set.fromList [parentDir f | f <- rqFiles rq, goSource (baseName f)]
    }
 where
  paths = rqFiles rq <> rqOrigins rq

-- | The go ladder's `package` test, per directory: a walked file directly
-- in it named `*.go` and not `*_test.go` (a file `dir/x.go` is directly
-- in `dir` exactly when `dir` is its parent).
goSource :: String -> Bool
goSource base = ".go" `isSuffixOf` base && not ("_test.go" `isSuffixOf` base)

member :: World -> String -> Bool
member w p = Set.member p (wFiles w)

pathOf :: World -> Int -> String
pathOf w i = wPaths w ! i

-- | `c_search::in_scope`: the walked file a directory holds under a name.
inScope :: World -> String -> String -> Maybe String
inScope w dir name = case joinRel dir name of
  Just p | member w p -> Just p
  _ -> Nothing

-- | `paths::declared`: the walked files a name names under each of one
-- language's declared `[graph.search_roots]` directories.
declaredIn :: World -> [String] -> String -> Set.Set String
declaredIn w dirs name = Set.fromList [p | dir <- dirs, Just p <- [inScope w dir name]]

-- | `paths::beside_or_root`: a path as a script's working directory
-- reads it — beside the file that names it, then under the tree root;
-- the first walked one. A rooted path names no file of the tree.
besideOrRoot :: World -> String -> String -> Maybe String
besideOrRoot w from spec
  | rooted spec = Nothing
  | otherwise = case [p | dir <- [parentDir from, ""], Just p <- [inScope w dir spec]] of
      (p : _) -> Just p
      [] -> Nothing

-- | `Lang::from_path` asked of a set of languages: the path's extension
-- is one of theirs (the `languages` rows, by name).
ofLangs :: [String] -> String -> Bool
ofLangs names path = maybe False (`elem` exts) (extension path)
 where
  exts = concat [lgExts l | l <- languages, lgName l `elem` names]
