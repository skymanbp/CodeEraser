-- | Lua rungs (plan v2.30 step 4; design booklet §8 row Lua; moved from
-- cli/src/graph/ladder/lua.rs in plan v2.33 wave W2a). A Lua module is
-- whatever `package.searchers` finds (Lua 5.4 manual §6.3): `require`
-- first answers a name `package.loaded` already holds — the standard
-- libraries, LuaJIT's built-ins — and otherwise asks each template of
-- `package.path` in turn, `?` standing for the name with its dots
-- turned into slashes.
--   R1 `require "a.b"`: `a/b` in every search directory — the tree
--      root, `src` and `lua` (the luarocks and Neovim layouts), the
--      declared `[graph.search_roots] lua` and the requiring file's own
--      directory, each tried as `a/b.lua` then `a/b/init.lua` (the
--      standard order), and every template the tree's own files assign
--      to `package.path` as it is written. A directory is one entry
--      whoever names it, and `.lua` goes first wherever it is tried.
--      Two directories answering two different files is
--      ambiguous_root: which one a run tries first is the order its
--      path was built in, no fact of the text;
--   R2 `dofile` / `loadfile` (label `load`): the path beside the
--      loading file, then under the tree root; the first hit;
--   R3 External: a standard library or built-in name — it never
--      reaches the searchers, whatever files the tree holds.
-- Anything else is out_of_scope. A `require` name arrives as its pieces
-- with the separator between each two (`.` or `/`); a `load` path as
-- its slash pieces and whether it is rooted.
module CE.Resolve.Lua (resolveLua) where

import CE.Resolve.Answer
import CE.Resolve.Cost
import CE.Resolve.Request (LuaFacts (..))
import CE.Resolve.Vocab (Affix (..), Word' (..), kindLoad, kindRequire, luaStdlib)
import CE.Resolve.World
import Data.List (intercalate, sortOn)
import Data.Maybe (listToMaybe)

-- | A suffix a directory is tried with: the piece glued onto the name's
-- last piece (empty = none) and the pieces after it.
type Suffix = (Int, [Int])

-- | One search directory: its text as pieces (the key) and its suffixes.
type Entry = ([Int], [Suffix])

-- | One site: kind, file, form, tokens.
resolveLua :: World -> LuaFacts -> Integer -> Int -> Integer -> [Integer] -> Answer
resolveLua w facts kind from form toks
  | kind == kindRequire && stdlibName = AExternal 3
  | kind == kindRequire = moduleRung w facts from pieces
  | kind == kindLoad = loaded w from (form == formRooted) pieces
  | otherwise = AUnresolved Unsupported
 where
  pieces = [fromInteger t | t <- toks, t >= 0]
  -- the measuring side's exact text match: the name's tokens are an
  -- entry's pieces with a dot between each two
  stdlibName = toks `elem` [intercalate [sepDot] (map (pure . toInteger) e) | e <- map (spelled w) luaStdlib]

-- | R1: the module's path in every search directory the requiring file
-- has; one distinct file resolves, two refuse. A name with an empty
-- piece (`a..b`, a leading or trailing dot or slash) names no file.
moduleRung :: World -> LuaFacts -> Int -> [Int] -> Answer
moduleRung w facts from name
  | any (isWord w WEmpty) name = AUnresolved OutOfScope
  | otherwise =
      maybe (AUnresolved OutOfScope) id (oneOf (hits (map (candidate w name) dirs)) 1)
 where
  own = case parentOf w from of
    [] -> [word w WEmpty]
    dir -> dir
  dirs = add w (searched w facts) (own, standard w)

-- | The first suffix of one directory that names a walked file.
candidate :: World -> [Int] -> Entry -> Maybe Int
candidate w name (key, suffixes) =
  listToMaybe [f | s <- suffixes, Just f <- [inScope w key (spellAt s)]]
 where
  spellAt (h, tl) = map K (init name) <> [glued h] <> map K tl
  glued h
    | isWord w WEmpty h = K (last name)
    | otherwise = glue w h (K (last name))

-- | The two suffixes the standard path tries a directory with, in order.
standard :: World -> [Suffix]
standard w = [(affix w ALua, []), (word w WEmpty, [word w WInitLua])]

-- | The tree's search directories: the defaults and the declared roots
-- take the standard two, a template adds its own.
searched :: World -> LuaFacts -> [Entry]
searched w facts = foldl (add w) [] (defaults <> templates)
 where
  defaults =
    [ (key, standard w)
    | key <- [[word w WEmpty], [word w WSrc], [word w WLua]] <> map ints (luaRoots facts)
    ]
  templates = [(ints (take (fromInteger n) rest), [(fromInteger h, ints (drop (fromInteger n) rest))]) | (h : n : rest) <- luaTemplates facts]
  ints = map fromInteger

-- | One directory's suffixes merged in, each once; `.lua` first.
add :: World -> [Entry] -> Entry -> [Entry]
add w entries (key, new) = case break ((== key) . fst) entries of
  (before, (_, tried) : after) -> before <> [(key, merged tried)] <> after
  _ -> entries <> [(key, merged [])]
 where
  merged tried = sortOn (not . isLua) (foldl push tried new)
  push acc s = if s `elem` acc then acc else acc <> [s]
  isLua (h, tl) = null tl && h == affix w ALua

-- | R2: the path beside the loading file, then under the tree root; the
-- first hit.
loaded :: World -> Int -> Bool -> [Int] -> Answer
loaded w from rooted path
  | rooted = AUnresolved OutOfScope
  | otherwise = case [f | dir <- [parentOf w from, []], Just f <- [inScope w dir (map K path)]] of
      (f : _) -> AFile f 2
      [] -> AUnresolved OutOfScope
