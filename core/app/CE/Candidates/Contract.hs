-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | The candidates.request shape and its boundary contract (plan v2.33
-- W3): the admitted units, each `[lang, file, key, nodes, kind, count,
-- kind, count, …]` (language, file and key request-local codes; the
-- kind histogram's kinds request-local codes, strictly ascending,
-- counts summing to the unit's nodes), the pairs the measuring side's
-- three generators found as `[a, b, sources]` (a < b, one language,
-- strictly ascending, sources a non-empty subset of the S1/S3/S4 bits),
-- and whether the exhaustive source runs. The first offender in
-- request order is named as `<table> <i>: <reason>`.
module CE.Candidates.Contract (CandReq (..), offence, overCap) where

import CE.Candidates.Cost (candidatePairCap, candidateUnitCap, sentSources)
import CE.Wire (rowCheck, tableOffence)
import Data.Aeson (FromJSON (..), Value, withObject, (.!=), (.:), (.:?))
import Data.Array (Array, listArray, (!))
import Data.Bits ((.&.))
import Data.Foldable (asum)

-- | The request: id, the unit rows, the generated pairs, and the
-- exhaustive switch (absent = off: the frozen four-source pass).
data CandReq = CandReq
  { reqId :: Value
  , unitRows :: [[Integer]]
  , pairRows :: [[Integer]]
  , exhaustive :: Bool
  }

instance FromJSON CandReq where
  parseJSON = withObject "CandReq" $ \o ->
    CandReq
      <$> o .: "id"
      <*> o .:? "units" .!= []
      <*> o .:? "pairs" .!= []
      <*> o .:? "exhaustive" .!= False

-- | Two dimensions, two ceilings.
overCap :: CandReq -> Bool
overCap req =
  toInteger (length (unitRows req)) > candidateUnitCap
    || toInteger (length (pairRows req)) > candidatePairCap

-- | Every unit row's own shape first, then the pair table (whose range
-- and language checks read the unit rows).
offence :: CandReq -> Maybe String
offence req =
  asum
    [ asum (zipWith unitShape [0 ..] units)
    , tableOffence "pair" (take 2) (pairShape langs) (pairRows req)
    ]
 where
  units = unitRows req
  langs = listArray (0, length units - 1) (map (take 1) units)

-- | [lang, file, key, nodes, kind, count, …]: the codes non-negative,
-- the histogram's kinds strictly ascending and non-negative, its counts
-- positive and summing to nodes.
unitShape :: Int -> [Integer] -> Maybe String
unitShape i row = case row of
  (lang : fileCode : keyCode : nodes : hist)
    | lang < 0 -> label "negative language"
    | fileCode < 0 -> label "negative file"
    | keyCode < 0 -> label "negative key"
    | nodes < 1 -> label "non-positive nodes"
    | odd (length hist) -> label "histogram not [kind,count] pairs"
    | any (< 0) kinds -> label "negative kind"
    | any (< 1) counts -> label "non-positive count"
    | or (zipWith (>=) kinds (drop 1 kinds)) -> label "kinds not strictly ascending"
    | sum counts /= nodes -> label "histogram does not sum to nodes"
    | otherwise -> Nothing
   where
    kinds = everyOther hist
    counts = everyOther (drop 1 hist)
  _ -> label "malformed unit (need [lang,file,key,nodes,kind,count,...])"
 where
  label why = Just ("unit " <> show i <> ": " <> why)

-- | The even positions of a list.
everyOther :: [a] -> [a]
everyOther xs = case xs of
  (x : _ : rest) -> x : everyOther rest
  [x] -> [x]
  [] -> []

-- | [a, b, sources]: endpoints in range and ordered, one language,
-- sources a non-empty subset of the S1/S3/S4 bits.
pairShape :: Array Int [Integer] -> Int -> [Integer] -> Maybe String
pairShape langs = rowCheck "pair" "malformed pair (need [a,b,sources])" 3 checks
 where
  n = toInteger (length langs)
  langOf k = langs ! fromInteger k
  checks row = case row of
    [a, b, s]
      | a < 0 || b >= n -> Just "endpoint out of range"
      | a >= b -> Just "endpoints not ordered (need a < b)"
      | langOf a /= langOf b -> Just "cross-language pair"
      | s < 1 || s .&. sentSources /= s -> Just "sources not a subset of the sent generators"
    _ -> Nothing
