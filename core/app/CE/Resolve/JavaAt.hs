-- | One Java site's questions to the package index (plan v2.33 W2-text
-- stage C; moved from cli/src/graph/ladder/java.rs's `At`, each function
-- named after the Rust method it replaces). `from` is the referencing
-- file: every candidate handed out is one it can see (CE.Resolve.JavaPick
-- `visibleIn`).
module CE.Resolve.JavaAt (
  classOf,
  classFiles,
  packageDir,
  declared,
  lastSegment,
) where

import CE.Resolve.Answer
import CE.Resolve.Cost (Reason (..))
import CE.Resolve.JavaHeader
import CE.Resolve.JavaPick
import CE.Resolve.Str (joinDir, parentDir, splitOn)
import Control.Applicative ((<|>))
import Data.List (find, intercalate)
import Data.Maybe (fromMaybe, listToMaybe, mapMaybe)
import qualified Data.Set as Set

-- | `class_of`: the file declaring the class a fully qualified name
-- names — the longest prefix `p.C` whose package `p` holds a file
-- declaring `C`; the whole name at `exact`, a shorter prefix (an
-- enclosing class) at `nested`. A package segment is required.
classOf :: JEnv -> String -> [String] -> Int -> Int -> Maybe Answer
classOf env from segs exact nested = listToMaybe (mapMaybe at [n, n - 1 .. 2])
 where
  n = length segs
  at k = settle env (classFiles env from (intercalate "." (take (k - 1) segs)) (segs !! (k - 1))) (if k == n then exact else nested) AmbiguousRoot

-- | `package_dir`: `import a.b.*` — the one directory holding the
-- package's files this file can see; its own source root's part of a
-- split package, else the declared roots' one.
packageDir :: JEnv -> String -> String -> Maybe Answer
packageDir env from package = do
  let dirs = Set.fromList (map parentDir (visibleIn env from package))
  if Set.null dirs
    then Nothing
    else Just $ case maybe (declaredOne env dirs) Right (ownRootDir from (Set.toList dirs)) of
      Right dir -> APackage dir 2
      Left () -> AUnresolved AmbiguousRoot

-- | `declared`: the unit's own view of a simple name — a single-type
-- import of it (a type import before a static one: two namespaces), the
-- file's own package (its own directory first), then its star-imported
-- packages.
declared :: JEnv -> String -> String -> JHeader -> Maybe Answer
declared env from name h = case single False <|> single True of
  Just i ->
    let segs = splitOn '.' (iName i)
        cut = if iStatic i then length segs - 1 else length segs
     in Just (fromMaybe (jdk segs) (classOf env from (take cut segs) 3 3))
  Nothing
    | beside `elem` own -> Just (AFile beside 3)
    | otherwise -> settle env own 3 AmbiguousRoot <|> settle env starred 3 AmbiguousPaths
 where
  single wanted = find (\i -> not (iStar i) && iStatic i == wanted && lastSegment (iName i) == name) (hImports h)
  own = classFiles env from (hPackage h) name
  beside = joinDir (parentDir from) (name <> ".java")
  starred = concat [classFiles env from (iName i) name | i <- hImports h, iStar i, not (iStatic i)]

-- | `rsplit('.').next()`: the text after the last `.`.
lastSegment :: String -> String
lastSegment = reverse . takeWhile (/= '.') . reverse
