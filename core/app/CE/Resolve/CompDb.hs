{-# LANGUAGE OverloadedStrings #-}

-- | The C-family databases: translation-unit commands and directory-wide
-- flags (cli/src/graph/compdb.rs until plan v2.33 W2-text; each function
-- below is the Rust function of the same name).
-- https://clang.llvm.org/docs/JSONCompilationDatabase.html, Format:
-- `arguments` is argv; `command` uses quoting. File and flag paths belong
-- to `directory`. Placement stays lexical against the root: no
-- canonicalize, no stat, no host working directory. Outside paths cannot
-- hold an in-scope candidate.
-- https://github.com/llvm/llvm-project/blob/main/clang/lib/Tooling/CompilationDatabase.cpp,
-- expandResponseFiles and FixedCompilationDatabase::loadFromBuffer: JSON
-- commands expand response files, fixed databases trim nonempty flag
-- lines. Each trimmed flag line is one verbatim argument; separate
-- operands occupy separate lines, and response-looking words in that
-- form remain literal. Response cycles remain literal; sixteen open
-- files bound a recursive branch. Program spelling selects response
-- syntax (CE.Resolve.Cmdline), never the host OS.
--
-- The measuring side decoded the JSON document (null when it is no
-- array) and reads the response files: one this module names that the
-- request did not carry is `wanted`, and the measuring side reads it and
-- asks again; the set every expansion named is the resolve key's input.
module CE.Resolve.CompDb (
  Entry (..),
  Expanded (..),
  parseDb,
  parseFlags,
  relativize,
  isAbsolute,
) where

import CE.Resolve.Cmdline (splitCommand, splitGnu, splitWindows, windowsShaped)
import CE.Resolve.Flags (Chain, chain)
import CE.Resolve.Str (joinRel, parentDir, replaceChar, rustLines, rustTrim, utf8Len)
import Data.Aeson (Value (..))
import qualified Data.Aeson.Key as K
import qualified Data.Aeson.KeyMap as KM
import Data.Char (isAsciiUpper, ord, toLower)
import Data.Foldable (toList)
import Data.List (stripPrefix)
import qualified Data.Map.Strict as M
import qualified Data.Set as Set

-- | One translation unit, its placed working directory (Nothing outside
-- the tree, "" at its root), and its explicit include chain.
data Entry = Entry {eUnit :: String, eDir :: Maybe String, eChain :: Chain}

-- | What expanding one database read: every response path it named
-- (readable, missing, cyclic or depth-limited), and the ones the request
-- did not carry.
data Expanded = Expanded {xResponses :: Set.Set String, xWanted :: Set.Set String}

instance Semigroup Expanded where
  Expanded a b <> Expanded c d = Expanded (Set.union a c) (Set.union b d)

instance Monoid Expanded where
  mempty = Expanded Set.empty Set.empty

-- | `parse`: one JSON compilation database's rows; a row whose `file`
-- does not place inside the tree is skipped.
parseDb :: String -> M.Map String (Maybe String) -> [Value] -> ([Entry], Expanded)
parseDb base texts = foldl row ([], mempty)
 where
  row (entries, x) r = case str "file" r >>= relativize base dir of
    Nothing -> (entries, x)
    Just unit ->
      let argv = arguments r
          windows = case argv of
            (p : _) -> windowsShaped p
            [] -> False
          (argv', x') = expand base texts dir argv windows []
          place = relativize base dir
       in (entries <> [Entry unit (place "") (chain argv' place)], x <> x')
   where
    dir = maybe "" id (str "directory" r)

-- | A string member of an object row.
str :: String -> Value -> Maybe String
str k (Object o) = case KM.lookup (K.fromString k) o of
  Just (String s) -> Just (K.toString (K.fromText s))
  _ -> Nothing
str _ _ = Nothing

-- | `arguments`: an arguments array is verbatim (its strings), even
-- empty; otherwise tokenize the command.
arguments :: Value -> [String]
arguments r@(Object o) = case KM.lookup "arguments" o of
  Just (Array a) -> [K.toString (K.fromText s) | String s <- toList a]
  _ -> maybe [] splitCommand (str "command" r)
arguments _ = []

-- | `expand`: a branch against the entry's working directory, recording
-- before reading.
expand :: String -> M.Map String (Maybe String) -> String -> [String] -> Bool -> [String] -> ([String], Expanded)
expand base texts dir argv windows stack = foldl step ([], mempty) argv
 where
  step (out, x) arg = case stripPrefix "@" arg >>= relativize base dir of
    Nothing -> (out <> [arg], x)
    Just path
      | path `elem` stack || length stack >= 16 -> (out <> [arg], x <> named path False)
      | otherwise -> case M.lookup path texts of
          Nothing -> (out, x <> named path True)
          Just Nothing -> (out, x <> named path False)
          Just (Just text) ->
            let body = maybe text id (stripPrefix "\xFEFF" text)
                words' = if windows then splitWindows body False else splitGnu body
                (more, x') = expand base texts dir words' windows (stack <> [path])
             in (out <> more, x <> named path False <> x')
  named path wanted = Expanded (Set.singleton path) (if wanted then Set.singleton path else Set.empty)

-- | `parse_flags`: a compile_flags.txt's chain — one argument per
-- trimmed non-empty line after `clang`, anchored to the file's
-- directory, no response expansion.
parseFlags :: String -> String -> String -> (String, Chain)
parseFlags base rel text = (dir, chain argv (relativize base dir))
 where
  dir = parentDir rel
  argv = "clang" : filter (not . null) (map rustTrim (rustLines text))

-- | `relativize`: an absolute or working-directory-relative path placed
-- lexically inside the tree (either separator alike).
relativize :: String -> String -> String -> Maybe String
relativize root dir path
  | isAbsolute p = stripRoot root p >>= joinRel ""
  | otherwise = do
      b <- if isAbsolute d then stripRoot root d else Just d
      b' <- joinRel "" b
      joinRel b' p
 where
  p = replaceChar '\\' '/' path
  d = replaceChar '\\' '/' dir

-- | `is_absolute`: a POSIX path, or a second byte `:` (a drive).
isAbsolute :: String -> Bool
isAbsolute ('/' : _) = True
isAbsolute (c : ':' : _) = ord c < 0x80
isAbsolute _ = False

-- | `strip_root`: a whole root component off an absolute path, without
-- ASCII case only for a drive-lettered root; the prefix is taken in
-- bytes and must end on a character boundary.
stripRoot :: String -> String -> Maybe String
stripRoot root abs' = do
  (hd, rest) <- splitBytes (utf8Len root) abs'
  if same hd
    then if null rest then Just "" else stripPrefix "/" rest
    else Nothing
 where
  drive = case root of
    (c : ':' : _) -> ord c < 0x80
    _ -> False
  same hd = if drive then map lower hd == map lower root else hd == root
  lower c = if isAsciiUpper c then toLower c else c
  splitBytes 0 s = Just ("", s)
  splitBytes n (c : cs)
    | utf8Len [c] <= n = (\(a, b) -> (c : a, b)) <$> splitBytes (n - utf8Len [c]) cs
  splitBytes _ _ = Nothing
