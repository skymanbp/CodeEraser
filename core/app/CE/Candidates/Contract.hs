-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | The candidates.request shape and its boundary contract (plan v2.33
-- W3): the admitted units, each `[lang, file, key, nodes, start, end,
-- kind, count, kind, count, …]` (language, file, key and kind codes
-- request-local; file codes rank the paths, so code order is path
-- order; the kinds strictly ascending, the counts summing to the
-- unit's nodes; the span 1-based and inclusive); each unit's structural
-- shingle set in unit order; the fingerprint instances `[hash, file,
-- line, tok]` in index order; the near runs `[fileA, lineA, fileB,
-- lineB]`; and whether the exhaustive source runs. The first offender
-- in request order is named as `<table> <i>: <reason>`.
module CE.Candidates.Contract (CandReq (..), offence, overCap) where

import CE.Candidates.Cost (candidateNearCap, candidatePrintCap, candidateSigCap, candidateUnitCap)
import CE.Wire (elements, label, outsideU64, rowCheck, u64)
import CE.Wire.Tables (optTables)
import Data.Aeson (FromJSON (..), Value, withObject, (.!=), (.:), (.:?))
import Data.Foldable (asum)

-- | The request: id and the exhaustive switch (absent = off: the
-- frozen four-source pass), then the four tables — the unit rows and
-- their sets, the fingerprint instances, the near runs.
data CandReq = CandReq
  { reqId :: Value
  , exhaustive :: Bool
  , unitRows, sigRows, printRows, nearRows :: [[Integer]]
  }

-- The four tables are read by one traversal over their keys (each
-- absent = empty), then the switch.
instance FromJSON CandReq where
  parseJSON = withObject "CandReq" $ \o -> do
    req <- CandReq <$> o .: "id" <*> o .:? "exhaustive" .!= False
    tables <- optTables o ["units", "sigs", "prints", "near"]
    case tables of
      [us, ss, ps, ns] -> pure (req us ss ps ns)
      _ -> fail "four tables"

-- | Four dimensions, four ceilings.
overCap :: CandReq -> Bool
overCap req =
  over candidateUnitCap (length (unitRows req))
    || over candidateSigCap (sum (map length (sigRows req)))
    || over candidatePrintCap (length (printRows req))
    || over candidateNearCap (length (nearRows req))
 where
  over cap k = toInteger k > cap

-- | Every unit row's own shape, then one set per unit, then the sets,
-- the instances and the near runs, each table in request order.
offence :: CandReq -> Maybe String
offence req =
  asum
    [ asum (zipWith unitShape [0 ..] (unitRows req))
    , if length (sigRows req) /= length (unitRows req) then Just "sigs: not one set per unit" else Nothing
    , asum (zipWith setShape [0 ..] (sigRows req))
    , asum (zipWith printShape [0 ..] (printRows req))
    , asum (zipWith nearShape [0 ..] (nearRows req))
    ]

-- | [lang, file, key, nodes, start, end, kind, count, …]: the codes
-- non-negative, the span ordered from line 1, the histogram's kinds
-- strictly ascending and non-negative, its counts positive and summing
-- to nodes.
unitShape :: Int -> [Integer] -> Maybe String
unitShape i row = case row of
  (lang : fileCode : keyCode : nodes : from : to : hist)
    | lang < 0 -> refuse "negative language"
    | fileCode < 0 -> refuse "negative file"
    | keyCode < 0 -> refuse "negative key"
    | nodes < 1 -> refuse "non-positive nodes"
    | from < 1 || to < from -> refuse "span not 1-based and ordered"
    | odd (length hist) -> refuse "histogram not [kind,count] pairs"
    | any (< 0) kinds -> refuse "negative kind"
    | any (< 1) counts -> refuse "non-positive count"
    | or (zipWith (>=) kinds (drop 1 kinds)) -> refuse "kinds not strictly ascending"
    | sum counts /= nodes -> refuse "histogram does not sum to nodes"
    | otherwise -> Nothing
   where
    kinds = everyOther hist
    counts = everyOther (drop 1 hist)
  _ -> refuse "malformed unit (need [lang,file,key,nodes,start,end,kind,count,...])"
 where
  refuse why = Just (label "unit" i <> why)

-- | The even positions of a list.
everyOther :: [a] -> [a]
everyOther xs = case xs of
  (x : _ : rest) -> x : everyOther rest
  [x] -> [x]
  [] -> []

-- | One unit's set: non-empty, every element a u64 (CE.Wire.elements;
-- any order — the pass sorts).
setShape :: Int -> [Integer] -> Maybe String
setShape = elements "sig" "set" outsideU64 False

-- | [hash, file, line, tok]: the hash a u64, the rest non-negative.
printShape :: Int -> [Integer] -> Maybe String
printShape = rowCheck "print" "malformed print (need [hash,file,line,tok])" 4 checks
 where
  checks row = case row of
    (h : rest)
      | not (u64 h) -> Just "hash outside u64"
      | any (< 0) rest -> Just "negative file, line or tok"
    _ -> Nothing

-- | [fileA, lineA, fileB, lineB]: every cell non-negative.
nearShape :: Int -> [Integer] -> Maybe String
nearShape = rowCheck "near" "malformed near run (need [fileA,lineA,fileB,lineB])" 4 checks
 where
  checks row
    | any (< 0) row = Just "negative file or line"
    | otherwise = Nothing
