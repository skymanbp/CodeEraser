-- | The judged-language set as a request fact (7.2.0, plan v2.30
-- step 1): the bitmask a producer sends as `judgedMask`, the legacy
-- set a request that sends none is read against, and the mask's own
-- contract. Split from CE.Wire when plan v2.30 step 7b's target
-- table pushed that module past the E01 300-line line (the
-- CE.Structure.Request precedent in the same step): the set is one
-- concept with two readers — scan/1's naming rows and graph/1's
-- unres rows — and neither is the wire skeleton's business.
module CE.Wire.Mask (judgedLang, legacyJudged, maskOffence) where

import Data.Bits (testBit)
import Data.Maybe (fromMaybe)

-- | The judged-language set before it rode the wire (7.2.0): codes
-- 0..6, the bound `lang > 6` two validators spelled until plan v2.30
-- made the set a request fact. A request that declares no mask is
-- judged against exactly this one, byte for byte as before.
legacyJudged :: Integer
legacyJudged = 127

-- | Is @lang@ in the judged set the request declared (the legacy set
-- when it declared none)? A negative code is in no set, and a code
-- past bit 62 could be in no i64 mask a producer can send.
judgedLang :: Maybe Integer -> Integer -> Bool
judgedLang mask lang =
  lang >= 0 && lang < 63 && testBit (fromMaybe legacyJudged mask) (fromInteger lang)

-- | The mask's own contract: absent is the legacy road, else a
-- non-negative i64 — the only shape the producer's `judged_mask`
-- can take, so anything else is a foreign client, refused by name.
maskOffence :: Maybe Integer -> Maybe String
maskOffence (Just m)
  | m < 0 = Just "judgedMask: negative"
  | m >= 9223372036854775808 = Just "judgedMask: outside i64"
maskOffence _ = Nothing
