-- | Lua rungs (plan v2.30 step 4; design booklet §8 row Lua; moved from
-- cli/src/graph/ladder/lua.rs in plan v2.33 wave W2a, on text since
-- W2-text — each function below is the Rust function of the same name).
-- A Lua module is whatever `package.searchers` finds (Lua 5.4 manual
-- §6.3): `require` first answers a name `package.loaded` already holds —
-- the standard libraries, LuaJIT's built-ins — and otherwise asks each
-- template of `package.path` in turn, `?` standing for the name with its
-- dots turned into slashes.
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
-- Anything else is out_of_scope.
module CE.Resolve.Lua (resolveLua, searched) where

import CE.Resolve.Answer
import CE.Resolve.Cost (Reason (..))
import CE.Resolve.Str
import CE.Resolve.Tables (kindLoad, kindRequire, luaStdlib)
import CE.Resolve.World
import Data.List (sortOn)
import qualified Data.Map.Strict as M
import qualified Data.Set as Set

-- | Each search directory (by its text, the map's order) with the
-- suffixes it is tried with, in order.
type Dirs = M.Map String [String]

-- | `STANDARD`: the two suffixes the standard path tries, in order.
standard :: [String]
standard = [".lua", "/init.lua"]

-- | `resolve`: one site by its kind; the tree's directories are
-- `searched` once per request.
resolveLua :: World -> Dirs -> Integer -> String -> String -> Answer
resolveLua w dirs kind from spec
  | kind == kindRequire && Set.member spec luaStdlib = AExternal 3
  | kind == kindRequire = modul w dirs from spec
  | kind == kindLoad = loaded w from spec
  | otherwise = AUnresolved Unsupported

-- | R1: the module's path in every search directory the requiring file
-- has; one distinct file resolves, two refuse. A name with an empty
-- piece names no file.
modul :: World -> Dirs -> String -> String -> Answer
modul w searchedDirs from spec
  | any null (splitOn '/' name) = AUnresolved OutOfScope
  | otherwise = maybe (AUnresolved OutOfScope) id (oneOf hits 1)
 where
  name = replaceChar '.' '/' spec
  dirs = add (parentDir from) standard searchedDirs
  hits = Set.fromList [p | (dir, suffixes) <- M.toList dirs, Just p <- [firstIn dir suffixes]]
  firstIn dir suffixes = case [p | s <- suffixes, Just p <- [joinRel dir (name <> s)], member w p] of
    (p : _) -> Just p
    [] -> Nothing

-- | `searched`: the defaults and the declared roots take the standard
-- two, then each template (in the walk's set order) adds its suffix to
-- its directory.
searched :: [String] -> [(String, String)] -> Dirs
searched declared templates = foldl (\m (d, s) -> add d [s] m) defaults templates
 where
  defaults = foldl (\m d -> add d standard m) M.empty (["", "src", "lua"] <> declared)

-- | `add`: a directory's suffixes merged in, each once, then a stable
-- sort that puts `.lua` first.
add :: String -> [String] -> Dirs -> Dirs
add dir suffixes dirs = M.insert dir (sortOn (/= ".lua") (foldl push tried suffixes)) dirs
 where
  tried = M.findWithDefault [] dir dirs
  push acc s = if s `elem` acc then acc else acc <> [s]

-- | R2: `besideOrRoot` (CE.Resolve.World) — beside the loading file,
-- then under the tree root; the first hit.
loaded :: World -> String -> String -> Answer
loaded w from spec = maybe (AUnresolved OutOfScope) (`AFile` 2) (besideOrRoot w from spec)
