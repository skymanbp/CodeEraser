-- aeson 2.x object keys are Key, not String; string literals need
-- OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | The verdict wire batteries' road into CE.Verdict.respond since the
-- extent law (10.0.0) made the four tables required: a request line
-- that carries none of them gets the neutral ones its own tables imply
-- — every continuous row outside the universe (a function row on line
-- 1), no duplicate span, no reference line, and a ten-line count for
-- every tier file — so the batteries keep judging what they judged
-- (knob echoes, ratchet, refusals, fences) and none of them states a
-- line extent it never meant. A line that carries any of the four is
-- sent as it is; the contract judges it.
module VerdictTables (neutral, respondV) where

import CE.Verdict (respond)
import CE.Wire (Respond)
import Data.Aeson
import qualified Data.Aeson.KeyMap as KM
import qualified Data.ByteString.Char8 as B8
import qualified Data.ByteString.Lazy as BL

respondV :: Respond
respondV v line = respond v (maybe line filled (decodeStrict line))
 where
  filled o
    | any (`KM.member` o) ["contSpans", "dupSpans", "refLines", "proseLines"] = line
    | otherwise = B8.concat (BL.toChunks (encode (Object (o <> KM.fromList (tables o)))))

-- | A request with the neutral tables added (a battery that sets one of
-- them itself starts from here).
neutral :: Value -> Value
neutral (Object o) = Object (o <> KM.fromList (tables o))
neutral v = v

-- | The neutral tables of a request object.
tables :: Object -> [(Key, Value)]
tables o =
  [ ("contSpans", toJSON [spanOf r | r <- rowsOf "continuous"])
  , ("dupSpans", toJSON ([] :: [[Integer]]))
  , ("refLines", toJSON ([] :: [[Integer]]))
  , ("proseLines", toJSON [[u, 10] | u <- [0 .. toInteger (length (rowsOf "tier")) - 1]])
  ]
 where
  rowsOf k = case KM.lookup k o >>= decodeRows of
    Just rs -> rs
    Nothing -> []
  decodeRows x = case fromJSON x of
    Success rs -> Just (rs :: [[Integer]])
    Error _ -> Nothing
  spanOf r = if take 1 (drop 1 r) == [1] then [-1, 1, 1] else [-1, 0, 0 :: Integer]
