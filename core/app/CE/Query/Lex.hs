-- | The query language's scanner (design booklet §4.4; plan v2.33 W1
-- moved it here from cli/src/query/lexer.rs and program.rs at
-- 1324c927 — each function below is the Rust function of the same
-- name, camelCased): the program's three sources (the built-in
-- prelude, the rules file, the ad hoc query) into the `[kind, value]`
-- tokens the parser reads, each keeping the line and column it came
-- from, so an error reported at a token index reads back as `where
-- line:column`. The scanner knows spellings and nothing of shapes:
-- variables number per clause by first appearance (`_` takes a fresh
-- number), program predicates number from 1000 by first appearance
-- across every source, a lowercase word is a predicate when a `(`
-- follows it and a name constant otherwise, a quoted string is a name
-- except in the one position whose sort is a set — `set(S, …)`'s
-- first argument and the `in(F, "glob")` sugar, rewritten here to
-- `set(S, F)`. The text is scanned as its UTF-8 bytes, as the Rust
-- scanner did: a column counts the bytes that start a character.
module CE.Query.Lex (Tok (..), Lexed (..), Fault (..), lexProgram, referenced, sym) where

import CE.Query.Cost (idbFloor, kindAnon, kindInt, kindPred, kindSet, kindVar)
import CE.Query.LexState
import CE.Query.Schema (codeNamed)
import qualified Data.ByteString as B
import Data.Char (chr)
import Data.List (elemIndex)
import qualified Data.Set as S
import Data.Word (Word8)

-- | `Program::lex`: the three sources in wire order; the first
-- lexical fault stops the whole program (the core never parses it).
-- The tokens and the variable tables come back in order.
lexProgram :: String -> Maybe String -> Maybe String -> Either Fault Lexed
lexProgram prelude rules query = do
  p0 <- run 0 prelude emptyLexed
  let p1 = p0 {lPrelude = lClauses p0}
  p2 <- maybe (Right p1) (\t -> run 1 t p1) rules
  p3 <- maybe (Right p2) (\t -> run 2 t p2) query
  Right p3 {lTokens = reverse (lTokens p3), lVars = reverse (lVars p3)}

-- | `Program::referenced`: the schema predicates the program reads.
referenced :: Lexed -> S.Set Integer
referenced p = S.fromList [tValue t | t <- lTokens p, tKind t == kindPred, tValue t < idbFloor]

-- | `Lexer::run`: the whole source; an unfinished clause at the end
-- still seats its variable table (the parser names the syntax error).
run :: Int -> String -> Lexed -> Either Fault Lexed
run src text p = go (Lx p src (B.pack (utf8 text)) 0 1 1 [] [] Nothing)
 where
  go x0 =
    let x = skipSpace x0
     in if xI x >= B.length (xB x) then Right (finish x) else item x >>= go
  finish x = xP (if openClause x then endClause x else x)

-- | `Lexer::open_clause`.
openClause :: Lx -> Bool
openClause x = case lTokens (xP x) of
  t : _ -> tSrc t == xSrc x && not (tSpell t == "." && null (xFrames x))
  [] -> False

-- | `Lexer::item`.
item :: Lx -> Either Fault Lx
item x = case peek x of
  Just 34 -> string x l c
  Just b | b >= 48 && b <= 57 -> number False x l c
  Just 45 | digitNext && not (operandBefore x) -> number True x l c
  Just b | isAlphaB b || b == 95 -> wordToken x l c
  _ -> punctToken x l c
 where
  (l, c) = (xLine x, xCol x)
  digitNext = maybe False (\b -> b >= 48 && b <= 57) (B.indexMaybe (xB x) (xI x + 1))

isAlphaB, isAlnumB, isUpperB :: Word8 -> Bool
isAlphaB b = (b >= 65 && b <= 90) || (b >= 97 && b <= 122)
isAlnumB b = isAlphaB b || (b >= 48 && b <= 57)
isUpperB b = b >= 65 && b <= 90

