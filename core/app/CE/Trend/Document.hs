-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | The `ce trend` document (plan v2.32 step 5; design booklet
-- docs/reference/authority-track.md §5), transcribed from the face that
-- assembled it on the measuring side (cli/src/trend/report.rs
-- `report_json`, cli/src/trend/judge.rs `Row` and `judgment_json`): the
-- window, what is still pending, every measured point oldest first
-- with its score, scale and axes, the commits that refused to measure
-- with why, and the trend/2 judgment relayed whole — the slope, the
-- verdict code, the cliff and the decline run (each pointing at a
-- point), the fail bit and the knob echo. The measuring side sends its
-- points and the reply's six keys as integers; commits and the
-- refusals' reasons are references.
module CE.Trend.Document (doc) where

import CE.Document.Contract
import Data.Aeson (Value (..), object, toJSON, (.=))
import Data.Foldable (asum)

doc :: DocFamily
doc = docFamily "trend" "ce.trend-report/0.3.0" statement checked assemble []

-- | A point row is [i, ts, score, scale, then each axis's code and
-- value], oldest first; `slope`, `verdict`, `cliff` [point, drop] and
-- `declineRun` [point, length] the reply's, one row at most (none:
-- null); `knobs` the reply's echo [code, value]; `fail` its bit;
-- `window` and `pending` the mainline counts; each refusal is a
-- reference pair by its place.
statement :: String
statement =
  "range points\nrange failed\n\
  \fact window judged\nfact pending judged\nfact fail judged\n\
  \rows points 4+ judged points - - -\n\
  \rows slope 1 judged -\nrows verdict 1 judged -\nrows cliff 2 judged points -\nrows declineRun 2 judged points -\n\
  \rows knobs 2 judged - -\n\
  \ref commit points\nref short points\nref sha failed\nref reason failed\n"

checked :: DocReq -> Maybe String
checked req =
  asum
    [ dense req "points" (range req "points")
    , asum [Just ("points " <> show i <> ": an axis without its value") | (i, r) <- zip [0 :: Int ..] (rows req "points"), odd (length r)]
    , asum [single req t | t <- words "slope verdict cliff declineRun"]
    , codes req "verdict" 0 0 2
    , bits req ["fail"]
    ]

assemble :: DocReq -> Value
assemble req =
  object
    [ "schema" .= dfSchema doc
    , "window" .= fact req "window"
    , "pending" .= fact req "pending"
    , "rows" .= map point (rows req "points")
    , "failed" .= [[ref "sha" [k], ref "reason" [k]] | k <- [0 .. range req "failed" - 1]]
    , "judgment" .= judgment
    ]
 where
  judgment =
    object
      [ "slopeMicroPerDay" .= optional req "slope"
      , "verdict" .= optional req "verdict"
      , "cliff" .= whole "cliff"
      , "declineRun" .= whole "declineRun"
      , "fail" .= flag req "fail"
      , "knobs" .= rows req "knobs"
      ]
  whole t = case rows req t of
    [r] -> toJSON r
    _ -> Null
  point r = case r of
    i : ts : score : scale : axes -> object ["commit" .= ref "commit" [i], "ts" .= ts, "score" .= score, "scale" .= scale, "axes" .= pairs axes]
    _ -> object []
  pairs xs = case xs of
    c : p : rest -> [c, p] : pairs rest
    _ -> []
