-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | The trend family's battery (M7.5b; trend/2 since 2.31.0): the
-- Theil-Sen slope against an independent derivation (index-pair
-- enumeration + insertion-sort median — exact Rational, equality is
-- equality), the robustness counterfactual where mean and median
-- disagree on the SIGN, cliff and decline-run shape facts, the
-- tsWindow cut, sign/floor boundaries, the knob lever through the
-- REAL respond, refusal-by-name, the below-minPoints absence, and
-- the degraded-fails posture. Scaffolding lives in WireHarness.
module TrendProps (battery) where

import CE.Trend (respond)
import CE.Trend.Cost (judgedView, slopeMicroPerDay, trendRowCap, tsWindow, verdictOf)
import Data.Aeson
import Data.Ratio ((%))
import WireHarness (field, fieldsOf, refusedBy, replyObjWith, rowsRequest, runChecks, setKey)

battery :: IO Bool
battery =
  runChecks
    [ ("Theil-Sen slope == index-pair insertion-median on the grid", slopeGrid)
    , ("a collinear window's robust slope IS the line's slope", collinear)
    , ("one wild point flips the mean's sign, never the median's", robustness)
    , ("sign and floor boundaries classify exactly", boundaries)
    , ("a declared floor flips verdict and fail through respond", floorLever)
    , ("the cliff is the steepest fall, named by request index", cliffs)
    , ("the decline run is the longest, the first wins ties", runs)
    , ("the judged view caps at tsWindow recent, stable on ties", windowed)
    , ("below minPoints every judgment field is ABSENT", absence)
    , ("trend refusals name the offender", refusals)
    , ("row order is data: a shuffled window judges the same facts", orderFree)
    , ("an over-cap trend request degrades to a reply that FAILS", degradedFails)
    ]

