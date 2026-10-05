-- | Java rungs (plan v2.30 step 3; design booklet §8 row Java; the
-- inheritance, folded-import and second-top-level-class boundaries
-- closed in step 5b; moved from cli/src/graph/ladder/java.rs in plan
-- v2.33 W2-text stage C — each function below is the Rust function of
-- the same name). Java names a class by its package, and a package is
-- what a compilation unit DECLARES (JLS 7.4), not where the file sits —
-- so the candidate index is the walk's reading of every file's header
-- (CE.Resolve.JavaHeader): a class `a.b.C` is a file that declares
-- `package a.b` and a top-level type `C` (a file the reader found no
-- type in answers by its name, `C.java`), which is javac's own reading.
-- The sites walk:
--   R1 `import a.b.C`: that file — an import folded over lines, whose
--      site the detector cut at its first line (`a.b.`), is read whole
--      from the header's import on the site's line (`headerName`);
--   R2 a shorter prefix naming such a file — `import a.b.C.D` names the
--      nested D inside C.java, `import static a.b.C.m` a member of it;
--      `import a.b.*` the package's one directory, or — `a.b.C.*`, a
--      type's member types — C.java; `import static a.b.C.*` C.java;
--   R3 `type_ref`, in the JLS 6.4.1 order: a member type the class
--      enclosing the site inherits (CE.Resolve.JavaInherit), the file's
--      own single-type import of the name, the name's file in the file's
--      own package (its own directory first), then in the packages it
--      star-imports; a qualified `A.B` is whatever `A` names, else a fully
--      qualified name — its type annotations dropped (CE.Resolve.JavaName);
--   R4 External: a name the JDK answers (the package's `ladder.java`) — an
--      import or qualified name under a package it exports, a `java.lang`
--      type, or a simple name no rung holds in a file whose out-of-scope
--      star imports are all JDK packages.
-- Two files answering one rung is one class in two places —
-- ambiguous_root, or ambiguous_paths across star-imported packages or
-- supertypes — unless the declared `[graph.search_roots] java`
-- directories hold exactly one of them. A file in a standard-layout main
-- source set sees no test code, and a package split across source sets
-- answers the importing file's own part (CE.Resolve.JavaPick). A name the
-- file declares itself reaches no other file: own_unit. Anything else is
-- out_of_scope.
module CE.Resolve.Java (resolveJava) where

import CE.Resolve.Answer
import CE.Resolve.Chars (isRustWhite)
import CE.Resolve.Cost (Reason (..))
import CE.Resolve.JavaAt (classOf, declared, packageDir)
import CE.Resolve.JavaHeader
import CE.Resolve.JavaInherit (inherited)
import CE.Resolve.JavaName (unannotated)
import CE.Resolve.JavaPick
import CE.Resolve.Str (splitOn)
import CE.Resolve.Tables (javaLang, kindImport, kindImportStar, kindTypeRef)
import Control.Applicative ((<|>))
import Data.List (dropWhileEnd, isPrefixOf, stripPrefix)
import Data.Maybe (fromMaybe)
import qualified Data.Map.Strict as M
import qualified Data.Set as Set

-- | `resolve`: one site by its kind; `line` is the site's source line.
resolveJava :: JEnv -> Integer -> String -> Int -> String -> Answer
resolveJava env kind from line written
  | any null segs || (isStatic && length segs < 2) = AUnresolved OutOfScope
  | kind == kindImport && isStatic = found (classOf env from (init segs) 2 2)
  | kind == kindImport = found (classOf env from segs 1 2)
  | kind == kindImportStar && isStatic = found (classOf env from segs 2 2)
  | kind == kindImportStar = found (packageDir env from name <|> classOf env from segs 2 2)
  | kind == kindTypeRef = found (Just (typeRef env from segs line))
  | otherwise = AUnresolved Unsupported
 where
  (isStatic, spec) = case stripPrefix "static" written of
    Just rest@(c : _) | isRustWhite c -> (True, rest)
    _ -> (False, written)
  collapsed = filter (not . isRustWhite) (unannotated spec)
  name = fromMaybe collapsed (headerName env (kind, from, line) collapsed isStatic)
  segs = splitOn '.' name
  found = ownUnit from . fromMaybe (jdk segs)

-- | `header_name`: the import the header read on the site's line, when
-- the site's spec is a cut of it — among the imports of the site's kind
-- and flags on that line, one whose name IS the spec means the spec is
-- whole; else the one name the cut spec (trailing dots off) begins.
headerName :: JEnv -> (Integer, String, Int) -> String -> Bool -> Maybe String
headerName env (kind, from, line) spec isStatic = do
  star <- if kind == kindImport then Just False else if kind == kindImportStar then Just True else Nothing
  h <- M.lookup from (jHeaders env)
  let onLine = [i | i <- hImports h, iLine i == line, iStar i == star, iStatic i == isStatic]
  if any ((== spec) . iName) onLine
    then Nothing
    else case [iName i | i <- onLine, dropWhileEnd (== '.') spec `isPrefixOf` iName i] of
      [one] -> Just one
      _ -> Nothing

-- | `type_ref`: R3, then R4 for what no rung holds.
typeRef :: JEnv -> String -> [String] -> Int -> Answer
typeRef env from segs line = case M.lookup from (jHeaders env) of
  Nothing -> AUnresolved OutOfScope
  Just h -> fromMaybe (external h) (simple h <|> classOf env from segs 3 3)
 where
  first = concat (take 1 segs)
  simple h = inherited env from first h line <|> declared env from first h
  external h
    | Set.member first javaLang = AExternal 4
    | length segs > 1 = jdk segs
    | starJdk env h = AExternal 4
    | otherwise = AUnresolved OutOfScope
