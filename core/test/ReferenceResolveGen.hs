{-# LANGUAGE OverloadedStrings #-}

-- | The seeded cases behind ReferenceResolve (plan v2.33 wave W2a; on
-- text since W2-text) and the request a case turns into. A case is a tree
-- spelled as strings — walked files, sites, each language's configuration
-- — the way the measuring side holds it; `request` writes it as the
-- measuring side's resolve.request does (cli/src/graph/resolve/
-- request.rs): the walked paths in path order, the sites' specifiers as
-- written, the declared roots, a `pyproject.toml` document, the Lua
-- templates, each go.mod's TEXT (written here from the case's module and
-- replace directives, so the core's go.mod reader is on the path), each
-- DESCRIPTION's TEXT (written here from the case's package and Collate
-- list, over CRLF, continuation lines and quotes, so the core's
-- DESCRIPTION reader is on the path).
-- No RNG: an LCG over the case number (the Reference.hs posture).
module ReferenceResolveGen (
  Case (..),
  cases,
  request,
  answers,
  Ref (..),
  refShape,
  requestHeader,
  originsOf,
  splitOn,
  angled,
  rands,
  askCore,
  replyKey,
  settleWith,
  refEquivalence,
  siteRequest,
) where

import CE.Resolve (respond)
import CE.Resolve.Cost (Reason (..), langC, langCpp, langGo, langLua, langPy, langR)
import CE.Resolve.Tables (kindLibrary, kindLoad, kindRequire, kindSource)
import Data.Aeson (FromJSON, Value (..), decodeStrict, encode, object, parseJSON, toJSON, (.=))
import qualified Data.Aeson.Key as K
import qualified Data.Aeson.KeyMap as KM
import Data.Aeson.Types (Pair, parseMaybe)
import Data.List (isSuffixOf)
import qualified Data.ByteString.Lazy as BL
import qualified Data.Map.Strict as M
import qualified Data.Set as Set
import qualified WireHarness as W

data Case = Case
  { cFiles :: [String]
  , cExtra :: [String]
  , cSites :: [(Integer, Integer, String, String)]
  , cPyRoots, cPyDeps, cLuaRoots, cCRoots :: [String]
  , cLuaTemplates :: [(String, String)]
  , cGoMods :: [(String, String, [(String, String)])]
  , cRRoots :: [String]
  , cDescs :: [(String, Maybe String, [String])]
  }

-- | A site's answer with its target spelled.
data Ref = RFile String Int | RPkg String Int | RExt Int | RUnres Reason | RVia String Int | RSection String (Maybe String) Int | RInert String Int
  deriving (Eq, Show)

-- | An answer's shape, its target dropped: what a battery's "reach
-- every answer" check counts.
refShape :: Ref -> String
refShape a = case a of
  RFile _ r -> "file " <> show r
  RPkg _ r -> "package " <> show r
  RExt r -> "external " <> show r
  RUnres why -> show why
  RVia _ r -> "via " <> show r
  RSection _ slug r -> "section " <> show r <> maybe "" (const " slug") slug
  RInert _ r -> "inert " <> show r

-- | The fields every resolve.request opens with: proto, type, id, the
-- walked files and the origins after them.
requestHeader :: [String] -> [String] -> [Pair]
requestHeader files origins =
  [ "proto" .= ("9.0.0" :: String)
  , "type" .= ("resolve.request" :: String)
  , "id" .= (1 :: Int)
  , "files" .= files
  , "origins" .= origins
  ]

-- | The files a case's sites stand in that the walk did not read, once
-- each in path order, and every path's index in files ++ those.
originsOf :: [String] -> [String] -> ([String], M.Map String Integer)
originsOf files froms = (origins, M.fromList (zip (files <> origins) [0 ..]))
 where
  origins = Set.toList (Set.fromList [f | f <- froms, f `notElem` files])

-- | Two hundred seeded cases, ten sites each (two per language).
cases :: [Case]
cases = map caseOf [1 .. 200]

