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
--
-- Since plan v2.33 W3 a request may send each segment's UNSORTED
-- shingle sequence (`seqs`) instead of its set, with `[i,j]` pairs:
-- this side then derives the sets and measures each pair's verbatim
-- run itself (CE.Docdup.Runs) and answers the runs beside the scores.
-- The `sets` shape with measured runs is answered as before.
module CE.Docdup (respond, setShape) where

import CE.Docdup.Cost
  (
    docLineCap,
    licHeadLines,
    minDocTokens,
    docPairCap
  , docSeqCap
  , docSetCap
  , dupDecides
  , dupVerdict
  , jaccardDen
  , jaccardNum
  , shingleK
  , verbatimFloor
  )
import CE.Docdup.Jaccard (interUnion)
import CE.Docdup.Runs (indexed, runWords, setOf)
import Data.Array (listArray, (!))
import CE.Wire (Family (..), respondWith, tableOffence)
import Data.Aeson
import qualified Data.ByteString.Char8 as B8
import qualified Data.ByteString.Lazy as BL
import Data.Foldable (asum)
import qualified Data.IntMap.Strict as IM

-- | The request: the segments as sets (pairs carry their runs) or,
-- when `seqs` is sent, as sequences (pairs carry none).
data DocdupReq = DocdupReq
  { reqId :: Value
  , reqSets :: [[Integer]]
  , reqSeqs :: Maybe [[Integer]]
  , reqPairs :: [[Integer]]
  }

instance FromJSON DocdupReq where
  parseJSON = withObject "DocdupReq" $ \o ->
    DocdupReq <$> o .: "id" <*> o .:? "sets" .!= [] <*> o .:? "seqs" <*> o .: "pairs"

-- | The judged sets: the sent sets, or the sequences' sets.
setsOf :: DocdupReq -> [[Integer]]
setsOf req = maybe (reqSets req) (map setOf) (reqSeqs req)

-- | The shared cascade with this family's bindings (CE.Wire).
respond :: String -> B8.ByteString -> Either (Maybe Value, String, String) B8.ByteString
respond proto =
  respondWith
    Family
      { famName = "docdup"
      , famId = reqId
      , famOverCap = \req ->
          any (\s -> toInteger (length s) > docSetCap) (setsOf req)
            || any (\s -> toInteger (length s) > docSeqCap) (concat (reqSeqs req))
            || toInteger (length (reqPairs req)) > docPairCap
      , famOffence = violation
      , famDegraded = \req -> reply proto req [] 0 True
      , famJudged = \req ->
          let (rows, dups) = judge (setsOf req) (pairsOf req)
           in reply proto req rows dups False
      }

-- | First boundary-contract offender in request order (Clone.hs
-- posture: the message names the violator deterministically); the
-- ascending checker is CE.Wire's shared one.
violation :: DocdupReq -> Maybe String
violation req = case reqSeqs req of
  Nothing ->
    asum
      [ asum (zipWith setShape [0 :: Int ..] ss)
      , -- ascend on the (i,j) IDENTITY PREFIX, not the whole row
        -- (review C10: [[0,1,0],[0,1,60]] was lexicographically
        -- ascending and judged the same pair twice with two bits)
        tableOffence "pair" (take 2) (pairRow (length ss)) ps
      ]
  Just qs
    | not (null ss) -> Just "sets: sent beside seqs (send one shape)"
    | otherwise ->
        asum
          [ asum (zipWith seqShape [0 :: Int ..] qs)
          , tableOffence "pair" (take 2) (seqPair (length qs)) ps
          ]
 where
  ss = reqSets req
  ps = reqPairs req

-- | A sequence: non-empty, every element a u64 (any order, repeats
-- allowed).
seqShape :: Int -> [Integer] -> Maybe String
seqShape s xs
  | null xs = Just (label <> "empty sequence")
  | any (\x -> x < 0 || x >= 2 ^ (64 :: Int)) xs = Just (label <> "element outside u64")
  | otherwise = Nothing
 where
  label = "seq " <> show s <> ": "

-- | A pair over sequences carries no run: [i, j], then the same
-- endpoint checks.
seqPair :: Int -> Int -> [Integer] -> Maybe String
seqPair n p row = case row of
  [i, j] -> pairRow n p [i, j, 0]
  _ -> Just ("pair " <> show p <> ": malformed row (need [i,j] beside seqs)")

-- | The pairs as [i, j, run]: sent, or measured over the sequences.
pairsOf :: DocdupReq -> [[Integer]]
pairsOf req = case reqSeqs req of
  Nothing -> reqPairs req
  Just qs ->
    let idx = listArray (0, length qs - 1) (map indexed qs)
        run i j = runWords shingleK (idx ! fromInteger i) (idx ! fromInteger j)
     in [[i, j, run i j] | i : j : _ <- reqPairs req]

-- | A set: non-empty, strictly ascending u64 elements (docdup/1 and
-- docpairs/1 read the same contract).
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
    -- a segment is not a duplicate of itself (review C11)
    | i == j -> Just (label <> "self pair")
    | run < 0 -> Just (label <> "negative verbatim run")
    | otherwise -> Nothing
  _ -> Just (label <> "malformed row (need [i,j,verbatimRun])")
 where
  label = "pair " <> show p <> ": "

-- | Judge every pair: exact Jaccard on the two sets, raw counts out,
-- each row paired with the owner's FULL verdict (ADR-008 P1) — the
-- Jaccard-half tally stays as the additive counts.jaccardDups it
-- always was.
judge :: [[Integer]] -> [[Integer]] -> ([([Integer], Bool, Integer)], Int)
judge sets ps = foldr step ([], 0) ps
 where
  arr = IM.fromList (zip [0 ..] sets)
  step (i : j : run : _) (rows, dups) =
    ( ([i, j, inter, union], dupVerdict inter union run, run) : rows
    , if dupDecides inter union then dups + 1 else dups
    )
   where
    (inter, union) = interUnion (arr IM.! fromIntegral i) (arr IM.! fromIntegral j)
  step _ acc = acc -- unreachable: row shape validated upstream

-- | verdicts is the ADR-008 P1 additive field: one bit per score
-- row, same order; verbatimFloor joins the echo so the Rust mirror
-- is pinned like every other single-owner number. A sequence request
-- also answers each scored row's measured run (`runs`, same order).
reply :: String -> DocdupReq -> [([Integer], Bool, Integer)] -> Int -> Bool -> B8.ByteString
reply proto req scored dups degraded =
  BL.toStrict . encode . object $
    [ "proto" .= proto
    , "type" .= ("docdup.result" :: String)
    , "id" .= reqId req
    , "scores" .= [row | (row, _, _) <- scored]
    , "verdicts" .= [v | (_, v, _) <- scored]
    ]
      <> ["runs" .= [run | (_, _, run) <- scored] | Just _ <- [reqSeqs req]]
      <> [ "counts"
        .= object
          [ "sets" .= length (setsOf req)
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
          , "minDocTokens" .= minDocTokens
          , "docLineCap" .= docLineCap
          , "licHeadLines" .= licHeadLines
          ]
    , "degraded" .= degraded
    ]
      <> ["reason" .= ("docdup_too_large" :: String) | degraded]
