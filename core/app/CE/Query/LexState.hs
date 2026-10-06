-- | The query scanner's state and its byte-level moves (plan v2.33 W1;
-- split out of CE.Query.Lex at the module wall): the program as it
-- grows across the three sources (cli/src/query/program.rs `Program`
-- at 1324c927), the scanner's cursor over one source's UTF-8 bytes
-- (lexer.rs `Lexer`), and the moves every scanning function shares —
-- each the Rust function of the same name, camelCased.
module CE.Query.LexState (
  Tok (..),
  Lexed (..),
  Fault (..),
  Frame (..),
  Lx (..),
  emptyLexed,
  peek,
  bump,
  bumpN,
  isWs,
  skipSpace,
  nextSolid,
  push,
  name,
  endClause,
  fault,
  faultAt,
  intern,
  punct,
  slice,
  sym,
  utf8,
) where

import CE.Query.Cost (kindSym)
import Data.Bits (shiftR, xor, (.&.), (.|.))
import qualified Data.ByteString as B
import Data.Char (ord)
import qualified Data.Map.Strict as M
import Data.Word (Word64, Word8)

-- | One token: its wire kind and value, the source it came from (0
-- prelude, 1 rules, 2 query), where, and the spelling it stands for.
data Tok = Tok {tKind :: Integer, tValue :: Integer, tSrc :: Int, tLine :: Int, tCol :: Int, tSpell :: String}

-- | The lexed program: the tokens (newest first while scanning, in
-- order once `lexProgram` returns), the clauses over every source and
-- how many of them are the prelude's, the program predicates by code −
-- 1000, the globs by set id, every name spelled by hash, and per
-- clause its variable names by number (`_` for an anonymous one;
-- newest clause first while scanning).
data Lexed = Lexed
  { lTokens :: [Tok]
  , lClauses :: Int
  , lPrelude :: Int
  , lPreds :: [String]
  , lSets :: [String]
  , lNames :: M.Map Word64 String
  , lVars :: [[String]]
  }

-- | A lexical fault: where (source, line, column), and what.
data Fault = Fault {fSrc :: Int, fLine :: Int, fCol :: Int, fWhat :: String}

-- | One open parenthesis: the predicate it opened, when it opened an
-- atom, and the argument the scanner is inside.
data Frame = Frame {frPred :: Maybe String, frArg :: Int}

-- | The scanner over one source: the program so far, the source, its
-- bytes and the cursor (index, line, column), the clause's variables,
-- the open parentheses (innermost first) and the predicate a `(`
-- would open.
data Lx = Lx
  { xP :: Lexed
  , xSrc :: Int
  , xB :: B.ByteString
  , xI :: Int
  , xLine :: Int
  , xCol :: Int
  , xVars :: [String]
  , xFrames :: [Frame]
  , xPending :: Maybe String
  }

emptyLexed :: Lexed
emptyLexed = Lexed [] 0 0 [] [] M.empty []

peek :: Lx -> Maybe Word8
peek x = B.indexMaybe (xB x) (xI x)

-- | `Lexer::bump`: one byte; a newline starts the next line, and a
-- byte that starts a character moves the column.
bump :: Lx -> Lx
bump x = case peek x of
  Just 10 -> x {xI = xI x + 1, xLine = xLine x + 1, xCol = 1}
  Just c | c .&. 0xC0 /= 0x80 -> x {xI = xI x + 1, xCol = xCol x + 1}
  _ -> x {xI = xI x + 1}

bumpN :: Int -> Lx -> Lx
bumpN n x = iterate bump x !! n

-- | `u8::is_ascii_whitespace`: space, tab, newline, form feed, return.
isWs :: Word8 -> Bool
isWs c = c == 32 || c == 9 || c == 10 || c == 12 || c == 13

-- | `Lexer::skip_space`: whitespace and `#` comments to the line's end.
skipSpace :: Lx -> Lx
skipSpace x = case peek x of
  Just 35 -> skipSpace (bumpN (B.length (B.takeWhile (/= 10) (B.drop (xI x) (xB x)))) x)
  Just c | isWs c -> skipSpace (bump x)
  _ -> x