-- | `Lexer::operand_before`: whether the last token ends an operand —
-- then a `-` is the operator, not a negative literal's sign.
operandBefore :: Lx -> Bool
operandBefore x = case lTokens (xP x) of
  t : _ -> (tKind t >= 1 && tKind t <= 6) || tSpell t == ")"
  [] -> False

-- | `Lexer::word`: ASCII letters, digits and `_`.
word :: Lx -> (String, Lx)
word x = (map (chr . fromIntegral) (B.unpack w), bumpN (B.length w) x)
 where
  w = B.takeWhile (\b -> isAlnumB b || b == 95) (B.drop (xI x) (xB x))

-- | `Lexer::string`: `"…"` up to the closing quote on the same line —
-- a set in the set position, a name anywhere else.
string :: Lx -> Int -> Int -> Either Fault Lx
string x0 l c
  | peek x2 /= Just 34 = Left (faultAt x0 l c "unterminated string")
  | inSetPosition x3 = let (n, x4) = setId x3 text in Right (push x4 kindSet n text l c)
  | otherwise = Right (name x3 text l c)
 where
  x1 = bump x0
  open b = b /= 34 && b /= 10
  x2 = bumpN (B.length (B.takeWhile open (B.drop (xI x1) (xB x1)))) x1
  text = slice (xI x1) (xI x2) x2
  x3 = bump x2

-- | `Lexer::in_set_position`.
inSetPosition :: Lx -> Bool
inSetPosition x = case xFrames x of
  f : _ -> frPred f == Just "set" && frArg f == 0
  [] -> False

-- | `Program::set_id`: a glob's set id by first appearance from 0.
setId :: Lx -> String -> (Integer, Lx)
setId x glob = let (i, sets) = intern (lSets (xP x)) glob in (toInteger i, x {xP = (xP x) {lSets = sets}})

-- | `Program::idb_code`: a program predicate's code by first
-- appearance from 1000.
idbCode :: Lx -> String -> (Integer, Lx)
idbCode x w = let (i, preds) = intern (lPreds (xP x)) w in (idbFloor + toInteger i, x {xP = (xP x) {lPreds = preds}})

-- | `Lexer::number`: digits after an optional `-`, read as a u64 (a
-- negative literal down to −2^63).
number :: Bool -> Lx -> Int -> Int -> Either Fault Lx
number negative x0 l c = case value of
  Just v -> Right (push x2 kindInt v spell l c)
  Nothing -> Left (faultAt x0 l c "integer out of range")
 where
  x1 = if negative then bump x0 else x0
  x2 = bumpN (B.length (B.takeWhile (\b -> b >= 48 && b <= 57) (B.drop (xI x1) (xB x1)))) x1
  spell = slice (xI x0) (xI x2) x2
  n = read (dropWhile (== '-') spell) :: Integer
  value
    | n > 18446744073709551615 = Nothing
    | not negative = Just n
    | n <= 9223372036854775808 = Just (negate n)
    | otherwise = Nothing

-- | `Lexer::word_token`.
wordToken :: Lx -> Int -> Int -> Either Fault Lx
wordToken x0 l c
  | isUpperB first || first == 95 = Right (variable x1 w l c)
  | Just code <- punct w = Right (push x1 code 0 w l c)
  | nextSolid x1 == Just 40 && w == "in" = sugar x1 l c
  | nextSolid x1 == Just 40 = Right (predicate (maybe (idbCode x1 w) (\k -> (toInteger k, x1)) (codeNamed w)))
  | otherwise = Right (name x1 w l c)
 where
  (w, x1) = word x0
  first = B.index (xB x0) (xI x0)
  predicate (code, x2) = push x2 {xPending = Just w} kindPred code w l c

