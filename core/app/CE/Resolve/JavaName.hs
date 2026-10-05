-- | A Java specifier without its type annotations (plan v2.33 W2-text
-- stage C; cli/src/graph/ladder/java_pick.rs `unannotated` and the
-- annotation half of java_header.rs's lexer, each function below named
-- after the Rust one it replaces): `a.b.@Tag C` names `a.b.C` (JLS
-- 9.7.4) — each `@` with its dotted name and argument list, the
-- arguments read with strings, text blocks, char literals and comments
-- opaque. The lexer is a function from the unread text to the text
-- after the token; a step that fails leaves the trivia before it read,
-- as the Rust lexer's `&mut self` did.
module CE.Resolve.JavaName (unannotated) where

import CE.Resolve.Chars (isRustAlnum, isRustWhite)
import CE.Resolve.Str (splitOnce, splitOnceStr)
import Data.Char (isDigit)
import Data.List (stripPrefix)

-- | `unannotated`: the text before each `@`, then what follows its
-- annotation.
unannotated :: String -> String
unannotated name = case splitOnce '@' name of
  Just (before, after) -> before <> unannotated (pastAnnotation after)
  Nothing -> name

-- | `past_annotation`: the text after an annotation whose `@` is read.
pastAnnotation :: String -> String
pastAnnotation s = case dotted s of
  (Just _, t) -> case eat '(' t of
    (True, t') -> group (1 :: Int) t'
    (False, t') -> t'
  (Nothing, t) -> t

-- | `Lexer::trivia`: White_Space and comments (an unterminated block
-- comment runs to the end).
trivia :: String -> String
trivia s = case dropWhile isRustWhite s of
  '/' : '/' : rest -> trivia (maybe "" snd (splitOnce '\n' rest))
  '/' : '*' : rest -> trivia (maybe "" snd (splitOnceStr "*/" rest))
  t -> t

-- | `Lexer::eat`: the character after the trivia, if it is the one.
eat :: Char -> String -> (Bool, String)
eat c s = case trivia s of
  x : rest | x == c -> (True, rest)
  t -> (False, t)

-- | `Lexer::ident`: an identifier not starting with an ASCII digit.
ident :: String -> (Maybe String, String)
ident s = case span identChar t of
  (word@(w : _), rest) | not (isAsciiDigit w) -> (Just word, rest)
  _ -> (Nothing, t)
 where
  t = trivia s
  isAsciiDigit c = c < '\x80' && isDigit c

-- | `Lexer::name`: a dotted name, and whether it ended `.*`.
dotted :: String -> (Maybe (String, Bool), String)
dotted s = case ident s of
  (Just w, t) -> continue w t
  (Nothing, t) -> (Nothing, t)
 where
  continue acc t = case eat '.' t of
    (False, t') -> (Just (acc, False), t')
    (True, t') -> case eat '*' t' of
      (True, t'') -> (Just (acc, True), t'')
      (False, t'') -> case ident t'' of
        (Just w, t3) -> continue (acc <> "." <> w) t3
        (Nothing, t3) -> (Nothing, t3)

-- | `Lexer::group`: past a `(` already eaten, to its matching `)`.
group :: Int -> String -> String
group 0 s = s
group depth s = case trivia s of
  [] -> []
  c : rest -> case c of
    '(' -> group (depth + 1) rest
    ')' -> group (depth - 1) rest
    '"' -> group depth (literal '"' rest)
    '\'' -> group depth (literal '\'' rest)
    _ -> group depth rest

-- | `Lexer::literal`: past the opening quote to the closing one,
-- escapes honoured; `"""` opens a text block, closed by the next `"""`.
literal :: Char -> String -> String
literal quote s
  | quote == '"', Just rest <- stripPrefix "\"\"" s = maybe "" snd (splitOnceStr "\"\"\"" rest)
  | otherwise = go s
 where
  go [] = []
  go ('\\' : rest) = go (drop 1 rest)
  go (c : rest)
    | c == quote = rest
    | otherwise = go rest

-- | `ident_char` (java_types.rs): Alphabetic or Numeric, `_`, `$`.
identChar :: Char -> Bool
identChar c = isRustAlnum c || c == '_' || c == '$'
