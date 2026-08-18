-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | trend.request handler (M7.5b): the eighth judgment family — the
-- score trajectory the Rust side measured over mainline history
-- comes in as [ts, score, scale] rows (ascending ts), the slope
-- verdict goes back. Judgment is exact-Rational (CE.Trend.Cost);
-- fewer rows than minPoints answers null — an absence, never a
-- fabricated flat. Measurement and report rendering stay in Rust;
-- commit hashes and paths never cross the wire (§5.9.2).
module CE.Trend (respond) where

import CE.Trend.Cost (knobTable, slopeMicroPerDay, trendRowCap, verdictOf)
import CE.Wire (Family (..), ascendingOn, pick, respondWith)
import Data.Aeson
import qualified Data.ByteString.Char8 as B8
import qualified Data.ByteString.Lazy as BL
import Data.Foldable (asum)

data TrendReq = TrendReq
  { reqId :: Value
  , reqRows :: [[Integer]]
  , reqKnobs :: [[Integer]]
  }

instance FromJSON TrendReq where
  parseJSON = withObject "TrendReq" $ \o ->
    TrendReq <$> o .: "id" <*> o .: "rows" <*> o .:? "knobs" .!= []

-- | The shared cascade with this family's bindings (CE.Wire).
respond :: String -> B8.ByteString -> Either (Maybe Value, String, String) B8.ByteString
respond proto =
  respondWith
    Family
      { famName = "trend"
      , famId = reqId
      , famOverCap = \req ->
          toInteger (length (reqRows req) + length (reqKnobs req)) > trendRowCap
      , famOffence = violation
      , famDegraded = degraded proto
      , famJudged = judged proto
      }

-- | First boundary-contract offender in request order; ts must be
-- NON-DESCENDING — ties are a legal, common shape (same-second
-- commits: rebases and scripted pushes), and the all-tied window
-- answers a null slope instead (Cost's zero-variance guard). Only a
-- backwards step is a broken window.
violation :: TrendReq -> Maybe String
violation req =
  asum
    [ asum (zipWith rowShape [0 :: Int ..] (reqRows req))
    , asum (zipWith tsStep [1 :: Int ..] (steps (reqRows req)))
    , asum (zipWith knobShape [0 :: Int ..] (reqKnobs req))
    , ascendingOn "knob" (take 1) (reqKnobs req)
    ]
 where
  steps rows = zip rows (drop 1 rows)
  tsStep i (p, c)
    | take 1 c < take 1 p = Just ("row " <> show i <> ": ts descending")
    | otherwise = Nothing

rowShape :: Int -> [Integer] -> Maybe String
rowShape i row = case row of
  [ts, score, scale]
    | ts < 0 -> Just (label <> "negative timestamp")
    | scale <= 0 -> Just (label <> "non-positive scale")
    | score < 0 || score > scale -> Just (label <> "score outside 0..scale")
    | otherwise -> Nothing
  _ -> Just (label <> "malformed row (need [ts,score,scale])")
 where
  label = "row " <> show i <> ": "

knobShape :: Int -> [Integer] -> Maybe String
knobShape i row = case row of
  [code, v]
    | code < 0 || code > 1 -> Just (label <> "unknown knob code")
    | v < 0 -> Just (label <> "negative knob value")
    | code == 0 && v < 2 -> Just (label <> "minPoints below 2")
    | otherwise -> Nothing
  _ -> Just (label <> "malformed knob (need [code,value])")
 where
  label = "knob " <> show i <> ": "

-- | Below minPoints the verdict and slope are ABSENT (null) and the
-- fail bit stays false — nothing was judged, and an unjudged trend
-- must not gate (unlike a degraded reply, where judgment was DENIED
-- by the cap and fail=true says so).
judged :: String -> TrendReq -> B8.ByteString
judged proto req =
  reply proto req Judgment{jEffective = effective, jVerdict = verdict, jSlope = slope, jDegraded = False}
 where
  effective = [(c, pick (reqKnobs req) c d) | (c, d) <- knobTable]
  minPoints = pick (reqKnobs req) 0 3
  floorMicro = pick (reqKnobs req) 1 0
  rows = [(t, s, sc) | [t, s, sc] <- reqRows req]
  slope
    | toInteger (length rows) < minPoints = Nothing
    | otherwise = slopeMicroPerDay rows
  verdict = verdictOf floorMicro <$> slope

-- | Over-cap: a complete degraded reply that FAILS, echoing the
-- DEFAULT knobs — an unvalidated override must not be presented as
-- effective (the scan C14 posture).
degraded :: String -> TrendReq -> B8.ByteString
degraded proto req =
  reply proto req Judgment{jEffective = knobTable, jVerdict = Nothing, jSlope = Nothing, jDegraded = True}

-- | One judgment, four payloads — the record keeps reply at the
-- arity line its five sibling families hold (Structure's Knobs
-- precedent; a sixth positional argument was the scan warn). Local
-- to this family: one authority per family forbids a shared shape.
data Judgment = Judgment
  { jEffective :: [(Integer, Integer)]
  , jVerdict :: Maybe Integer
  , jSlope :: Maybe Rational
  , jDegraded :: Bool
  }

-- | The trend.result object: exact-Rational slope display-rounded
-- (round-half-even; the verdict compared the EXACT value and no
-- client re-derives it), fail = degraded or an armed decline floor.
reply :: String -> TrendReq -> Judgment -> B8.ByteString
reply proto req j =
  BL.toStrict . encode . object $
    [ "proto" .= proto
    , "type" .= ("trend.result" :: String)
    , "id" .= reqId req
    , "slopeMicroPerDay" .= fmap (round :: Rational -> Integer) (jSlope j)
    , "verdict" .= jVerdict j
    , "fail" .= (jDegraded j || (jVerdict j == Just 2 && lookup 1 (jEffective j) > Just 0))
    , "knobs" .= [[c, v] | (c, v) <- jEffective j]
    , "counts" .= object ["rows" .= length (reqRows req)]
    , "degraded" .= jDegraded j
    ]
      <> ["reason" .= ("trend_too_large" :: String) | jDegraded j]
