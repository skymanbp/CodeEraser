{-# LANGUAGE OverloadedStrings #-}

-- | The seeded cases behind ReferenceTs (plan v2.33 W2-text stage E) and
-- the world a case answers the core's file-system questions from. A case
-- is a tree of TS files, compiled JavaScript twins on disk beside some,
-- tsconfig records (an `extends` of one target or a list, a baseUrl, a
-- paths map; one now and then a file that does not read), package.json
-- records (a name, an exports value of strings, nulls and maps of
-- subpaths and conditions, dependencies; some not walked) and
-- node_modules directories, each config written out as JSONC text — a
-- comment and a trailing comma now and then, CRLF — and sites naming
-- files relatively, through the paths, through a member's exports, by a
-- bare or builtin name. No RNG: an LCG over the case number (the
-- Reference.hs posture).
module ReferenceTsGen (
  TCase (..),
  Cfg (..),
  Ext (..),
  Exp (..),
  Pkg (..),
  tsCases,
  textOf,
  isFileIn,
  isDirIn,
) where

import qualified Data.ByteString.Lazy.Char8 as BL8
import Data.Aeson (encode, toJSON)
import Data.List (intercalate, isSuffixOf)
import qualified Data.Map.Strict as M
import qualified Data.Set as Set
import ReferenceJavaGen (pickBy, subsetBy)
import ReferenceResolveGen (rands, splitOn)

-- | An `extends` value: one entry or a list; an entry a target or a
-- number (no target).
data Ext = One (Either Int String) | Many [Either Int String]

-- | A tsconfig record: its path, its extends, its baseUrl, its paths
-- (each pattern's targets, Nothing for a value that is no list).
data Cfg = Cfg
  { cPath :: String
  , cExtends :: Maybe Ext
  , cBase :: Maybe String
  , cPaths :: [(String, Maybe [Either Int String])]
  }

-- | An exports value.
data Exp = EStr String | ENull | ENum | EObj [(String, Exp)]

-- | A package.json record: its path, name, exports, dependencies and
-- whether the walk read it.
data Pkg = Pkg
  { pPath :: String
  , pName :: Maybe String
  , pExports :: Maybe Exp
  , pDeps :: [String]
  , pWalked :: Bool
  }

data TCase = TCase
  { tFiles :: [String]
  , tTwins :: [String]
  , tCfgs :: [Cfg]
  , tPkgs :: [Pkg]
  , tUnreadable :: [String]
  , tVendor :: [(String, String)]
  , tSites :: [(String, String)]
  }

-- | Two hundred cases, twelve sites each.
tsCases :: [TCase]
tsCases = map tsCase [1 .. 200]

tsCase :: Int -> TCase
tsCase k = TCase files twins cfgs pkgs unreadable vendor sites
 where
  -- ReferenceResolveGen's stream, from seeds of this battery's own
  r = rands (k + 1009)
  pick = pickBy r
  subset = subsetBy r
  -- every tenth case lays the paths refusal out whole: a root tsconfig
  -- whose one pattern names two walked files, a site spelling it
  dup = k `mod` 10 == 3
  files = Set.toList (Set.fromList ([f | dup, f <- planted !! 1] <> pick 3 planted <> zipWith under (map (\i -> pick (10 + i) dirs) [0 .. 13]) (map (\i -> pick (40 + i) bases) [0 .. 13])))
  twins = [stem <> js | (i, f) <- zip [0 ..] files, (ts, js) <- [(".ts", ".js"), (".mts", ".mjs"), (".cts", ".cjs")], Just stem <- [dropEnd ts f], ".d.ts" /= drop (length f - 5) f, r (60 + i) `mod` 4 == 0]
  cfgs = [Cfg "tsconfig.json" Nothing Nothing [("@dup/*", Just [Right "src/*", Right "src/a/*"])] | dup] <> [cfg (100 + 10 * j) p | (j, p) <- zip [0 ..] (subset 1 cfgPaths), p `notElem` unreadable, not dup || p /= "tsconfig.json"]
  unreadable = [pick 2 cfgPaths | r 4 `mod` 9 == 0] <> ["packages/p2/package.json" | r 5 `mod` 11 == 0]
  cfg i p = Cfg p (ext i) (if odd (r (i + 2)) then Just (pick (i + 3) baseUrls) else Nothing) (subset (i + 4) pathMaps)
  ext i = case r (i + 1) `mod` 8 of
    0 -> Nothing
    1 -> Nothing
    2 -> Just (Many [Right (pick (i + 5) extendsTo), Right (pick (i + 6) extendsTo)])
    3 -> Just (One (Left 7))
    4 -> Just (Many [Right (pick (i + 5) extendsTo), Left 3])
    _ -> Just (One (Right (pick (i + 5) extendsTo)))
  pkgs = [Pkg "package.json" (Just "root") Nothing (subset 6 ["react", "@types/node", "zod", "p2"]) (odd (r 7))]
    <> [Pkg "packages/p1/package.json" (Just "@scope/p1") (Just (pick 8 exportSets)) [] True | odd (r 9)]
    <> [Pkg "packages/p2/package.json" (Just (pick 10 ["p2", "p2", "@scope/p1"])) (Just (pick 11 exportSets)) ["zod"] (r 12 `mod` 4 /= 0) | odd (r 13)]
  vendor = subset 14 [("", "react"), ("a", "zod"), ("packages/p1", "@s/x"), ("src", "lodash")]
  sites = take 12 ([("src/a.ts", "@dup/x") | dup] <> [site (300 + 3 * n) | n <- [0 .. 11]])
  site i = (if r i `mod` 12 == 0 then "ghost/zz.ts" else pick i (files <> ["zz.ts"]), pick (i + 1) specs)

dropEnd :: String -> String -> Maybe String
dropEnd suf s = if suf `isSuffixOf` s then Just (take (length s - length suf) s) else Nothing

-- | A name under a directory ("" the root).
under :: String -> String -> String
under d b = if null d then b else d <> "/" <> b

-- | The case's word lists, each one text parted at `|`.
dirs, bases, cfgPaths, extendsTo, baseUrls, specs :: [String]
dirs = splitOn '|' "|src|src/a|lib|packages/p1/src|packages/p1/src/a|packages/p2/src|types|a|a/b|shared"
bases = splitOn '|' "index.ts|a.ts|x.ts|x.tsx|b.d.ts|m.mts|c.cts|util.ts|y.ts"
cfgPaths = splitOn '|' "tsconfig.json|a/tsconfig.json|a/b/tsconfig.json|shared/base.json|tsconfig.base.json|packages/p1/tsconfig.json"
extendsTo = splitOn '|' "./tsconfig.base.json|./tsconfig.base|../tsconfig.json|../../tsconfig.json|./shared/base|../shared/base.json|./tsconfig.json|../../../out.json|@tsconfig/node18|./missing.json|../a/tsconfig.json"
baseUrls = splitOn '|' ".|./src|src|..|../..|lib"
specs = splitOn '|' "./a|./x|../a|./a.js|./m.mjs|./c.cjs|../src/a|./util|.|./|../..|../../../x|@app/a|@app/a/x|@lib|@dup/x|a|src/a|x|util|@scope/p1|@scope/p1/sub|@scope/p1/lib/a|@scope/p1/lib/a/x|@scope/p1/null|p2|p2/x|react|zod/v4|@s/x|@s/x/y|lodash|fs|node:fs|node:test|node:nope|test|fs/promises|@types/node|nope|root"

planted :: [[String]]
planted = [[], ["src/a.ts", "src/a/x.ts", "src/x.ts", "lib/index.ts"], ["packages/p1/src/index.ts", "packages/p1/src/a/x.ts", "packages/p1/src/lib.ts", "packages/p1/src/a.ts", "packages/p1/src/a/a.ts"], ["src/util.ts", "src/x.tsx", "src/a/index.ts"]]

pathMaps :: [(String, Maybe [Either Int String])]
pathMaps =
  [ ("@app/*", Just [Right "src/*"])
  , ("@lib", Just [Right "lib/index"])
  , ("@dup/*", Just [Right "src/*", Right "src/a/*", Left 3])
  , ("*", Just [Right "src/*", Right "lib/*"])
  , ("x", Nothing)
  ]

exportSets :: [Exp]
exportSets =
  [ EStr "./src/index.ts"
  , EObj [(".", EStr "./src/index.ts"), ("./sub", EObj [("import", EStr "./src/a/x.ts"), ("types", EStr "./src/a.ts")]), ("./lib/*", EStr "./src/*.ts")]
  , EObj [("import", EStr "./src/index.ts"), ("require", EStr "./src/lib.ts")]
  , EObj [("./lib/*", EStr "./src/*"), ("./lib/a/*", EStr "./src/a/*.ts"), ("./null", ENull), ("./n", ENum)]
  , EObj [(".", EObj [("node", EObj [("import", EStr "./src/index.ts")])]), ("./*", EStr "./src/*.ts")]
  , EObj [("./lib/*", EObj [("import", EStr "./src/*.ts"), ("default", EStr "./src/a/*.ts")])]
  ]

-- | A string as a JSON literal (the case's words are ASCII).
lit :: String -> String
lit = BL8.unpack . encode . toJSON

-- | An object of rendered members: a comment before the first now and
-- then, a trailing comma now and then, the line end given.
obj :: (Bool, String) -> [(String, String)] -> String
obj (dressed, eol) ms = "{" <> (if dressed then " // c" <> eol else "") <> intercalate ("," <> eol) [lit kk <> ": " <> v | (kk, v) <- ms] <> (if dressed && not (null ms) then "," else "") <> "}"

-- | A config's text; a tsconfig's dressing taken from its path's length.
cfgText :: Cfg -> String
cfgText (Cfg p e b ps) = obj style (maybe [] (\x -> [("extends", ext x)]) e <> [("compilerOptions", obj style (maybe [] (\u -> [("baseUrl", lit u)]) b <> [("paths", obj style [(pat, targets t) | (pat, t) <- ps]) | not (null ps)]))])
 where
  style = (odd (length p), if length p `mod` 3 == 0 then "\r\n" else "\n")
  ext x = case x of
    One t -> entry t
    Many ts -> "[" <> intercalate ", " (map entry ts) <> "]"
  entry = either show lit
  targets = maybe (lit "notarray") (\ts -> "[" <> intercalate ", " (map entry ts) <> "]")

pkgText :: Pkg -> String
pkgText (Pkg p n e ds _) = obj (odd (length p), "\n") (maybe [] (\x -> [("name", lit x)]) n <> maybe [] (\x -> [("exports", expText x)]) e <> [("dependencies", obj (False, "") [(d, lit "^1") | d <- ds]) | not (null ds)])
 where
  expText x = case x of
    EStr s -> lit s
    ENull -> "null"
    ENum -> "4"
    EObj ms -> obj (False, " ") [(kk, expText v) | (kk, v) <- ms]

-- | A path's text in the case's world: Right (Just text) a config,
-- Right Nothing a file that does not read, Left () no file.
textOf :: TCase -> String -> Either () (Maybe String)
textOf c p = case M.lookup p texts of
  Just t -> Right (Just t)
  Nothing | p `elem` tUnreadable c -> Right Nothing
  _ -> Left ()
 where
  texts = M.fromList ([(cPath x, cfgText x) | x <- tCfgs c] <> [(pPath x, pkgText x) | x <- tPkgs c, pPath x `notElem` tUnreadable c])

isFileIn :: TCase -> String -> Bool
isFileIn c p = p `elem` tFiles c || p `elem` tTwins c || either (const False) (const True) (textOf c p)

-- | Whether `<a>/node_modules/<b>` is a directory.
isDirIn :: TCase -> String -> String -> Bool
isDirIn c a b = (a, b) `elem` tVendor c
