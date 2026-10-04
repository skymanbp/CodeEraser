-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | docpairs.request handler (plan v2.33 W3): the docdup coarse filter
-- (CE.Docdup.Coarse) over the segments the measuring side can send —
-- each as its strictly ascending shingle set, request-local by position
-- — answering the candidate pairs `[a, b]` ascending and the filter's
-- tally. No verdict: the docdup judgment (docdup/1) judges what this
-- family keeps.
module CE.Docdup.Pairs (PairsReq (..), overCap, respond) where

import CE.Candidates.Lsh (sets)
import CE.Docdup (setShape)
import CE.Docdup.Coarse (Coarse (..), coarse)
import CE.Docdup.Cost (docCorpusCap, docSetCap)
import CE.Wire (family)
import CE.Wire.Result (resultLine)
import Data.Aeson (FromJSON (..), Value, withObject, (.!=), (.:), (.:?), (.=))
import qualified Data.ByteString.Char8 as B8
import Data.Foldable (asum)
import qualified Data.IntSet as IS

data PairsReq = PairsReq {reqId :: Value, reqSets :: [[Integer]]}

instance FromJSON PairsReq where
  parseJSON = withObject "PairsReq" $ \o -> PairsReq <$> o .: "id" <*> o .:? "sets" .!= []

-- | decode → cap → contract → the filter.
respond :: String -> B8.ByteString -> Either (Maybe Value, String, String) B8.ByteString
respond proto = family "docpairs" reqId overCap offence (answer proto True) (answer proto False)

-- | A set over the per-set ceiling, or the sets together over the
-- corpus ceiling.
overCap :: PairsReq -> Bool
overCap req =
  any (\s -> toInteger (length s) > docSetCap) (reqSets req)
    || toInteger (sum (map length (reqSets req))) > docCorpusCap

-- | The first malformed set in request order.
offence :: PairsReq -> Maybe String
offence req = asum (zipWith setShape [0 ..] (reqSets req))

-- | The docpairs.result object: the kept pairs, the counts; degraded
-- answers no pairs and a zero tally.
answer :: String -> Bool -> PairsReq -> B8.ByteString
answer proto degraded req =
  resultLine
    proto
    "docpairs"
    (reqId req)
    ["pairs" .= [[k `div` n, k `mod` n] | k <- IS.toAscList (found c)]]
    (("sets" .= n) : zipWith (.=) ["lshPairs", "seedPairs", "hotBands", "hotShingles"] tally)
    (if degraded then Just "docpairs_too_large" else Nothing)
 where
  tally = map ($ c) [lshPairs, seedPairs, hotBands, hotShingles]
  n = length (reqSets req)
  c
    | degraded = Coarse IS.empty 0 0 0 0
    | otherwise = coarse (sets (reqSets req))
