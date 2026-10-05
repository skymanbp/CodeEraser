-- | The JSONC reader of the TS rungs (moved from cli/src/graph/jsonc.rs
-- and roots.rs `read_jsonc` in plan v2.33 W2-text stage E): tsconfig and
-- package.json in the wild carry comments and trailing commas, so
-- `clean` strips both (string-aware, char by char), and what is left is
-- read as JSON with exactly the measuring side's acceptance — serde_json
-- 1.0.151's `from_str` into a `Value` — since a document one side reads
-- and the other refuses would turn a tsconfig chain into config_depth on
-- one side only:
--   * whitespace is space, `\n`, `\t`, `\r`, nothing else, and nothing
--     but whitespace may follow the value;
--   * a string holds no raw character below U+0020; its escapes are the
--     eight one-letter ones and `\u` with four hex digits, a surrogate
--     only as a leading one followed at once by `\u` and a trailing one;
--   * a number is `-? (0 | [1-9][0-9]*) (. [0-9]+)? ([eE] [+-]? [0-9]+)?`
--     whose value rounds to a finite double (with `float_roundtrip`, the
--     rounding is exact: a magnitude of 2^1024 - 2^970 or more is
--     refused);
--   * arrays and objects nest at most 127 deep (serde's recursion limit
--     of 128 counts the opening bracket that reaches it);
--   * an object's repeated key keeps its last value (a `BTreeMap`
--     insert).
-- Numbers are read for their acceptance only: no rung reads one.
module CE.Resolve.Json (clean, readJsonc, parseJson) where

import CE.Resolve.Chars (isRustWhite)
import Data.Aeson (Value (..), toJSON)
import qualified Data.Aeson.Key as K
import qualified Data.Aeson.KeyMap as KM
import Data.Char (chr, digitToInt, isDigit, isHexDigit)
import Data.Ratio ((%))

-- | `read_jsonc` on a text the measuring side read: cleaned, then read.
readJsonc :: String -> Maybe Value
readJsonc = parseJson . clean

-- | `jsonc::clean`: comments out, then trailing commas out.
clean :: String -> String
clean = stripTrailingCommas . stripComments

-- | `strip_comments`: `//` and `/* */` outside string literals; a line
-- comment keeps its newline, an unclosed block runs to the end.
stripComments :: String -> String
stripComments s = case s of
  [] -> []
  '"' : rest -> '"' : copyString rest
  '/' : '/' : rest -> skipLine rest
  '/' : '*' : rest -> skipBlock ' ' rest
  c : rest -> c : stripComments rest
 where
  copyString t = case t of
    [] -> []
    '\\' : c : rest -> '\\' : c : copyString rest
    '\\' : [] -> "\\"
    '"' : rest -> '"' : stripComments rest
    c : rest -> c : copyString rest
  skipLine t = case break (== '\n') t of
    (_, '\n' : rest) -> '\n' : stripComments rest
    _ -> []
  skipBlock prev t = case t of
    [] -> []
    '/' : rest | prev == '*' -> stripComments rest
    c : rest -> skipBlock c rest

-- | `strip_trailing_commas`: a comma outside a string whose next
-- non-White_Space character closes a scope goes.
stripTrailingCommas :: String -> String
stripTrailingCommas = go False False
 where
  go _ _ [] = []
  go inStr escaped (c : rest)
    | inStr = c : go (escaped || c /= '"') (not escaped && c == '\\') rest
    | c == '"' = c : go True escaped rest
    | c == ',' && closes rest = go False escaped rest
    | otherwise = c : go False escaped rest
  closes rest = case dropWhile isRustWhite rest of
    (n : _) -> n == '}' || n == ']'
    [] -> False

-- | `serde_json::from_str::<Value>`: one value, whitespace around it.
parseJson :: String -> Maybe Value
parseJson s = case value 128 (white s) of
  Just (v, rest) | null (white rest) -> Just v
  _ -> Nothing

white :: String -> String
white = dropWhile (`elem` (" \n\t\r" :: String))

type P a = Maybe (a, String)

-- | One value at the remaining depth `d`.
value :: Int -> String -> P Value
value d s = case s of
  'n' : 'u' : 'l' : 'l' : rest -> Just (Null, rest)
  't' : 'r' : 'u' : 'e' : rest -> Just (Bool True, rest)
  'f' : 'a' : 'l' : 's' : 'e' : rest -> Just (Bool False, rest)
  '"' : rest -> (\(t, r) -> (toJSON t, r)) <$> string rest
  '[' : rest | d > 1 -> array (d - 1) (white rest)
  '{' : rest | d > 1 -> object (d - 1) (white rest)
  '-' : rest -> number rest
  c : _ | isDigit c -> number s
  _ -> Nothing

