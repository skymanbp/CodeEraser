-- | The split advisory's pressure model, the zone curve (CE.Structure.Zone,
-- its own model since v2.35): hand-exact at its edges and past them, and
-- against its second implementation (ReferenceSplit.refZone) over an
-- exhaustive grid. These legs lived in the verdict batteries until the
-- score became the extent law (10.0.0) and the advisory its one reader.
module SplitZoneProps (battery) where

import CE.Structure.Zone (zonePenalty)
import Data.Ratio ((%))
import ReferenceSplit (refZone)
import WireHarness (runChecks)

battery :: IO Bool
battery =
  runChecks
    [ ("the zone curve is hand-exact, monotone past H, degenerate-safe", zoneCurve)
    , ("the zone curve agrees with its reference on every (s, h, pMax, x) of the grid", zoneGrid)
    ]

-- | p at the edges and past them, exactly: 0 at S, P_max at H, 2.5
-- at midzone; past H the C¹ LINEAR arm (still costing — growth past
-- the wall is never free — but no quadratic outside the contracted
-- (S,H] domain: the batch-6 saturation lesson). 1200 = H + 450
-- prices 10·(1+2) = 30; one step past H adds exactly the slope
-- 2·P_max/(H−S) the quadratic reached AT H (no kink); and the
-- degenerate H<=S falls back to flat P_max instead of dividing by
-- zero. Since 9.0.0 the line is read as min S H: with S = 480 above
-- H = 400 a 450-line file past the wall prices P_max (it priced 0)
-- and one at 400 still prices nothing; H = S prices the same way.
zoneCurve :: Bool
zoneCurve =
  and
    [ zonePenalty 300 750 10 300 == 0
    , zonePenalty 300 750 10 525 == 5 % 2
    , zonePenalty 300 750 10 750 == 10
    , zonePenalty 300 750 10 1200 == 30
    , zonePenalty 300 750 10 751 - zonePenalty 300 750 10 750 == 2 * 10 % 450
    , zonePenalty 300 300 10 400 == 10
    , zonePenalty 300 200 10 400 == 10
    , zonePenalty 480 400 10 450 == 10
    , zonePenalty 480 400 10 400 == 0
    , zonePenalty 400 400 10 450 == 10
    , zonePenalty 300 200 10 250 == 10
    ]

-- | The curve against ReferenceSplit's one-fraction reading of it.
zoneGrid :: Bool
zoneGrid = and [zonePenalty s h p x == refZone s h p x | s <- [0 .. 6], h <- [0 .. 6], p <- [1, 10], x <- [0 .. 12]]
