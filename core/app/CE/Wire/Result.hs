-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | One `<kind>.result` line for the families that answer a table
-- set, a counts object and a degraded posture (plan v2.33 W3: the
-- candidate passes and rank/1). The envelope (proto, type, id), the
-- family's own fields, `counts` as one object, `degraded`, and — only
-- when degraded — the family's named reason. aeson orders the keys, so
-- the line's bytes do not depend on the order the caller lists them.
module CE.Wire.Result (resultLine) where

import Data.Aeson (Value, encode, object, (.=))
import Data.Aeson.Types (Pair)
import qualified Data.ByteString.Char8 as B8
import qualified Data.ByteString.Lazy as BL
import Data.Maybe (isJust)

-- | proto, kind, the echoed id, the fields, the counts, the reason a
-- degraded reply names (Nothing = judged).
resultLine :: String -> String -> Value -> [Pair] -> [Pair] -> Maybe String -> B8.ByteString
resultLine proto kind rid fields counts reason =
  BL.toStrict . encode . object $
    ("proto" .= proto)
      : ("type" .= (kind <> ".result"))
      : ("id" .= rid)
      : ("counts" .= object counts)
      : ("degraded" .= isJust reason)
      : fields
      <> maybe [] (\why -> ["reason" .= why]) reason
