-- | What every Java rung shares (plan v2.33 W2-text stage C; moved from
-- cli/src/graph/ladder/java_pick.rs and java_sets.rs, each function
-- named after the Rust one it replaces): the request's view of the
-- tree — the walked headers and the package index over them — the
-- settling rules (one hit resolves, the declared `[graph.search_roots]
-- java` directories pick among several, a file's own name draws no
-- edge), the JDK tables' readings (the definition package's
-- `ladder.java`), and the source-set rule: the Maven and Gradle
-- standard layout keeps a module's code in `<module>/src/<set>/java/`,
-- and a `main` set sees `main` code only.
module CE.Resolve.JavaPick (
  JEnv (..),
  javaEnv,
  ownUnit,
  settle,
  declaredOne,
  starJdk,
  jdk,
  sourceRoot,
  visibleIn,
  classFiles,
  ownRootDir,
) where

import CE.Resolve.Answer
import CE.Resolve.Cost (Reason (..))
import CE.Resolve.JavaHeader
import CE.Resolve.Tables (javaPackages)
import CE.Resolve.World (World, member)
import CE.Resolve.Str (baseName, splitOn)
import Data.Containers.ListUtils (nubOrd)
import Data.List (intercalate, isPrefixOf, isSuffixOf, tails)
import qualified Data.Map.Strict as M
import qualified Data.Set as Set

-- | The headers by path, every walked header's file by the package it
-- declares (`index_of`: the files in path order) and by (package, class
-- it declares) — a file declares its top-level types, or, when the
-- reader found none, the class its name spells (`C.java`) — each file
-- with whether it sits in a standard-layout set other than `main`; both
-- indexes and that bit are read once here, not per site. Then the
-- declared roots.
data JEnv = JEnv
  { jHeaders :: M.Map String JHeader
  , jIndex :: M.Map String [(String, Bool)]
  , jClasses :: M.Map (String, String) [(String, Bool)]
  , jRoots :: [String]
  }

javaEnv :: World -> [(String, JHeader)] -> [String] -> JEnv
javaEnv w rows roots = JEnv headers (by (\(_, h) -> [hPackage h])) (by classes) roots
 where
  headers = M.fromList rows
  walked = [(f, h) | (f, h) <- M.toList headers, member w f]
  by keys = M.fromListWith (flip (<>)) [(k, [(f, notMain f)]) | fh@(f, _) <- walked, k <- keys fh]
  classes (f, h) = case hTypes h of
    [] -> [(hPackage h, take (length b - 5) b) | let b = baseName f, ".java" `isSuffixOf` b]
    types -> [(hPackage h, c) | c <- nubOrd (map tName types)]
  notMain f = maybe False ((/= "main") . snd) (sourceRoot f)

-- | `index_of` filtered by `visible`: the package's files `from` can see,
-- in path order — a file in a `main` set sees no file of another
-- standard-layout set; everything else sees everything.
visibleIn :: JEnv -> String -> String -> [String]
visibleIn env from package = seen from (M.findWithDefault [] package (jIndex env))

-- | `class_files`: the package's files `from` can see that declare
-- `class`, in path order.
classFiles :: JEnv -> String -> String -> String -> [String]
classFiles env from package cls = seen from (M.findWithDefault [] (package, cls) (jClasses env))

seen :: String -> [(String, Bool)] -> [String]
seen from files = [f | (f, notMain) <- files, not (inMain && notMain)]
 where
  inMain = maybe False ((== "main") . snd) (sourceRoot from)

-- | `own_unit`: a name the referencing file declares itself reaches no
-- other file (JLS 7.3).
ownUnit :: String -> Answer -> Answer
ownUnit from found = case found of
  AFile path _ | path == from -> AUnresolved OwnUnit
  other -> other

-- | `settle`: one hit resolves; several resolve only when the declared
-- roots hold exactly one of them, else `many` refuses; none says
-- nothing.
settle :: JEnv -> [String] -> Int -> Reason -> Maybe Answer
settle _ [] _ _ = Nothing
settle env hits rung many = Just (either (const (AUnresolved many)) (`AFile` rung) (declaredOne env (Set.fromList hits)))

-- | `declared_one`: the one member of the set, or the one under the
-- declared `java` roots (a root `""` holds everything).
declaredOne :: JEnv -> Set.Set String -> Either () String
declaredOne env set
  | Set.size set == 1 = Right (Set.findMin set)
  | otherwise = case filter under (Set.toList set) of
      [one] -> Right one
      _ -> Left ()
 where
  under p = any (\d -> null d || p == d || (d <> "/") `isPrefixOf` p) (jRoots env)

-- | `star_jdk`: the file star-imports at least one package the tree does
-- not hold, and every such package is the JDK's.
starJdk :: JEnv -> JHeader -> Bool
starJdk env h = not (null out) && all exported out
 where
  out = [iName i | i <- hImports h, iStar i, not (iStatic i), not (M.member (iName i) (jIndex env))]

-- | `jdk`: External when some prefix of the dotted name is a package the
-- JDK exports.
jdk :: [String] -> Answer
jdk segs
  | any (\k -> exported (intercalate "." (take k segs))) [1 .. length segs] = AExternal 4
  | otherwise = AUnresolved OutOfScope

exported :: String -> Bool
exported p = Set.member p javaPackages

-- | `source_root`: the standard-layout root holding the path —
-- `<module>/src/<set>/java`, the module empty at the tree root — and
-- its set; the first `src` two segments before a `java` with a segment
-- after it.
sourceRoot :: String -> Maybe (String, String)
sourceRoot path = case [(at, set) | (at, "src" : set : "java" : _ : _) <- zip [0 ..] (tails segs)] of
  (at, set) : _ -> Just (intercalate "/" (take (at + 3) segs), set)
  [] -> Nothing
 where
  segs = splitOn '/' path

-- | `own_root_dir`: of a split package's directories, the one inside the
-- importing file's own source root, when exactly one is.
ownRootDir :: String -> [String] -> Maybe String
ownRootDir from dirs = do
  (root, _) <- sourceRoot from
  case filter ((root <> "/") `isPrefixOf`) dirs of
    [one] -> Just one
    _ -> Nothing
