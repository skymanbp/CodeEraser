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
  , ClassKnobs
  , ScoreKnobs (..)
  , chargeAt
  , classKnobsOf
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
import qualified Data.Map.Strict as M
import Data.Ratio ((%))
import qualified Data.IntSet as IS

-- | The validated fact tables (row shapes enforced by CE.Verdict's
-- boundary contract before anything reaches here). Continuous rows
-- are read by PREFIX throughout (3.1.0): a legacy three-column row
-- and a classed four-column row take the same patterns.
data Facts = Facts
  { fSim :: [[Integer]]
  -- ^ [u,v,simKind,num,den]
  , fPos :: [[Integer]]
  -- ^ [u,indeg,outdeg,sccId,sccSize,reachIn]
  , fChurn :: [[Integer]]
  -- ^ [u,rewrite,append] (3.0.0: the constant added / unmeasured survived left)
  , fCont :: [[Integer]]
  -- ^ [u,metricCode,value] or, classed (3.1.0), [u,metricCode,value,classId]
  , fDocFiles :: [Integer]
  -- ^ file-universe indices whose language is documentation
  , fClassKnobs :: ClassKnobs
  -- ^ (classId, code) -> value: the ceilings codes 0/1/2 under a class (3.1.0)
  , fSelfLoops :: [Integer]
  -- ^ file-universe indices carrying a self-arc (6.4.0): a singleton SCC is a cycle only through one
  }

-- | The class overrides as ONE Map, built once per judgment from the
-- wire rows (plan v2.13 ①): the wire's row order is a validation
-- fact and never a judgment fact (ClassProps pins the permutation).
-- Lookup falls back to the global reading, so an unclassed row — or
-- a class with no row for that code — judges exactly as before.
type ClassKnobs = M.Map (Integer, Integer) Integer

classKnobsOf :: [[Integer]] -> ClassKnobs
classKnobsOf rows = M.fromList [((c, code), v) | [c, code, v] <- rows]

-- | A continuous row's class: the 4th column when it rides, 0 (the
-- global table) on a legacy three-column row.
classOf :: [Integer] -> Integer
classOf row = case drop 3 row of
  (c : _) -> c
  [] -> 0

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
      , (6, cnt (cycleMembers k f), nodes - toInteger (length (fDocFiles f)))
      ]
  ]
 where
  cnt = fromInteger :: Integer -> Rational
  -- opportunity counts by metric-code prefix: a classed row is one
  -- file / one function exactly like its legacy shape
  files = count [() | (_ : 0 : _) <- fCont f]
  functions = count [() | (_ : 1 : _) <- fCont f]
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
-- A classed row (3.1.0) takes its class's own opening edge (code 0)
-- and hard line (code 2) where declared — the charge law itself is
-- untouched; only the two lines the row is measured against move.
sizeMass :: ScoreKnobs -> Maybe Integer -> Facts -> Rational
sizeMass k soft f =
  sum
    [ zonePenalty (line 0 s) (line 2 (sSizeHard k)) (sSizePMax k) v
    | row@(_ : 0 : v : _) <- fCont f
    , let line code g = M.findWithDefault g (classOf row, code) (fClassKnobs f)
    ]
 where
  s = maybe (sSizeCeil k) id soft

cocOver :: ScoreKnobs -> Facts -> Integer
cocOver k f =
  count
    [ ()
    | row@(_ : 1 : v : _) <- fCont f
    , v > M.findWithDefault (sCocCeil k) (classOf row, 1) (fClassKnobs f)
    ]

cloneHits :: ScoreKnobs -> Facts -> Integer
cloneHits k f =
  count [() | [_, _, kind, n, d] <- fSim f, kind <= 1, n * sCloneDen k >= d * sCloneNum k]

dupHits :: ScoreKnobs -> Facts -> Integer
dupHits k f = count [() | [_, _, 2, n, d] <- fSim f, n * sDupDen k >= d * sDupNum k]

deadFiles :: ScoreKnobs -> Facts -> Integer
deadFiles k f = count [() | [_, indeg, _, _, _, 0] <- fPos f, indeg <= sDeadIndegCeil k]

churnHeavy :: ScoreKnobs -> Facts -> Integer
churnHeavy k f =
  count [() | [_, rw, ap] <- fChurn f, rw + ap > 0, rw * sRewriteDen k >= (rw + ap) * sRewriteNum k]

-- | A pos row counts as a cycle member at or above the floor — and a
-- SINGLETON only through its own arc (6.4.0, O59): every isolated
-- file is a one-node SCC too, so at floor 1 the self-loop table is
-- what separates a cycle from a lonely file. At the shipped floor
-- (2) the size test alone decides, exactly as before.
cycleMembers :: ScoreKnobs -> Facts -> Integer
cycleMembers k f =
  count
    [ ()
    | [u, _, _, _, size, _] <- fPos f
    , size >= sCycleFloor k
    , size > 1 || IS.member (fromInteger u) loops
    , IS.notMember (fromInteger u) docs
    ]
 where
  docs = IS.fromList (map fromInteger (fDocFiles f))
  loops = IS.fromList (map fromInteger (fSelfLoops f))

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
