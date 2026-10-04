{-# LANGUAGE OverloadedStrings #-}

-- | The seeded cases behind ReferenceResolve (plan v2.33 wave W2a) and
-- the request a case lowers to. A case is a tree spelled as strings —
-- walked files, sites, each language's configuration — the way the
-- measuring side holds it before lowering; `lowered` turns it into the
-- integer request with the measuring side's conventions
-- (cli/src/graph/resolve/lower.rs, tokens.rs) but its own id
-- assignment: every string the request names is collected first and
-- numbered in sorted order (the measuring side numbers on first sight),
-- so an answer that leaned on the numbering would disagree here.
-- No RNG: an LCG over the case number (the Reference.hs posture).
module ReferenceResolveGen (
  Case (..),
  cases,
  lowered,
  request,
  answers,
  Ref (..),
  splitOn,
  angled,
) where

import CE.Resolve.Cost (Reason (..), langC, langCpp, langGo, langLua, langPy, sepDot, sepSlash)
import CE.Resolve.Vocab (affixes, kindLoad, kindRequire, words)
import Data.Aeson (Value (..), object, parseJSON, (.=))
import qualified Data.Aeson.Key as Key
import qualified Data.Aeson.KeyMap as KM
import Data.Aeson.Types (parseMaybe)
import Data.List (intersperse, isSuffixOf)
import qualified Data.Map.Strict as M
import qualified Data.Set as Set
import Prelude hiding (words)

data Case = Case
  { cFiles :: [String]
  , cExtra :: [String]
  , cSites :: [(Integer, Integer, String, String)]
  , cPyRoots, cPyDeps, cLuaRoots, cCRoots :: [String]
  , cLuaTemplates :: [(String, String)]
  , cGoMods :: [(String, String, [(String, String)])]
  }

-- | A site's answer with its target spelled.
data Ref = RFile String Int | RPkg String Int | RExt Int | RUnres Reason
  deriving (Eq, Show)

-- | Two hundred seeded cases, eight sites each (two per language).
cases :: [Case]
cases = map caseOf [1 .. 200]

caseOf :: Int -> Case
caseOf k = Case files [] sites (subset 4 ["lib", "src/a", "", "a/"]) (subset 5 ["dep", "a"]) (subset 6 ["lib", "a", "x/"]) (subset 7 ["a", "include", "x/"]) (subset 8 [("lib", ".lua"), ("a", "/init.lua"), ("", "_m.lua"), ("x", "/y.lua")]) gomods
 where
  r = rands k
  pick i xs = xs !! (r i `mod` length xs)
  subset i xs = [x | (b, x) <- zip [0 :: Int ..] xs, odd (r i `div` (2 ^ b))]
  files = Set.toList . Set.fromList $ anchors <> twoIncludes <> [pick (20 + 2 * i) dirs `slash` pick (21 + 2 * i) bases | i <- [0 .. 13]]
  -- every tenth case holds one header name in two `include` directories
  -- an including file's ancestors both own (the include rung's refusal)
  twoIncludes = if k `mod` 10 == 0 then ["a/include/m.h", "include/m.h", "a/b/w.c"] else []
  anchors = ["a/z.py", "z.lua", "a/b/z.go", "a/z.c"]
  gomods = take (r 9 `mod` 3) [(d, m, [x | (b, x) <- zip [0 :: Int ..] replaces, odd (r (10 + j) `div` (2 ^ b))]) | (j, (d, m)) <- zip [0 ..] (pick 11 modSets)]
  sites = concat [[site i lang | i <- [2 * n, 2 * n + 1]] | (n, lang) <- zip [0 ..] [langPy, langLua, langGo, langC]] <> [(langC, 0, "a/b/w.c", "m.h") | not (null twoIncludes)]
  site i lang =
    let from = pick (60 + i) [f | f <- files, langOf f `elem` [lang, if lang == langC then langCpp else lang]]
        (kind, spec) = specOf (70 + i) lang
     in (lang, kind, from, spec)
  specOf i lang
    | lang == langPy = (0, pick i [".m", "..m", ".", "...x", "a", "a.b", "a.m", "m", "b.n.q", "os", "os.path", "__future__", "dep", "src", "a..b", "m.", "z"])
    | lang == langLua = pick i ([(kindRequire, s) | s <- ["m", "a.m", "a/m", "n", "y", "x.y", "string", "a..m", "init", "m_m"]] <> [(kindLoad, s) | s <- ["m.lua", "a/m.lua", "../m.lua", "/abs.lua", "c:x.lua", "lua/init.lua"]] <> [(0, "m")])
    | lang == langGo = (0, pick i ["ex.com/mod", "ex.com/mod/a", "ex.com/mod/a/b", "fmt", "net/http", "other.org/z", "local/a", "mod2/x", "nodots", "local"])
    | otherwise = (0, pick i ["m.h", "a/m.h", "../m.h", "<m.h>", "<a/m.h>", "<stdio.h>", "n.h", "<>", "include/m.h", "z.c"])
  dirs = ["", "a", "a/b", "src", "src/a", "lua", "include", "a/include", "lib", "x"]
  bases = ["__init__.py", "m.py", "n.py", "b.py", "m.lua", "init.lua", "n_m.lua", "y.lua", "m.go", "m_test.go", "n.go", "m.h", "n.h", "m.c", "u.cpp"]
  modSets = [[("", "ex.com/mod"), ("a", "local")], [("a", "ex.com/mod/a"), ("src", "ex.com/mod")], [("x", "local"), ("", "mod2")], [("", "ex.com/mod"), ("a", "ex.com/mod")]]
  replaces = [("other.org/z", "./a"), ("local/a", "../x"), ("mod2", "ex.com/mod"), ("ex.com/mod/a", "nodots/q")]
  slash d b = if null d then b else d <> "/" <> b

