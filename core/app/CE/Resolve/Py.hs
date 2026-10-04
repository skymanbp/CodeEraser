-- | Python rungs (design §4 row 2; moved from cli/src/graph/ladder/py.rs
-- in plan v2.33 wave W2a). R1 leading-dot relative imports — n dots
-- climb n−1 package levels, then the dotted remainder walks down; R2
-- absolute dotted paths tried against every source root {repo root,
-- src/, pyproject-declared dirs} — two roots producing DIFFERENT files
-- is ambiguous_root, never a pick; R3 the `a/b/__init__.py`-exists-but-
-- `a/b/c.py`-does-not degradation: a file-level edge to the longest
-- prefix package's __init__.py, the symbol deliberately left to the
-- ledger (degrading beats guessing); R4 stdlib names (machine-generated
-- CPython 3.13 list) or pyproject-declared dependencies ⇒ External.
-- Within one root the package/module order is normative (CPython's
-- FileFinder checks directories before same-named modules), so a double
-- hit inside a root is not ambiguity — only cross-root disagreement is.
-- `importlib(var)` / `__import__` dynamism never reaches this ladder:
-- the site detector only opens import statements.
--
-- A specifier arrives as its leading-dot count and its dotted segments,
-- each segment the pieces its text holds between slashes (a segment
-- holds no slash in any import statement; the pieces keep the join
-- exact for the one that does).
module CE.Resolve.Py (resolvePy) where

import CE.Resolve.Answer
import CE.Resolve.Cost (Reason (..))
import CE.Resolve.Request (PyFacts (..))
import CE.Resolve.Vocab (Affix (..), Word' (..), pyStdlib)
import CE.Resolve.World
import Control.Applicative ((<|>))
import Data.Array ((!))
import qualified Data.IntSet as IS

-- | One site: the file it stands in, its leading dots, its dotted
-- segments (empty for a bare `from . import x`).
resolvePy :: World -> PyFacts -> Int -> Int -> [[Int]] -> Answer
resolvePy w facts from dots dotted
  | dots > 0 = relative w from dots dotted
  | otherwise =
      firstOf
        [ absolute w roots dotted (moduleAt w)
        , withRung 3 <$> absolute w roots dotted (initPrefix w)
        ]
        (stdlibOrDeps w facts dotted)
 where
  roots = [word w WEmpty] : [word w WSrc] : map (map fromInteger) (pyRoots facts)

-- | R1: dots climb, remainder walks down, package order normative.
relative :: World -> Int -> Int -> [[Int]] -> Answer
relative w from dots dotted = case climb (dots - 1) (parentOf w from) of
  Nothing -> AUnresolved OutOfScope
  Just dir -> case (moduleAt w dir dotted, initPrefix w dir dotted) of
    (Just f, _) -> AFile f 1
    (_, Just f) -> AFile f 3
    _ -> AUnresolved OutOfScope
 where
  climb 0 dir = Just dir
  climb n dir
    | null dir = Nothing
    | otherwise = climb (n - 1 :: Int) (init dir)

-- | R2 / R3 shared frame: one candidate rule against every source
-- root; distinct files from different roots ⇒ ambiguous_root.
absolute :: World -> [[Int]] -> [[Int]] -> ([Int] -> [[Int]] -> Maybe Int) -> Maybe Answer
absolute _ roots dotted rule =
  case IS.toList (IS.fromList [f | root <- roots, Just f <- [rule root dotted]]) of
    [] -> Nothing
    [f] -> Just (AFile f 2)
    _ -> Just (AUnresolved AmbiguousRoot)

-- | One dotted module under one root: package before module (CPython
-- finder order — normative, not ambiguous). An empty dotted path names
-- the root itself (`from . import x`), its text as it stands — the
-- empty root is no piece.
moduleAt :: World -> [Int] -> [[Int]] -> Maybe Int
moduleAt w root dotted = do
  base <-
    if null dotted
      then Just (if root == [word w WEmpty] then [] else map K root)
      else joinRel w root (map K (concat dotted))
  fileAt w (base <> [K (word w WInitPy)]) <|> fileAt w (moduleFile base)
 where
  moduleFile base = case unsnoc base of
    Nothing -> [glue w (affix w APy) (K (word w WEmpty))]
    Just (ini, l) -> ini <> [glue w (affix w APy) l]

-- | Longest strict prefix of the dotted path whose package __init__.py
-- is in scope (R3: file-level edge, symbol stays open): a hit counts
-- when the file is an `__init__.py` below the root.
initPrefix :: World -> [Int] -> [[Int]] -> Maybe Int
initPrefix w root dotted =
  case [f | k <- [length segs - 1, length segs - 2 .. 1], Just f <- [moduleAt w root (take k segs)], package f] of
    (f : _) -> Just f
    [] -> Nothing
 where
  segs = filter (/= [word w WEmpty]) dotted
  package f = wFileBase w ! f == word w WInitPy && not (null (parentOf w f))

-- | R4: stdlib table or pyproject-declared dependency ⇒ External.
-- `__future__` is a real stdlib module (PEP 236) the public-names table
-- filters out with every other underscore name, so it is answered here
-- by name.
stdlibOrDeps :: World -> PyFacts -> [[Int]] -> Answer
stdlibOrDeps w facts dotted = case take 1 dotted of
  [[top]]
    | top == word w WFuture || IS.member top stdlib || toInteger top `elem` pyDeps facts -> AExternal 4
  _ -> AUnresolved OutOfScope
 where
  stdlib = IS.fromList (concatMap (spelled w) pyStdlib)

unsnoc :: [a] -> Maybe ([a], a)
unsnoc [] = Nothing
unsnoc xs = Just (init xs, last xs)
