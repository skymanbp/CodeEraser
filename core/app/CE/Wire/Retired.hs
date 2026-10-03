-- | The request keys 8.0.0 retired (plan v2.32 step 6), refused by
-- name rather than read and ignored: a producer that still sends one
-- believes it states something the core now owns. One spelling for
-- every retired key, so the refusals read alike: `judgedMask` (scan/1,
-- graph/1, verdict — the core reads its language table, CE.Wire.Mask)
-- and `patterns` (structure/1 — the producer's pre-classified
-- name-pattern codes; the core classifies `patternShapes` itself,
-- CE.Structure.Shape).
module CE.Wire.Retired (retired) where

-- | @retired key why sent@: the named refusal when the request carried
-- @key@, else nothing.
retired :: String -> String -> Bool -> Maybe String
retired key why sent
  | sent = Just (key <> ": retired at 8.0.0 \8212 " <> why)
  | otherwise = Nothing