caseOf :: Int -> Case
caseOf k = Case files [] sites (subset 4 ["lib", "src/a", "", "a/"]) (subset 5 ["dep", "a"]) (subset 6 ["lib", "a", "x/"]) (subset 7 ["a", "include", "x/"]) (subset 8 [("lib", ".lua"), ("a", "/init.lua"), ("", "_m.lua"), ("x", "/y.lua")]) gomods rRoots descs
 where
  r = rands k
  pick i xs = xs !! (r i `mod` length xs)
  subset i xs = [x | (b, x) <- zip [0 :: Int ..] xs, odd (r i `div` (2 ^ b))]
  files = Set.toList . Set.fromList $ anchors <> twoIncludes <> twoRs <> [pick (20 + 2 * i) dirs `slash` pick (21 + 2 * i) bases | i <- [0 .. 13]]
  -- every tenth case holds one header name in two `include` directories
  -- an including file's ancestors both own (the include rung's refusal)
  twoIncludes = if k `mod` 10 == 0 then ["a/include/m.h", "include/m.h", "a/b/w.c"] else []
  -- and every tenth other one a script name under two declared R roots
  -- (the source rung's refusal)
  twoRs = if k `mod` 10 == 5 then ["R/w.R", "x/w.R"] else []
  rRoots = if null twoRs then subset 12 ["R", "pkg/R", "x"] else ["R", "x"]
  anchors = ["a/z.py", "z.lua", "a/b/z.go", "a/z.c", "a/z.R"]
  descs = subset 13 [("", Just "pkgA", pick 14 collates), ("pkg", Just "pkgB", pick 15 collates), ("x", Just "pkgA", []), ("a", Nothing, [])]
  collates = [[], ["m.R", "n.r"], ["b b.R", "sub/q.R", "../m.R"], ["m.R", "m.R", "zz.R"]]
  gomods = take (r 9 `mod` 3) [(d, m, [x | (b, x) <- zip [0 :: Int ..] replaces, odd (r (10 + j) `div` (2 ^ b))]) | (j, (d, m)) <- zip [0 ..] (pick 11 modSets)]
  sites = concat [[site i lang | i <- [2 * n, 2 * n + 1]] | (n, lang) <- zip [0 ..] [langPy, langLua, langGo, langC, langR]] <> [(langC, 0, "a/b/w.c", "m.h") | not (null twoIncludes)] <> [(langR, kindSource, "a/z.R", "w.R") | not (null twoRs)]
  site i lang =
    let from = pick (60 + i) [f | f <- files, langOf f `elem` [lang, if lang == langC then langCpp else lang]]
        (kind, spec) = specOf (70 + i) lang
     in (lang, kind, from, spec)
  specOf i lang
    | lang == langPy = (0, pick i [".m", "..m", ".", "...x", "a", "a.b", "a.m", "m", "b.n.q", "os", "os.path", "__future__", "dep", "src", "a..b", "m.", "z"])
    | lang == langLua = pick i ([(kindRequire, s) | s <- ["m", "a.m", "a/m", "n", "y", "x.y", "string", "a..m", "init", "m_m"]] <> [(kindLoad, s) | s <- ["m.lua", "a/m.lua", "../m.lua", "/abs.lua", "c:x.lua", "lua/init.lua"]] <> [(0, "m")])
    | lang == langGo = (0, pick i ["ex.com/mod", "ex.com/mod/a", "ex.com/mod/a/b", "fmt", "net/http", "other.org/z", "local/a", "mod2/x", "nodots", "local"])
    | lang == langR = pick i (map ((,) kindSource) (words "m.R R/m.R ../m.R /abs.R c:x.R https://x.org/y.R n.r q.R") <> map ((,) kindLibrary) (words "pkgA pkgB stats pkgC") <> [(0, "m")])
    | otherwise = (0, pick i ["m.h", "a/m.h", "../m.h", "<m.h>", "<a/m.h>", "<stdio.h>", "n.h", "<>", "include/m.h", "z.c"])
  dirs = ["", "a", "a/b", "src", "src/a", "lua", "include", "a/include", "lib", "x", "R", "pkg/R", "R/sub", "x/R"]
  bases = ["__init__.py", "m.py", "n.py", "b.py", "m.lua", "init.lua", "n_m.lua", "y.lua", "m.go", "m_test.go", "n.go", "m.h", "n.h", "m.c", "u.cpp", "m.R", "n.r", "b b.R", "q.R", ".R"]
  modSets = [[("", "ex.com/mod"), ("a", "local")], [("a", "ex.com/mod/a"), ("src", "ex.com/mod")], [("x", "local"), ("", "mod2")], [("", "ex.com/mod"), ("a", "ex.com/mod")]]
  replaces = [("other.org/z", "./a"), ("local/a", "../x"), ("mod2", "ex.com/mod"), ("ex.com/mod/a", "nodots/q")]
  slash d b = if null d then b else d <> "/" <> b

-- | A request put to the core in-process, its reply read back; Nothing
-- for a refusal or a reply that does not decode.
-- | A one-language reference battery: every site of every case answered
-- by the core as the reference answers it (the case's request, its
-- sites, the reference's answer to one), and the cases reaching every
-- answer `reach` names.
refEquivalence :: String -> [c] -> (c -> Value, c -> [s], c -> s -> Ref) -> [String] -> IO Bool
refEquivalence lang cs (ask, sitesOf, refOf) reach =
  W.runChecks
    ( W.battery
        ( "resolve: " <> show (length cs) <> " " <> lang <> " cases, " <> show (sum (map (length . sitesOf) cs)) <> " sites, shipped = reference"
        , "resolve: the " <> lang <> " cases reach every answer of the " <> lang <> " rungs"
        )
        (map disagree cs)
        reach
        (Set.fromList [refShape (refOf c s) | c <- cs, s <- sitesOf c])
    )
 where
  disagree c = case askCore (ask c) >>= answers of
    Nothing -> Just "no reply"
    Just got
      | got /= map (refOf c) (sitesOf c) -> Just ("core " <> show got <> " reference " <> show (map (refOf c) (sitesOf c)))
      | otherwise -> Nothing

-- | A one-language case's request: the header (the walked files, then the
-- sites' files the walk did not hold as origins), the sites, and the
-- language's own keys given the origins.
siteRequest :: Integer -> [String] -> [(Integer, String, String)] -> ([String] -> [Pair]) -> Value
siteRequest lang files sites own =
  object (requestHeader files origins <> ["sites" .= [(lang, kind, M.findWithDefault 0 from ix, spec) | (kind, from, spec) <- sites]] <> own origins)
 where
  (origins, ix) = originsOf files [f | (_, f, _) <- sites]

askCore :: Value -> Maybe Value
askCore v = either (const Nothing) decodeStrict (respond "9.0.0" (BL.toStrict (encode v)))

-- | One key of a reply object, decoded.
replyKey :: FromJSON a => String -> Value -> Maybe a
replyKey k (Object o) = parseMaybe parseJSON =<< KM.lookup (K.fromString k) o
replyKey _ _ = Nothing

-- | Ask until the core names no more facts under `tsWanted`, each round
-- adding the named facts as `answer` gives them; Nothing for a refusal or
-- facts still wanted after `rounds` more rounds.
settleWith :: Int -> ([Value] -> Value) -> ((Int, String, String) -> Value) -> Maybe Value
settleWith rounds ask answer = go [] rounds
 where
  go known n = askCore (ask known) >>= \v -> case replyKey "tsWanted" v of
    Nothing -> Nothing
    Just [] -> Just v
    Just wanted
      | n == 0 -> Nothing
      | otherwise -> go (known <> map answer wanted) (n - 1)

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
  | any (`isSuffixOf` f) [".R", ".r"] = langR
  | otherwise = 7

-- | The case's request: files then origins by index, the sites, the
-- configuration as the measuring side sends it.
request :: Case -> Value
request c =
  object $
    requestHeader (cFiles c) (cExtra c)
      <> [ "sites" .= [[toJSON l, toJSON k, toJSON (index M.! from), toJSON spec] | (l, k, from, spec) <- cSites c]
    , "config" .= object ["searchRoots" .= object ["lua" .= cLuaRoots c, "c" .= cCRoots c, "r" .= cRRoots c]]
    , "py" .= object ["pyproject" .= pyproject]
    , "lua" .= object ["templates" .= [[d, t] | (d, t) <- cLuaTemplates c]]
    , "go" .= object ["mods" .= [[if null d then "go.mod" else d <> "/go.mod", goModText m rs] | (d, m, rs) <- cGoMods c]]
    , "r" .= object ["descriptions" .= [[if null d then "DESCRIPTION" else d <> "/DESCRIPTION", descText p cs] | (d, p, cs) <- cDescs c]]
    ]
 where
  index = M.fromList (zip (cFiles c <> cExtra c) [0 :: Integer ..])
  pyproject =
    object
      [ "tool" .= object ["setuptools" .= object ["package-dir" .= M.fromList (zip ["k" <> show i | i <- [0 :: Int ..]] (cPyRoots c))]]
      , "project" .= object ["dependencies" .= [d <> ">=1" | d <- cPyDeps c]]
      ]
  goModText m rs = unlines (("module " <> m) : ["replace " <> old <> " v1 => " <> new | (old, new) <- rs])
  descText p cs = concatMap (<> "\r\n") (["Packaged: 2024-01-01; x", "Package:", maybe "  two words" ("  " <>) p, "Collate:"] <> ["    " <> quote n | n <- cs])
  quote n = if ' ' `elem` n then "'" <> n <> "'" else "\"" <> n <> "\""

-- | The name inside `<…>`, the system form of an include.
angled :: String -> Maybe String
angled spec = case spec of
  '<' : rest | '>' : r <- reverse rest -> Just (reverse r)
  _ -> Nothing

splitOn :: Char -> String -> [String]
splitOn c s = case break (== c) s of
  (a, _ : rest) -> a : splitOn c rest
  (a, []) -> [a]

-- | A reply's rows as answers with their targets spelled, a section's
-- slug from its `sections` row; Nothing for a reply that is no
-- resolve.result.
answers :: Value -> Maybe [Ref]
answers (Object o) = do
  rows <- parseMaybe parseJSON =<< KM.lookup "results" o
  slugs <- maybe (Just []) (parseMaybe parseJSON) (KM.lookup "sections" o)
  let slugAt = M.fromList (slugs :: [(Int, Maybe String)])
  sequence (zipWith (row slugAt) [0 ..] (rows :: [[Value]]))
 where
  row slugAt i r = case r of
    [g, Number 0, String t, _] -> RFile (text t) <$> int g
    [g, Number 1, String t, _] -> RPkg (text t) <$> int g
    [g, Number 2, _, _] -> RExt <$> int g
    [_, Number 3, _, why] -> RUnres . toEnum <$> int why
    [g, Number 4, String t, _] -> RVia (text t) <$> int g
    [g, Number 5, String t, _] -> RSection (text t) <$> M.lookup i slugAt <*> int g
    [g, Number 6, String t, _] -> RInert (text t) <$> int g
    _ -> Nothing
  int v = parseMaybe parseJSON v :: Maybe Int
  text t = maybe "" id (parseMaybe parseJSON (String t))
answers _ = Nothing
