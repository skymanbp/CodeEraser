-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | What the document batteries share (plan v2.32 steps 3 and 4): a
-- request's document through the real respond, the empty request a
-- family's statement implies, a value walked by key, every reference
-- in a document, and the four legs every family takes alike — the
-- empty document is the catalogue's and has the fields the battery's
-- statement table names, every reference in the seeded documents is a
-- stated class inside its ranges, and the same request assembles to
-- the same bytes. DocumentProps (step 3's five) and DocumentProps4
-- (step 4's seven) each add the legs only their families have.
module DocumentHarness (at, documentOf, emptiesHeld, emptyRequest, familiesNamed, fieldsHeld, int, ints, items, judgedBy, path, rankOf, refsHeld, refsIn, sameBytes) where

import CE.Document (catalogue, emptyOf, families, respond)
import CE.Document.Contract (DocFamily (..), Spec (..), Table (..))
import Data.Aeson
import qualified Data.Aeson.Key as Key
import qualified Data.Aeson.KeyMap as KM
import Data.Foldable (toList)
import Data.Maybe (fromMaybe, mapMaybe)
import DocumentGen (docRequest)
import WireHarness (field, replyObjWith)

-- | The catalogue's families of the given names, in catalogue order.
familiesNamed :: [String] -> [DocFamily]
familiesNamed names = [f | f <- families, dfName f `elem` names]

-- | One request's reply document, through the real respond.
documentOf :: Value -> Maybe Value
documentOf r = replyObjWith respond r >>= (`field` "document")

-- | The empty request of a family: every range and fact 0, no row,
-- and — when the family states a reason — `why` 1 and the reason 0.
emptyRequest :: DocFamily -> Value
emptyRequest f = docRequest (dfName f) ranges [(tName t, []) | t <- spTables sp] [(n, 0) | (n, _) <- spFacts sp] reason
 where
  sp = dfSpec f
  stated = "why" `elem` spRanges sp
  ranges = [(r, if r == "why" then 1 else 0) | r <- spRanges sp]
  reason = if stated then Just 0 else Nothing

at :: Value -> String -> Maybe Value
at v k = case v of
  Object o -> KM.lookup (Key.fromString k) o
  _ -> Nothing

-- | A path of keys down a value.
path :: [String] -> Value -> Maybe Value
path ks v = foldl (\acc k -> acc >>= (`at` k)) (Just v) ks

ints :: Maybe Value -> [[Integer]]
ints = fromMaybe [] . (>>= \v -> case fromJSON v of Success r -> Just r; _ -> Nothing)

int :: Maybe Value -> Integer
int v = case v >>= \x -> case fromJSON x of Success n -> Just n; _ -> Nothing of
  Just n -> n
  Nothing -> -1

-- | A key's array elements (none when it is not an array).
items :: Value -> String -> [Value]
items v k = case at v k of
  Just (Array xs) -> toList xs
  _ -> []

emptiesHeld :: [DocFamily] -> Bool
emptiesHeld = all ok
 where
  ok f = documentOf (emptyRequest f) == Just (emptyOf f) && path [dfName f, "empty"] catalogue == Just (emptyOf f)

-- | Each family's empty document holds exactly the fields the table
-- names (`family key type`; a type ending in `?` may also be null).
fieldsHeld :: String -> [DocFamily] -> Bool
fieldsHeld table = all stated
 where
  stated f =
    let want = [(k, t) | [fam, k, t] <- map words (lines table), fam == dfName f]
        doc = emptyOf f
        keys = case doc of Object o -> KM.size o; _ -> 0
     in not (null want) && keys == length want && all (\(k, t) -> maybe False (fits t . kind) (at doc k)) want
  fits t k = t == k || (last t == '?' && (k == init t || k == "null"))
  kind v = case v of
    String _ -> "string"
    Object o | KM.member "$" o -> "ref"
    Object _ -> "object"
    Array _ -> "array"
    Number _ -> "number"
    Bool _ -> "bool"
    Null -> "null"

-- | Every seeded request of the families with its document.
judgedBy :: (String -> [Value]) -> [DocFamily] -> [(DocFamily, Value, Value)]
judgedBy requests fams = [(f, r, d) | f <- fams, r <- requests (dfName f), Just d <- [documentOf r]]

-- | Every `{"$": [class, ints…]}` in a value.
refsIn :: Value -> [(String, [Integer])]
refsIn v = case v of
  Object o
    | [("$", Array xs)] <- KM.toList o, (c : rest) <- toList xs, Success cls <- fromJSON c -> [(cls, mapMaybe asInt rest)]
    | otherwise -> concatMap refsIn (KM.elems o)
  Array xs -> concatMap refsIn (toList xs)
  _ -> []
 where
  asInt x = case fromJSON x of Success n -> Just n; _ -> Nothing

-- | Every reference names a stated class with its integers inside the
-- universes the class states.
refsHeld :: [(DocFamily, Value, Value)] -> Bool
refsHeld = all held
 where
  held (f, r, d) = all (stated f r) (refsIn d)
  stated f r (cls, xs) = case lookup cls (spRefs (dfSpec f)) of
    Just args -> length args == length xs && and (zipWith (inside r) args xs)
    Nothing -> False
  inside r u x = maybe True (\name -> x >= 0 && x < int (path ["ranges", name] r)) u

-- | The same request, assembled twice, to the same bytes.
sameBytes :: [(DocFamily, Value, Value)] -> Bool
sameBytes = all (\(_, r, d) -> documentOf r == Just d && (encode <$> documentOf r) == Just (encode d))

-- | A reference's place in a rank table of the request: its first
-- integer's row (−1 when absent).
rankOf :: String -> Value -> Value -> Integer
rankOf table r ref = case refsIn ref of
  [(_, x : _)] -> fromMaybe (-1) (lookup x [(s, p) | [s, p] <- ints (path ["rows", table] r)])
  _ -> -1
