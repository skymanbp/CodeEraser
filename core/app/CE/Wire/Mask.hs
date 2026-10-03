-- | The judged-language set the request contracts read their language
-- codes against: scan/1's naming rows and graph/1's unres rows. Until
-- 8.0.0 (plan v2.32 step 6) a producer sent it as `judgedMask` (7.2.0)
-- and a request that sent none was read against the legacy seven;
-- since plan v2.32 step 2 the producer's value was this table's own
-- column, so the core reads its language table (CE.Lang) and the key
-- is retired — a request that still carries it is refused by name.
-- Split from CE.Wire when plan v2.30 step 7b's target table pushed that
-- module past the E01 300-line line (the CE.Structure.Request
-- precedent): the set is one concept with two readers, and neither is
-- the wire skeleton's business.
module CE.Wire.Mask (judgedLang, judgedMask, retiredMask) where

import CE.Lang (languages)
import CE.Lang.Spec (lgCode, lgJudged)
import CE.Wire.Retired (retired)
import Data.Bits (setBit, testBit)

-- | The judged-language set as a bitmask (bit = language wire code):
-- the language table's `judged` column (LangProps pins it).
judgedMask :: Integer
judgedMask = foldl' setBit 0 [lgCode l | l <- languages, lgJudged l]

-- | Is @lang@ a judged language? A negative code is in no set, and a
-- code past bit 62 could be in no i64 mask.
judgedLang :: Integer -> Bool
judgedLang lang = lang >= 0 && lang < 63 && testBit judgedMask (fromInteger lang)

-- | The retired key (8.0.0): a request that still carries
-- `judgedMask` comes from a producer that believes it declares the
-- set — refused by name rather than read and ignored. Checked by the
-- three families that carried it (scan/1, graph/1, verdict).
retiredMask :: Bool -> Maybe String
retiredMask = retired "judgedMask" "the core reads its language table"
