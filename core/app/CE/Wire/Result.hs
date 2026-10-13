-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | Every family's reply envelope, spelled once (plan v2.33 W3 for the
-- candidate passes and rank/1; step 11a C28 for every family): proto,
-- the family's `<name>.result` type (CE.Wire.Families) and the echoed
-- id. `resultLine` is the whole line for the families that answer a
-- table set, a counts object and a degraded posture — the family's own
-- fields, `counts` as one object, `degraded`, and, only when degraded,
-- the family's named reason. aeson orders the keys, so a line's bytes
-- do not depend on the order the caller lists them.
module CE.Wire.Result (envelope, faultLine, replyLine, resultLine, resultType, tooLarge, tooLargeOf) where

import CE.Wire.Families (FamilyName (..))
import Data.Aeson (Value, encode, object, (.=))
import Data.Aeson.Types (Pair)
import qualified Data.ByteString.Char8 as B8
import qualified Data.ByteString.Lazy as BL
import Data.Maybe (isJust)

-- | A family's reply type, `<name>.result`.
resultType :: FamilyName -> String
resultType fam = nameWord fam <> ".result"

-- | proto, the family's reply type, the echoed id.
envelope :: String -> FamilyName -> Value -> [Pair]
envelope proto fam rid = ["proto" .= proto, "type" .= resultType fam, "id" .= rid]

-- | One reply line: the envelope and the family's fields.
replyLine :: String -> FamilyName -> Value -> [Pair] -> B8.ByteString
replyLine proto fam rid fields = BL.toStrict (encode (object (envelope proto fam rid <> fields)))

-- | proto, family, the echoed id, the fields, the counts, the reason a
-- degraded reply names (Nothing = judged).
resultLine :: String -> FamilyName -> Value -> [Pair] -> [Pair] -> Maybe String -> B8.ByteString
resultLine proto fam rid fields counts reason =
  replyLine proto fam rid $
    ("counts" .= object counts)
      : ("degraded" .= isJust reason)
      : fields
      <> maybe [] (\why -> ["reason" .= why]) reason

-- | The reason a degraded reply names, `<name>_too_large`, spelled
-- once (plan v2.33 step 11a).
tooLargeOf :: FamilyName -> String
tooLargeOf fam = nameWord fam <> "_too_large"

-- | The reason when the reply is degraded; Nothing when it is judged.
tooLarge :: FamilyName -> Bool -> Maybe String
tooLarge fam degraded = if degraded then Just (tooLargeOf fam) else Nothing

-- | A path-form request the core cannot place (arch/1's focus,
-- structure/1's tree): the fault in place of a judgment, never
-- degraded. The measuring side stops on it.
faultLine :: String -> FamilyName -> Value -> String -> B8.ByteString
faultLine proto fam rid fault = replyLine proto fam rid ["fault" .= fault, "degraded" .= False]
