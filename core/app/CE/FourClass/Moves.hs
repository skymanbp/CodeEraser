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
import CE.Wire (Respond, tabled)
import qualified CE.Wire.Families as F
import Data.Aeson (Value, object, (.=))
import Data.Aeson.Types (Pair)
import Data.Array (listArray)

-- | decode → cap → contract → the judgment.
respond :: Respond
respond = tabled F.moves reqId overCap offence answer

-- | The moves.result object: one answer per pair, the counts of pairs
-- and line rows; degraded (the reason) answers no pairs.
answer :: Bool -> MovesReq -> ([Pair], [Pair])
answer degraded req =
  ( ["pairs" .= if degraded then [] else map (encode . judge) (pairsIn req)]
  , ["pairs" .= length (pairsIn req), "lines" .= sum [length (sideLines s) | p <- pairsIn req, s <- [pairBefore p, pairAfter p]]]
  )
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
