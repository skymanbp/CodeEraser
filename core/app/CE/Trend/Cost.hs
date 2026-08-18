-- | The trend-family knobs and the slope mathematics (M7.5b): the
-- score's trajectory over mainline history, judged in exact
-- 'Rational' arithmetic (the entropy/χ² stance — no floating point
-- ever crosses a verdict). Codes are frozen positions:
--   0 minPoints          fewer rows than this = no verdict (absence,
--                        never a fabricated flat) — default 3
--   1 declineFloorMicro  micro-per-mille per day; the fail bit trips
--                        only when a floor was DECLARED (>0) and the
--                        verdict is degrading — default 0 = the
--                        report-only posture every family launches in
-- ce.toml is the source, these are the DEFAULTS (the 27b9bc2
-- pattern), and the reply's knob echo pins the round trip.
module CE.Trend.Cost (
  knobTable,
  slopeMicroPerDay,
  trendRowCap,
  verdictOf,
) where

import Data.Ratio ((%))

knobTable :: [(Integer, Integer)]
knobTable = [(0, 3), (1, 0)]

-- | Row ceiling: one row per mainline commit — 4096 covers a decade
-- of daily commits; over-cap answers a complete degraded reply that
-- FAILS (the P1 posture).
trendRowCap :: Integer
trendRowCap = 4096

-- | Least-squares slope of score-per-mille over days, in
-- micro-per-mille per day: x_i = ts_i / 86400 days, y_i =
-- score_i * 10^6 / scale_i micro-per-mille. Nothing when the system
-- is underdetermined: fewer than two rows, OR zero timestamp
-- variance — same-second commits are a legal, common shape (rebases
-- and scripted pushes land whole runs in one second), so ties ride
-- the wire and only the all-tied window has no slope to state.
-- Product form; TrendProps proves it equal to the centered-moments
-- definition.
slopeMicroPerDay :: [(Integer, Integer, Integer)] -> Maybe Rational
slopeMicroPerDay rows
  | n < 2 || den == 0 = Nothing
  | otherwise = Just ((nR * sxy - sx * sy) / den)
 where
  n = length rows
  nR = fromIntegral n
  xs = [ts % 86400 | (ts, _, _) <- rows]
  ys = [(score * 1000000) % scale | (_, score, scale) <- rows]
  sx = sum xs
  sy = sum ys
  sxy = sum (zipWith (*) xs ys)
  sxx = sum (map (\x -> x * x) xs)
  den = nR * sxx - sx * sx

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
