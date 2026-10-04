-- | Python rungs (design §4 row 2; moved from cli/src/graph/ladder/py.rs
-- in plan v2.33 wave W2a, on text since W2-text — each function below is
-- the Rust function of the same name). R1 leading-dot relative imports —
-- n dots climb n−1 package levels, then the dotted remainder walks down;
-- R2 absolute dotted paths tried against every source root {repo root,
-- src/, pyproject-declared dirs} — two roots producing DIFFERENT files is
-- ambiguous_root, never a pick; R3 the `a/b/__init__.py`-exists-but-
-- `a/b/c.py`-does-not degradation: a file-level edge to the longest
-- prefix package's __init__.py, the symbol deliberately left to the
-- ledger (degrading beats guessing); R4 stdlib names (machine-generated
-- CPython 3.13 list) or pyproject-declared dependencies ⇒ External.
-- Within one root the package/module order is normative (CPython's
-- FileFinder checks directories before same-named modules), so a double
-- hit inside a root is not ambiguity — only cross-root disagreement is.
-- `importlib(var)` / `__import__` dynamism never reaches this ladder:
-- the site detector only opens import statements.
module CE.Resolve.Py (resolvePy, PyProject (..), pyproject) where

import CE.Resolve.Answer
import CE.Resolve.Chars (isRustAlnum)
import CE.Resolve.Cost (Reason (..))
import CE.Resolve.Str
import CE.Resolve.Tables (pyStdlib)
import CE.Resolve.World
import Data.Aeson (Value (..))
import qualified Data.Aeson.Key as K
import qualified Data.Aeson.KeyMap as KM
import Data.Foldable (toList)
import Data.List (isPrefixOf, isSuffixOf)
import qualified Data.Set as Set

-- | `roots::PyProject`: the extra import roots and the declared
-- dependency names of the root `pyproject.toml`.
data PyProject = PyProject {ppSourceDirs :: [String], ppDeps :: [String]}

-- | `resolve`: one site, its file and specifier.
resolvePy :: World -> Maybe PyProject -> String -> String -> Answer
resolvePy w py from spec
  | "." `isPrefixOf` spec = relative w from spec
  | otherwise =
      firstOf
        [ absolute w spec roots moduleAt
        , withRung 3 <$> absolute w spec roots initPrefix
        ]
        (stdlibOrDeps py spec)
 where
  roots = sourceRoots py

-- | R1: dots climb, remainder walks down, package order normative.
relative :: World -> String -> String -> Answer
relative w from spec = case climb (dots - 1) (parentDir from) of
  Nothing -> AUnresolved OutOfScope
  Just dir -> case (moduleAt w dir rest, initPrefix w dir rest) of
    (Just p, _) -> AFile p 1
    (_, Just p) -> AFile p 3
    _ -> AUnresolved OutOfScope
 where
  dots = length (takeWhile (== '.') spec)
  rest = drop dots spec
  climb :: Int -> String -> Maybe String
  climb n dir
    | n <= 0 = Just dir
    | null dir = Nothing
    | otherwise = climb (n - 1) (parentDir dir)

-- | R2 / R3 shared frame: one candidate rule against every source root;
-- distinct files from different roots ⇒ ambiguous_root.
absolute :: World -> String -> [String] -> (World -> String -> String -> Maybe String) -> Maybe Answer
absolute w spec roots rule = case Set.toList hits of
  [] -> Nothing
  [p] -> Just (AFile p 2)
  _ -> Just (AUnresolved AmbiguousRoot)
 where
  hits = Set.fromList [p | root <- roots, Just p <- [rule w root spec]]

-- | One dotted module under one root: package before module.
moduleAt :: World -> String -> String -> Maybe String
moduleAt w root dotted = do
  base <- if null dotted then Just root else joinRel root (replaceChar '.' '/' dotted)
  let package = if null base then "__init__.py" else base <> "/__init__.py"
      modul = base <> ".py"
  if member w package
    then Just package
    else if member w modul then Just modul else Nothing

-- | Longest strict prefix of the dotted path whose package __init__.py
-- is in scope (R3).
initPrefix :: World -> String -> String -> Maybe String
initPrefix w root dotted = case [hit | k <- [length segs - 1, length segs - 2 .. 1], Just hit <- [moduleAt w root (dotJoin (take k segs))], "/__init__.py" `isSuffixOf` hit] of
  (hit : _) -> Just hit
  [] -> Nothing
 where
  segs = filter (not . null) (splitOn '.' dotted)
  dotJoin [] = ""
  dotJoin (s : ss) = s <> concatMap ('.' :) ss

-- | R4: stdlib table or pyproject-declared dependency ⇒ External.
stdlibOrDeps :: Maybe PyProject -> String -> Answer
stdlibOrDeps py spec
  | declared || top == "__future__" || Set.member top pyStdlib = AExternal 4
  | otherwise = AUnresolved OutOfScope
 where
  top = takeWhile (/= '.') spec
  declared = maybe False ((top `elem`) . ppDeps) py

-- | `source_roots`: "", "src", the declared directories, consecutive
-- duplicates dropped.
sourceRoots :: Maybe PyProject -> [String]
sourceRoots py = dedup ("" : "src" : maybe [] ppSourceDirs py)
 where
  dedup (a : b : rest) | a == b = dedup (b : rest)
  dedup (a : rest) = a : dedup rest
  dedup [] = []

-- | `roots::pyproject` on the decoded document: the
-- `[tool.setuptools.package-dir]` values and `[tool.poetry.packages]`
-- `from` values that are strings, and each `[project] dependencies`
-- string reduced to its PEP 508 name. Nothing when the document did
-- not decode (the file is absent or not TOML).
pyproject :: Maybe Value -> Maybe PyProject
pyproject Nothing = Nothing
pyproject (Just Null) = Nothing
pyproject (Just doc) = Just (PyProject (setuptools <> poetry) deps)
 where
  setuptools = case at ["tool", "setuptools", "package-dir"] of
    Just (Object m) -> [s | String s' <- KM.elems m, let s = text s']
    _ -> []
  poetry = case at ["tool", "poetry", "packages"] of
    Just (Array ps) -> [text s | Object p <- toList ps, Just (String s) <- [KM.lookup (K.fromString "from") p]]
    _ -> []
  deps = case at ["project", "dependencies"] of
    Just (Array ds) -> [depName (text s) | String s <- toList ds]
    _ -> []
  at = tableAt doc
  text = K.toString . K.fromText

-- | `roots::table_at`: a dotted key path into a document.
tableAt :: Value -> [String] -> Maybe Value
tableAt v [] = Just v
tableAt (Object o) (k : ks) = KM.lookup (K.fromString k) o >>= (`tableAt` ks)
tableAt _ _ = Nothing

-- | `dep_name`: the PEP 508 name prefix — up to the first character
-- that is neither alphanumeric (Rust's `char::is_alphanumeric`) nor
-- `-`, `_`, `.` — trimmed.
depName :: String -> String
depName = rustTrim . takeWhile (\c -> isRustAlnum c || c `elem` ("-_." :: String))