-- | `Lexer::next_solid`: the byte after any whitespace, without moving.
nextSolid :: Lx -> Maybe Word8
nextSolid x = B.find (not . isWs) (B.drop (xI x) (xB x))

-- | `Lexer::push`.
push :: Lx -> Integer -> Integer -> String -> Int -> Int -> Lx
push x kind value spell l c = x {xP = (xP x) {lTokens = Tok kind value (xSrc x) l c spell : lTokens (xP x)}}

-- | `Lexer::name`: a name constant by its hash; the first spelling of
-- a hash is the one kept.
name :: Lx -> String -> Int -> Int -> Lx
name x text = push x {xP = (xP x) {lNames = M.insertWith (\_ old -> old) h text (lNames (xP x))}} kindSym (toInteger h) text
 where
  h = sym text

-- | `Lexer::end_clause`.
endClause :: Lx -> Lx
endClause x = x {xVars = [], xP = (xP x) {lVars = xVars x : lVars (xP x), lClauses = lClauses (xP x) + 1}}

-- | `Lexer::fault`: a fault where the scanner stands.
fault :: Lx -> String -> Fault
fault x = faultAt x (xLine x) (xCol x)

-- | `Lexer::fault_at`: a fault pinned where the offending item began.
faultAt :: Lx -> Int -> Int -> String -> Fault
faultAt x = Fault (xSrc x)

-- | `program.rs` `intern`: the index of a name in the list, appended
-- when new.
intern :: [String] -> String -> (Int, [String])
intern list w = case lookup w (zip list [0 ..]) of
  Just i -> (i, list)
  Nothing -> (length list, list <> [w])

-- | `lexer::punct`: punctuation and keywords by spelling
-- (CE.Query.Cost's codes, 10 up).
punct :: String -> Maybe Integer
punct s = lookup s (zip (words spellings) [10 ..])
 where
  spellings = ":- , . ( ) not ?- assert = != < <= > >= + - * / % count min max sum :"

-- | The bytes between two indices as the text they spell (the Rust
-- scanner's `String::from_utf8_lossy` over a slice of a `&str` cut at
-- ASCII bytes: always whole characters).
slice :: Int -> Int -> Lx -> String
slice from to x = fromUtf8 (B.unpack (B.take (to - from) (B.drop from (xB x))))

-- | `legend::sym`: the fnv1a64 of a name's UTF-8 bytes — the hash the
-- mention table and the term bags key by.
sym :: String -> Word64
sym = foldl (\h b -> (h `xor` fromIntegral b) * 1099511628211) 14695981039346656037 . utf8

-- | A string's UTF-8 bytes.
utf8 :: String -> [Word8]
utf8 = concatMap (enc . ord)
 where
  enc c
    | c < 0x80 = [fromIntegral c]
    | c < 0x800 = map fromIntegral [0xC0 .|. shiftR c 6, cont c]
    | c < 0x10000 = map fromIntegral [0xE0 .|. shiftR c 12, cont (shiftR c 6), cont c]
    | otherwise = map fromIntegral [0xF0 .|. shiftR c 18, cont (shiftR c 12), cont (shiftR c 6), cont c]
  cont c = 0x80 .|. (c .&. 0x3F)

-- | Well-formed UTF-8 bytes back to their characters.
fromUtf8 :: [Word8] -> String
fromUtf8 [] = []
fromUtf8 (b : bs)
  | b < 0x80 = toEnum (fromIntegral b) : fromUtf8 bs
  | b < 0xE0 = multi 1 (fromIntegral b .&. 0x1F)
  | b < 0xF0 = multi 2 (fromIntegral b .&. 0x0F)
  | otherwise = multi 3 (fromIntegral b .&. 0x07)
 where
  multi :: Int -> Int -> String
  multi n lead =
    let (cs, rest) = splitAt n bs
     in toEnum (foldl (\acc c -> acc * 64 + (fromIntegral c .&. 0x3F)) lead cs) : fromUtf8 rest
