-- | The trend-family knobs and the trajectory mathematics (M7.5b;
-- the estimator robust since 2.31.0 / trend/2): the score's path
-- over mainline history, judged in exact 'Rational' arithmetic (the
-- entropy/χ² stance — no floating point ever crosses a verdict).
-- Codes are frozen positions:
--   0 minPoints          fewer rows than this = no verdict (absence,
--                        never a fabricated flat) — default 3
--   1 declineFloorMicro  micro-per-mille per day; the fail bit trips
--                        only when a floor was DECLARED (>0) and the
--                        verdict is degrading — default 0 = the
--                        report-only posture every family launches in
-- ce.toml is the source, these are the DEFAULTS (the 27b9bc2
-- pattern), and the reply's knob echo pins the round trip.
module CE.Trend.Cost (
  cliffOf,
  declineRunOf,
  judgedView,
  knobTable,
  slopeMicroPerDay,
  trendRowCap,
  tsWindow,
  verdictOf,
) where

import Data.List (sort, sortBy, tails)
import Data.Ord (comparing)
import Data.Ratio ((%))

knobTable :: [(Integer, Integer)]
knobTable = [(0, 3), (1, 0)]

-- | Row ceiling: one row per mainline commit — 4096 covers a decade
-- of daily commits; over-cap answers a complete degraded reply that
-- FAILS (the P1 posture).
trendRowCap :: Integer
trendRowCap = 4096

-- | The judgment window. Theil-Sen prices n(n-1)/2 pairwise slopes,
-- so the O(n²) is bounded HERE, not by the wire cap: 512 recent
-- points = 130,816 exact-Rational pair slopes, measured at ~150 ms
-- end to end on the dev machine (median of 5, process spawn
-- included; 2026-08-24, request → reply over the wire). Older rows still cross and are counted
-- (counts.rows); the recent window is the trajectory judgment's
-- domain, and counts.judged names the cut.
tsWindow :: Int
tsWindow = 512

-- | The judged view: rows stable-sorted by timestamp (ties keep
-- request order — sortBy is stable), request indices preserved so
-- the client can name the commit behind a cliff by INDEX — indices
-- cross the wire, hashes and paths never do (§5.9.2) — then capped
-- to the tsWindow most recent points. ONE decomposition feeds
-- slope, cliff and decline run alike (the batch-9 P2 stance).
judgedView :: [[Integer]] -> [(Integer, Rational, Rational)]
judgedView rows = drop (length sorted - tsWindow) sorted
 where
  sorted =
    sortBy
      (comparing (\(_, x, _) -> x))
      [ (i, ts % 86400, (score * 1000000) % scale)
      | (i, [ts, score, scale]) <- zip [0 ..] rows
      ]

-- | Theil-Sen: the MEDIAN of pairwise slopes over timestamp-distinct
-- pairs. One wild point (a broken commit that still measured)
-- drags a least-squares mean anywhere; it cannot move a median past
-- its neighbors — TrendProps pins the counterfactual where the two
-- estimators disagree on the SIGN. Nothing when no pair has
-- distinct timestamps: same-second commits are legal rows (rebases,
-- scripted pushes), and the all-tied window has no RATE to state —
-- the falls it contains are still facts (cliffOf).
slopeMicroPerDay :: [(Integer, Rational, Rational)] -> Maybe Rational
slopeMicroPerDay view
  | null slopes = Nothing
  | otherwise = Just (median slopes)
 where
  slopes =
    [ (y2 - y1) / (x2 - x1)
    | ((_, x1, y1) : rest) <- tails view
    , (_, x2, y2) <- rest
    , x2 /= x1
    ]

-- | Order-statistic median, the even count averaged — exact in
-- Rational, deterministic by sort.
median :: [Rational] -> Rational
median xs
  | odd n = at mid
  | otherwise = (at (mid - 1) + at mid) / 2
 where
  sorted = sort xs
  n = length sorted
  mid = n `div` 2
  at k = sorted !! k

-- | The steepest single-step FALL between consecutive points of the
-- judged view: (request index of the LATER point, the exact drop in
-- micro units). The first occurrence wins a tie. Nothing when no
-- step falls. A fall between same-second commits counts — the drop
-- is a fact about scores, not about time.
cliffOf :: [(Integer, Rational, Rational)] -> Maybe (Integer, Rational)
cliffOf view = case falls of
  [] -> Nothing
  (f : fs) -> Just (foldl deeper f fs)
 where
  falls = [(i2, y1 - y2) | ((_, _, y1), (i2, _, y2)) <- steps view, y2 < y1]
  deeper a b = if snd b > snd a then b else a

-- | Consecutive pairs of the view — cliff and decline run read the
-- same walk.
steps ::
  [(Integer, Rational, Rational)] ->
  [((Integer, Rational, Rational), (Integer, Rational, Rational))]
steps view = zip view (drop 1 view)

-- | The longest run of consecutive strictly-falling steps: (request
-- index of the run's FIRST point, points in the run). The first run
-- wins a length tie. Nothing when no step falls — a single point is
-- not a decline.
declineRunOf :: [(Integer, Rational, Rational)] -> Maybe (Integer, Integer)
declineRunOf view = snd (foldl step (Nothing, Nothing) (steps view))
 where
  step (cur, top) ((i1, _, y1), (_, _, y2))
    | y2 < y1 =
        let cur' = maybe (i1, 2) (\(s, len) -> (s, len + 1)) cur
         in (Just cur', keep top cur')
    | otherwise = (Nothing, top)
  keep top run@(_, len) = case top of
    Just (_, best) | best >= len -> top
    _ -> Just run

-- | Sign classification against the declared floor: past the floor
-- downward = 2 (degrading), past it upward = 0 (improving), inside
-- the band = 1 (flat). With the default floor 0 the band is the
-- single point zero — the sign itself is the honest report; the
-- FAIL decision (floor declared AND degrading) lives at the reply,
-- not here.
verdictOf :: Integer -> Rational -> Integer
verdictOf floorMicro slope
  | slope < negate band = 2
  | slope > band = 0
  | otherwise = 1
 where
  band = fromInteger floorMicro
