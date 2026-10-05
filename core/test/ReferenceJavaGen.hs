{-# LANGUAGE OverloadedStrings #-}

-- | The seeded Java cases behind ReferenceJava (plan v2.33 W2-text stage
-- C) and the request a case turns into. A case holds what the measuring
-- side's walk hands the Java rungs — the walked files, each one's header
-- as the header lexer read it (package, imports with their lines, type
-- declarations with supertypes, member types and line ranges), the
-- declared `java` roots — and twelve sites, each with its line: imports
-- the header read (now and then cut at a dot, as a declaration folded
-- over lines leaves the detector's first line), drawn names, `static`
-- forms, type annotations (`@T`, `@a.T`, an argument list with a string
-- holding a parenthesis) before a segment. No RNG: an LCG over the case
-- number, its own stream.
module ReferenceJavaGen (
  JCase (..),
  Ty (..),
  JSite,
  javaCases,
  javaRequest,
  pickBy,
  subsetBy,
) where

import CE.Resolve.Cost (langJava)
import CE.Resolve.Tables (kindImport, kindImportStar, kindTypeRef)
import Data.Aeson (Value, object, toJSON, (.=))
import Data.List (isSuffixOf)
import qualified Data.Map.Strict as M
import qualified Data.Set as Set
import ReferenceResolveGen (originsOf, requestHeader)

-- | A type declaration: name, supertypes as written, members, lines.
data Ty = Ty String [String] [Ty] Int Int

-- | kind, from, spec, line.
type JSite = (Integer, String, String, Int)

data JCase = JCase
  { jFiles :: [String]
  , jHeads :: M.Map String (String, [(String, Bool, Bool, Int)], [Ty])
  , jRoots :: [String]
  , jSites :: [JSite]
  }

-- | Two hundred cases.
javaCases :: [JCase]
javaCases = map javaCase [1 .. 200]

-- | The case's LCG stream (seeded apart from ReferenceResolveGen's).
roll :: Int -> Int -> Int
roll k i = fromInteger (go (toInteger (k * 104723 + i * 7907 + 17)) `div` 65536)
 where
  go x = iterate (\y -> (y * 22695477 + 1) `mod` 4294967296) x !! 4

-- | One word of a list, drawn at stream position `i`.
pickBy :: (Int -> Int) -> Int -> [a] -> a
pickBy r i xs = xs !! mod (r i) (length xs)

-- | The words of a list whose bits stream position `i` sets.
subsetBy :: (Int -> Int) -> Int -> [a] -> [a]
subsetBy r i xs = [x | (x, bits) <- zip xs (iterate (`div` 2) (r i)), odd bits]

-- | Qualified class names the headers import and the sites write.
classes :: [String]
classes = ["a.A", "a.b.C", "a.b.C.N", "x.D", "java.util.List", "a.B", "zz.Q", "a.C.N", "a.b.Util"]

-- | The tree of case `k` (its files, their headers, its roots), then
-- its sites.
javaCase :: Int -> JCase
javaCase k = JCase files heads roots (sitesOf r heads (filter (".java" `isSuffixOf`) files))
 where
  r = roll k
  pick = pickBy r
  files = Set.toList (Set.fromList (["src/main/java/a/A.java", "src/test/java/a/T.java"] <> [pick (10 + i) dirs <> "/" <> pick (30 + i) bases | i <- [0 .. 9]]))
  ghost = ["a/Ghost.java" | r 5 `mod` 4 == 0]
  heads = M.fromList [(f, header j f) | (j, f) <- zip [0 ..] (files <> ghost), r (100 + j) `mod` 10 /= 0]
  roots = subsetBy r 6 ["src/main/java", "a", "lib/src/main/java", ""]
  header j f =
    ( if r (200 + j) `mod` 3 /= 0 then packageOf f else pick (210 + j) ["a", "a.b", "x", "", "java.util"]
    , [importOf (300 + 10 * j + n) | n <- [0 .. r (290 + j) `mod` 5 - 1]]
    , [ty (400 + 20 * j + n) (if n == 0 && even (r (410 + j)) then stem f else pick (420 + j + n) names) 0 | n <- [0 .. r (430 + j) `mod` 3 - 1]]
    )
  importOf i =
    let star = r i `mod` 4 == 0
        static = r (i + 1) `mod` 5 == 0
        name
          | star && not static && even (r (i + 2)) = pick (i + 3) ["a", "a.b", "x", "java.util", "zz"]
          | static && not star = pick (i + 3) classes <> ".m"
          | otherwise = pick (i + 3) classes
     in (name, star, static, 2 + r (i + 4) `mod` 5)
  ty :: Int -> String -> Int -> Ty
  ty i name depth =
    let first = 1 + r i `mod` 10
        lastL = first + r (i + 1) `mod` 20
        members = if depth > 0 then [] else [ty (i + 7 * (m + 1)) (pick (i + 2 + m) ["N", "M", "Inner"]) 1 | m <- [0 .. r (i + 3) `mod` 3 - 1]]
     in Ty name (subsetBy r (i + 4) ["C", "a.b.C", "A.N", "C.N", "B", "java.util.List", "x.D"]) members first lastL
  dirs = ["src/main/java/a", "src/main/java/a/b", "src/test/java/a", "lib/src/main/java/a", "a", "x"]
  bases = ["A.java", "B.java", "C.java", "Util.java", "D.java"]
  names = ["C", "B", "Util", "D", "N"]

-- | Twelve sites over a tree's headers and Java files.
sitesOf :: (Int -> Int) -> M.Map String (String, [(String, Bool, Bool, Int)], [Ty]) -> [String] -> [JSite]
sitesOf r heads javaFiles = [site (600 + 10 * s) s | s <- [0 .. 11]]
 where
  pick = pickBy r
  site i s =
    let from = if r i `mod` 12 == 0 then "z/Zz.java" else pick (i + 1) javaFiles
        kind = if r (i + 2) `mod` 25 == 0 then 2 else [kindImport, kindImportStar, kindTypeRef] !! (s `mod` 3)
        (spec, line) = specOf i kind (M.lookup from heads)
     in (kind, from, annotate (i + 5) spec, line)
  specOf i kind h
    | kind == kindTypeRef = (pick (i + 3) (simples <> classes), typeLine i h)
    | (name, _, st, line) : _ <- drop (r (i + 11) `mod` max 1 (length written)) written, even (r (i + 3)) = ((if st then "static " else "") <> cut (i + 4) name, line)
    | otherwise = ((if r (i + 6) `mod` 5 == 0 then "static " else "") <> pick (i + 3) (classes <> ["a", "a.b", "java.util"]), 1 + r (i + 7) `mod` 6)
   where
    written = [im | Just (_, ims, _) <- [h], im@(_, star, _, _) <- ims, star == (kind == kindImportStar)]
  typeLine i h = case h of
    Just (_, _, tys@(_ : _)) | r (i + 8) `mod` 4 /= 0 -> let Ty _ _ _ a b = tys !! (r (i + 9) `mod` length tys) in a + r (i + 10) `mod` (b - a + 1)
    _ -> 1 + r (i + 8) `mod` 30
  cut i name =
    let dots = [n | (n, c) <- zip [1 ..] name, c == '.']
     in if null dots || r i `mod` 3 /= 0 then name else take (dots !! (r (i + 1) `mod` length dots)) name
  annotate i spec
    | r i `mod` 5 /= 0 = spec
    | otherwise =
        let (pre, body) = if take 7 spec == "static " then ("static ", drop 7 spec) else ("", spec)
            places = 0 : [n | (n, c) <- zip [1 ..] body, c == '.']
            at = places !! (r (i + 1) `mod` length places)
         in pre <> take at body <> pick (i + 2) ["@T ", "@a.T ", "@T(1) ", "@T(\"(\") "] <> drop at body
  simples = ["A", "B", "C", "N", "M", "Inner", "String", "List", "D", "Util", "Ghost", "T"]

-- | The package a file's directory spells below `java/`, else the whole
-- directory.
packageOf :: String -> String
packageOf f = map (\c -> if c == '/' then '.' else c) (below (reverse (drop 1 (dropWhile (/= '/') (reverse f)))))
 where
  below d = case [drop (n + 5) d | n <- [0 .. length d - 5], take 5 (drop n d) == "java/"] of
    x : _ -> x
    [] -> d

stem :: String -> String
stem f = takeWhile (/= '.') (reverse (takeWhile (/= '/') (reverse f)))

-- | The case's request: walked files, the unwalked origins after them,
-- the sites with their lines, the headers in path order, the roots.
javaRequest :: JCase -> Value
javaRequest c =
  object $
    requestHeader (jFiles c) origins
      <> [ "sites" .= [[toJSON langJava, toJSON kind, toJSON (index M.! from), toJSON spec, toJSON line] | (kind, from, spec, line) <- jSites c]
    , "config" .= object ["searchRoots" .= object ["java" .= jRoots c]]
    , "java" .= object ["headers" .= [[toJSON p, toJSON pkg, toJSON ims, toJSON (map tyJson tys)] | (p, (pkg, ims, tys)) <- M.toAscList (jHeads c)]]
    ]
 where
  (origins, index) = originsOf (jFiles c) [f | (_, f, _, _) <- jSites c]
  tyJson (Ty n ss ms a b) = toJSON (n, ss, map tyJson ms, a, b)
