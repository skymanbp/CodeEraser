-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | The trend family against its second implementation (plan v2.32
-- step 7): every reply field a reader of `trend.result` sees — the
-- rounded slope, the verdict, the cliff, the decline run, the fail
-- bit, the knob echo, the counts, the degradation — must agree with
-- ReferenceTrend on every sequence of up to four points over a
-- three-day by three-score grid under three knob tables, on two
-- hundred seeded histories (shuffled, tied timestamps, declared
-- floors), on a history past the judgment window, on the contract's
-- refusals and on both sides of the cap.
module TrendEquivProps (battery) where

import CE.Trend (respond)
import CE.Trend.Cost (trendRowCap, tsWindow)
import Data.Ratio ((%))
import ReferenceFlowGen (S (..), rand, runG)
import ReferenceContract (answers, knobbedRequest, pairTable)
import ReferenceTrend (expected, roundEven)
import WireHarness (runLegs)

battery :: IO Bool
battery =
  runLegs
    [ "every grid history of up to four points agrees under three knob tables"
    , "two hundred seeded histories agree, shuffled and tied"
    , "a history past the judgment window agrees, and counts the cut"
    , "the contract's refusals agree with the reference's predicates"
    , "both sides of the cap agree, rows and knobs counted together"
    , "half-to-even rounding on both sides of every half"
    ]
    [ all agrees [(rs, ks) | n <- [0 .. 4], rs <- sequence (replicate n grid), ks <- knobTables]
    , all agrees seeded
    , 640 > tsWindow && agrees (history 640 7, [[0, 2]])
    , all agrees refusals
    , all agrees [(history (fromInteger trendRowCap) 3, replicate k [1, 1]) | k <- [0, 1]]
    , map roundEven [5 % 2, 7 % 2, -5 % 2, 1 % 3, -1 % 3, 2] == map round [5 % 2, 7 % 2, -5 % 2, 1 % 3, -1 % 3, 2 :: Rational]
    ]

-- | Three days by three scores on a scale of two.
grid :: [[Integer]]
grid = [[d * 86400, s, 2] | d <- [0 .. 2], s <- [0 .. 2]]

knobTables :: [[[Integer]]]
knobTables = [[], [[0, 2]], [[0, 2], [1, 400000]]]

agrees :: ([[Integer]], [[Integer]]) -> Bool
agrees (rows, knobs) = answers respond keys (knobbedRequest "trend.request" rows knobs) (expected rows knobs)
 where
  keys = ["slopeMicroPerDay", "verdict", "cliff", "declineRun", "fail", "knobs", "counts", "degraded", "reason"]

-- | n points over a fortnight with scores on a scale of ten, seeded.
history :: Int -> Int -> [[Integer]]
history n s = runG (mapM (const point) [1 .. n]) (S s 0 0 0)
 where
  point = do
    day <- rand 14
    sec <- rand 3
    score <- rand 11
    pure [toInteger day * 86400 + toInteger sec * 3600, toInteger score, 10]

-- | Two hundred seeded histories of zero to thirty points (repeated
-- seconds, shuffled order, odd scales) under a seeded knob table.
seeded :: [([[Integer]], [[Integer]])]
seeded = [runG one (S (n * 7121 + 5) 0 0 0) | n <- [1 .. 200 :: Int]]
 where
  one = do
    count <- rand 31
    rows <- mapM (const row) [1 .. count]
    pick <- rand 4
    floorMicro <- rand 3
    let knobs = [[[0, 4]], [[1, toInteger floorMicro * 300000]], [[0, 2], [1, 1]], []]
    pure (rows, knobs !! pick)
  row = do
    scale <- (+ 1) <$> rand 7
    score <- rand (scale + 1)
    ts <- rand 9
    pure [toInteger ts * 43200, toInteger score, toInteger scale]

-- | A row offence of each kind, then each knob offence; an offending
-- row stands before an offending knob.
refusals :: [([[Integer]], [[Integer]])]
refusals =
  pairTable
    [ "[[-1,1,2]] ; []"
    , "[[0,1,0]] ; []"
    , "[[0,3,2]] ; []"
    , "[[0,1]] ; []"
    , "[[0,1,2]] ; [[2,1]]"
    , "[[0,1,2]] ; [[1,-1]]"
    , "[[0,1,2]] ; [[0,1]]"
    , "[[0,1,2]] ; [[1,0],[0,3]]"
    , "[[0,1,2]] ; [[0,3,4]]"
    , "[[0,9,2]] ; [[5,5]]"
    ]