-- | `Lexer::variable`: a variable by first appearance in its clause;
-- `_` (and any word starting with it) takes a fresh number every time.
variable :: Lx -> String -> Int -> Int -> Lx
variable x w l c = push x {xVars = vars'} kind (toInteger n) w l c
 where
  anon = take 1 w == "_"
  seat = if anon then Nothing else elemIndex w (xVars x)
  (n, vars') = case seat of
    Just i -> (i, xVars x)
    Nothing -> (length (xVars x), xVars x <> [if anon then "_" else w])
  kind = if anon then kindAnon else kindVar

-- | `Lexer::punct_token`: a two-byte spelling when the two bytes are
-- one, else one byte; a byte that spells nothing is the fault.
punctToken :: Lx -> Int -> Int -> Either Fault Lx
punctToken x l c = case [s | s <- [two, one], not (null s), Just _ <- [punct s]] of
  spell : _ -> structural spell (bumpN (length spell) x) l c
  [] -> Left (fault x "unexpected character")
 where
  bytes k = B.take k (B.drop (xI x) (xB x))
  ascii bs = if B.length bs > 0 && B.all (< 128) bs then map (chr . fromIntegral) (B.unpack bs) else ""
  two = if B.length (bytes 2) == 2 then ascii (bytes 2) else ""
  one = ascii (bytes 1)

-- | The rest of `punct_token`: the frames a parenthesis or a comma
-- moves, the token, and the clause a top-level `.` ends.
structural :: String -> Lx -> Int -> Int -> Either Fault Lx
structural spell x l c = do
  x1 <- case (spell, xFrames x) of
    ("(", fs) -> Right x {xFrames = Frame (xPending x) 0 : fs, xPending = Nothing}
    (")", []) -> Left (faultAt x l c "unbalanced `)`")
    (")", _ : fs) -> Right x {xFrames = fs}
    (",", f : fs) -> Right x {xFrames = f {frArg = frArg f + 1} : fs}
    _ -> Right x
  let x2 = push x1 (maybe 0 id (punct spell)) 0 spell l c
  Right (if spell == "." && null (xFrames x2) then endClause x2 else x2)

-- | `Lexer::sugar`: `in(F, "glob")` → `set(S, F)` — the tokens the
-- spelled-out form carries, the glob and the file term swapped into
-- set order.
sugar :: Lx -> Int -> Int -> Either Fault Lx
sugar x0 l c = do
  x2 <- expect 40 (push x0 kindPred (maybe 0 toInteger (codeNamed "set")) "in" l c)
  let x3 = skipSpace x2
      (tl, tc) = (xLine x3, xCol x3)
  (term, x4) <- case peek x3 of
    Just b | isUpperB b || b == 95 -> Right (word x3)
    _ -> Left (fault x3 shape)
  x5 <- skipSpace <$> expect 44 x4
  x6 <- if peek x5 == Just 34 then Right x5 else Left (fault x5 shape)
  x7 <- string x6 {xFrames = Frame (Just "set") 0 : xFrames x6} (xLine x6) (xCol x6)
  let x8 = swapTop x7 {xFrames = drop 1 (xFrames x7)}
  expect 41 (variable x8 term tl tc)
 where
  shape = "in(File, \"glob\") takes a variable and a quoted glob"
  expect b x = expectB b shape x
  -- the newest two tokens (the scanner keeps them newest first)
  swapTop x = x {xP = (xP x) {lTokens = swapTwo (lTokens (xP x))}}
  swapTwo ts = case ts of
    a : b : rest -> b : a : rest
    _ -> ts

-- | `Lexer::expect`: one punctuation byte after any whitespace, or the
-- shape fault.
expectB :: Word8 -> String -> Lx -> Either Fault Lx
expectB b shape x0
  | peek x1 /= Just b = Left (fault x1 shape)
  | otherwise = Right (push (bump x1) (maybe 0 id (punct spell)) 0 spell (xLine x1) (xCol x1))
 where
  x1 = skipSpace x0
  spell = [chr (fromIntegral b)]
