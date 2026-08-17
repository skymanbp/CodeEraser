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

-- | Violation count per axis — 0 size / 1 complexity / 2 clone /
-- 3 docdup / 4 deadcode / 5 churn / 6 graph_cycle (decision ⑦; the
-- separate axisCodes constant was a dead export, M5-close review
-- LOW). Every axis is one NAMED predicate function owning exactly
-- one knob, so the per-axis perturbation battery has a lever per row
-- (the M5-close warn repayment: seven guarded comprehensions in one
-- body read as CC 17; the table now carries names, the predicates
-- carry the decisions).
penalties :: ScoreKnobs -> Facts -> [(Integer, Integer)]
penalties k f =
  [ (0, sizeOver k f)
  , (1, cocOver k f)
  , (2, cloneHits k f)
  , (3, dupHits k f)
  , (4, deadFiles k f)
  , (5, churnHeavy k f)
  , (6, cycleMembers k f)
  ]

count :: [()] -> Integer
count = toInteger . length

sizeOver :: ScoreKnobs -> Facts -> Integer
sizeOver k f = count [() | [_, 0, v] <- fCont f, v > sSizeCeil k]

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
