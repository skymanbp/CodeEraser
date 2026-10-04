-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | moves.request handler (plan v2.33 W3, fourclass L1): the within-pair
-- move judgment over integer line facts (CE.FourClass.Moves.Classify),
-- one answer per changed file pair in request order. The cross-pair
-- judgment (fourclass/1) then reads the leftover runs this family
-- answers. Names and text never cross: lines are content codes, hashes
-- and widths, keys are request-local codes with their fnv1a64.
module CE.FourClass.Moves (respond) where

import CE.FourClass.Moves.Classify (Classified (..), Line (..), classify, unitOf)
import CE.FourClass.Moves.Contract (MovesReq (..), PairIn (..), Side (..), offence, overCap)
import CE.Wire (family)
import CE.Wire.Result (resultLine)
import Data.Aeson (Value, object, (.=))
import Data.Array (listArray)
import qualified Data.ByteString.Char8 as B8

-- | decode → cap → contract → the judgment.
respond :: String -> B8.ByteString -> Either (Maybe Value, String, String) B8.ByteString
respond proto = family "moves" reqId overCap offence (answer proto (Just "moves_too_large")) (answer proto Nothing)

-- | The moves.result object: one answer per pair, the counts of pairs
-- and line rows; degraded (the reason) answers no pairs.
answer :: String -> Maybe String -> MovesReq -> B8.ByteString
answer proto reason req =
  resultLine
    proto
    "moves"
    (reqId req)
    ["pairs" .= maybe (map (encode . judge) (pairsIn req)) (const []) reason]
    ["pairs" .= length (pairsIn req), "lines" .= sum [length (sideLines s) | p <- pairsIn req, s <- [pairBefore p, pairAfter p]]]
    reason
 where
  hashes = listArray (0, length (keyHashes req) - 1) (keyHashes req)
  judge p = classify hashes (side (pairBefore p)) (side (pairAfter p))
  side s =
    ( listArray (0, length (sideLines s) - 1) [Line c h w | [c, h, w] <- sideLines s]
    , map fromInteger (sideChanged s)
    , map unitOf (sideUnits s)
    )

encode :: Classified -> Value
encode c =
  object
    [ "counts" .= counts c
    , "moved" .= moved c
    , "relocated" .= relocated c
    , "declRem" .= declRem c
    , "declAdd" .= declAdd c
    , "dupSpans" .= dupSpans c
    , "runsRem" .= runsRem c
    , "runsAdd" .= runsAdd c
    ]
