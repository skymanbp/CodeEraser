-- | The cabal layout walk (moved from cli/src/graph/cabal_parse.rs in
-- plan v2.33 W2-text stage D — each function below is the Rust function
-- of the same name): stanza headers, fields with their continuation
-- blocks, and the routing of each collected field into the surface.
-- CE.Resolve.Cabal owns the reads of the surface; this module owns the
-- types and the parse mechanics.
module CE.Resolve.CabalWalk (
  Cabal (..),
  Stanza (..),
  walk,
) where

import CE.Resolve.Str (joinRel, rustTrim, splitOn, splitWhitespace)
import Data.Char (isAsciiLower, isAsciiUpper, isDigit, toLower)
import Data.List (isPrefixOf)
import qualified Data.Map.Strict as M
import Data.Maybe (fromMaybe, isJust, isNothing, listToMaybe, mapMaybe)
import qualified Data.Set as Set

-- | One stanza: its source roots, repo-relative ("" = the tree root; a
-- stanza that declares no hs-source-dirs gets the package directory,
-- cabal's own default of "."), its main-is verbatim (relative to each
-- root), and whether it is the bare `library` — the public library.
data Stanza = Stanza
  { stRoots :: [String]
  , stMain :: Maybe String
  , stLibrary :: Bool
  }
  deriving (Eq, Show)

-- | One cabal: its directory, its `name:` ("" when it states none), its
-- stanzas (at least one), the build-depends names (sorted, once each),
-- whether a library stanza exists, the hidden modules (under
-- other-modules and under no exposed-modules) and the two module lists.
data Cabal = Cabal
  { cDir :: String
  , cName :: String
  , cStanzas :: [Stanza]
  , cDeps :: [String]
  , cHasLibrary :: Bool
  , cHidden :: Set.Set String
  , cExposed :: Set.Set String
  , cOther :: Set.Set String
  }
  deriving (Eq, Show)

-- | `Region`: where the walk stands — in a component stanza (fields land
-- on the last stanza), in a named `common` stanza (fields land on its
-- block, pulled into components by `import:`), or dead under an unknown
-- or nameless header.
data Region = Live | Common String | Dead

-- | `Common`: a common stanza's importable fields.
data Block = Block {bRoots :: [String], bExposed :: Set.Set String, bOther :: Set.Set String}

-- | `Walk`: the region and the common blocks, in file order; the
-- stanzas are held last-first while the walk runs.
data Walk = Walk {wRegion :: Region, wCommons :: M.Map String Block, wOut :: Cabal}

-- | `parse`'s loop and `finish`: the lines of one cabal held in `dir`.
walk :: String -> [String] -> Cabal
walk dir ls = finish dir (wOut (steps dir ls (Walk Live M.empty (Cabal dir "" [] [] False Set.empty Set.empty Set.empty))))

-- | `step`, line after line: a blank or comment line is skipped; a
-- column-0 line that is no field is a stanza header; a field is a field
-- at any column, its continuation block collected.
steps :: String -> [String] -> Walk -> Walk
steps _ [] st = st
steps dir (l : ls) st
  | null t || "--" `isPrefixOf` t = steps dir ls st
  | otherwise = case splitField t of
      Nothing -> steps dir ls (if indented l then st else open t st)
      Just (field, first) ->
        let (more, rest) = continuation ls
         in steps dir rest (consume dir field (first : more) st)
 where
  t = rustTrim l

indented :: String -> Bool
indented l = case l of
  (c : _) -> c == ' ' || c == '\t'
  [] -> False

-- | `open`: the region a header opens. The public library is the bare
-- `library` header; a named one is an internal sublibrary. A `common`
-- header without a name opens nothing.
open :: String -> Walk -> Walk
open t st
  | headW == "common" = st' {wRegion = maybe Dead Common name}
  | headW `notElem` heads = st' {wRegion = Dead}
  | otherwise = st' {wRegion = Live, wOut = out' {cStanzas = Stanza [] Nothing bare : cStanzas out'}}
 where
  headW = headWord t
  name = case splitWhitespace t of
    (_ : n : _) -> Just n
    _ -> Nothing
  bare = headW == "library" && isNothing name
  out' = (wOut st) {cHasLibrary = cHasLibrary (wOut st) || bare}
  st' = st {wOut = out'}

-- | `HEADS`: the headers that open a component with source dirs.
heads :: [String]
heads = ["library", "executable", "test-suite", "benchmark"]

-- | `continuation`: the field's continuation block — indented lines
-- without a field colon; comment lines inside it are skipped at any
-- indentation. Returns the values and the lines after the block.
continuation :: [String] -> ([String], [String])
continuation [] = ([], [])
continuation (l : ls)
  | "--" `isPrefixOf` t = continuation ls
  | not (indented l) || null t || isJust (splitField t) = ([], l : ls)
  | otherwise = let (vs, rest) = continuation ls in (t : vs, rest)
 where
  t = rustTrim l

