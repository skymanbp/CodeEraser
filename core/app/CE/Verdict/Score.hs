-- | The seven-axis score (design §7.4): polarity higher-is-better,
-- integer per-mille, penalties DERIVED from the request's fact
-- tables, weights effective per axis (wire override, default 1) and
-- wTotal derived from them — never a literal (the destFloor
-- convention: a hand-typed total is how a weight silently dies).
-- Knobs travel as parameters; production binds them at scoreBound —
-- the batteries perturb fields instead (the 2g mechanism). Axis
-- thresholds reuse their owning families where one exists (clone
-- 85/100, docdup 80/100, cycle floor = the graph family's sccFloor);
-- size/complexity ceilings are the plan §4.1 dogfood numbers.
module CE.Verdict.Score
  ( Facts (..)
  , ScoreKnobs (..)
  , chargeAt
  , scoreBound
  , effectiveWeights
  , penalties
  , score
  ) where

import CE.Clone.Cost (tsedDen, tsedNum)
import CE.Docdup.Cost (jaccardDen, jaccardNum)
import CE.Graph.Cost (sccFloor)
import qualified CE.Verdict.Cost as Cost
import CE.Verdict.Soft (zonePenalty)
import Data.Ratio ((%))

-- | The validated fact tables (row shapes enforced by CE.Verdict's
-- boundary contract before anything reaches here).
data Facts = Facts
  { fSim :: [[Integer]]
  -- ^ [u,v,simKind,num,den]
  , fPos :: [[Integer]]
  -- ^ [u,indeg,outdeg,sccId,sccSize,reachIn]
  , fChurn :: [[Integer]]
  -- ^ [u,rewrite,append,added,survived]
  , fCont :: [[Integer]]
  -- ^ [u,metricCode,value]
  }

data ScoreKnobs = ScoreKnobs
  { sSizeCeil :: Integer
  , sSizeHard :: Integer
  , sSizePMax :: Integer
  , sSoftK :: Integer
  , sCocCeil :: Integer
  , sCloneNum :: Integer
  , sCloneDen :: Integer
  , sDupNum :: Integer
  , sDupDen :: Integer
  , sDeadIndegCeil :: Integer
  , sRewriteNum :: Integer
  , sRewriteDen :: Integer
  , sCycleFloor :: Integer
  , sViolCost :: Integer
  , sDefaultWeight :: Integer
  , sScoreScale :: Integer
  }

scoreBound :: ScoreKnobs
scoreBound =
  ScoreKnobs
    { sSizeCeil = Cost.sizeCeil
    , sSizeHard = Cost.sizeHard
    , sSizePMax = Cost.sizePMax
    , sSoftK = Cost.softLineK
    , sCocCeil = Cost.cocCeil
    , sCloneNum = tsedNum
    , sCloneDen = tsedDen
    , sDupNum = jaccardNum
    , sDupDen = jaccardDen
    , sDeadIndegCeil = Cost.deadIndegCeil
    , sRewriteNum = Cost.rewriteNum
    , sRewriteDen = Cost.rewriteDen
    , sCycleFloor = sccFloor
    , sViolCost = Cost.violCost
    , sDefaultWeight = Cost.defaultWeight
    , sScoreScale = Cost.scoreScale
    }

-- | Per-axis CHARGE in per-mille of the scale — 0 size / 1
-- complexity / 2 clone / 3 docdup / 4 deadcode / 5 churn /
-- 6 graph_cycle (decision ⑦). Each axis pairs a violation MASS with
-- its OPPORTUNITY count and maps the odds through v/(v+n) — bounded
-- below the scale, strictly monotone in v (two repos with different
-- mass always differ before the floor), scale-free across repo
-- sizes, exact Rational throughout. The M9 batch-6 field test is
-- why: raw mass summed onto the bounded scale zeroed BOTH real
-- repositories measured (10176‰ and 4325‰ of size mass alike), the
-- exact dead-field saturation this project was founded against.
-- Every axis is still one NAMED predicate owning exactly one knob,
-- so the per-axis perturbation battery has a lever per row.
penalties :: ScoreKnobs -> Maybe Integer -> Facts -> [(Integer, Integer)]
penalties k soft f =
  [ (code, charge k v n)
  | (code, v, n) <-
      [ (0, sizeMass k soft f, files)
      , (1, cnt (cocOver k f), functions)
      , (2, cnt (cloneHits k f), files)
      , (3, cnt (dupHits k f), files)
      , (4, cnt (deadFiles k f), nodes)
      , (5, cnt (churnHeavy k f), churned)
      , (6, cnt (cycleMembers k f), nodes)
      ]
  ]
 where
  cnt = fromInteger :: Integer -> Rational
  files = count [() | [_, 0, _] <- fCont f]
  functions = count [() | [_, 1, _] <- fCont f]
  nodes = toInteger (length (fPos f))
  churned = toInteger (length (fChurn f))

-- | floor(scale · v/(v+n)): the odds→probability map. n = 0 means
-- no opportunity table (churn without --days) — no evidence charges
-- nothing, the honest-absence stance.
charge :: ScoreKnobs -> Rational -> Integer -> Integer
charge k = chargeAt (sScoreScale k)

-- | The scale-parameterized form — ONE density law, two scoring
-- families: CE.Structure imports this (batch 9 P9, the Split→Soft
-- precedent) so the structure fold can never re-diverge into the
-- raw-mass shape the batch-6 field test retired.
chargeAt :: Integer -> Rational -> Integer -> Integer
chargeAt scale v n
  | n <= 0 || v <= 0 = 0
  | otherwise = floor ((scale % 1) * v / (v + n % 1))

count :: [()] -> Integer
count = toInteger . length

-- | Axis-0 violation mass under the plan-v2.6 soft zone: the exact-
-- Rational zone penalties summed across every metricCode-0 row (in
-- P_max units — one hard-line file weighs P_max old violations, so
-- the mass shares the counting axes' scale). `soft` is the
-- baseline's committed S (Nothing = pre-v0.6 baseline, or no
-- derivable distribution) falling back to the sSizeCeil knob — the
-- old binary line becomes the zone's opening edge, never a cliff.
sizeMass :: ScoreKnobs -> Maybe Integer -> Facts -> Rational
sizeMass k soft f =
  sum [zonePenalty s (sSizeHard k) (sSizePMax k) v | [_, 0, v] <- fCont f]
 where
  s = maybe (sSizeCeil k) id soft

cocOver :: ScoreKnobs -> Facts -> Integer
cocOver k f = count [() | [_, 1, v] <- fCont f, v > sCocCeil k]

cloneHits :: ScoreKnobs -> Facts -> Integer
cloneHits k f =
  count [() | [_, _, kind, n, d] <- fSim f, kind <= 1, n * sCloneDen k >= d * sCloneNum k]

dupHits :: ScoreKnobs -> Facts -> Integer
dupHits k f = count [() | [_, _, 2, n, d] <- fSim f, n * sDupDen k >= d * sDupNum k]

deadFiles :: ScoreKnobs -> Facts -> Integer
deadFiles k f = count [() | [_, indeg, _, _, _, 0] <- fPos f, indeg <= sDeadIndegCeil k]

churnHeavy :: ScoreKnobs -> Facts -> Integer
churnHeavy k f =
  count [() | [_, rw, ap, _, _] <- fChurn f, rw + ap > 0, rw * sRewriteDen k >= (rw + ap) * sRewriteNum k]

cycleMembers :: ScoreKnobs -> Facts -> Integer
cycleMembers k f = count [() | [_, _, _, _, size, _] <- fPos f, size >= sCycleFloor k]

-- | One axis's effective weight: wire rows [axisCode, numerator]
-- override; unlisted axes weigh sDefaultWeight. ONE lookup, two
-- readers — the score fold and the reply's echo (review C3: weights
-- was the only knob family without a round trip; a fold-local
-- lookup was where the echo could silently diverge from the score).
effWeight :: ScoreKnobs -> [[Integer]] -> Integer -> Integer
effWeight k weights code = case [w | [c, w] <- weights, c == code] of
  (w : _) -> w
  [] -> sDefaultWeight k

-- | The full effective table 0..6 for the reply's echo.
effectiveWeights :: ScoreKnobs -> [[Integer]] -> [[Integer]]
effectiveWeights k weights = [[c, effWeight k weights c] | c <- [0 .. 6]]

-- | (perMille, total charge) under the effective weights; wTotal =
-- sum of effective weights (derived); validation refuses an all-zero
-- weight table, so the divisor is never 0. violCost is the global
-- strictness dial: at its neutral default (Cost.violCostNeutral) the
-- weighted mean of the bounded axis charges lands as-is and the
-- `max 0` is unreachable (every charge < scale); a repo dialing
-- violCost above neutral asks for harsher scores and may saturate —
-- by explicit choice, never by construction.
score :: ScoreKnobs -> [[Integer]] -> [(Integer, Integer)] -> (Integer, Integer)
score k weights pens = (perMille, totalViol)
 where
  weighted = [(effWeight k weights code, p) | (code, p) <- pens]
  wTotal = sum (map fst weighted)
  raw = sum [w * p * sViolCost k | (w, p) <- weighted]
  perMille = max 0 (sScoreScale k - raw `div` (Cost.violCostNeutral * wTotal))
  totalViol = sum (map snd pens)
