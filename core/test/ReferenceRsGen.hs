{-# LANGUAGE OverloadedStrings #-}

-- | The seeded cases behind ReferenceRs (plan v2.33 W2-text stage F) and
-- the world a case answers the core's questions from. A case is a tree
-- of Cargo packages — manifest records (a `[package]` name or none, a
-- `[lib]` path, `[[bin]]` paths, dependencies; one now and then a file
-- that does not read, some the walk does not list) — and Rust source
-- files held as item records, one item a row: `mod` declarations (pub or
-- not, a `#[path]` value now and then), bodied modules holding items of
-- their own, `use` items (pub or not; a hand fold's first line beside the
-- whole argument), definitions and `extern crate` bindings. The sites are
-- every declaration and every use, at their rows; the declared crate
-- roots a subset of the walked files. No RNG: an LCG over the case
-- number (the Reference.hs posture).
module ReferenceRsGen (
  RCase (..),
  Item (..),
  Man (..),
  Row (..),
  Block,
  rsCases,
  rowsOf,
  manPath,
  blocksOf,
  covering,
  atRow,
  modName,
  surfaceOf,
  defsOf,
  entriesOf,
  trim,
  stripSuffix',
  splitOnStr,
) where

import Data.Aeson (Value, object, toJSON, (.=))
import Data.List (isInfixOf, isPrefixOf, isSuffixOf, sortOn)
import Data.Maybe (listToMaybe)
import qualified Data.Set as Set
import ReferenceJavaGen (pickBy, subsetBy)
import ReferenceResolveGen (rands, splitOn)

-- | One item: a `mod x;` (pub, its `#[path]` value), a bodied `mod x { … }`
-- (pub, its items), a `use` (pub, its first line, its whole argument), a
-- definition (its name, pub), an `extern crate x [as y]` (pub).
data Item
  = IMod String Bool (Maybe String)
  | IInline String Bool [Item]
  | IUse Bool String String
  | IDef String Bool
  | IExtern String (Maybe String) Bool

-- | A manifest record: its directory, `[package]` name (Nothing: a
-- workspace root), `[lib]` path, `[[bin]]` paths, dependencies, whether
-- the walk lists it, whether it reads.
data Man = Man
  { mDir :: String
  , mName :: Maybe String
  , mLib :: Maybe String
  , mBins :: [String]
  , mDeps :: [String]
  , mWalked :: Bool
  , mReads :: Bool
  }

data RCase = RCase
  { rFiles :: [String]
  , rSources :: [(String, [Item])]
  , rMans :: [Man]
  , rDeclared :: [String]
  }

-- | One item at its row: the item, its row, the bodied modules enclosing
-- it (outermost first, each with its first and last row and its items).
data Row = Row
  { rwItem :: Item
  , rwRow :: Int
  , rwIn :: [(String, Int, Int, [Item])]
  }

manPath :: Man -> String
manPath m = if null (mDir m) then "Cargo.toml" else mDir m <> "/Cargo.toml"

-- | A file's items flattened to rows: a bodied module takes its own row,
-- its items' rows and a closing row.
rowsOf :: [Item] -> [Row]
rowsOf = fst . go [] 0
 where
  go _ n [] = ([], n)
  go outer n (it : rest) = case it of
    IInline name _ inner ->
      let (innerRows, end) = go (outer <> [(name, n, end, inner)]) (n + 1) inner
          (more, n') = go outer (end + 1) rest
       in (Row it n outer : innerRows <> more, n')
    _ -> let (more, n') = go outer (n + 1) rest in (Row it n outer : more, n')

-- | Two hundred cases.
rsCases :: [RCase]
rsCases = map rsCase [1 .. 200]

rsCase :: Int -> RCase
rsCase k = RCase walked sources mans declared
 where
  r = rands (k + 2003)
  pick = pickBy r
  subset = subsetBy r
  pkgDirs = "" : subset 1 ["crates/a", "crates/b", "tools/x"]
  mans = [man (10 * j) d | (j, d) <- zip [1 ..] pkgDirs, j == 1 || odd (r (10 * j))]
  man i d =
    Man d (if d == "" && r (i + 1) `mod` 5 == 0 then Nothing else Just (pick (i + 2) names)) (if r (i + 3) `mod` 4 == 0 then Just (pick (i + 4) libs) else Nothing) (subset (i + 5) bins) (subset (i + 6) deps) (r (i + 7) `mod` 7 /= 0) (r (i + 8) `mod` 9 /= 0)
  paths = Set.toList (Set.fromList [under d f | (j, d) <- zip [0 ..] pkgDirs, f <- take (3 + r (50 + j) `mod` 8) (rotate (r (60 + j)) files)])
  sources = [(p, items (200 + 20 * i)) | (i, p) <- zip [0 ..] paths]
  walked = [p | (i, p) <- zip [0 :: Int ..] paths, r (400 + i) `mod` 13 /= 0]
  declared = [p | (i, p) <- zip [0 :: Int ..] walked, r (500 + i) `mod` 17 == 0]
  items i = [item (i + 3 * n) True | n <- [0 .. 2 + r i `mod` 6]]
  item i top = case r i `mod` 9 of
    0 -> IMod (pick (i + 1) mods) (odd (r (i + 2))) (if r (i + 3) `mod` 4 == 0 then Just (pick (i + 4) attrs) else Nothing)
    1 | top -> IInline (pick (i + 1) mods) (odd (r (i + 2))) [item (i + 7 * n + 5) False | n <- [0 .. r (i + 3) `mod` 3]]
    5 -> IDef (pick (i + 1) defs) (odd (r (i + 2)))
    6 -> IExtern (pick (i + 1) externs) (if odd (r (i + 2)) then Just "y" else Nothing) (odd (r (i + 3)))
    _ -> let (first, whole) = pick (i + 1) uses in IUse (r (i + 2) `mod` 3 /= 0) first whole

rotate :: Int -> [a] -> [a]
rotate n xs = let m = n `mod` length xs in drop m xs <> take m xs

under :: String -> String -> String
under d b = if null d then b else d <> "/" <> b

-- | The case's word lists, each one text parted at `|`.
names, libs, bins, deps, files, mods, attrs, defs, externs :: [String]
names = splitOn '|' "a|a-b|core|x|a|b"
libs = splitOn '|' "src/x.rs|lib/core.rs|src/lib.rs|src/missing.rs"
bins = splitOn '|' "src/tools/gen.rs|src/main.rs|src/bin/x.rs"
deps = splitOn '|' "serde|a-b|tempfile|x"
files = splitOn '|' "src/lib.rs|src/main.rs|src/a.rs|src/a/deep.rs|src/b/mod.rs|src/b/c.rs|src/bin/x.rs|tests/t.rs|src/x.rs|lib/core.rs|src/tools/gen.rs|build.rs|src/c.rs|src/a/mod.rs|src/other.rs|src/bin/y/main.rs"
mods = splitOn '|' "a|b|c|deep|other|x|tests|a|b"
attrs = splitOn '|' "other.rs|../x.rs|a/deep.rs|deep.rs|b/c.rs|../../../escape.rs"
defs = splitOn '|' "Thing|Other|C|Deep|a|deep|Thing"
externs = splitOn '|' "core|a|serde|x|a_b"

-- | Use arguments: a first line and the whole argument (a hand fold's
-- first line ends in `::`).
uses :: [(String, String)]
uses =
  [(u, u) | u <- splitOn '|' whole]
    <> [("crate::a::", "crate::a::deep::Deep"), ("super::", "super::b::Other"), ("crate::", "crate::")]
 where
  whole = "crate::a|crate::a::Thing|crate::a::deep::Deep|crate::b::c::C|crate::b::Other|crate::Thing|crate::x::Thing|self::a|self::deep::Deep|self::Thing|super::Thing|super::super::a::Thing|super::b|super::super::super::x|a::Thing|a::deep|deep::Deep|b::c::C|a_b::x::Thing|x::Thing|core::fmt|std::fmt|serde::de::X|tempfile::T|::a::Thing|::core::fmt|crate::a::*|self::b::*|super::*|crate::a::Thing as Z|crate::b::{c, Other}|a::{deep, Thing}|test::Helper|crate::a::deep::Deep::f|crate::c::Thing|other::Deep"

-- the facts a source answers

-- | A bodied module as its key: name, first row, last row.
type Block = (String, Int, Int)

blocksOf :: Row -> [Block]
blocksOf rw = [(n, s, e) | (n, s, e, _) <- rwIn rw]

-- | The bodied modules covering a row, outermost first.
covering :: [Item] -> Int -> [Block]
covering its row = sortOn (\(_, s, _) -> s) [b | b@(_, s, e) <- Set.toList blocks, s <= row, row <= e]
 where
  blocks = Set.fromList (concatMap blocksOf (rowsOf its))

-- | A source's answers at a row (fact 4).
atRow :: [Item] -> Int -> Value
atRow its row =
  object
    [ "depth" .= length cover
    , "mods" .= [n | (n, _, _) <- cover]
    , "items" .= [[toJSON n, toJSON attr] | rw <- here, (n, attr) <- modItem (rwItem rw)]
    , "ns" .= [[toJSON n, toJSON bodied, toJSON (rwRow rw)] | rw <- rowsOf its, blocksOf rw == cover, Just (n, bodied) <- [modName (rwItem rw)]]
    , "uses" .= [[first, whole] | rw <- here, IUse _ first whole <- [rwItem rw]]
    ]
 where
  here = [rw | rw <- rowsOf its, rwRow rw == row]
  cover = covering its row
  modItem it = case it of
    IMod n _ attr -> [(n, attr)]
    IInline n _ _ -> [(n, Nothing)]
    _ -> []

modName :: Item -> Maybe (String, Bool)
modName it = case it of
  IMod n _ _ -> Just (n, False)
  IInline n _ _ -> Just (n, True)
  _ -> Nothing

-- | A source's top-level surface (fact 5).
surfaceOf :: [Item] -> Value
surfaceOf its = object ["defs" .= [[toJSON n, toJSON p] | (n, p) <- defsOf its], "uses" .= [[toJSON n, toJSON segs, toJSON row, toJSON p] | (n, segs, row, p) <- entriesOf its]]

defsOf :: [Item] -> [(String, Bool)]
defsOf its = [d | it <- its, Just d <- [def it]]
 where
  def it = case it of
    IMod n p _ -> Just (n, p)
    IInline n p _ -> Just (n, p)
    IDef n p -> Just (n, p)
    _ -> Nothing

-- | The top-level bindings: (bound name, path, row, pub).
entriesOf :: [Item] -> [(String, [String], Int, Bool)]
entriesOf its = concat [entries (rwItem rw) (rwRow rw) | rw <- rowsOf its, null (rwIn rw)]
 where
  entries it row = case it of
    IUse p _ whole -> [(n, segs, row, p) | (n, segs) <- flat whole]
    IExtern n alias p -> [(maybe n id alias, ["", n], row, p)]
    _ -> []

-- | A use argument flattened: a group expands, a rename binds its alias,
-- a glob binds `*`, a trailing `self` names its module.
flat :: String -> [(String, [String])]
flat whole = case break (== '{') whole of
  (pre, '{' : body) -> concat [flat (dropColons pre <> "::" <> trim e) | e <- splitOn ',' (takeWhile (/= '}') body)]
  _
    | " as " `isInfixOf` whole -> let (p, rest) = breakOn " as " whole in [(trim (drop 4 rest), segsOf p)]
    | Just p <- stripSuffix' "::*" whole -> [("*", segsOf p) | not (null (segsOf p))]
    | otherwise -> case reverse (segsOf whole) of
        ("self" : rest@(m : _)) -> [(m, reverse rest)]
        l@(n : _) -> [(n, reverse l)]
        [] -> []
 where
  dropColons s = maybe s id (stripSuffix' "::" s)

segsOf :: String -> [String]
segsOf p = ["" | "::" `isPrefixOf` trim p] <> filter (not . null) (map trim (splitOnStr "::" p))

-- strings, written here

trim :: String -> String
trim = reverse . dropWhile (== ' ') . reverse . dropWhile (== ' ')

stripSuffix' :: String -> String -> Maybe String
stripSuffix' suf s = if suf `isSuffixOf` s then Just (take (length s - length suf) s) else Nothing

splitOnStr :: String -> String -> [String]
splitOnStr sep s = case breakOn sep s of
  (a, rest) | sep `isPrefixOf` rest -> a : splitOnStr sep (drop (length sep) rest)
  (a, _) -> [a]

breakOn :: String -> String -> (String, String)
breakOn sep s = maybe (s, "") (`splitAt` s) (listToMaybe [i | i <- [0 .. length s], sep `isPrefixOf` drop i s])
