-- | Two readings of a link target the Markdown rungs make before any
-- path is joined (plan v2.33 W2-text stage G; once
-- cli/src/graph/ladder/md.rs `is_scheme` and md_slug.rs
-- `percent_decode`, which stay on the measuring side for the HTML rungs
-- until stage H moves them here): whether the target leaves the corpus
-- by a URI scheme, and the target percent-decoded.
module CE.Resolve.Url (isScheme, percentDecode) where

import Data.Char (chr, isAsciiLower, isAsciiUpper, isDigit, ord)
import Data.Maybe (fromMaybe)

-- | `is_scheme`: an RFC 3986 scheme head (an ASCII letter, then ASCII
-- letters, digits, `+`, `-` or `.`, then `:`, before any `/` or `#`),
-- or a protocol-relative `//` form — a join would normalize "//host/x"
-- into an in-tree path.
isScheme :: String -> Bool
isScheme spec
  | take 2 spec == "//" = True
  | otherwise = case break (== ':') (takeWhile (`notElem` ("/#" :: String)) spec) of
      (first : rest, ':' : _) -> letter first && all schemeChar rest
      _ -> False
 where
  letter c = isAsciiUpper c || isAsciiLower c
  schemeChar c = letter c || isDigit c || c `elem` ("+-." :: String)

-- | `percent_decode`: every `%` with two hexadecimal digits after it
-- (in either case; `u8::from_str_radix` also reads `%+f` as 0x0f) is
-- that byte, any other `%` stays as written, and a result that is not
-- UTF-8 leaves the whole text untouched — never a guess.
percentDecode :: String -> String
percentDecode s
  | '%' `notElem` s = s
  | otherwise = fromMaybe s (fromUtf8 (go (concatMap utf8 s)))
 where
  go (0x25 : a : b : rest) | Just v <- byte a b = v : go rest
  go (x : rest) = x : go rest
  go [] = []
  byte a b = case (digit a, digit b) of
    (Just x, Just y) -> Just (x * 16 + y)
    _ | a == 0x2B -> digit b
    _ -> Nothing
  digit x
    | x >= 0x30 && x <= 0x39 = Just (x - 0x30)
    | x >= 0x41 && x <= 0x46 = Just (x - 0x37)
    | x >= 0x61 && x <= 0x66 = Just (x - 0x57)
    | otherwise = Nothing

-- | A character's UTF-8 bytes.
utf8 :: Char -> [Int]
utf8 c
  | o < 0x80 = [o]
  | o < 0x800 = [0xC0 + o `div` 64, low o]
  | o < 0x10000 = [0xE0 + o `div` 4096, low (o `div` 64), low o]
  | otherwise = [0xF0 + o `div` 262144, low (o `div` 4096), low (o `div` 64), low o]
 where
  o = ord c
  low x = 0x80 + x `mod` 64

-- | `String::from_utf8`: the bytes as text when they are well-formed
-- UTF-8 (the Unicode standard's table 3-7: no overlong form, no
-- surrogate, nothing past U+10FFFF), else none.
fromUtf8 :: [Int] -> Maybe String
fromUtf8 [] = Just ""
fromUtf8 (b : bs)
  | b < 0x80 = (chr b :) <$> fromUtf8 bs
  | Just (lo, hi, n, lead) <- shape b
  , (c : cs) <- bs
  , lo <= c && c <= hi
  , (more, rest) <- splitAt (n - 1) cs
  , length more == n - 1 && all (\x -> x >= 0x80 && x <= 0xBF) more =
      (chr (foldl (\acc x -> acc * 64 + x - 0x80) (lead * 64 + c - 0x80) more) :) <$> fromUtf8 rest
  | otherwise = Nothing

-- | A lead byte's second-byte range, its continuation count and the
-- bits it carries.
shape :: Int -> Maybe (Int, Int, Int, Int)
shape b
  | b >= 0xC2 && b <= 0xDF = Just (0x80, 0xBF, 1, b - 0xC0)
  | b == 0xE0 = Just (0xA0, 0xBF, 2, 0)
  | b == 0xED = Just (0x80, 0x9F, 2, 0xD)
  | b >= 0xE1 && b <= 0xEF = Just (0x80, 0xBF, 2, b - 0xE0)
  | b == 0xF0 = Just (0x90, 0xBF, 3, 0)
  | b >= 0xF1 && b <= 0xF3 = Just (0x80, 0xBF, 3, b - 0xF0)
  | b == 0xF4 = Just (0x80, 0x8F, 3, 4)
  | otherwise = Nothing