-- | `finish`: a file with no stanza headers and a stanza with no
-- hs-source-dirs both root at the package directory; the hidden set is
-- other-modules minus every exposed-modules listing.
finish :: String -> Cabal -> Cabal
finish dir c =
  c
    { cStanzas = map rooted (if null stanzas then [Stanza [] Nothing False] else stanzas)
    , cDeps = Set.toAscList (Set.fromList (cDeps c))
    , cHidden = Set.difference (cOther c) (cExposed c)
    }
 where
  stanzas = reverse (cStanzas c)
  rooted s = if null (stRoots s) then s {stRoots = [dir]} else s

-- | `consume`: route one collected field. build-depends is the
-- file-wide union whatever the region; the three importable fields land
-- where the walk stands (`merge`); `name` is read only before any
-- stanza; an unknown field is ignored.
consume :: String -> String -> [String] -> Walk -> Walk
consume dir field values st = case field of
  "hs-source-dirs" -> merge (Block (mapMaybe (joinRel dir) ws) Set.empty Set.empty) st
  "exposed-modules" -> merge (Block [] (Set.fromList ws) Set.empty) st
  "other-modules" -> merge (Block [] Set.empty (Set.fromList ws)) st
  "name" | null (cStanzas out) -> st {wOut = out {cName = fromMaybe "" (listToMaybe values)}}
  "import" -> importCommons ws st
  "main-is" -> setMain values st
  "build-depends" -> st {wOut = out {cDeps = cDeps out <> filter (not . null) (map depName (concatMap (splitOn ',') values))}}
  _ -> st
 where
  ws = wordsOf values
  out = wOut st

-- | `merge`: land one block where the walk stands — on the last
-- component stanza (roots per stanza, module lists file-wide), on the
-- open common stanza's block, or nowhere in a dead region.
merge :: Block -> Walk -> Walk
merge b st = case wRegion st of
  Live ->
    st
      { wOut =
          out
            { cStanzas = case cStanzas out of
                (s : rest) -> s {stRoots = stRoots s <> bRoots b} : rest
                [] -> []
            , cExposed = Set.union (cExposed out) (bExposed b)
            , cOther = Set.union (cOther out) (bOther b)
            }
      }
  Common n -> st {wCommons = M.insertWith (\new old -> Block (bRoots old <> bRoots new) (Set.union (bExposed old) (bExposed new)) (Set.union (bOther old) (bOther new))) n b (wCommons st)}
  Dead -> st
 where
  out = wOut st

-- | `import_commons`: each named common block pulled in where the walk
-- stands; a name with no block pulls nothing.
importCommons :: [String] -> Walk -> Walk
importCommons names st0 = foldl pull st0 names
 where
  pull st n = maybe st (`merge` st) (M.lookup n (wCommons st))

-- | `words`: the list-valued fields' items, comma-, space- or
-- tab-separated.
wordsOf :: [String] -> [String]
wordsOf = filter (not . null) . concatMap pieces
 where
  pieces s = case break (`elem` [',', ' ', '\t']) s of
    (a, _ : rest) -> a : pieces rest
    (a, []) -> [a]

-- | `set_main`: the stanza main-is, the first collected value; a
-- component field only.
setMain :: [String] -> Walk -> Walk
setMain values st = case (wRegion st, cStanzas out) of
  (Live, s : rest) -> st {wOut = out {cStanzas = s {stMain = listToMaybe values} : rest}}
  _ -> st
 where
  out = wOut st

-- | `head_word`: the first word, ASCII-lowercased — stanza and field
-- names are case-insensitive, values are not.
headWord :: String -> String
headWord t = case splitWhitespace t of
  (w : _) -> map lowerAscii w
  [] -> ""

lowerAscii :: Char -> Char
lowerAscii c = if isAsciiUpper c then toLower c else c

alnumAscii :: Char -> Bool
alnumAscii c = isAsciiUpper c || isAsciiLower c || isDigit c

-- | `split_field`: `Field-Name: rest` → (lowercased name, trimmed
-- rest); Nothing for a line that carries no field colon.
splitField :: String -> Maybe (String, String)
splitField t = case break (== ':') t of
  (name, _ : rest) | not (null name) && all (\c -> alnumAscii c || c == '-') name -> Just (map lowerAscii name, rustTrim rest)
  _ -> Nothing

-- | `dep_name`: "base >=4.19 && <5" → "base"; a token that does not
-- open with an ASCII alphanumeric yields nothing.
depName :: String -> String
depName entry = case rustTrim entry of
  t@(c : _) | alnumAscii c -> takeWhile (\x -> alnumAscii x || x == '-') t
  _ -> ""
