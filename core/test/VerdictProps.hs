-- | The verdict family's PURE score/ratchet battery (design §7.4,
-- M5-3i; the wire-driven half — ablation, idempotence, refusals,
-- knob-table probes — lives in VerdictWireProps since the ADR-008 P4
-- split at the 300-line law). The two F16 preconditions come FIRST
-- and void the battery when absent: every axis penalty nonzero on
-- the fixture AND penalties pairwise distinct AND the fixture
-- weights pairwise distinct — equal weights over equal penalties
-- make the weighted mean immune to exactly the perturbations this
-- battery exists to feel.
module VerdictProps (battery) where

import qualified CE.Verdict.Ratchet as R
import CE.Verdict.Score (Facts (..), ScoreKnobs (..), penalties, score, scoreBound)
import CE.Verdict.Soft (softLine, zonePenalty)
import Data.List (nub)
import Data.Ratio ((%))
import qualified Data.Set as S

battery :: IO Bool
battery = do
  a <- check "precondition: every axis penalty nonzero and pairwise distinct" preNonzero
  b <- check "precondition: battery weights pairwise distinct" preWeights
  c <- check "each weight +1 moves the (score, violations) tuple" weightKnobs
  d <- check "each axis threshold knob moves the score" axisKnobs
  e <- check "tolerance-over sets shrink as tolAbs grows (inclusion)" overMono
  f <- check "the soft line derives by order statistics and clamps both ends" softDerive
  g <- check "the zone curve is hand-exact, monotone past H, degenerate-safe" zoneCurve
  pure (and [a, b, c, d, e, f, g])

check :: String -> Bool -> IO Bool
check name ok = do
  putStrLn ((if ok then "ok   " else "FAIL ") <> name)
  pure ok

-- | The score fixture: per axis 0..6 the penalties are
-- 1,2,3,4,5,6,17, each axis with one row EXACTLY on its threshold so
-- a +1 knob probe has a boundary to flip. fPos is written
-- node-ascending (16 = the near-dead row only the deadIndegCeil
-- probe counts). The cycle axis carries 17, not 7: with 1..7 the
-- weighted MEAN landed exactly on penalty 5, and a +1 weight at the
-- mean moves nothing — pairwise-distinct penalties alone do not
-- prevent one axis SITTING on the mean, so the fixture keeps the
-- mean off every penalty (all seven bumps verified to move).
facts :: Facts
facts =
  Facts
    { fSim =
        [ [3, 4, 1, 85, 100] -- clone boundary
        , [5, 6, 1, 90, 100]
        , [7, 8, 0, 100, 100]
        , [9, 10, 2, 80, 100] -- dup boundary
        , [11, 12, 2, 85, 100]
        , [13, 14, 2, 90, 100]
        , [15, 16, 2, 100, 100]
        ]
    , fPos =
        [[16, 1, 0, 16, 1, 0]]
          <> [[u, 0, 0, u, 1, 0] | u <- [17 .. 21]]
          <> [[u, 1, 1, 50, 2, 1] | u <- [22 .. 38]]
    , fChurn =
        -- five clearly rewrite-heavy, one exactly at 50/100, one under
        [[u, 30, 10, 0, 0] | u <- [39 .. 43]]
          <> [[44, 20, 20, 0, 0], [45, 1, 50, 0, 0]]
    , -- 450 lines: the graded axis 0 (v0.6 soft zone, fallback
      -- S=300) charges floor(10·(150/450)²·…)=floor(10/9)=1 — the
      -- old 400 rounded to ZERO under the curve and voided the
      -- nonzero precondition
      fCont = [[0, 0, 450], [1, 1, 20], [2, 1, 30]]
    }

battWeights :: [[Integer]]
battWeights = [[c, c + 1] | c <- [0 .. 6]]

pens :: [(Integer, Integer)]
pens = penalties scoreBound Nothing facts

preNonzero :: Bool
preNonzero =
  map fst pens == [0 .. 6]
    && all ((> 0) . snd) pens
    && length (nub (map snd pens)) == 7

