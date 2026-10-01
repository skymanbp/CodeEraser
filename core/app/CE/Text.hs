-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | The message catalogue's machinery (plan v2.32 step 5; design
-- booklet docs/reference/authority-track.md §6): a family's console
-- sentences are templates `(en, zh)` keyed by a plain string, with
-- `{}` holes filled left to right — numbers formatted here, product
-- words written here, a reference left as `{}` for the measuring side
-- to bind (it holds the repository's strings; the core never sees
-- one). A filled template is a `Piece`: its text, the references its
-- holes still wait for, in order, and the keys it was built from (the
-- battery's coverage reading). A `Line` is a piece on a stream; on
-- the wire it is `[stream, text, ref…]`. The templates themselves
-- live one module per family under CE.Text.<Family>; this module
-- holds no sentence.
module CE.Text (Catalogue, Fill (..), Lang (..), Line (..), Piece (..), fixed, holes, join, fillOf, langOf, leftIn, line, lineValue, phrase, piece, plain, rightIn, signed, table) where

import Data.Aeson (Result (..), Value (..), fromJSON, toJSON)
import Data.List (isPrefixOf)

-- | The two console languages, by the request's code (0 en, 1 zh).
data Lang = En | Zh
  deriving (Eq, Show)

langOf :: Maybe Integer -> Lang
langOf c = if c == Just 1 then Zh else En

-- | A family's templates by key.
type Catalogue = [(String, (String, String))]

-- | A catalogue written as one text, a line per template: the key, the
-- English template and the Chinese apart by tabs, the Chinese left out
-- where it is the English. A line of another shape keeps a `!` key no
-- face asks for, so the battery's coverage leg names it.
table :: String -> Catalogue
table = map entry . lines
 where
  entry l = case fields l of
    [k, en] -> (k, (en, en))
    [k, en, zh] -> (k, (en, zh))
    _ -> ("!" <> l, ("", ""))
  fields s = case break (== '\t') s of
    (f, _ : rest) -> f : fields rest
    (f, []) -> [f]

-- | What fills a hole: a number (as Rust's `Display` prints it), a
-- product word already in the requested language, a reference the
-- measuring side binds, or a piece spliced whole.
data Fill = N Integer | W String | R Value | P Piece

-- | Text with its open holes' references, and the keys it was built
-- from. A key the catalogue lacks, or a template whose holes and fills
-- differ in number, is recorded as `!key` so the battery names it.
data Piece = Piece {pText :: String, pRefs :: [Value], pKeys :: [String]}

-- | One console line: 0 stdout, 1 stderr.
data Line = Line {lStream :: Integer, lPiece :: Piece}

-- | A template filled: holes left to right, a number or word written
-- in, a reference kept as `{}`, a piece spliced with its own holes.
phrase :: Catalogue -> Lang -> String -> [Fill] -> Piece
phrase cat lang key fills = case lookup key cat of
  Nothing -> Piece ("?" <> key) [] ["!" <> key]
  Just (en, zh) ->
    let template = if lang == Zh then zh else en
        Piece text refs keys = fill template fills
        arity = holes template == length fills
     in Piece text refs ((if arity then key else "!" <> key) : keys)

fill :: String -> [Fill] -> Piece
fill template fills = case (breakHole template, fills) of
  ((before, Just after), f : rest) ->
    let Piece t r k = fill after rest
     in case f of
          N n -> Piece (before <> show n <> t) r k
          W w -> Piece (before <> w <> t) r k
          R v -> Piece (before <> "{}" <> t) (v : r) k
          P (Piece pt pr pk) -> Piece (before <> pt <> t) (pr <> r) (pk <> k)
  ((before, _), _) -> Piece before [] []

-- | The text before the first `{}`, and the rest after it (Nothing:
-- no hole left).
breakHole :: String -> (String, Maybe String)
breakHole s = case s of
  [] -> ([], Nothing)
  _ | "{}" `isPrefixOf` s -> ([], Just (drop 2 s))
  c : rest -> let (b, a) = breakHole rest in (c : b, a)

-- | The `{}` holes in a text.
holes :: String -> Int
holes s = case breakHole s of
  (_, Just rest) -> 1 + holes rest
  _ -> 0

-- | A fill as a piece of its own: a word, a number, one open hole.
piece :: Fill -> Piece
piece f = case f of
  N n -> plain (show n)
  W w -> plain w
  R v -> Piece "{}" [v] []
  P p -> p

-- | A document field as a fill: a string the core wrote is a word, a
-- number a number, anything else (a reference) a hole.
fillOf :: Value -> Fill
fillOf v = case (v, fromJSON v) of
  (String _, Success w) -> W w
  (Number n, _) -> N (round n)
  _ -> R v

-- | A word with no template behind it: a product word, a number
-- written as text.
plain :: String -> Piece
plain w = Piece w [] []

-- | Pieces joined by a separator (the separator holds no hole).
join :: String -> [Piece] -> Piece
join sep ps = case ps of
  [] -> plain ""
  p : rest -> foldl (\(Piece t r k) (Piece t' r' k') -> Piece (t <> sep <> t') (r <> r') (k <> k')) p rest

line :: Integer -> Piece -> Line
line = Line

-- | The wire form: `[stream, text, ref…]`.
lineValue :: Line -> Value
lineValue (Line s (Piece t r _)) = toJSON (toJSON s : toJSON t : r)

-- | A text in `w` columns, left-aligned (Rust's `{:w$}` on a string)
-- or right-aligned (`{:>w}`); a longer text is never cut.
leftIn, rightIn :: Int -> String -> String
leftIn w t = t <> replicate (w - length t) ' '
rightIn w t = replicate (w - length t) ' ' <> t

-- | A signed integer as Rust's `{:+}` prints it: `+3`, `-3`, `+0`.
signed :: Integer -> String
signed n = (if n >= 0 then "+" else "") <> show n

-- | `num / den` as Rust's `{:.d}` prints the double it computes
-- (`num as f64 / den as f64`): the exact binary value of that double,
-- rounded half to even at `d` decimals (Rust's exact mode), the sign
-- kept on a negative value that rounds to zero; `inf` / `-inf` /
-- `NaN` past a zero denominator, as Rust prints them at any precision.
fixed :: Int -> Integer -> Integer -> String
fixed d num den
  | isNaN x = "NaN"
  | isInfinite x = if x < 0 then "-inf" else "inf"
  | otherwise = (if x < 0 || isNegativeZero x then "-" else "") <> whole <> (if d > 0 then "." <> frac else "")
 where
  x = fromInteger num / fromInteger den :: Double
  n = round (abs (toRational x) * 10 ^ d) :: Integer
  digits = let s = show n in replicate (d + 1 - length s) '0' <> s
  (whole, frac) = splitAt (length digits - d) digits
