-- | The text form the definition documents are written in (plan v2.32
-- step 1; design booklet docs/reference/authority-track.md §4.2): the
-- subset of TOML the measuring side's own tables were written in before
-- they moved here — bare keys, `key = value` statements, basic ("…",
-- with \" \\ \n \t escapes) and literal ('…') strings, arrays, inline
-- tables, booleans and non-negative integers, `[a.b]` table and
-- `[[a]]` array-of-tables headers — plus `null` for an absent optional.
-- Comments (`#` to the end of the line) and line breaks may sit between
-- any two tokens. A language's whole definition is one document, so
-- thirteen languages of one shape never read as copies of one another
-- under the clone gate (the reason the Rust side wrote its tables so).
-- Not JSON text read by Aeson, though Aeson is already here: a JSON
-- string escapes every quote, so the tables' own quoted kinds and
-- delimiters (`"\""`, `'"'`) would read as escape soup, and comments
-- have nowhere to sit; TOML is already the repository's configuration
-- language (ce.toml), so a reader of one reads the other.
module CE.Lang.Toml (toml) where

import Control.Monad (foldM)
import Data.Aeson (Object, Value (..), toJSON)
import qualified Data.Aeson.Key as K
import qualified Data.Aeson.KeyMap as KM
import Data.Char (isAlphaNum, isDigit, isSpace)
import Data.Foldable (toList)
import Data.List (intercalate)
import Data.Maybe (fromMaybe)

type Parse a = String -> Either String (a, String)

-- | Where a header puts its table: at a dotted path, or appended to the
-- array at that path.
data Header = At [K.Key] | Each [K.Key]

-- | A document as one JSON object: the statements before the first
-- header at the top, then every header's table at its path. A key or a
-- table stated twice, or anything the subset does not spell, is refused
-- with its place.
toml :: String -> Either String Value
toml text = do
  (top, rest) <- statements (skip text)
  foldM place (Object top) =<< tables rest

-- | A run of statements, up to the next header or the end.
statements :: Parse Object
statements = go []
 where
  go seen s
    | null s || take 1 s == "[" = Right (KM.fromList (reverse seen), s)
    | otherwise = do
        (k, v, rest) <- statement s
        if any ((== k) . fst) seen
          then Left ("key `" <> K.toString k <> "` stated twice")
          else go ((k, v) : seen) (skip rest)

tables :: String -> Either String [(Header, Object)]
tables "" = Right []
tables s = do
  (h, r1) <- header s
  (o, r2) <- statements (skip r1)
  ((h, o) :) <$> tables r2

header :: Parse Header
header s = case s of
  '[' : '[' : r -> path r >>= \(ks, r') -> (,) (Each ks) <$> closing "]]" r'
  '[' : r -> path r >>= \(ks, r') -> (,) (At ks) <$> closing "]" r'
  _ -> Left ("a header expected at: " <> take 40 s)
 where
  closing c r = maybe (Left ("`" <> c <> "` expected at: " <> take 40 r)) Right (stripped c r)
  stripped c r = if take (length c) r == c then Just (drop (length c) r) else Nothing

-- | `a.b.c`.
path :: Parse [K.Key]
path s = do
  (k, r) <- key s
  case r of
    '.' : more -> (\(ks, r') -> (k : ks, r')) <$> path more
    _ -> Right ([k], r)

-- | A header's table into the document: a new key at its path, or one
-- more element of the array there.
place :: Value -> (Header, Object) -> Either String Value
place root (h, o) = case h of
  At ks -> at ks (maybe (Right table) (const (twice ks))) root
  Each ks -> at ks (maybe (Right (toJSON [table])) (more ks)) root
 where
  table = Object o
  twice ks = Left ("table `" <> dotted ks <> "` stated twice")
  more _ (Array xs) = Right (toJSON (toList xs <> [table]))
  more ks _ = Left ("`" <> dotted ks <> "` is not an array of tables")

-- | Rewrite the value at a path, making the tables on the way.
at :: [K.Key] -> (Maybe Value -> Either String Value) -> Value -> Either String Value
at ks f (Object m) = case ks of
  [k] -> set k <$> f (KM.lookup k m)
  k : rest -> set k <$> at rest f (fromMaybe (Object KM.empty) (KM.lookup k m))
  [] -> f (Just (Object m))
 where
  set k v = Object (KM.insert k v m)
at ks _ _ = Left ("`" <> dotted ks <> "` lies under a value that is not a table")

dotted :: [K.Key] -> String
dotted = intercalate "." . map K.toString

-- | `key = value`.
statement :: String -> Either String (K.Key, Value, String)
statement s = do
  (k, r1) <- key s
  r2 <- expect '=' (skip r1)
  (v, r3) <- value (skip r2)
  pure (k, v, r3)

-- | Blanks, line breaks and comments.
skip :: String -> String
skip s = case dropWhile isSpace s of
  '#' : rest -> skip (dropWhile (/= '\n') rest)
  rest -> rest

expect :: Char -> String -> Either String String
expect c (x : rest) | x == c = Right rest
expect c s = Left ("`" <> [c] <> "` expected at: " <> take 40 s)

key :: Parse K.Key
key s = case span keyChar s of
  ("", _) -> Left ("a key expected at: " <> take 40 s)
  (k, rest) -> Right (K.fromString k, rest)
 where
  keyChar c = isAlphaNum c || c == '_' || c == '-'

value :: Parse Value
value s = case s of
  '"' : r -> basic "" r
  '\'' : r -> case break (== '\'') r of
    (body, _ : rest) -> Right (toJSON body, rest)
    _ -> Left "an unterminated literal string"
  '[' : r -> array [] (skip r)
  '{' : r -> inline [] (skip r)
  _ -> word s

-- | The bare values: booleans, `null` and integers.
word :: Parse Value
word s = case span isAlphaNum s of
  ("true", r) -> Right (Bool True, r)
  ("false", r) -> Right (Bool False, r)
  ("null", r) -> Right (Null, r)
  (ds@(_ : _), r) | all isDigit ds -> Right (toJSON (read ds :: Integer), r)
  _ -> Left ("a value expected at: " <> take 40 s)

basic :: String -> Parse Value
basic acc s = case s of
  '"' : r -> Right (toJSON (reverse acc), r)
  '\\' : c : r | Just e <- lookup c escapes -> basic (e : acc) r
  c : r | c /= '\\' && c /= '\n' -> basic (c : acc) r
  _ -> Left ("a bad or unterminated basic string at: " <> take 40 s)
 where
  escapes = [('"', '"'), ('\\', '\\'), ('n', '\n'), ('t', '\t')]

array :: [Value] -> Parse Value
array acc (']' : r) = Right (toJSON (reverse acc), r)
array acc s = do
  (v, r) <- value s
  case skip r of
    ',' : more -> array (v : acc) (skip more)
    ']' : more -> Right (toJSON (reverse (v : acc)), more)
    other -> Left ("`,` or `]` expected at: " <> take 40 other)

inline :: [(K.Key, Value)] -> Parse Value
inline acc ('}' : r) = Right (Object (KM.fromList (reverse acc)), r)
inline acc s = do
  (k, v, r) <- statement s
  case skip r of
    ',' : more -> inline ((k, v) : acc) (skip more)
    '}' : more -> Right (Object (KM.fromList (reverse ((k, v) : acc))), more)
    other -> Left ("`,` or `}` expected at: " <> take 40 other)
