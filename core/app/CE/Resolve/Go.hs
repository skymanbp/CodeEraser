-- | Go rungs (design §4 row 4; moved from cli/src/graph/ladder/go.rs in
-- plan v2.33 wave W2a, on text since W2-text, with the go.mod reader of
-- cli/src/graph/gomod.rs — each function below is the Rust function of
-- the same name). Go resolves at PACKAGE granularity: an import path
-- names a directory, the node identity is (pkg_dir, ""), and collapsing
-- to one file would be a guess — so a directory only counts as an
-- importable package while it directly holds an in-scope non-_test.go
-- file (a test-only directory is not a target).
--   R1 the module-prefix walk: the LONGEST in-scope go.mod module path
--      owning the spec wins — longest is how nested modules own their
--      subtrees; two modules declaring one path is ambiguous_workspace.
--   R2 the importer's nearest module's replace directives (the longest
--      old path wins, the first of equals): a filesystem target (./ or
--      ../) maps into the corpus, a module target is rewritten and
--      retried against the module set.
--   R3 External: the stdlib table (machine-generated, importable set
--      only), or a dotted first segment matching no local module — heads
--      without a dot are reserved for the standard library, so a dotless
--      head outside the table is out_of_scope, never External.
-- //go:build-constrained files still count toward package existence.
-- Lengths are UTF-8 byte lengths, as `str::len` measures them.
module CE.Resolve.Go (GoMod (..), parseGoMod, resolveGo) where

import CE.Resolve.Answer
import CE.Resolve.Cost (Reason (..))
import CE.Resolve.Str
import CE.Resolve.Tables (goStd)
import CE.Resolve.World
import Data.List (isPrefixOf, stripPrefix)
import qualified Data.Set as Set

-- | `gomod::GoMod`: the go.mod's directory, its module path, its replace
-- directives `(old, new)` in file order.
data GoMod = GoMod {gmDir :: String, gmModule :: Maybe String, gmReplaces :: [(String, String)]}

-- | `gomod::parse` on a go.mod's text: `//` comments cut, each line
-- trimmed; `module x` (quotes trimmed), `replace (` … `)` blocks and
-- single `replace` lines.
parseGoMod :: String -> String -> GoMod
parseGoMod rel text = GoMod (parentDir rel) modul (reverse reps)
 where
  (modul, reps, _) = foldl step (Nothing, [], False) (rustLines text)
  step (m, rs, inBlock) raw
    | inBlock = if line == ")" then (m, rs, False) else (m, push line rs, True)
    | Just rest <- stripPrefix "module " line = (Just (trimQuotes (rustTrim rest)), rs, False)
    | line == "replace (" = (m, rs, True)
    | Just rest <- stripPrefix "replace " line = (m, push rest rs, False)
    | otherwise = (m, rs, False)
   where
    line = rustTrim (takeBefore "//" raw)
  trimQuotes = reverse . dropWhile (== '"') . reverse . dropWhile (== '"')

-- | `split("//").next()`: the text before the first separator.
takeBefore :: String -> String -> String
takeBefore sep raw = case splitOnceStr sep raw of
  Just (before, _) -> before
  Nothing -> raw

-- | `push_replace`: "old [version] => new [version]", versions dropped.
push :: String -> [(String, String)] -> [(String, String)]
push entry rs = case splitOnceStr "=>" entry of
  Just (lhs, rhs) | (old : _) <- splitWhitespace lhs, (new : _) <- splitWhitespace rhs -> (old, new) : rs
  _ -> rs

-- | `resolve`: the modules (the go.mods that declare one, in the
-- configs' order), then the three rungs.
resolveGo :: World -> [GoMod] -> String -> String -> Answer
resolveGo w mods from spec =
  firstOf [moduleRung w mods spec, replaceRung w mods from spec] (externalRung spec)

-- | R1: the longest module prefix owns the import; the remainder names
-- a package directory under the module's own directory.
moduleRung :: World -> [GoMod] -> String -> Maybe Answer
moduleRung w mods spec = case Set.toList dirs of
  [] -> Nothing
  [d] -> Just (package w d 1)
  _ -> Just (AUnresolved AmbiguousWorkspace)
 where
  (_, dirs) = foldl step (0, Set.empty) mods
  step (bestLen, ds) m = case gmModule m >>= \modul -> (,) modul <$> stripModule spec modul of
    Nothing -> (bestLen, ds)
    Just (modul, rest)
      | utf8Len modul > bestLen -> (utf8Len modul, Set.singleton (dirOf m rest))
      | utf8Len modul == bestLen -> (bestLen, Set.insert (dirOf m rest) ds)
      | otherwise -> (bestLen, ds)
  dirOf m rest = if null rest then gmDir m else joinDir (gmDir m) rest

-- | `strip_module`: the remainder of a spec its module path owns.
stripModule :: String -> String -> Maybe String
stripModule spec modul = case stripPrefix modul spec of
  Just "" -> Just ""
  Just rest -> stripPrefix "/" rest
  Nothing -> Nothing

-- | R2: the importer's module's replace directives, the shortest
-- remainder (the longest old) wins, the first of equals. A filesystem
-- target that climbs out of the tree answers nothing here.
replaceRung :: World -> [GoMod] -> String -> String -> Maybe Answer
replaceRung w mods from spec = do
  m <- owner from mods
  (rest, new) <- firstMin [(rest, new) | (old, new) <- gmReplaces m, Just rest <- [stripModule spec old]]
  if "./" `isPrefixOf` new || "../" `isPrefixOf` new
    then do
      base <- joinRel (gmDir m) new
      Just (package w (if null rest then base else joinDir base rest) 2)
    else
      let rewritten = if null rest then new else new <> "/" <> rest
       in Just (maybe (externalRung rewritten) (withRung 2) (moduleRung w mods rewritten))
 where
  firstMin [] = Nothing
  firstMin (x : xs) = Just (foldl (\a b -> if utf8Len (fst b) < utf8Len (fst a) then b else a) x xs)

-- | `owner`: the module whose directory holds the importer, the deepest
-- (the last of equal byte lengths, as `max_by_key` takes it).
owner :: String -> [GoMod] -> Maybe GoMod
owner from mods = case [m | m <- mods, null (gmDir m) || (gmDir m <> "/") `isPrefixOf` from] of
  [] -> Nothing
  (x : xs) -> Just (foldl (\a b -> if utf8Len (gmDir b) >= utf8Len (gmDir a) then b else a) x xs)

-- | A directory is an importable package while it directly holds an
-- in-scope non-test .go file.
package :: World -> String -> Int -> Answer
package w dir rung
  | Set.member dir (wGoDirs w) = APackage dir rung
  | otherwise = AUnresolved OutOfScope

-- | R3: the stdlib table, or a dotted head with no local match.
externalRung :: String -> Answer
externalRung spec
  | '.' `elem` takeWhile (/= '/') spec || Set.member spec goStd = AExternal 3
  | otherwise = AUnresolved OutOfScope
