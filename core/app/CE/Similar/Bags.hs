-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | bags.request handler (plan v2.33 W6; booklet 15 §1): the six-channel
-- bag of each code unit, read off the facts the measuring side took from
-- the tree, and the bag of a free-text query. Moved from
-- cli/src/similar/bag.rs (`build`, `name_words`, `arity`, `shape` — every
-- function below named after the Rust one it replaces) and
-- cli/src/similar/query.rs (`text_terms`); the words go through
-- CE.Similar.Terms. The tree stays on the measuring side: the unit key,
-- its kind word and whether a callable declares a return (P), the callee
-- spellings of its own body (C), the literal kinds (L), the structure
-- histogram (S) and the lines of the comments and docstrings it owns (D)
-- cross as text and integers; terms come back as [term, channel, tf].
--
-- Request:
--   units    [[key, kind, ret, [callee], [literal], [[structKind, n]], [doc line]]]
--            ret null (not a callable), 0 or 1; structKind 0 ≤ k < 2^64 strictly
--            ascending, 1 ≤ n < 2^32
--   texts    [text]                    free-text queries
--   inspect  [code point]              the character classes (the differential's)
-- Reply: bags (one per unit) and texts (one per text), each [[term,
-- channel, tf]] ascending by term; chars [[alnum, numeric, lower, upper,
-- [lowercase]]] per inspected code point; counts units / texts / terms.
module CE.Similar.Bags (BagsReq (..), bagsCap, overCap, respond) where

import CE.Similar.Bags.Contract (BagsReq (..), Unit, bagsCap, offence, overCap)
import CE.Similar.Terms
import CE.Wire (family)
import CE.Wire.Result (resultLine)
import Data.Aeson (Value, (.=))
import qualified Data.ByteString as B
import qualified Data.ByteString.Char8 as B8
import Data.Char (chr, isDigit)
import Data.List (isPrefixOf)
import qualified Data.Map.Strict as M
import Data.Word (Word64)

respond :: String -> B8.ByteString -> Either (Maybe Value, String, String) B8.ByteString
respond proto = family "bags" reqId overCap offence (answer proto True) (answer proto False)

answer :: String -> Bool -> BagsReq -> B8.ByteString
answer proto degraded r =
  resultLine
    proto
    "bags"
    (reqId r)
    ["bags" .= bags, "texts" .= textBags, "chars" .= chars]
    ["units" .= length bags, "texts" .= length textBags, "terms" .= sum (map length (bags <> textBags))]
    (if degraded then Just "bags_too_large" else Nothing)
 where
  bags = if degraded then [] else map (rows . build) (units r)
  textBags = if degraded then [] else map (rows . textTerms) (texts r)
  chars = if degraded then [] else map (charClasses . chr . fromInteger) (inspect r)
  rows m = [[toInteger t, toInteger c, toInteger n] | (t, (c, n)) <- M.toList m]

-- | term → (channel, tf): a term met again adds its count and keeps the
-- channel it was first met under (Rust's `entry(term).or_insert((ch,
-- 0)).1 += n`).
type Bag = M.Map Word64 (Int, Int)

add :: Bag -> (Word64, Int, Int) -> Bag
add m (t, c, n) = M.insertWith (\(_, new) (c0, old) -> (c0, old + new)) t (c, n) m

-- | One unit's bag, its terms met in the Rust's order: name, shape,
-- callee, literal, structure, doc.
build :: Unit -> Bag
build (key, kind, ret, callees, literals, struct, docs) =
  foldl add M.empty $
    [(wordTerm chanName w, chanName, 1) | w <- nameWords key]
      <> [(featureTerm chanShape (utf8 s), chanShape, 1) | s <- shape key kind ret]
      <> [(wordTerm chanCallee w, chanCallee, 1) | c <- callees, w <- splitIdent c]
      <> [(featureTerm chanLiteral (utf8 l), chanLiteral, 1) | l <- literals]
      <> [(featureTerm chanStructure (le64 k), chanStructure, fromInteger n) | [k, n] <- struct]
      <> [(wordTerm chanDoc w, chanDoc, 1) | line <- docs, w <- proseWords line]

-- | The eight little-endian bytes of a structure kind (`kind.to_le_bytes()`).
le64 :: Integer -> B.ByteString
le64 k = B.pack [fromInteger ((k `div` (256 ^ i)) `mod` 256) | i <- [0 .. 7 :: Int]]

-- | The arity suffix of a function key (`name/3` → the base and "3"): the
-- text after the LAST slash, non-empty and all ASCII digits.
arity :: String -> Maybe (String, String)
arity key = case break (== '/') (reverse key) of
  (digits, '/' : base) | not (null digits) && all isDigit digits -> Just (reverse base, reverse digits)
  _ -> Nothing

-- | The words of a unit key: the arity dropped, an `impl T for U` its two
-- keywords, an anonymous unit none at all; splitIdent does the rest.
nameWords :: String -> [String]
nameWords key
  | base == "(anonymous)" = []
  | "impl " `isPrefixOf` base = filter (`notElem` ["impl", "for"]) (splitIdent base)
  | otherwise = splitIdent base
 where
  base = maybe key fst (arity key)

-- | Shape features: the unit's kind word, its arity, and for a callable
-- whether it declares a return.
shape :: String -> String -> Maybe Integer -> [String]
shape key kind ret = ("k:" <> kind) : ["p:" <> n | Just (_, n) <- [arity key]] <> ["ret:" <> show f | Just f <- [ret]]

-- | Free text as a query bag: its prose words as NAME and DOC evidence.
textTerms :: String -> Bag
textTerms t = foldl add M.empty [(wordTerm c w, c, 1) | w <- proseWords t, c <- [chanName, chanDoc]]
