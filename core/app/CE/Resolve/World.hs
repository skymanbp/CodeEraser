-- | The tree one resolve.request describes (plan v2.33 wave W2a; text
-- since W2-text): the walked paths as a set — the ladders' `scope.files`,
-- the only place a candidate may come from — each site file's path by
-- its index, and the directories that hold an importable Go file.
module CE.Resolve.World (
  World (..),
  world,
  member,
  pathOf,
  inScope,
) where

import CE.Resolve.Request (ResolveReq (..))
import CE.Resolve.Str (baseName, joinRel, parentDir)
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
