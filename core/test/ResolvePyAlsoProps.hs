{-# LANGUAGE OverloadedStrings #-}

-- | The Python `also` rows' property legs (lane v235_pyimport, proto
-- 9.1.0; CE.Resolve.PyAlso): the ancestor packages a dotted import
-- executes, the imported names that are submodules, the relative
-- importer's own package chain — each derived from importlib's
-- semantics, each a tree of strings put to the shipped core; the misses
-- are quiet (no row, the site's own answer unchanged); `py.names` is
-- refused by name when malformed; and over the two hundred seeded cases
-- the names never move a site's answer, every row is strictly ascending
-- and none names the site's own file or its answer.
module ResolvePyAlsoProps (battery) where

import CE.Resolve (respond)
import CE.Resolve.Cost (Reason (..), langPy)
import Data.Aeson (Value, object, toJSON, (.=))
import Data.Aeson.Types (Pair)
import qualified Data.Map.Strict as M
import ReferenceResolveGen
import WireHarness (refusedBy, runLegs, setKey)

battery :: IO Bool
battery = runLegs (names <> fst starLegs) (map tree (lines trees) <> refusals <> seeded <> snd starLegs)
 where
  names =
    [ "py also: `from pkg import sub` draws pkg/sub.py beside pkg/__init__.py"
    , "py also: `from . import a, b` in pkg/__init__.py draws both names, no self edge"
    , "py also: `import pkg.sub` draws the ancestor package"
    , "py also: `from pkg.sub import f` draws the ancestor, the attribute f nothing"
    , "py also: an attribute name is quiet (`from pkg import helper`)"
    , "py also: a name that is both an attribute and a submodule draws the submodule"
    , "py also: `import a.b.c` draws both ancestor packages, outermost first"
    , "py also: an R3 answer's ancestors ride at rung 3"
    , "py also: `from ns import mod` reaches a namespace package's module, the answer still out_of_scope"
    , "py also: `import ns.mod` with no ns/__init__.py adds nothing"
    , "py also: `from ..util import g` draws the importer's contiguous package chain at rung 1"
    , "py also: the importer's chain stops at the first directory without __init__.py"
    , "py also: `from .. import x` above the root: no frame, no row, the answer refused as before"
    , "py also: `from .. import x` in a top-level package reads the root"
    , "py also: pkg/__main__.py `from . import cli` reaches pkg/cli.py, pkg/__init__.py is the answer"
    , "py also: names are tried under src/"
    , "py also: names are tried under a pyproject-declared directory"
    , "py also: two roots holding pkg/sub.py both draw with their packages, the answer stays ambiguous_root"
    , "py also: an External answer still tries its names"
    , "py also: a missed name under another root draws nothing there (attribute)"
    , "py also: a hit name under the primary's root draws it with its packages"
    , "py also: two import sites (`import a.b as x, c.d`) each draw their own ancestor"
    , "py also: py.names out of site order is refused by name"
    , "py also: py.names naming a site out of range is refused by name"
    , "py also: py.names on a site that is no Python import_from is refused by name"
    , "py also: py.names with an empty name is refused by name"
    , "py also: 200 seeded cases, names never move an answer"
    , "py also: 200 seeded cases, rows strictly ascending, never the site's own file or its answer"
    ]
  refusals =
    [ refusedNames [(1, ["a"]), (0, ["b"])] "py.names 1: not after the site before it"
    , refusedNames [(5, ["a"])] "py.names 0: site out of range"
    , refusedNames [(1, ["a"])] "py.names 0: not a Python import_from site"
    , refusedNames [(0, ["a", ""])] "py.names 0: no names, or an empty one"
    ]
  seeded = [all namesNeverMove cases, all wellFormed cases]
  refusedNames ns = refusedBy respond (pyRequest [] ["__main__.py", "pkg/__init__.py"] [(1, "__main__.py", "pkg"), (0, "__main__.py", "pkg")] ns)

-- | The star-import legs (lane v235_graphtrust, T3), names and bodies.
starLegs :: ([String], [Bool])
starLegs = (names, stars)
 where
  names =
    [ "py star: `from pkg import *` draws each submodule the package's literal __all__ names, an attribute nothing"
    , "py star: `from . import *` reads the climbed package's __all__"
    , "py star: an __all__ no literal reads draws nothing and names the site opaque"
    , "py star: a literal __all__ augmented by what no literal reads draws its literal names and names the site opaque"
    , "py star: a site with names of its own is no star import, whatever __all__ says"
    , "py star: py.all out of path order is refused by name"
    , "py star: py.all naming a file that is no walked __init__.py is refused by name"
    , "py star: py.all with an empty name is refused by name"
    ]
  stars =
    [ starred pkgFiles [(1, "__main__.py", "pkg")] [] [("pkg/__init__.py", ["helper", "sub"], False)] ([(0, "pkg/sub.py", 2)], [])
    , starred ["__main__.py", "pkg/__init__.py", "pkg/inner.py", "pkg/sub.py"] [(1, "pkg/inner.py", ".")] [] [("pkg/__init__.py", ["sub"], False)] ([(0, "pkg/sub.py", 1)], [])
    , starred pkgFiles [(1, "__main__.py", "pkg")] [] [("pkg/__init__.py", [], True)] ([], [0])
    , starred pkgFiles [(1, "__main__.py", "pkg")] [] [("pkg/__init__.py", ["sub"], True)] ([(0, "pkg/sub.py", 2)], [0])
    , starred pkgFiles [(1, "__main__.py", "pkg")] [(0, ["helper"])] [("pkg/__init__.py", [], True)] ([], [])
    , refusedAll [("pkg/sub.py", [], False), ("pkg/__init__.py", [], False)] "py.all 1: not strictly ascending"
    , refusedAll [("pkg/sub.py", ["x"], False)] "py.all 0: not a walked __init__.py"
    , refusedAll [("pkg/__init__.py", [""], True)] "py.all 0: an empty name"
    ]
  pkgFiles = ["__main__.py", "pkg/__init__.py", "pkg/sub.py"]
  refusedAll :: [(String, [String], Bool)] -> String -> Bool
  refusedAll alls = refusedBy respond (pyRequest ["all" .= alls] pkgFiles [(1, "__main__.py", "pkg")] [])

-- | One tree per line, in the order of `names`: `files ; sites ; names ;
-- answers ; also [; lib]` — files space-separated in path order; a site
-- `kind from spec` (0 import, 1 import_from); a names row `site name…`;
-- an answer `f path rung`, `x rung` (External) or `u reason`; an `also`
-- row `site path rung`; items comma-separated; a sixth field `lib`
-- declares `lib` a source root through pyproject.toml. One literal, not a
-- tuple table: rows of one shape rhyme under the clone gate.
trees :: String
trees =
  "__main__.py pkg/__init__.py pkg/sub.py ; 1 __main__.py pkg ; 0 sub ; f pkg/__init__.py 2 ; 0 pkg/sub.py 2\n\
  \pkg/__init__.py pkg/a.py pkg/b.py ; 1 pkg/__init__.py . ; 0 a b ; f pkg/__init__.py 1 ; 0 pkg/a.py 1, 0 pkg/b.py 1\n\
  \__main__.py pkg/__init__.py pkg/sub.py ; 0 __main__.py pkg.sub ; ; f pkg/sub.py 2 ; 0 pkg/__init__.py 2\n\
  \__main__.py pkg/__init__.py pkg/sub.py ; 1 __main__.py pkg.sub ; 0 f ; f pkg/sub.py 2 ; 0 pkg/__init__.py 2\n\
  \__main__.py pkg/__init__.py pkg/sub.py ; 1 __main__.py pkg ; 0 helper ; f pkg/__init__.py 2 ;\n\
  \__main__.py pkg/__init__.py pkg/helper.py pkg/sub.py ; 1 __main__.py pkg ; 0 helper sub VALUE ; f pkg/__init__.py 2 ; 0 pkg/helper.py 2, 0 pkg/sub.py 2\n\
  \__main__.py a/__init__.py a/b/__init__.py a/b/c.py ; 0 __main__.py a.b.c ; ; f a/b/c.py 2 ; 0 a/__init__.py 2, 0 a/b/__init__.py 2\n\
  \__main__.py a/__init__.py a/b/__init__.py ; 0 __main__.py a.b.gone ; ; f a/b/__init__.py 3 ; 0 a/__init__.py 3\n\
  \__main__.py ns/mod.py ; 1 __main__.py ns ; 0 mod ; u out_of_scope ; 0 ns/mod.py 2\n\
  \__main__.py ns/mod.py ; 0 __main__.py ns.mod ; ; f ns/mod.py 2 ;\n\
  \app/__init__.py app/sub/__init__.py app/sub/mod.py app/util.py ; 1 app/sub/mod.py ..util ; 0 g ; f app/util.py 1 ; 0 app/__init__.py 1, 0 app/sub/__init__.py 1\n\
  \a/__init__.py a/b/c/__init__.py a/b/c/m.py a/b/c/x.py ; 1 a/b/c/m.py . ; 0 x ; f a/b/c/__init__.py 1 ; 0 a/b/c/x.py 1\n\
  \m.py x.py ; 1 m.py .. ; 0 x ; u out_of_scope ;\n\
  \pkg/__init__.py x.py ; 1 pkg/__init__.py .. ; 0 x ; u out_of_scope ; 0 x.py 1\n\
  \pkg/__init__.py pkg/__main__.py pkg/cli.py ; 1 pkg/__main__.py . ; 0 cli ; f pkg/__init__.py 1 ; 0 pkg/cli.py 1\n\
  \__main__.py src/pkg/__init__.py src/pkg/sub.py ; 1 __main__.py pkg ; 0 sub ; f src/pkg/__init__.py 2 ; 0 src/pkg/sub.py 2\n\
  \__main__.py lib/pkg/__init__.py lib/pkg/sub.py ; 1 __main__.py pkg ; 0 sub ; f lib/pkg/__init__.py 2 ; 0 lib/pkg/sub.py 2 ; lib\n\
  \__main__.py pkg/__init__.py pkg/sub.py src/pkg/__init__.py src/pkg/sub.py ; 1 __main__.py pkg ; 0 sub ; u ambiguous_root ; 0 pkg/__init__.py 2, 0 pkg/sub.py 2, 0 src/pkg/__init__.py 2, 0 src/pkg/sub.py 2\n\
  \__main__.py os/path.py ; 1 __main__.py os ; 0 path ; x 4 ; 0 os/path.py 2\n\
  \__main__.py a/__init__.py src/a/b.py ; 1 __main__.py a.b ; 0 attr ; f src/a/b.py 2 ;\n\
  \__main__.py a/__init__.py src/a/b/__init__.py src/a/b/sub.py ; 1 __main__.py a.b ; 0 sub ; f src/a/b/__init__.py 2 ; 0 src/a/b/sub.py 2\n\
  \__main__.py a/__init__.py a/b.py c/__init__.py c/d.py ; 0 __main__.py a.b, 0 __main__.py c.d ; ; f a/b.py 2, f c/d.py 2 ; 0 a/__init__.py 2, 1 c/__init__.py 2"

-- | One line of `trees` put to the shipped core: its answers and `also`
-- rows as the line states them.
tree :: String -> Bool
tree line = case splitOn ';' line of
  files : sites : ns : want : also : more ->
    asked (py more) (words files, map site (items sites), map named (items ns)) (map answer (items want), map row (items also))
  _ -> False
 where
  items field = filter (not . null) (map words (splitOn ',' field))
  site :: [String] -> (Integer, String, String)
  site [k, from, spec] = (read k, from, spec)
  site _ = (-1, "", "")
  named :: [String] -> (Int, [String])
  named (i : ns) = (read i, ns)
  named [] = (-1, [])
  answer ["f", p, r] = RFile p (read r)
  answer ["x", r] = RExt (read r)
  answer ["u", "out_of_scope"] = RUnres OutOfScope
  answer ["u", "ambiguous_root"] = RUnres AmbiguousRoot
  answer other = RUnres (error ("no answer form " <> unwords other))
  row :: [String] -> (Int, String, Int)
  row [i, p, r] = (read i, p, read r)
  row _ = (-1, "", -1)
  py more = [lib | any ((== ["lib"]) . words) more]
  lib = "pyproject" .= object ["tool" .= object ["setuptools" .= object ["package-dir" .= object ["" .= ("lib" :: String)]]]]
  asked :: [Pair] -> ([String], [(Integer, String, String)], [(Int, [String])]) -> ([Ref], [(Int, String, Int)]) -> Bool
  asked pairs (files, sites, ns) want = Just want == do
    v <- askCore (pyRequest pairs files sites ns)
    (,) <$> answers v <*> replyKey "also" v

-- | A star-import request (lane v235_graphtrust, T3): the files, the
-- sites, `py.names`, `py.all`, and the `also` and `opaque` tables the
-- core must answer.
starred :: [String] -> [(Integer, String, String)] -> [(Int, [String])] -> [(String, [String], Bool)] -> ([(Int, String, Int)], [Int]) -> Bool
starred files sites ns alls want = Just want == do
  v <- askCore (pyRequest ["all" .= alls] files sites ns)
  (,) <$> replyKey "also" v <*> replyKey "opaque" v

-- | A Python-only request: the sites `(kind, from, spec)`, `py.names`
-- and any other `py` keys.
pyRequest :: [Pair] -> [String] -> [(Integer, String, String)] -> [(Int, [String])] -> Value
pyRequest py files sites ns = siteRequest langPy files sites (const ["py" .= object (("names" .= ns) : py)])

-- | A seeded case with its Python sites made `import_from` and given
-- names: every answer equals the case's own.
namesNeverMove :: Case -> Bool
namesNeverMove c = (askCore (request c) >>= answers) == (askCore (namedCase c) >>= answers)

-- | A seeded case with names: the `also` table strictly ascending by
-- (site, target), no row naming the site's file or its answer.
wellFormed :: Case -> Bool
wellFormed c = case askCore (namedCase c) of
  Nothing -> False
  Just v -> case (answers v, replyKey "also" v :: Maybe [(Int, String, Int)]) of
    (Just got, Just rows) -> and (zipWith (\a b -> key a < key b) rows (drop 1 rows)) && all (fine got) rows
    _ -> False
 where
  key (i, t, _) = (i, t)
  froms = M.fromList (zip [0 ..] [from | (_, _, from, _) <- cSites c])
  fine got (i, t, r) = r `elem` [1, 2, 3] && Just t /= M.lookup i froms && t `notElem` [p | RFile p _ <- take 1 (drop i got)]

-- | The case's request with each Python site `import_from` and names
-- that hit and miss the case's trees, set into its `py` object.
namedCase :: Case -> Value
namedCase c = case replyKey "py" plain :: Maybe (M.Map String Value) of
  Just py -> setKey "py" (toJSON (M.insert "names" names py)) plain
  Nothing -> plain
 where
  plain = request c {cSites = [if l == langPy then (l, 1, f, s) else (l, k, f, s) | (l, k, f, s) <- cSites c]}
  names = toJSON [(i, ["m", "n", "b", "z", "x"] :: [String]) | (i, (l, _, _, _)) <- zip [0 :: Int ..] (cSites c), l == langPy]
