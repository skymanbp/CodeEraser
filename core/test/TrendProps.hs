-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | The trend family's battery (M7.5b): the product-form slope
-- against an independent centered-moments derivation over an
-- enumerated vector grid (exact Rational — equality is equality),
-- sign/floor boundaries, the knob lever through the REAL respond,
-- refusal-by-name, the below-minPoints absence, and the
-- degraded-fails posture. Scaffolding lives in WireHarness.
module TrendProps (battery) where

import CE.Trend (respond)
import CE.Trend.Cost (slopeMicroPerDay, trendRowCap, verdictOf)
import Data.Aeson
import Data.Ratio ((%))
import WireHarness (field, refusedBy, replyObjWith, rowsRequest, runChecks, setKey)

battery :: IO Bool
battery =
  runChecks
    [ ("product-form slope == centered-moments slope on the grid", slopeGrid)
    , ("sign and floor boundaries classify exactly", boundaries)
    , ("a declared floor flips verdict and fail through respond", floorLever)
    , ("below minPoints the verdict is ABSENT, never a flat", absence)
    , ("trend refusals name the offender", refusals)
    , ("an over-cap trend request degrades to a reply that FAILS", degradedFails)
    ]

-- | Second derivation: slope = Σ(x-x̄)(y-ȳ) / Σ(x-x̄)² — different
-- algebra, same Rational; the grid enumerates lengths 2..5 over a
-- small value alphabet with distinct ascending timestamps.
centered :: [(Integer, Integer, Integer)] -> Rational
centered rows = sum (zipWith (*) dx dy) / sum (map (\d -> d * d) dx)
 where
  xs = [ts % 86400 | (ts, _, _) <- rows]
  ys = [(s * 1000000) % sc | (_, s, sc) <- rows]
  n = fromIntegral (length rows)
  (mx, my) = (sum xs / n, sum ys / n)
  dx = map (subtract mx) xs
  dy = map (subtract my) ys

slopeGrid :: Bool
slopeGrid =
  and
    [ slopeMicroPerDay rows == Just (centered rows)
    | n <- [2 .. 5 :: Int]
    , scores <- sequences n [0, 250, 500, 1000]
    , let rows = zip3 [86400, 172800 ..] scores (repeat 1000)
    ]
    && slopeMicroPerDay [(0, 5, 10)] == Nothing
 where
  sequences 0 _ = [[]]
  sequences n alphabet = [v : rest | v <- alphabet, rest <- sequences (n - 1) alphabet]

-- | One day apart, one per-mille drop = -1000 micro-per-mille/day:
-- floor 999 says degrading, floor 1000 says flat (band inclusive),
-- rising mirrors to improving, constant is flat at every floor.
boundaries :: Bool
boundaries =
  map (`verdictOf` (-1000)) [0, 999, 1000] == [2, 2, 1]
    && map (`verdictOf` 1000) [0, 999, 1000] == [0, 0, 1]
    && map (`verdictOf` 0) [0, 5000] == [1, 1]

wireReq :: [[Integer]] -> Value
wireReq = rowsRequest "2.13.0" "trend.request"

replyObj :: Value -> Maybe Object
replyObj = replyObjWith respond

-- | Three points falling 1‰/day: floor 0 reports degrading but
-- cannot fail (report-only default); floor 500 declared = the SAME
-- rows now FAIL; floor 5000 = flat and clean. The knob echo carries
-- the declared floor for the round-trip pin.
floorLever :: Bool
floorLever = case (run Nothing, run (Just 500), run (Just 5000)) of
  (Just (v0, f0, _), Just (v1, f1, k1), Just (v2, f2, _)) ->
    (v0, f0) == (toJSON (2 :: Int), Bool False)
      && (v1, f1) == (toJSON (2 :: Int), Bool True)
      && k1 == toJSON [[0, 3], [1, 500 :: Integer]]
      && (v2, f2) == (toJSON (1 :: Int), Bool False)
  _ -> False
 where
  falling = [[86400, 900, 1000], [172800, 899, 1000], [259200, 898, 1000]]
  run floorMicro = do
    o <-
      replyObj
        ( maybe id (\f -> setKey "knobs" (toJSON [[1 :: Integer, f]])) floorMicro
            (wireReq falling)
        )
    (,,) <$> field o "verdict" <*> field o "fail" <*> field o "knobs"

-- | Absence is one posture, three doors: below minPoints, and an
-- all-tied window (same-second commits are legal rows whose slope
-- is underdetermined — never a refusal, never a fabricated flat).
absence :: Bool
absence =
  unjudged (wireReq [[86400, 500, 1000], [172800, 400, 1000]])
    && unjudged (wireReq [[86400, 500, 1000], [86400, 400, 1000], [86400, 300, 1000]])
 where
  unjudged req = case replyObj req of
    Nothing -> False
    Just o ->
      field o "verdict" == Just Null
        && field o "slopeMicroPerDay" == Just Null
        && field o "fail" == Just (Bool False)

refusals :: Bool
refusals =
  and
    [ refused (wireReq [[86400, 500]]) "malformed row (need [ts,score,scale])"
    , refused (wireReq [[86400, 500, 0]]) "non-positive scale"
    , refused (wireReq [[86400, 1001, 1000]]) "score outside 0..scale"
    , refused (wireReq [[172800, 1, 2], [86400, 1, 2]]) "ts descending"
    , refused (knobReq [[2, 1]]) "unknown knob code"
    , refused (knobReq [[0, 1]]) "minPoints below 2"
    ]
 where
  knobReq ks = setKey "knobs" (toJSON (ks :: [[Integer]])) (wireReq [])
  refused = refusedBy respond

degradedFails :: Bool
degradedFails = case replyObj (wireReq [[t, 1, 2] | t <- [0 .. trendRowCap]]) of
  Nothing -> False
  Just o ->
    field o "degraded" == Just (Bool True)
      && field o "fail" == Just (Bool True)
