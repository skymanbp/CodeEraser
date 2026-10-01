-- | Reading an assembled document back (plan v2.32 step 5): a family's
-- console lines are a rendering of its document — the measuring side's
-- printers read the report it assembled, the core's read the one it
-- assembles — so each family's lines walk the same Value the face
-- would print, with these few readers. A field the document does not
-- hold reads as null / empty / 0 / False. It hands a lines module the
-- request contract and the catalogue machinery too: one import, the
-- same for every family.
module CE.Document.Read (module CE.Document.Contract, module CE.Text, Result (..), Value (..), arr, bool, fromJSON, int, key, nullAt, txt) where

import CE.Document.Contract
import CE.Text
import Data.Aeson (Result (..), Value (..), fromJSON)
import qualified Data.Aeson.Key as Key
import qualified Data.Aeson.KeyMap as KM
import Data.Foldable (toList)
import Data.Maybe (fromMaybe)

-- | A field (null when absent).
key :: String -> Value -> Value
key k v = case v of
  Object o -> fromMaybe Null (KM.lookup (Key.fromString k) o)
  _ -> Null

arr :: String -> Value -> [Value]
arr k v = case key k v of
  Array xs -> toList xs
  _ -> []

int :: String -> Value -> Integer
int k v = case key k v of
  Number n -> round n
  _ -> 0

bool :: String -> Value -> Bool
bool k v = key k v == Bool True

txt :: String -> Value -> String
txt k v = case fromJSON (key k v) of
  Success t -> t
  _ -> ""

nullAt :: String -> Value -> Bool
nullAt k v = key k v == Null
