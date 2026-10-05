-- | The string steps the resolve family's readers share (plan v2.33
-- W2-text), each spelled after the measuring side's function it
-- replaces so a reader can hold the two side by side:
--   * the path joins of cli/src/graph/roots.rs (`join_rel`,
--     `parent_dir`, `join_dir`, `ancestors`) — paths are repo-relative
--     strings with `/` separators, as the walk produced them;
--   * Rust's `str` methods the configuration readers called (`split`
--     keeping empty pieces, `lines`, `trim`, `split_whitespace`,
--     `split_once`, `strip_suffix`, byte lengths). Strings compare and
--     order by code point, which is the order of their UTF-8 bytes, so
--     a Rust `BTreeSet<String>` and a `Data.Set String` agree.
module CE.Resolve.Str (
  joinRel,
  parentDir,
  joinDir,
  ancestors,
  baseName,
  extension,
  splitOn,
  splitOnStr,
  splitOnce,
  splitOnceStr,
  stripSuffix,
  rustLines,
  rustTrim,
  splitWhitespace,
  utf8Len,
  replaceChar,
  rooted,
) where

import CE.Resolve.Chars (isRustWhite)
import Data.Char (ord)
import Data.List (dropWhileEnd, isPrefixOf, isSuffixOf, stripPrefix)

-- | `roots::join_rel`: the directory's non-empty pieces, then each
-- piece of the spec — an empty piece and `.` stay put, `..` climbs and
-- climbing above the root is no path.
joinRel :: String -> String -> Maybe String
joinRel dir spec = joined <$> foldl step (Just (reverse (filter (not . null) (splitOn '/' dir)))) (splitOn '/' spec)
 where
  step acc seg = acc >>= move seg
  move seg parts = case seg of
    "" -> Just parts
    "." -> Just parts
    ".." -> case parts of
      [] -> Nothing
      (_ : rest) -> Just rest
    s -> Just (s : parts)
  joined = joinWith . reverse
  joinWith [] = ""
  joinWith (p : ps) = p <> concatMap ('/' :) ps

-- | `roots::parent_dir`: the text before the last `/`, "" when none.
parentDir :: String -> String
parentDir p = case break (== '/') (reverse p) of
  (_, _ : before) -> reverse before
  _ -> ""

-- | `roots::join_dir`: a name under a directory, the root adding no `/`.
joinDir :: String -> String -> String
joinDir "" name = name
joinDir dir name = dir <> "/" <> name

-- | `roots::ancestors`: a directory, then each parent up to "".
ancestors :: String -> [String]
ancestors "" = [""]
ancestors d = d : ancestors (parentDir d)

-- | The text after the last `/` (`rsplit('/').next()`).
baseName :: String -> String
baseName = reverse . takeWhile (/= '/') . reverse

-- | `Path::extension` of a path: after its basename's last `.`, with a
-- non-empty stem before it; `..` has none.
extension :: String -> Maybe String
extension path
  | base == ".." = Nothing
  | otherwise = case break (== '.') (reverse base) of
      (ext, '.' : stem) | not (null stem) -> Just (reverse ext)
      _ -> Nothing
 where
  base = baseName path

-- | `str::split(char)`: every piece, empty ones kept ("" is one piece).
splitOn :: Char -> String -> [String]
splitOn c s = case break (== c) s of
  (a, _ : rest) -> a : splitOn c rest
  (a, []) -> [a]

-- | `str::split(&str)` for a non-empty separator.
splitOnStr :: String -> String -> [String]
splitOnStr sep = go ""
 where
  go acc [] = [reverse acc]
  go acc s@(x : xs) = case stripPrefix sep s of
    Just rest -> reverse acc : go "" rest
    Nothing -> go (x : acc) xs

-- | `str::split_once(char)`.
splitOnce :: Char -> String -> Maybe (String, String)
splitOnce c s = case break (== c) s of
  (a, _ : rest) -> Just (a, rest)
  _ -> Nothing

-- | `str::split_once(&str)` for a non-empty separator.
splitOnceStr :: String -> String -> Maybe (String, String)
splitOnceStr sep = go ""
 where
  go _ [] = Nothing
  go acc s@(x : xs) = case stripPrefix sep s of
    Just rest -> Just (reverse acc, rest)
    Nothing -> go (x : acc) xs

-- | `str::strip_suffix(&str)`.
stripSuffix :: String -> String -> Maybe String
stripSuffix suf s
  | suf `isSuffixOf` s = Just (take (length s - length suf) s)
  | otherwise = Nothing

-- | `str::lines` (Rust 1.94): split at `\n`, a `\r` before that `\n`
-- dropped; a final line ending adds no empty line; a bare `\r` ending
-- the last line stays.
rustLines :: String -> [String]
rustLines "" = []
rustLines s = case break (== '\n') s of
  (a, _ : rest) -> dropCr a : rustLines rest
  (a, []) -> [a]
 where
  dropCr a = if "\r" `isSuffixOf` a then init a else a

-- | `str::trim`: White_Space off both ends.
rustTrim :: String -> String
rustTrim = dropWhileEnd isRustWhite . dropWhile isRustWhite

-- | `str::split_whitespace`: the non-empty White_Space-separated words.
splitWhitespace :: String -> [String]
splitWhitespace s = case dropWhile isRustWhite s of
  "" -> []
  t -> let (w, rest) = break isRustWhite t in w : splitWhitespace rest

-- | A string's length in UTF-8 bytes (`str::len`).
utf8Len :: String -> Int
utf8Len = sum . map width
 where
  width c
    | ord c < 0x80 = 1
    | ord c < 0x800 = 2
    | ord c < 0x10000 = 3
    | otherwise = 4

-- | `str::replace(char, char)`.
replaceChar :: Char -> Char -> String -> String
replaceChar a b = map (\c -> if c == a then b else c)

-- | A path that names no file of the tree when a script loads it: rooted
-- at `/` or `~`, or holding a drive or a colon (CE.Resolve.World
-- `besideOrRoot`).
rooted :: String -> Bool
rooted spec = "/" `isPrefixOf` spec || "~" `isPrefixOf` spec || ':' `elem` spec
