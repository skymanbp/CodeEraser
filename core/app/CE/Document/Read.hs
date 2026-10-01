-- | Reading an assembled document back (plan v2.32 step 5): a family's
-- console lines are a rendering of its document — the measuring side's
-- printers read the report it assembled, the core's read the one it
-- assembles — so each family's lines walk the same Value the face
-- would print, with these few readers. A field the document does not
-- hold reads as null / empty / 0 / False. It hands a lines module the
-- request contract and the catalogue machinery too: one import, the
-- same for every family.
module CE.Document.Read (module CE.Document.Contract, module CE.Text, Result (..), Say, Value (..), arr, bool, emitted, fields, fromJSON, int, key, nullAt, spoken, txt) where

import CE.Document.Contract
import CE.Text
import Data.Aeson (Result (..), Value (..), fromJSON)
import qualified Data.Aeson.Key as Key
import qualified Data.Aeson.KeyMap as KM
import Data.Foldable (toList)
import Data.List (intercalate)
import Data.Maybe (fromMaybe)

-- | A catalogue's sentence in the request's language: a key and the
-- fills of its holes.
type Say = String -> [Fill] -> Piece

-- | A family with its console lines and its veto, both handed the
-- sentences of its catalogue in the request's language and the
-- document the family assembles — a Lines module renders, it does not
-- rebuild either.
spoken :: Catalogue -> (Say -> Value -> DocReq -> [Line]) -> (Value -> DocReq -> Bool) -> DocFamily -> DocFamily
spoken cat ls ex fam =
  fam
    { dfText = cat
    , dfLines = \lang req -> ls (phrase cat lang) (dfAssemble fam req) req
    , dfExit = \req -> ex (dfAssemble fam req) req
    }

-- | The shared report throat's console (cli/src/report.rs `emit`, the
-- clone and docdup families): one `hit` line per row of the document's
-- hit table, then the `summary` sentence over the counters it names
-- (in the order given) with the raw tail of the rest.
emitted :: (String -> [Fill] -> Piece) -> (Value -> [Fill]) -> String -> [String] -> Value -> [Line]
emitted say hit hits named doc =
  map (line 0) ([say "hit" (hit h) | h <- arr hits doc] <> [say "summary" (map (N . (`int` counts)) named <> [P (rawTail named counts)])])
 where
  counts = key "counts" doc

-- | The raw tail `emit` prints after the summary sentence: every
-- counter of the counts object the sentence does not name, `k n` in key
-- order after ` | `, nothing when none is left — English under both
-- languages.
rawTail :: [String] -> Value -> Piece
rawTail named counts = case [k <> " " <> show (round n :: Integer) | (k, Number n) <- fields counts, k `notElem` named] of
  [] -> plain ""
  rest -> plain (" | " <> intercalate ", " rest)

-- | An object's fields in key order (none when it is not one).
fields :: Value -> [(String, Value)]
fields v = case v of
  Object o -> [(Key.toString k, x) | (k, x) <- KM.toAscList o]
  _ -> []

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
