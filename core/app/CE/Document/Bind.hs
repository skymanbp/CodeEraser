-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | The core spells the repository's strings itself (plan v2.33 W7;
-- design booklet docs/reference/algorithm-track.md §3 point 3): a
-- document request that carries `strings` — the measuring side's
-- strings by reference class, one JSON array level per integer the
-- class's reference carries — is answered with every reference
-- `{"$": [class, integers…]}` of the document replaced by its string
-- and every console line's `{}` holes filled, `[stream, text]`; the
-- measuring side prints what it gets. The binding is the one
-- cli/src/document.rs `bind` / `resolved` and document/lines.rs
-- `bound` performed (frozen in cli/tests/unit/document/frozen/),
-- spelled here: a reference is an object whose one key is `$`, holding
-- a class and integers; a line's holes are counted as Rust's
-- `str::matches("{}")` counts them and filled left to right, one per
-- reference, each reference resolving to a string. A request without
-- `strings` (the PreToolUse guard; query until its program crosses)
-- is answered as before, references and holes for the measuring side.
module CE.Document.Bind (Resolver, Strings (..), bindDocument, bindLine, entry, indexed, nested, textOf) where

import CE.Document.Contract (DocReq (..))
import CE.Text (Line (..), Piece (..))
import Data.Aeson (Result (..), Value (..), encode, fromJSON, toJSON)
import qualified Data.ByteString.Lazy.Char8 as BL8
import qualified Data.Aeson.KeyMap as KM
import Data.Foldable (toList)
import qualified Data.IntMap.Strict as IM
import Data.List (isPrefixOf)
import qualified Data.Map.Strict as M

-- | A reference's class and integers to its text; Nothing when the
-- request holds none (an integer outside the list, a class the family
-- does not hold).
type Resolver = String -> [Integer] -> Maybe String

-- | The request's strings indexed once: every array an IntMap, so a
-- reference is a lookup per integer however long the lists run.
data Strings = Leaf Value | Branch (IM.IntMap Strings)

indexed :: DocReq -> M.Map String Strings
indexed = maybe M.empty (M.map node) . dStrings
 where
  node v = case v of
    Array xs -> Branch (IM.fromList (zip [0 ..] (map node (toList xs))))
    other -> Leaf other

-- | A value as Rust reads a string out of it (a JSON string only).
textOf :: Value -> Maybe String
textOf v = case (v, fromJSON v) of
  (String _, Success t) -> Just t
  _ -> Nothing

-- | A class's entry at an index (a list of the strings that are not
-- strings themselves: a flow file's units, by index).
entry :: M.Map String Strings -> String -> [Integer] -> Maybe Strings
entry strings cls ints = M.lookup cls strings >>= go ints
 where
  go is s = case (is, s) of
    ([], _) -> Just s
    (i : rest, Branch m) | i >= 0, i <= toInteger (maxBound :: Int) -> IM.lookup (fromInteger i) m >>= go rest
    _ -> Nothing

-- | The plain lookup: the class's value, one array level per integer
-- (an integer inside the array), a string at the end — `document::at`
-- for a one-integer class, the value itself for a class with none.
nested :: M.Map String Strings -> Resolver
nested strings cls ints = case entry strings cls ints of
  Just (Leaf v) -> textOf v
  _ -> Nothing

-- | Every reference of the document replaced by its string (`bind`),
-- or the first reference that resolves to none.
bindDocument :: Resolver -> Value -> Either String Value
bindDocument r v = case v of
  Array xs -> Array <$> traverse (bindDocument r) xs
  Object o
    | KM.size o == 1, Just ref <- KM.lookup "$" o -> toJSON <$> resolved r ref
    | otherwise -> Object <$> traverse (bindDocument r) o
  other -> Right other

-- | One reference's string (`resolved`): `[class, integers…]`, every
-- integer within i64 / u64 as Rust reads it.
resolved :: Resolver -> Value -> Either String String
resolved r ref = case ref of
  Array parts | (c : ints) <- toList parts, Just cls <- textOf c -> do
    is <- maybe (Left ("document: " <> shown <> " holds a non-integer")) Right (traverse integral ints)
    maybe (Left ("document: no string for " <> shown)) Right (r cls is)
  _ -> Left ("document: a reference is not [class, integers…]: " <> shown)
 where
  shown = json ref
  integral x = case (x, fromJSON x) of
    (Number _, Success i) | i >= -(2 ^ (63 :: Int)), i < (2 ^ (64 :: Int) :: Integer) -> Just i
    _ -> Nothing

-- | One console line with every hole filled (`bound`): as many holes as
-- references, each reference an object whose one key is `$`.
bindLine :: Resolver -> Line -> Either String Value
bindLine r (Line s (Piece text refs _))
  | holes /= length refs = Left (show holes <> " hole(s) and " <> show (length refs) <> " reference(s)")
  | otherwise = do
      fills <- traverse one refs
      pure (toJSON [toJSON s, toJSON (concat (zipWith (<>) pieces (fills <> [""])))])
 where
  pieces = splitHoles text
  holes = length pieces - 1
  one v = case v of
    Object o | KM.size o == 1, Just ref <- KM.lookup "$" o -> resolved r ref
    _ -> Left (json v <> " is not a reference")

-- | A value as serde_json displays it in the Rust refusals: compact
-- JSON (the classes and integers are ASCII, so the bytes are the text).
json :: Value -> String
json = BL8.unpack . encode

-- | The text cut at every `{}`, left to right (Rust's `split_once`
-- repeated; `matches` counts the same non-overlapping holes).
splitHoles :: String -> [String]
splitHoles s = go s ""
 where
  go rest acc
    | "{}" `isPrefixOf` rest = reverse acc : go (drop 2 rest) ""
    | otherwise = case rest of
        [] -> [reverse acc]
        c : cs -> go cs (c : acc)
