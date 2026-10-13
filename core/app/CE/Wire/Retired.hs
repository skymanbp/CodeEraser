-- aeson 2.x object keys are Key, not String; string literals for
-- the key lookups need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | What the request contracts retired, refused by name rather than
-- read and ignored: a producer that still sends one believes it states
-- something the core now owns. The one place each is spelled — the key,
-- how a request is seen to carry it, and its refusal: `judgedMask`
-- (8.0.0; scan/1, graph/1, verdict — the core reads its language
-- table, CE.Wire.Mask), `patterns` (8.0.0; structure/1 — the
-- producer's pre-classified name-pattern codes; the core classifies
-- `patternShapes` itself, CE.Structure.Shape) and erase/1's class 0
-- (superseded by 3 at 2.32.0, retired 4.0.0).
module CE.Wire.Retired (judgedMaskSent, patternsSent, retiredClassZero, retiredMask, retiredPatterns) where

import Data.Aeson (Object)
import qualified Data.Aeson.KeyMap as KM

-- | Did the request carry `judgedMask` / `patterns`?
judgedMaskSent, patternsSent :: Object -> Bool
judgedMaskSent = KM.member "judgedMask"
patternsSent = KM.member "patterns"

-- | The named refusals when the request carried the key, else nothing.
retiredMask, retiredPatterns :: Bool -> Maybe String
retiredMask = retired "judgedMask" "the core reads its language table"
retiredPatterns = retired "patterns" "the core classifies patternShapes"

-- | erase/1's row refusal for a class-0 row.
retiredClassZero :: String
retiredClassZero = "retired class 0 (superseded by 3 at 2.32.0, retired 4.0.0)"

-- | @retired key why sent@: the named refusal when the request carried
-- @key@, else nothing.
retired :: String -> String -> Bool -> Maybe String
retired key why sent
  | sent = Just (key <> ": retired at 8.0.0 \8212 " <> why)
  | otherwise = Nothing
