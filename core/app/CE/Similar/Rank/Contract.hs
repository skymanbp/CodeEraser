-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | The rank.request shape and its boundary contract (plan v2.33 W3).
-- Seats are the corpus's units in identity order (path, key, nth) —
-- seat order IS the tie order. Terms are channel-tagged fnv1a64 hashes.
--
--   n, avg, k          corpus size, mean bag length (≥ 1), kept per arm
--   exclude            the query's own seat (optional)
--   seen               seats dropped from the kept list (ascending)
--   query              [term, channel, tf], terms strictly ascending
--   widen              rank the PPMI-widened query instead of the bare one
--   words              [term, marg] per spelled word term (ascending)
--   pairs              [a, b, nab, margB] (ascending on a, b; a a word row)
--   terms              [term, df] (ascending, df ≤ n)
--   postings           [term, seat, tf] (ascending; term a terms row)
--   lens               [seat, len] (ascending; every posting seat)
--
-- Completeness the measuring side's fetch must meet: every query term
-- (and, widened, every expansion term) has a terms row; a term that
-- scores (n > 2·df) has exactly df postings; a spelled word term has a
-- words row whenever words ride.
module CE.Similar.Rank.Contract (RankReq (..), offence, overCap, rowsOf) where

import CE.Similar.Rank.Cost (channels, rankCap, scoredDfRatio, wordChannel)
import CE.Wire (rowCheck, tableOffence)
import Data.Aeson (FromJSON (..), Value, withObject, (.!=), (.:), (.:?))
import Data.Foldable (asum)
import qualified Data.Map.Strict as M
import qualified Data.Set as S

data RankReq = RankReq
  { reqId :: Value
  , corpusN :: Integer
  , avgLen :: Integer
  , keep :: Integer
  , exclude :: Maybe Integer
  , seen :: [Integer]
  , query :: [[Integer]]
  , widen :: Bool
  , words_ :: [[Integer]]
  , pairs :: [[Integer]]
  , terms :: [[Integer]]
  , postings :: [[Integer]]
  , lens :: [[Integer]]
  }

instance FromJSON RankReq where
  parseJSON = withObject "RankReq" $ \o ->
    RankReq
      <$> o .: "id"
      <*> o .: "n"
      <*> o .: "avg"
      <*> o .: "k"
      <*> o .:? "exclude"
      <*> o .:? "seen" .!= []
      <*> o .:? "query" .!= []
      <*> o .:? "widen" .!= False
      <*> o .:? "words" .!= []
      <*> o .:? "pairs" .!= []
      <*> o .:? "terms" .!= []
      <*> o .:? "postings" .!= []
      <*> o .:? "lens" .!= []

-- | Every table's rows together.
rowsOf :: RankReq -> Int
rowsOf r = sum (length (seen r) : map length [query r, words_ r, pairs r, terms r, postings r, lens r])

overCap :: RankReq -> Bool
overCap r = toInteger (rowsOf r) > rankCap

-- | The scalars, then each table's shape and order, then the cross-table
-- completeness the ranking relies on.
offence :: RankReq -> Maybe String
offence r =
  asum
    [ scalars r
    , tableOffence "seen" id (\i s -> seat "seen" i s) (seen r)
    , tableOffence "query" (take 1) (rowCheck "query" "malformed query term (need [term,channel,tf])" 3 queryRow) (query r)
    , tableOffence "words" (take 1) (rowCheck "words" "malformed word (need [term,marg])" 2 wordRow) (words_ r)
    , tableOffence "pairs" (take 2) (rowCheck "pairs" "malformed pair (need [a,b,nab,margB])" 4 pairRow) (pairs r)
    , tableOffence "terms" (take 1) (rowCheck "terms" "malformed term (need [term,df])" 2 termRow) (terms r)
    , tableOffence "postings" (take 2) (rowCheck "postings" "malformed posting (need [term,seat,tf])" 3 postingRow) (postings r)
    , tableOffence "lens" (take 1) (rowCheck "lens" "malformed length (need [seat,len])" 2 lenRow) (lens r)
    , complete r
    ]
 where
  n = corpusN r
  seat what i s
    | s < 0 || s >= n = Just (what <> " " <> show i <> ": seat out of range")
    | otherwise = Nothing
  inSeat s = s >= 0 && s < n
  queryRow row = case row of
    [t, ch, tf]
      | t < 0 -> Just "negative term"
      | ch < 0 || ch >= channels -> Just "channel out of range"
      | tf < 1 -> Just "non-positive tf"
    _ -> Nothing
  wordRow row = case row of
    [t, m] | t < 0 -> Just "negative term" | m < 0 -> Just "negative marg"
    _ -> Nothing
  pairRow row = case row of
    [a, b', nab, mb]
      | a < 0 || b' < 0 -> Just "negative term"
      | nab < 1 -> Just "non-positive nab"
      | mb < nab -> Just "margB below nab"
    _ -> Nothing
  termRow row = case row of
    [t, df] | t < 0 -> Just "negative term" | df < 0 || df > n -> Just "df out of range"
    _ -> Nothing
  postingRow row = case row of
    [t, s, tf] | t < 0 -> Just "negative term" | not (inSeat s) -> Just "seat out of range" | tf < 1 -> Just "non-positive tf"
    _ -> Nothing
  lenRow row = case row of
    [s, l] | not (inSeat s) -> Just "seat out of range" | l < 0 -> Just "negative len"
    _ -> Nothing

scalars :: RankReq -> Maybe String
scalars r
  | corpusN r < 0 = Just "n: negative"
  | avgLen r < 1 = Just "avg: non-positive"
  | keep r < 0 = Just "k: negative"
  | maybe False (\s -> s < 0 || s >= corpusN r) (exclude r) = Just "exclude: seat out of range"
  | otherwise = Nothing

-- | The cross-table rules: postings name a terms row and a lens row; a
-- term that scores carries exactly df postings; every query term has a
-- terms row; words ride for every spelled word term, and a pair's a is
-- a words row. (Expansion terms are checked when the ranking reaches
-- them — CE.Similar.Rank.Score answers which.)
complete :: RankReq -> Maybe String
complete r =
  asum
    [ rule "postings: a term without a terms row" (any (`M.notMember` df) postingTerms)
    , rule "postings: a seat without a lens row" (any (`S.notMember` lenSeats) [s | [_, s, _] <- postings r])
    , rule "terms: a scoring term without exactly df postings" (any short (M.toList df))
    , rule "query: a term without a terms row" (any (`M.notMember` df) [t | (t : _) <- query r])
    , if null (words_ r) && null (pairs r) then Nothing else cooc
    ]
 where
  df = M.fromList [(t, d) | [t, d] <- terms r]
  postingTerms = [t | (t : _) <- postings r]
  posted = M.fromListWith (+) [(t, 1 :: Integer) | t <- postingTerms]
  lenSeats = S.fromList [s | (s : _) <- lens r]
  wordSet = S.fromList [t | (t : _) <- words_ r]
  -- a scoring term carries all df postings; a non-scoring one all or none
  short (t, d) =
    let got = M.findWithDefault 0 t posted
     in if corpusN r > scoredDfRatio * d then got /= d else got /= 0 && got /= d
  cooc =
    asum
      [ rule "words: a spelled word term without a words row" (or [S.notMember t wordSet | [t, ch, _] <- query r, wordChannel ch])
      , rule "pairs: a not a words row" (any (`S.notMember` wordSet) [a | (a : _) <- pairs r])
      ]
  rule msg bad = if bad then Just msg else Nothing