preWeights :: Bool
preWeights = length (nub (map (!! 1) battWeights)) == 7

baseTuple :: (Integer, Integer)
baseTuple = score scoreBound battWeights pens

weightKnobs :: Bool
weightKnobs = and [score scoreBound (bump c) pens /= baseTuple | c <- [0 .. 6]]
 where
  bump c = [[c', if c' == c then w + 1 else w] | [c', w] <- battWeights]

axisKnobs :: Bool
axisKnobs =
  all
    (\k -> score k battWeights (penalties k Nothing facts) /= baseTuple)
    [ scoreBound {sSizeCeil = sSizeCeil scoreBound + 100}
    , scoreBound {sCocCeil = sCocCeil scoreBound + 5}
    , scoreBound {sCloneNum = sCloneNum scoreBound + 1}
    , scoreBound {sDupNum = sDupNum scoreBound + 1}
    , scoreBound {sDeadIndegCeil = sDeadIndegCeil scoreBound + 1}
    , scoreBound {sRewriteNum = sRewriteNum scoreBound + 1}
    , scoreBound {sCycleFloor = sCycleFloor scoreBound + 1}
    , -- the last scoring literal, now a knob (ADR-008 P4 gap 1)
      scoreBound {sScoreScale = sScoreScale scoreBound + 100}
    , -- the v0.6 zone knobs are live levers too (F16): a farther
      -- hard line flattens the curve (1 -> 0 on the 450 row), a
      -- doubled P_max steepens it (1 -> 2)
      scoreBound {sSizeHard = sSizeHard scoreBound + 250}
    , scoreBound {sSizePMax = sSizePMax scoreBound + 10}
    ]

-- | Monotonicity as SET INCLUSION (never count comparison): growing
-- the absolute tolerance leg can only shrink the over set, checked
-- on (entity, metric) identities across seeded fact tables.
overMono :: Bool
overMono = all one [1 .. 60 :: Integer]
 where
  one i =
    let cont = [[u, 0, 100 + ((i * (u + 1) * 7) `mod` 40)] | u <- [0 .. 9]]
        base = R.Baseline [[u, 0, 100] | u <- [0 .. 9]] [] Nothing
        ids k = S.fromList [(u, c) | [u, c, _, _] <- R.rOver (R.ratchet k (Just base) cont [])]
        wide = R.ratchetBound {R.rTolAbs = R.rTolAbs R.ratchetBound + 15}
     in ids wide `S.isSubsetOf` ids R.ratchetBound

-- | Hand-computed derivations (plan v2.6 §B, the log-free form):
-- {240,300,375} → m=300, r=5/4, k=2 → floor(300·25/16)=468;
-- {100,300} even-median → m=200, r=7/4 → 612 → clamp 500;
-- all-100 → r=1 → 100 → clamp 200; k=1 halves the spread's bite;
-- zeros are dropped and an empty multiset derives NOTHING.
softDerive :: Bool
softDerive =
  and
    [ softLine 2 200 500 [240, 300, 375] == Just 468
    , softLine 2 200 500 [100, 300] == Just 500
    , softLine 2 200 500 [100, 100, 100] == Just 200
    , softLine 1 200 500 [240, 300, 375] == Just 375
    , softLine 2 200 500 [0, 0] == Nothing
    , softLine 2 200 500 [] == Nothing
    ]

-- | p at the edges and past them, exactly: 0 at S, P_max at H, 2.5
-- at midzone, quadratic past H (no cap — growth past the wall keeps
-- costing), and the degenerate H<=S falls back to flat P_max
-- instead of dividing by zero.
zoneCurve :: Bool
zoneCurve =
  and
    [ zonePenalty 300 750 10 300 == 0
    , zonePenalty 300 750 10 525 == 5 % 2
    , zonePenalty 300 750 10 750 == 10
    , zonePenalty 300 750 10 1200 == 40
    , zonePenalty 300 300 10 400 == 10
    , zonePenalty 300 200 10 400 == 10
    ]
