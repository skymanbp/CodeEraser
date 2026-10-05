{-# LANGUAGE OverloadedStrings #-}

-- | The seeded cases behind ReferenceHs (plan v2.33 W2-text stage D) and
-- the request a case turns into. A case is a tree of Haskell files with
-- .cabal files at some of its directories, each cabal a small record
-- written out as text — a `name:`, stanzas under a bare or named
-- `library`, an `executable` or a `test-suite`, each with
-- `hs-source-dirs`, `exposed-modules`, `other-modules`, `build-depends`
-- and `main-is` lines (field names in either case, CRLF now and then) —
-- and sites naming modules, bare or under PackageImports. The request
-- carries every cabal's text and, for the mounts table's question, each
-- walked Haskell file's owner: the nearest cabal by directory, the first
-- by name in a directory holding two. No RNG: an LCG over the case number
-- (the Reference.hs posture).
module ReferenceHsGen (
  HCase (..),
  Stanza (..),
  Cabal (..),
  hsCases,
  hsRequest,
  cabalPath,
  owners,
) where

import CE.Resolve.Cost (langHs)
import Data.Aeson (Value, object, toJSON, (.=))
import Data.List (intercalate, isPrefixOf, sortOn)
import qualified Data.Map.Strict as M
import qualified Data.Set as Set
import ReferenceResolveGen (originsOf, requestHeader)

-- | A stanza as the case writes it: its header, roots, exposed and other
-- modules, depends, main-is.
data Stanza = Stanza String [String] [String] [String] [String] (Maybe String)

-- | A cabal: its directory, its file name, its `name:` (Nothing: none
-- written), its stanzas, whether its text uses CRLF and capitalised field
-- names.
data Cabal = Cabal String String (Maybe String) [Stanza] Bool

data HCase = HCase
  { hFiles :: [String]
  , hCabals :: [Cabal]
  , hSites :: [(String, String)]
  }

-- | The case's LCG stream: the fourth step from a seed of the case and
-- the position.
roll :: Int -> Int -> Int
roll k i = fromInteger (steps !! 4 `div` 65536)
 where
  steps = iterate (\y -> (y * 1664525 + 1013904223) `mod` 4294967296) (toInteger (k * 7901 + i * 104677 + 29))

hsCase :: Int -> HCase
hsCase k = HCase files cabals sites
 where
  r = roll k
  pick i xs = xs !! (r i `mod` length xs)
  subset i xs = [x | (x, bits) <- zip xs (iterate (`div` 2) (r i)), odd bits]
  files = Set.toList (Set.fromList ([pick (10 + i) dirs `slash` pick (40 + i) bases | i <- [0 .. 13]] <> planted))
  planted = if even (r 7) then ["other/lib/A.hs", "other/lib/Data/Map.hs", "pkg/src/Main.hs"] else []
  cabals = M.elems (M.fromListWith (\_ old -> old) [(cabalPath cb, cb) | cb <- drawn <> twin])
  drawn = [cabal (100 + 40 * j) d | (j, d) <- zip [0 ..] (subset 1 ["", "pkg", "other", "x"])]
  twin = [cabal 90 "pkg" | r 3 `mod` 6 == 0]
  cabal i d =
    Cabal
      d
      (pick i ["p", "q"] <> ".cabal")
      (if r (i + 1) `mod` 5 == 0 then Nothing else Just (pick (i + 2) names))
      [stanza (i + 3 + 6 * n) | n <- [0 .. r (i + 4) `mod` 3]]
      (odd (r (i + 5)))
  stanza i =
    Stanza
      (pick i ["library", "library", "library sub", "executable e", "test-suite t", "Library"])
      (subset (i + 1) ["src", "lib", ".", "app"])
      (subset (i + 2) modules)
      (subset (i + 3) modules)
      (subset (i + 4) ["base", "other", "pkg", "containers"])
      (if odd (r (i + 5)) then Just (pick (i + 6) ["Main.hs", "A.hs", "src/Main.hs"]) else Nothing)
  sites = [site (300 + 3 * n) | n <- [0 .. 11]]
  site i = (from i, prefix i <> pick (i + 1) specs)
  from i = if r i `mod` 10 == 0 then "ghost/Zz.hs" else pick i (filter (".hs" `isSuffixOfS`) files <> ["Zz.hs"])
  prefix i = pick (i + 2) ["", "", "", "\"other\" ", "\"pkg\" ", "\"base\" ", "\"containers\"  "]

-- | Two hundred cases, twelve sites each.
hsCases :: [HCase]
hsCases = map hsCase [1 .. 200]

isSuffixOfS :: String -> String -> Bool
isSuffixOfS suf s = reverse suf `isPrefixOf` reverse s

dirs, bases, names, modules, specs :: [String]
dirs = ["", "src", "src/A", "pkg/src", "pkg/src/A", "other/lib", "other/lib/A", "x", "app", "pkg/app", "Data"]
bases = ["A.hs", "B.hs", "Main.hs", "Util.hs", "lower.hs"]
names = ["pkg", "other", "base", "x"]
modules = ["A", "A.B", "B", "Util", "Data.Map", "Main"]
specs = ["A", "A.B", "B", "Util", "Main", "Data.A", "Data.Map", "Data.List", "Prelude", "lower", "A..B", "Control.Monad"]

slash :: String -> String -> String
slash "" b = b
slash d b = d <> "/" <> b

cabalPath :: Cabal -> String
cabalPath (Cabal d n _ _ _) = slash d n

-- | The cabal's text.
cabalText :: Cabal -> String
cabalText (Cabal _ _ nm stanzas crlf) = intercalate eol (maybe [] (\n -> [field "name" <> ": " <> n]) nm <> concatMap stanza stanzas) <> eol
 where
  eol = if crlf then "\r\n" else "\n"
  field f = if crlf then capital f else f
  capital = concatMap up . splitDash
  up w = case w of
    (c : cs) | c >= 'a' && c <= 'z' -> toEnum (fromEnum c - 32) : cs
    _ -> w
  splitDash s = case break (== '-') s of
    (a, '-' : rest) -> (a <> "-") : splitDash rest
    (a, _) -> [a]
  stanza (Stanza h roots ex ot deps mainIs) =
    [h]
      <> ["  " <> field "hs-source-dirs" <> ": " <> unwords roots | not (null roots)]
      <> ["  " <> field "exposed-modules" <> ": " <> intercalate ", " ex | not (null ex)]
      <> ["  " <> field "other-modules" <> ": " <> unwords ot | not (null ot)]
      <> ["  " <> field "build-depends" <> ": " <> intercalate ", " [d <> " >=1" | d <- deps] | not (null deps)]
      <> ["  " <> field "main-is" <> ": " <> m | Just m <- [mainIs]]

-- | Each walked Haskell file's nearest cabal: the deepest directory
-- holding one, the first by file name there.
owners :: HCase -> [(String, String)]
owners c = [(f, cabalPath o) | f <- hFiles c, ".hs" `isSuffixOfS` f, Just o <- [nearest f]]
 where
  nearest f = case sortOn (\(Cabal d n _ _ _) -> (negate (length d), n)) [cb | cb@(Cabal d _ _ _ _) <- hCabals c, holds d f] of
    (o : _) -> Just o
    [] -> Nothing
  holds d f = null d || (d <> "/") `isPrefixOf` f

hsRequest :: HCase -> Value
hsRequest c =
  object $
    requestHeader (hFiles c) origins
      <> [ "sites" .= [[toJSON langHs, toJSON (0 :: Int), toJSON (index M.! from), toJSON spec] | (from, spec) <- hSites c]
         , "hs" .= object ["cabals" .= M.toAscList (M.fromList [(cabalPath cb, cabalText cb) | cb <- hCabals c]), "owners" .= owners c]
         ]
 where
  (origins, index) = originsOf (hFiles c) (map fst (hSites c))