array :: Int -> String -> P Value
array d s = case s of
  ']' : rest -> Just (toJSON ([] :: [Value]), rest)
  _ -> go [] s
 where
  go acc t = do
    (v, rest) <- value d t
    case white rest of
      ',' : more -> go (v : acc) (white more)
      ']' : more -> Just (toJSON (reverse (v : acc)), more)
      _ -> Nothing

object :: Int -> String -> P Value
object d s = case s of
  '}' : rest -> Just (Object KM.empty, rest)
  _ -> go [] s
 where
  -- KM.fromList keeps the last of a repeated key
  done acc = Object (KM.fromList (reverse acc))
  go acc t = do
    (k, rest) <- case t of
      '"' : r -> string r
      _ -> Nothing
    (v, rest') <- case white rest of
      ':' : r -> value d (white r)
      _ -> Nothing
    let acc' = (K.fromString k, v) : acc
    case white rest' of
      ',' : more -> go acc' (white more)
      '}' : more -> Just (done acc', more)
      _ -> Nothing

-- | The text of a string literal, its opening quote read.
string :: String -> P String
string = go []
 where
  go acc t = case t of
    '"' : rest -> Just (reverse acc, rest)
    '\\' : e : rest -> escape acc e rest
    c : rest | c >= ' ' -> go (c : acc) rest
    _ -> Nothing
  escape acc e rest = case lookup e simple of
    Just c -> go (c : acc) rest
    Nothing | e == 'u' -> do
      (n, rest') <- hex4 rest
      unicode acc n rest'
    Nothing -> Nothing
  simple = zip "\"\\/bfnrt" "\"\\/\b\f\n\r\t"
  unicode acc n rest
    | n >= 0xDC00 && n <= 0xDFFF = Nothing
    | n < 0xD800 || n > 0xDBFF = go (chr n : acc) rest
    | otherwise = case rest of
        '\\' : 'u' : more -> do
          (n2, rest') <- hex4 more
          if n2 < 0xDC00 || n2 > 0xDFFF
            then Nothing
            else go (chr (0x10000 + (n - 0xD800) * 0x400 + (n2 - 0xDC00)) : acc) rest'
        _ -> Nothing

hex4 :: String -> P Int
hex4 t = case splitAt 4 t of
  (h, rest) | length h == 4 && all isHexDigit h -> Just (foldl (\a c -> a * 16 + digitToInt c) 0 h, rest)
  _ -> Nothing

-- | A number after its sign: its grammar, then the finiteness of the
-- double it rounds to. Its value is not kept.
number :: String -> P Value
number s = do
  (int, r1) <- integer s
  (frac, r2) <- fraction r1
  (ex, r3) <- power r2
  let digits = int <> frac
      coef = read ('0' : digits) :: Integer
  if finite coef (ex - toInteger (length frac)) then Just (Number 0, r3) else Nothing
 where
  integer t = case t of
    '0' : c : _ | isDigit c -> Nothing
    '0' : rest -> Just ("0", rest)
    _ -> case span isDigit t of
      ("", _) -> Nothing
      (ds, rest) -> Just (ds, rest)
  fraction t = case t of
    '.' : rest -> case span isDigit rest of
      ("", _) -> Nothing
      (ds, more) -> Just (ds, more)
    _ -> Just ("", t)
  power t = case t of
    e : rest | e == 'e' || e == 'E' -> case rest of
      '+' : more -> digitsOf 1 more
      '-' : more -> digitsOf (-1) more
      _ -> digitsOf 1 rest
    _ -> Just (0, t)
  digitsOf sign t = case span isDigit t of
    ("", _) -> Nothing
    (ds, more) -> Just (sign * read ds, more)

-- | Whether coef × 10^e rounds (to nearest, ties to even) to a finite
-- double: a magnitude below 2^1024 - 2^970, the midpoint between the
-- largest double and 2^1024 (which rounds to even, upward).
finite :: Integer -> Integer -> Bool
finite coef e
  | coef == 0 = True
  | width + e > 310 = False
  | width + e < -330 = True
  | e >= 0 = coef * 10 ^ e < limit
  | otherwise = coef % (10 ^ negate e) < fromInteger limit
 where
  width = toInteger (length (show coef))
  limit = 2 ^ (1024 :: Int) - 2 ^ (970 :: Int)
