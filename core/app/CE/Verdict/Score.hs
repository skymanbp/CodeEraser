-- | The verdict score's knobs and the two family predicates its law
-- reads (plan v2.35 score amendment, protocol 10.0.0): the extent law
-- itself is CE.Verdict.Extent, its facts CE.Verdict.ExtentFacts. Knobs
-- travel as parameters; production binds them at scoreBound — the
-- batteries perturb fields instead (the 2g mechanism). The cycle floor
-- is the graph family's sccFloor; the size and complexity ceilings are
-- the plan §4.1 dogfood numbers. The density law that lived here
-- (per-axis charges v/(v+n) folded by weight) retired with 10.0.0, and
-- with it the zone curve's place in the score, P_max and the dead
-- in-degree ceiling.
module CE.Verdict.Score
  ( Facts (..)
  , ClassKnobs
  , ScoreKnobs (..)
  , churnRows
  , cycleRows
  , classKnobsOf
  , classOf
  , scoreBound
  , effectiveWeights
  ) where

import CE.Graph.Cost (sccFloor)
import qualified CE.Verdict.Cost as Cost
import CE.Verdict.Facts
import qualified Data.IntSet as IS

data ScoreKnobs = ScoreKnobs
  { sSizeCeil :: Integer
  , sSizeHard :: Integer
  , sSoftK :: Integer
  , sCocCeil :: Integer
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
    , sSoftK = Cost.softLineK
    , sCocCeil = Cost.cocCeil
    , sRewriteNum = Cost.rewriteNum
    , sRewriteDen = Cost.rewriteDen
    , sCycleFloor = sccFloor
    , sViolCost = Cost.violCost
    , sDefaultWeight = Cost.defaultWeight
    , sScoreScale = Cost.scoreScale
    }

-- | The rewrite-heavy churn rows: rewritten / (rewritten + appended)
-- at or above the rewrite threshold.
churnRows :: ScoreKnobs -> Facts -> [[Integer]]
churnRows k f = [row | row@[_, rw, ap] <- fChurn f, rw + ap > 0, rw * sRewriteDen k >= (rw + ap) * sRewriteNum k]

-- | A pos row counts as a cycle member at or above the floor — and a
-- SINGLETON only through its own arc (6.4.0, O59): every isolated
-- file is a one-node SCC too, so at floor 1 the self-loop table is
-- what separates a cycle from a lonely file. A documentation file is
-- never a member.
cycleRows :: ScoreKnobs -> Facts -> [[Integer]]
cycleRows k f =
  [ row
  | row@[u, _, _, _, size, _] <- fPos f
  , size >= sCycleFloor k
  , size > 1 || IS.member (fromInteger u) loops
  , IS.notMember (fromInteger u) docs
  ]
 where
  loops = IS.fromList (map fromInteger (fSelfLoops f))
  docs = IS.fromList (map fromInteger (fDocFiles f))

-- | One axis's effective weight: wire rows [axisCode, numerator]
-- override; unlisted axes weigh sDefaultWeight. ONE lookup, two
-- readers — the score fold and the reply's echo (review C3).
effWeight :: ScoreKnobs -> [[Integer]] -> Integer -> Integer
effWeight k weights code = case [w | [c, w] <- weights, c == code] of
  (w : _) -> w
  [] -> sDefaultWeight k

-- | The full effective table 0..6, the extent law's layers and the
-- reply's echo.
effectiveWeights :: ScoreKnobs -> [[Integer]] -> [[Integer]]
effectiveWeights k weights = [[c, effWeight k weights c] | c <- [0 .. 6]]
