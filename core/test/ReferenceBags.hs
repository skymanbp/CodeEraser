-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | The bags family's second spelling (plan v2.33 W6). The identifier
-- splitter is written by cut positions — an index list of where each
-- piece starts, then `zip` over consecutive cuts — instead of the core's
-- left fold carrying a reversed buffer; a key's arity is found by
-- `elemIndices`, not by reversing; the bag is a stable `sortOn` grouped by
-- term (the first-met channel is the head of its group, the count the
-- sum) instead of `Map.insertWith`. The stemmer, the character classes and
-- the hashes are the core's own primitives, held to the measuring side's
-- frozen road by the tests subrepo's differential (unit/similar/
-- bags_diff.rs), not respelled here. It draws two hundred seeded rows.
module ReferenceBags (BCase (..), cases, encodeCase, expected) where

import CE.Resolve.Chars (isRustAlnum)
import CE.Similar.Chars (isRustLower, isRustNumeric, isRustUpper)
import CE.Similar.Lower (rustToLower)
import CE.Similar.Terms (featureTerm, utf8, wordTerm)
import Data.Aeson (Value, object, toJSON, (.=))
import Data.Char (isDigit)
import Data.Function (on)
import qualified Data.ByteString as B
import Data.List (elemIndices, groupBy, isPrefixOf, sortOn)
import qualified Data.Set as Set
import ReferenceFlowGen (G, S (..), rand, runG)

-- | One unit row and one text.
data BCase = BCase
  { bKey, bKind :: String
  , bRet :: Maybe Integer
  , bCallees, bLiterals :: [String]
  , bStruct :: [[Integer]]
  , bDocs :: [String]
  , bText :: String
  }

encodeCase :: BCase -> Value
encodeCase c =
  object
    [ "proto" .= ("9.0.0" :: String)
    , "type" .= ("bags.request" :: String)
    , "id" .= (1 :: Int)
    , "units" .= [[toJSON (bKey c), toJSON (bKind c), toJSON (bRet c), toJSON (bCallees c), toJSON (bLiterals c), toJSON (bStruct c), toJSON (bDocs c)]]
    , "texts" .= [bText c]
    ]

-- | The pieces of a text: a piece starts at an alphanumeric character
-- whose predecessor is not one, or where the case / digit rules cut.
pieces :: String -> [String]
pieces t = [concatMap rustToLower (take (b - a) (drop a t)) | (a, b) <- spans]
 where
  n = length t
  alnum i = i >= 0 && i < n && isRustAlnum (t !! i)
  starts = [i | i <- [0 .. n - 1], alnum i, not (alnum (i - 1)) || cut i]
  spans = [(a, runEnd a) | a <- starts]
  runEnd a = case [j | j <- [a + 1 .. n - 1], not (alnum j) || j `elem` starts] of
    (j : _) -> j
    [] -> n
  cut i =
    let (p, c) = (t !! (i - 1), t !! i)
        next = [t !! (i + 1) | i + 1 < n]
     in isRustNumeric p /= isRustNumeric c
          || (isRustLower p && isRustUpper c)
          || (isRustUpper p && isRustUpper c && any isRustLower next)

stops :: [String]
stops = words "a an the of to and or in on for is are be was this that it its as by with from at if not we you i s t into than then so but do does all any no our can will which when what each one"

-- | The base and arity of a key: after the last slash, all digits.
keyParts :: String -> (String, Maybe String)
keyParts key = case elemIndices '/' key of
  [] -> (key, Nothing)
  is ->
    let (base, rest) = splitAt (last is) key
        digits = drop 1 rest
     in if not (null digits) && all isDigit digits then (base, Just digits) else (key, Nothing)

-- | Every term the case meets, in the measuring side's order.
met :: BCase -> [(Integer, Integer, Integer)]
met c =
  [(w 0 x, 0, 1) | x <- names]
    <> [(f 1 s, 1, 1) | s <- ("k:" <> bKind c) : ["p:" <> d | Just d <- [arity]] <> ["ret:" <> show r | Just r <- [bRet c]]]
    <> [(w 2 x, 2, 1) | cl <- bCallees c, x <- pieces cl]
    <> [(f 5 l, 5, 1) | l <- bLiterals c]
    <> [(toInteger (featureTerm 4 (B.pack [fromInteger ((k `div` 256 ^ i) `mod` 256) | i <- [0 .. 7 :: Int]])), 4, n) | [k, n] <- bStruct c]
    <> [(w 3 x, 3, 1) | line <- bDocs c, x <- pieces line, x `notElem` stops]
 where
  (base, arity) = keyParts (bKey c)
  names
    | base == "(anonymous)" = []
    | "impl " `isPrefixOf` base = filter (`notElem` ["impl", "for"]) (pieces base)
    | otherwise = pieces base
  w ch = toInteger . wordTerm ch
  f ch = toInteger . featureTerm ch . utf8

-- | Grouped by term, ascending: the first channel met, the summed count.
grouped :: [(Integer, Integer, Integer)] -> [[Integer]]
grouped ts = [[t, ch, sum [n | (_, _, n) <- g]] | g@((t, ch, _) : _) <- groupBy ((==) `on` fst3) (sortOn fst3 ts)]
 where
  fst3 (t, _, _) = t

-- | The reply's bags and texts for one case.
expected :: BCase -> [Maybe Value]
expected c =
  map
    Just
    [ toJSON [grouped (met c)]
    , toJSON [grouped [(toInteger (wordTerm ch x), toInteger ch, 1) | x <- pieces (bText c), x `notElem` stops, ch <- [0, 3]]]
    ]

cases :: [BCase]
cases = [runG draw (S (n * 6007 + 29) 0 0 0) | n <- [1 .. 200 :: Int]]

draw :: G BCase
draw = do
  key <- ident
  arity <- rand 3
  shapeKey <- rand 4
  kind <- pick ["fn", "method", "lambda", "class", "const", "mod", "impl", "type"]
  ret <- rand 3
  callees <- many 4 ident
  literals <- many 3 (pick ["l:str", "l:num", "l:bool", "l:other"])
  kinds <- many 4 (toInteger <$> rand 1000)
  counts <- mapM (const ((+ 1) . toInteger <$> rand 5)) kinds
  docs <- many 3 (unwords <$> many 6 ident)
  text <- unwords <$> many 8 ident
  let key'
        | shapeKey == 0 = "(anonymous)/" <> show arity
        | shapeKey == 1 = "impl " <> key <> " for Vec"
        | shapeKey == 2 = key <> "/" <> show arity
        | otherwise = key
      structure = [[k * 18446744073709551 + 7, c] | (k, c) <- zip (Set.toList (Set.fromList kinds)) counts]
  pure (BCase key' kind ([Nothing, Just 0, Just 1] !! ret) callees literals structure docs text)

many :: Int -> G a -> G [a]
many n g = rand (n + 1) >>= \k -> mapM (const g) [1 .. k]

pick :: [String] -> G String
pick xs = (xs !!) <$> rand (length xs)

-- | An identifier of words, case runs, digits and a few other scripts,
-- glued, snake- or camel-joined; now and then a stop word or a suffix
-- the stemmer folds.
ident :: G String
ident = do
  n <- (+ 1) <$> rand 3
  ws <- mapM (const (pick vocab)) [1 .. n]
  joiner <- pick ["", "_", "-", " "]
  pure (foldr1 (\a b -> a <> joiner <> b) ws)
 where
  vocab = words "parse JSON file http2 server getX ABC __init__ the of is fetching relational hopeful caresses İstanbul Straße ΣΑΣ Ⅻ ⓐ x1 café naïve 日本 controll generalizations adoption"