-- | The LCG stream of one case.
rands :: Int -> Int -> Int
rands k i = fromInteger (iterate step (toInteger (k * 7919 + i * 104729)) !! 3 `div` 65536)
 where
  step x = (x * 1103515245 + 12345) `mod` 2147483648

langOf :: String -> Integer
langOf f
  | ".py" `isSuffixOf` f = langPy
  | ".lua" `isSuffixOf` f = langLua
  | ".go" `isSuffixOf` f = langGo
  | ".c" `isSuffixOf` f = langC
  | any (`isSuffixOf` f) [".h", ".cpp"] = langCpp
  | otherwise = 7

-- | One leaf of a lowered row: a number, or a string to number.
data L = I Integer | S String

-- | The case's request (strings numbered in sorted order), its file and
-- directory tables.
lowered :: Case -> (Value, [String], [String])
lowered = loweredWith id

-- | The same request with the numbering reversed — the ids are names.
request :: Bool -> Case -> Value
request flip' c = let (v, _, _) = loweredWith (if flip' then reverse else id) c in v

loweredWith :: ([String] -> [String]) -> Case -> (Value, [String], [String])
loweredWith order c = (object (["proto" .= ("8.1.0" :: String), "type" .= ("resolve.request" :: String), "id" .= (1 :: Int), "segs" .= M.size ids] <> fields), files, dirs)
 where
  files = cFiles c <> cExtra c
  dirs = "" : Set.toList (Set.fromList [a | f <- files, a <- ancestors (parent f), not (null a)])
  dirId = M.fromList (zip dirs [0 :: Integer ..])
  fileId = M.fromList (zip files [0 :: Integer ..])
  tables =
    [ ("vocab", [map S (words <> affixes)])
    , ("affixes", [[S a, S (take (length n - length a) n), S n] | a <- affixList, n <- names, a `isSuffixOf` n])
    , ("dirs", [[I (dirId M.! parent d), S (base d)] | d <- drop 1 dirs])
    , ("files", [[I (dirId M.! parent f), S (base f), I (langOf f), I (if f `elem` cFiles c then 1 else 0)] | f <- files])
    , ("sites", [[I l, I k, I (fileId M.! from), I form] <> toks | (l, k, from, spec) <- cSites c, let (form, toks) = tokens l k spec])
    , ("py.roots", map raw (cPyRoots c))
    , ("py.deps", [map S (cPyDeps c)])
    , ("lua.roots", map raw (cLuaRoots c))
    , ("lua.templates", [S h : I (toInteger (length (splitOn '/' d))) : raw d <> maybe [] raw t | (d, suf) <- cLuaTemplates c, let (h, t) = headTail suf])
    , ("go.mods", [I (toInteger (length (pieces d))) : map S (pieces d) <> raw m | (d, m, _) <- cGoMods c])
    , ("go.replaces", [[I j, I (form' new), I (toInteger (length (splitOn '/' old)))] <> raw old <> raw new | (j, (_, _, rs)) <- zip [0 ..] (cGoMods c), (old, new) <- rs])
    , ("c.roots", map raw (cCRoots c))
    ]
  names = map base files <> map base dirs
  affixList = Set.toList (Set.fromList (affixes <> filter (not . null) [fst (headTail s) | (_, s) <- cLuaTemplates c]))
  strings = Set.fromList [s | (_, rs) <- tables, row <- rs, S s <- row]
  ids = M.fromList (zip (order (Set.toList strings)) [0 :: Integer ..])
  leaf x = case x of I n -> n; S s -> ids M.! s
  rows name = map (map leaf) (maybe [] id (lookup name tables))
  sub ks = object [Key.fromString k .= rows (p <> "." <> k) | (p, k) <- ks]
  fields =
    [ "vocab" .= concat (rows "vocab")
    , "affixes" .= rows "affixes"
    , "dirs" .= rows "dirs"
    , "files" .= rows "files"
    , "sites" .= rows "sites"
    , "py" .= object ["roots" .= rows "py.roots", "deps" .= concat (rows "py.deps")]
    , "lua" .= sub [("lua", "roots"), ("lua", "templates")]
    , "go" .= sub [("go", "mods"), ("go", "replaces")]
    , "c" .= sub [("c", "roots")]
    ]
  form' new = (if take 2 new == "./" || take 3 new == "../" then 2 else 0) + dottedHead new

-- | A specifier's form and tokens, by its language's separator rule.
tokens :: Integer -> Integer -> String -> (Integer, [L])
tokens l k spec
  | l == langPy = let dots = length (takeWhile (== '.') spec); rest = drop dots spec in (toInteger dots, if null rest then [] else joined sepDot (map raw (splitOn '.' rest)))
  | l == langLua && k == kindRequire = (0, luaName spec)
  | l == langLua = (if take 1 spec `elem` ["/", "~"] || ':' `elem` spec then 1 else 0, raw spec)
  | l == langGo = (dottedHead spec, raw spec)
  | otherwise = maybe (0, raw spec) (\inner -> (1, raw inner)) (angled spec)
 where
  joined sep = concat . intersperse [I sep]
  luaName s = case break (`elem` ['.', '/']) s of
    (p, c : rest) -> S p : I (if c == '.' then sepDot else sepSlash) : luaName rest
    (p, []) -> [S p]

-- | The name inside `<…>`, the system form of an include.
angled :: String -> Maybe String
angled spec = case spec of
  '<' : rest | '>' : r <- reverse rest -> Just (reverse r)
  _ -> Nothing

dottedHead :: String -> Integer
dottedHead s = if '.' `elem` takeWhile (/= '/') s then 1 else 0

raw :: String -> [L]
raw = map S . splitOn '/'

pieces :: String -> [String]
pieces = filter (not . null) . splitOn '/'

headTail :: String -> (String, Maybe String)
headTail s = case break (== '/') s of
  (h, _ : t) -> (h, Just t)
  (h, []) -> (h, Nothing)

splitOn :: Char -> String -> [String]
splitOn c s = case break (== c) s of
  (a, _ : rest) -> a : splitOn c rest
  (a, []) -> [a]

parent, base :: String -> String
parent f = reverse (drop 1 (dropWhile (/= '/') (reverse f)))
base = reverse . takeWhile (/= '/') . reverse

ancestors :: String -> [String]
ancestors "" = [""]
ancestors d = d : ancestors (parent d)

-- | A reply's rows as answers with their targets spelled; Nothing for
-- a reply that is no resolve.result.
answers :: [String] -> [String] -> Value -> Maybe [Ref]
answers files dirs (Object o) = do
  rows <- parseMaybe parseJSON =<< KM.lookup "results" o
  traverse row (rows :: [[Int]])
 where
  row r = case r of
    [g, 0, t, _] -> Just (RFile (files !! t) g)
    [g, 1, t, _] -> Just (RPkg (dirs !! t) g)
    [g, 2, _, _] -> Just (RExt g)
    [_, 3, _, why] -> Just (RUnres (toEnum why))
    _ -> Nothing
answers _ _ _ = Nothing
