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
-- never a ratio, and since ADR-008 P1 each score row carries the
-- OWNER's full verdict bit (Cost.dupVerdict: Jaccard half ∨ verbatim
-- half — the run rides the request precisely so one wire transcript
-- holds the complete verdict inputs, F26, and stays absent from
-- every reply field). The reported set is the core's decision,
-- relayed by Rust, never re-derived there. The M5-3a stub refused
-- here; this batch replaced exactly that refusal, and the
-- computation lives behind the exhaustive reference harness
-- (core/test/ReferenceJaccard.hs).
module CE.Docdup (respond) where

import CE.Docdup.Cost
  ( docPairCap
  , docSetCap
  , dupDecides
  , dupVerdict
  , jaccardDen
  , jaccardNum
  , shingleK
  , verbatimFloor
  )
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
-- each row paired with the owner's FULL verdict (ADR-008 P1) — the
-- Jaccard-half tally stays as the additive counts.jaccardDups it
-- always was.
judge :: [[Integer]] -> [[Integer]] -> ([([Integer], Bool)], Int)
judge sets ps = foldr step ([], 0) ps
 where
  arr = IM.fromList (zip [0 ..] sets)
  step (i : j : run : _) (rows, dups) =
    ( ([i, j, inter, union], dupVerdict inter union run) : rows
    , if dupDecides inter union then dups + 1 else dups
    )
   where
    (inter, union) = interUnion (arr IM.! fromIntegral i) (arr IM.! fromIntegral j)
  step _ acc = acc -- unreachable: row shape validated upstream

-- | verdicts is the ADR-008 P1 additive field: one bit per score
-- row, same order; verbatimFloor joins the echo so the Rust mirror
-- is pinned like every other single-owner number.
reply :: String -> DocdupReq -> [([Integer], Bool)] -> Int -> Bool -> B8.ByteString
reply proto req scored dups degraded =
  BL.toStrict . encode . object $
    [ "proto" .= proto
    , "type" .= ("docdup.result" :: String)
    , "id" .= reqId req
    , "scores" .= map fst scored
    , "verdicts" .= map snd scored
    , "counts"
        .= object
          [ "sets" .= length (reqSets req)
          , "pairs" .= length (reqPairs req)
          , "judged" .= length scored
          , "jaccardDups" .= dups
          ]
    , "knobs"
        .= object
          [ "jaccardNum" .= jaccardNum
          , "jaccardDen" .= jaccardDen
          , "shingleK" .= shingleK
          , "verbatimFloor" .= verbatimFloor
          ]
    , "degraded" .= degraded
    ]
      <> ["reason" .= ("docdup_too_large" :: String) | degraded]
