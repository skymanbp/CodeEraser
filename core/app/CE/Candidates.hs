-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | candidates.request handler (plan v2.33 W3; design booklet
-- docs/reference/algorithm-track.md §6): the T3 candidate pass over
-- integer facts. The measuring side generates the three index-bound
-- sources (S1 near runs, S3 fingerprints, S4 MinHash bands) and sends
-- every admitted unit as its language, file, key, node count and kind
-- histogram, with those sources' pairs; this family adds the same-key
-- source S2, applies the two admissible bounds (CE.Clone.Prefilter)
-- and, when asked, the exhaustive source S5, and answers the kept pairs
-- and the whole tally. Names, paths and kind strings never cross: every
-- code is request-local. No verdict here — the clone judgment
-- (CE.Clone) judges what this family keeps.
module CE.Candidates (respond) where

import CE.Candidates.Contract (CandReq (..), offence, overCap)
import CE.Candidates.T3 (Answer (..), Tally (..), judgeT3)
import CE.Wire (family)
import Data.Aeson (Value, encode, object, (.=))
import qualified Data.ByteString.Char8 as B8
import qualified Data.ByteString.Lazy as BL

-- | decode → cap → contract → the pass.
respond :: String -> B8.ByteString -> Either (Maybe Value, String, String) B8.ByteString
respond proto = family "candidates" reqId overCap offence (answer proto True) (answer proto False)

-- | The candidates.result object: the kept pairs, S2's pairs per
-- language, the counts. Degraded: no pairs and a zero tally — a request
-- the core refused to judge keeps nothing — the two request tables
-- still counted, and the reason.
answer :: String -> Bool -> CandReq -> B8.ByteString
answer proto degraded req =
  BL.toStrict . encode . object $
    [ "proto" .= proto
    , "type" .= ("candidates.result" :: String)
    , "id" .= reqId req
    , "pairs" .= kept a
    , "keyPairs" .= keyPairs a
    , "counts"
        .= object
          [ "units" .= length (unitRows req)
          , "pairs" .= length (pairRows req)
          , "union" .= union t
          , "crossLanguage" .= crossLanguage t
          , "prunedSize" .= prunedSize t
          , "prunedLabel" .= prunedLabel t
          , "survivors" .= survivors t
          , "s5Windowed" .= s5Windowed t
          , "s5PrunedLabel" .= s5PrunedLabel t
          , "s5Already" .= s5Already t
          , "s5New" .= s5New t
          ]
    , "degraded" .= degraded
    ]
      <> ["reason" .= ("candidates_too_large" :: String) | degraded]
 where
  a
    | degraded = Answer [] (Tally 0 0 0 0 0 0 0 0 0) []
    | otherwise = judgeT3 (exhaustive req) (unitRows req) (pairRows req)
  t = tally a
