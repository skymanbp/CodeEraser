-- | The split advisory's pressure model (plan v2.6 §C; its own model
-- since v2.35): how much a file between the soft line S and the hard
-- line H weighs on its reader, so that a seam can be priced as the
-- weight a split takes off. It was the verdict family's size curve
-- too until the score became the extent law (10.0.0), which reads
-- lines past S and no curve; the advisory keeps the curve because the
-- hard line is still where the guard refuses a write, and the urgency
-- near the wall belongs in the advice about splitting before it.
-- CE.Structure.Split is its one reader; S, H and P_max ride as the
-- structure knobs 12, 13 and 14 (CE.Structure.Cost).
module CE.Structure.Zone
  ( zonePenalty
  , classedZone
  ) where

import qualified Data.Map.Strict as M
import Data.Ratio ((%))

-- | p(x) for one file: 0 at or under the soft line; the convex
-- curve pMax·((x−s)/(h−s))² on the zone (s,h]; past the hard line
-- the C¹ LINEAR continuation pMax·(1 + 2(x−h)/(h−s)) — the slope at
-- h⁻ is 2·pMax/(h−s) and the linear arm keeps it exactly. In the
-- advisory's terms: a seam in a file far past the wall is still worth
-- more than one in a file at it — every line past H weighs what a line
-- at H weighed, by the slope the curve reached there and no more — so
-- splitting the larger file always earns more, and the convex square
-- does not run on outside the zone (S,H] it was drawn for
-- (size-advisory.md §A). The deny AT the wall is the guard's job.
--
-- The curve opens at s' = min s h (9.0.0): the relative line widens
-- the comfort zone, never past the wall. A baseline committed before
-- 9.0.0 may carry S above H, and a class's own hard line may sit
-- below the global S — either way a file past H would price 0
-- here. Degenerate h <= s' (only h == s' now) falls back to the
-- pre-v0.6 binary (x > s' costs pMax flat): a misdeclared hard line
-- must not divide by zero or flip the sign.
zonePenalty :: Integer -> Integer -> Integer -> Integer -> Rational
zonePenalty s0 h pMax x
  | x <= s = 0
  | h <= s = pMax % 1
  | x <= h = (pMax % 1) * step * step
  | otherwise = (pMax % 1) * (1 + 2 * ((x - h) % (h - s)))
 where
  s = min s0 h
  step = (x - s) % (h - s)

-- | zonePenalty for a file of class `cls` (0 = none): the class's own
-- opening edge (class code 0) and hard line (class code 2) where the
-- rulepack's class rows declare them, the global (s, h) otherwise;
-- P_max has no class channel. The class rows are the verdict's own
-- (classKnobs, 9.0.0), so a classed file's advice opens and walls
-- where its class's soft line and hard line stand.
classedZone :: M.Map (Integer, Integer) Integer -> Integer -> (Integer, Integer, Integer) -> Integer -> Rational
classedZone knobs cls (s, h, pMax) = zonePenalty (line 0 s) (line 2 h) pMax
 where
  line code g = M.findWithDefault g (cls, code) knobs
