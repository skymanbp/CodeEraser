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
  , scoreBound
  , axisCodes
  , penalties
  , score
  ) where

import CE.Clone.Cost (tsedDen, tsedNum)
import CE.Docdup.Cost (jaccardDen, jaccardNum)
import CE.Graph.Cost (sccFloor)
import qualified CE.Verdict.Cost as Cost

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
  }

scoreBound :: ScoreKnobs
scoreBound =
  ScoreKnobs
    { sSizeCeil = Cost.sizeCeil
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
    }

-- | The seven axes: 0 size / 1 complexity / 2 clone / 3 docdup /
-- 4 deadcode / 5 churn / 6 graph_cycle (decision ⑦).
axisCodes :: [Integer]
axisCodes = [0 .. 6]

-- | Violation count per axis, one predicate each — every predicate
-- owns exactly one knob so the per-axis perturbation battery has a
-- lever per row.
penalties :: ScoreKnobs -> Facts -> [(Integer, Integer)]
penalties k f =
  [ (0, count [() | [_, 0, v] <- fCont f, v > sSizeCeil k])
  , (1, count [() | [_, 1, v] <- fCont f, v > sCocCeil k])
  , (2, count [() | [_, _, kind, n, d] <- fSim f, kind <= 1, n * sCloneDen k >= d * sCloneNum k])
  , (3, count [() | [_, _, 2, n, d] <- fSim f, n * sDupDen k >= d * sDupNum k])
  , (4, count [() | [_, indeg, _, _, _, 0] <- fPos f, indeg <= sDeadIndegCeil k])
  , (5, count [() | [_, rw, ap, _, _] <- fChurn f, rw + ap > 0, rw * sRewriteDen k >= (rw + ap) * sRewriteNum k])
  , (6, count [() | [_, _, _, _, size, _] <- fPos f, size >= sCycleFloor k])
  ]
 where
  count = toInteger . length

-- | (perMille, total violation count) under the effective weights:
-- wire rows [axisCode, numerator] override; unlisted axes weigh
-- sDefaultWeight. wTotal = sum of effective weights (derived);
-- validation refuses an all-zero weight table, so the divisor is
-- never 0.
score :: ScoreKnobs -> [[Integer]] -> [(Integer, Integer)] -> (Integer, Integer)
score k weights pens = (perMille, totalViol)
 where
  effWeight code = case [w | [c, w] <- weights, c == code] of
    (w : _) -> w
    [] -> sDefaultWeight k
  weighted = [(effWeight code, p) | (code, p) <- pens]
  wTotal = sum (map fst weighted)
  raw = sum [w * p * sViolCost k | (w, p) <- weighted]
  perMille = max 0 (1000 - raw `div` wTotal)
  totalViol = sum (map snd pens)
