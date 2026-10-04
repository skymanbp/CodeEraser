-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | rank.request handler (plan v2.33 W3; booklet 15): the advisor's
-- ranking over integer facts. The measuring side keeps the text → term
-- road and the stored tables; per query it sends the bag as [term,
-- channel, tf], the df of every term it asks about, the postings of the
-- terms that score, the candidate seats' lengths and — for the widened
-- view — the words' unit counts and co-occurrence rows. This family
-- weights the bag, widens it, scores it and keeps the top k; it answers
-- the kept candidates with their evidence integers, the expansion it
-- found (so the measuring side can fetch the expansion terms' rows and
-- ask for the widened arm), and the weighted bag the role judgment
-- (CE.Similar, similar/1) reads. Seats, never names, cross the wire.
module CE.Similar.Rank (respond) where

import CE.Similar.Rank.Contract (RankReq (..), offence, overCap, rowsOf)
import CE.Similar.Rank.Cost (scoreFracBits)
import CE.Similar.Rank.Score (Hit (..), QTerm (..), bare, expansion, rankTop)
import CE.Wire (family)
import Control.Applicative ((<|>))
import Data.Aeson (Value, encode, object, (.=))
import Data.Bits (shiftR)
import qualified Data.ByteString.Char8 as B8
import qualified Data.ByteString.Lazy as BL
import Data.Function (on)
import qualified Data.IntMap.Strict as IM
import Data.List (groupBy)
import qualified Data.Map.Strict as M
import qualified Data.Set as S

-- | decode → cap → contract (and, widened, every expansion term must
-- have a terms row) → the ranking.
respond :: String -> B8.ByteString -> Either (Maybe Value, String, String) B8.ByteString
respond proto = family "rank" reqId overCap (\r -> offence r <|> widened r) (answer proto True) (answer proto False)

-- | The expansion this request's cooc rows give its bare query.
added :: RankReq -> [QTerm]
added r
  | null (words_ r) = []
  | otherwise = expansion (corpusN r) (bare (query r)) margs pairsBy
 where
  margs = M.fromList [(t, m) | [t, m] <- words_ r]
  pairsBy = M.fromAscListWith (flip (<>)) [(a, [[b', nab, mb]]) | [a, b', nab, mb] <- pairs r]

widened :: RankReq -> Maybe String
widened r
  | widen r && any (\q -> S.notMember (qTerm q) known) (added r) = Just "widen: an expansion term without a terms row"
  | otherwise = Nothing
 where
  known = S.fromList [t | (t : _) <- terms r]

-- | The rank.result object: the kept hits [seat, score, scoreNum, N, P,
-- C, D, S, L] over scoreDen, the expansion [term, channel], the
-- weighted bag [term, weight] ranked, the counts. Degraded: empty
-- tables and the reason.
answer :: String -> Bool -> RankReq -> B8.ByteString
answer proto degraded r =
  BL.toStrict . encode . object $
    [ "proto" .= proto
    , "type" .= ("rank.result" :: String)
    , "id" .= reqId r
    , "hits" .= [toInteger (hSeat h) : hScore h `shiftR` scoreFracBits : hScore h : hHits h | h <- hits]
    , "scoreDen" .= (2 ^ scoreFracBits :: Integer)
    , "added" .= [[qTerm q, qChan q] | q <- if degraded then [] else added r]
    , "query" .= if degraded then [] else M.toList (M.fromListWith (+) [(qTerm q, qWeight q) | q <- ranked])
    , "counts" .= object ["rows" .= rowsOf r, "queryTerms" .= length ranked, "hits" .= length hits]
    , "degraded" .= degraded
    ]
      <> ["reason" .= ("rank_too_large" :: String) | degraded]
 where
  ranked
    | degraded = []
    | widen r = bare (query r) <> added r
    | otherwise = bare (query r)
  hits
    | degraded = []
    | otherwise =
        rankTop
          (corpusN r)
          (avgLen r)
          (fromInteger (keep r))
          (fromInteger <$> exclude r)
          (S.fromList (map fromInteger (seen r)))
          (M.fromList [(t, d) | [t, d] <- terms r])
          postingsBy
          (IM.fromList [(fromInteger s, l) | [s, l] <- lens r])
          ranked
  postingsBy =
    M.fromDistinctAscList
      [ (t, [(fromInteger s, tf) | [_, s, tf] <- grp])
      | grp@((t : _) : _) <- groupBy ((==) `on` take 1) (postings r)
      ]
