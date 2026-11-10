-- | One helper, one home (plan v2.35 cleanup C2/C3): the core's FNV-1a
-- 64, its UTF-8 encoder, its `{}` splitter and its piece joiner live
-- once, in CE.Text. Each had copies, and the copies the product read
-- are kept HERE, verbatim, as the reference the new owner is held to:
-- ten thousand seeded inputs per leg (every UTF-8 width, the surrogate
-- block included, and the brace alphabet the hole rules turn on) give
-- the same bytes, the same hash and the same pieces under both.
module TextProps (battery) where

import CE.Similar.Terms (featureTerm)
import CE.Text (Piece (..), fnv1a64, fnvBasis, fnvPrime, fnvStep, holes, join, splitHoles, utf8)
import Data.Aeson (toJSON)
import Data.Bits (shiftR, xor, (.&.), (.|.))
import qualified Data.ByteString as B
import qualified Data.ByteString.Builder as BB
import qualified Data.ByteString.Lazy as BL
import Data.Char (chr, ord)
import Data.List (isPrefixOf, tails)
import Data.Word (Word64, Word8)
import WireHarness (runChecks)

battery :: IO Bool
battery =
  runChecks
    [ ("10,000 strings: CE.Text.utf8 = the query lexer's own encoder, byte for byte", all (\s -> utf8 s == B.pack (lexerUtf8 s)) strings)
    , ("10,000 strings: CE.Text.utf8 = the term bags' builder encoder", all (\s -> utf8 s == builderUtf8 s) strings)
    , ("10,000 strings: fnv1a64 . utf8 = the query lexer's `sym`", all (\s -> fnv1a64 (utf8 s) == lexerSym s) strings)
    , ("10,000 strings over six channels: featureTerm = the term bags' own fold", and [featureTerm ch (utf8 s) == bagTerm ch (utf8 s) | (ch, s) <- zip (cycle [0 .. 5]) strings])
    , ("10,000 byte runs: the lazy fnvStep fold = the definition package digest's step", all (\s -> let b = BL.fromStrict (utf8 s) in BL.foldl' fnvStep fnvBasis b == digestFold b) strings)
    , ("the constants are FNV-1a 64's (basis 14695981039346656037, prime 1099511628211)", fnvBasis == 14695981039346656037 && fnvPrime == 1099511628211)
    , ("10,000 brace strings: splitHoles = the binder's splitter, holes = the old count", all (\s -> splitHoles s == binderSplit s && holes s == oldHoles s) braces)
    , ("10,000 piece lists: join = the left-fold joiner (text, references, keys)", all (\(sep, ps) -> same (join sep ps) (oldJoin sep ps)) pieceLists)
    , ("the strings reach every UTF-8 width and the surrogate block", all (\w -> any (any w) strings) [(< '\x80'), inWidth 0x80 0x7FF, inWidth 0x800 0xFFFF, (> '\xFFFF'), inWidth 0xD800 0xDFFF])
    , ("the brace strings hold several holes and `{{}}` runs", any ((> 1) . oldHoles) braces && any (any ("{{}}" `isPrefixOf`) . tails) braces)
    , ("the piece lists hold empty, single and long lists", all (\k -> any ((== k) . length . snd) pieceLists) [0, 1, 9])
    ]
 where
  same (Piece a b c) (Piece a' b' c') = a == a' && b == b' && c == c'
  inWidth lo hi c = ord c >= lo && ord c <= hi

-- | A deterministic stream (the 31-bit LCG the flow generator uses),
-- its high bits.
stream :: Integer -> [Int]
stream seed = map (fromInteger . (`div` 65536)) (drop 1 (iterate step seed))
 where
  step x = (x * 1103515245 + 12345) `mod` 2147483648

-- | Ten thousand strings of 0..23 characters, each character drawn from
-- one of five bands: ASCII, two-byte, three-byte (surrogates
-- included), four-byte, and the surrogate block alone.
strings :: [String]
strings = take 10000 (go (stream 20261109))
 where
  go (n : rest) = let (xs, more) = splitAt (2 * (n `mod` 24)) rest in map charOf (pairs xs) : go more
  go [] = []
  pairs (a : b : rest) = (a, b) : pairs rest
  pairs _ = []
  charOf (band, x) = chr $ case band `mod` 5 of
    0 -> x `mod` 0x80
    1 -> 0x80 + x `mod` 0x780
    2 -> 0x800 + x `mod` 0xF800
    3 -> 0x10000 + (x * 37) `mod` 0x100000
    _ -> 0xD800 + x `mod` 0x800

-- | Ten thousand strings over `{`, `}`, `a`, `b`, 0..15 long.
braces :: [String]
braces = take 10000 (go (stream 7))
 where
  go (n : rest) = let (cs, more) = splitAt (n `mod` 16) rest in map (\c -> "{}ab" !! (c `mod` 4)) cs : go more
  go [] = []

-- | Ten thousand (separator, pieces) pairs: 0..9 pieces of 0..5 chars,
-- each with 0..2 references and 0..1 keys.
pieceLists :: [(String, [Piece])]
pieceLists = take 10000 (go (stream 99))
 where
  go (s : n : rest) =
    let (ps, more) = pieces (n `mod` 10) rest
     in ([", ", "", " / "] !! (s `mod` 3), ps) : go more
  go _ = []
  pieces :: Int -> [Int] -> ([Piece], [Int])
  pieces 0 rest = ([], rest)
  pieces k (l : r : rest) =
    let (cs, more) = splitAt (l `mod` 6) rest
        (ps, after) = pieces (k - 1) more
        piece = Piece (map (\c -> "xyz{}" !! (c `mod` 5)) cs) (map toJSON (take (r `mod` 3) [l, r])) (take (r `mod` 2) [show l])
     in (piece : ps, after)
  pieces _ _ = ([], [])

-- The copies the product read before C2/C3, verbatim.

-- | CE.Query.LexState.utf8.
lexerUtf8 :: String -> [Word8]
lexerUtf8 = concatMap (enc . ord)
 where
  enc c
    | c < 0x80 = [fromIntegral c]
    | c < 0x800 = map fromIntegral [0xC0 .|. shiftR c 6, cont c]
    | c < 0x10000 = map fromIntegral [0xE0 .|. shiftR c 12, cont (shiftR c 6), cont c]
    | otherwise = map fromIntegral [0xF0 .|. shiftR c 18, cont (shiftR c 12), cont (shiftR c 6), cont c]
  cont c = 0x80 .|. (c .&. 0x3F)

-- | CE.Query.LexState.sym.
lexerSym :: String -> Word64
lexerSym = foldl (\h b -> (h `xor` fromIntegral b) * 1099511628211) 14695981039346656037 . lexerUtf8

-- | CE.Similar.Terms.utf8.
builderUtf8 :: String -> B.ByteString
builderUtf8 = BL.toStrict . BB.toLazyByteString . BB.stringUtf8

-- | CE.Similar.Terms.term.
bagTerm :: Int -> B.ByteString -> Word64
bagTerm ch bytes = B.foldl' (\h x -> (h `xor` fromIntegral x) * 1099511628211) 14695981039346656037 tagged
 where
  tagged = builderUtf8 (take 1 (drop ch "NPCDSL") <> ":") <> bytes

-- | CE.Lang.digestOf's fold.
digestFold :: BL.ByteString -> Word64
digestFold = BL.foldl' step 14695981039346656037
 where
  step :: Word64 -> Word8 -> Word64
  step h b = (h `xor` fromIntegral b) * 1099511628211

-- | CE.Document.Bind.splitHoles.
binderSplit :: String -> [String]
binderSplit s = go s ""
 where
  go rest acc
    | "{}" `isPrefixOf` rest = reverse acc : go (drop 2 rest) ""
    | otherwise = case rest of
        [] -> [reverse acc]
        c : cs -> go cs (c : acc)

-- | CE.Text.holes over CE.Text.breakHole.
oldHoles :: String -> Int
oldHoles s = case breakHole s of
  (_, Just rest) -> 1 + oldHoles rest
  _ -> 0
 where
  breakHole t = case t of
    [] -> ([], Nothing)
    _ | "{}" `isPrefixOf` t -> ([], Just (drop 2 t))
    c : rest -> let (b, a) = breakHole rest in (c : b, a)

-- | CE.Text.join.
oldJoin :: String -> [Piece] -> Piece
oldJoin sep ps = case ps of
  [] -> Piece "" [] []
  p : rest -> foldl (\(Piece t r k) (Piece t' r' k') -> Piece (t <> sep <> t') (r <> r') (k <> k')) p rest
