-- | Optional row tables read by key (plan v2.33 W3): each absent table
-- is empty, the tables come back in the order of their keys — one
-- reading for the requests whose tables all default empty.
module CE.Wire.Tables (optTables) where

import Data.Aeson (Object, (.!=), (.:?))
import Data.Aeson.Key (Key)
import Data.Aeson.Types (Parser)

optTables :: Object -> [Key] -> Parser [[[Integer]]]
optTables o = traverse (\k -> o .:? k .!= [])
