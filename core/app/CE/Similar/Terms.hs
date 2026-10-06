-- | Term production of the same-role advisor (plan v2.33 W6; moved from
-- cli/src/similar/terms.rs, every function below named after the Rust
-- one it replaces) — the ONE tokenization road index and query share: a
-- query is hashed the way the index was. Only channel-tagged fnv1a64
-- hashes leave this family; the measuring side stores them and never a
-- word (plan §5.9.2 index privacy).
--
-- Channels are the evidence row's positions [N, P, C, D, S, L] = 0..5;
-- the one-letter label is mixed into every term hash, so a name word and
-- a callee word spelled alike are two terms.
module CE.Similar.Terms (
  chanName,
  chanShape,
  chanCallee,
  chanDoc,
  chanStructure,
  chanLiteral,
  splitIdent,
  proseWords,
  wordTerm,
  featureTerm,
  utf8,
  charClasses,
) where

import CE.Resolve.Chars (isRustAlnum)
import CE.Similar.Chars (isRustLower, isRustNumeric, isRustUpper)
import CE.Similar.Lower (rustToLower)
import CE.Similar.Stem (stem)
import Data.Bits (xor)
import Data.Char (ord)
import qualified Data.ByteString as B
import qualified Data.ByteString.Builder as BB
import qualified Data.ByteString.Lazy as BL
import qualified Data.Set as S
import Data.Word (Word64)

chanName, chanShape, chanCallee, chanDoc, chanStructure, chanLiteral :: Int
chanName = 0
chanShape = 1
chanCallee = 2
chanDoc = 3
chanStructure = 4
chanLiteral = 5

-- | Prose stop words dropped from the doc channel — a fixed small table,
-- never learned from a corpus. Identifier pieces are NOT filtered: `get`,
-- `set` and `is` are what a role is made of.
stopWords :: S.Set String
stopWords =
  S.fromList . words $
    "a an the of to and or in on for is are be was this that it its as by with from at if not we you \
    \i s t into than then so but do does all any no our can will which when what each one"

-- | Identifier pieces at camel / underscore / digit boundaries, lowercased
-- (Rust's full lowercase mapping), empties dropped: `parseJSONFile` →
-- parse json file, `http2_server` → http 2 server. Any character that is
-- not alphanumeric is a boundary, so prose lines split through the same
-- function (`proseWords`). The buffer is kept reversed.
splitIdent :: String -> [String]
splitIdent ident = reverse (flush final)
 where
  final = foldl step ("", []) (zip3 (Nothing : map Just ident) ident (map Just (drop 1 ident) <> [Nothing]))
  step (buf, out) (prev, c, next)
    | not (isRustAlnum c) = ("", flush (buf, out))
    | not (null buf) && boundary prev c next = (lowered c, flush (buf, out))
    | otherwise = (lowered c <> buf, out)
  lowered = reverse . rustToLower
  flush ("", out) = out
  flush (buf, out) = reverse buf : out

-- | Whether a piece boundary falls BEFORE `c` (it has an alphanumeric
-- predecessor whenever the buffer holds a piece): letter↔digit,
-- lower→Upper, or the last capital of a run followed by a lowercase
-- (`JSONFile` → JSON | File). `next` is the following character of the
-- text, alphanumeric or not.
boundary :: Maybe Char -> Char -> Maybe Char -> Bool
boundary Nothing _ _ = False
boundary (Just prev) c next =
  isRustNumeric prev /= isRustNumeric c
    || (isRustLower prev && isRustUpper c)
    || (isRustUpper prev && isRustUpper c && maybe False isRustLower next)

-- | Words of one prose line: identifier-split pieces with the stop words
-- dropped, so `parseJSON` in a comment meets `parse_json` in a name.
proseWords :: String -> [String]
proseWords = filter (`S.notMember` stopWords) . splitIdent

-- | The term of one WORD under a channel: stemmed, then hashed with the
-- channel tag.
wordTerm :: Int -> String -> Word64
wordTerm ch = term ch . utf8 . stem

-- | The term of one FEATURE (a shape, structure or literal-kind spelling)
-- under a channel — hashed as spelled, never stemmed.
featureTerm :: Int -> B.ByteString -> Word64
featureTerm = term

-- | fnv1a64 over the channel label, a colon and the bytes.
term :: Int -> B.ByteString -> Word64
term ch bytes = B.foldl' (\h x -> (h `xor` fromIntegral x) * 1099511628211) 14695981039346656037 tagged
 where
  tagged = utf8 (take 1 (drop ch "NPCDSL") <> ":") <> bytes

utf8 :: String -> B.ByteString
utf8 = BL.toStrict . BB.toLazyByteString . BB.stringUtf8

-- | The classes the splitter reads off one character, as the
-- differential's `inspect` asks them: alphanumeric, numeric, lowercase,
-- uppercase (0 / 1 each) and the full lowercase mapping's code points.
charClasses :: Char -> (Int, Int, Int, Int, [Int])
charClasses c = (fromEnum (isRustAlnum c), fromEnum (isRustNumeric c), fromEnum (isRustLower c), fromEnum (isRustUpper c), map ord (rustToLower c))
