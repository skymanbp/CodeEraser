-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | similar.request handler (plan v2.29 step 5): the twelfth judgment
-- family — the same-role ADVISOR behind `ce similar`, the MCP tool and
-- the GUI screen. Rust ranks a query's candidates off its own inverted
-- tables and sends the query bag as [termHash, weight] pairs plus one
-- [nHit, pHit, cHit, dHit, sHit, lHit, shapeEqual, bm25Num, bm25Den]
-- row per candidate; this family answers the order the candidates
-- stand in (exact rationals, never the measuring side's fixed point)
-- and which of them play the query's role. Names, words and paths
-- never cross the wire (§5.9.2) — hashes and counts only, row index is
-- identity and Rust re-labels on return. Advisory only: no condition
-- bit, no knob, no tier (booklet 13's posture) — a knobless family
-- whose one table is not the shared RowsReq (the query bag is its own
-- key), so it binds the cascade directly.
module CE.Similar (respond) where

import CE.Similar.Cost (isRole, ratio, rowWidth, similarCap)
import CE.Wire (Family (..), respondWith, rowCheck, tableOffence)
import Data.Aeson (FromJSON (..), Value, encode, object, withObject, (.!=), (.:), (.:?), (.=))
import qualified Data.ByteString.Char8 as B8
import qualified Data.ByteString.Lazy as BL
import Data.Foldable (asum)
import Data.List (sortBy)
import Data.Ord (Down (..), comparing)

-- | The request: id, the query bag (absent = an empty query, which
-- ranks nothing but is well formed), the candidate rows.
data SimilarReq = SimilarReq
  { simId :: Value
  , queryOf :: [[Integer]]
  , candidatesOf :: [[Integer]]
  }

instance FromJSON SimilarReq where
  parseJSON = withObject "SimilarReq" $ \o ->
    SimilarReq <$> o .: "id" <*> o .:? "query" .!= [] <*> o .: "rows"

-- | decode → cap (query terms and rows together) → contract (the query
-- table first: shape then strictly ascending hashes — a bag is a set;
-- then every row) → judge.
respond :: String -> B8.ByteString -> Either (Maybe Value, String, String) B8.ByteString
respond proto =
  respondWith
    Family
      { famName = "similar"
      , famId = simId
      , famOverCap = overCap
      , famOffence = offence
      , famDegraded = degraded proto
      , famJudged = judged proto
      }

-- | Query terms and candidate rows are priced together (the scan C15
-- discipline: every request dimension counts against the cap).
overCap :: SimilarReq -> Bool
overCap req = toInteger (length (queryOf req) + length (candidatesOf req)) > similarCap

offence :: SimilarReq -> Maybe String
offence req =
  asum
    [ tableOffence "query" (take 1) termShape (queryOf req)
    , asum (zipWith rowShape [0 :: Int ..] (candidatesOf req))
    ]

termShape :: Int -> [Integer] -> Maybe String
termShape = rowCheck "query" "malformed query term (need [termHash,weight])" 2 termChecks

termChecks :: [Integer] -> Maybe String
termChecks term = case term of
  [hash, w]
    | hash < 0 -> Just "negative term hash"
    | w < 1 -> Just "non-positive weight"
  _ -> Nothing

rowShape :: Int -> [Integer] -> Maybe String
rowShape =
  rowCheck
    "row"
    "malformed row (need [nHit,pHit,cHit,dHit,sHit,lHit,shapeEqual,bm25Num,bm25Den])"
    rowWidth
    rowChecks

rowChecks :: [Integer] -> Maybe String
rowChecks row = case row of
  [n, p, c, d, s, l, shape, num, den]
    | any (< 0) [n, p, c, d, s, l] -> Just "negative hit"
    | shape `notElem` [0, 1] -> Just "shapeEqual not a boolean"
    | num < 0 -> Just "negative score"
    | den < 1 -> Just "non-positive denominator"
  _ -> Nothing

-- | The order: score descending as exact rationals, ties by request
-- index (the measuring side sends its candidates in identity order, so
-- a tie keeps that order); the role bit per row in REQUEST order.
judged :: String -> SimilarReq -> B8.ByteString
judged proto req = reply proto req order (map isRole rows) False
 where
  rows = candidatesOf req
  order = map fst (sortBy (comparing (Down . ratio . snd) <> comparing fst) (zip [0 :: Integer ..] rows))

-- | Over-cap: a complete degraded reply with empty tables — a query the
-- core refused to judge has no order and no roles; the faces name the
-- degradation instead of showing the measuring side's order.
degraded :: String -> SimilarReq -> B8.ByteString
degraded proto req = reply proto req [] [] True

-- | The similar.result object: the candidate indices in judged order,
-- the role bit per candidate in request order, the counts.
reply :: String -> SimilarReq -> [Integer] -> [Bool] -> Bool -> B8.ByteString
reply proto req order roles isDegraded =
  BL.toStrict . encode . object $
    [ "proto" .= proto
    , "type" .= ("similar.result" :: String)
    , "id" .= simId req
    , "order" .= order
    , "roles" .= roles
    , "counts"
        .= object
          [ "rows" .= length (candidatesOf req)
          , "queryTerms" .= length (queryOf req)
          , "role" .= length (filter id roles)
          ]
    , "degraded" .= isDegraded
    ]
      <> ["reason" .= ("similar_too_large" :: String) | isDegraded]
