-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | graph.request handler, M5-2a boundary stub: decode, enforce the
-- node/edge caps (the real oversize guard — the envelope byte
-- precheck is relaxed for the trusted same-machine child, 2026-08-12
-- decision), machine-check the boundary contract (edge rows are
-- [src,dst,kind,rung] four-tuples, endpoints and pos indices in
-- range, edges strictly ascending hence duplicate-free) — then
-- refuse. The algorithms land at M5-2g behind their exhaustive
-- reference harness; a stub that answered would be inventing
-- judgments. The validation layer written here survives 2g intact:
-- only the final refusal is replaced by computation.
module CE.Graph (respond) where

import CE.Graph.Cost (edgeCap, nodeCap)
import Data.Aeson
import qualified Data.ByteString.Char8 as B8
import qualified Data.ByteString.Lazy as BL
import Data.Foldable (asum)

-- | Wire shape (design brief §2): index = node identity, nothing
-- text-shaped crosses. @unresolved@ is part of the family shape but
-- carries no validation obligation, so the stub ignores it (§1
-- unknown-field rule). Absent @pos@ = counts only.
data GraphReq = GraphReq
  { reqId :: Value
  , reqNodes :: [Value]
  , reqEdges :: [[Integer]]
  , reqPos :: [Integer]
  }

instance FromJSON GraphReq where
  parseJSON = withObject "GraphReq" $ \o ->
    GraphReq
      <$> o .: "id"
      <*> o .: "nodes"
      <*> o .: "edges"
      <*> o .:? "pos" .!= []

-- | Left = (id to echo, error code, message) for the dispatcher's
-- error encoder; Right = the encoded graph.result line (M5-2a: only
-- the degraded graph_too_large shape exists).
respond :: String -> B8.ByteString -> Either (Maybe Value, String, String) B8.ByteString
respond proto line = case eitherDecodeStrict line of
  Left e -> Left (Nothing, "bad_request", "graph: " <> e)
  Right req
    | toInteger (length (reqNodes req)) > nodeCap
        || toInteger (length (reqEdges req)) > edgeCap ->
        Right (tooLarge proto req)
    | Just why <- violation req ->
        Left (Just (reqId req), "contract", why)
    | otherwise ->
        Left
          ( Just (reqId req)
          , "contract"
          , "graph algorithms land at M5-2g behind their reference \
            \harness; a stub answer would be an invented judgment"
          )

-- | First boundary-contract offender, if any — checked in request
-- order so the message is deterministic. Shape errors surface before
-- ordering errors, so the ascending pass only ever compares
-- well-formed four-tuples (list Ord is lexicographic).
violation :: GraphReq -> Maybe String
violation req =
  asum
    [ asum (zipWith (edgeRow n) [0 :: Int ..] es)
    , asum (zipWith notAscending [1 :: Int ..] (zip es (drop 1 es)))
    , asum (zipWith (posRow n) [0 :: Int ..] (reqPos req))
    ]
 where
  n = fromIntegral (length (reqNodes req))
  es = reqEdges req

edgeRow :: Integer -> Int -> [Integer] -> Maybe String
edgeRow n i row = case row of
  [src, dst, kind, rung]
    | any (< 0) [src, dst, kind, rung] -> Just (label <> "negative field")
    | src >= n || dst >= n -> Just (label <> "endpoint out of range")
    | otherwise -> Nothing
  _ -> Just (label <> "malformed row (need [src,dst,kind,rung])")
 where
  label = "edge " <> show i <> ": "

notAscending :: Int -> ([Integer], [Integer]) -> Maybe String
notAscending i (prev, cur)
  | prev < cur = Nothing
  | otherwise = Just ("edge " <> show i <> ": not strictly ascending")

posRow :: Integer -> Int -> Integer -> Maybe String
posRow n i p
  | p < 0 || p >= n = Just ("pos " <> show i <> ": index out of range")
  | otherwise = Nothing

-- | Over-cap refusal: a well-formed degraded result, never a
-- truncated graph. counts echoes what arrived (informational);
-- kept = 0 because nothing was analyzed.
tooLarge :: String -> GraphReq -> B8.ByteString
tooLarge proto req =
  BL.toStrict . encode $
    object
      [ "proto" .= proto
      , "type" .= ("graph.result" :: String)
      , "id" .= reqId req
      , "dead" .= ([] :: [Value])
      , "pos" .= ([] :: [Value])
      , "cycles" .= ([] :: [Value])
      , "counts"
          .= object
            [ "nodes" .= length (reqNodes req)
            , "edges" .= length (reqEdges req)
            , "kept" .= (0 :: Int)
            ]
      , "degraded" .= True
      , "reason" .= ("graph_too_large" :: String)
      ]
