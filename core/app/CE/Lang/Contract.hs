-- | The `tables.request` contract (plan v2.32 step 1, VERSIONING
-- 7.7.0): the request is its envelope and nothing else. The package has
-- no parameter — no language to pick, no table to filter — so a key
-- beyond the envelope's three is a client asking for something the
-- family does not offer, refused by name rather than ignored (the
-- envelope rule that ignores unknown fields is for forward
-- compatibility of families that read a body; this one reads none).
module CE.Lang.Contract (offence) where

import Data.Aeson (Object)
import qualified Data.Aeson.Key as K
import qualified Data.Aeson.KeyMap as KM

-- | The first key the envelope does not own, as the refusal names it.
offence :: Object -> Maybe String
offence o = case filter (`notElem` envelope) (map K.toString (KM.keys o)) of
  k : _ -> Just ("tables: unexpected key " <> k)
  [] -> Nothing
 where
  envelope = ["type", "id", "proto"]
