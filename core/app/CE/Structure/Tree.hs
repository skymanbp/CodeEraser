-- | The directory tree over walked relative paths (plan v2.33 W1 item 3;
-- the port of cli/src/structure/tree.rs at c2abca5d): dense directory
-- nodes with parent links, per-directory fanout and depth, the
-- sibling-set stem shapes (the S1 facts CE.Structure.Shape folds) and the
-- convention bits (S4: README / config presence). Directories are
-- discovered in sorted path order and numbered densely, so the same paths
-- always give the same tree. The structure, arch and query families build
-- their trees here (CE.Structure.Raw, CE.Arch.Tables, CE.Query.Tree).
--
-- Strings are the paths as the measuring side spelled them: forward
-- slashes, root-relative. Sorting a list of Haskell Strings orders by code
-- point, which is the byte order of their UTF-8 spelling — the order
-- `Vec<&String>::sort` and the `BTreeMap<String, _>` key order give.
module CE.Structure.Tree (Dir (..), Tree (..), build, dirOf, dirId, dirPaths, splitDir, stem, shapeBits) where

import Data.Bits ((.|.))
import Data.Char (isAsciiLower, isAsciiUpper, isDigit, toUpper)
import Data.List (isPrefixOf, sort)
import qualified Data.IntMap.Strict as IM
import qualified Data.Map.Strict as M

-- | `Dir`: one directory node. `dParent` is the node itself for the root
-- (dense id 0); `dShapes` is the file count per stem shape, keyed by
-- the shape bits, ascending — the `patternShapes` rows.
data Dir = Dir
  { dParent :: !Int
  , dDepth :: !Int
  , dSubdirs :: !Int
  , dFiles :: !Int
  , dShapes :: !(M.Map Int Int)
  , dConventions :: !Int
  }

-- | `Tree`: dense ids, root = 0 (the walk root); `tIds` maps directory
-- paths to their id — the join key every aggregation resolves through.
data Tree = Tree
  { tDirs :: !(IM.IntMap Dir)
  , tIds :: !(M.Map String Int)
  }

-- | `CONV_README` / `CONV_CONFIG`: bit 0 = a README.* lives here, bit 1
-- = a recognized config basename lives here.
convReadme, convConfig :: Int
convReadme = 1
convConfig = 2

-- | `CONFIG_NAMES`: the recognized config basenames — the presence fact
-- for the S4 axis, not a policy.
configNames :: [String]
configNames = words "ce.toml Cargo.toml pyproject.toml package.json go.mod Makefile CMakeLists.txt flake.nix"

-- | `build`: every ancestor directory of every file becomes a node; the
-- root is the empty prefix. Files are visited in sorted order.
build :: [String] -> Tree
build paths = foldl visit (Tree (IM.singleton 0 root) (M.singleton "" 0)) (sort paths)
 where
  visit t path =
    let (dirPath, name) = splitDir path
        (t', d) = ensureDir t dirPath
        upper = map asciiUpper name
        readme = upper == "README" || "README." `isPrefixOf` upper
        conv = (if readme then convReadme else 0) .|. (if name `elem` configNames then convConfig else 0)
        bump dir =
          dir
            { dFiles = dFiles dir + 1
            , dShapes = M.insertWith (+) (shapeBits (stem name)) 1 (dShapes dir)
            , dConventions = dConventions dir .|. conv
            }
     in t' {tDirs = IM.adjust bump d (tDirs t')}

-- | `to_ascii_uppercase`: ASCII letters only.
asciiUpper :: Char -> Char
asciiUpper c = if isAsciiLower c then toUpper c else c

-- | `root()`: the root node, its own parent at depth 0.
root :: Dir
root = Dir 0 0 0 0 M.empty 0

-- | `dir_of`: the owning directory id of a FILE path; Nothing = the
-- file's directory never entered this tree.
dirOf :: Tree -> String -> Maybe Int
dirOf t path = M.lookup (fst (splitDir path)) (tIds t)

-- | `dir_id`: the id of a DIRECTORY path (no trailing slash, "" the
-- root), entered with every ancestor when absent.
dirId :: Tree -> String -> (Tree, Int)
dirId = ensureDir

-- | `split_dir`: at the last '/', the directory before it and the name
-- after it; no '/' = the root's file.
splitDir :: String -> (String, String)
splitDir path = case break (== '/') (reverse path) of
  (rname, _ : rdir) -> (reverse rdir, reverse rname)
  (_, []) -> ("", path)

-- | `stem`: up to the first '.'; a leading dot classifies by everything
-- after it (`&name[1..]`, other dots kept).
stem :: String -> String
stem name = case break (== '.') name of
  ([], _ : rest) -> rest
  (before, _) -> before

-- | `ensure_dir`: the dense id of a directory path, creating the chain
-- of ancestors on first sight (each creation increments the parent's
-- subdir count exactly once — the id map is the visited set).
ensureDir :: Tree -> String -> (Tree, Int)
ensureDir t dirPath = case M.lookup dirPath (tIds t) of
  Just d -> (t, d)
  Nothing ->
    let (t1, parent) = ensureDir t (fst (splitDir dirPath))
        d = IM.size (tDirs t1)
        depth = maybe 0 dDepth (IM.lookup parent (tDirs t1)) + 1
        dirs = IM.insert d (Dir parent depth 0 0 M.empty 0) (IM.adjust (\p -> p {dSubdirs = dSubdirs p + 1}) parent (tDirs t1))
     in (Tree dirs (M.insert dirPath d (tIds t1)), d)

-- | Each directory's path by dense id (the root ""), in id order.
dirPaths :: Tree -> [String]
dirPaths t = IM.elems (IM.fromList [(d, p) | (p, d) <- M.toList (tIds t)])

-- | `shape_bits`: the seven stem facts at their frozen positions —
-- 0 an underscore / 1 a dash / 2 an ASCII lowercase letter / 3 an ASCII
-- uppercase letter / 4 digit-led / 5 first char uppercase / 6
-- unclassifiable (an empty stem, or a char outside ASCII letters,
-- digits, dash and underscore). The style decision is CE.Structure.Shape.
shapeBits :: String -> Int
shapeBits s = foldl (.|.) firstBits (map charBits s)
 where
  charBits c
    | c == '_' = 1
    | c == '-' = 2
    | isAsciiLower c = 4
    | isAsciiUpper c = 8
    | isDigit c = 0
    | otherwise = 64
  firstBits = case s of
    [] -> 64
    c : _
      | isDigit c -> 16
      | isAsciiUpper c -> 32
      | otherwise -> 0