-- | Independent derivation: pairwise slopes by INDEX enumeration
-- over the raw triples (not the view's tails walk), median by
-- insertion sort (not Data.List.sort) — different roads, same
-- exact Rational.
tsRef :: [(Integer, Integer, Integer)] -> Maybe Rational
tsRef rows = case slopes of
  [] -> Nothing
  ss -> Just (mid (foldr ins [] ss))
 where
  pts = [(ts % 86400, (s * 1000000) % sc) | (ts, s, sc) <- rows]
  k = length pts
  slopes =
    [ (snd (pts !! j) - snd (pts !! i)) / (fst (pts !! j) - fst (pts !! i))
    | i <- [0 .. k - 1]
    , j <- [i + 1 .. k - 1]
    , fst (pts !! i) /= fst (pts !! j)
    ]
  ins x [] = [x]
  ins x (y : ys) = if x <= y then x : y : ys else y : ins x ys
  mid sorted =
    let n = length sorted
     in if odd n
          then sorted !! (n `div` 2)
          else (sorted !! (n `div` 2 - 1) + sorted !! (n `div` 2)) / 2

slopeGrid :: Bool
slopeGrid =
  and
    [ slopeMicroPerDay (judgedView [[t, s, 1000] | (t, s) <- rows]) == tsRef [(t, s, 1000) | (t, s) <- rows]
    | n <- [2 .. 5 :: Int]
    , scores <- sequences n [0, 250, 500, 1000]
    , let rows = zip [86400, 172800 ..] scores
    ]
    && slopeMicroPerDay (judgedView [[0, 5, 10]]) == Nothing
 where
  sequences 0 _ = [[]]
  sequences n alphabet = [v : rest | v <- alphabet, rest <- sequences (n - 1) alphabet]

-- | On an exact line the pairwise slopes are all the line's slope,
-- so the median IS it — pinned at three slopes including zero.
collinear :: Bool
collinear = all probe [-1000, 0, 777]
 where
  probe b =
    slopeMicroPerDay (judgedView [[86400 * d, 500000 + b * d, 1000000] | d <- [1 .. 4]])
      == Just (fromInteger b)

-- | THE counterfactual that bought the estimator change: five
-- points falling 1‰/day plus one wild high outlier. The retired
-- least-squares mean (kept HERE as the reference) says +14000 —
-- improving; the median says -1000 — degrading, through the real
-- respond. One broken-but-measured commit must not flip the gate.
robustness :: Bool
robustness = case replyObj (wireReq spiked) of
  Nothing -> False
  Just o ->
    field o "verdict" == Just (toJSON (2 :: Integer))
      && field o "slopeMicroPerDay" == Just (toJSON (-1000 :: Integer))
      && olsRef > 0
 where
  spiked = daily [900, 899, 898, 897, 896, 1000]
  olsRef = sum (zipWith (*) dx dy) / sum (map (\d -> d * d) dx)
   where
    xs = [fromInteger d | d <- [1 .. 6 :: Integer]] :: [Rational]
    ys = [s * 1000 | s <- [900, 899, 898, 897, 896, 1000]]
    n = 6 :: Rational
    (mx, my) = (sum xs / n, sum ys / n)
    dx = map (subtract mx) xs
    dy = map (subtract my) ys

-- | One day apart, one per-mille drop = -1000 micro-per-mille/day:
-- floor 999 says degrading, floor 1000 says flat (band inclusive),
-- rising mirrors to improving, constant is flat at every floor.
boundaries :: Bool
boundaries =
  map (`verdictOf` (-1000)) [0, 999, 1000] == [2, 2, 1]
    && map (`verdictOf` 1000) [0, 999, 1000] == [0, 0, 1]
    && map (`verdictOf` 0) [0, 5000] == [1, 1]

wireReq :: [[Integer]] -> Value
wireReq = rowsRequest "6.1.0" "trend.request"

-- | The fixture shape every probe speaks: one commit per day at
-- scale 1000, scores as given. Shuffled or tied-timestamp windows
-- stay literal — their ts pattern IS the probe.
daily :: [Integer] -> [[Integer]]
daily scores = [[86400 * d, s, 1000] | (d, s) <- zip [1 ..] scores]

replyObj :: Value -> Maybe Object
replyObj = replyObjWith respond

-- | replyObj's many-field sibling: rows to request to projected
-- reply fields in one call (WireHarness.fieldsOf with this
-- battery's respond and request builder applied once).
project :: [[Integer]] -> [String] -> Maybe [Maybe Value]
project rows = fieldsOf respond (wireReq rows)

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
  falling = daily [900, 899, 898]
  run floorMicro = do
    o <-
      replyObj
        ( maybe id (\f -> setKey "knobs" (toJSON [[1 :: Integer, f]])) floorMicro
            (wireReq falling)
        )
    (,,) <$> field o "verdict" <*> field o "fail" <*> field o "knobs"

-- | The cliff names the LATER point of the steepest fall by request
-- index — the request is deliberately shuffled so the index proves
-- itself against the ts-sorted walk. A monotone rise has no cliff;
-- equal falls keep the first; the all-tied window (no slope to
-- state) still reports its falls — a drop between same-second
-- commits is a fact about scores, not about time.
cliffs :: Bool
cliffs =
  and
    [ facts shuffledFall == want [0, 50000] [1, 4]
    , facts rising == Just [Just Null, Just Null]
    , facts equalFalls == want [1, 10000] [0, 3]
    , project allTied ["slopeMicroPerDay", "cliff", "declineRun"]
        == Just [Just Null, Just (toJSON [1, 100000 :: Integer]), Just (toJSON [0, 3 :: Integer])]
    ]
 where
  -- ts order: idx1(900) -> idx2(890) -> idx3(850) -> idx0(800)
  shuffledFall =
    [[345600, 800, 1000], [86400, 900, 1000], [172800, 890, 1000], [259200, 850, 1000]]
  rising = daily [501, 502, 503]
  equalFalls = daily [900, 890, 880]
  allTied = [[86400, 500, 1000], [86400, 400, 1000], [86400, 300, 1000]]
  facts rows = project rows ["cliff", "declineRun"]
  want c r = Just [Just (toJSON (c :: [Integer])), Just (toJSON (r :: [Integer]))]

-- | Longest strictly-falling run of consecutive points, first wins
-- a tie: fall/rise/fall/fall/rise picks the late 3-point run; two
-- 2-point runs pick the earlier.
runs :: Bool
runs =
  run [900, 880, 900, 890, 880, 900] == Just (toJSON [2, 3 :: Integer])
    && run [900, 880, 900, 880] == Just (toJSON [0, 2 :: Integer])
 where
  run scores = do
    o <- replyObj (wireReq (daily scores))
    field o "declineRun"

-- | 513 rows: the OLDEST is a wild outlier, and the tsWindow cut
-- leaves it no trace — the 512 kept points are an exact line, the
-- slope is exactly the line's, and counts names both numbers. The
-- pure view is pinned beside it: length, the dropped index, and
-- tie stability (equal ts keeps request order).
windowed :: Bool
windowed =
  length bigView == tsWindow
    && [i | (i, _, _) <- take 1 bigView] == [1]
    && [i | (i, _, _) <- judgedView tied] == [0, 1]
    && case replyObj (wireReq big) of
      Nothing -> False
      Just o ->
        field o "slopeMicroPerDay" == Just (toJSON (-500 :: Integer))
          && field o "counts"
            == Just (object ["rows" .= (513 :: Int), "judged" .= (512 :: Int)])
 where
  big = [0, 1000000, 1000000] : [[86400 * d, 800000 - 500 * d, 1000000] | d <- [1 .. 512]]
  bigView = judgedView big
  tied = [[100, 1, 2], [100, 0, 2]]

-- | Below minPoints EVERY judgment field is absent — slope,
-- verdict, cliff, decline run — and the fail bit stays false:
-- nothing was judged, and an unjudged trend must not gate (unlike
-- a degraded reply, where judgment was DENIED by the cap and
-- fail=true says so).
absence :: Bool
absence = case replyObj (wireReq [[86400, 500, 1000], [172800, 400, 1000]]) of
  Nothing -> False
  Just o ->
    all
      (\k -> field o k == Just Null)
      ["slopeMicroPerDay", "verdict", "cliff", "declineRun"]
      && field o "fail" == Just (Bool False)

refusals :: Bool
refusals =
  and
    [ refused (wireReq [[86400, 500]]) "malformed row (need [ts,score,scale])"
    , refused (wireReq [[86400, 500, 0]]) "non-positive scale"
    , refused (wireReq [[86400, 1001, 1000]]) "score outside 0..scale"
    , refused (knobReq [[2, 1]]) "unknown knob code"
    , refused (knobReq [[0, 1]]) "minPoints below 2"
    ]
 where
  knobReq ks = setKey "knobs" (toJSON (ks :: [[Integer]])) (wireReq [])
  refused = refusedBy respond

-- | The retired ts-descending refusal, inverted (review 2026-08-20
-- #9): the judged view sorts, so a shuffled window states the same
-- slope, verdict, fail — and the same cliff FACT: the index moves
-- with the request, the timestamp it points at must not.
orderFree :: Bool
orderFree = case (run shuffled, run sorted) of
  (Just [s1, v1, f1, c1], Just [s2, v2, f2, c2]) ->
    (s1, v1, f1) == (s2, v2, f2)
      && s1 /= Just Null
      && cliffTs shuffled c1 == cliffTs sorted c2
      && cliffTs sorted c2 == Just 259200
  _ -> False
 where
  sorted = daily [900, 899, 850]
  shuffled = [[259200, 850, 1000], [86400, 900, 1000], [172800, 899, 1000]]
  run rows = project rows ["slopeMicroPerDay", "verdict", "fail", "cliff"]
  cliffTs rows c = case fmap fromJSON c of
    Just (Success [i, _]) | (ts : _) <- rows !! fromInteger i -> Just ts
    _ -> Nothing

degradedFails :: Bool
degradedFails = case replyObj (wireReq [[t, 1, 2] | t <- [0 .. trendRowCap]]) of
  Nothing -> False
  Just o ->
    field o "degraded" == Just (Bool True)
      && field o "fail" == Just (Bool True)
