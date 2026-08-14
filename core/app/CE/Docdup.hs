-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | docdup.request handler (design vol.2 §5): decode ascending
-- deduped shingle-hash sets, enforce the Cost caps (over-cap = a
-- complete degraded reply, never a truncated one), machine-check the
-- boundary contract in request order (non-empty strictly-ascending
-- u64 sets, [i,j,verbatimRun] rows with in-range endpoints and
-- non-negative runs, strictly ascending rows) — then judge: exact
-- Jaccard per pair via Data.Set. Raw inter and union cross the wire,
-- never a ratio; the Jaccard half of the verdict is also applied
-- HERE by its owner (counts.jaccardDups, Cost.dupDecides), while the
-- verbatim half stays with Rust, which computed the runs — the run
-- rides the request so one wire transcript carries the complete
-- verdict inputs (F26), and is deliberately absent from every reply
-- field. The M5-3a stub refused here; this batch replaced exactly
-- that refusal, and the computation lives behind the exhaustive
-- reference harness (core/test/ReferenceJaccard.hs).
module CE.Docdup (respond) where

import CE.Docdup.Cost (docPairCap, docSetCap, dupDecides, jaccardDen, jaccardNum, shingleK)
import CE.Docdup.Jaccard (interUnion)
import Data.Aeson
import qualified Data.ByteString.Char8 as B8
import qualified Data.ByteString.Lazy as BL
import Data.Foldable (asum)
import qualified Data.IntMap.Strict as IM

data DocdupReq = DocdupReq
  { reqId :: Value
  , reqSets :: [[Integer]]
  , reqPairs :: [[Integer]]
  }

instance FromJSON DocdupReq where
  parseJSON = withObject "DocdupReq" $ \o ->
    DocdupReq <$> o .: "id" <*> o .: "sets" <*> o .: "pairs"

-- | Left = (id to echo, error code, message); Right = the encoded
-- docdup.result line.
respond :: String -> B8.ByteString -> Either (Maybe Value, String, String) B8.ByteString
respond proto line = case eitherDecodeStrict line of
  Left e -> Left (Nothing, "bad_request", "docdup: " <> e)
  Right req
    | any (\s -> toInteger (length s) > docSetCap) (reqSets req)
        || toInteger (length (reqPairs req)) > docPairCap ->
        Right (reply proto req [] 0 True)
    | Just why <- violation req -> Left (Just (reqId req), "contract", why)
    | otherwise ->
        let (rows, dups) = judge (reqSets req) (reqPairs req)
         in Right (reply proto req rows dups False)

-- | First boundary-contract offender in request order (Clone.hs
-- posture: the message names the violator deterministically).
violation :: DocdupReq -> Maybe String
violation req =
  asum
    [ asum (zipWith setShape [0 :: Int ..] ss)
    , asum (zipWith (pairRow (length ss)) [0 :: Int ..] ps)
    , asum (zipWith notAscending [1 :: Int ..] (zip ps (drop 1 ps)))
    ]
 where
  ss = reqSets req
  ps = reqPairs req

setShape :: Int -> [Integer] -> Maybe String
setShape s set
  | null set = Just (label <> "empty set")
  | any (< 0) set = Just (label <> "negative element")
  | any (>= 2 ^ (64 :: Int)) set = Just (label <> "element exceeds u64")
  | or (zipWith (>=) set (drop 1 set)) = Just (label <> "not strictly ascending")
  | otherwise = Nothing
 where
  label = "set " <> show s <> ": "

pairRow :: Int -> Int -> [Integer] -> Maybe String
pairRow n p row = case row of
  [i, j, run]
    | i < 0 || j < 0 || i >= toInteger n || j >= toInteger n ->
        Just (label <> "endpoint out of range")
    | run < 0 -> Just (label <> "negative verbatim run")
    | otherwise -> Nothing
  _ -> Just (label <> "malformed row (need [i,j,verbatimRun])")
 where
  label = "pair " <> show p <> ": "

notAscending :: Int -> ([Integer], [Integer]) -> Maybe String
notAscending p (prev, cur)
  | prev < cur = Nothing
  | otherwise = Just ("pair " <> show p <> ": not strictly ascending")

-- | Judge every pair: exact Jaccard on the two sets, raw counts out,
-- the owner's threshold half tallied (never per-row verdicts — the
-- cut table downstream recomputes from one run).
judge :: [[Integer]] -> [[Integer]] -> ([[Integer]], Int)
judge sets ps = foldr step ([], 0) ps
 where
  arr = IM.fromList (zip [0 ..] sets)
  step (i : j : _) (rows, dups) =
    ( [i, j, inter, union] : rows
    , if dupDecides inter union then dups + 1 else dups
    )
   where
    (inter, union) = interUnion (arr IM.! fromIntegral i) (arr IM.! fromIntegral j)
  step _ acc = acc -- unreachable: row shape validated upstream

reply :: String -> DocdupReq -> [[Integer]] -> Int -> Bool -> B8.ByteString
reply proto req scores dups degraded =
  BL.toStrict . encode . object $
    [ "proto" .= proto
    , "type" .= ("docdup.result" :: String)
    , "id" .= reqId req
    , "scores" .= scores
    , "counts"
        .= object
          [ "sets" .= length (reqSets req)
          , "pairs" .= length (reqPairs req)
          , "judged" .= length scores
          , "jaccardDups" .= dups
          ]
    , "knobs"
        .= object
          [ "jaccardNum" .= jaccardNum
          , "jaccardDen" .= jaccardDen
          , "shingleK" .= shingleK
          ]
    , "degraded" .= degraded
    ]
      <> ["reason" .= ("docdup_too_large" :: String) | degraded]
